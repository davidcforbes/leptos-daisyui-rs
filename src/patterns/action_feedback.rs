//! Keyed action feedback for snapshot-table pages.
//!
//! The pure model (`ActionFeedbackState`, `ActionFeedbackContent`,
//! `ActionFeedbackEntry`, `ActionAnnouncement`, `ActionTransitionError`,
//! `ActionFeedbackModel`) lives in the shared `snapshot-core` crate — it is
//! also the type `snapshot_table::SnapshotTableState` binds internally, so
//! keeping a second local definition here would make the two disagree. This
//! module re-exports that model and keeps only what is renderer-coupled:
//! the localizable copy, the presentation-mapping helpers, and the
//! `ActionFeedback` Leptos component itself.

use crate::components::{Button, ButtonSize, ButtonStyle};
use leptos::prelude::*;
use std::rc::Rc;

pub use snapshot_core::{
    ActionAnnouncement, ActionFeedbackContent, ActionFeedbackEntry, ActionFeedbackModel,
    ActionFeedbackState, ActionTransitionError,
};

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

/// Composes one entry's rendered sentence: the caller-supplied primary text
/// when present, else the localized framework default; the caller-supplied
/// detail, when present and non-empty, is appended after it. Both pieces are
/// plain strings destined for a text node, never `inner_html`.
fn content_text(
    texts: &ActionFeedbackTexts,
    state: ActionFeedbackState,
    content: &ActionFeedbackContent,
) -> String {
    let primary = match content.primary.as_deref() {
        Some(primary) if !primary.is_empty() => primary.to_owned(),
        _ => state_text(texts, state),
    };
    match content.detail.as_deref() {
        Some(detail) if !detail.is_empty() => format!("{primary} {detail}"),
        _ => primary,
    }
}

fn keyed_entry_text(
    texts: &ActionFeedbackTexts,
    key_label: &str,
    state: ActionFeedbackState,
    content: &ActionFeedbackContent,
) -> String {
    format!("{key_label}: {}", content_text(texts, state, content))
}

#[cfg(test)]
fn keyed_state_text(
    texts: &ActionFeedbackTexts,
    key_label: &str,
    state: ActionFeedbackState,
) -> String {
    keyed_entry_text(texts, key_label, state, &ActionFeedbackContent::default())
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
                        let message = keyed_entry_text(&texts, &label, state, entry.content());
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
                            keyed_entry_text(texts, &label, announcement.state(), announcement.content())
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

    #[test]
    fn missing_content_falls_back_to_the_localized_default() {
        let texts = ActionFeedbackTexts::default();
        let content = ActionFeedbackContent::default();
        assert!(content.is_empty());
        assert_eq!(
            content_text(&texts, ActionFeedbackState::RetryableFailure, &content),
            texts.retryable_failure
        );
    }

    #[test]
    fn caller_primary_overrides_default_and_detail_is_appended() {
        let texts = ActionFeedbackTexts::default();

        // Detail only: default primary, appended detail (partial-success count).
        let partial = ActionFeedbackContent {
            primary: None,
            detail: Some("3 of 5 items updated.".to_owned()),
        };
        assert_eq!(
            content_text(&texts, ActionFeedbackState::PartialSuccess, &partial),
            format!("{} 3 of 5 items updated.", texts.partial_success)
        );

        // Primary override with no detail (conflict reason as the whole sentence).
        let conflict = ActionFeedbackContent {
            primary: Some("Another editor changed this record 2 minutes ago.".to_owned()),
            detail: None,
        };
        assert_eq!(
            content_text(&texts, ActionFeedbackState::RecoverableConflict, &conflict),
            "Another editor changed this record 2 minutes ago."
        );

        // Empty-string detail is treated as absent, not a trailing space.
        let empty_detail = ActionFeedbackContent {
            primary: None,
            detail: Some(String::new()),
        };
        assert_eq!(
            content_text(&texts, ActionFeedbackState::Success, &empty_detail),
            texts.success
        );

        // Empty-string primary is treated as absent, not an empty sentence:
        // it falls back to the localized default exactly like an empty-string
        // detail falls back to no appended detail.
        let empty_primary = ActionFeedbackContent {
            primary: Some(String::new()),
            detail: None,
        };
        assert_eq!(
            content_text(&texts, ActionFeedbackState::Success, &empty_primary),
            texts.success
        );

        // An empty-string primary combined with a real detail still falls
        // back to the localized default primary, with the detail appended.
        let empty_primary_with_detail = ActionFeedbackContent {
            primary: Some(String::new()),
            detail: Some("3 of 5 items updated.".to_owned()),
        };
        assert_eq!(
            content_text(
                &texts,
                ActionFeedbackState::PartialSuccess,
                &empty_primary_with_detail
            ),
            format!("{} 3 of 5 items updated.", texts.partial_success)
        );
    }

    #[test]
    fn content_never_reaches_a_raw_html_sink() {
        // Attempt-specific text is caller-controlled (a transport error message,
        // say) and must render as a literal text node. This crate has no
        // `inner_html`/`dangerously_set_inner_html` path for ActionFeedback, so
        // the composed sentence is guaranteed to be plain text; assert the
        // composition itself does not strip or re-interpret markup-like input,
        // which would only be a risk if something downstream ever did parse it.
        let texts = ActionFeedbackTexts::default();
        let hostile = ActionFeedbackContent {
            primary: Some("<script>alert(1)</script>".to_owned()),
            detail: Some("<b>bold</b> & \"quoted\"".to_owned()),
        };
        let message = content_text(&texts, ActionFeedbackState::TerminalFailure, &hostile);
        assert_eq!(
            message,
            "<script>alert(1)</script> <b>bold</b> & \"quoted\""
        );
    }

    // `set_with_content_replaces_rather_than_merges_prior_content` and
    // `concurrent_keys_retain_independent_content_and_the_announcement_matches_latest`
    // exercised only the pure `ActionFeedbackModel` now re-exported from
    // `snapshot-core`; that crate's own test suite covers both (ldui-gzmf).
}
