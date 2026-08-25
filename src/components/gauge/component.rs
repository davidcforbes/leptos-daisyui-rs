use super::style::{
    gauge_arc_path, gauge_bands, gauge_fraction, gauge_readout, gauge_value_paint,
};
use crate::charts::paint::stroke_attrs;
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// Dial circle center and radius in viewBox units. The 240-degree sweep
/// leaves the bottom of the circle open, so the viewBox is cropped below the
/// shoulders (`y = 50 + 40 * sin(150deg) = 70`, plus half the stroke).
const CX: f64 = 50.0;
const CY: f64 = 50.0;
const R: f64 = 40.0;

/// An open-arc dial gauge with budget bands: a ~240-degree track, warn/error
/// zones painted on the track at threshold fractions, a value arc that
/// escalates from primary to the zone tone it has entered, and a center
/// readout (tabular-nums value, unit, and sub-caption). Built for the
/// 4iiz-etl portal's server CPU/memory/disk/network cluster (`ldui-nx5`).
///
/// Unlike [`crate::components::RadialProgress`] (daisyUI's full-ring percent
/// progress), this is an instrument dial: an open arc with banded thresholds
/// and a unit readout, not a completion ring. Pure props, no fetching --
/// the same posture as [`crate::components::CapacityBar`] and
/// [`crate::components::SlaChip`].
///
/// The root is `role="meter"` (a gauge measures a quantity within known
/// bounds; it is not progress toward completion) with the value, bounds, and
/// an accessible name derived from the caption and unit unless `label`
/// overrides it.
///
/// (Keep every inline code span on ONE line in this doc comment: a span that
/// wraps across two `///` lines ICEs clippy's `doc::include_in_doc_without_cfg`
/// lint on 1.95, which takes the whole gate down.)
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::Gauge;
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         // CPU at 62%, warn band from 70%, error band from 90%.
///         <Gauge
///             value=62.0
///             max=100.0
///             unit="%"
///             caption="CPU"
///             warn_from=0.7
///             error_from=0.9
///             class="w-40"
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("relative inline-block w-full");
/// @source inline("absolute flex flex-col items-center");
/// @source inline("text-2xl font-semibold tabular-nums leading-none");
/// @source inline("text-sm font-normal text-base-content/75");
/// @source inline("text-xs text-base-content/75");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Gauge(
    /// Current reading, in the same units as `max`.
    #[prop(into)]
    value: Signal<f64>,

    /// Scale maximum (the end of the dial). A non-positive max renders an
    /// empty dial rather than dividing by zero.
    #[prop(into)]
    max: Signal<f64>,

    /// Unit suffix rendered after the readout value (`"%"`, `"GB"`, `"Mbps"`).
    #[prop(optional, into)]
    unit: MaybeProp<String>,

    /// Sub-caption under the readout naming the measured quantity
    /// (`"CPU"`, `"Memory"`).
    #[prop(optional, into)]
    caption: MaybeProp<String>,

    /// Start of the warning band, as a fraction of the dial (`0.0..=1.0`).
    /// The band runs to `error_from` (or the end of the dial).
    #[prop(optional, into)]
    warn_from: Signal<Option<f64>>,

    /// Start of the error band, as a fraction of the dial (`0.0..=1.0`).
    /// The band runs to the end of the dial.
    #[prop(optional, into)]
    error_from: Signal<Option<f64>>,

    /// Host display string for the readout value, overriding the default
    /// formatting (whole numbers at ten and above, one decimal below).
    #[prop(optional, into)]
    display: MaybeProp<String>,

    /// Additional CSS classes for the wrapper (size the gauge here, e.g.
    /// `"w-40"`).
    #[prop(optional, into)]
    class: &'static str,

    /// Accessible name override. Defaults to caption plus unit (e.g.
    /// `"CPU (%)"`), falling back to `"gauge"` when neither is given.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let fraction = move || gauge_fraction(value.get(), max.get());

    let (track_stroke, track_style) = stroke_attrs("var(--color-base-300)".to_string());
    let (warn_stroke, warn_style) = stroke_attrs("var(--color-warning)".to_string());
    let (error_stroke, error_style) = stroke_attrs("var(--color-error)".to_string());
    let value_paint = move || gauge_value_paint(fraction(), warn_from.get(), error_from.get());
    let value_stroke = move || stroke_attrs(value_paint().to_string()).0;
    let value_style = move || stroke_attrs(value_paint().to_string()).1;

    let track_path = gauge_arc_path(CX, CY, R, 0.0, 1.0);
    let warn_path = move || {
        let (warn, _) = gauge_bands(warn_from.get(), error_from.get());
        warn.map(|(from, to)| gauge_arc_path(CX, CY, R, from, to))
            .unwrap_or_default()
    };
    let error_path = move || {
        let (_, error) = gauge_bands(warn_from.get(), error_from.get());
        error
            .map(|(from, to)| gauge_arc_path(CX, CY, R, from, to))
            .unwrap_or_default()
    };
    let value_path = move || gauge_arc_path(CX, CY, R, 0.0, fraction());

    let readout = move || display.get().unwrap_or_else(|| gauge_readout(value.get()));
    let accessible_name = move || {
        label.get().unwrap_or_else(|| {
            match (caption.get(), unit.get()) {
                (Some(caption), Some(unit)) => format!("{caption} ({unit})"),
                (Some(caption), None) => caption,
                (None, Some(unit)) => format!("gauge ({unit})"),
                (None, None) => "gauge".to_string(),
            }
        })
    };

    view! {
        <div
            node_ref=node_ref
            role="meter"
            aria-label=accessible_name
            aria-valuemin="0"
            aria-valuemax=move || max.get().to_string()
            aria-valuenow=move || value.get().to_string()
            class=move || merge_classes!("relative inline-block w-full", class)
        >
            <svg viewBox="0 0 100 76" class="block w-full" xmlns="http://www.w3.org/2000/svg">
                // Track: the full open arc in a muted tone.
                <path
                    d=track_path
                    fill="none"
                    stroke=track_stroke
                    style=track_style
                    stroke-width="8"
                    stroke-linecap="round"
                />
                // Budget bands painted on the track. The value arc overdraws
                // the portion already consumed; the zones ahead stay visible.
                <path
                    d=warn_path
                    fill="none"
                    stroke=warn_stroke
                    style=warn_style
                    stroke-width="8"
                    stroke-opacity="0.35"
                    data-gauge-band="warn"
                />
                <path
                    d=error_path
                    fill="none"
                    stroke=error_stroke
                    style=error_style
                    stroke-width="8"
                    stroke-opacity="0.35"
                    data-gauge-band="error"
                />
                // Value arc, colored by the zone the reading sits in.
                <path
                    d=value_path
                    fill="none"
                    stroke=value_stroke
                    style=value_style
                    stroke-width="8"
                    stroke-linecap="round"
                    data-gauge-value=""
                />
            </svg>
            // Readout at the dial center (50/76 of the viewBox height).
            <div
                class="absolute flex flex-col items-center"
                style="left:50%;top:66%;transform:translate(-50%,-50%);"
            >
                <span class="text-2xl font-semibold tabular-nums leading-none">
                    {readout}
                    {move || {
                        unit.get()
                            .map(|unit| {
                                view! {
                                    <span class="text-sm font-normal text-base-content/75">
                                        {unit}
                                    </span>
                                }
                            })
                    }}
                </span>
                {move || {
                    caption
                        .get()
                        .map(|caption| {
                            view! { <span class="text-xs text-base-content/75">{caption}</span> }
                        })
                }}
            </div>
        </div>
    }
}
