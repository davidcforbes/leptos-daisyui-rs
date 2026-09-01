mod geometry;
mod interaction;
mod normalize;
mod scale;
mod types;

pub use scale::{HeatScale, HeatmapSense};
pub use types::{
    HeatmapActivation, HeatmapActivationSource, HeatmapCategory, HeatmapCell, HeatmapDataSource,
    HeatmapInteractionMode, HeatmapMatrix, HeatmapModifiers, HeatmapTexts, HeatmapValue,
};

use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;

use super::paint::{paint_attrs, stroke_attrs};
use geometry::{DEFAULT_PAD_LEFT, Frame, Padding, cell_rect, default_pad_top, frame};
use interaction::{Action, Axes, CellKey, HeatmapInteraction, Nav};
use normalize::{NormalizedCell, NormalizedHeatmap, normalize};
use scale::{HeatPalette, cell_fill};

/// Per-instance sequence for SVG title, description and focus-target ids, so
/// several heatmaps can coexist without ARIA ids colliding.
static HEATMAP_SEQ: AtomicU64 = AtomicU64::new(0);

/// SVG-based heatmap component for a generic N x M grid.
///
/// Two data surfaces share one geometry.
///
/// The original **positional** surface — `row_labels`, `col_labels` and a
/// `Vec<HeatmapCell>` addressed by array index — renders exactly as it always
/// did: the same rects, the same labels, no wrapper, no roles and no tab stops.
///
/// The **typed** surface (`data`, a [`HeatmapMatrix`]) adds everything a
/// localized accessible application page needs: stable row and column keys, an
/// accessible chart name and description, an equivalent semantic data table,
/// reactive copy, and — when a callback is wired — real focusable cells with
/// pointer and Enter/Space activation whose payload carries the two stable keys
/// rather than a pair of array indices.
///
/// The positional surface is source-compatible — this is exactly what every
/// existing caller writes, and it still compiles and still draws the element
/// tree it always drew:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::charts::{Heatmap, HeatmapCell};
///
/// #[component]
/// fn LegacyGrid() -> impl IntoView {
///     let cells = vec![HeatmapCell {
///         row: 0,
///         col: 1,
///         label: "12%".to_string(),
///         intensity: 0.6,
///     }];
///     let on_cell_click = Callback::new(|(row, col): (usize, usize)| {
///         let _ = (row, col);
///     });
///
///     view! {
///         <Heatmap
///             row_labels=vec!["North".to_string(), "South".to_string()]
///             col_labels=vec!["Closed".to_string(), "SLA met".to_string()]
///             cells=cells
///             max_cell_h=44.0
///             on_cell_click=on_cell_click
///         />
///     }
/// }
/// ```
///
/// ## Color scales
///
/// By default the grid is a single hue whose alpha carries magnitude. Set
/// `scale=HeatScale::Judgement` to turn it into a favorable/unfavorable axis:
/// each cell's intensity becomes signed, the sign picks the hue and the
/// magnitude still picks the alpha. The two hues default to daisyUI's
/// `--color-success` and `--color-error` theme tokens, so no new color enters
/// the palette and both follow a theme switch.
///
/// Color on that axis carries the **judgement**, never the category — the API
/// offers no per-cell color, only a signed number, so there is nothing to tint
/// a series with. The *sense* of a measure is the caller's sign convention and
/// is therefore per-value (hence per-column) rather than a global flag: negate
/// the deviation for a column where lower is better.
///
/// On the typed surface that judgement is **never colour alone**. A judged cell
/// also carries a solid (favorable) or dashed (unfavorable) sense rule, which
/// survives forced colours and a colour-vision deficiency, and states its
/// verdict in words in the data table.
///
/// ## Keyboard model
///
/// A heatmap is two-dimensional, so it navigates like the ARIA grid every other
/// data grid uses: Left/Right move along the row, Up/Down along the column,
/// Home/End jump to the first/last column **of the current row**, and
/// Ctrl+Home / Ctrl+End to the first/last cell of the whole grid. Every move
/// clamps rather than wrapping, Escape drops the highlight without moving the
/// tab stop, and the grid is one tab stop: Tab enters and leaves it.
///
/// ## The Office by KPI case
///
/// One office, twelve KPIs — the shape this component's typed surface was
/// built for. Each cell carries the localized KPI reading a screen reader
/// should hear, and an activation reports the office and the KPI by their
/// stable ids so a later drill cannot land on the wrong column:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::charts::{
///     HeatScale, Heatmap, HeatmapActivation, HeatmapCategory, HeatmapMatrix, HeatmapValue,
/// };
///
/// #[component]
/// fn OfficeScorecard() -> impl IntoView {
///     let kpis = [
///         ("closed", "Matters closed"),
///         ("sla", "SLA met"),
///         ("handle", "Handle time"),
///     ];
///     let columns: Vec<HeatmapCategory> = kpis
///         .iter()
///         .map(|(key, label)| HeatmapCategory::new(*key, *label))
///         .collect();
///     let matrix = HeatmapMatrix::new(
///         vec![HeatmapCategory::new("office-north", "North")],
///         columns,
///         vec![
///             HeatmapValue::new("office-north", "closed", 0.6)
///                 .with_display_value("+12%")
///                 .with_accessible_value("12 percent above the 12-week baseline"),
///             HeatmapValue::new("office-north", "sla", 0.2).with_display_value("+4%"),
///             // Lower is better for handle time, so the caller negates it.
///             HeatmapValue::missing("office-north", "handle"),
///         ],
///     );
///     let on_cell_activate = Callback::new(|intent: HeatmapActivation| {
///         // The host maps the two stable keys to its own route or request.
///         let _ = (intent.row_key, intent.column_key);
///     });
///
///     view! {
///         <Heatmap
///             data=matrix
///             scale=HeatScale::Judgement
///             accessible_label="Current versus baseline by office and KPI".to_string()
///             on_cell_activate=on_cell_activate
///         />
///     }
/// }
/// ```
#[component]
pub fn Heatmap(
    /// Row labels, top-to-bottom. The **positional** surface; ignored when
    /// `data` is supplied.
    #[prop(optional)]
    row_labels: Vec<String>,
    /// Column labels, left-to-right. The **positional** surface; ignored when
    /// `data` is supplied.
    #[prop(optional)]
    col_labels: Vec<String>,
    /// Populated cells addressed by array position. A `(row, col)` not present
    /// in this list renders as transparent (no rect drawn for that grid
    /// position). The **positional** surface; ignored when `data` is supplied.
    #[prop(optional)]
    cells: Vec<HeatmapCell>,
    /// Typed rows, columns and values, static or reactive. Supplying it selects
    /// the accessible/interactive render; leaving it out keeps the positional
    /// one exactly as it was.
    #[prop(optional, into)]
    data: HeatmapDataSource,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 500)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 250)]
    height: u32,
    /// Tint base as a CSS `<r> <g> <b>` triplet, e.g. `"220 38 38"`. Cell
    /// fill is `rgb(<rgb> / <alpha>)` where `alpha = intensity * 0.55`.
    /// Used by [`HeatScale::Magnitude`] only.
    #[prop(default = "220 38 38".to_string())]
    rgb: String,
    /// Which color scale the cells use (ldui-7zj). Defaults to
    /// [`HeatScale::Magnitude`] — the legacy single-hue behavior — so existing
    /// callers render exactly as before. [`HeatScale::Judgement`] switches to
    /// the signed favorable/unfavorable axis.
    #[prop(default = HeatScale::Magnitude)]
    scale: HeatScale,
    /// Hue for favorable (positive-intensity) cells under
    /// [`HeatScale::Judgement`]. Defaults to daisyUI's `--color-success` theme
    /// token so the heatmap introduces no new color and follows theme changes.
    ///
    /// Pass a daisyUI theme token — `"var(--color-success)"`,
    /// `"var(--color-warning)"`, `"var(--color-error)"` or
    /// `"var(--color-info)"`. Staying on the tokens is the point: the palette
    /// should not grow a hue the rest of the app does not already use, and
    /// tokens follow a theme switch. Ignored under [`HeatScale::Magnitude`].
    #[prop(default = "var(--color-success)".to_string())]
    favorable_color: String,
    /// Hue for unfavorable (negative-intensity) cells under
    /// [`HeatScale::Judgement`]. Defaults to daisyUI's `--color-error` theme
    /// token; pass `"var(--color-warning)"` for a softer at-risk read. Same
    /// token guidance as `favorable_color`. Ignored under
    /// [`HeatScale::Magnitude`].
    #[prop(default = "var(--color-error)".to_string())]
    unfavorable_color: String,
    /// When `true`, column header labels rotate -45deg around their anchor
    /// (for wide grids, e.g. a 16-column VaR matrix).
    #[prop(default = false)]
    slant_col_labels: bool,
    /// Optional left-padding override (space reserved for row labels).
    /// Defaults to 100.0; raise it when row labels are long enough to clip
    /// (e.g. the VaR matrix's "U-Visa Investigation" / "*Est." prefixes).
    /// bd_4iiz-inventory-43e.
    #[prop(optional)]
    pad_left: Option<f64>,
    /// Optional top-padding override (space reserved for column headers).
    /// Defaults to 70.0 when `slant_col_labels` else 30.0; raise it when
    /// slanted headers overlap the first cell row. bd_4iiz-inventory-43e.
    #[prop(optional)]
    pad_top: Option<f64>,
    /// Optional per-row height cap in px. When set, each row is drawn at
    /// `min(natural_row_height, max_cell_h)` and the SVG viewBox height shrinks
    /// to fit exactly `pad_top + n_rows*row_h + pad_bottom` — so a grid with
    /// few rows in a tall viewport renders compact tiles instead of giant
    /// stretched bricks with large inter-row gaps (bd_4iiz-inventory-toe.4).
    /// `None` keeps the legacy stretch-to-fill behavior.
    #[prop(optional)]
    max_cell_h: Option<f64>,
    /// Optional per-cell click handler, called with `(row, col)` (0-based, in
    /// the same index space as `row_labels`/`col_labels`). When set, EVERY grid
    /// position — including empty ones — becomes clickable via a transparent
    /// overlay rect, so a consumer can drill from a cell whether or not it drew
    /// a tile there. Purely additive: `None` (the default) is the legacy
    /// non-interactive heatmap, unchanged for every existing consumer.
    ///
    /// It is also the **migration path**: on the typed surface it still fires,
    /// with the activated cell's current row and column indices, alongside
    /// `on_cell_activate`. A caller can therefore adopt [`HeatmapMatrix`]
    /// without rewriting its handler in the same commit — but the indices are
    /// positions in the *current* render and re-point the moment either axis is
    /// sorted or filtered, which is exactly why [`HeatmapActivation`] carries
    /// keys instead.
    #[prop(optional)]
    on_cell_click: Option<Callback<(usize, usize)>>,
    /// Accessible name for the typed heatmap.
    #[prop(into, default = Signal::stored("Heatmap".to_string()))]
    accessible_label: Signal<String>,
    /// Optional longer description for the typed heatmap.
    #[prop(optional, into)]
    description: MaybeProp<String>,
    /// Whether the typed heatmap includes its accessible data table; defaults
    /// to true. Turning it off removes the grid's only non-visual
    /// representation, so do it only when the surrounding page already states
    /// the same matrix.
    #[prop(default = true)]
    show_data_table: bool,
    /// Controls typed-cell interaction; defaults to
    /// [`HeatmapInteractionMode::Auto`], which is interactive exactly when an
    /// activation callback is wired. The positional surface is never
    /// interactive beyond its legacy click overlay, whatever this says.
    #[prop(default = HeatmapInteractionMode::Auto)]
    interaction_mode: HeatmapInteractionMode,
    /// Chart copy that is not supplied per cell, including the empty state.
    /// Reactive, so a locale change re-renders the words without touching
    /// keys, intensities, order, focus or the identity an activation reports.
    #[prop(into, default = Signal::stored(HeatmapTexts::default()))]
    texts: Signal<HeatmapTexts>,
    /// Optional callback invoked by a typed cell activation. Without it the
    /// heatmap still describes itself and tabulates its values, but claims no
    /// button behaviour and adds no `role="button"`.
    #[prop(optional)]
    on_cell_activate: Option<Callback<HeatmapActivation>>,
) -> impl IntoView {
    let padding = Padding {
        left: pad_left.unwrap_or(DEFAULT_PAD_LEFT),
        top: pad_top.unwrap_or_else(|| default_pad_top(slant_col_labels)),
    };
    let style = RenderStyle {
        width,
        height,
        padding,
        max_cell_h,
        slant_col_labels,
        scale,
        rgb: StoredValue::new(rgb),
        favorable: StoredValue::new(favorable_color),
        unfavorable: StoredValue::new(unfavorable_color),
        texts,
    };

    if data.is_absent() {
        return render_positional(&style, row_labels, col_labels, cells, on_cell_click).into_any();
    }

    let instance = HEATMAP_SEQ.fetch_add(1, Ordering::Relaxed);
    let data = Memo::new(move |_| data.get());
    let description = Signal::derive(move || {
        description
            .get()
            .unwrap_or_else(|| "Categorical heatmap".to_string())
    });
    let interactive = match interaction_mode {
        HeatmapInteractionMode::Auto => on_cell_activate.is_some() || on_cell_click.is_some(),
        HeatmapInteractionMode::Enabled => true,
        HeatmapInteractionMode::Disabled => false,
    };

    // Interaction state lives outside the data-driven render, so hover, focus
    // and the roving tab stop survive a data replacement and are *reconciled*
    // against it rather than reset.
    let state = RwSignal::new(HeatmapInteraction::default());
    let previous_axes: StoredValue<Option<Axes>> = StoredValue::new(None);
    let refocus: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    on_cleanup(move || {
        if let Some(handle) = refocus.try_get_value().flatten() {
            handle.clear();
        }
    });

    // Reconcile by key whenever the matrix changes: focus follows a cell
    // through a sort of either axis, and a removed row or column hands focus to
    // whatever now occupies its position, without firing any activation.
    Effect::new(move |_| {
        let grid = normalize(&data.get());
        let next = Axes::new(grid.row_keys(), grid.column_keys());
        let previous = previous_axes.get_value();
        previous_axes.set_value(Some(next.clone()));
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
        let Some((row, column)) = grid.iter().find_map(|(row, column, cell)| {
            (cell.row_key == focused.row && cell.column_key == focused.column)
                .then_some((row, column))
        }) else {
            return;
        };
        // Focus after the re-rendered targets exist in the DOM. The handle is
        // held so an unmount between scheduling and firing cannot leave a
        // closure reaching into a torn-down tree.
        let id = target_id(instance, row, column);
        if let Ok(handle) =
            set_timeout_with_handle(move || focus_svg_element(&id), std::time::Duration::ZERO)
        {
            if let Some(previous) = refocus.try_get_value().flatten() {
                previous.clear();
            }
            let _ = refocus.try_set_value(Some(handle));
        }
    });

    let chrome = TypedChrome {
        instance,
        interactive,
        accessible_label,
        description,
        show_data_table,
        on_cell_activate,
        on_cell_click,
        state,
    };
    (move || render_typed(&style, &normalize(&data.get()), chrome)).into_any()
}

/// Everything about how a heatmap is drawn that does not come from the data.
/// Copied into both render paths so they cannot drift in geometry or colour.
#[derive(Clone, Copy)]
struct RenderStyle {
    width: u32,
    height: u32,
    padding: Padding,
    max_cell_h: Option<f64>,
    slant_col_labels: bool,
    scale: HeatScale,
    rgb: StoredValue<String>,
    favorable: StoredValue<String>,
    unfavorable: StoredValue<String>,
    texts: Signal<HeatmapTexts>,
}

impl RenderStyle {
    fn frame(&self, n_rows: usize, n_cols: usize) -> Frame {
        frame(
            n_rows,
            n_cols,
            self.width,
            self.height,
            self.padding,
            self.max_cell_h,
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
    on_cell_activate: Option<Callback<HeatmapActivation>>,
    on_cell_click: Option<Callback<(usize, usize)>>,
    state: RwSignal<HeatmapInteraction>,
}

fn target_id(instance: u64, row: usize, column: usize) -> String {
    format!("heatmap-{instance}-r{row}-c{column}")
}

/// Focuses a cell's keyboard target after roving navigation.
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

/// The SVG's own role.
///
/// `role="img"` makes every descendant presentational, which contradicts the
/// focusable targets inside (axe: nested-interactive, and the reactivity lane
/// carries a zero-blocking axe gate). An interactive heatmap is therefore a
/// named group; only the target-less descriptive render keeps the pure-image
/// role. Same rule as `LineChart` (ldui-9tr.6) and `BarChart` (ldui-y2ed).
fn svg_role(interactive: bool) -> &'static str {
    if interactive { "group" } else { "img" }
}

/// A focus target's role.
///
/// Only a wired callback earns button semantics. A grid a reader may explore
/// but not act on must not announce every cell as a button, because pressing
/// one would do nothing.
fn target_role(has_activation: bool) -> &'static str {
    if has_activation { "button" } else { "group" }
}

/// What a reader hears when focus lands on a cell.
///
/// Both axis names are stated because an SVG target has no table structure to
/// borrow them from: without them a cell announces `"North, Closed, +12%"` with
/// no clue which half is which. Every word comes from the supplied copy, and
/// the value half resolves exactly as the data table's cell does.
fn accessible_name(cell: &NormalizedCell, scale: HeatScale, texts: &HeatmapTexts) -> String {
    format!(
        "{}: {}, {}: {}, {}: {}",
        texts.row_header,
        cell.row_label,
        texts.column_header,
        cell.column_label,
        texts.value_header,
        cell.stated_text(scale, texts)
    )
}

/// The typed activation intent for `cell`.
///
/// Every grid position produces one, including a position with no measurement:
/// a heatmap cell is a *coordinate* — this office by that KPI — and a reader
/// drilling into it is asking about the coordinate, not about a number that may
/// not exist. The intensity is therefore an `Option` rather than a fabricated
/// zero, and the display value states the localized missing copy.
fn activation_for(
    cell: &NormalizedCell,
    scale: HeatScale,
    texts: &HeatmapTexts,
    source: HeatmapActivationSource,
    modifiers: HeatmapModifiers,
) -> HeatmapActivation {
    HeatmapActivation {
        row_key: cell.row_key.clone(),
        row_label: cell.row_label.clone(),
        column_key: cell.column_key.clone(),
        column_label: cell.column_label.clone(),
        intensity: cell.intensity,
        display_value: cell.value_text(texts),
        sense: cell.sense(scale),
        source,
        modifiers,
    }
}

fn modifiers_of(shift: bool, ctrl: bool, alt: bool, meta: bool) -> HeatmapModifiers {
    HeatmapModifiers {
        shift,
        ctrl,
        alt,
        meta,
    }
}

/// The empty placeholder both paths draw, with its copy supplied rather than
/// hardcoded.
fn empty_view(style: &RenderStyle) -> AnyView {
    let RenderStyle {
        width,
        height,
        texts,
        ..
    } = *style;
    view! {
        <svg data-heatmap-empty="" viewBox=format!("0 0 {width} {height}") class="w-full h-auto"
            xmlns="http://www.w3.org/2000/svg">
            <text x=format!("{}", width / 2) y=format!("{}", height / 2)
                text-anchor="middle" fill="currentColor" font-size="14">
                {move || texts.with(|texts| texts.no_data.clone())}
            </text>
        </svg>
    }
    .into_any()
}

/// The row labels down the left gutter.
///
/// `decorative` marks them `aria-hidden`, which the typed path does when its
/// data table restates them — otherwise a screen reader reads the axis twice,
/// once as a stream of bare words with no structure.
fn row_label_views(labels: &[String], frame: Frame, decorative: bool) -> AnyView {
    let hidden = decorative.then_some("true");
    labels
        .iter()
        .enumerate()
        .map(|(ri, label)| {
            let x = format!("{:.2}", frame.layout.pad_left - 8.0);
            let y = format!(
                "{:.2}",
                frame.layout.pad_top + ri as f64 * frame.cell_h + frame.cell_h / 2.0
            );
            let label = label.clone();
            view! {
                <text x=x y=y text-anchor="end" dominant-baseline="middle"
                    fill="currentColor" font-size="11" aria-hidden=hidden>
                    {label}
                </text>
            }
        })
        .collect_view()
        .into_any()
}

/// The column headers above the grid, slanted when asked.
fn col_label_views(labels: &[String], frame: Frame, slant: bool, decorative: bool) -> AnyView {
    let hidden = decorative.then_some("true");
    labels
        .iter()
        .enumerate()
        .map(|(ci, label)| {
            let x = frame.layout.pad_left + ci as f64 * frame.cell_w + frame.cell_w / 2.0;
            let y = frame.layout.pad_top - 8.0;
            let x_str = format!("{x:.2}");
            let y_str = format!("{y:.2}");
            let label = label.clone();
            if slant {
                // Rise UP-left from just above each column (positive rotate +
                // end-anchor sends the text body to negative-y), so long
                // headers never dip DOWN into the first cell row — the
                // `rotate(-45)` overlap bug (bd_4iiz-inventory-43e).
                let t = format!("rotate(45, {x:.2}, {y:.2})");
                view! {
                    <text x=x_str y=y_str text-anchor="end" fill="currentColor"
                        font-size="10" transform=t aria-hidden=hidden>
                        {label}
                    </text>
                }
                .into_any()
            } else {
                view! {
                    <text x=x_str y=y_str text-anchor="middle" fill="currentColor" font-size="10"
                        aria-hidden=hidden>
                        {label}
                    </text>
                }
                .into_any()
            }
        })
        .collect_view()
        .into_any()
}

/// The chart's original element tree, preserved exactly: one rect and one label
/// per supplied cell, the two axis label rows, and — only when a click handler
/// is supplied — the transparent overlay.
fn render_positional(
    style: &RenderStyle,
    row_labels: Vec<String>,
    col_labels: Vec<String>,
    cells: Vec<HeatmapCell>,
    on_cell_click: Option<Callback<(usize, usize)>>,
) -> AnyView {
    if row_labels.is_empty() || col_labels.is_empty() {
        return empty_view(style);
    }

    let n_rows = row_labels.len();
    let n_cols = col_labels.len();
    let frame = style.frame(n_rows, n_cols);
    let layout = frame.layout;
    let viewbox = format!("0 0 {} {:.0}", style.width, frame.height_eff);
    let rgb = style.rgb.get_value();
    let favorable = style.favorable.get_value();
    let unfavorable = style.unfavorable.get_value();
    let palette = HeatPalette {
        scale: style.scale,
        rgb: &rgb,
        favorable: &favorable,
        unfavorable: &unfavorable,
    };

    let cell_views = cells
        .into_iter()
        .map(|c| {
            let (x, y, w, h) = cell_rect(c.row, c.col, layout);
            // Exactly one of these is Some — see `paint_attrs` for why a
            // token-bearing colour cannot ride on the `fill` attribute.
            let (fill, style) = paint_attrs(cell_fill(c.intensity, &palette));
            let cx = format!("{:.2}", x + w / 2.0);
            let cy = format!("{:.2}", y + h / 2.0);
            view! {
                <rect x=format!("{x:.2}") y=format!("{y:.2}") width=format!("{w:.2}") height=format!("{h:.2}") fill=fill style=style />
                <text x=cx y=cy text-anchor="middle" dominant-baseline="middle"
                    fill="currentColor" font-size="9">
                    {c.label}
                </text>
            }
        })
        .collect_view();

    let row_label_views = row_label_views(&row_labels, frame, false);
    let col_label_views = col_label_views(&col_labels, frame, style.slant_col_labels, false);

    // Optional interaction overlay (beads-1qhd): one transparent rect per grid
    // position — including empty ones — so a click lands on any (row, col) even
    // where no tile was drawn. Only rendered when a handler is supplied, so the
    // legacy heatmap emits no extra DOM.
    let click_overlay = on_cell_click.map(|cb| {
        (0..n_rows)
            .flat_map(|ri| (0..n_cols).map(move |ci| (ri, ci)))
            .map(|(ri, ci)| {
                let (x, y, w, h) = cell_rect(ri, ci, layout);
                view! {
                    <rect
                        x=format!("{x:.2}")
                        y=format!("{y:.2}")
                        width=format!("{w:.2}")
                        height=format!("{h:.2}")
                        fill="transparent"
                        style="cursor: pointer"
                        on:click=move |_| cb.run((ri, ci))
                    />
                }
            })
            .collect_view()
    });

    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            {cell_views}
            {row_label_views}
            {col_label_views}
            {click_overlay}
        </svg>
    }
    .into_any()
}

/// The typed surface: a named group, a described SVG, an equivalent data table
/// and — when a callback is wired — focusable per-cell targets.
fn render_typed(style: &RenderStyle, grid: &NormalizedHeatmap, chrome: TypedChrome) -> AnyView {
    let TypedChrome {
        instance,
        interactive,
        accessible_label,
        description,
        show_data_table,
        state,
        ..
    } = chrome;
    let scale_token = match style.scale {
        HeatScale::Magnitude => "magnitude",
        HeatScale::Judgement => "judgement",
    };

    if grid.is_empty() {
        let empty = empty_view(style);
        return view! {
            <div data-testid="heatmap" data-heatmap-scale=scale_token role="group"
                aria-label=move || accessible_label.get() class="w-full">
                {empty}
            </div>
        }
        .into_any();
    }

    let frame = style.frame(grid.rows.len(), grid.columns.len());
    let viewbox = format!("0 0 {} {:.0}", style.width, frame.height_eff);
    let title_id = format!("heatmap-{instance}-title");
    let desc_id = format!("heatmap-{instance}-desc");
    let labelled_by = format!("{title_id} {desc_id}");
    let interactive = interactive && !grid.is_empty();

    let marks = render_marks(style, grid, frame);
    let row_labels: Vec<String> = grid.rows.iter().map(|row| row.label.clone()).collect();
    let col_labels: Vec<String> = grid
        .columns
        .iter()
        .map(|column| column.label.clone())
        .collect();
    let rows_view = row_label_views(&row_labels, frame, show_data_table);
    let cols_view = col_label_views(&col_labels, frame, style.slant_col_labels, show_data_table);
    let targets = interactive.then(|| focus_targets(style, grid, frame, chrome));
    let table = show_data_table.then(|| data_table(style, grid));
    let active_row = move || {
        state
            .read()
            .active_key()
            .map(|key| key.row.clone())
            .unwrap_or_default()
    };
    let active_column = move || {
        state
            .read()
            .active_key()
            .map(|key| key.column.clone())
            .unwrap_or_default()
    };

    view! {
        <div data-testid="heatmap" data-heatmap-scale=scale_token role="group"
            aria-label=move || accessible_label.get() data-active-row=active_row
            data-active-column=active_column class="w-full">
            <svg data-heatmap-plot role=svg_role(interactive) aria-labelledby=labelled_by
                viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
                <title id=title_id>{move || accessible_label.get()}</title>
                <desc id=desc_id>{move || description.get()}</desc>
                {marks}
                {rows_view}
                {cols_view}
                {targets}
            </svg>
            {table}
        </div>
    }
    .into_any()
}

/// One group per grid position: its tile when it has a measurement, its drawn
/// abbreviation when it has one, and the sense rule that keeps a judgement from
/// living in the hue alone.
fn render_marks(style: &RenderStyle, grid: &NormalizedHeatmap, frame: Frame) -> AnyView {
    let rgb = style.rgb.get_value();
    let favorable = style.favorable.get_value();
    let unfavorable = style.unfavorable.get_value();
    let palette = HeatPalette {
        scale: style.scale,
        rgb: &rgb,
        favorable: &favorable,
        unfavorable: &unfavorable,
    };
    let scale = style.scale;

    grid.iter()
        .map(|(row, column, cell)| {
            let (x, y, w, h) = cell_rect(row, column, frame.layout);
            let sense = cell.sense(scale);
            let tile = cell.intensity.map(|intensity| {
                // Exactly one of these is Some — see `paint_attrs` for why a
                // token-bearing colour cannot ride on the `fill` attribute.
                let (fill, fill_style) = paint_attrs(cell_fill(intensity, &palette));
                view! {
                    <rect x=format!("{x:.2}") y=format!("{y:.2}") width=format!("{w:.2}")
                        height=format!("{h:.2}") fill=fill style=fill_style />
                }
            });
            let label = cell.visible_text().map(|text| {
                let cx = format!("{:.2}", x + w / 2.0);
                let cy = format!("{:.2}", y + h / 2.0);
                view! {
                    <text x=cx y=cy text-anchor="middle" dominant-baseline="middle"
                        fill="currentColor" font-size="9" aria-hidden="true">
                        {text}
                    </text>
                }
            });
            // The judgement, drawn rather than only tinted: solid for
            // favorable, dashed for unfavorable, absent for no verdict. Same
            // convention as BarChart's status caps, and the half of the
            // judgement that survives forced colours.
            let rule = sense.dash().map(|dash| {
                let (stroke, stroke_style) = stroke_attrs("currentColor".to_string());
                let rule_y = format!("{:.2}", y + h - 4.0);
                view! {
                    <line data-heatmap-sense-rule=sense.token() x1=format!("{:.2}", x + w * 0.3)
                        y1=rule_y.clone() x2=format!("{:.2}", x + w * 0.7) y2=rule_y
                        stroke=stroke style=stroke_style stroke-width="2" stroke-dasharray=dash
                        stroke-linecap="butt" aria-hidden="true" />
                }
            });
            let missing = (!cell.is_measured()).then_some("");
            view! {
                <g data-heatmap-cell="" data-row-key=cell.row_key.clone()
                    data-column-key=cell.column_key.clone() data-heatmap-sense=sense.token()
                    data-heatmap-missing=missing>
                    {tile}
                    {label}
                    {rule}
                </g>
            }
        })
        .collect_view()
        .into_any()
}

/// One focusable, clickable target per grid position, spanning its whole cell.
///
/// Dense on purpose: a coordinate with no measurement is still a coordinate a
/// reader may want to drill into, and skipping it would also make an arrow key
/// jump an unpredictable distance.
fn focus_targets(
    style: &RenderStyle,
    grid: &NormalizedHeatmap,
    frame: Frame,
    chrome: TypedChrome,
) -> AnyView {
    let TypedChrome {
        instance,
        on_cell_activate,
        on_cell_click,
        state,
        ..
    } = chrome;
    let texts = style.texts;
    let scale = style.scale;
    let has_activation = on_cell_activate.is_some() || on_cell_click.is_some();
    let target_role = target_role(has_activation);
    let axes = StoredValue::new(Axes::new(grid.row_keys(), grid.column_keys()));
    let ids = StoredValue::new(
        grid.iter()
            .map(|(row, column, cell)| {
                (
                    CellKey::new(cell.row_key.clone(), cell.column_key.clone()),
                    target_id(instance, row, column),
                )
            })
            .collect::<Vec<_>>(),
    );
    let dispatch = move |action: Action| {
        let all = axes.get_value();
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

    grid.iter()
        .map(|(row, column, cell)| {
            let cell = cell.clone();
            let key = CellKey::new(cell.row_key.clone(), cell.column_key.clone());
            let id = target_id(instance, row, column);
            let (x, y, w, h) = cell_rect(row, column, frame.layout);
            let focused_key = key.clone();
            let is_focused = move || state.read().focused_key.as_ref() == Some(&focused_key);
            let roving_key = key.clone();
            let is_roving = move || state.read().roving_key.as_ref() == Some(&roving_key);
            let name_cell = cell.clone();
            let label = move || texts.with(|texts| accessible_name(&name_cell, scale, texts));
            let activate = move |source: HeatmapActivationSource, modifiers: HeatmapModifiers| {
                if let Some(callback) = on_cell_activate {
                    let payload = texts.with_untracked(|texts| {
                        activation_for(&cell, scale, texts, source, modifiers)
                    });
                    callback.run(payload);
                }
                // The documented migration path: a caller that has not yet
                // rewritten its positional handler still hears the click.
                if let Some(callback) = on_cell_click {
                    callback.run((row, column));
                }
            };
            let key_activate = activate.clone();
            let on_key = move |ev: web_sys::KeyboardEvent| {
                let nav = match ev.key().as_str() {
                    "ArrowLeft" => Some(Nav::PreviousColumn),
                    "ArrowRight" => Some(Nav::NextColumn),
                    "ArrowUp" => Some(Nav::PreviousRow),
                    "ArrowDown" => Some(Nav::NextRow),
                    "Home" if ev.ctrl_key() => Some(Nav::GridStart),
                    "End" if ev.ctrl_key() => Some(Nav::GridEnd),
                    "Home" => Some(Nav::RowStart),
                    "End" => Some(Nav::RowEnd),
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
                    // button behaviour, so a purely explorable grid never
                    // swallows a key the page itself wanted.
                    "Enter" | " " if has_activation => {
                        ev.prevent_default();
                        key_activate(
                            HeatmapActivationSource::Keyboard,
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
                    HeatmapActivationSource::Pointer,
                    modifiers_of(ev.shift_key(), ev.ctrl_key(), ev.alt_key(), ev.meta_key()),
                );
            };
            let focus_dispatch_key = key.clone();
            let hover_dispatch_key = key.clone();
            view! {
                <rect id=id data-heatmap-focus="" data-row-key=key.row.clone()
                    data-column-key=key.column.clone() x=format!("{x:.2}") y=format!("{y:.2}")
                    width=format!("{w:.2}") height=format!("{h:.2}") fill="transparent"
                    pointer-events="all" role=target_role rx="2" stroke="currentColor"
                    stroke-opacity="0.75"
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

/// The heatmap's non-visual truth, as a MATRIX rather than a list.
///
/// Row labels are `th[scope=row]` and column labels are `th[scope=col]`, which
/// is what lets a screen reader announce both headers when a reader lands on a
/// cell — so a value is located by "North, SLA met" rather than by counting
/// position in a flat stream. The corner cell names the row axis. Every
/// `(row, column)` combination has a cell, so a gap is heard as the localized
/// missing copy at its own coordinate instead of being silently skipped.
fn data_table(style: &RenderStyle, grid: &NormalizedHeatmap) -> AnyView {
    let texts = style.texts;
    let scale = style.scale;
    let headers = grid
        .columns
        .iter()
        .map(|column| {
            let label = column.label.clone();
            view! { <th scope="col" data-column-key=column.key.clone()>{label}</th> }
        })
        .collect_view();
    let rows = grid
        .rows
        .iter()
        .enumerate()
        .map(|(row_index, row)| {
            let label = row.label.clone();
            let cells = grid
                .columns
                .iter()
                .enumerate()
                .map(|(column_index, column)| {
                    let cell = grid
                        .cell(row_index, column_index)
                        .cloned()
                        .expect("the grid is dense");
                    let missing = (!cell.is_measured()).then_some("");
                    let sense = cell.sense(scale).token();
                    view! {
                        <td data-column-key=column.key.clone() data-heatmap-sense=sense
                            data-heatmap-missing=missing>
                            {move || texts.with(|texts| cell.stated_text(scale, texts))}
                        </td>
                    }
                })
                .collect_view();
            view! {
                <tr data-row-key=row.key.clone()>
                    <th scope="row">{label}</th>
                    {cells}
                </tr>
            }
        })
        .collect_view();

    view! {
        <table data-heatmap-table class="sr-only">
            <caption>{move || texts.with(|texts| texts.data_table_caption.clone())}</caption>
            <thead>
                <tr>
                    <th scope="col">{move || texts.with(|texts| texts.row_header.clone())}</th>
                    {headers}
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>
    }
    .into_any()
}

#[cfg(test)]
mod tests;
