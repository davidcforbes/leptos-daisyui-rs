//! Pure column-resize math for [`DataTable`](super::DataTable)'s draggable
//! header dividers. Kept separate from `header.rs`'s view code so the width
//! computation is unit-testable without a DOM. Mirrors d2d-ui's
//! `set_column_width` / `begin_col_resize` / `drag_col_resize` clamp logic
//! (`Rust-DeskApp/crates/d2d-ui/src/controls/data_table.rs`).

/// Fallback minimum column width (px) when a column doesn't declare its own
/// `min_width`. Mirrors d2d-ui's `RESIZE_MIN_W`.
pub const DEFAULT_MIN_COLUMN_WIDTH: f64 = 48.0;

/// Absolute ceiling on a column's width (px), regardless of drag distance.
/// Mirrors d2d-ui's `RESIZE_MAX_W` (scaled down for typical web layouts).
pub const MAX_COLUMN_WIDTH: f64 = 1200.0;

/// The minimum width to enforce for a column during a resize drag: its own
/// `min_width` if set, else [`DEFAULT_MIN_COLUMN_WIDTH`].
pub fn effective_min_width(column_min_width: Option<u32>) -> f64 {
    column_min_width
        .map(|w| w as f64)
        .unwrap_or(DEFAULT_MIN_COLUMN_WIDTH)
}

/// Compute a column's width mid-drag: `start_width` plus the pointer delta
/// (`current_x - start_x`), clamped to `[min_width, MAX_COLUMN_WIDTH]`.
///
/// `min_width` is clamped to be non-negative and never exceeds
/// [`MAX_COLUMN_WIDTH`] itself, so a caller-supplied `min_width` larger than
/// the max ceiling still produces a valid (non-inverted) range.
pub fn resized_width(start_width: f64, start_x: f64, current_x: f64, min_width: f64) -> f64 {
    let min_width = min_width.clamp(0.0, MAX_COLUMN_WIDTH);
    (start_width + (current_x - start_x)).clamp(min_width, MAX_COLUMN_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── effective_min_width ──

    #[test]
    fn effective_min_width_uses_column_value() {
        assert_eq!(effective_min_width(Some(80)), 80.0);
    }

    #[test]
    fn effective_min_width_falls_back_to_default() {
        assert_eq!(effective_min_width(None), DEFAULT_MIN_COLUMN_WIDTH);
    }

    #[test]
    fn effective_min_width_respects_small_explicit_value() {
        assert_eq!(effective_min_width(Some(10)), 10.0);
    }

    // ── resized_width ──

    #[test]
    fn resized_width_grows_with_positive_delta() {
        assert_eq!(resized_width(100.0, 50.0, 90.0, 48.0), 140.0);
    }

    #[test]
    fn resized_width_shrinks_with_negative_delta() {
        assert_eq!(resized_width(100.0, 90.0, 50.0, 48.0), 60.0);
    }

    #[test]
    fn resized_width_zero_delta_is_unchanged() {
        assert_eq!(resized_width(100.0, 50.0, 50.0, 48.0), 100.0);
    }

    #[test]
    fn resized_width_clamped_to_min() {
        assert_eq!(resized_width(100.0, 90.0, 0.0, 48.0), 48.0);
    }

    #[test]
    fn resized_width_clamped_to_max() {
        assert_eq!(resized_width(100.0, 0.0, 100_000.0, 48.0), MAX_COLUMN_WIDTH);
    }

    #[test]
    fn resized_width_negative_min_is_treated_as_zero() {
        assert_eq!(resized_width(10.0, 90.0, 0.0, -20.0), 0.0);
    }

    #[test]
    fn resized_width_min_above_max_still_produces_valid_range() {
        // A pathological min_width above MAX_COLUMN_WIDTH must not invert the
        // clamp range (which would panic in `f64::clamp`).
        let result = resized_width(100.0, 0.0, 0.0, MAX_COLUMN_WIDTH + 500.0);
        assert_eq!(result, MAX_COLUMN_WIDTH);
    }
}
