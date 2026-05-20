use super::*;

// RadioColor tests
#[test]
fn test_radio_color_default() {
    let color = RadioColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_radio_color_primary() {
    assert_eq!(RadioColor::Primary.as_str(), "radio-primary");
}

#[test]
fn test_radio_color_secondary() {
    assert_eq!(RadioColor::Secondary.as_str(), "radio-secondary");
}

#[test]
fn test_radio_color_accent() {
    assert_eq!(RadioColor::Accent.as_str(), "radio-accent");
}

#[test]
fn test_radio_color_success() {
    assert_eq!(RadioColor::Success.as_str(), "radio-success");
}

#[test]
fn test_radio_color_warning() {
    assert_eq!(RadioColor::Warning.as_str(), "radio-warning");
}

#[test]
fn test_radio_color_info() {
    assert_eq!(RadioColor::Info.as_str(), "radio-info");
}

#[test]
fn test_radio_color_error() {
    assert_eq!(RadioColor::Error.as_str(), "radio-error");
}

#[test]
fn test_radio_color_clone() {
    let c1 = RadioColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_radio_color_debug() {
    let color = RadioColor::Accent;
    assert!(format!("{:?}", color).contains("Accent"));
}

// RadioSize tests
#[test]
fn test_radio_size_default() {
    let size = RadioSize::default();
    assert_eq!(size.as_str(), "radio-md");
}

#[test]
fn test_radio_size_xs() {
    assert_eq!(RadioSize::Xs.as_str(), "radio-xs");
}

#[test]
fn test_radio_size_sm() {
    assert_eq!(RadioSize::Sm.as_str(), "radio-sm");
}

#[test]
fn test_radio_size_md() {
    assert_eq!(RadioSize::Md.as_str(), "radio-md");
}

#[test]
fn test_radio_size_lg() {
    assert_eq!(RadioSize::Lg.as_str(), "radio-lg");
}

#[test]
fn test_radio_size_xl() {
    assert_eq!(RadioSize::Xl.as_str(), "radio-xl");
}

#[test]
fn test_radio_size_clone() {
    let s1 = RadioSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_radio_size_debug() {
    let size = RadioSize::Xl;
    assert!(format!("{:?}", size).contains("Xl"));
}

#[test]
fn test_all_radio_colors_return_valid_classes() {
    let variants = vec![
        (RadioColor::Default, ""),
        (RadioColor::Primary, "radio-primary"),
        (RadioColor::Secondary, "radio-secondary"),
        (RadioColor::Accent, "radio-accent"),
        (RadioColor::Success, "radio-success"),
        (RadioColor::Warning, "radio-warning"),
        (RadioColor::Info, "radio-info"),
        (RadioColor::Error, "radio-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_radio_sizes_return_valid_classes() {
    let variants = vec![
        (RadioSize::Xs, "radio-xs"),
        (RadioSize::Sm, "radio-sm"),
        (RadioSize::Md, "radio-md"),
        (RadioSize::Lg, "radio-lg"),
        (RadioSize::Xl, "radio-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
