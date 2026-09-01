mod format;
mod geometry;
mod interaction;
mod normalize;
mod types;

pub use types::{
    BarChartActivation, BarChartActivationSource, BarChartData, BarChartDataSource, BarChartItem,
    BarChartLayout, BarChartModifiers, BarChartTexts, BarInteractionMode, BarStatus,
    BarValueFormat,
};

use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;

use super::paint::{paint_attrs, stroke_attrs};
use geometry::{
    BarRect, Bounds, Insets, band, bar_rect, cap_line, n, plot_bounds, slot, zero_position,
};
use interaction::{Action, BarInteraction, Nav};
use normalize::{Domain, NormalizedBar, NormalizedBarChart, normalize, resolve_color};

/// Per-instance sequence for SVG title, description and focus-target ids, so
/// several bar charts can coexist without ARIA ids colliding.
static BAR_CHART_SEQ: AtomicU64 = AtomicU64::new(0);

/// SVG-based bar chart component.
///
/// Two data surfaces share one geometry. The original positional
/// `Vec<(String, f64)>` renders exactly as it always did, and a typed
/// [`BarChartData::Categorical`] adds stable keys, caller-owned status,
/// per-item colour, reactive copy, an accessible data table and optional
/// keyboard/pointer activation.
///
/// Bars are measured from a **zero line that is always on the axis**, so a
/// negative value extends the other way from it instead of producing invalid
/// geometry. For all-positive data the zero line is the plot's bottom (or left)
/// edge and every coordinate is what it was before signed support existed.
///
/// Legacy positional data (source-compatible — a plain vector still just works):
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::charts::BarChart;
///
/// #[component]
/// fn ClosedByDay() -> impl IntoView {
///     view! {
///         <BarChart
///             data=vec![("Mon".to_string(), 4.0), ("Tue".to_string(), 7.0)]
///             bar_colors=vec!["var(--color-error)".to_string()]
///             height=180
///         />
///     }
/// }
/// ```
///
/// Typed diverging data — a signed decomposition of one measure, sorted
/// most-dragging-first by the caller. Colour, judgement and value travel in one
/// item, so sorting cannot pair a value with a neighbour's colour:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::charts::{
///     BarChart, BarChartActivation, BarChartItem, BarChartLayout, BarStatus,
/// };
///
/// #[component]
/// fn DeltaToBaseline() -> impl IntoView {
///     let items = vec![
///         BarChartItem::new("north", "North", -12.5)
///             .with_display_value("-12.5 vs baseline")
///             .with_status(BarStatus::Unfavorable),
///         BarChartItem::new("east", "East", 0.0),
///         BarChartItem::new("west", "West", 8.0).with_status(BarStatus::Favorable),
///         BarChartItem::missing("south", "South"),
///     ];
///     let on_bar_activate = Callback::new(|intent: BarChartActivation| {
///         // The host maps this key to its own route, filter or request.
///         let _ = intent.category_key;
///     });
///
///     view! {
///         <BarChart
///             data=items
///             layout=BarChartLayout::DivergingHorizontal
///             accessible_label="Current minus trailing baseline by office".to_string()
///             on_bar_activate=on_bar_activate
///         />
///     }
/// }
/// ```
#[component]
pub fn BarChart(
    /// Static or reactive chart data. A legacy `Vec<(String, f64)>` converts
    /// automatically, as does a `Vec<BarChartItem>`.
    #[prop(into)]
    data: BarChartDataSource,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 400)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 200)]
    height: u32,
    /// Fill color for bars that carry neither their own colour nor a status.
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
    ///
    /// Being positional, it is mismatch-*safe* but not misalignment-safe: a
    /// caller that sorts its data without sorting this list repaints every bar
    /// with a neighbour's judgement, silently. Typed data carries its colour
    /// and status inside [`BarChartItem`] for exactly that reason, and a typed
    /// item's own colour or status wins over this list.
    #[prop(optional)]
    bar_colors: Option<Vec<String>>,
    /// If true, render horizontal bars instead of vertical. Superseded by
    /// `layout`, and kept because every existing caller passes it.
    #[prop(default = false)]
    horizontal: bool,
    /// Bar orientation, and whether the zero rule is always drawn. Defaults to
    /// [`BarChartLayout::Auto`], which follows `horizontal`.
    #[prop(default = BarChartLayout::Auto)]
    layout: BarChartLayout,
    /// Controls typed-data interaction; defaults to automatic. Legacy
    /// positional data is never interactive whatever this says.
    #[prop(default = BarInteractionMode::Auto)]
    interaction_mode: BarInteractionMode,
    /// Accessible name for the typed chart.
    #[prop(into, default = Signal::stored("Bar chart".to_string()))]
    accessible_label: Signal<String>,
    /// Optional longer description for the typed chart.
    #[prop(optional, into)]
    description: MaybeProp<String>,
    /// Whether typed data includes its accessible table; defaults to true.
    #[prop(default = true)]
    show_data_table: bool,
    /// Unit and precision for every value this chart states. Defaults to the
    /// one-decimal, unit-less rendering the chart already produced.
    #[prop(optional, into)]
    value_format: BarValueFormat,
    /// Chart copy that is not supplied per item, including the empty state.
    /// Reactive, so a locale change re-renders the words without touching
    /// keys, values, order, focus or selection.
    #[prop(into, default = Signal::stored(BarChartTexts::default()))]
    texts: Signal<BarChartTexts>,
    /// Optional callback invoked by a typed bar activation. Without it the
    /// chart still navigates and describes itself, but claims no button
    /// behaviour and adds no `role="button"`.
    #[prop(optional)]
    on_bar_activate: Option<Callback<BarChartActivation>>,
) -> impl IntoView {
    let data = Memo::new(move |_| data.get());
    let instance = BAR_CHART_SEQ.fetch_add(1, Ordering::Relaxed);
    let legacy_colors = StoredValue::new(bar_colors.unwrap_or_default());
    let color = StoredValue::new(color);
    let value_format = StoredValue::new(value_format);
    let description = Signal::derive(move || {
        description
            .get()
            .unwrap_or_else(|| "Categorical bar chart".to_string())
    });

    // Interaction state lives outside the data-driven render, so hover, focus
    // and the roving tab stop survive a data replacement and are *reconciled*
    // against it rather than reset.
    let state = RwSignal::new(BarInteraction::default());
    let previous_keys: StoredValue<Option<Vec<String>>> = StoredValue::new(None);
    let refocus: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    on_cleanup(move || {
        if let Some(handle) = refocus.try_get_value().flatten() {
            handle.clear();
        }
    });

    // Reconcile by key whenever the typed data changes: focus follows a bar
    // through a sort, and a removed bar hands focus to whatever now occupies
    // its position, without firing any activation.
    Effect::new(move |_| {
        let current = data.get();
        if !matches!(current, BarChartData::Categorical(_)) {
            previous_keys.set_value(None);
            return;
        }
        let chart = normalize(&current);
        let next = chart.navigable_keys();
        let previous = previous_keys.get_value();
        previous_keys.set_value(Some(next.clone()));
        let Some(previous) = previous else {
            return;
        };
        if previous == next {
            return;
        }
        let old = state.get_untracked();
        let had_focus = old.focused_key.is_some();
        let reconciled = interaction::reduce(&old, Action::ReconcileData, &previous, &next);
        if reconciled != old {
            state.set(reconciled.clone());
        }
        let Some(focused) = reconciled.focused_key.filter(|_| had_focus) else {
            return;
        };
        let Some(index) = chart.bars.iter().position(|bar| bar.key == focused) else {
            return;
        };
        // Focus after the re-rendered targets exist in the DOM. The handle is
        // held so an unmount between scheduling and firing cannot leave a
        // closure reaching into a torn-down tree.
        let id = target_id(instance, index);
        if let Ok(handle) =
            set_timeout_with_handle(move || focus_svg_element(&id), std::time::Duration::ZERO)
        {
            if let Some(previous) = refocus.try_get_value().flatten() {
                previous.clear();
            }
            let _ = refocus.try_set_value(Some(handle));
        }
    });

    move || {
        let current = data.get();
        let legacy = matches!(current, BarChartData::Simple(_));
        let context = RenderContext {
            chart: normalize(&current),
            layout: layout.resolve(horizontal),
            width,
            height,
            color,
            legacy_colors,
            value_format,
            texts,
        };
        if legacy {
            return render_legacy(context);
        }
        let interactive = match interaction_mode {
            BarInteractionMode::Auto | BarInteractionMode::Enabled => true,
            BarInteractionMode::Disabled => false,
        };
        render_typed(
            context,
            TypedChrome {
                instance,
                interactive,
                accessible_label,
                description,
                show_data_table,
                on_bar_activate,
                state,
            },
        )
    }
}

/// Everything the marks need, bundled so the two render paths cannot drift in
/// what they pass.
struct RenderContext {
    chart: NormalizedBarChart,
    layout: BarChartLayout,
    width: u32,
    height: u32,
    color: StoredValue<String>,
    legacy_colors: StoredValue<Vec<String>>,
    value_format: StoredValue<BarValueFormat>,
    texts: Signal<BarChartTexts>,
}

impl RenderContext {
    /// The plot rectangle, resolved identically by the marks and the focus
    /// targets so a target can never sit off its own bar.
    fn bounds(&self, domain: Domain) -> (Insets, Bounds) {
        let insets = Insets::new(self.layout, domain.has_negative());
        (
            insets,
            plot_bounds(self.width as f64, self.height as f64, insets),
        )
    }
}

/// The accessible and interactive surfaces only typed data gets.
#[derive(Clone, Copy)]
struct TypedChrome {
    instance: u64,
    interactive: bool,
    accessible_label: Signal<String>,
    description: Signal<String>,
    show_data_table: bool,
    on_bar_activate: Option<Callback<BarChartActivation>>,
    state: RwSignal<BarInteraction>,
}

fn target_id(instance: u64, index: usize) -> String {
    format!("bar-chart-{instance}-bar-{index}")
}

/// Focuses a bar's keyboard target after roving navigation.
#[cfg(target_arch = "wasm32")]
fn focus_svg_element(id: &str) {
    use wasm_bindgen::JsCast;

    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
    else {
        return;
    };
    let Ok(value) =
        js_sys::Reflect::get(element.as_ref(), &wasm_bindgen::JsValue::from_str("focus"))
    else {
        return;
    };
    let Ok(focus) = value.dyn_into::<js_sys::Function>() else {
        return;
    };
    let _ = focus.call0(element.as_ref());
}

/// Native tests have no SVG document to focus.
#[cfg(not(target_arch = "wasm32"))]
fn focus_svg_element(_id: &str) {}

/// The typed activation intent for `bar`, or `None` when it carries no finite
/// value — which is what keeps a gap in the data from ever reaching a host as a
/// fabricated zero.
fn activation_for(
    bar: &NormalizedBar,
    format: &BarValueFormat,
    texts: &BarChartTexts,
    source: BarChartActivationSource,
    modifiers: BarChartModifiers,
) -> Option<BarChartActivation> {
    let value = bar.value?;
    Some(BarChartActivation {
        category_key: bar.key.clone(),
        category_label: bar.label.clone(),
        value,
        display_value: bar.value_text(format, texts),
        status: bar.status,
        source,
        modifiers,
    })
}

/// The bar's accessible name: its label, its value stated exactly as the table
/// states it, and its judgement whenever the caller supplied one.
fn accessible_name(bar: &NormalizedBar, format: &BarValueFormat, texts: &BarChartTexts) -> String {
    let value = bar.value_text(format, texts);
    match bar.status {
        BarStatus::Neutral => format!("{}: {value}", bar.label),
        status => format!("{}: {value}, {}", bar.label, texts.status_text(status)),
    }
}

fn modifiers_of(shift: bool, ctrl: bool, alt: bool, meta: bool) -> BarChartModifiers {
    BarChartModifiers {
        shift,
        ctrl,
        alt,
        meta,
    }
}

/// The SVG's own role.
///
/// `role="img"` makes every descendant presentational, which contradicts the
/// focusable targets inside (axe: nested-interactive, and the reactivity lane
/// carries a zero-blocking axe gate). An interactive chart is therefore a named
/// group; only the target-less descriptive render keeps the pure-image role.
/// Same rule as `LineChart` (ldui-9tr.6).
fn svg_role(interactive: bool) -> &'static str {
    if interactive { "group" } else { "img" }
}

/// A focus target's role.
///
/// Only a wired callback earns button semantics. A chart that merely describes
/// itself must not announce every bar as a button a reader can press, because
/// pressing one would do nothing.
fn target_role(has_activation: bool) -> &'static str {
    if has_activation { "button" } else { "group" }
}

/// Whether a zero rule is drawn.
///
/// A vertical chart has always drawn its baseline, and for all-positive data
/// the zero line *is* that baseline, so nothing moves. A horizontal chart never
/// drew one, so it gains one only when the data actually reaches below zero or
/// the caller asked for the diverging layout — where the rule is the reference
/// every bar is read against and must stay visible under any filtering.
fn draws_zero_rule(layout: BarChartLayout, domain: Domain) -> bool {
    match layout {
        BarChartLayout::DivergingHorizontal => true,
        BarChartLayout::Horizontal => domain.has_negative(),
        BarChartLayout::Auto | BarChartLayout::Vertical => true,
    }
}

/// The dash pattern a status cap is drawn with, or `None` for a bar carrying no
/// judgement. See `geometry::cap_line`.
fn status_dash(status: BarStatus) -> Option<&'static str> {
    match status {
        BarStatus::Neutral => None,
        BarStatus::Favorable => Some("none"),
        BarStatus::Unfavorable => Some("3 2"),
    }
}

/// The empty placeholder both paths draw, with its copy supplied rather than
/// hardcoded.
fn empty_text_view(width: u32, height: u32, texts: Signal<BarChartTexts>) -> AnyView {
    view! {
        <text data-bar-chart-empty="" x=format!("{}", width / 2) y=format!("{}", height / 2)
            text-anchor="middle" fill="currentColor" font-size="14">
            {move || texts.with(|texts| texts.empty.clone())}
        </text>
    }
    .into_any()
}

/// The bars, their labels, their value text and the zero rule.
///
/// Shared by both render paths, so a typed chart and a legacy one cannot drift
/// apart in geometry: they are literally the same code.
fn render_marks(context: &RenderContext) -> AnyView {
    let layout = context.layout;
    let Some(domain) = context.chart.domain else {
        return empty_text_view(context.width, context.height, context.texts);
    };
    let (insets, bounds) = context.bounds(domain);
    let count = context.chart.bars.len();
    let zero = zero_position(domain, bounds, layout);
    let format = context.value_format.get_value();
    let legacy_colors = context.legacy_colors.get_value();
    let color = context.color.get_value();

    let zero_rule = draws_zero_rule(layout, domain).then(|| {
        let (rule_stroke, rule_style) = stroke_attrs("currentColor".to_string());
        let (x1, y1, x2, y2) = if layout.is_horizontal() {
            (zero, bounds.top, zero, bounds.bottom)
        } else {
            (bounds.left, zero, bounds.right, zero)
        };
        view! {
            <line data-bar-chart-zero-rule="" x1=n(x1) y1=n(y1) x2=n(x2) y2=n(y2)
                stroke=rule_stroke style=rule_style stroke-opacity="0.3" stroke-width="1" />
        }
    });

    let bars = context
        .chart
        .bars
        .iter()
        .enumerate()
        .map(|(index, bar)| {
            let label = category_label_view(layout, bounds, insets, index, count, bar);
            let Some(value) = bar.value else {
                // A missing measurement draws its label and nothing else: no
                // rect on the baseline that a reader would take for a zero.
                return view! {
                    <g data-bar-chart-bar="" data-bar-key=bar.key.clone() data-bar-missing="">
                        {label}
                    </g>
                }
                .into_any();
            };
            let paint = resolve_color(bar, index, &legacy_colors, &color);
            let rect = bar_rect(layout, bounds, domain, index, count, value);
            // A theme token must not ride on the `fill` presentation
            // attribute — see `crate::charts::paint`.
            let (bar_fill_attr, bar_fill_style) = paint_attrs(paint.clone());
            let cap = status_dash(bar.status).map(|dash| {
                let (cap_x1, cap_y1, cap_x2, cap_y2) = cap_line(layout, rect, value < 0.0);
                let (cap_stroke_attr, cap_stroke_style) = stroke_attrs(paint.clone());
                view! {
                    <line data-bar-chart-cap=bar.status.token() x1=n(cap_x1) y1=n(cap_y1)
                        x2=n(cap_x2) y2=n(cap_y2) stroke=cap_stroke_attr style=cap_stroke_style
                        stroke-width="3" stroke-dasharray=dash stroke-linecap="butt" />
                }
            });
            let value_label = value_label_view(layout, rect, value, bar, &format);
            view! {
                <g data-bar-chart-bar="" data-bar-key=bar.key.clone() data-status=bar.status.token()>
                    <rect x=n(rect.x) y=n(rect.y) width=n(rect.width) height=n(rect.height)
                        fill=bar_fill_attr style=bar_fill_style rx="2" />
                    {cap}
                    {label}
                    {value_label}
                </g>
            }
            .into_any()
        })
        .collect_view();

    view! {
        <>
            {zero_rule}
            {bars}
        </>
    }
    .into_any()
}

/// The category label, in the bottom row for a vertical chart and the left
/// gutter for a horizontal one.
fn category_label_view(
    layout: BarChartLayout,
    bounds: Bounds,
    insets: Insets,
    index: usize,
    count: usize,
    bar: &NormalizedBar,
) -> AnyView {
    let (label_fill_attr, label_fill_style) = paint_attrs("currentColor".to_string());
    let label = bar.label.clone();
    if layout.is_horizontal() {
        let (offset, thickness) = band(index, count, bounds.top, bounds.height());
        view! {
            <text x=n(bounds.left - insets.category_gutter_offset) y=n(offset + thickness / 2.0)
                text-anchor="end" dominant-baseline="middle" fill=label_fill_attr
                style=label_fill_style font-size="11">
                {label}
            </text>
        }
        .into_any()
    } else {
        let (offset, thickness) = band(index, count, bounds.left, bounds.width());
        view! {
            <text x=n(offset + thickness / 2.0) y=n(bounds.bottom + insets.category_label_offset)
                text-anchor="middle" fill=label_fill_attr style=label_fill_style font-size="11">
                {label}
            </text>
        }
        .into_any()
    }
}

/// The value text, always at the bar's *outward* end so a negative bar's label
/// sits beyond it rather than on top of its neighbour or its category name.
fn value_label_view(
    layout: BarChartLayout,
    rect: BarRect,
    value: f64,
    bar: &NormalizedBar,
    format: &BarValueFormat,
) -> AnyView {
    let (value_fill_attr, value_fill_style) = paint_attrs("currentColor".to_string());
    let text = bar
        .display_value
        .clone()
        .unwrap_or_else(|| format::value_text(value, format));
    let negative = value < 0.0;
    if layout.is_horizontal() {
        let x = if negative {
            rect.x - 5.0
        } else {
            rect.x + rect.width + 5.0
        };
        let anchor = if negative { "end" } else { "start" };
        view! {
            <text x=n(x) y=n(rect.y + rect.height / 2.0) text-anchor=anchor
                dominant-baseline="middle" fill=value_fill_attr style=value_fill_style
                font-size="10" opacity="0.7">
                {text}
            </text>
        }
        .into_any()
    } else {
        let y = if negative {
            rect.y + rect.height + 11.0
        } else {
            rect.y - 5.0
        };
        view! {
            <text x=n(rect.x + rect.width / 2.0) y=n(y) text-anchor="middle"
                fill=value_fill_attr style=value_fill_style font-size="10" opacity="0.7">
                {text}
            </text>
        }
        .into_any()
    }
}

/// The preserved positional surface: a bare SVG, with no wrapper, no roles, no
/// tab stops and no table — the element tree this chart has always produced.
fn render_legacy(context: RenderContext) -> AnyView {
    let viewbox = format!("0 0 {} {}", context.width, context.height);
    let marks = render_marks(&context);
    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            {marks}
        </svg>
    }
    .into_any()
}

/// The typed surface: a named group, a described SVG, focusable per-bar targets
/// and an equivalent data table.
fn render_typed(context: RenderContext, chrome: TypedChrome) -> AnyView {
    let TypedChrome {
        instance,
        interactive,
        accessible_label,
        description,
        show_data_table,
        on_bar_activate,
        state,
    } = chrome;
    let viewbox = format!("0 0 {} {}", context.width, context.height);
    let title_id = format!("bar-chart-{instance}-title");
    let desc_id = format!("bar-chart-{instance}-desc");
    let labelled_by = format!("{title_id} {desc_id}");
    let layout_token = context.layout.token();
    let interactive = interactive && !context.chart.is_empty();

    let svg_role = svg_role(interactive);
    let targets = interactive.then(|| focus_targets(&context, instance, state, on_bar_activate));
    let marks = render_marks(&context);
    let table = show_data_table.then(|| data_table(&context, accessible_label));
    let active_attr = move || {
        state
            .read()
            .active_key()
            .map(str::to_owned)
            .unwrap_or_default()
    };

    view! {
        <div data-testid="bar-chart" role="group" aria-label=move || accessible_label.get()
            data-bar-chart-layout=layout_token data-active-category=active_attr class="w-full">
            <svg data-bar-chart-plot role=svg_role aria-labelledby=labelled_by viewBox=viewbox
                class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
                <title id=title_id>{move || accessible_label.get()}</title>
                <desc id=desc_id>{move || description.get()}</desc>
                {marks}
                {targets}
            </svg>
            {table}
        </div>
    }
    .into_any()
}

/// One focusable, clickable target per activatable bar, spanning its whole
/// category slot so a zero-length bar is still reachable.
fn focus_targets(
    context: &RenderContext,
    instance: u64,
    state: RwSignal<BarInteraction>,
    on_bar_activate: Option<Callback<BarChartActivation>>,
) -> AnyView {
    let Some(domain) = context.chart.domain else {
        return ().into_any();
    };
    let layout = context.layout;
    let (_, bounds) = context.bounds(domain);
    let count = context.chart.bars.len();
    let texts = context.texts;
    let value_format = context.value_format;
    let target_role = target_role(on_bar_activate.is_some());
    // Navigation is over *keys*, and each key's DOM id is looked up here
    // rather than derived from a position, because the navigable list skips
    // missing bars while the ids are indexed by bar.
    let navigable: Vec<(String, String)> = context
        .chart
        .bars
        .iter()
        .enumerate()
        .filter(|(_, bar)| bar.is_activatable())
        .map(|(index, bar)| (bar.key.clone(), target_id(instance, index)))
        .collect();
    let ids = StoredValue::new(navigable.clone());
    let keys = StoredValue::new(
        navigable
            .iter()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>(),
    );
    let dispatch = move |action: Action| {
        let all = keys.get_value();
        let old = state.get_untracked();
        let next = interaction::reduce(&old, action, &all, &all);
        if next != old {
            state.set(next);
        }
    };
    let focus_current = move || {
        let Some(active) = state.get_untracked().focused_key else {
            return;
        };
        let id = ids.with_value(|ids| {
            ids.iter()
                .find(|(key, _)| *key == active)
                .map(|(_, id)| id.clone())
        });
        if let Some(id) = id {
            focus_svg_element(&id);
        }
    };

    context
        .chart
        .bars
        .iter()
        .enumerate()
        .filter(|(_, bar)| bar.is_activatable())
        .map(|(index, bar)| {
            let bar = bar.clone();
            let key = bar.key.clone();
            let id = target_id(instance, index);
            let (offset, extent) = if layout.is_horizontal() {
                slot(index, count, bounds.top, bounds.height())
            } else {
                slot(index, count, bounds.left, bounds.width())
            };
            let (x, y, w, h) = if layout.is_horizontal() {
                (bounds.left, offset, bounds.width(), extent)
            } else {
                (offset, bounds.top, extent, bounds.height())
            };
            let focused_key = key.clone();
            let is_focused =
                move || state.read().focused_key.as_deref() == Some(focused_key.as_str());
            let roving_key = key.clone();
            let is_roving = move || state.read().roving_key.as_deref() == Some(roving_key.as_str());
            let name_bar = bar.clone();
            let label = move || {
                texts.with(|texts| accessible_name(&name_bar, &value_format.get_value(), texts))
            };
            let activate = move |source: BarChartActivationSource, modifiers: BarChartModifiers| {
                let Some(callback) = on_bar_activate else {
                    return;
                };
                let payload = texts.with_untracked(|texts| {
                    activation_for(&bar, &value_format.get_value(), texts, source, modifiers)
                });
                if let Some(payload) = payload {
                    callback.run(payload);
                }
            };
            let key_activate = activate.clone();
            let on_key = move |ev: web_sys::KeyboardEvent| {
                let nav = match ev.key().as_str() {
                    "ArrowLeft" | "ArrowUp" => Some(Nav::Previous),
                    "ArrowRight" | "ArrowDown" => Some(Nav::Next),
                    "Home" => Some(Nav::First),
                    "End" => Some(Nav::Last),
                    _ => None,
                };
                if let Some(nav) = nav {
                    ev.prevent_default();
                    dispatch(Action::MoveFocus(nav));
                    focus_current();
                    return;
                }
                match ev.key().as_str() {
                    "Escape" => {
                        ev.prevent_default();
                        dispatch(Action::Dismiss);
                    }
                    // Inert without a callback: no preventDefault, no claimed
                    // button behaviour, so a purely descriptive chart never
                    // swallows a key the page itself wanted.
                    "Enter" | " " if on_bar_activate.is_some() => {
                        ev.prevent_default();
                        key_activate(
                            BarChartActivationSource::Keyboard,
                            modifiers_of(
                                ev.shift_key(),
                                ev.ctrl_key(),
                                ev.alt_key(),
                                ev.meta_key(),
                            ),
                        );
                    }
                    _ => {}
                }
            };
            let on_click = move |ev: web_sys::MouseEvent| {
                activate(
                    BarChartActivationSource::Pointer,
                    modifiers_of(ev.shift_key(), ev.ctrl_key(), ev.alt_key(), ev.meta_key()),
                );
            };
            let focus_dispatch_key = key.clone();
            let hover_dispatch_key = key.clone();
            view! {
                <rect id=id data-bar-chart-focus="" data-bar-key=key.clone()
                    x=n(x) y=n(y) width=n(w) height=n(h) fill="transparent" pointer-events="all"
                    role=target_role rx="3" stroke="currentColor" stroke-opacity="0.55"
                    stroke-width=move || if is_focused() { "2" } else { "0" }
                    tabindex=move || if is_roving() { "0" } else { "-1" }
                    aria-label=label
                    on:focus=move |_| dispatch(Action::Focused(focus_dispatch_key.clone()))
                    on:blur=move |_| dispatch(Action::Blurred)
                    on:pointerenter=move |_| dispatch(Action::Hovered(hover_dispatch_key.clone()))
                    on:pointerleave=move |_| dispatch(Action::HoverEnded)
                    on:keydown=on_key
                    on:click=on_click />
            }
        })
        .collect_view()
        .into_any()
}

/// The chart's non-visual truth: one row per item, stating the localized
/// label, the same value text the bar draws, and the caller's judgement in
/// words rather than only in colour.
fn data_table(context: &RenderContext, accessible_label: Signal<String>) -> AnyView {
    let texts = context.texts;
    let value_format = context.value_format;
    let rows = context
        .chart
        .bars
        .iter()
        .map(|bar| {
            let label = bar.label.clone();
            let value_bar = bar.clone();
            let status = bar.status;
            view! {
                <tr data-bar-key=bar.key.clone() data-status=bar.status.token()>
                    <th scope="row">{label}</th>
                    <td>
                        {move || {
                            texts.with(|texts| value_bar.value_text(&value_format.get_value(), texts))
                        }}
                    </td>
                    <td>{move || texts.with(|texts| texts.status_text(status).to_string())}</td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <table data-bar-chart-table class="sr-only">
            <caption>{move || accessible_label.get()}</caption>
            <thead>
                <tr>
                    <th scope="col">{move || texts.with(|texts| texts.category_header.clone())}</th>
                    <th scope="col">{move || texts.with(|texts| texts.value_header.clone())}</th>
                    <th scope="col">{move || texts.with(|texts| texts.status_header.clone())}</th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>
    }
    .into_any()
}

#[cfg(test)]
mod tests;
