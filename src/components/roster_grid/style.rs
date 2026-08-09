/// # Shift State
///
/// What one worker is doing on one day of a [`RosterGrid`](super::RosterGrid):
/// a full shift, a half shift, a scheduled day off, a public holiday, or
/// booked leave. Merges the two filings behind `ldui-m9s` (working/off/holiday
/// and full/half/off/leave) into one five-variant vocabulary.
///
/// The class mapping follows [`SchedulerEventColor`](crate::components::SchedulerEventColor):
/// a soft tinted background plus a solid left accent bar in the same hue. Two
/// deliberate additions:
///
/// - **Colour is never the only channel.** A cell always renders its own
///   `label` text, and [`as_label`](Self::as_label) supplies the state's name
///   to assistive tech. On top of that,
///   [`as_border_class`](Self::as_border_class) gives the non-working states a
///   *dashed* accent bar and the working states a *solid* one, so the
///   working/not-working split survives greyscale, low vision and every form
///   of colour blindness without reading any text.
/// - **The names are English defaults, not a hardcoding.** `RosterGrid` takes
///   a `state_label` callback that overrides [`as_label`](Self::as_label)
///   wholesale, the same escape hatch `hour_label` gives
///   [`WeekView`](crate::components::WeekView).
///
/// **This enum is deliberately NOT `#[non_exhaustive]`, unlike
/// [`RosterDensity`].** It is a value consumers must map exhaustively — a
/// `state_label` callback returns one string per state — so when a sixth state
/// lands, a compile error in every consumer is the correct outcome.
/// `#[non_exhaustive]` would instead route the new state into an existing
/// `_ =>` arm and announce the wrong name to a screen reader, silently turning
/// a loud build failure into an accessibility regression. Do not add it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShiftState {
    /// A full working shift.
    Full,

    /// A half (partial) working shift.
    Half,

    /// Not scheduled — the default, so an unspecified cell reads as "off"
    /// rather than inventing work that was never rostered.
    #[default]
    Off,

    /// A public/company holiday: nobody is expected in.
    Holiday,

    /// Booked leave for this worker specifically (annual, sick, parental).
    Leave,
}

impl ShiftState {
    /// Soft tinted background CSS class for the shift tile.
    pub fn as_class(&self) -> &'static str {
        match self {
            ShiftState::Full => "bg-success/15",
            ShiftState::Half => "bg-info/15",
            ShiftState::Off => "bg-base-200/60",
            ShiftState::Holiday => "bg-accent/15",
            ShiftState::Leave => "bg-warning/15",
        }
    }

    /// Left accent-bar CSS class for the shift tile: the hue plus the
    /// border *style*. Working states get a solid bar, non-working states a
    /// dashed one — the redundant, non-colour encoding of
    /// [`is_working`](Self::is_working).
    pub fn as_border_class(&self) -> &'static str {
        match self {
            ShiftState::Full => "border-success border-solid",
            ShiftState::Half => "border-info border-solid",
            ShiftState::Off => "border-base-300 border-dashed",
            ShiftState::Holiday => "border-accent border-dashed",
            ShiftState::Leave => "border-warning border-dashed",
        }
    }

    /// Default English name for the state, used as the visually-hidden text
    /// that carries the state to a screen reader. Override per-app with
    /// `RosterGrid`'s `state_label` callback.
    pub fn as_label(&self) -> &'static str {
        match self {
            ShiftState::Full => "Full shift",
            ShiftState::Half => "Half shift",
            ShiftState::Off => "Off",
            ShiftState::Holiday => "Holiday",
            ShiftState::Leave => "Leave",
        }
    }

    /// Whether the worker is rostered to work at all in this cell. Drives the
    /// solid-vs-dashed accent bar in [`as_border_class`](Self::as_border_class),
    /// and is the question a coverage summary actually asks.
    pub fn is_working(&self) -> bool {
        matches!(self, ShiftState::Full | ShiftState::Half)
    }

    /// Every variant, in display order — for demos, legends and exhaustive
    /// tests that must fail when a variant is added without a class mapping.
    ///
    /// A slice rather than `[ShiftState; 5]`: baking the count into the public
    /// type makes adding a sixth state a breaking API change on top of the
    /// (intended) breaking change to the enum itself, and forces every caller
    /// that named the array type to be edited for no benefit.
    pub const ALL: &'static [ShiftState] = &[
        ShiftState::Full,
        ShiftState::Half,
        ShiftState::Off,
        ShiftState::Holiday,
        ShiftState::Leave,
    ];
}

/// # Roster Density
///
/// How tall a roster row is. This is a **size** ramp, not spacing: the two
/// steps are 32px and 40px, and 40 is deliberately off the nine-step spacing
/// scale because it is `ui_tokens::spacing::TABLE_ROW_HEIGHT` — the row height
/// the Direct2D desktop face draws. Padding inside a cell still lands on the
/// canonical scale; only the height comes from here.
///
/// `#[non_exhaustive]`: a third step (an ultra-dense wall-display roster, say)
/// should not be a breaking change. Unlike [`ShiftState`] this is a *knob* the
/// consumer picks from, not a value it must map, so a `_ =>` fallback in a
/// consumer's `match` is harmless.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RosterDensity {
    /// 32px rows with extra-small text — a full department on one screen.
    Compact,

    /// 40px rows with small text (default), matching the shared
    /// `TABLE_ROW_HEIGHT` token the desktop face uses.
    #[default]
    Comfortable,
}

impl RosterDensity {
    /// daisyUI table size class, which sets the header/row-header padding.
    pub fn as_table_class(&self) -> &'static str {
        match self {
            RosterDensity::Compact => "table-sm",
            RosterDensity::Comfortable => "table-md",
        }
    }

    /// Height and text-size classes for the shift tile inside each cell.
    ///
    /// Horizontal padding is intentionally NOT part of this: the tile pads
    /// `px-2` (8px) at both densities and the cell pads `p-1` (4px), so the
    /// gap *between* two tiles is 8px and the internal-must-not-exceed-external
    /// rule holds at either density. Growing the padding with the density
    /// would break it at the comfortable step.
    pub fn as_cell_class(&self) -> &'static str {
        match self {
            RosterDensity::Compact => "h-8 text-xs",
            RosterDensity::Comfortable => "h-10 text-sm",
        }
    }

    /// Row height in pixels — the numeric form of
    /// [`as_cell_class`](Self::as_cell_class)'s `h-*` step.
    pub fn row_height_px(&self) -> u32 {
        match self {
            RosterDensity::Compact => 32,
            RosterDensity::Comfortable => 40,
        }
    }
}
