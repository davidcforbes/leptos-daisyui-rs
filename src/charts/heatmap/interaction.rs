//! Keyed hover/focus state for a TWO-AXIS grid, and the pure reducer that
//! moves it.
//!
//! Every input event — pointer, focus, arrow key, Home/End, Escape and a data
//! replacement — funnels through [`reduce`], which is a pure function of the
//! previous state and the two axes before and after. DOM side effects (moving
//! focus, invoking the host's callback) stay at the call sites, so all of the
//! behaviour the acceptance criteria describe is testable natively.
//!
//! **State is held by a PAIR of keys, never by a pair of indices.** An index
//! re-points at a different office the moment the caller sorts by worst-first,
//! and at a different KPI the moment a column is hidden; a key does not. That
//! is the same identity rule `ldui-nz6d` / `ldui-px06` established for tables,
//! `ldui-9tr` for the line chart's categories and `ldui-y2ed` for the bar
//! chart's.
//!
//! The keyboard model is the ARIA grid pattern, which is what a reader who has
//! met any other data grid already knows:
//!
//! | Key | Moves to |
//! |---|---|
//! | Left / Right | the previous / next COLUMN, same row |
//! | Up / Down | the previous / next ROW, same column |
//! | Home / End | the FIRST / LAST column of the current row |
//! | Ctrl+Home / Ctrl+End | the first / last cell of the whole GRID |
//!
//! Every move clamps rather than wrapping: silently jumping from the last
//! column back to the first hides the edge of the matrix from exactly the
//! reader who cannot see where it is.

/// The identity of one grid position: which row, and which column.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CellKey {
    pub row: String,
    pub column: String,
}

impl CellKey {
    pub(super) fn new(row: impl Into<String>, column: impl Into<String>) -> Self {
        Self {
            row: row.into(),
            column: column.into(),
        }
    }
}

/// The two axes as they currently stand. The reducer needs both before and
/// after a data change, which is what lets it follow a key through a reorder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Axes {
    pub rows: Vec<String>,
    pub columns: Vec<String>,
}

impl Axes {
    pub(super) fn new(rows: Vec<String>, columns: Vec<String>) -> Self {
        Self { rows, columns }
    }

    fn is_empty(&self) -> bool {
        self.rows.is_empty() || self.columns.is_empty()
    }

    fn contains(&self, key: &CellKey) -> bool {
        self.rows.contains(&key.row) && self.columns.contains(&key.column)
    }

    fn first(&self) -> Option<CellKey> {
        Some(CellKey::new(self.rows.first()?, self.columns.first()?))
    }

    /// The `(row, column)` indices of `key`, when both axes still carry it.
    fn position(&self, key: &CellKey) -> Option<(usize, usize)> {
        let row = self.rows.iter().position(|row| *row == key.row)?;
        let column = self
            .columns
            .iter()
            .position(|column| *column == key.column)?;
        Some((row, column))
    }

    fn at(&self, row: usize, column: usize) -> Option<CellKey> {
        Some(CellKey::new(self.rows.get(row)?, self.columns.get(column)?))
    }
}

/// Which cell the reader is hovering, has focused, and would tab to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct HeatmapInteraction {
    /// The single tab stop. Always a cell that exists, once [`reduce`] has
    /// seen any data.
    pub roving_key: Option<CellKey>,
    /// The focused cell, if focus is inside the grid.
    pub focused_key: Option<CellKey>,
    /// The hovered cell, if a pointer is over one.
    pub hovered_key: Option<CellKey>,
}

impl HeatmapInteraction {
    /// The cell a reader is currently attending to: the pointer wins over
    /// focus, matching every hover surface in this crate.
    pub(super) fn active_key(&self) -> Option<&CellKey> {
        self.hovered_key.as_ref().or(self.focused_key.as_ref())
    }
}

/// A keyboard navigation request, in two axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Nav {
    /// Left: the previous column of the current row.
    PreviousColumn,
    /// Right: the next column of the current row.
    NextColumn,
    /// Up: the previous row of the current column.
    PreviousRow,
    /// Down: the next row of the current column.
    NextRow,
    /// Home: the first column of the current row.
    RowStart,
    /// End: the last column of the current row.
    RowEnd,
    /// Ctrl+Home: the first cell of the grid.
    GridStart,
    /// Ctrl+End: the last cell of the grid.
    GridEnd,
}

/// Something that happened to the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Focused(CellKey),
    Blurred,
    Hovered(CellKey),
    HoverEnded,
    MoveFocus(Nav),
    /// Escape: give up the hover and the focus highlight without moving the
    /// tab stop, so a reader who dismisses is not sent back to cell one.
    Dismiss,
    /// The data changed. Keys are re-resolved rather than reset.
    ReconcileData,
}

/// Applies `action` to `state`.
///
/// `previous` and `next` are the axes before and after the change; they are the
/// same for every action but [`Action::ReconcileData`].
pub(super) fn reduce(
    state: &HeatmapInteraction,
    action: Action,
    previous: &Axes,
    next: &Axes,
) -> HeatmapInteraction {
    let mut out = state.clone();
    match action {
        Action::Focused(key) => {
            if next.contains(&key) {
                out.focused_key = Some(key.clone());
                out.roving_key = Some(key);
            }
        }
        Action::Blurred => out.focused_key = None,
        Action::Hovered(key) => {
            if next.contains(&key) {
                out.hovered_key = Some(key);
            }
        }
        Action::HoverEnded => out.hovered_key = None,
        Action::Dismiss => {
            out.hovered_key = None;
            out.focused_key = None;
        }
        Action::MoveFocus(nav) => {
            if let Some(key) = moved(state, nav, next) {
                out.focused_key = Some(key.clone());
                out.roving_key = Some(key);
            }
        }
        Action::ReconcileData => {
            out.roving_key = reconcile(state.roving_key.as_ref(), previous, next);
            out.focused_key = state
                .focused_key
                .as_ref()
                .and_then(|key| reconcile(Some(key), previous, next));
            // A pointer's position after a data replacement is not knowable, so
            // the hover is dropped rather than guessed at.
            out.hovered_key = None;
        }
    }
    // The tab stop must always land somewhere real: the first cell when the
    // grid has one, and nothing at all when it does not.
    if out
        .roving_key
        .as_ref()
        .is_none_or(|key| !next.contains(key))
    {
        out.roving_key = next.first();
    }
    out
}

/// Where `nav` sends focus, from wherever it currently is.
fn moved(state: &HeatmapInteraction, nav: Nav, axes: &Axes) -> Option<CellKey> {
    if axes.is_empty() {
        return None;
    }
    let last_row = axes.rows.len() - 1;
    let last_column = axes.columns.len() - 1;
    let (row, column) = state
        .focused_key
        .as_ref()
        .and_then(|key| axes.position(key))
        .or_else(|| state.roving_key.as_ref().and_then(|key| axes.position(key)))
        .unwrap_or((0, 0));
    let (row, column) = match nav {
        Nav::PreviousColumn => (row, column.saturating_sub(1)),
        Nav::NextColumn => (row, (column + 1).min(last_column)),
        Nav::PreviousRow => (row.saturating_sub(1), column),
        Nav::NextRow => ((row + 1).min(last_row), column),
        Nav::RowStart => (row, 0),
        Nav::RowEnd => (row, last_column),
        Nav::GridStart => (0, 0),
        Nav::GridEnd => (last_row, last_column),
    };
    axes.at(row, column)
}

/// Follows a cell key through a data change.
///
/// The two axes reconcile INDEPENDENTLY, which is the two-dimensional form of
/// the rule the bar chart established. A row that still exists keeps focus on
/// that row wherever it moved to, and likewise for the column — so sorting the
/// offices worst-first leaves a reader on the same office and the same KPI. An
/// axis entry that was removed hands its half of the position to whatever now
/// occupies its old index, clamped to the end of that axis, so focus moves
/// *predictably* instead of vanishing or jumping to the corner.
fn reconcile(key: Option<&CellKey>, previous: &Axes, next: &Axes) -> Option<CellKey> {
    let key = key?;
    if next.is_empty() {
        return None;
    }
    let row = follow(&key.row, &previous.rows, &next.rows)?;
    let column = follow(&key.column, &previous.columns, &next.columns)?;
    Some(CellKey::new(row, column))
}

/// Follows one axis entry through a change of that axis.
fn follow(key: &str, previous: &[String], next: &[String]) -> Option<String> {
    if next.iter().any(|candidate| candidate == key) {
        return Some(key.to_string());
    }
    let old_index = previous.iter().position(|candidate| candidate == key)?;
    next.get(old_index.min(next.len() - 1)).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axes(rows: &[&str], columns: &[&str]) -> Axes {
        Axes::new(
            rows.iter().map(|row| row.to_string()).collect(),
            columns.iter().map(|column| column.to_string()).collect(),
        )
    }

    fn office_kpi() -> Axes {
        axes(&["north", "south", "east"], &["closed", "sla", "handle"])
    }

    fn start(all: &Axes) -> HeatmapInteraction {
        reduce(
            &HeatmapInteraction::default(),
            Action::ReconcileData,
            all,
            all,
        )
    }

    fn focused(all: &Axes, row: &str, column: &str) -> HeatmapInteraction {
        reduce(
            &start(all),
            Action::Focused(CellKey::new(row, column)),
            all,
            all,
        )
    }

    fn go(state: &HeatmapInteraction, nav: Nav, all: &Axes) -> HeatmapInteraction {
        reduce(state, Action::MoveFocus(nav), all, all)
    }

    fn at(state: &HeatmapInteraction) -> Option<(String, String)> {
        state
            .focused_key
            .as_ref()
            .map(|key| (key.row.clone(), key.column.clone()))
    }

    #[test]
    fn the_tab_stop_starts_on_the_first_cell_of_the_grid() {
        let all = office_kpi();
        let state = start(&all);

        assert_eq!(state.roving_key, Some(CellKey::new("north", "closed")));
        assert_eq!(state.focused_key, None, "no focus is claimed on render");
        assert_eq!(state.hovered_key, None);
        assert_eq!(state.active_key(), None);
    }

    #[test]
    fn a_grid_with_an_empty_axis_has_no_tab_stop_at_all() {
        assert_eq!(start(&axes(&[], &["closed"])).roving_key, None);
        assert_eq!(start(&axes(&["north"], &[])).roving_key, None);
    }

    #[test]
    fn focus_and_hover_move_the_state_and_the_pointer_wins_for_active() {
        let all = office_kpi();
        let state = focused(&all, "north", "closed");
        assert_eq!(state.active_key(), Some(&CellKey::new("north", "closed")));

        let hovered = reduce(
            &state,
            Action::Hovered(CellKey::new("south", "sla")),
            &all,
            &all,
        );
        assert_eq!(
            hovered.active_key(),
            Some(&CellKey::new("south", "sla")),
            "a pointer over one cell wins over focus on another"
        );
        assert_eq!(
            hovered.focused_key,
            Some(CellKey::new("north", "closed")),
            "hovering must not steal focus"
        );

        let unhovered = reduce(&hovered, Action::HoverEnded, &all, &all);
        assert_eq!(
            unhovered.active_key(),
            Some(&CellKey::new("north", "closed"))
        );
    }

    #[test]
    fn a_key_that_is_not_on_the_grid_is_refused_rather_than_stored() {
        let all = office_kpi();
        let state = reduce(
            &start(&all),
            Action::Focused(CellKey::new("gone", "closed")),
            &all,
            &all,
        );
        assert_eq!(state.focused_key, None);

        let state = reduce(
            &start(&all),
            Action::Focused(CellKey::new("north", "gone")),
            &all,
            &all,
        );
        assert_eq!(state.focused_key, None, "both halves must exist");
    }

    // --- movement in two axes -------------------------------------------

    #[test]
    fn left_and_right_move_along_the_row() {
        let all = office_kpi();
        let state = go(&focused(&all, "south", "closed"), Nav::NextColumn, &all);
        assert_eq!(at(&state), Some(("south".into(), "sla".into())));

        let state = go(&state, Nav::PreviousColumn, &all);
        assert_eq!(at(&state), Some(("south".into(), "closed".into())));
    }

    #[test]
    fn up_and_down_move_along_the_column() {
        let all = office_kpi();
        let state = go(&focused(&all, "north", "sla"), Nav::NextRow, &all);
        assert_eq!(at(&state), Some(("south".into(), "sla".into())));

        let state = go(&state, Nav::PreviousRow, &all);
        assert_eq!(at(&state), Some(("north".into(), "sla".into())));
    }

    #[test]
    fn both_axes_clamp_at_their_edges_rather_than_wrapping() {
        // Wrapping would hide the edge of the matrix from the one reader who
        // cannot see where it is.
        let all = office_kpi();
        let corner = focused(&all, "north", "closed");
        assert_eq!(
            at(&go(&corner, Nav::PreviousColumn, &all)),
            Some(("north".into(), "closed".into()))
        );
        assert_eq!(
            at(&go(&corner, Nav::PreviousRow, &all)),
            Some(("north".into(), "closed".into()))
        );

        let far = focused(&all, "east", "handle");
        assert_eq!(
            at(&go(&far, Nav::NextColumn, &all)),
            Some(("east".into(), "handle".into()))
        );
        assert_eq!(
            at(&go(&far, Nav::NextRow, &all)),
            Some(("east".into(), "handle".into()))
        );
    }

    #[test]
    fn home_and_end_stay_inside_the_current_row() {
        // The ARIA grid pattern: Home/End are ROW-wise, which is what makes a
        // twelve-column KPI row traversable in two keystrokes without losing
        // the office the reader is on.
        let all = office_kpi();
        let state = go(&focused(&all, "south", "sla"), Nav::RowEnd, &all);
        assert_eq!(at(&state), Some(("south".into(), "handle".into())));

        let state = go(&state, Nav::RowStart, &all);
        assert_eq!(
            at(&state),
            Some(("south".into(), "closed".into())),
            "the row must not change"
        );
    }

    #[test]
    fn control_home_and_control_end_move_to_the_corners_of_the_grid() {
        let all = office_kpi();
        let state = go(&focused(&all, "south", "sla"), Nav::GridEnd, &all);
        assert_eq!(at(&state), Some(("east".into(), "handle".into())));

        let state = go(&state, Nav::GridStart, &all);
        assert_eq!(at(&state), Some(("north".into(), "closed".into())));
    }

    #[test]
    fn a_one_row_by_twelve_column_grid_traverses_the_whole_row() {
        // The consumer's exact shape (op-dlfua.7.35): one office, twelve KPIs.
        let columns: Vec<String> = (0..12).map(|index| format!("kpi-{index}")).collect();
        let all = Axes::new(vec!["north".to_string()], columns.clone());
        let mut state = focused(&all, "north", "kpi-0");

        for expected in columns.iter().skip(1) {
            state = go(&state, Nav::NextColumn, &all);
            assert_eq!(
                state.focused_key.as_ref().map(|key| key.column.as_str()),
                Some(expected.as_str())
            );
        }
        // Down from the only row stays put rather than falling off the grid.
        state = go(&state, Nav::NextRow, &all);
        assert_eq!(at(&state), Some(("north".into(), "kpi-11".into())));
        assert_eq!(
            at(&go(&state, Nav::RowStart, &all)),
            Some(("north".into(), "kpi-0".into()))
        );
    }

    #[test]
    fn navigating_before_anything_is_focused_starts_from_the_tab_stop() {
        let all = office_kpi();
        let state = go(&start(&all), Nav::NextColumn, &all);

        assert_eq!(at(&state), Some(("north".into(), "sla".into())));
    }

    #[test]
    fn escape_drops_the_highlight_without_moving_the_tab_stop() {
        let all = office_kpi();
        let state = focused(&all, "east", "handle");
        let dismissed = reduce(&state, Action::Dismiss, &all, &all);

        assert_eq!(dismissed.focused_key, None);
        assert_eq!(dismissed.hovered_key, None);
        assert_eq!(
            dismissed.roving_key,
            Some(CellKey::new("east", "handle")),
            "a reader who dismisses must not be sent back to the corner"
        );
    }

    // --- reconciliation across a data change ----------------------------

    #[test]
    fn focus_follows_a_cell_through_a_row_reorder() {
        // Sorting the offices worst-first is exactly what a consumer does
        // between renders.
        let before = office_kpi();
        let after = axes(&["east", "north", "south"], &["closed", "sla", "handle"]);
        let state = focused(&before, "south", "sla");

        let reconciled = reduce(&state, Action::ReconcileData, &before, &after);

        assert_eq!(reconciled.focused_key, Some(CellKey::new("south", "sla")));
        assert_eq!(reconciled.roving_key, Some(CellKey::new("south", "sla")));
    }

    #[test]
    fn focus_follows_a_cell_through_a_column_reorder() {
        let before = office_kpi();
        let after = axes(&["north", "south", "east"], &["handle", "closed", "sla"]);
        let state = focused(&before, "south", "handle");

        let reconciled = reduce(&state, Action::ReconcileData, &before, &after);

        assert_eq!(
            reconciled.focused_key,
            Some(CellKey::new("south", "handle"))
        );
    }

    #[test]
    fn removing_the_focused_row_moves_focus_predictably_and_keeps_the_column() {
        let before = office_kpi();
        let after = axes(&["north", "east"], &["closed", "sla", "handle"]);
        let state = focused(&before, "south", "handle");

        let reconciled = reduce(&state, Action::ReconcileData, &before, &after);

        assert_eq!(
            reconciled.focused_key,
            Some(CellKey::new("east", "handle")),
            "the row now at the removed one's position takes focus, same column"
        );
    }

    #[test]
    fn removing_the_focused_column_keeps_the_row() {
        let before = office_kpi();
        let after = axes(&["north", "south", "east"], &["closed", "handle"]);
        let state = focused(&before, "south", "sla");

        let reconciled = reduce(&state, Action::ReconcileData, &before, &after);

        assert_eq!(
            reconciled.focused_key,
            Some(CellKey::new("south", "handle"))
        );
    }

    #[test]
    fn removing_the_last_row_clamps_to_the_new_end() {
        let before = office_kpi();
        let after = axes(&["north", "south"], &["closed", "sla", "handle"]);
        let state = focused(&before, "east", "sla");

        let reconciled = reduce(&state, Action::ReconcileData, &before, &after);

        assert_eq!(reconciled.focused_key, Some(CellKey::new("south", "sla")));
    }

    #[test]
    fn emptying_the_grid_clears_every_key_rather_than_dangling() {
        let before = office_kpi();
        let state = focused(&before, "north", "closed");

        let reconciled = reduce(&state, Action::ReconcileData, &before, &axes(&[], &[]));

        assert_eq!(reconciled.focused_key, None);
        assert_eq!(reconciled.roving_key, None);
        assert_eq!(reconciled.hovered_key, None);
    }

    #[test]
    fn a_data_change_drops_a_hover_it_cannot_verify() {
        let all = office_kpi();
        let hovered = reduce(
            &start(&all),
            Action::Hovered(CellKey::new("north", "closed")),
            &all,
            &all,
        );

        let reconciled = reduce(&hovered, Action::ReconcileData, &all, &all);

        assert_eq!(
            reconciled.hovered_key, None,
            "where the pointer now sits is not knowable from the data alone"
        );
    }

    #[test]
    fn a_wholesale_replacement_still_leaves_a_reachable_tab_stop() {
        let before = office_kpi();
        let after = axes(&["x", "y"], &["p", "q"]);
        let state = focused(&before, "north", "closed");

        let reconciled = reduce(&state, Action::ReconcileData, &before, &after);

        assert_eq!(reconciled.focused_key, Some(CellKey::new("x", "p")));
        assert_eq!(reconciled.roving_key, Some(CellKey::new("x", "p")));
    }

    #[test]
    fn the_tab_stop_is_always_a_cell_that_exists() {
        // The invariant the render relies on when deciding which rect gets
        // tabindex=0: swept over every action against a changed grid.
        let before = office_kpi();
        let after = axes(&["south"], &["sla"]);
        let actions = [
            Action::Focused(CellKey::new("north", "closed")),
            Action::Hovered(CellKey::new("east", "handle")),
            Action::MoveFocus(Nav::GridEnd),
            Action::MoveFocus(Nav::RowEnd),
            Action::Dismiss,
            Action::Blurred,
        ];
        for action in actions {
            let state = reduce(&start(&before), action, &before, &before);
            let reconciled = reduce(&state, Action::ReconcileData, &before, &after);
            let roving = reconciled.roving_key.expect("a nonempty grid has a stop");
            assert!(after.contains(&roving), "{roving:?} is not on {after:?}");
            if let Some(focused) = reconciled.focused_key {
                assert!(after.contains(&focused), "{focused:?} is not on {after:?}");
            }
        }
    }
}
