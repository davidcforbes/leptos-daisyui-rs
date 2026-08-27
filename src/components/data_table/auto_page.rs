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
//! *measures* the rendered header and first data row and passes them in. The
//! constants here are only fallbacks for when there is nothing to measure yet
//! (first paint, or an empty table).

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
}
