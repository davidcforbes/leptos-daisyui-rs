//! How a chart shape's color attaches to its SVG element.
//!
//! SVG2 parses a *presentation attribute* (`fill`, `stroke`, …) using the
//! property's value grammar in isolation, so `var()` substitution — a mechanism
//! defined for declarations — is not specified to run there. It does work for
//! `fill` in all three engines today, but that is unlegislated behavior rather
//! than a guarantee: Chromium already declines to substitute custom properties
//! in *length* presentation attributes, which shows engines do not treat
//! presentation attributes as uniformly declaration-parsed, and the SVG WG has
//! an open request (Nov 2025) to either specify or forbid it.
//!
//! If `fill` ever lands on the "forbid" side, a `var()`-bearing value stops
//! parsing and the shape falls back to the initial `fill: black` — a solid
//! black bar or cell, silently, with no console error. A `style` attribute IS
//! specified to parse as a declaration block, so `var()` is unambiguously
//! supported there.

/// Splits a CSS color into `(fill_attr, style_attr)` for an SVG shape. Exactly
/// one is `Some`.
///
/// A color that references a custom property rides on `style`, where `var()` is
/// specified to work. Anything else — an `oklch(...)` or `rgb(...)` literal,
/// `currentColor` — keeps the `fill` presentation attribute, which is fully
/// specified for those and is what every chart in this module has always
/// emitted. That keeps the DOM byte-identical for every literal-color caller.
///
/// Keying on the presence of `var(` rather than on a component-level flag is
/// deliberate: the hazard belongs to the *value*, not to the chart or the
/// scale, so any caller passing a theme token is covered no matter which prop
/// it arrived through.
///
/// The tradeoff is that an inline `style` outranks author stylesheet rules,
/// where a presentation attribute loses to them. Chart shapes carry no per-shape
/// class or id to select, so that override path does not exist in practice.
pub(crate) fn paint_attrs(color: String) -> (Option<String>, Option<String>) {
    if color.contains("var(") {
        (None, Some(format!("fill: {color}")))
    } else {
        (Some(color), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_colors_keep_the_fill_attribute() {
        for c in [
            "oklch(0.65 0.2 250)",
            "rgb(220 38 38 / 0.5500)",
            "currentColor",
            "#e05654",
        ] {
            let (fill, style) = paint_attrs(c.to_string());
            assert_eq!(fill.as_deref(), Some(c), "{c} should ride on fill");
            assert_eq!(style, None, "{c} should not need a style attribute");
        }
    }

    #[test]
    fn theme_tokens_ride_on_the_style_attribute() {
        let (fill, style) = paint_attrs("var(--color-success)".to_string());
        assert_eq!(fill, None, "var() must not go in the fill attribute");
        assert_eq!(style.as_deref(), Some("fill: var(--color-success)"));
    }

    #[test]
    fn a_token_nested_inside_color_mix_is_still_detected() {
        // The heatmap's judgement axis wraps the token in color-mix, so the
        // check must look anywhere in the value, not just at the start.
        let c = "color-mix(in oklab, var(--color-error) 55.00%, transparent)";
        let (fill, style) = paint_attrs(c.to_string());
        assert_eq!(fill, None);
        assert_eq!(style.as_deref(), Some(&*format!("fill: {c}")));
    }

    #[test]
    fn a_color_mix_of_literals_keeps_the_fill_attribute() {
        // No custom property involved, so the presentation attribute is fine.
        let c = "color-mix(in oklab, #e05654 55%, transparent)";
        let (fill, style) = paint_attrs(c.to_string());
        assert_eq!(fill.as_deref(), Some(c));
        assert_eq!(style, None);
    }

    #[test]
    fn exactly_one_attribute_always_carries_the_color() {
        for c in [
            "oklch(0.65 0.2 250)",
            "var(--color-success)",
            "color-mix(in oklab, var(--color-error) 10%, transparent)",
            "",
        ] {
            let (fill, style) = paint_attrs(c.to_string());
            assert_ne!(
                fill.is_some(),
                style.is_some(),
                "exactly one attribute must carry {c:?}"
            );
        }
    }
}
