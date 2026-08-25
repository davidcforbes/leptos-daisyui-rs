use super::style::{segmented_bar_percent, segmented_bar_total};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// A horizontal multi-segment distribution bar -- N proportional, distinctly
/// colored segments drawn side-by-side in one track (e.g. a green/yellow/red
/// SLA-buffer breakdown, pass/fail/pending, or a budget-category split).
///
/// Unlike [`crate::components::CapacityBar`] (a single value against a
/// capacity threshold), `SegmentedBar` has no single "value" and no cap-line
/// -- every segment is sized to its share of the segments' combined total, so
/// the whole track always sums to (at most) 100%. Ported from the pattern
/// duplicated by hand in `inventory-web`'s Lifecycle wall (three stacked
/// `<div>`s with inline `width: N%`, one per G/Y/R buffer-status bucket) --
/// see the `4iiz-inventory` framework-purity audit finding #6
/// (`bd_4iiz-inventory-xsa`/`4gj`).
///
/// Segment widths are normalized to the SUM of every segment's value, not to
/// a fixed 100 -- raw counts and the equivalent pre-normalized fractions render
/// identically (a 4/3/3 count split draws the same bar as 0.4/0.3/0.3).
/// A non-positive total (every value zero/negative, or an empty `segments`
/// list) renders an empty track -- never a divide-by-zero panic or a
/// misleadingly full bar.
///
/// (Keep every inline code span on ONE line in this doc comment: a span that
/// wraps across two `///` lines ICEs clippy's `doc::include_in_doc_without_cfg`
/// lint on 1.95, which takes the whole gate down.)
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::SegmentedBar;
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         // G/Y/R buffer-status distribution: 40% in-SLA, 30% late, 30% dormant.
///         <SegmentedBar segments=vec![
///             (0.4, "bg-success"),
///             (0.3, "bg-warning"),
///             (0.3, "bg-error"),
///         ] />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex h-2 w-full overflow-hidden rounded bg-base-200");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn SegmentedBar(
    /// `(value, css-class)` pairs, in left-to-right draw order. `css-class`
    /// is any Tailwind/daisyUI background-color utility (`"bg-success"`,
    /// `"bg-warning"`, `"bg-error"`, ...) passed straight through -- this is
    /// deliberately not a closed color enum (unlike `CapacityBarColor`) so
    /// callers aren't limited to daisyUI's semantic palette for an arbitrary
    /// N-way breakdown.
    #[prop(into)]
    segments: Signal<Vec<(f64, &'static str)>>,

    /// Additional CSS classes for the track.
    #[prop(optional, into)]
    class: &'static str,

    /// Accessible name for the distribution image. Callers should summarize
    /// the categories the colored segments encode.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            role="img"
            aria-label=move || label.get()
            class=move || {
                merge_classes!("flex h-2 w-full overflow-hidden rounded bg-base-200", class)
            }
        >
            {move || {
                let segs = segments.get();
                let total = segmented_bar_total(&segs);
                segs.into_iter()
                    .map(|(value, seg_class)| {
                        let width = segmented_bar_percent(value, total);
                        view! { <div class=seg_class style:width=format!("{width}%")></div> }
                    })
                    .collect_view()
            }}
        </div>
    }
}
