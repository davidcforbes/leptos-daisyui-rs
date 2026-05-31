use leptos::prelude::*;

/// A single data series within a stacked bar chart.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSeries {
    /// Display name for the series (shown in the legend).
    pub name: String,
    /// Values for each category, ordered to match the `categories` vec.
    pub values: Vec<f64>,
    /// Fill color for this series.
    pub color: String,
}

/// SVG-based stacked bar chart component.
///
/// Renders vertically stacked bars for each category with a legend.
#[component]
pub fn StackedBarChart(
    /// Category labels along the x-axis.
    categories: Vec<String>,
    /// Data series to stack. Each series must have `values.len() == categories.len()`.
    series: Vec<ChartSeries>,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 400)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 250)]
    height: u32,
) -> impl IntoView {
    let viewbox = format!("0 0 {width} {height}");

    if categories.is_empty() || series.is_empty() {
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

    let pad_left: f64 = 45.0;
    let pad_right: f64 = 10.0;
    let pad_top: f64 = 10.0;
    let pad_bottom: f64 = 55.0; // extra room for labels + legend
    let legend_height: f64 = 20.0;

    let chart_w = width as f64 - pad_left - pad_right;
    let chart_h = height as f64 - pad_top - pad_bottom;

    let n = categories.len();

    // Compute max stacked total across categories
    let max_total: f64 = (0..n)
        .map(|ci| {
            series
                .iter()
                .map(|s| s.values.get(ci).copied().unwrap_or(0.0))
                .sum::<f64>()
        })
        .fold(0.0_f64, f64::max);
    let v_range = if max_total.abs() < f64::EPSILON {
        1.0
    } else {
        max_total
    };

    let bar_width = (chart_w / n as f64) * 0.7;
    let gap = (chart_w / n as f64) * 0.3;

    // Build rect data for every segment
    struct Segment {
        x: String,
        y: String,
        w: String,
        h: String,
        color: String,
    }

    let mut segments: Vec<Segment> = Vec::new();
    let mut cat_labels: Vec<(String, String)> = Vec::new();

    for (ci, cat_label) in categories.iter().enumerate().take(n) {
        let bx = pad_left + ci as f64 * (bar_width + gap) + gap / 2.0;
        let label_x = bx + bar_width / 2.0;

        cat_labels.push((format!("{label_x:.2}"), cat_label.clone()));

        let mut cumulative = 0.0_f64;
        for s in &series {
            let val = s.values.get(ci).copied().unwrap_or(0.0);
            if val <= 0.0 {
                cumulative += val;
                continue;
            }
            let seg_h = (val / v_range) * chart_h;
            let by = pad_top + chart_h - cumulative / v_range * chart_h - seg_h;

            segments.push(Segment {
                x: format!("{bx:.2}"),
                y: format!("{by:.2}"),
                w: format!("{bar_width:.2}"),
                h: format!("{seg_h:.2}"),
                color: s.color.clone(),
            });

            cumulative += val;
        }
    }

    // Legend items
    let legend_y = height as f64 - legend_height;

    let baseline_y = format!("{:.2}", pad_top + chart_h);
    let label_y = format!("{:.2}", pad_top + chart_h + 15.0);

    let segment_views = segments
        .into_iter()
        .map(|seg| {
            view! {
                <rect x=seg.x y=seg.y width=seg.w height=seg.h fill=seg.color rx="1" />
            }
        })
        .collect_view();

    let cat_label_views = cat_labels
        .into_iter()
        .map(|(cx, label)| {
            let ly = label_y.clone();
            view! {
                <text x=cx y=ly text-anchor="middle"
                    fill="currentColor" font-size="11">
                    {label}
                </text>
            }
        })
        .collect_view();

    let legend_views = series
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let lx = pad_left + i as f64 * 100.0;
            let lx_str = format!("{lx:.2}");
            let ly_str = format!("{legend_y:.2}");
            let col = s.color.clone();
            let name = s.name.clone();
            let text_x = format!("{:.2}", lx + 14.0);
            let text_y = ly_str.clone();
            view! {
                <rect x=lx_str y=ly_str width="10" height="10" fill=col rx="2" />
                <text x=text_x y=text_y dominant-baseline="hanging"
                    fill="currentColor" font-size="10">
                    {name}
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
            {segment_views}
            {cat_label_views}
            {legend_views}
        </svg>
    }
    .into_any()
}
