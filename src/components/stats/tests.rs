use super::*;

#[test]
fn test_stat_delta_trend_default_is_positive() {
    let trend = StatDeltaTrend::default();
    assert_eq!(trend, StatDeltaTrend::Positive);
}

#[test]
fn test_stat_delta_trend_positive_color() {
    assert_eq!(StatDeltaTrend::Positive.as_str(), "text-success");
}

#[test]
fn test_stat_delta_trend_negative_color() {
    assert_eq!(StatDeltaTrend::Negative.as_str(), "text-error");
}

#[test]
fn test_stat_delta_trend_neutral_returns_empty() {
    // Neutral intentionally returns "" so the surrounding stat-desc
    // base-content/60 styling shows through unchanged.
    assert_eq!(StatDeltaTrend::Neutral.as_str(), "");
}

#[test]
fn test_stat_delta_trend_arrow_glyphs() {
    // Match the demo's hand-typed glyphs so migrating existing call
    // sites is visually a no-op.
    assert_eq!(StatDeltaTrend::Positive.arrow(), "↗︎");
    assert_eq!(StatDeltaTrend::Negative.arrow(), "↘︎");
    assert_eq!(StatDeltaTrend::Neutral.arrow(), "→");
}

#[test]
fn test_stat_delta_trend_clone_and_eq() {
    let a = StatDeltaTrend::Negative;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn test_stat_delta_trend_debug_format() {
    assert!(format!("{:?}", StatDeltaTrend::Positive).contains("Positive"));
    assert!(format!("{:?}", StatDeltaTrend::Negative).contains("Negative"));
    assert!(format!("{:?}", StatDeltaTrend::Neutral).contains("Neutral"));
}

#[test]
fn test_all_trends_round_trip() {
    let cases = [
        (StatDeltaTrend::Positive, "text-success", "↗︎"),
        (StatDeltaTrend::Negative, "text-error", "↘︎"),
        (StatDeltaTrend::Neutral, "", "→"),
    ];
    for (trend, expected_color, expected_arrow) in cases {
        assert_eq!(trend.as_str(), expected_color);
        assert_eq!(trend.arrow(), expected_arrow);
    }
}
