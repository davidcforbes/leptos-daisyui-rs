use leptos::prelude::*;

/// A single populated cell within a [`Heatmap`] grid.
#[derive(Clone, Debug, PartialEq)]
pub struct HeatmapCell {
    /// Row index (0-based), matched against `row_labels` by position.
    pub row: usize,
    /// Column index (0-based), matched against `col_labels` by position.
    pub col: usize,
    /// Text rendered centered inside the cell.
    pub label: String,
    /// Intensity in the `0.0..=1.0` range; mapped to fill alpha (capped at
    /// 0.55). Callers compute `intensity = value / max` before passing
    /// cells in — this component only applies the linear alpha mapping.
    pub intensity: f64,
}

/// Maps a raw intensity value to the cell fill alpha.
///
/// Negative intensity clamps to 0; intensity above 1.0 clamps to 1.0 — so the
/// resulting alpha is always in `0.0..=0.55`.
fn heat_alpha(intensity: f64) -> f64 {
    intensity.clamp(0.0, 1.0) * 0.55
}

/// Pixel geometry of a [`Heatmap`]'s grid area: dimensions plus the offset
/// of the grid's top-left corner within the SVG viewport. Bundled into a
/// struct (rather than passed as loose args) to keep [`cell_rect`] under
/// clippy's argument-count lint.
#[derive(Clone, Copy, Debug)]
struct GridLayout {
    n_rows: usize,
    n_cols: usize,
    chart_w: f64,
    chart_h: f64,
    pad_left: f64,
    pad_top: f64,
}

/// Computes the pixel rect `(x, y, w, h)` for grid cell `(row, col)` within
/// `layout`. Returns a zero-sized rect when either grid dimension is zero
/// (guards division by zero; callers should not draw a rect for a
/// zero-sized grid).
fn cell_rect(row: usize, col: usize, layout: GridLayout) -> (f64, f64, f64, f64) {
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

/// SVG-based heatmap component for a generic N x M grid.
///
/// Renders one `<rect>` per supplied [`HeatmapCell`] — `(row, col)` pairs
/// not present in `cells` are left transparent (no rect drawn) — plus a
/// centered label per cell, row labels to the left of the grid, and column
/// labels above it.
#[component]
pub fn Heatmap(
    /// Row labels, top-to-bottom.
    row_labels: Vec<String>,
    /// Column labels, left-to-right.
    col_labels: Vec<String>,
    /// Populated cells. A `(row, col)` not present in this list renders as
    /// transparent (no rect drawn for that grid position).
    cells: Vec<HeatmapCell>,
    /// SVG width in pixels (viewBox coordinate space).
    #[prop(default = 500)]
    width: u32,
    /// SVG height in pixels (viewBox coordinate space).
    #[prop(default = 250)]
    height: u32,
    /// Tint base as a CSS `<r> <g> <b>` triplet, e.g. `"220 38 38"`. Cell
    /// fill is `rgb(<rgb> / <alpha>)` where `alpha = intensity * 0.55`.
    #[prop(default = "220 38 38".to_string())]
    rgb: String,
    /// When `true`, column header labels rotate -45deg around their anchor
    /// (for wide grids, e.g. a 16-column VaR matrix).
    #[prop(default = false)]
    slant_col_labels: bool,
    /// Optional left-padding override (space reserved for row labels).
    /// Defaults to 100.0; raise it when row labels are long enough to clip
    /// (e.g. the VaR matrix's "U-Visa Investigation" / "*Est." prefixes).
    /// bd_4iiz-inventory-43e.
    #[prop(optional)]
    pad_left: Option<f64>,
    /// Optional top-padding override (space reserved for column headers).
    /// Defaults to 70.0 when `slant_col_labels` else 30.0; raise it when
    /// slanted headers overlap the first cell row. bd_4iiz-inventory-43e.
    #[prop(optional)]
    pad_top: Option<f64>,
) -> impl IntoView {
    let viewbox = format!("0 0 {width} {height}");

    if row_labels.is_empty() || col_labels.is_empty() {
        return view! {
            <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
                <text x=format!("{}", width / 2) y=format!("{}", height / 2)
                    text-anchor="middle" fill="currentColor" font-size="14">
                    "No data"
                </text>
            </svg>
        }
        .into_any();
    }

    let n_rows = row_labels.len();
    let n_cols = col_labels.len();

    let pad_left: f64 = pad_left.unwrap_or(100.0);
    let pad_right: f64 = 10.0;
    let pad_top: f64 = pad_top.unwrap_or(if slant_col_labels { 70.0 } else { 30.0 });
    let pad_bottom: f64 = 10.0;

    let chart_w = width as f64 - pad_left - pad_right;
    let chart_h = height as f64 - pad_top - pad_bottom;

    let cell_w = chart_w / n_cols as f64;
    let cell_h = chart_h / n_rows as f64;

    let layout = GridLayout {
        n_rows,
        n_cols,
        chart_w,
        chart_h,
        pad_left,
        pad_top,
    };

    let cell_views = cells
        .into_iter()
        .map(|c| {
            let (x, y, w, h) = cell_rect(c.row, c.col, layout);
            let alpha = heat_alpha(c.intensity);
            let fill = format!("rgb({rgb} / {alpha:.4})");
            let cx = format!("{:.2}", x + w / 2.0);
            let cy = format!("{:.2}", y + h / 2.0);
            view! {
                <rect x=format!("{x:.2}") y=format!("{y:.2}") width=format!("{w:.2}") height=format!("{h:.2}") fill=fill />
                <text x=cx y=cy text-anchor="middle" dominant-baseline="middle"
                    fill="currentColor" font-size="9">
                    {c.label}
                </text>
            }
        })
        .collect_view();

    let row_label_views = row_labels
        .into_iter()
        .enumerate()
        .map(|(ri, label)| {
            let x = format!("{:.2}", pad_left - 8.0);
            let y = format!("{:.2}", pad_top + ri as f64 * cell_h + cell_h / 2.0);
            view! {
                <text x=x y=y text-anchor="end" dominant-baseline="middle"
                    fill="currentColor" font-size="11">
                    {label}
                </text>
            }
        })
        .collect_view();

    let col_label_views = col_labels
        .into_iter()
        .enumerate()
        .map(|(ci, label)| {
            let x = pad_left + ci as f64 * cell_w + cell_w / 2.0;
            let y = pad_top - 8.0;
            let x_str = format!("{x:.2}");
            let y_str = format!("{y:.2}");
            if slant_col_labels {
                // Rise UP-left from just above each column (positive rotate +
                // end-anchor sends the text body to negative-y), so long
                // headers never dip DOWN into the first cell row — the
                // `rotate(-45)` overlap bug (bd_4iiz-inventory-43e).
                let t = format!("rotate(45, {x:.2}, {y:.2})");
                view! {
                    <text x=x_str y=y_str text-anchor="end" fill="currentColor"
                        font-size="10" transform=t>
                        {label}
                    </text>
                }
                .into_any()
            } else {
                view! {
                    <text x=x_str y=y_str text-anchor="middle" fill="currentColor" font-size="10">
                        {label}
                    </text>
                }
                .into_any()
            }
        })
        .collect_view();

    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            {cell_views}
            {row_label_views}
            {col_label_views}
        </svg>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_alpha_clamps_negative_to_zero() {
        assert_eq!(heat_alpha(-0.5), 0.0);
    }

    #[test]
    fn heat_alpha_caps_at_point_fifty_five() {
        assert_eq!(heat_alpha(1.0), 0.55);
        assert_eq!(heat_alpha(2.0), 0.55);
    }

    #[test]
    fn heat_alpha_scales_linearly() {
        assert!((heat_alpha(0.5) - 0.275).abs() < 1e-9);
    }

    #[test]
    fn heat_alpha_zero_intensity_is_zero_alpha() {
        assert_eq!(heat_alpha(0.0), 0.0);
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
}
