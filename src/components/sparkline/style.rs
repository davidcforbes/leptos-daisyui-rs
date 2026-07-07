/// # Sparkline Color Variants
///
/// Style enum for the Sparkline component's polyline stroke and readout
/// accents. daisyUI has no dedicated sparkline component, so the SVG stroke
/// uses `currentColor` -- applying one of these `text-*` utility classes to
/// the wrapper lets the sparkline pick up the active theme color (and follow
/// theme switches) without any custom CSS.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SparklineColor {
    /// Base content color (no explicit color class beyond `text-base-content`)
    #[default]
    Default,

    /// Primary brand color
    Primary,

    /// Secondary brand color
    Secondary,

    /// Accent brand color
    Accent,

    /// Success (positive trend) color
    Success,

    /// Info color
    Info,

    /// Warning color
    Warning,

    /// Error color
    Error,
}

impl SparklineColor {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            SparklineColor::Default => "text-base-content",
            SparklineColor::Primary => "text-primary",
            SparklineColor::Secondary => "text-secondary",
            SparklineColor::Accent => "text-accent",
            SparklineColor::Success => "text-success",
            SparklineColor::Info => "text-info",
            SparklineColor::Warning => "text-warning",
            SparklineColor::Error => "text-error",
        }
    }
}

/// Most recent sample, or `0.0` if `samples` is empty.
pub fn sparkline_current(samples: &[f32]) -> f32 {
    samples.last().copied().unwrap_or(0.0)
}

/// Peak sample, floored at `1.0` so the polyline scale never divides by zero
/// and a flat/empty series renders along the baseline.
pub fn sparkline_peak(samples: &[f32]) -> f32 {
    samples.iter().copied().fold(1.0_f32, f32::max)
}

/// Whether the title/current/peak readout row should render. An empty title
/// hides the row -- appropriate for an inline/in-cell sparkline.
pub fn sparkline_has_readout(title: &str) -> bool {
    !title.is_empty()
}

/// Build the SVG `<polyline>` `points` attribute string for `samples`,
/// scaled into a `width` x `height` viewBox (origin top-left, y grows down).
/// Returns an empty string when there are fewer than 2 samples or the
/// viewBox is degenerate, so nothing is drawn beyond the baseline.
pub fn sparkline_points(samples: &[f32], width: f32, height: f32) -> String {
    let n = samples.len();
    if n < 2 || width <= 0.0 || height <= 0.0 {
        return String::new();
    }

    let peak = sparkline_peak(samples);
    let step = width / (n - 1) as f32;

    samples
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = i as f32 * step;
            let y = height - (v / peak) * height;
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format the bold title + current-value readout label, e.g.
/// `"Throughput  4.0 KB/s"` (or `"Throughput  4.0"` with no unit).
pub fn sparkline_current_label(title: &str, unit: &str, current: f32) -> String {
    if unit.is_empty() {
        format!("{title}  {current:.1}")
    } else {
        format!("{title}  {current:.1} {unit}")
    }
}

/// Format the muted peak readout label, e.g. `"peak 9"`.
pub fn sparkline_peak_label(peak: f32) -> String {
    format!("peak {peak:.0}")
}
