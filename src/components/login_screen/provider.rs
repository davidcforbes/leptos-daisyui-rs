//! Branded provider-login shell (`ProviderLoginScreen`).
//!
//! [`super::LoginScreen`] is deliberately a credential/MFA/passkey state
//! machine; a plain server-redirect OAuth landing ("Sign in with Zoho")
//! cannot use it without pretending credential state exists. Consumers were
//! therefore hand-rolling the entire screen — background, brand card, error
//! message and provider button — in inline HTML/CSS (the office-perf
//! production login). This component is the shared shell: brand slot,
//! heading, error surface, and one button (or link) per identity provider.
//! The host keeps its own assets and redirect endpoints; the shell keeps the
//! layout, theming, busy state and accessibility consistent across apps.

use crate::components::icon::Icon;
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// One identity provider offered on a [`ProviderLoginScreen`].
///
/// A provider with an `href` renders as a link styled as a button — the
/// server-redirect OAuth shape (`<a href="/auth/zoho">`), where navigation
/// itself starts the flow. One without renders as a `<button>` reporting its
/// `id` through `on_provider`, for hosts that start the flow from script.
#[derive(Clone, Debug, PartialEq)]
pub struct LoginProvider {
    /// Stable identifier reported through `on_provider` (e.g. `"zoho"`).
    pub id: &'static str,
    /// Button label (e.g. `"Sign in with Zoho"`). Owned, so it can come from
    /// a runtime localization lookup.
    pub label: String,
    /// Server-redirect target. `Some` renders `<a href>`; `None` renders a
    /// `<button>` firing `on_provider`.
    pub href: Option<String>,
    /// Optional Lucide icon name drawn before the label.
    pub icon: Option<String>,
    /// daisyUI button style classes (default `"btn-primary"`); e.g.
    /// `"btn-outline"` for secondary providers.
    pub style_class: &'static str,
}

impl LoginProvider {
    /// A provider button with the default (`btn-primary`) styling.
    pub fn new(id: &'static str, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            href: None,
            icon: None,
            style_class: "btn-primary",
        }
    }

    /// Render as a server-redirect link to `href`.
    pub fn with_href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    /// Draw a Lucide icon before the label.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Override the button style classes (default `"btn-primary"`).
    pub fn with_style_class(mut self, style_class: &'static str) -> Self {
        self.style_class = style_class;
        self
    }

    /// Full class list for this provider's button/link.
    pub fn button_class(&self, busy: bool) -> String {
        merge_classes!(
            "btn btn-block",
            self.style_class,
            if busy { "btn-disabled" } else { "" }
        )
        .to_class()
    }
}

/// # Provider Login Screen
///
/// A branded OAuth/provider landing: centered card with a brand slot,
/// product heading, optional subtitle, an error surface, and one button per
/// [`LoginProvider`]. No credential state — for username/password/MFA use
/// [`super::LoginScreen`]; this shell is for hosts whose IdP flow is a
/// redirect (or a scripted popup) and starts from a single button.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{LoginProvider, ProviderLoginScreen};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let providers = Signal::derive(|| {
///         vec![
///             LoginProvider::new("zoho", "Sign in with Zoho")
///                 .with_href("/auth/zoho")
///                 .with_icon("log-in"),
///         ]
///     });
///     view! {
///         <ProviderLoginScreen
///             app_name="Office Performance"
///             subtitle="Use your Zoho account to continue"
///             providers=providers
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex min-h-screen items-center justify-center bg-base-200 p-4");
/// @source inline("card w-full max-w-sm bg-base-100 shadow-xl card-body gap-6");
/// @source inline("btn btn-block btn-primary btn-outline btn-disabled");
/// @source inline("alert alert-error text-sm loading loading-spinner loading-sm");
/// @source inline("text-2xl font-bold text-center opacity-70 flex flex-col gap-3");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ProviderLoginScreen(
    /// Product name shown as the card's heading.
    #[prop(into)]
    app_name: String,

    /// Supporting line under the heading.
    #[prop(into, optional)]
    subtitle: Option<String>,

    /// Brand slot rendered above the heading — a logo `<img>`, an SVG, any
    /// view. The host keeps its own assets; the shell only positions them.
    #[prop(optional, into)]
    brand: Option<ViewFn>,

    /// Error text shown in an `alert` with `role="alert"`. Empty = no error.
    #[prop(optional, into)]
    error: Signal<String>,

    /// Whether a sign-in is in flight: provider buttons disable and a
    /// spinner shows. (A redirect flow usually never returns to this page,
    /// but a popup/scripted flow does.)
    #[prop(optional, into)]
    busy: Signal<bool>,

    /// The identity providers to offer, in order.
    #[prop(into)]
    providers: Signal<Vec<LoginProvider>>,

    /// A provider `<button>` (one without `href`) was pressed, reported with
    /// the provider's `id`.
    #[prop(optional, into)]
    on_provider: Option<Callback<&'static str>>,

    /// Whether the wrapper claims the full viewport height (`min-h-screen`,
    /// the default). Pass `false` when embedding the shell inside another
    /// layout (a demo page, a modal) rather than as the route's whole screen.
    #[prop(optional, into, default = Signal::derive(|| true))]
    full_screen: Signal<bool>,

    /// Additional classes for the full-screen wrapper (e.g. a brand
    /// background override).
    #[prop(optional, into)]
    class: &'static str,

    /// Additional classes for the card.
    #[prop(optional, into)]
    card_class: &'static str,

    /// Extra content at the bottom of the card (footnote, help link, legal).
    #[prop(optional)]
    children: Option<Children>,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "flex items-center justify-center bg-base-200 p-4",
                    if full_screen.get() { "min-h-screen" } else { "" },
                    class
                ).to_class()
            }
        >
            <div class=merge_classes!("card w-full max-w-sm bg-base-100 shadow-xl", card_class)>
                <div class="card-body gap-6">
                    {brand.map(|b| view! { <div class="flex justify-center">{b.run()}</div> })}

                    <div class="text-center">
                        <h1 class="text-2xl font-bold">{app_name}</h1>
                        {subtitle.map(|s| view! { <p class="text-sm opacity-70">{s}</p> })}
                    </div>

                    {move || {
                        let msg = error.get();
                        (!msg.is_empty()).then(|| view! {
                            <div role="alert" class="alert alert-error text-sm">{msg}</div>
                        })
                    }}

                    <div class="flex flex-col gap-3">
                        <For
                            each=move || providers.get()
                            key=|p| p.id
                            children=move |p| {
                                let id = p.id;
                                let icon = p.icon.clone();
                                let label = p.label.clone();
                                let btn_class = {
                                    let p = p.clone();
                                    move || p.button_class(busy.get())
                                };
                                let inner = view! {
                                    {icon.map(|i| view! { <Icon name=i /> })}
                                    {move || busy.get().then(|| view! {
                                        <span class="loading loading-spinner loading-sm"></span>
                                    })}
                                    {label}
                                };
                                match p.href.clone() {
                                    Some(href) => view! {
                                        <a
                                            class=btn_class
                                            href=href
                                            aria-disabled=move || busy.get().then_some("true")
                                            tabindex=move || busy.get().then_some(-1)
                                        >
                                            {inner}
                                        </a>
                                    }
                                    .into_any(),
                                    None => view! {
                                        <button
                                            class=btn_class
                                            disabled=move || busy.get()
                                            on:click=move |_| {
                                                if let Some(cb) = on_provider {
                                                    cb.run(id);
                                                }
                                            }
                                        >
                                            {inner}
                                        </button>
                                    }
                                    .into_any(),
                                }
                            }
                        />
                    </div>

                    {children.map(|c| c())}
                </div>
            </div>
        </div>
    }
}
