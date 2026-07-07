/// One entry in a [`VerticalSteps`](super::VerticalSteps) rail.
///
/// Ported from d2d-ui's `controls::vertical_steps::VerticalStep` (title +
/// plain-language body + optional technical sub-line + optional action
/// button label). The renderer-owned geometry fields (`rect`, pixel offsets)
/// do not port — the browser lays the content out.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VerticalStep {
    /// Current status, colors the dot and (if `Ready`) lights the rail
    /// segment below it.
    pub status: super::StepStatus,
    /// Step title (bold).
    pub title: String,
    /// Plain-language status line shown under the title.
    pub body: String,
    /// Optional technical sub-line (e.g. a hostname or error code); omitted
    /// when `None`.
    pub tech: Option<String>,
    /// Optional action-button label (e.g. "Fix", "Retry"); omitted when
    /// `None`.
    pub action_label: Option<String>,
}

impl VerticalStep {
    /// Create a step with no tech line and no action button.
    pub fn new(
        status: super::StepStatus,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            status,
            title: title.into(),
            body: body.into(),
            tech: None,
            action_label: None,
        }
    }

    /// Attach a technical sub-line.
    pub fn with_tech(mut self, tech: impl Into<String>) -> Self {
        self.tech = Some(tech.into());
        self
    }

    /// Attach an action-button label.
    pub fn with_action(mut self, label: impl Into<String>) -> Self {
        self.action_label = Some(label.into());
        self
    }
}

/// Content fingerprint of a step, used (together with the step's index) as
/// the row key inside [`VerticalSteps`](super::VerticalSteps)'s `<For>` so a
/// row re-renders whenever *any* of its fields change — status flips, but
/// also body-text updates that arrive without a status change (e.g.
/// "Checking..." -> "Checked 3s ago").
pub fn step_key(step: &VerticalStep) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    step.hash(&mut hasher);
    hasher.finish()
}
