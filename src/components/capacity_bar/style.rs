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
        }
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
