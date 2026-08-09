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
//!
//! **`stroke` is the same attribute class and carries the same hazard.** The
//! argument above is about *presentation attributes*, not about `fill`
//! specifically, and `stroke` is one — so a `var()`-bearing stroke would fall
//! back to the initial `stroke: none` and the line would simply vanish. That
//! matters more than it sounds: `LineChart`, `AreaChart` and `Sparkline` draw
//! their actual line with `stroke` alone, so covering only `fill` would leave
//! their primary mark unprotected while looking protected. Hence
//! [`stroke_attrs`] alongside [`paint_attrs`], both built on the same
//! value-keyed split (ldui-1g5).

/// Splits a CSS color into `(attr_value, style_attr)` for `property` on an SVG
/// shape. Exactly one is `Some`.
///
/// A color that references a custom property rides on `style`, where `var()` is
/// specified to work. Anything else — an `oklch(...)` or `rgb(...)` literal,
/// `currentColor` — keeps the presentation attribute, which is fully specified
/// for those and is what every chart in this module has always emitted. That
/// keeps the DOM byte-identical for every literal-color caller.
///
/// Keying on the presence of `var(` rather than on a component-level flag is
/// deliberate: the hazard belongs to the *value*, not to the chart, the scale
/// or the property, so any caller passing a theme token is covered no matter
/// which prop it arrived through.
///
/// The tradeoff is that an inline `style` outranks author stylesheet rules,
/// where a presentation attribute loses to them. Chart shapes carry no per-shape
/// class or id to select, so that override path does not exist in practice.
fn attrs_for(property: &str, color: String) -> (Option<String>, Option<String>) {
    if color.contains("var(") {
        (None, Some(format!("{property}: {color}")))
    } else {
        (Some(color), None)
    }
}

/// Splits a CSS color into `(fill_attr, style_attr)` for an SVG shape. Exactly
/// one is `Some`. See [`attrs_for`].
pub(crate) fn paint_attrs(color: String) -> (Option<String>, Option<String>) {
    attrs_for("fill", color)
}

/// Splits a CSS color into `(stroke_attr, style_attr)` for an SVG shape.
/// Exactly one is `Some`. See [`attrs_for`], and the module doc for why
/// `stroke` needs the same treatment as `fill`.
pub(crate) fn stroke_attrs(color: String) -> (Option<String>, Option<String>) {
    attrs_for("stroke", color)
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
    fn literal_colors_keep_the_stroke_attribute() {
        for c in ["oklch(0.65 0.2 250)", "currentColor", "#e05654"] {
            let (stroke, style) = stroke_attrs(c.to_string());
            assert_eq!(stroke.as_deref(), Some(c), "{c} should ride on stroke");
            assert_eq!(style, None, "{c} should not need a style attribute");
        }
    }

    #[test]
    fn theme_tokens_ride_on_the_style_attribute_for_stroke_too() {
        // A var()-bearing stroke that stopped parsing would fall back to the
        // initial `stroke: none` — the line would silently disappear.
        let (stroke, style) = stroke_attrs("var(--color-primary)".to_string());
        assert_eq!(stroke, None, "var() must not go in the stroke attribute");
        assert_eq!(style.as_deref(), Some("stroke: var(--color-primary)"));
    }

    #[test]
    fn fill_and_stroke_never_share_a_declaration() {
        // The two helpers write different properties, so an element carrying a
        // token-bearing stroke and a literal fill (LineChart's polyline) keeps
        // both, without one clobbering the other.
        let (fill, fill_style) = paint_attrs("none".to_string());
        let (stroke, stroke_style) = stroke_attrs("var(--color-primary)".to_string());
        assert_eq!(fill.as_deref(), Some("none"));
        assert_eq!(fill_style, None);
        assert_eq!(stroke, None);
        assert_eq!(
            stroke_style.as_deref(),
            Some("stroke: var(--color-primary)")
        );
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
            let (stroke, style) = stroke_attrs(c.to_string());
            assert_ne!(
                stroke.is_some(),
                style.is_some(),
                "exactly one attribute must carry {c:?}"
            );
        }
    }
}
