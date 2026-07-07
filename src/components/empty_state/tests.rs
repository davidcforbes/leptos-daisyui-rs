use super::*;

// EmptyStateColor tests

#[test]
fn test_empty_state_color_default() {
    let color = EmptyStateColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_empty_state_color_neutral() {
    assert_eq!(EmptyStateColor::Neutral.as_str(), "text-neutral");
}

#[test]
fn test_empty_state_color_primary() {
    assert_eq!(EmptyStateColor::Primary.as_str(), "text-primary");
}

#[test]
fn test_empty_state_color_secondary() {
    assert_eq!(EmptyStateColor::Secondary.as_str(), "text-secondary");
}

#[test]
fn test_empty_state_color_accent() {
    assert_eq!(EmptyStateColor::Accent.as_str(), "text-accent");
}

#[test]
fn test_empty_state_color_info() {
    assert_eq!(EmptyStateColor::Info.as_str(), "text-info");
}

#[test]
fn test_empty_state_color_success() {
    assert_eq!(EmptyStateColor::Success.as_str(), "text-success");
}

#[test]
fn test_empty_state_color_warning() {
    assert_eq!(EmptyStateColor::Warning.as_str(), "text-warning");
}

#[test]
fn test_empty_state_color_error() {
    assert_eq!(EmptyStateColor::Error.as_str(), "text-error");
}

#[test]
fn test_empty_state_color_clone_and_debug() {
    let c1 = EmptyStateColor::Accent;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
    assert!(format!("{:?}", c1).contains("Accent"));
}

#[test]
fn test_all_empty_state_colors_return_valid_classes() {
    let variants = vec![
        (EmptyStateColor::Default, ""),
        (EmptyStateColor::Neutral, "text-neutral"),
        (EmptyStateColor::Primary, "text-primary"),
        (EmptyStateColor::Secondary, "text-secondary"),
        (EmptyStateColor::Accent, "text-accent"),
        (EmptyStateColor::Info, "text-info"),
        (EmptyStateColor::Success, "text-success"),
        (EmptyStateColor::Warning, "text-warning"),
        (EmptyStateColor::Error, "text-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
