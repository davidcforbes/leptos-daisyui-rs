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
//!
//! The hazard is a property of *SVG presentation attributes*, not of charts, so
//! this module is the crate's routing for any SVG paint — the Gantt timeline
//! draws through it too (ldui-xxc). Elements outside `charts` tend to carry
//! their own static `style`, which is why [`merge_style`] and the
//! [`paint_attrs_with`] / [`stroke_attrs_with`] pair exist.

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
/// where a presentation attribute loses to them. For chart shapes that is free:
/// they carry no per-shape class or id, so the override path does not exist.
/// Outside `charts` it is a real, if small, behaviour change — the Gantt
/// elements routed in ldui-xxc do carry classes (`gantt-today-line`,
/// `gantt-today-background`, `marker-line`, `marker-label`,
/// `dependency-preview-line`), so their colour moves from losing to an author
/// rule to beating it. No stylesheet in this repo or its demo targets those
/// selectors, so nothing changes today; a consumer who wants to restyle them
/// needs `!important` or a `style:` binding, which is the documented cost of
/// routing a token off a presentation attribute at all.
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

/// Joins CSS declarations into one `style` attribute value, or `None` if they
/// contribute nothing.
///
/// [`attrs_for`] returns the *whole* `style` value, so on its own it only fits
/// an element whose `style` carries nothing else. Chart shapes happen to be
/// that shape; most other SVG is not. The Gantt dependency preview already
/// declares `opacity` and `pointer-events`, and a timeline marker declares
/// `cursor` — a second writer of the attribute would silently drop the first,
/// and losing `pointer-events: none` there would make an overlay swallow
/// clicks.
///
/// The composable unit is therefore the *declaration*, not the attribute. This
/// takes declarations in source order, so a static base is simply the first
/// item and a routed colour is another; an element routing both a fill and a
/// stroke passes three. `None` items are skipped, which is exactly what the
/// routers hand back when the colour stayed on its presentation attribute, so
/// no call site has to branch on which half it got.
pub(crate) fn merge_style<I: IntoIterator<Item = Option<String>>>(
    declarations: I,
) -> Option<String> {
    let mut out = String::new();
    for decl in declarations.into_iter().flatten() {
        let decl = decl.trim().trim_end_matches(';').trim();
        if decl.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push_str("; ");
        }
        out.push_str(decl);
    }
    (!out.is_empty()).then_some(out)
}

/// Splits a colour for `property` on an element whose `style` attribute already
/// carries the `base` declarations. See [`merge_style`].
fn attrs_for_with(property: &str, base: &str, color: String) -> (Option<String>, Option<String>) {
    let (attr, routed) = attrs_for(property, color);
    (attr, merge_style([Some(base.to_string()), routed]))
}

/// [`paint_attrs`] for an element whose `style` already carries `base`: the
/// routed declaration is appended to it instead of replacing it.
pub(crate) fn paint_attrs_with(base: &str, color: String) -> (Option<String>, Option<String>) {
    attrs_for_with("fill", base, color)
}

/// [`stroke_attrs`] for an element whose `style` already carries `base`: the
/// routed declaration is appended to it instead of replacing it.
pub(crate) fn stroke_attrs_with(base: &str, color: String) -> (Option<String>, Option<String>) {
    attrs_for_with("stroke", base, color)
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
    fn merge_style_keeps_the_base_declarations_when_a_token_is_routed() {
        // The dependency preview's case: losing `pointer-events: none` would
        // let the overlay swallow clicks, so the base must survive.
        let (stroke, style) = stroke_attrs_with(
            "opacity: 0.7; pointer-events: none",
            "var(--color-success)".into(),
        );
        assert_eq!(stroke, None);
        assert_eq!(
            style.as_deref(),
            Some("opacity: 0.7; pointer-events: none; stroke: var(--color-success)")
        );
    }

    #[test]
    fn merge_style_keeps_the_base_declarations_when_a_literal_stays_on_the_attribute() {
        let (stroke, style) = stroke_attrs_with("cursor: pointer; opacity: 0.7", "#6b7280".into());
        assert_eq!(stroke.as_deref(), Some("#6b7280"));
        assert_eq!(style.as_deref(), Some("cursor: pointer; opacity: 0.7"));
    }

    #[test]
    fn merge_style_normalises_a_trailing_semicolon_rather_than_doubling_it() {
        // Base strings are copied off elements that wrote `...; ` by hand.
        let merged = merge_style([
            Some("opacity: 0.7;".to_string()),
            Some("fill: var(--color-base-100)".to_string()),
        ]);
        assert_eq!(
            merged.as_deref(),
            Some("opacity: 0.7; fill: var(--color-base-100)")
        );
    }

    #[test]
    fn merge_style_is_none_when_nothing_is_contributed() {
        // An empty base plus a literal colour must not emit `style=""`.
        assert_eq!(merge_style([None, None]), None);
        assert_eq!(
            merge_style([Some(String::new()), Some("  ; ".into())]),
            None
        );
        let (fill, style) = paint_attrs_with("", "currentColor".into());
        assert_eq!(fill.as_deref(), Some("currentColor"));
        assert_eq!(style, None);
    }

    #[test]
    fn merge_style_composes_a_routed_fill_and_a_routed_stroke_on_one_element() {
        // The reason declarations rather than whole attribute values are the
        // unit: two routers can write the same element without clobbering.
        let (_, fill_decl) = paint_attrs("var(--color-base-100)".to_string());
        let (_, stroke_decl) = stroke_attrs("var(--color-primary)".to_string());
        let merged = merge_style([Some("opacity: 0.7".to_string()), fill_decl, stroke_decl]);
        assert_eq!(
            merged.as_deref(),
            Some("opacity: 0.7; fill: var(--color-base-100); stroke: var(--color-primary)")
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
