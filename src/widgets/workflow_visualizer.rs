use leptos::prelude::*;

/// Status of a single workflow step.
#[derive(Clone, Debug, PartialEq)]
pub enum StepStatus {
    /// Step has not yet started.
    Pending,
    /// Step is currently being worked on.
    InProgress,
    /// Step completed successfully.
    Completed,
    /// Step was rejected.
    Rejected,
    /// Step was skipped.
    Skipped,
}

impl StepStatus {
    /// Returns a DaisyUI-compatible CSS class for the step indicator.
    fn step_class(&self) -> &'static str {
        match self {
            StepStatus::Pending => "step",
            StepStatus::InProgress => "step step-primary",
            StepStatus::Completed => "step step-success",
            StepStatus::Rejected => "step step-error",
            StepStatus::Skipped => "step step-warning",
        }
    }

    /// Human-readable label for the status.
    fn label(&self) -> &'static str {
        match self {
            StepStatus::Pending => "Pending",
            StepStatus::InProgress => "In Progress",
            StepStatus::Completed => "Completed",
            StepStatus::Rejected => "Rejected",
            StepStatus::Skipped => "Skipped",
        }
    }

    /// Badge class for the status label.
    fn badge_class(&self) -> &'static str {
        match self {
            StepStatus::Pending => "badge badge-ghost badge-sm",
            StepStatus::InProgress => "badge badge-primary badge-sm",
            StepStatus::Completed => "badge badge-success badge-sm",
            StepStatus::Rejected => "badge badge-error badge-sm",
            StepStatus::Skipped => "badge badge-warning badge-sm",
        }
    }
}

/// A single step in a workflow sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowStep {
    /// Display name for this step.
    pub name: String,
    /// Current status of the step.
    pub status: StepStatus,
    /// Optional user or group assigned to this step.
    pub assignee: Option<String>,
}

/// Horizontal workflow step visualizer for request lifecycle visualization.
///
/// Renders steps in a horizontal sequence connected by lines, using the
/// DaisyUI steps component pattern. Each step is color-coded by its status,
/// and the current step is highlighted.
#[component]
pub fn WorkflowVisualizer(
    /// The ordered list of workflow steps.
    steps: Vec<WorkflowStep>,
    /// Optional index of the currently active step (0-based).
    #[prop(optional)]
    current_step: Option<usize>,
) -> impl IntoView {
    view! {
        <div class="w-full overflow-x-auto py-4">
            // DaisyUI steps (horizontal)
            <ul class="steps steps-horizontal w-full">
                {steps.iter().enumerate().map(|(idx, step)| {
                    let is_current = current_step == Some(idx);
                    let step_class = step.status.step_class();
                    let name = step.name.clone();
                    let assignee = step.assignee.clone();
                    let badge_class = step.status.badge_class();
                    let status_label = step.status.label();

                    view! {
                        <li class=step_class data-content={match step.status {
                            StepStatus::Completed => "\u{2713}",
                            StepStatus::Rejected => "\u{2717}",
                            StepStatus::Skipped => "\u{2014}",
                            StepStatus::InProgress => "\u{25CF}",
                            StepStatus::Pending => "",
                        }}>
                            <div class="flex flex-col items-center gap-1 pt-2">
                                <span class=move || {
                                    if is_current {
                                        "font-bold text-sm"
                                    } else {
                                        "text-sm"
                                    }
                                }>
                                    {name}
                                </span>
                                <span class=badge_class>{status_label}</span>
                                {assignee.map(|a| view! {
                                    <span class="text-xs text-base-content/50">{a}</span>
                                })}
                            </div>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}
