use super::style::{RosterDensity, ShiftState};
use super::types::{
    RosterRow, cell_aria_label, cell_is_selected, cell_key_activates, cell_title, clamp_focus_cell,
    default_empty_title, default_roster_columns, grid_is_interactive, next_focus_cell,
    normalize_cells, roster_cell_dom_id, roster_focus_move, roster_table_aria_label,
};
use crate::components::data_table::TABLE_SCROLL_WRAPPER_CLASS;
use crate::components::empty_state::EmptyState;
use crate::components::gantt::utils::focus_element_by_id;
use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use std::sync::atomic::{AtomicU64, Ordering};

/// Per-instance sequence so each `RosterGrid` on a page mints collision-free
/// cell DOM ids for the roving-focus machinery, the same device
/// [`Menu`](crate::components::Menu) uses for `aria-activedescendant`.
static ROSTER_GRID_SEQ: AtomicU64 = AtomicU64::new(0);

/// # Roster Grid Component
///
/// A staffing roster: worker rows by weekday columns, where every cell carries
/// a shift *value* (`"09:00-17:00"`, `"AM"`) plus a
/// [`ShiftState`](super::ShiftState) classification. This is the orthogonal
/// problem to [`WeekView`](crate::components::WeekView) and
/// [`DayScheduler`](crate::components::DayScheduler), which are day-columns by
/// hour-rows with time-positioned blocks; a roster has no time axis, so a
/// scheduler cannot express it.
///
/// Four properties are structural rather than decorative:
///
/// 1. **It is a real `<table>`.** Weekday headers are `<th scope="col">` and
///    worker names are `<th scope="row">`, so a screen reader announces which
///    worker and which day a cell belongs to. Tabular data built from `div`s
///    loses that association entirely.
/// 2. **Colour is never the only channel.** Each tile renders its own label
///    text, carries visually-hidden state text for assistive tech, and gets a
///    solid accent bar when working versus a dashed one when not — so the
///    working/off split survives greyscale and colour blindness.
/// 3. **Ragged input is normal input.** A row with fewer cells than there are
///    columns pads with `Off`; a row with more truncates. The rule lives in
///    [`normalize_cells`](super::normalize_cells) and is unit-tested, because
///    a misaligned roster row is a silently wrong answer rather than a crash.
/// 4. **It is locale-free.** Column labels come from the caller (defaulting to
///    Mon-Fri), and `state_label` overrides the English state names, the same
///    escape hatch `hour_label` gives the scheduler components.
/// 5. **An interactive roster is ONE tab stop, not one per cell.** A grid is
///    two-dimensional, so a tile-per-tab-stop scheme costs `rows x columns`
///    presses to cross — 210 on a 30-worker week. The component implements the
///    ARIA Data Grid roving tabindex instead: the focused tile carries
///    `tabindex=0`, every other tile `tabindex=-1`, and the arrows move real
///    DOM focus between them. See the keyboard table below.
///
/// ### Keyboard (interactive rosters only)
///
/// | Key | Effect |
/// |---|---|
/// | `Tab` | Enter the grid at the focused cell, or leave it entirely |
/// | Arrow keys | Move one cell, stopping at the edges rather than wrapping |
/// | `Home` / `End` | First / last column of the current row |
/// | `Ctrl+Home` / `Ctrl+End` | First / last cell of the whole grid |
/// | `Enter` / `Space` | Activate the focused cell |
///
/// The focused coordinate is internal state, and `rows`/`columns` are
/// `Signal`s that shrink: [`clamp_focus_cell`](super::clamp_focus_cell) brings
/// it back in range on every read, so a filter that empties the roster can
/// never leave the grid without a tab stop or index out of bounds.
///
/// Name the table with `labelled_by` (the id of the visible heading above it)
/// or `label`. Spread attributes land on the root `<div>`, so those props are
/// the only way to reach the `<table>` itself — and two rosters on one page are
/// otherwise indistinguishable to a screen reader.
///
/// Empty `rows` or empty `columns` renders an
/// [`EmptyState`](crate::components::EmptyState) rather than a zero-column
/// table.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{RosterCell, RosterGrid, RosterRow, ShiftState};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let rows = Signal::derive(|| {
///         vec![
///             RosterRow::new(
///                 "Ada Lovelace",
///                 vec![
///                     RosterCell::new("09:00-17:00", ShiftState::Full),
///                     RosterCell::new("09:00-13:00", ShiftState::Half),
///                     RosterCell::off(),
///                 ],
///             ),
///             // Short row: the missing columns pad with `Off`.
///             RosterRow::new("Grace Hopper", vec![RosterCell::new("Nights", ShiftState::Full)]),
///         ]
///     });
///
///     view! { <RosterGrid rows=rows /> }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("w-full overflow-x-auto table table-sm table-md whitespace-nowrap font-medium");
/// @source inline("p-1 align-middle text-center flex items-center justify-center overflow-hidden");
/// @source inline("rounded-sm border-l-4 border-solid border-dashed px-2 min-w-0 truncate sr-only");
/// @source inline("h-8 text-xs h-10 text-sm");
/// @source inline("bg-success/15 bg-info/15 bg-base-200/60 bg-accent/15 bg-warning/15");
/// @source inline("border-success border-info border-base-300 border-accent border-warning");
/// @source inline("ring-2 ring-primary cursor-pointer");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn RosterGrid(
    /// One row per worker. Each row's cells align to `columns` by index; see
    /// [`normalize_cells`](super::normalize_cells) for the ragged cases.
    #[prop(optional, into)]
    rows: Signal<Vec<RosterRow>>,

    /// Column headers, left to right — weekday names in the consumer's own
    /// locale. Defaults to
    /// [`DEFAULT_ROSTER_COLUMNS`](super::DEFAULT_ROSTER_COLUMNS) (Mon-Fri).
    /// Supplying seven labels gives a full week; supplying dates gives a
    /// fortnight. The component reads only the count and the text.
    #[prop(optional, into, default = Signal::derive(default_roster_columns))]
    columns: Signal<Vec<String>>,

    /// Row height and text size. See [`RosterDensity`](super::RosterDensity).
    #[prop(optional, into)]
    density: Signal<RosterDensity>,

    /// Header for the worker-name column (the grid's top-left corner cell).
    /// Defaults to "Worker"; pass a localised string to override.
    #[prop(optional, into, default = Signal::derive(|| "Worker".to_string()))]
    worker_header: Signal<String>,

    /// Optional override for the state names announced to assistive tech,
    /// replacing [`ShiftState::as_label`](super::ShiftState::as_label). This is
    /// the localisation hook: an app with its own i18n catalogue maps the
    /// state to its own wording, exactly as `hour_label` does for the
    /// scheduler components.
    #[prop(optional, into)]
    state_label: Option<Callback<ShiftState, String>>,

    /// Optional cell activation, called with `(row, col)` — the index into
    /// `rows` and into `columns`. Supplying this — and ONLY this — turns the
    /// grid into a keyboard widget: `role="grid"` on the table, `role="button"`
    /// tiles with an accessible name of "worker, column, label, state", and a
    /// roving tabindex giving the whole roster a single tab stop with arrow-key
    /// navigation inside it. A display-only roster gains no tab stops at all
    /// and stays a plain semantic table.
    ///
    /// **An interactive roster should carry `label` or `labelled_by`.** ARIA
    /// 1.2 marks `grid` as name-required, and a grid is announced on entry in a
    /// way a plain `table` largely is not — so two unnamed interactive rosters
    /// on one page are both read as "grid, 13 rows, 8 columns", with nothing to
    /// tell them apart.
    #[prop(optional, into)]
    on_cell_activate: Option<Callback<(usize, usize)>>,

    /// Optional controlled selection: the `(row, col)` currently selected.
    /// The consumer owns it — update it from `on_cell_activate`. A coordinate
    /// outside the grid selects nothing rather than panicking, so a stale
    /// selection left over from a larger roster is harmless.
    ///
    /// This prop alone does NOT make the grid interactive: it is a read-only
    /// `Signal`, so passing it without `on_cell_activate` is a legitimate
    /// display-only highlight (today's shift, a search hit) and the tiles stay
    /// plain content. The selection ring renders either way.
    #[prop(optional, into)]
    selected_cell: Option<Signal<Option<(usize, usize)>>>,

    /// Accessible name for the roster `<table>` (`aria-label`) — its
    /// (translated) subject, e.g. "Ward B, week of 12 May". Spread attributes
    /// land on the component's root `<div>`, so this is the only way to name
    /// the table itself; without it two rosters on one page are announced
    /// identically.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Id of the element that names the table (`aria-labelledby`) — typically
    /// the visible heading above it. Takes precedence over `label`, so
    /// assistive technology hears exactly what sighted users read, in whatever
    /// language the page is rendering. Follows [`Modal`](crate::components::Modal).
    #[prop(optional, into)]
    labelled_by: MaybeProp<String>,

    /// Id of the element that describes the table (`aria-describedby`) — e.g.
    /// the paragraph explaining what the shift codes mean.
    #[prop(optional, into)]
    described_by: MaybeProp<String>,

    /// Title shown when there is nothing to render (no rows, or no columns).
    #[prop(optional, into, default = Signal::derive(default_empty_title))]
    empty_title: Signal<String>,

    /// Subtitle shown beneath `empty_title` in the empty state.
    #[prop(optional, into)]
    empty_subtitle: Signal<String>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    // Focus and keyboard semantics exist only where activating a tile actually
    // does something. See `grid_is_interactive` for why `selected_cell` alone
    // deliberately does NOT count, unlike in DataTable/DayScheduler.
    let interactive = grid_is_interactive(on_cell_activate.is_some(), selected_cell.is_some());

    // ---- Roving tabindex (ARIA Data Grid) -------------------------------
    //
    // A roster is two-dimensional, so `tabindex=0` per tile costs `rows x
    // columns` Tab presses to cross -- 210 on a 30x7 grid. The whole grid is
    // therefore ONE tab stop: the focused cell carries `tabindex=0` and every
    // other cell `tabindex=-1`, with the arrows moving focus inside.
    let instance = ROSTER_GRID_SEQ.fetch_add(1, Ordering::Relaxed);

    // The remembered position, which may be out of range after the data
    // shrinks; `focused` clamps it on the READ path so a stale coordinate can
    // never leave the grid with no `tabindex=0` at all. See `clamp_focus_cell`.
    let focus_raw = RwSignal::new((0usize, 0usize));
    // A `Memo`, not a `Signal::derive`: every tile's `tabindex` closure reads
    // this, so a derived signal would re-run the clamp -- and re-subscribe to
    // `rows` and `columns` -- once per tile per arrow press (84 times on the
    // demo's department roster). The memo computes once and only notifies when
    // the clamped coordinate actually changes.
    let focused = Memo::new(move |_| {
        clamp_focus_cell(focus_raw.get(), rows.with(Vec::len), columns.with(Vec::len))
    });

    // Moving focus is not just a signal write: `document.activeElement` has to
    // move too, or Tab leaves from the old tile and the screen reader reads the
    // wrong cell. Only ever called from a key press, so the component never
    // steals focus on mount or when the data changes.
    let move_focus = move |movement| {
        let n_rows = rows.with_untracked(Vec::len);
        let n_cols = columns.with_untracked(Vec::len);
        let current = focus_raw.get_untracked();
        let Some(next) = next_focus_cell(current, n_rows, n_cols, movement) else {
            return;
        };
        // A movement that changes nothing (ArrowRight at the last column) must
        // NOT write: the raw coordinate may be a larger, still-restorable
        // position that a transient shrink clamped, and overwriting it with the
        // clamp would quietly discard the user's place. The tile already holds
        // DOM focus -- the key press came from it -- so there is nothing to do.
        if Some(next) == clamp_focus_cell(current, n_rows, n_cols) {
            return;
        }
        focus_raw.set(next);
        focus_element_by_id(&roster_cell_dom_id(instance, next.0, next.1));
    };

    // The state's announced name, honouring `state_label` when supplied.
    // `Callback` is `Copy`, so this closure is `Copy` and can be handed to
    // every per-cell closure independently.
    let state_name = move |state: ShiftState| match state_label {
        Some(cb) => cb.run(state),
        None => state.as_label().to_string(),
    };

    // Requirement 4: a roster with no workers, or no days, is an empty state —
    // never a zero-column table (which renders as a stray header rule) and
    // never a header with no body.
    let has_data = move || !rows.with(Vec::is_empty) && !columns.with(Vec::is_empty);

    view! {
        <div
            node_ref=node_ref
            // The horizontal-overflow contract is shared with both DataTable
            // variants rather than respelled here: a seven-day roster of long
            // shift codes overflows exactly as a wide table does, and one
            // definition means one place to change it.
            class=move || merge_classes!("w-full", TABLE_SCROLL_WRAPPER_CLASS, class)
        >
            <Show
                when=has_data
                fallback=move || {
                    view! { <EmptyState title=empty_title subtitle=empty_subtitle /> }
                }
            >
                // `role="grid"` ONLY when interactive. A grid is a *widget*
                // role: it promises one tab stop and arrow navigation, and a
                // screen reader switches out of browse mode for it -- which the
                // roving focus actually needs, because in browse mode the AT
                // eats the arrow keys and this handler never sees them. On a
                // display-only roster that promise would be a lie (the same
                // WCAG 4.1.2 argument as `grid_is_interactive`), so that path
                // keeps native table semantics. Either way the `<table>`,
                // `<th scope=col>` and `<th scope=row>` markup is unchanged, so
                // the header association survives in both.
                <table
                    role=interactive.then_some("grid")
                    aria-label=move || {
                        roster_table_aria_label(label.get(), labelled_by.get().is_some())
                    }
                    aria-labelledby=move || labelled_by.get()
                    aria-describedby=move || described_by.get()
                    class=move || {
                        merge_classes!("table w-full", density.get().as_table_class())
                    }
                >
                    <thead>
                        <tr>
                            <th scope="col">{move || worker_header.get()}</th>
                            {move || {
                                columns
                                    .get()
                                    .into_iter()
                                    .map(|heading| {
                                        view! { <th scope="col" class="text-center">{heading}</th> }
                                    })
                                    .collect_view()
                            }}
                        </tr>
                    </thead>

                    // The row structure is rebuilt when the data or the columns
                    // change; selection and density are read inside each tile's
                    // own closures instead, so selecting a cell re-styles it
                    // without replacing the DOM node under the user's focus.
                    <tbody>
                        {move || {
                            let cols = columns.get();
                            let n_cols = cols.len();
                            rows.get()
                                .into_iter()
                                .enumerate()
                                .map(|(ri, row)| {
                                    let worker = row.worker.clone();
                                    let cells = normalize_cells(&row.cells, n_cols);
                                    view! {
                                        <tr>
                                            <th scope="row" class="whitespace-nowrap font-medium">
                                                {worker.clone()}
                                            </th>
                                            {cells
                                                .into_iter()
                                                .enumerate()
                                                .map(|(ci, cell)| {
                                                    let col_label = cols
                                                        .get(ci)
                                                        .cloned()
                                                        .unwrap_or_default();
                                                    let announced = state_name(cell.state);
                                                    let aria = cell_aria_label(
                                                        &worker,
                                                        &col_label,
                                                        &cell,
                                                        &announced,
                                                    );
                                                    // Only the shift value, not
                                                    // the accessible name: see
                                                    // `cell_title`.
                                                    let tooltip = cell_title(&cell);
                                                    let state = cell.state;
                                                    let shift_value = cell.label.clone();
                                                    let is_selected = move || {
                                                        selected_cell
                                                            .is_some_and(|s| {
                                                                cell_is_selected(s.get(), ri, ci)
                                                            })
                                                    };
                                                    let activate = move || {
                                                        if let Some(cb) = on_cell_activate {
                                                            cb.run((ri, ci));
                                                        }
                                                    };
                                                    // Selection is the GRID's state, so it belongs
                                                    // on the gridcell as `aria-selected` -- ARIA's
                                                    // own selection mechanism, and the repo's
                                                    // precedent (`DataTable` puts it on rows).
                                                    // `aria-pressed` on the inner button described
                                                    // a toggle this never was: activating a tile
                                                    // runs a consumer callback, and whether
                                                    // selection follows is the consumer's choice.
                                                    let aria_selected = move || {
                                                        (interactive && selected_cell.is_some())
                                                            .then(|| {
                                                                if is_selected() { "true" } else { "false" }
                                                            })
                                                    };
                                                    view! {
                                                        <td
                                                            role=interactive
                                                                .then_some("gridcell")
                                                            aria-selected=aria_selected
                                                            class="p-1 align-middle"
                                                        >
                                                            <div
                                                                id=interactive
                                                                    .then(|| {
                                                                        roster_cell_dom_id(instance, ri, ci)
                                                                    })
                                                                class=move || {
                                                                    merge_classes!(
                                                                        "flex items-center justify-center overflow-hidden rounded-sm border-l-4 px-2 text-center",
                                                                        density.get().as_cell_class(),
                                                                        state.as_class(),
                                                                        state.as_border_class(),
                                                                        if interactive { "cursor-pointer" } else { "" }
                                                                    )
                                                                        .to_class()
                                                                }
                                                                class:ring-2=is_selected
                                                                class:ring-primary=is_selected
                                                                role=interactive.then_some("button")
                                                                // The roving part: exactly one tile
                                                                // is reachable by Tab; the rest are
                                                                // programmatically focusable only.
                                                                tabindex=move || {
                                                                    interactive
                                                                        .then(|| {
                                                                            if focused.get() == Some((ri, ci)) { 0 } else { -1 }
                                                                        })
                                                                }
                                                                title=tooltip
                                                                aria-label=interactive.then_some(aria)
                                                                on:click=move |_| {
                                                                    if interactive {
                                                                        // A click is also a focus
                                                                        // move: the tab stop must
                                                                        // follow the user's actual
                                                                        // position.
                                                                        focus_raw.set((ri, ci));
                                                                        activate();
                                                                    }
                                                                }
                                                                on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                                    if !interactive {
                                                                        return;
                                                                    }
                                                                    let key = ev.key();
                                                                    if cell_key_activates(&key) {
                                                                        ev.prevent_default();
                                                                        activate();
                                                                    } else if let Some(movement) = roster_focus_move(
                                                                        &key,
                                                                        ev.ctrl_key(),
                                                                        ev.alt_key(),
                                                                        ev.meta_key(),
                                                                    ) {
                                                                        // Reached only for a chord
                                                                        // the grid owns, so
                                                                        // Alt+Arrow (Back/Forward)
                                                                        // still reaches the
                                                                        // browser. Bare arrows are
                                                                        // stopped here or they
                                                                        // scroll the page out from
                                                                        // under the grid.
                                                                        ev.prevent_default();
                                                                        move_focus(movement);
                                                                    }
                                                                }
                                                            >
                                                                // `min-w-0` is load-bearing: a flex item
                                                                // defaults to `min-width: auto`, which
                                                                // refuses to shrink below its content, so
                                                                // `truncate` never ellipsises and the
                                                                // tile's `overflow-hidden` hard-clips the
                                                                // text mid-glyph instead.
                                                                <span class="min-w-0 truncate">{shift_value}</span>
                                                                // The state reaches assistive tech even
                                                                // when the tile is not an interactive
                                                                // widget with an `aria-label`.
                                                                <span class="sr-only">{announced}</span>
                                                            </div>
                                                        </td>
                                                    }
                                                })
                                                .collect_view()}
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }}
                    </tbody>
                </table>
            </Show>
        </div>
    }
}
