use super::*;

// MetricRowColor tests

#[test]
fn test_metric_row_color_default() {
    let color = MetricRowColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_metric_row_color_neutral() {
    assert_eq!(MetricRowColor::Neutral.as_str(), "text-neutral");
}

#[test]
fn test_metric_row_color_primary() {
    assert_eq!(MetricRowColor::Primary.as_str(), "text-primary");
}

#[test]
fn test_metric_row_color_secondary() {
    assert_eq!(MetricRowColor::Secondary.as_str(), "text-secondary");
}

#[test]
fn test_metric_row_color_accent() {
    assert_eq!(MetricRowColor::Accent.as_str(), "text-accent");
}

#[test]
fn test_metric_row_color_info() {
    assert_eq!(MetricRowColor::Info.as_str(), "text-info");
}

#[test]
fn test_metric_row_color_success() {
    assert_eq!(MetricRowColor::Success.as_str(), "text-success");
}

#[test]
fn test_metric_row_color_warning() {
    assert_eq!(MetricRowColor::Warning.as_str(), "text-warning");
}

#[test]
fn test_metric_row_color_error() {
    assert_eq!(MetricRowColor::Error.as_str(), "text-error");
}

#[test]
fn test_metric_row_color_clone_and_debug() {
    let c1 = MetricRowColor::Accent;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
    assert!(format!("{:?}", c1).contains("Accent"));
}

#[test]
fn test_all_metric_row_colors_return_valid_classes() {
    let variants = vec![
        (MetricRowColor::Default, ""),
        (MetricRowColor::Neutral, "text-neutral"),
        (MetricRowColor::Primary, "text-primary"),
        (MetricRowColor::Secondary, "text-secondary"),
        (MetricRowColor::Accent, "text-accent"),
        (MetricRowColor::Info, "text-info"),
        (MetricRowColor::Success, "text-success"),
        (MetricRowColor::Warning, "text-warning"),
        (MetricRowColor::Error, "text-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

// container_class tests

#[test]
fn test_container_class_row() {
    assert_eq!(
        container_class(false),
        "flex items-baseline justify-between gap-2"
    );
}

#[test]
fn test_container_class_stacked() {
    assert_eq!(container_class(true), "flex flex-col gap-1");
}

// label_class tests

#[test]
fn test_label_class_row() {
    assert_eq!(label_class(false), "text-sm opacity-60");
}

#[test]
fn test_label_class_stacked() {
    assert_eq!(label_class(true), "text-xs opacity-60");
}

// value_class tests

#[test]
fn test_value_class_row_plain() {
    assert_eq!(value_class(false, false), "text-sm text-right");
}

#[test]
fn test_value_class_row_bold() {
    assert_eq!(value_class(false, true), "text-sm font-semibold text-right");
}

#[test]
fn test_value_class_stacked_plain() {
    assert_eq!(value_class(true, false), "text-sm");
}

#[test]
fn test_value_class_stacked_bold() {
    assert_eq!(value_class(true, true), "text-sm font-semibold");
}

// divider_class tests

#[test]
fn test_divider_class_off() {
    assert_eq!(divider_class(false), "");
}

#[test]
fn test_divider_class_on() {
    assert_eq!(divider_class(true), "pb-1 border-b border-base-200");
}
