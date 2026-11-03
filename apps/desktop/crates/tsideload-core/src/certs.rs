//! User-provided signing certificates (.p12 + .mobileprovision), KSign-style.
//!
//! Unlike the Apple-ID flow (free provisioning, 7-day certs), an imported
//! certificate lets you sign with your own identity — e.g. a paid Apple
//! Developer certificate or an enterprise certificate you legitimately own.
//! TSideload never bundles certificates; the user imports their own.
//!
//! Storage layout (under [`crate::storage_dir`]):
//! ```text
//! certs/<name>/identity.p12
//! certs/<name>/app.mobileprovision
//! certs/<name>/meta.json
//! ```
//! The .p12 password is **never stored**: it is needed once at import time
//! (to parse the key) and again at install time (or via the
//! `TSIDELOAD_P12_PASSWORD` env var).
//!
//! Signing itself reuses the `isideload` crate's battle-tested pipeline
//! ([`isideload::sideload::sign::sign`]): we build a `CertificateIdentity`
//! from the .p12 and a `Profile` from the .mobileprovision, then let the
//! crate do the CMS signing. What the crate *cannot* do for us is the Apple
//! Developer Portal dance (App IDs, team profiles) — with an imported cert
//! the user is responsible for providing a provision that matches the IPA's
//! bundle id and covers the target device, and we validate exactly that.

use std::fs;
use std::path::{Path, PathBuf};

use apple_codesign_quick::ProvisioningProfile;
use chrono::{DateTime, Utc};
use p12_keystore::{KeyStore, Pkcs12ImportPolicy};
use rsa::RsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use serde::{Deserialize, Serialize};
use x509_cert::Certificate;
use x509_cert::der::Decode;

use crate::{Error, Result};

/// Metadata persisted next to an imported certificate (`meta.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedCertMeta {
    pub name: String,
    pub team_id: String,
    /// Full `application-identifier` from the provision, e.g. `ABCDE12345.com.example.*`.
    pub application_identifier: String,
    pub profile_name: String,
    pub profile_uuid: String,
    /// RFC 3339 expiry of the provisioning profile.
    pub expires_rfc3339: String,
    /// `true` when the provision lists explicit `ProvisionedDevices`.
    pub device_restricted: bool,
    pub device_count: Option<usize>,
    pub cert_subject: String,
    pub imported_at_rfc3339: String,
}

/// An imported certificate, decrypted and ready to sign with.
#[derive(Debug)]
pub struct LoadedCert {
    pub meta: ImportedCertMeta,
    pub private_key: RsaPrivateKey,
    pub certificate: Certificate,
    pub provision_bytes: Vec<u8>,
}

fn certs_dir() -> PathBuf {
    crate::storage_dir().join("certs")
}

fn sanitize_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::Cert("certificate name must not be empty".to_string()));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(Error::Cert(format!(
            "certificate name '{name}' may only contain letters, digits, '-' and '_'"
        )));
    }
    Ok(name.to_string())
}

/// Parse a PKCS#12 file into an RSA private key + X.509 certificate.
///
/// Returns the key, the certificate, and the certificate's DER bytes (used
/// to check it against the provision's `DeveloperCertificates`).
fn parse_p12(bytes: &[u8], password: &str) -> Result<(RsaPrivateKey, Certificate, Vec<u8>)> {
    let ks = KeyStore::from_pkcs12(bytes, password, Pkcs12ImportPolicy::default())
        .map_err(|e| Error::Cert(format!("cannot parse .p12 (wrong password?): {e:?}")))?;
    let (_alias, chain) = ks
        .private_key_chain()
        .ok_or_else(|| Error::Cert("no private key found in .p12".to_string()))?;
    let private_key = RsaPrivateKey::from_pkcs8_der(chain.key().as_der()).map_err(|e| {
        Error::Cert(format!(
            "private key is not RSA PKCS#8 ({e}); only RSA keys are supported"
        ))
    })?;
    let cert_der: Vec<u8> = chain
        .certs()
        .first()
        .ok_or_else(|| Error::Cert("no certificate found in .p12".to_string()))?
        .as_der()
        .to_vec();
    let certificate = Certificate::from_der(&cert_der)
        .map_err(|e| Error::Cert(format!("cannot parse certificate: {e}")))?;
    Ok((private_key, certificate, cert_der))
}

struct ParsedProvision {
    team_id: String,
    application_identifier: String,
    profile_name: String,
    profile_uuid: String,
    expires: DateTime<Utc>,
    devices: Option<Vec<String>>,
    developer_cert_ders: Vec<Vec<u8>>,
    entitlements: plist::Dictionary,
}

/// Parse and validate a .mobileprovision file.
///
/// Checks: it parses, it has not expired. Device/bundle-id matching happens
/// at install time ([`sign_ipa`]) because it needs the target device and IPA.
fn parse_provision(bytes: &[u8]) -> Result<ParsedProvision> {
    let profile = ProvisioningProfile::parse(bytes)
        .map_err(|e| Error::Cert(format!("cannot parse .mobileprovision: {e}")))?;
    let dict = profile.plist();

    let get_str = |key: &str| -> Result<String> {
        dict.get(key)
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Cert(format!(".mobileprovision is missing '{key}'")))
    };
    let expires: DateTime<Utc> = dict
        .get("ExpirationDate")
        .and_then(|v| v.as_date())
        .map(|d| d.to_xml_format())
        .ok_or_else(|| Error::Cert(".mobileprovision is missing 'ExpirationDate'".to_string()))?
        .parse()
        .map_err(|e| Error::Cert(format!("bad ExpirationDate in .mobileprovision: {e}")))?;
    if expires <= Utc::now() {
        return Err(Error::Cert(format!(
            "provisioning profile '{}' expired on {}",
            get_str("Name").unwrap_or_default(),
            expires.format("%Y-%m-%d")
        )));
    }
    let entitlements = dict
        .get("Entitlements")
        .and_then(|v| v.as_dictionary())
        .cloned()
        .ok_or_else(|| Error::Cert(".mobileprovision is missing 'Entitlements'".to_string()))?;
    let application_identifier = entitlements
        .get("application-identifier")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            Error::Cert(".mobileprovision Entitlements is missing 'application-identifier'".to_string())
        })?;
    let devices: Option<Vec<String>> = dict
        .get("ProvisionedDevices")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_string().map(|s| s.to_string()))
                .collect()
        });
    let developer_cert_ders: Vec<Vec<u8>> = dict
        .get("DeveloperCertificates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_data().map(|d| AsRef::<[u8]>::as_ref(d).to_vec()))
                .collect()
        })
        .unwrap_or_default();

    Ok(ParsedProvision {
        team_id: profile.team_id().to_string(),
        application_identifier,
        profile_name: get_str("Name")?,
        profile_uuid: get_str("UUID")?,
        expires,
        devices,
        developer_cert_ders,
        entitlements,
    })
}

/// Import a .p12 + .mobileprovision pair under `name`.
///
/// Validates both files (parseable, provision not expired, the .p12's
/// certificate is authorized by the provision) and stores copies under the
/// storage dir. The password is used only to parse the .p12 and is never
/// stored.
pub fn import_cert(
    p12: &Path,
    password: &str,
    provision: &Path,
    name: Option<&str>,
) -> Result<ImportedCertMeta> {
    let p12_bytes = fs::read(p12)
        .map_err(|e| Error::Io(format!("cannot read {}: {e}", p12.display())))?;
    let prov_bytes = fs::read(provision)
        .map_err(|e| Error::Io(format!("cannot read {}: {e}", provision.display())))?;

    let (_key, _cert, cert_der) = parse_p12(&p12_bytes, password)?;
    let parsed = parse_provision(&prov_bytes)?;

    if !parsed.developer_cert_ders.iter().any(|d| d == &cert_der) {
        return Err(Error::Cert(
            "the .p12's certificate is not listed in the .mobileprovision's \
             DeveloperCertificates — this pair cannot sign together; make sure \
             the provision was created for this exact certificate"
                .to_string(),
        ));
    }

    let name = match name {
        Some(n) => sanitize_name(n)?,
        None => p12
            .file_stem()
            .and_then(|s| s.to_str())
            .map(sanitize_name)
            .transpose()?
            .unwrap_or_else(|| "default".to_string()),
    };
    let dir = certs_dir().join(&name);
    if dir.exists() {
        return Err(Error::Cert(format!(
            "a certificate named '{name}' is already imported; \
             delete {} to replace it",
            dir.display()
        )));
    }
    fs::create_dir_all(&dir)
        .map_err(|e| Error::Io(format!("cannot create {}: {e}", dir.display())))?;
    fs::write(dir.join("identity.p12"), &p12_bytes)
        .map_err(|e| Error::Io(format!("cannot store .p12: {e}")))?;
    fs::write(dir.join("app.mobileprovision"), &prov_bytes)
        .map_err(|e| Error::Io(format!("cannot store .mobileprovision: {e}")))?;

    let cert_subject = x509_cert::Certificate::from_der(&cert_der)
        .map(|c| c.tbs_certificate.subject.to_string())
        .unwrap_or_default();
    let meta = ImportedCertMeta {
        application_identifier: parsed.application_identifier,
        profile_name: parsed.profile_name,
        profile_uuid: parsed.profile_uuid,
        expires_rfc3339: parsed.expires.to_rfc3339(),
        device_restricted: parsed.devices.is_some(),
        device_count: parsed.devices.as_ref().map(|d| d.len()),
        team_id: parsed.team_id,
        cert_subject,
        imported_at_rfc3339: Utc::now().to_rfc3339(),
        name: name.clone(),
    };
    fs::write(
        dir.join("meta.json"),
        serde_json::to_string_pretty(&meta)
            .map_err(|e| Error::Io(format!("cannot serialize metadata: {e}")))?,
    )
    .map_err(|e| Error::Io(format!("cannot store metadata: {e}")))?;

    Ok(meta)
}

/// List all imported certificates (metadata only, no secrets).
pub fn list_certs() -> Result<Vec<ImportedCertMeta>> {
    let dir = certs_dir();
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entries =
        fs::read_dir(&dir).map_err(|e| Error::Io(format!("cannot list {}: {e}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| Error::Io(format!("cannot read cert dir entry: {e}")))?;
        let meta_path = entry.path().join("meta.json");
        if !meta_path.is_file() {
            continue;
        }
        let bytes = fs::read(&meta_path)
            .map_err(|e| Error::Io(format!("cannot read {}: {e}", meta_path.display())))?;
        if let Ok(meta) = serde_json::from_slice::<ImportedCertMeta>(&bytes) {
            out.push(meta);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Load an imported certificate, decrypting the .p12 with `password`.
pub fn load_cert(name: &str, password: &str) -> Result<LoadedCert> {
    let name = sanitize_name(name)?;
    let dir = certs_dir().join(&name);
    let meta_path = dir.join("meta.json");
    if !meta_path.is_file() {
        let known: Vec<String> = list_certs().unwrap_or_default().into_iter().map(|m| m.name).collect();
        return Err(Error::Cert(format!(
            "no imported certificate named '{name}'{}",
            if known.is_empty() {
                String::new()
            } else {
                format!(" (known: {})", known.join(", "))
            }
        )));
    }
    let meta: ImportedCertMeta = serde_json::from_slice(
        &fs::read(&meta_path)
            .map_err(|e| Error::Io(format!("cannot read {}: {e}", meta_path.display())))?,
    )
    .map_err(|e| Error::Cert(format!("corrupt metadata for '{name}': {e}")))?;
    let p12_bytes = fs::read(dir.join("identity.p12"))
        .map_err(|e| Error::Io(format!("cannot read stored .p12: {e}")))?;
    let provision_bytes = fs::read(dir.join("app.mobileprovision"))
        .map_err(|e| Error::Io(format!("cannot read stored .mobileprovision: {e}")))?;
    let (private_key, certificate, _der) = parse_p12(&p12_bytes, password)?;
    Ok(LoadedCert {
        meta,
        private_key,
        certificate,
        provision_bytes,
    })
}

/// Does `application-identifier` (e.g. `TEAMID.com.example.*`) cover `bundle_id`?
fn bundle_matches(application_identifier: &str, team_id: &str, bundle_id: &str) -> bool {
    let pattern = application_identifier
        .strip_prefix(&format!("{team_id}."))
        .unwrap_or(application_identifier);
    if pattern == "*" {
        return true;
    }
    if let Some(stem) = pattern.strip_suffix(".*") {
        return bundle_id == stem || bundle_id.starts_with(&format!("{stem}."));
    }
    bundle_id == pattern
}

/// Sign `ipa` with an imported certificate for device `udid`.
///
/// Returns the signed `.app` directory (in a temp dir). Validates at signing
/// time: the provision must not have expired since import, the device must be
/// in `ProvisionedDevices` (when the provision is device-restricted), and the
/// IPA's bundle id must match the provision's `application-identifier`.
pub(crate) async fn sign_ipa(
    ipa: &Path,
    cert: &LoadedCert,
    udid: &str,
) -> Result<PathBuf> {
    use isideload::dev::{app_ids::Profile, teams::DeveloperTeam};
    use isideload::sideload::application::{Application, SpecialApp};
    use isideload::sideload::cert_identity::CertificateIdentity;
    use isideload::sideload::sign;

    let parsed = parse_provision(&cert.provision_bytes)?;
    if let Some(devices) = &parsed.devices {
        if !devices.iter().any(|d| d == udid) {
            return Err(Error::Cert(format!(
                "device {udid} is not in this provision's ProvisionedDevices list \
                 ({} devices); add it on the Apple Developer portal and re-import",
                devices.len()
            )));
        }
    }

    let mut app = Application::new(ipa.to_path_buf())
        .map_err(|e| Error::Sideload(format!("cannot open ipa: {e:?}")))?;
    let bundle_id = app
        .main_bundle_id()
        .map_err(|e| Error::Sideload(format!("cannot read ipa bundle id: {e:?}")))?;
    if !bundle_matches(
        &parsed.application_identifier,
        &parsed.team_id,
        &bundle_id,
    ) {
        return Err(Error::Cert(format!(
            "provision '{}' covers '{}' but the IPA's bundle id is '{}'; \
             KSign-style import requires a provision made for this exact app id \
             (wildcards like com.example.* are fine)",
            parsed.profile_name, parsed.application_identifier, bundle_id
        )));
    }

    let identity = CertificateIdentity {
        machine_id: String::new(),
        machine_name: "tsideload-desktop".to_string(),
        private_key: cert.private_key.clone(),
        certificate: cert.certificate.clone(),
    };
    // `Profile` is the crate's Apple-Portal DTO; only `encoded_profile` is
    // read by the signer — the rest is filled from the real provision for
    // honesty in case anything else inspects it.
    let profile = Profile {
        encoded_profile: plist::Data::new(cert.provision_bytes.clone()),
        filename: "imported.mobileprovision".to_string(),
        provisioning_profile_id: parsed.profile_uuid.clone(),
        name: parsed.profile_name.clone(),
        status: "Active".to_string(),
        r#type: "Imported".to_string(),
        distribution_method: "imported".to_string(),
        pro_pro_platorm: None,
        uuid: parsed.profile_uuid.clone(),
        date_expire: plist::Date::from_xml_format(&parsed.expires.to_rfc3339())
            .unwrap_or_else(|_| plist::Date::from_xml_format("2030-01-01T00:00:00Z").unwrap()),
        managing_app: None,
        app_id_id: String::new(),
        is_template_profile: false,
        is_team_profile: None,
        is_free_provisioning_profile: None,
    };
    let team = DeveloperTeam {
        name: None,
        team_id: parsed.team_id.clone(),
        r#type: None,
        status: None,
    };

    // App extensions reuse the main provision (mirrors the crate's
    // `ExtensionsBehavior::ReuseMain` path used by the Apple-ID flow).
    let entitlements = parsed.entitlements.clone();
    let mut all_profiles: Vec<(String, Profile, plist::Dictionary)> = Vec::new();
    for ext in app.bundle.app_extensions() {
        all_profiles.push((
            ext.bundle_identifier().unwrap_or_default().to_string(),
            profile.clone(),
            entitlements.clone(),
        ));
    }

    app.bundle
        .write_info()
        .map_err(|e| Error::Sideload(format!("cannot rewrite Info.plist: {e:?}")))?;
    for ext in app.bundle.app_extensions_mut() {
        ext.write_info()
            .map_err(|e| Error::Sideload(format!("cannot rewrite extension Info.plist: {e:?}")))?;
    }
    for fw in app.bundle.frameworks_mut() {
        fw.write_info()
            .map_err(|e| Error::Sideload(format!("cannot rewrite framework Info.plist: {e:?}")))?;
    }

    let special: Option<SpecialApp> = None;
    sign::sign(
        &mut app,
        &identity,
        &profile,
        &mut all_profiles,
        &special,
        &team,
        None::<fn(f32) -> std::future::Ready<()>>,
    )
    .await
    .map_err(|e| Error::Sideload(format!("sign failed: {e:?}")))?;

    Ok(app.bundle.bundle_dir.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_pattern_matching() {
        // exact
        assert!(bundle_matches("ABCDE12345.com.example.app", "ABCDE12345", "com.example.app"));
        assert!(!bundle_matches("ABCDE12345.com.example.app", "ABCDE12345", "com.example.other"));
        // wildcard
        assert!(bundle_matches("ABCDE12345.com.example.*", "ABCDE12345", "com.example.app"));
        assert!(bundle_matches("ABCDE12345.com.example.*", "ABCDE12345", "com.example.app.ext"));
        assert!(!bundle_matches("ABCDE12345.com.example.*", "ABCDE12345", "com.other.app"));
        // bare star
        assert!(bundle_matches("ABCDE12345.*", "ABCDE12345", "anything.at.all"));
        // team prefix mismatch falls back to comparing the raw pattern
        assert!(!bundle_matches("WRONGTEAM.com.example.app", "ABCDE12345", "com.example.app"));
    }

    #[test]
    fn name_sanitization() {
        assert_eq!(sanitize_name("my-cert_1").unwrap(), "my-cert_1");
        assert!(sanitize_name("").is_err());
        assert!(sanitize_name("../evil").is_err());
        assert!(sanitize_name("a b").is_err());
        assert!(sanitize_name("café").is_err());
    }

    #[test]
    fn import_rejects_missing_files() {
        let err = import_cert(
            Path::new("/nonexistent/a.p12"),
            "",
            Path::new("/nonexistent/b.mobileprovision"),
            Some("x"),
        )
        .unwrap_err();
        assert!(matches!(err, Error::Io(_)), "expected Io error, got {err:?}");
    }

    #[test]
    fn load_unknown_cert_lists_known() {
        let err = load_cert("definitely-not-a-real-cert-xyz", "").unwrap_err();
        assert!(matches!(err, Error::Cert(_)), "expected Cert error, got {err:?}");
    }
}
