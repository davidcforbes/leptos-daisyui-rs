use super::style::{BadgeColor, BadgeSize, BadgeStyle};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Badge Component
///
/// A reactive Leptos wrapper for daisyUI's badge component that displays status indicators,
/// labels, counters, and other contextual information in a compact format.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("badge badge-outline badge-dash badge-soft badge-ghost badge-neutral badge-primary badge-secondary badge-accent badge-info badge-success badge-warning badge-error badge-xs badge-sm badge-md badge-lg badge-xl");
/// ```
///
/// ## Node References
/// - `node_ref` - References the top `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Badge(
    /// Visual style of the badge
    #[prop(optional, into)]
    style: Signal<BadgeStyle>,

    /// Semantic color of the badge
    #[prop(optional, into)]
    color: Signal<BadgeColor>,

    /// Size of the badge
    #[prop(optional, into)]
    size: Signal<BadgeSize>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Let the badge's content WRAP, growing the outline with it (owner,
    /// 2026-08-17: a wrapped value spilled past the badge's fixed height in
    /// every data table). daisyUI's `.badge` pins height and forbids
    /// wrapping in the FRAMEWORK stylesheet, so the escape hatch is a class:
    /// `badge-wrap` — ship it in the consuming app's `input.css`:
    ///
    /// ```css
    /// .badge-wrap {
    ///   height: auto;
    ///   min-height: calc(var(--size, 1.25rem));
    ///   white-space: normal;
    ///   line-height: 1.2;
    ///   padding-top: 0.125rem;
    ///   padding-bottom: 0.125rem;
    ///   text-align: left;
    /// }
    /// ```
    #[prop(optional, into)]
    wrap: Signal<bool>,

    /// Node reference for the badge element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Badge content (text, numbers, icons, or empty for simple indicators)
    children: Children,
) -> impl IntoView {
    view! {
        <div
            aria-label="badge"
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "badge",
                    style.get().as_str(),
                    color.get().as_str(),
                    size.get().as_str(),
                    if wrap.get() { "badge-wrap" } else { "" },
                    class
                )
            }
        >
            {children()}
        </div>
    }
}
