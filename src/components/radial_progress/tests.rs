use super::*;

// RadialProgressColor tests
#[test]
fn test_radial_progress_color_default() {
    let color = RadialProgressColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_radial_progress_color_primary() {
    assert_eq!(RadialProgressColor::Primary.as_str(), "text-primary");
}

#[test]
fn test_radial_progress_color_secondary() {
    assert_eq!(RadialProgressColor::Secondary.as_str(), "text-secondary");
}

#[test]
fn test_radial_progress_color_accent() {
    assert_eq!(RadialProgressColor::Accent.as_str(), "text-accent");
}

#[test]
fn test_radial_progress_color_success() {
    assert_eq!(RadialProgressColor::Success.as_str(), "text-success");
}

#[test]
fn test_radial_progress_color_info() {
    assert_eq!(RadialProgressColor::Info.as_str(), "text-info");
}

#[test]
fn test_radial_progress_color_warning() {
    assert_eq!(RadialProgressColor::Warning.as_str(), "text-warning");
}

#[test]
fn test_radial_progress_color_error() {
    assert_eq!(RadialProgressColor::Error.as_str(), "text-error");
}

#[test]
fn test_radial_progress_color_clone() {
    let c1 = RadialProgressColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_radial_progress_color_debug() {
    let color = RadialProgressColor::Success;
    assert!(format!("{:?}", color).contains("Success"));
}

#[test]
fn test_all_radial_progress_colors_return_valid_classes() {
    let variants = vec![
        (RadialProgressColor::Default, ""),
        (RadialProgressColor::Primary, "text-primary"),
        (RadialProgressColor::Secondary, "text-secondary"),
        (RadialProgressColor::Accent, "text-accent"),
        (RadialProgressColor::Success, "text-success"),
        (RadialProgressColor::Info, "text-info"),
        (RadialProgressColor::Warning, "text-warning"),
        (RadialProgressColor::Error, "text-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
