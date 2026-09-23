use super::*;
use crate::components::*;

fn ready() -> ClientCallWorkspaceState {
    ClientCallWorkspaceState {
        call: SoftphoneState {
            context_id: "actor/client/attempt-1".into(),
            client: SoftphoneClient {
                name: "Elena".into(),
                phones: vec![SoftphoneNumber {
                    id: "phone".into(),
                    label: "Phone".into(),
                    number: "+14155550142".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        },
        destination: "+14155550142".into(),
        dial_blocked_reason: None,
        ..Default::default()
    }
}

#[test]
fn ambiguous_or_accepted_bridge_attempt_never_permits_redial() {
    let mut state = ready();
    let dial = ClientCallAction::Dial {
        number: state.destination.clone(),
    };
    assert!(state.can_dispatch(&dial));
    for attempt in [
        ClientCallAttempt::Submitting,
        ClientCallAttempt::AgentRinging,
        ClientCallAttempt::Uncertain,
        ClientCallAttempt::Finished,
    ] {
        state.attempt = attempt;
        assert!(!state.can_dispatch(&dial), "{attempt:?}");
        assert!(!state.can_dispatch(&ClientCallAction::EditDestination("123".into())));
    }
    state.attempt = ClientCallAttempt::Refused;
    assert!(state.can_dispatch(&dial));
    state.dial_blocked_reason = Some("Permission revoked".into());
    assert!(!state.can_dispatch(&dial));
}

#[test]
fn contact_update_is_explicit_scoped_and_requires_a_changed_valid_draft() {
    let mut state = ready();
    state.number_update = Some(ClientCallNumberUpdate {
        field_id: "phone".into(),
        label: "Contact phone".into(),
        saved_number: state.destination.clone(),
        ..Default::default()
    });
    let save = |s: &ClientCallWorkspaceState| ClientCallAction::SaveNumber {
        field_id: "phone".into(),
        number: s.destination.clone(),
    };
    assert!(!state.can_dispatch(&save(&state)));
    state.destination = "+14155550186".into();
    assert!(state.can_dispatch(&save(&state)));
    assert!(!state.can_dispatch(&ClientCallAction::SaveNumber {
        field_id: "another-contact".into(),
        number: state.destination.clone()
    }));
    state.number_error = Some("Not callable".into());
    assert!(!state.can_dispatch(&save(&state)));
    state.number_error = None;
    state.number_update.as_mut().unwrap().pending = true;
    assert!(!state.can_dispatch(&save(&state)));
    assert!(!state.can_dispatch(&ClientCallAction::Dial {
        number: state.destination.clone()
    }));
}

#[test]
fn wrap_up_does_not_invent_an_outcome_duration_or_callback() {
    let mut state = ready();
    state.attempt = ClientCallAttempt::Finished;
    let wrap = state.wrap_up.as_mut().unwrap();
    assert!(wrap.payload().is_none());
    wrap.outcome = Some(ClientCallOutcome::RequestedCallBack);
    assert!(wrap.payload().is_none());
    wrap.follow_up = "Tomorrow after 2 pm Pacific; assigned consultant".into();
    let payload = wrap.payload().unwrap();
    assert_eq!(payload.duration_minutes, None);
    assert_eq!(payload.notes, "");
    for invalid in ["0", "-1", "1.5", "4294967296"] {
        state.wrap_up.as_mut().unwrap().duration_minutes = invalid.into();
        assert!(state.wrap_up.as_ref().unwrap().payload().is_none());
    }
    state.wrap_up.as_mut().unwrap().duration_minutes = "12".into();
    let payload = state.wrap_up.as_ref().unwrap().payload().unwrap();
    assert!(state.can_dispatch(&ClientCallAction::SaveWrapUp(payload.clone())));
    state.attempt = ClientCallAttempt::Uncertain;
    assert!(!state.can_dispatch(&ClientCallAction::SaveWrapUp(payload)));
}

#[test]
fn managed_call_retains_end_during_pending_control_and_never_bypasses_dial_guard() {
    let mut state = ready();
    state.attempt = ClientCallAttempt::Managed;
    state.call.phase = SoftphonePhase::Active;
    state.call.pending = Some(SoftphoneActionKind::Record);
    let end_call = ClientCallAction::Session(SoftphoneAction::EndCall);
    let end_command = ClientCallCommand {
        context_id: state.call.context_id.clone(),
        action: end_call.clone(),
    };
    assert!(!state.can_dispatch(&end_call));
    assert!(!state.accepts(&end_command));

    state.call.capabilities.end_call = true;
    assert!(state.can_dispatch(&end_call));
    assert!(state.accepts(&end_command));
    assert!(!state.can_dispatch(&ClientCallAction::Session(SoftphoneAction::SetMuted(true))));
    state.call.phase = SoftphonePhase::Ended;
    state.call.pending = None;
    assert!(
        !state.can_dispatch(&ClientCallAction::Session(SoftphoneAction::Call {
            phone_id: "phone".into()
        }))
    );
    state.call.context_id.clear();
    assert!(!state.can_dispatch(&ClientCallAction::Dismiss));
}

#[test]
fn command_envelope_rejects_stale_context_and_wrap_up_snapshot() {
    let mut state = ready();
    let command = ClientCallCommand {
        context_id: state.call.context_id.clone(),
        action: ClientCallAction::Dial {
            number: state.destination.clone(),
        },
    };
    assert!(state.accepts(&command));
    state.call.context_id = "actor/other-client/attempt-2".into();
    assert!(!state.accepts(&command));
    state.attempt = ClientCallAttempt::Finished;
    state.wrap_up.as_mut().unwrap().outcome = Some(ClientCallOutcome::Interested);
    let payload = state.wrap_up.as_ref().unwrap().payload().unwrap();
    state.wrap_up.as_mut().unwrap().notes = "New note".into();
    assert!(!state.can_dispatch(&ClientCallAction::SaveWrapUp(payload)));
}

#[test]
fn structured_refusals_allow_retry_only_with_current_readiness() {
    let mut state = ready();
    for reason in [
        ClientCallRefusal::AgentNotReady,
        ClientCallRefusal::AgentNotAvailable,
        ClientCallRefusal::NotSubmitted,
    ] {
        state.attempt = ClientCallAttempt::RefusedWith(reason);
        let dial = ClientCallAction::Dial {
            number: state.destination.clone(),
        };
        assert!(state.can_dispatch(&dial));
        state.dial_blocked_reason = Some("Not ready".into());
        assert!(!state.can_dispatch(&dial));
        state.dial_blocked_reason = None;
    }
    state.attempt = ClientCallAttempt::Uncertain;
    assert!(!state.can_edit_destination());
}

#[test]
fn blocked_saved_numbers_cannot_be_chosen_or_dialed_but_remain_writable() {
    let mut state = ready();
    state.call.client.phones[0].blocked_reason = Some("Restricted number".into());
    assert!(!state.can_dispatch(&ClientCallAction::ChooseSavedNumber("phone".into())));
    state.destination = " +14155550142 ".into();
    assert!(!state.can_dispatch(&ClientCallAction::Dial {
        number: state.destination.clone()
    }));
    state.number_update = Some(ClientCallNumberUpdate {
        field_id: "mobile".into(),
        ..Default::default()
    });
    assert!(state.can_dispatch(&ClientCallAction::SaveNumber {
        field_id: "mobile".into(),
        number: state.destination.clone()
    }));
    state.destination = "+14155550186".into();
    assert!(state.can_dispatch(&ClientCallAction::Dial {
        number: state.destination.clone()
    }));
}

fn target(id: &str) -> ClientCallNumberTarget {
    ClientCallNumberTarget {
        field_id: id.into(),
        label: id.into(),
        ..Default::default()
    }
}

#[test]
fn contact_targets_require_unique_identity_and_exact_host_selected_payload() {
    let mut state = ready();
    state.number_update = Some(ClientCallNumberUpdate {
        field_id: "phone".into(),
        targets: vec![target("phone"), target("mobile")],
        ..Default::default()
    });
    let mobile = ClientCallAction::ChooseNumberTarget("mobile".into());
    assert!(state.can_dispatch(&mobile));
    assert_eq!(
        state
            .number_update
            .as_ref()
            .unwrap()
            .selected_target()
            .unwrap()
            .field_id,
        "phone"
    );
    let save_mobile = ClientCallAction::SaveNumber {
        field_id: "mobile".into(),
        number: state.destination.clone(),
    };
    assert!(!state.can_dispatch(&save_mobile));
    state.number_update.as_mut().unwrap().field_id = "mobile".into();
    assert!(state.can_dispatch(&save_mobile));
    state.number_update.as_mut().unwrap().pending = true;
    assert!(!state.can_dispatch(&mobile));
    assert!(!state.can_dispatch(&save_mobile));
    state.number_update.as_mut().unwrap().pending = false;
    state
        .number_update
        .as_mut()
        .unwrap()
        .targets
        .push(target("mobile"));
    assert!(
        state
            .number_update
            .as_ref()
            .unwrap()
            .selected_target()
            .is_none()
    );
    assert!(!state.can_dispatch(&mobile));
    assert!(!state.can_dispatch(&save_mobile));
    state.number_update.as_mut().unwrap().field_id = " ".into();
    state.number_update.as_mut().unwrap().targets = vec![target(" ")];
    assert!(
        state
            .number_update
            .as_ref()
            .unwrap()
            .selected_target()
            .is_none()
    );
    assert!(!state.can_dispatch(&ClientCallAction::ChooseNumberTarget(" ".into())));
}

#[test]
fn blocked_targets_and_unchanged_values_reject_contact_writes() {
    let mut state = ready();
    state.number_update = Some(ClientCallNumberUpdate {
        field_id: "mobile".into(),
        targets: vec![ClientCallNumberTarget {
            saved_number: state.destination.clone(),
            ..target("mobile")
        }],
        ..Default::default()
    });
    let save = ClientCallAction::SaveNumber {
        field_id: "mobile".into(),
        number: state.destination.clone(),
    };
    assert!(!state.can_dispatch(&save));
    state.number_update.as_mut().unwrap().targets[0]
        .saved_number
        .clear();
    assert!(state.can_dispatch(&save));
    state.number_update.as_mut().unwrap().targets[0].blocked_reason =
        Some("No field permission".into());
    assert!(!state.can_dispatch(&save));
    assert!(!state.can_dispatch(&ClientCallAction::ChooseNumberTarget("mobile".into())));
}

#[test]
fn regeneration_requires_capability_context_and_terminal_or_idle_state() {
    let mut state = ready();
    let action = ClientCallAction::RegenerateGuidance;
    assert!(!state.can_dispatch(&action));
    state.guidance = Some(ClientCallGuidance::default());
    assert!(!state.can_dispatch(&action));
    state.guidance.as_mut().unwrap().regeneration = Some(ClientCallRegeneration::default());
    for (status, allowed) in [
        (ClientCallRegenerationState::Idle, true),
        (ClientCallRegenerationState::Busy, false),
        (ClientCallRegenerationState::Accepted, false),
        (ClientCallRegenerationState::Succeeded, true),
        (ClientCallRegenerationState::Failed, true),
    ] {
        state
            .guidance
            .as_mut()
            .unwrap()
            .regeneration
            .as_mut()
            .unwrap()
            .state = status;
        let before = state.clone();
        assert_eq!(state.can_dispatch(&action), allowed);
        assert_eq!(state, before);
    }
    state
        .guidance
        .as_mut()
        .unwrap()
        .regeneration
        .as_mut()
        .unwrap()
        .blocked_reason = Some("Unavailable".into());
    assert!(!state.can_dispatch(&action));
    state
        .guidance
        .as_mut()
        .unwrap()
        .regeneration
        .as_mut()
        .unwrap()
        .blocked_reason = None;
    state.call.context_id.clear();
    assert!(!state.can_dispatch(&action));
}

#[test]
fn provider_talk_evidence_does_not_fill_manual_duration_or_confirm_completion() {
    let mut state = ready();
    state.call.timer = SoftphoneTimer::Stopped { seconds: 240 };
    state.wrap_up.as_mut().unwrap().outcome = Some(ClientCallOutcome::Interested);
    for evidence in [None, Some(0), Some(42)] {
        state.provider_talk_seconds = evidence;
        assert_eq!(
            state
                .wrap_up
                .as_ref()
                .unwrap()
                .payload()
                .unwrap()
                .duration_minutes,
            None
        );
        assert!(!state.finished());
    }
}

#[test]
fn regeneration_stays_independent_of_call_and_write_state_but_rejects_stale_context() {
    let mut state = ready();
    state.guidance = Some(ClientCallGuidance {
        regeneration: Some(ClientCallRegeneration::default()),
        ..Default::default()
    });
    state.attempt = ClientCallAttempt::AgentRinging;
    state.number_update = Some(ClientCallNumberUpdate {
        pending: true,
        ..Default::default()
    });
    state.wrap_up.as_mut().unwrap().pending = true;
    let command = ClientCallCommand {
        context_id: state.call.context_id.clone(),
        action: ClientCallAction::RegenerateGuidance,
    };
    assert!(state.accepts(&command));
    state.call.context_id = "actor/other-client/attempt-2".into();
    assert!(!state.accepts(&command));
}

#[test]
fn localized_structured_statuses_preserve_host_distinctions() {
    let texts = ClientCallWorkspaceTexts {
        refusal_labels: ["not-ready", "unavailable", "not-submitted"].map(String::from),
        regeneration_labels: ["idle", "sending", "accepted", "done", "failed"].map(String::from),
        ..Default::default()
    };
    for (refusal, label, identifier) in [
        (
            ClientCallRefusal::AgentNotReady,
            "not-ready",
            "agent-not-ready",
        ),
        (
            ClientCallRefusal::AgentNotAvailable,
            "unavailable",
            "agent-not-available",
        ),
        (
            ClientCallRefusal::NotSubmitted,
            "not-submitted",
            "not-submitted",
        ),
    ] {
        let attempt = ClientCallAttempt::RefusedWith(refusal);
        assert_eq!(texts.attempt(attempt), label);
        assert_eq!(attempt.as_str(), identifier);
    }
    for (status, label) in [
        (ClientCallRegenerationState::Idle, "idle"),
        (ClientCallRegenerationState::Busy, "sending"),
        (ClientCallRegenerationState::Accepted, "accepted"),
        (ClientCallRegenerationState::Succeeded, "done"),
        (ClientCallRegenerationState::Failed, "failed"),
    ] {
        assert_eq!(texts.regeneration(status), label);
    }
}

// Office op-stjm9: every control the workspace renders announces a name, and
// every disabled one a reason. The render path reads these same functions for
// `aria-label`, `disabled` and the `aria-describedby` reason line.
fn assert_named_and_explained(state: &ClientCallWorkspaceState, case: &str) -> usize {
    let texts = ClientCallWorkspaceTexts::default();
    let mut disabled = 0;
    for control in ClientCallControl::inventory(state) {
        let name = texts.control_name(&control, state);
        assert!(
            !name.trim().is_empty(),
            "{case}: {control:?} has no accessible name"
        );
        let reason = texts.disabled_reason(&control, state);
        if state.control_enabled(&control) {
            assert_eq!(
                reason, None,
                "{case}: enabled {control:?} must not claim a reason"
            );
        } else {
            disabled += 1;
            assert!(
                reason.as_deref().is_some_and(|r| !r.trim().is_empty()),
                "{case}: disabled {control:?} has no reason"
            );
        }
    }
    disabled
}

fn not_ready() -> ClientCallWorkspaceState {
    let mut state = ready();
    state.call.context_id = String::new();
    state
}

#[test]
fn every_control_is_named_and_every_disabled_control_says_why() {
    let mut cases: Vec<(&str, ClientCallWorkspaceState)> = vec![
        ("default", ClientCallWorkspaceState::default()),
        ("not-ready", not_ready()),
        ("ready", ready()),
    ];
    let mut ringing = ready();
    ringing.attempt = ClientCallAttempt::AgentRinging;
    cases.push(("ringing", ringing));
    let mut finished = ready();
    finished.attempt = ClientCallAttempt::Finished;
    cases.push(("finished-empty-draft", finished.clone()));
    let mut saved = finished.clone();
    saved.wrap_up.as_mut().unwrap().saved = true;
    cases.push(("saved", saved));
    let mut pending = finished.clone();
    pending.wrap_up.as_mut().unwrap().pending = true;
    cases.push(("pending", pending));
    let mut callback = ready();
    callback.wrap_up.as_mut().unwrap().outcome = Some(ClientCallOutcome::RequestedCallBack);
    cases.push(("callback-draft", callback));
    let mut blocked = ready();
    blocked.destination = String::new();
    blocked.call.client.phones[0].blocked_reason = Some("Do not call".into());
    blocked.number_update = Some(ClientCallNumberUpdate {
        field_id: "phone".into(),
        targets: vec![target("phone"), target("mobile")],
        blocked_reason: Some("Read-only contact".into()),
        ..Default::default()
    });
    blocked.guidance = Some(ClientCallGuidance {
        regeneration: Some(ClientCallRegeneration {
            state: ClientCallRegenerationState::Busy,
            ..Default::default()
        }),
        ..Default::default()
    });
    cases.push(("blocked", blocked));
    let mut full = ready();
    full.destination = "1".repeat(64);
    cases.push(("full", full));
    for (case, state) in &cases {
        assert_named_and_explained(state, case);
    }
}

#[test]
fn a_missing_call_context_disables_all_nine_production_controls_with_one_reason() {
    let texts = ClientCallWorkspaceTexts::default();
    let state = not_ready();
    let visible: Vec<_> = ClientCallControl::inventory(&state)
        .into_iter()
        .filter(|c| {
            !matches!(
                c,
                ClientCallControl::Digit(_) | ClientCallControl::Backspace
            )
        })
        .collect();
    assert_eq!(visible.len(), 9, "{visible:?}");
    for control in &visible {
        assert!(!state.control_enabled(control), "{control:?}");
        assert_eq!(
            texts.disabled_reason(control, &state),
            Some(texts.not_ready.clone()),
            "{control:?}"
        );
    }
}

#[test]
fn names_match_visible_labels_and_digits_name_themselves() {
    let texts = ClientCallWorkspaceTexts::default();
    let state = ready();
    assert_eq!(
        texts.control_name(&ClientCallControl::Dial, &state),
        texts.call
    );
    assert_eq!(
        texts.control_name(&ClientCallControl::Dismiss, &state),
        texts.close
    );
    assert_eq!(
        texts.control_name(&ClientCallControl::Outcome, &state),
        texts.outcome
    );
    assert_eq!(
        texts.control_name(&ClientCallControl::SavedNumber("phone".into()), &state),
        "Phone · +14155550142"
    );
    for digit in ClientCallControl::DIGITS {
        assert_eq!(
            texts.control_name(&ClientCallControl::Digit(digit), &state),
            digit.to_string()
        );
    }
}

#[test]
fn disabled_reasons_prefer_the_hosts_explanation_then_the_lock() {
    let texts = ClientCallWorkspaceTexts::default();
    let mut state = ready();
    state.dial_blocked_reason = Some("Set yourself Available".into());
    assert_eq!(
        texts
            .disabled_reason(&ClientCallControl::Dial, &state)
            .as_deref(),
        Some("Set yourself Available")
    );
    state.attempt = ClientCallAttempt::AgentRinging;
    assert_eq!(
        texts.disabled_reason(&ClientCallControl::Destination, &state),
        Some(texts.destination_locked.clone())
    );
    let mut state = ready();
    assert_eq!(
        texts.disabled_reason(&ClientCallControl::SaveWrapUp, &state),
        Some(texts.finish_hint.clone())
    );
    state.attempt = ClientCallAttempt::Finished;
    assert_eq!(
        texts.disabled_reason(&ClientCallControl::SaveWrapUp, &state),
        Some(texts.wrap_up_incomplete.clone())
    );
    state.wrap_up.as_mut().unwrap().saved = true;
    assert_eq!(
        texts.disabled_reason(&ClientCallControl::Notes, &state),
        Some(texts.wrap_up_locked.clone())
    );
}

// ldui-tyyn: the inventory is CLOSED. `inventoried` matches every variant
// with no wildcard, so adding a variant to `ClientCallControl` fails to
// compile until it is added here, and this test then fails until it is also
// returned by `ClientCallControl::inventory` for the maximal state below.
fn inventoried(control: &ClientCallControl) {
    match control {
        ClientCallControl::Dismiss
        | ClientCallControl::Destination
        | ClientCallControl::SavedNumber(_)
        | ClientCallControl::Keypad
        | ClientCallControl::Digit(_)
        | ClientCallControl::Backspace
        | ClientCallControl::NumberTarget
        | ClientCallControl::SaveNumber
        | ClientCallControl::Dial
        | ClientCallControl::Regenerate
        | ClientCallControl::Outcome
        | ClientCallControl::Notes
        | ClientCallControl::Duration
        | ClientCallControl::FollowUp
        | ClientCallControl::SaveWrapUp => {}
    }
}

/// A state in which every control the workspace can render is rendered.
fn maximal() -> ClientCallWorkspaceState {
    let mut state = ready();
    state.number_update = Some(ClientCallNumberUpdate {
        field_id: "phone".into(),
        targets: vec![target("phone"), target("mobile")],
        ..Default::default()
    });
    state.guidance = Some(ClientCallGuidance {
        regeneration: Some(ClientCallRegeneration::default()),
        ..Default::default()
    });
    state.wrap_up.get_or_insert_with(Default::default).outcome =
        Some(ClientCallOutcome::RequestedCallBack);
    state
}

#[test]
fn inventory_is_closed_and_lists_every_control_in_document_order() {
    let state = maximal();
    let inventory = ClientCallControl::inventory(&state);
    let mut expected = vec![
        ClientCallControl::Dismiss,
        ClientCallControl::Destination,
        ClientCallControl::SavedNumber("phone".into()),
        ClientCallControl::Keypad,
    ];
    expected.extend(
        ClientCallControl::DIGITS
            .into_iter()
            .map(ClientCallControl::Digit),
    );
    expected.extend([
        ClientCallControl::Backspace,
        ClientCallControl::NumberTarget,
        ClientCallControl::SaveNumber,
        ClientCallControl::Dial,
        ClientCallControl::Regenerate,
        ClientCallControl::Outcome,
        ClientCallControl::Notes,
        ClientCallControl::Duration,
        ClientCallControl::FollowUp,
        ClientCallControl::SaveWrapUp,
    ]);
    assert_eq!(inventory, expected);
    for control in &inventory {
        inventoried(control);
    }
    // Every rendered control is named, and each disabled one explained.
    let _disabled = assert_named_and_explained(&state, "maximal");
}

#[test]
fn inventory_drops_controls_the_state_does_not_render() {
    let mut state = maximal();
    state.number_update = None;
    state.guidance = None;
    state.wrap_up = None;
    let inventory = ClientCallControl::inventory(&state);
    for absent in [
        ClientCallControl::NumberTarget,
        ClientCallControl::SaveNumber,
        ClientCallControl::Regenerate,
        ClientCallControl::Outcome,
        ClientCallControl::Notes,
        ClientCallControl::Duration,
        ClientCallControl::FollowUp,
        ClientCallControl::SaveWrapUp,
    ] {
        assert!(!inventory.contains(&absent), "{absent:?}");
    }
    let mut state = maximal();
    state.number_update.as_mut().unwrap().targets.clear();
    let inventory = ClientCallControl::inventory(&state);
    assert!(!inventory.contains(&ClientCallControl::NumberTarget));
    assert!(inventory.contains(&ClientCallControl::SaveNumber));
    let mut state = maximal();
    state.wrap_up.as_mut().unwrap().outcome = Some(ClientCallOutcome::Interested);
    assert!(!ClientCallControl::inventory(&state).contains(&ClientCallControl::FollowUp));
}

#[test]
fn default_not_ready_text() {
    assert_eq!(
        ClientCallWorkspaceTexts::default().not_ready,
        "Calling is not available for this client right now."
    );
}

#[test]
fn default_destination_locked_text() {
    assert_eq!(
        ClientCallWorkspaceTexts::default().destination_locked,
        "The number cannot change while a call is in progress or a record is saving."
    );
}

#[test]
fn default_destination_full_text() {
    assert_eq!(
        ClientCallWorkspaceTexts::default().destination_full,
        "The number has reached its 64-character limit."
    );
}

#[test]
fn default_needs_number_text() {
    assert_eq!(
        ClientCallWorkspaceTexts::default().needs_number,
        "Enter or choose a number first."
    );
}

#[test]
fn default_save_number_hint_text() {
    assert_eq!(
        ClientCallWorkspaceTexts::default().save_number_hint,
        "Choose a contact field and enter a different number to save it."
    );
}

#[test]
fn default_wrap_up_locked_text() {
    assert_eq!(
        ClientCallWorkspaceTexts::default().wrap_up_locked,
        "This call record is saved and can no longer be edited."
    );
}

#[test]
fn default_wrap_up_incomplete_text() {
    assert_eq!(
        ClientCallWorkspaceTexts::default().wrap_up_incomplete,
        "Choose an outcome (and callback instructions for a callback) to save."
    );
}
