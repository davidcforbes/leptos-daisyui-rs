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
// submit/reset button actually gets. This is what makes "disabled and
// loading submit buttons cannot submit" true: a native `<button disabled>`
// dispatches no click event at all, so proving this function is `||` proves
// both inputs independently block activation.
#[test]
fn test_resolve_native_disabled_false_when_neither_set() {
    assert!(!resolve_native_disabled(false, false));
}

#[test]
fn test_resolve_native_disabled_true_when_only_disabled() {
    assert!(resolve_native_disabled(true, false));
}

#[test]
fn test_resolve_native_disabled_true_when_only_loading() {
    assert!(resolve_native_disabled(false, true));
}

#[test]
fn test_resolve_native_disabled_true_when_both_set() {
    assert!(resolve_native_disabled(true, true));
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
