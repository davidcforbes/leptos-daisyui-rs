use super::style::{
    CapacityBarColor, capacity_bar_default_max, capacity_bar_overflow_band, capacity_bar_percent,
    capacity_bar_under_cap_percent,
};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// A horizontal load bar with a capacity-threshold marker.
///
/// A track with a value fill, a thin cap-line tick at the capacity
/// threshold, a distinct-colored overflow band for the portion of the value
/// that exceeds the cap, and an optional translucent "ghost"/preview fill
/// (e.g. showing the effect of a pending move before it is committed).
/// Ported from d2d-ui's owner-drawn `CapacityBar` control -- the Direct2D
/// `rect`/brush/`draw()` code is replaced by absolutely-positioned `<span>`s
/// over a relatively-positioned track `<div>`, since daisyUI's `progress`
/// element has no notion of a cap-line, overflow band, or ghost fill.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{CapacityBar, CapacityBarColor};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         // 12 of 10 units used — renders the under-cap fill up to the
///         // cap-line plus a red overflow band from 10 to 12.
///         <CapacityBar value=12.0 cap=10.0 />
///
///         // With a translucent ghost preview and an explicit color.
///         <CapacityBar
///             value=6.0
///             cap=10.0
///             ghost=8.0
///             color=CapacityBarColor::Success
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("relative h-3 w-full overflow-hidden rounded-full bg-base-200");
/// @source inline("absolute inset-y-0 left-0 top-0 h-full rounded-full w-px");
/// @source inline("bg-info/30 bg-base-content/50");
/// @source inline("bg-neutral bg-primary bg-secondary bg-accent bg-info bg-success bg-warning bg-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn CapacityBar(
    /// Current load, in the same units as `cap`.
    #[prop(into)]
    value: Signal<f64>,

    /// Capacity threshold -- the cap-line position along the track.
    #[prop(into)]
    cap: Signal<f64>,

    /// Scale maximum (right edge of the track). Defaults to `cap * 1.25`,
    /// clamped to be at least `cap` and `value`, giving headroom to show the
    /// overflow band. Pass a value to override the computed default; an explicit
    /// override is still clamped to be at least `cap`.
    #[prop(optional, into)]
    max: Signal<Option<f64>>,

    /// Optional translucent preview/ghost value (e.g. the effect of a
    /// pending move), drawn behind the real fill.
    #[prop(optional, into)]
    ghost: Signal<Option<f64>>,

    /// Color of the under-cap portion of the fill.
    #[prop(optional, into)]
    color: Signal<CapacityBarColor>,

    /// Color of the over-cap overflow band.
    #[prop(optional, into, default = Signal::derive(|| CapacityBarColor::Error))]
    over_color: Signal<CapacityBarColor>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Accessible name for the progressbar (axe `aria-progressbar-name`:
    /// a `role="progressbar"` with values but no name is a serious WCAG
    /// violation). Optional and additive; pass the metric's own label.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let effective_max = move || {
        max.get()
            .unwrap_or_else(|| capacity_bar_default_max(cap.get(), value.get()))
            .max(cap.get())
    };
    let cap_percent = move || capacity_bar_percent(cap.get(), effective_max());
    let value_percent = move || capacity_bar_percent(value.get(), effective_max());
    let under_cap_percent = move || capacity_bar_under_cap_percent(value_percent(), cap_percent());
    let overflow_band = move || capacity_bar_overflow_band(value_percent(), cap_percent());
    let ghost_percent = move || {
        ghost
            .get()
            .map(|g| capacity_bar_percent(g, effective_max()))
    };

    view! {
        <div
            node_ref=node_ref
            role="progressbar"
            aria-label=move || label.get()
            aria-valuenow=move || value.get().to_string()
            aria-valuemin="0"
            aria-valuemax=move || effective_max().to_string()
            class=move || {
                merge_classes!(
                    "relative h-3 w-full overflow-hidden rounded-full bg-base-200",
                    class
                )
            }
        >
            // Ghost/preview fill (behind the real fill).
            <Show when=move || ghost_percent().is_some_and(|g| g > 0.0)>
                <span
                    class="bg-info/30 absolute inset-y-0 left-0 top-0 h-full rounded-full"
                    style:width=move || format!("{}%", ghost_percent().unwrap_or(0.0))
                ></span>
            </Show>

            // Under-cap portion of the fill.
            <Show when=move || { under_cap_percent() > 0.0 }>
                <span
                    class=move || {
                        merge_classes!(
                            "absolute inset-y-0 left-0 top-0 h-full rounded-full",
                            color.get().as_str()
                        )
                    }
                    style:width=move || format!("{}%", under_cap_percent())
                ></span>
            </Show>

            // Over-cap (overflow) band, drawn in the distinct over_color.
            <Show when=move || overflow_band().is_some()>
                <span
                    class=move || {
                        merge_classes!("absolute inset-y-0 top-0 h-full rounded-full", over_color.get().as_str())
                    }
                    style:left=move || {
                        format!("{}%", overflow_band().map(|(left, _)| left).unwrap_or(0.0))
                    }
                    style:width=move || {
                        format!("{}%", overflow_band().map(|(_, width)| width).unwrap_or(0.0))
                    }
                ></span>
            </Show>

            // Cap-line tick — a thin vertical marker at the threshold.
            <span
                class="bg-base-content/50 absolute inset-y-0 top-0 h-full w-px"
                style:left=move || format!("{}%", cap_percent())
            ></span>
        </div>
    }
}
