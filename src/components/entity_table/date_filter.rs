//! Framework-owned civil-date filtering for [`EntityTable`](super::EntityTable)
//! (`ldui-lx5t`).
//!
//! The rendered control lives with the other opinionated filters in
//! [`EntityColumnFilter::date`](super::EntityColumnFilter::date); this module
//! owns the value side -- what a date filter actually compares, how a caller
//! supplies comparable values, and what happens at every edge a consumer would
//! otherwise discover by experiment.
//!
//! ## Why a type instead of a string comparison
//!
//! A table cell is display copy. Comparing it means comparing whatever the
//! current locale, format callback or column renderer decided to print, so a
//! filter built on rendered text silently changes meaning when the copy does.
//! Every value on this path is an [`EntityDate`] -- a calendar day with no
//! timezone, no clock and no formatting -- and the caller maps a row to one
//! with an ordinary accessor:
//!
//! ```rust,ignore
//! let cutoff = EntityDateFilter::parse_on_or_before(&cutoff_text.get());
//! let visible: Vec<Matter> = matters
//!     .iter()
//!     .filter(|matter| cutoff.matches(matter.arrived_on))
//!     .cloned()
//!     .collect();
//! ```
//!
//! `matter.arrived_on` is an `Option<EntityDate>` the consumer produced from
//! its own storage under its own timezone policy. LDUI never guesses a zone,
//! never reads the browser clock, and never parses a cell.
//!
//! ## The three states that are easy to get silently wrong
//!
//! - **Unbounded.** Both ends open. Every row passes, *including* rows with no
//!   date at all. This is "the user has not filtered", not "match nothing".
//! - **Half-open.** One end bounded. Only that end is compared, and a row with
//!   no date does NOT pass -- a record with no arrival date cannot satisfy
//!   "arrived on or before 4 August".
//! - **Unparseable.** The caller-owned text is not a real `YYYY-MM-DD` day.
//!   Nothing matches, [`EntityDateFilter::status`] reports
//!   [`EntityDateFilterStatus::Invalid`], and the offending text is retrievable
//!   through [`EntityDateFilter::invalid_input`] so the consumer can say so.
//!   The rendered control marks itself `aria-invalid` and carries a localized
//!   description. A silent no-match is the one outcome this module refuses.

use std::fmt;

/// A timezone-free civil date -- the only value an EntityTable date filter
/// compares.
///
/// Deliberately not an instant. "Arrived on or before 4 August" is a claim
/// about the calendar the user is reading, not about a point on the UTC
/// timeline, so collapsing a timestamp to a calendar day is the consumer's
/// job and belongs wherever that consumer's timezone policy already lives.
///
/// Ordering is calendar ordering: the derived [`Ord`] compares year, then
/// month, then day, and every constructor rejects a value outside the
/// year 1 through 9999 range that HTML's own `date` control accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityDate {
    year: i32,
    month: u8,
    day: u8,
}

impl EntityDate {
    /// Creates one real calendar day, or `None` for anything that is not one.
    ///
    /// Rejects a year outside 1 through 9999, a month outside 1 through 12,
    /// and a day outside the month's own length -- 29 February is accepted in
    /// a leap year and rejected otherwise.
    pub fn from_ymd(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
            return None;
        }
        if !(1..=days_in_month(year, month)).contains(&day) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// Parses one strict ISO 8601 calendar date, exactly `YYYY-MM-DD`.
    ///
    /// Surrounding whitespace is trimmed first, because a value restored from
    /// a URL query or a saved view often carries it. Nothing else is
    /// tolerated: no two-digit years, no `/` separators, no time part, no
    /// expanded-year sign. Empty text is [`EntityDateParseError::Empty`] and
    /// is the caller's "no constraint", never a parse failure to report.
    pub fn parse(raw: &str) -> Result<Self, EntityDateParseError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(EntityDateParseError::Empty);
        }
        if !trimmed.is_ascii() || trimmed.len() != 10 {
            return Err(EntityDateParseError::Malformed);
        }
        let bytes = trimmed.as_bytes();
        if bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(EntityDateParseError::Malformed);
        }
        let year = parse_digits(&trimmed[0..4]).ok_or(EntityDateParseError::Malformed)?;
        let month = parse_digits(&trimmed[5..7]).ok_or(EntityDateParseError::Malformed)?;
        let day = parse_digits(&trimmed[8..10]).ok_or(EntityDateParseError::Malformed)?;
        let month = u8::try_from(month).map_err(|_| EntityDateParseError::OutOfRange)?;
        let day = u8::try_from(day).map_err(|_| EntityDateParseError::OutOfRange)?;
        Self::from_ymd(year, month, day).ok_or(EntityDateParseError::OutOfRange)
    }

    /// The calendar year.
    pub const fn year(self) -> i32 {
        self.year
    }

    /// The calendar month, 1 through 12.
    pub const fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, 1 through the month's length.
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Renders the date back to the `YYYY-MM-DD` text a control round-trips.
    ///
    /// This is a machine format, not display copy: it is what the native
    /// `date` input and a URL query carry. Anything a person reads should be
    /// formatted by the consumer's own localization.
    pub fn to_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl fmt::Display for EntityDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

fn parse_digits(text: &str) -> Option<i32> {
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Why one piece of text is not an [`EntityDate`].
///
/// [`Self::Empty`] is separated from the two real failures precisely so an
/// absent filter cannot be mistaken for a broken one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityDateParseError {
    /// The text was empty or whitespace only -- no constraint was expressed.
    Empty,
    /// The text was not shaped like `YYYY-MM-DD` with ASCII digits.
    Malformed,
    /// The text was shaped correctly but names no real day, such as
    /// `2026-02-30`, `2026-13-01`, or a year outside 1 through 9999.
    OutOfRange,
}

impl fmt::Display for EntityDateParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Empty => "no date was supplied",
            Self::Malformed => "expected an ISO 8601 date of the form YYYY-MM-DD",
            Self::OutOfRange => "the value is not a real calendar date",
        };
        formatter.write_str(text)
    }
}

impl std::error::Error for EntityDateParseError {}

/// One end of an [`EntityDateFilter`].
///
/// A bounded end is **inclusive**, which is why the variant is named
/// [`Self::Inclusive`]: end-point inclusivity is the single most commonly
/// re-derived-by-experiment property of a date range, so it is stated in the
/// type rather than left to a consumer's off-by-one day.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum EntityDateBound {
    /// No constraint on this end.
    #[default]
    Open,
    /// Constrained inclusively at this calendar day.
    Inclusive(EntityDate),
    /// The caller-owned text for this end named no real day. The offending
    /// text is retained so a consumer can show it back to the user.
    Invalid(String),
}

impl EntityDateBound {
    /// Interprets one caller-owned raw control value.
    ///
    /// Empty or whitespace-only text becomes [`Self::Open`]. A real
    /// `YYYY-MM-DD` day becomes [`Self::Inclusive`]. Anything else becomes
    /// [`Self::Invalid`] carrying the trimmed offending text -- never
    /// [`Self::Open`], because degrading an unreadable constraint to "no
    /// constraint" would silently widen the result set.
    pub fn parse(raw: &str) -> Self {
        match EntityDate::parse(raw) {
            Ok(date) => Self::Inclusive(date),
            Err(EntityDateParseError::Empty) => Self::Open,
            Err(_) => Self::Invalid(raw.trim().to_owned()),
        }
    }

    /// The bounded day, or `None` when this end is open or invalid.
    pub fn date(&self) -> Option<EntityDate> {
        match self {
            Self::Inclusive(date) => Some(*date),
            _ => None,
        }
    }

    /// Whether this end places no constraint at all.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Whether this end's raw text could not be read as a calendar day.
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }

    /// The offending raw text when this end is invalid.
    pub fn invalid_input(&self) -> Option<&str> {
        match self {
            Self::Invalid(raw) => Some(raw.as_str()),
            _ => None,
        }
    }
}

/// What an [`EntityDateFilter`] currently does to a result set.
///
/// Every variant is reachable from ordinary user input, and each one is named
/// so that "nothing matched" can always be explained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityDateFilterStatus {
    /// Both ends open: every row passes, including rows with no date.
    Unconstrained,
    /// At least one end bounded and the range is satisfiable.
    Constrained,
    /// Both ends bounded with the start after the end. No day can satisfy it,
    /// so nothing matches -- deliberately, and reportably.
    Impossible,
    /// At least one end's text named no real day. Nothing matches, and
    /// [`EntityDateFilter::invalid_input`] carries the text to show the user.
    Invalid,
}

/// An inclusive civil-date range predicate over caller-supplied values.
///
/// Both ends are inclusive when bounded, either end may be open, and an
/// unbounded filter is the identity. Construct one from parsed control text
/// with [`Self::parse_bounds`] (or the single-ended
/// [`Self::parse_on_or_before`] / [`Self::parse_on_or_after`]), then ask it
/// about each row's own [`EntityDate`] through [`Self::matches`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityDateFilter {
    start: EntityDateBound,
    end: EntityDateBound,
}

impl EntityDateFilter {
    /// Creates a filter from two already-interpreted ends.
    pub fn new(start: EntityDateBound, end: EntityDateBound) -> Self {
        Self { start, end }
    }

    /// The identity filter: both ends open, every row passes.
    pub fn unbounded() -> Self {
        Self::default()
    }

    /// Matches exactly one calendar day.
    pub fn on(date: EntityDate) -> Self {
        Self::new(
            EntityDateBound::Inclusive(date),
            EntityDateBound::Inclusive(date),
        )
    }

    /// Matches days up to and **including** `date`.
    pub fn on_or_before(date: EntityDate) -> Self {
        Self::new(EntityDateBound::Open, EntityDateBound::Inclusive(date))
    }

    /// Matches days from `date` onward, **including** `date`.
    pub fn on_or_after(date: EntityDate) -> Self {
        Self::new(EntityDateBound::Inclusive(date), EntityDateBound::Open)
    }

    /// Matches days between `start` and `end`, **including both**.
    pub fn between(start: EntityDate, end: EntityDate) -> Self {
        Self::new(
            EntityDateBound::Inclusive(start),
            EntityDateBound::Inclusive(end),
        )
    }

    /// Interprets two caller-owned raw control values, either of which may be
    /// empty for an open end.
    pub fn parse_bounds(start_raw: &str, end_raw: &str) -> Self {
        Self::new(
            EntityDateBound::parse(start_raw),
            EntityDateBound::parse(end_raw),
        )
    }

    /// Interprets one raw control value as an inclusive upper bound -- the
    /// shape a single [`EntityColumnFilter::date`](super::EntityColumnFilter::date)
    /// cutoff control drives. Empty text leaves the filter unbounded.
    pub fn parse_on_or_before(raw: &str) -> Self {
        Self::new(EntityDateBound::Open, EntityDateBound::parse(raw))
    }

    /// Interprets one raw control value as an inclusive lower bound. Empty
    /// text leaves the filter unbounded.
    pub fn parse_on_or_after(raw: &str) -> Self {
        Self::new(EntityDateBound::parse(raw), EntityDateBound::Open)
    }

    /// The filter's lower end.
    pub fn start(&self) -> &EntityDateBound {
        &self.start
    }

    /// The filter's upper end.
    pub fn end(&self) -> &EntityDateBound {
        &self.end
    }

    /// What this filter currently does to a result set.
    ///
    /// [`EntityDateFilterStatus::Invalid`] outranks
    /// [`EntityDateFilterStatus::Impossible`]: unreadable text is the more
    /// actionable message, and an invalid end cannot be ordered against the
    /// other one anyway.
    pub fn status(&self) -> EntityDateFilterStatus {
        if self.start.is_invalid() || self.end.is_invalid() {
            return EntityDateFilterStatus::Invalid;
        }
        match (self.start.date(), self.end.date()) {
            (None, None) => EntityDateFilterStatus::Unconstrained,
            (Some(start), Some(end)) if start > end => EntityDateFilterStatus::Impossible,
            _ => EntityDateFilterStatus::Constrained,
        }
    }

    /// Whether this filter can exclude any row -- the "active filter" signal a
    /// responsive panel or summary needs.
    ///
    /// An invalid or impossible filter counts as active: it is excluding
    /// everything, which the user must be able to see and clear.
    pub fn constrains(&self) -> bool {
        self.status() != EntityDateFilterStatus::Unconstrained
    }

    /// The offending raw text of the first unreadable end, if any.
    pub fn invalid_input(&self) -> Option<&str> {
        self.start
            .invalid_input()
            .or_else(|| self.end.invalid_input())
    }

    /// Whether one row's calendar day satisfies this filter.
    ///
    /// - An unbounded filter accepts everything, `None` included: the user has
    ///   not filtered, so no row may be hidden.
    /// - A bounded filter rejects `None`: a row with no date cannot satisfy a
    ///   date constraint, and quietly keeping it would misreport the result.
    /// - An invalid or impossible filter rejects everything, and
    ///   [`Self::status`] says which so the emptiness can be explained.
    /// - Both bounded ends are inclusive.
    pub fn matches(&self, value: Option<EntityDate>) -> bool {
        match self.status() {
            EntityDateFilterStatus::Unconstrained => true,
            EntityDateFilterStatus::Invalid | EntityDateFilterStatus::Impossible => false,
            EntityDateFilterStatus::Constrained => value.is_some_and(|date| {
                self.start.date().is_none_or(|start| date >= start)
                    && self.end.date().is_none_or(|end| date <= end)
            }),
        }
    }
}

/// What the user did to produce an [`EntityDateFilterProposal`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityDateFilterCause {
    /// The date control's own value changed -- picked, typed, or cleared
    /// through the browser's native control.
    Edited,
    /// The responsive filter panel's clear action ran.
    Cleared,
}

/// One atomic proposed replacement for a caller-owned date filter value.
///
/// Same contract as [`EntityTableSelectionProposal`](super::EntityTableSelectionProposal)
/// (`ldui-nz6d`): `raw` is the COMPLETE resulting value, never a delta, and the
/// component never applies it optimistically. The caller owns accepted truth;
/// a proposal it ignores leaves the control showing the accepted value.
///
/// `column_id` and `control_id` are the scope stamp. A caller routing several
/// date filters through one callback matches on them instead of inferring the
/// source from call order, and a caller whose columns changed underneath the
/// gesture can reject a proposal stamped with a column it no longer renders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityDateFilterProposal {
    /// The complete proposed control value. Empty means "no constraint".
    pub raw: String,
    /// `raw` already interpreted, so a caller storing a parsed filter never
    /// re-parses and cannot disagree with the control about validity.
    pub bound: EntityDateBound,
    /// The gesture that produced this proposal.
    pub cause: EntityDateFilterCause,
    /// Stable identity of the column the filter targets.
    pub column_id: String,
    /// Base control identity supplied when the filter was declared. This is
    /// the caller's own ID, not the placement-suffixed DOM ID, so header and
    /// responsive copies of one filter propose under the same identity.
    pub control_id: String,
}

impl EntityDateFilterProposal {
    /// Creates one proposal, interpreting `raw` once.
    pub fn new(
        raw: impl Into<String>,
        cause: EntityDateFilterCause,
        column_id: impl Into<String>,
        control_id: impl Into<String>,
    ) -> Self {
        let raw = raw.into();
        let bound = EntityDateBound::parse(&raw);
        Self {
            raw,
            bound,
            cause,
            column_id: column_id.into(),
            control_id: control_id.into(),
        }
    }

    /// The proposed calendar day, or `None` when the proposal clears the
    /// filter or carries unreadable text.
    pub fn date(&self) -> Option<EntityDate> {
        self.bound.date()
    }
}
