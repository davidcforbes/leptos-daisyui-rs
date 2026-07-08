use super::*;

const NOW: i64 = 1_000_000_000_000;
const H: i64 = 60 * 60 * 1000;

fn deadline(offset_h: Option<i64>) -> Option<i64> {
    offset_h.map(|h| NOW + h * H)
}

// SlaTone tests

#[test]
fn test_sla_tone_default_is_none() {
    assert_eq!(SlaTone::default(), SlaTone::None);
}

#[test]
fn test_sla_tone_as_str() {
    assert_eq!(SlaTone::Green.as_str(), "badge-success");
    assert_eq!(SlaTone::Amber.as_str(), "badge-warning");
    assert_eq!(SlaTone::Red.as_str(), "badge-error");
    assert_eq!(SlaTone::None.as_str(), "badge-neutral");
}

#[test]
fn test_sla_tone_border_class() {
    assert_eq!(SlaTone::Green.border_class(), "border border-success/45");
    assert_eq!(SlaTone::Amber.border_class(), "border border-warning/45");
    assert_eq!(SlaTone::Red.border_class(), "border border-error/45");
    assert_eq!(SlaTone::None.border_class(), "border border-neutral/45");
}

#[test]
fn test_sla_tone_icons_and_enriched_opt_in() {
    // Each severity maps to a leading glyph; None has none. beads-p4v4
    assert_eq!(SlaTone::Green.icon_name(), Some("clock"));
    assert_eq!(SlaTone::Amber.icon_name(), Some("triangle-alert"));
    assert_eq!(SlaTone::Red.icon_name(), Some("circle-alert"));
    assert_eq!(SlaTone::None.icon_name(), None);
}

// sla_chip_tone tests (ported from d2d-ui's SlaChip::tone tests)

#[test]
fn test_tone_none_when_no_deadline() {
    assert_eq!(
        sla_chip_tone(deadline(None), NOW, SLA_CHIP_DEFAULT_THRESHOLD_MS),
        SlaTone::None
    );
}

#[test]
fn test_tone_green_amber_red_by_remaining() {
    // 5h out -> green (threshold is the default 2h).
    assert_eq!(
        sla_chip_tone(deadline(Some(5)), NOW, SLA_CHIP_DEFAULT_THRESHOLD_MS),
        SlaTone::Green
    );
    // 1h out -> amber (within 2h).
    assert_eq!(
        sla_chip_tone(deadline(Some(1)), NOW, SLA_CHIP_DEFAULT_THRESHOLD_MS),
        SlaTone::Amber
    );
    // exactly at threshold (2h) -> amber (inclusive).
    assert_eq!(
        sla_chip_tone(deadline(Some(2)), NOW, SLA_CHIP_DEFAULT_THRESHOLD_MS),
        SlaTone::Amber
    );
    // 3h overdue -> red.
    assert_eq!(
        sla_chip_tone(deadline(Some(-3)), NOW, SLA_CHIP_DEFAULT_THRESHOLD_MS),
        SlaTone::Red
    );
}

#[test]
fn test_custom_threshold_changes_amber_window() {
    // With a 6h threshold, 5h out is amber (not green).
    assert_eq!(sla_chip_tone(deadline(Some(5)), NOW, 6 * H), SlaTone::Amber);
}

// sla_chip_label tests (ported from d2d-ui's SlaChip::label tests)

#[test]
fn test_label_none_when_no_deadline() {
    assert_eq!(sla_chip_label(deadline(None), NOW), "No SLA");
}

#[test]
fn test_label_remaining_and_over() {
    // 5h ahead -> "5h 0m".
    assert_eq!(sla_chip_label(deadline(Some(5)), NOW), "5h 0m");
    // 3h overdue -> "+3h 0m over".
    assert_eq!(sla_chip_label(deadline(Some(-3)), NOW), "+3h 0m over");
    // > 1 day ahead -> days+hours.
    assert_eq!(sla_chip_label(Some(NOW + 50 * H), NOW), "2d 2h");
}

#[test]
fn test_label_minutes_only_under_an_hour() {
    assert_eq!(sla_chip_label(Some(NOW + 15 * 60_000), NOW), "15m");
}
