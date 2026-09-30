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

// ---------------------------------------------------------------------
// Weekday / WeekdaySet / week_start_for_weekday (ldui-9sip)
// ---------------------------------------------------------------------

#[test]
fn weekday_from_epoch_day_knows_the_epoch_was_a_thursday() {
    assert_eq!(Weekday::from_epoch_day(0), Weekday::Thursday);
    assert_eq!(Weekday::from_epoch_day(-1), Weekday::Wednesday);
    assert_eq!(Weekday::from_epoch_day(MON_2026_03_02), Weekday::Monday);
    assert_eq!(Weekday::from_epoch_day(MON_2026_03_02 + 6), Weekday::Sunday);
}

#[test]
fn weekday_names_agree_with_the_column_abbreviations() {
    for (i, day) in Weekday::ALL.into_iter().enumerate() {
        assert_eq!(day.index_from_monday(), i);
        assert_eq!(day.abbrev(), weekday_abbrev(i));
        assert!(day.name().starts_with(day.abbrev()), "{day:?}");
    }
}

#[test]
fn weekday_set_default_is_a_seven_day_workweek() {
    let set = WeekdaySet::default();
    assert_eq!(set, WeekdaySet::ALL);
    assert!(Weekday::ALL.into_iter().all(|d| set.contains(d)));
}

#[test]
fn weekday_set_monday_to_friday_excludes_only_the_weekend() {
    let set = WeekdaySet::MONDAY_TO_FRIDAY;
    assert!(set.contains(Weekday::Monday) && set.contains(Weekday::Friday));
    assert!(!set.contains(Weekday::Saturday) && !set.contains(Weekday::Sunday));
}

#[test]
fn weekday_set_with_without_and_from_days_round_trip() {
    // A Gulf-region working week: Sunday to Thursday.
    let gulf = WeekdaySet::from_days([
        Weekday::Sunday,
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
    ]);
    assert!(!gulf.contains(Weekday::Friday) && !gulf.contains(Weekday::Saturday));
    assert_eq!(
        WeekdaySet::ALL
            .without(Weekday::Friday)
            .without(Weekday::Saturday),
        gulf
    );
    assert_eq!(
        gulf.with(Weekday::Friday).with(Weekday::Saturday),
        WeekdaySet::ALL
    );
    assert_eq!(
        WeekdaySet::NONE
            .with(Weekday::Monday)
            .without(Weekday::Monday),
        WeekdaySet::NONE
    );
}

#[test]
fn week_start_for_weekday_monday_matches_week_start_for() {
    for offset in -10..=10 {
        let d = MON_2026_03_02 + offset;
        assert_eq!(
            week_start_for_weekday(d, Weekday::Monday),
            week_start_for(d)
        );
    }
}

#[test]
fn week_start_for_weekday_finds_the_previous_sunday() {
    let sun_mar_1 = MON_2026_03_02 - 1;
    // Wednesday Mar 4 -> Sunday Mar 1; Sunday itself is idempotent.
    assert_eq!(
        week_start_for_weekday(MON_2026_03_02 + 2, Weekday::Sunday),
        sun_mar_1
    );
    assert_eq!(
        week_start_for_weekday(sun_mar_1, Weekday::Sunday),
        sun_mar_1
    );
    // Saturday Mar 7 is still in the week that began Sunday Mar 1.
    assert_eq!(
        week_start_for_weekday(MON_2026_03_02 + 5, Weekday::Sunday),
        sun_mar_1
    );
    for offset in -10..=10 {
        let start = week_start_for_weekday(MON_2026_03_02 + offset, Weekday::Saturday);
        assert_eq!(Weekday::from_epoch_day(start), Weekday::Saturday);
        assert!((0..7).contains(&(MON_2026_03_02 + offset - start)));
    }
}

#[test]
fn column_weekday_follows_the_week_start_not_the_column_index() {
    assert_eq!(column_weekday(MON_2026_03_02, 0), Weekday::Monday);
    let sunday_start = MON_2026_03_02 - 1;
    assert_eq!(column_weekday(sunday_start, 0), Weekday::Sunday);
    assert_eq!(column_weekday(sunday_start, 6), Weekday::Saturday);
    // Out-of-range columns clamp like every other column helper.
    assert_eq!(column_weekday(sunday_start, 42), Weekday::Saturday);
}

#[test]
fn column_date_label_names_weekday_month_and_day() {
    assert_eq!(column_date_label(MON_2026_03_02, 0), "Monday Mar 2");
    // Mon Mar 30 + 2 crosses into April.
    assert_eq!(column_date_label(MON_2026_03_02 + 28, 2), "Wednesday Apr 1");
}

// ---------------------------------------------------------------------
// Accessible names (ldui-9sip)
// ---------------------------------------------------------------------

#[test]
fn week_event_aria_label_carries_the_day() {
    let ev = CalEvent::new(
        "Standup",
        1,
        9 * 60,
        9 * 60 + 15,
        SchedulerEventColor::Primary,
    );
    assert_eq!(
        week_event_aria_label(&ev, MON_2026_03_02),
        "Standup, Tuesday Mar 3, 09:00 to 09:15"
    );
}

#[test]
fn week_event_aria_label_all_day_names_no_times() {
    let ev = CalEvent::new("Holiday", 4, 0, 0, SchedulerEventColor::Error).all_day();
    assert_eq!(
        week_event_aria_label(&ev, MON_2026_03_02),
        "Holiday, Friday Mar 6, all day"
    );
}

#[test]
fn resolve_week_label_uses_the_override_and_falls_back_on_blank() {
    let ev = CalEvent::new("Cita", 0, 9 * 60, 10 * 60, SchedulerEventColor::Primary);
    let english = week_event_aria_label(&ev, MON_2026_03_02);
    assert_eq!(
        resolve_week_event_accessible_label(&ev, MON_2026_03_02, None),
        english
    );
    assert_eq!(
        resolve_week_event_accessible_label(&ev, MON_2026_03_02, Some("   ".into())),
        english
    );
    assert_eq!(
        resolve_week_event_accessible_label(
            &ev,
            MON_2026_03_02,
            Some("Cita, lunes 2, de 09:00 a 10:00".into())
        ),
        "Cita, lunes 2, de 09:00 a 10:00"
    );
}

// ---------------------------------------------------------------------
// Keyboard contract (ldui-9sip)
// ---------------------------------------------------------------------

#[test]
fn left_right_arrows_request_a_one_day_move() {
    assert_eq!(
        week_event_key_intent("ArrowLeft", false, 15),
        Some(WeekEventKeyIntent::MoveDay(-1))
    );
    assert_eq!(
        week_event_key_intent("ArrowRight", false, 15),
        Some(WeekEventKeyIntent::MoveDay(1))
    );
}

#[test]
fn shift_left_right_is_reserved() {
    assert_eq!(week_event_key_intent("ArrowLeft", true, 15), None);
    assert_eq!(week_event_key_intent("ArrowRight", true, 15), None);
}

#[test]
fn week_keys_keep_the_day_scheduler_time_contract() {
    assert_eq!(
        week_event_key_intent("Enter", false, 15),
        Some(WeekEventKeyIntent::Activate)
    );
    assert_eq!(
        week_event_key_intent(" ", false, 15),
        Some(WeekEventKeyIntent::Activate)
    );
    assert_eq!(
        week_event_key_intent("ArrowDown", false, 30),
        Some(WeekEventKeyIntent::Move(30))
    );
    assert_eq!(
        week_event_key_intent("ArrowUp", true, 30),
        Some(WeekEventKeyIntent::Resize(-30))
    );
    assert_eq!(week_event_key_intent("Tab", false, 15), None);
}

// ---------------------------------------------------------------------
// compute_week_layer_layout -- one layer over all seven columns
// ---------------------------------------------------------------------

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn layer_layout_offsets_each_event_into_its_column() {
    let events = vec![
        CalEvent::new("Mon", 0, 9 * 60, 10 * 60, SchedulerEventColor::Primary),
        CalEvent::new("Sun", 6, 9 * 60, 10 * 60, SchedulerEventColor::Primary),
    ];
    let layouts = compute_week_layer_layout(&events, 8, 18);
    let col = 100.0 / 7.0;
    let mon = layouts[0].expect("timed");
    let sun = layouts[1].expect("timed");
    assert!(approx(mon.left_pct, 0.0) && approx(mon.width_pct, col));
    assert!(approx(sun.left_pct, 6.0 * col) && approx(sun.width_pct, col));
    // The vertical axis is untouched by the column offset.
    assert!(approx(mon.top_pct, 10.0) && approx(mon.height_pct, 10.0));
}

#[test]
fn layer_layout_packs_lanes_per_day_only() {
    // Two overlapping events on Wednesday split that column; the same time
    // on Thursday is a different column and does not share its lanes.
    let events = vec![
        CalEvent::new("A", 2, 9 * 60, 10 * 60, SchedulerEventColor::Primary),
        CalEvent::new(
            "B",
            2,
            9 * 60 + 30,
            10 * 60 + 30,
            SchedulerEventColor::Warning,
        ),
        CalEvent::new("C", 3, 9 * 60, 10 * 60, SchedulerEventColor::Info),
    ];
    let layouts = compute_week_layer_layout(&events, 8, 18);
    let col = 100.0 / 7.0;
    let (a, b, c) = (
        layouts[0].expect("timed"),
        layouts[1].expect("timed"),
        layouts[2].expect("timed"),
    );
    assert!(approx(a.width_pct, col / 2.0) && approx(b.width_pct, col / 2.0));
    assert!(approx(a.left_pct, 2.0 * col) && approx(b.left_pct, 2.5 * col));
    assert!(approx(c.left_pct, 3.0 * col) && approx(c.width_pct, col));
}

#[test]
fn layer_layout_leaves_all_day_events_out_of_the_time_grid() {
    let events = vec![
        CalEvent::new("Holiday", 4, 0, 0, SchedulerEventColor::Error).all_day(),
        CalEvent::new("Sync", 4, 9 * 60, 10 * 60, SchedulerEventColor::Primary),
    ];
    let layouts = compute_week_layer_layout(&events, 8, 18);
    assert_eq!(layouts.len(), 2, "index-aligned with events");
    assert!(layouts[0].is_none());
    assert!(layouts[1].is_some());
}

#[test]
fn layer_layout_moving_an_event_a_day_moves_it_one_column() {
    let mut events = vec![CalEvent::new(
        "Move me",
        1,
        9 * 60,
        10 * 60,
        SchedulerEventColor::Primary,
    )];
    let before = compute_week_layer_layout(&events, 8, 18)[0].expect("timed");
    events[0].day = 2;
    let after = compute_week_layer_layout(&events, 8, 18)[0].expect("timed");
    assert!(approx(after.left_pct - before.left_pct, 100.0 / 7.0));
    assert!(approx(after.top_pct, before.top_pct));
}

// ---------------------------------------------------------------------
// week_slot_at -- consumer drop targets
// ---------------------------------------------------------------------

#[test]
fn week_slot_at_maps_fractions_to_day_and_minute() {
    // Middle of the third column, a quarter of the way down an 8-18 band.
    let slot = week_slot_at(2.5 / 7.0, 0.25, 8, 18);
    assert_eq!(
        slot,
        WeekSlot {
            day: 2,
            minute: 8 * 60 + 150
        }
    );
}

#[test]
fn week_slot_at_clamps_the_edges_and_bad_input() {
    assert_eq!(
        week_slot_at(1.0, 1.0, 8, 18),
        WeekSlot {
            day: 6,
            minute: 18 * 60
        }
    );
    assert_eq!(
        week_slot_at(-3.0, -1.0, 8, 18),
        WeekSlot {
            day: 0,
            minute: 8 * 60
        }
    );
    assert_eq!(
        week_slot_at(f64::NAN, f64::INFINITY, 8, 18),
        WeekSlot {
            day: 0,
            minute: 8 * 60
        }
    );
    // A degenerate band still yields a one-hour span.
    assert_eq!(week_slot_at(0.0, 1.0, 9, 9).minute, 10 * 60);
}
