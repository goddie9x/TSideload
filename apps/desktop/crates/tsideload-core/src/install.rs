//! Sign an IPA with the user's Apple ID and install it on a device.

use std::path::{Path, PathBuf};

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

/// Where anisette state / certificates are cached. File-based on purpose:
/// no OS keyring dependency, works the same on Windows/macOS/Linux.
///
/// TODO(P1): use the platform config dir (via the `dirs` crate) instead of
/// the current working directory.
fn storage_dir() -> PathBuf {
    PathBuf::from("tsideload-data")
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
        .storage(Box::new(FsStorage::new(storage_dir())))
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
