/// # Scheduler Event Color Variants
///
/// Semantic color for a [`SchedulerEvent`](super::types::SchedulerEvent)
/// block: a soft tinted background plus a solid left accent bar. Mirrors
/// d2d-ui's `DayScheduler::draw`, which filled each block with a translucent
/// (alpha 0.15) version of the event's color and drew a solid 3px accent bar
/// in the same color along the block's left edge. [`bg_class`](Self::bg_class)
/// uses Tailwind's `/15` opacity classes (15%, matching d2d's alpha 0.15);
/// [`border_class`](Self::border_class) renders as a `border-l-4` (4px)
/// accent bar, a Tailwind rounding of d2d's 3px accent bar.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SchedulerEventColor {
    /// Neutral color (default)
    #[default]
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

impl SchedulerEventColor {
    /// Soft tinted background CSS class for the event block's body.
    pub fn bg_class(&self) -> &'static str {
        match self {
            SchedulerEventColor::Neutral => "bg-neutral/15",
            SchedulerEventColor::Primary => "bg-primary/15",
            SchedulerEventColor::Secondary => "bg-secondary/15",
            SchedulerEventColor::Accent => "bg-accent/15",
            SchedulerEventColor::Info => "bg-info/15",
            SchedulerEventColor::Success => "bg-success/15",
            SchedulerEventColor::Warning => "bg-warning/15",
            SchedulerEventColor::Error => "bg-error/15",
        }
    }

    /// Solid left-accent-bar border CSS class for the event block.
    pub fn border_class(&self) -> &'static str {
        match self {
            SchedulerEventColor::Neutral => "border-neutral",
            SchedulerEventColor::Primary => "border-primary",
            SchedulerEventColor::Secondary => "border-secondary",
            SchedulerEventColor::Accent => "border-accent",
            SchedulerEventColor::Info => "border-info",
            SchedulerEventColor::Success => "border-success",
            SchedulerEventColor::Warning => "border-warning",
            SchedulerEventColor::Error => "border-error",
        }
    }
}
