//! Apple ID login with 2FA, producing a reusable developer session.

use std::sync::Arc;

use isideload::anisette::remote_v3::RemoteV3AnisetteProvider;
use isideload::auth::apple_account::{
    AppleAccount, TwoFactorCallbackParams, TwoFactorCallbackResponse,
};
use isideload::dev::developer_session::DeveloperSession;

use crate::{Error, Result};

/// One trusted phone number Apple can send the 2FA code to.
#[derive(Debug, Clone)]
pub struct TrustedPhoneNumber {
    pub id: u32,
    pub number: String,
}

/// What the UI (CLI prompt or GUI dialog) should ask the user for.
#[derive(Debug, Clone)]
pub struct TwoFactorPrompt {
    /// True when Apple offers SMS delivery for this attempt.
    pub sms: bool,
    /// True when Apple didn't say which 2FA method will work. In this state
    /// the UI must NOT offer code entry — only `SendToDevices` / `SendSms`
    /// / `Abort` are valid (submitting a code makes upstream bail out).
    pub unknown: bool,
    pub numbers: Vec<TrustedPhoneNumber>,
    pub selected_number_id: Option<u32>,
    /// Set when the previous attempt failed (e.g. wrong code).
    pub last_error: Option<String>,
}

/// The user's answer to a [`TwoFactorPrompt`].
#[derive(Debug, Clone)]
pub enum TwoFactorAnswer {
    /// The 6-digit code the user typed.
    Code(String),
    /// Ask Apple to resend the code.
    Resend,
    /// Deliver to the user's trusted devices instead of SMS.
    SendToDevices,
    /// Deliver via SMS to the given trusted number id.
    SendSms(u32),
    /// Give up on this login attempt.
    Abort,
}

/// An authenticated Apple developer session. Create once via [`login`] and
/// reuse for every install — it holds the development certificate.
pub struct AppleSession {
    pub apple_id: String,
    pub(crate) session: DeveloperSession,
}

/// Log in with an Apple ID + password.
///
/// `ask` is called (possibly multiple times) whenever Apple needs a 2FA
/// response. It may block (the core runs it via `block_in_place`), but call
/// `login` from a **multi-thread** Tokio runtime.
///
/// Tip: use a throwaway Apple ID for sideloading, and prefer an
/// app-specific password when 2FA is on.
pub async fn login(
    apple_id: &str,
    password: &str,
    ask: Arc<dyn Fn(TwoFactorPrompt) -> TwoFactorAnswer + Send + Sync>,
) -> Result<AppleSession> {
    // Adapt the caller's sync prompt into isideload's async 2FA callback.
    // The prompt may block (e.g. stdin); run it via block_in_place so we
    // don't stall the Tokio worker. Callers must therefore invoke `login`
    // from a multi-thread runtime (tsd and the GUI both do).
    let callback = move |params: TwoFactorCallbackParams| {
        let ask = Arc::clone(&ask);
        async move {
            let prompt = TwoFactorPrompt {
                sms: params.sms,
                unknown: params.unknown,
                numbers: params
                    .numbers
                    .iter()
                    .map(|n| TrustedPhoneNumber {
                        id: n.id,
                        number: n.number_with_dial_code.clone(),
                    })
                    .collect(),
                selected_number_id: params.selected_number_id,
                last_error: params.last_error.clone(),
            };
            let answer = tokio::task::block_in_place(|| ask(prompt));
            Ok(match answer {
                TwoFactorAnswer::Code(code) => TwoFactorCallbackResponse::SubmitCode(code),
                TwoFactorAnswer::Resend => TwoFactorCallbackResponse::ResendCode,
                TwoFactorAnswer::SendToDevices => TwoFactorCallbackResponse::SendToDevices,
                TwoFactorAnswer::SendSms(id) => TwoFactorCallbackResponse::SendSms(id),
                TwoFactorAnswer::Abort => TwoFactorCallbackResponse::Abort,
            })
        }
    };

    // NOTE: the serial override mirrors upstream isideload's own example.
    // In the pinned rev the provider never actually reads this field, so it
    // is currently a no-op — kept for forward-compatibility, not cargo-cult.
    let anisette = RemoteV3AnisetteProvider::default()
        .map_err(|e| Error::Login(format!("anisette init failed: {e:?}")))?
        .set_serial_number("2".to_string());

    let mut account = AppleAccount::builder(apple_id)
        .anisette_provider(anisette)
        .login(password, callback)
        .await
        .map_err(|e| Error::Login(format!("{e:?}")))?;

    let session = DeveloperSession::from_account(&mut account)
        .await
        .map_err(|e| Error::Login(format!("developer session failed: {e:?}")))?;

    Ok(AppleSession {
        apple_id: apple_id.to_string(),
        session,
    })
}
