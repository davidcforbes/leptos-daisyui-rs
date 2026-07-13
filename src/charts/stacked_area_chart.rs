use leptos::prelude::*;

use super::stacked_bar_chart::ChartSeries;

/// Estimated on-screen width (SVG user units) of a `font-size="10"` text
/// label, used to lay out the legend row without a fixed per-entry slot. A
/// fixed 100px slot per legend entry overlapped a long series name (e.g.
/// "Inventory Before Submission", 27 chars) into the next swatch/label,
/// rendering as garbled overlapping text (visual-parity audit finding —
/// `inventory-web`'s Trends CFD legend). ~6px/character matches the desktop
/// GUI's own `draw_cfd` legend layout formula
/// (`crates/inventory-gui/src/screens/trends.rs`, `8.0 + name.len() as f32 *
/// 7.0`), which solves the identical fixed-slot overlap problem — there is
/// no text-measurement API available at render time (no live DOM to query
/// before paint).
fn legend_label_width(name: &str) -> f64 {
    name.chars().count() as f64 * 6.0
}

/// The x offset (from `pad_left`) of each legend entry's swatch, laid out
/// left-to-right with each entry's width derived from
/// [`legend_label_width`] rather than a fixed slot.
fn legend_x_offsets(names: &[String]) -> Vec<f64> {
    let mut x = 0.0;
    names
        .iter()
        .map(|name| {
            let offset = x;
            x += 14.0 + legend_label_width(name) + 16.0; // swatch + gap, text, gap to next
            offset
        })
        .collect()
}

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

/// SVG `text-anchor` for the x-axis category label at index `i` of `n`: the
/// first label anchors toward the plot interior (`"start"`) and the last
/// toward it from the other side (`"end"`), so a wide label (e.g. a full
/// `"YYYY-MM-DD"` week-ending date) at the plot's left/right edge never
/// overflows the SVG viewBox and gets clipped in a screenshot (visual-parity
/// audit finding — `inventory-web`'s Trends CFD showed a right-edge
/// "2026-06-2[2]" clip). Interior labels stay centered (`"middle"`). Mirrors
/// `line_chart::tick_anchor`.
fn x_label_anchor(i: usize, n: usize) -> &'static str {
    if n > 1 && i == 0 {
        "start"
    } else if n > 1 && i == n - 1 {
        "end"
    } else {
        "middle"
    }
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

    // Y-axis min/max value labels (bd_4iiz-inventory-toe.5): the desktop CFD
    // prints the max stacked total at the top-left and "0" at the baseline for
    // scale context — the web chart previously showed no y-scale at all.
    let y_axis_label_views = {
        let x_pos = format!("{:.2}", pad_left - 5.0);
        let max_lbl = format!("{max_total:.0}");
        let top_y = format!("{:.2}", pad_top);
        let base_y = format!("{baseline_y:.2}");
        view! {
            <text x=x_pos.clone() y=top_y text-anchor="end" dominant-baseline="hanging"
                fill="currentColor" font-size="10" opacity="0.6">
                {max_lbl}
            </text>
            <text x=x_pos y=base_y text-anchor="end" dominant-baseline="middle"
                fill="currentColor" font-size="10" opacity="0.6">
                "0"
            </text>
        }
    };

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
            let anchor = x_label_anchor(i, n);
            view! {
                <text x=x_str y=y_str text-anchor=anchor
                    fill="currentColor" font-size="10" opacity="0.6">
                    {label.clone()}
                </text>
            }
        })
        .collect_view();

    let legend_y = height as f64 - legend_height;
    let legend_views = if show_legend {
        // Each entry's x offset is DERIVED from the previous entries' actual
        // (estimated) text width via [`legend_x_offsets`], not a fixed 100px
        // slot (visual-parity audit fix — see that fn's doc comment).
        let names: Vec<String> = series.iter().map(|s| s.name.clone()).collect();
        let offsets = legend_x_offsets(&names);
        series
            .iter()
            .zip(offsets)
            .map(|(s, offset)| {
                let lx = pad_left + offset;
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
            {y_axis_label_views}
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

    // --- legend layout (visual-parity audit fix) ---

    #[test]
    fn legend_label_width_scales_with_char_count() {
        assert_eq!(legend_label_width(""), 0.0);
        assert_eq!(legend_label_width("IBS"), 18.0); // 3 chars * 6.0
        assert_eq!(legend_label_width("Submitted"), 54.0); // 9 chars * 6.0
    }

    #[test]
    fn legend_x_offsets_starts_at_zero_and_is_monotonically_increasing() {
        let names = vec!["IBS".to_string(), "Submitted".to_string()];
        let offsets = legend_x_offsets(&names);
        assert_eq!(offsets.len(), 2);
        assert_eq!(offsets[0], 0.0);
        assert!(offsets[1] > offsets[0]);
    }

    /// Pins the exact bug from the visual-parity audit: with the OLD fixed
    /// 100px-per-entry layout, "Inventory Before Submission" (27 chars) at
    /// ~6px/char is ~162px + swatch/gaps wide -- well past the 100px slot --
    /// so the second entry's swatch/label at x=100 would land INSIDE the
    /// first entry's still-rendering text, garbling it
    /// ("Inventory Before Su[chip]Sabmitted"). The second entry's offset
    /// must now clear the first entry's full estimated width.
    #[test]
    fn legend_x_offsets_long_first_label_pushes_second_entry_past_it() {
        let names = vec![
            "Inventory Before Submission".to_string(),
            "Submitted".to_string(),
        ];
        let offsets = legend_x_offsets(&names);
        let first_label_end = 14.0 + legend_label_width(&names[0]); // swatch+gap + text
        assert!(
            offsets[1] >= first_label_end,
            "second legend entry (x={}) must start at/after the first label's \
             estimated end (x={first_label_end}), never overlapping it",
            offsets[1]
        );
        // The old bug: a fixed 100px slot would place the second entry well
        // BEFORE the first (27-char) label's estimated end.
        assert!(
            100.0 < first_label_end,
            "sanity: the old fixed slot did overlap"
        );
    }

    #[test]
    fn legend_x_offsets_empty_input_is_empty() {
        assert!(legend_x_offsets(&[]).is_empty());
    }

    #[test]
    fn x_label_anchor_edges_avoid_viewbox_clipping() {
        // Endpoints anchor inward so a wide date label can't overflow the
        // viewBox edge and clip (the "2026-06-2[2]" bug).
        assert_eq!(x_label_anchor(0, 5), "start");
        assert_eq!(x_label_anchor(4, 5), "end");
        assert_eq!(x_label_anchor(2, 5), "middle");
        // A single label has no edge to avoid -- stays centered.
        assert_eq!(x_label_anchor(0, 1), "middle");
        // Two labels: both are edges.
        assert_eq!(x_label_anchor(0, 2), "start");
        assert_eq!(x_label_anchor(1, 2), "end");
    }
}
