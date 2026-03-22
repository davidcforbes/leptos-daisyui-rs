use super::*;

// ProgressColor tests
#[test]
fn test_progress_color_default() {
    let val = ProgressColor::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_progress_color_primary() {
    let val = ProgressColor::Primary;
    assert_eq!(val.as_str(), "progress-primary");
}

#[test]
fn test_progress_color_secondary() {
    let val = ProgressColor::Secondary;
    assert_eq!(val.as_str(), "progress-secondary");
}

#[test]
fn test_progress_color_accent() {
    let val = ProgressColor::Accent;
    assert_eq!(val.as_str(), "progress-accent");
}

#[test]
fn test_progress_color_success() {
    let val = ProgressColor::Success;
    assert_eq!(val.as_str(), "progress-success");
}

#[test]
fn test_progress_color_info() {
    let val = ProgressColor::Info;
    assert_eq!(val.as_str(), "progress-info");
}

#[test]
fn test_progress_color_warning() {
    let val = ProgressColor::Warning;
    assert_eq!(val.as_str(), "progress-warning");
}

#[test]
fn test_progress_color_error() {
    let val = ProgressColor::Error;
    assert_eq!(val.as_str(), "progress-error");
}

#[test]
fn test_progress_color_clone() {
    let v1 = ProgressColor::Primary;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_progress_color_debug() {
    let val = ProgressColor::Success;
    assert!(format!("{:?}", val).contains("Success"));
}

// Comprehensive coverage test
#[test]
fn test_all_progress_colors_return_valid_classes() {
    let variants = vec![
        (ProgressColor::Default, ""),
        (ProgressColor::Primary, "progress-primary"),
        (ProgressColor::Secondary, "progress-secondary"),
        (ProgressColor::Accent, "progress-accent"),
        (ProgressColor::Success, "progress-success"),
        (ProgressColor::Info, "progress-info"),
        (ProgressColor::Warning, "progress-warning"),
        (ProgressColor::Error, "progress-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
