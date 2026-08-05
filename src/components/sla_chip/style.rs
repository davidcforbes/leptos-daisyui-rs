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

    /// The thing being timed has STOPPED — it did not merely breach its deadline.
    ///
    /// ⚠️ NOT A DEGREE OF `Red`, A DIFFERENT CLAIM. `Red` says "past the deadline and still
    /// counting"; this says "there is nothing left to count". A stopped feed shown as breached
    /// is indistinguishable from a very late one, which is how a mirror that had been dead for
    /// a year kept reading as an ordinary overdue row.
    ///
    /// ⚠️ NOT COMPUTED BY [`sla_chip_tone`], DELIBERATELY. Whether a series has ended is a fact
    /// about the DATA — for that mirror it was `max(modified) == max(created)` — and the chip
    /// cannot derive it from a deadline. Callers pass it in.
    Stopped,
}

impl SlaTone {
    /// daisyUI badge color class for this tone.
    pub fn as_str(&self) -> &'static str {
        match self {
            SlaTone::Green => "badge-success",
            SlaTone::Amber => "badge-warning",
            SlaTone::Red => "badge-error",
            SlaTone::None => "badge-neutral",
            SlaTone::Stopped => "badge-error",
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
            // A heavier border than `Red`: the pill must not read as one more overdue row.
            SlaTone::Stopped => "border-2 border-error/70",
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
            // "square" reads as a halted transport control, not another alarm.
            SlaTone::Stopped => Some("square"),
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

/// Format a non-negative duration (ms) compactly, **with precision that
/// degrades as the magnitude grows**: `Ym`, `Xh Ym`, `Xh`, `Xd Yh`, `Xd`,
/// `X.Yy`.
///
/// # ★ THIS FUNCTION'S "ported verbatim" CLAIM WAS FALSE (office op-jmxb)
///
/// The header said "Ported verbatim from d2d-ui's `fmt_duration`". It was not.
/// d2d-ui's version carries a clamp this port dropped:
///
/// ```text
/// if days > MAX_DISPLAY_DAYS { format!(">{MAX_DISPLAY_DAYS}d") }   // 99
/// ```
///
/// added by 4iiz-Office op-4tr for precisely this failure. So the desktop had
/// already solved the problem, the web port silently arrived without the
/// guard, and the "verbatim" note is what made the gap invisible — anyone
/// comparing the two would have trusted the comment instead of the code.
/// **That is the actual root cause of op-jmxb**, not the format itself.
///
/// # Why the fix is not simply re-adding d2d-ui's clamp
///
/// `>99d` is bounded, and it is bounded by DISCARDING ORDER. In a queue sorted
/// by urgency, 100 days overdue and 5.4 years overdue both render `>99d`, so
/// the rows that most need distinguishing become identical at exactly the top
/// of the list. Degrading the PRECISION keeps the ranking legible while
/// costing the same width. (d2d-ui should adopt this too — filed there so the
/// two surfaces do not disagree about the same number.)
///
/// # What it looked like
///
/// The old form was `Xd Yh` / `Xh Ym` / `Ym`, unconditionally two units. That
/// is right for the magnitudes a mockup contains and wrong for the ones a
/// warehouse contains. Against real data (4iiz Office Performance, Consultant
/// Task Queue, 2026-07-31) every breached row rendered as
///
/// ```text
/// +1981d 13h
/// over
/// ```
///
/// — three tokens, too wide for a one-line pill, so the word "over" wrapped
/// out through the chip's own red border. Twelve of twelve visible rows. The
/// breach values were real (+1981d, +1785d, +1462d ... from 2021-2023 due
/// dates); the number was right and the chip could not hold it.
///
/// The fix is not a wider column, and deliberately not: **the second unit is
/// noise once the first one is large.** Nobody reading "5.4 years overdue"
/// needs the 13 hours, and printing it costs the exact width that broke the
/// pill. So the finer unit is dropped as soon as the coarser one reaches two
/// digits, and past a year the unit itself changes.
///
/// This bounds the breached label at **12 characters** (`+9d 23h over`) — the
/// same width the design's own hours/days samples already fit — for ANY input,
/// including ones nobody has seen yet. That is the property worth having: the
/// old format had no bound at all, so it was only ever one unusually stale
/// record away from breaking again.
///
/// The chip also sets `whitespace-nowrap` (see `component.rs`). Belt and
/// braces on purpose: this function bounds what is normally produced, and the
/// class guarantees that even an unforeseen label cannot wrap out of its own
/// border. A visual defect that reappears is worse than one that clips.
fn sla_chip_fmt_duration(ms: i64) -> String {
    let total_min = ms / 60_000;
    let days = total_min / (60 * 24);
    let hours = (total_min / 60) % 24;
    let mins = total_min % 60;
    // A year, in days. 365 flat: this is a glanceable magnitude, not a date
    // calculation, and leap years would move the boundary by a day while
    // changing nothing anyone reads.
    const DAYS_PER_YEAR: i64 = 365;
    // Where the finer unit stops carrying information and starts costing width.
    const TWO_DIGITS: i64 = 10;

    if days >= DAYS_PER_YEAR {
        // One decimal, so "1.0y" and "2.7y" are distinguishable — a whole
        // number alone would round two very different staleness levels
        // together at the top of the scale, which is where the worst records
        // live.
        format!("{:.1}y", days as f64 / DAYS_PER_YEAR as f64)
    } else if days >= TWO_DIGITS {
        format!("{days}d")
    } else if days > 0 {
        format!("{days}d {hours}h")
    } else if hours >= TWO_DIGITS {
        format!("{hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}
