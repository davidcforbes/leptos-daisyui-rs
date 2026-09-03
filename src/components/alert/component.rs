use super::style::{AlertColor, AlertDirection, AlertLiveness, AlertStyle};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Alert Component
///
/// A reactive Leptos wrapper for daisyUI's alert component that displays important messages,
/// notifications, and contextual feedback to users.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("alert alert-outline alert-dash alert-soft alert-info alert-success alert-warning alert-error alert-vertical alert-horizontal");
/// ```
///
/// ## Node References
/// - `node_ref` - References the top `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Alert(
    /// Visual style of the alert
    #[prop(optional, into)]
    style: Signal<AlertStyle>,

    /// Semantic color of the alert
    #[prop(optional, into)]
    color: Signal<AlertColor>,

    /// Layout direction of alert content
    #[prop(optional, into)]
    direction: Signal<AlertDirection>,

    /// Node reference for the alert element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// How assistive technology treats this content (`ldui-fmiu`). Defaults to
    /// [`AlertLiveness::Assertive`] — `role="alert"` — so existing call sites
    /// are unchanged. Use [`AlertLiveness::Static`] for a permanent panel that
    /// only wants the visual treatment.
    #[prop(optional, into)]
    liveness: Signal<AlertLiveness>,

    /// Alert content (text, icons, buttons, or other elements)
    children: Children,
) -> impl IntoView {
    view! {
        <div
            // `ldui-fmiu`: was a hardcoded role="alert", whose implicit
            // aria-live="assertive" interrupts a screen-reader user. Correct
            // for a transient message, wrong for a permanent panel -- and this
            // component is used for both because it owns the visual. `Static`
            // emits no role at all, because a live region that never changes
            // has nothing to announce.
            role=move || liveness.get().role()
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "alert",
                    style.get().as_str(),
                    color.get().as_str(),
                    direction.get().as_str(),
                    class
                )
            }
        >
            {children()}
        </div>
    }
}
