use super::*;

// ToggleColor tests
#[test]
fn test_toggle_color_default() {
    let color = ToggleColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_toggle_color_primary() {
    assert_eq!(ToggleColor::Primary.as_str(), "toggle-primary");
}

#[test]
fn test_toggle_color_secondary() {
    assert_eq!(ToggleColor::Secondary.as_str(), "toggle-secondary");
}

#[test]
fn test_toggle_color_accent() {
    assert_eq!(ToggleColor::Accent.as_str(), "toggle-accent");
}

#[test]
fn test_toggle_color_success() {
    assert_eq!(ToggleColor::Success.as_str(), "toggle-success");
}

#[test]
fn test_toggle_color_warning() {
    assert_eq!(ToggleColor::Warning.as_str(), "toggle-warning");
}

#[test]
fn test_toggle_color_info() {
    assert_eq!(ToggleColor::Info.as_str(), "toggle-info");
}

#[test]
fn test_toggle_color_error() {
    assert_eq!(ToggleColor::Error.as_str(), "toggle-error");
}

#[test]
fn test_toggle_color_clone() {
    let c1 = ToggleColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_toggle_color_debug() {
    let color = ToggleColor::Success;
    assert!(format!("{:?}", color).contains("Success"));
}

// ToggleSize tests
#[test]
fn test_toggle_size_default() {
    let size = ToggleSize::default();
    assert_eq!(size.as_str(), "toggle-md");
}

#[test]
fn test_toggle_size_xs() {
    assert_eq!(ToggleSize::Xs.as_str(), "toggle-xs");
}

#[test]
fn test_toggle_size_sm() {
    assert_eq!(ToggleSize::Sm.as_str(), "toggle-sm");
}

#[test]
fn test_toggle_size_md() {
    assert_eq!(ToggleSize::Md.as_str(), "toggle-md");
}

#[test]
fn test_toggle_size_lg() {
    assert_eq!(ToggleSize::Lg.as_str(), "toggle-lg");
}

#[test]
fn test_toggle_size_xl() {
    assert_eq!(ToggleSize::Xl.as_str(), "toggle-xl");
}

#[test]
fn test_toggle_size_clone() {
    let s1 = ToggleSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_toggle_size_debug() {
    let size = ToggleSize::Xs;
    assert!(format!("{:?}", size).contains("Xs"));
}

#[test]
fn test_all_toggle_colors_return_valid_classes() {
    let variants = vec![
        (ToggleColor::Default, ""),
        (ToggleColor::Primary, "toggle-primary"),
        (ToggleColor::Secondary, "toggle-secondary"),
        (ToggleColor::Accent, "toggle-accent"),
        (ToggleColor::Success, "toggle-success"),
        (ToggleColor::Warning, "toggle-warning"),
        (ToggleColor::Info, "toggle-info"),
        (ToggleColor::Error, "toggle-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_toggle_sizes_return_valid_classes() {
    let variants = vec![
        (ToggleSize::Xs, "toggle-xs"),
        (ToggleSize::Sm, "toggle-sm"),
        (ToggleSize::Md, "toggle-md"),
        (ToggleSize::Lg, "toggle-lg"),
        (ToggleSize::Xl, "toggle-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
