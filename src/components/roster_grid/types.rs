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

/// Whether `(row, col)` is the selected cell.
///
/// Trivial by construction, and that is the point: selection is compared, never
/// indexed with, so a `selected_cell` pointing outside the grid (a stale
/// coordinate left over from a larger roster) simply matches nothing. There is
/// no bounds check to forget.
pub fn cell_is_selected(selected: Option<(usize, usize)>, row: usize, col: usize) -> bool {
    selected == Some((row, col))
}
