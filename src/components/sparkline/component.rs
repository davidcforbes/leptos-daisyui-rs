use super::style::{
    SparklineColor, sparkline_current, sparkline_current_label, sparkline_has_readout,
    sparkline_peak, sparkline_peak_label, sparkline_points,
};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Sparkline Component
///
/// A small time-series line chart -- an inline SVG polyline of `f32` samples
/// drawn over a baseline, with an optional framed card and a title/current
/// value/peak readout row. Useful for a throughput meter (KB/s), CPU%,
/// latency, or any at-a-glance trend. Ported from d2d-ui's owner-drawn
/// `Sparkline` control.
///
/// daisyUI has no dedicated sparkline component, so the polyline stroke uses
/// `currentColor`; apply a [`SparklineColor`] to theme it, or leave it
/// unframed and drop it inline (e.g. inside a `DataTable` cell).
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{Sparkline, SparklineColor};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let samples = RwSignal::new(vec![2.0, 5.0, 3.0, 9.0, 4.0]);
///     view! {
///         <Sparkline
///             samples=Signal::derive(move || samples.get())
///             title="Throughput"
///             unit="KB/s"
///             color=SparklineColor::Primary
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("card card-border bg-base-100 p-2 text-base-content text-primary text-secondary text-accent text-success text-info text-warning text-error");
/// ```
///
/// See also [`crate::charts::Sparkline`] for a minimal non-reactive chart primitive without daisyUI framing.
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Sparkline(
    /// Time-series sample values, oldest first. The last sample is shown as
    /// the "current" readout value.
    #[prop(optional, into)]
    samples: Signal<Vec<f32>>,

    /// Readout title (e.g. "Throughput"). Empty (the default) hides the
    /// title/current/peak readout row -- appropriate for an inline sparkline.
    #[prop(optional, into)]
    title: Signal<String>,

    /// Unit suffix appended to the current-value readout (e.g. "KB/s").
    #[prop(optional, into)]
    unit: Signal<String>,

    /// Render as its own bordered card. Turn off for inline use, e.g. inside
    /// a table cell.
    #[prop(optional, into, default=Signal::derive(|| true))]
    framed: Signal<bool>,

    /// Stroke color scheme, applied via `currentColor`
    #[prop(optional, into)]
    color: Signal<SparklineColor>,

    /// SVG viewBox width in user units
    #[prop(optional, into, default=Signal::derive(|| 200.0))]
    width: Signal<f32>,

    /// SVG viewBox height in user units
    #[prop(optional, into, default=Signal::derive(|| 60.0))]
    height: Signal<f32>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "sparkline",
                    if framed.get() { "card card-border bg-base-100 p-2" } else { "" },
                    color.get().as_str(),
                    class
                )
            }
        >
            <Show when=move || sparkline_has_readout(&title.get())>
                <div class="mb-1 flex items-baseline justify-between gap-2 text-xs">
                    <span class="truncate font-semibold">
                        {move || {
                            let current = sparkline_current(&samples.get());
                            sparkline_current_label(&title.get(), &unit.get(), current)
                        }}
                    </span>
                    <span class="shrink-0 opacity-60">
                        {move || sparkline_peak_label(sparkline_peak(&samples.get()))}
                    </span>
                </div>
            </Show>
            <svg
                viewBox=move || format!("0 0 {} {}", width.get(), height.get())
                preserveAspectRatio="none"
                class="h-auto w-full"
                xmlns="http://www.w3.org/2000/svg"
            >
                <line
                    x1="0"
                    y1=move || height.get().to_string()
                    x2=move || width.get().to_string()
                    y2=move || height.get().to_string()
                    stroke="currentColor"
                    stroke-opacity="0.2"
                    stroke-width="1"
                />
                <polyline
                    points=move || sparkline_points(&samples.get(), width.get(), height.get())
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
            </svg>
        </div>
    }
}
