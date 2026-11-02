//! TSideload desktop GUI (Tauri shell — scaffold).
//!
//! `list_devices` is wired to `tsideload-core`. The install flow needs an
//! interactive Apple ID + 2FA dialog and is P1; until then the command
//! below reports that honestly instead of half-working.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use tsideload_core::DeviceInfo;

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

#[tauri::command]
async fn list_devices() -> Result<Vec<DeviceDto>, String> {
    tsideload_core::init();
    let devices = tsideload_core::devices::list_devices()
        .await
        .map_err(|e| e.to_string())?;
    Ok(devices.into_iter().map(DeviceDto::from).collect())
}

/// P1: full install flow (Apple ID + password + 2FA dialog) lives here.
/// The core already exposes everything needed:
/// `auth::login` + `install::install_ipa`.
#[tauri::command]
async fn install_ipa(
    _apple_id: String,
    _password: String,
    _ipa_path: String,
    _udid: Option<String>,
) -> Result<String, String> {
    Err(
        "Install from the GUI is not wired yet (P1). Use `tsd install` for now."
            .to_string(),
    )
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_devices, install_ipa])
        .run(tauri::generate_context!())
        .expect("failed to run TSideload GUI");
}
