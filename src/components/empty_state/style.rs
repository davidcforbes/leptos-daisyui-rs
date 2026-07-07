/// Text-color override for EmptyState's icon, title, and subtitle slots.
///
/// `Default` applies no override class at all, so the slot keeps whatever
/// resting appearance its static layout classes already give it (a muted
/// `opacity-60` icon/subtitle, a plain-color title). The other variants add
/// a `text-*` utility class on top, which composes cleanly with the static
/// `opacity-*` classes since they affect different CSS properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum EmptyStateColor {
    /// No color override (default resting appearance)
    #[default]
    Default,

    /// Neutral color
    Neutral,

    /// Primary theme color
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

    /// Error color
    Error,
}

impl EmptyStateColor {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            EmptyStateColor::Default => "",
            EmptyStateColor::Neutral => "text-neutral",
            EmptyStateColor::Primary => "text-primary",
            EmptyStateColor::Secondary => "text-secondary",
            EmptyStateColor::Accent => "text-accent",
            EmptyStateColor::Info => "text-info",
            EmptyStateColor::Success => "text-success",
            EmptyStateColor::Warning => "text-warning",
            EmptyStateColor::Error => "text-error",
        }
    }
}
