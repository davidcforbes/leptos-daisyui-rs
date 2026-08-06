use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::auth::{CognitoClient, CognitoConfig, SignInOutcome};
use leptos_daisyui_rs::components::{LoginProvider, LoginScreen, LoginState, ProviderLoginScreen};
use leptos_daisyui_rs::utils::create_credential;
use wasm_bindgen_futures::spawn_local;

/// A public SPA app client in the office-perf pool (`us-east-1_6OfldylII`) — the
/// SAME pool the desktop app enrolls against, so a user provisioned there can be
/// tested end-to-end here. Public client, no secret.
const REGION: &str = "us-east-1";
const CLIENT_ID: &str = "40hna4voji12ch2g0ot8s6vctv";

/// Live end-to-end Cognito install-enrollment against the real pool.
///
/// Drives the reusable [`LoginScreen`] through the full ceremony — sign in with a
/// temporary password → create a new password → set up TOTP (scan the QR) →
/// verify the code → offer a passkey — by wiring each callback to the
/// [`CognitoClient`]. The component owns none of this; the host (this page) does,
/// exactly as a real app would.
///
/// Passkey note: browser WebAuthn requires the pool's `RelyingPartyId` to match
/// the page origin. On `localhost` the "Set up a passkey" step only succeeds if
/// the pool RP is configured for `localhost`; otherwise the password + TOTP path
/// is what this demo exercises.
#[component]
pub fn LoginEnrollmentDemo() -> impl IntoView {
    let client = StoredValue::new(CognitoClient::new(CognitoConfig {
        region: REGION.to_string(),
        client_id: CLIENT_ID.to_string(),
    }));
    let (state, set_state) = signal(LoginState::EnterTempPassword);
    let (mfa_secret, set_mfa_secret) = signal(None::<String>);
    // Carried across steps — a challenge `Session` is single-use per response, so
    // each step overwrites it with the next one.
    let username = StoredValue::new(String::new());
    let session = StoredValue::new(String::new());
    let access_token = StoredValue::new(String::new());

    // Route a sign-in / challenge outcome to the next UI step. All captures are
    // `Copy` (StoredValue + signals), so this closure is reused across callbacks.
    let apply_outcome = move |outcome: SignInOutcome| match outcome {
        SignInOutcome::Tokens(t) => {
            access_token.set_value(t.access_token);
            set_state.set(LoginState::OfferPasskey);
        }
        SignInOutcome::NewPasswordRequired { session: s } => {
            session.set_value(s);
            set_state.set(LoginState::SetNewPassword);
        }
        SignInOutcome::MfaSetupRequired { session: s } => {
            spawn_local(async move {
                match client.get_value().begin_totp_setup(&s).await {
                    Ok((secret, next)) => {
                        session.set_value(next);
                        set_mfa_secret.set(Some(secret));
                        set_state.set(LoginState::SetUpMfa);
                    }
                    Err(e) => set_state.set(LoginState::Error(e.to_string())),
                }
            });
        }
        SignInOutcome::MfaRequired { session: s } => {
            session.set_value(s);
            set_state.set(LoginState::EnterMfaCode);
        }
    };

    let on_password_submit = Callback::new(move |(u, p): (String, String)| {
        username.set_value(u.clone());
        set_state.set(LoginState::Authenticating);
        spawn_local(async move {
            match client.get_value().sign_in_with_password(&u, &p).await {
                Ok(o) => apply_outcome(o),
                Err(e) => set_state.set(LoginState::Error(e.to_string())),
            }
        });
    });

    let on_new_password_submit = Callback::new(move |p: String| {
        set_state.set(LoginState::Authenticating);
        spawn_local(async move {
            let u = username.get_value();
            let s = session.get_value();
            match client.get_value().respond_new_password(&u, &s, &p).await {
                Ok(o) => apply_outcome(o),
                Err(e) => set_state.set(LoginState::Error(e.to_string())),
            }
        });
    });

    // One callback for BOTH the returning-user code and the first-time setup code;
    // branch on the state we own (matches the component's single on_mfa_submit).
    let on_mfa_submit = Callback::new(move |code: String| {
        let is_setup = state.get_untracked().shows_mfa_setup();
        set_state.set(LoginState::Authenticating);
        spawn_local(async move {
            let u = username.get_value();
            let s = session.get_value();
            let c = client.get_value();
            let result = if is_setup {
                match c.verify_totp_setup(&s, &code).await {
                    Ok(next) => c.respond_mfa_setup(&u, &next).await,
                    Err(e) => Err(e),
                }
            } else {
                c.respond_mfa(&u, &s, &code).await
            };
            match result {
                Ok(t) => {
                    access_token.set_value(t.access_token);
                    set_state.set(LoginState::OfferPasskey);
                }
                Err(e) => set_state.set(LoginState::Error(e.to_string())),
            }
        });
    });

    let on_setup_passkey = Callback::new(move |_| {
        set_state.set(LoginState::Authenticating);
        spawn_local(async move {
            let c = client.get_value();
            let at = access_token.get_value();
            let outcome = async {
                let opts = c
                    .start_passkey_registration(&at)
                    .await
                    .map_err(|e| e.to_string())?;
                let resp = create_credential(&opts).await.map_err(|e| e.to_string())?;
                c.complete_passkey_registration(&at, &resp)
                    .await
                    .map_err(|e| e.to_string())
            }
            .await;
            match outcome {
                Ok(()) => set_state.set(LoginState::Succeeded),
                Err(e) => set_state.set(LoginState::Error(e)),
            }
        });
    });

    let on_skip_passkey = Callback::new(move |_| set_state.set(LoginState::Succeeded));

    let account = Signal::derive(move || username.get_value());

    view! {
        <ContentLayout
            title="Login / Enrollment"
            description="End-to-end Cognito install-enrollment against the real office-perf pool. Enter an email + temporary password to walk new-password -> TOTP setup -> passkey."
        >
            <Section title="Enrollment ceremony" col=true>
                <Show
                    when=move || matches!(state.get(), LoginState::Succeeded)
                    fallback=move || {
                        view! {
                            <LoginScreen
                                state=state
                                app_name="AWS SSM Monitor"
                                subtitle="Set up AWS access"
                                mfa_secret=mfa_secret
                                mfa_account=account
                                on_password_submit=on_password_submit
                                on_new_password_submit=on_new_password_submit
                                on_mfa_submit=on_mfa_submit
                                on_setup_passkey=on_setup_passkey
                                on_skip_passkey=on_skip_passkey
                            />
                        }
                    }
                >
                    <div class="alert alert-success">
                        "Signed in. Enrollment complete."
                    </div>
                </Show>
            </Section>

            <Section title="Provider login shell (ProviderLoginScreen)" col=true>
                <p class="text-sm opacity-70 mb-2">
                    "For hosts whose IdP flow is a server redirect: no credential state, just a "
                    "branded card with one button per provider. A provider with an "
                    <code>"href"</code> " renders as a link (navigation starts the OAuth flow); "
                    "one without reports its id through " <code>"on_provider"</code> "."
                </p>
                <ProviderLoginDemo />
            </Section>
        </ContentLayout>
    }
}

#[component]
fn ProviderLoginDemo() -> impl IntoView {
    let last_provider = RwSignal::new(String::new());
    let providers = Signal::derive(|| {
        vec![
            LoginProvider::new("zoho", "Sign in with Zoho")
                .with_href("#zoho-redirect")
                .with_icon("log-in"),
            LoginProvider::new("google", "Sign in with Google")
                .with_icon("chrome")
                .with_style_class("btn-outline"),
        ]
    });
    view! {
        <p class="text-sm mb-2">
            "Last scripted provider: "
            <code data-testid="provider-clicked">
                {move || {
                    let p = last_provider.get();
                    if p.is_empty() { "(none)".to_string() } else { p }
                }}
            </code>
        </p>
        <div class="w-full max-w-xl overflow-hidden rounded-box border border-base-300" id="provider-login">
            <ProviderLoginScreen
                app_name="Office Performance"
                subtitle="Use your Zoho account to continue"
                providers=providers
                brand=ViewFn::from(|| view! {
                    <span class="text-4xl" aria-hidden="true">"🏛️"</span>
                })
                on_provider=Callback::new(move |id: &'static str| {
                    last_provider.set(id.to_string());
                })
                // The demo page embeds the shell rather than routing to it.
                full_screen=false
                class="p-6"
            >
                <p class="text-center text-xs opacity-60">
                    "Trouble signing in? Contact your administrator."
                </p>
            </ProviderLoginScreen>
        </div>
    }
}
