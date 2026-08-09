use super::style::ProgressColor;
use crate::merge_classes;
use leptos::{html::Progress as HtmlProgress, prelude::*};

/// Normalizes the `max` attribute of a `<progress>` element.
///
/// HTML requires `max` to be a positive finite number and defaults it to `1.0`
/// when absent or invalid. A non-positive or non-finite value is normalized to
/// that same `1.0` here rather than emitted into the DOM as garbage.
pub(crate) fn progress_max(max: f64) -> f64 {
    if max.is_finite() && max > 0.0 {
        max
    } else {
        1.0
    }
}

/// Resolves the `value` attribute of a `<progress>` element.
///
/// `None` means *indeterminate* and the caller MUST omit the attribute
/// entirely — a `<progress>` with no `value` is what daisyUI animates as a
/// sliding stripe, whereas `value=0` renders a determinate, empty bar. A
/// non-finite value (NaN, infinity) is treated as indeterminate too, since
/// emitting it would produce a bar the browser cannot lay out.
///
/// A finite value is clamped into `0.0..=max`, mirroring the browser's own
/// clamping so the emitted DOM agrees with what is painted (and with what an
/// assistive technology reads back).
pub(crate) fn progress_value(value: Option<f64>, max: f64) -> Option<f64> {
    match value {
        Some(v) if v.is_finite() => Some(v.clamp(0.0, progress_max(max))),
        _ => None,
    }
}

/// # Progress Component
///
/// A reactive Leptos wrapper for daisyUI's progress component that displays the completion
/// progress of tasks or operations with visual progress bars.
///
/// ## Determinate vs indeterminate
///
/// Leaving `value` unset renders an **indeterminate** bar: the `value`
/// attribute is omitted entirely and daisyUI animates a sliding stripe. This
/// is the default and is deliberately distinct from `value=0.0`, which is a
/// determinate bar that happens to be empty. Set `value` to make the bar
/// data-driven; `max` defaults to HTML's own default of `1.0`, so pass
/// `max=100.0` if your values are percentages.
///
/// ```rust,ignore
/// // Indeterminate — no value attribute is emitted.
/// view! { <Progress class="w-56" /> }
///
/// // Determinate, driven by a signal.
/// let pct = RwSignal::new(70.0);
/// view! { <Progress value=pct max=100.0 color=ProgressColor::Success class="w-56" /> }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("progress progress-primary progress-secondary progress-accent progress-info progress-success progress-warning progress-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the top `<progress>` element ([HTMLProgressElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLProgressElement))
#[component]
pub fn Progress(
    /// Progress bar color variant
    #[prop(optional, into)]
    color: Signal<ProgressColor>,

    /// Current progress, in the same units as `max`.
    ///
    /// Unset (the default) renders an **indeterminate** bar — the `value`
    /// attribute is omitted, not emitted as zero. Values outside `0.0..=max`
    /// are clamped; a non-finite value falls back to indeterminate.
    #[prop(optional, into)]
    value: MaybeProp<f64>,

    /// Upper bound of the progress scale. Defaults to HTML's own default of
    /// `1.0`, so `value=0.7` is 70%. Pass `max=100.0` to drive the bar with
    /// percentages. A non-positive or non-finite `max` falls back to `1.0`.
    #[prop(optional, into, default = Signal::derive(|| 1.0))]
    max: Signal<f64>,

    /// Node reference for the progress element
    #[prop(optional)]
    node_ref: NodeRef<HtmlProgress>,

    /// Additional CSS classes to apply
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    view! {
        <progress
            node_ref=node_ref
            value=move || progress_value(value.get(), max.get())
            max=move || progress_max(max.get())
            class=move || {
                merge_classes!(
                    "progress",
                    color.get().as_str(),
                    class
                )
            }
        />
    }
}
