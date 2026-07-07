use leptos::prelude::*;

/// Sparkline component -- an inline mini chart with no axes or labels.
///
/// Ideal for embedding in table cells or KPI cards to show trends at a glance.
///
/// See also [`crate::components::Sparkline`] for a reactive daisyUI-framed component with title and readout row.
#[component]
pub fn Sparkline(
    /// Data values (y-only; x is inferred from index).
    data: Vec<f64>,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 80)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 24)]
    height: u32,
    /// Stroke color for the sparkline.
    #[prop(default = "oklch(0.65 0.2 250)".to_string())]
    color: String,
) -> impl IntoView {
    let viewbox = format!("0 0 {width} {height}");

    if data.len() < 2 {
        return view! {
            <svg viewBox=viewbox class="inline-block align-middle"
                xmlns="http://www.w3.org/2000/svg">
            </svg>
        }
        .into_any();
    }

    let pad: f64 = 2.0;
    let chart_w = width as f64 - pad * 2.0;
    let chart_h = height as f64 - pad * 2.0;

    let y_min = data.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_range = if (y_max - y_min).abs() < f64::EPSILON {
        1.0
    } else {
        y_max - y_min
    };

    let n = data.len();
    let x_step = if n > 1 { chart_w / (n - 1) as f64 } else { 0.0 };

    let points: String = data
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let sx = pad + i as f64 * x_step;
            let sy = pad + chart_h - (val - y_min) / y_range * chart_h;
            format!("{sx:.2},{sy:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <svg viewBox=viewbox class="inline-block align-middle"
            xmlns="http://www.w3.org/2000/svg">
            <polyline
                points=points
                fill="none"
                stroke=color
                stroke-width="1.5"
                stroke-linejoin="round"
                stroke-linecap="round"
            />
        </svg>
    }
    .into_any()
}
