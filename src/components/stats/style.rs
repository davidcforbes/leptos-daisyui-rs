/// # StatDelta Trend Variants
///
/// Semantic trend indicator for the `StatDelta` helper.  "Positive" here
/// means **semantically good**, not directional — sales going up is
/// Positive, churn going up is Negative.  The arrow direction and the
/// color are driven from this enum together, so the rendered glyph
/// always matches the colour:
///
/// | Variant   | Color class    | Arrow |
/// |-----------|---------------|-------|
/// | Positive  | `text-success` | ↗︎    |
/// | Negative  | `text-error`   | ↘︎    |
/// | Neutral   | (inherits)    | →     |
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatDeltaTrend {
    /// Good direction (up-and-to-the-right arrow, success color).
    #[default]
    Positive,

    /// Bad direction (down-and-to-the-right arrow, error color).
    Negative,

    /// No change / neutral (right arrow, inherits stat-desc color).
    Neutral,
}

impl StatDeltaTrend {
    /// daisyUI / Tailwind color class for this trend.  `Neutral` returns
    /// an empty string so the surrounding `stat-desc` styling shows
    /// through unchanged.
    pub fn as_str(&self) -> &'static str {
        match self {
            StatDeltaTrend::Positive => "text-success",
            StatDeltaTrend::Negative => "text-error",
            StatDeltaTrend::Neutral => "",
        }
    }

    /// Single-glyph Unicode arrow rendered at the start of the delta.
    /// Uses the same glyphs the existing Stats demo already inlines
    /// (`↗︎` / `↘︎`) so the visual is unchanged when migrating call
    /// sites.
    pub fn arrow(&self) -> &'static str {
        match self {
            StatDeltaTrend::Positive => "↗︎",
            StatDeltaTrend::Negative => "↘︎",
            StatDeltaTrend::Neutral => "→",
        }
    }
}
