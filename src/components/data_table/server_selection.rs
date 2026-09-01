//! Controlled checkbox multi-selection for
//! [`ServerDataTable`](super::ServerDataTable) (`ldui-px06`).
//!
//! # A server table only ever holds one page
//!
//! That single fact shapes every type here, exactly as it shaped
//! [`ServerTableDisplayedSlice`](super::ServerTableDisplayedSlice) for export
//! (`ldui-9j16`). The export hazard was a CSV that silently contains one page
//! while claiming to be the whole filtered result set. Selection carries the
//! same hazard in reverse: a "select all" affordance that appears to select
//! rows the client has never seen, followed by a bulk mutation applied to
//! them.
//!
//! So the header checkbox here is **not** "select all". It is
//! *select-the-current-displayed-slice*, and that is true of its behaviour,
//! its state machine ([`ServerTableSliceSelectionState`]), its default copy,
//! and its emitted [`ServerTableSelectionCause::CurrentSlice`] payload — which
//! carries the exact keys the gesture covered. No code path can widen it,
//! because nothing in this module can name a row the caller did not render.
//!
//! # Accepted truth is caller-owned
//!
//! The component never holds selection state. The caller supplies the accepted
//! set of stable keys as a `Signal`, and every user gesture emits one
//! [`ServerTableSelectionProposal`] — a complete replacement set, not a delta
//! to apply. A rejected or delayed proposal leaves the rendered checkboxes,
//! `aria-selected`, and row styling aligned with the accepted signal, because
//! nothing optimistic was written.
//!
//! # Keys outside the current page
//!
//! Selection is keyed by the stable business key `row_key` already requires,
//! never by page position, so accepted keys for rows that are not on the
//! current page are simply carried through untouched by every proposal
//! ([`propose_row_toggle`], [`propose_slice_toggle`]). They survive paging by
//! construction rather than by a preservation step that could be forgotten.
//! [`off_slice_selected_count`] reports how many there are so the UI can say
//! so out loud.

use crate::components::data_table::server_component::SELECTION_WITHOUT_ROW_KEY_CONFIGURATION;
use crate::components::data_table::types::TableRow;
use leptos::prelude::*;
use std::collections::BTreeSet;

/// Combining the single-row and multi-row selection models is rejected, not
/// resolved: silently honouring one of them would make a bulk-assignment
/// workflow act on a single row, or a single-row workflow act on a set.
pub(crate) const CONFLICTING_SELECTION_MODES_CONFIGURATION: &str =
    "ServerDataTable accepts either selection or multi_selection, not both";

/// Multi-selection is keyed by stable business identity, so it structurally
/// requires `row_key`.
pub(crate) const MULTI_SELECTION_WITHOUT_ROW_KEY_CONFIGURATION: &str =
    "ServerDataTable controlled multi-selection requires row_key";

/// Whether one displayed row may participate in selection.
///
/// A blocked row still renders a checkbox so its state is legible, but the
/// checkbox is `aria-disabled` (focusable, so keyboard users reach the
/// reason) and its gesture emits nothing.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum ServerTableRowSelectability {
    /// The row participates in row and header-slice selection.
    #[default]
    Selectable,
    /// The row cannot be selected, for the caller-supplied reason.
    Blocked {
        /// Human-readable, localized explanation shown to pointer users as a
        /// tooltip and folded into the checkbox's accessible name.
        reason: String,
    },
}

impl ServerTableRowSelectability {
    /// Marks a row unselectable with a caller-supplied localized reason.
    pub fn blocked(reason: impl Into<String>) -> Self {
        Self::Blocked {
            reason: reason.into(),
        }
    }

    /// Whether this row participates in selection.
    pub fn is_selectable(&self) -> bool {
        matches!(self, Self::Selectable)
    }

    /// The caller-supplied reason, when the row is blocked.
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Selectable => None,
            Self::Blocked { reason } => Some(reason.as_str()),
        }
    }
}

/// What the user did to produce a [`ServerTableSelectionProposal`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerTableSelectionCause {
    /// One row's checkbox was toggled.
    Row {
        /// The stable key of the toggled row.
        key: String,
        /// `true` when the gesture asks for the key to become accepted.
        selected: bool,
    },
    /// The header checkbox toggled the current displayed slice.
    ///
    /// Named `CurrentSlice`, never `All`: it can only ever cover the
    /// selectable rows the caller currently rendered, which `keys` states
    /// exactly.
    CurrentSlice {
        /// `true` when the gesture asks for the slice to become accepted.
        selected: bool,
        /// The selectable displayed keys the gesture covered, in row order.
        keys: Vec<String>,
    },
}

/// One user-proposed replacement for the caller's accepted selected-key set.
///
/// `keys` is the COMPLETE proposed set, not a delta: apply it wholesale or
/// reject it wholesale. Nothing is applied until the caller's own signal
/// changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerTableSelectionProposal {
    /// The complete proposed accepted set, including keys that are not on the
    /// current page.
    pub keys: BTreeSet<String>,
    /// The gesture that produced this proposal.
    pub cause: ServerTableSelectionCause,
    /// The dataset/scope identity the proposal was computed against, read at
    /// gesture time.
    ///
    /// A caller that changes datasets (a new cursor stream, a different
    /// tenant, a re-scoped query) compares this against its current identity
    /// and rejects a proposal minted against the previous one — which is what
    /// makes "a new page/query/dataset must not silently relabel accepted
    /// keys" checkable rather than merely intended. Empty when the caller
    /// declared no scope.
    pub scope: String,
}

/// Header-checkbox state, computed over the CURRENT DISPLAYED SLICE only.
///
/// Accepted keys that are not on the current page never move the header off
/// [`None`](ServerTableSliceSelectionState::None) and never force
/// [`Partial`](ServerTableSliceSelectionState::Partial). That is deliberate:
/// the header checkbox answers "are the rows in front of me selected?", and
/// letting unseen rows tint it would be the component claiming knowledge of a
/// population it does not hold. The off-slice count is surfaced separately as
/// its own explicit line of copy (see [`off_slice_selected_count`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerTableSliceSelectionState {
    /// The displayed slice has no selectable rows at all — the header
    /// checkbox is unchecked and disabled.
    NoSelectableRows,
    /// No selectable displayed row is accepted.
    None,
    /// Some, but not all, selectable displayed rows are accepted. This is the
    /// `indeterminate` DOM property.
    Partial,
    /// Every selectable displayed row is accepted.
    All,
}

impl ServerTableSliceSelectionState {
    /// Whether the header checkbox renders checked.
    pub fn is_checked(self) -> bool {
        matches!(self, Self::All)
    }

    /// Whether the header checkbox renders indeterminate.
    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Partial)
    }

    /// Whether the header checkbox is inert (nothing on this page can be
    /// selected).
    pub fn is_disabled(self) -> bool {
        matches!(self, Self::NoSelectableRows)
    }

    /// Whether activating the header checkbox proposes selecting the slice
    /// (rather than clearing it).
    pub fn toggles_to_selected(self) -> bool {
        !matches!(self, Self::All)
    }

    /// Stable DOM marker for tests and consumers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoSelectableRows => "no-selectable-rows",
            Self::None => "none",
            Self::Partial => "partial",
            Self::All => "all",
        }
    }
}

/// Localized copy for the selection column. Every default explicitly names
/// *this page* rather than "all", so nothing in the rendered UI can be read as
/// a claim about unseen server rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerTableSelectionTexts {
    /// Accessible name of the leading selection column itself.
    pub column_header: String,
    /// Header-checkbox accessible name when activating it selects the slice;
    /// `{count}` is the number of selectable rows on this page.
    pub select_slice: String,
    /// Header-checkbox accessible name when activating it clears the slice;
    /// `{count}` is the number of selectable rows on this page.
    pub clear_slice: String,
    /// Header-checkbox accessible name when this page has nothing selectable.
    pub no_selectable_rows: String,
    /// Row-checkbox accessible name when unchecked; `{row}` is the row label.
    pub select_row: String,
    /// Row-checkbox accessible name when checked; `{row}` is the row label.
    pub deselect_row: String,
    /// Blocked-row accessible name; `{row}` and `{reason}` are substituted.
    pub blocked_row: String,
    /// Status line naming accepted keys that are NOT on this page;
    /// `{count}` is substituted.
    pub off_slice_notice: String,
}

impl Default for ServerTableSelectionTexts {
    fn default() -> Self {
        Self {
            column_header: "Select rows on this page".to_owned(),
            select_slice: "Select all {count} rows on this page".to_owned(),
            clear_slice: "Clear the selected rows on this page".to_owned(),
            no_selectable_rows: "No rows on this page can be selected".to_owned(),
            select_row: "Select {row}".to_owned(),
            deselect_row: "Deselect {row}".to_owned(),
            blocked_row: "{row} cannot be selected: {reason}".to_owned(),
            off_slice_notice: "{count} selected rows are not on this page".to_owned(),
        }
    }
}

impl ServerTableSelectionTexts {
    /// Accessible name for the header checkbox in `state`, where `count` is
    /// the number of selectable rows on the current displayed slice.
    pub fn slice_label(&self, state: ServerTableSliceSelectionState, count: usize) -> String {
        let template = match state {
            ServerTableSliceSelectionState::NoSelectableRows => {
                return self.no_selectable_rows.clone();
            }
            ServerTableSliceSelectionState::All => &self.clear_slice,
            _ => &self.select_slice,
        };
        template.replace("{count}", &count.to_string())
    }

    /// Accessible name for one row's checkbox.
    pub fn row_label(&self, row: &str, selected: bool, blocked_reason: Option<&str>) -> String {
        if let Some(reason) = blocked_reason {
            return self
                .blocked_row
                .replace("{row}", row)
                .replace("{reason}", reason);
        }
        let template = if selected {
            &self.deselect_row
        } else {
            &self.select_row
        };
        template.replace("{row}", row)
    }

    /// Status copy for accepted keys that are not on the current page.
    pub fn off_slice_label(&self, count: usize) -> String {
        self.off_slice_notice.replace("{count}", &count.to_string())
    }
}

/// Opt-in controlled checkbox multi-selection for
/// [`ServerDataTable`](super::ServerDataTable), keyed by the table's required
/// `row_key`.
///
/// Mutually exclusive with the single-row
/// [`ServerTableSelection`](super::ServerTableSelection): supplying both is a
/// configuration error that renders a visible `role="alert"` panel rather than
/// being resolved to one of them.
///
/// ```rust,no_run
/// # use std::collections::BTreeSet;
/// # use leptos::prelude::*;
/// # use leptos_daisyui_rs::components::*;
/// # fn demo() {
/// let accepted = RwSignal::new(BTreeSet::<String>::new());
/// let selection = ServerTableMultiSelection::controlled(
///     accepted.into(),
///     Callback::new(move |proposal: ServerTableSelectionProposal| {
///         // Accepted truth stays caller-owned: apply, or decline.
///         accepted.set(proposal.keys);
///     }),
/// );
/// # let _ = selection;
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct ServerTableMultiSelection {
    pub(crate) selected_keys: Signal<BTreeSet<String>>,
    pub(crate) on_change: Callback<ServerTableSelectionProposal>,
    pub(crate) scope: Option<Signal<String>>,
    pub(crate) row_label: Option<Callback<TableRow, String>>,
    pub(crate) selectable: Option<Callback<TableRow, ServerTableRowSelectability>>,
    pub(crate) texts: Signal<ServerTableSelectionTexts>,
}

impl ServerTableMultiSelection {
    /// Creates controlled multi-selection ownership over a set of stable row
    /// keys. `on_change` receives complete replacement proposals.
    pub fn controlled(
        selected_keys: Signal<BTreeSet<String>>,
        on_change: Callback<ServerTableSelectionProposal>,
    ) -> Self {
        Self {
            selected_keys,
            on_change,
            scope: None,
            row_label: None,
            selectable: None,
            texts: Signal::stored(ServerTableSelectionTexts::default()),
        }
    }

    /// Declares the dataset/scope identity every proposal is stamped with.
    ///
    /// Change it whenever the meaning of a key changes — a different tenant,
    /// a re-scoped query, a new cursor stream — and reject any proposal whose
    /// [`ServerTableSelectionProposal::scope`] no longer matches. The
    /// component itself never clears the caller's set; clearing is an atomic
    /// caller action taken on the same change that moved the scope.
    pub fn with_scope(mut self, scope: Signal<String>) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Supplies the human-readable row name used in each checkbox's
    /// accessible name. Defaults to the row's stable key, which is always
    /// present but rarely the best thing to hear read aloud.
    pub fn with_row_label(mut self, row_label: Callback<TableRow, String>) -> Self {
        self.row_label = Some(row_label);
        self
    }

    /// Marks individual displayed rows unselectable with a reason.
    pub fn with_row_selectable(
        mut self,
        selectable: Callback<TableRow, ServerTableRowSelectability>,
    ) -> Self {
        self.selectable = Some(selectable);
        self
    }

    /// Replaces the localized selection copy.
    pub fn with_texts(mut self, texts: Signal<ServerTableSelectionTexts>) -> Self {
        self.texts = texts;
        self
    }

    /// The caller-owned accepted key set.
    pub fn selected_keys(self) -> Signal<BTreeSet<String>> {
        self.selected_keys
    }

    /// The declared dataset scope, or `""` when none was declared.
    pub fn scope_value(self) -> String {
        self.scope.map(|scope| scope.get()).unwrap_or_default()
    }

    pub(crate) fn scope_value_untracked(self) -> String {
        self.scope
            .map(|scope| scope.get_untracked())
            .unwrap_or_default()
    }
}

/// Header state over the SELECTABLE keys of the current displayed slice.
///
/// `slice_keys` must already be filtered to selectable rows; `accepted` is the
/// caller's complete accepted set and may legitimately contain keys that are
/// not on this page — those are ignored here on purpose.
pub fn slice_selection_state(
    slice_keys: &[String],
    accepted: &BTreeSet<String>,
) -> ServerTableSliceSelectionState {
    if slice_keys.is_empty() {
        return ServerTableSliceSelectionState::NoSelectableRows;
    }
    let accepted_here = slice_keys
        .iter()
        .filter(|key| accepted.contains(*key))
        .count();
    if accepted_here == 0 {
        ServerTableSliceSelectionState::None
    } else if accepted_here == slice_keys.len() {
        ServerTableSliceSelectionState::All
    } else {
        ServerTableSliceSelectionState::Partial
    }
}

/// Complete proposed set after toggling one row. Every other accepted key —
/// on this page or not — is carried through untouched.
pub fn propose_row_toggle(
    accepted: &BTreeSet<String>,
    key: &str,
    selected: bool,
) -> BTreeSet<String> {
    let mut next = accepted.clone();
    if selected {
        next.insert(key.to_owned());
    } else {
        next.remove(key);
    }
    next
}

/// Complete proposed set after the header checkbox toggled the displayed
/// slice. Only `slice_keys` are added or removed; accepted keys outside the
/// slice are carried through untouched, which is how a bulk selection
/// survives paging.
pub fn propose_slice_toggle(
    accepted: &BTreeSet<String>,
    slice_keys: &[String],
    selected: bool,
) -> BTreeSet<String> {
    let mut next = accepted.clone();
    for key in slice_keys {
        if selected {
            next.insert(key.clone());
        } else {
            next.remove(key);
        }
    }
    next
}

/// How many accepted keys are NOT on the current displayed slice.
///
/// `displayed_keys` is every rendered row's key, selectable or not: a blocked
/// row that is already accepted is still on this page.
pub fn off_slice_selected_count(accepted: &BTreeSet<String>, displayed_keys: &[String]) -> usize {
    let displayed: BTreeSet<&str> = displayed_keys.iter().map(String::as_str).collect();
    accepted
        .iter()
        .filter(|key| !displayed.contains(key.as_str()))
        .count()
}

/// The selection model a configuration resolves to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResolvedSelectionMode {
    /// No selection binding at all.
    None,
    /// Controlled single-row selection.
    Single,
    /// Controlled checkbox multi-selection.
    Multi,
}

/// Rejects incompatible selection configurations instead of silently
/// resolving them to one mode.
pub(crate) fn resolve_selection_mode(
    has_single: bool,
    has_multi: bool,
    has_row_key: bool,
) -> Result<ResolvedSelectionMode, &'static str> {
    match (has_single, has_multi) {
        (true, true) => Err(CONFLICTING_SELECTION_MODES_CONFIGURATION),
        (false, true) if !has_row_key => Err(MULTI_SELECTION_WITHOUT_ROW_KEY_CONFIGURATION),
        (true, false) if !has_row_key => Err(SELECTION_WITHOUT_ROW_KEY_CONFIGURATION),
        (false, true) => Ok(ResolvedSelectionMode::Multi),
        (true, false) => Ok(ResolvedSelectionMode::Single),
        (false, false) => Ok(ResolvedSelectionMode::None),
    }
}

/// One displayed row's selection facts, resolved once per rows change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DisplayedSelectionRow {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) blocked_reason: Option<String>,
}

impl DisplayedSelectionRow {
    pub(crate) fn is_selectable(&self) -> bool {
        self.blocked_reason.is_none()
    }
}

/// The selectable keys of a displayed slice, in row order.
pub(crate) fn selectable_keys(rows: &[DisplayedSelectionRow]) -> Vec<String> {
    rows.iter()
        .filter(|row| row.is_selectable())
        .map(|row| row.key.clone())
        .collect()
}

/// Every displayed key, selectable or not, in row order.
pub(crate) fn displayed_keys(rows: &[DisplayedSelectionRow]) -> Vec<String> {
    rows.iter().map(|row| row.key.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(keys: &[&str]) -> BTreeSet<String> {
        keys.iter().map(|key| (*key).to_owned()).collect()
    }

    fn keys(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn header_state_is_computed_only_over_the_displayed_slice() {
        let page = keys(&["a", "b", "c"]);
        assert_eq!(
            slice_selection_state(&page, &set(&[])),
            ServerTableSliceSelectionState::None
        );
        assert_eq!(
            slice_selection_state(&page, &set(&["b"])),
            ServerTableSliceSelectionState::Partial
        );
        assert_eq!(
            slice_selection_state(&page, &set(&["a", "b", "c"])),
            ServerTableSliceSelectionState::All
        );
    }

    #[test]
    fn accepted_keys_outside_the_page_never_tint_the_header() {
        let page = keys(&["a", "b"]);

        // The whole hazard in one assertion: a bulk selection carried in from
        // other pages must NOT make this page look partially selected, and
        // must not stop a fully-selected page reading as fully selected.
        assert_eq!(
            slice_selection_state(&page, &set(&["zz-not-on-this-page"])),
            ServerTableSliceSelectionState::None
        );
        assert_eq!(
            slice_selection_state(&page, &set(&["a", "b", "zz-not-on-this-page"])),
            ServerTableSliceSelectionState::All
        );
        assert_eq!(
            slice_selection_state(&page, &set(&["a", "zz-not-on-this-page"])),
            ServerTableSliceSelectionState::Partial
        );
    }

    #[test]
    fn an_empty_or_fully_blocked_page_disables_the_header_checkbox() {
        let state = slice_selection_state(&[], &set(&["a"]));
        assert_eq!(state, ServerTableSliceSelectionState::NoSelectableRows);
        assert!(state.is_disabled());
        assert!(!state.is_checked());
        assert!(!state.is_indeterminate());
    }

    #[test]
    fn header_state_maps_to_exactly_one_dom_presentation() {
        for (state, checked, indeterminate, disabled) in [
            (
                ServerTableSliceSelectionState::NoSelectableRows,
                false,
                false,
                true,
            ),
            (ServerTableSliceSelectionState::None, false, false, false),
            (ServerTableSliceSelectionState::Partial, false, true, false),
            (ServerTableSliceSelectionState::All, true, false, false),
        ] {
            assert_eq!(state.is_checked(), checked, "{state:?}");
            assert_eq!(state.is_indeterminate(), indeterminate, "{state:?}");
            assert_eq!(state.is_disabled(), disabled, "{state:?}");
        }
    }

    #[test]
    fn a_row_toggle_preserves_every_off_page_key() {
        let accepted = set(&["page-a", "elsewhere-1", "elsewhere-2"]);

        let added = propose_row_toggle(&accepted, "page-b", true);
        assert_eq!(
            added,
            set(&["page-a", "page-b", "elsewhere-1", "elsewhere-2"])
        );

        let removed = propose_row_toggle(&accepted, "page-a", false);
        assert_eq!(removed, set(&["elsewhere-1", "elsewhere-2"]));
    }

    #[test]
    fn a_row_toggle_is_idempotent_in_both_directions() {
        let accepted = set(&["a"]);
        assert_eq!(propose_row_toggle(&accepted, "a", true), set(&["a"]));
        assert_eq!(propose_row_toggle(&accepted, "b", false), set(&["a"]));
    }

    #[test]
    fn the_header_gesture_only_ever_touches_the_displayed_slice() {
        let accepted = set(&["elsewhere-1", "page-a"]);
        let page = keys(&["page-a", "page-b"]);

        let selected = propose_slice_toggle(&accepted, &page, true);
        assert_eq!(selected, set(&["elsewhere-1", "page-a", "page-b"]));

        // Clearing the page leaves the off-page keys exactly where they were:
        // a header checkbox can never mean "clear the whole dataset".
        let cleared = propose_slice_toggle(&accepted, &page, false);
        assert_eq!(cleared, set(&["elsewhere-1"]));
    }

    #[test]
    fn paging_across_three_slices_accumulates_without_losing_earlier_keys() {
        let mut accepted = BTreeSet::new();
        for page in [keys(&["a1", "a2"]), keys(&["b1", "b2"]), keys(&["c1"])] {
            accepted = propose_slice_toggle(&accepted, &page, true);
        }
        assert_eq!(accepted, set(&["a1", "a2", "b1", "b2", "c1"]));

        // Returning to the first page and clearing it removes only that page.
        let back = propose_slice_toggle(&accepted, &keys(&["a1", "a2"]), false);
        assert_eq!(back, set(&["b1", "b2", "c1"]));
    }

    #[test]
    fn a_replaced_page_cannot_alias_selection_onto_a_different_row() {
        // Same page positions, entirely different business keys: index-based
        // selection would carry "row 0 is selected" onto `x1`.
        let accepted = propose_row_toggle(&BTreeSet::new(), "a1", true);
        let replaced_page = keys(&["x1", "x2"]);

        assert_eq!(
            slice_selection_state(&replaced_page, &accepted),
            ServerTableSliceSelectionState::None
        );
        assert!(!accepted.contains("x1"));
        assert_eq!(off_slice_selected_count(&accepted, &replaced_page), 1);
    }

    #[test]
    fn off_slice_count_ignores_blocked_but_present_rows() {
        let accepted = set(&["visible", "blocked-but-here", "gone"]);
        // `displayed_keys` includes blocked rows: they ARE on this page.
        let displayed = keys(&["visible", "blocked-but-here"]);
        assert_eq!(off_slice_selected_count(&accepted, &displayed), 1);
        assert_eq!(off_slice_selected_count(&BTreeSet::new(), &displayed), 0);
    }

    #[test]
    fn removing_an_accepted_row_from_the_page_leaves_the_key_accepted_and_reported() {
        let accepted = set(&["a", "b"]);
        // "b" was deleted server-side and is no longer returned.
        let displayed = keys(&["a"]);
        assert_eq!(
            slice_selection_state(&displayed, &accepted),
            ServerTableSliceSelectionState::All
        );
        assert_eq!(off_slice_selected_count(&accepted, &displayed), 1);
    }

    #[test]
    fn blocked_rows_are_excluded_from_the_header_slice_but_still_displayed() {
        let rows = vec![
            DisplayedSelectionRow {
                key: "a".to_owned(),
                label: "Alpha".to_owned(),
                blocked_reason: None,
            },
            DisplayedSelectionRow {
                key: "b".to_owned(),
                label: "Beta".to_owned(),
                blocked_reason: Some("Closed".to_owned()),
            },
        ];
        assert_eq!(selectable_keys(&rows), keys(&["a"]));
        assert_eq!(displayed_keys(&rows), keys(&["a", "b"]));
        assert_eq!(
            slice_selection_state(&selectable_keys(&rows), &set(&["a"])),
            ServerTableSliceSelectionState::All,
            "a blocked row must not hold the header checkbox at Partial forever"
        );
    }

    #[test]
    fn incompatible_selection_modes_are_rejected_rather_than_resolved() {
        assert_eq!(
            resolve_selection_mode(true, true, true),
            Err(CONFLICTING_SELECTION_MODES_CONFIGURATION)
        );
        assert_eq!(
            resolve_selection_mode(false, true, false),
            Err(MULTI_SELECTION_WITHOUT_ROW_KEY_CONFIGURATION)
        );
        assert_eq!(
            resolve_selection_mode(true, false, false),
            Err(SELECTION_WITHOUT_ROW_KEY_CONFIGURATION)
        );
        assert_eq!(
            resolve_selection_mode(false, false, false),
            Ok(ResolvedSelectionMode::None)
        );
        assert_eq!(
            resolve_selection_mode(true, false, true),
            Ok(ResolvedSelectionMode::Single)
        );
        assert_eq!(
            resolve_selection_mode(false, true, true),
            Ok(ResolvedSelectionMode::Multi)
        );
    }

    #[test]
    fn default_copy_names_this_page_and_never_claims_all() {
        let texts = ServerTableSelectionTexts::default();
        for label in [
            texts.slice_label(ServerTableSliceSelectionState::None, 10),
            texts.slice_label(ServerTableSliceSelectionState::Partial, 10),
            texts.slice_label(ServerTableSliceSelectionState::All, 10),
            texts.column_header.clone(),
            texts.off_slice_label(3),
        ] {
            let lowered = label.to_lowercase();
            assert!(
                lowered.contains("this page"),
                "selection copy must name the current displayed slice: {label:?}"
            );
        }
        assert_eq!(
            texts.slice_label(ServerTableSliceSelectionState::None, 10),
            "Select all 10 rows on this page"
        );
        assert_eq!(
            texts.off_slice_label(3),
            "3 selected rows are not on this page"
        );
    }

    #[test]
    fn row_copy_names_the_row_and_folds_in_a_blocked_reason() {
        let texts = ServerTableSelectionTexts::default();
        assert_eq!(
            texts.row_label("Ticket 12", false, None),
            "Select Ticket 12"
        );
        assert_eq!(
            texts.row_label("Ticket 12", true, None),
            "Deselect Ticket 12"
        );
        assert_eq!(
            texts.row_label("Ticket 12", false, Some("Already assigned")),
            "Ticket 12 cannot be selected: Already assigned"
        );
    }

    #[test]
    fn selectability_reports_its_caller_supplied_reason() {
        assert!(ServerTableRowSelectability::default().is_selectable());
        assert_eq!(ServerTableRowSelectability::default().reason(), None);
        let blocked = ServerTableRowSelectability::blocked("Locked");
        assert!(!blocked.is_selectable());
        assert_eq!(blocked.reason(), Some("Locked"));
    }
}
