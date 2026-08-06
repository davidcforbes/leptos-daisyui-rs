use super::style::{ButtonColor, ButtonShape, ButtonSize, ButtonStyle};
use crate::merge_classes;
use crate::utils::{RippleOverlay, use_ripple};
use leptos::{
    html::{A, Button as HTMLButton},
    prelude::*,
};

/// # Button Component
///
/// A reactive wrapper for daisyUI's button component with comprehensive styling options
/// including colors, sizes, shapes, and interactive states.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("btn btn-neutral btn-primary btn-secondary btn-accent btn-info btn-success btn-warning btn-error btn-outline btn-dash btn-soft btn-ghost btn-link btn-xs btn-sm btn-md btn-lg btn-xl btn-wide btn-block btn-square btn-circle btn-active btn-disabled loading");
/// ```
///
/// ## Node References
/// - `node_ref` - References the `<button>` element ([HTMLButtonElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement))
#[component]
pub fn Button(
    /// Shows loading spinner when true
    #[prop(optional, into)]
    loading: Signal<bool>,

    /// Button color variant
    #[prop(optional, into)]
    color: Signal<ButtonColor>,

    /// Button visual style
    #[prop(optional, into)]
    style: Signal<ButtonStyle>,

    /// Button size variant
    #[prop(optional, into)]
    size: Signal<ButtonSize>,

    /// Button shape/layout modifier
    #[prop(optional, into)]
    shape: Signal<ButtonShape>,

    /// Whether the button appears in active state
    #[prop(optional, into)]
    active: Signal<bool>,

    /// Whether the button is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Enables the click ripple effect. Defaults to off.
    #[prop(optional, into)]
    ripple: Signal<bool>,

    /// Node reference for the button element
    #[prop(optional)]
    node_ref: NodeRef<HTMLButton>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Button content (text, icons, or other elements)
    children: Children,
) -> impl IntoView {
    let ripple_handle = use_ripple();
    view! {
        <button
            disabled=disabled
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "btn ld-eased ld-pressable ld-focus-ring",
                    color.get().as_str(),
                    style.get().as_str(),
                    size.get().as_str(),
                    shape.get().as_str(),
                    class
                )
            }

            class:btn-active=active
            class:btn-disabled=disabled
            class:loading=loading
            class:ld-ripple-host=ripple
            on:click=move |ev| {
                if ripple.get_untracked() {
                    ripple_handle.trigger.run(ev);
                }
            }
        >
            {children()}
            {move || ripple.get().then(|| view! { <RippleOverlay handle=ripple_handle /> })}
        </button>
    }
}

/// # Link Button Component
///
/// An anchor element styled as a daisyUI button for navigation actions.
/// Provides the same styling options as Button but renders as a link.
///
/// ## Node References
/// - `node_ref` - References the `<a>` element ([HTMLAnchorElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement))
#[component]
pub fn LinkButton(
    /// URL to navigate to when clicked. Accepts static strings, owned `String`s
    /// (e.g. a per-row route like `format!("/matter/{case_no}")`), or reactive
    /// signals. Rendered verbatim as the `<a href>`: do not interpolate
    /// untrusted input (a `javascript:` scheme would execute), same contract
    /// as `MenuItem`/`BreadcrumbItem` hrefs.
    #[prop(optional, into)]
    href: MaybeProp<String>,

    /// Button color variant (same as Button component)
    #[prop(optional, into)]
    color: Signal<ButtonColor>,

    /// Button visual style (same as Button component)
    #[prop(optional, into)]
    style: Signal<ButtonStyle>,

    /// Button size variant (same as Button component)
    #[prop(optional, into)]
    size: Signal<ButtonSize>,

    /// Button shape/layout modifier (same as Button component)
    #[prop(optional, into)]
    shape: Signal<ButtonShape>,

    /// Enables the click ripple effect. Defaults to off.
    #[prop(optional, into)]
    ripple: Signal<bool>,

    /// Node reference for the anchor element
    #[prop(optional)]
    node_ref: NodeRef<A>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Link content (text, icons, or other elements)
    children: Children,
) -> impl IntoView {
    let ripple_handle = use_ripple();
    view! {
        <a
            href=move || href.get()
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "btn ld-eased ld-pressable ld-focus-ring",
                    color.get().as_str(),
                    style.get().as_str(),
                    size.get().as_str(),
                    shape.get().as_str(),
                    class
                )
            }
            class:ld-ripple-host=ripple
            on:click=move |ev| {
                if ripple.get_untracked() {
                    ripple_handle.trigger.run(ev);
                }
            }
        >

            {children()}
            {move || ripple.get().then(|| view! { <RippleOverlay handle=ripple_handle /> })}
        </a>
    }
}
