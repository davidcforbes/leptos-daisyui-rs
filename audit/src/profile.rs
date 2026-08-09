use pixelproof_style_audit::{ShadowSpec, StyleProfile};

/// Profile defaults derived from `ui_tokens` at compile time, so a profile
/// cannot drift from the token crate. Apps chain builder overrides for their
/// deliberate deviations only. `font_family` is a parameter because the token
/// crate does not own a web font-family name.
pub fn from_ui_tokens(font_family: impl Into<String>) -> StyleProfile {
    StyleProfile::new(font_family)
        .body_weight(ui_tokens::typography::Weight::Regular.value())
        .type_ramp(ui_tokens::typography::RAMP.iter().map(|&v| v as f64))
        .line_ramp(ui_tokens::typography::LINE_RAMP.iter().map(|&v| v as f64))
        .radii(
            [
                ui_tokens::radius::CONTROL,
                ui_tokens::radius::CARD,
                ui_tokens::radius::BADGE,
                ui_tokens::radius::PILL,
            ]
            .iter()
            .map(|&v| v as f64),
        )
        .shadows(ui_tokens::elevation::LEVELS.iter().map(|s| {
            ShadowSpec::new(
                s.offset_x as f64,
                s.offset_y as f64,
                s.blur as f64,
                s.opacity as f64,
            )
        }))
        .spacing_scale(ui_tokens::spacing::SCALE.iter().map(|&v| v as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_pin_to_the_token_crate() {
        let p = from_ui_tokens("Inter");
        assert_eq!(p.font_family, "Inter");
        assert_eq!(p.body_weight, 400);
        assert_eq!(p.type_ramp, vec![28.0, 20.0, 16.0, 14.0, 12.0, 11.0]);
        assert_eq!(p.line_ramp, vec![36.0, 28.0, 24.0, 20.0, 16.0, 16.0]);
        assert_eq!(p.radii, vec![4.0, 8.0, 12.0, 9999.0]);
        assert_eq!(
            p.spacing_scale,
            vec![4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 96.0]
        );
        assert_eq!(p.shadows.len(), 5);
        assert!(
            p.shadow_ok(&ShadowSpec::new(0.0, 2.0, 4.0, 0.14)),
            "LEVEL_4"
        );
    }

    #[test]
    fn overrides_replace_not_append() {
        let p = from_ui_tokens("Manrope").radii([15.0, 8.0, 999.0]);
        assert!(p.radius_ok(15.0));
        assert!(
            !p.radius_ok(12.0),
            "token BADGE radius replaced by override"
        );
    }
}
