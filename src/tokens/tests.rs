use super::{ui_animations_css, ui_tokens_css};

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

#[test]
fn animations_css_defines_state_transition_class() {
    let css = ui_animations_css();
    assert!(css.contains(".ld-eased"), "missing ld-eased: {css}");
    assert!(
        css.contains("var(--ld-duration-fast)"),
        "ld-eased must reference duration token: {css}"
    );
    assert!(
        css.contains("var(--ld-ease-standard)"),
        "ld-eased must reference easing token: {css}"
    );
}

#[test]
fn animations_css_defines_pressable_class() {
    let css = ui_animations_css();
    assert!(
        css.contains(".ld-pressable:active"),
        "missing ld-pressable:active: {css}"
    );
}

#[test]
fn animations_css_defines_elevated_lift() {
    let css = ui_animations_css();
    assert!(css.contains(".ld-elevated"), "missing ld-elevated: {css}");
    assert!(
        css.contains("var(--ld-elevation-4)"),
        "ld-elevated must reference elevation token: {css}"
    );
    assert!(
        css.contains("var(--ld-elevation-8)"),
        "ld-elevated:hover must reference elevation-8 token: {css}"
    );
}

#[test]
fn animations_css_respects_reduced_motion() {
    let css = ui_animations_css();
    assert!(
        css.contains("prefers-reduced-motion"),
        "ld animations must respect prefers-reduced-motion: {css}"
    );
}

#[test]
fn animations_css_has_no_html_unsafe_characters() {
    let css = ui_animations_css();
    assert!(!css.contains('<'), "stray '<' in css: {css}");
    assert!(!css.contains("</"), "stray '</' in css: {css}");
}

#[test]
fn animations_css_defines_dialog_keyframes() {
    let css = ui_animations_css();
    assert!(
        css.contains("@keyframes ld-dialog-in"),
        "missing ld-dialog-in: {css}"
    );
    assert!(
        css.contains("@keyframes ld-backdrop-in"),
        "missing ld-backdrop-in: {css}"
    );
    assert!(
        css.contains("dialog.modal[open] > .modal-box"),
        "missing modal-box selector: {css}"
    );
    assert!(
        css.contains("dialog.modal[open]::backdrop"),
        "missing ::backdrop selector: {css}"
    );
}

#[test]
fn animations_css_defines_toast_keyframes() {
    let css = ui_animations_css();
    assert!(
        css.contains("@keyframes ld-toast-in-top"),
        "missing ld-toast-in-top: {css}"
    );
    assert!(
        css.contains("@keyframes ld-toast-in-bottom"),
        "missing ld-toast-in-bottom: {css}"
    );
    assert!(
        css.contains(".toast.toast-top > *"),
        "missing toast-top selector: {css}"
    );
    assert!(
        css.contains(".toast.toast-bottom > *"),
        "missing toast-bottom selector: {css}"
    );
}

#[test]
fn animations_css_defines_dropdown_keyframes() {
    let css = ui_animations_css();
    assert!(
        css.contains("@keyframes ld-dropdown-in"),
        "missing ld-dropdown-in: {css}"
    );
    assert!(
        css.contains(".dropdown-open > .dropdown-content"),
        "missing dropdown-open selector: {css}"
    );
    assert!(
        css.contains(".dropdown.dropdown-hover:hover"),
        "missing dropdown-hover selector: {css}"
    );
}

#[test]
fn animations_css_defines_ripple() {
    let css = ui_animations_css();
    assert!(
        css.contains(".ld-ripple-host"),
        "missing ld-ripple-host: {css}"
    );
    assert!(
        css.contains(".ld-ripple-element"),
        "missing ld-ripple-element: {css}"
    );
    assert!(
        css.contains("@keyframes ld-ripple"),
        "missing ld-ripple keyframes: {css}"
    );
}

#[test]
fn animations_css_defines_focus_ring() {
    let css = ui_animations_css();
    assert!(
        css.contains(".ld-focus-ring:focus-visible"),
        "missing ld-focus-ring:focus-visible: {css}"
    );
    assert!(
        css.contains("@keyframes ld-focus-ring-in"),
        "missing ld-focus-ring-in keyframes: {css}"
    );
    assert!(
        css.contains("var(--p"),
        "ld-focus-ring must reference daisyUI --p primary token: {css}"
    );
}

#[test]
fn animations_css_defines_drawer_keyframes() {
    let css = ui_animations_css();
    assert!(
        css.contains("@keyframes ld-drawer-in-start"),
        "missing ld-drawer-in-start: {css}"
    );
    assert!(
        css.contains("@keyframes ld-drawer-in-end"),
        "missing ld-drawer-in-end: {css}"
    );
    assert!(
        css.contains("@keyframes ld-drawer-overlay-in"),
        "missing ld-drawer-overlay-in: {css}"
    );
    assert!(
        css.contains(".drawer-open:not(.drawer-end)"),
        "missing drawer-start selector: {css}"
    );
    assert!(
        css.contains(".drawer-open.drawer-end"),
        "missing drawer-end selector: {css}"
    );
}
