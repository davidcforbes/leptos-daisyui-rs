//! Row selection state machine for [`DataTable`](super::DataTable).
//!
//! Mirrors the semantics of d2d-ui's `data_table::handle_row_click`:
//! plain click selects one row, `Ctrl`+click toggles, `Shift`+click extends
//! a range from the anchor. The function is pure so it's straightforward
//! to test exhaustively and equally usable from non-DOM contexts.

use crate::components::data_table::types::TableRow;
use std::collections::BTreeSet;

/// Apply a row click to a selection state.
///
/// `row_idx` is the **absolute** index into the underlying data (not the
/// visible page). `total` is the row count used to clamp range selections
/// when the anchor sits outside the current data window after a refresh.
///
/// Behaviour:
///
/// - **Plain** (`!ctrl && !shift`) — clear selection, select `row_idx`, set anchor to `row_idx`.
/// - **Ctrl** (`ctrl && !shift`) — toggle membership of `row_idx`, set anchor to `row_idx`.
/// - **Shift** (`!ctrl && shift`) — clear selection, then add the inclusive range from the anchor (or `row_idx` if no anchor) up to `row_idx`. The anchor is unchanged.
/// - **Ctrl+Shift** — keep existing selection and add the inclusive range from the anchor (or `row_idx`) up to `row_idx`. The anchor is unchanged.
///
/// All inserted indices are clamped to `0..total`. If `total == 0` the
/// call is a no-op.
pub fn handle_row_click(
    row_idx: usize,
    ctrl: bool,
    shift: bool,
    selected: &mut BTreeSet<usize>,
    anchor: &mut Option<usize>,
    total: usize,
) {
    if total == 0 {
        return;
    }
    let row_idx = row_idx.min(total - 1);

    match (ctrl, shift) {
        (false, false) => {
            selected.clear();
            selected.insert(row_idx);
            *anchor = Some(row_idx);
        }
        (true, false) => {
            if !selected.remove(&row_idx) {
                selected.insert(row_idx);
            }
            *anchor = Some(row_idx);
        }
        (ctrl_held, true) => {
            let anchor_idx = anchor.unwrap_or(row_idx).min(total - 1);
            if !ctrl_held {
                selected.clear();
            }
            let (lo, hi) = if anchor_idx <= row_idx {
                (anchor_idx, row_idx)
            } else {
                (row_idx, anchor_idx)
            };
            for i in lo..=hi {
                selected.insert(i);
            }
            // Anchor is unchanged on Shift / Ctrl+Shift.
        }
    }
}

/// Whether a row click should notify the consumer's `on_row_activate`
/// callback (a plain click, when the consumer opted in) or feed the
/// internal selection state machine (`handle_row_click`) instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowClickKind {
    /// Plain click (no Ctrl/Shift) with an `on_row_activate` callback set --
    /// the consumer wants to navigate/act on the row, not select it.
    Activate,
    /// Modified click (Ctrl and/or Shift), or no `on_row_activate` callback
    /// registered at all -- feed the existing selection semantics.
    Select,
}

/// The row keys of the selected indices, via the consumer's `row_key`
/// function. Indices no longer present in `data` contribute nothing.
///
/// Together with [`remap_selection`] this is what makes a keyed selection
/// survive data replacement: capture keys at click time, remap them onto
/// whatever rows the next data set holds.
pub fn selection_keys(
    data: &[TableRow],
    selected: &BTreeSet<usize>,
    key_of: impl Fn(&TableRow) -> String,
) -> BTreeSet<String> {
    selected
        .iter()
        .filter_map(|&i| data.get(i).map(&key_of))
        .collect()
}

/// The indices in `data` whose row key is in `keys` -- a stored key-set
/// selection re-expressed against a new data vec. Keys with no matching row
/// (e.g. a claimed row removed from a live pool) simply select nothing, and
/// re-select if their row returns later.
pub fn remap_selection(
    data: &[TableRow],
    keys: &BTreeSet<String>,
    key_of: impl Fn(&TableRow) -> String,
) -> BTreeSet<usize> {
    data.iter()
        .enumerate()
        .filter(|(_, row)| keys.contains(&key_of(row)))
        .map(|(i, _)| i)
        .collect()
}

/// The index in `data` of the row with key `key`, if any -- used to remap the
/// selection anchor across a data replacement.
pub fn index_of_key(
    data: &[TableRow],
    key: &str,
    key_of: impl Fn(&TableRow) -> String,
) -> Option<usize> {
    data.iter().position(|row| key_of(row) == key)
}

/// Whether a table's rows should be keyboard-operable -- focusable
/// (`tabindex=0`) with Enter/Space mirroring a click.
///
/// True exactly when the consumer opted into row interaction: either a
/// `selected_rows` signal (`has_selection`) or an `on_row_activate` callback
/// (`has_activate`) was supplied. A plain display table with neither gains no
/// tab stops -- a 50-row page would otherwise add 50 tab stops to a table the
/// user only reads. `aria-selected` is a narrower gate than this one -- see
/// [`row_aria_selected`].
pub fn row_is_interactive(has_selection: bool, has_activate: bool) -> bool {
    has_selection || has_activate
}

/// The `aria-selected` attribute value for a row, gated on `has_selection`
/// (whether the table was given a selection binding at all) rather than on
/// general row interactivity (`row_is_interactive`). An `on_row_activate`-only
/// table (no selection concept) has nothing to report, so it must emit `None`
/// -- no `aria-selected` attribute on any row -- rather than
/// `aria-selected="false"`, which would wrongly tell assistive tech the row is
/// selectable. Only a table with selection configured emits `Some` on every
/// row, `"true"` or `"false"`. Mirrors `entity_table::selection`'s
/// `entity_row_aria_selected` (ldui-sh3 / ldui-cyhz).
pub fn row_aria_selected(has_selection: bool, is_selected: bool) -> Option<String> {
    has_selection.then(|| is_selected.to_string())
}

/// Decide whether a row click should activate or select.
///
/// `has_activate` reports whether the consumer passed an `on_row_activate`
/// callback to `DataTable`. When it's absent, every click selects, exactly
/// as before this callback existed -- opt-in, zero behavior change by
/// default.
pub fn row_click_kind(ctrl: bool, shift: bool, has_activate: bool) -> RowClickKind {
    if has_activate && !ctrl && !shift {
        RowClickKind::Activate
    } else {
        RowClickKind::Select
    }
}

/// Whether a click event should be swallowed because it is a repeat click of
/// a double-click aimed at `on_row_inspect` (`ldui-tmr`).
///
/// A double-click fires `click` (detail 1), `click` (detail 2), `dblclick` --
/// so with an inspect callback wired, letting every click through would run
/// `on_row_activate` twice before the inspector opens. Swallowing clicks
/// with `detail > 1` keeps activation to exactly one call per double-click
/// (the first click still activates: standard dblclick discrimination is
/// acceptable because single-click navigation is non-destructive). Without
/// an inspect callback nothing changes.
pub fn click_swallowed_by_inspect(detail: i32, has_inspect: bool) -> bool {
    has_inspect && detail > 1
}

/// Whether a keydown should fire `on_row_inspect`: Shift+Enter with an
/// inspect callback wired (`ldui-tmr`). Ctrl+Shift+Enter stays with the
/// selection state machine (range select), as does Shift+Space, so the
/// keyboard selection ergonomics survive an inspect-enabled table.
pub fn key_inspects(key: &str, ctrl: bool, shift: bool, has_inspect: bool) -> bool {
    has_inspect && shift && !ctrl && key == "Enter"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sel(items: &[usize]) -> BTreeSet<usize> {
        items.iter().copied().collect()
    }

    // ── row_click_kind ──

    #[test]
    fn plain_click_activates_when_callback_set() {
        assert_eq!(row_click_kind(false, false, true), RowClickKind::Activate);
    }

    #[test]
    fn ctrl_click_selects_even_with_callback_set() {
        assert_eq!(row_click_kind(true, false, true), RowClickKind::Select);
    }

    #[test]
    fn shift_click_selects_even_with_callback_set() {
        assert_eq!(row_click_kind(false, true, true), RowClickKind::Select);
    }

    #[test]
    fn ctrl_shift_click_selects_even_with_callback_set() {
        assert_eq!(row_click_kind(true, true, true), RowClickKind::Select);
    }

    #[test]
    fn plain_click_selects_when_no_callback_set() {
        // No `on_row_activate` registered -- behavior is unchanged from
        // before this callback existed: every click feeds selection.
        assert_eq!(row_click_kind(false, false, false), RowClickKind::Select);
    }

    // ── click_swallowed_by_inspect / key_inspects (ldui-tmr) ──

    #[test]
    fn repeat_click_of_a_double_click_is_swallowed_only_with_inspect_wired() {
        assert!(click_swallowed_by_inspect(2, true));
        assert!(click_swallowed_by_inspect(3, true));
        // Without an inspect callback, behavior is unchanged: every click
        // (whatever its detail) feeds activate/select as before.
        assert!(!click_swallowed_by_inspect(2, false));
        assert!(!click_swallowed_by_inspect(1, true));
        assert!(!click_swallowed_by_inspect(1, false));
    }

    #[test]
    fn shift_enter_inspects_only_with_inspect_wired() {
        assert!(key_inspects("Enter", false, true, true));
        assert!(!key_inspects("Enter", false, true, false));
    }

    #[test]
    fn other_keys_and_modifier_combos_do_not_inspect() {
        // Plain Enter activates; Shift+Space range-selects; Ctrl+Shift+Enter
        // stays with the selection state machine.
        assert!(!key_inspects("Enter", false, false, true));
        assert!(!key_inspects(" ", false, true, true));
        assert!(!key_inspects("Enter", true, true, true));
    }

    // ── row_is_interactive ──

    #[test]
    fn interactive_when_selection_surfaced() {
        assert!(row_is_interactive(true, false));
    }

    #[test]
    fn interactive_when_activation_set() {
        assert!(row_is_interactive(false, true));
    }

    #[test]
    fn interactive_when_both() {
        assert!(row_is_interactive(true, true));
    }

    #[test]
    fn not_interactive_for_a_plain_display_table() {
        // Neither selection nor activation -> no tab stops added.
        assert!(!row_is_interactive(false, false));
    }

    // ── row_aria_selected (ldui-cyhz) ──

    #[test]
    fn an_activate_only_table_emits_no_aria_selected_attribute() {
        // `on_row_activate` alone makes rows interactive (keyboard-operable),
        // but there is no selection concept to report -- no `aria-selected`
        // attribute at all, on any row, restoring the exact DOM of a table
        // that predates this prop.
        assert_eq!(row_aria_selected(false, false), None);
        assert_eq!(row_aria_selected(false, true), None);
    }

    #[test]
    fn a_selection_configured_table_emits_aria_selected_on_every_row() {
        assert_eq!(row_aria_selected(true, true), Some("true".to_owned()));
        assert_eq!(row_aria_selected(true, false), Some("false".to_owned()));
    }

    // ── plain click ──

    #[test]
    fn plain_click_on_empty_selects_only_clicked() {
        let mut s = BTreeSet::new();
        let mut a = None;
        handle_row_click(3, false, false, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[3]));
        assert_eq!(a, Some(3));
    }

    #[test]
    fn plain_click_clears_prior_selection() {
        let mut s = sel(&[0, 1, 2]);
        let mut a = Some(2);
        handle_row_click(5, false, false, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[5]));
        assert_eq!(a, Some(5));
    }

    // ── Ctrl click ──

    #[test]
    fn ctrl_click_adds_to_selection() {
        let mut s = sel(&[1]);
        let mut a = Some(1);
        handle_row_click(4, true, false, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[1, 4]));
        assert_eq!(a, Some(4));
    }

    #[test]
    fn ctrl_click_on_selected_row_toggles_off() {
        let mut s = sel(&[1, 4]);
        let mut a = Some(1);
        handle_row_click(4, true, false, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[1]));
        assert_eq!(a, Some(4));
    }

    #[test]
    fn ctrl_click_moves_anchor() {
        let mut s = sel(&[1]);
        let mut a = Some(1);
        handle_row_click(7, true, false, &mut s, &mut a, 10);
        assert_eq!(a, Some(7));
    }

    // ── Shift click ──

    #[test]
    fn shift_click_with_anchor_extends_forward() {
        let mut s = sel(&[2]);
        let mut a = Some(2);
        handle_row_click(5, false, true, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[2, 3, 4, 5]));
        assert_eq!(a, Some(2), "anchor must not move on Shift+click");
    }

    #[test]
    fn shift_click_with_anchor_extends_backward() {
        let mut s = sel(&[5]);
        let mut a = Some(5);
        handle_row_click(2, false, true, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[2, 3, 4, 5]));
        assert_eq!(a, Some(5));
    }

    #[test]
    fn shift_click_without_anchor_behaves_like_plain_click() {
        let mut s = sel(&[1, 2]);
        let mut a = None;
        handle_row_click(7, false, true, &mut s, &mut a, 10);
        // With no anchor we range-select from row to itself, effectively selecting one row.
        assert_eq!(s, sel(&[7]));
        assert_eq!(a, None, "Shift without anchor must not set an anchor");
    }

    #[test]
    fn shift_click_clears_prior_unrelated_selection() {
        let mut s = sel(&[0, 8, 9]);
        let mut a = Some(2);
        handle_row_click(5, false, true, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[2, 3, 4, 5]));
    }

    // ── Ctrl+Shift click ──

    #[test]
    fn ctrl_shift_extends_range_without_clearing() {
        let mut s = sel(&[0, 8]);
        let mut a = Some(2);
        handle_row_click(5, true, true, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[0, 2, 3, 4, 5, 8]));
        assert_eq!(a, Some(2), "anchor must not move on Ctrl+Shift");
    }

    // ── clamping ──

    #[test]
    fn out_of_bounds_row_is_clamped_to_last_row() {
        let mut s = BTreeSet::new();
        let mut a = None;
        handle_row_click(999, false, false, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[9]));
        assert_eq!(a, Some(9));
    }

    #[test]
    fn shift_range_with_stale_anchor_is_clamped() {
        let mut s = BTreeSet::new();
        let mut a = Some(50);
        handle_row_click(3, false, true, &mut s, &mut a, 10);
        // anchor 50 clamps to 9, so range becomes 3..=9
        assert_eq!(s, sel(&[3, 4, 5, 6, 7, 8, 9]));
    }

    #[test]
    fn empty_data_is_noop() {
        let mut s = sel(&[1, 2]);
        let mut a = Some(1);
        handle_row_click(0, false, false, &mut s, &mut a, 0);
        assert_eq!(s, sel(&[1, 2]), "no-op when total==0");
        assert_eq!(a, Some(1));
    }

    // ── anchor semantics ──

    #[test]
    fn plain_click_after_ctrl_resets_to_single_selection() {
        let mut s = sel(&[1, 4]);
        let mut a = Some(4);
        handle_row_click(7, false, false, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[7]));
        assert_eq!(a, Some(7));
    }

    #[test]
    fn shift_followed_by_plain_resets_selection() {
        let mut s = BTreeSet::new();
        let mut a = Some(2);
        handle_row_click(5, false, true, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[2, 3, 4, 5]));

        handle_row_click(8, false, false, &mut s, &mut a, 10);
        assert_eq!(s, sel(&[8]));
        assert_eq!(a, Some(8));
    }

    // ── keyed identity (selection_keys / remap_selection / index_of_key) ──

    fn rows(ids: &[&str]) -> Vec<TableRow> {
        ids.iter()
            .map(|id| TableRow::from([("id", id.to_string())]))
            .collect()
    }

    fn key_of(row: &TableRow) -> String {
        row.get("id").cloned().unwrap_or_default()
    }

    #[test]
    fn selection_keys_captures_keys_of_selected_indices() {
        let data = rows(&["a", "b", "c"]);
        let keys = selection_keys(&data, &sel(&[0, 2]), key_of);
        assert_eq!(keys, BTreeSet::from(["a".to_string(), "c".to_string()]));
    }

    #[test]
    fn selection_keys_ignores_out_of_range_indices() {
        let data = rows(&["a"]);
        let keys = selection_keys(&data, &sel(&[0, 9]), key_of);
        assert_eq!(keys, BTreeSet::from(["a".to_string()]));
    }

    #[test]
    fn remap_selection_survives_reordering() {
        // The row the user selected moves position; its selection follows it.
        let keys = BTreeSet::from(["b".to_string()]);
        let reordered = rows(&["c", "b", "a"]);
        assert_eq!(remap_selection(&reordered, &keys, key_of), sel(&[1]));
    }

    #[test]
    fn remap_selection_drops_keys_of_removed_rows() {
        // A live pool removed the selected row (e.g. it was claimed): the
        // selection maps to nothing rather than sliding onto a neighbour.
        let keys = BTreeSet::from(["b".to_string()]);
        let without_b = rows(&["a", "c"]);
        assert!(remap_selection(&without_b, &keys, key_of).is_empty());
    }

    #[test]
    fn remap_selection_reselects_a_returning_row() {
        let keys = BTreeSet::from(["b".to_string()]);
        let with_b_back = rows(&["a", "b", "c"]);
        assert_eq!(remap_selection(&with_b_back, &keys, key_of), sel(&[1]));
    }

    #[test]
    fn index_of_key_finds_moved_row() {
        let data = rows(&["x", "y", "z"]);
        assert_eq!(index_of_key(&data, "z", key_of), Some(2));
        assert_eq!(index_of_key(&data, "missing", key_of), None);
    }
}
