use super::ui_tokens_css;

#[test]
fn css_starts_with_root_selector() {
    let css = ui_tokens_css();
    assert!(css.starts_with(":root {"), "css was: {css}");
    assert!(css.trim_end().ends_with('}'), "css was: {css}");
}

#[test]
fn css_emits_all_three_duration_constants() {
    let css = ui_tokens_css();
    assert!(
        css.contains("--ld-duration-fast: 83ms;"),
        "missing duration-fast: {css}"
    );
    assert!(
        css.contains("--ld-duration-normal: 200ms;"),
        "missing duration-normal: {css}"
    );
    assert!(
        css.contains("--ld-duration-slow: 300ms;"),
        "missing duration-slow: {css}"
    );
}

#[test]
fn duration_constants_pin_ui_tokens_values() {
    assert_eq!(ui_tokens::motion::DURATION_FAST_MS, 83);
    assert_eq!(ui_tokens::motion::DURATION_NORMAL_MS, 200);
    assert_eq!(ui_tokens::motion::DURATION_SLOW_MS, 300);
}

#[test]
fn css_emits_all_four_named_easings() {
    let css = ui_tokens_css();
    assert!(
        css.contains("--ld-ease-linear: cubic-bezier(0, 0, 1, 1);"),
        "missing ease-linear: {css}"
    );
    assert!(
        css.contains("--ld-ease-standard: cubic-bezier(0.33, 0, 0.67, 1);"),
        "missing ease-standard: {css}"
    );
    assert!(
        css.contains("--ld-ease-decelerate: cubic-bezier(0.1, 0.9, 0.2, 1);"),
        "missing ease-decelerate: {css}"
    );
    assert!(
        css.contains("--ld-ease-accelerate: cubic-bezier(0.7, 0, 1, 0.5);"),
        "missing ease-accelerate: {css}"
    );
}

#[test]
fn css_emits_every_elevation_tier() {
    let css = ui_tokens_css();
    for tier in ["2", "4", "8", "16", "64"] {
        let needle = format!("--ld-elevation-{}:", tier);
        assert!(css.contains(&needle), "missing tier {tier}: {css}");
    }
}

#[test]
fn css_elevation_4_matches_canonical_shadow() {
    // ui_tokens::elevation::LEVEL_4 = Shadow::new(0.0, 2.0, 4.0, 0.14)
    let css = ui_tokens_css();
    assert!(
        css.contains("--ld-elevation-4: 0px 2px 4px rgba(0, 0, 0, 0.14);"),
        "elevation-4 mis-formatted: {css}"
    );
}

#[test]
fn css_elevation_16_matches_canonical_shadow() {
    // ui_tokens::elevation::LEVEL_16 = Shadow::new(0.0, 8.0, 16.0, 0.18)
    let css = ui_tokens_css();
    assert!(
        css.contains("--ld-elevation-16: 0px 8px 16px rgba(0, 0, 0, 0.18);"),
        "elevation-16 mis-formatted: {css}"
    );
}

#[test]
fn css_has_no_html_unsafe_characters() {
    // The preamble is dropped into a <style> element raw; HTML special
    // chars inside would break parsing or leak out of the style scope.
    let css = ui_tokens_css();
    assert!(!css.contains('<'), "stray '<' in css: {css}");
    assert!(!css.contains('>'), "stray '>' in css: {css}");
    assert!(!css.contains('&'), "stray '&' in css: {css}");
}
