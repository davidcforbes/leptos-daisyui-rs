use super::*;

// ButtonColor tests
#[test]
fn test_button_color_default() {
    let color = ButtonColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_button_color_neutral() {
    let color = ButtonColor::Neutral;
    assert_eq!(color.as_str(), "btn-neutral");
}

#[test]
fn test_button_color_primary() {
    let color = ButtonColor::Primary;
    assert_eq!(color.as_str(), "btn-primary");
}

#[test]
fn test_button_color_secondary() {
    let color = ButtonColor::Secondary;
    assert_eq!(color.as_str(), "btn-secondary");
}

#[test]
fn test_button_color_accent() {
    let color = ButtonColor::Accent;
    assert_eq!(color.as_str(), "btn-accent");
}

#[test]
fn test_button_color_info() {
    let color = ButtonColor::Info;
    assert_eq!(color.as_str(), "btn-info");
}

#[test]
fn test_button_color_success() {
    let color = ButtonColor::Success;
    assert_eq!(color.as_str(), "btn-success");
}

#[test]
fn test_button_color_warning() {
    let color = ButtonColor::Warning;
    assert_eq!(color.as_str(), "btn-warning");
}

#[test]
fn test_button_color_error() {
    let color = ButtonColor::Error;
    assert_eq!(color.as_str(), "btn-error");
}

#[test]
fn test_button_color_clone() {
    let color1 = ButtonColor::Primary;
    let color2 = color1.clone();
    assert_eq!(color1.as_str(), color2.as_str());
}

#[test]
fn test_button_color_debug() {
    let color = ButtonColor::Success;
    assert!(format!("{:?}", color).contains("Success"));
}

// ButtonStyle tests
#[test]
fn test_button_style_default() {
    let style = ButtonStyle::default();
    assert_eq!(style.as_str(), "");
}

#[test]
fn test_button_style_outline() {
    let style = ButtonStyle::Outline;
    assert_eq!(style.as_str(), "btn-outline");
}

#[test]
fn test_button_style_dash() {
    let style = ButtonStyle::Dash;
    assert_eq!(style.as_str(), "btn-dash");
}

#[test]
fn test_button_style_soft() {
    let style = ButtonStyle::Soft;
    assert_eq!(style.as_str(), "btn-soft");
}

#[test]
fn test_button_style_ghost() {
    let style = ButtonStyle::Ghost;
    assert_eq!(style.as_str(), "btn-ghost");
}

#[test]
fn test_button_style_link() {
    let style = ButtonStyle::Link;
    assert_eq!(style.as_str(), "btn-link");
}

#[test]
fn test_button_style_clone() {
    let style1 = ButtonStyle::Outline;
    let style2 = style1.clone();
    assert_eq!(style1.as_str(), style2.as_str());
}

#[test]
fn test_button_style_debug() {
    let style = ButtonStyle::Ghost;
    assert!(format!("{:?}", style).contains("Ghost"));
}

// ButtonSize tests
#[test]
fn test_button_size_default() {
    let size = ButtonSize::default();
    assert_eq!(size.as_str(), "btn-md");
}

#[test]
fn test_button_size_xs() {
    let size = ButtonSize::Xs;
    assert_eq!(size.as_str(), "btn-xs");
}

#[test]
fn test_button_size_sm() {
    let size = ButtonSize::Sm;
    assert_eq!(size.as_str(), "btn-sm");
}

#[test]
fn test_button_size_md() {
    let size = ButtonSize::Md;
    assert_eq!(size.as_str(), "btn-md");
}

#[test]
fn test_button_size_lg() {
    let size = ButtonSize::Lg;
    assert_eq!(size.as_str(), "btn-lg");
}

#[test]
fn test_button_size_xl() {
    let size = ButtonSize::Xl;
    assert_eq!(size.as_str(), "btn-xl");
}

#[test]
fn test_button_size_clone() {
    let size1 = ButtonSize::Lg;
    let size2 = size1.clone();
    assert_eq!(size1.as_str(), size2.as_str());
}

#[test]
fn test_button_size_debug() {
    let size = ButtonSize::Xl;
    assert!(format!("{:?}", size).contains("Xl"));
}

// ButtonShape tests
#[test]
fn test_button_shape_default() {
    let shape = ButtonShape::default();
    assert_eq!(shape.as_str(), "");
}

#[test]
fn test_button_shape_wide() {
    let shape = ButtonShape::Wide;
    assert_eq!(shape.as_str(), "btn-wide");
}

#[test]
fn test_button_shape_block() {
    let shape = ButtonShape::Block;
    assert_eq!(shape.as_str(), "btn-block");
}

#[test]
fn test_button_shape_square() {
    let shape = ButtonShape::Square;
    assert_eq!(shape.as_str(), "btn-square");
}

#[test]
fn test_button_shape_circle() {
    let shape = ButtonShape::Circle;
    assert_eq!(shape.as_str(), "btn-circle");
}

#[test]
fn test_button_shape_clone() {
    let shape1 = ButtonShape::Circle;
    let shape2 = shape1.clone();
    assert_eq!(shape1.as_str(), shape2.as_str());
}

#[test]
fn test_button_shape_debug() {
    let shape = ButtonShape::Square;
    assert!(format!("{:?}", shape).contains("Square"));
}

// ButtonType tests (ldui-9vs) — the emitted native `type` attribute.
#[test]
fn test_button_type_default_is_button() {
    let button_type = ButtonType::default();
    assert_eq!(button_type, ButtonType::Button);
    assert_eq!(button_type.as_str(), "button");
}

#[test]
fn test_button_type_submit() {
    assert_eq!(ButtonType::Submit.as_str(), "submit");
}

#[test]
fn test_button_type_reset() {
    assert_eq!(ButtonType::Reset.as_str(), "reset");
}

#[test]
fn test_button_type_clone_and_copy() {
    let t1 = ButtonType::Submit;
    let t2 = t1; // Copy, not a move — proves the `Copy` derive is present.
    assert_eq!(t1, t2);
}

#[test]
fn test_button_type_debug() {
    assert!(format!("{:?}", ButtonType::Reset).contains("Reset"));
}

#[test]
fn test_button_type_partial_eq_distinguishes_all_variants() {
    assert_ne!(ButtonType::Button, ButtonType::Submit);
    assert_ne!(ButtonType::Button, ButtonType::Reset);
    assert_ne!(ButtonType::Submit, ButtonType::Reset);
}

#[test]
fn test_all_button_types_return_valid_type_attribute_values() {
    let variants = vec![
        (ButtonType::Button, "button"),
        (ButtonType::Submit, "submit"),
        (ButtonType::Reset, "reset"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

// `resolve_native_disabled` (ldui-9vs) — the DOM `disabled` attribute a
// button actually gets. `disabled` alone always wins, regardless of type.
// `loading` alone only disables a Submit button (review ruling: broadly
// disabling every loading button regressed non-submit loading buttons from
// "spinner, still clickable/focusable" to "inert, not tab-reachable", which
// the bead's acceptance never asked for). This is what makes "disabled and
// loading SUBMIT buttons cannot submit" true, while a loading
// Button/Reset button stays interactive unless `disabled` is also passed.
#[test]
fn test_resolve_native_disabled_false_when_nothing_set() {
    assert!(!resolve_native_disabled(false, false, ButtonType::Submit));
    assert!(!resolve_native_disabled(false, false, ButtonType::Button));
    assert!(!resolve_native_disabled(false, false, ButtonType::Reset));
}

#[test]
fn test_resolve_native_disabled_explicit_disabled_wins_regardless_of_type() {
    assert!(resolve_native_disabled(true, false, ButtonType::Submit));
    assert!(resolve_native_disabled(true, false, ButtonType::Button));
    assert!(resolve_native_disabled(true, false, ButtonType::Reset));
    // ...and stays true even loading, for every type.
    assert!(resolve_native_disabled(true, true, ButtonType::Submit));
    assert!(resolve_native_disabled(true, true, ButtonType::Button));
    assert!(resolve_native_disabled(true, true, ButtonType::Reset));
}

#[test]
fn test_resolve_native_disabled_loading_alone_disables_only_submit() {
    assert!(resolve_native_disabled(false, true, ButtonType::Submit));
}

#[test]
fn test_resolve_native_disabled_loading_alone_leaves_button_and_reset_enabled() {
    // The review-ruling fix (ldui-9vs): a loading Button/Reset must NOT be
    // natively disabled — it keeps its historical clickable/focusable,
    // spinner-only behavior.
    assert!(!resolve_native_disabled(false, true, ButtonType::Button));
    assert!(!resolve_native_disabled(false, true, ButtonType::Reset));
}

/// Full 2x2x3 matrix against the documented formula, so a future edit to
/// `resolve_native_disabled` that drifts from
/// `disabled || (loading && button_type == Submit)` fails here rather than
/// only in the targeted cases above.
#[test]
fn test_resolve_native_disabled_matches_the_documented_formula_for_every_combination() {
    for &disabled in &[false, true] {
        for &loading in &[false, true] {
            for &button_type in &[ButtonType::Button, ButtonType::Submit, ButtonType::Reset] {
                let expected = disabled || (loading && button_type == ButtonType::Submit);
                assert_eq!(
                    resolve_native_disabled(disabled, loading, button_type),
                    expected,
                    "disabled={disabled}, loading={loading}, button_type={button_type:?}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// `disabled_reason` (ldui-p82h)
// ---------------------------------------------------------------------------
//
// This crate has no DOM/SSR renderer in native tests (see the module doc on
// `filter_bar/tests.rs`), so the reason handling is split into pure
// functions asserted directly, plus source-level guards on the wiring in the
// `view!` -- the same shape `page_quick_actions.rs` uses for its own
// contract tests.

const BUTTON_VIEW_SRC: &str = include_str!("component.rs");

/// The `Button` component body only, so a guard cannot be satisfied by
/// `LinkButton` or by the doc comment above the component.
fn button_component_src() -> &'static str {
    BUTTON_VIEW_SRC
        .split_once("pub fn Button(")
        .expect("Button component source")
        .1
        .split_once("pub fn LinkButton(")
        .expect("LinkButton follows Button")
        .0
}

#[test]
fn disabled_reason_text_is_kept_and_trimmed() {
    assert_eq!(
        resolve_disabled_reason("  No rows to export  ".to_owned()).as_deref(),
        Some("No rows to export")
    );
    assert_eq!(
        resolve_disabled_reason("Already complete".to_owned()).as_deref(),
        Some("Already complete")
    );
}

/// A blank reason is the same as none: it must not disable the button and
/// must not produce a hint, so one reactive signal can drive both the state
/// and the explanation.
#[test]
fn blank_disabled_reason_is_no_reason() {
    assert_eq!(resolve_disabled_reason(String::new()), None);
    assert_eq!(resolve_disabled_reason("   ".to_owned()), None);
    assert_eq!(resolve_disabled_reason("\n\t".to_owned()), None);
}

#[test]
fn minted_reason_ids_are_prefixed_and_unique() {
    let a = next_button_reason_id();
    let b = next_button_reason_id();
    assert!(a.starts_with("ld-btn-reason-"), "{a}");
    assert!(b.starts_with("ld-btn-reason-"), "{b}");
    assert_ne!(a, b);
}

/// The prop is the crate's idiomatic optional-signal form (`Select::value`,
/// `EntityTable::focus_scope`), so a caller passes a `Signal<String>` and
/// omits it freely.
#[test]
fn disabled_reason_prop_is_an_optional_signal() {
    let src = button_component_src();
    assert!(
        src.contains("#[prop(optional, into)]\n    disabled_reason: Signal<String>,")
            || src.contains("#[prop(optional, into)]\r\n    disabled_reason: Signal<String>,"),
        "disabled_reason must be an optional `Signal<String>` (blank = no reason)"
    );
}

/// A present reason disables the button through the SAME resolver that
/// `disabled` and `loading` go through, so a reasoned button can never be
/// natively enabled while visually `.btn-disabled`, or vice versa.
#[test]
fn a_present_reason_disables_natively_and_visually() {
    let src = button_component_src();
    assert!(
        src.contains("let is_disabled = move || disabled.get() || reason_present();"),
        "the combined disabled state must fold the reason in"
    );
    assert!(
        src.contains(
            "disabled=move || resolve_native_disabled(is_disabled(), loading.get(), button_type.get())"
        ),
        "the native attribute must be resolved from the combined state"
    );
    assert!(
        src.contains("class:btn-disabled=is_disabled"),
        "the visual class must follow the combined state"
    );
}

/// The reason reaches assistive technology as a DESCRIPTION (never the
/// name) and sighted mouse users as the native tooltip.
#[test]
fn a_present_reason_is_described_and_titled() {
    let src = button_component_src();
    assert!(
        src.contains("aria-describedby=move || reason_present().then(|| describedby_id.clone())"),
        "aria-describedby must reference the minted id only while a reason is present"
    );
    assert!(
        src.contains("title=reason_text"),
        "the native title carries the reason"
    );
    assert!(
        src.contains(
            r#"<span class="sr-only" aria-hidden="true" id=id data-button-disabled-reason="true">"#
        ),
        "the hint is a visually-hidden, name-excluded span carrying the minted id"
    );
}

/// `disabled=true` with no reason is not a panic and not silently fine: it
/// is stamped so the audit's `disabled-without-reason` rule can report it.
#[test]
fn disabled_without_a_reason_is_stamped_for_the_audit() {
    let src = button_component_src();
    assert!(
        src.contains("data-disabled-without-reason=move || {"),
        "the audit marker must be emitted reactively"
    );
    assert!(
        src.contains(r#"(disabled.get() && !reason_present()).then_some("true")"#),
        "the marker is present exactly when disabled without a reason"
    );
    assert!(
        !src.contains("panic!(") && !src.contains("unreachable!("),
        "a missing reason must never panic"
    );
}

/// The `sr-only` hint class must be discoverable by Tailwind's source scan
/// (the demo scans `../src/**/*.rs`), so it is listed in the documented
/// `@source inline(...)` line as well as emitted literally.
#[test]
fn sr_only_is_in_the_documented_source_inline_list() {
    let doc_line = BUTTON_VIEW_SRC
        .lines()
        .find(|line| line.contains("@source inline(\"btn btn-neutral"))
        .expect("Button's @source inline line");
    assert!(doc_line.contains(" sr-only"), "{doc_line}");
}

// Comprehensive enum variant coverage tests
#[test]
fn test_all_button_colors_return_valid_classes() {
    let colors = vec![
        (ButtonColor::Default, ""),
        (ButtonColor::Neutral, "btn-neutral"),
        (ButtonColor::Primary, "btn-primary"),
        (ButtonColor::Secondary, "btn-secondary"),
        (ButtonColor::Accent, "btn-accent"),
        (ButtonColor::Info, "btn-info"),
        (ButtonColor::Success, "btn-success"),
        (ButtonColor::Warning, "btn-warning"),
        (ButtonColor::Error, "btn-error"),
    ];

    for (color, expected) in colors {
        assert_eq!(color.as_str(), expected);
    }
}

#[test]
fn test_all_button_styles_return_valid_classes() {
    let styles = vec![
        (ButtonStyle::Default, ""),
        (ButtonStyle::Outline, "btn-outline"),
        (ButtonStyle::Dash, "btn-dash"),
        (ButtonStyle::Soft, "btn-soft"),
        (ButtonStyle::Ghost, "btn-ghost"),
        (ButtonStyle::Link, "btn-link"),
    ];

    for (style, expected) in styles {
        assert_eq!(style.as_str(), expected);
    }
}

#[test]
fn test_all_button_sizes_return_valid_classes() {
    let sizes = vec![
        (ButtonSize::Xs, "btn-xs"),
        (ButtonSize::Sm, "btn-sm"),
        (ButtonSize::Md, "btn-md"),
        (ButtonSize::Lg, "btn-lg"),
        (ButtonSize::Xl, "btn-xl"),
    ];

    for (size, expected) in sizes {
        assert_eq!(size.as_str(), expected);
    }
}

#[test]
fn test_all_button_shapes_return_valid_classes() {
    let shapes = vec![
        (ButtonShape::Default, ""),
        (ButtonShape::Wide, "btn-wide"),
        (ButtonShape::Block, "btn-block"),
        (ButtonShape::Square, "btn-square"),
        (ButtonShape::Circle, "btn-circle"),
    ];

    for (shape, expected) in shapes {
        assert_eq!(shape.as_str(), expected);
    }
}
