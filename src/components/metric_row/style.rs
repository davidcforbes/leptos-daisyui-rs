/// Text-color override for MetricRow's label and value slots.
///
/// `Default` applies no override class, so each slot keeps its static
/// resting appearance (a muted `opacity-75` label (75, not 60: 60 fails WCAG AA color-contrast on base-100 — axe BLOCKING, office-perf tier1_a11y 2026-08-16), a plain
/// `text-base-content` value). The other variants add a `text-*` utility
/// class on top -- useful for status-tinted values, e.g. a red overdue
/// amount or a green positive delta. Mirrors d2d-ui's `MetricRow`, which
/// stored independent `D2D1_COLOR_F` values for `label_color`/`value_color`.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MetricRowColor {
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

impl MetricRowColor {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricRowColor::Default => "",
            MetricRowColor::Neutral => "text-neutral",
            MetricRowColor::Primary => "text-primary",
            MetricRowColor::Secondary => "text-secondary",
            MetricRowColor::Accent => "text-accent",
            MetricRowColor::Info => "text-info",
            MetricRowColor::Success => "text-success",
            MetricRowColor::Warning => "text-warning",
            MetricRowColor::Error => "text-error",
        }
    }
}

/// Root container layout classes for the row (default) vs. stacked variant.
///
/// The default variant is a two-column `label ... value` flex row; `stacked`
/// switches to a column so the label sits above the value instead. Mirrors
/// d2d-ui's `MetricRow::stacked()` builder flag.
pub fn container_class(stacked: bool) -> &'static str {
    if stacked {
        "flex flex-col gap-1"
    } else {
        "flex items-baseline justify-between gap-2"
    }
}

/// Label slot classes. The stacked variant uses a smaller label size
/// (mirrors d2d-ui's `fmt_small` for the stacked label vs. the row variant's
/// body-sized muted label); both stay muted via `opacity-75` (the AA-passing muted level).
pub fn label_class(stacked: bool) -> &'static str {
    if stacked {
        "text-xs opacity-75"
    } else {
        "text-sm opacity-75"
    }
}

/// Value slot classes. `bold` maps to d2d-ui's `bold_value` (semi-bold
/// emphasis); the row (non-stacked) variant is right-aligned to match the
/// renderer's trailing-format column, while the stacked variant is left
/// (block) aligned beneath the label.
pub fn value_class(stacked: bool, bold: bool) -> &'static str {
    match (stacked, bold) {
        (true, true) => "text-sm font-semibold",
        (true, false) => "text-sm",
        (false, true) => "text-sm font-semibold text-right",
        (false, false) => "text-sm text-right",
    }
}

/// Hairline bottom divider classes, or empty when `divider` is `false`.
/// Mirrors d2d-ui's optional `with_divider()` bottom-edge line.
pub fn divider_class(divider: bool) -> &'static str {
    if divider {
        "pb-1 border-b border-base-200"
    } else {
        ""
    }
}
