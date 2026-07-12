use leptos::prelude::*;

/// Number of x-axis ticks to render: capped at 5, but never more than the
/// number of data points. A fixed 5 ticks over a sparse (<=4 point) series
/// made several ticks round to the SAME source label index and print a
/// duplicated tick (visual-parity audit finding — `inventory-web`'s Trends
/// line chart showed "2026-06-15 2026-06-15 2026-06-22 2026-06-22 …").
/// `tick_count(len) <= len` guarantees [`tick_label_index`] always lands on
/// a distinct 0-based index for `i in 0..tick_count(len)` (see this
/// module's tests) — no dedup pass needed downstream.
fn tick_count(len: usize) -> usize {
    5usize.min(len.max(1))
}

/// The `0.0..=1.0` position fraction for tick `i` of `tick_count` evenly-
/// spaced ticks. A single tick sits at the start (`0.0`).
fn tick_frac(i: usize, tick_count: usize) -> f64 {
    if tick_count <= 1 {
        0.0
    } else {
        i as f64 / (tick_count - 1) as f64
    }
}

/// Maps a tick's position fraction to a 0-based index into a same-length-as-
/// `data` label list (e.g. `x_labels`), by nearest-rounding.
fn tick_label_index(frac: f64, labels_len: usize) -> usize {
    (frac * labels_len.saturating_sub(1) as f64).round() as usize
}

/// SVG `text-anchor` for tick `i` of `tick_count`: the first tick anchors
/// toward the plot interior (`"start"`) and the last toward it from the
/// other side (`"end"`), so a wide label (e.g. a full `"YYYY-MM-DD"` date)
/// never overflows the SVG viewBox's left/right edge and gets clipped in a
/// screenshot; interior ticks stay centered (`"middle"`).
fn tick_anchor(i: usize, tick_count: usize) -> &'static str {
    if tick_count > 1 && i == 0 {
        "start"
    } else if tick_count > 1 && i == tick_count - 1 {
        "end"
    } else {
        "middle"
    }
}

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

    // Tick count/position/anchor math lives in the pure fns above (visual-
    // parity audit fix — see [`tick_count`]'s doc comment for the bug this
    // replaces: a fixed 5 ticks over a sparse series duplicated labels).
    let n_ticks = tick_count(data.len());
    let x_tick_views = (0..n_ticks)
        .map(|i| {
            let frac = tick_frac(i, n_ticks);
            let sx = pad_left + frac * chart_w;
            let x_pos = format!("{sx:.2}");
            let y_pos = format!("{:.2}", pad_top + chart_h + 15.0);
            let label = if x_labels.is_empty() {
                let val = x_min + frac * x_range;
                format!("{val:.1}")
            } else {
                // Sample the supplied labels at this tick's data-index
                // (`x_labels` is expected to align 1:1 with `data`).
                let idx = tick_label_index(frac, x_labels.len());
                x_labels.get(idx).cloned().unwrap_or_default()
            };
            let anchor = tick_anchor(i, n_ticks);
            view! {
                <text x=x_pos y=y_pos text-anchor=anchor
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_count_caps_at_five_but_never_exceeds_data_len() {
        assert_eq!(tick_count(0), 1); // "No data" branch short-circuits before this, but never 0
        assert_eq!(tick_count(1), 1);
        assert_eq!(tick_count(2), 2);
        assert_eq!(tick_count(3), 3);
        assert_eq!(tick_count(4), 4);
        assert_eq!(tick_count(5), 5);
        assert_eq!(tick_count(6), 5);
        assert_eq!(tick_count(100), 5);
    }

    /// The bug this whole module fixes: with the OLD fixed-5-tick behavior,
    /// a 2- or 3-point series (`inventory-web`'s Trends line chart in the
    /// sparse demo fixture) produced ticks that rounded to the SAME source
    /// label index, printing a duplicated date. `tick_count(len) <= len`
    /// must make every tick's rounded label index DISTINCT for every
    /// realistic series length.
    #[test]
    fn tick_label_indices_are_always_distinct_for_any_series_length() {
        for len in 1..=50usize {
            let n = tick_count(len);
            let mut indices: Vec<usize> = (0..n)
                .map(|i| tick_label_index(tick_frac(i, n), len))
                .collect();
            let before = indices.len();
            indices.dedup();
            assert_eq!(
                indices.len(),
                before,
                "len={len} produced a duplicate tick label index: {indices:?}"
            );
        }
    }

    /// Pins the exact bug from the visual-parity audit: a 2-point series
    /// (`x_labels` = ["2026-06-15", "2026-06-22"]) used to render 5 ticks
    /// that repeated the two labels ("2026-06-15 2026-06-15 2026-06-22
    /// 2026-06-22 2026-06-2[2, clipped]"). It must now render exactly 2
    /// ticks, one per label, each distinct.
    #[test]
    fn two_point_series_renders_exactly_two_distinct_ticks() {
        let len = 2;
        let n = tick_count(len);
        assert_eq!(n, 2);
        let labels = ["2026-06-15", "2026-06-22"];
        let rendered: Vec<&str> = (0..n)
            .map(|i| labels[tick_label_index(tick_frac(i, n), len)])
            .collect();
        assert_eq!(rendered, vec!["2026-06-15", "2026-06-22"]);
    }

    #[test]
    fn tick_frac_single_tick_sits_at_start() {
        assert_eq!(tick_frac(0, 1), 0.0);
    }

    #[test]
    fn tick_frac_spans_zero_to_one_for_multiple_ticks() {
        assert_eq!(tick_frac(0, 5), 0.0);
        assert_eq!(tick_frac(4, 5), 1.0);
        assert_eq!(tick_frac(2, 5), 0.5);
    }

    #[test]
    fn tick_anchor_edges_avoid_viewbox_clipping() {
        assert_eq!(tick_anchor(0, 5), "start");
        assert_eq!(tick_anchor(4, 5), "end");
        assert_eq!(tick_anchor(2, 5), "middle");
        // A single tick has no "edge" to avoid clipping toward -- stays centered.
        assert_eq!(tick_anchor(0, 1), "middle");
    }
}
