use super::{
    SPACE_STEPS, STROKE_STEPS, TYPE_STEPS, spacing_scale_px, ui_animations_css, ui_tokens_css,
};

// ---------------------------------------------------------------------------
// Spacing scale (ldui-d14 / ldui-1mx)
// ---------------------------------------------------------------------------

#[test]
fn css_emits_every_spacing_step() {
    let css = ui_tokens_css();
    for (name, dips) in SPACE_STEPS {
        let needle = format!("--ld-space-{}: {}px;", name, dips);
        assert!(css.contains(&needle), "missing {needle}: {css}");
    }
}

#[test]
fn spacing_scale_is_the_canonical_nine_steps() {
    assert_eq!(
        spacing_scale_px(),
        [4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 96.0]
    );
}

#[test]
fn spacing_scale_lives_on_the_4px_grid() {
    for step in spacing_scale_px() {
        assert_eq!(step % 4.0, 0.0, "{step} is not a multiple of 4");
    }
}

#[test]
fn spacing_scale_mirrors_ui_tokens_exactly() {
    // The web scale must not drift from the shared crate. If this fails,
    // the two faces have forked.
    assert_eq!(spacing_scale_px(), ui_tokens::spacing::SCALE);
}

// ---------------------------------------------------------------------------
// Stroke family — separate from spacing on purpose
// ---------------------------------------------------------------------------

#[test]
fn css_emits_every_stroke_width() {
    let css = ui_tokens_css();
    for (name, dips) in STROKE_STEPS {
        let needle = format!("--ld-stroke-{}: {}px;", name, dips);
        assert!(css.contains(&needle), "missing {needle}: {css}");
    }
}

#[test]
fn strokes_are_not_spacing_steps() {
    // A hairline is not a gap. If a stroke width ever lands on the spacing
    // scale the checker can no longer tell a divider from a bad value.
    let scale = spacing_scale_px();
    for (name, dips) in STROKE_STEPS {
        if name == "emphasis" {
            continue; // 4px deliberately meets the spacing floor
        }
        assert!(
            !scale.contains(&dips),
            "stroke {name} ({dips}px) collides with the spacing scale"
        );
    }
}

// ---------------------------------------------------------------------------
// Line-height ramp — the rhythm this ticket exists to pin
// ---------------------------------------------------------------------------

#[test]
fn css_emits_size_and_line_height_for_every_ramp_step() {
    let css = ui_tokens_css();
    for (name, size, line) in TYPE_STEPS {
        assert!(
            css.contains(&format!("--ld-text-{}: {}px;", name, size)),
            "missing size for {name}: {css}"
        );
        assert!(
            css.contains(&format!("--ld-line-{}: {}px;", name, line)),
            "missing line height for {name}: {css}"
        );
    }
}

#[test]
fn every_type_ramp_class_pins_an_explicit_line_height() {
    // The point of the ticket: no ramp step may inherit the browser or
    // Tailwind default, or a stack of lines drifts off the grid.
    let css = ui_tokens_css();
    for (name, _, _) in TYPE_STEPS {
        let rule = format!(
            ".ld-text-{name} {{\n  font-size: var(--ld-text-{name});\n  line-height: var(--ld-line-{name});\n}}"
        );
        assert!(
            css.contains(&rule),
            "missing or partial rule for {name}: {css}"
        );
    }
}

#[test]
fn line_heights_land_on_the_4px_grid() {
    for (name, _, line) in TYPE_STEPS {
        assert_eq!(
            line % 4.0,
            0.0,
            "line height for {name} ({line}px) is off-grid"
        );
    }
}

#[test]
fn a_stack_of_n_body_lines_is_exactly_n_times_20px() {
    // The acceptance criterion stated in ldui-d14: a paragraph of body text
    // occupies N * 20px, not N * (font metric).
    let line = ui_tokens::typography::LINE_BODY;
    assert_eq!(line, 20.0);
    for n in 1..=10u32 {
        let stack = line * n as f32;
        assert_eq!(stack, (n * 20) as f32);
        assert_eq!(stack % 4.0, 0.0, "{n} lines lands off the grid");
    }
}

#[test]
fn line_height_always_clears_its_font_size() {
    // A line box smaller than the glyph size clips descenders.
    for (name, size, line) in TYPE_STEPS {
        assert!(
            line >= size,
            "{name}: line height {line}px is smaller than font size {size}px"
        );
    }
}

#[test]
fn type_ramp_mirrors_ui_tokens_exactly() {
    let sizes: Vec<f32> = TYPE_STEPS.iter().map(|(_, s, _)| *s).collect();
    let lines: Vec<f32> = TYPE_STEPS.iter().map(|(_, _, l)| *l).collect();
    assert_eq!(sizes, ui_tokens::typography::RAMP.to_vec());
    assert_eq!(lines, ui_tokens::typography::LINE_RAMP.to_vec());
}

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
