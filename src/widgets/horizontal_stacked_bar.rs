//! Horizontal stacked bar component (UT2-14).
//!
//! Renders a vertical list of horizontal bars, one per category, with
//! coloured sub-segments inside each bar. Used by the Sandbox Utilization
//! chart (CPU / Memory / GPU per environment template) and is general enough
//! to be reused by the QA / Security / Deployment pages once they need a
//! similar visualization.
//!
//! Shape contract:
//! - `categories`: Vec<String> — one row per category.
//! - `series`: Vec<HorizontalSeries> — each series holds one value per
//!   category (`values.len() == categories.len()`). Segments are stacked
//!   left-to-right inside each row's bar.
//! - `right_labels`: optional Vec<String> — tail label per row (e.g.
//!   `"4 instances"`). Same length as `categories`.
//! - `subtitle`: optional sub-text under each category label (e.g.
//!   `"4 vCPU / 8GB / 20GB"`).

use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct HorizontalSeries {
    pub name: String,
    pub values: Vec<f64>,
    /// Tailwind background color class (e.g. `"bg-blue-500"`).
    pub color: String,
}

#[component]
pub fn HorizontalStackedBar(
    categories: Vec<String>,
    series: Vec<HorizontalSeries>,
    #[prop(optional)] subtitles: Option<Vec<String>>,
    #[prop(optional)] right_labels: Option<Vec<String>>,
    /// Show legend swatches at the top.
    #[prop(default = true)]
    show_legend: bool,
) -> impl IntoView {
    if categories.is_empty() || series.is_empty() {
        return view! {
            <div class="text-sm text-base-content/50 italic">"No data"</div>
        }
        .into_any();
    }

    // Compute the global max sum-per-category so all bars share a scale.
    let n = categories.len();
    let max_total: f64 = (0..n)
        .map(|ci| {
            series
                .iter()
                .map(|s| s.values.get(ci).copied().unwrap_or(0.0))
                .sum::<f64>()
        })
        .fold(0.0_f64, f64::max);
    let scale = if max_total.abs() < f64::EPSILON {
        1.0
    } else {
        max_total
    };

    let legend_view = if show_legend {
        Some(view! {
            <div class="flex items-center gap-4 mb-3">
                {series.iter().map(|s| {
                    let color = s.color.clone();
                    let name = s.name.clone();
                    view! {
                        <div class="flex items-center gap-1.5">
                            <span class=format!("inline-block w-3 h-3 rounded-sm {color}")></span>
                            <span class="text-xs text-base-content/70">{name}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        })
    } else {
        None
    };

    let rows = (0..n).map(|ci| {
        let cat = categories[ci].clone();
        let subtitle = subtitles.as_ref().and_then(|s| s.get(ci)).cloned();
        let right = right_labels.as_ref().and_then(|s| s.get(ci)).cloned();
        let segments = series.iter().map(|s| {
            let raw = s.values.get(ci).copied().unwrap_or(0.0);
            let pct = (raw / scale * 100.0).clamp(0.0, 100.0);
            let color = s.color.clone();
            let label = if raw > 0.0 { format!("{}", raw as i64) } else { String::new() };
            view! {
                <div
                    class=format!("h-6 {color} flex items-center justify-center text-[10px] text-white font-medium overflow-hidden")
                    style=format!("width: {}%;", pct)
                    title=format!("{}: {}", s.name, raw as i64)
                >
                    {label}
                </div>
            }
        }).collect::<Vec<_>>();

        view! {
            <div class="grid grid-cols-[160px_1fr_90px] items-center gap-3 py-1.5">
                <div class="min-w-0">
                    <div class="text-sm font-medium truncate">{cat}</div>
                    {subtitle.map(|s| view! {
                        <div class="text-[11px] text-base-content/50 truncate">{s}</div>
                    })}
                </div>
                <div class="flex w-full h-6 rounded-md overflow-hidden bg-base-200">
                    {segments}
                </div>
                <div class="text-xs text-base-content/60 text-right truncate">
                    {right.unwrap_or_default()}
                </div>
            </div>
        }
    }).collect::<Vec<_>>();

    view! {
        <div class="space-y-0">
            {legend_view}
            <div class="space-y-1">
                {rows}
            </div>
        </div>
    }
    .into_any()
}
