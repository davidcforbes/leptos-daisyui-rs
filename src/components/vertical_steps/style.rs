/// Status of a single [`VerticalStep`](super::VerticalStep) — colors the step's
/// dot on the rail and decides whether the rail segment leading away from it
/// is "lit" (colored + animated).
///
/// Ported from d2d-ui's `controls::vertical_steps::StepStatus`, which mapped
/// each variant to a `D2D1_COLOR_F` for its owner-drawn dot. Here each variant
/// maps to a daisyUI/Tailwind utility-class string instead of a raw color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StepStatus {
    /// Done / healthy — solid `success` dot.
    Ready,
    /// In flight — solid `accent` dot (pulses; the rail below a `Ready` step
    /// animates a flow, not the `Checking` dot itself).
    Checking,
    /// Not yet reached — hollow ring (matches the card background so only the
    /// border reads as a dot).
    #[default]
    Pending,
    /// Blocked on the user — solid `warning` dot.
    NeedsYou,
    /// Failed — solid `error` dot.
    Down,
}

impl StepStatus {
    /// CSS classes for the step's dot.
    pub fn dot_class(&self) -> &'static str {
        match self {
            StepStatus::Ready => "bg-success border-2 border-success",
            StepStatus::Checking => "bg-accent border-2 border-accent animate-pulse",
            StepStatus::Pending => "bg-base-100 border-2 border-base-300",
            StepStatus::NeedsYou => "bg-warning border-2 border-warning",
            StepStatus::Down => "bg-error border-2 border-error",
        }
    }

    /// Human-readable status word. Rendered as visually-hidden text next to
    /// each step's title so the status is announced to screen readers, since
    /// the dot otherwise conveys it by color alone.
    pub fn label(&self) -> &'static str {
        match self {
            StepStatus::Ready => "Ready",
            StepStatus::Checking => "Checking",
            StepStatus::Pending => "Pending",
            StepStatus::NeedsYou => "Needs you",
            StepStatus::Down => "Down",
        }
    }
}

/// Whether the rail segment leading away from a step with this `status` is
/// "lit" (colored `accent` with an animated flowing dash) rather than dim
/// `base-300`.
///
/// Direct port of d2d-ui's `VerticalSteps::draw` rail logic: `let lit =
/// self.steps[i - 1].status == StepStatus::Ready;` — the segment above step
/// `i` lights up when the step above it (`i - 1`) is `Ready`. Rendered here as
/// the segment *below* the `Ready` step instead of *above* the next one
/// (equivalent relationship, simpler to express per-row in the view).
pub fn segment_lit(status: StepStatus) -> bool {
    status == StepStatus::Ready
}

/// CSS classes for a rail segment, given whether it is [`segment_lit`].
pub fn vstep_rail_class(lit: bool) -> &'static str {
    if lit { "bg-accent" } else { "bg-base-300" }
}

/// Whether step `index` (of `len` total steps) has a rail segment connecting
/// it to the next step. Every step has one except the last.
pub fn has_rail_segment(index: usize, len: usize) -> bool {
    index + 1 < len
}

/// Layout classes for a step's content slot. Non-last steps (those with a
/// rail segment) carry bottom padding so the flex-stretched rail has vertical
/// travel between dots; the last step drops it so the list ends flush.
pub fn content_class(has_segment: bool) -> &'static str {
    if has_segment { "flex-1 pb-6" } else { "flex-1" }
}
