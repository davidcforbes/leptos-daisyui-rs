//! Native tests for `MonthRoster`'s pure half (ldui-5lmt.1) and the source
//! guards for its class tables. The DOM behaviour is proven in a real
//! browser by `tests/month_roster_smoke.rs` (`cargo xtask test-month-roster`).

use super::style::{
    AbsenceKind, BLOCK, CHIP, DATE_HEADING, DAY_BOX, DAY_BOX_LAYOUT, DAY_CLOSED, DAY_HOLIDAY,
    DAY_OUTSIDE, DAY_TODAY, DAY_WEEKEND, GRID_ONLY, LayoutClasses, ODD_HOUR, OUTSIDE_LAYOUT, ROOT,
    RosterLayout, RosterZoom, TABLE, TABLE_BASE, TIMELINE_ROW, WEEK_BUTTON, WEEK_COLUMNS,
    WEEK_HEAD, WEEKDAY_HEAD,
};
use super::texts::MonthRosterTexts;
use super::types::*;
use crate::components::{
    ShiftState, Weekday, days_from_civil, days_in_month, week_range_label, week_start_for_weekday,
};

/// Epoch day of a civil date, for readable fixtures.
fn d(year: i64, month: u32, day: u32) -> i64 {
    days_from_civil(year, month, day)
}

// ---- date math --------------------------------------------------------------

#[test]
fn the_fixture_day_is_epoch_20740() {
    assert_eq!(d(2026, 10, 14), 20_740);
    assert_eq!(iso_date(20_740), "2026-10-14");
    assert_eq!(Weekday::from_epoch_day(20_740), Weekday::Wednesday);
}

#[test]
fn days_from_civil_round_trips_civil_from_days() {
    for day in [-800_000_i64, -1, 0, 1, 11_016, 20_740, 59, 60, 2_932_896] {
        let (year, month, dom) = civil(day);
        assert_eq!(d(year, month, dom), day, "round trip of {day}");
    }
    assert_eq!(d(1970, 1, 1), 0);
}

#[test]
fn days_in_month_knows_leap_years() {
    assert_eq!(days_in_month(2026, 2), 28);
    assert_eq!(days_in_month(2028, 2), 29);
    assert_eq!(days_in_month(1900, 2), 28);
    assert_eq!(days_in_month(2000, 2), 29);
    assert_eq!(days_in_month(2026, 10), 31);
    assert_eq!(days_in_month(2026, 11), 30);
    assert_eq!(days_in_month(2026, 13), 0);
}

#[test]
fn october_2026_spans_five_monday_weeks() {
    assert_eq!(
        month_span(2026, 10, Weekday::Monday),
        (d(2026, 9, 28), d(2026, 11, 1), 5)
    );
}

#[test]
fn october_2026_spans_five_sunday_weeks() {
    assert_eq!(
        month_span(2026, 10, Weekday::Sunday),
        (d(2026, 9, 27), d(2026, 10, 31), 5)
    );
}

#[test]
fn august_2026_needs_six_monday_weeks() {
    assert_eq!(
        month_span(2026, 8, Weekday::Monday),
        (d(2026, 7, 27), d(2026, 9, 6), 6)
    );
}

#[test]
fn february_2027_is_exactly_four_monday_weeks_and_five_sunday_weeks() {
    assert_eq!(
        month_span(2027, 2, Weekday::Monday),
        (d(2027, 2, 1), d(2027, 2, 28), 4)
    );
    assert_eq!(month_span(2027, 2, Weekday::Sunday).2, 5);
}

#[test]
fn every_span_is_whole_weeks_starting_on_the_week_start() {
    for year in [2025_i64, 2026, 2027, 2028] {
        for month in 1..=12 {
            for start in Weekday::ALL {
                let (first, last, rows) = month_span(year, month, start);
                assert_eq!(Weekday::from_epoch_day(first), start);
                assert_eq!(last - first + 1, rows as i64 * 7);
                assert!((4..=6).contains(&rows), "{year}-{month} {start:?}: {rows}");
                assert!(first <= d(year, month, 1));
                assert!(last >= d(year, month, days_in_month(year, month)));
                assert!(last - d(year, month, days_in_month(year, month)) < 7);
            }
        }
    }
}

#[test]
fn a_month_step_keeps_the_day_of_month_clamped() {
    assert_eq!(step(RosterZoom::Month, d(2027, 1, 31), 1), d(2027, 2, 28));
    assert_eq!(step(RosterZoom::Month, d(2028, 1, 31), 1), d(2028, 2, 29));
    assert_eq!(step(RosterZoom::Month, d(2026, 10, 14), 1), d(2026, 11, 14));
    assert_eq!(step(RosterZoom::Month, d(2026, 10, 14), -1), d(2026, 9, 14));
    assert_eq!(step(RosterZoom::Month, d(2026, 1, 15), -1), d(2025, 12, 15));
    assert_eq!(step(RosterZoom::Month, d(2026, 12, 31), 2), d(2027, 2, 28));
}

#[test]
fn week_and_day_steps_move_seven_and_one_days() {
    assert_eq!(step(RosterZoom::Week, 20_740, 1), 20_747);
    assert_eq!(step(RosterZoom::Week, 20_740, -1), 20_733);
    assert_eq!(step(RosterZoom::Day, 20_740, -1), 20_739);
}

#[test]
fn visible_range_per_zoom() {
    assert_eq!(
        visible_range(RosterZoom::Month, 20_740, Weekday::Monday),
        (d(2026, 9, 28), d(2026, 11, 1))
    );
    assert_eq!(
        visible_range(RosterZoom::Week, 20_740, Weekday::Monday),
        (d(2026, 10, 12), d(2026, 10, 18))
    );
    assert_eq!(
        visible_range(RosterZoom::Day, 20_740, Weekday::Monday),
        (20_740, 20_740)
    );
}

#[test]
fn a_sunday_start_week_runs_sunday_to_saturday() {
    let (first, last) = visible_range(RosterZoom::Week, 20_740, Weekday::Sunday);
    assert_eq!((first, last), (d(2026, 10, 11), d(2026, 10, 17)));
    assert_eq!(Weekday::from_epoch_day(first), Weekday::Sunday);
    assert_eq!(Weekday::from_epoch_day(last), Weekday::Saturday);
}

#[test]
fn iso_week_numbers() {
    assert_eq!(iso_week(d(2026, 10, 14)), 42);
    assert_eq!(iso_week(d(2026, 10, 12)), 42);
    assert_eq!(iso_week(d(2026, 1, 1)), 1); // a Thursday
    assert_eq!(iso_week(d(2027, 1, 1)), 53); // a Friday: 2026 has 53 weeks
    assert_eq!(iso_week(d(2024, 12, 30)), 1); // the Monday of 2025-W01
    assert_eq!(iso_week(d(2021, 1, 3)), 53); // a Sunday in 2020-W53
}

// ---- entries ------------------------------------------------------------

fn sample_day() -> Vec<RosterEntry> {
    vec![
        RosterEntry::off("o1", "Zed Off"),
        RosterEntry::absent("s1", "Sam Sick", AbsenceKind::Sick),
        RosterEntry::working("w3", "Cara Late", ShiftState::Full).times(12 * 60, 20 * 60),
        RosterEntry::absent("v1", "Vic Vacation", AbsenceKind::Vacation).pending(),
        RosterEntry::holiday("h1", "Hal Holiday"),
        RosterEntry::working("w1", "Bea Early", ShiftState::Full).times(7 * 60, 15 * 60),
        RosterEntry::working("w0", "Ana Untimed", ShiftState::Half),
        RosterEntry::working("w2", "Abe Early", ShiftState::Half).times(7 * 60, 11 * 60),
        RosterEntry::absent("x1", "Oscar Away", AbsenceKind::OutOfOffice),
        RosterEntry::absent("a1", "Ada Away", AbsenceKind::OutOfOffice),
    ]
}

#[test]
fn a_day_lists_working_by_start_then_out_by_kind_then_holiday_then_off() {
    let ids: Vec<String> = sorted_entries(&sample_day())
        .into_iter()
        .map(|e| e.id)
        .collect();
    assert_eq!(
        ids,
        ["w2", "w1", "w3", "w0", "v1", "a1", "x1", "s1", "h1", "o1"],
        "working by start (untimed last, then name), out by kind then name, holiday, off"
    );
}

#[test]
fn misused_working_states_normalise_to_what_they_describe() {
    assert_eq!(
        EntryKind::Working(ShiftState::Off).normalized(),
        EntryKind::Off
    );
    assert_eq!(
        EntryKind::Working(ShiftState::Holiday).normalized(),
        EntryKind::Holiday
    );
    assert_eq!(
        EntryKind::Working(ShiftState::Leave).group(),
        EntryGroup::Out,
        "Leave is an absence, never work"
    );
    assert_eq!(EntryKind::Working(ShiftState::Leave).slug(), "other");
}

#[test]
fn a_pending_absence_keeps_its_border_and_has_no_fill() {
    let pending = RosterEntry::absent("v", "V", AbsenceKind::Vacation).pending();
    assert!(pending.kind.is_pending());
    assert_eq!(
        pending.kind.colour_classes(),
        "border-warning border-dashed"
    );
    let approved = RosterEntry::absent("v", "V", AbsenceKind::Vacation);
    assert!(approved.kind.colour_classes().contains("bg-warning/15"));
    // `pending` and `partial_hours` do nothing to a non-absence.
    let shift = RosterEntry::working("w", "W", ShiftState::Full)
        .pending()
        .partial_hours(2.0);
    assert_eq!(shift.kind, EntryKind::Working(ShiftState::Full));
}

#[test]
fn working_is_solid_and_everything_else_is_dashed() {
    let kinds = [
        EntryKind::Working(ShiftState::Full),
        EntryKind::Working(ShiftState::Half),
        EntryKind::Holiday,
        EntryKind::Off,
    ]
    .into_iter()
    .chain(AbsenceKind::ALL.into_iter().flat_map(|kind| {
        [false, true].map(|pending| EntryKind::Absent {
            kind,
            pending,
            partial_hours: None,
        })
    }));
    for kind in kinds {
        let classes = kind.colour_classes();
        let working = kind.group() == EntryGroup::Working;
        assert_eq!(
            classes.contains("border-solid"),
            working,
            "{kind:?}: {classes}"
        );
        assert_eq!(
            classes.contains("border-dashed"),
            !working,
            "{kind:?}: {classes}"
        );
    }
}

#[test]
fn the_content_key_sees_every_field() {
    let base = RosterEntry::absent("s", "Sam", AbsenceKind::Sick);
    let keys = [
        base.clone().content_key(),
        base.clone().partial_hours(4.0).content_key(),
        base.clone().pending().content_key(),
        base.clone().detail("Flu").content_key(),
        base.clone().times(8 * 60, 12 * 60).content_key(),
        RosterEntry::absent("s", "Sam", AbsenceKind::Vacation).content_key(),
        RosterEntry::absent("s", "Samuel", AbsenceKind::Sick).content_key(),
        // ldui-k145: a chip that gains or loses its "x" must re-render.
        base.clone().removable().content_key(),
    ];
    for (i, a) in keys.iter().enumerate() {
        for b in &keys[i + 1..] {
            assert_ne!(a, b, "two different entries share a key");
        }
    }
    assert_eq!(base.content_key(), base.clone().content_key());
}

#[test]
fn counts_group_working_out_and_off() {
    assert_eq!(
        day_counts(&sample_day()),
        DayCounts {
            working: 4,
            out: 4,
            off: 2
        }
    );
}

// ---- roving focus ----------------------------------------------------------

#[test]
fn arrows_move_a_day_or_a_week_inside_the_span() {
    let (first, last) = visible_range(RosterZoom::Month, 20_740, Weekday::Monday);
    assert_eq!(
        roving_target("ArrowRight", 20_740, first, last),
        Some(20_741)
    );
    assert_eq!(
        roving_target("ArrowLeft", 20_740, first, last),
        Some(20_739)
    );
    assert_eq!(
        roving_target("ArrowDown", 20_740, first, last),
        Some(20_747)
    );
    assert_eq!(roving_target("ArrowUp", 20_740, first, last), Some(20_733));
    // 14 Oct is a Wednesday in the row Mon 12 .. Sun 18.
    assert_eq!(
        roving_target("Home", 20_740, first, last),
        Some(d(2026, 10, 12))
    );
    assert_eq!(
        roving_target("End", 20_740, first, last),
        Some(d(2026, 10, 18))
    );
    // Never past the rendered span.
    assert_eq!(roving_target("ArrowLeft", first, first, last), None);
    assert_eq!(roving_target("ArrowUp", first + 3, first, last), None);
    assert_eq!(roving_target("ArrowDown", last - 3, first, last), None);
    assert_eq!(roving_target("ArrowRight", last, first, last), None);
    assert_eq!(roving_target("Tab", 20_740, first, last), None);
}

#[test]
fn the_active_day_is_remembered_today_or_the_anchor() {
    let october = d(2026, 10, 1);
    // Remembered in the month shown: kept.
    assert_eq!(
        active_day(Some((d(2026, 10, 3), october)), 20_740, 20_740),
        d(2026, 10, 3)
    );
    // Remembered in another month: today when today is in the month shown.
    assert_eq!(
        active_day(Some((d(2026, 9, 3), d(2026, 9, 1))), 20_740, 20_740),
        20_740
    );
    assert_eq!(active_day(None, d(2026, 10, 1), 20_740), 20_740);
    // Today outside the month: the anchor.
    assert_eq!(active_day(None, d(2026, 11, 20), 20_740), d(2026, 11, 20));
}

// ---- the Day timeline -------------------------------------------------------

#[test]
fn the_axis_is_seven_to_twenty_widened_to_whole_hours() {
    assert_eq!(timeline_axis(&[]), (7 * 60, 20 * 60));
    let early = [RosterEntry::working("e", "E", ShiftState::Full).times(6 * 60 + 30, 14 * 60)];
    assert_eq!(timeline_axis(&early), (6 * 60, 20 * 60));
    let late = [RosterEntry::working("l", "L", ShiftState::Full).times(14 * 60, 22 * 60 + 15)];
    assert_eq!(timeline_axis(&late), (7 * 60, 23 * 60));
    // Absences never widen the axis.
    let out = [RosterEntry::absent("a", "A", AbsenceKind::Sick).times(0, 24 * 60)];
    assert_eq!(timeline_axis(&out), (7 * 60, 20 * 60));
}

#[test]
fn a_bar_is_positioned_in_percent_of_the_axis() {
    let axis = (7 * 60, 20 * 60); // 13 hours
    let (left, width) = bar_geometry(8 * 60, 16 * 60 + 30, axis);
    assert!((left - 100.0 / 13.0).abs() < 1e-9);
    assert!((width - 8.5 * 100.0 / 13.0).abs() < 1e-9);
    assert_eq!(bar_geometry(7 * 60, 20 * 60, axis), (0.0, 100.0));
    // Clamped into the axis.
    assert_eq!(bar_geometry(0, 7 * 60, axis), (0.0, 0.0));
}

#[test]
fn absences_group_by_kind_in_order_and_skip_empty_kinds() {
    let groups = absence_groups(&sample_day());
    let summary: Vec<(AbsenceKind, Vec<String>)> = groups
        .into_iter()
        .map(|(kind, people)| (kind, people.into_iter().map(|e| e.person).collect()))
        .collect();
    assert_eq!(
        summary,
        vec![
            (AbsenceKind::Vacation, vec!["Vic Vacation".to_string()]),
            (
                AbsenceKind::OutOfOffice,
                vec!["Ada Away".to_string(), "Oscar Away".to_string()]
            ),
            (AbsenceKind::Sick, vec!["Sam Sick".to_string()]),
        ]
    );
}

#[test]
fn clock_does_not_wrap_at_midnight() {
    assert_eq!(clock(8 * 60), "08:00");
    assert_eq!(clock(16 * 60 + 30), "16:30");
    assert_eq!(clock(24 * 60), "24:00");
}

// ---- texts ------------------------------------------------------------------

#[test]
fn english_dates_follow_the_kit() {
    let t = MonthRosterTexts::default();
    assert_eq!(t.month_title_of(20_740), "October 2026");
    assert_eq!(t.date_long_of(20_740), "Wednesday, Oct 14, 2026");
    assert_eq!(t.date_full_of(20_740), "Wednesday, October 14, 2026");
    assert_eq!(
        t.title(RosterZoom::Week, 20_740, Weekday::Monday),
        "Oct 12 - 18, 2026"
    );
}

#[test]
fn spanish_dates_follow_the_kit() {
    let t = MonthRosterTexts::es();
    assert_eq!(t.month_title_of(20_740), "octubre de 2026");
    assert_eq!(t.date_long_of(20_740), "miércoles, 14 oct 2026");
    assert_eq!(t.date_full_of(20_740), "miércoles, 14 de octubre de 2026");
    assert_eq!(t.week_title_of(d(2026, 9, 28)), "28 sept - 4 oct 2026");
    assert_eq!(t.week_title_of(d(2026, 12, 28)), "28 dic 2026 - 3 ene 2027");
}

#[test]
fn the_english_week_title_is_week_view_s_label() {
    let t = MonthRosterTexts::default();
    for start in [
        d(2026, 10, 12),
        d(2026, 9, 28),
        d(2026, 12, 28),
        d(2027, 2, 22),
    ] {
        assert_eq!(
            t.week_title_of(start),
            week_range_label(start),
            "week of {start}"
        );
    }
    assert_eq!(
        week_start_for_weekday(20_740, Weekday::Monday),
        d(2026, 10, 12)
    );
}

#[test]
fn a_day_name_never_says_zero_people() {
    let t = MonthRosterTexts::default();
    let unknown = t.day_name(20_740, false, None);
    assert_eq!(unknown, "Wednesday, Oct 14, 2026, no schedule published");
    let empty = RosterDay::new(20_740);
    assert_eq!(
        t.day_name(20_740, true, Some(&empty)),
        "Wednesday, Oct 14, 2026, Today, no one scheduled"
    );
    let busy = RosterDay::new(20_740)
        .holiday("Founders' Day")
        .staffing(4, 6)
        .entries(sample_day());
    assert_eq!(
        t.day_name(20_740, false, Some(&busy)),
        "Wednesday, Oct 14, 2026, 4 working, 4 out, 2 off, Holiday: Founders' Day, 4 of 6 needed"
    );
    let only_out = RosterDay::new(20_740).entry(RosterEntry::absent("s", "S", AbsenceKind::Sick));
    let name = t.day_name(20_740, false, Some(&only_out));
    assert_eq!(name, "Wednesday, Oct 14, 2026, 1 out");
    for name in [unknown, name] {
        assert!(!name.contains(" 0 "), "a zero count was spoken: {name}");
    }
}

#[test]
fn state_words_say_requested_and_partial_hours() {
    let t = MonthRosterTexts::default();
    let pending = RosterEntry::absent("v", "V", AbsenceKind::Vacation).pending();
    assert_eq!(t.state_words(pending.kind), "Vacation, Requested");
    let partial = RosterEntry::absent("s", "S", AbsenceKind::Sick).partial_hours(4.0);
    assert_eq!(t.state_words(partial.kind), "Sick, 4 h");
    let half = RosterEntry::absent("s", "S", AbsenceKind::Other).partial_hours(2.5);
    assert_eq!(t.state_words(half.kind), "Out, 2.5 h");
    assert_eq!(
        t.state_words(EntryKind::Working(ShiftState::Half)),
        "Half shift"
    );
    let es = MonthRosterTexts::es();
    assert_eq!(es.state_words(pending.kind), "Vacaciones, Solicitado");
}

#[test]
fn the_overflow_names_its_count() {
    let t = MonthRosterTexts::default();
    assert_eq!(t.more_visible(5), "+5");
    assert_eq!(t.more_spoken(5), "5 more people");
    assert_eq!(t.more_spoken(1), "1 more person");
    assert_eq!(MonthRosterTexts::es().more_spoken(5), "5 personas más");
}

#[test]
fn spanish_translates_every_word_and_leaves_no_template_empty() {
    let (en, es) = (MonthRosterTexts::default(), MonthRosterTexts::es());
    let words = |t: &MonthRosterTexts| {
        vec![
            t.today.clone(),
            t.previous_month.clone(),
            t.next_month.clone(),
            t.previous_week.clone(),
            t.next_week.clone(),
            t.previous_day.clone(),
            t.next_day.clone(),
            t.zoom_label.clone(),
            t.zoom_month.clone(),
            t.zoom_week.clone(),
            t.zoom_day.clone(),
            t.week_column.clone(),
            t.show_week.clone(),
            t.show_day.clone(),
            t.no_schedule.clone(),
            t.no_one.clone(),
            t.no_schedule_spoken.clone(),
            t.no_one_spoken.clone(),
            t.month_empty.clone(),
            t.holiday_mark.clone(),
            t.half_day_holiday_mark.clone(),
            t.closed_mark.clone(),
            t.staffing.clone(),
            t.more_one.clone(),
            t.more_other.clone(),
            t.working_count.clone(),
            t.out_count.clone(),
            t.off_count.clone(),
            t.full_shift.clone(),
            t.half_shift.clone(),
            t.vacation.clone(),
            t.out_of_office.clone(),
            t.sick.clone(),
            t.other_absence.clone(),
            t.holiday.clone(),
            t.off.clone(),
            t.requested.clone(),
            t.working_spoken.clone(),
            t.week_out.clone(),
            t.out_today.clone(),
            t.all_day.clone(),
            t.coverage.clone(),
            t.no_one_working.clone(),
            t.holiday_line.clone(),
            t.off_line.clone(),
            t.add_to_day.clone(),
            t.remove_entry.clone(),
        ]
    };
    for (english, spanish) in words(&en).into_iter().zip(words(&es)) {
        assert!(!spanish.trim().is_empty());
        assert_ne!(english, spanish, "untranslated: {english}");
    }
    for (english, spanish) in en.month_names.iter().zip(&es.month_names) {
        assert_ne!(english, spanish);
    }
}

// ---- class tables -----------------------------------------------------------

/// Every class this pattern can emit, from the class tables and the entry
/// colours, so the safelist guard below covers them all.
fn emitted_classes() -> Vec<&'static str> {
    let tables: [LayoutClasses; 10] = [
        TABLE,
        GRID_ONLY,
        BLOCK,
        DAY_BOX_LAYOUT,
        OUTSIDE_LAYOUT,
        DATE_HEADING,
        WEEK_COLUMNS,
        TIMELINE_ROW,
        ODD_HOUR,
        GRID_ONLY,
    ];
    let mut strings: Vec<&'static str> = tables
        .iter()
        .flat_map(|t| {
            [
                t.pick(None),
                t.pick(Some(RosterLayout::Grid)),
                t.pick(Some(RosterLayout::Agenda)),
            ]
        })
        .collect();
    strings.extend([
        ROOT,
        TABLE_BASE,
        WEEKDAY_HEAD,
        WEEK_HEAD,
        WEEK_BUTTON,
        DAY_BOX,
        DAY_OUTSIDE,
        DAY_CLOSED,
        DAY_HOLIDAY,
        DAY_WEEKEND,
        DAY_TODAY,
        CHIP,
        "btn-ghost btn-sm btn-square join-item btn-active print:hidden",
        // ldui-k145's "+" / "x" buttons, the chip row that holds an "x", and
        // the activatable box or column.
        "btn-ghost btn-xs btn-square gap-0 shrink-0 print:hidden flex items-start items-center gap-1 gap-2 min-w-0 flex-1 touch-manipulation",
    ]);
    for kind in AbsenceKind::ALL {
        strings.push(kind.as_class());
        strings.push(kind.as_border_class());
        for pending in [false, true] {
            strings.push(
                EntryKind::Absent {
                    kind,
                    pending,
                    partial_hours: None,
                }
                .colour_classes(),
            );
        }
    }
    for kind in [
        EntryKind::Working(ShiftState::Full),
        EntryKind::Working(ShiftState::Half),
        EntryKind::Holiday,
        EntryKind::Off,
    ] {
        strings.push(kind.colour_classes());
    }
    strings
        .into_iter()
        .flat_map(str::split_whitespace)
        .collect()
}

#[test]
fn every_emitted_class_is_in_the_documented_safelist() {
    let source = include_str!("component.rs");
    let safelist: Vec<&str> = source
        .lines()
        .filter_map(|line| line.split("@source inline(\"").nth(1))
        .flat_map(|rest| {
            rest.split('"')
                .next()
                .unwrap_or_default()
                .split_whitespace()
        })
        .collect();
    // daisyUI's `btn` family and the crate's own `ld-*` classes are supplied
    // by their own components' safelists and the token preamble.
    let shared = |class: &str| class.starts_with("ld-") || class == "btn";
    let missing: Vec<&str> = emitted_classes()
        .into_iter()
        .filter(|class| !shared(class) && !safelist.contains(class))
        .collect();
    assert!(
        missing.is_empty(),
        "add to MonthRoster's @source inline block: {missing:?}"
    );
}

#[test]
fn a_forced_layout_never_carries_a_container_query() {
    for table in [
        TABLE,
        GRID_ONLY,
        BLOCK,
        DAY_BOX_LAYOUT,
        DATE_HEADING,
        WEEK_COLUMNS,
        TIMELINE_ROW,
    ] {
        for forced in [RosterLayout::Grid, RosterLayout::Agenda] {
            assert!(
                !table.pick(Some(forced)).contains('@'),
                "{table:?} {forced:?}"
            );
        }
        // The auto layout is the container query (or nothing for the
        // grid-only date heading's sr-only default).
        assert!(table.pick(None).contains("@max-2xl:"), "{table:?}");
    }
}

#[test]
fn the_root_is_a_container_not_a_viewport_breakpoint() {
    assert!(ROOT.split_whitespace().any(|class| class == "@container"));
    for class in emitted_classes() {
        for viewport in ["sm:", "md:", "lg:", "xl:", "2xl:"] {
            assert!(
                !class.starts_with(viewport),
                "{class} is a viewport breakpoint; the roster must follow its container"
            );
        }
    }
}

#[test]
fn zoom_and_absence_slugs_are_distinct() {
    let zooms: Vec<&str> = RosterZoom::ALL.iter().map(|z| z.as_str()).collect();
    assert_eq!(zooms, ["month", "week", "day"]);
    let mut slugs: Vec<&str> = AbsenceKind::ALL.iter().map(|k| k.as_str()).collect();
    slugs.extend(["full", "half", "holiday", "off"]);
    let unique: std::collections::BTreeSet<&str> = slugs.iter().copied().collect();
    assert_eq!(unique.len(), slugs.len());
}

// ---- ldui-k145: editing hooks -----------------------------------------------

#[test]
fn days_and_entries_are_not_editable_unless_the_host_says_so() {
    let day = RosterDay::new(d(2026, 10, 20));
    assert!(
        !day.addable,
        "a day takes a \"+\" only when the host marks it"
    );
    assert!(day.clone().addable().addable);
    let entry = RosterEntry::working("a", "Ana Lopez", ShiftState::Full);
    assert!(
        !entry.removable,
        "an entry takes an \"x\" only when the host marks it"
    );
    assert!(entry.clone().removable().removable);
    // The flag is independent of every other field.
    assert_eq!(
        RosterDay {
            addable: false,
            ..day.clone().addable()
        },
        day
    );
}

#[test]
fn the_add_and_remove_buttons_say_the_day_and_the_person() {
    let day = d(2026, 10, 20);
    let (en, es) = (MonthRosterTexts::default(), MonthRosterTexts::es());
    assert_eq!(
        en.add_to_day_of(day),
        "Add someone on Tuesday, Oct 20, 2026"
    );
    assert_eq!(
        en.remove_entry_of("Ana Lopez", day),
        "Remove Ana Lopez from Tuesday, Oct 20, 2026"
    );
    assert_eq!(
        es.add_to_day_of(day),
        "A\u{f1}adir a alguien el martes, 20 oct 2026"
    );
    assert_eq!(
        es.remove_entry_of("Ana Lopez", day),
        "Quitar a Ana Lopez del martes, 20 oct 2026"
    );
}

/// Every new control is a real, named button with the data hook a host's
/// tests address it by, and none is drawn without the host's callback.
///
/// Deliberate break: drop `aria-label` from the "+" -> the first loop fails.
#[test]
fn the_editing_controls_are_named_buttons_behind_their_callbacks() {
    let source = include_str!("component.rs");
    for hook in ["data-day-add=", "data-entry-remove="] {
        let at = source
            .find(hook)
            .unwrap_or_else(|| panic!("no {hook} in component.rs"));
        let button = source[..at].rfind("<button").expect("inside a <button>");
        let end = at + source[at..].find("on:click").expect("its handler");
        let tag = &source[button..end];
        assert!(tag.contains("type=\"button\""), "{hook}: {tag}");
        assert!(tag.contains("aria-label="), "{hook} is unnamed: {tag}");
        assert!(
            tag.contains("tabindex="),
            "{hook} ignores the roving Tab order"
        );
    }
    assert!(source.contains("match ctx.on_day_add {"));
    assert!(source.contains("ctx.on_entry_remove.filter(|_| entry.removable)"));
    let day = include_str!("day.rs");
    assert!(day.contains("ctx.on_entry_remove.filter(|_| entry.removable)"));
}

/// WCAG 2.2 2.5.8 (ldui-3lxs, 4iiz-Office staging): the day-number button
/// measured 7.8x20 for a one-digit day, and axe target-size fired on a
/// one-digit Monday beside the week column. It keeps a 24x24 minimum.
///
/// Deliberate break: drop `min-w-6` -> fails. Revert; passes.
#[test]
fn the_day_number_button_is_at_least_24_by_24() {
    let source = include_str!("component.rs");
    let at = source.find("data-day-number=\"\"").expect("the day number");
    let button = source[..at].rfind("<button").expect("a button");
    let end = at + source[at..].find("on:click").expect("its handler");
    let tag = &source[button..end];
    for class in ["inline-flex", "min-h-6", "min-w-6", "items-center"] {
        assert!(
            tag.split(['"', ' ']).any(|c| c == class),
            "{class} missing: {tag}"
        );
    }
}

// ---- ldui-on0i: contrast on the month grid ----------------------------------

/// Office's axe (2026-10-12, 1440, its theme) found the week buttons at
/// 1.06:1 and an adjacent month's chips at 4.4:1. The week button carries
/// the header's content colour itself and no daisyUI `btn` class, because
/// a `btn-ghost`'s rest colour is daisyUI's to decide and builds disagree
/// (ldui-u0yr: colouring only the cell fixed it here and not in Office). Its
/// hover is translucent white, never the ghost's light fill under white
/// text. An adjacent month's days dim to /85, never the /75 that failed.
///
/// Deliberate break: put `btn btn-ghost` back on the week button -> the
/// source assertion fails. Revert; passes.
#[test]
fn the_week_column_and_adjacent_days_keep_readable_text() {
    let classes = |s: &'static str| s.split_whitespace().collect::<Vec<_>>();
    assert!(classes(WEEK_HEAD).contains(&"text-table-header-content"));
    assert!(classes(WEEK_HEAD).contains(&"bg-table-header"));
    assert!(classes(WEEKDAY_HEAD).contains(&"text-table-header-content"));
    let button = classes(WEEK_BUTTON);
    assert!(button.contains(&"text-table-header-content"));
    assert!(button.contains(&"hover:bg-white/15"));
    assert!(
        button.contains(&"[--ld-focus-ring-color:var(--color-table-header-content)]"),
        "the focus ring must take the header's content colour (ldui-fq9x)"
    );
    assert!(
        !button.iter().any(|c| c.starts_with("btn")),
        "a daisyUI btn decides its own colour: {WEEK_BUTTON}"
    );
    let source = include_str!("component.rs");
    let at = source
        .find("data-month-roster-action=\"week\"")
        .expect("the week button");
    let tag = &source[at..at + source[at..].find("aria-label").expect("its label")];
    assert!(tag.contains("class=WEEK_BUTTON"), "{tag}");
    assert!(tag.contains("data-pressable=\"true\""), "{tag}");
    assert!(classes(DAY_OUTSIDE).contains(&"text-base-content/85"));
    assert!(!DAY_OUTSIDE.contains("/75"), "4.4:1 on a chip's tint");
}
