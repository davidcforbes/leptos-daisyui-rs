use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
// The SVG chart family lives in `charts`, not `components`. Imported by name
// rather than glob because `charts` also exports a `Sparkline`, which would
// collide with the reactive daisyUI-framed `components::Sparkline`.
use leptos_daisyui_rs::charts::{
    AreaChart, BarChart, ChartSeries, HeatScale, Heatmap, HeatmapCell, LineCategory, LineChart,
    LineChartData, LinePattern, LinePoint, LineSeries, MarkerShape, MarkerStyle, PieChart,
    PieSlice, Sparkline, StackedAreaChart, StackedBarChart,
};

/// Every chart in `leptos_daisyui_rs::charts`, each shown with at least one
/// non-default prop combination (ldui-40g).
///
/// Colours here are daisyUI theme tokens (`var(--color-primary)` and friends)
/// rather than the charts' literal `oklch(...)` defaults, for two reasons. It
/// is what a consuming app actually does — a chart that ignores the active
/// theme looks pasted on — and it is the exact path `charts::paint` protects
/// (ldui-1g5): a token must reach the DOM through `style`, never through a
/// `fill=`/`stroke=` presentation attribute. This page is therefore also the
/// browser-visible evidence for that fix.
#[component]
pub fn ChartsDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Charts"
            description="Dependency-free SVG charts -- line, area, bar, stacked bar, stacked area, pie, heatmap and a bare inline sparkline. Pure Leptos markup with primitive props: no canvas, no JS charting library, and every colour accepts a daisyUI theme token."
        >
            <Section title="LineChart" col=true>
                <p class="text-sm opacity-70">
                    "Categorical x_labels replace the raw fractional x values, and the tick count is capped at the number of data points so a sparse series cannot print a duplicated date. Axis titles are optional."
                </p>
                <div class="w-full max-w-2xl">
                    <LineChart
                        data=interactive_line_data()
                        accessible_label="Weekly resolution trend".to_string()
                        description="Actual, rolling average, and target resolution counts by week.".to_string()
                        width=560
                        height=260
                    />
                </div>
                <div class="w-full max-w-xl">
                    <LineChart
                        data=weekly_series()
                        x_labels=week_labels()
                        x_label="Week ending".to_string()
                        y_label="Closed".to_string()
                        color="var(--color-primary)".to_string()
                        width=480
                        height=220
                    />
                </div>

                <p class="text-sm opacity-70">
                    "minimal=true drops the y-axis, the y-scale and both axis titles, keeping a single baseline, square markers and a value label at each endpoint -- the desktop face's trend chart."
                </p>
                <div class="w-full max-w-xl">
                    <LineChart
                        data=weekly_series()
                        x_labels=week_labels()
                        minimal=true
                        color="var(--color-accent)".to_string()
                        width=480
                        height=160
                    />
                </div>
            </Section>

            <Section title="AreaChart" col=true>
                <p class="text-sm opacity-70">
                    "fill_color and stroke_color are separate props, so the band and its edge can be tuned independently; fill_opacity keeps the band from swamping the gridlines."
                </p>
                <div class="w-full max-w-xl">
                    <AreaChart
                        data=weekly_series()
                        fill_color="var(--color-info)".to_string()
                        stroke_color="var(--color-info)".to_string()
                        fill_opacity=0.25
                        width=480
                        height=200
                    />
                </div>
            </Section>

            <Section title="BarChart" col=true>
                <p class="text-sm opacity-70">
                    "bar_colors is an optional list positionally parallel to data, so each bar can carry its own judgement -- weeks at or above target in success, weeks behind in error -- instead of painting the whole chart by the series' majority state. The list need not match data in length: short lists fall back to color, surplus entries are ignored, and the bar count always comes from data."
                </p>
                <div class="w-full max-w-xl">
                    <BarChart data=closed_by_week() bar_colors=closed_by_week_colors() height=180 />
                </div>

                <p class="text-sm opacity-70">
                    "Two overrides against a four-bar series -- bars 3 and 4 fall back to the chart-wide color rather than panicking or vanishing:"
                </p>
                <div class="w-full max-w-xl">
                    <BarChart
                        data=vec![
                            ("Mon".to_string(), 4.0),
                            ("Tue".to_string(), 7.0),
                            ("Wed".to_string(), 5.0),
                            ("Thu".to_string(), 9.0),
                        ]
                        bar_colors=vec![
                            "var(--color-error)".to_string(),
                            "var(--color-success)".to_string(),
                        ]
                        height=180
                    />
                </div>

                <p class="text-sm opacity-70">
                    "horizontal=true swaps the axes and moves the category labels into a left gutter -- the right shape when the labels are words rather than dates."
                </p>
                <div class="w-full max-w-xl">
                    <BarChart
                        data=queue_depth()
                        horizontal=true
                        color="var(--color-secondary)".to_string()
                        height=180
                    />
                </div>
            </Section>

            <Section title="StackedBarChart" col=true>
                <p class="text-sm opacity-70">
                    "Multi-series: each ChartSeries carries its own colour and one value per category, stacked bottom-up in the order supplied. A series shorter than categories contributes zero at the missing indices rather than shifting the stack."
                </p>
                <div class="w-full max-w-xl">
                    <StackedBarChart
                        categories=week_labels()
                        series=workflow_series()
                        width=480
                        height=250
                    />
                </div>
            </Section>

            <Section title="StackedAreaChart" col=true>
                <p class="text-sm opacity-70">
                    "The same series as continuous bands -- a cumulative-flow view. The legend lays each entry out from the previous label's estimated width instead of a fixed slot, so a long series name cannot overlap its neighbour."
                </p>
                <div class="w-full max-w-2xl">
                    <StackedAreaChart
                        categories=week_labels()
                        series=workflow_series()
                        width=520
                        height=240
                    />
                </div>

                <p class="text-sm opacity-70">
                    "show_legend=false reclaims the legend row when the series are already named elsewhere:"
                </p>
                <div class="w-full max-w-2xl">
                    <StackedAreaChart
                        categories=week_labels()
                        series=workflow_series()
                        show_legend=false
                        width=520
                        height=180
                    />
                </div>
            </Section>

            <Section title="PieChart" col=true>
                <p class="text-sm opacity-70">
                    "Each PieSlice carries its own colour and value; percentages in the legend are derived from the values, never passed in. A single 100% slice draws as a full circle rather than a degenerate arc."
                </p>
                <div class="w-full max-w-md">
                    <PieChart slices=channel_slices() width=340 height=220 />
                </div>

                <p class="text-sm opacity-70">
                    "show_labels=false drops the legend and re-centres the pie -- for a chart captioned by its surroundings:"
                </p>
                <div class="w-full max-w-xs">
                    <PieChart slices=channel_slices() show_labels=false width=200 height=200 />
                </div>
            </Section>

            <Section title="Heatmap" col=true>
                <p class="text-sm opacity-70">
                    "scale=HeatScale::Judgement makes intensity signed: the sign picks the hue (success above target, error below) and the magnitude still picks the alpha. The hues default to the daisyUI --color-success and --color-error theme tokens, so no new colour enters the palette. Sense is the caller's sign convention and therefore per-column: 'Handle time' below is a lower-is-better measure, so its deviation is negated before being passed in."
                </p>
                <div class="w-full max-w-2xl">
                    <Heatmap
                        row_labels=office_rows()
                        col_labels=kpi_cols()
                        cells=kpi_cells()
                        scale=HeatScale::Judgement
                        pad_left=80.0
                        max_cell_h=48.0
                        height=200
                    />
                </div>

                <p class="text-sm opacity-70">
                    "The default scale is unchanged -- a single hue whose alpha carries magnitude only:"
                </p>
                <div class="w-full max-w-2xl">
                    <Heatmap
                        row_labels=office_rows()
                        col_labels=kpi_cols()
                        cells=magnitude_cells()
                        pad_left=80.0
                        max_cell_h=48.0
                        height=200
                    />
                </div>
            </Section>

            <Section title="Sparkline (bare, inline)" col=true>
                <p class="text-sm opacity-70">
                    "charts::Sparkline is the unframed primitive -- axis-less, label-less, sized to sit inside a table cell or a sentence. See /components/sparkline for the reactive daisyUI-framed component built on the same idea."
                </p>
                <p class="text-sm">
                    "Throughput this week: "
                    <Sparkline data=spark_values() color="var(--color-success)".to_string() />
                    " (trending up)"
                </p>
            </Section>
        </ContentLayout>
    }
}

/// Week-ending labels shared by the line, stacked-bar and stacked-area
/// examples, so the three read as views of one dataset.
fn week_labels() -> Vec<String> {
    ["2026-07-06", "2026-07-13", "2026-07-20", "2026-07-27"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// A plain (x, y) series plotted against a synthetic index -- the shape that
/// makes `x_labels` necessary.
fn weekly_series() -> Vec<(f64, f64)> {
    vec![(0.0, 18.0), (1.0, 24.0), (2.0, 21.0), (3.0, 31.0)]
}

/// Deterministic categorical fixture for the interactive `LineChart` demo.
/// It deliberately includes a missing interior actual, a short target series,
/// host-formatted values, labels, and three redundant paint/shape patterns.
fn interactive_line_data() -> LineChartData {
    LineChartData::categorical(
        (1..=14)
            .map(|week| LineCategory {
                key: format!("week-{week:02}"),
                label: format!("W{week:02}"),
            })
            .collect(),
        vec![actual_series(), rolling_average_series(), target_series()],
    )
}

fn actual_series() -> LineSeries {
    let values = [
        42.0, 45.0, 44.0, 49.0, 52.0, 50.0, 0.0, 55.0, 58.0, 57.0, 61.0, 64.0, 62.0, 67.0,
    ];
    LineSeries {
        id: "actual".to_string(),
        name: "Actual".to_string(),
        points: values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                if index == 6 {
                    LinePoint::missing()
                } else {
                    let point =
                        LinePoint::new(value).with_display_value(format!("{value:.0} resolved"));
                    if index == 0 || index == 13 {
                        point.with_data_label(format!("{value:.0}"))
                    } else {
                        point
                    }
                }
            })
            .collect(),
        color: "var(--color-primary)".to_string(),
        pattern: LinePattern::Solid,
        marker: MarkerStyle {
            shape: MarkerShape::Circle,
            size: 4.0,
            fill: None,
            stroke_width: 1.0,
        },
        show_data_labels: true,
    }
}

fn rolling_average_series() -> LineSeries {
    let values = [
        43.0, 44.0, 45.0, 47.0, 49.0, 50.0, 51.0, 53.0, 55.0, 57.0, 59.0, 61.0, 63.0, 65.0,
    ];
    LineSeries {
        id: "rolling-average".to_string(),
        name: "Rolling average".to_string(),
        points: values
            .into_iter()
            .map(|value| LinePoint::new(value).with_display_value(format!("{value:.1} average")))
            .collect(),
        color: "var(--color-secondary)".to_string(),
        pattern: LinePattern::Dashed,
        marker: MarkerStyle {
            shape: MarkerShape::Square,
            size: 3.5,
            fill: None,
            stroke_width: 1.0,
        },
        show_data_labels: false,
    }
}

fn target_series() -> LineSeries {
    let values = [48.0, 48.0, 50.0, 50.0, 52.0, 52.0, 54.0, 54.0, 56.0, 56.0];
    LineSeries {
        id: "target".to_string(),
        name: "Target".to_string(),
        points: values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                let point = LinePoint::new(value).with_display_value(format!("Target {value:.0}"));
                if index == 9 {
                    point.with_data_label("56 target")
                } else {
                    point
                }
            })
            .collect(),
        color: "var(--color-accent)".to_string(),
        pattern: LinePattern::Dotted,
        marker: MarkerStyle {
            shape: MarkerShape::Diamond,
            size: 4.0,
            fill: None,
            stroke_width: 1.0,
        },
        show_data_labels: true,
    }
}

/// Weekly closed-work counts for the per-bar-colour example.
fn closed_by_week() -> Vec<(String, f64)> {
    vec![
        ("W31".to_string(), 18.0),
        ("W32".to_string(), 24.0),
        ("W33".to_string(), 11.0),
        ("W34".to_string(), 27.0),
        ("W35".to_string(), 9.0),
    ]
}

/// Per-bar judgement for [`closed_by_week`]: at or above the target of 20 is
/// favourable, below it is not. One colour per bar, derived from the same
/// comparison a consumer would already be making.
fn closed_by_week_colors() -> Vec<String> {
    closed_by_week()
        .iter()
        .map(|(_, v)| {
            if *v >= 20.0 {
                "var(--color-success)".to_string()
            } else {
                "var(--color-error)".to_string()
            }
        })
        .collect()
}

/// Word-labelled categories, where a horizontal bar chart reads better than a
/// vertical one.
fn queue_depth() -> Vec<(String, f64)> {
    vec![
        ("Intake".to_string(), 42.0),
        ("Review".to_string(), 28.0),
        ("Awaiting client".to_string(), 15.0),
        ("Ready to file".to_string(), 7.0),
    ]
}

/// Three workflow stages over [`week_labels`], shared by the stacked bar and
/// stacked area examples.
fn workflow_series() -> Vec<ChartSeries> {
    vec![
        ChartSeries {
            name: "Intake".to_string(),
            values: vec![12.0, 15.0, 11.0, 18.0],
            color: "var(--color-primary)".to_string(),
        },
        ChartSeries {
            name: "In review".to_string(),
            values: vec![8.0, 9.0, 13.0, 10.0],
            color: "var(--color-secondary)".to_string(),
        },
        ChartSeries {
            name: "Awaiting client".to_string(),
            values: vec![5.0, 4.0, 7.0, 6.0],
            color: "var(--color-accent)".to_string(),
        },
    ]
}

/// Intake channels for the pie example, coloured from the theme tokens.
fn channel_slices() -> Vec<PieSlice> {
    vec![
        PieSlice {
            label: "Referral".to_string(),
            value: 42.0,
            color: "var(--color-primary)".to_string(),
        },
        PieSlice {
            label: "Web".to_string(),
            value: 31.0,
            color: "var(--color-secondary)".to_string(),
        },
        PieSlice {
            label: "Phone".to_string(),
            value: 18.0,
            color: "var(--color-accent)".to_string(),
        },
        PieSlice {
            label: "Walk-in".to_string(),
            value: 9.0,
            color: "var(--color-info)".to_string(),
        },
    ]
}

fn office_rows() -> Vec<String> {
    ["North", "South", "East"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn kpi_cols() -> Vec<String> {
    ["Closed", "SLA met", "Handle time"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// KPI-by-office cells on the signed judgement axis. Columns 0 and 1 are
/// higher-is-better; column 2 (handle time) is lower-is-better, so its
/// deviation is negated -- the sense lives in the sign, per column.
fn kpi_cells() -> Vec<HeatmapCell> {
    // (row, col, label, signed intensity)
    let raw = [
        (0, 0, "+12%", 0.6),
        (0, 1, "+4%", 0.2),
        (0, 2, "-8%", 0.4),
        (1, 0, "-9%", -0.45),
        (1, 1, "-2%", -0.1),
        (1, 2, "+21%", -1.0),
        (2, 0, "+1%", 0.05),
        (2, 1, "+18%", 0.9),
        (2, 2, "0%", 0.0),
    ];
    raw.iter()
        .map(|(row, col, label, intensity)| HeatmapCell {
            row: *row,
            col: *col,
            label: label.to_string(),
            intensity: *intensity,
        })
        .collect()
}

/// The same grid on the default magnitude scale -- absolute deviation only,
/// single hue, judgement not expressible in colour.
fn magnitude_cells() -> Vec<HeatmapCell> {
    kpi_cells()
        .into_iter()
        .map(|c| HeatmapCell {
            intensity: c.intensity.abs(),
            ..c
        })
        .collect()
}

fn spark_values() -> Vec<f64> {
    vec![12.0, 18.0, 15.0, 22.0, 19.0, 27.0, 24.0, 31.0]
}
