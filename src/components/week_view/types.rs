use crate::components::day_scheduler::{
    EventLayout, SchedulerEvent, SchedulerEventColor, compute_event_layout,
};

const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// One calendar event on the week grid. `day` is a Monday-based column index
/// (`0` = Monday .. `6` = Sunday). Mirrors d2d-ui's `CalEvent`, generalised
/// with an explicit `day` column (instead of an absolute Unix timestamp) so
/// the component itself stays free of any date/timezone math -- callers
/// place events on the grid by column index directly. Reuses
/// [`SchedulerEventColor`] from [`DayScheduler`](super::super::day_scheduler::DayScheduler)
/// for the accent-bar / tint color, matching that component's palette.
#[derive(Clone, Debug, PartialEq)]
pub struct CalEvent {
    /// Display title (the "subject" in d2d-ui), shown in bold in the block.
    pub title: String,
    /// Optional location line, shown under the title when there's room.
    pub location: String,
    /// Day column, `0` (Monday) .. `6` (Sunday).
    pub day: usize,
    /// Start time, in minutes from midnight. Ignored when `all_day` is set.
    pub start_min: u32,
    /// End time, in minutes from midnight (always `> start_min`). Ignored
    /// when `all_day` is set.
    pub end_min: u32,
    /// When set, the event is drawn in the all-day strip instead of being
    /// time-positioned in the grid.
    pub all_day: bool,
    /// Semantic color for the event block's tint and accent bar.
    pub color: SchedulerEventColor,
}

impl CalEvent {
    /// Create a new timed event. `day` is clamped to `0..=6`; `end_min` is
    /// clamped to at least `start_min + 1` so every event has a strictly
    /// positive, visible duration -- mirrors `SchedulerEvent::new`.
    pub fn new(
        title: impl Into<String>,
        day: usize,
        start_min: u32,
        end_min: u32,
        color: SchedulerEventColor,
    ) -> Self {
        Self {
            title: title.into(),
            location: String::new(),
            day: day.min(6),
            start_min,
            end_min: end_min.max(start_min + 1),
            all_day: false,
            color,
        }
    }

    /// Attach a location line, shown under the title.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = location.into();
        self
    }

    /// Mark this event as an all-day event (drawn in the all-day strip).
    pub fn all_day(mut self) -> Self {
        self.all_day = true;
        self
    }
}

/// Convert `days` (count of days since 1970-01-01, may be negative) into a
/// `(year, month 1..=12, day 1..=31)` civil date. Howard Hinnant's
/// `civil_from_days` -- exact for the whole `i64` range, no external deps.
/// Ported verbatim from d2d-ui's `week_view::civil_from_days`.
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The epoch day (days since 1970-01-01) of the Monday of the week
/// containing `epoch_day`.
///
/// 1970-01-01 was a Thursday (Monday-based index 3), so the Monday-based
/// weekday of any epoch day is `(day + 3) mod 7`. Ported from d2d-ui's
/// `week_start_for`, adapted to take/return an epoch day instead of Unix
/// seconds (the component has no need for time-of-day precision).
pub fn week_start_for(epoch_day: i64) -> i64 {
    let weekday_mon0 = (epoch_day + 3).rem_euclid(7);
    epoch_day - weekday_mon0
}

/// A human label for the week beginning at `week_start_epoch_day`, e.g.
/// `"Mar 2 - 8, 2026"`. Spans month and year boundaries sensibly
/// (`"Mar 30 - Apr 5, 2026"`, `"Dec 29, 2025 - Jan 4, 2026"`). Ported from
/// d2d-ui's `week_range_label`.
pub fn week_range_label(week_start_epoch_day: i64) -> String {
    let end_day = week_start_epoch_day + 6;
    let (sy, sm, sd) = civil_from_days(week_start_epoch_day);
    let (ey, em, ed) = civil_from_days(end_day);
    let smon = MONTHS[(sm - 1) as usize];
    let emon = MONTHS[(em - 1) as usize];
    if sy == ey && sm == em {
        format!("{smon} {sd} - {ed}, {sy}")
    } else if sy == ey {
        format!("{smon} {sd} - {emon} {ed}, {sy}")
    } else {
        format!("{smon} {sd}, {sy} - {emon} {ed}, {ey}")
    }
}

/// The three-letter weekday abbreviation for Monday-based column `day`
/// (`0` = "Mon" .. `6` = "Sun"), clamped into range.
pub fn weekday_abbrev(day: usize) -> &'static str {
    WEEKDAYS[day.min(6)]
}

/// The day-of-month number for Monday-based column `day` of the week
/// beginning at `week_start_epoch_day` -- used for the header's date number
/// under the weekday abbreviation.
pub fn day_of_month(week_start_epoch_day: i64, day: usize) -> u32 {
    civil_from_days(week_start_epoch_day + day.min(6) as i64).2
}

/// Lay out one day-column's timed events into side-by-side overlap lanes.
/// `events` must already be filtered to a single day (this function does not
/// look at [`CalEvent::day`]). Thin adapter over
/// [`compute_event_layout`](super::super::day_scheduler::compute_event_layout)
/// -- reuses the DayScheduler lane algorithm rather than re-implementing it,
/// per beads-aea8's "shares lane logic with DayScheduler" contract.
pub fn compute_week_event_layout(
    events: &[CalEvent],
    start_hour: u32,
    end_hour: u32,
) -> Vec<EventLayout> {
    let scheduler_events: Vec<SchedulerEvent> = events
        .iter()
        .map(|e| SchedulerEvent::new(e.title.clone(), e.start_min, e.end_min, e.color.clone()))
        .collect();
    compute_event_layout(&scheduler_events, start_hour, end_hour)
}
