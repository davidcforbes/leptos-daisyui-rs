use leptos::prelude::*;

/// A single slice in a pie chart.
#[derive(Clone, Debug, PartialEq)]
pub struct PieSlice {
    /// Display label for this slice.
    pub label: String,
    /// Numeric value (determines arc proportion).
    pub value: f64,
    /// Fill color for this slice.
    pub color: String,
}

/// SVG-based pie chart component.
///
/// Renders circular arc paths computed with sin/cos. Optionally shows labels.
#[component]
pub fn PieChart(
    /// Slices to render.
    slices: Vec<PieSlice>,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 250)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 250)]
    height: u32,
    /// Whether to show labels next to each slice.
    #[prop(default = true)]
    show_labels: bool,
) -> impl IntoView {
    let viewbox = format!("0 0 {width} {height}");

    if slices.is_empty() {
        return view! {
            <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
                <text x=format!("{}", width / 2) y=format!("{}", height / 2)
                    text-anchor="middle" fill="currentColor" font-size="14">
                    "No data"
                </text>
            </svg>
        }
        .into_any();
    }

    let total: f64 = slices.iter().map(|s| s.value).sum();
    if total.abs() < f64::EPSILON {
        return view! {
            <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
                <text x=format!("{}", width / 2) y=format!("{}", height / 2)
                    text-anchor="middle" fill="currentColor" font-size="14">
                    "No data"
                </text>
            </svg>
        }
        .into_any();
    }

    // If labels are shown, shift the pie to the left to make room for the legend
    let cx = if show_labels {
        width as f64 * 0.35
    } else {
        width as f64 / 2.0
    };
    let cy = height as f64 / 2.0;
    let radius = (cx.min(cy) - 10.0).max(20.0);

    // Build arc path data
    struct ArcData {
        path: String,
        color: String,
    }

    let mut arcs: Vec<ArcData> = Vec::new();
    let mut start_angle: f64 = -std::f64::consts::FRAC_PI_2; // Start at 12 o'clock

    for slice in &slices {
        let fraction = slice.value / total;
        let sweep = fraction * 2.0 * std::f64::consts::PI;

        // Handle the case where a single slice is 100%
        let path = if (fraction - 1.0).abs() < f64::EPSILON {
            // Full circle: draw two semicircles
            let x1 = cx + radius;
            let y1 = cy;
            let x2 = cx - radius;
            let y2 = cy;
            format!(
                "M {x1:.2} {y1:.2} A {r:.2} {r:.2} 0 1 1 {x2:.2} {y2:.2} A {r:.2} {r:.2} 0 1 1 {x1:.2} {y1:.2} Z",
                r = radius,
            )
        } else {
            let end_angle = start_angle + sweep;
            let x1 = cx + radius * start_angle.cos();
            let y1 = cy + radius * start_angle.sin();
            let x2 = cx + radius * end_angle.cos();
            let y2 = cy + radius * end_angle.sin();
            let large_arc = if sweep > std::f64::consts::PI { 1 } else { 0 };

            format!(
                "M {cx:.2} {cy:.2} L {x1:.2} {y1:.2} A {r:.2} {r:.2} 0 {large_arc} 1 {x2:.2} {y2:.2} Z",
                r = radius,
            )
        };

        arcs.push(ArcData {
            path,
            color: slice.color.clone(),
        });

        start_angle += sweep;
    }

    // Legend items (displayed on the right side when show_labels is true)
    let legend_x = width as f64 * 0.65;
    let legend_start_y = cy - (slices.len() as f64 * 20.0) / 2.0;

    let arc_views = arcs
        .into_iter()
        .map(|arc| {
            view! {
                <path d=arc.path fill=arc.color stroke="white" stroke-width="1" />
            }
        })
        .collect_view();

    let legend_views = if show_labels {
        slices
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let ly = legend_start_y + i as f64 * 20.0;
                let lx = format!("{legend_x:.2}");
                let ly_str = format!("{ly:.2}");
                let col = s.color.clone();
                let pct = (s.value / total * 100.0).round();
                let text = format!("{} ({}%)", s.label, pct);
                let text_x = format!("{:.2}", legend_x + 14.0);
                let text_y = ly_str.clone();
                view! {
                    <rect x=lx y=ly_str width="10" height="10" fill=col rx="2" />
                    <text x=text_x y=text_y dominant-baseline="hanging"
                        fill="currentColor" font-size="11">
                        {text}
                    </text>
                }
            })
            .collect_view()
            .into_any()
    } else {
        ().into_any()
    };

    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            {arc_views}
            {legend_views}
        </svg>
    }
    .into_any()
}
