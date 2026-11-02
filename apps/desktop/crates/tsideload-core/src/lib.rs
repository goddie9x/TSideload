//! `tsideload-core`: shared sideloading logic for the TSideload desktop app.
//!
//! Thin, opinionated wrapper around the [`isideload`] crate (MIT), which
//! implements the Apple free-provisioning dance: anisette -> Apple ID login
//! (+2FA) -> development certificate -> App ID -> provisioning profile ->
//! re-sign IPA -> install over usbmuxd.
//!
//! [`isideload`]: https://github.com/nab138/isideload

pub mod auth;
pub mod certs;
pub mod devices;
pub mod install;

pub use auth::{AppleSession, TwoFactorAnswer, TwoFactorPrompt};
pub use devices::DeviceInfo;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Anything bubbled up from the `isideload` crate / device protocols.
    #[error("sideload error: {0}")]
    Sideload(String),
    /// No iOS device visible over USB.
    #[error("no device found: connect an iPhone/iPad via USB and tap \"Trust this computer\"")]
    NoDevice,
    /// The requested UDID is not connected.
    #[error("device not found: {0}")]
    DeviceNotFound(String),
    /// Apple ID login / developer session failed.
    #[error("login failed: {0}")]
    Login(String),
    /// User-provided signing certificate problems (P1: import not implemented yet).
    #[error("certificate error: {0}")]
    Cert(String),
    /// Local I/O problems (missing IPA, unreadable files, ...).
    #[error("i/o error: {0}")]
    Io(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Initialize global state: rustls crypto provider + `isideload` error-reporting
/// hooks. Idempotent — safe to call more than once.
pub fn init() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let _ = isideload::init();
    });
}
