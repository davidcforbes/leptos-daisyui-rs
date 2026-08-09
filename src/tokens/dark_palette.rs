//! The shared dark form-control palette (beads-afda / beads-cuet), sourced
//! from `ui_tokens::color::dark` — the same contrast-validated values
//! `d2d_ui::theme::ControlPalette::dark()` renders with on the desktop, so a
//! web app can present a brand-matched dark surface instead of re-inventing
//! one.
//!
//! [`dark_form_css_vars`] maps the palette onto daisyUI 5's `--color-*`
//! custom properties (a starting mapping — override per app as needed), and
//! the tests hold the palette to the same WCAG AA bar the desktop asserts:
//! every non-disabled text/fill pair >= 4.5:1. Placeholders belong on the
//! DISABLED tier (WCAG-exempt by design), matching the desktop's attribution
//! — placeholder ink is `--ld-dark-text-disabled`, not the secondary tone.

use ui_tokens::color::{dark, to_css_hex};

/// The dark palette as `--ld-dark-*` custom properties plus a daisyUI-5
/// `--color-*` mapping, ready to drop inside a `[data-theme="..."]` or
/// `:root` block:
///
/// | daisyUI variable | ui-tokens slot |
/// |---|---|
/// | `--color-base-100` | `dark::CARD_BG` (input fill) |
/// | `--color-base-200` | `dark::CONTROL_REST` (button/checkbox fill) |
/// | `--color-base-300` | `dark::CONTROL_HOVER` |
/// | `--color-base-content` | `dark::TEXT_PRIMARY` |
pub fn dark_form_css_vars() -> String {
    let mut css = String::new();
    for (name, hex) in [
        ("text-primary", dark::TEXT_PRIMARY),
        ("text-secondary", dark::TEXT_SECONDARY),
        ("text-disabled", dark::TEXT_DISABLED),
        ("card-bg", dark::CARD_BG),
        ("control-rest", dark::CONTROL_REST),
        ("control-hover", dark::CONTROL_HOVER),
        ("control-pressed", dark::CONTROL_PRESSED),
        ("control-border", dark::CONTROL_BORDER),
        ("control-border-bottom", dark::CONTROL_BORDER_BOTTOM),
    ] {
        css.push_str(&format!("  --ld-dark-{}: {};\n", name, to_css_hex(hex)));
    }
    css.push_str(&format!(
        "  --color-base-100: {};\n",
        to_css_hex(dark::CARD_BG)
    ));
    css.push_str(&format!(
        "  --color-base-200: {};\n",
        to_css_hex(dark::CONTROL_REST)
    ));
    css.push_str(&format!(
        "  --color-base-300: {};\n",
        to_css_hex(dark::CONTROL_HOVER)
    ));
    css.push_str(&format!(
        "  --color-base-content: {};\n",
        to_css_hex(dark::TEXT_PRIMARY)
    ));
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG 2.x relative luminance of a packed 0xRRGGBB color — the same
    /// formula d2d-ui's theme tests use, so both faces hold one bar.
    fn rel_lum(hex: u32) -> f64 {
        fn ch(c: u32) -> f64 {
            let c = c as f64 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        let (r, g, b) = ((hex >> 16) & 0xFF, (hex >> 8) & 0xFF, hex & 0xFF);
        0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
    }

    fn contrast_ratio(a: u32, b: u32) -> f64 {
        let (la, lb) = (rel_lum(a), rel_lum(b));
        let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// beads-cuet: the web face holds the shared dark palette to the same
    /// WCAG AA bar the desktop asserts (measured, not eyeballed).
    #[test]
    fn shared_dark_palette_meets_wcag_aa_contrast() {
        for (bg_name, bg) in [
            ("card_bg", dark::CARD_BG),
            ("control_rest", dark::CONTROL_REST),
            ("control_hover", dark::CONTROL_HOVER),
            ("control_pressed", dark::CONTROL_PRESSED),
        ] {
            let r = contrast_ratio(dark::TEXT_PRIMARY, bg);
            assert!(
                r >= 4.5,
                "text_primary on {bg_name} is {r:.2}:1, needs 4.5:1"
            );
        }
        for (bg_name, bg) in [
            ("card_bg", dark::CARD_BG),
            ("control_rest", dark::CONTROL_REST),
        ] {
            let r = contrast_ratio(dark::TEXT_SECONDARY, bg);
            assert!(
                r >= 4.5,
                "text_secondary on {bg_name} is {r:.2}:1, needs 4.5:1"
            );
        }
    }

    /// The disabled/placeholder tier is deliberately BELOW the AA bar
    /// (WCAG-exempt); if it ever clears 4.5:1 the palette roles have likely
    /// been shuffled and placeholder mapping should be re-checked.
    #[test]
    fn disabled_tier_is_the_muted_one() {
        let r = contrast_ratio(dark::TEXT_DISABLED, dark::CARD_BG);
        assert!(r < 4.5, "text_disabled reads as body text ({r:.2}:1)");
        // ...but still darker-vs-lighter ordered sanely within the ramp.
        assert!(rel_lum(dark::TEXT_DISABLED) > rel_lum(dark::CARD_BG));
    }

    #[test]
    fn css_vars_emit_every_slot_and_the_daisyui_mapping() {
        let css = dark_form_css_vars();
        for needle in [
            "--ld-dark-text-primary: #e6e6e6;",
            "--ld-dark-text-disabled: #6a6a6a;",
            "--ld-dark-card-bg: #1f1f1f;",
            "--ld-dark-control-border-bottom: #5a5a5a;",
            "--color-base-100: #1f1f1f;",
            "--color-base-content: #e6e6e6;",
        ] {
            assert!(css.contains(needle), "missing {needle} in:\n{css}");
        }
    }
}
