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

/// The `aria-selected` attribute value for a row, gated on `has_selection`
/// (whether the table was given a `selection` prop at all) rather than on
/// general row interactivity. An `on_row_activate`-only table (no
/// `selection`) has no selection concept to report, so it must emit `None`
/// -- no `aria-selected` attribute on any row -- rather than
/// `aria-selected="false"`, which would wrongly tell assistive tech the row
/// is selectable. Only a table with `selection` configured emits `Some`
/// on every row, `"true"` or `"false"`.
pub(super) fn entity_row_aria_selected(has_selection: bool, is_selected: bool) -> Option<String> {
    has_selection.then(|| is_selected.to_string())
}

/// `<tr>`-level hover classes for one row (ldui-jdzr): a light-blue
/// semantic hover, reusing the table hierarchy's `--color-table-filter`
/// token rather than a new hardcoded hex, painted only on rows that would
/// otherwise receive a click/keyboard handler -- `interactive` is the same
/// predicate `render_keyed_row` already uses for `tabindex`/
/// `cursor-pointer`, never a second notion of "interactive".
///
/// **Precedence: hover < selected.** The hover utility is present in the
/// class list only while the row is not selected, rather than always
/// present and merely out-ranked. Selection's `bg-base-200` is an
/// unconditional class, not itself a `:hover` rule, so if both classes were
/// always emitted together the `:hover` pseudo-class selector would win the
/// specificity fight over the plain class selector on hover regardless of
/// stylesheet order -- making a hovered, selected row visually read as
/// unselected. Dropping the hover class outright when `selected` is true
/// keeps the selected treatment dominant no matter how Tailwind/daisyUI
/// order their generated rules.
pub(super) const fn entity_row_hover_class(interactive: bool, selected: bool) -> &'static str {
    if interactive && !selected {
        "hover:bg-table-filter forced-colors:hover:bg-[Highlight] forced-colors:hover:text-[HighlightText]"
    } else {
        ""
    }
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

    // ── entity_row_aria_selected ──

    #[test]
    fn an_activate_only_table_emits_no_aria_selected_attribute() {
        // No `selection` configured (whether or not `on_row_activate` is):
        // no `aria-selected` attribute at all, on any row -- restoring the
        // exact DOM of a table that predates this prop.
        assert_eq!(entity_row_aria_selected(false, false), None);
        assert_eq!(entity_row_aria_selected(false, true), None);
    }

    #[test]
    fn a_selection_configured_table_emits_aria_selected_on_every_row() {
        assert_eq!(
            entity_row_aria_selected(true, true),
            Some("true".to_owned())
        );
        assert_eq!(
            entity_row_aria_selected(true, false),
            Some("false".to_owned())
        );
    }

    // ── entity_row_hover_class ──

    #[test]
    fn interactive_unselected_row_carries_the_light_blue_hover() {
        let class = entity_row_hover_class(true, false);
        assert!(class.contains("hover:bg-table-filter"));
        assert!(!class.contains("bg-base-200"));
    }

    #[test]
    fn interactive_selected_row_carries_no_hover_class() {
        // Precedence: hover < selected. The hover utility is dropped
        // outright, not merely out-ranked, so a `:hover` pseudo-class
        // selector (higher specificity than a plain class selector) can
        // never win over the selected row's `bg-base-200` on hover.
        assert_eq!(entity_row_hover_class(true, true), "");
    }

    #[test]
    fn non_interactive_row_carries_no_hover_class_regardless_of_selection() {
        assert_eq!(entity_row_hover_class(false, false), "");
        assert_eq!(entity_row_hover_class(false, true), "");
    }

    #[test]
    fn hover_class_sets_no_background_outside_the_hover_pseudo_class() {
        // The hover-only utility must never leak a static background --
        // only `hover:` (and its forced-colors compound) may appear.
        let class = entity_row_hover_class(true, false);
        assert!(!class.split_whitespace().any(|token| {
            (token.starts_with("bg-") || token.contains(":bg-")) && !token.contains("hover:")
        }));
    }

    #[test]
    fn forced_colors_hover_uses_the_system_highlight_pair() {
        let class = entity_row_hover_class(true, false);
        assert!(class.contains("forced-colors:hover:bg-[Highlight]"));
        assert!(class.contains("forced-colors:hover:text-[HighlightText]"));
    }
}
