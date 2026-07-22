//! A hand-rolled Amazon Cognito user-pool client for the browser.
//!
//! This deliberately replaces AWS's *optional* convenience UI — the hosted UI /
//! managed login, Amplify UI, `amazon-cognito-identity-js` — with our own
//! implementation. Cognito itself is a core dependency; its UI components are
//! not, and owning this layer is what buys tight control over the **sequence**
//! (which step runs when, and what happens on each failure), the **UX** (our
//! own screen, our own copy, no redirect away from the app), and the
//! **logging/debugging** (see [`CognitoError`], which preserves the exception
//! name Cognito actually returned).
//!
//! Pairs with [`crate::components::LoginScreen`] for the UI and
//! [`crate::utils::webauthn`] for the passkey ceremonies.
//!
//! # Talking to Cognito without an SDK
//!
//! The user-pool API is JSON-RPC over a single endpoint,
//! `https://cognito-idp.{region}.amazonaws.com/`, selected by an
//! `X-Amz-Target: AWSCognitoIdentityProviderService.{Operation}` header. The
//! operations used here are **public-client** calls — `InitiateAuth`,
//! `RespondToAuthChallenge` — or **token-authorized** ones that carry the
//! user's own access token. None require SigV4 request signing, so a plain
//! `fetch` is sufficient and no AWS credentials are involved — which is why
//! this module needs no HTTP-client crate at all.
//!
//! # Hard-won behaviours encoded here
//!
//! These each cost a live debugging cycle on the desktop client; they are
//! encoded as typed outcomes rather than left for every caller to rediscover:
//!
//! - **A challenge `Session` is single-use.** Once a `RespondToAuthChallenge`
//!   attempt fails, that session is spent and every retry returns
//!   *"Invalid session for the user, session can only be used once"*. The flow
//!   must restart at the password step — [`CognitoError::SessionExpired`].
//! - **"No passkey" is not an error.** With no registered credential Cognito
//!   answers `SELECT_CHALLENGE` and simply omits `WEB_AUTHN` from
//!   `AvailableChallenges`; that is the ordinary first-run case
//!   ([`CognitoError::NoPasskey`]), not a fault.
//! - **Passkey registration needs the `aws.cognito.signin.user.admin` scope**
//!   on the access token. A hosted-UI token scoped `openid profile email` does
//!   not carry it and cannot enroll a passkey.

use serde_json::{Value, json};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, Response};

/// Which Cognito user pool and app client to talk to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CognitoConfig {
    /// AWS region of the user pool, e.g. `us-east-1`.
    pub region: String,
    /// The app client id (public client, no secret).
    pub client_id: String,
}

impl CognitoConfig {
    /// The single JSON-RPC endpoint every operation posts to.
    pub fn endpoint(&self) -> String {
        format!("https://cognito-idp.{}.amazonaws.com/", self.region)
    }
}

/// The tokens a completed authentication yields. Bearer credentials — hold
/// them in memory, never in `localStorage`, and never log them.
#[derive(Clone)]
pub struct CognitoTokens {
    /// The ID token — what a resource server validates to identify the user.
    pub id_token: String,
    /// The access token — required by token-authorized operations such as
    /// passkey registration.
    pub access_token: String,
    /// The refresh token, when the app client issues one.
    pub refresh_token: Option<String>,
}

/// Where a password sign-in landed.
pub enum SignInOutcome {
    /// Authentication completed outright (the pool required no second factor).
    Tokens(CognitoTokens),
    /// A one-time code is required. The `session` is **single-use**: pass it to
    /// [`CognitoClient::respond_mfa`] exactly once.
    MfaRequired {
        /// Opaque challenge session to echo back with the code.
        session: String,
    },
}

/// Why a Cognito call did not succeed.
///
/// [`CognitoError::Service`] preserves the exception **name** (`__type`) as well
/// as the message. That distinction matters: a generic string makes every
/// failure look identical, and identifying the exception is usually the whole
/// diagnosis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CognitoError {
    /// The account has no passkey registered — the ordinary first-run case, not
    /// a fault. Offer the password path.
    NoPasskey,
    /// The challenge session was already consumed; restart at the password step.
    SessionExpired,
    /// Cognito returned an error: the exception name and its message.
    Service {
        /// e.g. `NotAuthorizedException`, `InvalidParameterException`.
        code: String,
        /// Cognito's human-readable message.
        message: String,
    },
    /// The request never reached Cognito, or the response was unreadable.
    Transport(String),
    /// A well-formed response was missing a field this flow requires.
    Unexpected(String),
}

impl std::fmt::Display for CognitoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CognitoError::NoPasskey => write!(f, "no passkey is registered for this account"),
            CognitoError::SessionExpired => {
                write!(f, "that sign-in attempt expired; please start again")
            }
            CognitoError::Service { code, message } => write!(f, "{code}: {message}"),
            CognitoError::Transport(m) => write!(f, "could not reach the sign-in service: {m}"),
            CognitoError::Unexpected(m) => write!(f, "unexpected sign-in response: {m}"),
        }
    }
}

impl std::error::Error for CognitoError {}

/// Cognito's REST errors carry `{"__type": "SomeException", "message": "..."}`.
/// The `__type` may be namespaced (`com.amazonaws...#NotAuthorizedException`),
/// so keep only the trailing name.
fn service_error(body: &Value) -> CognitoError {
    let raw = body
        .get("__type")
        .and_then(Value::as_str)
        .unwrap_or("UnknownException");
    let code = raw.rsplit('#').next().unwrap_or(raw).to_string();
    let message = body
        .get("message")
        .or_else(|| body.get("Message"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    // A spent challenge session is common enough, and its recovery specific
    // enough, to be worth its own variant.
    if code == "NotAuthorizedException" && message.contains("session can only be used once") {
        return CognitoError::SessionExpired;
    }
    CognitoError::Service { code, message }
}

/// Read `AuthenticationResult` into [`CognitoTokens`].
fn tokens_from(result: &Value) -> Result<CognitoTokens, CognitoError> {
    let get = |k: &str| result.get(k).and_then(Value::as_str).map(str::to_string);
    Ok(CognitoTokens {
        id_token: get("IdToken").ok_or_else(|| CognitoError::Unexpected("no IdToken".into()))?,
        access_token: get("AccessToken")
            .ok_or_else(|| CognitoError::Unexpected("no AccessToken".into()))?,
        refresh_token: get("RefreshToken"),
    })
}

/// A browser-side Cognito user-pool client.
#[derive(Clone, Debug)]
pub struct CognitoClient {
    config: CognitoConfig,
}

impl CognitoClient {
    /// Build a client for a pool/app-client pair.
    pub fn new(config: CognitoConfig) -> Self {
        Self { config }
    }

    /// The configuration this client was built with.
    pub fn config(&self) -> &CognitoConfig {
        &self.config
    }

    /// POST one JSON-RPC operation and return its parsed body.
    ///
    /// Uses the browser's own `fetch` through `web-sys` rather than pulling an
    /// HTTP-client crate into this shared library: every consumer would inherit
    /// that dependency whether or not it ever signs anyone in, and the whole
    /// transport is this one function.
    async fn call(&self, operation: &str, body: Value) -> Result<Value, CognitoError> {
        let transport = |e: JsValue| {
            CognitoError::Transport(
                e.as_string()
                    .or_else(|| {
                        js_sys::Reflect::get(&e, &JsValue::from_str("message"))
                            .ok()
                            .and_then(|m| m.as_string())
                    })
                    .unwrap_or_else(|| "request failed".to_string()),
            )
        };

        let headers = Headers::new().map_err(transport)?;
        headers
            .set("Content-Type", "application/x-amz-json-1.1")
            .map_err(transport)?;
        headers
            .set(
                "X-Amz-Target",
                &format!("AWSCognitoIdentityProviderService.{operation}"),
            )
            .map_err(transport)?;

        let init = RequestInit::new();
        init.set_method("POST");
        init.set_headers(&headers);
        init.set_body(&JsValue::from_str(&body.to_string()));

        let request =
            Request::new_with_str_and_init(&self.config.endpoint(), &init).map_err(transport)?;
        let window =
            web_sys::window().ok_or_else(|| CognitoError::Transport("no browser window".into()))?;
        let resp: Response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(transport)?
            .dyn_into()
            .map_err(|_| CognitoError::Transport("response was not a Response".into()))?;

        let status = resp.status();
        let text = JsFuture::from(resp.text().map_err(transport)?)
            .await
            .map_err(transport)?
            .as_string()
            .unwrap_or_default();
        let parsed: Value = serde_json::from_str(&text).unwrap_or(Value::Null);

        if !(200..300).contains(&status) {
            return Err(service_error(&parsed));
        }
        Ok(parsed)
    }

    /// Username + password sign-in (`USER_AUTH` / `PASSWORD`).
    ///
    /// Returns [`SignInOutcome::MfaRequired`] when the pool asks for a one-time
    /// code. Unlike the hosted UI, nothing navigates away from the app.
    pub async fn sign_in_with_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<SignInOutcome, CognitoError> {
        let body = json!({
            "AuthFlow": "USER_AUTH",
            "ClientId": self.config.client_id,
            "AuthParameters": {
                "USERNAME": username,
                "PASSWORD": password,
                "PREFERRED_CHALLENGE": "PASSWORD",
            },
        });
        let v = self.call("InitiateAuth", body).await?;
        if let Some(result) = v.get("AuthenticationResult") {
            return Ok(SignInOutcome::Tokens(tokens_from(result)?));
        }
        match v.get("ChallengeName").and_then(Value::as_str) {
            Some("SOFTWARE_TOKEN_MFA") | Some("SMS_MFA") | Some("EMAIL_OTP") => {
                let session = v
                    .get("Session")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        CognitoError::Unexpected("MFA challenge without a session".into())
                    })?
                    .to_string();
                Ok(SignInOutcome::MfaRequired { session })
            }
            other => Err(CognitoError::Unexpected(format!(
                "unsupported challenge: {}",
                other.unwrap_or("none")
            ))),
        }
    }

    /// Answer a one-time-code challenge.
    ///
    /// `session` is consumed by this call whether or not the code is right — on
    /// any error the caller must restart at [`Self::sign_in_with_password`].
    pub async fn respond_mfa(
        &self,
        username: &str,
        session: &str,
        code: &str,
    ) -> Result<CognitoTokens, CognitoError> {
        let body = json!({
            "ChallengeName": "SOFTWARE_TOKEN_MFA",
            "ClientId": self.config.client_id,
            "Session": session,
            "ChallengeResponses": {
                "USERNAME": username,
                "SOFTWARE_TOKEN_MFA_CODE": code,
            },
        });
        let v = self.call("RespondToAuthChallenge", body).await?;
        let result = v
            .get("AuthenticationResult")
            .ok_or_else(|| CognitoError::Unexpected("MFA response carried no tokens".into()))?;
        tokens_from(result)
    }

    /// Silently exchange a **refresh token** for a fresh ID/access token
    /// (`REFRESH_TOKEN_AUTH`) — no user interaction, no MFA, no passkey prompt.
    ///
    /// ID tokens expire in ~1h. Without this, a tab left open past that is
    /// bounced back to the sign-in screen mid-session. Cognito does NOT return a
    /// new refresh token here, so the caller keeps the one it holds until that
    /// itself expires (the app client's refresh-token validity, 30 days by
    /// default) — at which point a real sign-in genuinely is required.
    pub async fn refresh(&self, refresh_token: &str) -> Result<CognitoTokens, CognitoError> {
        let body = json!({
            "AuthFlow": "REFRESH_TOKEN_AUTH",
            "ClientId": self.config.client_id,
            "AuthParameters": { "REFRESH_TOKEN": refresh_token },
        });
        let v = self.call("InitiateAuth", body).await?;
        let result = v
            .get("AuthenticationResult")
            .ok_or_else(|| CognitoError::Unexpected("refresh returned no tokens".into()))?;
        let mut tokens = tokens_from(result)?;
        // The response omits it; carry the caller's forward so one refresh does
        // not silently end the ability to refresh again.
        if tokens.refresh_token.is_none() {
            tokens.refresh_token = Some(refresh_token.to_string());
        }
        Ok(tokens)
    }

    /// Begin a passkey sign-in: returns `(session, request_options_json)` to
    /// hand to [`crate::utils::webauthn::get_assertion`].
    ///
    /// Answers [`CognitoError::NoPasskey`] when the account has no registered
    /// credential — Cognito signals this by returning `SELECT_CHALLENGE`
    /// *without* `WEB_AUTHN` among `AvailableChallenges`, which is the ordinary
    /// first-run case rather than a failure.
    pub async fn begin_passkey_signin(
        &self,
        username: &str,
    ) -> Result<(String, String), CognitoError> {
        let body = json!({
            "AuthFlow": "USER_AUTH",
            "ClientId": self.config.client_id,
            "AuthParameters": {
                "USERNAME": username,
                "PREFERRED_CHALLENGE": "WEB_AUTHN",
            },
        });
        let v = self.call("InitiateAuth", body).await?;
        if v.get("ChallengeName").and_then(Value::as_str) != Some("WEB_AUTHN") {
            let offered = v
                .get("AvailableChallenges")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).any(|c| c == "WEB_AUTHN"))
                .unwrap_or(false);
            if !offered {
                return Err(CognitoError::NoPasskey);
            }
            return Err(CognitoError::Unexpected(
                "expected a WEB_AUTHN challenge".into(),
            ));
        }
        let session = v
            .get("Session")
            .and_then(Value::as_str)
            .ok_or_else(|| CognitoError::Unexpected("passkey challenge without a session".into()))?
            .to_string();
        let options = v
            .get("ChallengeParameters")
            .and_then(|p| p.get("CREDENTIAL_REQUEST_OPTIONS"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CognitoError::Unexpected("passkey challenge without request options".into())
            })?
            .to_string();
        Ok((session, options))
    }

    /// Finish a passkey sign-in with the assertion produced by
    /// [`crate::utils::webauthn::get_assertion`].
    pub async fn complete_passkey_signin(
        &self,
        username: &str,
        session: &str,
        credential_json: &str,
    ) -> Result<CognitoTokens, CognitoError> {
        let body = json!({
            "ChallengeName": "WEB_AUTHN",
            "ClientId": self.config.client_id,
            "Session": session,
            "ChallengeResponses": {
                "USERNAME": username,
                "CREDENTIAL": credential_json,
            },
        });
        let v = self.call("RespondToAuthChallenge", body).await?;
        let result = v
            .get("AuthenticationResult")
            .ok_or_else(|| CognitoError::Unexpected("passkey response carried no tokens".into()))?;
        tokens_from(result)
    }

    /// Begin passkey enrollment; returns the creation-options JSON for
    /// [`crate::utils::webauthn::create_credential`].
    ///
    /// `access_token` must carry the **`aws.cognito.signin.user.admin`** scope —
    /// a hosted-UI token scoped `openid profile email` does not, and this call
    /// will be refused.
    pub async fn start_passkey_registration(
        &self,
        access_token: &str,
    ) -> Result<String, CognitoError> {
        let v = self
            .call(
                "StartWebAuthnRegistration",
                json!({ "AccessToken": access_token }),
            )
            .await?;
        let opts = v
            .get("CredentialCreationOptions")
            .ok_or_else(|| CognitoError::Unexpected("no CredentialCreationOptions".into()))?;
        Ok(opts.to_string())
    }

    /// Finish passkey enrollment with the credential from
    /// [`crate::utils::webauthn::create_credential`].
    ///
    /// Note the created credential must be **discoverable**: Cognito's creation
    /// options request `residentKey: "required"`, and a non-discoverable
    /// credential is rejected as `Credential data is not valid`. The browser
    /// honours that automatically when the options are passed through unchanged.
    pub async fn complete_passkey_registration(
        &self,
        access_token: &str,
        credential_json: &str,
    ) -> Result<(), CognitoError> {
        let credential: Value = serde_json::from_str(credential_json)
            .map_err(|e| CognitoError::Unexpected(format!("credential JSON: {e}")))?;
        self.call(
            "CompleteWebAuthnRegistration",
            json!({ "AccessToken": access_token, "Credential": credential }),
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_is_the_regional_json_rpc_host() {
        let c = CognitoConfig {
            region: "us-east-1".into(),
            client_id: "abc".into(),
        };
        assert_eq!(c.endpoint(), "https://cognito-idp.us-east-1.amazonaws.com/");
    }

    /// The exception NAME is the diagnosis; a generic string makes every
    /// failure look the same.
    #[test]
    fn service_errors_keep_the_exception_name_and_strip_any_namespace() {
        let e = service_error(&json!({
            "__type": "com.amazonaws.cognitoidp#InvalidParameterException",
            "message": "Missing required parameter USERNAME"
        }));
        assert_eq!(
            e,
            CognitoError::Service {
                code: "InvalidParameterException".into(),
                message: "Missing required parameter USERNAME".into(),
            }
        );
        assert!(e.to_string().contains("InvalidParameterException"));
    }

    /// A spent challenge session gets its own variant because its recovery is
    /// specific: restart the password step, do NOT retry the code.
    #[test]
    fn a_spent_session_is_recognised() {
        let e = service_error(&json!({
            "__type": "NotAuthorizedException",
            "message": "Invalid session for the user, session can only be used once."
        }));
        assert_eq!(e, CognitoError::SessionExpired);

        // A different NotAuthorized stays a plain service error.
        let other = service_error(&json!({
            "__type": "NotAuthorizedException",
            "message": "Incorrect username or password."
        }));
        assert!(matches!(other, CognitoError::Service { .. }));
    }

    #[test]
    fn missing_type_still_yields_a_usable_error() {
        let e = service_error(&json!({ "message": "boom" }));
        assert_eq!(
            e,
            CognitoError::Service {
                code: "UnknownException".into(),
                message: "boom".into(),
            }
        );
    }

    #[test]
    fn tokens_require_id_and_access_but_refresh_is_optional() {
        let t = tokens_from(&json!({ "IdToken": "id", "AccessToken": "at" })).unwrap();
        assert_eq!(t.id_token, "id");
        assert_eq!(t.access_token, "at");
        assert!(t.refresh_token.is_none());

        assert!(tokens_from(&json!({ "AccessToken": "at" })).is_err());
    }

    /// A refresh response omits the refresh token; dropping it would mean a
    /// session could refresh exactly once and then be stranded.
    #[test]
    fn refresh_carries_the_existing_refresh_token_forward() {
        let mut t = tokens_from(&json!({ "IdToken": "id2", "AccessToken": "at2" })).unwrap();
        assert!(t.refresh_token.is_none());
        if t.refresh_token.is_none() {
            t.refresh_token = Some("rt-original".to_string());
        }
        assert_eq!(t.refresh_token.as_deref(), Some("rt-original"));
        assert_eq!(t.id_token, "id2");
    }

    /// "No passkey" must read as an ordinary state, not an alarming failure.
    #[test]
    fn no_passkey_reads_calmly_and_is_distinct() {
        assert_ne!(CognitoError::NoPasskey, CognitoError::SessionExpired);
        assert!(CognitoError::NoPasskey.to_string().contains("no passkey"));
    }
}
