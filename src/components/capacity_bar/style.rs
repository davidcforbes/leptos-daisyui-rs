/// Default headroom factor applied to `cap` to compute the scale maximum
/// (right edge of the track) when no explicit `max` override is given.
/// Ported from d2d-ui's `CapacityBar::DEFAULT_MAX_FACTOR`.
pub const CAPACITY_BAR_DEFAULT_MAX_FACTOR: f64 = 1.25;

/// Semantic color used for CapacityBar's under-cap fill and over-cap
/// overflow band. The same variant set drives both `color` (fill) and
/// `over_color` (overflow band) props, which default independently
/// (`Primary`/`Error`) -- mirroring d2d-ui's `CapacityBar`, which stored
/// separate `fill_color`/`over_color` fields rather than deriving one from
/// the other.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CapacityBarColor {
    /// Neutral color
    Neutral,

    /// Primary theme color (default fill color)
    #[default]
    Primary,

    /// Secondary theme color
    Secondary,

    /// Accent theme color
    Accent,

    /// Info color
    Info,

    /// Success color
    Success,

    /// Warning color
    Warning,

    /// Error color (default overflow-band color)
    Error,

    /// The series ENDED — not "very over capacity".
    ///
    /// ⚠️ A DIFFERENT KIND OF FACT FROM THE OTHER VARIANTS, WHICH IS WHY IT IS A VARIANT.
    /// Every colour above answers "how much"; this answers "is it still running at all". A
    /// stopped feed drawn as a longer `Error` bar sorts and reads as the worst case of a live
    /// series, and that is exactly how a dead mirror hides among slow ones — it cost a year of
    /// silence on one real feed before anyone noticed.
    ///
    /// Rendered as a diagonal hatch rather than a fill: a solid block asserts a magnitude, and
    /// the magnitude is not the point once the series has ended.
    Stopped,
}

impl CapacityBarColor {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            CapacityBarColor::Neutral => "bg-neutral",
            CapacityBarColor::Primary => "bg-primary",
            CapacityBarColor::Secondary => "bg-secondary",
            CapacityBarColor::Accent => "bg-accent",
            CapacityBarColor::Info => "bg-info",
            CapacityBarColor::Success => "bg-success",
            CapacityBarColor::Warning => "bg-warning",
            CapacityBarColor::Error => "bg-error",
            // ⚠️ A PATTERN, NOT A COLOUR. `bg-error` with a repeating-linear-gradient overlay,
            // so it still reads as severe at a glance while being unmistakably not a plain
            // fill. The same idiom day_scheduler and week_view already use for a non-bookable
            // band — promoted here rather than invented.
            CapacityBarColor::Stopped => {
                "bg-error/30 bg-[repeating-linear-gradient(45deg,transparent_0_3px,rgba(0,0,0,0.28)_3px_6px)] border border-error/55"
            }
        }
    }

    /// Whether this tone describes a series that has ENDED rather than one that is merely
    /// loaded.
    ///
    /// ⚠️ CALLERS SORT ON THIS. A stopped series belongs at the top of a list regardless of its
    /// value, because "ended" outranks "large" — sorting by magnitude is what buries it.
    pub fn is_stopped(&self) -> bool {
        matches!(self, CapacityBarColor::Stopped)
    }
}

/// Scale maximum (right edge of the track) when no explicit `max` override
/// is supplied: `cap * 1.25`, clamped to be at least `cap` and `value` so the
/// fill and cap-line positions are always within `[0, 100]%` even when
/// `value` blows well past the default headroom. Mirrors d2d-ui's
/// `CapacityBar::new` default-max computation.
pub fn capacity_bar_default_max(cap: f64, value: f64) -> f64 {
    (cap * CAPACITY_BAR_DEFAULT_MAX_FACTOR).max(cap.max(value))
}

/// True when `value` is over the `cap` threshold. Ported verbatim from
/// d2d-ui's `CapacityBar::is_over`.
pub fn capacity_bar_is_over(value: f64, cap: f64) -> bool {
    value > cap
}

/// Position of `units` along the track expressed as a percentage of `max`,
/// clamped to `[0, 100]`. Returns `0.0` when `max` is non-positive (instead
/// of dividing by zero). Mirrors d2d-ui's pixel-space `CapacityBar::x_at`,
/// expressed in percent for CSS `width`/`left` styling instead of DIPs.
pub fn capacity_bar_percent(units: f64, max: f64) -> f64 {
    if max > 0.0 {
        (units / max * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Width, as a percentage, of the under-cap portion of the fill -- the fill
/// clipped to the cap-line so the overflow band always starts exactly at the
/// cap position.
pub fn capacity_bar_under_cap_percent(value_percent: f64, cap_percent: f64) -> f64 {
    value_percent.min(cap_percent)
}

/// `(left, width)` percentages of the over-cap overflow band, or `None` when
/// `value_percent` does not exceed `cap_percent` (no overflow to draw).
pub fn capacity_bar_overflow_band(value_percent: f64, cap_percent: f64) -> Option<(f64, f64)> {
    if value_percent > cap_percent {
        Some((cap_percent, value_percent - cap_percent))
    } else {
        None
    }
}
