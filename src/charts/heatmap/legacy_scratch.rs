use super::paint::paint_attrs;
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
    /// Cell intensity. Its meaning depends on the heatmap's [`HeatScale`]:
    ///
    /// - [`HeatScale::Magnitude`] (default): `0.0..=1.0`, mapped to fill alpha
    ///   (capped at 0.55) over the single `rgb` hue. Negative values clamp to 0.
    /// - [`HeatScale::Judgement`]: `-1.0..=1.0`. The SIGN picks the hue
    ///   (positive = favorable, negative = unfavorable) and the MAGNITUDE picks
    ///   the alpha over the same ramp.
    ///
    /// Callers normalize before passing cells in — this component only applies
    /// the linear alpha mapping and the hue choice.
    pub intensity: f64,
}

/// How a [`HeatmapCell`]'s intensity becomes a fill color.
///
/// `#[non_exhaustive]`: a third scale (a diverging three-band read, say) should
/// not be a breaking change for consumers that `match` on this.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeatScale {
    /// Single hue, magnitude only: every cell tints with the `rgb` prop and
    /// intensity drives alpha alone. Negative intensity clamps to zero. This
    /// is the legacy behavior and remains the default, so existing callers are
    /// unaffected.
    #[default]
    Magnitude,
    /// Signed judgement axis (ldui-7zj): the sign of the intensity picks the
    /// hue and the magnitude picks the alpha, over the same `0.0..=0.55` ramp.
    ///
    /// Positive intensity uses `favorable_color` (daisyUI's `--color-success`
    /// token by default), negative uses `unfavorable_color` (`--color-error`).
    /// Zero is fully transparent under either hue, so there is no visual jump
    /// at the sign flip.
    ///
    /// Two consequences are deliberate. First, the color expresses a
    /// **judgement**, never a category: there is no per-cell color prop to
    /// reach for, only a signed number, so a caller cannot accidentally tint
    /// by series. Second, the *sense* of a measure lives in the caller's sign
    /// convention, which is per-value and therefore per-column by
    /// construction: a column where higher is better passes
    /// `(value - target) / scale`, and a column where lower is better (handle
    /// time, overdue count) passes the negation. No global "higher is good"
    /// flag is needed or offered.
    Judgement,
}

/// Maps a raw intensity value to the cell fill alpha.
///
/// Negative intensity clamps to 0; intensity above 1.0 clamps to 1.0 — so the
/// resulting alpha is always in `0.0..=0.55`.
///
/// A non-finite intensity folds to 0 rather than propagating. `f64::clamp`
/// passes NaN straight through, which would format the fill as `NaN` — an
/// unparseable value, so the `fill` falls back to its initial `black` and the
/// cell paints as a solid black tile. That is the loudest possible rendering of
/// a missing datapoint and the exact opposite of what a caller wants, so a
/// non-finite intensity is treated as no signal: fully transparent. Same
/// convention as `progress_value` in the Progress component.
fn heat_alpha(intensity: f64) -> f64 {
    if !intensity.is_finite() {
        return 0.0;
    }
    intensity.clamp(0.0, 1.0) * 0.55
}

/// The color inputs a [`Heatmap`] uses to turn an intensity into a fill.
/// Bundled into a struct so [`cell_fill`] stays a pure two-argument function.
#[derive(Clone, Copy, Debug)]
struct HeatPalette<'a> {
    scale: HeatScale,
    rgb: &'a str,
    favorable: &'a str,
    unfavorable: &'a str,
}

/// Turns a cell intensity into the CSS color painted into `fill`.
///
/// This is the whole color decision for a heatmap cell, kept pure and
/// unit-tested because it is the only place the judgement axis is expressed.
///
/// Under [`HeatScale::Magnitude`] the result is the legacy
/// `rgb(<triplet> / <alpha>)`, byte-identical to what the component emitted
/// before the judgement axis existed. Under [`HeatScale::Judgement`] the sign
/// selects `favorable`/`unfavorable` and the magnitude drives a `color-mix`
/// against `transparent` — the same construction daisyUI itself uses, so a
/// theme token like `var(--color-success)` works directly and follows a theme
/// switch.
///
/// A non-finite intensity yields a fully transparent fill (see [`heat_alpha`]);
/// under the judgement axis it also reads as non-negative, so it takes the
/// favorable hue — invisible either way, since the alpha is zero.
fn cell_fill(intensity: f64, palette: &HeatPalette<'_>) -> String {
    match palette.scale {
        HeatScale::Magnitude => {
            let alpha = heat_alpha(intensity);
            format!("rgb({} / {alpha:.4})", palette.rgb)
        }
        HeatScale::Judgement => {
            let base = if intensity < 0.0 {
                palette.unfavorable
            } else {
                palette.favorable
            };
            let pct = heat_alpha(intensity.abs()) * 100.0;
            format!("color-mix(in oklab, {base} {pct:.2}%, transparent)")
        }
    }
}

/// Chooses the per-row cell height. When `max_cell_h` is set and the natural
/// stretch-to-fill height exceeds it, the row is capped at `max_cell_h` so a
/// few-row grid in a tall viewport renders as compact tiles rather than giant
/// stretched bricks (bd_4iiz-inventory-toe.4). `None`, or a natural height
/// already within the cap, keeps the natural height.
fn clamp_cell_h(natural_cell_h: f64, max_cell_h: Option<f64>) -> f64 {
    match max_cell_h {
        Some(m) if natural_cell_h > m => m,
        _ => natural_cell_h,
    }
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
///
/// ## Color scales
///
/// By default the grid is a single hue whose alpha carries magnitude. Set
/// `scale=HeatScale::Judgement` to turn it into a favorable/unfavorable axis:
/// each cell's intensity becomes signed, the sign picks the hue and the
/// magnitude still picks the alpha. The two hues default to daisyUI's
/// `--color-success` and `--color-error` theme tokens, so no new color enters
/// the palette and both follow a theme switch.
///
/// Color on that axis carries the **judgement**, never the category — the API
/// offers no per-cell color, only a signed number, so there is nothing to tint
/// a series with. The *sense* of a measure is the caller's sign convention and
/// is therefore per-value (hence per-column) rather than a global flag: negate
/// the deviation for a column where lower is better.
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
    /// Used by [`HeatScale::Magnitude`] only.
    #[prop(default = "220 38 38".to_string())]
    rgb: String,
    /// Which color scale the cells use (ldui-7zj). Defaults to
    /// [`HeatScale::Magnitude`] — the legacy single-hue behavior — so existing
    /// callers render exactly as before. [`HeatScale::Judgement`] switches to
    /// the signed favorable/unfavorable axis.
    #[prop(default = HeatScale::Magnitude)]
    scale: HeatScale,
    /// Hue for favorable (positive-intensity) cells under
    /// [`HeatScale::Judgement`]. Defaults to daisyUI's `--color-success` theme
    /// token so the heatmap introduces no new color and follows theme changes.
    ///
    /// Pass a daisyUI theme token — `"var(--color-success)"`,
    /// `"var(--color-warning)"`, `"var(--color-error)"` or
    /// `"var(--color-info)"`. Staying on the tokens is the point: the palette
    /// should not grow a hue the rest of the app does not already use, and
    /// tokens follow a theme switch. Ignored under [`HeatScale::Magnitude`].
    #[prop(default = "var(--color-success)".to_string())]
    favorable_color: String,
    /// Hue for unfavorable (negative-intensity) cells under
    /// [`HeatScale::Judgement`]. Defaults to daisyUI's `--color-error` theme
    /// token; pass `"var(--color-warning)"` for a softer at-risk read. Same
    /// token guidance as `favorable_color`. Ignored under
    /// [`HeatScale::Magnitude`].
    #[prop(default = "var(--color-error)".to_string())]
    unfavorable_color: String,
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
    /// Optional per-row height cap in px. When set, each row is drawn at
    /// `min(natural_row_height, max_cell_h)` and the SVG viewBox height shrinks
    /// to fit exactly `pad_top + n_rows*row_h + pad_bottom` — so a grid with
    /// few rows in a tall viewport renders compact tiles instead of giant
    /// stretched bricks with large inter-row gaps (bd_4iiz-inventory-toe.4).
    /// `None` keeps the legacy stretch-to-fill behavior.
    #[prop(optional)]
    max_cell_h: Option<f64>,
    /// Optional per-cell click handler, called with `(row, col)` (0-based, in
    /// the same index space as `row_labels`/`col_labels`). When set, EVERY grid
    /// position — including empty ones — becomes clickable via a transparent
    /// overlay rect, so a consumer can drill from a cell whether or not it drew a
    /// tile there. Purely additive: `None` (the default) is the legacy
    /// non-interactive heatmap, unchanged for every existing consumer.
    #[prop(optional)]
    on_cell_click: Option<Callback<(usize, usize)>>,
) -> impl IntoView {
    if row_labels.is_empty() || col_labels.is_empty() {
        return view! {
            <svg viewBox=format!("0 0 {width} {height}") class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
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
    let natural_chart_h = height as f64 - pad_top - pad_bottom;

    // Cap the per-row height (if requested) and shrink the SVG to fit so a
    // few-row grid doesn't stretch into tall bricks — bd_4iiz-inventory-toe.4.
    let cell_h = clamp_cell_h(natural_chart_h / n_rows as f64, max_cell_h);
    let chart_h = cell_h * n_rows as f64;
    let height_eff = pad_top + chart_h + pad_bottom;
    let viewbox = format!("0 0 {width} {height_eff:.0}");

    let cell_w = chart_w / n_cols as f64;

    let layout = GridLayout {
        n_rows,
        n_cols,
        chart_w,
        chart_h,
        pad_left,
        pad_top,
    };

    let palette = HeatPalette {
        scale,
        rgb: &rgb,
        favorable: &favorable_color,
        unfavorable: &unfavorable_color,
    };

    let cell_views = cells
        .into_iter()
        .map(|c| {
            let (x, y, w, h) = cell_rect(c.row, c.col, layout);
            // Exactly one of these is Some — see `paint_attrs` for why a
            // token-bearing colour cannot ride on the `fill` attribute.
            let (fill, style) = paint_attrs(cell_fill(c.intensity, &palette));
            let cx = format!("{:.2}", x + w / 2.0);
            let cy = format!("{:.2}", y + h / 2.0);
            view! {
                <rect x=format!("{x:.2}") y=format!("{y:.2}") width=format!("{w:.2}") height=format!("{h:.2}") fill=fill style=style />
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

    // Optional interaction overlay (beads-1qhd): one transparent rect per grid
    // position — including empty ones — so a click lands on any (row, col) even
    // where no tile was drawn. Only rendered when a handler is supplied, so the
    // legacy heatmap emits no extra DOM.
    let click_overlay = on_cell_click.map(|cb| {
        (0..n_rows)
            .flat_map(|ri| (0..n_cols).map(move |ci| (ri, ci)))
            .map(|(ri, ci)| {
                let (x, y, w, h) = cell_rect(ri, ci, layout);
                view! {
                    <rect
                        x=format!("{x:.2}")
                        y=format!("{y:.2}")
                        width=format!("{w:.2}")
                        height=format!("{h:.2}")
                        fill="transparent"
                        style="cursor: pointer"
                        on:click=move |_| cb.run((ri, ci))
                    />
                }
            })
            .collect_view()
    });

    view! {
        <svg viewBox=viewbox class="w-full h-auto" xmlns="http://www.w3.org/2000/svg">
            {cell_views}
            {row_label_views}
            {col_label_views}
            {click_overlay}
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

    // --- Judgement axis (ldui-7zj) -------------------------------------

    fn magnitude_palette() -> HeatPalette<'static> {
        HeatPalette {
            scale: HeatScale::Magnitude,
            rgb: "220 38 38",
            favorable: "var(--color-success)",
            unfavorable: "var(--color-error)",
        }
    }

    fn judgement_palette() -> HeatPalette<'static> {
        HeatPalette {
            scale: HeatScale::Judgement,
            rgb: "220 38 38",
            favorable: "var(--color-success)",
            unfavorable: "var(--color-error)",
        }
    }

    #[test]
    fn cell_fill_magnitude_is_the_legacy_single_hue_output() {
        // Byte-for-byte what the component emitted before the axis existed.
        let p = magnitude_palette();
        assert_eq!(cell_fill(1.0, &p), "rgb(220 38 38 / 0.5500)");
        assert_eq!(cell_fill(0.0, &p), "rgb(220 38 38 / 0.0000)");
        assert_eq!(cell_fill(0.5, &p), "rgb(220 38 38 / 0.2750)");
    }

    #[test]
    fn cell_fill_magnitude_ignores_sign_and_clamps() {
        let p = magnitude_palette();
        // Negative clamps to zero alpha; the hue never changes.
        assert_eq!(cell_fill(-1.0, &p), "rgb(220 38 38 / 0.0000)");
        assert_eq!(cell_fill(5.0, &p), "rgb(220 38 38 / 0.5500)");
    }

    #[test]
    fn cell_fill_judgement_zero_is_fully_transparent() {
        let p = judgement_palette();
        assert_eq!(
            cell_fill(0.0, &p),
            "color-mix(in oklab, var(--color-success) 0.00%, transparent)"
        );
    }

    #[test]
    fn cell_fill_judgement_plus_one_is_full_favorable() {
        let p = judgement_palette();
        assert_eq!(
            cell_fill(1.0, &p),
            "color-mix(in oklab, var(--color-success) 55.00%, transparent)"
        );
    }

    #[test]
    fn cell_fill_judgement_minus_one_is_full_unfavorable() {
        let p = judgement_palette();
        assert_eq!(
            cell_fill(-1.0, &p),
            "color-mix(in oklab, var(--color-error) 55.00%, transparent)"
        );
    }

    #[test]
    fn cell_fill_judgement_clamps_beyond_plus_minus_one() {
        let p = judgement_palette();
        assert_eq!(
            cell_fill(4.2, &p),
            "color-mix(in oklab, var(--color-success) 55.00%, transparent)"
        );
        assert_eq!(
            cell_fill(-4.2, &p),
            "color-mix(in oklab, var(--color-error) 55.00%, transparent)"
        );
    }

    #[test]
    fn cell_fill_judgement_flips_hue_at_zero() {
        let p = judgement_palette();
        // Zero and above is favorable; anything below zero is unfavorable.
        assert!(cell_fill(0.0, &p).contains("--color-success"));
        assert!(cell_fill(f64::MIN_POSITIVE, &p).contains("--color-success"));
        assert!(cell_fill(-f64::MIN_POSITIVE, &p).contains("--color-error"));
        assert!(cell_fill(-0.25, &p).contains("--color-error"));
    }

    /// Extracts the mix percentage token from a `color-mix(...)` fill.
    ///
    /// `"color-mix(in oklab, var(--color-success) 55.00%, transparent)"` splits
    /// on spaces into `["color-mix(in", "oklab,", "var(--color-success)",
    /// "55.00%,", "transparent)"]`, so the percentage is index **3**. It was
    /// index 4 until review caught it, which made the symmetry test below
    /// compare `"transparent)"` against itself and pass unconditionally. The
    /// shape assertion is the guard against that recurring: if the format ever
    /// changes, this fails loudly instead of silently comparing the wrong
    /// token.
    fn mix_pct(fill: &str) -> &str {
        let tok = fill
            .split(' ')
            .nth(3)
            .unwrap_or_else(|| panic!("no token 3 in {fill:?}"));
        assert!(
            tok.ends_with("%,"),
            "token 3 of {fill:?} is {tok:?}, not a percentage — the format moved"
        );
        tok
    }

    #[test]
    fn cell_fill_judgement_magnitude_is_symmetric_about_zero() {
        // The alpha ramp must not favor one side: |x| drives it on both.
        let p = judgement_palette();
        for x in [0.1_f64, 0.33, 0.5, 0.9, 1.0] {
            let pos = cell_fill(x, &p);
            let neg = cell_fill(-x, &p);
            assert_eq!(mix_pct(&pos), mix_pct(&neg), "asymmetric ramp at {x}");
        }
    }

    #[test]
    fn cell_fill_judgement_honours_overridden_hues() {
        // A caller wanting an at-risk read swaps in another theme token.
        let p = HeatPalette {
            scale: HeatScale::Judgement,
            rgb: "220 38 38",
            favorable: "var(--color-info)",
            unfavorable: "var(--color-warning)",
        };
        assert!(cell_fill(0.5, &p).contains("var(--color-info)"));
        assert!(cell_fill(-0.5, &p).contains("var(--color-warning)"));
    }

    // Non-finite intensity must not reach the DOM. `f64::clamp` propagates NaN,
    // so before the fold these formatted `NaN` / `inf` into the fill, which
    // fails to parse and drops the rect back to the initial `fill: black` — a
    // solid black tile for a missing datapoint.
    #[test]
    fn cell_fill_magnitude_folds_non_finite_to_transparent() {
        let p = magnitude_palette();
        assert_eq!(cell_fill(f64::NAN, &p), "rgb(220 38 38 / 0.0000)");
        assert_eq!(cell_fill(f64::INFINITY, &p), "rgb(220 38 38 / 0.0000)");
        assert_eq!(cell_fill(f64::NEG_INFINITY, &p), "rgb(220 38 38 / 0.0000)");
    }

    #[test]
    fn cell_fill_judgement_folds_non_finite_to_transparent() {
        let p = judgement_palette();
        for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let got = cell_fill(x, &p);
            assert!(
                !got.contains("NaN") && !got.contains("inf"),
                "non-finite leaked into the fill: {got}"
            );
            assert_eq!(mix_pct(&got), "0.00%,", "non-finite must be transparent");
        }
    }

    #[test]
    fn heat_alpha_folds_non_finite_to_zero() {
        assert_eq!(heat_alpha(f64::NAN), 0.0);
        assert_eq!(heat_alpha(f64::INFINITY), 0.0);
        assert_eq!(heat_alpha(f64::NEG_INFINITY), 0.0);
    }

    // How each scale's colour reaches the DOM. The rule itself lives in
    // `super::paint` and is tested there; these pin the *composition* — that
    // the default magnitude fill keeps the legacy presentation attribute while
    // the default judgement fill is routed to `style`.
    #[test]
    fn magnitude_fill_keeps_the_legacy_fill_attribute() {
        let (fill, style) = paint_attrs(cell_fill(1.0, &magnitude_palette()));
        assert_eq!(fill.as_deref(), Some("rgb(220 38 38 / 0.5500)"));
        assert_eq!(style, None, "magnitude must keep the legacy DOM");
    }

    #[test]
    fn judgement_fill_is_routed_to_the_style_attribute() {
        // The default hues are theme tokens, so var() substitution — only
        // specified inside a declaration block — must not land in `fill`.
        for intensity in [1.0_f64, -1.0, 0.25] {
            let (fill, style) = paint_attrs(cell_fill(intensity, &judgement_palette()));
            assert_eq!(fill, None, "var() must not go in the fill attribute");
            assert!(style.is_some_and(|s| s.starts_with("fill: color-mix(")));
        }
    }

    #[test]
    fn heat_scale_default_is_magnitude() {
        // Existing callers must keep the single-hue behavior unchanged.
        assert_eq!(HeatScale::default(), HeatScale::Magnitude);
    }

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
}
