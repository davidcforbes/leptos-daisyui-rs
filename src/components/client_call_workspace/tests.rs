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
    assert_named_and_explained_in(state, ClientCallLayout::Full, case)
}

fn assert_named_and_explained_in(
    state: &ClientCallWorkspaceState,
    layout: ClientCallLayout,
    case: &str,
) -> usize {
    let texts = ClientCallWorkspaceTexts::default();
    let mut disabled = 0;
    for control in ClientCallControl::inventory_for(state, layout) {
        let name = texts.control_name_for(&control, state, layout);
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

/// ldui-eray (Office op-1yxvd): with no call context every button's own line
/// stays empty -- the shared context line says why -- EXCEPT the number pad's,
/// which sits apart under the destination field and states `not_ready` itself.
#[test]
fn a_missing_context_leaves_own_lines_empty_except_the_number_pad() {
    use super::component::own_line_reason;
    let texts = ClientCallWorkspaceTexts::default();
    let state = not_ready();
    assert_eq!(
        own_line_reason(&ClientCallControl::Keypad, &state, &texts),
        Some(texts.not_ready.clone())
    );
    for control in [
        ClientCallControl::Dial,
        ClientCallControl::SaveNumber,
        ClientCallControl::Dismiss,
    ] {
        assert_eq!(
            own_line_reason(&control, &state, &texts),
            None,
            "{control:?}"
        );
    }
    // With a context, the pad's line is its ordinary reason (or none).
    assert_eq!(
        own_line_reason(&ClientCallControl::Keypad, &ready(), &texts),
        texts.disabled_reason(&ClientCallControl::Keypad, &ready())
    );
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
        | ClientCallControl::StartAnotherCall
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

// Office op-flpq1: LAUNCHER MODE. When the host's own phone does the dialling
// (a separate softphone window), the panel offers the saved numbers and Place
// call only - no typed number, no number pad, no contact write.
fn launcher() -> ClientCallWorkspaceState {
    let mut state = ready();
    state.destination_entry = ClientCallDestinationEntry::SavedOnly;
    state
}

#[test]
fn launcher_mode_renders_saved_numbers_and_the_launch_button_only() {
    let mut state = launcher();
    state.number_update = Some(ClientCallNumberUpdate {
        field_id: "phone".into(),
        targets: vec![target("phone"), target("mobile")],
        ..Default::default()
    });
    state.wrap_up = None;
    assert_eq!(
        ClientCallControl::inventory(&state),
        vec![
            ClientCallControl::Dismiss,
            ClientCallControl::SavedNumber("phone".into()),
            ClientCallControl::Dial,
        ]
    );
    // Every control it does render is still named, and each disabled one explained.
    assert_named_and_explained(&state, "launcher");
    let mut not_ready = state.clone();
    not_ready.call.context_id = String::new();
    assert_named_and_explained(&not_ready, "launcher-not-ready");
    // Typed mode keeps the original panel for the same state.
    state.destination_entry = ClientCallDestinationEntry::Typed;
    let typed = ClientCallControl::inventory(&state);
    for present in [
        ClientCallControl::Destination,
        ClientCallControl::Keypad,
        ClientCallControl::Backspace,
        ClientCallControl::NumberTarget,
        ClientCallControl::SaveNumber,
    ] {
        assert!(typed.contains(&present), "{present:?}");
    }
}

#[test]
fn launcher_mode_refuses_typed_numbers_and_contact_writes() {
    let mut state = launcher();
    state.number_update = Some(ClientCallNumberUpdate {
        field_id: "phone".into(),
        targets: vec![target("phone"), target("mobile")],
        ..Default::default()
    });
    assert!(!state.can_dispatch(&ClientCallAction::EditDestination("+1415".into())));
    assert!(!state.can_dispatch(&ClientCallAction::ChooseNumberTarget("mobile".into())));
    assert!(!state.can_dispatch(&ClientCallAction::SaveNumber {
        field_id: "phone".into(),
        number: state.destination.clone(),
    }));
    assert!(state.can_dispatch(&ClientCallAction::ChooseSavedNumber("phone".into())));
    assert!(state.can_dispatch(&ClientCallAction::Dial {
        number: state.destination.clone(),
    }));
    // A destination that is not one of the saved numbers is never dialled.
    state.destination = "+14155550199".into();
    assert!(!state.can_dispatch(&ClientCallAction::Dial {
        number: state.destination.clone(),
    }));
    // The same draft is a valid typed number in the original panel.
    state.destination_entry = ClientCallDestinationEntry::Typed;
    assert!(state.can_dispatch(&ClientCallAction::Dial {
        number: state.destination.clone(),
    }));
}

#[test]
fn launcher_mode_marks_the_chosen_number_and_asks_for_a_saved_one() {
    let texts = ClientCallWorkspaceTexts::default();
    let mut state = launcher();
    assert!(state.saved_only());
    assert!(state.is_selected_number("phone"));
    assert!(!state.is_selected_number("other"));
    state.destination = String::new();
    assert!(!state.is_selected_number("phone"));
    assert_eq!(
        texts.disabled_reason(&ClientCallControl::Dial, &state),
        Some(texts.needs_saved_number.clone())
    );
    state.destination_entry = ClientCallDestinationEntry::Typed;
    assert!(!state.saved_only());
    assert_eq!(
        texts.disabled_reason(&ClientCallControl::Dial, &state),
        Some(texts.needs_number.clone())
    );
}

#[test]
fn history_is_display_only_and_labels_each_direction() {
    let texts = ClientCallWorkspaceTexts::default();
    assert_eq!(texts.direction(ClientCallDirection::Outbound), "Outgoing");
    assert_eq!(texts.direction(ClientCallDirection::Inbound), "Incoming");
    assert_eq!(ClientCallDirection::Inbound.as_str(), "inbound");
    assert_eq!(ClientCallDestinationEntry::SavedOnly.as_str(), "saved-only");
    let state = ClientCallWorkspaceState::default();
    assert_eq!(state.history, None, "absent history hides the section");
    assert_eq!(
        state.destination_entry,
        ClientCallDestinationEntry::Typed,
        "the original panel stays the default"
    );
    let mut with_history = ready();
    with_history.history = Some(ClientCallHistory {
        entries: vec![ClientCallHistoryEntry {
            id: "4525415226".into(),
            when: "Oct 2, 7:11 AM".into(),
            who: "Chris".into(),
            direction: ClientCallDirection::Outbound,
            outcome: "No answer".into(),
            talk_seconds: Some(0),
        }],
        ..Default::default()
    });
    assert_eq!(
        ClientCallControl::inventory(&with_history),
        ClientCallControl::inventory(&ready()),
        "history adds no control"
    );
}

#[test]
fn default_launcher_and_history_texts() {
    let texts = ClientCallWorkspaceTexts::default();
    assert_eq!(texts.needs_saved_number, "Choose a saved number first.");
    assert_eq!(
        texts.no_saved_numbers,
        "No saved numbers. Add a number to the contact before calling."
    );
    assert_eq!(
        texts.destination_saved_hint,
        "Choose one of the client's saved numbers."
    );
    assert_eq!(texts.history, "Recent calls");
    assert_eq!(texts.history_empty, "No calls yet.");
    assert_eq!(texts.history_talk_time, "Talk time");
}

// ldui-eq1e (Office op-flpq1): the COMPACT layout says each thing once. The
// owner measured `destination_locked` rendered six times in a 339 px rail.
fn compact_launcher() -> ClientCallWorkspaceState {
    let mut state = launcher();
    state.call.client.phones.push(SoftphoneNumber {
        id: "mobile".into(),
        label: "Mobile".into(),
        number: "+14155550199".into(),
        blocked_reason: Some("Mobile calling is restricted.".into()),
    });
    state
}

/// Every line the compact numbers block can show, in render order: the
/// shared context line, each number's own line, the shared lock line and the
/// call button's line.
fn compact_lines(state: &ClientCallWorkspaceState) -> Vec<String> {
    let texts = ClientCallWorkspaceTexts::default();
    let mut lines = Vec::new();
    if state.context_missing() {
        lines.push(texts.not_ready.clone());
    }
    for phone in &state.call.client.phones {
        lines.extend(texts.compact_number_line(state, &phone.id));
    }
    lines.extend(texts.compact_lock_line(state));
    lines.extend(texts.compact_call_line(state));
    lines
}

fn locked_states() -> Vec<(&'static str, ClientCallWorkspaceState)> {
    let mut cases = Vec::new();
    for attempt in [
        ClientCallAttempt::Submitting,
        ClientCallAttempt::AgentRinging,
        ClientCallAttempt::Uncertain,
        ClientCallAttempt::Finished,
    ] {
        let mut state = compact_launcher();
        state.attempt = attempt;
        cases.push((attempt.as_str(), state));
    }
    let mut saving = compact_launcher();
    saving.attempt = ClientCallAttempt::Finished;
    saving.wrap_up.as_mut().unwrap().outcome = Some(ClientCallOutcome::Interested);
    saving.wrap_up.as_mut().unwrap().pending = true;
    cases.push(("record-saving", saving));
    let mut blocked = compact_launcher();
    blocked.attempt = ClientCallAttempt::AgentRinging;
    blocked.dial_blocked_reason = Some("Set yourself Available".into());
    cases.push(("ringing-and-dial-blocked", blocked));
    cases
}

#[test]
fn compact_locked_numbers_show_exactly_one_lock_line() {
    let texts = ClientCallWorkspaceTexts::default();
    for (case, state) in locked_states() {
        assert!(state.saved_numbers_locked(), "{case}");
        let lines = compact_lines(&state);
        let locks = lines
            .iter()
            .filter(|line| {
                **line == texts.numbers_locked || **line == texts.numbers_locked_after_call
            })
            .count();
        assert_eq!(locks, 1, "{case}: {lines:?}");
        // After a finished call the line names the next step instead of a
        // call that is over; a pending save still locks "during a call".
        let expected = if case == "finished" {
            &texts.numbers_locked_after_call
        } else {
            &texts.numbers_locked
        };
        assert_eq!(
            texts.compact_lock_line(&state).as_ref(),
            Some(expected),
            "{case}"
        );
        assert!(
            !lines.contains(&texts.destination_locked),
            "{case}: the full lock sentence must not appear: {lines:?}"
        );
        // The blocked number still says why it is blocked, inline, once.
        assert_eq!(
            lines
                .iter()
                .filter(|line| *line == "Mobile calling is restricted.")
                .count(),
            1,
            "{case}: {lines:?}"
        );
        // A disabled launch whose cause is the lock says the lock line's words.
        if ClientCallControl::compact_call(&state) == ClientCallControl::Dial {
            assert_eq!(
                texts.compact_call_reason(&state),
                Some(texts.numbers_locked.clone()),
                "{case}"
            );
        }
    }
    // The full layout's own per-control reason is still the full sentence.
    let (_, ringing) = &locked_states()[1];
    assert_eq!(
        texts.disabled_reason(&ClientCallControl::Dial, ringing),
        Some(texts.destination_locked.clone())
    );
}

#[test]
fn compact_unlocked_numbers_show_no_lock_line_and_other_reasons_once() {
    let texts = ClientCallWorkspaceTexts::default();
    let state = compact_launcher();
    assert!(!state.saved_numbers_locked());
    assert_eq!(texts.compact_lock_line(&state), None);
    assert_eq!(texts.compact_call_line(&state), None, "enabled launch");
    assert_eq!(texts.compact_call_reason(&state), None);
    // A host dial block shows once, under the button.
    let mut blocked = state.clone();
    blocked.dial_blocked_reason = Some("Set yourself Available".into());
    assert_eq!(
        texts.compact_call_line(&blocked).as_deref(),
        Some("Set yourself Available")
    );
    assert_eq!(
        compact_lines(&blocked)
            .iter()
            .filter(|line| *line == "Set yourself Available")
            .count(),
        1
    );
    // No number chosen yet.
    let mut unchosen = state.clone();
    unchosen.destination = String::new();
    assert_eq!(
        texts.compact_call_line(&unchosen),
        Some(texts.needs_saved_number.clone())
    );
    // No context: the shared context line says why; nothing else repeats it.
    let mut missing = state;
    missing.call.context_id = String::new();
    assert!(!missing.saved_numbers_locked());
    assert_eq!(texts.compact_lock_line(&missing), None);
    assert_eq!(texts.compact_call_line(&missing), None);
    assert_eq!(
        texts.compact_call_reason(&missing),
        Some(texts.not_ready.clone())
    );
    assert_eq!(
        compact_lines(&missing)
            .iter()
            .filter(|line| **line == texts.not_ready)
            .count(),
        1
    );
}

#[test]
fn compact_status_line_composition() {
    let texts = ClientCallWorkspaceTexts::default();
    let mut state = compact_launcher();
    assert_eq!(
        texts.compact_status(&state),
        None,
        "hidden before any attempt"
    );
    state.status_detail = "  ".into();
    assert_eq!(
        texts.compact_status(&state),
        None,
        "blank detail says nothing"
    );
    // Ready with something to explain shows it.
    state.status_detail = "Calling opens the Office phone.".into();
    assert_eq!(
        texts.compact_status(&state).as_deref(),
        Some("Calling opens the Office phone.")
    );
    // The host's detail beats the attempt label.
    state.attempt = ClientCallAttempt::AgentRinging;
    state.status_detail = "Dialing +1 415 555 0142 from the Office phone".into();
    assert_eq!(
        texts.compact_status(&state).as_deref(),
        Some("Dialing +1 415 555 0142 from the Office phone")
    );
    state.status_detail.clear();
    assert_eq!(
        texts.compact_status(&state).as_deref(),
        Some("Ringing your phone"),
        "no bridge hint in compact"
    );
    // Finished: the attempt label, the final duration, then the talk time.
    state.attempt = ClientCallAttempt::Finished;
    state.call.phase = SoftphonePhase::Ended;
    state.call.timer = SoftphoneTimer::Stopped { seconds: 4 };
    state.provider_talk_seconds = Some(0);
    assert_eq!(
        texts.compact_status(&state).as_deref(),
        Some("Call finished · 00:04 · Talk time: 00:00")
    );
    // Unknown talk time is never zero.
    state.provider_talk_seconds = None;
    assert_eq!(
        texts.compact_status(&state).as_deref(),
        Some("Call finished · 00:04 · Talk time: Not confirmed")
    );
    // Uncertain keeps its safety warning as a second line.
    let mut uncertain = compact_launcher();
    uncertain.attempt = ClientCallAttempt::Uncertain;
    uncertain.status_detail = "The provider response was lost.".into();
    assert_eq!(
        texts.compact_status(&uncertain).as_deref(),
        Some("The provider response was lost.")
    );
    assert_eq!(
        texts.compact_uncertain_hint(&uncertain),
        Some(texts.uncertain_hint.clone())
    );
    assert_eq!(texts.compact_uncertain_hint(&state), None);
    assert_eq!(uncertain.compact_status_tone(), "status status-warning");
}

#[test]
fn compact_call_button_reads_call_again_and_starts_another_call_after_finish() {
    let texts = ClientCallWorkspaceTexts::default();
    let mut state = compact_launcher();
    assert_eq!(
        ClientCallControl::compact_call(&state),
        ClientCallControl::Dial
    );
    assert_eq!(texts.compact_call_label(&state), texts.call);
    // Refused: the same Dial, relabelled.
    for attempt in [
        ClientCallAttempt::Refused,
        ClientCallAttempt::RefusedWith(ClientCallRefusal::AgentNotReady),
    ] {
        state.attempt = attempt;
        assert_eq!(
            ClientCallControl::compact_call(&state),
            ClientCallControl::Dial
        );
        assert_eq!(texts.compact_call_label(&state), texts.call_again);
        assert_eq!(
            texts.control_name_for(&ClientCallControl::Dial, &state, ClientCallLayout::Compact),
            texts.call_again
        );
        // The full layout keeps its label.
        assert_eq!(
            texts.control_name_for(&ClientCallControl::Dial, &state, ClientCallLayout::Full),
            texts.call
        );
        assert!(state.can_dispatch(&ClientCallAction::Dial {
            number: state.destination.clone()
        }));
    }
    // Finished: Call again proposes a new attempt; Dial stays refused.
    state.attempt = ClientCallAttempt::Finished;
    assert_eq!(
        ClientCallControl::compact_call(&state),
        ClientCallControl::StartAnotherCall
    );
    assert_eq!(texts.compact_call_label(&state), texts.call_again);
    assert!(state.can_dispatch(&ClientCallAction::StartAnotherCall));
    assert!(!state.can_dispatch(&ClientCallAction::Dial {
        number: state.destination.clone()
    }));
    assert_eq!(
        texts.compact_call_reason(&state),
        None,
        "Call again is operable"
    );
    let inventory = ClientCallControl::inventory_for(&state, ClientCallLayout::Compact);
    assert!(inventory.contains(&ClientCallControl::StartAnotherCall));
    assert!(!inventory.contains(&ClientCallControl::Dial));
    assert!(
        ClientCallControl::inventory_for(&state, ClientCallLayout::Full)
            .contains(&ClientCallControl::Dial),
        "the full layout is unchanged"
    );
    // A record still saving holds the new attempt back, and says why.
    let mut saving = state.clone();
    saving.wrap_up.as_mut().unwrap().outcome = Some(ClientCallOutcome::Interested);
    saving.wrap_up.as_mut().unwrap().pending = true;
    assert!(!saving.can_dispatch(&ClientCallAction::StartAnotherCall));
    assert_eq!(
        texts.compact_call_reason(&saving),
        Some(texts.saving.clone())
    );
    // A managed call that ended counts as finished.
    let mut managed = compact_launcher();
    managed.attempt = ClientCallAttempt::Managed;
    managed.call.phase = SoftphonePhase::Ended;
    assert!(managed.can_dispatch(&ClientCallAction::StartAnotherCall));
}

#[test]
fn start_another_call_is_never_offered_before_completion() {
    let mut state = compact_launcher();
    for attempt in [
        ClientCallAttempt::Ready,
        ClientCallAttempt::Submitting,
        ClientCallAttempt::AgentRinging,
        ClientCallAttempt::Uncertain,
        ClientCallAttempt::Refused,
        ClientCallAttempt::Managed,
    ] {
        state.attempt = attempt;
        assert!(
            !state.can_dispatch(&ClientCallAction::StartAnotherCall),
            "{attempt:?}"
        );
    }
    state.attempt = ClientCallAttempt::Finished;
    state.call.context_id = String::new();
    assert!(!state.can_dispatch(&ClientCallAction::StartAnotherCall));
    // A typed panel in the compact layout keeps the ordinary launch.
    let mut typed = ready();
    typed.attempt = ClientCallAttempt::Finished;
    assert_eq!(
        ClientCallControl::compact_call(&typed),
        ClientCallControl::Dial
    );
}

#[test]
fn compact_inventory_is_named_and_explained() {
    let mut cases = vec![("compact-ready", compact_launcher())];
    cases.extend(locked_states());
    let mut missing = compact_launcher();
    missing.call.context_id = String::new();
    cases.push(("compact-not-ready", missing));
    let mut refused = compact_launcher();
    refused.attempt = ClientCallAttempt::Refused;
    cases.push(("compact-refused", refused));
    let mut guided = compact_launcher();
    guided.guidance = Some(ClientCallGuidance {
        regeneration: Some(ClientCallRegeneration::default()),
        ..Default::default()
    });
    cases.push(("compact-guided", guided));
    for (case, state) in &cases {
        assert_named_and_explained_in(state, ClientCallLayout::Compact, case);
        let inventory = ClientCallControl::inventory_for(state, ClientCallLayout::Compact);
        for control in &inventory {
            inventoried(control);
        }
    }
    // The launcher renders Dismiss, each saved number, then the call button.
    let mut finished = compact_launcher();
    finished.attempt = ClientCallAttempt::Finished;
    finished.wrap_up = None;
    assert_eq!(
        ClientCallControl::inventory_for(&finished, ClientCallLayout::Compact),
        vec![
            ClientCallControl::Dismiss,
            ClientCallControl::SavedNumber("phone".into()),
            ClientCallControl::SavedNumber("mobile".into()),
            ClientCallControl::StartAnotherCall,
        ]
    );
}

#[test]
fn compact_history_shows_three_then_toggles() {
    let texts = ClientCallWorkspaceTexts::default();
    assert_eq!(COMPACT_HISTORY_VISIBLE, 3);
    assert_eq!(texts.history_toggle(false, 8), "Show all (8)");
    assert_eq!(texts.history_toggle(true, 8), "Show fewer");
    let spanish = ClientCallWorkspaceTexts {
        history_show_all: "Ver todas ({n})".into(),
        ..Default::default()
    };
    assert_eq!(spanish.history_toggle(false, 12), "Ver todas (12)");
}

#[test]
fn compact_script_summary_names_the_disclosure() {
    let texts = ClientCallWorkspaceTexts::default();
    let mut guidance = ClientCallGuidance {
        title: "Payment call script".into(),
        ..Default::default()
    };
    assert_eq!(
        texts.compact_script_summary(&guidance),
        "Payment call script"
    );
    guidance.state = ClientCallGuidanceState::Preparing {
        elapsed_secs: 3,
        typical_secs: 20,
    };
    assert_eq!(
        texts.compact_script_summary(&guidance),
        "Payment call script · Preparing script"
    );
    guidance.title = " ".into();
    guidance.state = ClientCallGuidanceState::Ready;
    assert_eq!(texts.compact_script_summary(&guidance), "Call script");
}

#[test]
fn default_compact_texts_and_layout() {
    let texts = ClientCallWorkspaceTexts::default();
    assert_eq!(texts.numbers_locked, "Numbers lock during a call.");
    assert_eq!(
        texts.numbers_locked_after_call,
        "Press Call again to choose a number."
    );
    assert_eq!(texts.call_again, "Call again");
    assert_eq!(texts.launch_hint, "", "host text; blank hides the line");
    assert_eq!(texts.compact_launch_hint(), None);
    assert_eq!(texts.history_show_all, "Show all ({n})");
    assert_eq!(texts.history_show_fewer, "Show fewer");
    assert_eq!(texts.script, "Call script");
    assert_eq!(ClientCallLayout::default(), ClientCallLayout::Full);
    assert_eq!(ClientCallLayout::Compact.as_str(), "compact");
    let hinted = ClientCallWorkspaceTexts {
        launch_hint: "Opens in the Office phone window.".into(),
        ..Default::default()
    };
    assert_eq!(
        hinted.compact_launch_hint().as_deref(),
        Some("Opens in the Office phone window.")
    );
}
