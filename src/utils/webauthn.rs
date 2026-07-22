//! Browser WebAuthn (passkey / Windows Hello) ceremonies.
//!
//! This is the fiddly half of passkey sign-in: driving
//! `navigator.credentials.create()` / `.get()` and marshalling the option and
//! response objects across the JS boundary. It is shaped to the **JSON**
//! WebAuthn contract (`PublicKeyCredentialCreationOptionsJSON` /
//! `RegistrationResponseJSON` / `AuthenticationResponseJSON`), which is exactly
//! what identity providers hand out and expect back — so a caller can pass an
//! IdP's options string straight in and post the returned string straight back.
//!
//! ## No HTTP here, by design
//!
//! Like [`crate::utils::swr`], this module **never performs a request**: the
//! fetch layer stays the host's choice (`gloo-net`, `reqwest`, a hand-rolled
//! binding). That keeps the library free of any one IdP's transport, auth
//! headers, or error envelope.
//!
//! ### Pairing it with Amazon Cognito
//!
//! Registration (needs an access token with `aws.cognito.signin.user.admin`):
//!
//! 1. host: `StartWebAuthnRegistration` → `CredentialCreationOptions` (JSON)
//! 2. here: [`create_credential`] → `RegistrationResponseJSON`
//! 3. host: `CompleteWebAuthnRegistration { Credential: <that JSON> }`
//!
//! Sign-in:
//!
//! 1. host: `InitiateAuth` (`USER_AUTH`, `PREFERRED_CHALLENGE=WEB_AUTHN`) →
//!    `ChallengeParameters.CREDENTIAL_REQUEST_OPTIONS` (JSON) + `Session`
//! 2. here: [`get_assertion`] → `AuthenticationResponseJSON`
//! 3. host: `RespondToAuthChallenge { ChallengeName: WEB_AUTHN, Session,
//!    ChallengeResponses: { USERNAME, CREDENTIAL: <that JSON> } }`
//!
//! ## Why a JS shim
//!
//! `PublicKeyCredential.parseCreationOptionsFromJSON` / `.toJSON()` are the
//! JSON-native WebAuthn APIs; they are not exposed by `web-sys` (whose WebAuthn
//! surface is unstable and requires hand base64url-ing every binary field).
//! Calling the browser's own JSON helpers via a tiny `inline_js` shim is both
//! smaller and less error-prone — the browser does the encoding.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function __ldui_webauthn_supported() {
  return !!(window.PublicKeyCredential
    && PublicKeyCredential.parseCreationOptionsFromJSON
    && PublicKeyCredential.parseRequestOptionsFromJSON);
}
export async function __ldui_webauthn_create(optionsJson) {
  const opts = PublicKeyCredential.parseCreationOptionsFromJSON(JSON.parse(optionsJson));
  const cred = await navigator.credentials.create({ publicKey: opts });
  if (!cred) throw new Error("no credential returned");
  return JSON.stringify(cred.toJSON());
}
export async function __ldui_webauthn_get(optionsJson) {
  const opts = PublicKeyCredential.parseRequestOptionsFromJSON(JSON.parse(optionsJson));
  const cred = await navigator.credentials.get({ publicKey: opts });
  if (!cred) throw new Error("no assertion returned");
  return JSON.stringify(cred.toJSON());
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = __ldui_webauthn_supported)]
    fn js_supported() -> bool;

    #[wasm_bindgen(js_name = __ldui_webauthn_create, catch)]
    async fn js_create(options_json: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = __ldui_webauthn_get, catch)]
    async fn js_get(options_json: &str) -> Result<JsValue, JsValue>;
}

/// Why a WebAuthn ceremony did not produce a credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebAuthnError {
    /// The browser has no JSON-native WebAuthn support (too old), or the page
    /// is not in a secure context.
    Unsupported,
    /// The user dismissed the prompt, or no matching credential was available.
    ///
    /// This is the ORDINARY "not this time" case — offer the password path
    /// rather than presenting it as a failure.
    Cancelled,
    /// Anything else, with the browser's message.
    Failed(String),
}

impl std::fmt::Display for WebAuthnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebAuthnError::Unsupported => {
                write!(f, "this browser does not support passkeys")
            }
            WebAuthnError::Cancelled => write!(f, "the passkey prompt was dismissed"),
            WebAuthnError::Failed(m) => write!(f, "passkey error: {m}"),
        }
    }
}

impl std::error::Error for WebAuthnError {}

/// Whether this browser exposes the JSON-native WebAuthn API.
///
/// Check before offering a passkey action — an action that cannot succeed
/// should be disabled, not left to fail on click.
pub fn is_supported() -> bool {
    js_supported()
}

/// Classify a rejected DOM promise. `NotAllowedError` covers both a user
/// dismissal and "no credential matched", which the spec deliberately makes
/// indistinguishable so a site cannot probe for enrolled credentials.
fn classify(err: JsValue) -> WebAuthnError {
    let msg = err
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(&err, &JsValue::from_str("message"))
                .ok()
                .and_then(|m| m.as_string())
        })
        .unwrap_or_else(|| "unknown error".to_string());
    let name = js_sys::Reflect::get(&err, &JsValue::from_str("name"))
        .ok()
        .and_then(|n| n.as_string())
        .unwrap_or_default();
    if name == "NotAllowedError" || name == "AbortError" {
        WebAuthnError::Cancelled
    } else {
        WebAuthnError::Failed(msg)
    }
}

/// Run a **registration** ceremony: enroll a new passkey on this device.
///
/// `options_json` is the IdP's `PublicKeyCredentialCreationOptionsJSON`
/// (Cognito: `StartWebAuthnRegistration`'s `CredentialCreationOptions`). The
/// returned string is `RegistrationResponseJSON`, to post back verbatim
/// (Cognito: `CompleteWebAuthnRegistration`'s `Credential`).
///
/// Shows the platform prompt — Windows Hello, Touch ID, a security key.
pub async fn create_credential(options_json: &str) -> Result<String, WebAuthnError> {
    if !is_supported() {
        return Err(WebAuthnError::Unsupported);
    }
    match js_create(options_json).await {
        Ok(v) => v
            .as_string()
            .ok_or_else(|| WebAuthnError::Failed("credential was not a string".into())),
        Err(e) => Err(classify(e)),
    }
}

/// Run an **authentication** ceremony: prove possession of an enrolled passkey.
///
/// `options_json` is the IdP's `PublicKeyCredentialRequestOptionsJSON` (Cognito:
/// the `CREDENTIAL_REQUEST_OPTIONS` challenge parameter). The returned string is
/// `AuthenticationResponseJSON`, to post back verbatim (Cognito: the
/// `CREDENTIAL` challenge response).
pub async fn get_assertion(options_json: &str) -> Result<String, WebAuthnError> {
    if !is_supported() {
        return Err(WebAuthnError::Unsupported);
    }
    match js_get(options_json).await {
        Ok(v) => v
            .as_string()
            .ok_or_else(|| WebAuthnError::Failed("assertion was not a string".into())),
        Err(e) => Err(classify(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dismissed prompt is the ordinary case and must be distinguishable from
    /// a real error, so a host can fall back to the password path quietly
    /// instead of showing an alarming message.
    #[test]
    fn cancellation_is_its_own_variant_and_reads_calmly() {
        assert_eq!(WebAuthnError::Cancelled, WebAuthnError::Cancelled);
        assert_ne!(
            WebAuthnError::Cancelled,
            WebAuthnError::Failed("boom".into())
        );
        assert!(WebAuthnError::Cancelled.to_string().contains("dismissed"));
        assert!(
            WebAuthnError::Unsupported
                .to_string()
                .contains("does not support")
        );
        assert!(
            WebAuthnError::Failed("boom".into())
                .to_string()
                .contains("boom")
        );
    }
}
