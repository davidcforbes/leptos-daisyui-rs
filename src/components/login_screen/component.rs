use super::style::{
    LoginState, login_password_autocomplete, login_password_label, login_submit_label, password_ok,
    password_rules,
};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// A centered sign-in card: username + password, an optional one-time-code
/// step, and a passkey / Windows Hello action.
///
/// Ported from d2d-ui's owner-drawn `LoginScreen` control so the desktop and
/// web sign-in surfaces share one vocabulary ([`LoginState`]) and one shape.
/// The Direct2D `layout()`/`view()`/`hit()` triple becomes a daisyUI `card`
/// with real form controls.
///
/// ## The component is UI + intent only
///
/// It performs NO authentication. The host owns every identity-provider call
/// and pushes the outcome back down through `state`; the component just reports
/// what the user asked for (`on_password_submit`, `on_mfa_submit`,
/// `on_passkey`, ...). That keeps it reusable across apps and IdPs — the same
/// screen serves Cognito, Entra, or a bespoke API.
///
/// ## Why the fields are real `<input>`s
///
/// Unlike a natively-drawn desktop field, these carry `autocomplete` hints, so
/// **password managers and browser autofill work**, and Tab order, IME,
/// accessibility and clipboard all come from the platform rather than being
/// re-implemented.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{LoginScreen, LoginState};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let (state, set_state) = signal(LoginState::EnterPassword);
///     view! {
///         <LoginScreen
///             state=state
///             app_name="FLOW"
///             on_password_submit=Callback::new(move |(_user, _pass): (String, String)| {
///                 // host: call the IdP, then push the outcome back:
///                 set_state.set(LoginState::Authenticating);
///             })
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex min-h-screen items-center justify-center bg-base-200 p-4");
/// @source inline("card w-full max-w-sm bg-base-100 shadow-xl");
/// @source inline("card-body gap-4");
/// @source inline("fieldset floating-label form-control w-full");
/// @source inline("input input-bordered w-full");
/// @source inline("btn btn-primary btn-outline btn-ghost btn-block btn-sm");
/// @source inline("alert alert-error text-sm");
/// @source inline("loading loading-spinner loading-sm");
/// @source inline("divider text-xs opacity-60");
/// @source inline("text-2xl font-bold text-center opacity-70 text-sm");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn LoginScreen(
    /// Where the sign-in currently stands. Host-owned: the component never
    /// transitions itself.
    #[prop(into)]
    state: Signal<LoginState>,

    /// Product name shown as the card's heading.
    #[prop(into, default = "Sign in".to_string())]
    app_name: String,

    /// Supporting line under the heading.
    #[prop(into, optional)]
    subtitle: Option<String>,

    /// Whether a passkey / Windows Hello credential is usable here.
    ///
    /// When false the passkey action is DISABLED with an explanatory caption:
    /// offering a sign-in that cannot succeed before enrollment is a dead end
    /// (and it is the button a first-run user reaches for). Defaults to
    /// `false` — a host that knows a credential exists opts in.
    #[prop(into, default = Signal::derive(|| false))]
    hello_available: Signal<bool>,

    /// Prefill for the username field, e.g. the last account to sign in
    /// successfully. Applied once, on mount.
    #[prop(into, optional)]
    initial_username: Option<String>,

    /// "Sign in" pressed: `(username, password)`.
    #[prop(into, optional)]
    on_password_submit: Option<Callback<(String, String)>>,

    /// "Verify" pressed with the one-time code.
    #[prop(into, optional)]
    on_mfa_submit: Option<Callback<String>>,

    /// The passkey / Windows Hello action was pressed. The host runs the
    /// WebAuthn ceremony (see [`crate::auth::cognito_webauthn`] for a Cognito
    /// implementation).
    #[prop(into, optional)]
    on_passkey: Option<Callback<()>>,

    /// "Create password" pressed on the [`LoginState::SetNewPassword`] step: the
    /// new password (already policy-checked by the component).
    #[prop(into, optional)]
    on_new_password_submit: Option<Callback<String>>,

    /// The base32 TOTP secret for the [`LoginState::SetUpMfa`] step (from
    /// `AssociateSoftwareToken`). When present, the QR + manual key are shown.
    #[prop(into, default = Signal::derive(|| None))]
    mfa_secret: Signal<Option<String>>,

    /// The account label for the `otpauth://` URI (usually the email), so the
    /// entry in the user's authenticator app is recognisable. Reactive because
    /// the email is only known after the username step.
    #[prop(into, default = Signal::derive(|| "account".to_string()))]
    mfa_account: Signal<String>,

    /// "Set up" pressed on the [`LoginState::OfferPasskey`] step.
    #[prop(into, optional)]
    on_setup_passkey: Option<Callback<()>>,

    /// "Skip" pressed on the [`LoginState::OfferPasskey`] step.
    #[prop(into, optional)]
    on_skip_passkey: Option<Callback<()>>,

    /// Optional escape hatch, e.g. "use the hosted sign-in page instead".
    /// Omitted entirely when not supplied.
    #[prop(into, optional)]
    on_fallback: Option<Callback<()>>,

    /// Label for the fallback link.
    #[prop(into, default = "Trouble signing in? Use another method".to_string())]
    fallback_label: String,

    /// Additional classes for the outer wrapper.
    #[prop(into, optional)]
    class: String,

    /// Reference to the wrapping `div`.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let (username, set_username) = signal(initial_username.unwrap_or_default());
    let (password, set_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (code, set_code) = signal(String::new());

    let busy = Signal::derive(move || state.get().is_busy());

    // A single submit path so Enter (form submit) and the button behave
    // identically — a login where Enter does nothing is a classic annoyance.
    let submit = move || {
        let s = state.get();
        if s.is_busy() {
            return;
        }
        if s.shows_new_password() {
            // Only submit a policy-compliant password (the checklist + disabled
            // button already gate it; guard here too).
            let p = new_password.get();
            if let Some(cb) = on_new_password_submit.filter(|_| password_ok(&p)) {
                cb.run(p);
            }
        } else if s.shows_mfa() || s.shows_mfa_setup() {
            // The returning-user MFA code AND the first-time setup code share one
            // callback; the host branches on the state it owns.
            let c = code.get().trim().to_string();
            if c.is_empty() {
                return;
            }
            if let Some(cb) = on_mfa_submit {
                cb.run(c);
            }
        } else {
            let u = username.get().trim().to_string();
            // Never trim a password: a trailing space is part of the secret.
            let p = password.get();
            if u.is_empty() || p.is_empty() {
                return;
            }
            if let Some(cb) = on_password_submit {
                cb.run((u, p));
            }
        }
    };

    view! {
        <div
            node_ref=node_ref
            class=merge_classes!(
                "flex min-h-screen items-center justify-center bg-base-200 p-4",
                class
            )
        >
            <div class="card w-full max-w-sm bg-base-100 shadow-xl">
                <div class="card-body gap-4">
                    <h1 class="text-2xl font-bold text-center">{app_name}</h1>
                    {subtitle
                        .map(|s| view! { <p class="text-center opacity-70 text-sm">{s}</p> })}

                    <Show when=move || state.get().error_message().is_some()>
                        <div role="alert" class="alert alert-error text-sm">
                            {move || state.get().error_message().unwrap_or_default().to_string()}
                        </div>
                    </Show>

                    <form on:submit=move |ev| {
                        ev.prevent_default();
                        submit();
                    }>
                        <Show when=move || state.get().shows_credentials()>
                            <label class="form-control w-full">
                                <span class="label-text">"Username"</span>
                                <input
                                    type="text"
                                    class="input input-bordered w-full"
                                    autocomplete="username"
                                    autocapitalize="none"
                                    spellcheck="false"
                                    disabled=move || busy.get()
                                    prop:value=move || username.get()
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="form-control w-full">
                                <span class="label-text">
                                    {move || login_password_label(&state.get())}
                                </span>
                                <input
                                    type="password"
                                    class="input input-bordered w-full"
                                    autocomplete=move || login_password_autocomplete(&state.get())
                                    disabled=move || busy.get()
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                />
                            </label>
                        </Show>

                        <Show when=move || state.get().shows_mfa()>
                            <label class="form-control w-full">
                                <span class="label-text">"Authenticator code"</span>
                                <input
                                    type="text"
                                    class="input input-bordered w-full"
                                    // Lets the browser / OS offer the code from
                                    // an SMS or authenticator app.
                                    autocomplete="one-time-code"
                                    inputmode="numeric"
                                    disabled=move || busy.get()
                                    prop:value=move || code.get()
                                    on:input=move |ev| set_code.set(event_target_value(&ev))
                                />
                            </label>
                        </Show>

                        // Dedicated create-new-password step: single field + a
                        // live requirements checklist (each rule greens as met).
                        <Show when=move || state.get().shows_new_password()>
                            <p class="text-sm opacity-70">
                                {move || format!("for {}", username.get())}
                            </p>
                            <label class="form-control w-full">
                                <span class="label-text">"New password"</span>
                                <input
                                    type="password"
                                    class="input input-bordered w-full"
                                    autocomplete="new-password"
                                    disabled=move || busy.get()
                                    prop:value=move || new_password.get()
                                    on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                />
                            </label>
                            <ul class="text-xs mt-1 space-y-0.5">
                                <For
                                    each=move || {
                                        password_rules(&new_password.get())
                                            .into_iter()
                                            .enumerate()
                                            .collect::<Vec<_>>()
                                    }
                                    key=|(i, _)| *i
                                    let:item
                                >
                                    <li class=move || {
                                        if item.1.1 { "text-success" } else { "opacity-60" }
                                    }>{if item.1.1 { "\u{2713} " } else { "\u{25CB} " }}{item.1.0}</li>
                                </For>
                            </ul>
                        </Show>

                        // First-time TOTP setup: QR + manual key + the code field.
                        <Show when=move || state.get().shows_mfa_setup()>
                            {
                                move || {
                                    mfa_secret
                                        .get()
                                        .map(|secret| {
                                            let account = mfa_account.get();
                                            let uri = super::style::otpauth_uri(
                                                "AWS SSM Monitor",
                                                &account,
                                                &secret,
                                            );
                                            let svg = super::style::qr_svg(&uri);
                                            let key = super::style::group_key(&secret);
                                            view! {
                                                <div class="flex flex-col items-center gap-2">
                                                    <div
                                                        class="bg-white p-2 rounded"
                                                        inner_html=svg
                                                    ></div>
                                                    <p class="text-xs opacity-70">
                                                        "Can't scan? Enter this key:"
                                                    </p>
                                                    <code class="text-xs break-all text-center">
                                                        {key}
                                                    </code>
                                                </div>
                                            }
                                        })
                                }
                            }
                            <label class="form-control w-full mt-2">
                                <span class="label-text">"Authenticator code"</span>
                                <input
                                    type="text"
                                    class="input input-bordered w-full"
                                    autocomplete="one-time-code"
                                    inputmode="numeric"
                                    disabled=move || busy.get()
                                    prop:value=move || code.get()
                                    on:input=move |ev| set_code.set(event_target_value(&ev))
                                />
                            </label>
                        </Show>

                        <Show when=move || {
                            let s = state.get();
                            s.shows_credentials()
                                || s.shows_mfa()
                                || s.shows_new_password()
                                || s.shows_mfa_setup()
                        }>
                            <button
                                type="submit"
                                class="btn btn-primary btn-block mt-2"
                                disabled=move || {
                                    busy.get()
                                        || (state.get().shows_new_password()
                                            && !password_ok(&new_password.get()))
                                }
                            >
                                <Show when=move || busy.get()>
                                    <span class="loading loading-spinner loading-sm"></span>
                                </Show>
                                {move || login_submit_label(&state.get())}
                            </button>
                        </Show>
                    </form>

                    // Post-sign-in passkey offer: set up / skip.
                    <Show when=move || state.get().shows_passkey_offer()>
                        <button
                            type="button"
                            class="btn btn-primary btn-block"
                            disabled=move || busy.get()
                            on:click=move |_| {
                                if let Some(cb) = on_setup_passkey {
                                    cb.run(());
                                }
                            }
                        >
                            "Set up a passkey"
                        </button>
                        <button
                            type="button"
                            class="btn btn-ghost btn-block"
                            disabled=move || busy.get()
                            on:click=move |_| {
                                if let Some(cb) = on_skip_passkey {
                                    cb.run(());
                                }
                            }
                        >
                            "Skip"
                        </button>
                    </Show>

                    {on_passkey
                        .map(|cb| {
                            view! {
                                <div class="divider text-xs opacity-60">"or"</div>
                                <button
                                    type="button"
                                    class="btn btn-outline btn-block"
                                    disabled=move || busy.get() || !hello_available.get()
                                    on:click=move |_| cb.run(())
                                >
                                    "Sign in with a passkey"
                                </button>
                                <Show when=move || !hello_available.get()>
                                    <p class="text-xs text-center opacity-70">
                                        "Available after your first password sign-in."
                                    </p>
                                </Show>
                            }
                        })}

                    {on_fallback
                        .map(|cb| {
                            view! {
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-sm btn-block"
                                    disabled=move || busy.get()
                                    on:click=move |_| cb.run(())
                                >
                                    {fallback_label}
                                </button>
                            }
                        })}
                </div>
            </div>
        </div>
    }
}
