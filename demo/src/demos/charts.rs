use crate::core::{ContentLayout, Section};
use crate::debug_state;
use leptos::prelude::*;
// The SVG chart family lives in `charts`, not `components`. Imported by name
// rather than glob because `charts` also exports a `Sparkline`, which would
// collide with the reactive daisyUI-framed `components::Sparkline`.
use leptos_daisyui_rs::charts::{
    AreaChart, BarChart, BarChartActivation, BarChartActivationSource, BarChartData, BarChartItem,
    BarChartLayout, BarChartTexts, BarStatus, BarValueFormat, ChartSeries, HeatScale, Heatmap,
    HeatmapActivation, HeatmapActivationSource, HeatmapCategory, HeatmapCell, HeatmapMatrix,
    HeatmapSense, HeatmapTexts, HeatmapValue, LineAxisOptions, LineCategory, LineChart,
    LineChartActivation, LineChartActivationSource, LineChartData, LinePattern, LinePoint,
    LineSeries, LineValueAxis, MarkerShape, MarkerStyle, PieChart, PieSlice, Sparkline,
    StackedAreaChart, StackedBarChart,
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
    // Interactive categorical fixture: reactive so the reorder/remove/restore
    // controls exercise reconciliation-by-key, with every activation mirrored
    // into the debug oracle. The count is written alongside the payload so a
    // duplicated callback cannot pass by overwriting the same key.
    let line_data = RwSignal::new(interactive_line_data());
    let activation_count = RwSignal::new(0_u64);
    let on_line_activate = Callback::new(move |activation: LineChartActivation| {
        let count = activation_count.get_untracked() + 1;
        activation_count.set(count);
        debug_state::set("chart.activation_count", count);
        debug_state::set(
            "chart.activation",
            serde_json::json!({
                "categoryIndex": activation.category_index,
                "categoryKey": activation.category_key,
                "categoryLabel": activation.category_label,
                "preferredSeriesId": activation.preferred_series_id,
                "values": activation
                    .values
                    .iter()
                    .map(|value| serde_json::json!({
                        "seriesId": value.series_id,
                        "seriesName": value.series_name,
                        "value": value.value,
                        "displayValue": value.display_value,
                    }))
                    .collect::<Vec<_>>(),
                "source": match activation.source {
                    LineChartActivationSource::Pointer => "pointer",
                    LineChartActivationSource::Keyboard => "keyboard",
                },
                "modifiers": {
                    "shift": activation.modifiers.shift,
                    "ctrl": activation.modifiers.ctrl,
                    "alt": activation.modifiers.alt,
                    "meta": activation.modifiers.meta,
                },
            }),
        );
    });

    // Typed diverging fixture: reactive so the sort/remove/restore controls
    // exercise reconciliation by key, and the locale control proves the copy
    // reacts EN -> ES -> EN without disturbing keys, values or order.
    let bar_data = RwSignal::new(divergence_data());
    let bar_spanish = RwSignal::new(false);
    let bar_texts = Signal::derive(move || {
        if bar_spanish.get() {
            spanish_bar_texts()
        } else {
            BarChartTexts::default()
        }
    });
    let bar_activation_count = RwSignal::new(0_u64);
    let on_bar_activate = Callback::new(move |activation: BarChartActivation| {
        let count = bar_activation_count.get_untracked() + 1;
        bar_activation_count.set(count);
        debug_state::set("bar_chart.activation_count", count);
        debug_state::set(
            "bar_chart.activation",
            serde_json::json!({
                "categoryKey": activation.category_key,
                "categoryLabel": activation.category_label,
                "value": activation.value,
                "displayValue": activation.display_value,
                "status": match activation.status {
                    BarStatus::Neutral => "neutral",
                    BarStatus::Favorable => "favorable",
                    BarStatus::Unfavorable => "unfavorable",
                },
                "source": match activation.source {
                    BarChartActivationSource::Pointer => "pointer",
                    BarChartActivationSource::Keyboard => "keyboard",
                },
                "modifiers": {
                    "shift": activation.modifiers.shift,
                    "ctrl": activation.modifiers.ctrl,
                    "alt": activation.modifiers.alt,
                    "meta": activation.modifiers.meta,
                },
            }),
        );
    });

    // Typed heatmap fixture: reactive so the sort/remove/clear controls
    // exercise reconciliation by row key AND by column key, with the locale
    // control proving every framework-owned word changes while the keys, the
    // intensities and the activated identity do not.
    let heatmap_data = RwSignal::new(office_kpi_matrix());
    let heatmap_spanish = RwSignal::new(false);
    let heatmap_texts = Signal::derive(move || {
        if heatmap_spanish.get() {
            spanish_heatmap_texts()
        } else {
            office_heatmap_texts()
        }
    });
    let heatmap_activation_count = RwSignal::new(0_u64);
    let on_heatmap_activate = Callback::new(move |activation: HeatmapActivation| {
        let count = heatmap_activation_count.get_untracked() + 1;
        heatmap_activation_count.set(count);
        debug_state::set("heatmap.activation_count", count);
        debug_state::set(
            "heatmap.activation",
            serde_json::json!({
                "rowKey": activation.row_key,
                "rowLabel": activation.row_label,
                "columnKey": activation.column_key,
                "columnLabel": activation.column_label,
                "intensity": activation.intensity,
                "displayValue": activation.display_value,
                "sense": match activation.sense {
                    HeatmapSense::Neutral => "neutral",
                    HeatmapSense::Favorable => "favorable",
                    HeatmapSense::Unfavorable => "unfavorable",
                },
                "source": match activation.source {
                    HeatmapActivationSource::Pointer => "pointer",
                    HeatmapActivationSource::Keyboard => "keyboard",
                },
                "modifiers": {
                    "shift": activation.modifiers.shift,
                    "ctrl": activation.modifiers.ctrl,
                    "alt": activation.modifiers.alt,
                    "meta": activation.modifiers.meta,
                },
            }),
        );
    });

    view! {
        <ContentLayout
            title="Charts"
            description="Dependency-free SVG charts -- line, area, bar, stacked bar, stacked area, pie, heatmap and a bare inline sparkline. Pure Leptos markup with primitive props: no canvas, no JS charting library, and every colour accepts a daisyUI theme token."
        >
            <Section title="LineChart" col=true>
                <p class="text-sm opacity-70">
                    "Categorical data keeps each named series aligned to the same weeks, renders gaps without joining them, and pairs solid, dashed, and dotted lines with circle, square, and diamond markers. The responsive legend and screen-reader table carry the same labels and values."
                </p>
                <div class="w-full max-w-2xl">
                    <LineChart
                        data=line_data
                        accessible_label="Weekly resolution trend".to_string()
                        description="Actual, rolling average, and target resolution counts by week.".to_string()
                        width=560
                        height=260
                        on_point_activate=on_line_activate
                    />
                </div>
                <div class="flex flex-wrap gap-2">
                    <button class="btn btn-sm" data-testid="line-chart-reorder"
                        on:click=move |_| line_data.update(|data| *data = reorder_line_data(data))>
                        "Reorder data"
                    </button>
                    <button class="btn btn-sm" data-testid="line-chart-remove"
                        on:click=move |_| line_data.update(|data| *data = remove_week(data, "week-08"))>
                        "Remove active week"
                    </button>
                    <button class="btn btn-sm" data-testid="line-chart-restore"
                        on:click=move |_| line_data.set(interactive_line_data())>
                        "Restore data"
                    </button>
                    <button class="btn btn-sm" data-testid="line-chart-gaps"
                        on:click=move |_| line_data.set(gapped_line_data())>
                        "Show gaps"
                    </button>
                </div>
                <p class="text-sm opacity-70">
                    "Without an activation callback the same chart keeps descriptive group semantics: hover and keyboard navigation still show the card, but nothing claims button behavior."
                </p>
                <div class="w-full max-w-xl">
                    <LineChart
                        data=static_line_data()
                        accessible_label="Weekly intake trend".to_string()
                        description="Intake and staffed-capacity counts by week.".to_string()
                        width=480
                        height=200
                    />
                </div>
                <p class="text-sm text-base-content/75">
                    "A series can opt onto a second value axis, so counts and a duration read together instead of the duration flatlining against the counts. Each axis computes its own domain and declares its own unit once: the right-hand ticks, the hover card, the legend captions and the screen-reader table all state a series against the axis it belongs to. Series that say nothing stay on the primary axis, and the right axis is drawn only when something is assigned to it."
                </p>
                <div class="w-full max-w-2xl" data-testid="dual-axis-line-chart">
                    <LineChart
                        data=dual_axis_line_data()
                        accessible_label="Conversations and first response by week".to_string()
                        description="Opened, resolved and abandoned conversation counts against average first response time.".to_string()
                        primary_axis=LineAxisOptions::default().with_label("Conversations")
                        secondary_axis=LineAxisOptions::default()
                            .with_label("First response")
                            .with_unit(" s")
                            .with_decimals(1)
                        width=560
                        height=260
                    />
                </div>

                <p class="text-sm opacity-70">
                    "For the preserved numeric XY chart, x_labels replace raw fractional x values, and the tick count is capped at the number of data points so a sparse series cannot print a duplicated date. Axis titles are optional."
                </p>
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

                <p class="text-sm text-base-content/75">
                    "Typed data replaces both positional vectors at once: each BarChartItem carries its own stable key, localized label, signed value, formatted display text, caller-owned status and optional colour, so sorting the rows cannot pair a value with a neighbour's judgement. layout=BarChartLayout::DivergingHorizontal draws every bar from one visible zero rule -- negative left, non-negative right, equal magnitudes equal length -- and a missing measurement draws no bar rather than a fabricated zero. Status is the caller's: an outcome measure judges, an activity measure stays neutral, and judged bars carry a solid or dashed end cap so the distinction survives forced colours."
                </p>
                <div class="w-full max-w-2xl" data-testid="diverging-bar-chart">
                    <BarChart
                        data=bar_data
                        layout=BarChartLayout::DivergingHorizontal
                        accessible_label="Current minus trailing baseline by office".to_string()
                        description="Signed delta to the trailing 12-week baseline, most dragging first.".to_string()
                        value_format=BarValueFormat::default().with_unit(" pts")
                        texts=bar_texts
                        width=560
                        height=260
                        on_bar_activate=on_bar_activate
                    />
                </div>
                <div class="flex flex-wrap gap-2">
                    <button class="btn btn-sm" data-testid="bar-chart-sort"
                        on:click=move |_| bar_data.update(|data| *data = sorted_divergence(data))>
                        "Sort most dragging first"
                    </button>
                    <button class="btn btn-sm" data-testid="bar-chart-remove"
                        on:click=move |_| bar_data.update(|data| *data = remove_office(data, "north"))>
                        "Remove North"
                    </button>
                    <button class="btn btn-sm" data-testid="bar-chart-restore"
                        on:click=move |_| bar_data.set(divergence_data())>
                        "Restore data"
                    </button>
                    <button class="btn btn-sm" data-testid="bar-chart-locale"
                        on:click=move |_| bar_spanish.update(|spanish| *spanish = !*spanish)>
                        "Toggle locale"
                    </button>
                </div>

                <p class="text-sm text-base-content/75">
                    "The same layout with an activity measure -- a count of things that happened, where neither direction is a verdict. Every item is neutral, so no bar is capped or coloured by judgement, and without an activation callback the chart claims no button role at all: it still navigates and describes itself, but nothing pretends to be actionable."
                </p>
                <div class="w-full max-w-2xl" data-testid="neutral-bar-chart">
                    <BarChart
                        data=activity_data()
                        layout=BarChartLayout::DivergingHorizontal
                        accessible_label="Net change in open matters by office".to_string()
                        width=560
                        height=220
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

                <p class="text-sm text-base-content/75">
                    "Typed data replaces both label vectors and the positional cells at once: rows and columns are HeatmapCategory values carrying a stable key beside a localized label, and each HeatmapValue names the row and column it belongs to rather than a pair of array indices. The grid is then named, described, and restated as a real row/column matrix for a screen reader, every cell is reachable by arrow keys (Home/End along the row, Ctrl+Home/Ctrl+End to the grid corners), and an activation reports the office and the KPI by key. A judged cell also carries a solid or dashed sense rule, so the verdict survives forced colours."
                </p>
                <div class="w-full max-w-3xl" data-testid="typed-heatmap">
                    <Heatmap
                        data=heatmap_data
                        scale=HeatScale::Judgement
                        accessible_label="Current versus baseline by office and KPI".to_string()
                        description="Signed deviation from the trailing 12-week baseline; lower-is-better measures are negated by the caller.".to_string()
                        texts=heatmap_texts
                        slant_col_labels=true
                        pad_left=80.0
                        max_cell_h=44.0
                        width=760
                        height=320
                        on_cell_activate=on_heatmap_activate
                    />
                </div>
                <div class="flex flex-wrap gap-2">
                    <button
                        class="btn btn-sm"
                        data-testid="heatmap-sort"
                        on:click=move |_| heatmap_data.update(|data| *data = sorted_offices(data))
                    >
                        "Sort offices worst first"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="heatmap-remove-column"
                        on:click=move |_| heatmap_data.update(|data| *data = remove_kpi(data, "sla"))
                    >
                        "Remove SLA column"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="heatmap-clear"
                        on:click=move |_| heatmap_data.set(HeatmapMatrix::default())
                    >
                        "Clear data"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="heatmap-restore"
                        on:click=move |_| heatmap_data.set(office_kpi_matrix())
                    >
                        "Restore data"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="heatmap-locale"
                        on:click=move |_| heatmap_spanish.update(|spanish| *spanish = !*spanish)
                    >
                        "Toggle locale"
                    </button>
                </div>

                <p class="text-sm text-base-content/75">
                    "The consumer's own shape: one office by twelve KPIs, with no activation callback. It still names and describes itself and still publishes the full matrix as a data table, but it claims no button role and adds no tab stop -- a heatmap that only reports is not an interactive control."
                </p>
                <div class="w-full max-w-3xl" data-testid="single-office-heatmap">
                    <Heatmap
                        data=single_office_matrix()
                        scale=HeatScale::Judgement
                        accessible_label="North office scorecard".to_string()
                        texts=office_heatmap_texts()
                        slant_col_labels=true
                        pad_left=80.0
                        max_cell_h=44.0
                        width=760
                        height=160
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

/// The signed decomposition shape the bead describes: one selected measure's
/// current value minus its trailing baseline, per office. Deliberately carries
/// a negative, an exact zero, a positive, and a missing measurement, plus a
/// pair of equal magnitudes with opposite signs so the symmetry is visible.
fn divergence_data() -> BarChartData {
    BarChartData::categorical(vec![
        BarChartItem::new("north", "North", -12.5)
            .with_display_value("-12.5 pts")
            .with_status(BarStatus::Unfavorable),
        BarChartItem::new("harbour", "Harbour", -4.0).with_status(BarStatus::Unfavorable),
        BarChartItem::missing("riverside", "Riverside"),
        BarChartItem::new("central", "Central", 0.0),
        BarChartItem::new("east", "East", 4.0).with_status(BarStatus::Favorable),
        BarChartItem::new("west", "West", 9.5).with_status(BarStatus::Favorable),
    ])
}

/// The caller owns the sort, not the chart. Most dragging first: the lowest
/// signed delta at the top, missing measurements last.
fn sorted_divergence(data: &BarChartData) -> BarChartData {
    let BarChartData::Categorical(items) = data else {
        return data.clone();
    };
    let mut items = items.clone();
    items.sort_by(|a, b| match (a.value, b.value) {
        (Some(a), Some(b)) => a.total_cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    BarChartData::Categorical(items)
}

/// Removes one office by key, so the removal journey can prove focus moves
/// predictably rather than vanishing.
fn remove_office(data: &BarChartData, key: &str) -> BarChartData {
    let BarChartData::Categorical(items) = data else {
        return data.clone();
    };
    BarChartData::Categorical(
        items
            .iter()
            .filter(|item| item.key != key)
            .cloned()
            .collect(),
    )
}

/// The activity-neutral variant: signed values with no judgement attached,
/// because a net change in open matters is not good or bad on its own.
fn activity_data() -> BarChartData {
    BarChartData::categorical(vec![
        BarChartItem::new("north", "North", -6.0),
        BarChartItem::new("harbour", "Harbour", -2.0),
        BarChartItem::new("central", "Central", 2.0),
        BarChartItem::new("east", "East", 6.0),
    ])
}

/// Every string the chart produces itself, in a second locale. Switching to
/// these must change the words and nothing else.
fn spanish_bar_texts() -> BarChartTexts {
    BarChartTexts {
        empty: "Sin datos".to_string(),
        category_header: "Categoria".to_string(),
        value_header: "Valor".to_string(),
        status_header: "Estado".to_string(),
        no_value: "Sin dato".to_string(),
        status_neutral: "Neutral".to_string(),
        status_favorable: "Favorable".to_string(),
        status_unfavorable: "Desfavorable".to_string(),
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
                    let mut point =
                        LinePoint::new(value).with_display_value(format!("{value:.0} resolved"));
                    if index == 0 {
                        point.marker_color = Some("var(--color-success)".to_string());
                    }
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
            fill: Some("var(--color-primary)".to_string()),
            stroke_width: 1.0,
        },
        show_data_labels: true,
        axis: LineValueAxis::Primary,
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
        axis: LineValueAxis::Primary,
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
        axis: LineValueAxis::Primary,
    }
}

/// Reverses category order (and every series' aligned points) so the
/// reconciliation journeys can prove active/focused state follows a
/// category's *key* through a reorder, not its index.
fn reorder_line_data(data: &LineChartData) -> LineChartData {
    match data {
        LineChartData::Categorical { categories, series } => LineChartData::categorical(
            categories.iter().rev().cloned().collect(),
            series
                .iter()
                .map(|series| {
                    let mut series = series.clone();
                    series.points.reverse();
                    series
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Removes one category (and every series' aligned point) by key. The demo
/// button removes `week-08` — the same category the removal journey holds
/// active — so the card-close/focus-move behavior is deterministic.
fn remove_week(data: &LineChartData, key: &str) -> LineChartData {
    match data {
        LineChartData::Categorical { categories, series } => {
            let Some(removed) = categories.iter().position(|category| category.key == key) else {
                return data.clone();
            };
            let mut categories = categories.clone();
            categories.remove(removed);
            LineChartData::categorical(
                categories,
                series
                    .iter()
                    .map(|series| {
                        let mut series = series.clone();
                        if removed < series.points.len() {
                            series.points.remove(removed);
                        }
                        series
                    })
                    .collect(),
            )
        }
        other => other.clone(),
    }
}

/// Deterministic multi-gap variant for the `Show gaps` control and the
/// missing-data visual baseline: several interior gaps per series, which the
/// renderer must segment around rather than bridge.
fn gapped_line_data() -> LineChartData {
    let LineChartData::Categorical { categories, series } = interactive_line_data() else {
        unreachable!("interactive fixture is categorical");
    };
    LineChartData::categorical(
        categories,
        series
            .into_iter()
            .map(|mut series| {
                let gaps: &[usize] = match series.id.as_str() {
                    "actual" => &[2, 3, 6, 10],
                    "rolling-average" => &[7, 8],
                    _ => &[4],
                };
                for &index in gaps {
                    if index < series.points.len() {
                        series.points[index] = LinePoint::missing();
                    }
                }
                series
            })
            .collect(),
    )
}

/// The Office Conversations Reporting shape (ldui-j0mt): three count series
/// against one duration series three orders of magnitude smaller. Deliberately
/// deterministic, and deliberately includes a gap in the duration series so
/// the secondary axis is exercised with missing data too.
fn dual_axis_line_data() -> LineChartData {
    let counts = [
        (
            "opened",
            "Opened",
            "var(--color-primary)",
            [120.0, 138.0, 151.0, 144.0, 162.0, 158.0],
        ),
        (
            "resolved",
            "Resolved",
            "var(--color-secondary)",
            [112.0, 129.0, 140.0, 139.0, 155.0, 151.0],
        ),
        (
            "abandoned",
            "Abandoned",
            "var(--color-warning)",
            [8.0, 9.0, 11.0, 5.0, 7.0, 7.0],
        ),
    ];
    let mut series = counts
        .into_iter()
        .map(|(id, name, color, values)| {
            LineSeries::new(
                id,
                name,
                color,
                values.into_iter().map(LinePoint::new).collect(),
            )
        })
        .collect::<Vec<_>>();
    let durations = [
        Some(41.2),
        Some(38.7),
        None,
        Some(29.4),
        Some(24.8),
        Some(22.1),
    ];
    let mut first_response = LineSeries::new(
        "first-response",
        "Average first response",
        "var(--color-accent)",
        durations
            .into_iter()
            .map(|value| value.map(LinePoint::new).unwrap_or_else(LinePoint::missing))
            .collect(),
    )
    .on_secondary_axis();
    first_response.pattern = LinePattern::Dashed;
    first_response.marker = MarkerStyle {
        shape: MarkerShape::Diamond,
        ..MarkerStyle::default()
    };
    series.push(first_response);

    LineChartData::categorical(
        (1..=6)
            .map(|week| LineCategory {
                key: format!("conv-week-{week:02}"),
                label: format!("W{week:02}"),
            })
            .collect(),
        series,
    )
}

/// Small callback-less categorical fixture: proves descriptive `group`
/// semantics (no false button roles) on a chart with no activation callback.
fn static_line_data() -> LineChartData {
    LineChartData::categorical(
        (1..=4)
            .map(|week| LineCategory {
                key: format!("intake-{week:02}"),
                label: format!("W{week:02}"),
            })
            .collect(),
        vec![
            LineSeries::new(
                "intake",
                "Intake",
                "var(--color-primary)",
                vec![
                    LinePoint::new(12.0),
                    LinePoint::new(15.0),
                    LinePoint::new(11.0),
                    LinePoint::new(17.0),
                ],
            ),
            LineSeries::new(
                "capacity",
                "Staffed capacity",
                "var(--color-secondary)",
                vec![
                    LinePoint::new(14.0),
                    LinePoint::new(14.0),
                    LinePoint::new(16.0),
                    LinePoint::new(16.0),
                ],
            ),
        ],
    )
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

/// The twelve KPI columns the consumer's dashboard carries, each with a stable
/// id beside its localized label. Deliberately a `HeatmapCategory` list rather
/// than a `Vec<String>` of labels: the id is what an activation reports, so
/// re-ordering or hiding a column cannot re-point a drill.
fn kpi_columns() -> Vec<HeatmapCategory> {
    [
        ("closed", "Matters closed"),
        ("sla", "SLA met"),
        ("handle", "Handle time"),
        ("intake", "Intake conversion"),
        ("backlog", "Backlog age"),
        ("first-touch", "First touch"),
        ("reopened", "Reopened"),
        ("billable", "Billable hours"),
        ("no-show", "No shows"),
        ("escalated", "Escalated"),
        ("satisfaction", "Satisfaction"),
        ("overdue", "Overdue tasks"),
    ]
    .iter()
    .map(|(key, label)| HeatmapCategory::new(*key, *label))
    .collect()
}

/// A deterministic signed deviation for `(office, kpi)`, spread across the
/// whole `-1.0..=1.0` ramp so both hues, both sense rules and an exact zero are
/// all visible at once.
fn kpi_intensity(office: usize, kpi: usize) -> f64 {
    let raw = ((office * 7 + kpi * 5) % 21) as f64 / 10.0 - 1.0;
    (raw * 10.0).round() / 10.0
}

/// Builds one office's row of cells. `gap` names the KPI that office did not
/// report, which stays a missing cell rather than becoming a fabricated zero.
fn office_row(office: usize, row_key: &str, gap: Option<&str>) -> Vec<HeatmapValue> {
    kpi_columns()
        .into_iter()
        .enumerate()
        .map(|(index, column)| {
            if gap == Some(column.key.as_str()) {
                return HeatmapValue::missing(row_key, column.key);
            }
            let intensity = kpi_intensity(office, index);
            let percent = (intensity * 20.0).round() as i64;
            HeatmapValue::new(row_key, column.key.clone(), intensity)
                .with_display_value(format!("{percent:+}%"))
                .with_accessible_value(format!("{percent:+} percent versus the 12-week baseline"))
        })
        .collect()
}

/// The multirow fixture: three offices by twelve KPIs, one reported gap.
fn office_kpi_matrix() -> HeatmapMatrix {
    let rows = vec![
        HeatmapCategory::new("north", "North"),
        HeatmapCategory::new("south", "South"),
        HeatmapCategory::new("east", "East"),
    ];
    let values = rows
        .iter()
        .enumerate()
        .flat_map(|(index, row)| {
            let gap = (row.key == "south").then_some("handle");
            office_row(index, &row.key, gap)
        })
        .collect();
    HeatmapMatrix::new(rows, kpi_columns(), values)
}

/// The consumer's exact shape (op-dlfua.7.35): one office by twelve KPIs.
fn single_office_matrix() -> HeatmapMatrix {
    HeatmapMatrix::new(
        vec![HeatmapCategory::new("north", "North")],
        kpi_columns(),
        office_row(0, "north", Some("reopened")),
    )
}

/// The caller owns the sort, not the chart: worst mean deviation at the top.
/// Only the row axis moves -- the values name their row by key, so none of them
/// has to be touched.
fn sorted_offices(data: &HeatmapMatrix) -> HeatmapMatrix {
    let mean = |key: &str| {
        let measured: Vec<f64> = data
            .values
            .iter()
            .filter(|value| value.row_key == key)
            .filter_map(|value| value.intensity)
            .collect();
        if measured.is_empty() {
            0.0
        } else {
            measured.iter().sum::<f64>() / measured.len() as f64
        }
    };
    let mut rows = data.rows.clone();
    rows.sort_by(|a, b| mean(&a.key).total_cmp(&mean(&b.key)));
    HeatmapMatrix::new(rows, data.columns.clone(), data.values.clone())
}

/// Drops one KPI column by key. Its values are left in place: a value naming a
/// column the axis no longer carries is simply not rendered, which is what
/// makes hiding a column a one-line change for a consumer.
fn remove_kpi(data: &HeatmapMatrix, key: &str) -> HeatmapMatrix {
    HeatmapMatrix::new(
        data.rows.clone(),
        data.columns
            .iter()
            .filter(|column| column.key != key)
            .cloned()
            .collect(),
        data.values.clone(),
    )
}

/// The framework copy named for this page's domain, so the axis names a reader
/// hears are "Office" and "KPI" rather than the generic defaults.
fn office_heatmap_texts() -> HeatmapTexts {
    HeatmapTexts {
        data_table_caption: "Office by KPI deviation from baseline".to_string(),
        row_header: "Office".to_string(),
        column_header: "KPI".to_string(),
        value_header: "Deviation".to_string(),
        missing_value: "Not reported".to_string(),
        ..HeatmapTexts::default()
    }
}

/// The same copy in Spanish. Every field is framework- or page-owned; no key,
/// intensity or caller-supplied label appears here, which is the point of the
/// EN -> ES -> EN journey.
fn spanish_heatmap_texts() -> HeatmapTexts {
    HeatmapTexts {
        no_data: "Sin datos".to_string(),
        data_table_caption: "Desviacion por oficina e indicador".to_string(),
        row_header: "Oficina".to_string(),
        column_header: "Indicador".to_string(),
        value_header: "Desviacion".to_string(),
        missing_value: "No reportado".to_string(),
        sense_favorable: "Favorable".to_string(),
        sense_unfavorable: "Desfavorable".to_string(),
        sense_neutral: "Neutral".to_string(),
    }
}

fn spark_values() -> Vec<f64> {
    vec![12.0, 18.0, 15.0, 22.0, 19.0, 27.0, 24.0, 31.0]
}
