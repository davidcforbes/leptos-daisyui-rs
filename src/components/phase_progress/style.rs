/// Fill percentage of one phase segment, given the active phase index and
/// the active phase's completion percent.
///
/// Completed phases (`index < current`) are solid (100%), the active phase
/// (`index == current`) is partially filled to `pct` (clamped to 100), and
/// future phases are empty (0%). A `current` past the end of the phase list
/// means every phase has completed, so every segment is solid.
pub fn phase_fill_percent(index: usize, current: usize, pct: u8) -> f64 {
    if index < current {
        100.0
    } else if index == current {
        f64::from(pct.min(100))
    } else {
        0.0
    }
}

/// Overall run progress across all phases as a `[0, 100]` percentage, for
/// `aria-valuenow`: each phase contributes an equal share, and the active
/// phase contributes its share scaled by `pct`. Returns `0.0` for an empty
/// phase list (never a divide-by-zero), and `100.0` when `current` is past
/// the last phase (the run finished).
pub fn phase_overall_percent(phase_count: usize, current: usize, pct: u8) -> f64 {
    if phase_count == 0 {
        return 0.0;
    }
    if current >= phase_count {
        return 100.0;
    }
    let done = current as f64;
    let active = f64::from(pct.min(100)) / 100.0;
    ((done + active) / phase_count as f64 * 100.0).clamp(0.0, 100.0)
}

/// Human-readable progress description for `aria-valuetext` (and the default
/// accessible name): names the active phase, its percent, its ordinal, and
/// whether it failed -- e.g. `"reconcile 40% (phase 2 of 3)"` or
/// `"reconcile failed at 40% (phase 2 of 3)"`. An empty phase list reads
/// `"no phases"`; a `current` past the end reads `"all N phases complete"`.
pub fn phase_progress_value_text(
    phases: &[String],
    current: usize,
    pct: u8,
    failed: bool,
) -> String {
    if phases.is_empty() {
        return "no phases".to_string();
    }
    if current >= phases.len() {
        return format!("all {} phases complete", phases.len());
    }
    let name = &phases[current];
    let pct = pct.min(100);
    let ordinal = format!("(phase {} of {})", current + 1, phases.len());
    if failed {
        format!("{name} failed at {pct}% {ordinal}")
    } else {
        format!("{name} {pct}% {ordinal}")
    }
}

/// State of one phase segment, exposed as `data-phase-state` so tests and
/// consumers can assert on run state without reverse-engineering fill widths.
pub fn phase_state(index: usize, current: usize, failed: bool) -> &'static str {
    if index < current {
        "complete"
    } else if index == current {
        if failed { "failed" } else { "active" }
    } else {
        "pending"
    }
}
