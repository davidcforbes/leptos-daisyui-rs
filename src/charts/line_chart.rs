use super::paint::{merge_style, paint_attrs, stroke_attrs};
use leptos::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

mod geometry;
mod interaction;
mod normalize;
mod types;
pub use types::{
    LineCategory, LineChartActivation, LineChartActivationSource, LineChartActivationValue,
    LineChartData, LineChartDataSource, LineChartModifiers, LineInteractionMode, LineLegendMode,
    LinePattern, LinePoint, LineSeries, MarkerShape, MarkerStyle,
};

/// Per-instance sequence for categorical SVG title, description, tooltip, and
/// focus-target IDs. Multiple charts can therefore coexist without ARIA ids
/// colliding, following the same pattern as `RosterGrid`.
static LINE_CHART_SEQ: AtomicU64 = AtomicU64::new(0);

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
///
/// Shared with [`super::area_chart`] rather than copied: a centered first tick
/// also overlaps the y-axis scale label sitting at the plot's bottom-left
/// corner, which is how `AreaChart` failed the layout audit's hard overlap
/// check (ldui-40g). `StackedAreaChart` reaches the same rule from its own
/// category-label geometry (`x_label_anchor`).
pub(super) fn tick_anchor(i: usize, tick_count: usize) -> &'static str {
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
    /// Static or reactive chart data; legacy `(x, y)` vectors convert automatically.
    #[prop(into)]
    data: LineChartDataSource,
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
    /// Minimal "sparkline" mode matching the desktop GUI's Trends line chart
    /// (bd_4iiz-inventory-toe.5): drops the vertical y-axis, the multi-tick
    /// y-scale, and both axis TITLE labels, keeping only a single bottom
    /// baseline, the endpoint x-labels, small square markers, and a value
    /// label printed next to the first and last data points. `false` keeps the
    /// full-axis chart.
    #[prop(default = false)]
    minimal: bool,
    /// Controls whether categorical data shows a legend; defaults to automatic.
    #[prop(default = LineLegendMode::Auto)]
    legend_mode: LineLegendMode,
    /// Controls categorical point interaction; defaults to automatic.
    #[prop(default = LineInteractionMode::Auto)]
    interaction_mode: LineInteractionMode,
    /// Accessible name for the categorical chart; defaults to `Line chart`.
    #[prop(default = "Line chart".to_string())]
    accessible_label: String,
    /// Optional longer description for categorical chart consumers.
    #[prop(optional)]
    description: Option<String>,
    /// Whether categorical data includes its accessible table; defaults to true.
    #[prop(default = true)]
    show_data_table: bool,
    /// Optional callback invoked by a categorical point activation.
    #[prop(optional)]
    on_point_activate: Option<Callback<LineChartActivation>>,
) -> impl IntoView {
    let data = Memo::new(move |_| data.get());
    let instance = LINE_CHART_SEQ.fetch_add(1, Ordering::Relaxed);

    move || match data.get() {
        LineChartData::XY(data) => render_xy(
            data,
            width,
            height,
            color.clone(),
            show_dots,
            x_label.clone(),
            y_label.clone(),
            x_labels.clone(),
            minimal,
        )
        .into_any(),
        LineChartData::Categorical { categories, series } => render_categorical(
            categories,
            series,
            width,
            height,
            legend_mode,
            interaction_mode,
            accessible_label.clone(),
            description.clone(),
            show_data_table,
            on_point_activate.clone(),
            instance,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_categorical(
    categories: Vec<LineCategory>,
    series: Vec<LineSeries>,
    width: u32,
    height: u32,
    legend_mode: LineLegendMode,
    interaction_mode: LineInteractionMode,
    accessible_label: String,
    description: Option<String>,
    show_data_table: bool,
    on_point_activate: Option<Callback<LineChartActivation>>,
    instance: u64,
) -> AnyView {
    use geometry::{
        Projection, dasharray, marker_size, marker_stroke_width, path_segments, plot_bounds,
        svg_number, visible_tick_indices,
    };
    use normalize::normalize_categorical;

    let chart = normalize_categorical(&categories, &series);
    let Some(domain) = chart.domain else {
        return render_empty_categorical(&chart, accessible_label, show_data_table, instance);
    };
    if chart.categories.is_empty() || chart.series.is_empty() {
        return render_empty_categorical(&chart, accessible_label, show_data_table, instance);
    }

    let max_marker_radius = chart
        .series
        .iter()
        .map(|series| marker_size(&series.marker) + marker_stroke_width(&series.marker) / 2.0)
        .fold(0.0_f64, f64::max);
    let has_data_labels = chart.series.iter().any(|series| series.show_data_labels);
    let bounds = plot_bounds(
        width as f64,
        height as f64,
        max_marker_radius,
        has_data_labels,
    );
    let projection = Projection {
        bounds,
        category_count: chart.categories.len(),
        domain,
    };
    let plot_width = (bounds.right - bounds.left).max(1.0);
    // Task 5 replaces this deterministic viewBox-width initial value with a
    // measured CSS width after mount.
    let tick_indices = visible_tick_indices(chart.categories.len(), width as f64, 48.0);
    let title_id = format!("line-chart-{instance}-title");
    let desc_id = format!("line-chart-{instance}-desc");
    let tooltip_id = format!("line-chart-{instance}-tooltip");
    let first_category_key = chart.categories[0].key.clone();
    let preferred_series = chart
        .series
        .iter()
        .find(|series| series.points.iter().any(|point| point.value.is_some()))
        .map(|series| series.id.clone())
        .unwrap_or_default();
    let show_legend = match legend_mode {
        LineLegendMode::Auto => chart.series.len() >= 2,
        LineLegendMode::Always => true,
        LineLegendMode::Never => false,
    };
    let interaction_enabled = match interaction_mode {
        LineInteractionMode::Auto | LineInteractionMode::Enabled => true,
        LineInteractionMode::Disabled => false,
    };
    let _ = on_point_activate;

    let grid_lines = (0..=4)
        .map(|index| {
            let y = bounds.top + (bounds.bottom - bounds.top) * index as f64 / 4.0;
            view! {
                <line x1=svg_number(bounds.left) y1=svg_number(y) x2=svg_number(bounds.right) y2=svg_number(y)
                    stroke="currentColor" stroke-opacity="0.14" stroke-width="1" />
            }
        })
        .collect_view();
    let y_ticks = (0..=4)
        .map(|index| {
            let fraction = index as f64 / 4.0;
            let value = domain.max - (domain.max - domain.min) * fraction;
            let y = bounds.top + (bounds.bottom - bounds.top) * fraction;
            view! {
                <text x=svg_number(bounds.left - 6.0) y=svg_number(y) text-anchor="end" dominant-baseline="middle"
                    fill="currentColor" font-size="12" opacity="0.65">
                    {format!("{value:.1}")}
                </text>
            }
        })
        .collect_view();
    let x_ticks = tick_indices
        .into_iter()
        .map(|index| {
            let category = &chart.categories[index];
            let anchor = tick_anchor(index, chart.categories.len());
            view! {
                <text x=svg_number(projection.category_x(index)) y=svg_number(bounds.bottom + 18.0)
                    text-anchor=anchor fill="currentColor" font-size="12" opacity="0.7">
                    {category.label.clone()}
                </text>
            }
        })
        .collect_view();

    let line_paths = chart
        .series
        .iter()
        .flat_map(|series| {
            let (stroke, stroke_style) = stroke_attrs(series.color.clone());
            let dash = dasharray(&series.pattern);
            path_segments(series, projection).into_iter().map(move |d| {
                view! {
                    <path d=d data-series-id=series.id.clone() fill="none" stroke=stroke.clone() style=stroke_style.clone()
                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray=dash.clone() />
                }
            })
        })
        .collect_view();
    let mut marker_views = Vec::new();
    let mut data_label_views = Vec::new();
    for series in &chart.series {
        for (index, point) in series.points.iter().enumerate() {
            let Some(value) = point.value else {
                continue;
            };
            marker_views.push(marker_view(
                &series.id,
                index,
                &chart.categories[index].key,
                projection.category_x(index),
                projection.value_y(value),
                series,
                point.marker_color.as_deref(),
            ));
            if series.show_data_labels
                && let Some(label) = point.data_label.clone()
            {
                let anchor = tick_anchor(index, chart.categories.len());
                let (fill, style) = paint_attrs(series.color.clone());
                data_label_views.push(view! {
                    <text x=svg_number(projection.category_x(index)) y=svg_number(projection.value_y(value) - marker_size(&series.marker) - 5.0)
                        text-anchor=anchor fill=fill style=style font-size="12" font-weight="600">
                        {label}
                    </text>
                });
            }
        }
    }
    let markers = marker_views.collect_view();
    let data_labels = data_label_views.collect_view();
    let focus_targets = interaction_enabled.then(|| {
        chart
            .categories
            .iter()
            .enumerate()
            .filter(|(index, _)| chart.series.iter().any(|series| series.points[*index].value.is_some()))
            .map(|(index, category)| {
                let values = chart
                    .series
                    .iter()
                    .filter_map(|series| series.points[index].value.map(|value| format!("{} {value}", series.name)))
                    .collect::<Vec<_>>()
                    .join(", ");
                view! {
                    <rect id=format!("line-chart-{instance}-category-{index}") x=svg_number(projection.category_x(index) - plot_width / (chart.categories.len().max(2) - 1) as f64 / 2.0)
                        y=svg_number(bounds.top) width=svg_number(plot_width / (chart.categories.len().max(2) - 1) as f64)
                        height=svg_number(bounds.bottom - bounds.top) fill="transparent" role="button"
                        tabindex=if index == 0 { "0" } else { "-1" }
                        aria-describedby=tooltip_id.clone() aria-label=format!("{}: {values}", category.label)
                        data-category-index=index data-category-key=category.key.clone()
                        data-active-category=first_category_key.clone() data-preferred-series=preferred_series.clone() />
                }
            })
            .collect_view()
    });
    let overlay = interaction_enabled.then(|| {
        view! {
            <rect data-line-chart-pointer-overlay="" x=svg_number(bounds.left) y=svg_number(bounds.top)
                width=svg_number(plot_width) height=svg_number(bounds.bottom - bounds.top) fill="transparent" pointer-events="all" />
        }
    });
    let legend = show_legend.then(|| {
        chart
            .series
            .iter()
            .map(|series| legend_entry(series))
            .collect_view()
    });
    let description =
        description.unwrap_or_else(|| "Categorical multi-series line chart".to_string());
    let viewbox = format!("0 0 {width} {height}");

    view! {
        <div data-testid="interactive-line-chart" role="group" aria-label=accessible_label.clone()
            data-active-category=first_category_key.clone() data-preferred-series=preferred_series class="w-full">
            {show_legend.then(|| view! {
                <div data-line-chart-legend class="flex flex-wrap gap-x-4 gap-y-2 text-sm" aria-label="Chart legend">
                    {legend}
                </div>
            })}
            <div data-line-chart-stage class="relative mt-2">
                <svg data-line-chart-plot role="img" aria-labelledby=format!("{title_id} {desc_id}")
                    viewBox=viewbox class="h-auto w-full" xmlns="http://www.w3.org/2000/svg">
                    <title id=title_id.clone()>{accessible_label.clone()}</title>
                    <desc id=desc_id.clone()>{description}</desc>
                    {grid_lines}
                    <line x1=svg_number(bounds.left) y1=svg_number(bounds.bottom) x2=svg_number(bounds.right) y2=svg_number(bounds.bottom)
                        stroke="currentColor" stroke-opacity="0.35" stroke-width="1" />
                    <line x1=svg_number(bounds.left) y1=svg_number(bounds.top) x2=svg_number(bounds.left) y2=svg_number(bounds.bottom)
                        stroke="currentColor" stroke-opacity="0.35" stroke-width="1" />
                    {y_ticks}
                    {x_ticks}
                    {line_paths}
                    {markers}
                    {data_labels}
                    {focus_targets}
                    {overlay}
                </svg>
                <div data-testid="line-chart-tooltip" id=tooltip_id role="tooltip" class="hidden text-xs"></div>
            </div>
            {show_data_table.then(|| categorical_table(&chart, accessible_label.clone()))}
        </div>
    }
    .into_any()
}

fn marker_view(
    series_id: &str,
    category_index: usize,
    category_key: &str,
    x: f64,
    y: f64,
    series: &normalize::NormalizedSeries,
    point_marker_color: Option<&str>,
) -> AnyView {
    use geometry::{marker_size, marker_stroke_width, svg_number};

    let size = marker_size(&series.marker);
    let (fill, fill_style) = paint_attrs(
        series
            .marker
            .fill
            .as_deref()
            .or(point_marker_color)
            .unwrap_or(&series.color)
            .to_string(),
    );
    let (stroke, stroke_style) = stroke_attrs(series.color.clone());
    let style = merge_style([fill_style, stroke_style]);
    let common = (
        series_id.to_string(),
        category_index,
        category_key.to_string(),
        fill,
        stroke,
        style,
        svg_number(marker_stroke_width(&series.marker)),
    );
    match series.marker.shape {
        MarkerShape::None => ().into_any(),
        MarkerShape::Circle => view! {
            <circle cx=svg_number(x) cy=svg_number(y) r=svg_number(size) fill=common.3 stroke=common.4 style=common.5 stroke-width=common.6
                data-series-id=common.0 data-category-index=common.1 data-category-key=common.2 data-marker-shape="circle" />
        }
        .into_any(),
        MarkerShape::Square => view! {
            <rect x=svg_number(x - size) y=svg_number(y - size) width=svg_number(size * 2.0) height=svg_number(size * 2.0)
                fill=common.3 stroke=common.4 style=common.5 stroke-width=common.6
                data-series-id=common.0 data-category-index=common.1 data-category-key=common.2 data-marker-shape="square" />
        }
        .into_any(),
        MarkerShape::Diamond => view! {
            <path d=format!("M {} {} L {} {} L {} {} L {} {} Z", svg_number(x), svg_number(y - size), svg_number(x + size), svg_number(y), svg_number(x), svg_number(y + size), svg_number(x - size), svg_number(y))
                fill=common.3 stroke=common.4 style=common.5 stroke-width=common.6
                data-series-id=common.0 data-category-index=common.1 data-category-key=common.2 data-marker-shape="diamond" />
        }
        .into_any(),
    }
}

fn legend_entry(series: &normalize::NormalizedSeries) -> AnyView {
    use geometry::{dasharray, marker_size, svg_number};
    let (stroke, stroke_style) = stroke_attrs(series.color.clone());
    let (fill, fill_style) = paint_attrs(
        series
            .marker
            .fill
            .as_deref()
            .unwrap_or(&series.color)
            .to_string(),
    );
    let marker_style = merge_style([fill_style, stroke_style.clone()]);
    let dash = dasharray(&series.pattern);
    let marker = match series.marker.shape {
        MarkerShape::None => ().into_any(),
        MarkerShape::Circle => view! {
            <circle cx="14" cy="7" r=svg_number(marker_size(&series.marker).min(3.5)) fill=fill stroke=stroke.clone() style=marker_style stroke-width="1" />
        }
        .into_any(),
        MarkerShape::Square => view! {
            <rect x="11" y="4" width="6" height="6" fill=fill stroke=stroke.clone() style=marker_style stroke-width="1" />
        }
        .into_any(),
        MarkerShape::Diamond => view! {
            <path d="M 14 3 L 18 7 L 14 11 L 10 7 Z" fill=fill stroke=stroke.clone() style=marker_style stroke-width="1" />
        }
        .into_any(),
    };
    view! {
        <span data-series-id=series.id.clone() class="inline-flex items-center gap-2 whitespace-nowrap">
            <svg data-line-chart-pattern-swatch="" aria-hidden="true" viewBox="0 0 28 14" class="h-4 w-7">
                <line x1="1" y1="7" x2="27" y2="7" fill="none" stroke=stroke style=stroke_style stroke-width="2" stroke-dasharray=dash />
                {marker}
            </svg>
            {series.name.clone()}
        </span>
    }
    .into_any()
}

fn categorical_table(chart: &normalize::NormalizedChart, accessible_label: String) -> AnyView {
    let header = chart
        .series
        .iter()
        .map(|series| view! { <th scope="col">{series.name.clone()}</th> })
        .collect_view();
    let rows = chart
        .categories
        .iter()
        .enumerate()
        .map(|(index, category)| {
            let cells = chart
                .series
                .iter()
                .map(|series| {
                    let value = series.points[index]
                        .value
                        .map(|value| {
                            series.points[index]
                                .display_value
                                .clone()
                                .unwrap_or_else(|| value.to_string())
                        })
                        .unwrap_or_else(|| "No value".to_string());
                    view! { <td>{value}</td> }
                })
                .collect_view();
            view! { <tr><th scope="row">{category.label.clone()}</th>{cells}</tr> }
        })
        .collect_view();
    view! {
        <table data-line-chart-table class="sr-only">
            <caption>{accessible_label}</caption>
            <thead><tr><th scope="col">"Category"</th>{header}</tr></thead>
            <tbody>{rows}</tbody>
        </table>
    }
    .into_any()
}

fn render_empty_categorical(
    chart: &normalize::NormalizedChart,
    accessible_label: String,
    show_data_table: bool,
    _instance: u64,
) -> AnyView {
    view! {
        <div data-testid="interactive-line-chart" role="group" aria-label=accessible_label.clone() class="w-full">
            <div data-line-chart-empty role="status" class="text-sm opacity-70">"No chart data"</div>
            {show_data_table.then(|| categorical_table(chart, accessible_label.clone()))}
        </div>
    }
    .into_any()
}

/// Renders the preserved legacy numeric XY chart surface.
#[allow(clippy::too_many_arguments)]
fn render_xy(
    data: Vec<(f64, f64)>,
    width: u32,
    height: u32,
    color: String,
    show_dots: bool,
    x_label: Option<String>,
    y_label: Option<String>,
    x_labels: Vec<String>,
    minimal: bool,
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

    // Minimal/sparkline mode suppresses the axis TITLE labels entirely
    // (bd_4iiz-inventory-toe.5) — endpoint value labels stand in for the
    // y-scale, and the x-labels remain the only axis annotation.
    let (x_label, y_label) = if minimal {
        (None, None)
    } else {
        (x_label, y_label)
    };

    // Padding around the chart area for axes and labels. Minimal mode needs
    // no left gutter for a y-scale, and reserves a little right/top room so
    // the first/last value labels don't clip.
    let pad_left: f64 = if minimal {
        30.0
    } else if y_label.is_some() {
        60.0
    } else {
        40.0
    };
    let pad_right: f64 = if minimal { 40.0 } else { 20.0 };
    let pad_top: f64 = 20.0;
    let pad_bottom: f64 = if minimal {
        28.0
    } else if x_label.is_some() {
        50.0
    } else {
        35.0
    };

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

    // Build marker views: small squares in minimal mode (matching desktop's
    // understated endpoint markers), circles otherwise.
    let dot_views = if show_dots {
        data.iter()
            .map(|&(x, y)| {
                let (sx, sy) = to_svg(x, y);
                // A theme token must not ride on the `fill` presentation
                // attribute — see `super::paint::paint_attrs`.
                let (c, st) = paint_attrs(color.clone());
                if minimal {
                    let rx = format!("{:.2}", sx - 2.0);
                    let ry = format!("{:.2}", sy - 2.0);
                    view! { <rect x=rx y=ry width="4" height="4" fill=c style=st /> }.into_any()
                } else {
                    let cx_str = format!("{sx:.2}");
                    let cy_str = format!("{sy:.2}");
                    view! { <circle cx=cx_str cy=cy_str r="3" fill=c style=st /> }.into_any()
                }
            })
            .collect_view()
            .into_any()
    } else {
        ().into_any()
    };

    // Endpoint value labels (minimal mode): print the first and last y-values
    // next to their markers instead of a full y-axis scale — the desktop
    // sparkline's "40"/"42" endpoint annotations (bd_4iiz-inventory-toe.5).
    let endpoint_label_views = if minimal {
        let fmt = |v: f64| {
            if v.fract().abs() < 1e-9 {
                format!("{v:.0}")
            } else {
                format!("{v:.1}")
            }
        };
        let mut views = Vec::new();
        if let Some(&(x, y)) = data.first() {
            let (sx, sy) = to_svg(x, y);
            let (c, st) = paint_attrs(color.clone());
            views.push(view! {
                <text x=format!("{:.2}", sx) y=format!("{:.2}", sy - 8.0)
                    text-anchor="start" fill=c style=st font-size="12" font-weight="600">
                    {fmt(y)}
                </text>
            });
        }
        if data.len() > 1
            && let Some(&(x, y)) = data.last()
        {
            let (sx, sy) = to_svg(x, y);
            let (c, st) = paint_attrs(color.clone());
            views.push(view! {
                <text x=format!("{:.2}", sx) y=format!("{:.2}", sy - 8.0)
                    text-anchor="end" fill=c style=st font-size="12" font-weight="600">
                    {fmt(y)}
                </text>
            });
        }
        views.collect_view().into_any()
    } else {
        ().into_any()
    };

    // Axis tick views — suppressed in minimal mode (endpoint value labels
    // stand in for the y-scale).
    let y_tick_views = if minimal {
        ().into_any()
    } else {
        (0..=4)
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
            .collect_view()
            .into_any()
    };

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

    // The line itself is drawn with `stroke` alone, so it needs the same
    // treatment `fill` gets — a `var()` that stopped parsing there would fall
    // back to `stroke: none` and the polyline would vanish (ldui-1g5).
    let (line_stroke, line_style) = stroke_attrs(color);

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

    // The vertical y-axis line is suppressed in minimal mode; the horizontal
    // bottom baseline is kept as the sparkline's single gridline.
    let y_axis_line = if minimal {
        ().into_any()
    } else {
        view! {
            <line
                x1=pad_left_str.clone()
                y1=pad_top_str
                x2=pad_left_str.clone()
                y2=axis_y_end.clone()
                stroke="currentColor"
                stroke-opacity="0.3"
                stroke-width="1"
            />
        }
        .into_any()
    };

    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            {y_axis_line}
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
                stroke=line_stroke
                style=line_style
                stroke-width="2"
                stroke-linejoin="round"
                stroke-linecap="round"
            />
            {dot_views}
            {endpoint_label_views}
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
