use super::*;
use crate::components::day_scheduler::SchedulerEventColor;

// Monday 2026-03-02 = epoch day 20514 (matches d2d-ui's week_view test fixture).
const MON_2026_03_02: i64 = 20_514;

// ---------------------------------------------------------------------
// civil_from_days
// ---------------------------------------------------------------------

#[test]
fn civil_from_days_epoch() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
}

#[test]
fn civil_from_days_known_date() {
    assert_eq!(civil_from_days(MON_2026_03_02), (2026, 3, 2));
}

#[test]
fn civil_from_days_negative_days() {
    // One day before the epoch is 1969-12-31.
    assert_eq!(civil_from_days(-1), (1969, 12, 31));
}

// ---------------------------------------------------------------------
// week_start_for
// ---------------------------------------------------------------------

#[test]
fn week_start_for_returns_monday() {
    // Wednesday = MON_2026_03_02 + 2.
    assert_eq!(week_start_for(MON_2026_03_02 + 2), MON_2026_03_02);
    // Sunday = MON_2026_03_02 + 6.
    assert_eq!(week_start_for(MON_2026_03_02 + 6), MON_2026_03_02);
}

#[test]
fn week_start_for_idempotent_on_monday() {
    assert_eq!(week_start_for(MON_2026_03_02), MON_2026_03_02);
}

#[test]
fn week_start_for_result_is_always_a_monday() {
    for offset in -10..=10 {
        let ws = week_start_for(MON_2026_03_02 + offset);
        assert_eq!((ws + 3).rem_euclid(7), 0, "offset {offset} -> {ws}");
    }
}

// ---------------------------------------------------------------------
// week_range_label
// ---------------------------------------------------------------------

#[test]
fn week_range_label_within_month() {
    assert_eq!(week_range_label(MON_2026_03_02), "Mar 2 - 8, 2026");
}

#[test]
fn week_range_label_spans_month_boundary_same_year() {
    // Mon 2026-03-30 .. Sun 2026-04-05.
    let mar30 = MON_2026_03_02 + 28;
    assert_eq!(week_range_label(mar30), "Mar 30 - Apr 5, 2026");
}

#[test]
fn week_range_label_spans_year_boundary() {
    // Mon 2025-12-29 .. Sun 2026-01-04 -> epoch day for 2025-12-29.
    let dec29 = MON_2026_03_02 - 63; // 9 weeks earlier, verified below
    let (y, m, d) = civil_from_days(dec29);
    assert_eq!((y, m, d), (2025, 12, 29));
    assert_eq!(week_range_label(dec29), "Dec 29, 2025 - Jan 4, 2026");
}

// ---------------------------------------------------------------------
// weekday_abbrev / day_of_month
// ---------------------------------------------------------------------

#[test]
fn weekday_abbrev_maps_all_seven_columns() {
    assert_eq!(weekday_abbrev(0), "Mon");
    assert_eq!(weekday_abbrev(1), "Tue");
    assert_eq!(weekday_abbrev(2), "Wed");
    assert_eq!(weekday_abbrev(3), "Thu");
    assert_eq!(weekday_abbrev(4), "Fri");
    assert_eq!(weekday_abbrev(5), "Sat");
    assert_eq!(weekday_abbrev(6), "Sun");
}

#[test]
fn weekday_abbrev_clamps_out_of_range() {
    assert_eq!(weekday_abbrev(9), "Sun");
}

#[test]
fn day_of_month_matches_known_week() {
    assert_eq!(day_of_month(MON_2026_03_02, 0), 2); // Mon
    assert_eq!(day_of_month(MON_2026_03_02, 6), 8); // Sun
}

// ---------------------------------------------------------------------
// CalEvent
// ---------------------------------------------------------------------

#[test]
fn cal_event_new_keeps_valid_range() {
    let ev = CalEvent::new("Standup", 0, 540, 555, SchedulerEventColor::Primary);
    assert_eq!(ev.title, "Standup");
    assert_eq!(ev.day, 0);
    assert_eq!(ev.start_min, 540);
    assert_eq!(ev.end_min, 555);
    assert!(!ev.all_day);
    assert_eq!(ev.location, "");
}

#[test]
fn cal_event_new_clamps_zero_or_negative_duration() {
    let ev = CalEvent::new("Instant", 0, 600, 600, SchedulerEventColor::Error);
    assert_eq!(ev.end_min, 601);
    let ev2 = CalEvent::new("Backwards", 0, 600, 500, SchedulerEventColor::Error);
    assert_eq!(ev2.end_min, 601);
}

#[test]
fn cal_event_new_clamps_day_to_valid_column() {
    let ev = CalEvent::new("Overflow", 9, 0, 60, SchedulerEventColor::Neutral);
    assert_eq!(ev.day, 6);
}

#[test]
fn cal_event_with_location_and_all_day_builders() {
    let ev = CalEvent::new("Trip", 2, 0, 60, SchedulerEventColor::Info)
        .with_location("Airport")
        .all_day();
    assert_eq!(ev.location, "Airport");
    assert!(ev.all_day);
}

// ---------------------------------------------------------------------
// compute_week_event_layout -- thin adapter over DayScheduler's lane math
// ---------------------------------------------------------------------

#[test]
fn compute_week_event_layout_empty_is_empty() {
    assert!(compute_week_event_layout(&[], 8, 18).is_empty());
}

#[test]
fn compute_week_event_layout_single_event_spans_full_width() {
    let events = vec![CalEvent::new(
        "Sync",
        2,
        11 * 60,
        12 * 60,
        SchedulerEventColor::Primary,
    )];
    let layouts = compute_week_event_layout(&events, 8, 18);
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].left_pct, 0.0);
    assert_eq!(layouts[0].width_pct, 100.0);
}

#[test]
fn compute_week_event_layout_overlapping_events_split_into_lanes() {
    let events = vec![
        CalEvent::new("A", 3, 9 * 60, 10 * 60, SchedulerEventColor::Primary),
        CalEvent::new(
            "B",
            3,
            9 * 60 + 30,
            10 * 60 + 30,
            SchedulerEventColor::Warning,
        ),
    ];
    let layouts = compute_week_event_layout(&events, 8, 18);
    assert_eq!(layouts[0].width_pct, 50.0);
    assert_eq!(layouts[1].width_pct, 50.0);
    assert_eq!(layouts[0].left_pct, 0.0);
    assert_eq!(layouts[1].left_pct, 50.0);
}

#[test]
fn compute_week_event_layout_non_overlapping_each_spans_full_width() {
    let events = vec![
        CalEvent::new("A", 1, 500, 600, SchedulerEventColor::Primary),
        CalEvent::new("B", 1, 700, 800, SchedulerEventColor::Primary),
    ];
    let layouts = compute_week_event_layout(&events, 0, 24);
    for l in &layouts {
        assert_eq!(l.width_pct, 100.0);
        assert_eq!(l.left_pct, 0.0);
    }
}
