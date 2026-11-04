//! TSideload desktop GUI.
//!
//! `list_devices` lists USB devices. `list_certs` lists imported .p12
//! certificates (metadata only). `install_ipa` runs the full install flow:
//! either Apple ID login (2FA is mediated by the `tsideload://twofa-prompt`
//! event + the `twofa_answer` command) or, when `req.cert_name` is set,
//! KSign-style signing with an imported certificate (no Apple ID, no 2FA).
//! Then sign + install via `tsideload-core`. Coarse progress goes out as
//! `tsideload://install-stage` events (`login` / `signing` / `done` /
//! `done_cert` / `error`).
//!
//! Build note: this crate is excluded from the workspace and needs
//! WebView2 (Windows) / WebKit (macOS). It has never been compiled on the
//! Linux dev VM (no webkit2gtk) — P1b was written by careful reading only.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tsideload_core::{DeviceInfo, TwoFactorAnswer, TwoFactorPrompt};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
struct DeviceDto {
    udid: String,
    connection_type: String,
}

impl From<DeviceInfo> for DeviceDto {
    fn from(d: DeviceInfo) -> Self {
        DeviceDto {
            udid: d.udid,
            connection_type: d.connection_type,
        }
    }
}

#[derive(Debug, Deserialize)]
struct InstallRequest {
    apple_id: String,
    password: String,
    ipa_path: String,
    udid: Option<String>,
    increased_memory_limit: bool,
    /// KSign-style flow: when set (non-empty), Apple ID login is skipped and
    /// the IPA is signed with this imported certificate instead. The .p12
    /// password is only used for this call and never stored.
    cert_name: Option<String>,
    p12_password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TrustedNumberDto {
    id: u32,
    number: String,
}

/// What the frontend 2FA dialog receives. Mirrors
/// `tsideload_core::TwoFactorPrompt` (minus `selected_number_id`, which the
/// UI does not need).
#[derive(Debug, Clone, Serialize)]
struct TwoFactorPromptDto {
    sms: bool,
    unknown: bool,
    numbers: Vec<TrustedNumberDto>,
    last_error: Option<String>,
}

impl From<&TwoFactorPrompt> for TwoFactorPromptDto {
    fn from(p: &TwoFactorPrompt) -> Self {
        TwoFactorPromptDto {
            sms: p.sms,
            unknown: p.unknown,
            numbers: p
                .numbers
                .iter()
                .map(|n| TrustedNumberDto {
                    id: n.id,
                    number: n.number.clone(),
                })
                .collect(),
            last_error: p.last_error.clone(),
        }
    }
}

/// Answer from the frontend 2FA dialog. Same contract as the `tsd` CLI:
/// when the prompt had `unknown` set, the frontend never offers code entry —
/// only method selection / abort reach the backend in that state.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TwoFactorAnswerDto {
    Code { code: String },
    Resend,
    SendToDevices,
    SendSms { number_id: u32 },
    Abort,
}

impl From<TwoFactorAnswerDto> for TwoFactorAnswer {
    fn from(a: TwoFactorAnswerDto) -> Self {
        match a {
            TwoFactorAnswerDto::Code { code } => TwoFactorAnswer::Code(code),
            TwoFactorAnswerDto::Resend => TwoFactorAnswer::Resend,
            TwoFactorAnswerDto::SendToDevices => TwoFactorAnswer::SendToDevices,
            TwoFactorAnswerDto::SendSms { number_id } => TwoFactorAnswer::SendSms(number_id),
            TwoFactorAnswerDto::Abort => TwoFactorAnswer::Abort,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct InstallStage {
    stage: String,
    detail: Option<String>,
}

// ---------------------------------------------------------------------------
// 2FA bridge. The core's `login` needs a *blocking* prompt callback, but the
// answer comes from the webview: the callback stores a oneshot sender, emits
// an event, and blocks on `recv()` (std mpsc — no Tokio involvement, safe
// inside `block_in_place`). `twofa_answer` delivers the user's answer.
// Single-flight: the Install button is disabled while an install runs.
// ---------------------------------------------------------------------------

struct TwofaBridge {
    app: AppHandle,
    slot: Mutex<Option<mpsc::Sender<TwoFactorAnswer>>>,
}

/// Bridge of the currently running install, if any.
struct PendingTwofa(Mutex<Option<Arc<TwofaBridge>>>);

fn clear_pending(pending: &PendingTwofa) {
    if let Ok(mut guard) = pending.0.lock() {
        *guard = None;
    }
}

fn emit_stage(app: &AppHandle, stage: &str, detail: Option<String>) {
    let _ = app.emit(
        "tsideload://install-stage",
        InstallStage {
            stage: stage.to_string(),
            detail,
        },
    );
}

/// Best-effort pairing-file push after a successful install: warn (stderr)
/// and continue — the install itself already succeeded. The iOS app
/// self-imports the pushed file on first launch; if the push failed the
/// user can still import the file manually.
async fn push_pairing_best_effort(udid: Option<&str>) {
    if let Err(e) = tsideload_core::pairing::push_pairing_for_udid(udid).await {
        eprintln!(
            "warning: could not push pairing file ({e}) — import it manually in the app if needed."
        );
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn list_devices() -> Result<Vec<DeviceDto>, String> {
    tsideload_core::init();
    let devices = tsideload_core::devices::list_devices()
        .await
        .map_err(|e| e.to_string())?;
    Ok(devices.into_iter().map(DeviceDto::from).collect())
}

/// Imported certificates (metadata only, no secrets). Used to populate the
/// "Imported certificate" signing method in the GUI.
#[tauri::command]
async fn list_certs() -> Result<Vec<tsideload_core::certs::ImportedCertMeta>, String> {
    tsideload_core::init();
    tsideload_core::certs::list_certs().map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_ipa(
    app: AppHandle,
    pending: State<'_, PendingTwofa>,
    req: InstallRequest,
) -> Result<String, String> {
    tsideload_core::init();

    let ipa_path = PathBuf::from(&req.ipa_path);
    let install_opts = tsideload_core::install::InstallOptions {
        udid: req.udid.clone(),
        increased_memory_limit: req.increased_memory_limit,
    };

    // KSign-style flow: imported certificate, no Apple ID, no 2FA.
    if let Some(cert_name) = req.cert_name.clone().filter(|s| !s.trim().is_empty()) {
        let udid = req.udid.clone();
        let p12_password = req.p12_password.clone().unwrap_or_default();
        drop(req); // clear any credential strings promptly
        emit_stage(&app, "signing", Some(cert_name.clone()));
        let result = tsideload_core::install::install_ipa_with_cert(
            &ipa_path,
            &cert_name,
            &p12_password,
            &install_opts,
        )
        .await;
        // No 2FA bridge was armed on this path; nothing to clear.
        return match result {
            Ok(()) => {
                push_pairing_best_effort(udid.as_deref()).await;
                emit_stage(&app, "done_cert", None);
                Ok("installed".to_string())
            }
            Err(e) => {
                let msg = e.to_string();
                emit_stage(&app, "error", Some(msg.clone()));
                Err(msg)
            }
        };
    }

    // The password lives only in this call: moved into `login` below, never
    // logged, never emitted. (The frontend clears its field on submit.)
    let bridge = Arc::new(TwofaBridge {
        app: app.clone(),
        slot: Mutex::new(None),
    });
    pending
        .0
        .lock()
        .map_err(|e| format!("internal state error: {e}"))?
        .replace(bridge.clone());

    emit_stage(&app, "login", Some(req.apple_id.clone()));

    let ask = {
        let bridge = Arc::clone(&bridge);
        Arc::new(move |prompt: TwoFactorPrompt| -> TwoFactorAnswer {
            let (tx, rx) = mpsc::channel();
            match bridge.slot.lock() {
                Ok(mut slot) => *slot = Some(tx),
                Err(_) => return TwoFactorAnswer::Abort,
            }
            let _ = bridge
                .app
                .emit("tsideload://twofa-prompt", TwoFactorPromptDto::from(&prompt));
            // `login` runs this via `block_in_place`, so blocking the thread
            // here is safe. A dropped sender (superseded install) aborts.
            rx.recv().unwrap_or(TwoFactorAnswer::Abort)
        })
    };

    let session = match tsideload_core::auth::login(&req.apple_id, &req.password, ask).await {
        Ok(s) => s,
        Err(e) => {
            clear_pending(&pending);
            let msg = e.to_string();
            emit_stage(&app, "error", Some(msg.clone()));
            return Err(msg);
        }
    };
    // `req` — and the password with it — is dropped here.
    drop(req);

    emit_stage(&app, "signing", None);
    // NOTE: fine-grained (byte-level) progress needs core support —
    // `install::install_ipa` currently takes no progress callback — so the
    // GUI reports coarse stages instead of a real progress bar.
    let result = tsideload_core::install::install_ipa(session, &ipa_path, &install_opts).await;
    clear_pending(&pending);
    match result {
        Ok(()) => {
            push_pairing_best_effort(install_opts.udid.as_deref()).await;
            emit_stage(&app, "done", None);
            Ok("installed".to_string())
        }
        Err(e) => {
            let msg = e.to_string();
            emit_stage(&app, "error", Some(msg.clone()));
            Err(msg)
        }
    }
}

#[tauri::command]
async fn twofa_answer(
    pending: State<'_, PendingTwofa>,
    answer: TwoFactorAnswerDto,
) -> Result<(), String> {
    let bridge = pending
        .0
        .lock()
        .map_err(|e| format!("internal state error: {e}"))?
        .clone()
        .ok_or_else(|| "no install in progress".to_string())?;
    let tx = bridge
        .slot
        .lock()
        .map_err(|e| format!("internal state error: {e}"))?
        .take()
        .ok_or_else(|| "no pending 2FA prompt".to_string())?;
    tx.send(TwoFactorAnswer::from(answer))
        .map_err(|_| "2FA prompt already answered".to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(PendingTwofa(Mutex::new(None)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_devices,
            list_certs,
            install_ipa,
            twofa_answer
        ])
        .run(tauri::generate_context!())
        .expect("failed to run TSideload GUI");
}
