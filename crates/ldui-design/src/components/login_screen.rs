/// Where a sign-in attempt currently stands.
///
/// The host owns this: it drives the identity-provider calls and pushes the
/// resulting state back down. The component never transitions itself, so the
/// same screen serves any IdP (Cognito, Entra, a bespoke API) without the
/// component knowing which.
///
/// Mirrors d2d-ui's `LoginState` (the desktop `LoginScreen`) so the two
/// surfaces share one vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum LoginState {
    /// Idle: offer the sign-in affordances.
    #[default]
    SignedOut,
    /// Username + password are being collected.
    EnterPassword,
    /// A one-time MFA/TOTP code is required to finish the password step.
    ///
    /// Note an IdP challenge is typically SINGLE-USE — after a failed code the
    /// host usually has to restart at [`LoginState::EnterPassword`] rather than
    /// let the user retry against a spent challenge.
    EnterMfaCode,
    /// A request is in flight; inputs and actions are disabled.
    Authenticating,
    /// The account has no credential enrolled on this device yet.
    NeedsEnrollment,
    /// Collecting a one-time/temporary password as part of enrollment.
    EnterTempPassword,
    /// Create a NEW permanent password (after an admin-created user's first
    /// sign-in). A dedicated step — distinct heading, the email read-only, and a
    /// live requirements checklist — so it is unmistakably "set a new password".
    SetNewPassword,
    /// First-time TOTP setup: show the shared secret (QR + manual key) and collect
    /// the user's first verification code.
    SetUpMfa,
    /// Offer to enroll a passkey / Windows Hello after a successful sign-in.
    OfferPasskey,
    /// Enrollment is in flight.
    Enrolling,
    /// The last attempt failed; the message is shown to the user.
    Error(String),
    /// Sign-in completed; the host is expected to navigate away.
    Succeeded,
}

impl LoginState {
    /// A stable, machine-readable name — handy for tests, telemetry and
    /// snapshot assertions.
    pub fn name(&self) -> &'static str {
        match self {
            LoginState::SignedOut => "signed_out",
            LoginState::EnterPassword => "enter_password",
            LoginState::EnterMfaCode => "enter_mfa_code",
            LoginState::Authenticating => "authenticating",
            LoginState::NeedsEnrollment => "needs_enrollment",
            LoginState::EnterTempPassword => "enter_temp_password",
            LoginState::SetNewPassword => "set_new_password",
            LoginState::SetUpMfa => "set_up_mfa",
            LoginState::OfferPasskey => "offer_passkey",
            LoginState::Enrolling => "enrolling",
            LoginState::Error(_) => "error",
            LoginState::Succeeded => "succeeded",
        }
    }

    /// Whether a request is in flight — every input and action is disabled
    /// while this is true, so a double submit cannot spend a single-use
    /// challenge twice.
    pub fn is_busy(&self) -> bool {
        matches!(self, LoginState::Authenticating | LoginState::Enrolling)
    }

    /// Whether the username + password fields are shown.
    pub fn shows_credentials(&self) -> bool {
        matches!(
            self,
            LoginState::EnterPassword | LoginState::EnterTempPassword
        )
    }

    /// Whether the one-time-code field is shown.
    pub fn shows_mfa(&self) -> bool {
        matches!(self, LoginState::EnterMfaCode)
    }

    /// The error message, if this is [`LoginState::Error`].
    pub fn error_message(&self) -> Option<&str> {
        match self {
            LoginState::Error(m) => Some(m.as_str()),
            _ => None,
        }
    }

    /// Whether the dedicated single "new password" field + checklist are shown.
    pub fn shows_new_password(&self) -> bool {
        matches!(self, LoginState::SetNewPassword)
    }

    /// Whether the first-time TOTP-setup UI (QR + key + code) is shown.
    pub fn shows_mfa_setup(&self) -> bool {
        matches!(self, LoginState::SetUpMfa)
    }

    /// Whether the post-sign-in passkey offer is shown.
    pub fn shows_passkey_offer(&self) -> bool {
        matches!(self, LoginState::OfferPasskey)
    }
}

/// The five password-policy rules and whether `pw` satisfies each — drives the
/// live requirements checklist on the create-password step. Mirrors the desktop
/// `d2d-ui` rules so the two surfaces agree.
pub fn password_rules(pw: &str) -> [(&'static str, bool); 5] {
    [
        ("At least 12 characters", pw.chars().count() >= 12),
        (
            "An uppercase letter",
            pw.chars().any(|c| c.is_ascii_uppercase()),
        ),
        (
            "A lowercase letter",
            pw.chars().any(|c| c.is_ascii_lowercase()),
        ),
        ("A number", pw.chars().any(|c| c.is_ascii_digit())),
        (
            "A symbol",
            pw.chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace()),
        ),
    ]
}

/// True when `pw` satisfies every rule in [`password_rules`].
pub fn password_ok(pw: &str) -> bool {
    password_rules(pw).iter().all(|(_, ok)| *ok)
}

/// Percent-encode one `otpauth://` label/param component (RFC 3986 unreserved
/// stays literal; everything else becomes `%XX`).
fn pct(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Build the `otpauth://totp/<issuer>:<account>?secret=…&issuer=…` URI rendered
/// as the setup QR. Mirrors the desktop builder.
pub fn otpauth_uri(issuer: &str, account: &str, secret: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        pct(issuer),
        pct(account),
        pct(secret),
        pct(issuer),
    )
}

/// Format a base32 secret in space-separated groups of four for legibility.
pub fn group_key(secret: &str) -> String {
    secret
        .as_bytes()
        .chunks(4)
        .filter_map(|c| std::str::from_utf8(c).ok())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Render `text` as an inline SVG QR code (black modules on white, 4-module quiet
/// zone) for embedding via `inner_html`. Empty string if it can't encode.
pub fn qr_svg(text: &str) -> String {
    use qrcodegen::{QrCode, QrCodeEcc};
    let Ok(qr) = QrCode::encode_text(text, QrCodeEcc::Medium) else {
        return String::new();
    };
    let n = qr.size();
    let quiet = 4;
    let dim = n + 2 * quiet;
    let mut rects = String::new();
    for y in 0..n {
        for x in 0..n {
            if qr.get_module(x, y) {
                rects.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="1" height="1"/>"#,
                    x + quiet,
                    y + quiet
                ));
            }
        }
    }
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {dim} {dim}" width="180" height="180" shape-rendering="crispEdges"><rect width="{dim}" height="{dim}" fill="#fff"/><g fill="#000">{rects}</g></svg>"##
    )
}

/// The label for the primary submit button in a given state.
pub fn login_submit_label(state: &LoginState) -> &'static str {
    match state {
        LoginState::EnterMfaCode | LoginState::SetUpMfa => "Verify",
        LoginState::EnterTempPassword => "Continue",
        LoginState::SetNewPassword => "Create password",
        _ => "Sign in",
    }
}

/// The label for the password field — an enrollment step collects a
/// *temporary* password, which is worth saying plainly.
pub fn login_password_label(state: &LoginState) -> &'static str {
    match state {
        LoginState::EnterTempPassword => "Temporary password",
        _ => "Password",
    }
}

/// The `autocomplete` value for the password field.
///
/// This is what lets a password manager (LastPass, 1Password, the browser's
/// own) recognise and fill the field — the single biggest advantage the web
/// login has over a natively-drawn desktop one, where the field is invisible to
/// such tools. A temporary/one-time password is marked `one-time-code` so a
/// manager does not offer to save it as the account password.
pub fn login_password_autocomplete(state: &LoginState) -> &'static str {
    match state {
        LoginState::EnterTempPassword => "one-time-code",
        _ => "current-password",
    }
}
