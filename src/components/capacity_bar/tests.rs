use super::*;

// CapacityBarColor tests

#[test]
fn test_capacity_bar_color_default() {
    assert_eq!(CapacityBarColor::default(), CapacityBarColor::Primary);
}

#[test]
fn test_capacity_bar_color_neutral() {
    assert_eq!(CapacityBarColor::Neutral.as_str(), "bg-neutral");
}

#[test]
fn test_capacity_bar_color_primary() {
    assert_eq!(CapacityBarColor::Primary.as_str(), "bg-primary");
}

#[test]
fn test_capacity_bar_color_secondary() {
    assert_eq!(CapacityBarColor::Secondary.as_str(), "bg-secondary");
}

#[test]
fn test_capacity_bar_color_accent() {
    assert_eq!(CapacityBarColor::Accent.as_str(), "bg-accent");
}

#[test]
fn test_capacity_bar_color_info() {
    assert_eq!(CapacityBarColor::Info.as_str(), "bg-info");
}

#[test]
fn test_capacity_bar_color_success() {
    assert_eq!(CapacityBarColor::Success.as_str(), "bg-success");
}

#[test]
fn test_capacity_bar_color_warning() {
    assert_eq!(CapacityBarColor::Warning.as_str(), "bg-warning");
}

#[test]
fn test_capacity_bar_color_error() {
    assert_eq!(CapacityBarColor::Error.as_str(), "bg-error");
}

#[test]
fn test_capacity_bar_color_clone_and_debug() {
    let c1 = CapacityBarColor::Warning;
    let c2 = c1.clone();
    assert_eq!(c1, c2);
    assert!(format!("{:?}", c1).contains("Warning"));
}

#[test]
fn test_all_capacity_bar_colors_return_valid_bg_classes() {
    let variants = vec![
        CapacityBarColor::Neutral,
        CapacityBarColor::Primary,
        CapacityBarColor::Secondary,
        CapacityBarColor::Accent,
        CapacityBarColor::Info,
        CapacityBarColor::Success,
        CapacityBarColor::Warning,
        CapacityBarColor::Error,
    ];
    for variant in variants {
        assert!(
            variant.as_str().starts_with("bg-"),
            "CapacityBarColor '{:?}' should map to a bg-* class",
            variant
        );
    }
}

// capacity_bar_default_max tests (ported behavior from d2d-ui capacity_bar.rs)

#[test]
fn test_default_max_gives_overflow_headroom() {
    // cap * 1.25, matching d2d-ui's DEFAULT_MAX_FACTOR.
    let max = capacity_bar_default_max(10.0, 8.0);
    assert!((max - 12.5).abs() < 0.0001);
}

#[test]
fn test_default_max_expands_when_value_exceeds_headroom() {
    // value (20) is beyond cap*1.25 (12.5); max must grow to fit value.
    let max = capacity_bar_default_max(10.0, 20.0);
    assert!((max - 20.0).abs() < 0.0001);
}

#[test]
fn test_default_max_never_below_cap() {
    let max = capacity_bar_default_max(0.0, 0.0);
    assert!(max >= 0.0);
}

// capacity_bar_is_over tests

#[test]
fn test_is_over_true_when_value_exceeds_cap() {
    assert!(capacity_bar_is_over(12.0, 10.0));
}

#[test]
fn test_is_over_false_when_value_equals_cap() {
    assert!(!capacity_bar_is_over(10.0, 10.0));
}

#[test]
fn test_is_over_false_when_value_under_cap() {
    assert!(!capacity_bar_is_over(8.0, 10.0));
}

// capacity_bar_percent tests

#[test]
fn test_percent_basic_fraction() {
    assert!((capacity_bar_percent(5.0, 10.0) - 50.0).abs() < 0.0001);
}

#[test]
fn test_percent_clamps_to_track_width() {
    // Way over max — clamps to 100%, mirroring d2d-ui's x_at clamp to the right edge.
    assert!((capacity_bar_percent(999.0, 10.0) - 100.0).abs() < 0.0001);
}

#[test]
fn test_percent_clamps_to_zero_for_negative_units() {
    assert!((capacity_bar_percent(-5.0, 10.0) - 0.0).abs() < 0.0001);
}

#[test]
fn test_percent_zero_when_max_non_positive() {
    assert_eq!(capacity_bar_percent(5.0, 0.0), 0.0);
    assert_eq!(capacity_bar_percent(5.0, -1.0), 0.0);
}

// capacity_bar_under_cap_percent tests

#[test]
fn test_under_cap_percent_uses_value_when_under_cap() {
    assert!((capacity_bar_under_cap_percent(30.0, 50.0) - 30.0).abs() < 0.0001);
}

#[test]
fn test_under_cap_percent_clips_to_cap_when_over() {
    assert!((capacity_bar_under_cap_percent(80.0, 50.0) - 50.0).abs() < 0.0001);
}

// capacity_bar_overflow_band tests

#[test]
fn test_overflow_band_none_when_under_cap() {
    assert_eq!(capacity_bar_overflow_band(30.0, 50.0), None);
}

#[test]
fn test_overflow_band_none_when_exactly_at_cap() {
    assert_eq!(capacity_bar_overflow_band(50.0, 50.0), None);
}

#[test]
fn test_overflow_band_some_with_correct_left_and_width_when_over() {
    let band = capacity_bar_overflow_band(80.0, 50.0);
    assert_eq!(band, Some((50.0, 30.0)));
}

// Integration-style check across the full pipeline for one scenario, mirroring
// d2d-ui's "over_cap_is_detected_and_positions_are_ordered" test.
#[test]
fn test_over_cap_scenario_positions_are_ordered() {
    let cap = 10.0;
    let value = 12.0;
    let max = 16.0; // explicit override, like d2d-ui's with_max(16.0)

    assert!(capacity_bar_is_over(value, cap));

    let cap_pct = capacity_bar_percent(cap, max);
    let value_pct = capacity_bar_percent(value, max);
    assert!(cap_pct < value_pct);
    assert!(value_pct <= 100.0);

    let band = capacity_bar_overflow_band(value_pct, cap_pct).expect("value exceeds cap");
    assert!((band.0 - cap_pct).abs() < 0.0001);
    assert!((band.0 + band.1 - value_pct).abs() < 0.0001);
}

// ── Stopped (bead 4iiz-Database 1x6y.7) ──────────────────────────────────────

/// ⚠️ STOPPED MUST NOT RENDER AS A PLAIN FILL. Every other tone answers "how much"; this one
/// answers "is it still running at all". A solid block asserts a magnitude, and once the series
/// has ended the magnitude is not the point — a stopped feed drawn as a longer Error bar reads
/// as the worst case of a LIVE series, which is exactly how a dead mirror hides among slow ones.
#[test]
fn stopped_is_a_pattern_not_a_fill() {
    let s = CapacityBarColor::Stopped.as_str();
    assert!(
        s.contains("repeating-linear-gradient"),
        "Stopped must be hatched so it cannot be mistaken for a full bar: {s}"
    );
    for solid in [
        CapacityBarColor::Error,
        CapacityBarColor::Warning,
        CapacityBarColor::Success,
    ] {
        assert!(
            !solid.as_str().contains("repeating-linear-gradient"),
            "{solid:?} is a magnitude and must stay a plain fill"
        );
    }
}

/// ⚠️ AND ONLY STOPPED ANSWERS `is_stopped`. Callers SORT on it: an ended series belongs at the
/// top regardless of its value, because "ended" outranks "large". Sorting by magnitude is what
/// buries it.
#[test]
fn only_stopped_reports_itself_as_stopped() {
    assert!(CapacityBarColor::Stopped.is_stopped());
    for live in [
        CapacityBarColor::Neutral,
        CapacityBarColor::Primary,
        CapacityBarColor::Secondary,
        CapacityBarColor::Accent,
        CapacityBarColor::Info,
        CapacityBarColor::Success,
        CapacityBarColor::Warning,
        CapacityBarColor::Error,
    ] {
        assert!(!live.is_stopped(), "{live:?} describes a live series");
    }
}
