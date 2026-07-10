use leptos::prelude::*;

/// SVG-based line chart component.
///
/// Renders a responsive polyline chart with optional dot markers and axis labels.
#[component]
pub fn LineChart(
    /// Data points as (x, y) pairs.
    data: Vec<(f64, f64)>,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 400)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 200)]
    height: u32,
    /// Stroke color for the line. Accepts any CSS color value.
    #[prop(default = "oklch(0.65 0.2 250)".to_string())]
    color: String,
    /// Whether to render circle markers at each data point.
    #[prop(default = true)]
    show_dots: bool,
    /// Optional label for the x-axis.
    #[prop(optional)]
    x_label: Option<String>,
    /// Optional label for the y-axis.
    #[prop(optional)]
    y_label: Option<String>,
    /// Optional categorical x-axis tick labels (e.g. week-ending dates). When
    /// non-empty, the five x ticks show labels sampled evenly from this list
    /// instead of the raw fractional data value — so a chart plotted against a
    /// synthetic 0,1,2… index no longer prints meaningless "0.0/0.2/…" ticks.
    #[prop(optional)]
    x_labels: Vec<String>,
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

    // Padding around the chart area for axes and labels
    let pad_left: f64 = if y_label.is_some() { 60.0 } else { 40.0 };
    let pad_right: f64 = 20.0;
    let pad_top: f64 = 20.0;
    let pad_bottom: f64 = if x_label.is_some() { 50.0 } else { 35.0 };

    let chart_w = width as f64 - pad_left - pad_right;
    let chart_h = height as f64 - pad_top - pad_bottom;

    // Compute data bounds
    let x_min = data.iter().map(|d| d.0).fold(f64::INFINITY, f64::min);
    let x_max = data.iter().map(|d| d.0).fold(f64::NEG_INFINITY, f64::max);
    let y_min = data.iter().map(|d| d.1).fold(f64::INFINITY, f64::min);
    let y_max = data.iter().map(|d| d.1).fold(f64::NEG_INFINITY, f64::max);

    let x_range = if (x_max - x_min).abs() < f64::EPSILON {
        1.0
    } else {
        x_max - x_min
    };
    let y_range = if (y_max - y_min).abs() < f64::EPSILON {
        1.0
    } else {
        y_max - y_min
    };

    // Map data point to SVG coordinates
    let to_svg = |x: f64, y: f64| -> (f64, f64) {
        let sx = pad_left + (x - x_min) / x_range * chart_w;
        let sy = pad_top + chart_h - (y - y_min) / y_range * chart_h;
        (sx, sy)
    };

    // Build polyline points string
    let points: String = data
        .iter()
        .map(|&(x, y)| {
            let (sx, sy) = to_svg(x, y);
            format!("{sx:.2},{sy:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Build dot circle views
    let dot_views = if show_dots {
        data.iter()
            .map(|&(x, y)| {
                let (sx, sy) = to_svg(x, y);
                let cx_str = format!("{sx:.2}");
                let cy_str = format!("{sy:.2}");
                let c = color.clone();
                view! {
                    <circle cx=cx_str cy=cy_str r="3" fill=c />
                }
            })
            .collect_view()
            .into_any()
    } else {
        ().into_any()
    };

    // Axis tick views
    let y_tick_views = (0..=4)
        .map(|i| {
            let frac = i as f64 / 4.0;
            let val = y_min + frac * y_range;
            let sy = pad_top + chart_h - frac * chart_h;
            let x_pos = format!("{:.2}", pad_left - 5.0);
            let y_pos = format!("{sy:.2}");
            let label = format!("{val:.1}");
            view! {
                <text x=x_pos y=y_pos text-anchor="end"
                    dominant-baseline="middle" fill="currentColor"
                    font-size="10" opacity="0.6">
                    {label}
                </text>
            }
        })
        .collect_view();

    let x_tick_views = (0..=4)
        .map(|i| {
            let frac = i as f64 / 4.0;
            let sx = pad_left + frac * chart_w;
            let x_pos = format!("{sx:.2}");
            let y_pos = format!("{:.2}", pad_top + chart_h + 15.0);
            let label = if x_labels.is_empty() {
                let val = x_min + frac * x_range;
                format!("{val:.1}")
            } else {
                // Sample the supplied labels at the same five tick fractions.
                let idx = (frac * (x_labels.len().saturating_sub(1)) as f64).round() as usize;
                x_labels.get(idx).cloned().unwrap_or_default()
            };
            view! {
                <text x=x_pos y=y_pos text-anchor="middle"
                    fill="currentColor" font-size="10" opacity="0.6">
                    {label}
                </text>
            }
        })
        .collect_view();

    let viewbox = format!("0 0 {width} {height}");
    let axis_y_end = format!("{:.2}", pad_top + chart_h);
    let axis_x_end = format!("{:.2}", pad_left + chart_w);
    let pad_left_str = format!("{pad_left:.2}");
    let pad_top_str = format!("{pad_top:.2}");

    let x_label_view = x_label
        .map(|label| {
            let lx = format!("{:.2}", pad_left + chart_w / 2.0);
            let ly = format!("{:.2}", height as f64 - 5.0);
            view! {
                <text x=lx y=ly text-anchor="middle"
                    fill="currentColor" font-size="12">
                    {label}
                </text>
            }
        })
        .into_any();

    let y_label_view = y_label
        .map(|label| {
            let lx = "15.00".to_string();
            let ly = format!("{:.2}", pad_top + chart_h / 2.0);
            let t = format!("rotate(-90, 15, {:.2})", pad_top + chart_h / 2.0);
            view! {
                <text x=lx y=ly text-anchor="middle"
                    fill="currentColor" font-size="12" transform=t>
                    {label}
                </text>
            }
        })
        .into_any();

    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            <line
                x1=pad_left_str.clone()
                y1=pad_top_str
                x2=pad_left_str.clone()
                y2=axis_y_end.clone()
                stroke="currentColor"
                stroke-opacity="0.3"
                stroke-width="1"
            />
            <line
                x1=pad_left_str
                y1=axis_y_end.clone()
                x2=axis_x_end
                y2=axis_y_end
                stroke="currentColor"
                stroke-opacity="0.3"
                stroke-width="1"
            />
            {y_tick_views}
            {x_tick_views}
            <polyline
                points=points
                fill="none"
                stroke=color
                stroke-width="2"
                stroke-linejoin="round"
                stroke-linecap="round"
            />
            {dot_views}
            {x_label_view}
            {y_label_view}
        </svg>
    }
    .into_any()
}
