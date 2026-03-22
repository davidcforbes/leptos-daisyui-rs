use super::*;

// TooltipPosition tests
#[test]
fn test_tooltip_position_default() {
    let pos = TooltipPosition::default();
    assert_eq!(pos.as_str(), "tooltip-top");
}

#[test]
fn test_tooltip_position_top() {
    assert_eq!(TooltipPosition::Top.as_str(), "tooltip-top");
}

#[test]
fn test_tooltip_position_bottom() {
    assert_eq!(TooltipPosition::Bottom.as_str(), "tooltip-bottom");
}

#[test]
fn test_tooltip_position_left() {
    assert_eq!(TooltipPosition::Left.as_str(), "tooltip-left");
}

#[test]
fn test_tooltip_position_right() {
    assert_eq!(TooltipPosition::Right.as_str(), "tooltip-right");
}

#[test]
fn test_tooltip_position_clone() {
    let p1 = TooltipPosition::Left;
    let p2 = p1.clone();
    assert_eq!(p1.as_str(), p2.as_str());
}

#[test]
fn test_tooltip_position_debug() {
    let pos = TooltipPosition::Right;
    assert!(format!("{:?}", pos).contains("Right"));
}

// TooltipColor tests
#[test]
fn test_tooltip_color_default() {
    let color = TooltipColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_tooltip_color_neutral() {
    assert_eq!(TooltipColor::Neutral.as_str(), "tooltip-neutral");
}

#[test]
fn test_tooltip_color_primary() {
    assert_eq!(TooltipColor::Primary.as_str(), "tooltip-primary");
}

#[test]
fn test_tooltip_color_secondary() {
    assert_eq!(TooltipColor::Secondary.as_str(), "tooltip-secondary");
}

#[test]
fn test_tooltip_color_accent() {
    assert_eq!(TooltipColor::Accent.as_str(), "tooltip-accent");
}

#[test]
fn test_tooltip_color_info() {
    assert_eq!(TooltipColor::Info.as_str(), "tooltip-info");
}

#[test]
fn test_tooltip_color_success() {
    assert_eq!(TooltipColor::Success.as_str(), "tooltip-success");
}

#[test]
fn test_tooltip_color_warning() {
    assert_eq!(TooltipColor::Warning.as_str(), "tooltip-warning");
}

#[test]
fn test_tooltip_color_error() {
    assert_eq!(TooltipColor::Error.as_str(), "tooltip-error");
}

#[test]
fn test_tooltip_color_clone() {
    let c1 = TooltipColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_tooltip_color_debug() {
    let color = TooltipColor::Warning;
    assert!(format!("{:?}", color).contains("Warning"));
}

#[test]
fn test_all_tooltip_positions_return_valid_classes() {
    let variants = vec![
        (TooltipPosition::Top, "tooltip-top"),
        (TooltipPosition::Bottom, "tooltip-bottom"),
        (TooltipPosition::Left, "tooltip-left"),
        (TooltipPosition::Right, "tooltip-right"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_tooltip_colors_return_valid_classes() {
    let variants = vec![
        (TooltipColor::Default, ""),
        (TooltipColor::Neutral, "tooltip-neutral"),
        (TooltipColor::Primary, "tooltip-primary"),
        (TooltipColor::Secondary, "tooltip-secondary"),
        (TooltipColor::Accent, "tooltip-accent"),
        (TooltipColor::Info, "tooltip-info"),
        (TooltipColor::Success, "tooltip-success"),
        (TooltipColor::Warning, "tooltip-warning"),
        (TooltipColor::Error, "tooltip-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
