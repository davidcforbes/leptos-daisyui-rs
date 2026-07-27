use super::style::{MetricRowColor, container_class, divider_class, label_class, value_class};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Metric Row Component
///
/// A compact `label ... value` key/value row: a muted left-aligned label and
/// a right-aligned value (optionally emphasized with `font-semibold`), with
/// an optional stacked layout (label above value) and an optional hairline
/// bottom divider. Useful for facts grids and detail panels. Ported from
/// d2d-ui's owner-drawn `MetricRow` control -- the manual text measurement
/// used there to keep the label/value from overlapping is replaced here by
/// plain CSS flexbox (`justify-between`).
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{MetricRow, MetricRowColor};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         <div>
///             <MetricRow label="Case value" value="$1,200" bold=true />
///             <MetricRow
///                 label="Status"
///                 value="Overdue"
///                 value_color=MetricRowColor::Error
///                 divider=true
///             />
///             <MetricRow label="Opened" value="2026-01-14" stacked=true />
///         </div>
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col items-baseline justify-between gap-1 gap-2");
/// @source inline("text-xs text-sm opacity-60 font-semibold text-right pb-1 border-b border-base-200");
/// @source inline("text-neutral text-primary text-secondary text-accent text-info text-success text-warning text-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the outer `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn MetricRow(
    /// Left-aligned label text (muted).
    #[prop(into)]
    label: Signal<String>,

    /// Value text -- right-aligned in the default row layout, or below the
    /// label when `stacked`.
    #[prop(into)]
    value: Signal<String>,

    /// Emphasize the value with `font-semibold`.
    #[prop(optional, into)]
    bold: Signal<bool>,

    /// Stack the label above the value instead of the default two-column
    /// `label ... value` row.
    #[prop(optional, into)]
    stacked: Signal<bool>,

    /// Draw a hairline divider along the row's bottom edge.
    #[prop(optional, into)]
    divider: Signal<bool>,

    /// Color override for the label text.
    #[prop(optional, into)]
    label_color: Signal<MetricRowColor>,

    /// Color override for the value text (e.g. a status-tinted amount).
    #[prop(optional, into)]
    value_color: Signal<MetricRowColor>,

    /// Additional CSS classes for the outer container.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    container_class(stacked.get()),
                    divider_class(divider.get()),
                    class
                )
            }
        >
            <span class=move || {
                merge_classes!(label_class(stacked.get()), label_color.get().as_str())
            }>
                {move || label.get()}
            </span>
            <span class=move || {
                merge_classes!(value_class(stacked.get(), bold.get()), value_color.get().as_str())
            }>
                {move || value.get()}
            </span>
        </div>
    }
}
