//! `MonthRoster`'s variants live in `ldui_design::month_roster` (so a
//! non-Leptos renderer shares them); re-exported here. The class tables below
//! are the pattern's layout classes, written as literals so Tailwind sees
//! every one.

pub use ldui_design::components::month_roster::*;

/// One element's classes in each layout: the auto layout (a container query
/// on the roster's root), the grid forced, and the agenda forced. Every
/// string is a complete literal, never assembled, because Tailwind only
/// generates classes it can read in the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutClasses {
    /// No `layout` from the host: below `@2xl` (42rem) of the ROOT's width.
    pub auto: &'static str,
    /// `RosterLayout::Grid` forced.
    pub grid: &'static str,
    /// `RosterLayout::Agenda` forced.
    pub agenda: &'static str,
}

impl LayoutClasses {
    /// The classes for the host's `layout` choice.
    pub fn pick(self, layout: Option<RosterLayout>) -> &'static str {
        match layout {
            None => self.auto,
            Some(RosterLayout::Grid) => self.grid,
            Some(RosterLayout::Agenda) => self.agenda,
        }
    }
}

/// The month `<table>`.
pub const TABLE: LayoutClasses = LayoutClasses {
    auto: "@max-2xl:block",
    grid: "",
    agenda: "block",
};

/// The weekday header row and the week-button column: gone in the agenda.
pub const GRID_ONLY: LayoutClasses = LayoutClasses {
    auto: "@max-2xl:hidden",
    grid: "",
    agenda: "hidden",
};

/// `<tbody>` and `<tr>`: blocks in the agenda.
pub const BLOCK: LayoutClasses = LayoutClasses {
    auto: "@max-2xl:block",
    grid: "",
    agenda: "block",
};

/// A date box: a full-width block in the agenda, with no fixed height.
pub const DAY_BOX_LAYOUT: LayoutClasses = LayoutClasses {
    auto: "@max-2xl:block @max-2xl:h-auto @max-2xl:border-x-0",
    grid: "",
    agenda: "block h-auto border-x-0",
};

/// A day of an adjacent month: left out of the agenda.
pub const OUTSIDE_LAYOUT: LayoutClasses = GRID_ONLY;

/// A date box's full-date heading: for screen readers in the grid (the
/// column header and the number say it), visible in the agenda.
pub const DATE_HEADING: LayoutClasses = LayoutClasses {
    auto: "sr-only @max-2xl:not-sr-only",
    grid: "sr-only",
    agenda: "",
};

/// The Week view's columns: seven, or one stacked list.
pub const WEEK_COLUMNS: LayoutClasses = LayoutClasses {
    auto: "grid-cols-7 @max-2xl:grid-cols-1",
    grid: "grid-cols-7",
    agenda: "grid-cols-1",
};

/// The Day timeline's name column: narrower in the compressed layout.
pub const TIMELINE_ROW: LayoutClasses = LayoutClasses {
    auto: "grid-cols-[10rem_1fr] @max-2xl:grid-cols-[6rem_1fr]",
    grid: "grid-cols-[10rem_1fr]",
    agenda: "grid-cols-[6rem_1fr]",
};

/// The Day axis's odd-hour labels: hidden when the axis is compressed.
pub const ODD_HOUR: LayoutClasses = GRID_ONLY;

/// The roster's root: a container, so its own width (not the window's)
/// picks the layout.
pub const ROOT: &str = "@container flex w-full flex-col gap-3";

/// The month `<table>`'s grid look (the kit's).
pub const TABLE_BASE: &str = "table w-full table-fixed border-collapse border border-table-grid";

/// A weekday header cell.
pub const WEEKDAY_HEAD: &str = "border border-table-grid bg-table-header px-1 py-1 text-center text-xs font-semibold text-table-header-content";

/// The week-button column's cells. They paint the header fill, so they
/// carry the header's content colour too, as [`WEEKDAY_HEAD`] does.
pub const WEEK_HEAD: &str = "w-10 border border-table-grid bg-table-header p-1 text-center align-top text-table-header-content";

/// The week button on the header fill. It carries the header's content
/// colour ITSELF and uses no daisyUI `btn` class (ldui-u0yr). A `btn-ghost`
/// takes its rest colour from daisyUI, and daisyUI builds disagree: ours
/// (5.5.19) draws the ghost in `currentColor`, while the build 4iiz-Office
/// serves draws it in `--color-base-content`. That put base-content on the
/// navy header at 1.06:1 again after ldui-on0i colour-coded only the cell.
/// The hover is a translucent white, so the light text never sits on a
/// light fill, as it would on the ghost's own hover.
pub const WEEK_BUTTON: &str = "ld-eased ld-pressable ld-focus-ring cursor-pointer rounded-sm px-1 py-1 text-xs font-semibold text-table-header-content hover:bg-white/15 hover:underline";

/// A date box.
pub const DAY_BOX: &str = "h-28 border border-table-grid p-1 align-top print:h-auto";

/// A day of an adjacent month. Dimmed to `/85`, not the crate's usual muted
/// `/75`: a chip's 12px text sits on the chip's tint over this surface, and
/// `/75` measured 4.4:1 there (ldui-on0i, Office's axe).
pub const DAY_OUTSIDE: &str = "bg-base-200/40 text-base-content/85";

/// A closed day's surface.
pub const DAY_CLOSED: &str = "bg-base-200/60";

/// A holiday's surface.
pub const DAY_HOLIDAY: &str = "bg-accent/10";

/// A weekend day's surface.
pub const DAY_WEEKEND: &str = "bg-base-200/30";

/// Today: a ring, a mark that is not only colour (the word "Today" is the
/// other).
pub const DAY_TODAY: &str = "outline-2 -outline-offset-2 outline-primary";

/// A chip: a tinted tile with the state's accent bar.
pub const CHIP: &str = "rounded-sm border-l-4 px-1 py-1 text-xs leading-tight";
