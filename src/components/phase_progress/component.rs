use super::style::{
    phase_fill_percent, phase_overall_percent, phase_progress_value_text, phase_state,
};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// A phase-run progress instrument: N equal segments, one per phase, where
/// completed phases are solid, the current phase is partially filled to a
/// percent, and future phases are muted -- e.g. an ETL run's
/// `capture -> reconcile -> apply` capsule. The current segment can carry a
/// failed tone when the run stopped inside it.
///
/// Unlike [`crate::components::SegmentedBar`] (N proportional shares of one
/// total) and [`crate::components::CapacityBar`] (one value against a cap),
/// each segment here is one *phase* of equal width, and the only variable
/// quantity is how far through the current phase the run has progressed.
///
/// The root is a `role="progressbar"` whose `aria-valuenow` is the overall
/// run percent (phases weighted equally) and whose `aria-valuetext` names
/// the current phase, its percent, and its ordinal -- so a screen reader
/// hears "reconcile 40% (phase 2 of 3)", not a bare number.
///
/// (Keep every inline code span on ONE line in this doc comment: a span that
/// wraps across two `///` lines ICEs clippy's `doc::include_in_doc_without_cfg`
/// lint on 1.95, which takes the whole gate down.)
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::PhaseProgress;
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         // capture done, reconcile 40% through, apply not started.
///         <PhaseProgress
///             phases=vec!["capture".to_string(), "reconcile".to_string(), "apply".to_string()]
///             current=1_usize
///             pct=40_u8
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex w-full gap-1");
/// @source inline("h-2 flex-1 overflow-hidden rounded bg-base-200");
/// @source inline("h-full bg-primary bg-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn PhaseProgress(
    /// Phase names in run order (e.g. `["capture", "reconcile", "apply"]`).
    /// Used for segment count and for the accessible value text.
    #[prop(into)]
    phases: Signal<Vec<String>>,

    /// Index of the phase currently running. Phases before it render solid,
    /// phases after it render empty. An index past the end means the run
    /// finished (every segment solid).
    #[prop(into)]
    current: Signal<usize>,

    /// Percent completion of the current phase, `0..=100` (clamped).
    #[prop(into)]
    pct: Signal<u8>,

    /// The run stopped inside the current phase: its partial fill renders in
    /// the error tone instead of the primary tone. Completed and pending
    /// segments keep their factual state.
    #[prop(optional, into)]
    failed: Signal<bool>,

    /// Additional CSS classes for the wrapping track row.
    #[prop(optional, into)]
    class: &'static str,

    /// Accessible name override. Defaults to the computed value text
    /// (current phase + percent), which satisfies axe's progressbar-name
    /// rule without requiring every call site to restate the obvious.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let value_text =
        move || phase_progress_value_text(&phases.get(), current.get(), pct.get(), failed.get());

    view! {
        <div
            node_ref=node_ref
            role="progressbar"
            aria-label=move || label.get().unwrap_or_else(value_text)
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow=move || {
                format!("{:.0}", phase_overall_percent(phases.get().len(), current.get(), pct.get()))
            }
            aria-valuetext=value_text
            class=move || merge_classes!("flex w-full gap-1", class)
        >
            {move || {
                let count = phases.get().len();
                let current = current.get();
                let pct = pct.get();
                let failed = failed.get();
                (0..count)
                    .map(|index| {
                        let width = phase_fill_percent(index, current, pct);
                        let state = phase_state(index, current, failed);
                        let fill = if state == "failed" { "bg-error" } else { "bg-primary" };
                        view! {
                            <div
                                class="h-2 flex-1 overflow-hidden rounded bg-base-200"
                                data-phase-index=index
                                data-phase-state=state
                            >
                                <div class=format!("h-full {fill}") style:width=format!("{width}%")></div>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
