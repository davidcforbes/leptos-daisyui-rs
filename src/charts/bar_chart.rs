use leptos::prelude::*;

/// Resolves the fill for bar `index`.
///
/// `overrides` is the optional per-bar color list, which is deliberately
/// **not** required to match `data` in length: an index past its end, or an
/// empty string at that index, falls back to `fallback` (the chart-wide
/// `color`). Bars are always driven by `data`, never by this list, so a
/// mismatched list can neither drop a bar nor panic — a chart that panicked on
/// a length mismatch would take the consumer's whole page down (ldui-jm6).
///
/// The empty-string escape hatch lets a caller override only some bars, e.g.
/// `vec![String::new(), "red".into()]` colors the second bar only.
fn bar_fill<'a>(index: usize, overrides: &'a [String], fallback: &'a str) -> &'a str {
    match overrides.get(index) {
        Some(c) if !c.is_empty() => c,
        _ => fallback,
    }
}

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
    /// Optional per-bar fill overrides, positionally parallel to `data`
    /// (ldui-jm6). Use it to color each bar by its own judgement — e.g. an
    /// above/below-target series where some weeks are on track and some are
    /// behind — instead of painting the whole chart by the series' majority
    /// state.
    ///
    /// The length is NOT required to match `data`. A shorter list colors the
    /// leading bars and the rest fall back to `color`; a longer list has its
    /// surplus entries ignored. An empty string at an index also falls back to
    /// `color`, so only some bars need overriding. The bar count always comes
    /// from `data`, so no mismatch can drop a bar or panic.
    #[prop(optional)]
    bar_colors: Option<Vec<String>>,
    /// If true, render horizontal bars instead of vertical.
    #[prop(default = false)]
    horizontal: bool,
) -> impl IntoView {
    let bar_colors = bar_colors.unwrap_or_default();
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
            .enumerate()
            .map(|(i, (bx, by, bw, bh, lx, ly, vx, label, val))| {
                let c = bar_fill(i, &bar_colors, &color).to_string();
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
            .enumerate()
            .map(|(i, (bx, by, bw, bh, lx, ly, vy, label, val))| {
                let c = bar_fill(i, &bar_colors, &color).to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cols(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn bar_fill_no_overrides_uses_chart_color() {
        assert_eq!(bar_fill(0, &[], "base"), "base");
        assert_eq!(bar_fill(7, &[], "base"), "base");
    }

    #[test]
    fn bar_fill_uses_override_at_index() {
        let o = cols(&["red", "green", "blue"]);
        assert_eq!(bar_fill(0, &o, "base"), "red");
        assert_eq!(bar_fill(1, &o, "base"), "green");
        assert_eq!(bar_fill(2, &o, "base"), "blue");
    }

    #[test]
    fn bar_fill_shorter_list_falls_back_for_out_of_range() {
        // 2 overrides against a 4-bar series: bars 2 and 3 keep the chart color
        // rather than panicking or dropping.
        let o = cols(&["red", "green"]);
        assert_eq!(bar_fill(2, &o, "base"), "base");
        assert_eq!(bar_fill(3, &o, "base"), "base");
    }

    #[test]
    fn bar_fill_longer_list_ignores_surplus() {
        // 5 overrides against a 2-bar series: only the first two are consulted,
        // and asking for them never reads past `data`.
        let o = cols(&["a", "b", "c", "d", "e"]);
        assert_eq!(bar_fill(0, &o, "base"), "a");
        assert_eq!(bar_fill(1, &o, "base"), "b");
    }

    #[test]
    fn bar_fill_empty_entry_falls_back() {
        // Escape hatch: override only some bars by leaving the others blank.
        let o = cols(&["", "green", ""]);
        assert_eq!(bar_fill(0, &o, "base"), "base");
        assert_eq!(bar_fill(1, &o, "base"), "green");
        assert_eq!(bar_fill(2, &o, "base"), "base");
    }

    #[test]
    fn bar_fill_never_panics_across_a_full_series_sweep() {
        // The property that matters: for every bar index a chart would draw,
        // `bar_fill` returns something, whatever the override length.
        let fallback = "base";
        for overrides_len in 0..8usize {
            let o: Vec<String> = (0..overrides_len).map(|i| format!("c{i}")).collect();
            for i in 0..5usize {
                let got = bar_fill(i, &o, fallback);
                assert!(!got.is_empty());
            }
        }
    }
}
