//! Pixel geometry of the grid, shared by the legacy positional render and the
//! typed categorical one so a focus target can never sit off its own cell.

/// Chooses the per-row cell height. When `max_cell_h` is set and the natural
/// stretch-to-fill height exceeds it, the row is capped at `max_cell_h` so a
/// few-row grid in a tall viewport renders as compact tiles rather than giant
/// stretched bricks (bd_4iiz-inventory-toe.4). `None`, or a natural height
/// already within the cap, keeps the natural height.
pub(super) fn clamp_cell_h(natural_cell_h: f64, max_cell_h: Option<f64>) -> f64 {
    match max_cell_h {
        Some(m) if natural_cell_h > m => m,
        _ => natural_cell_h,
    }
}

/// Pixel geometry of a heatmap's grid area: dimensions plus the offset
/// of the grid's top-left corner within the SVG viewport. Bundled into a
/// struct (rather than passed as loose args) to keep [`cell_rect`] under
/// clippy's argument-count lint.
#[derive(Clone, Copy, Debug)]
pub(super) struct GridLayout {
    pub n_rows: usize,
    pub n_cols: usize,
    pub chart_w: f64,
    pub chart_h: f64,
    pub pad_left: f64,
    pub pad_top: f64,
}

/// Computes the pixel rect `(x, y, w, h)` for grid cell `(row, col)` within
/// `layout`. Returns a zero-sized rect when either grid dimension is zero
/// (guards division by zero; callers should not draw a rect for a
/// zero-sized grid).
pub(super) fn cell_rect(row: usize, col: usize, layout: GridLayout) -> (f64, f64, f64, f64) {
    let cell_w = if layout.n_cols == 0 {
        0.0
    } else {
        layout.chart_w / layout.n_cols as f64
    };
    let cell_h = if layout.n_rows == 0 {
        0.0
    } else {
        layout.chart_h / layout.n_rows as f64
    };
    let x = layout.pad_left + col as f64 * cell_w;
    let y = layout.pad_top + row as f64 * cell_h;
    (x, y, cell_w, cell_h)
}

/// The whole resolved frame of one heatmap render: the grid rectangle plus the
/// effective SVG height and view box the caps produced.
///
/// Both render paths build this from the same props through [`frame`], so the
/// typed surface cannot drift from the legacy geometry a consumer already
/// depends on.
#[derive(Clone, Copy, Debug)]
pub(super) struct Frame {
    pub layout: GridLayout,
    pub cell_w: f64,
    pub cell_h: f64,
    pub height_eff: f64,
}

/// The padding a heatmap reserves around its grid.
#[derive(Clone, Copy, Debug)]
pub(super) struct Padding {
    pub left: f64,
    pub top: f64,
}

/// Resolves the grid frame for a non-empty `n_rows` x `n_cols` heatmap.
pub(super) fn frame(
    n_rows: usize,
    n_cols: usize,
    width: u32,
    height: u32,
    padding: Padding,
    max_cell_h: Option<f64>,
) -> Frame {
    let pad_right: f64 = 10.0;
    let pad_bottom: f64 = 10.0;
    let chart_w = width as f64 - padding.left - pad_right;
    let natural_chart_h = height as f64 - padding.top - pad_bottom;

    // Cap the per-row height (if requested) and shrink the SVG to fit so a
    // few-row grid doesn't stretch into tall bricks — bd_4iiz-inventory-toe.4.
    let cell_h = clamp_cell_h(natural_chart_h / n_rows as f64, max_cell_h);
    let chart_h = cell_h * n_rows as f64;

    Frame {
        layout: GridLayout {
            n_rows,
            n_cols,
            chart_w,
            chart_h,
            pad_left: padding.left,
            pad_top: padding.top,
        },
        cell_w: chart_w / n_cols as f64,
        cell_h,
        height_eff: padding.top + chart_h + pad_bottom,
    }
}

/// The default top padding: enough room for slanted column headers when they
/// are requested, and the original tight gutter when they are not.
pub(super) fn default_pad_top(slant_col_labels: bool) -> f64 {
    if slant_col_labels { 70.0 } else { 30.0 }
}

/// The default left padding, reserving space for the row labels.
pub(super) const DEFAULT_PAD_LEFT: f64 = 100.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_cell_h_none_keeps_natural() {
        assert_eq!(clamp_cell_h(97.5, None), 97.5);
    }

    #[test]
    fn clamp_cell_h_caps_when_natural_exceeds_max() {
        // VaR case: 4 rows in a 520px viewport → ~97px natural, capped to 44.
        assert_eq!(clamp_cell_h(97.5, Some(44.0)), 44.0);
    }

    #[test]
    fn clamp_cell_h_keeps_natural_when_already_within_cap() {
        // A dense grid whose natural rows are already compact is left alone.
        assert_eq!(clamp_cell_h(30.0, Some(44.0)), 30.0);
    }

    fn layout(n_rows: usize, n_cols: usize, chart_w: f64, chart_h: f64) -> GridLayout {
        GridLayout {
            n_rows,
            n_cols,
            chart_w,
            chart_h,
            pad_left: 100.0,
            pad_top: 30.0,
        }
    }

    #[test]
    fn cell_rect_places_origin_cell_at_pad() {
        let (x, y, w, h) = cell_rect(0, 0, layout(4, 4, 400.0, 400.0));
        assert_eq!(x, 100.0);
        assert_eq!(y, 30.0);
        assert_eq!(w, 100.0);
        assert_eq!(h, 100.0);
    }

    #[test]
    fn cell_rect_offsets_by_row_and_col() {
        let (x, y, _w, _h) = cell_rect(2, 3, layout(4, 4, 400.0, 400.0));
        assert_eq!(x, 100.0 + 3.0 * 100.0);
        assert_eq!(y, 30.0 + 2.0 * 100.0);
    }

    #[test]
    fn cell_rect_non_square_grid() {
        let mut l = layout(2, 8, 400.0, 100.0);
        l.pad_left = 0.0;
        l.pad_top = 0.0;
        let (_x, _y, w, h) = cell_rect(0, 0, l);
        assert_eq!(w, 50.0);
        assert_eq!(h, 50.0);
    }

    #[test]
    fn cell_rect_zero_grid_dims_are_zero_sized() {
        let (_x, _y, w, h) = cell_rect(0, 0, layout(0, 0, 400.0, 400.0));
        assert_eq!(w, 0.0);
        assert_eq!(h, 0.0);
    }

    #[test]
    fn the_frame_reproduces_the_geometry_the_component_always_computed() {
        // The original component inlined exactly this arithmetic. Both render
        // paths now share it, so the typed surface cannot drift from the
        // legacy one a consumer already depends on.
        let padding = Padding {
            left: 100.0,
            top: 30.0,
        };
        let frame = frame(3, 3, 500, 250, padding, None);

        assert_eq!(frame.layout.chart_w, 500.0 - 100.0 - 10.0);
        assert_eq!(frame.cell_h, (250.0 - 30.0 - 10.0) / 3.0);
        assert_eq!(frame.cell_w, (500.0 - 100.0 - 10.0) / 3.0);
        assert_eq!(frame.height_eff, 250.0, "no cap means no shrink");
    }

    #[test]
    fn a_capped_frame_shrinks_the_svg_to_fit_its_rows() {
        let padding = Padding {
            left: 80.0,
            top: 30.0,
        };
        let frame = frame(3, 3, 500, 200, padding, Some(48.0));

        assert_eq!(frame.cell_h, 48.0);
        assert_eq!(frame.layout.chart_h, 144.0);
        assert_eq!(frame.height_eff, 30.0 + 144.0 + 10.0);
    }

    #[test]
    fn the_default_paddings_are_the_ones_the_component_always_used() {
        assert_eq!(DEFAULT_PAD_LEFT, 100.0);
        assert_eq!(default_pad_top(false), 30.0);
        assert_eq!(default_pad_top(true), 70.0);
    }
}
