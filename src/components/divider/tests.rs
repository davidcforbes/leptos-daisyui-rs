use super::*;

// DividerColor tests
#[test]
fn test_divider_color_default() {
    let val = DividerColor::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_divider_color_neutral() {
    let val = DividerColor::Neutral;
    assert_eq!(val.as_str(), "divider-neutral");
}

#[test]
fn test_divider_color_primary() {
    let val = DividerColor::Primary;
    assert_eq!(val.as_str(), "divider-primary");
}

#[test]
fn test_divider_color_secondary() {
    let val = DividerColor::Secondary;
    assert_eq!(val.as_str(), "divider-secondary");
}

#[test]
fn test_divider_color_accent() {
    let val = DividerColor::Accent;
    assert_eq!(val.as_str(), "divider-accent");
}

#[test]
fn test_divider_color_success() {
    let val = DividerColor::Success;
    assert_eq!(val.as_str(), "divider-success");
}

#[test]
fn test_divider_color_warning() {
    let val = DividerColor::Warning;
    assert_eq!(val.as_str(), "divider-warning");
}

#[test]
fn test_divider_color_info() {
    let val = DividerColor::Info;
    assert_eq!(val.as_str(), "divider-info");
}

#[test]
fn test_divider_color_error() {
    let val = DividerColor::Error;
    assert_eq!(val.as_str(), "divider-error");
}

#[test]
fn test_divider_color_clone() {
    let v1 = DividerColor::Primary;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_divider_color_debug() {
    let val = DividerColor::Success;
    assert!(format!("{:?}", val).contains("Success"));
}

// DividerDirection tests
#[test]
fn test_divider_direction_default() {
    let val = DividerDirection::default();
    assert_eq!(val.as_str(), "divider-horizontal");
}

#[test]
fn test_divider_direction_horizontal() {
    let val = DividerDirection::Horizontal;
    assert_eq!(val.as_str(), "divider-horizontal");
}

#[test]
fn test_divider_direction_vertical() {
    let val = DividerDirection::Vertical;
    assert_eq!(val.as_str(), "divider-vertical");
}

#[test]
fn test_divider_direction_clone() {
    let v1 = DividerDirection::Vertical;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_divider_direction_debug() {
    let val = DividerDirection::Vertical;
    assert!(format!("{:?}", val).contains("Vertical"));
}

// DividerPlacement tests
#[test]
fn test_divider_placement_default() {
    let val = DividerPlacement::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_divider_placement_start() {
    let val = DividerPlacement::Start;
    assert_eq!(val.as_str(), "divider-start");
}

#[test]
fn test_divider_placement_end() {
    let val = DividerPlacement::End;
    assert_eq!(val.as_str(), "divider-end");
}

#[test]
fn test_divider_placement_clone() {
    let v1 = DividerPlacement::Start;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_divider_placement_debug() {
    let val = DividerPlacement::End;
    assert!(format!("{:?}", val).contains("End"));
}

// Comprehensive coverage tests
#[test]
fn test_all_divider_colors_return_valid_classes() {
    let variants = vec![
        (DividerColor::Default, ""),
        (DividerColor::Neutral, "divider-neutral"),
        (DividerColor::Primary, "divider-primary"),
        (DividerColor::Secondary, "divider-secondary"),
        (DividerColor::Accent, "divider-accent"),
        (DividerColor::Success, "divider-success"),
        (DividerColor::Warning, "divider-warning"),
        (DividerColor::Info, "divider-info"),
        (DividerColor::Error, "divider-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_divider_directions_return_valid_classes() {
    let variants = vec![
        (DividerDirection::Horizontal, "divider-horizontal"),
        (DividerDirection::Vertical, "divider-vertical"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_divider_placements_return_valid_classes() {
    let variants = vec![
        (DividerPlacement::Default, ""),
        (DividerPlacement::Start, "divider-start"),
        (DividerPlacement::End, "divider-end"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
