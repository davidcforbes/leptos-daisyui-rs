use pixelproof_style_audit::web::WebAuditConfig;

/// The ldui-flavored web-audit defaults: freeze/oracle query switch, the
/// freeze style tag as readiness proof (the wasm booted in test mode), and
/// the caller's dev-server base URL. Everything else keeps the engine
/// defaults (isolated profile, 500 ms settle, 60 s selector budget,
/// VISUAL_TEST_BASE_URL override).
pub fn ldui_web_config(base_url: impl Into<String>) -> WebAuditConfig {
    let mut cfg = WebAuditConfig::default();
    cfg.harness = cfg.harness.with_base_url(base_url.into());
    cfg.query_suffix = "?pp-freeze=1".into();
    cfg.ready_selectors = vec![r#"style[data-pixelproof="freeze"]"#.into()];
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ldui_web_config_sets_query_suffix_and_ready_selectors() {
        let cfg = ldui_web_config("http://localhost:3000");
        assert_eq!(cfg.query_suffix, "?pp-freeze=1");
        assert_eq!(cfg.ready_selectors.len(), 1);
        assert_eq!(cfg.ready_selectors[0], r#"style[data-pixelproof="freeze"]"#);
        assert_eq!(cfg.harness.base_url, "http://localhost:3000");
    }

    #[test]
    fn ldui_web_config_keeps_isolated_profile() {
        let cfg = ldui_web_config("http://localhost:3000");
        assert!(cfg.harness.isolated_profile);
    }
}
