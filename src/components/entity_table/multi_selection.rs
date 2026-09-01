//! Controlled checkbox multi-selection for [`EntityTable`](super::EntityTable)
//! (`ldui-nz6d`).
//!
//! # What the header checkbox governs, said in the type
//!
//! `EntityTable` is the client-snapshot table: it holds the *complete*
//! dataset, so unlike [`ServerDataTable`](crate::components::ServerDataTable)
//! it could technically offer a genuine "select every row in the dataset"
//! affordance. It deliberately does not, and the type says so.
//!
//! [`EntityTableDisplayedPageSelection`] is computed over
//! [`EntityTableDisplayedPage`] -- the exact stable keys the table is
//! rendering right now, after filtering, sorting and paging have all been
//! applied. Nothing in this module can name a key the table is not currently
//! showing, so no code path can widen the header checkbox into "select all"
//! by accident. A user who wants every row selects a large enough page size
//! first, which makes the widening an explicit, visible act.
//!
//! `indeterminate` therefore has one precise meaning here: *some but not all
//! of the rows currently displayed are selected*. Accepted keys that live on
//! another page never tint the header checkbox and, in particular, can never
//! turn it checked -- that would tell a user that the rows in front of them
//! are all selected when they are not. Those off-page keys are reported
//! separately and out loud by [`off_page_selected_count`].
//!
//! # Accepted truth is caller-owned, and the callback is atomic
//!
//! The component holds no selection state. The caller supplies the accepted
//! key set as a `Signal`, and every gesture -- a row checkbox or the header
//! checkbox covering any number of rows -- emits exactly ONE
//! [`EntityTableSelectionProposal`] carrying the COMPLETE resulting set. It
//! is never a stream of per-row deltas the caller has to reassemble, and it
//! is never a partial patch: apply `keys` wholesale or decline it wholesale.
//! Both checkboxes re-assert accepted truth onto the element the browser just
//! toggled before emitting, so a declined or delayed proposal leaves no
//! optimistic divergence.
//!
//! # Keys cannot be aliased
//!
//! Selection is keyed by the stable business key the table's mandatory
//! `row_key` already produces, never by row position. Every proposal is a
//! pure set operation over named keys ([`propose_entity_row_toggle`],
//! [`propose_entity_displayed_page_toggle`]) that carries every other accepted key
//! through untouched, so removing a row, replacing the dataset, re-sorting or
//! re-paging can only ever stop a key from being *rendered* -- there is no
//! index anywhere that a different entity could slide into. Off-page keys
//! survive paging by construction rather than by a preservation step that
//! could be forgotten. [`EntityTableSelectionProposal::scope`] additionally
//! stamps each proposal with the dataset identity it was computed against, so
//! a caller that swaps datasets can refuse a proposal minted against the
//! previous one instead of having keys silently relabelled.

use leptos::prelude::*;
use std::collections::BTreeSet;

/// Combining the single-row and multi-row selection models is rejected, not
/// resolved: silently honouring one of them would make a bulk-assignment
/// workflow act on a single row, or a single-row workflow act on a set.
pub(super) const CONFLICTING_SELECTION_MODES_CONFIGURATION: &str =
    "EntityTable configuration cannot combine selection with multi_selection";

/// Fixed `<col>` track id for the leading selection control column.
///
/// Namespaced out of caller reach: it is not a column, so it must never
/// collide with a caller's `EntityColumn` id, appear in the column chooser,
/// take part in sorting, filtering or resizing, or show up in an
/// `on_display_projection` export.
pub(super) const SELECTION_COLUMN_TRACK_ID: &str = "__ldui-entity-selection";

/// Fixed width of the leading selection control track, in pixels.
pub(super) const SELECTION_COLUMN_TRACK_WIDTH: u32 = 48;

/// The selection model one `EntityTable` configuration resolves to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EntityTableSelectionMode {
    /// No selection binding at all.
    None,
    /// Controlled single-row selection.
    Single,
    /// Controlled checkbox multi-selection.
    Multi,
}

/// Rejects incompatible selection configurations instead of resolving them to
/// one mode by an invisible precedence rule.
///
/// `EntityTable`'s `row_key` is mandatory, so unlike `ServerDataTable` there
/// is no "selection without row identity" case to represent -- the only
/// incompatible configuration is asking for both models at once.
pub(super) fn resolve_entity_selection_mode(
    has_single: bool,
    has_multi: bool,
) -> Result<EntityTableSelectionMode, &'static str> {
    match (has_single, has_multi) {
        (true, true) => Err(CONFLICTING_SELECTION_MODES_CONFIGURATION),
        (false, true) => Ok(EntityTableSelectionMode::Multi),
        (true, false) => Ok(EntityTableSelectionMode::Single),
        (false, false) => Ok(EntityTableSelectionMode::None),
    }
}

/// The stable keys `EntityTable` is rendering right now, in row order.
///
/// This is the *entire* population the header checkbox may act on, and the
/// only population [`EntityTableDisplayedPageSelection`] is computed over. It
/// is built from the rows the table actually paints -- the output of the one
/// resolved [`EntityPageSize`](super::EntityPageSize) every other part of the
/// render reads (`ldui-5p06`) -- rather than recomputed from a page index and
/// a preference, so the header checkbox cannot govern a different set of rows
/// than the body shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityTableDisplayedPage {
    keys: Vec<String>,
}

impl EntityTableDisplayedPage {
    /// Wraps the rendered page's stable keys, in row order.
    pub fn new(keys: Vec<String>) -> Self {
        Self { keys }
    }

    /// The rendered keys, in row order.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// How many rows are currently displayed.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Whether the table is currently rendering no rows at all.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Header-checkbox state for this page against the caller's accepted set.
    pub fn selection_state(
        &self,
        accepted: &BTreeSet<String>,
    ) -> EntityTableDisplayedPageSelection {
        displayed_page_selection_state(&self.keys, accepted)
    }
}

/// Header-checkbox state, computed over the CURRENTLY DISPLAYED ROWS only.
///
/// Accepted keys that are not on the current page never move this off
/// [`None`](EntityTableDisplayedPageSelection::None) and never force
/// [`Partial`](EntityTableDisplayedPageSelection::Partial). The header
/// checkbox answers exactly one question -- "are the rows in front of me
/// selected?" -- and letting unseen rows answer it would make a checked box
/// mean something the user cannot verify. The off-page count is surfaced
/// separately, as its own explicit line of copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityTableDisplayedPageSelection {
    /// The table is displaying no rows -- the header checkbox is unchecked
    /// and disabled.
    NoRows,
    /// No displayed row is selected.
    None,
    /// Some, but not all, displayed rows are selected. This is exactly the
    /// `indeterminate` DOM property.
    Partial,
    /// Every displayed row is selected.
    All,
}

impl EntityTableDisplayedPageSelection {
    /// Whether the header checkbox renders checked.
    pub fn is_checked(self) -> bool {
        matches!(self, Self::All)
    }

    /// Whether the header checkbox renders indeterminate.
    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Partial)
    }

    /// Whether the header checkbox is inert (nothing is displayed).
    pub fn is_disabled(self) -> bool {
        matches!(self, Self::NoRows)
    }

    /// Whether activating the header checkbox proposes selecting the
    /// displayed page rather than clearing it.
    pub fn toggles_to_selected(self) -> bool {
        !matches!(self, Self::All)
    }

    /// Stable DOM marker for tests and consumers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoRows => "no-rows",
            Self::None => "none",
            Self::Partial => "partial",
            Self::All => "all",
        }
    }
}

/// Header state over the keys of the currently displayed page.
///
/// `accepted` is the caller's complete accepted set and may legitimately
/// contain keys that are not on this page; those are ignored here on purpose.
pub fn displayed_page_selection_state(
    displayed_keys: &[String],
    accepted: &BTreeSet<String>,
) -> EntityTableDisplayedPageSelection {
    if displayed_keys.is_empty() {
        return EntityTableDisplayedPageSelection::NoRows;
    }
    let accepted_here = displayed_keys
        .iter()
        .filter(|key| accepted.contains(*key))
        .count();
    if accepted_here == 0 {
        EntityTableDisplayedPageSelection::None
    } else if accepted_here == displayed_keys.len() {
        EntityTableDisplayedPageSelection::All
    } else {
        EntityTableDisplayedPageSelection::Partial
    }
}

/// How many accepted keys are NOT on the currently displayed page.
pub fn off_page_selected_count(accepted: &BTreeSet<String>, displayed_keys: &[String]) -> usize {
    let displayed: BTreeSet<&str> = displayed_keys.iter().map(String::as_str).collect();
    accepted
        .iter()
        .filter(|key| !displayed.contains(key.as_str()))
        .count()
}

/// Complete proposed set after toggling one row. Every other accepted key --
/// on this page or not -- is carried through untouched.
pub fn propose_entity_row_toggle(
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
/// page. Only `displayed_keys` are added or removed; accepted keys outside
/// the page are carried through untouched, which is how a bulk selection
/// survives paging, filtering and sorting.
pub fn propose_entity_displayed_page_toggle(
    accepted: &BTreeSet<String>,
    displayed_keys: &[String],
    selected: bool,
) -> BTreeSet<String> {
    let mut next = accepted.clone();
    for key in displayed_keys {
        if selected {
            next.insert(key.clone());
        } else {
            next.remove(key);
        }
    }
    next
}

/// What the user did to produce an [`EntityTableSelectionProposal`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityTableSelectionCause {
    /// One row's checkbox was toggled.
    Row {
        /// The stable key of the toggled row.
        key: String,
        /// `true` when the gesture asks for the key to become accepted.
        selected: bool,
    },
    /// The header checkbox toggled the currently displayed page.
    ///
    /// Named `DisplayedPage`, never `All`: it can only ever cover the rows
    /// the table is rendering, which `keys` states exactly.
    DisplayedPage {
        /// `true` when the gesture asks for the page to become accepted.
        selected: bool,
        /// The displayed keys the gesture covered, in row order.
        keys: Vec<String>,
    },
}

/// One user-proposed replacement for the caller's accepted selected-key set.
///
/// `keys` is the COMPLETE proposed set, not a delta: apply it wholesale or
/// reject it wholesale. One gesture produces exactly one of these, however
/// many rows it covered, so a caller never has to reassemble a set from a
/// stream of per-row events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityTableSelectionProposal {
    /// The complete proposed accepted set, including keys that are not on the
    /// currently displayed page.
    pub keys: BTreeSet<String>,
    /// The gesture that produced this proposal.
    pub cause: EntityTableSelectionCause,
    /// The dataset/scope identity the proposal was computed against, read at
    /// gesture time.
    ///
    /// Defaults to the table's `dataset_identity`. A caller that changes
    /// datasets compares this against its current identity and rejects a
    /// proposal minted against the previous one, which is what makes "a
    /// dataset change must not silently relabel accepted keys" checkable
    /// rather than merely intended.
    pub scope: String,
}

/// Localized copy for the selection column. Every default explicitly names
/// *this page*, so nothing in the rendered UI can be read as a claim about
/// rows the user is not looking at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityTableSelectionTexts {
    /// Accessible name of the leading selection column itself.
    pub column_header: String,
    /// Header-checkbox accessible name when activating it selects the page;
    /// `{count}` is the number of rows currently displayed.
    pub select_page: String,
    /// Header-checkbox accessible name when activating it clears the page;
    /// `{count}` is the number of rows currently displayed.
    pub clear_page: String,
    /// Header-checkbox accessible name when no rows are displayed.
    pub no_rows: String,
    /// Row-checkbox accessible name when unchecked; `{row}` is the row label.
    pub select_row: String,
    /// Row-checkbox accessible name when checked; `{row}` is the row label.
    pub deselect_row: String,
    /// Live-region status naming how many rows are selected in total and how
    /// many of those are not on this page; `{total}` and `{off_page}` are
    /// substituted.
    pub selection_summary: String,
    /// Live-region status when nothing is selected.
    pub selection_summary_empty: String,
}

impl Default for EntityTableSelectionTexts {
    fn default() -> Self {
        Self {
            column_header: "Select rows on this page".to_owned(),
            select_page: "Select all {count} rows on this page".to_owned(),
            clear_page: "Clear the selected rows on this page".to_owned(),
            no_rows: "No rows are displayed".to_owned(),
            select_row: "Select {row}".to_owned(),
            deselect_row: "Deselect {row}".to_owned(),
            selection_summary: "{total} rows selected, {off_page} of them not on this page"
                .to_owned(),
            selection_summary_empty: "No rows selected".to_owned(),
        }
    }
}

impl EntityTableSelectionTexts {
    /// Accessible name for the header checkbox in `state`, where `count` is
    /// the number of rows currently displayed.
    pub fn page_label(&self, state: EntityTableDisplayedPageSelection, count: usize) -> String {
        let template = match state {
            EntityTableDisplayedPageSelection::NoRows => return self.no_rows.clone(),
            EntityTableDisplayedPageSelection::All => &self.clear_page,
            _ => &self.select_page,
        };
        template.replace("{count}", &count.to_string())
    }

    /// Accessible name for one row's checkbox. `row` is the row's own
    /// human-readable name, never the bare word "checkbox".
    pub fn row_label(&self, row: &str, selected: bool) -> String {
        let template = if selected {
            &self.deselect_row
        } else {
            &self.select_row
        };
        template.replace("{row}", row)
    }

    /// Live-region copy for the current accepted set.
    pub fn summary_label(&self, total: usize, off_page: usize) -> String {
        if total == 0 {
            return self.selection_summary_empty.clone();
        }
        self.selection_summary
            .replace("{total}", &total.to_string())
            .replace("{off_page}", &off_page.to_string())
    }
}

/// The human-readable name a row's checkbox announces.
///
/// Prefers the row's own leading visible cell text, so a screen reader hears
/// "Select Mexico City Client 2" rather than "Select checkbox" or a raw
/// database id. Falls back to the stable key when that text is missing or
/// blank, which is always present and still identifies the row.
pub(super) fn displayed_row_label(key: &str, primary_text: Option<&str>) -> String {
    primary_text
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map_or_else(|| key.to_owned(), str::to_owned)
}

/// Opt-in controlled checkbox multi-selection for
/// [`EntityTable`](super::EntityTable), keyed by the table's mandatory
/// `row_key`.
///
/// Mutually exclusive with the single-row
/// [`EntityTableSelection`](super::EntityTableSelection): supplying both is a
/// configuration error refused at construction, never resolved to one of them
/// by precedence.
///
/// ```rust,no_run
/// # use std::collections::BTreeSet;
/// # use leptos::prelude::*;
/// # use leptos_daisyui_rs::components::*;
/// # fn demo() {
/// let accepted = RwSignal::new(BTreeSet::<String>::new());
/// let selection = EntityTableMultiSelection::controlled(
///     accepted.into(),
///     Callback::new(move |proposal: EntityTableSelectionProposal| {
///         // One atomic event carrying the complete resulting set.
///         accepted.set(proposal.keys);
///     }),
/// );
/// # let _ = selection;
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct EntityTableMultiSelection {
    pub(super) selected_keys: Signal<BTreeSet<String>>,
    pub(super) on_change: Callback<EntityTableSelectionProposal>,
    pub(super) scope: Option<Signal<String>>,
    pub(super) row_label: Option<Callback<String, String>>,
    pub(super) texts: Signal<EntityTableSelectionTexts>,
}

impl EntityTableMultiSelection {
    /// Creates controlled multi-selection ownership over a set of stable row
    /// keys. `on_change` receives one complete replacement proposal per
    /// gesture.
    pub fn controlled(
        selected_keys: Signal<BTreeSet<String>>,
        on_change: Callback<EntityTableSelectionProposal>,
    ) -> Self {
        Self {
            selected_keys,
            on_change,
            scope: None,
            row_label: None,
            texts: Signal::stored(EntityTableSelectionTexts::default()),
        }
    }

    /// Overrides the dataset/scope identity every proposal is stamped with.
    ///
    /// Defaults to the table's `dataset_identity`. Supply this when the
    /// meaning of a key changes on some other axis -- a different tenant, a
    /// re-scoped query -- and reject any proposal whose
    /// [`EntityTableSelectionProposal::scope`] no longer matches. The
    /// component never clears the caller's set; clearing is an atomic caller
    /// action taken on the same change that moved the scope.
    pub fn with_scope(mut self, scope: Signal<String>) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Supplies the human-readable row name each checkbox announces, resolved
    /// from the row's stable key.
    ///
    /// Defaults to the row's leading visible cell text, falling back to the
    /// key itself.
    pub fn with_row_label(mut self, row_label: Callback<String, String>) -> Self {
        self.row_label = Some(row_label);
        self
    }

    /// Replaces the localized selection copy.
    pub fn with_texts(mut self, texts: Signal<EntityTableSelectionTexts>) -> Self {
        self.texts = texts;
        self
    }

    /// The caller-owned accepted key set.
    pub fn selected_keys(self) -> Signal<BTreeSet<String>> {
        self.selected_keys
    }
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

    // ── resolve_entity_selection_mode ──

    #[test]
    fn no_selection_binding_resolves_to_no_selection() {
        assert_eq!(
            resolve_entity_selection_mode(false, false),
            Ok(EntityTableSelectionMode::None)
        );
    }

    #[test]
    fn one_binding_resolves_to_that_model() {
        assert_eq!(
            resolve_entity_selection_mode(true, false),
            Ok(EntityTableSelectionMode::Single)
        );
        assert_eq!(
            resolve_entity_selection_mode(false, true),
            Ok(EntityTableSelectionMode::Multi)
        );
    }

    #[test]
    fn combining_both_selection_models_is_rejected_not_resolved() {
        // Not a precedence rule: neither model wins. The caller is told, by
        // name, that the configuration is refused.
        let error = resolve_entity_selection_mode(true, true)
            .expect_err("combining both selection models must be refused");
        assert_eq!(error, CONFLICTING_SELECTION_MODES_CONFIGURATION);
        assert!(error.contains("selection"));
        assert!(error.contains("multi_selection"));
    }

    // ── displayed_page_selection_state ──

    #[test]
    fn an_empty_page_has_nothing_to_select() {
        let state = displayed_page_selection_state(&[], &set(&["a"]));
        assert_eq!(state, EntityTableDisplayedPageSelection::NoRows);
        assert!(state.is_disabled());
        assert!(!state.is_checked());
        assert!(!state.is_indeterminate());
    }

    #[test]
    fn no_displayed_row_selected_is_unchecked() {
        let state = displayed_page_selection_state(&keys(&["a", "b"]), &BTreeSet::new());
        assert_eq!(state, EntityTableDisplayedPageSelection::None);
        assert!(!state.is_checked());
        assert!(!state.is_indeterminate());
        assert!(state.toggles_to_selected());
    }

    #[test]
    fn some_displayed_rows_selected_is_indeterminate() {
        let state = displayed_page_selection_state(&keys(&["a", "b"]), &set(&["a"]));
        assert_eq!(state, EntityTableDisplayedPageSelection::Partial);
        assert!(state.is_indeterminate());
        assert!(!state.is_checked());
        // A partial page still toggles TOWARD selected, so the first
        // activation completes the page rather than clearing the user's
        // existing picks.
        assert!(state.toggles_to_selected());
    }

    #[test]
    fn every_displayed_row_selected_is_checked() {
        let state = displayed_page_selection_state(&keys(&["a", "b"]), &set(&["a", "b"]));
        assert_eq!(state, EntityTableDisplayedPageSelection::All);
        assert!(state.is_checked());
        assert!(!state.is_indeterminate());
        assert!(!state.toggles_to_selected());
    }

    #[test]
    fn off_page_keys_never_check_the_header_checkbox() {
        // The load-bearing rule. Three keys accepted, none of them on this
        // page: the header must read UNCHECKED and NOT indeterminate, or it
        // would claim the rows in front of the user are selected.
        let accepted = set(&["p2-a", "p2-b", "p2-c"]);
        let state = displayed_page_selection_state(&keys(&["p1-a", "p1-b"]), &accepted);
        assert_eq!(state, EntityTableDisplayedPageSelection::None);
        assert!(!state.is_checked());
        assert!(!state.is_indeterminate());
    }

    #[test]
    fn off_page_keys_never_force_indeterminate_on_a_fully_selected_page() {
        // Every displayed row IS selected, and there are also selected rows
        // elsewhere. The header is checked -- "all the rows in front of me"
        // -- not partial.
        let accepted = set(&["p1-a", "p1-b", "elsewhere"]);
        let state = displayed_page_selection_state(&keys(&["p1-a", "p1-b"]), &accepted);
        assert_eq!(state, EntityTableDisplayedPageSelection::All);
    }

    #[test]
    fn header_state_reads_only_the_supplied_displayed_keys() {
        // The same accepted set produces different header states for
        // different pages of the same dataset, because the page is the whole
        // population the header governs.
        let accepted = set(&["r1", "r3"]);
        assert_eq!(
            displayed_page_selection_state(&keys(&["r1"]), &accepted),
            EntityTableDisplayedPageSelection::All
        );
        assert_eq!(
            displayed_page_selection_state(&keys(&["r1", "r2"]), &accepted),
            EntityTableDisplayedPageSelection::Partial
        );
        assert_eq!(
            displayed_page_selection_state(&keys(&["r2"]), &accepted),
            EntityTableDisplayedPageSelection::None
        );
    }

    #[test]
    fn displayed_page_wrapper_agrees_with_the_free_function() {
        let page = EntityTableDisplayedPage::new(keys(&["a", "b"]));
        assert_eq!(page.len(), 2);
        assert!(!page.is_empty());
        assert_eq!(page.keys(), keys(&["a", "b"]).as_slice());
        assert_eq!(
            page.selection_state(&set(&["a"])),
            EntityTableDisplayedPageSelection::Partial
        );
        assert!(EntityTableDisplayedPage::default().is_empty());
    }

    #[test]
    fn state_markers_are_stable_dom_strings() {
        assert_eq!(
            EntityTableDisplayedPageSelection::NoRows.as_str(),
            "no-rows"
        );
        assert_eq!(EntityTableDisplayedPageSelection::None.as_str(), "none");
        assert_eq!(
            EntityTableDisplayedPageSelection::Partial.as_str(),
            "partial"
        );
        assert_eq!(EntityTableDisplayedPageSelection::All.as_str(), "all");
    }

    // ── off_page_selected_count ──

    #[test]
    fn off_page_count_ignores_displayed_keys() {
        let accepted = set(&["a", "b", "c", "d"]);
        assert_eq!(off_page_selected_count(&accepted, &keys(&["a", "c"])), 2);
        assert_eq!(off_page_selected_count(&accepted, &[]), 4);
        assert_eq!(
            off_page_selected_count(&accepted, &keys(&["a", "b", "c", "d"])),
            0
        );
    }

    #[test]
    fn off_page_count_ignores_displayed_rows_that_are_not_selected() {
        assert_eq!(off_page_selected_count(&set(&["a"]), &keys(&["b", "c"])), 1);
    }

    // ── propose_entity_row_toggle ──

    #[test]
    fn selecting_one_row_adds_only_that_key() {
        assert_eq!(
            propose_entity_row_toggle(&set(&["a"]), "b", true),
            set(&["a", "b"])
        );
    }

    #[test]
    fn deselecting_one_row_removes_only_that_key() {
        assert_eq!(
            propose_entity_row_toggle(&set(&["a", "b"]), "b", false),
            set(&["a"])
        );
    }

    #[test]
    fn a_row_toggle_carries_off_page_keys_through_untouched() {
        let accepted = set(&["page2-x", "page2-y"]);
        let proposed = propose_entity_row_toggle(&accepted, "page1-a", true);
        assert!(proposed.contains("page2-x"));
        assert!(proposed.contains("page2-y"));
        assert!(proposed.contains("page1-a"));
        assert_eq!(proposed.len(), 3);
    }

    #[test]
    fn a_row_toggle_is_idempotent_in_each_direction() {
        assert_eq!(
            propose_entity_row_toggle(&set(&["a"]), "a", true),
            set(&["a"])
        );
        assert_eq!(
            propose_entity_row_toggle(&set(&["a"]), "b", false),
            set(&["a"])
        );
    }

    // ── propose_entity_displayed_page_toggle ──

    #[test]
    fn selecting_the_page_adds_exactly_the_displayed_keys() {
        assert_eq!(
            propose_entity_displayed_page_toggle(&BTreeSet::new(), &keys(&["a", "b"]), true),
            set(&["a", "b"])
        );
    }

    #[test]
    fn clearing_the_page_removes_only_the_displayed_keys() {
        // The acceptance criterion, stated directly: header selection affects
        // only the rows currently displayed, and stable selected keys OUTSIDE
        // the current page are preserved.
        let accepted = set(&["p1-a", "p1-b", "p2-a", "p2-b"]);
        let proposed =
            propose_entity_displayed_page_toggle(&accepted, &keys(&["p1-a", "p1-b"]), false);
        assert_eq!(proposed, set(&["p2-a", "p2-b"]));
    }

    #[test]
    fn selecting_a_page_preserves_every_other_accepted_key() {
        let accepted = set(&["p2-a", "p2-b"]);
        let proposed =
            propose_entity_displayed_page_toggle(&accepted, &keys(&["p1-a", "p1-b"]), true);
        assert_eq!(proposed, set(&["p1-a", "p1-b", "p2-a", "p2-b"]));
    }

    #[test]
    fn a_bulk_selection_survives_a_full_paging_round_trip() {
        // Select page 1, page to 2 and select it, page back to 1: nothing
        // was lost, and nothing was aliased onto page 2's keys.
        let page1 = keys(&["p1-a", "p1-b"]);
        let page2 = keys(&["p2-a", "p2-b"]);
        let after_page1 = propose_entity_displayed_page_toggle(&BTreeSet::new(), &page1, true);
        let after_page2 = propose_entity_displayed_page_toggle(&after_page1, &page2, true);
        assert_eq!(after_page2, set(&["p1-a", "p1-b", "p2-a", "p2-b"]));
        // Back on page 1, the header reads checked from the page's own keys.
        assert_eq!(
            displayed_page_selection_state(&page1, &after_page2),
            EntityTableDisplayedPageSelection::All
        );
        // Clearing page 1 leaves page 2 intact.
        let cleared = propose_entity_displayed_page_toggle(&after_page2, &page1, false);
        assert_eq!(cleared, set(&["p2-a", "p2-b"]));
    }

    #[test]
    fn a_page_toggle_over_no_displayed_keys_changes_nothing() {
        let accepted = set(&["a"]);
        assert_eq!(
            propose_entity_displayed_page_toggle(&accepted, &[], true),
            accepted
        );
        assert_eq!(
            propose_entity_displayed_page_toggle(&accepted, &[], false),
            accepted
        );
    }

    // ── no aliasing across row removal / dataset replacement ──

    #[test]
    fn removing_a_selected_row_cannot_alias_its_selection_to_a_neighbour() {
        // Page was [a, b, c] with `b` selected. `b` is deleted; the page is
        // now [a, c]. Position 1 is now `c`, but selection is keyed, so `c`
        // is NOT selected and the header is NOT indeterminate from `b`.
        let accepted = set(&["b"]);
        let after_removal = keys(&["a", "c"]);
        assert!(!after_removal.iter().any(|key| accepted.contains(key)));
        assert_eq!(
            displayed_page_selection_state(&after_removal, &accepted),
            EntityTableDisplayedPageSelection::None
        );
        // The accepted set itself is caller-owned and untouched; the key is
        // simply reported as off-page until the caller reconciles it.
        assert_eq!(off_page_selected_count(&accepted, &after_removal), 1);
    }

    #[test]
    fn replacing_the_dataset_cannot_relabel_accepted_keys() {
        // Every key in the replacement dataset is different, so no accepted
        // key matches anything rendered -- rather than the first N rows of
        // the new dataset inheriting the old selection.
        let accepted = set(&["office-mx-1", "office-mx-2"]);
        let replacement = keys(&["office-in-1", "office-in-2", "office-in-3"]);
        assert_eq!(
            displayed_page_selection_state(&replacement, &accepted),
            EntityTableDisplayedPageSelection::None
        );
        assert_eq!(off_page_selected_count(&accepted, &replacement), 2);
    }

    #[test]
    fn a_sort_reorders_rows_without_moving_selection() {
        let accepted = set(&["r2"]);
        let ascending = keys(&["r1", "r2", "r3"]);
        let descending = keys(&["r3", "r2", "r1"]);
        assert_eq!(
            displayed_page_selection_state(&ascending, &accepted),
            displayed_page_selection_state(&descending, &accepted)
        );
        assert_eq!(
            displayed_page_selection_state(&descending, &accepted),
            EntityTableDisplayedPageSelection::Partial
        );
    }

    // ── texts ──

    #[test]
    fn default_copy_never_claims_more_than_this_page() {
        let texts = EntityTableSelectionTexts::default();
        for copy in [
            &texts.column_header,
            &texts.select_page,
            &texts.clear_page,
            &texts.selection_summary,
        ] {
            assert!(
                copy.contains("this page") || copy.contains("not on this page"),
                "selection copy must name the displayed page: {copy}"
            );
        }
    }

    #[test]
    fn header_label_names_the_count_and_the_direction_of_the_gesture() {
        let texts = EntityTableSelectionTexts::default();
        assert_eq!(
            texts.page_label(EntityTableDisplayedPageSelection::None, 3),
            "Select all 3 rows on this page"
        );
        assert_eq!(
            texts.page_label(EntityTableDisplayedPageSelection::Partial, 3),
            "Select all 3 rows on this page"
        );
        assert_eq!(
            texts.page_label(EntityTableDisplayedPageSelection::All, 3),
            "Clear the selected rows on this page"
        );
        assert_eq!(
            texts.page_label(EntityTableDisplayedPageSelection::NoRows, 0),
            "No rows are displayed"
        );
    }

    #[test]
    fn row_label_identifies_the_row_never_the_widget() {
        let texts = EntityTableSelectionTexts::default();
        assert_eq!(texts.row_label("Acme Ltd", false), "Select Acme Ltd");
        assert_eq!(texts.row_label("Acme Ltd", true), "Deselect Acme Ltd");
        assert!(!texts.row_label("Acme Ltd", false).contains("checkbox"));
    }

    #[test]
    fn summary_label_reports_total_and_off_page_counts() {
        let texts = EntityTableSelectionTexts::default();
        assert_eq!(texts.summary_label(0, 0), "No rows selected");
        assert_eq!(
            texts.summary_label(5, 2),
            "5 rows selected, 2 of them not on this page"
        );
    }

    #[test]
    fn every_default_string_is_replaceable_for_localization() {
        let custom = EntityTableSelectionTexts {
            column_header: "Elegir".to_owned(),
            select_page: "Elegir {count}".to_owned(),
            clear_page: "Borrar".to_owned(),
            no_rows: "Vacio".to_owned(),
            select_row: "Elegir {row}".to_owned(),
            deselect_row: "Quitar {row}".to_owned(),
            selection_summary: "{total}/{off_page}".to_owned(),
            selection_summary_empty: "Nada".to_owned(),
        };
        assert_eq!(
            custom.page_label(EntityTableDisplayedPageSelection::None, 2),
            "Elegir 2"
        );
        assert_eq!(custom.row_label("Fila", true), "Quitar Fila");
        assert_eq!(custom.summary_label(3, 1), "3/1");
        assert_eq!(custom.summary_label(0, 0), "Nada");
    }

    // ── displayed_row_label ──

    #[test]
    fn a_row_checkbox_announces_the_row_not_the_key_when_text_exists() {
        assert_eq!(
            displayed_row_label("mx-2", Some("Mexico City Client 2")),
            "Mexico City Client 2"
        );
    }

    #[test]
    fn a_blank_or_missing_primary_cell_falls_back_to_the_stable_key() {
        assert_eq!(displayed_row_label("mx-2", None), "mx-2");
        assert_eq!(displayed_row_label("mx-2", Some("   ")), "mx-2");
        assert_eq!(displayed_row_label("mx-2", Some("")), "mx-2");
    }

    // ── track identity ──

    #[test]
    fn the_selection_track_id_is_namespaced_out_of_caller_reach() {
        assert!(SELECTION_COLUMN_TRACK_ID.starts_with("__ldui-"));
        assert_eq!(SELECTION_COLUMN_TRACK_WIDTH, 48);
    }
}
