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
    assert!(state.can_dispatch(&ClientCallAction::Session(SoftphoneAction::EndCall)));
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
