use super::*;

// segmented_bar_total

#[test]
fn test_total_sums_positive_values() {
    let segs = [(4.0, "bg-success"), (3.0, "bg-warning"), (3.0, "bg-error")];
    assert!((segmented_bar_total(&segs) - 10.0).abs() < 0.0001);
}

#[test]
fn test_total_clamps_negative_values_to_zero() {
    // A stray negative shouldn't shrink the denominator below what the
    // positive segments alone would give.
    let segs = [(5.0, "bg-success"), (-2.0, "bg-error")];
    assert!((segmented_bar_total(&segs) - 5.0).abs() < 0.0001);
}

#[test]
fn test_total_empty_is_zero() {
    let segs: [(f64, &str); 0] = [];
    assert_eq!(segmented_bar_total(&segs), 0.0);
}

// segmented_bar_percent

#[test]
fn test_percent_basic_share_of_total() {
    assert!((segmented_bar_percent(4.0, 10.0) - 40.0).abs() < 0.0001);
}

#[test]
fn test_percent_pre_normalized_fractions_pass_through() {
    // Fractions that already sum to 1.0 behave the same as raw counts.
    assert!((segmented_bar_percent(0.4, 1.0) - 40.0).abs() < 0.0001);
}

#[test]
fn test_percent_zero_when_total_non_positive() {
    assert_eq!(segmented_bar_percent(4.0, 0.0), 0.0);
    assert_eq!(segmented_bar_percent(4.0, -1.0), 0.0);
}

#[test]
fn test_percent_zero_when_value_non_positive() {
    assert_eq!(segmented_bar_percent(0.0, 10.0), 0.0);
    assert_eq!(segmented_bar_percent(-1.0, 10.0), 0.0);
}

#[test]
fn test_percent_clamps_to_one_hundred() {
    // value > total shouldn't happen given segmented_bar_total's own
    // definition, but the clamp guards against a caller building the
    // percentage from a different (smaller) total than the one it renders.
    assert!((segmented_bar_percent(12.0, 10.0) - 100.0).abs() < 0.0001);
}

// Integration-style check across a full G/Y/R scenario.

#[test]
fn test_gyr_scenario_segments_sum_to_one_hundred() {
    let segs = [(4.0, "bg-success"), (3.0, "bg-warning"), (3.0, "bg-error")];
    let total = segmented_bar_total(&segs);
    let sum: f64 = segs
        .iter()
        .map(|(v, _)| segmented_bar_percent(*v, total))
        .sum();
    assert!((sum - 100.0).abs() < 0.0001);
}

#[test]
fn test_all_zero_segments_renders_empty_track() {
    let segs = [(0.0, "bg-success"), (0.0, "bg-warning"), (0.0, "bg-error")];
    let total = segmented_bar_total(&segs);
    for (v, _) in segs {
        assert_eq!(segmented_bar_percent(v, total), 0.0);
    }
}
