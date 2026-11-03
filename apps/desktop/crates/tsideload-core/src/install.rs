//! Sign an IPA and install it on a device — either with the user's Apple ID
//! (free provisioning) or with an imported .p12 + .mobileprovision.

use std::path::Path;

use isideload::sideload::{SideloaderBuilder, TeamSelection};
use isideload::util::callbacks::{ExtensionsBehaviorCallbackBox, MaxCertsCallbackBox};
use isideload::util::fs_storage::FsStorage;

use crate::auth::AppleSession;
use crate::devices::find_provider;
use crate::{Error, Result};

/// Install options.
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Target device UDID. `None` = first connected device.
    pub udid: Option<String>,
    /// Pass through `isideload`'s increased-memory-limit entitlement hack.
    /// Only needed for specific apps (e.g. JIT-heavy ones); default false.
    pub increased_memory_limit: bool,
}

/// Sign `ipa` with the Apple ID behind `session` and install it.
///
/// After install, the device needs one manual step:
/// Settings > General > VPN & Device Management > trust the Apple ID.
pub async fn install_ipa(
    session: AppleSession,
    ipa: &Path,
    opts: &InstallOptions,
) -> Result<()> {
    if !ipa.is_file() {
        return Err(Error::Io(format!("ipa not found: {}", ipa.display())));
    }

    let provider = find_provider(opts.udid.as_deref()).await?;

    let mut sideloader =
        SideloaderBuilder::<MaxCertsCallbackBox, ExtensionsBehaviorCallbackBox>::new(
            session.session,
            session.apple_id,
        )
        .team_selection(TeamSelection::First)
        .storage(Box::new(FsStorage::new(crate::storage_dir())))
        .machine_name("tsideload-desktop".to_string())
        .build();

    // Progress callback left empty for now; the CLI/GUI print their own
    // coarse status lines. TODO(P1): thread real progress through.
    let _installed = sideloader
        .install_app(
            &provider,
            ipa.to_path_buf(),
            opts.increased_memory_limit,
            None::<fn(f32) -> std::future::Ready<()>>,
            None,
        )
        .await
        .map_err(|e| Error::Sideload(format!("{e:?}")))?;

    Ok(())
}

/// Sign `ipa` with an imported certificate (`--cert <name>`) and install it.
///
/// No Apple ID involved: the .p12 password comes from `p12_password`
/// (CLI: `TSIDELOAD_P12_PASSWORD` env or interactive prompt). The device and
/// bundle-id checks happen inside [`crate::certs::sign_ipa`].
///
/// After install, the device needs one manual step:
/// Settings > General > VPN & Device Management > trust the certificate.
pub async fn install_ipa_with_cert(
    ipa: &Path,
    cert_name: &str,
    p12_password: &str,
    opts: &InstallOptions,
) -> Result<()> {
    if !ipa.is_file() {
        return Err(Error::Io(format!("ipa not found: {}", ipa.display())));
    }
    // Resolve the target device first so the provision can be validated
    // against its UDID before we spend time signing.
    let devices = crate::devices::list_devices().await?;
    let udid = match opts.udid.as_deref() {
        Some(want) => devices
            .into_iter()
            .find(|d| d.udid == want)
            .map(|d| d.udid)
            .ok_or_else(|| Error::DeviceNotFound(want.to_string()))?,
        None => devices
            .into_iter()
            .next()
            .map(|d| d.udid)
            .ok_or(Error::NoDevice)?,
    };

    let loaded = crate::certs::load_cert(cert_name, p12_password)?;
    let signed_app_dir = crate::certs::sign_ipa(ipa, &loaded, &udid).await?;

    let provider = find_provider(Some(&udid)).await?;
    isideload::sideload::install::install_app(&provider, &signed_app_dir, |_| {})
        .await
        .map_err(|e| Error::Sideload(format!("install failed: {e:?}")))?;

    Ok(())
}
