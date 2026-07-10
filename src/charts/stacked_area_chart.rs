use leptos::prelude::*;

use super::stacked_bar_chart::ChartSeries;

/// The number of `(category, value)` pairs usable across all series: the
/// shortest of `categories.len()` and every series' `values.len()`. Extra
/// category labels or series values beyond this length are ignored
/// (truncated) — never padded with zeros.
fn effective_len(categories_len: usize, series: &[ChartSeries]) -> usize {
    series
        .iter()
        .map(|s| s.values.len())
        .fold(categories_len, |acc, l| acc.min(l))
}

/// Computes, for each series (in stacking order), the cumulative sum at each
/// of the first `effective_len` category indices.
///
/// `series[0]`'s cumulative row equals its own values; `series[1]`'s
/// cumulative row is `series[0] + series[1]`, and so on — this gives the
/// *top* boundary of each stacked band directly. The bottom boundary of
/// band `i` is band `i - 1`'s cumulative row (or the zero baseline for
/// `i == 0`).
fn cumulate(series: &[ChartSeries], effective_len: usize) -> Vec<Vec<f64>> {
    let mut running = vec![0.0_f64; effective_len];
    series
        .iter()
        .map(|s| {
            for (i, r) in running.iter_mut().enumerate() {
                *r += s.values.get(i).copied().unwrap_or(0.0);
            }
            running.clone()
        })
        .collect()
}

/// Maps a data index in `0..effective_len` to an SVG x coordinate, evenly
/// spanning the chart width. A single-point series is placed at `pad_left`.
fn x_for_index(index: usize, effective_len: usize, pad_left: f64, chart_w: f64) -> f64 {
    if effective_len <= 1 {
        return pad_left;
    }
    pad_left + (index as f64 / (effective_len - 1) as f64) * chart_w
}

/// Maps a value against `max` to an SVG y coordinate within the chart area.
/// A `max` of (near) zero falls back to a range of `1.0` to avoid dividing
/// by zero.
fn y_for_value(value: f64, max: f64, pad_top: f64, chart_h: f64) -> f64 {
    let range = if max.abs() < f64::EPSILON { 1.0 } else { max };
    pad_top + chart_h - (value / range) * chart_h
}

/// SVG-based stacked area chart component.
///
/// Renders `series` as cumulative, bottom-up filled bands over `categories`
/// on the x-axis: band 0 fills from the baseline to its own cumulative sum,
/// band 1 from band 0's cumulative sum to its own, and so on. The y-axis
/// scales to the largest per-category cumulative total across all bands.
#[component]
pub fn StackedAreaChart(
    /// X-axis category labels (e.g. weeks).
    ///
    /// Values in `series` align to `categories` by index. If a series has
    /// fewer values than `categories`, or series differ in length from each
    /// other, all data truncates to the shortest length found — it is never
    /// padded.
    categories: Vec<String>,
    /// Data series to stack, in stacking order: `series[0]` renders as the
    /// bottom-most band.
    series: Vec<ChartSeries>,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 500)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 220)]
    height: u32,
    /// Whether to render a legend row (swatch + series name) beneath the chart.
    #[prop(default = true)]
    show_legend: bool,
) -> impl IntoView {
    let viewbox = format!("0 0 {width} {height}");

    let n = effective_len(categories.len(), &series);

    if n == 0 || series.is_empty() {
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

    let pad_left: f64 = 40.0;
    let pad_right: f64 = 20.0;
    let pad_top: f64 = 15.0;
    let pad_bottom: f64 = if show_legend { 55.0 } else { 35.0 };
    let legend_height: f64 = 20.0;

    let chart_w = width as f64 - pad_left - pad_right;
    let chart_h = height as f64 - pad_top - pad_bottom;

    let cumulative_rows = cumulate(&series, n);

    let max_total = cumulative_rows
        .last()
        .map(|row| row.iter().cloned().fold(0.0_f64, f64::max))
        .unwrap_or(0.0);

    let baseline_y = pad_top + chart_h;

    let band_views = cumulative_rows
        .iter()
        .enumerate()
        .map(|(si, top_row)| {
            let bottom_row: Option<&Vec<f64>> = if si == 0 {
                None
            } else {
                Some(&cumulative_rows[si - 1])
            };

            let mut points: Vec<String> = Vec::with_capacity(n * 2);
            for (i, val) in top_row.iter().enumerate() {
                let x = x_for_index(i, n, pad_left, chart_w);
                let y = y_for_value(*val, max_total, pad_top, chart_h);
                points.push(format!("{x:.2},{y:.2}"));
            }
            for i in (0..n).rev() {
                let x = x_for_index(i, n, pad_left, chart_w);
                let y = bottom_row
                    .map(|row| y_for_value(row[i], max_total, pad_top, chart_h))
                    .unwrap_or(baseline_y);
                points.push(format!("{x:.2},{y:.2}"));
            }

            let poly_points = points.join(" ");
            let color = series[si].color.clone();
            view! { <polygon points=poly_points fill=color opacity="0.85" /> }
        })
        .collect_view();

    let x_tick_views = categories
        .iter()
        .take(n)
        .enumerate()
        .map(|(i, label)| {
            let x = x_for_index(i, n, pad_left, chart_w);
            let x_str = format!("{x:.2}");
            let y_str = format!("{:.2}", baseline_y + 15.0);
            view! {
                <text x=x_str y=y_str text-anchor="middle"
                    fill="currentColor" font-size="10" opacity="0.6">
                    {label.clone()}
                </text>
            }
        })
        .collect_view();

    let legend_y = height as f64 - legend_height;
    let legend_views = if show_legend {
        series
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
            .collect_view()
            .into_any()
    } else {
        ().into_any()
    };

    let axis_y_end = format!("{baseline_y:.2}");
    let pad_left_str = format!("{pad_left:.2}");
    let axis_x_end = format!("{:.2}", pad_left + chart_w);

    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            <line
                x1=pad_left_str
                y1=axis_y_end.clone()
                x2=axis_x_end
                y2=axis_y_end
                stroke="currentColor"
                stroke-opacity="0.3"
                stroke-width="1"
            />
            {band_views}
            {x_tick_views}
            {legend_views}
        </svg>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(name: &str, values: &[f64], color: &str) -> ChartSeries {
        ChartSeries {
            name: name.to_string(),
            values: values.to_vec(),
            color: color.to_string(),
        }
    }

    #[test]
    fn effective_len_is_shortest_of_categories_and_all_series() {
        let s = vec![
            series("a", &[1.0, 2.0, 3.0], "red"),
            series("b", &[1.0, 2.0], "blue"),
        ];
        assert_eq!(effective_len(5, &s), 2);
    }

    #[test]
    fn effective_len_with_no_series_is_category_len() {
        assert_eq!(effective_len(4, &[]), 4);
    }

    #[test]
    fn effective_len_series_longer_than_categories_truncates() {
        let s = vec![series("a", &[1.0, 2.0, 3.0, 4.0], "red")];
        assert_eq!(effective_len(2, &s), 2);
    }

    #[test]
    fn cumulate_stacks_bottom_up() {
        let s = vec![
            series("a", &[1.0, 2.0], "red"),
            series("b", &[3.0, 4.0], "blue"),
        ];
        let rows = cumulate(&s, 2);
        assert_eq!(rows[0], vec![1.0, 2.0]);
        assert_eq!(rows[1], vec![4.0, 6.0]);
    }

    #[test]
    fn cumulate_truncates_to_effective_len() {
        let s = vec![series("a", &[1.0, 2.0, 3.0], "red")];
        let rows = cumulate(&s, 2);
        assert_eq!(rows[0], vec![1.0, 2.0]);
    }

    #[test]
    fn cumulate_missing_values_treated_as_zero_within_effective_len() {
        // effective_len is computed by the caller; cumulate itself pads a
        // too-short series with 0.0 for any index it's asked to cover.
        let s = vec![series("a", &[1.0], "red")];
        let rows = cumulate(&s, 2);
        assert_eq!(rows[0], vec![1.0, 0.0]);
    }

    #[test]
    fn x_for_index_spans_full_width_for_multi_point() {
        assert_eq!(x_for_index(0, 3, 10.0, 100.0), 10.0);
        assert_eq!(x_for_index(2, 3, 10.0, 100.0), 110.0);
    }

    #[test]
    fn x_for_index_single_point_uses_pad_left() {
        assert_eq!(x_for_index(0, 1, 10.0, 100.0), 10.0);
        assert_eq!(x_for_index(0, 0, 10.0, 100.0), 10.0);
    }

    #[test]
    fn y_for_value_zero_max_defaults_to_unit_range() {
        // max == 0 must not divide by zero; falls back to a range of 1.0
        let y = y_for_value(0.0, 0.0, 0.0, 100.0);
        assert_eq!(y, 100.0);
    }

    #[test]
    fn y_for_value_scales_within_chart_height() {
        let y = y_for_value(50.0, 100.0, 0.0, 100.0);
        assert_eq!(y, 50.0);
    }

    #[test]
    fn y_for_value_max_value_touches_top() {
        let y = y_for_value(100.0, 100.0, 0.0, 100.0);
        assert_eq!(y, 0.0);
    }
}
