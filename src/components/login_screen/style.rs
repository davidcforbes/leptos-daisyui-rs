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
}

/// The label for the primary submit button in a given state.
pub fn login_submit_label(state: &LoginState) -> &'static str {
    match state {
        LoginState::EnterMfaCode => "Verify",
        LoginState::EnterTempPassword => "Continue",
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
