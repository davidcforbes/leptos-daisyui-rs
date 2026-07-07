use super::*;

// SparklineColor tests

#[test]
fn test_sparkline_color_default() {
    let color = SparklineColor::default();
    assert_eq!(color.as_str(), "text-base-content");
}

#[test]
fn test_sparkline_color_primary() {
    assert_eq!(SparklineColor::Primary.as_str(), "text-primary");
}

#[test]
fn test_sparkline_color_secondary() {
    assert_eq!(SparklineColor::Secondary.as_str(), "text-secondary");
}

#[test]
fn test_sparkline_color_accent() {
    assert_eq!(SparklineColor::Accent.as_str(), "text-accent");
}

#[test]
fn test_sparkline_color_success() {
    assert_eq!(SparklineColor::Success.as_str(), "text-success");
}

#[test]
fn test_sparkline_color_info() {
    assert_eq!(SparklineColor::Info.as_str(), "text-info");
}

#[test]
fn test_sparkline_color_warning() {
    assert_eq!(SparklineColor::Warning.as_str(), "text-warning");
}

#[test]
fn test_sparkline_color_error() {
    assert_eq!(SparklineColor::Error.as_str(), "text-error");
}

#[test]
fn test_sparkline_color_clone_and_debug() {
    let c1 = SparklineColor::Accent;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
    assert!(format!("{:?}", c1).contains("Accent"));
}

#[test]
fn test_all_sparkline_colors_return_valid_classes() {
    let variants = vec![
        (SparklineColor::Default, "text-base-content"),
        (SparklineColor::Primary, "text-primary"),
        (SparklineColor::Secondary, "text-secondary"),
        (SparklineColor::Accent, "text-accent"),
        (SparklineColor::Success, "text-success"),
        (SparklineColor::Info, "text-info"),
        (SparklineColor::Warning, "text-warning"),
        (SparklineColor::Error, "text-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

// sparkline_current / sparkline_peak tests (ported behavior from d2d-ui sparkline.rs)

#[test]
fn test_empty_is_flat_current_zero_peak_one() {
    let samples: Vec<f32> = vec![];
    assert_eq!(sparkline_current(&samples), 0.0);
    assert_eq!(sparkline_peak(&samples), 1.0); // floored so a flat series sits on the baseline
}

#[test]
fn test_current_is_last_peak_is_max_floored() {
    let samples = vec![2.0, 9.0, 4.0];
    assert_eq!(sparkline_current(&samples), 4.0);
    assert_eq!(sparkline_peak(&samples), 9.0);
}

#[test]
fn test_peak_never_below_one() {
    let samples = vec![0.1, 0.2, 0.0];
    assert_eq!(sparkline_peak(&samples), 1.0);
}

#[test]
fn test_single_sample_is_current_and_peak() {
    let samples = vec![42.0];
    assert_eq!(sparkline_current(&samples), 42.0);
    assert_eq!(sparkline_peak(&samples), 42.0);
}

// sparkline_has_readout tests

#[test]
fn test_has_readout_true_when_title_present() {
    assert!(sparkline_has_readout("Throughput"));
}

#[test]
fn test_has_readout_false_when_title_empty() {
    assert!(!sparkline_has_readout(""));
}

// sparkline_points tests

#[test]
fn test_points_empty_when_fewer_than_two_samples() {
    assert_eq!(sparkline_points(&[], 100.0, 40.0), "");
    assert_eq!(sparkline_points(&[5.0], 100.0, 40.0), "");
}

#[test]
fn test_points_empty_when_viewbox_degenerate() {
    let samples = vec![1.0, 2.0, 3.0];
    assert_eq!(sparkline_points(&samples, 0.0, 40.0), "");
    assert_eq!(sparkline_points(&samples, 100.0, 0.0), "");
    assert_eq!(sparkline_points(&samples, -10.0, 40.0), "");
}

#[test]
fn test_points_two_samples_span_full_width() {
    // peak = 2.0 (max of [1.0, 2.0]); first sample sits at half height, last at top.
    let points = sparkline_points(&[1.0, 2.0], 100.0, 40.0);
    assert_eq!(points, "0.00,20.00 100.00,0.00");
}

#[test]
fn test_points_flat_series_sits_on_baseline() {
    // All-zero samples: peak floors to 1.0, so y = height - 0/1*height = height (baseline).
    let points = sparkline_points(&[0.0, 0.0, 0.0], 90.0, 30.0);
    assert_eq!(points, "0.00,30.00 45.00,30.00 90.00,30.00");
}

#[test]
fn test_points_count_matches_sample_count() {
    let samples = vec![1.0, 5.0, 3.0, 8.0, 2.0];
    let points = sparkline_points(&samples, 200.0, 60.0);
    assert_eq!(points.split(' ').count(), samples.len());
}

#[test]
fn test_points_peak_sample_touches_top_edge() {
    let samples = vec![3.0, 9.0, 1.0];
    let points = sparkline_points(&samples, 200.0, 80.0);
    // Peak sample (9.0) is the second point; its y should be 0.00 (top of viewBox).
    let second = points.split(' ').nth(1).unwrap();
    assert_eq!(second, "100.00,0.00");
}

// sparkline_current_label / sparkline_peak_label tests

#[test]
fn test_current_label_with_unit() {
    assert_eq!(
        sparkline_current_label("Throughput", "KB/s", 4.0),
        "Throughput  4.0 KB/s"
    );
}

#[test]
fn test_current_label_without_unit() {
    assert_eq!(sparkline_current_label("CPU", "", 12.34), "CPU  12.3");
}

#[test]
fn test_peak_label_rounds_to_whole_number() {
    assert_eq!(sparkline_peak_label(9.6), "peak 10");
    assert_eq!(sparkline_peak_label(1.0), "peak 1");
}
