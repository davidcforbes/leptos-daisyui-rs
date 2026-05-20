use super::style::SvgFit;
use crate::merge_classes;
use leptos::{html::Figure, prelude::*};

/// # SVG Display Component
///
/// A reactive Leptos component that renders SVG content inline within a figure element.
/// Supports configurable sizing, object-fit modes, and accessibility labels.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("object-contain object-cover object-fill object-scale-down object-none");
/// ```
///
/// ## Node References
/// - `node_ref` - References the `<figure>` element ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement))
#[component]
pub fn SvgDisplay(
    /// SVG content string to render inline
    #[prop(optional, into)]
    content: Signal<String>,

    /// Alt text for accessibility
    #[prop(optional, into)]
    alt: Signal<String>,

    /// Maximum width in pixels (CSS max-width)
    #[prop(optional, into)]
    max_width: Signal<Option<f32>>,

    /// Maximum height in pixels (CSS max-height)
    #[prop(optional, into)]
    max_height: Signal<Option<f32>>,

    /// Object fit mode
    #[prop(optional, into)]
    fit: Signal<SvgFit>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the figure element
    #[prop(optional)]
    node_ref: NodeRef<Figure>,

    /// Child content (overlaid on the SVG, e.g. captions)
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <figure
            node_ref=node_ref
            aria-label=move || {
                let a = alt.get();
                if a.is_empty() { None } else { Some(a) }
            }
            class=move || {
                merge_classes!(
                    "svg-display",
                    fit.get().as_str(),
                    class
                )
            }
        >
            <div
                inner_html=move || content.get()
                style=("max-width", move || max_width.get().map(|w| format!("{}px", w)))
                style=("max-height", move || max_height.get().map(|h| format!("{}px", h)))
            />
            {children.map(|v| v())}
        </figure>
    }
}
