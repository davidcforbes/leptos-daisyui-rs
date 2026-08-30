//! Controlled single-row selection for [`EntityTable`](super::EntityTable).
//!
//! Mirrors ldui-4lp's `ServerTableSelection` proposal-first shape (see
//! `crate::components::data_table::server_component`): the caller supplies
//! the accepted stable row key and receives one replacement proposal per
//! plain click or keyboard Enter/Space. `EntityTable` never optimistically
//! paints a proposed key -- a rejected or delayed proposal leaves
//! `aria-selected` and styling aligned with whatever key the caller
//! currently supplies through `selected_key`. Unlike `ServerDataTable`,
//! `EntityTable`'s `row_key` is mandatory, so there is no fail-closed
//! "selection without row identity" configuration error to represent here.

use leptos::prelude::*;

/// Controlled single-row selection for [`EntityTable`](super::EntityTable).
///
/// The supplied stable key (from the table's mandatory `row_key`) is always
/// displayed truth. A plain click or keyboard Enter/Space on a row emits one
/// proposed replacement key without changing what is rendered; the caller
/// decides whether to accept it by writing to the signal backing
/// `selected_key`. Ctrl/Meta/Shift gestures never propose -- this is
/// deliberately not multi-select, matching `ServerTableSelection`.
#[derive(Clone, Copy)]
pub struct EntityTableSelection {
    selected_key: Signal<Option<String>>,
    on_change: Callback<Option<String>>,
}

impl EntityTableSelection {
    /// Creates controlled single-selection ownership.
    pub fn controlled(
        selected_key: Signal<Option<String>>,
        on_change: Callback<Option<String>>,
    ) -> Self {
        Self {
            selected_key,
            on_change,
        }
    }

    /// Returns the caller-owned accepted-selection signal.
    pub fn selected_key(self) -> Signal<Option<String>> {
        self.selected_key
    }

    /// Emits a proposed replacement key to the caller's callback. Internal
    /// to `entity_table` -- callers only ever read `selected_key`; they never
    /// call this directly.
    pub(super) fn propose(self, key: String) {
        self.on_change.run(Some(key));
    }
}

/// The proposed replacement key for a plain click or Enter/Space on `key`,
/// or `None` when a modifier gesture (Ctrl/Meta/Shift) is held. Selection is
/// deliberately single-select, so a modified click or keypress is swallowed
/// rather than folded into a range or toggle -- the caller's existing
/// `on_row_activate` still receives an unmodified click/keypress as before.
pub(super) fn entity_selection_proposal(key: &str, ctrl: bool, shift: bool) -> Option<String> {
    (!ctrl && !shift).then(|| key.to_owned())
}

/// Whether the row rendered under `rendered_key` should carry the accepted
/// selection state -- exact equality against the caller's `selected_key`.
/// A selected key with no matching row on the current page (removed,
/// filtered out, sorted to another page, or from a replaced dataset) simply
/// matches nothing; `EntityTable` never falls back to selecting by position,
/// so no row renders as selected until the caller's key is visible again.
pub(super) fn entity_row_is_selected(rendered_key: &str, selected_key: Option<&str>) -> bool {
    selected_key == Some(rendered_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── entity_selection_proposal ──

    #[test]
    fn plain_gesture_proposes_the_row_key() {
        assert_eq!(
            entity_selection_proposal("matter-1", false, false),
            Some("matter-1".to_owned())
        );
    }

    #[test]
    fn ctrl_gesture_does_not_propose() {
        assert_eq!(entity_selection_proposal("matter-1", true, false), None);
    }

    #[test]
    fn shift_gesture_does_not_propose() {
        assert_eq!(entity_selection_proposal("matter-1", false, true), None);
    }

    #[test]
    fn ctrl_shift_gesture_does_not_propose() {
        assert_eq!(entity_selection_proposal("matter-1", true, true), None);
    }

    // ── entity_row_is_selected ──

    #[test]
    fn matching_key_is_selected() {
        assert!(entity_row_is_selected("matter-1", Some("matter-1")));
    }

    #[test]
    fn non_matching_key_is_not_selected() {
        assert!(!entity_row_is_selected("matter-1", Some("matter-2")));
    }

    #[test]
    fn no_selected_key_selects_nothing() {
        assert!(!entity_row_is_selected("matter-1", None));
    }

    #[test]
    fn a_selected_key_absent_from_the_visible_page_selects_no_row() {
        // Fail-safe: the caller's accepted key belongs to a row that is not
        // on this page (paging/sorting/filtering moved it, or it was
        // removed). Every rendered row's equality check simply fails --
        // there is no positional fallback that could alias a different row.
        let page_keys = ["matter-3", "matter-4", "matter-5"];
        assert!(
            page_keys
                .iter()
                .all(|key| !entity_row_is_selected(key, Some("matter-1")))
        );
    }
}
