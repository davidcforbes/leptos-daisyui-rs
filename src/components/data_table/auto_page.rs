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

/// Identifies "what's being measured" for [`RowHeightEra`]: a
/// `(data_revision, container_width_px, table_size_class)` triple. The
/// caller bumps `data_revision` whenever the underlying rows data changes; a
/// different `container_width_px` (the scroll wrapper's own border-box
/// width, immune to its own vertical scrollbar the same way
/// `rows_per_page_for_height`'s viewport height is) also starts a fresh era.
/// `table_size_class` is the active daisyUI table-size class (e.g.
/// `"table-md"`, from `TableSize::as_str`) -- a density change moves the row
/// height's *ceiling*, not just this pass's reading, so a high-water mark
/// carried over from the previous density would keep the derived page size
/// under-filled until an unrelated data or width change happened to reset it
/// (ldui-wgc3). Two measurement passes with an equal key are considered the
/// same era.
pub type RowHeightEraKey = (u64, i32, &'static str);

/// High-water mark of measured row heights across measurement passes within
/// one era (ldui-89rp).
///
/// `auto_page_size` self-corrects across passes: applying a derived count
/// re-renders the table, which re-triggers a measurement of what actually
/// rendered. Undamped, that loop can fail to converge with ordinary data --
/// a short first render measures a small max and derives a large count; the
/// larger render reveals a tall row further down the page, measures a large
/// max, and derives a small (or `min_rows`-floored) count; the smaller
/// render excludes the tall row again, measures a small max, and derives the
/// large count again -- oscillating between the two forever, since the two
/// measured maxes genuinely differ pass to pass and the caller's
/// change-guard never sees a repeat.
///
/// Ratcheting the row height fed to [`rows_per_page_for_height`] so it can
/// only *increase* within an era breaks the cycle: once any pass sees the
/// tall row, every later pass in the same era uses at least that height,
/// so the derived count can only shrink or hold from that point on -- a
/// monotone (non-increasing) sequence of positive integers, which must reach
/// a fixed point in finitely many passes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowHeightEra {
    key: RowHeightEraKey,
    high_water_mark: f64,
}

impl RowHeightEra {
    /// A fresh era for `key` with nothing measured yet.
    pub const fn empty(key: RowHeightEraKey) -> Self {
        Self {
            key,
            high_water_mark: 0.0,
        }
    }

    /// Feeds one measurement pass's max row height (see [`max_row_height`])
    /// for `key`. A `key` equal to the era's own ratchets the high-water mark
    /// up to `measured_max.max(previous_high_water_mark)`; a different `key`
    /// starts a brand new era at `measured_max`. A non-finite or
    /// non-positive `measured_max` (nothing measurable this pass) never
    /// ratchets anything, and on a `key` change resets the new era's
    /// high-water mark to 0.0 (nothing measured yet) rather than seeding it
    /// with garbage.
    #[must_use]
    pub fn observe(self, key: RowHeightEraKey, measured_max: f64) -> Self {
        let measured_max = if measured_max.is_finite() && measured_max > 0.0 {
            measured_max
        } else {
            0.0
        };
        let high_water_mark = if key == self.key {
            self.high_water_mark.max(measured_max)
        } else {
            measured_max
        };
        Self {
            key,
            high_water_mark,
        }
    }

    /// The row height to feed [`rows_per_page_for_height`] for the *current*
    /// pass: the era's high-water mark, or `fallback` when nothing has been
    /// measured in this era yet (empty table, first paint -- matches
    /// [`FALLBACK_ROW_HEIGHT`]'s existing role).
    pub fn effective_row_height(self, fallback: f64) -> f64 {
        if self.high_water_mark > 0.0 {
            self.high_water_mark
        } else {
            fallback
        }
    }
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

/// Floor for the belt-and-braces overflow-shrink check (ldui-89rp regression
/// caught by `auto_page_size_keeps_a_usable_page_and_scrolls_short_viewports`):
/// normally `min_rows` (never below 1), matching `auto_page_size_for_height`'s
/// own contract that a fit below `min_rows` retains the configured page size
/// and the wrapper scrolls instead of shrinking pagination toward
/// unusability.
///
/// But when the *measured* fit (`measured_rows`, from
/// [`rows_per_page_for_height`]) is itself already below `min_rows`, `rows`
/// -- the value `auto_page_size_for_height` actually derived -- IS that
/// retained configured page size, not a responsively-fitted count. The
/// belt-and-braces check exists to correct a responsive measurement that
/// missed growth; applying it to the retained fallback instead shaved a row
/// off a value the component had already promised never to shrink (a single
/// pass took a documented "retain 10 and scroll" case down to 9). Flooring
/// at `rows` itself in that case makes the check a no-op there, exactly as
/// intended.
///
/// ```
/// use leptos_daisyui_rs::components::overflow_check_floor;
///
/// // A normal responsive fit (measured >= min_rows): floor is min_rows.
/// assert_eq!(overflow_check_floor(12, 5, 12), 5);
///
/// // Below-floor fallback (measured < min_rows): floor is `rows` itself, so
/// // `current > floor` can never fire and shrink the retained fallback.
/// assert_eq!(overflow_check_floor(2, 5, 10), 10);
/// ```
pub fn overflow_check_floor(measured_rows: usize, min_rows: usize, rows: usize) -> usize {
    let min_rows = min_rows.max(1);
    if measured_rows < min_rows {
        rows
    } else {
        min_rows
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

    // ── overflow_check_floor (ldui-89rp) ──

    #[test]
    fn overflow_check_floor_is_min_rows_for_a_responsive_fit() {
        assert_eq!(overflow_check_floor(12, 5, 12), 5);
    }

    #[test]
    fn overflow_check_floor_is_the_retained_rows_below_the_usability_floor() {
        // Production repro: a 128px wrapper measures 2 rows, below
        // min_rows=5, so `auto_page_size_for_height` retains the configured
        // 10. The belt-and-braces check must never touch that 10 -- the
        // floor is 10 itself, not min_rows(5).
        assert_eq!(overflow_check_floor(2, 5, 10), 10);
    }

    #[test]
    fn overflow_check_floor_clamps_min_rows_to_at_least_one() {
        assert_eq!(overflow_check_floor(3, 0, 3), 1);
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

    // ── RowHeightEra (ldui-89rp critical fix: damping the multi-pass loop) ──

    #[test]
    fn row_height_era_of_nothing_measured_yet_falls_back() {
        let era = RowHeightEra::empty((1, 400, "table-md"));
        assert_eq!(
            era.effective_row_height(FALLBACK_ROW_HEIGHT),
            FALLBACK_ROW_HEIGHT
        );
    }

    #[test]
    fn row_height_era_ratchets_up_and_never_decreases_within_an_era() {
        let key = (1_u64, 400_i32, "table-md");
        let era = RowHeightEra::empty(key);

        let era = era.observe(key, 24.0); // pass 1: only short rows rendered
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 24.0);

        let era = era.observe(key, 76.0); // pass 2: a tall row rendered too
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 76.0);

        // pass 3: back to only short rows (the tall row scrolled out of a
        // smaller derived page) -- the high-water mark must NOT decrease.
        let era = era.observe(key, 24.0);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 76.0);
    }

    #[test]
    fn row_height_era_resets_on_a_new_key() {
        let era = RowHeightEra::empty((1, 400, "table-md")).observe((1, 400, "table-md"), 76.0);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 76.0);

        // Dataset changed (key's first component differs) -> fresh era, the
        // old tall reading must not carry over.
        let era = era.observe((2, 400, "table-md"), 24.0);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 24.0);

        // Container width changed (key's second component differs) -> also
        // a fresh era.
        let era = era.observe((2, 320, "table-md"), 60.0);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 60.0);
    }

    #[test]
    fn row_height_era_resets_on_a_density_change() {
        // ldui-wgc3: a `table_size`/density change moves the row-height
        // ceiling itself, so a high-water mark measured under a taller
        // density must not survive into a shorter one -- otherwise the
        // derived page size stays stuck too small (under-filled) until an
        // unrelated data or width change happens to reset it.
        let tall_density = (1_u64, 400_i32, "table-lg");
        let era = RowHeightEra::empty(tall_density).observe(tall_density, 76.0);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 76.0);

        // Same data revision and container width, but a smaller density
        // (key's third component differs) -> a fresh era, the tall-density
        // high-water mark must not carry over.
        let small_density = (1_u64, 400_i32, "table-xs");
        let era = era.observe(small_density, 24.0);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 24.0);
    }

    #[test]
    fn row_height_era_ignores_invalid_measurements_without_resetting() {
        let key = (1_u64, 400_i32, "table-md");
        let era = RowHeightEra::empty(key).observe(key, 76.0);

        let era = era.observe(key, f64::NAN);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 76.0);

        let era = era.observe(key, -5.0);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 76.0);

        let era = era.observe(key, 0.0);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 76.0);

        let era = era.observe(key, f64::INFINITY);
        assert_eq!(era.effective_row_height(FALLBACK_ROW_HEIGHT), 76.0);
    }

    #[test]
    fn row_height_era_a_fresh_era_seeded_by_an_invalid_reading_still_falls_back() {
        // A `key` change resets the high-water mark even when the very first
        // reading in the new era is garbage -- it must not "seed" the new
        // era with a stale non-finite/negative value.
        let era = RowHeightEra::empty((1, 400, "table-md"))
            .observe((1, 400, "table-md"), 76.0)
            .observe((2, 400, "table-md"), f64::NAN);
        assert_eq!(
            era.effective_row_height(FALLBACK_ROW_HEIGHT),
            FALLBACK_ROW_HEIGHT
        );
    }

    #[test]
    fn oscillation_trace_settles_to_a_fixed_point_within_an_era() {
        // Reproduces the reviewer's traced production shape (ldui-89rp
        // CRITICAL fix): 20 rows, one tall row (76px) at absolute index 12
        // (past the default page_size of 10), every other row a uniform
        // 24px. `measured_max_of_first_n` mirrors what
        // `query_selector_all("tbody tr")` would see when the component
        // currently renders the unsorted top `n` rows.
        const TALL_INDEX: usize = 12;
        const SHORT: f64 = 24.0;
        const TALL: f64 = 76.0;
        let row_height_at = |i: usize| if i == TALL_INDEX { TALL } else { SHORT };
        let measured_max_of_first_n =
            |n: usize| -> f64 { (0..n).map(row_height_at).fold(f64::NEG_INFINITY, f64::max) };

        // Chosen so a 24px-row measurement derives 15 (>= min_rows, fully
        // responsive) and a 76px-row measurement derives 4 (< min_rows,
        // falls back to the configured page size) -- the exact numbers from
        // the reviewer's traced cycle.
        let viewport = 410.0;
        let header = 40.0;
        let configured_page_size = 10;
        let min_rows = 5;
        let key = (1_u64, 400_i32, "table-md");

        let mut era = RowHeightEra::empty(key);
        // The very first pass renders whatever `page_size` renders before
        // any measurement exists (the caller's configured fallback).
        let mut rendered_rows = configured_page_size;
        let mut history = Vec::new();
        for _ in 0..6 {
            let measured = measured_max_of_first_n(rendered_rows.min(20));
            era = era.observe(key, measured);
            let effective = era.effective_row_height(FALLBACK_ROW_HEIGHT);
            rendered_rows = auto_page_size_for_height(
                viewport,
                header,
                effective,
                configured_page_size,
                min_rows,
            );
            history.push(rendered_rows);
        }

        // Undamped (pre-fix) this sequence is 15, 10, 15, 10, 15, 10 --
        // forever oscillating, because pass 3's un-ratcheted measurement
        // reverts to the short-only max once the smaller page excludes the
        // tall row again. With the high-water-mark ratchet, once pass 2 ever
        // sees the tall row the era never forgets it, so every pass from
        // then on derives from 76px and the sequence settles.
        assert_eq!(
            history[0], 15,
            "pass 1 sees only short rows and derives the exact-fit count: {history:?}"
        );
        let settled = history[1];
        assert!(
            history[1..].iter().all(|&r| r == settled),
            "expected the derived count to settle to a fixed point after the tall \
             row is first seen, got {history:?}"
        );
        assert_eq!(
            settled, configured_page_size,
            "settles at the min_rows fallback (the documented scroll case), not an \
             oscillation back up to 15: {history:?}"
        );
    }
}
