//! Keyed action feedback for snapshot-table pages.

use crate::components::{Button, ButtonSize, ButtonStyle};
use leptos::prelude::*;
use std::rc::Rc;

/// One framework-owned action outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionFeedbackState {
    /// Work is currently in flight.
    Pending,
    /// The action completed successfully.
    Success,
    /// The authoritative record changed and the action may be retried.
    RecoverableConflict,
    /// The addressed row is stale and needs reconciliation.
    StaleRow,
    /// Some, but not all, requested work completed.
    PartialSuccess,
    /// Transport or service failure that may be retried.
    RetryableFailure,
    /// Terminal failure for which retry is not offered.
    TerminalFailure,
}

impl ActionFeedbackState {
    /// Whether the renderer may expose a retry intent for this state.
    pub const fn can_retry(self) -> bool {
        matches!(
            self,
            Self::RecoverableConflict
                | Self::StaleRow
                | Self::PartialSuccess
                | Self::RetryableFailure
        )
    }

    /// Whether this completed outcome may be dismissed.
    pub const fn can_dismiss(self) -> bool {
        !matches!(self, Self::Pending)
    }
}

/// Current state for one stable action key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionFeedbackEntry {
    state: ActionFeedbackState,
}

impl ActionFeedbackEntry {
    /// Returns the action's current outcome.
    pub const fn state(&self) -> ActionFeedbackState {
        self.state
    }

    /// Whether this entry may expose Retry.
    pub const fn can_retry(&self) -> bool {
        self.state.can_retry()
    }

    /// Whether this entry may expose Dismiss.
    pub const fn can_dismiss(&self) -> bool {
        self.state.can_dismiss()
    }
}

/// The single most recent transition announced to assistive technology.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionAnnouncement<K> {
    sequence: u64,
    key: K,
    state: ActionFeedbackState,
}

impl<K> ActionAnnouncement<K> {
    /// Monotonic transition sequence.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Stable action key that changed.
    pub const fn key(&self) -> &K {
        &self.key
    }

    /// Outcome produced by the transition.
    pub const fn state(&self) -> ActionFeedbackState {
        self.state
    }
}

/// Failure to record an action transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionTransitionError {
    /// The monotonic announcement sequence cannot advance without wrapping.
    SequenceExhausted,
    /// Pending is established only by minting a framework action handle.
    PendingRequiresStart,
}

/// Private-field keyed collection that permits unrelated actions to remain
/// pending concurrently while exposing exactly one latest announcement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionFeedbackModel<K> {
    entries: Vec<(K, ActionFeedbackEntry)>,
    latest_announcement: Option<ActionAnnouncement<K>>,
    next_sequence: u64,
}

impl<K> Default for ActionFeedbackModel<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> ActionFeedbackModel<K> {
    /// Creates an empty action collection.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            latest_announcement: None,
            next_sequence: 0,
        }
    }

    /// Number of currently rendered keyed outcomes.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no keyed outcome is present.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Stable insertion-order view of every keyed entry.
    pub fn entries(&self) -> impl Iterator<Item = (&K, &ActionFeedbackEntry)> {
        self.entries.iter().map(|(key, entry)| (key, entry))
    }

    /// Removes all action bindings, for example when access generation changes.
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.latest_announcement = None;
    }

    #[cfg(test)]
    pub(super) fn force_next_sequence_for_test(&mut self, sequence: u64) {
        self.next_sequence = sequence;
    }
}

impl<K: Eq> ActionFeedbackModel<K> {
    /// Reads one keyed outcome without changing any other action.
    pub fn entry(&self, key: &K) -> Option<&ActionFeedbackEntry> {
        self.entries
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, entry)| entry)
    }

    /// Removes a completed outcome. Pending work cannot be dismissed into a
    /// misleading idle presentation.
    pub fn dismiss(&mut self, key: &K) -> bool {
        let Some(index) = self
            .entries
            .iter()
            .position(|(candidate, entry)| candidate == key && entry.can_dismiss())
        else {
            return false;
        };
        let clears_announcement = self
            .latest_announcement
            .as_ref()
            .is_some_and(|announcement| announcement.key() == key);
        self.entries.remove(index);
        if clears_announcement {
            self.latest_announcement = None;
        }
        true
    }
}

impl<K: Clone + Eq> ActionFeedbackModel<K> {
    /// Replaces only `key` and records it as the latest transition.
    pub fn set(
        &mut self,
        key: K,
        state: ActionFeedbackState,
    ) -> Result<u64, ActionTransitionError> {
        let sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(ActionTransitionError::SequenceExhausted)?;
        self.next_sequence = sequence;

        if let Some((_, entry)) = self
            .entries
            .iter_mut()
            .find(|(candidate, _)| candidate == &key)
        {
            entry.state = state;
        } else {
            self.entries
                .push((key.clone(), ActionFeedbackEntry { state }));
        }
        self.latest_announcement = Some(ActionAnnouncement {
            sequence,
            key,
            state,
        });
        Ok(sequence)
    }

    /// The only transition emitted through the live-region renderer.
    pub fn latest_announcement(&self) -> Option<&ActionAnnouncement<K>> {
        self.latest_announcement.as_ref()
    }
}

/// Complete localizable copy owned by [`ActionFeedback`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionFeedbackTexts {
    /// Pending outcome label.
    pub pending: String,
    /// Successful outcome label.
    pub success: String,
    /// Recoverable-conflict label.
    pub recoverable_conflict: String,
    /// Stale-row reconciliation label.
    pub stale_row: String,
    /// Partial-success label.
    pub partial_success: String,
    /// Retryable-failure label.
    pub retryable_failure: String,
    /// Terminal-failure label.
    pub terminal_failure: String,
    /// Retry action label.
    pub retry: String,
    /// Dismiss action label.
    pub dismiss: String,
}

impl Default for ActionFeedbackTexts {
    fn default() -> Self {
        Self {
            pending: "Action in progress.".to_owned(),
            success: "Action completed.".to_owned(),
            recoverable_conflict: "The record changed; review and retry.".to_owned(),
            stale_row: "The row is stale and needs reconciliation.".to_owned(),
            partial_success: "The action completed only partially.".to_owned(),
            retryable_failure: "The action failed and may be retried.".to_owned(),
            terminal_failure: "The action could not be completed.".to_owned(),
            retry: "Retry".to_owned(),
            dismiss: "Dismiss".to_owned(),
        }
    }
}

fn state_slug(state: ActionFeedbackState) -> &'static str {
    match state {
        ActionFeedbackState::Pending => "pending",
        ActionFeedbackState::Success => "success",
        ActionFeedbackState::RecoverableConflict => "recoverable-conflict",
        ActionFeedbackState::StaleRow => "stale-row",
        ActionFeedbackState::PartialSuccess => "partial-success",
        ActionFeedbackState::RetryableFailure => "retryable-failure",
        ActionFeedbackState::TerminalFailure => "terminal-failure",
    }
}

fn state_text(texts: &ActionFeedbackTexts, state: ActionFeedbackState) -> String {
    match state {
        ActionFeedbackState::Pending => texts.pending.clone(),
        ActionFeedbackState::Success => texts.success.clone(),
        ActionFeedbackState::RecoverableConflict => texts.recoverable_conflict.clone(),
        ActionFeedbackState::StaleRow => texts.stale_row.clone(),
        ActionFeedbackState::PartialSuccess => texts.partial_success.clone(),
        ActionFeedbackState::RetryableFailure => texts.retryable_failure.clone(),
        ActionFeedbackState::TerminalFailure => texts.terminal_failure.clone(),
    }
}

fn keyed_state_text(
    texts: &ActionFeedbackTexts,
    key_label: &str,
    state: ActionFeedbackState,
) -> String {
    format!("{key_label}: {}", state_text(texts, state))
}

fn state_class(state: ActionFeedbackState) -> &'static str {
    match state {
        ActionFeedbackState::Pending => "alert-info",
        ActionFeedbackState::Success => "alert-success",
        ActionFeedbackState::RecoverableConflict
        | ActionFeedbackState::StaleRow
        | ActionFeedbackState::PartialSuccess => "alert-warning",
        ActionFeedbackState::RetryableFailure | ActionFeedbackState::TerminalFailure => {
            "alert-error"
        }
    }
}

/// Renders all keyed outcomes without competing live regions. Only the latest
/// transition is mirrored into the single polite announcement node.
#[component]
pub fn ActionFeedback<K>(
    /// Reactive keyed outcomes.
    model: Signal<ActionFeedbackModel<K>, LocalStorage>,
    /// Reactive complete framework-owned copy.
    #[prop(into, default = Signal::stored(ActionFeedbackTexts::default()))]
    texts: Signal<ActionFeedbackTexts>,
    /// Stable human-readable label for one action key.
    key_label: Rc<dyn Fn(&K) -> String>,
    /// Optional retry intent; the consumer still owns transport and completion.
    #[prop(optional)]
    on_retry: Option<Callback<K>>,
    /// Optional dismissal intent; the consumer decides when to update state.
    #[prop(optional)]
    on_dismiss: Option<Callback<K>>,
) -> impl IntoView
where
    K: Clone + Eq + Send + Sync + 'static,
{
    let key_label = StoredValue::new_local(key_label);
    view! {
        <section class="space-y-2" data-action-feedback="true">
            {move || {
                let texts = texts.get();
                model.with(|model| {
                    model.entries().map(|(key, entry)| {
                        let key = key.clone();
                        let state = entry.state();
                        let label = key_label.with_value(|label| label(&key));
                        let message = keyed_state_text(&texts, &label, state);
                        let retry_key = key.clone();
                        let dismiss_key = key.clone();
                        let retry_text = texts.retry.clone();
                        let dismiss_text = texts.dismiss.clone();
                        view! {
                            <div
                                class=format!("alert alert-soft {}", state_class(state))
                                role="group"
                                data-action-feedback-key=label
                                data-action-feedback-state=state_slug(state)
                            >
                                {matches!(state, ActionFeedbackState::Pending).then(|| view! {
                                    <span class="loading loading-spinner loading-sm" aria-hidden="true"></span>
                                })}
                                <p class="min-w-0 flex-1">{message}</p>
                                {entry.can_retry().then(|| on_retry.map(|callback| view! {
                                    <Button
                                        style=ButtonStyle::Outline
                                        size=ButtonSize::Sm
                                        on_click=Callback::new(move |_| callback.run(retry_key.clone()))
                                    >
                                        {retry_text}
                                    </Button>
                                }))}
                                {entry.can_dismiss().then(|| on_dismiss.map(|callback| view! {
                                    <Button
                                        style=ButtonStyle::Ghost
                                        size=ButtonSize::Sm
                                        on_click=Callback::new(move |_| callback.run(dismiss_key.clone()))
                                    >
                                        {dismiss_text}
                                    </Button>
                                }))}
                            </div>
                        }
                    }).collect_view()
                })
            }}
            <p class="sr-only" aria-live="polite" aria-atomic="true" data-action-announcement="true">
                {move || model.with(|model| {
                    model.latest_announcement().map(|announcement| {
                        let label = key_label.with_value(|label| label(announcement.key()));
                        texts.with(|texts| {
                            keyed_state_text(texts, &label, announcement.state())
                        })
                    }).unwrap_or_default()
                })}
            </p>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_outcome_has_copy_slug_and_visual_treatment() {
        let texts = ActionFeedbackTexts::default();
        for state in [
            ActionFeedbackState::Pending,
            ActionFeedbackState::Success,
            ActionFeedbackState::RecoverableConflict,
            ActionFeedbackState::StaleRow,
            ActionFeedbackState::PartialSuccess,
            ActionFeedbackState::RetryableFailure,
            ActionFeedbackState::TerminalFailure,
        ] {
            assert!(!state_slug(state).is_empty());
            assert!(!state_text(&texts, state).trim().is_empty());
            assert!(keyed_state_text(&texts, "row-1", state).starts_with("row-1: "));
            assert!(state_class(state).starts_with("alert-"));
        }
    }
}
