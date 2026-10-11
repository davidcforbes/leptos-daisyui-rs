//! `MonthRoster` (ldui-5lmt): an opinionated staff month calendar with Week
//! and Day zoom, ported from the 4iiz-kit `MonthRoster`. Design:
//! `docs/superpowers/specs/2026-10-11-month-roster-design.md`; guide:
//! `doc/components/month-roster.md`.

mod component;
mod day;
mod style;
mod texts;
mod types;
mod week;

pub use component::MonthRoster;
pub use style::{AbsenceKind, RosterLayout, RosterZoom};
pub use texts::MonthRosterTexts;
pub use types::{
    DayCounts, DayMark, EntryGroup, EntryKind, RosterDay, RosterEntry, Staffing, add_months,
    day_counts, iso_week, month_span, sort_entries, step, visible_range,
};

#[cfg(test)]
mod tests;
