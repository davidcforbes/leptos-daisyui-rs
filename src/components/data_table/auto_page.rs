//! Pure "how many rows fit" math for [`DataTable`](super::DataTable)'s opt-in
//! responsive paging (`auto_page_size`). Kept separate from `component.rs`'s
//! view code so the arithmetic is unit-testable without a DOM. Mirrors d2d-ui's
//! `rows_per_page_for_height`
//! (`Rust-DeskApp/crates/d2d-ui/src/controls/data_table.rs`), which the desktop
//! table calls from its layout pass; the web port feeds the same arithmetic
//! from a `ResizeObserver`.
//!
//! The desktop uses fixed `HEADER_HEIGHT`/`ROW_HEIGHT` constants because it
//! draws its own rows. On the web a row's height depends on the daisyUI table
//! size (`table-xs` .. `table-lg`), theme, and cell content, so the component
//! *measures* the rendered header and the MAX height across every currently
//! rendered data row (not just the first -- ldui-89rp: a short first row must
//! not derive a count that overflows once a taller row further down the page
//! is accounted for) and passes that in. The constants here are only
//! fallbacks for when there is nothing to measure yet (first paint, or an
//! empty table).

/// Fallback data-row height (px) used when no rendered `<tbody> <tr>` is
/// available to measure (empty table, or the very first paint). Matches
/// d2d-ui's `ROW_HEIGHT` and daisyUI's default `table-md` row.
pub const FALLBACK_ROW_HEIGHT: f64 = 40.0;

/// Fallback header-row height (px) used when `<thead>` cannot be measured.
/// Matches d2d-ui's `HEADER_HEIGHT`.
pub const FALLBACK_HEADER_HEIGHT: f64 = 36.0;

/// Default usability threshold for responsive paging. If fewer rows fit, the
/// configured page size is retained and the table viewport scrolls.
pub const DEFAULT_AUTO_MIN_ROWS: usize = 5;

/// Number of data rows that fit in a scroll viewport of `viewport_height` px,
/// after subtracting the sticky header row. Always at least 1 — a viewport too
/// short for even one row still shows one rather than an empty table.
///
/// `row_height` and `header_height` are the *measured* pixel heights of the
/// rendered `<tbody> <tr>` and `<thead>`. A non-finite or non-positive
/// `row_height` (nothing measurable yet) falls back to [`FALLBACK_ROW_HEIGHT`]
/// rather than dividing by zero; a non-finite or negative `header_height` is
/// treated as 0 (no header).
///
/// Unlike the desktop original this takes no pagination strip: the web
/// component observes the table's scroll wrapper, whose height already excludes
/// the search box and the pager (they are siblings in the flex column, not
/// children of the wrapper).
///
/// ```
/// use leptos_daisyui_rs::components::rows_per_page_for_height;
///
/// // 436px of viewport, a 36px header, 40px rows -> 10 rows fit.
/// assert_eq!(rows_per_page_for_height(436.0, 36.0, 40.0), 10);
/// ```
pub fn rows_per_page_for_height(
    viewport_height: f64,
    header_height: f64,
    row_height: f64,
) -> usize {
    let row_height = if row_height.is_finite() && row_height > 0.0 {
        row_height
    } else {
        FALLBACK_ROW_HEIGHT
    };
    let header_height = if header_height.is_finite() && header_height > 0.0 {
        header_height
    } else {
        0.0
    };

    let available = viewport_height - header_height;
    if !available.is_finite() || available <= 0.0 {
        return 1;
    }

    // `f64 as usize` saturates (never UB / never wraps) since Rust 1.45, so an
    // absurd viewport height clamps to usize::MAX instead of misbehaving.
    ((available / row_height).floor() as usize).max(1)
}

/// Reduces a set of measured `<tbody> <tr>` heights (px) to the single row
/// height [`rows_per_page_for_height`] divides by. Uses the MAX, not the
/// first or an average: with variable-height rows (e.g. a wrapped cell), a
/// short first row and a taller later row are both "the row height" in some
/// sense, but only the max keeps every rendered row inside the derived page
/// from overflowing the wrapper (ldui-89rp). Non-finite/non-positive heights
/// are ignored (unmeasurable or not-yet-laid-out elements), mirroring
/// `rows_per_page_for_height`'s own fallback-on-invalid contract; an empty or
/// all-invalid slice yields `fallback`.
///
/// ```
/// use leptos_daisyui_rs::components::max_row_height;
///
/// // A short first row (24px) and a taller wrapped row (76px) later on the
/// // page: the max is what must drive the division, not the first value.
/// assert_eq!(max_row_height(&[24.0, 76.0, 40.0], 40.0), 76.0);
/// assert_eq!(max_row_height(&[], 40.0), 40.0, "nothing measured -> fallback");
/// ```
pub fn max_row_height(heights: &[f64], fallback: f64) -> f64 {
    heights
        .iter()
        .copied()
        .filter(|h| h.is_finite() && *h > 0.0)
        .fold(None::<f64>, |acc, h| Some(acc.map_or(h, |m| m.max(h))))
        .unwrap_or(fallback)
}

/// Resolves the effective responsive page size for a measured table viewport.
///
/// A measured fit at or above `min_rows` remains fully responsive. Below that
/// usability threshold, the caller's configured page size takes over (never
/// below `min_rows`) and the table's existing scroll wrapper absorbs the
/// overflow. This prevents tall, wrapped rows from collapsing pagination to a
/// one-row page while preserving exact-fit behavior in normal viewports.
pub fn auto_page_size_for_height(
    viewport_height: f64,
    header_height: f64,
    row_height: f64,
    configured_page_size: usize,
    min_rows: usize,
) -> usize {
    let measured = rows_per_page_for_height(viewport_height, header_height, row_height);
    let min_rows = min_rows.max(1);
    if measured < min_rows {
        configured_page_size.max(min_rows)
    } else {
        measured
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_fit_counts_every_row() {
        // 36px header + 10 * 40px rows = 436px.
        assert_eq!(rows_per_page_for_height(436.0, 36.0, 40.0), 10);
    }

    #[test]
    fn partial_row_is_floored_not_rounded() {
        // Room for 10.9 rows -> 10 whole rows; a half-drawn row is not a row.
        assert_eq!(rows_per_page_for_height(472.0, 36.0, 40.0), 10);
    }

    #[test]
    fn taller_viewport_yields_more_rows() {
        let short = rows_per_page_for_height(436.0, 36.0, 40.0);
        let tall = rows_per_page_for_height(836.0, 36.0, 40.0);
        assert!(tall > short, "{tall} should exceed {short}");
        assert_eq!(tall, 20);
    }

    #[test]
    fn smaller_rows_fit_more_of_them() {
        // A `table-xs` row (24px) fits more rows in the same viewport.
        assert_eq!(rows_per_page_for_height(436.0, 36.0, 24.0), 16);
    }

    #[test]
    fn viewport_shorter_than_one_row_still_yields_one() {
        assert_eq!(rows_per_page_for_height(50.0, 36.0, 40.0), 1);
    }

    #[test]
    fn viewport_smaller_than_header_yields_one() {
        assert_eq!(rows_per_page_for_height(20.0, 36.0, 40.0), 1);
    }

    #[test]
    fn zero_viewport_yields_one() {
        // Pre-layout / display:none — never 0, which would render an empty page.
        assert_eq!(rows_per_page_for_height(0.0, 36.0, 40.0), 1);
    }

    #[test]
    fn negative_viewport_yields_one() {
        assert_eq!(rows_per_page_for_height(-100.0, 36.0, 40.0), 1);
    }

    #[test]
    fn zero_row_height_falls_back_instead_of_dividing_by_zero() {
        // Nothing measurable yet: behave as if rows were FALLBACK_ROW_HEIGHT.
        assert_eq!(
            rows_per_page_for_height(436.0, 36.0, 0.0),
            rows_per_page_for_height(436.0, 36.0, FALLBACK_ROW_HEIGHT)
        );
    }

    #[test]
    fn negative_row_height_falls_back() {
        assert_eq!(
            rows_per_page_for_height(436.0, 36.0, -40.0),
            rows_per_page_for_height(436.0, 36.0, FALLBACK_ROW_HEIGHT)
        );
    }

    #[test]
    fn non_finite_row_height_falls_back() {
        assert_eq!(
            rows_per_page_for_height(436.0, 36.0, f64::NAN),
            rows_per_page_for_height(436.0, 36.0, FALLBACK_ROW_HEIGHT)
        );
        assert_eq!(
            rows_per_page_for_height(436.0, 36.0, f64::INFINITY),
            rows_per_page_for_height(436.0, 36.0, FALLBACK_ROW_HEIGHT)
        );
    }

    #[test]
    fn non_finite_viewport_yields_one() {
        assert_eq!(rows_per_page_for_height(f64::NAN, 36.0, 40.0), 1);
    }

    #[test]
    fn headerless_table_uses_the_whole_viewport() {
        assert_eq!(rows_per_page_for_height(400.0, 0.0, 40.0), 10);
    }

    #[test]
    fn negative_header_height_is_treated_as_headerless() {
        assert_eq!(rows_per_page_for_height(400.0, -36.0, 40.0), 10);
    }

    #[test]
    fn absurd_viewport_saturates_rather_than_wrapping() {
        // `f64 as usize` saturates; assert we get a huge count, not 0 or a panic.
        assert!(rows_per_page_for_height(f64::MAX, 36.0, 40.0) > 0);
    }

    #[test]
    fn tall_rows_below_the_minimum_fall_back_to_the_configured_page_size() {
        // Production reproduction from ldui-495: a 224px wrapper, 77px
        // two-row header, and 76px badge rows mathematically fit one row. A
        // one-row pager is unusable, so auto sizing falls back and scrolls.
        assert_eq!(auto_page_size_for_height(224.0, 77.0, 76.0, 10, 5), 10);
    }

    #[test]
    fn configured_fallback_cannot_undercut_the_minimum() {
        assert_eq!(auto_page_size_for_height(224.0, 77.0, 76.0, 3, 5), 5);
    }

    #[test]
    fn a_measured_fit_at_the_minimum_remains_responsive() {
        // 77px header + five 76px rows.
        assert_eq!(auto_page_size_for_height(457.0, 77.0, 76.0, 10, 5), 5);
    }

    // ── max_row_height (ldui-89rp) ──

    #[test]
    fn max_row_height_picks_the_tallest_not_the_first() {
        assert_eq!(
            max_row_height(&[24.0, 76.0, 40.0], FALLBACK_ROW_HEIGHT),
            76.0
        );
    }

    #[test]
    fn max_row_height_of_a_single_reading_is_that_reading() {
        assert_eq!(max_row_height(&[52.0], FALLBACK_ROW_HEIGHT), 52.0);
    }

    #[test]
    fn max_row_height_of_nothing_measured_falls_back() {
        assert_eq!(
            max_row_height(&[], FALLBACK_ROW_HEIGHT),
            FALLBACK_ROW_HEIGHT
        );
    }

    #[test]
    fn max_row_height_ignores_invalid_readings_around_a_valid_max() {
        // 0 (not yet laid out), a negative stray, and NaN must not win, hide,
        // or poison the fold; the one valid reading is still found.
        assert_eq!(
            max_row_height(&[0.0, -5.0, f64::NAN, 48.0], FALLBACK_ROW_HEIGHT),
            48.0
        );
    }

    #[test]
    fn max_row_height_of_all_invalid_readings_falls_back() {
        assert_eq!(
            max_row_height(&[0.0, -5.0, f64::NAN, f64::INFINITY], FALLBACK_ROW_HEIGHT),
            FALLBACK_ROW_HEIGHT
        );
    }

    #[test]
    fn feeding_the_max_row_height_avoids_the_short_first_row_overflow_trap() {
        // ldui-89rp production repro: the first rendered row is a short 24px,
        // but a later row on the same page wraps to 76px. A page size derived
        // from just the first row (or an average) overflows once the tall row
        // renders; feeding rows_per_page_for_height the MAX of the rendered
        // set instead derives a count every one of those rows actually fits
        // in.
        let heights = [24.0, 76.0, 40.0, 24.0];
        let fed_first = heights[0];
        let fed_max = max_row_height(&heights, FALLBACK_ROW_HEIGHT);
        assert_eq!(fed_max, 76.0);

        let viewport = 400.0;
        let header = 36.0;
        let rows_from_first = rows_per_page_for_height(viewport, header, fed_first);
        let rows_from_max = rows_per_page_for_height(viewport, header, fed_max);

        assert!(
            rows_from_max < rows_from_first,
            "the max-fed count ({rows_from_max}) must be strictly smaller than the \
             first-row-fed count ({rows_from_first}) or this scenario doesn't \
             exercise the bug"
        );
        // 400 - 36 = 364px available; 364 / 76 = 4.78 -> 4 whole rows, every
        // one of which is tall enough to actually fit.
        assert_eq!(rows_from_max, 4);
    }
}
