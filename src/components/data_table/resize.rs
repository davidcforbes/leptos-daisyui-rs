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

/// Width adjustment made by one keyboard arrow press, in CSS pixels.
pub const KEYBOARD_RESIZE_STEP: f64 = 16.0;

/// The minimum width to enforce for a column during resizing: its own
/// `min_width` if set, else [`DEFAULT_MIN_COLUMN_WIDTH`], capped at
/// [`MAX_COLUMN_WIDTH`] so consumers cannot create an inverted range.
pub fn effective_min_width(column_min_width: Option<u32>) -> f64 {
    column_min_width
        .map(|w| w as f64)
        .unwrap_or(DEFAULT_MIN_COLUMN_WIDTH)
        .clamp(0.0, MAX_COLUMN_WIDTH)
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

/// Compute the requested width for a keyboard-operated column separator.
///
/// Left/right arrows resize by [`KEYBOARD_RESIZE_STEP`], while Home and End
/// move to the column's minimum and the global maximum. Unrelated keys are
/// not consumed.
pub fn keyboard_resized_width(current_width: f64, key: &str, min_width: f64) -> Option<f64> {
    let min_width = min_width.clamp(0.0, MAX_COLUMN_WIDTH);
    match key {
        "ArrowLeft" => {
            Some((current_width - KEYBOARD_RESIZE_STEP).clamp(min_width, MAX_COLUMN_WIDTH))
        }
        "ArrowRight" => {
            Some((current_width + KEYBOARD_RESIZE_STEP).clamp(min_width, MAX_COLUMN_WIDTH))
        }
        "Home" => Some(min_width),
        "End" => Some(MAX_COLUMN_WIDTH),
        _ => None,
    }
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

    #[test]
    fn effective_min_width_caps_values_above_the_global_maximum() {
        assert_eq!(effective_min_width(Some(u32::MAX)), MAX_COLUMN_WIDTH);
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

    #[test]
    fn keyboard_resize_supports_arrows_and_range_boundaries() {
        assert_eq!(keyboard_resized_width(100.0, "ArrowLeft", 48.0), Some(84.0));
        assert_eq!(
            keyboard_resized_width(100.0, "ArrowRight", 48.0),
            Some(116.0)
        );
        assert_eq!(keyboard_resized_width(100.0, "Home", 48.0), Some(48.0));
        assert_eq!(
            keyboard_resized_width(100.0, "End", 48.0),
            Some(MAX_COLUMN_WIDTH)
        );
    }

    #[test]
    fn keyboard_resize_clamps_and_ignores_unrelated_keys() {
        assert_eq!(keyboard_resized_width(50.0, "ArrowLeft", 48.0), Some(48.0));
        assert_eq!(
            keyboard_resized_width(MAX_COLUMN_WIDTH - 4.0, "ArrowRight", 48.0),
            Some(MAX_COLUMN_WIDTH)
        );
        assert_eq!(keyboard_resized_width(100.0, "Enter", 48.0), None);
    }
}
