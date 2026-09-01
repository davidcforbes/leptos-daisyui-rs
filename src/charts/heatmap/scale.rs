//! How a cell intensity becomes a fill colour, and what judgement that colour
//! carries.
//!
//! Kept pure and unit-tested because it is the only place the judgement axis is
//! expressed, and because the same functions serve both the legacy positional
//! render and the typed categorical one — so the two cannot drift apart in what
//! a given intensity looks like.

/// How a heatmap cell's intensity becomes a fill color.
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
    ///
    /// Colour is not the only carrier of that judgement on the typed surface:
    /// a judged cell also gets a solid-or-dashed sense rule and states its
    /// judgement in words in the accessible data table. See
    /// [`HeatmapSense`].
    Judgement,
}

/// The caller-owned judgement a cell's intensity expresses, derived from the
/// scale and the sign rather than supplied separately.
///
/// It exists because a hue alone cannot carry a verdict: under forced colours,
/// for a reader with a colour-vision deficiency, or for a screen-reader user,
/// `--color-success` and `--color-error` are the same cell. The typed render
/// therefore draws a solid (favorable) or dashed (unfavorable) sense rule and
/// states the judgement in words in the data table, both keyed off this.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeatmapSense {
    /// No judgement: the magnitude scale, an exactly-zero deviation, or a cell
    /// with no measurement at all.
    #[default]
    Neutral,
    /// The caller's sign convention says this cell is good.
    Favorable,
    /// The caller's sign convention says this cell is bad.
    Unfavorable,
}

impl HeatmapSense {
    /// The judgement `intensity` carries under `scale`.
    ///
    /// [`HeatScale::Magnitude`] has no sign to read, so it is always neutral —
    /// which is exactly why the magnitude scale cannot express a verdict and a
    /// caller who needs one reaches for the judgement axis.
    pub(super) fn of(intensity: Option<f64>, scale: HeatScale) -> Self {
        let Some(intensity) = intensity.filter(|value| value.is_finite()) else {
            return Self::Neutral;
        };
        match scale {
            HeatScale::Magnitude => Self::Neutral,
            HeatScale::Judgement if intensity > 0.0 => Self::Favorable,
            HeatScale::Judgement if intensity < 0.0 => Self::Unfavorable,
            HeatScale::Judgement => Self::Neutral,
        }
    }

    /// The machine-readable token written to `data-heatmap-sense`, so a browser
    /// test locates a cell's judgement by identity rather than by its colour.
    pub(super) fn token(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Favorable => "favorable",
            Self::Unfavorable => "unfavorable",
        }
    }

    /// The dash pattern the sense rule is drawn with, or `None` for a cell
    /// carrying no judgement — which draws no rule at all. Same solid/dashed
    /// convention as `BarChart`'s status caps, so the two charts read alike.
    pub(super) fn dash(self) -> Option<&'static str> {
        match self {
            Self::Neutral => None,
            Self::Favorable => Some("none"),
            Self::Unfavorable => Some("3 2"),
        }
    }
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
pub(super) fn heat_alpha(intensity: f64) -> f64 {
    if !intensity.is_finite() {
        return 0.0;
    }
    intensity.clamp(0.0, 1.0) * 0.55
}

/// The color inputs a heatmap uses to turn an intensity into a fill.
/// Bundled into a struct so [`cell_fill`] stays a pure two-argument function.
#[derive(Clone, Copy, Debug)]
pub(super) struct HeatPalette<'a> {
    pub scale: HeatScale,
    pub rgb: &'a str,
    pub favorable: &'a str,
    pub unfavorable: &'a str,
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
pub(super) fn cell_fill(intensity: f64, palette: &HeatPalette<'_>) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::paint::paint_attrs;

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

    // --- The judgement a cell carries, independent of its hue ------------

    #[test]
    fn the_magnitude_scale_can_never_express_a_judgement() {
        // The reason the judgement axis exists: a single hue whose alpha is an
        // absolute deviation has no sign to read, so no verdict can be derived
        // from it — in colour or in words.
        for intensity in [-1.0_f64, -0.1, 0.0, 0.1, 1.0] {
            assert_eq!(
                HeatmapSense::of(Some(intensity), HeatScale::Magnitude),
                HeatmapSense::Neutral,
                "{intensity}"
            );
        }
    }

    #[test]
    fn the_judgement_scale_reads_its_verdict_from_the_sign() {
        assert_eq!(
            HeatmapSense::of(Some(0.6), HeatScale::Judgement),
            HeatmapSense::Favorable
        );
        assert_eq!(
            HeatmapSense::of(Some(-0.6), HeatScale::Judgement),
            HeatmapSense::Unfavorable
        );
        assert_eq!(
            HeatmapSense::of(Some(0.0), HeatScale::Judgement),
            HeatmapSense::Neutral,
            "an exactly-zero deviation is a measurement, not a verdict"
        );
    }

    #[test]
    fn an_absent_or_non_finite_intensity_carries_no_verdict() {
        assert_eq!(
            HeatmapSense::of(None, HeatScale::Judgement),
            HeatmapSense::Neutral
        );
        for intensity in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                HeatmapSense::of(Some(intensity), HeatScale::Judgement),
                HeatmapSense::Neutral,
                "{intensity}"
            );
        }
    }

    #[test]
    fn a_judged_cell_gets_a_rule_and_the_two_judgements_differ_in_pattern() {
        // Forced colours and colour-vision deficiency both collapse the two
        // hues; the pattern is what survives either.
        assert_eq!(HeatmapSense::Neutral.dash(), None);
        assert_eq!(HeatmapSense::Favorable.dash(), Some("none"));
        assert_eq!(HeatmapSense::Unfavorable.dash(), Some("3 2"));
        assert_ne!(
            HeatmapSense::Favorable.dash(),
            HeatmapSense::Unfavorable.dash()
        );
    }

    #[test]
    fn sense_tokens_are_stable_selectors() {
        assert_eq!(HeatmapSense::Neutral.token(), "neutral");
        assert_eq!(HeatmapSense::Favorable.token(), "favorable");
        assert_eq!(HeatmapSense::Unfavorable.token(), "unfavorable");
        assert_eq!(HeatmapSense::default(), HeatmapSense::Neutral);
    }
}
