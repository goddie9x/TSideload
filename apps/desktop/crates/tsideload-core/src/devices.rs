//! USB device discovery over usbmuxd.

use idevice::provider::UsbmuxdProvider;
use idevice::usbmuxd::{Connection, UsbmuxdAddr, UsbmuxdConnection};

use crate::{Error, Result};

/// A connected iOS/iPadOS device, as seen over USB.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub udid: String,
    pub connection_type: String,
}

fn connection_label(c: &Connection) -> String {
    // NOTE: if upstream `idevice` adds variants, this match will fail to
    // compile and remind us to handle them — preferable to silent fallthrough.
    match c {
        Connection::Usb => "usb".to_string(),
        Connection::Network(ip) => format!("network:{ip}"),
        Connection::Unknown(s) => format!("unknown:{s}"),
    }
}

/// List devices currently visible to usbmuxd.
///
/// Note: on Windows this needs the Apple mobile-device driver, which ships
/// with **Apple Devices** / **iTunes downloaded from Apple's website**
/// (the Microsoft Store builds do not expose it).
pub async fn list_devices() -> Result<Vec<DeviceInfo>> {
    let mut conn = UsbmuxdConnection::default()
        .await
        .map_err(|e| Error::Sideload(format!("cannot talk to usbmuxd: {e:?}")))?;
    let devices = conn
        .get_devices()
        .await
        .map_err(|e| Error::Sideload(format!("device list failed: {e:?}")))?;
    Ok(devices
        .into_iter()
        .map(|d| DeviceInfo {
            udid: d.udid,
            connection_type: connection_label(&d.connection_type),
        })
        .collect())
}

/// Build a device provider for installs: the requested UDID, or the first
/// connected device when `udid` is `None`.
pub(crate) async fn find_provider(udid: Option<&str>) -> Result<UsbmuxdProvider> {
    let mut conn = UsbmuxdConnection::default()
        .await
        .map_err(|e| Error::Sideload(format!("cannot talk to usbmuxd: {e:?}")))?;
    let devices = conn
        .get_devices()
        .await
        .map_err(|e| Error::Sideload(format!("device list failed: {e:?}")))?;
    let device = match udid {
        Some(want) => devices
            .into_iter()
            .find(|d| d.udid == want)
            .ok_or_else(|| Error::DeviceNotFound(want.to_string()))?,
        None => devices.into_iter().next().ok_or(Error::NoDevice)?,
    };
    Ok(device.to_provider(UsbmuxdAddr::default(), "tsideload-desktop"))
}
