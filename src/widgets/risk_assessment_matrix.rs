//! 3x3 Risk Assessment Matrix component (UT2-16 / EUC-s35j).
//!
//! Renders a colour-coded grid of (impact x likelihood) cells. Each cell
//! shows a count + a band label (Negligible / Low / Medium / High / Critical).
//!
//! Reusable across:
//!   - /requests/security      — current home (UT2-16)
//!   - /incidents              — future use, same axes
//!   - /standards              — future use as a per-standard heat map
//!
//! Input is a flat list of (impact, likelihood, count) triples — rendering
//! collapses missing cells to 0, so the caller is free to send partial data.

use leptos::prelude::*;

use crate::components::{Card, CardBody};

/// Single cell of the 3x3 matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskMatrixDatum {
    /// "high" | "medium" | "low"
    pub impact: String,
    /// "high" | "medium" | "low"
    pub likelihood: String,
    pub count: i64,
}

/// Maps an (impact, likelihood) pair to a (Tailwind background colour, band label).
fn band_for(impact: &str, likelihood: &str) -> (&'static str, &'static str) {
    match (impact, likelihood) {
        ("high", "high") => ("bg-red-600 text-white", "Critical"),
        ("high", "medium") => ("bg-red-400 text-white", "High"),
        ("high", "low") => ("bg-amber-400 text-white", "Medium"),
        ("medium", "high") => ("bg-red-400 text-white", "High"),
        ("medium", "medium") => ("bg-amber-400 text-white", "Medium"),
        ("medium", "low") => ("bg-emerald-400 text-white", "Low"),
        ("low", "high") => ("bg-amber-400 text-white", "Medium"),
        ("low", "medium") => ("bg-emerald-400 text-white", "Low"),
        ("low", "low") => ("bg-emerald-500 text-white", "Negligible"),
        _ => ("bg-base-200 text-base-content", "—"),
    }
}

#[component]
pub fn RiskAssessmentMatrix(
    /// Title shown at the top of the card.
    #[prop(into, default = "Risk Assessment Matrix".into())]
    title: String,
    /// Cell data — order doesn't matter; missing combinations render as 0.
    data: Vec<RiskMatrixDatum>,
) -> impl IntoView {
    let lookup = move |i: &str, l: &str| -> i64 {
        data.iter()
            .find(|c| c.impact == i && c.likelihood == l)
            .map(|c| c.count)
            .unwrap_or(0)
    };

    let impact_rows = ["high", "medium", "low"];
    let impact_labels = ["High", "Medium", "Low"];
    let likelihood_cols = ["low", "medium", "high"];

    view! {
        <Card class="shadow-sm">
            <CardBody class="p-4 space-y-3">
                <h3 class="font-semibold text-sm">{title}</h3>
                <div class="overflow-x-auto">
                    <table class="w-full border-collapse">
                        <thead>
                            <tr>
                                <th class="text-xs text-base-content/50 p-1 w-20"></th>
                                <th class="text-xs text-center text-base-content/60 p-1 font-medium">"Low"</th>
                                <th class="text-xs text-center text-base-content/60 p-1 font-medium">"Medium"</th>
                                <th class="text-xs text-center text-base-content/60 p-1 font-medium">"High"</th>
                            </tr>
                            <tr>
                                <th></th>
                                <th colspan="3" class="text-xs text-center text-base-content/40 pb-1 font-normal">"Likelihood \u{2192}"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {impact_rows.iter().enumerate().map(|(idx, impact_code)| {
                                let label = impact_labels[idx];
                                let cells: Vec<_> = likelihood_cols.iter().map(|lk| {
                                    let count = lookup(impact_code, lk);
                                    let (color, band) = band_for(impact_code, lk);
                                    view! {
                                        <td class="p-1">
                                            <div
                                                class=format!("rounded-lg {color} flex flex-col items-center justify-center h-16 cursor-default")
                                                title=band
                                            >
                                                <span class="text-lg font-bold">{count}</span>
                                                <span class="text-xs opacity-80">{band}</span>
                                            </div>
                                        </td>
                                    }
                                }).collect();
                                view! {
                                    <tr>
                                        <td class="text-xs text-base-content/60 font-medium p-1 align-middle">{label}</td>
                                        {cells}
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
                <div class="text-xs text-base-content/40 text-center">"\u{2191} Impact"</div>
            </CardBody>
        </Card>
    }
}
