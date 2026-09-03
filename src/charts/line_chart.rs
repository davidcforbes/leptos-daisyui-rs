use super::paint::{merge_style, paint_attrs, stroke_attrs};
use leptos::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

mod format;
mod geometry;
mod interaction;
mod normalize;
mod tooltip;
mod types;
use types::LineAxes;
pub use types::{
    LineAxisOptions, LineCategory, LineChartActivation, LineChartActivationSource,
    LineChartActivationValue, LineChartData, LineChartDataSource, LineChartModifiers,
    LineChartTexts, LineInteractionMode, LineLabelPlacement, LineLegendMode, LinePattern,
    LinePoint, LineSeries, LineValueAxis, LineValueDomain, MarkerShape, MarkerStyle,
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

/// Minimum distance between label baselines that share a category x-position.
///
/// Categorical labels use a 12-unit SVG font. Chromium's glyph box is about
/// 16.7 viewBox units tall with the showcase line-height, so 18 leaves a small
/// visible gap after the responsive SVG scale is applied.
const DATA_LABEL_BASELINE_SEPARATION: f64 = 18.0;

/// Moves a colliding label farther in its declared direction.
///
/// Fixed Above/Below marker offsets alone are insufficient when series
/// converge: the difference between the marker values can cancel those
/// offsets and put both labels on the same baseline. Moving only away from the
/// marker preserves the placement contract while keeping every caller-owned
/// label visible. Rechecking is necessary because clearing one occupied lane
/// can enter another when more than two labelled series share a category.
fn resolve_data_label_y(initial_y: f64, placement: LineLabelPlacement, occupied: &[f64]) -> f64 {
    let mut resolved = initial_y;
    loop {
        let conflict = occupied
            .iter()
            .copied()
            .filter(|other| (resolved - other).abs() < DATA_LABEL_BASELINE_SEPARATION)
            .reduce(|left, right| match placement {
                LineLabelPlacement::Above => left.min(right),
                LineLabelPlacement::Below => left.max(right),
            });

        let Some(conflict) = conflict else {
            return resolved;
        };
        resolved = match placement {
            LineLabelPlacement::Above => conflict - DATA_LABEL_BASELINE_SEPARATION,
            LineLabelPlacement::Below => conflict + DATA_LABEL_BASELINE_SEPARATION,
        };
    }
}

/// SVG-based line chart component.
///
/// Renders a responsive polyline chart with optional dot markers and axis
/// labels. Legacy `Vec<(f64, f64)>` callers keep the numeric XY surface
/// unchanged; categorical data ([`LineChartData::categorical`]) adds multiple
/// patterned series, markers, gaps, a legend, a hidden data table, and an
/// interactive hover/focus card with typed activation.
///
/// Legacy XY (source-compatible — a plain vector still just works):
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::charts::LineChart;
///
/// #[component]
/// fn WeeklyClosed() -> impl IntoView {
///     view! {
///         <LineChart
///             data=vec![(0.0, 18.0), (1.0, 24.0), (2.0, 21.0), (3.0, 31.0)]
///             x_labels=vec!["W31".to_string(), "W32".to_string(), "W33".to_string(), "W34".to_string()]
///             color="var(--color-primary)".to_string()
///         />
///     }
/// }
/// ```
///
/// Categorical with a reactive source and typed activation — the host owns
/// what an activation *means* (a route, a filter, a request):
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::charts::{
///     LineCategory, LineChart, LineChartActivation, LineChartData, LinePoint, LineSeries,
/// };
///
/// #[component]
/// fn WeeklyMatters() -> impl IntoView {
///     let categories = vec![
///         LineCategory { key: "week-01".to_string(), label: "W01".to_string() },
///         LineCategory { key: "week-02".to_string(), label: "W02".to_string() },
///     ];
///     let series = vec![LineSeries::new(
///         "matters",
///         "Matters",
///         "var(--color-primary)",
///         vec![LinePoint::new(42.0), LinePoint::new(45.0)],
///     )];
///     let data = RwSignal::new(LineChartData::categorical(categories, series));
///     let activated_key = RwSignal::new(None::<String>);
///     let on_point_activate = Callback::new(move |intent: LineChartActivation| {
///         // The host maps this key to its own route/filter/request.
///         activated_key.set(Some(intent.category_key));
///     });
///
///     view! {
///         <LineChart
///             data=data
///             accessible_label="Weekly matters".to_string()
///             on_point_activate=on_point_activate
///         />
///     }
/// }
/// ```
///
/// Two value axes — counts on the left, a duration on the right. A series opts
/// in with [`LineSeries::on_secondary_axis`]; every series defaults to
/// [`LineValueAxis::Primary`], so a chart that never mentions an axis renders
/// exactly as it did before this existed. Each axis computes its own domain,
/// and its unit is declared once in [`LineAxisOptions`] rather than formatted
/// separately for the ticks, the card and the accessible table:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::charts::{
///     LineAxisOptions, LineCategory, LineChart, LineChartData, LinePoint, LineSeries,
/// };
///
/// #[component]
/// fn ConversationsReport() -> impl IntoView {
///     let categories = vec![
///         LineCategory { key: "week-01".to_string(), label: "W01".to_string() },
///         LineCategory { key: "week-02".to_string(), label: "W02".to_string() },
///     ];
///     let series = vec![
///         LineSeries::new(
///             "opened",
///             "Opened",
///             "var(--color-primary)",
///             vec![LinePoint::new(120.0), LinePoint::new(150.0)],
///         ),
///         LineSeries::new(
///             "first-response",
///             "Average first response",
///             "var(--color-accent)",
///             vec![LinePoint::new(41.0), LinePoint::new(28.5)],
///         )
///         .on_secondary_axis(),
///     ];
///
///     view! {
///         <LineChart
///             data=LineChartData::categorical(categories, series)
///             accessible_label="Conversations by week".to_string()
///             primary_axis=LineAxisOptions::default().with_label("Conversations")
///             secondary_axis=LineAxisOptions::default()
///                 .with_label("First response")
///                 .with_unit(" s")
///                 .with_decimals(1)
///         />
///     }
/// }
/// ```
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
    /// Naming and value formatting for the left-hand value axis, which every
    /// series uses unless it opts out. Defaults to naming and formatting
    /// nothing, which is the pre-secondary-axis rendering.
    #[prop(optional, into)]
    primary_axis: LineAxisOptions,
    /// Naming and value formatting for the right-hand value axis. The axis
    /// itself is drawn only when a series is assigned to it, so setting this
    /// alone never adds a scale.
    #[prop(optional, into)]
    secondary_axis: LineAxisOptions,
    /// Chart copy that is neither per series nor per point. Defaults reproduce
    /// the strings the chart already emitted.
    #[prop(optional, into)]
    texts: LineChartTexts,
    /// Optional callback invoked by a categorical point activation.
    #[prop(optional)]
    on_point_activate: Option<Callback<LineChartActivation>>,
) -> impl IntoView {
    let data = Memo::new(move |_| data.get());
    let instance = LINE_CHART_SEQ.fetch_add(1, Ordering::Relaxed);
    let axes = LineAxes {
        primary: primary_axis,
        secondary: secondary_axis,
    };
    // The reconcile effect below normalizes independently of the render, so it
    // needs the same axis options; a chart normalized with different options
    // would compare unequal to itself and re-run reconciliation every frame.
    let axes_stored = StoredValue::new(axes.clone());

    // Interaction state lives here — outside the data-driven render closure —
    // so hover/focus/dismissal and the roving tab stop survive a data
    // replacement and can be *reconciled* against it rather than reset.
    let interaction = RwSignal::new(interaction::InteractionState::default());
    let previous_chart: StoredValue<Option<normalize::NormalizedChart>> = StoredValue::new(None);
    // Measured CSS width of the chart stage (written by a ResizeObserver);
    // `None` until first measurement, when the viewBox width stands in as the
    // deterministic initial value for tick thinning.
    let measured_width = RwSignal::new(None::<f64>);

    // Reconcile interaction state by category *key* whenever the categorical
    // data changes: an active category follows its key through a reorder,
    // and a removed key closes the card and moves focus to the nearest valid
    // category without firing any activation.
    Effect::new(move |_| {
        let LineChartData::Categorical { categories, series } = data.get() else {
            previous_chart.set_value(None);
            return;
        };
        let next = normalize::normalize_categorical(&categories, &series)
            .with_axes(axes_stored.get_value());
        let previous = previous_chart.get_value();
        previous_chart.set_value(Some(next.clone()));
        let Some(previous) = previous else {
            return;
        };
        if previous == next {
            return;
        }
        let old = interaction.get_untracked();
        let had_focus = old.focused.is_some();
        let next_state = interaction::reduce(
            &old,
            interaction::InteractionAction::ReconcileData,
            &previous,
            &next,
        );
        let refocus = if had_focus {
            next_state.focused.clone()
        } else {
            None
        };
        if next_state != old {
            interaction.set(next_state);
        }
        if let Some(active) = refocus {
            // Focus after the re-rendered targets exist in the DOM.
            let id = format!("line-chart-{instance}-category-{}", active.category_index);
            let _ = set_timeout_with_handle(
                move || interaction::focus_svg_element(&id),
                std::time::Duration::ZERO,
            );
        }
    });

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
            on_point_activate,
            instance,
            interaction,
            measured_width,
            axes.clone(),
            texts.clone(),
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
    interaction: RwSignal<interaction::InteractionState>,
    measured_width: RwSignal<Option<f64>>,
    axes: LineAxes,
    texts: LineChartTexts,
) -> AnyView {
    use geometry::{
        AxisProjections, PlotInsets, dasharray, marker_size, marker_stroke_width,
        nearest_series_at, path_segments, place_tooltip, plot_bounds, point, size, svg_number,
        visible_tick_indices,
    };
    use interaction::{
        ActivePoint, InteractionAction, NavigationKey, activation_for, displayed_active,
        focus_svg_element, reduce,
    };
    use normalize::normalize_categorical;
    use web_sys::wasm_bindgen::JsCast;

    let (width, height) = resolve_dimensions(width, height);
    let chart = normalize_categorical(&categories, &series).with_axes(axes);
    // The right-hand axis exists only when a series actually put finite values
    // on it. A caller that configures `secondary_axis` but assigns no series
    // therefore gets no phantom scale, ticks or gutter.
    let has_secondary_axis = chart.has_secondary_axis();
    let empty = |chart: &normalize::NormalizedChart| {
        render_empty_categorical(
            chart,
            width,
            height,
            accessible_label.clone(),
            description.clone(),
            show_data_table,
            instance,
            texts.clone(),
        )
    };
    if chart.categories.is_empty() || chart.series.is_empty() {
        return empty(&chart);
    }

    let max_marker_radius = chart
        .series
        .iter()
        .map(|series| marker_size(&series.marker) + marker_stroke_width(&series.marker) / 2.0)
        .fold(0.0_f64, f64::max);
    let has_data_labels = chart.series.iter().any(|series| series.show_data_labels);
    let primary_title = chart.axes.primary.label.clone();
    let secondary_title = has_secondary_axis
        .then(|| chart.axes.secondary.label.clone())
        .flatten();
    let bounds = plot_bounds(
        width as f64,
        height as f64,
        PlotInsets {
            max_marker_radius,
            has_data_labels,
            secondary_ticks: has_secondary_axis,
            primary_label: primary_title.is_some(),
            secondary_label: secondary_title.is_some(),
        },
    );
    let Some(projections) = AxisProjections::new(
        bounds,
        chart.categories.len(),
        chart.domain,
        chart.secondary_domain,
    ) else {
        return empty(&chart);
    };
    let plot_width = (bounds.right - bounds.left).max(1.0);
    // `None` while one axis is enough: every caption, accessible value and
    // table header then keeps the bare series name it has always had. The
    // moment two scales are on screen, one attribution is built here and used
    // by the legend, the focus targets' accessible names and the hidden table
    // alike, so a reader meets the same wording everywhere.
    let axis_names = has_secondary_axis.then(|| {
        (
            format::axis_name(LineValueAxis::Primary, &chart.axes, &texts),
            format::axis_name(LineValueAxis::Secondary, &chart.axes, &texts),
        )
    });
    let title_id = format!("line-chart-{instance}-title");
    let desc_id = format!("line-chart-{instance}-desc");
    let tooltip_id = format!("line-chart-{instance}-tooltip");
    let Some(initial_state) = initial_categorical_state(&chart) else {
        return empty(&chart);
    };
    let show_legend = match legend_mode {
        LineLegendMode::Auto => chart.series.len() >= 2,
        LineLegendMode::Always => true,
        LineLegendMode::Never => false,
    };
    let interaction_enabled = match interaction_mode {
        LineInteractionMode::Auto | LineInteractionMode::Enabled => true,
        LineInteractionMode::Disabled => false,
    };
    let focus_role = if on_point_activate.is_some() {
        "button"
    } else {
        "group"
    };
    // role="img" makes descendants presentational, which contradicts the
    // focusable category targets inside (axe: nested-interactive +
    // svg-img-alt, found by the ldui-9tr.6 gate). An interactive chart is a
    // named group; only the target-less non-interactive render keeps the
    // pure-image role.
    let svg_role = if interaction_enabled { "group" } else { "img" };

    // The chart is rebuilt per data change; the interaction signal outlives
    // it. Reactive attribute closures read the current chart through this
    // stored copy so they never capture a stale borrow.
    let chart_stored = StoredValue::new(chart.clone());
    let initial_index = initial_state.category_index;

    // Every input event funnels through the pure reducer; DOM side effects
    // (focus moves, the callback) stay at the call sites.
    let dispatch = move |action: InteractionAction| {
        chart_stored.with_value(|current| {
            let old = interaction.get_untracked();
            let next = reduce(&old, action, current, current);
            if next != old {
                interaction.set(next);
            }
        })
    };
    let roving_index = move || {
        interaction
            .read()
            .roving_category_key
            .as_deref()
            .and_then(|key| {
                chart_stored.with_value(|current| {
                    current
                        .categories
                        .iter()
                        .position(|category| category.key == key)
                })
            })
            .unwrap_or(initial_index)
    };
    let displayed =
        move || chart_stored.with_value(|current| displayed_active(&interaction.get(), current));
    let displayed_index = move || displayed().map(|active| active.category_index);
    let focused_index = move || {
        interaction
            .read()
            .focused
            .as_ref()
            .map(|active| active.category_index)
    };
    let active_category_attr = move || {
        displayed()
            .and_then(|active| {
                chart_stored.with_value(|current| {
                    current
                        .categories
                        .get(active.category_index)
                        .map(|category| category.key.clone())
                })
            })
            .unwrap_or_default()
    };
    let preferred_series_attr = move || {
        displayed()
            .and_then(|active| {
                chart_stored.with_value(|current| preferred_series_id(current, &active))
            })
            .unwrap_or_default()
    };
    let modifiers_of = |shift: bool, ctrl: bool, alt: bool, meta: bool| LineChartModifiers {
        shift,
        ctrl,
        alt,
        meta,
    };
    // Converts a pointer event's client coordinates (relative to the overlay,
    // which spans exactly the plot rectangle) into an active point.
    let plot_active_at = move |target: Option<web_sys::EventTarget>,
                               client_x: f64,
                               client_y: f64| {
        let element = target?.dyn_into::<web_sys::Element>().ok()?;
        let rect = element.get_bounding_client_rect();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return None;
        }
        let svg_x =
            bounds.left + (client_x - rect.left()) / rect.width() * (bounds.right - bounds.left);
        let svg_y =
            bounds.top + (client_y - rect.top()) / rect.height() * (bounds.bottom - bounds.top);
        let category_index = projections.category_at_x(svg_x)?;
        let preferred_series_index = chart_stored
            .with_value(|current| nearest_series_at(current, &projections, category_index, svg_y));
        Some(ActivePoint {
            category_index,
            preferred_series_index,
        })
    };

    let stage_ref = NodeRef::<leptos::html::Div>::new();
    let tooltip_ref = NodeRef::<leptos::html::Div>::new();
    let tooltip_style = RwSignal::new(String::from("display:none"));
    let tooltip_model_memo = {
        let tooltip_id = tooltip_id.clone();
        Memo::new(move |_| {
            displayed().and_then(|active| {
                chart_stored.with_value(|current| {
                    tooltip::tooltip_model(current, &projections, &active, &tooltip_id)
                })
            })
        })
    };

    // Measure the card only after it has rendered, then place and reveal it —
    // an unclamped first frame is never painted. Re-placing also keys off the
    // measured stage width so a resize while a card is open re-clamps it.
    Effect::new(move |_| {
        let _ = measured_width.get();
        let Some(model) = tooltip_model_memo.get() else {
            tooltip_style.set("display:none".to_string());
            return;
        };
        tooltip_style.set("visibility:hidden;left:0;top:0".to_string());
        let anchor = model.anchor;
        request_animation_frame(move || {
            let (Some(stage), Some(card)) =
                (stage_ref.get_untracked(), tooltip_ref.get_untracked())
            else {
                return;
            };
            let stage_rect = stage.get_bounding_client_rect();
            let card_rect = card.get_bounding_client_rect();
            if stage_rect.width() <= 0.0 || stage_rect.height() <= 0.0 {
                return;
            }
            let anchor_css = point(
                anchor.x * stage_rect.width() / width as f64,
                anchor.y * stage_rect.height() / height as f64,
            );
            let placement = place_tooltip(
                anchor_css,
                size(card_rect.width(), card_rect.height()),
                size(stage_rect.width(), stage_rect.height()),
                8.0,
            );
            tooltip_style.set(format!(
                "left:{:.1}px;top:{:.1}px",
                placement.origin.x, placement.origin.y
            ));
        });
    });

    // One ResizeObserver on the stage (never the SVG, whose rendered size
    // depends on the signal this writes — observing it would loop). Width is
    // written only when it actually changes.
    Effect::new(move |_| {
        let Some(stage) = stage_ref.get() else {
            return;
        };
        let update_width = {
            let stage = stage.clone();
            move || {
                let measured = stage.get_bounding_client_rect().width();
                if measured > 0.0
                    && measured_width
                        .get_untracked()
                        .is_none_or(|old| (old - measured).abs() > 0.5)
                {
                    measured_width.set(Some(measured));
                }
            }
        };
        update_width();
        let closure = web_sys::wasm_bindgen::closure::Closure::wrap(Box::new({
            let update_width = update_width.clone();
            move |_entries: js_sys::Array, _observer: web_sys::ResizeObserver| update_width()
        })
            as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);
        match web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) {
            Ok(observer) => {
                observer.observe(stage.unchecked_ref::<web_sys::Element>());
                // Same single-threaded-wasm rationale as DataTable's observer.
                let guard = send_wrapper::SendWrapper::new((closure, observer));
                on_cleanup(move || {
                    let (closure, observer) = guard.take();
                    observer.disconnect();
                    drop(closure);
                });
            }
            Err(_) => drop(closure),
        }
    });

    let grid_lines = (0..=4)
        .map(|index| {
            let y = bounds.top + (bounds.bottom - bounds.top) * index as f64 / 4.0;
            view! {
                <line x1=svg_number(bounds.left) y1=svg_number(y) x2=svg_number(bounds.right) y2=svg_number(y)
                    stroke="currentColor" stroke-opacity="0.14" stroke-width="1" />
            }
        })
        .collect_view();
    // Both axes read against the same five gridline fractions, so a reader
    // compares a left tick and a right tick at the same height.
    let y_ticks = chart.domain.map(|domain| {
        value_axis_ticks(
            domain,
            bounds,
            &chart.axes.primary,
            AxisTickSide::Left,
            None,
        )
    });
    let secondary_y_ticks = chart.secondary_domain.map(|domain| {
        let ticks = value_axis_ticks(
            domain,
            bounds,
            &chart.axes.secondary,
            AxisTickSide::Right,
            Some("secondary"),
        );
        let (axis_stroke, axis_style) = stroke_attrs("currentColor".to_string());
        view! {
            <g data-line-chart-y-axis="secondary">
                <line x1=svg_number(bounds.right) y1=svg_number(bounds.top) x2=svg_number(bounds.right) y2=svg_number(bounds.bottom)
                    stroke=axis_stroke style=axis_style stroke-opacity="0.35" stroke-width="1" />
                {ticks}
            </g>
        }
    });
    let primary_axis_title = primary_title.map(|label| {
        axis_title_view(
            label,
            12.0,
            (bounds.top + bounds.bottom) / 2.0,
            -90.0,
            "primary",
        )
    });
    let secondary_axis_title = secondary_title.map(|label| {
        axis_title_view(
            label,
            width as f64 - 12.0,
            (bounds.top + bounds.bottom) / 2.0,
            90.0,
            "secondary",
        )
    });
    // Tick thinning keys off the measured CSS width once the ResizeObserver
    // has reported one; the viewBox width is the deterministic initial value.
    // 56px minimum (not 48): the first/last ticks anchor start/end to avoid
    // viewBox clipping, which extends them toward their neighbours — at 48px
    // a 14-category chart put W01/W02 exactly in contact (style-audit hard
    // OVERLAP, ldui-9tr.7).
    let x_ticks = move || {
        let css_width = measured_width.get().unwrap_or(width as f64);
        chart_stored.with_value(|current| {
            visible_tick_indices(current.categories.len(), css_width, 56.0)
                .into_iter()
                .map(|index| {
                    let category = &current.categories[index];
                    let anchor = tick_anchor(index, current.categories.len());
                    let (fill, fill_style) = paint_attrs("currentColor".to_string());
                    view! {
                        <text x=svg_number(projections.category_x(index)) y=svg_number(bounds.bottom + 18.0)
                            text-anchor=anchor fill=fill style=fill_style font-size="12" opacity="0.7">
                            {category.label.clone()}
                        </text>
                    }
                })
                .collect_view()
        })
    };

    let line_paths = chart
        .series
        .iter()
        .flat_map(|series| {
            let color = series.color.clone();
            let id = series.id.clone();
            let dash = dasharray(&series.pattern);
            path_segments(series, &projections).into_iter().map(move |d| {
                // Router-headed binding per segment so the paint-routing scan
                // sees a bare `stroke=stroke` (svg_paint_routing).
                let (stroke, stroke_style) = stroke_attrs(color.clone());
                view! {
                    <path d=d data-series-id=id.clone() fill="none" stroke=stroke style=stroke_style
                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray=dash.clone() />
                }
            })
        })
        .collect_view();
    let mut marker_views = Vec::new();
    let mut data_label_views = Vec::new();
    let mut data_label_ys = vec![Vec::new(); chart.categories.len()];
    for series in &chart.series {
        for (index, point) in series.points.iter().enumerate() {
            let Some(value) = point.value else {
                continue;
            };
            // Every mark is placed by the projection of the series' own axis;
            // there is no shared value scale left in the render path.
            let value_y = projections.value_y(series.axis, value);
            marker_views.push(marker_view(
                &series.id,
                index,
                &chart.categories[index].key,
                projections.category_x(index),
                value_y,
                series,
                point.marker_color.as_deref(),
            ));
            // `ldui-raa7`. Two changes to what used to be here:
            //
            // 1. `display_value` is accepted as the label when no explicit
            //    `data_label` was given. A caller who turned labels on and
            //    supplied only the server's formatted string obviously meant
            //    that string -- and this can never FABRICATE one, which is the
            //    consumer's no-math contract: absent stays absent.
            // 2. The offset honours `label_placement`. It used to be
            //    unconditionally above, so two close series drew their labels
            //    on top of each other.
            if series.show_data_labels
                && let Some(label) = point
                    .data_label
                    .clone()
                    .or_else(|| point.display_value.clone())
            {
                let anchor = tick_anchor(index, chart.categories.len());
                let (fill, style) = paint_attrs(series.color.clone());
                let clearance = marker_size(&series.marker) + 5.0;
                let (initial_label_y, placement) = match series.label_placement {
                    LineLabelPlacement::Above => (value_y - clearance, "above"),
                    // `+ 12.0` rather than `+ clearance`: text hangs DOWN from
                    // its baseline, so a below-marker label needs the glyph
                    // height cleared too or it overlaps the node it labels.
                    LineLabelPlacement::Below => (value_y + clearance + 7.0, "below"),
                };
                let label_y = resolve_data_label_y(
                    initial_label_y,
                    series.label_placement,
                    &data_label_ys[index],
                );
                data_label_ys[index].push(label_y);
                data_label_views.push(view! {
                    <text x=svg_number(projections.category_x(index)) y=svg_number(label_y)
                        text-anchor=anchor fill=fill style=style font-size="12" font-weight="600"
                        data-line-label-placement=placement>
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
                let values = category_accessible_values(&chart, index, axis_names.as_ref());
                let describedby_id = tooltip_id.clone();
                view! {
                    <rect id=format!("line-chart-{instance}-category-{index}") x=svg_number(projections.category_x(index) - plot_width / (chart.categories.len().max(2) - 1) as f64 / 2.0)
                        y=svg_number(bounds.top) width=svg_number(plot_width / (chart.categories.len().max(2) - 1) as f64)
                        height=svg_number(bounds.bottom - bounds.top) fill="transparent" role=focus_role
                        data-line-chart-focus=""
                        // The overlay owns pointer hit testing; targets are
                        // keyboard/AT surfaces only.
                        pointer-events="none"
                        // Focus cue that is not color-alone: the focused
                        // category's generous hit box gains a visible ring.
                        stroke="currentColor" rx="3"
                        stroke-width=move || if focused_index() == Some(index) { "2" } else { "0" }
                        stroke-opacity="0.55"
                        tabindex=move || if roving_index() == index { "0" } else { "-1" }
                        aria-describedby=move || (displayed_index() == Some(index)).then(|| describedby_id.clone())
                        aria-label=format!("{}: {values}", category.label)
                        data-category-index=index data-category-key=category.key.clone()
                        on:focus=move |_| dispatch(InteractionAction::Focused(ActivePoint {
                            category_index: index,
                            preferred_series_index: None,
                        }))
                        on:blur=move |_| dispatch(InteractionAction::Blurred)
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            let key = ev.key();
                            let nav = match key.as_str() {
                                "ArrowLeft" => Some(NavigationKey::Left),
                                "ArrowRight" => Some(NavigationKey::Right),
                                "ArrowUp" => Some(NavigationKey::Up),
                                "ArrowDown" => Some(NavigationKey::Down),
                                "Home" => Some(NavigationKey::Home),
                                "End" => Some(NavigationKey::End),
                                _ => None,
                            };
                            if let Some(nav) = nav {
                                // Prevent default only for the horizontal/edge
                                // keys the composite claims outright.
                                if !matches!(nav, NavigationKey::Up | NavigationKey::Down) {
                                    ev.prevent_default();
                                }
                                dispatch(InteractionAction::MoveFocus(nav));
                                if let Some(active) = interaction.get_untracked().focused {
                                    focus_svg_element(&format!(
                                        "line-chart-{instance}-category-{}",
                                        active.category_index
                                    ));
                                }
                                return;
                            }
                            match key.as_str() {
                                "Escape" => {
                                    ev.prevent_default();
                                    dispatch(InteractionAction::Dismiss);
                                }
                                "Enter" | " " => {
                                    // Inert without a callback: no preventDefault,
                                    // no claimed button behavior.
                                    if let Some(callback) = on_point_activate {
                                        ev.prevent_default();
                                        let active = interaction
                                            .get_untracked()
                                            .focused
                                            .unwrap_or(ActivePoint {
                                                category_index: index,
                                                preferred_series_index: None,
                                            });
                                        let payload = chart_stored.with_value(|current| {
                                            activation_for(
                                                current,
                                                active,
                                                LineChartActivationSource::Keyboard,
                                                modifiers_of(
                                                    ev.shift_key(),
                                                    ev.ctrl_key(),
                                                    ev.alt_key(),
                                                    ev.meta_key(),
                                                ),
                                            )
                                        });
                                        if let Some(payload) = payload {
                                            callback.run(payload);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        } />
                }
            })
            .collect_view()
    });
    let overlay = interaction_enabled.then(|| {
        view! {
            <rect data-line-chart-pointer-overlay="" x=svg_number(bounds.left) y=svg_number(bounds.top)
                width=svg_number(plot_width) height=svg_number(bounds.bottom - bounds.top) fill="transparent" pointer-events="all"
                on:pointerenter=move |_| dispatch(InteractionAction::PointerEntered)
                on:pointermove=move |ev: web_sys::PointerEvent| {
                    if let Some(active) = plot_active_at(
                        ev.current_target(),
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                    ) {
                        dispatch(InteractionAction::PointerMoved(active));
                    }
                }
                on:pointerleave=move |_| dispatch(InteractionAction::PointerLeft)
                // The overlay is the only pointer activation path: markers
                // carry no click handlers of their own, so one click is one
                // callback invocation.
                on:click=move |ev: web_sys::MouseEvent| {
                    let Some(callback) = on_point_activate else {
                        return;
                    };
                    let Some(active) = plot_active_at(
                        ev.current_target(),
                        ev.client_x() as f64,
                        ev.client_y() as f64,
                    ) else {
                        return;
                    };
                    let payload = chart_stored.with_value(|current| {
                        activation_for(
                            current,
                            active,
                            LineChartActivationSource::Pointer,
                            modifiers_of(
                                ev.shift_key(),
                                ev.ctrl_key(),
                                ev.alt_key(),
                                ev.meta_key(),
                            ),
                        )
                    });
                    if let Some(payload) = payload {
                        callback.run(payload);
                    }
                } />
        }
    });
    let legend = show_legend.then(|| {
        chart
            .series
            .iter()
            .map(|series| legend_entry(series, axis_names.as_ref()))
            .collect_view()
    });
    let description =
        description.unwrap_or_else(|| "Categorical multi-series line chart".to_string());
    let viewbox = format!("0 0 {width} {height}");

    let tooltip_card = {
        let tooltip_id = tooltip_id.clone();
        view! {
            <div data-testid="line-chart-tooltip" id=tooltip_id role="tooltip" node_ref=tooltip_ref
                class="pointer-events-none absolute z-10 rounded-sm border border-base-300 bg-base-100 p-2 text-xs shadow-sm"
                style=move || tooltip_style.get()>
                {move || tooltip_model_memo.get().map(|model| tooltip_card_content(model, has_secondary_axis))}
            </div>
        }
    };

    view! {
        <div data-testid="interactive-line-chart" role="group" aria-label=accessible_label.clone()
            data-active-category=active_category_attr data-preferred-series=preferred_series_attr
            data-line-chart-axes=has_secondary_axis.then_some("dual") class="w-full">
            {show_legend.then(|| view! {
                <div data-line-chart-legend class="flex flex-wrap gap-x-4 gap-y-2 text-sm" aria-label="Chart legend">
                    {legend}
                </div>
            })}
            <div data-line-chart-stage node_ref=stage_ref class="relative mt-2">
                <svg data-line-chart-plot role=svg_role aria-labelledby=format!("{title_id} {desc_id}")
                    viewBox=viewbox class="h-auto w-full" xmlns="http://www.w3.org/2000/svg">
                    <title id=title_id.clone()>{accessible_label.clone()}</title>
                    <desc id=desc_id.clone()>{description}</desc>
                    {grid_lines}
                    <line x1=svg_number(bounds.left) y1=svg_number(bounds.bottom) x2=svg_number(bounds.right) y2=svg_number(bounds.bottom)
                        stroke="currentColor" stroke-opacity="0.35" stroke-width="1" />
                    <line x1=svg_number(bounds.left) y1=svg_number(bounds.top) x2=svg_number(bounds.left) y2=svg_number(bounds.bottom)
                        stroke="currentColor" stroke-opacity="0.35" stroke-width="1" />
                    {y_ticks}
                    {secondary_y_ticks}
                    {primary_axis_title}
                    {secondary_axis_title}
                    {x_ticks}
                    {line_paths}
                    {markers}
                    {data_labels}
                    {focus_targets}
                    {overlay}
                </svg>
                {tooltip_card}
            </div>
            {show_data_table.then(|| categorical_table(&chart, accessible_label.clone(), &texts, axis_names.as_ref()))}
        </div>
    }
    .into_any()
}

/// Renders one tooltip card body: category header plus one row per finite
/// series with the same pattern/marker swatch identity the legend draws. No
/// focusable content — the card is a described-by surface, not a stop.
fn tooltip_card_content(model: tooltip::TooltipModel, dual_axis: bool) -> AnyView {
    use geometry::{dasharray, svg_number};

    let rows = model
        .rows
        .iter()
        .map(|row| {
            let preferred = model.preferred_series_id.as_deref() == Some(row.series_id.as_str());
            let (stroke, stroke_style) = stroke_attrs(row.color.clone());
            let (fill, fill_style) = paint_attrs(row.color.clone());
            let dash = dasharray(&row.pattern);
            let marker = match row.marker_shape {
                MarkerShape::None => ().into_any(),
                MarkerShape::Circle => view! {
                    <circle cx="10" cy="4" r=svg_number(2.5) fill=fill style=fill_style />
                }
                .into_any(),
                MarkerShape::Square => view! {
                    <rect x="7.5" y="1.5" width="5" height="5" fill=fill style=fill_style />
                }
                .into_any(),
                MarkerShape::Diamond => view! {
                    <path d="M 10 1 L 13 4 L 10 7 L 7 4 Z" fill=fill style=fill_style />
                }
                .into_any(),
            };
            view! {
                <div class="flex items-center gap-2" data-series-id=row.series_id.clone()
                    data-preferred=preferred.to_string()
                    data-axis=dual_axis.then(|| format::axis_token(row.axis))>
                    <svg aria-hidden="true" viewBox="0 0 20 8" class="h-2 w-5">
                        <line x1="1" y1="4" x2="19" y2="4" fill="none" stroke=stroke style=stroke_style
                            stroke-width="2" stroke-dasharray=dash />
                        {marker}
                    </svg>
                    <span class=if preferred { "font-semibold" } else { "" }>
                        {row.series_name.clone()}
                    </span>
                    <span class="ml-auto pl-3 tabular-nums">{row.display_value.clone()}</span>
                </div>
            }
        })
        .collect_view();
    view! {
        <div class="font-semibold">{model.category_label.clone()}</div>
        <div class="mt-1 flex min-w-32 flex-col gap-1">{rows}</div>
    }
    .into_any()
}

/// Which side of the plot an axis' tick labels sit on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AxisTickSide {
    Left,
    Right,
}

/// Renders one value axis' five tick labels against the shared gridlines.
///
/// Both axes come through here so the right-hand scale cannot drift from the
/// left-hand one in position, size or formatting. `data_axis` is `None` for
/// the primary axis, which keeps a single-axis chart's tick markup exactly
/// what it was; the secondary axis carries `data-axis="secondary"` so a
/// browser test locates it by identity rather than by document position.
fn value_axis_ticks(
    domain: normalize::Domain,
    bounds: geometry::PlotBounds,
    options: &LineAxisOptions,
    side: AxisTickSide,
    data_axis: Option<&'static str>,
) -> AnyView {
    use geometry::svg_number;

    let (x, anchor) = match side {
        AxisTickSide::Left => (bounds.left - 6.0, "end"),
        AxisTickSide::Right => (bounds.right + 6.0, "start"),
    };
    (0..=4)
        .map(|index| {
            let fraction = index as f64 / 4.0;
            let value = domain.max - (domain.max - domain.min) * fraction;
            let y = bounds.top + (bounds.bottom - bounds.top) * fraction;
            let (fill, fill_style) = paint_attrs("currentColor".to_string());
            view! {
                <text x=svg_number(x) y=svg_number(y) text-anchor=anchor dominant-baseline="middle"
                    fill=fill style=fill_style font-size="12" opacity="0.65" data-axis=data_axis>
                    {format::tick_text(value, options)}
                </text>
            }
        })
        .collect_view()
        .into_any()
}

/// Renders a rotated axis title outside its tick column.
fn axis_title_view(label: String, x: f64, y: f64, rotation: f64, axis: &'static str) -> AnyView {
    use geometry::svg_number;

    let (fill, fill_style) = paint_attrs("currentColor".to_string());
    view! {
        <text x=svg_number(x) y=svg_number(y) text-anchor="middle" fill=fill style=fill_style
            font-size="12" transform=format!("rotate({rotation}, {}, {})", svg_number(x), svg_number(y))
            data-line-chart-axis-label=axis>
            {label}
        </text>
    }
    .into_any()
}

/// How a series is named wherever its numbers appear beside another axis'.
///
/// With one axis this is the bare series name, byte for byte what it always
/// was. With two, the axis is named alongside it — once, here — so the legend,
/// the accessible names and the hidden table cannot describe the same series
/// three different ways.
fn series_caption(
    series: &normalize::NormalizedSeries,
    axis_names: Option<&(String, String)>,
) -> String {
    match axis_names {
        None => series.name.clone(),
        Some((primary, secondary)) => format::series_caption(
            &series.name,
            match series.axis {
                LineValueAxis::Primary => primary,
                LineValueAxis::Secondary => secondary,
            },
        ),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InitialCategoricalState {
    category_index: usize,
    category_key: String,
    preferred_series: String,
}

fn resolve_dimensions(width: u32, height: u32) -> (u32, u32) {
    (width.max(1), height.max(1))
}

fn initial_categorical_state(
    chart: &normalize::NormalizedChart,
) -> Option<InitialCategoricalState> {
    chart
        .categories
        .iter()
        .enumerate()
        .find_map(|(category_index, category)| {
            chart
                .series
                .iter()
                .find(|series| series.points[category_index].value.is_some())
                .map(|series| InitialCategoricalState {
                    category_index,
                    category_key: category.key.clone(),
                    preferred_series: series.id.clone(),
                })
        })
}

/// The series id a card/root attribute should report as preferred for
/// `active`: the reducer's preferred index when it is finite at this
/// category, else the first finite series (mirroring `activation_for`).
fn preferred_series_id(
    chart: &normalize::NormalizedChart,
    active: &interaction::ActivePoint,
) -> Option<String> {
    let finite = |index: usize| {
        chart
            .series
            .get(index)
            .and_then(|series| series.points.get(active.category_index))
            .and_then(|point| point.value)
            .is_some()
    };
    let index = active
        .preferred_series_index
        .filter(|index| finite(*index))
        .or_else(|| (0..chart.series.len()).find(|index| finite(*index)))?;
    chart.series.get(index).map(|series| series.id.clone())
}

fn category_accessible_values(
    chart: &normalize::NormalizedChart,
    category_index: usize,
    axis_names: Option<&(String, String)>,
) -> String {
    chart
        .series
        .iter()
        .filter_map(|series| {
            let point = &series.points[category_index];
            point.value.map(|value| {
                let display_value = point
                    .display_value
                    .clone()
                    .unwrap_or_else(|| format::value_text(value, &series.format));
                format!("{} {display_value}", series_caption(series, axis_names))
            })
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn marker_fill<'a>(
    series: &'a normalize::NormalizedSeries,
    point_marker_color: Option<&'a str>,
) -> &'a str {
    point_marker_color
        .or(series.marker.fill.as_deref())
        .unwrap_or(&series.color)
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
    // Bare router-headed bindings (not a packed tuple): the paint-routing
    // scan accepts only `fill=fill`/`stroke=stroke` from a `paint_attrs`/
    // `stroke_attrs` let (svg_paint_routing). Only one match arm runs, so
    // every arm may move the same bindings.
    let (fill, fill_style) = paint_attrs(marker_fill(series, point_marker_color).to_string());
    let (stroke, stroke_style) = stroke_attrs(series.color.clone());
    let style = merge_style([fill_style, stroke_style]);
    let sid = series_id.to_string();
    let key = category_key.to_string();
    let width = svg_number(marker_stroke_width(&series.marker));
    match series.marker.shape {
        MarkerShape::None => ().into_any(),
        MarkerShape::Circle => view! {
            <circle cx=svg_number(x) cy=svg_number(y) r=svg_number(size) fill=fill stroke=stroke style=style stroke-width=width
                data-series-id=sid data-category-index=category_index data-category-key=key data-marker-shape="circle" />
        }
        .into_any(),
        MarkerShape::Square => view! {
            <rect x=svg_number(x - size) y=svg_number(y - size) width=svg_number(size * 2.0) height=svg_number(size * 2.0)
                fill=fill stroke=stroke style=style stroke-width=width
                data-series-id=sid data-category-index=category_index data-category-key=key data-marker-shape="square" />
        }
        .into_any(),
        MarkerShape::Diamond => view! {
            <path d=format!("M {} {} L {} {} L {} {} L {} {} Z", svg_number(x), svg_number(y - size), svg_number(x + size), svg_number(y), svg_number(x), svg_number(y + size), svg_number(x - size), svg_number(y))
                fill=fill stroke=stroke style=style stroke-width=width
                data-series-id=sid data-category-index=category_index data-category-key=key data-marker-shape="diamond" />
        }
        .into_any(),
    }
}

fn legend_entry(
    series: &normalize::NormalizedSeries,
    axis_names: Option<&(String, String)>,
) -> AnyView {
    use geometry::{dasharray, marker_size, svg_number};
    let (stroke, stroke_style) = stroke_attrs(series.color.clone());
    // The marker glyph gets its own router-headed stroke binding rather than
    // cloning the line stroke: the paint-routing scan only accepts a bare
    // identifier bound directly by `stroke_attrs` (svg_paint_routing).
    let (marker_stroke, marker_stroke_style) = stroke_attrs(series.color.clone());
    let (fill, fill_style) = paint_attrs(
        series
            .marker
            .fill
            .as_deref()
            .unwrap_or(&series.color)
            .to_string(),
    );
    let marker_style = merge_style([fill_style, marker_stroke_style]);
    let dash = dasharray(&series.pattern);
    let marker = match series.marker.shape {
        MarkerShape::None => ().into_any(),
        MarkerShape::Circle => view! {
            <circle cx="14" cy="7" r=svg_number(marker_size(&series.marker).min(3.5)) fill=fill stroke=marker_stroke style=marker_style stroke-width="1" />
        }
        .into_any(),
        MarkerShape::Square => view! {
            <rect x="11" y="4" width="6" height="6" fill=fill stroke=marker_stroke style=marker_style stroke-width="1" />
        }
        .into_any(),
        MarkerShape::Diamond => view! {
            <path d="M 14 3 L 18 7 L 14 11 L 10 7 Z" fill=fill stroke=marker_stroke style=marker_style stroke-width="1" />
        }
        .into_any(),
    };
    view! {
        <span data-series-id=series.id.clone() data-axis=axis_names.map(|_| format::axis_token(series.axis))
            class="inline-flex items-center gap-2 whitespace-nowrap">
            <svg data-line-chart-pattern-swatch="" aria-hidden="true" viewBox="0 0 28 14" class="h-4 w-7">
                <line x1="1" y1="7" x2="27" y2="7" fill="none" stroke=stroke style=stroke_style stroke-width="2" stroke-dasharray=dash />
                {marker}
            </svg>
            {series_caption(series, axis_names)}
        </span>
    }
    .into_any()
}

/// The chart's non-visual truth.
///
/// With two axes a bare column of numbers is ambiguous, so each series column
/// is headed by the same caption the legend shows — series name plus its axis
/// — and each cell carries its axis' unit through the shared formatter. Both
/// the header and the cell also carry `data-axis`, but only when there are two
/// axes to distinguish: a single-axis table is byte for byte what it was.
fn categorical_table(
    chart: &normalize::NormalizedChart,
    accessible_label: String,
    texts: &LineChartTexts,
    axis_names: Option<&(String, String)>,
) -> AnyView {
    let axis_attr =
        |series: &normalize::NormalizedSeries| axis_names.map(|_| format::axis_token(series.axis));
    let header = chart
        .series
        .iter()
        .map(|series| {
            view! { <th scope="col" data-axis=axis_attr(series)>{series_caption(series, axis_names)}</th> }
        })
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
                                .unwrap_or_else(|| format::value_text(value, &series.format))
                        })
                        .unwrap_or_else(|| texts.no_value.clone());
                    view! { <td data-axis=axis_attr(series)>{value}</td> }
                })
                .collect_view();
            view! { <tr><th scope="row">{category.label.clone()}</th>{cells}</tr> }
        })
        .collect_view();
    let category_header = texts.category_header.clone();
    view! {
        <table data-line-chart-table class="sr-only">
            <caption>{accessible_label}</caption>
            <thead><tr><th scope="col">{category_header}</th>{header}</tr></thead>
            <tbody>{rows}</tbody>
        </table>
    }
    .into_any()
}

#[allow(clippy::too_many_arguments)]
fn render_empty_categorical(
    chart: &normalize::NormalizedChart,
    width: u32,
    height: u32,
    accessible_label: String,
    description: Option<String>,
    show_data_table: bool,
    instance: u64,
    texts: LineChartTexts,
) -> AnyView {
    use geometry::{PlotInsets, plot_bounds, svg_number};

    let bounds = plot_bounds(width as f64, height as f64, PlotInsets::default());
    let title_id = format!("line-chart-{instance}-title");
    let desc_id = format!("line-chart-{instance}-desc");
    let description =
        description.unwrap_or_else(|| "Categorical multi-series line chart".to_string());
    let viewbox = format!("0 0 {width} {height}");

    view! {
        <div data-testid="interactive-line-chart" role="group" aria-label=accessible_label.clone() class="w-full">
            <div data-line-chart-stage class="relative mt-2">
                <svg data-line-chart-plot role="img" aria-labelledby=format!("{title_id} {desc_id}")
                    viewBox=viewbox class="h-auto w-full" xmlns="http://www.w3.org/2000/svg">
                    <title id=title_id.clone()>{accessible_label.clone()}</title>
                    <desc id=desc_id.clone()>{description}</desc>
                    <line x1=svg_number(bounds.left) y1=svg_number(bounds.bottom) x2=svg_number(bounds.right) y2=svg_number(bounds.bottom)
                        stroke="currentColor" stroke-opacity="0.35" stroke-width="1" />
                    <line x1=svg_number(bounds.left) y1=svg_number(bounds.top) x2=svg_number(bounds.left) y2=svg_number(bounds.bottom)
                        stroke="currentColor" stroke-opacity="0.35" stroke-width="1" />
                </svg>
                <div data-line-chart-empty role="status" class="text-sm opacity-70">"No chart data"</div>
            </div>
            {show_data_table.then(|| categorical_table(chart, accessible_label.clone(), &texts, None))}
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
    fn categorical_initial_focus_uses_first_finite_category_and_display_values() {
        let categories = vec![
            LineCategory {
                key: "empty".to_string(),
                label: "Empty".to_string(),
            },
            LineCategory {
                key: "ready".to_string(),
                label: "Ready".to_string(),
            },
        ];
        let series = vec![
            LineSeries::new(
                "actual",
                "Actual",
                "var(--color-primary)",
                vec![LinePoint::missing(), LinePoint::missing()],
            ),
            LineSeries::new(
                "target",
                "Target",
                "var(--color-accent)",
                vec![
                    LinePoint::missing(),
                    LinePoint::new(42.0).with_display_value("42 resolved"),
                ],
            ),
        ];

        let chart = normalize::normalize_categorical(&categories, &series);
        let state = initial_categorical_state(&chart).expect("a finite category");

        assert_eq!(state.category_index, 1);
        assert_eq!(state.category_key, "ready");
        assert_eq!(state.preferred_series, "target");
        assert_eq!(
            category_accessible_values(&chart, state.category_index, None),
            "Target 42 resolved"
        );
    }

    #[test]
    fn categorical_marker_color_overrides_series_marker_fill() {
        let categories = vec![LineCategory {
            key: "week-01".to_string(),
            label: "W01".to_string(),
        }];
        let series = vec![LineSeries {
            id: "actual".to_string(),
            name: "Actual".to_string(),
            points: vec![LinePoint {
                value: Some(42.0),
                display_value: None,
                data_label: None,
                marker_color: Some("var(--color-success)".to_string()),
            }],
            color: "var(--color-primary)".to_string(),
            pattern: LinePattern::Solid,
            marker: MarkerStyle {
                fill: Some("var(--color-secondary)".to_string()),
                ..MarkerStyle::default()
            },
            show_data_labels: false,
            label_placement: LineLabelPlacement::default(),
            axis: LineValueAxis::Primary,
        }];

        let chart = normalize::normalize_categorical(&categories, &series);

        assert_eq!(
            marker_fill(
                &chart.series[0],
                chart.series[0].points[0].marker_color.as_deref()
            ),
            "var(--color-success)"
        );
    }

    /// The whole no-regression claim in one place: with no series naming an
    /// axis, the captions, the accessible values and the axis-attribution
    /// state are exactly what they were before a second axis existed.
    #[test]
    fn a_single_axis_chart_keeps_bare_series_names_everywhere() {
        let chart = normalize::normalize_categorical(
            &[LineCategory {
                key: "week-01".to_string(),
                label: "W01".to_string(),
            }],
            &[
                LineSeries::new("actual", "Actual", "blue", vec![LinePoint::new(42.0)]),
                LineSeries::new("target", "Target", "red", vec![LinePoint::new(48.0)]),
            ],
        )
        .with_axes(LineAxes::default());

        assert!(!chart.has_secondary_axis());
        assert_eq!(series_caption(&chart.series[0], None), "Actual");
        assert_eq!(series_caption(&chart.series[1], None), "Target");
        assert_eq!(
            category_accessible_values(&chart, 0, None),
            "Actual 42, Target 48"
        );
    }

    /// And the same surfaces once two axes are on screen: one attribution,
    /// built once, with each value carrying its own axis' unit.
    #[test]
    fn a_dual_axis_chart_names_each_series_axis_once() {
        let chart = normalize::normalize_categorical(
            &[LineCategory {
                key: "week-01".to_string(),
                label: "W01".to_string(),
            }],
            &[
                LineSeries::new("opened", "Opened", "blue", vec![LinePoint::new(120.0)]),
                LineSeries::new(
                    "first-response",
                    "First response",
                    "orange",
                    vec![LinePoint::new(41.0)],
                )
                .on_secondary_axis(),
            ],
        )
        .with_axes(LineAxes {
            primary: LineAxisOptions::default().with_label("Conversations"),
            secondary: LineAxisOptions::default()
                .with_label("Duration")
                .with_unit(" s")
                .with_decimals(1),
        });
        let texts = LineChartTexts::default();
        let axis_names = Some((
            format::axis_name(LineValueAxis::Primary, &chart.axes, &texts),
            format::axis_name(LineValueAxis::Secondary, &chart.axes, &texts),
        ));

        assert!(chart.has_secondary_axis());
        assert_eq!(
            series_caption(&chart.series[0], axis_names.as_ref()),
            "Opened (Conversations)"
        );
        assert_eq!(
            series_caption(&chart.series[1], axis_names.as_ref()),
            "First response (Duration)"
        );
        assert_eq!(
            category_accessible_values(&chart, 0, axis_names.as_ref()),
            "Opened (Conversations) 120, First response (Duration) 41.0 s"
        );
    }

    /// A caller may configure the secondary axis and assign nothing to it. The
    /// axis must then not exist at all — no domain, no gutter, no attribution.
    #[test]
    fn configuring_a_secondary_axis_without_assigning_a_series_renders_no_second_axis() {
        let chart = normalize::normalize_categorical(
            &[LineCategory {
                key: "week-01".to_string(),
                label: "W01".to_string(),
            }],
            &[LineSeries::new(
                "actual",
                "Actual",
                "blue",
                vec![LinePoint::new(42.0)],
            )],
        )
        .with_axes(LineAxes {
            primary: LineAxisOptions::default(),
            secondary: LineAxisOptions::default()
                .with_label("Duration")
                .with_unit(" s"),
        });

        assert!(!chart.has_secondary_axis());
        assert_eq!(chart.secondary_domain, None);
        assert_eq!(
            chart.series[0].format.unit, None,
            "the unassigned axis' unit must not leak onto a primary series"
        );
        assert_eq!(
            geometry::plot_bounds(
                400.0,
                200.0,
                geometry::PlotInsets {
                    max_marker_radius: 4.0,
                    secondary_ticks: chart.has_secondary_axis(),
                    ..geometry::PlotInsets::default()
                }
            ),
            geometry::plot_bounds(
                400.0,
                200.0,
                geometry::PlotInsets {
                    max_marker_radius: 4.0,
                    ..geometry::PlotInsets::default()
                }
            ),
            "no right gutter is reserved for an axis that is not drawn"
        );
    }

    #[test]
    fn categorical_zero_dimensions_use_a_finite_one_unit_viewbox() {
        assert_eq!(resolve_dimensions(0, 0), (1, 1));
    }

    #[test]
    fn empty_categorical_chart_keeps_a_named_svg_shell_without_interaction_targets() {
        let categories = vec![LineCategory {
            key: "week-01".to_string(),
            label: "W01".to_string(),
        }];
        let series = vec![LineSeries::new(
            "actual",
            "Actual",
            "var(--color-primary)",
            vec![LinePoint::missing()],
        )];

        let chart = normalize::normalize_categorical(&categories, &series);

        assert!(initial_categorical_state(&chart).is_none());
    }

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

    /// `ldui-b3rp`: opposite placements are constraints, not merely fixed
    /// offsets. A higher Below-series marker can otherwise cancel the two
    /// offsets and put its label directly on an Above-series label.
    #[test]
    fn converging_series_labels_separate_in_their_declared_directions() {
        // These are the pre-fix W02 baseline positions from the showcase:
        // Chromium measured only 1.96 viewBox units between 20px glyph boxes.
        let occupied = [191.46];
        let colliding = 193.42;

        let below = resolve_data_label_y(colliding, LineLabelPlacement::Below, &occupied);
        assert!(
            below - occupied[0] >= DATA_LABEL_BASELINE_SEPARATION,
            "Below must move downward past the occupied label: {below}"
        );

        let above = resolve_data_label_y(colliding, LineLabelPlacement::Above, &occupied);
        assert!(
            occupied[0] - above >= DATA_LABEL_BASELINE_SEPARATION,
            "Above must move upward past the occupied label: {above}"
        );
    }

    #[test]
    fn label_resolution_rechecks_every_occupied_lane() {
        // Clearing 90 upward lands near 60, so a one-pass resolver would
        // simply exchange one collision for another.
        let occupied = [60.0, 90.0];
        let resolved = resolve_data_label_y(100.0, LineLabelPlacement::Above, &occupied);
        assert!(
            occupied
                .iter()
                .all(|other| (resolved - other).abs() >= DATA_LABEL_BASELINE_SEPARATION)
        );
        assert!(resolved < 60.0, "Above keeps moving upward: {resolved}");

        let clear = resolve_data_label_y(130.0, LineLabelPlacement::Below, &occupied);
        assert_eq!(clear, 130.0, "a clear lane must not drift");
    }
}
