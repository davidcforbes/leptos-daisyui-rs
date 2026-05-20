use super::*;

// SliderColor tests
#[test]
fn test_slider_color_default() {
    let color = SliderColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_slider_color_primary() {
    assert_eq!(SliderColor::Primary.as_str(), "range-primary");
}

#[test]
fn test_slider_color_secondary() {
    assert_eq!(SliderColor::Secondary.as_str(), "range-secondary");
}

#[test]
fn test_slider_color_accent() {
    assert_eq!(SliderColor::Accent.as_str(), "range-accent");
}

#[test]
fn test_slider_color_success() {
    assert_eq!(SliderColor::Success.as_str(), "range-success");
}

#[test]
fn test_slider_color_warning() {
    assert_eq!(SliderColor::Warning.as_str(), "range-warning");
}

#[test]
fn test_slider_color_info() {
    assert_eq!(SliderColor::Info.as_str(), "range-info");
}

#[test]
fn test_slider_color_error() {
    assert_eq!(SliderColor::Error.as_str(), "range-error");
}

#[test]
fn test_slider_color_clone() {
    let c1 = SliderColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_slider_color_debug() {
    let color = SliderColor::Accent;
    assert!(format!("{:?}", color).contains("Accent"));
}

// SliderSize tests
#[test]
fn test_slider_size_default() {
    let size = SliderSize::default();
    assert_eq!(size.as_str(), "range-md");
}

#[test]
fn test_slider_size_extra_small() {
    assert_eq!(SliderSize::ExtraSmall.as_str(), "range-xs");
}

#[test]
fn test_slider_size_small() {
    assert_eq!(SliderSize::Small.as_str(), "range-sm");
}

#[test]
fn test_slider_size_medium() {
    assert_eq!(SliderSize::Medium.as_str(), "range-md");
}

#[test]
fn test_slider_size_large() {
    assert_eq!(SliderSize::Large.as_str(), "range-lg");
}

#[test]
fn test_slider_size_clone() {
    let s1 = SliderSize::Large;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_slider_size_debug() {
    let size = SliderSize::ExtraSmall;
    assert!(format!("{:?}", size).contains("ExtraSmall"));
}

#[test]
fn test_all_slider_colors_return_valid_classes() {
    let variants = vec![
        (SliderColor::Default, ""),
        (SliderColor::Primary, "range-primary"),
        (SliderColor::Secondary, "range-secondary"),
        (SliderColor::Accent, "range-accent"),
        (SliderColor::Success, "range-success"),
        (SliderColor::Warning, "range-warning"),
        (SliderColor::Info, "range-info"),
        (SliderColor::Error, "range-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_slider_sizes_return_valid_classes() {
    let variants = vec![
        (SliderSize::ExtraSmall, "range-xs"),
        (SliderSize::Small, "range-sm"),
        (SliderSize::Medium, "range-md"),
        (SliderSize::Large, "range-lg"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
