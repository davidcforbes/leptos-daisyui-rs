use super::{ClientCallAttempt, ClientCallOutcome, ClientCallRefusal, ClientCallRegenerationState};
use crate::components::SoftphoneTexts;

/// Reactive user-facing copy. Outcome labels follow the fixed vocabulary order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientCallWorkspaceTexts {
    /// Region name.
    pub label: String,
    /// Dismissal proposal label.
    pub close: String,
    /// Destination field label.
    pub destination: String,
    /// Pre-dial editing explanation.
    pub destination_hint: String,
    /// Saved-number choices heading.
    pub saved_numbers: String,
    /// Empty saved-number list explanation.
    pub no_numbers: String,
    /// Number pad disclosure label.
    pub keypad: String,
    /// Delete final character action.
    pub backspace: String,
    /// Explicit contact-write label.
    pub save_number: String,
    /// Contact-field destination for the explicit write.
    pub number_target: String,
    /// Shared pending write label.
    pub saving: String,
    /// Launch button label.
    pub call: String,
    /// Attempt labels in Ready, Submitting, AgentRinging, Uncertain, Refused, Finished, Managed order.
    pub attempt_labels: [String; 7],
    /// Structured refusal labels in AgentNotReady, AgentNotAvailable, NotSubmitted order.
    pub refusal_labels: [String; 3],
    /// Script preparation status.
    pub script_preparing: String,
    /// Host-measured elapsed preparation label.
    pub script_elapsed: String,
    /// Host-supplied typical preparation duration label.
    pub script_typical: String,
    /// Preparation time unit.
    pub script_seconds: String,
    /// Script preparation failure label.
    pub script_failed: String,
    /// Missing file-specific detail explanation for a script beat.
    pub script_no_file_detail: String,
    /// Explicit script-regeneration request label.
    pub regenerate: String,
    /// Regeneration labels in Idle, Busy, Accepted, Succeeded, Failed order.
    pub regeneration_labels: [String; 5],
    /// Provider-reported talk-time label, separate from elapsed and manual duration.
    pub provider_talk_time: String,
    /// Unknown provider talk-time explanation; must not imply zero.
    pub provider_talk_unknown: String,
    /// Explanation that bridge acceptance is not a client connection.
    pub bridge_hint: String,
    /// Explanation of reconciliation before retry.
    pub uncertain_hint: String,
    /// Work-record heading.
    pub wrap_up: String,
    /// Work-record explanation.
    pub wrap_up_hint: String,
    /// Outcome field label.
    pub outcome: String,
    /// Empty outcome select prompt.
    pub choose_outcome: String,
    /// Fixed six-outcome labels in the order of the outcome vocabulary.
    pub outcomes: [String; 6],
    /// Notes field label.
    pub notes: String,
    /// Manual optional duration field label.
    pub duration: String,
    /// Invalid optional duration explanation.
    pub duration_error: String,
    /// Callback instructions label.
    pub follow_up: String,
    /// Callback instructions hint.
    pub follow_up_hint: String,
    /// Save work-record label.
    pub save_wrap_up: String,
    /// Confirmed persisted state label.
    pub saved: String,
    /// Explanation while completion has not been established.
    pub finish_hint: String,
    /// Existing live-console labels and statuses.
    pub softphone: SoftphoneTexts,
}

impl Default for ClientCallWorkspaceTexts {
    fn default() -> Self {
        Self {
            label: "Client call workspace".into(), close: "Close".into(), destination: "Number to call".into(),
            destination_hint: "Choose a saved number or enter a destination. Changes here do not update the contact.".into(),
            saved_numbers: "Saved numbers".into(), no_numbers: "No saved numbers. Enter a destination to continue.".into(),
            keypad: "Number pad".into(), backspace: "Backspace".into(), save_number: "Save contact number".into(),
            saving: "Saving…".into(), call: "Place call".into(),
            number_target: "Contact field to update".into(),
            attempt_labels: ["Ready to call", "Submitting call", "Ringing your phone", "Call status unknown", "Call not placed", "Call finished", "Managed call"].map(String::from),
            refusal_labels: ["Agent not ready", "Agent not available", "Call not submitted"].map(String::from),
            script_preparing: "Preparing script".into(), script_elapsed: "Elapsed".into(),
            script_typical: "Typically".into(), script_seconds: "seconds".into(),
            script_failed: "Script preparation failed".into(), script_no_file_detail: "No file-specific detail available.".into(),
            regenerate: "Regenerate script".into(),
            regeneration_labels: ["Ready to regenerate", "Submitting regeneration", "Regeneration accepted", "Script regenerated", "Regeneration failed"].map(String::from),
            provider_talk_time: "Provider talk time".into(), provider_talk_unknown: "Not confirmed".into(),
            bridge_hint: "The request was accepted. Client connection is not yet confirmed.".into(),
            uncertain_hint: "A call may have been placed. Check the provider before trying again.".into(),
            wrap_up: "Call record".into(), wrap_up_hint: "Record what happened. Save after the call finishes.".into(),
            outcome: "Outcome".into(), choose_outcome: "Choose an outcome".into(),
            outcomes: ["Interested", "Not interested", "No response / busy", "Requested more information", "Requested a callback", "Invalid number"].map(String::from),
            notes: "Call notes".into(), duration: "Duration in minutes (optional)".into(),
            duration_error: "Enter a positive whole number of minutes, or leave blank.".into(),
            follow_up: "Callback instructions".into(), follow_up_hint: "Include when and who should call. This records instructions; it does not schedule a call.".into(),
            save_wrap_up: "Save call record".into(), saved: "Saved".into(),
            finish_hint: "Saving becomes available when the host confirms this call has finished.".into(),
            softphone: SoftphoneTexts::default(),
        }
    }
}

impl ClientCallWorkspaceTexts {
    /// Localized structured attempt status.
    pub fn attempt(&self, attempt: ClientCallAttempt) -> String {
        if let ClientCallAttempt::RefusedWith(refusal) = attempt {
            let index = match refusal {
                ClientCallRefusal::AgentNotReady => 0,
                ClientCallRefusal::AgentNotAvailable => 1,
                ClientCallRefusal::NotSubmitted => 2,
            };
            return self.refusal_labels[index].clone();
        }
        let index = match attempt {
            ClientCallAttempt::Ready => 0,
            ClientCallAttempt::Submitting => 1,
            ClientCallAttempt::AgentRinging => 2,
            ClientCallAttempt::Uncertain => 3,
            ClientCallAttempt::Refused => 4,
            ClientCallAttempt::RefusedWith(_) => unreachable!("structured refusals handled above"),
            ClientCallAttempt::Finished => 5,
            ClientCallAttempt::Managed => 6,
        };
        self.attempt_labels[index].clone()
    }

    /// Localized independent regeneration acknowledgment status.
    pub fn regeneration(&self, state: ClientCallRegenerationState) -> String {
        let index = match state {
            ClientCallRegenerationState::Idle => 0,
            ClientCallRegenerationState::Busy => 1,
            ClientCallRegenerationState::Accepted => 2,
            ClientCallRegenerationState::Succeeded => 3,
            ClientCallRegenerationState::Failed => 4,
        };
        self.regeneration_labels[index].clone()
    }

    /// Localized label for an explicit outcome.
    pub fn outcome(&self, outcome: ClientCallOutcome) -> String {
        let index = ClientCallOutcome::ALL
            .iter()
            .position(|item| *item == outcome)
            .expect("complete outcome vocabulary");
        self.outcomes[index].clone()
    }
}
