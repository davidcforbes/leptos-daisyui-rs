//! The pure, Leptos-free half of `MonthRoster` (ldui-5lmt.1): the data
//! model, the month/week/day date math, the in-day order, the roving-focus
//! moves and the Day timeline's geometry. Everything here is a plain function
//! so it is unit-tested natively; the component only wires it to the DOM.

use super::style::{AbsenceKind, RosterZoom};
use crate::components::{
    ShiftState, Weekday, civil_from_days, days_from_civil, days_in_month, week_start_for_weekday,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// How many chips a month date box shows before its "+N" (the rest stay in
/// the DOM behind the toggle).
pub const VISIBLE_PER_DAY: usize = 4;

/// The Day timeline's default axis start, 07:00, in minutes from midnight.
pub const AXIS_START_MIN: u16 = 7 * 60;

/// The Day timeline's default axis end, 20:00, in minutes from midnight.
pub const AXIS_END_MIN: u16 = 20 * 60;

/// One day of the host's data. A day the host did not supply is UNKNOWN
/// ("No schedule published"); a supplied day with no entries is KNOWN EMPTY
/// ("No one scheduled"). The roster never says "0 people".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RosterDay {
    /// Epoch day (days since 1970-01-01). 2026-10-14 is 20740.
    pub day: i64,
    /// A public or company holiday on this day.
    pub holiday: Option<DayMark>,
    /// The office is closed, and why.
    pub closed: Option<String>,
    /// How many people are scheduled against how many are needed.
    pub staffing: Option<Staffing>,
    /// The people on this day, in any order (the roster sorts them).
    pub entries: Vec<RosterEntry>,
    /// The host lets someone be added to this day: with `on_day_add` the
    /// day shows a labelled "+" (ldui-k145). The host decides which days
    /// (4iiz-Office marks only days after today).
    pub addable: bool,
}

impl RosterDay {
    /// A known day with no marks and no one on it yet.
    pub fn new(day: i64) -> Self {
        Self {
            day,
            ..Self::default()
        }
    }

    /// Mark a whole-day holiday.
    pub fn holiday(mut self, label: impl Into<String>) -> Self {
        self.holiday = Some(DayMark {
            label: label.into(),
            half_day: false,
        });
        self
    }

    /// Mark a half-day holiday.
    pub fn half_day_holiday(mut self, label: impl Into<String>) -> Self {
        self.holiday = Some(DayMark {
            label: label.into(),
            half_day: true,
        });
        self
    }

    /// Mark the office closed, with the reason shown on the day.
    pub fn closed(mut self, reason: impl Into<String>) -> Self {
        self.closed = Some(reason.into());
        self
    }

    /// "3 of 4 needed".
    pub fn staffing(mut self, scheduled: u32, needed: u32) -> Self {
        self.staffing = Some(Staffing { scheduled, needed });
        self
    }

    /// Add one entry.
    pub fn entry(mut self, entry: RosterEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Add several entries.
    pub fn entries(mut self, entries: impl IntoIterator<Item = RosterEntry>) -> Self {
        self.entries.extend(entries);
        self
    }

    /// Let someone be added to this day (see the `addable` field).
    pub fn addable(mut self) -> Self {
        self.addable = true;
        self
    }
}

/// A holiday mark on a day.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct DayMark {
    /// The holiday's name, in the page's language.
    pub label: String,
    /// Only half the day is a holiday.
    pub half_day: bool,
}

/// A day's staffing: scheduled against needed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Staffing {
    /// People scheduled to work.
    pub scheduled: u32,
    /// People the day needs.
    pub needed: u32,
}

impl Staffing {
    /// Fewer people scheduled than needed: the line is emphasised.
    pub fn is_short(self) -> bool {
        self.scheduled < self.needed
    }
}

/// What one entry records. `Working` reuses `RosterGrid`'s
/// [`ShiftState`] for its colour; only `Full` and `Half` mean work, and
/// [`normalized`](Self::normalized) maps any other state to the kind it
/// actually describes, so a misuse never paints work that was not rostered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntryKind {
    /// Rostered to work, a full or a half shift.
    Working(ShiftState),
    /// Out for the day (or part of it).
    Absent {
        /// Why.
        kind: AbsenceKind,
        /// Requested and not yet approved: no fill, and the word
        /// "Requested". The host decides whether to pass pending absences.
        pending: bool,
        /// Out for only these hours of the day.
        partial_hours: Option<f32>,
    },
    /// The person's holiday.
    Holiday,
    /// A scheduled day off.
    Off,
}

impl EntryKind {
    /// `Working(Off)` is `Off`, `Working(Holiday)` is `Holiday` and
    /// `Working(Leave)` is an `Other` absence; everything else is itself.
    pub fn normalized(self) -> Self {
        match self {
            EntryKind::Working(ShiftState::Off) => EntryKind::Off,
            EntryKind::Working(ShiftState::Holiday) => EntryKind::Holiday,
            EntryKind::Working(ShiftState::Leave) => EntryKind::Absent {
                kind: AbsenceKind::Other,
                pending: false,
                partial_hours: None,
            },
            other => other,
        }
    }

    /// Which part of a day this kind is listed in.
    pub fn group(self) -> EntryGroup {
        match self.normalized() {
            EntryKind::Working(_) => EntryGroup::Working,
            EntryKind::Absent { .. } => EntryGroup::Out,
            EntryKind::Holiday => EntryGroup::Holiday,
            EntryKind::Off => EntryGroup::Off,
        }
    }

    /// The `data-entry-state` slug: `full`, `half`, `vacation`,
    /// `out-of-office`, `sick`, `other`, `holiday`, `off`.
    pub fn slug(self) -> &'static str {
        match self.normalized() {
            EntryKind::Working(ShiftState::Half) => "half",
            EntryKind::Working(_) => "full",
            EntryKind::Absent { kind, .. } => kind.as_str(),
            EntryKind::Holiday => "holiday",
            EntryKind::Off => "off",
        }
    }

    /// A pending (requested) absence.
    pub fn is_pending(self) -> bool {
        matches!(self.normalized(), EntryKind::Absent { pending: true, .. })
    }

    /// The chip's tint and accent bar (the kit's `ShiftState` colours plus
    /// the absence hues). A pending absence keeps its bar and has no fill.
    pub fn colour_classes(self) -> &'static str {
        match self.normalized() {
            EntryKind::Working(ShiftState::Half) => "bg-info/15 border-info border-solid",
            EntryKind::Working(_) => "bg-success/15 border-success border-solid",
            EntryKind::Absent {
                kind,
                pending: true,
                ..
            } => match kind {
                AbsenceKind::Vacation => "border-warning border-dashed",
                AbsenceKind::OutOfOffice => "border-secondary border-dashed",
                AbsenceKind::Sick => "border-error border-dashed",
                AbsenceKind::Other => "border-base-300 border-dashed",
            },
            EntryKind::Absent { kind, .. } => match kind {
                AbsenceKind::Vacation => "bg-warning/15 border-warning border-dashed",
                AbsenceKind::OutOfOffice => "bg-secondary/15 border-secondary border-dashed",
                AbsenceKind::Sick => "bg-error/15 border-error border-dashed",
                AbsenceKind::Other => "bg-base-200/60 border-base-300 border-dashed",
            },
            EntryKind::Holiday => "bg-accent/15 border-accent border-dashed",
            EntryKind::Off => "bg-base-200/60 border-base-300 border-dashed",
        }
    }
}

/// The four parts of a day, in the order the roster lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntryGroup {
    /// Rostered to work.
    Working,
    /// Absent.
    Out,
    /// On holiday.
    Holiday,
    /// Off.
    Off,
}

/// One person on one day.
#[derive(Clone, Debug, PartialEq)]
pub struct RosterEntry {
    /// The host's id for this entry; `on_entry_activate` receives it.
    pub id: String,
    /// The person's name (rendered `translate="no"`).
    pub person: String,
    /// The second line: the host's shift label ("Early"). Without one the
    /// roster writes the times, "08:00-16:30", when it has them.
    pub detail: Option<String>,
    /// Start, minutes from midnight.
    pub start_min: Option<u16>,
    /// End, minutes from midnight (up to 1440).
    pub end_min: Option<u16>,
    /// What the entry records.
    pub kind: EntryKind,
    /// Where the chip links when the roster has no `on_entry_activate`.
    pub link: Option<String>,
    /// The host lets this entry be removed: with `on_entry_remove` the chip
    /// shows a labelled "x" (ldui-k145).
    pub removable: bool,
}

impl RosterEntry {
    fn new(id: impl Into<String>, person: impl Into<String>, kind: EntryKind) -> Self {
        Self {
            id: id.into(),
            person: person.into(),
            detail: None,
            start_min: None,
            end_min: None,
            kind,
            link: None,
            removable: false,
        }
    }

    /// A working shift (`ShiftState::Full` or `ShiftState::Half`).
    pub fn working(id: impl Into<String>, person: impl Into<String>, shift: ShiftState) -> Self {
        Self::new(id, person, EntryKind::Working(shift))
    }

    /// An approved absence.
    pub fn absent(id: impl Into<String>, person: impl Into<String>, kind: AbsenceKind) -> Self {
        Self::new(
            id,
            person,
            EntryKind::Absent {
                kind,
                pending: false,
                partial_hours: None,
            },
        )
    }

    /// The person's holiday.
    pub fn holiday(id: impl Into<String>, person: impl Into<String>) -> Self {
        Self::new(id, person, EntryKind::Holiday)
    }

    /// A scheduled day off.
    pub fn off(id: impl Into<String>, person: impl Into<String>) -> Self {
        Self::new(id, person, EntryKind::Off)
    }

    /// The shift label shown on the second line.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Start and end, minutes from midnight. `end` is raised to `start + 1`
    /// so a bar always has a width.
    pub fn times(mut self, start_min: u16, end_min: u16) -> Self {
        self.start_min = Some(start_min);
        self.end_min = Some(end_min.max(start_min + 1));
        self
    }

    /// Mark an absence as requested, not yet approved. No effect on any
    /// other kind.
    pub fn pending(mut self) -> Self {
        if let EntryKind::Absent { pending, .. } = &mut self.kind {
            *pending = true;
        }
        self
    }

    /// Out for only `hours` of the day. No effect on any other kind.
    pub fn partial_hours(mut self, hours: f32) -> Self {
        if let EntryKind::Absent { partial_hours, .. } = &mut self.kind {
            *partial_hours = Some(hours);
        }
        self
    }

    /// The chip's link when the roster has no `on_entry_activate`.
    pub fn link(mut self, href: impl Into<String>) -> Self {
        self.link = Some(href.into());
        self
    }

    /// Let this entry be removed (see the `removable` field).
    pub fn removable(mut self) -> Self {
        self.removable = true;
        self
    }

    /// Both times, when the entry has them.
    pub fn time_span(&self) -> Option<(u16, u16)> {
        self.start_min.zip(self.end_min)
    }

    /// A hash of the WHOLE entry: the `<For>` key of a chip, so a changed
    /// entry with the same id re-renders instead of keeping a stale node
    /// (the crate's recurring `<For>`-key lesson).
    pub fn content_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.person.hash(&mut hasher);
        self.detail.hash(&mut hasher);
        self.start_min.hash(&mut hasher);
        self.end_min.hash(&mut hasher);
        self.link.hash(&mut hasher);
        self.removable.hash(&mut hasher);
        match self.kind {
            EntryKind::Working(shift) => (0u8, shift).hash(&mut hasher),
            EntryKind::Absent {
                kind,
                pending,
                partial_hours,
            } => (1u8, kind, pending, partial_hours.map(f32::to_bits)).hash(&mut hasher),
            EntryKind::Holiday => 2u8.hash(&mut hasher),
            EntryKind::Off => 3u8.hash(&mut hasher),
        }
        hasher.finish()
    }
}

/// `(year, month 1..=12, day 1..=31)` of an epoch day.
pub fn civil(day: i64) -> (i64, u32, u32) {
    civil_from_days(day)
}

/// `2026-10-14`: the `data-day` value.
pub fn iso_date(day: i64) -> String {
    let (year, month, dom) = civil(day);
    format!("{year:04}-{month:02}-{dom:02}")
}

/// `HH:MM` of minutes from midnight. Unlike `minute_label` it does not wrap,
/// so a shift that ends at midnight reads "24:00".
pub fn clock(min: u16) -> String {
    format!("{:02}:{:02}", min / 60, min % 60)
}

/// The first epoch day of the month holding `day`.
pub fn first_of_month(day: i64) -> i64 {
    let (year, month, _) = civil(day);
    days_from_civil(year, month, 1)
}

/// The month `year`-`month` padded to WHOLE weeks starting on `week_start`:
/// `(first day shown, last day shown, rows)`. Rows are 4 to 6, never padded
/// to 6: February 2027 with a Monday week is exactly 4.
pub fn month_span(year: i64, month: u32, week_start: Weekday) -> (i64, i64, usize) {
    let first = days_from_civil(year, month, 1);
    let last = days_from_civil(year, month, days_in_month(year, month));
    let start = week_start_for_weekday(first, week_start);
    let rows = ((last - start) / 7 + 1) as usize;
    (start, start + rows as i64 * 7 - 1, rows)
}

/// The days the host must load for `zoom` at `anchor`, inclusive: the month
/// padded to whole weeks, the seven days of the anchor's week, or the anchor
/// alone.
pub fn visible_range(zoom: RosterZoom, anchor: i64, week_start: Weekday) -> (i64, i64) {
    match zoom {
        RosterZoom::Month => {
            let (year, month, _) = civil(anchor);
            let (first, last, _) = month_span(year, month, week_start);
            (first, last)
        }
        RosterZoom::Week => {
            let start = week_start_for_weekday(anchor, week_start);
            (start, start + 6)
        }
        RosterZoom::Day => (anchor, anchor),
    }
}

/// `day` moved by `delta` months, keeping the day of the month and
/// clamping it to the new month's length: Jan 31 + 1 month = Feb 28.
pub fn add_months(day: i64, delta: i64) -> i64 {
    let (year, month, dom) = civil(day);
    let index = year * 12 + i64::from(month) - 1 + delta;
    let (new_year, new_month) = (index.div_euclid(12), (index.rem_euclid(12) + 1) as u32);
    days_from_civil(
        new_year,
        new_month,
        dom.min(days_in_month(new_year, new_month)),
    )
}

/// `anchor` moved by `delta` of the zoom's unit: months (day of the month
/// kept, clamped), weeks or days.
pub fn step(zoom: RosterZoom, anchor: i64, delta: i64) -> i64 {
    match zoom {
        RosterZoom::Month => add_months(anchor, delta),
        RosterZoom::Week => anchor + 7 * delta,
        RosterZoom::Day => anchor + delta,
    }
}

/// The ISO 8601 week number of `day` (weeks start on Monday; week 1 holds
/// the year's first Thursday).
pub fn iso_week(day: i64) -> u32 {
    let monday_index = (day + 3).rem_euclid(7);
    let thursday = day - monday_index + 3;
    let (year, _, _) = civil(thursday);
    ((thursday - days_from_civil(year, 1, 1)) / 7 + 1) as u32
}

/// Sort a day's entries into the roster's order: working (by start time,
/// untimed last, then name), then out (Vacation, Out of office, Sick, Other,
/// then name), then holiday, then off (by name).
pub fn sort_entries(entries: &mut [RosterEntry]) {
    entries.sort_by(|a, b| {
        let (ka, kb) = (a.kind.normalized(), b.kind.normalized());
        ka.group()
            .cmp(&kb.group())
            .then_with(|| match (ka, kb) {
                (EntryKind::Working(_), EntryKind::Working(_)) => {
                    let start = |e: &RosterEntry| e.start_min.unwrap_or(u16::MAX);
                    start(a).cmp(&start(b))
                }
                (EntryKind::Absent { kind: x, .. }, EntryKind::Absent { kind: y, .. }) => {
                    absence_rank(x).cmp(&absence_rank(y))
                }
                _ => std::cmp::Ordering::Equal,
            })
            .then_with(|| a.person.cmp(&b.person))
            .then_with(|| a.id.cmp(&b.id))
    });
}

/// A sorted copy of `entries`.
pub fn sorted_entries(entries: &[RosterEntry]) -> Vec<RosterEntry> {
    let mut sorted = entries.to_vec();
    sort_entries(&mut sorted);
    sorted
}

fn absence_rank(kind: AbsenceKind) -> usize {
    AbsenceKind::ALL
        .iter()
        .position(|k| *k == kind)
        .unwrap_or(AbsenceKind::ALL.len())
}

/// How many of a day's entries are working, out, and off (holiday or off).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DayCounts {
    /// Working.
    pub working: usize,
    /// Absent.
    pub out: usize,
    /// On holiday or off.
    pub off: usize,
}

/// Count a day's entries by group.
pub fn day_counts(entries: &[RosterEntry]) -> DayCounts {
    entries
        .iter()
        .fold(DayCounts::default(), |mut counts, entry| {
            match entry.kind.group() {
                EntryGroup::Working => counts.working += 1,
                EntryGroup::Out => counts.out += 1,
                EntryGroup::Holiday | EntryGroup::Off => counts.off += 1,
            }
            counts
        })
}

/// A month date box's roving-focus move for `key`, or `None` when the key
/// is not a move or would leave the rendered span `(first, last)`:
/// ArrowLeft/Right one day, ArrowUp/Down one week, Home/End the ends of the
/// box's week row. The span always starts a row, so rows are `first + 7k`.
pub fn roving_target(key: &str, active: i64, first: i64, last: i64) -> Option<i64> {
    let row_start = first + (active - first).div_euclid(7) * 7;
    let target = match key {
        "ArrowLeft" => active - 1,
        "ArrowRight" => active + 1,
        "ArrowUp" => active - 7,
        "ArrowDown" => active + 7,
        "Home" => row_start,
        "End" => row_start + 6,
        _ => return None,
    };
    (first..=last).contains(&target).then_some(target)
}

/// The month grid's active (tab-stop) day: the remembered day when it was
/// set in the month now shown, else today when today is in that month, else
/// the anchor. `remembered` is `(day, first day of the month it was set in)`
/// so a month change (by the arrows or by the host) never leaves the tab
/// stop on an adjacent month's box.
pub fn active_day(remembered: Option<(i64, i64)>, anchor: i64, today: i64) -> i64 {
    let month = first_of_month(anchor);
    match remembered {
        Some((day, set_in)) if set_in == month => day,
        _ if first_of_month(today) == month => today,
        _ => anchor,
    }
}

/// The Day timeline's axis, minutes from midnight: 07:00-20:00 widened to
/// whole hours covering every timed working entry.
pub fn timeline_axis(entries: &[RosterEntry]) -> (u16, u16) {
    entries
        .iter()
        .filter(|e| e.kind.group() == EntryGroup::Working)
        .filter_map(RosterEntry::time_span)
        .fold((AXIS_START_MIN, AXIS_END_MIN), |(lo, hi), (start, end)| {
            (
                lo.min(start / 60 * 60),
                hi.max(end.div_ceil(60) * 60).min(24 * 60),
            )
        })
}

/// A bar's `(left, width)` in percent of the axis `(axis_start, axis_end)`.
pub fn bar_geometry(start: u16, end: u16, axis: (u16, u16)) -> (f64, f64) {
    let (lo, hi) = (f64::from(axis.0), f64::from(axis.1.max(axis.0 + 1)));
    let span = hi - lo;
    let clamp = |m: u16| f64::from(m).clamp(lo, hi);
    let left = (clamp(start) - lo) / span * 100.0;
    let width = (clamp(end) - clamp(start)) / span * 100.0;
    (left, width)
}

/// A day's absences grouped by kind, in [`AbsenceKind::ALL`] order, empty
/// kinds left out; each group sorted by name.
pub fn absence_groups(entries: &[RosterEntry]) -> Vec<(AbsenceKind, Vec<RosterEntry>)> {
    let sorted = sorted_entries(entries);
    AbsenceKind::ALL
        .iter()
        .filter_map(|&kind| {
            let people: Vec<RosterEntry> = sorted
                .iter()
                .filter(|e| {
                    matches!(e.kind.normalized(), EntryKind::Absent { kind: k, .. } if k == kind)
                })
                .cloned()
                .collect();
            (!people.is_empty()).then_some((kind, people))
        })
        .collect()
}
