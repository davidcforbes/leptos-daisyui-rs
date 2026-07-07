/// Default "approaching" threshold: 2 hours, in milliseconds. Ported from
/// d2d-ui's `SlaChip::DEFAULT_THRESHOLD_MS` (Office Performance's
/// `APPROACH_MS`).
pub const SLA_CHIP_DEFAULT_THRESHOLD_MS: i64 = 120 * 60 * 1000;

/// Visual severity of an [`SlaChip`](crate::components::SlaChip), computed
/// purely from the deadline vs a caller-supplied `now_ms` (the caller owns
/// the clock -- no internal timer). Ported near-verbatim from d2d-ui's
/// `SlaTone`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlaTone {
    /// Inside target: more than `threshold_ms` remaining.
    Green,

    /// Approaching: within `threshold_ms` of the deadline (inclusive).
    Amber,

    /// Breached: past the deadline.
    Red,

    /// No SLA defined. Also the default tone.
    #[default]
    None,
}

impl SlaTone {
    /// daisyUI badge color class for this tone.
    pub fn as_str(&self) -> &'static str {
        match self {
            SlaTone::Green => "badge-success",
            SlaTone::Amber => "badge-warning",
            SlaTone::Red => "badge-error",
            SlaTone::None => "badge-neutral",
        }
    }

    /// Border color class used by the enriched (`show_icon`) variant, so the
    /// pale/soft pill still reads at a glance on a light page. Mirrors
    /// d2d-ui's enriched-border behavior (beads-p4v4), translated from an
    /// alpha-blended stroke to a Tailwind border-color-with-opacity utility.
    pub fn border_class(&self) -> &'static str {
        match self {
            SlaTone::Green => "border border-success/45",
            SlaTone::Amber => "border border-warning/45",
            SlaTone::Red => "border border-error/45",
            SlaTone::None => "border border-neutral/45",
        }
    }

    /// Leading severity icon name (for the crate's [`Icon`](crate::components::Icon)
    /// component, which wraps Lucide icons), or `None` for the "No SLA" tone.
    /// Ported from d2d-ui's `SlaTone::icon` -- Segoe Fluent glyphs there
    /// (clock / warning / error badge) become the equivalent Lucide names
    /// here. beads-p4v4
    pub fn icon_name(&self) -> Option<&'static str> {
        match self {
            SlaTone::Green => Some("clock"),
            SlaTone::Amber => Some("triangle-alert"),
            SlaTone::Red => Some("circle-alert"),
            SlaTone::None => None,
        }
    }
}

/// Tone for `deadline_ms` at `now_ms`, given an "approaching" window of
/// `threshold_ms`. Ported verbatim from d2d-ui's `SlaChip::tone`.
pub fn sla_chip_tone(deadline_ms: Option<i64>, now_ms: i64, threshold_ms: i64) -> SlaTone {
    match deadline_ms {
        None => SlaTone::None,
        Some(deadline) => {
            let rem = deadline - now_ms;
            if rem < 0 {
                SlaTone::Red
            } else if rem <= threshold_ms {
                SlaTone::Amber
            } else {
                SlaTone::Green
            }
        }
    }
}

/// Chip text for `deadline_ms` at `now_ms`: remaining `Xd Yh`/`Xh Ym`/`Ym`
/// while inside the deadline, `+... over` once breached, or `No SLA` when
/// none is defined. Ported verbatim from d2d-ui's `SlaChip::label`.
pub fn sla_chip_label(deadline_ms: Option<i64>, now_ms: i64) -> String {
    match deadline_ms {
        None => "No SLA".to_string(),
        Some(deadline) => {
            let rem = deadline - now_ms;
            if rem < 0 {
                format!("+{} over", sla_chip_fmt_duration(-rem))
            } else {
                sla_chip_fmt_duration(rem)
            }
        }
    }
}

/// Format a non-negative duration (ms) compactly: `Xd Yh`, `Xh Ym`, or `Ym`.
/// Ported verbatim from d2d-ui's `fmt_duration`.
fn sla_chip_fmt_duration(ms: i64) -> String {
    let total_min = ms / 60_000;
    let days = total_min / (60 * 24);
    let hours = (total_min / 60) % 24;
    let mins = total_min % 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}
