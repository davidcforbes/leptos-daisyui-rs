use super::style::EmptyStateColor;
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Empty State Component
///
/// A centered icon + title + muted subtitle for empty regions -- "no
/// results", "nothing to do", "connection lost", "not found" -- with an
/// optional action slot (e.g. a "Retry" [`Button`](super::button::Button))
/// rendered below the subtitle. Ported from d2d-ui's owner-drawn
/// `EmptyState` control: the manual centering/measurement math there is
/// replaced here by plain CSS flexbox.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{Button, ButtonColor, EmptyState, EmptyStateColor};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         <EmptyState
///             title="No results"
///             subtitle="Try a different search."
///             icon=Box::new(move || view! { <span class="text-5xl">"🔍"</span> }.into_any())
///             subtitle_color=EmptyStateColor::Default
///         >
///             <Button color=ButtonColor::Primary>"Clear filters"</Button>
///         </EmptyState>
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("text-neutral text-primary text-secondary text-accent text-info text-success text-warning text-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the outer `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn EmptyState(
    /// Title text, shown large/semibold beneath the icon.
    #[prop(optional, into)]
    title: Signal<String>,

    /// Muted subtitle text, shown beneath the title.
    #[prop(optional, into)]
    subtitle: Signal<String>,

    /// Optional icon slot (e.g. an inline SVG or emoji) rendered large and
    /// centered above the title. Structural -- like `leading_icon` on
    /// [`Input`](super::input::Input) -- its presence is decided once, at
    /// creation.
    #[prop(optional)]
    icon: Option<Children>,

    /// Color override for the icon slot. `Default` leaves it at its muted
    /// resting opacity.
    #[prop(optional, into)]
    icon_color: Signal<EmptyStateColor>,

    /// Color override for the title text.
    #[prop(optional, into)]
    title_color: Signal<EmptyStateColor>,

    /// Color override for the subtitle text.
    #[prop(optional, into)]
    subtitle_color: Signal<EmptyStateColor>,

    /// Additional CSS classes for the outer container
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer `<div>` element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Optional action slot (e.g. a "Retry" button) rendered below the
    /// subtitle.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "flex flex-col items-center justify-center gap-2 p-8 text-center",
                    class
                )
            }
        >
            {icon.map(|icon_fn| {
                view! {
                    <div class=move || {
                        merge_classes!("text-4xl opacity-60", icon_color.get().as_str())
                    }>{icon_fn()}</div>
                }
            })}
            <h3 class=move || merge_classes!("text-lg font-semibold", title_color.get().as_str())>
                {move || title.get()}
            </h3>
            <p class=move || merge_classes!("text-sm opacity-60", subtitle_color.get().as_str())>
                {move || subtitle.get()}
            </p>
            {children.map(|children_fn| view! { <div class="mt-2">{children_fn()}</div> })}
        </div>
    }
}
