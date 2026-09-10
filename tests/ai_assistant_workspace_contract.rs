//! Public-boundary proof for controlled assistant requests (synthetic data only).
//!
//! These tests are authored ahead of the remaining Phase 1 types/dispatcher.
//! They join the native gate once that foundation is implemented; no browser,
//! provider, network, credentials or Office records are involved.

use leptos_daisyui_rs::components::ai_assistant_workspace::*;

fn reason() -> AssistantReason {
    AssistantReason {
        code: "unavailable".into(),
        message: "The host has not confirmed this operation".into(),
    }
}

fn ask_grant() -> AssistantCapabilities {
    AssistantCapabilities {
        granted: vec![AssistantCapability::Ask],
        details: vec![],
    }
}

fn ready() -> AssistantWorkspaceState {
    AssistantWorkspaceState {
        access: AssistantAccess::Granted,
        context: Some(AssistantContext {
            id: "page-1".into(),
            revision: 2,
            epoch: 3,
            actor: AssistantPerson {
                id: "actor-a".into(),
                label: "Actor A".into(),
            },
            subject: Some(AssistantPerson {
                id: "subject-b".into(),
                label: "Subject B".into(),
            }),
            scope: AssistantScope {
                id: "scope-1".into(),
                revision: 4,
                label: "These offices".into(),
                basis: ScopeBasis::PagePopulation {
                    population_id: "offices-1".into(),
                    population_revision: 5,
                    description: "Complete displayed population".into(),
                },
            },
            data_revision: 6,
            permission_revision: 7,
            policy_generation: 8,
            as_of: "10 September".into(),
            freshness: "Current snapshot".into(),
            capabilities: ask_grant(),
        }),
        now_ms: 1_000,
        next_request_id: Some("request-1".into()),
        conversation: AssistantLoad::Ready(AssistantConversation {
            id: "conversation-1".into(),
            revision: 9,
            context_id: "page-1".into(),
            scope_id: "scope-1".into(),
            closed: false,
            continuity: ConversationContinuity::Session,
            capabilities: ask_grant(),
            draft: AssistantDraft {
                id: "draft-1".into(),
                context_id: "page-1".into(),
                context_revision: 2,
                text: "Review these rows".into(),
                revision: 10,
                max_chars: 1000,
            },
            suggestions: vec![],
            submission: SubmissionDisposition::Idle,
            attempts: vec![],
        }),
        settings: AssistantLoad::Ready(AssistantSettings {
            owner_actor_id: "actor-a".into(),
            accepted_revision: 11,
            proposed_revision: 12,
            accepted: AssistantPreferences {
                engine_id: Some("engine-1".into()),
                memory_use: false,
                memory_capture: false,
            },
            proposed: AssistantPreferences {
                engine_id: Some("engine-1".into()),
                memory_use: false,
                memory_capture: false,
            },
            reasoning_tier: AssistantAccess::Granted,
            engines: vec![AssistantEngine {
                id: "engine-1".into(),
                revision: 13,
                label: "Host-configured engine".into(),
                availability: EngineAvailability::Enabled,
                capabilities: ask_grant(),
                connection: AssistantConnection {
                    state: ConnectionState::SignedIn,
                    shape: SignInShape::Device,
                    device: None,
                    account_label: Some("Synthetic operator".into()),
                    verified_at: Some("10 September".into()),
                    expires_at: None,
                    reason: None,
                },
            }],
            budget: None,
            capabilities: ask_grant(),
        }),
        ..Default::default()
    }
}

fn conversation(state: &mut AssistantWorkspaceState) -> &mut AssistantConversation {
    match &mut state.conversation {
        AssistantLoad::Ready(conversation) => conversation,
        _ => panic!("fixture conversation must be ready"),
    }
}

fn ask(state: &AssistantWorkspaceState) -> AssistantAction {
    AssistantAction {
        context: state.context.as_ref().unwrap().stamp(),
        target: AssistantIntentTarget::Conversation {
            conversation: AssistantItemRevision {
                id: "conversation-1".into(),
                revision: 9,
            },
            draft: AssistantItemRevision {
                id: "draft-1".into(),
                revision: 10,
            },
        },
        kind: AssistantActionKind::Ask,
    }
}

#[test]
fn asking_emits_the_exact_accepted_scope_and_draft_without_mutating_state() {
    let state = ready();
    let before = state.clone();
    assert_eq!(validate_workspace(&state), Ok(()));
    let command = command_for(&state, ask(&state)).expect("the explicit Ask is eligible");
    assert!(accepts_command(&state, &command));
    let request = command.request.as_ref().expect("Ask has host allocation");
    assert_eq!(request.id, "request-1");
    assert_eq!(request.context.actor_id, "actor-a");
    assert_eq!(request.context.data_revision, 6);
    assert_eq!(request.context.permission_revision, 7);
    assert_eq!(request.context.policy_generation, 8);
    assert_eq!(request.target, command.action.target);
    match command.payload {
        AssistantCommandPayload::Ask {
            question,
            selected_engine_id,
            scope,
        } => {
            assert_eq!(question, "Review these rows");
            assert_eq!(selected_engine_id, "engine-1");
            assert_eq!(scope.id, "scope-1");
            assert_eq!(scope.revision, 4);
            assert!(
                matches!(scope.basis, ScopeBasis::PagePopulation { population_id, .. } if population_id == "offices-1")
            );
        }
        other => panic!("Ask must carry its bounded typed payload, got {other:?}"),
    }
    assert_eq!(
        state, before,
        "building an intent neither admits work nor clears a draft"
    );
}

#[test]
fn asking_requires_both_page_and_conversation_grants() {
    let mut state = ready();
    let action = ask(&state);
    assert!(can_dispatch(&state, &action));
    conversation(&mut state).capabilities.granted.clear();
    assert!(!can_dispatch(&state, &action));
    conversation(&mut state).capabilities = ask_grant();
    state.context.as_mut().unwrap().capabilities.granted.clear();
    assert!(!can_dispatch(&state, &action));
}

#[test]
fn current_request_allocation_is_required_and_uncertain_admission_never_reasks() {
    let mut state = ready();
    let action = ask(&state);
    assert!(command_for(&state, action.clone()).is_some());
    state.next_request_id = None;
    assert!(command_for(&state, action.clone()).is_none());
    state.next_request_id = Some("another-allocation".into());
    conversation(&mut state).submission = SubmissionDisposition::Uncertain {
        request_id: "original-request".into(),
        reason: reason(),
    };
    assert!(command_for(&state, action.clone()).is_none());
    assert_eq!(conversation(&mut state).draft.text, "Review these rows");
    conversation(&mut state).submission = SubmissionDisposition::Submitting {
        request_id: "original-request".into(),
    };
    assert!(command_for(&state, action).is_none());
}

#[test]
fn question_bounds_count_rust_scalars_not_bytes_or_utf16_units() {
    let mut state = ready();
    let action = ask(&state);
    conversation(&mut state).draft.max_chars = 2;
    conversation(&mut state).draft.text = "🦀é".into();
    assert!(
        can_dispatch(&state, &action),
        "two Unicode scalar values fit the host bound"
    );
    conversation(&mut state).draft.text.push('x');
    assert!(!can_dispatch(&state, &action));
    conversation(&mut state).draft.text = " \n".into();
    assert!(!can_dispatch(&state, &action));
}

#[test]
fn stale_stamps_and_draft_bindings_are_rejected_without_clearing_text() {
    let baseline = ready();
    let action = ask(&baseline);
    for mutation in 0..9 {
        let mut state = baseline.clone();
        let context = state.context.as_mut().unwrap();
        match mutation {
            0 => context.id = "page-2".into(),
            1 => context.revision += 1,
            2 => context.epoch += 1,
            3 => context.actor.id = "actor-c".into(),
            4 => context.scope.id = "scope-2".into(),
            5 => context.scope.revision += 1,
            6 => context.data_revision += 1,
            7 => context.permission_revision += 1,
            8 => context.policy_generation += 1,
            _ => unreachable!(),
        }
        assert!(
            !can_dispatch(&state, &action),
            "stale stamp field {mutation} dispatched"
        );
        assert_eq!(conversation(&mut state).draft.text, "Review these rows");
    }
    let mut state = baseline;
    conversation(&mut state).draft.context_revision = 1;
    assert!(
        !can_dispatch(&state, &ask(&state)),
        "new header does not silently rebind a draft"
    );
}

#[test]
fn optional_service_errors_do_not_disable_an_authorized_question() {
    let mut state = ready();
    state.memory = AssistantLoad::ContractError(AssistantContractError::UnknownState);
    state.learning = AssistantLoad::Unavailable { reason: reason() };
    assert!(can_dispatch(&state, &ask(&state)));
    state.access = AssistantAccess::Denied { reason: reason() };
    assert!(!can_dispatch(&state, &ask(&state)));
}

#[test]
fn hand_built_commands_cannot_smuggle_different_scope_text_or_identity() {
    let state = ready();
    let command = command_for(&state, ask(&state)).unwrap();
    let mut altered = command.clone();
    altered.request.as_mut().unwrap().context.actor_id = "subject-b".into();
    assert!(!accepts_command(&state, &altered));
    altered = command.clone();
    altered.request.as_mut().unwrap().id = "unallocated-request".into();
    assert!(!accepts_command(&state, &altered));
    altered = command.clone();
    if let AssistantCommandPayload::Ask { question, .. } = &mut altered.payload {
        *question = "Different unsupplied question".into();
    }
    assert!(!accepts_command(&state, &altered));
    altered = command;
    if let AssistantCommandPayload::Ask { scope, .. } = &mut altered.payload {
        scope.id = "all-offices".into();
    }
    assert!(!accepts_command(&state, &altered));
}

#[test]
fn pre_admission_refusals_have_distinct_next_actions_and_unknown_is_not_retryable() {
    assert_eq!(
        RefusalReason::EvidenceAdmissionUnavailable.next_action(),
        Some(RefusalNextAction::RetryLater)
    );
    assert_eq!(
        RefusalReason::AssistantPreferenceUnavailable.next_action(),
        Some(RefusalNextAction::OpenSettings)
    );
    assert_eq!(
        RefusalReason::ConversationClosed.next_action(),
        Some(RefusalNextAction::NewConversation)
    );
    assert_eq!(
        RefusalReason::Unknown("future-refusal".into()).next_action(),
        None
    );
}
