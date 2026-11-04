//! `tsd` — TSideload desktop CLI.
//!
//! ```sh
//! tsd devices
//! tsd install app.ipa --apple-id you@example.com [--udid <UDID>]
//! tsd install app.ipa --cert mycert [--udid <UDID>]
//! tsd import-cert --p12 cert.p12 --provision app.mobileprovision [--name mycert]
//! tsd export-pairing [--udid <UDID>] [--output pairing.mobiledevicepairing]
//! ```

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use tsideload_core::{TwoFactorAnswer, TwoFactorPrompt};

#[derive(Parser)]
#[command(name = "tsd", version, about = "TSideload desktop CLI — free iOS sideloading")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List iOS/iPadOS devices connected over USB
    Devices,
    /// Sign an IPA and install it (Apple ID, or --cert for an imported certificate)
    Install {
        /// Path to the .ipa file
        ipa: PathBuf,
        /// Apple ID (email); not needed with --cert
        #[arg(long)]
        apple_id: Option<String>,
        /// Target device UDID (default: first connected device)
        #[arg(long)]
        udid: Option<String>,
        /// Use an imported certificate (see `tsd import-cert`) instead of Apple ID signing
        #[arg(long)]
        cert: Option<String>,
    },
    /// Import a .p12 certificate + .mobileprovision for later use with `install --cert`
    ImportCert {
        /// Path to the .p12 file (private key + certificate)
        #[arg(long)]
        p12: PathBuf,
        /// Path to the .mobileprovision file covering the target app id
        #[arg(long)]
        provision: PathBuf,
        /// Name to store it under (default: .p12 file stem)
        #[arg(long)]
        name: Option<String>,
    },
    /// Export the device's pairing record as a .mobiledevicepairing plist
    /// (for manual transfer to the iPhone, or debugging)
    ExportPairing {
        /// Target device UDID (default: first connected device)
        #[arg(long)]
        udid: Option<String>,
        /// Where to write the file (default: ./TSideloadPairing.mobiledevicepairing)
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() {
    tsideload_core::init();
    let cli = Cli::parse();
    if let Err(e) = run(cli.cmd).await {
        eprintln!("tsd: error: {e}");
        std::process::exit(1);
    }
}

async fn run(cmd: Cmd) -> tsideload_core::Result<()> {
    match cmd {
        Cmd::Devices => {
            let devices = tsideload_core::devices::list_devices().await?;
            if devices.is_empty() {
                println!("No devices found. Connect your iPhone via USB and tap \"Trust\".");
            } else {
                for d in devices {
                    println!("{}  ({})", d.udid, d.connection_type);
                }
            }
            Ok(())
        }
        Cmd::Install {
            ipa,
            apple_id,
            udid,
            cert,
        } => {
            if let Some(cert_name) = cert {
                if apple_id.is_some() {
                    eprintln!("tsd: error: --cert cannot be combined with --apple-id");
                    std::process::exit(2);
                }
                let password = std::env::var("TSIDELOAD_P12_PASSWORD").unwrap_or_else(|_| {
                    rpassword::prompt_password(format!(
                        "Password for imported cert '{cert_name}' (or set TSIDELOAD_P12_PASSWORD): "
                    ))
                    .unwrap_or_default()
                });
                println!("Signing {} with imported cert '{cert_name}' ...", ipa.display());
                let opts = tsideload_core::install::InstallOptions {
                    udid,
                    increased_memory_limit: false,
                };
                tsideload_core::install::install_ipa_with_cert(&ipa, &cert_name, &password, &opts)
                    .await?;
                push_pairing_best_effort(opts.udid.as_deref()).await;
                println!("Done. On the device: Settings > General > VPN & Device Management > trust the certificate.");
                return Ok(());
            }
            let apple_id = apple_id.unwrap_or_else(|| read_line("Apple ID (email): "));
            let password = std::env::var("TSIDELOAD_APPLE_PASSWORD").unwrap_or_else(|_| {
                rpassword::prompt_password("Apple ID password (or set TSIDELOAD_APPLE_PASSWORD): ")
                    .unwrap_or_default()
            });
            println!("Logging in as {apple_id} ...");
            let ask = Arc::new(|prompt: TwoFactorPrompt| -> TwoFactorAnswer {
                ask_two_factor(&prompt)
            });
            let session = tsideload_core::auth::login(&apple_id, &password, ask).await?;
            println!("Login OK. Signing and installing {} ...", ipa.display());
            let opts = tsideload_core::install::InstallOptions {
                udid,
                increased_memory_limit: false,
            };
            tsideload_core::install::install_ipa(session, &ipa, &opts).await?;
            push_pairing_best_effort(opts.udid.as_deref()).await;
            println!("Done. On the device: Settings > General > VPN & Device Management > trust your Apple ID.");
            Ok(())
        }
        Cmd::ImportCert {
            p12,
            provision,
            name,
        } => {
            let password = std::env::var("TSIDELOAD_P12_PASSWORD").unwrap_or_else(|_| {
                rpassword::prompt_password(
                    ".p12 password (empty for none, or set TSIDELOAD_P12_PASSWORD): ",
                )
                .unwrap_or_default()
            });
            match tsideload_core::certs::import_cert(&p12, &password, &provision, name.as_deref()) {
                Ok(meta) => {
                    println!("Imported certificate '{}':", meta.name);
                    println!("  team:      {}", meta.team_id);
                    println!("  app id:    {}", meta.application_identifier);
                    println!("  profile:   {} ({})", meta.profile_name, meta.profile_uuid);
                    println!("  expires:   {}", meta.expires_rfc3339);
                    match meta.device_count {
                        Some(n) => println!("  devices:   {n} provisioned"),
                        None => println!("  devices:   unrestricted"),
                    }
                    println!("Use it with: tsd install app.ipa --cert {}", meta.name);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(2);
                }
            }
        }
        Cmd::ExportPairing { udid, output } => {
            let out = output
                .unwrap_or_else(|| PathBuf::from(tsideload_core::pairing::PAIRING_FILENAME));
            let plist = tsideload_core::pairing::export_pairing_plist(udid.as_deref()).await?;
            std::fs::write(&out, &plist)
                .map_err(|e| tsideload_core::Error::Io(format!("cannot write {}: {e}", out.display())))?;
            println!("Wrote pairing file to {}", out.display());
            println!("Transfer it to the iPhone (Files app) and pick it in the TSideload app,");
            println!("or let `tsd install` push it automatically next time.");
            Ok(())
        }
    }
}

/// Best-effort pairing-file push after a successful install: warn and
/// continue on failure — the install itself already succeeded.
async fn push_pairing_best_effort(udid: Option<&str>) {
    match tsideload_core::pairing::push_pairing_for_udid(udid).await {
        Ok(()) => println!("Pairing file pushed to the TSideload app on the device."),
        Err(e) => eprintln!(
            "warning: could not push pairing file ({e}) — import it manually in the app if needed."
        ),
    }
}

fn read_line(label: &str) -> String {
    print!("{label}");
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    s.trim().to_string()
}

/// Interactive 2FA: type the code, or `resend` / `devices` / `sms:<id>` / `abort`.
/// When Apple doesn't say which method works (`unknown`), code entry is
/// refused upstream — only method selection or abort is offered.
fn ask_two_factor(prompt: &TwoFactorPrompt) -> TwoFactorAnswer {
    if let Some(err) = &prompt.last_error {
        println!("2FA note: {err}");
    }
    if prompt.unknown {
        println!("Apple didn't specify a working 2FA method. Choose one:");
        println!("  devices          - send the code to your trusted devices");
        if prompt.sms {
            for n in &prompt.numbers {
                println!("  sms:<id>         - e.g. sms:{} ({})", n.id, n.number);
            }
        }
        println!("  abort            - give up on this login");
    } else if prompt.sms {
        println!("Apple can text the code. Trusted numbers:");
        for n in &prompt.numbers {
            println!("  [{}] {}", n.id, n.number);
        }
        println!("Type the 6-digit code, or: resend | devices | sms:<id> | abort");
    } else {
        println!("Enter the 2FA code sent to your trusted devices (or: resend | abort)");
    }
    let input = read_line("2FA> ");
    match input.to_ascii_lowercase().as_str() {
        "resend" if !prompt.unknown => TwoFactorAnswer::Resend,
        "devices" => TwoFactorAnswer::SendToDevices,
        "abort" => TwoFactorAnswer::Abort,
        _ if input.to_ascii_lowercase().starts_with("sms:") => input["sms:".len()..]
            .trim()
            .parse::<u32>()
            .map(TwoFactorAnswer::SendSms)
            .unwrap_or(TwoFactorAnswer::Abort),
        _ if prompt.unknown => {
            println!("Code entry is not allowed in this state; aborting.");
            TwoFactorAnswer::Abort
        }
        _ => TwoFactorAnswer::Code(input),
    }
}
