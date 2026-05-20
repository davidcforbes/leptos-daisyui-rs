use super::*;

// RangeColor tests
#[test]
fn test_range_color_default() {
    let color = RangeColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_range_color_primary() {
    assert_eq!(RangeColor::Primary.as_str(), "range-primary");
}

#[test]
fn test_range_color_secondary() {
    assert_eq!(RangeColor::Secondary.as_str(), "range-secondary");
}

#[test]
fn test_range_color_accent() {
    assert_eq!(RangeColor::Accent.as_str(), "range-accent");
}

#[test]
fn test_range_color_success() {
    assert_eq!(RangeColor::Success.as_str(), "range-success");
}

#[test]
fn test_range_color_warning() {
    assert_eq!(RangeColor::Warning.as_str(), "range-warning");
}

#[test]
fn test_range_color_info() {
    assert_eq!(RangeColor::Info.as_str(), "range-info");
}

#[test]
fn test_range_color_error() {
    assert_eq!(RangeColor::Error.as_str(), "range-error");
}

#[test]
fn test_range_color_clone() {
    let c1 = RangeColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_range_color_debug() {
    let color = RangeColor::Warning;
    assert!(format!("{:?}", color).contains("Warning"));
}

// RangeSize tests
#[test]
fn test_range_size_default() {
    let size = RangeSize::default();
    assert_eq!(size.as_str(), "range-md");
}

#[test]
fn test_range_size_xs() {
    assert_eq!(RangeSize::Xs.as_str(), "range-xs");
}

#[test]
fn test_range_size_sm() {
    assert_eq!(RangeSize::Sm.as_str(), "range-sm");
}

#[test]
fn test_range_size_md() {
    assert_eq!(RangeSize::Md.as_str(), "range-md");
}

#[test]
fn test_range_size_lg() {
    assert_eq!(RangeSize::Lg.as_str(), "range-lg");
}

#[test]
fn test_range_size_xl() {
    assert_eq!(RangeSize::Xl.as_str(), "range-xl");
}

#[test]
fn test_range_size_clone() {
    let s1 = RangeSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_range_size_debug() {
    let size = RangeSize::Sm;
    assert!(format!("{:?}", size).contains("Sm"));
}

#[test]
fn test_all_range_colors_return_valid_classes() {
    let variants = vec![
        (RangeColor::Default, ""),
        (RangeColor::Primary, "range-primary"),
        (RangeColor::Secondary, "range-secondary"),
        (RangeColor::Accent, "range-accent"),
        (RangeColor::Success, "range-success"),
        (RangeColor::Warning, "range-warning"),
        (RangeColor::Info, "range-info"),
        (RangeColor::Error, "range-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_range_sizes_return_valid_classes() {
    let variants = vec![
        (RangeSize::Xs, "range-xs"),
        (RangeSize::Sm, "range-sm"),
        (RangeSize::Md, "range-md"),
        (RangeSize::Lg, "range-lg"),
        (RangeSize::Xl, "range-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
