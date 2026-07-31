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

// Precision that degrades with magnitude (ldui-sla1 / office op-jmxb).

/// THE DEFECT, as it actually appeared. `+1981d 13h over` is three tokens; the
/// pill is one line, so "over" printed outside the chip's own red border — on
/// twelve of twelve breached rows in the Consultant Task Queue.
#[test]
fn a_multi_year_breach_does_not_print_a_third_token() {
    let label = sla_chip_label(Some(NOW - 1981 * 24 * H), NOW);
    assert_eq!(label, "+5.4y over");
    assert_eq!(
        label.split_whitespace().count(),
        2,
        "the chip is one line: value + 'over', never a third token"
    );
}

/// The finer unit is dropped exactly when the coarser one reaches two digits —
/// that is where it stops informing and starts costing width.
#[test]
fn the_second_unit_is_dropped_once_the_first_has_two_digits() {
    // Days: 9 keeps its hours, 10 does not.
    assert_eq!(sla_chip_label(Some(NOW + (9 * 24 + 5) * H), NOW), "9d 5h");
    assert_eq!(sla_chip_label(Some(NOW + (10 * 24 + 5) * H), NOW), "10d");
    // Hours: same boundary, one scale down.
    assert_eq!(
        sla_chip_label(Some(NOW + 9 * H + 30 * 60_000), NOW),
        "9h 30m"
    );
    assert_eq!(sla_chip_label(Some(NOW + 10 * H + 30 * 60_000), NOW), "10h");
}

/// A year is where the UNIT changes, not merely the precision.
#[test]
fn a_year_switches_to_years_with_one_decimal() {
    assert_eq!(sla_chip_label(Some(NOW + 364 * 24 * H), NOW), "364d");
    assert_eq!(sla_chip_label(Some(NOW + 365 * 24 * H), NOW), "1.0y");
    // One decimal, so two very different staleness levels stay distinguishable
    // at the top of the scale — which is exactly where the worst records live.
    assert_eq!(sla_chip_label(Some(NOW + 730 * 24 * H), NOW), "2.0y");
    assert_ne!(
        sla_chip_label(Some(NOW + 400 * 24 * H), NOW),
        sla_chip_label(Some(NOW + 700 * 24 * H), NOW)
    );
}

/// ★ THE PROPERTY WORTH HAVING, and the one the old format lacked entirely:
/// the label is BOUNDED. The previous `Xd Yh` form had no upper width at all,
/// so it was always one unusually stale record away from breaking the pill
/// again. Swept across seven orders of magnitude, in both directions.
#[test]
fn every_label_fits_the_one_line_pill() {
    // The widest string the design is known to fit: "+9d 23h over".
    const MAX: usize = 12;
    let mut widest = String::new();
    let mut minutes = 1i64;
    while minutes < 60 * 24 * 365 * 100 {
        for signed in [minutes, -minutes] {
            let label = sla_chip_label(Some(NOW + signed * 60_000), NOW);
            assert!(
                label.chars().count() <= MAX,
                "{label:?} is {} chars (> {MAX}) at {minutes} minutes",
                label.chars().count()
            );
            if label.chars().count() > widest.chars().count() {
                widest = label;
            }
        }
        minutes = minutes * 3 / 2 + 1;
    }
    assert!(!widest.is_empty());
}
