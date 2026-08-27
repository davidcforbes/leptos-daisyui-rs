//! Pure pagination-window and row-range logic for [`DataTableControls`].
//!
//! Kept free of any Leptos/view dependencies so it can be unit tested
//! headlessly with `cargo test --lib`.
//!
//! [`DataTableControls`]: crate::components::data_table::controls::DataTableControls

use std::ops::Range;

/// Returns the total number of pages, treating a zero page size as one.
pub fn page_count(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        0
    } else {
        total_items.div_ceil(page_size.max(1))
    }
}

/// Clamps a zero-based page index to the last available page.
pub fn clamp_page(current_page: usize, page_size: usize, total_items: usize) -> usize {
    page_count(total_items, page_size)
        .saturating_sub(1)
        .min(current_page)
}

/// Returns the zero-based source-index range for the available page.
pub fn page_bounds(current_page: usize, page_size: usize, total_items: usize) -> Range<usize> {
    if total_items == 0 {
        return 0..0;
    }
    let page_size = page_size.max(1);
    let page = clamp_page(current_page, page_size, total_items);
    let start = page.saturating_mul(page_size);
    start..(start + page_size).min(total_items)
}

/// A single slot in a rendered page-number strip: either a clickable page
/// number, or an ellipsis standing in for a run of skipped pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageSlot {
    /// A clickable page, 0-based (matches `DataTable`'s internal `current_page`).
    Page(usize),
    /// A non-clickable "..." gap between shown pages.
    Ellipsis,
}

/// Compute which page slots to render for a pagination strip, windowing
/// around the current page with ellipses for skipped runs.
///
/// `current` and the returned `Page` indices are 0-based. `total` is the
/// total number of pages. `max_visible` is the number of page slots (not
/// counting ellipses) below which every page is shown with no windowing.
///
/// Behavior (mirrors d2d-ui: `MAX_VISIBLE` ~7, radius 2):
/// - `total == 0` -> empty (nothing to page through).
/// - `total <= max_visible` -> all pages `0..total`, no ellipsis.
/// - Otherwise: always show the first and last page, plus `current` and up
///   to 2 pages on each side; gaps of more than one skipped page become a
///   single `Ellipsis` (never adjacent/duplicate ellipses).
pub fn page_window(current: usize, total: usize, max_visible: usize) -> Vec<PageSlot> {
    if total == 0 {
        return Vec::new();
    }
    if total <= max_visible {
        return (0..total).map(PageSlot::Page).collect();
    }

    const RADIUS: usize = 2;
    let last = total - 1;

    let mut pages: Vec<usize> = Vec::with_capacity(max_visible + 2);
    pages.push(0);
    pages.push(last);

    let lo = current.saturating_sub(RADIUS);
    let hi = (current + RADIUS).min(last);
    for p in lo..=hi {
        pages.push(p);
    }

    pages.sort_unstable();
    pages.dedup();

    let mut result = Vec::with_capacity(pages.len() + 2);
    let mut prev: Option<usize> = None;
    for p in pages {
        if let Some(prev_p) = prev
            && p > prev_p + 1
        {
            result.push(PageSlot::Ellipsis);
        }
        result.push(PageSlot::Page(p));
        prev = Some(p);
    }
    result
}

/// Compute the 1-based `(start, end)` row range for the row-range caption
/// ("Showing {start}-{end} of {total}").
///
/// `current_page` is 0-based. `end` is clamped to `total_items`. Returns
/// `(0, 0)` when `total_items == 0` or when `current_page` is past the last
/// page (nothing to show).
pub fn row_range(current_page: usize, page_size: usize, total_items: usize) -> (usize, usize) {
    if total_items == 0 {
        return (0, 0);
    }
    let page_size = page_size.max(1);
    let start = current_page.saturating_mul(page_size);
    if start >= total_items {
        return (0, 0);
    }
    let end = (start + page_size).min(total_items);
    (start + 1, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: flatten a `Vec<PageSlot>` into 0-based page numbers, using
    /// `-1` for `Ellipsis`, for terser assertions.
    fn to_nums(slots: &[PageSlot]) -> Vec<i64> {
        slots
            .iter()
            .map(|s| match s {
                PageSlot::Page(n) => *n as i64,
                PageSlot::Ellipsis => -1,
            })
            .collect()
    }

    #[test]
    fn shared_page_state_clamps_and_slices_the_last_partial_page() {
        assert_eq!(page_count(74, 25), 3);
        assert_eq!(clamp_page(8, 25, 74), 2);
        assert_eq!(page_bounds(8, 25, 74), 50..74);
        assert_eq!(page_bounds(0, 25, 0), 0..0);
    }

    // ── page_window: total <= max_visible (no ellipsis) ──

    #[test]
    fn zero_total_pages_is_empty() {
        assert_eq!(page_window(0, 0, 7), Vec::new());
    }

    #[test]
    fn one_total_page() {
        assert_eq!(to_nums(&page_window(0, 1, 7)), vec![0]);
    }

    #[test]
    fn total_equal_to_max_visible_shows_all_no_ellipsis() {
        assert_eq!(to_nums(&page_window(3, 7, 7)), vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn total_less_than_max_visible_shows_all_no_ellipsis() {
        assert_eq!(to_nums(&page_window(0, 4, 7)), vec![0, 1, 2, 3]);
        assert_eq!(to_nums(&page_window(3, 4, 7)), vec![0, 1, 2, 3]);
    }

    // ── page_window: total > max_visible (windowing + ellipsis) ──

    #[test]
    fn current_near_start_no_left_ellipsis() {
        // current=0, radius 2 -> [0,1,2], plus first(0)/last(19) already covered for 0.
        // pages: [0,1,2,19] -> [0,1,2,...,19]
        assert_eq!(to_nums(&page_window(0, 20, 7)), vec![0, 1, 2, -1, 19]);
    }

    #[test]
    fn current_near_end_no_right_ellipsis() {
        // current=19 (last), radius 2 -> [17,18,19]
        // pages: [0,17,18,19] -> [0,...,17,18,19]
        assert_eq!(to_nums(&page_window(19, 20, 7)), vec![0, -1, 17, 18, 19]);
    }

    #[test]
    fn current_in_middle_has_both_ellipses() {
        // current=10, radius 2 -> [8,9,10,11,12]; first=0, last=19
        // pages: [0,8,9,10,11,12,19] -> [0,...,8,9,10,11,12,...,19]
        assert_eq!(
            to_nums(&page_window(10, 20, 7)),
            vec![0, -1, 8, 9, 10, 11, 12, -1, 19]
        );
    }

    #[test]
    fn no_duplicate_or_adjacent_ellipsis_when_gap_is_exactly_one() {
        // total=8 (>max_visible=7), current=2, radius 2 -> [0,1,2,3,4]; first=0 already
        // included, last=7. pages: [0,1,2,3,4,7] -> gap between 4 and 7 is 3 (>1) so one
        // ellipsis; no ellipsis between 0 and 1 (adjacent).
        assert_eq!(to_nums(&page_window(2, 8, 7)), vec![0, 1, 2, 3, 4, -1, 7]);
    }

    #[test]
    fn current_at_zero_with_large_total() {
        assert_eq!(to_nums(&page_window(0, 100, 7)), vec![0, 1, 2, -1, 99]);
    }

    #[test]
    fn current_at_last_with_large_total() {
        assert_eq!(to_nums(&page_window(99, 100, 7)), vec![0, -1, 97, 98, 99]);
    }

    #[test]
    fn page_window_never_emits_adjacent_duplicate_ellipsis() {
        for total in 8..=30 {
            for current in 0..total {
                let slots = page_window(current, total, 7);
                for pair in slots.windows(2) {
                    assert!(
                        !(pair[0] == PageSlot::Ellipsis && pair[1] == PageSlot::Ellipsis),
                        "adjacent ellipses for total={total} current={current}"
                    );
                }
            }
        }
    }

    // ── row_range ──

    #[test]
    fn row_range_empty_when_total_items_zero() {
        assert_eq!(row_range(0, 10, 0), (0, 0));
    }

    #[test]
    fn row_range_first_page_full() {
        assert_eq!(row_range(0, 10, 25), (1, 10));
    }

    #[test]
    fn row_range_middle_page_full() {
        assert_eq!(row_range(1, 10, 25), (11, 20));
    }

    #[test]
    fn row_range_last_partial_page() {
        assert_eq!(row_range(2, 10, 25), (21, 25));
    }

    #[test]
    fn row_range_last_page_exact_multiple() {
        assert_eq!(row_range(1, 10, 20), (11, 20));
    }

    #[test]
    fn row_range_single_row_total() {
        assert_eq!(row_range(0, 10, 1), (1, 1));
    }

    #[test]
    fn row_range_page_size_zero_is_treated_as_one() {
        assert_eq!(row_range(0, 0, 5), (1, 1));
    }

    #[test]
    fn row_range_current_page_past_end_is_empty() {
        // Only 3 pages exist (0,1,2) for 25 items at page_size 10; page 5 is out of range.
        assert_eq!(row_range(5, 10, 25), (0, 0));
    }
}
