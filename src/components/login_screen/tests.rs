use super::provider::LoginProvider;
use super::style::{
    LoginState, login_password_autocomplete, login_password_label, login_submit_label,
};

#[test]
fn state_names_are_stable_and_distinct() {
    let all = [
        LoginState::SignedOut,
        LoginState::EnterPassword,
        LoginState::EnterMfaCode,
        LoginState::Authenticating,
        LoginState::NeedsEnrollment,
        LoginState::EnterTempPassword,
        LoginState::Enrolling,
        LoginState::Error("x".into()),
        LoginState::Succeeded,
    ];
    let mut names: Vec<&str> = all.iter().map(|s| s.name()).collect();
    let total = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), total, "state names must be distinct");
}

/// Busy states disable every input and action — this is what stops a double
/// submit from spending a single-use IdP challenge twice.
#[test]
fn only_in_flight_states_are_busy() {
    assert!(LoginState::Authenticating.is_busy());
    assert!(LoginState::Enrolling.is_busy());
    assert!(!LoginState::EnterPassword.is_busy());
    assert!(!LoginState::EnterMfaCode.is_busy());
    assert!(!LoginState::SignedOut.is_busy());
}

#[test]
fn each_step_shows_only_its_own_fields() {
    assert!(LoginState::EnterPassword.shows_credentials());
    assert!(LoginState::EnterTempPassword.shows_credentials());
    assert!(!LoginState::EnterPassword.shows_mfa());

    assert!(LoginState::EnterMfaCode.shows_mfa());
    assert!(!LoginState::EnterMfaCode.shows_credentials());

    assert!(!LoginState::SignedOut.shows_credentials());
    assert!(!LoginState::SignedOut.shows_mfa());
}

#[test]
fn error_message_is_readable_only_in_the_error_state() {
    assert_eq!(
        LoginState::Error("bad code".into()).error_message(),
        Some("bad code")
    );
    assert_eq!(LoginState::EnterPassword.error_message(), None);
}

#[test]
fn labels_follow_the_step() {
    assert_eq!(login_submit_label(&LoginState::EnterMfaCode), "Verify");
    assert_eq!(
        login_submit_label(&LoginState::EnterTempPassword),
        "Continue"
    );
    assert_eq!(login_submit_label(&LoginState::EnterPassword), "Sign in");

    assert_eq!(
        login_password_label(&LoginState::EnterTempPassword),
        "Temporary password"
    );
    assert_eq!(login_password_label(&LoginState::EnterPassword), "Password");
}

/// The autocomplete hint is what makes password managers work at all — and a
/// TEMPORARY password must not be offered for saving as the account password.
#[test]
fn password_autocomplete_distinguishes_a_temporary_password() {
    assert_eq!(
        login_password_autocomplete(&LoginState::EnterPassword),
        "current-password"
    );
    assert_eq!(
        login_password_autocomplete(&LoginState::EnterTempPassword),
        "one-time-code"
    );
}

// ---------------------------------------------------------------------
// LoginProvider (ProviderLoginScreen)
// ---------------------------------------------------------------------

#[test]
fn login_provider_defaults_to_primary_button() {
    let p = LoginProvider::new("zoho", "Sign in with Zoho");
    assert_eq!(p.id, "zoho");
    assert_eq!(p.href, None);
    assert_eq!(p.icon, None);
    let class = p.button_class(false);
    assert!(class.contains("btn"));
    assert!(class.contains("btn-block"));
    assert!(class.contains("btn-primary"));
    assert!(!class.contains("btn-disabled"));
}

#[test]
fn login_provider_builder_sets_href_icon_and_style() {
    let p = LoginProvider::new("zoho", "Sign in with Zoho")
        .with_href("/auth/zoho")
        .with_icon("log-in")
        .with_style_class("btn-outline");
    assert_eq!(p.href.as_deref(), Some("/auth/zoho"));
    assert_eq!(p.icon.as_deref(), Some("log-in"));
    assert!(p.button_class(false).contains("btn-outline"));
    assert!(!p.button_class(false).contains("btn-primary"));
}

#[test]
fn login_provider_busy_disables_the_button() {
    let p = LoginProvider::new("zoho", "Zoho");
    assert!(p.button_class(true).contains("btn-disabled"));
}

#[test]
fn login_provider_label_accepts_owned_localized_text() {
    let translated = String::from("Iniciar sesión con Zoho");
    let p = LoginProvider::new("zoho", translated);
    assert_eq!(p.label, "Iniciar sesión con Zoho");
}
