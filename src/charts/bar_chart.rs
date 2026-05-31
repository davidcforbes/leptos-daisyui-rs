use leptos::prelude::*;

/// SVG-based bar chart component.
///
/// Renders vertical or horizontal bars with category labels.
#[component]
pub fn BarChart(
    /// Data as (label, value) pairs.
    data: Vec<(String, f64)>,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 400)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 200)]
    height: u32,
    /// Fill color for the bars.
    #[prop(default = "oklch(0.65 0.2 250)".to_string())]
    color: String,
    /// If true, render horizontal bars instead of vertical.
    #[prop(default = false)]
    horizontal: bool,
) -> impl IntoView {
    if data.is_empty() {
        return view! {
            <svg
                viewBox=format!("0 0 {width} {height}")
                class="w-full h-auto"
                xmlns="http://www.w3.org/2000/svg"
            >
                <text x=format!("{}", width / 2) y=format!("{}", height / 2)
                    text-anchor="middle" fill="currentColor" font-size="14">
                    "No data"
                </text>
            </svg>
        }
        .into_any();
    }

    let viewbox = format!("0 0 {width} {height}");

    let v_max = data
        .iter()
        .map(|d| d.1)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);
    let v_range = if v_max.abs() < f64::EPSILON {
        1.0
    } else {
        v_max
    };

    let n = data.len();

    if horizontal {
        // Horizontal bars
        let pad_left: f64 = 80.0;
        let pad_right: f64 = 30.0;
        let pad_top: f64 = 10.0;
        let pad_bottom: f64 = 10.0;
        let chart_w = width as f64 - pad_left - pad_right;
        let chart_h = height as f64 - pad_top - pad_bottom;
        let bar_height = (chart_h / n as f64) * 0.7;
        let gap = (chart_h / n as f64) * 0.3;

        let bars: Vec<_> = data
            .iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let bw = (value / v_range) * chart_w;
                let by = pad_top + i as f64 * (bar_height + gap) + gap / 2.0;
                let bx = pad_left;
                let label_y = by + bar_height / 2.0;
                let value_x = bx + bw + 5.0;
                let val_text = format!("{value:.1}");
                let label_text = label.clone();
                (
                    format!("{bx:.2}"),
                    format!("{by:.2}"),
                    format!("{bw:.2}"),
                    format!("{bar_height:.2}"),
                    format!("{:.2}", pad_left - 5.0),
                    format!("{label_y:.2}"),
                    format!("{value_x:.2}"),
                    label_text,
                    val_text,
                )
            })
            .collect();

        let bar_views = bars
            .into_iter()
            .map(|(bx, by, bw, bh, lx, ly, vx, label, val)| {
                let c = color.clone();
                let vy = ly.clone();
                view! {
                    <rect x=bx y=by width=bw height=bh fill=c rx="2" />
                    <text x=lx y=vy text-anchor="end" dominant-baseline="middle"
                        fill="currentColor" font-size="11">
                        {label}
                    </text>
                    <text x=vx y=ly dominant-baseline="middle"
                        fill="currentColor" font-size="10" opacity="0.7">
                        {val}
                    </text>
                }
            })
            .collect_view();

        view! {
            <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
                {bar_views}
            </svg>
        }
        .into_any()
    } else {
        // Vertical bars
        let pad_left: f64 = 40.0;
        let pad_right: f64 = 10.0;
        let pad_top: f64 = 10.0;
        let pad_bottom: f64 = 40.0;
        let chart_w = width as f64 - pad_left - pad_right;
        let chart_h = height as f64 - pad_top - pad_bottom;
        let bar_width = (chart_w / n as f64) * 0.7;
        let gap = (chart_w / n as f64) * 0.3;

        let bars: Vec<_> = data
            .iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let bh = (value / v_range) * chart_h;
                let bx = pad_left + i as f64 * (bar_width + gap) + gap / 2.0;
                let by = pad_top + chart_h - bh;
                let label_x = bx + bar_width / 2.0;
                let label_y = pad_top + chart_h + 15.0;
                let val_y = by - 5.0;
                let val_text = format!("{value:.1}");
                let label_text = label.clone();
                (
                    format!("{bx:.2}"),
                    format!("{by:.2}"),
                    format!("{bar_width:.2}"),
                    format!("{bh:.2}"),
                    format!("{label_x:.2}"),
                    format!("{label_y:.2}"),
                    format!("{val_y:.2}"),
                    label_text,
                    val_text,
                )
            })
            .collect();

        let baseline_y = format!("{:.2}", pad_top + chart_h);

        let bar_views = bars
            .into_iter()
            .map(|(bx, by, bw, bh, lx, ly, vy, label, val)| {
                let c = color.clone();
                let vx = lx.clone();
                view! {
                    <rect x=bx y=by width=bw height=bh fill=c rx="2" />
                    <text x=lx y=ly text-anchor="middle"
                        fill="currentColor" font-size="11">
                        {label}
                    </text>
                    <text x=vx y=vy text-anchor="middle"
                        fill="currentColor" font-size="10" opacity="0.7">
                        {val}
                    </text>
                }
            })
            .collect_view();

        view! {
            <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
                <line
                    x1=format!("{pad_left:.2}")
                    y1=baseline_y.clone()
                    x2=format!("{:.2}", pad_left + chart_w)
                    y2=baseline_y
                    stroke="currentColor"
                    stroke-opacity="0.3"
                    stroke-width="1"
                />
                {bar_views}
            </svg>
        }
        .into_any()
    }
}
