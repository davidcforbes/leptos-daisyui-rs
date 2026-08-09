use super::style::{RosterDensity, ShiftState};
use super::types::{
    RosterRow, cell_aria_label, cell_is_selected, cell_key_activates, default_empty_title,
    default_roster_columns, normalize_cells,
};
use crate::components::empty_state::EmptyState;
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

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
/// @source inline("rounded-sm border-l-4 border-solid border-dashed px-2 truncate sr-only");
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
    /// `rows` and into `columns`. Supplying this (or `selected_cell`) makes
    /// tiles focusable (`tabindex=0`, `role="button"`) with an accessible name
    /// of "worker, column, label, state"; a display-only roster gains no tab
    /// stops at all.
    #[prop(optional, into)]
    on_cell_activate: Option<Callback<(usize, usize)>>,

    /// Optional controlled selection: the `(row, col)` currently selected.
    /// The consumer owns it — update it from `on_cell_activate`. A coordinate
    /// outside the grid selects nothing rather than panicking, so a stale
    /// selection left over from a larger roster is harmless.
    #[prop(optional, into)]
    selected_cell: Option<Signal<Option<(usize, usize)>>>,

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
    // Focus and keyboard semantics exist only where the consumer opted into an
    // interaction, mirroring DataTable's `row_is_interactive` rule.
    let interactive = on_cell_activate.is_some() || selected_cell.is_some();

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
            class=move || merge_classes!("w-full overflow-x-auto", class)
        >
            <Show
                when=has_data
                fallback=move || {
                    view! { <EmptyState title=empty_title subtitle=empty_subtitle /> }
                }
            >
                <table class=move || {
                    merge_classes!("table w-full", density.get().as_table_class())
                }>
                    <thead>
                        <tr>
                            <th scope="col">{move || worker_header.get()}</th>
                            {move || {
                                columns
                                    .get()
                                    .into_iter()
                                    .map(|label| {
                                        view! { <th scope="col" class="text-center">{label}</th> }
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
                                                    let aria_attr = aria.clone();
                                                    let state = cell.state;
                                                    let label = cell.label.clone();
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
                                                    view! {
                                                        <td class="p-1 align-middle">
                                                            <div
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
                                                                tabindex=interactive.then_some(0)
                                                                title=aria
                                                                aria-label=interactive.then_some(aria_attr)
                                                                aria-pressed=move || {
                                                                    (interactive && selected_cell.is_some())
                                                                        .then(|| {
                                                                            if is_selected() { "true" } else { "false" }
                                                                        })
                                                                }
                                                                on:click=move |_| {
                                                                    if interactive {
                                                                        activate();
                                                                    }
                                                                }
                                                                on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                                    if interactive && cell_key_activates(&ev.key()) {
                                                                        ev.prevent_default();
                                                                        activate();
                                                                    }
                                                                }
                                                            >
                                                                <span class="truncate">{label}</span>
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
