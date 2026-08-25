use super::*;

// gauge_fraction

#[test]
fn fraction_is_value_over_max_clamped() {
    assert!((gauge_fraction(5.0, 10.0) - 0.5).abs() < 1e-9);
    assert_eq!(gauge_fraction(15.0, 10.0), 1.0);
    assert_eq!(gauge_fraction(-3.0, 10.0), 0.0);
}

#[test]
fn fraction_never_divides_by_zero_or_propagates_nan() {
    assert_eq!(gauge_fraction(5.0, 0.0), 0.0);
    assert_eq!(gauge_fraction(5.0, -1.0), 0.0);
    assert_eq!(gauge_fraction(f64::NAN, 10.0), 0.0);
}

// gauge_point geometry

#[test]
fn dial_endpoints_sit_on_the_bottom_shoulders_and_pass_the_top() {
    let (x0, y0) = gauge_point(50.0, 50.0, 40.0, 0.0);
    // 150 degrees: bottom-left shoulder (left of center, below center).
    assert!(x0 < 50.0 && y0 > 50.0, "start at bottom-left: ({x0}, {y0})");

    let (x1, y1) = gauge_point(50.0, 50.0, 40.0, 1.0);
    // 390 degrees == 30 degrees: bottom-right shoulder.
    assert!(x1 > 50.0 && y1 > 50.0, "end at bottom-right: ({x1}, {y1})");

    let (xm, ym) = gauge_point(50.0, 50.0, 40.0, 0.5);
    // Halfway (270 degrees) is the top of the dial.
    assert!((xm - 50.0).abs() < 1e-6, "midpoint centered: {xm}");
    assert!((ym - 10.0).abs() < 1e-6, "midpoint at the top: {ym}");
}

// gauge_arc_path

#[test]
fn arc_path_is_empty_for_an_empty_or_inverted_span() {
    assert_eq!(gauge_arc_path(50.0, 50.0, 40.0, 0.5, 0.5), "");
    assert_eq!(gauge_arc_path(50.0, 50.0, 40.0, 0.8, 0.2), "");
}

#[test]
fn arc_path_uses_the_large_arc_flag_only_past_half_the_sweep() {
    // 0.5 of a 240-degree sweep is 120 degrees: small arc.
    let small = gauge_arc_path(50.0, 50.0, 40.0, 0.0, 0.5);
    assert!(small.contains(" 0 1 "), "120-degree arc is small: {small}");

    // The full sweep is 240 degrees: large arc.
    let large = gauge_arc_path(50.0, 50.0, 40.0, 0.0, 1.0);
    assert!(large.contains(" 1 1 "), "240-degree arc is large: {large}");
}

#[test]
fn arc_path_contains_no_nan() {
    let path = gauge_arc_path(50.0, 50.0, 40.0, 0.0, 1.0);
    assert!(!path.contains("NaN"), "finite path: {path}");
    assert!(path.starts_with("M "), "move-to first: {path}");
}

// gauge_bands

#[test]
fn warn_band_runs_to_the_error_threshold() {
    let (warn, error) = gauge_bands(Some(0.7), Some(0.9));
    assert_eq!(warn, Some((0.7, 0.9)));
    assert_eq!(error, Some((0.9, 1.0)));
}

#[test]
fn warn_band_runs_to_the_end_without_an_error_threshold() {
    let (warn, error) = gauge_bands(Some(0.7), None);
    assert_eq!(warn, Some((0.7, 1.0)));
    assert_eq!(error, None);
}

#[test]
fn empty_or_inverted_bands_are_dropped() {
    // warn at/past error: nothing to paint yellow.
    assert_eq!(gauge_bands(Some(0.9), Some(0.9)).0, None);
    assert_eq!(gauge_bands(Some(0.95), Some(0.9)).0, None);
    // error at the very end of the dial: zero-width band.
    assert_eq!(gauge_bands(None, Some(1.0)).1, None);
    assert_eq!(gauge_bands(None, None), (None, None));
}

#[test]
fn band_thresholds_are_clamped() {
    let (warn, error) = gauge_bands(Some(-0.5), Some(1.5));
    assert_eq!(warn, Some((0.0, 1.0)));
    assert_eq!(error, None);
}

// gauge_value_paint

#[test]
fn value_paint_escalates_through_the_zones() {
    let warn = Some(0.7);
    let error = Some(0.9);
    assert_eq!(gauge_value_paint(0.5, warn, error), "var(--color-primary)");
    assert_eq!(gauge_value_paint(0.7, warn, error), "var(--color-warning)");
    assert_eq!(gauge_value_paint(0.95, warn, error), "var(--color-error)");
}

#[test]
fn value_paint_stays_primary_without_thresholds() {
    assert_eq!(gauge_value_paint(0.99, None, None), "var(--color-primary)");
}

// gauge_readout

#[test]
fn readout_drops_decimals_at_ten_and_above() {
    assert_eq!(gauge_readout(87.3), "87");
    assert_eq!(gauge_readout(10.0), "10");
}

#[test]
fn readout_keeps_one_decimal_below_ten_and_trims_point_zero() {
    assert_eq!(gauge_readout(6.25), "6.2");
    assert_eq!(gauge_readout(6.0), "6");
    assert_eq!(gauge_readout(0.0), "0");
}

#[test]
fn readout_placeholder_for_non_finite_values() {
    assert_eq!(gauge_readout(f64::NAN), "\u{2013}");
    assert_eq!(gauge_readout(f64::INFINITY), "\u{2013}");
}
