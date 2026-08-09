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

// `max` normalization (ldui-c1s)
#[test]
fn test_progress_max_passes_through_positive_finite() {
    assert_eq!(progress_max(100.0), 100.0);
    assert_eq!(progress_max(1.0), 1.0);
    assert!((progress_max(0.5) - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_progress_max_falls_back_to_html_default_when_non_positive() {
    assert_eq!(progress_max(0.0), 1.0);
    assert_eq!(progress_max(-10.0), 1.0);
}

#[test]
fn test_progress_max_falls_back_to_html_default_when_non_finite() {
    assert_eq!(progress_max(f64::NAN), 1.0);
    assert_eq!(progress_max(f64::INFINITY), 1.0);
    assert_eq!(progress_max(f64::NEG_INFINITY), 1.0);
}

// `value` resolution (ldui-c1s). The load-bearing case is the first one: an
// unset value MUST stay `None` so the attribute is omitted and the bar remains
// indeterminate, rather than collapsing to a determinate `value=0`.
#[test]
fn test_progress_value_unset_is_indeterminate() {
    assert_eq!(progress_value(None, 100.0), None);
    assert_eq!(progress_value(None, 1.0), None);
}

#[test]
fn test_progress_value_zero_is_determinate_not_indeterminate() {
    assert_eq!(progress_value(Some(0.0), 100.0), Some(0.0));
}

#[test]
fn test_progress_value_passes_through_in_range() {
    assert_eq!(progress_value(Some(70.0), 100.0), Some(70.0));
    assert_eq!(progress_value(Some(0.7), 1.0), Some(0.7));
    assert_eq!(progress_value(Some(100.0), 100.0), Some(100.0));
}

#[test]
fn test_progress_value_clamps_below_zero() {
    assert_eq!(progress_value(Some(-5.0), 100.0), Some(0.0));
}

#[test]
fn test_progress_value_clamps_above_max() {
    assert_eq!(progress_value(Some(150.0), 100.0), Some(100.0));
}

#[test]
fn test_progress_value_clamps_against_normalized_max() {
    // A degenerate max normalizes to 1.0 first, so the value clamps to 1.0.
    assert_eq!(progress_value(Some(5.0), 0.0), Some(1.0));
}

#[test]
fn test_progress_value_non_finite_falls_back_to_indeterminate() {
    assert_eq!(progress_value(Some(f64::NAN), 100.0), None);
    assert_eq!(progress_value(Some(f64::INFINITY), 100.0), None);
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
