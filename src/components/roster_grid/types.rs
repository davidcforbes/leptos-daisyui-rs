use super::style::ShiftState;

/// Default weekday column labels for a [`RosterGrid`](super::RosterGrid):
/// Monday to Friday.
///
/// These are a *default*, not a hardcoding. `RosterGrid` takes its column
/// labels from the caller — exactly as [`WeekView`](crate::components::WeekView)
/// takes its hour labels — so a Sunday-first week, a seven-day roster, a
/// fortnightly grid or a non-English locale is a matter of passing different
/// strings rather than patching the component.
pub const DEFAULT_ROSTER_COLUMNS: [&str; 5] = ["Mon", "Tue", "Wed", "Thu", "Fri"];

/// [`DEFAULT_ROSTER_COLUMNS`] as an owned `Vec`, for the `columns` prop's
/// default.
pub fn default_roster_columns() -> Vec<String> {
    DEFAULT_ROSTER_COLUMNS
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Default title for the empty state a [`RosterGrid`](super::RosterGrid)
/// renders when it has no rows or no columns. Override with the component's
/// `empty_title` prop in a localised app.
pub fn default_empty_title() -> String {
    "No roster".to_string()
}

/// One cell of the roster: what this worker is doing on this day.
///
/// `label` is the shift *value* the consumer wants on screen — `"09:00-17:00"`,
/// `"AM"`, `"Night"`, or `""` for a cell whose state says everything. `state`
/// is the semantic classification that drives the tile's colour, its
/// solid-vs-dashed accent bar, and the visually-hidden text a screen reader
/// announces.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RosterCell {
    /// Shift value shown inside the tile. May be empty.
    pub label: String,
    /// Semantic shift classification.
    pub state: ShiftState,
}

impl RosterCell {
    /// A cell with an explicit label and state.
    pub fn new(label: impl Into<String>, state: ShiftState) -> Self {
        Self {
            label: label.into(),
            state,
        }
    }

    /// An empty, not-rostered cell — what a short row is padded with.
    pub fn off() -> Self {
        Self::default()
    }
}

/// One worker's week: a name plus the cells for that worker, aligned to the
/// grid's columns **by index**.
///
/// The cell count is not required to match the column count. See
/// [`normalize_cells`] for what happens when it does not — the short/long
/// cases are handled there, once, rather than at every read site.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RosterRow {
    /// The worker's display name, rendered as the row's `<th scope="row">`.
    pub worker: String,
    /// Cells positionally aligned to the grid's columns.
    pub cells: Vec<RosterCell>,
}

impl RosterRow {
    /// A row for `worker` with the given cells.
    pub fn new(worker: impl Into<String>, cells: Vec<RosterCell>) -> Self {
        Self {
            worker: worker.into(),
            cells,
        }
    }
}

/// Align `cells` to exactly `n_cols` entries: pad a short row with
/// [`RosterCell::off`], truncate a long one.
///
/// This is the single most likely defect in a matrix control, so it is one
/// pure function with its own tests rather than an index guard sprinkled
/// through the view. Ragged data is not exceptional — it is what a partially
/// filled roster, a mid-edit form, or a backend that omits trailing nulls
/// actually sends. Both directions are silent and total: never a panic, never
/// a row that visually slips one column left of its header.
///
/// Padding with `Off` (rather than dropping the cell) is the honest reading:
/// a worker with no entry for Friday is not rostered on Friday.
pub fn normalize_cells(cells: &[RosterCell], n_cols: usize) -> Vec<RosterCell> {
    let mut out: Vec<RosterCell> = cells.iter().take(n_cols).cloned().collect();
    out.resize(n_cols, RosterCell::off());
    out
}

/// The accessible name of one shift tile: worker, column, the tile's own
/// label when it has one, and the state's name.
///
/// A screen-reader user hears "Ada Lovelace, Mon, 09:00-17:00, Full shift"
/// rather than a bare time floating in an unlabelled grid — the same shape as
/// [`event_aria_label`](crate::components::event_aria_label) in `DayScheduler`.
/// `state_name` is passed in rather than read from the state so a localised
/// consumer's `state_label` callback flows through unchanged.
pub fn cell_aria_label(worker: &str, column: &str, cell: &RosterCell, state_name: &str) -> String {
    if cell.label.is_empty() {
        format!("{worker}, {column}, {state_name}")
    } else {
        format!("{worker}, {column}, {}, {state_name}", cell.label)
    }
}

/// Whether a key press on a focused shift tile should activate it.
///
/// Enter and Space only, matching `DataTable`'s row keys and `DayScheduler`'s
/// event keys. Everything else — Tab above all — is left alone so focus
/// navigation keeps working.
pub fn cell_key_activates(key: &str) -> bool {
    key == "Enter" || key == " "
}

/// Whether the grid's tiles carry focus and keyboard semantics: a roving
/// `tabindex`, `role="button"`, `aria-pressed`, arrow-key navigation and
/// Enter/Space activation — and whether the table itself is announced as an
/// ARIA `grid` widget rather than a plain data table.
///
/// The named counterpart of `DataTable`'s
/// [`row_is_interactive`](crate::components::row_is_interactive), but with a
/// deliberately narrower rule, so the difference is visible and testable rather
/// than buried in a `let` inside the view.
///
/// **Only an activation callback counts.** `DataTable` and `DayScheduler` also
/// treat their selection prop as opting in, because theirs is an `RwSignal`
/// *they* write — a click on a selection-only grid there really does change
/// state. `RosterGrid`'s `selected_cell` is a read-only `Signal`, so with no
/// callback there is nothing for a click or an Enter press to do. Counting it
/// would put `role="button"` and `tabindex=0` on every tile of a display-only
/// roster — on a 20x7 grid, 140 focusable elements announced as buttons that
/// ignore every key. Advertising a role and a state without the behaviour is
/// exactly WCAG 4.1.2 (Name, Role, Value), and it would also make a
/// today's-shift highlight unusable to keyboard users.
///
/// The selection ring is drawn independently of this, so a display-only roster
/// keeps its highlight and loses only the dead tab stops.
pub fn grid_is_interactive(has_activate: bool, has_selection: bool) -> bool {
    let _ = has_selection;
    has_activate
}

/// A focus movement requested by a key press inside an interactive
/// [`RosterGrid`](super::RosterGrid).
///
/// The ARIA grid pattern's navigation vocabulary, kept as a value so the
/// key-to-intent decision and the coordinate arithmetic are two separately
/// testable steps rather than one `match` buried in an event handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RosterFocusMove {
    /// One column left.
    Left,
    /// One column right.
    Right,
    /// One row up.
    Up,
    /// One row down.
    Down,
    /// First column of the current row.
    RowStart,
    /// Last column of the current row.
    RowEnd,
    /// The grid's first cell.
    GridStart,
    /// The grid's last cell.
    GridEnd,
}

/// Map a key press on a focused shift tile to a focus movement, or `None` when
/// the key is not ours to handle.
///
/// Deliberately disjoint from [`cell_key_activates`]: Enter and Space activate,
/// the arrows and Home/End move, and everything else — Tab above all — is left
/// to the browser so the grid stays escapable. `Ctrl` promotes Home/End from
/// the row's extremes to the grid's, which is what the ARIA Data Grid pattern
/// specifies and what a 30-row roster actually needs.
pub fn roster_focus_move(key: &str, ctrl: bool) -> Option<RosterFocusMove> {
    match key {
        "ArrowLeft" => Some(RosterFocusMove::Left),
        "ArrowRight" => Some(RosterFocusMove::Right),
        "ArrowUp" => Some(RosterFocusMove::Up),
        "ArrowDown" => Some(RosterFocusMove::Down),
        "Home" if ctrl => Some(RosterFocusMove::GridStart),
        "End" if ctrl => Some(RosterFocusMove::GridEnd),
        "Home" => Some(RosterFocusMove::RowStart),
        "End" => Some(RosterFocusMove::RowEnd),
        _ => None,
    }
}

/// Bring a remembered focus coordinate back inside a grid that is now
/// `n_rows` x `n_cols`, or report that there is no grid to focus.
///
/// **This is the roving-focus equivalent of [`normalize_cells`] and the single
/// most likely defect in the feature.** `rows` and `columns` are `Signal`s: a
/// filter, a search, a page change or a fresh fetch can shrink the roster under
/// a focus position the component is still holding. A stored `(12, 5)` against
/// a grid that just became 3x3 must land on `(2, 2)` — never index out of
/// bounds, and never leave *no* cell carrying `tabindex=0`, which would silently
/// delete the grid's only tab stop and strand keyboard users outside it.
///
/// The component calls this on the **read** path (a derived signal), not from an
/// effect, so a clamped coordinate cannot be stale by an ordering accident. The
/// clamp is also non-destructive: the raw remembered position is kept, so a
/// transient shrink (a filter typed and then cleared) restores the user's place
/// rather than dumping them at the grid's edge. It is rewritten to the clamped
/// value on the next deliberate move.
///
/// Returns `None` for a zero-row or zero-column grid, which is exactly the case
/// the component renders as an [`EmptyState`](crate::components::EmptyState) —
/// there is no cell to focus, so there is no coordinate to invent.
pub fn clamp_focus_cell(
    focus: (usize, usize),
    n_rows: usize,
    n_cols: usize,
) -> Option<(usize, usize)> {
    if n_rows == 0 || n_cols == 0 {
        return None;
    }
    Some((focus.0.min(n_rows - 1), focus.1.min(n_cols - 1)))
}

/// Apply `movement` to `current` within an `n_rows` x `n_cols` grid, returning
/// the new focus coordinate.
///
/// `current` is clamped first, so a movement requested against a roster that
/// shrank between the last key press and this one still starts from a real
/// cell. Movement **stops at the edges rather than wrapping**: the ARIA Data
/// Grid pattern makes wrapping optional, and in a roster a wrap would silently
/// jump the user from Friday to Monday of a different worker — a plausible-
/// looking answer to a question they did not ask. `Ctrl+Home`/`Ctrl+End` are
/// the deliberate way to cross the whole grid.
///
/// Returns `None` only when there is no grid at all.
pub fn next_focus_cell(
    current: (usize, usize),
    n_rows: usize,
    n_cols: usize,
    movement: RosterFocusMove,
) -> Option<(usize, usize)> {
    let (row, col) = clamp_focus_cell(current, n_rows, n_cols)?;
    let last_row = n_rows - 1;
    let last_col = n_cols - 1;

    Some(match movement {
        RosterFocusMove::Left => (row, col.saturating_sub(1)),
        RosterFocusMove::Right => (row, (col + 1).min(last_col)),
        RosterFocusMove::Up => (row.saturating_sub(1), col),
        RosterFocusMove::Down => ((row + 1).min(last_row), col),
        RosterFocusMove::RowStart => (row, 0),
        RosterFocusMove::RowEnd => (row, last_col),
        RosterFocusMove::GridStart => (0, 0),
        RosterFocusMove::GridEnd => (last_row, last_col),
    })
}

/// The DOM id of one shift tile, used to move real focus to it.
///
/// Roving focus is not merely a signal changing colour: the browser's
/// `document.activeElement` has to actually move, or Tab still lands on the
/// tile the user left and a screen reader announces the wrong cell. `instance`
/// makes the ids collision-free when a page renders more than one roster —
/// the same reason [`Menu`](crate::components::Menu) mints per-instance item
/// ids for its `aria-activedescendant` wiring.
pub fn roster_cell_dom_id(instance: u64, row: usize, col: usize) -> String {
    format!("ld-roster-{instance}-cell-{row}-{col}")
}

/// Whether `(row, col)` is the selected cell.
///
/// Trivial by construction, and that is the point: selection is compared, never
/// indexed with, so a `selected_cell` pointing outside the grid (a stale
/// coordinate left over from a larger roster) simply matches nothing. There is
/// no bounds check to forget.
pub fn cell_is_selected(selected: Option<(usize, usize)>, row: usize, col: usize) -> bool {
    selected == Some((row, col))
}
