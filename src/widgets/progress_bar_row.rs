//! Reusable label + horizontal progress bar + percentage row.
//!
//! Used by the Business Scorecards right rail (UT2-19) to render the six
//! "Component Scores" rows. The bar colour switches between green (>=85%),
//! amber (70..84%), and red (<70%) so the panel reads as a quick traffic
//! light per component without the caller having to compute classes.
//!
//! The component intentionally takes an owned `String` label so it can be
//! used with dynamic, per-row data fetched from GraphQL.

use leptos::prelude::*;

/// Color threshold used to pick the bar colour. The default thresholds
/// (>=85 green, >=70 amber, <70 red) match the bead's spec.
fn bar_color(pct: f64) -> &'static str {
    if pct >= 85.0 {
        "bg-success"
    } else if pct >= 70.0 {
        "bg-warning"
    } else {
        "bg-error"
    }
}

fn text_color(pct: f64) -> &'static str {
    if pct >= 85.0 {
        "text-success"
    } else if pct >= 70.0 {
        "text-warning"
    } else {
        "text-error"
    }
}

/// A single label / percentage / horizontal-bar row.
#[component]
pub fn ProgressBarRow(
    /// Human-readable component label (e.g. "Inventory Completeness").
    #[prop(into)]
    label: String,
    /// Component score, 0..100.
    pct: f64,
    /// Bar height utility — defaults to `h-1.5` (used in the right rail).
    #[prop(default = "h-1.5", into)]
    bar_height: &'static str,
) -> impl IntoView {
    let pct_clamped = pct.clamp(0.0, 100.0);
    let pct_label = format!("{:.0}%", pct_clamped);
    let bar_cls = bar_color(pct_clamped);
    let txt_cls = text_color(pct_clamped);
    let width_style = format!("width: {:.1}%", pct_clamped);

    view! {
        <div>
            <div class="flex justify-between text-xs mb-1">
                <span>{label}</span>
                <span class=format!("{} font-bold", txt_cls)>{pct_label}</span>
            </div>
            <div class=format!("w-full bg-base-200 rounded-full {}", bar_height)>
                <div
                    class=format!("rounded-full {} {}", bar_height, bar_cls)
                    style=width_style
                ></div>
            </div>
        </div>
    }
}
