//! Row selection state machine for [`DataTable`](super::DataTable).
//!
//! Mirrors the semantics of d2d-ui's `data_table::handle_row_click`:
//! plain click selects one row, `Ctrl`+click toggles, `Shift`+click extends
//! a range from the anchor. The function is pure so it's straightforward
//! to test exhaustively and equally usable from non-DOM contexts.

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

#[cfg(test)]
mod tests {
    use super::*;

    fn sel(items: &[usize]) -> BTreeSet<usize> {
        items.iter().copied().collect()
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
}
