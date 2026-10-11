//! Variants of the `MonthRoster` pattern (ldui-5lmt): the zoom level, the
//! kind of absence an entry records, and the layout a host can force.
//!
//! The pattern itself lives in `leptos_daisyui_rs::patterns::MonthRoster`;
//! these enums are here so a non-Leptos renderer (4iiz-kit) can speak the
//! same vocabulary and paint the same classes.

/// # Roster Zoom
///
/// The three zoom levels of one `MonthRoster` data set. The roster is
/// controlled: it never changes its own zoom, it proposes a change through
/// `on_navigate` and the host answers by setting the zoom signal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RosterZoom {
    /// The month grid: whole weeks of date boxes, four people per box and
    /// "+N" for the rest.
    #[default]
    Month,
    /// Seven day columns, each listing every entry.
    Week,
    /// One day: a shift coverage timeline and who is out.
    Day,
}

impl RosterZoom {
    /// Every zoom level, widest first: the order of the segmented switch.
    pub const ALL: [RosterZoom; 3] = [RosterZoom::Month, RosterZoom::Week, RosterZoom::Day];

    /// The `data-zoom` slug (`"month"`, `"week"`, `"day"`).
    pub fn as_str(self) -> &'static str {
        match self {
            RosterZoom::Month => "month",
            RosterZoom::Week => "week",
            RosterZoom::Day => "day",
        }
    }
}

/// # Absence Kind
///
/// Why a person is out on a day. Each kind has its own hue and a DASHED
/// accent bar (not working), and every chip also says the kind in words, so
/// colour is never the only channel.
///
/// Deliberately NOT `#[non_exhaustive]`, for the reason `ShiftState` gives:
/// a host maps every kind to its own words, and a new kind should be a
/// compile error there rather than a silent wrong name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AbsenceKind {
    /// Booked vacation (annual leave).
    Vacation,
    /// Working, but away from the office (a client site, a conference).
    OutOfOffice,
    /// Sick leave.
    Sick,
    /// Any other absence (jury duty, bereavement, parental leave).
    Other,
}

impl AbsenceKind {
    /// Every kind, in the order a day lists them.
    pub const ALL: [AbsenceKind; 4] = [
        AbsenceKind::Vacation,
        AbsenceKind::OutOfOffice,
        AbsenceKind::Sick,
        AbsenceKind::Other,
    ];

    /// The `data-entry-state` / `data-out-group` slug.
    pub fn as_str(self) -> &'static str {
        match self {
            AbsenceKind::Vacation => "vacation",
            AbsenceKind::OutOfOffice => "out-of-office",
            AbsenceKind::Sick => "sick",
            AbsenceKind::Other => "other",
        }
    }

    /// The soft tint of an APPROVED absence's chip. A pending (requested)
    /// absence has no fill; see [`as_border_class`](Self::as_border_class).
    pub fn as_class(self) -> &'static str {
        match self {
            AbsenceKind::Vacation => "bg-warning/15",
            AbsenceKind::OutOfOffice => "bg-secondary/15",
            AbsenceKind::Sick => "bg-error/15",
            AbsenceKind::Other => "bg-base-200/60",
        }
    }

    /// The accent bar: the kind's hue, always dashed (not working).
    pub fn as_border_class(self) -> &'static str {
        match self {
            AbsenceKind::Vacation => "border-warning border-dashed",
            AbsenceKind::OutOfOffice => "border-secondary border-dashed",
            AbsenceKind::Sick => "border-error border-dashed",
            AbsenceKind::Other => "border-base-300 border-dashed",
        }
    }
}

/// # Roster Layout
///
/// The layout a host can force on a `MonthRoster`. Without one the roster
/// follows its own container's width: the grid from 42rem up, the agenda
/// below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RosterLayout {
    /// The month as a grid of whole weeks, the week as seven columns.
    Grid,
    /// The month as a list of its days, the week stacked.
    Agenda,
}

impl RosterLayout {
    /// The `data-layout` slug.
    pub fn as_str(self) -> &'static str {
        match self {
            RosterLayout::Grid => "grid",
            RosterLayout::Agenda => "agenda",
        }
    }
}
