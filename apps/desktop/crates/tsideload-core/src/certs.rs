//! User-provided signing certificates (.p12 + .mobileprovision), KSign-style.
//!
//! # Status: NOT IMPLEMENTED (P1)
//!
//! The plan, once upstream `isideload` stabilizes its (currently rudimentary
//! and untested) imported-certificate support:
//!
//! 1. Parse the `.p12` (PKCS#12) into a signing identity (cert + private key).
//! 2. Parse the `.mobileprovision` plist; verify it covers the target device
//!    UDID and the IPA's bundle id, and that it has not expired.
//! 3. Plumb the identity into the signing step so `install_ipa` can use it
//!    instead of the Apple-ID developer flow.
//!
//! Until then this module only validates that the files exist, and
//! `tsd import-cert` reports the status honestly instead of pretending.

use std::path::Path;

use crate::{Error, Result};

/// A validated (but not yet usable) imported certificate bundle.
#[derive(Debug, Clone)]
pub struct ImportedCert {
    pub p12_path: String,
    pub provision_path: String,
}

/// Validate the files and register the bundle for future use.
///
/// Currently always returns [`Error::Cert`] explaining the P1 status,
/// after confirming both files exist.
pub fn import_cert(p12: &Path, _password: &str, provision: &Path) -> Result<ImportedCert> {
    for path in [p12, provision] {
        if !path.is_file() {
            return Err(Error::Cert(format!("file not found: {}", path.display())));
        }
    }
    Err(Error::Cert(
        "imported-certificate signing is not implemented yet (P1, see module docs); \
         use Apple ID signing via `tsd install` for now"
            .to_string(),
    ))
}
