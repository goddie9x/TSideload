//! Pairing-file export and push.
//!
//! The on-device TSideload iOS app needs the lockdown pairing record to
//! talk to the phone's own services over its local VPN loopback (that's
//! how it installs/refreshes apps without a PC). Instead of making the
//! user dig the file out of the OS lockdown directory by hand, the
//! desktop tool fetches the record straight from usbmuxd and pushes it
//! into the app's Documents folder over AFC — the app self-imports it
//! on first launch.
//!
//! Everything here is best-effort: pushing must never fail an install.

use std::time::Duration;

use idevice::provider::{IdeviceProvider, UsbmuxdProvider};
use idevice::services::afc::opcode::AfcFopenMode;
use idevice::services::house_arrest::HouseArrestClient;
use idevice::IdeviceService;

use crate::{Error, Result};

/// Bundle ID of the on-device TSideload iOS app.
pub const TSIDELOAD_BUNDLE_ID: &str = "dev.hmt9x.tsideload";
/// Pairing filename the iOS app auto-imports from its Documents.
/// MUST stay in sync with the iOS side.
pub const PAIRING_FILENAME: &str = "TSideloadPairing.mobiledevicepairing";

/// Fetch the device's lockdown pairing record from usbmuxd and serialize
/// it to the XML plist format the iOS app's `LockdownPairingFile` parser
/// expects (`DeviceCertificate`/`HostPrivateKey`/`HostCertificate`/
/// `RootPrivateKey`/`RootCertificate`/`SystemBUID`/`HostID`/`EscrowBag`/
/// `WiFiMACAddress`/`UDID`).
pub async fn get_pairing_plist(provider: &UsbmuxdProvider) -> Result<Vec<u8>> {
    let pairing = provider
        .get_pairing_file()
        .await
        .map_err(|e| Error::Sideload(format!("cannot read pairing record: {e:?}")))?;
    pairing
        .serialize()
        .map_err(|e| Error::Sideload(format!("cannot serialize pairing record: {e:?}")))
}

/// Push the pairing plist into the TSideload iOS app's Documents folder
/// over AFC (house_arrest `VendDocuments`).
///
/// Fails when the app isn't installed or AFC is unavailable — callers must
/// treat this as best-effort and only warn, never abort the install.
pub async fn push_pairing_file(provider: &UsbmuxdProvider) -> Result<()> {
    let plist = get_pairing_plist(provider).await?;

    let client = HouseArrestClient::connect(provider as &dyn IdeviceProvider)
        .await
        .map_err(|e| Error::Sideload(format!("house_arrest connect failed: {e:?}")))?;
    let mut afc = client
        .vend_documents(TSIDELOAD_BUNDLE_ID)
        .await
        .map_err(|e| Error::Sideload(format!("cannot vend app Documents: {e:?}")))?;

    // `Wr` (w+) = O_RDWR|O_CREAT|O_TRUNC: creates or overwrites the file.
    // `vend_documents` roots us at Documents, so the bare filename lands
    // in Documents/TSideloadPairing.mobiledevicepairing.
    let mut fd = afc
        .open(PAIRING_FILENAME, AfcFopenMode::Wr)
        .await
        .map_err(|e| Error::Sideload(format!("cannot create pairing file on device: {e:?}")))?;
    fd.write_entire(&plist)
        .await
        .map_err(|e| Error::Sideload(format!("cannot write pairing file: {e:?}")))?;
    fd.close()
        .await
        .map_err(|e| Error::Sideload(format!("cannot close pairing file: {e:?}")))?;
    Ok(())
}

/// Convenience for CLI/GUI callers, which don't hold a provider:
/// resolve the device, then push. See [`push_pairing_file`].
///
/// Retries the whole push (3 attempts, ~2s apart): the app container may
/// not be ready immediately after install, so a first attempt can fail
/// even though a later one succeeds. Returns the last error when every
/// attempt fails, so the caller can show the actual failure reason.
pub async fn push_pairing_for_udid(udid: Option<&str>) -> Result<()> {
    const PUSH_ATTEMPTS: u32 = 3;
    const PUSH_RETRY_DELAY: Duration = Duration::from_secs(2);

    let mut last_err = None;
    for attempt in 1..=PUSH_ATTEMPTS {
        let outcome = async {
            let provider = crate::devices::find_provider(udid).await?;
            push_pairing_file(&provider).await
        }
        .await;
        match outcome {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = Some(e);
                if attempt < PUSH_ATTEMPTS {
                    tokio::time::sleep(PUSH_RETRY_DELAY).await;
                }
            }
        }
    }
    Err(last_err.expect("at least one push attempt ran"))
}

/// Convenience for the `export-pairing` command: fetch + serialize,
/// leaving the bytes to the caller to write wherever the user wants.
pub async fn export_pairing_plist(udid: Option<&str>) -> Result<Vec<u8>> {
    let provider = crate::devices::find_provider(udid).await?;
    get_pairing_plist(&provider).await
}
