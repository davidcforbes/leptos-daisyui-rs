use super::*;

fn reason() -> AssistantReason {
    AssistantReason {
        code: "host-policy".into(),
        message: "Not available for this context".into(),
    }
}

fn context() -> AssistantContext {
    AssistantContext {
        id: "page-1".into(),
        revision: 2,
        epoch: 3,
        actor: AssistantPerson {
            id: "operator-a".into(),
            label: "Operator A".into(),
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
        as_of: "10 September".into(),
        freshness: "Current snapshot".into(),
        policy_generation: 8,
        capabilities: AssistantCapabilities {
            granted: vec![AssistantCapability::Ask],
            details: vec![],
        },
    }
}

#[test]
fn capability_details_restrict_but_never_create_a_grant() {
    let mut capabilities = AssistantCapabilities::default();
    assert!(!capabilities.allows(&AssistantCapability::Ask));
    capabilities.details.push(AssistantCapabilityDetail {
        capability: AssistantCapability::Ask,
        state: AssistantCapabilityState::Granted,
    });
    assert!(
        !capabilities.allows(&AssistantCapability::Ask),
        "a detail cannot invent an absent grant"
    );
    capabilities.granted.push(AssistantCapability::Ask);
    assert!(capabilities.allows(&AssistantCapability::Ask));
    for restriction in [
        AssistantCapabilityState::Denied { reason: reason() },
        AssistantCapabilityState::Unavailable { reason: reason() },
        AssistantCapabilityState::PolicyNotConfigured { reason: reason() },
        AssistantCapabilityState::Busy { reason: reason() },
        AssistantCapabilityState::Unknown("future-policy".into()),
    ] {
        capabilities.details[0].state = restriction;
        assert!(!capabilities.allows(&AssistantCapability::Ask));
    }
}

#[test]
fn duplicate_or_unknown_capabilities_cannot_be_interpreted_as_permission() {
    let mut capabilities = context().capabilities;
    assert_eq!(capabilities.validate(), Ok(()));
    capabilities.granted.push(AssistantCapability::Ask);
    assert!(capabilities.validate().is_err());
    assert!(!capabilities.allows(&AssistantCapability::Ask));
    capabilities.granted.pop();
    let detail = AssistantCapabilityDetail {
        capability: AssistantCapability::Ask,
        state: AssistantCapabilityState::Granted,
    };
    capabilities.details = vec![detail.clone(), detail];
    assert!(capabilities.validate().is_err());
    assert!(!capabilities.allows(&AssistantCapability::Ask));
    capabilities.details.clear();
    capabilities
        .granted
        .push(AssistantCapability::Unknown("future-grant".into()));
    assert!(capabilities.validate().is_err());
    assert!(!capabilities.allows(&AssistantCapability::Unknown("future-grant".into())));
}

#[test]
fn unresolved_defaults_do_not_claim_loaded_empty_or_eligible() {
    assert!(matches!(
        AssistantAccess::default(),
        AssistantAccess::Unresolved { .. }
    ));
    assert!(matches!(
        AssistantLoad::<Vec<String>>::default(),
        AssistantLoad::Unavailable { .. }
    ));
    assert!(matches!(
        AssistantEligibility::default(),
        AssistantEligibility::Ineligible { .. }
    ));
    assert!(AssistantCapabilities::default().granted.is_empty());
}

#[test]
fn context_requires_explicit_actor_and_population_authority() {
    let accepted = context();
    assert_eq!(validate_context(&accepted), Ok(()));
    let mut invalid = accepted.clone();
    invalid.actor.id.clear();
    assert_eq!(
        validate_context(&invalid),
        Err(AssistantContractError::MissingIdentity)
    );
    invalid = accepted.clone();
    invalid.scope.basis = ScopeBasis::Expanded {
        authorization_id: " ".into(),
        authorization_revision: 0,
        description: "All offices".into(),
    };
    assert_eq!(
        validate_context(&invalid),
        Err(AssistantContractError::MissingIdentity)
    );
    invalid.scope.basis = ScopeBasis::Unknown("implicit-from-question".into());
    assert_eq!(
        validate_context(&invalid),
        Err(AssistantContractError::UnknownState)
    );
    invalid.scope.basis = ScopeBasis::Expanded {
        authorization_id: "permission-9".into(),
        authorization_revision: 0,
        description: "Explicitly authorized all offices".into(),
    };
    assert_eq!(validate_context(&invalid), Ok(()));
}

#[test]
fn action_stamp_keeps_actor_and_every_authority_generation() {
    assert_eq!(
        context().stamp(),
        AssistantContextStamp {
            context_id: "page-1".into(),
            context_revision: 2,
            epoch: 3,
            actor_id: "operator-a".into(),
            scope_id: "scope-1".into(),
            scope_revision: 4,
            data_revision: 6,
            permission_revision: 7,
            policy_generation: 8,
        }
    );
}

#[test]
fn zero_is_a_present_qualified_fact_not_an_unavailable_value() {
    let fact = AssistantFact::new(
        "Hires",
        Some("0".into()),
        None,
        "Only admitted records",
        None,
    )
    .expect("a measured zero is valid");
    assert_eq!(fact.label(), "Hires");
    assert_eq!(fact.current(), Some("0"));
    assert_eq!(fact.qualification(), "Only admitted records");
    assert_eq!(fact.availability(), None);
}

#[test]
fn unavailable_fact_keeps_its_own_reason_and_qualification() {
    let fact = AssistantFact::new(
        "Hires",
        None,
        None,
        "Current page population",
        Some("Snapshot delayed".into()),
    )
    .expect("an explicitly unavailable fact is valid");
    assert_eq!(fact.current(), None);
    assert_eq!(fact.availability(), Some("Snapshot delayed"));
    assert_eq!(fact.qualification(), "Current page population");
}

#[test]
fn fact_rejects_missing_or_contradictory_availability() {
    for (current, availability) in [
        (None, None),
        (None, Some("  ".into())),
        (Some("0".into()), Some("Unavailable".into())),
        (Some("  ".into()), None),
    ] {
        assert_eq!(
            AssistantFact::new(
                "Hires",
                current,
                None,
                "Current page population",
                availability
            ),
            Err(AssistantContractError::InvalidFact)
        );
    }
}

#[test]
fn fact_rejects_blank_label_or_qualification() {
    for (label, qualification) in [
        ("", "Qualified"),
        ("  ", "Qualified"),
        ("Hires", ""),
        ("Hires", "\n "),
    ] {
        assert_eq!(
            AssistantFact::new(label, Some("1".into()), None, qualification, None),
            Err(AssistantContractError::InvalidFact)
        );
    }
}

#[test]
fn comparison_retains_its_basis_without_rewriting_display_text() {
    let comparison = AssistantComparison::new(" 12 % ", " Prior complete week ").unwrap();
    let fact = AssistantFact::new(
        "Hires",
        Some("0".into()),
        Some(comparison),
        "Selected offices only",
        None,
    )
    .unwrap();
    assert_eq!(fact.baseline().unwrap().display(), " 12 % ");
    assert_eq!(fact.baseline().unwrap().basis(), " Prior complete week ");
}

#[test]
fn comparison_rejects_blank_display_or_basis() {
    for (display, basis) in [("", "Prior week"), ("1", "  "), ("\n", "Prior week")] {
        assert_eq!(
            AssistantComparison::new(display, basis),
            Err(AssistantContractError::InvalidComparison)
        );
    }
}

fn answer() -> AssistantAnswer {
    AssistantAnswer {
        id: "answer-1".into(),
        revision: 1,
        answered_scope: context().scope,
        as_of: "10 September".into(),
        outcome: AnswerOutcome::Answered {
            text: "No admitted hires in this population.".into(),
        },
        facts: vec![
            AssistantFact::new(
                "Hires",
                Some("0".into()),
                None,
                "Admitted records only",
                None,
            )
            .unwrap(),
        ],
        evidence: vec![AssistantEvidence {
            id: "source-1".into(),
            label: "Selected population".into(),
            source_kind: "Snapshot".into(),
            revision: 1,
            as_of: "10 September".into(),
            availability: AssistantAccess::Granted,
            qualification: "Only admitted records".into(),
        }],
        provenance: AssistantProvenance {
            mode: ProvenanceMode::Generated,
            actual_engine: Some("actual-engine".into()),
            model_version: None,
            prompt_version: None,
            skill_version: None,
            memory_revisions: vec![],
            foundation_revisions: vec![],
            supplied_sources: vec![],
            omissions: vec![],
            built_at: "10 September".into(),
        },
        limitations: vec![],
    }
}

fn attempt() -> AssistantAttempt {
    AssistantAttempt {
        id: "attempt-1".into(),
        revision: 1,
        request_id: "request-1".into(),
        question: "Review these rows".into(),
        accepted_context: context(),
        lifecycle: AttemptLifecycle::Completed(answer()),
        transport: AssistantTransport::Connected { last_sequence: 4 },
        preview: None,
        cancel_requested: false,
        eligibility: AssistantEligibility::Eligible,
        requested_engine: Some("requested-engine".into()),
    }
}

fn conversation_fixture() -> AssistantConversation {
    AssistantConversation {
        id: "conversation-1".into(),
        revision: 1,
        context_id: "page-1".into(),
        scope_id: "scope-1".into(),
        closed: false,
        continuity: ConversationContinuity::Session,
        capabilities: context().capabilities,
        draft: AssistantDraft {
            id: "draft-1".into(),
            context_id: "page-1".into(),
            context_revision: 2,
            text: "A newer question".into(),
            revision: 2,
            max_chars: 1000,
        },
        suggestions: vec![],
        submission: SubmissionDisposition::Admitted {
            request_id: "request-1".into(),
            attempt_id: "attempt-1".into(),
        },
        attempts: vec![attempt()],
    }
}

#[test]
fn every_context_identity_rejects_blank_and_unknown_detail_capabilities() {
    for field in 0..9 {
        let mut value = context();
        match field {
            0 => value.id.clear(),
            1 => value.actor.label.clear(),
            2 => value.subject.as_mut().unwrap().id.clear(),
            3 => value.subject.as_mut().unwrap().label.clear(),
            4 => value.scope.id.clear(),
            5 => value.scope.label.clear(),
            6 | 7 => {
                if let ScopeBasis::PagePopulation {
                    population_id,
                    description,
                    ..
                } = &mut value.scope.basis
                {
                    if field == 6 {
                        population_id.clear();
                    } else {
                        description.clear();
                    }
                }
            }
            8 => {
                value.scope.basis = ScopeBasis::Expanded {
                    authorization_id: "authority-1".into(),
                    authorization_revision: 1,
                    description: " \n".into(),
                }
            }
            _ => unreachable!(),
        }
        assert_eq!(
            validate_context(&value),
            Err(AssistantContractError::MissingIdentity),
            "field {field}"
        );
    }
    let mut caps = context().capabilities;
    caps.details.push(AssistantCapabilityDetail {
        capability: AssistantCapability::Unknown("future-capability".into()),
        state: AssistantCapabilityState::Granted,
    });
    assert!(caps.validate().is_err());
    assert!(!caps.allows(&AssistantCapability::Ask));
}

#[test]
fn answer_requires_complete_accepted_scope_not_merely_its_display_label() {
    let accepted = answer();
    assert_eq!(validate_answer(&accepted, &context().scope), Ok(()));
    for field in 0..5 {
        let mut changed = accepted.clone();
        match field {
            0 => changed.answered_scope.id = "another-scope".into(),
            1 => changed.answered_scope.revision += 1,
            2 => changed.answered_scope.label = "All offices".into(),
            3 | 4 => {
                if let ScopeBasis::PagePopulation {
                    population_id,
                    population_revision,
                    ..
                } = &mut changed.answered_scope.basis
                {
                    if field == 3 {
                        *population_id = "all-records".into();
                    } else {
                        *population_revision += 1;
                    }
                }
            }
            _ => unreachable!(),
        }
        assert_eq!(
            validate_answer(&changed, &context().scope),
            Err(AssistantContractError::ScopeMismatch),
            "field {field}"
        );
    }
}

#[test]
fn completed_decline_is_healthy_only_with_explicit_nonblank_limitations() {
    let mut value = answer();
    value.outcome = AnswerOutcome::Declined {
        limitations: vec!["Admission coverage is incomplete".into()],
        as_of: "10 September".into(),
    };
    assert_eq!(validate_answer(&value, &context().scope), Ok(()));
    for limitations in [
        vec![],
        vec![" \n".into()],
        vec!["Coverage is incomplete".into(), "".into()],
    ] {
        value.outcome = AnswerOutcome::Declined {
            limitations,
            as_of: "10 September".into(),
        };
        assert_eq!(
            validate_answer(&value, &context().scope),
            Err(AssistantContractError::InvalidAnswer)
        );
    }
    value.outcome = AnswerOutcome::Answered { text: " \n".into() };
    assert_eq!(
        validate_answer(&value, &context().scope),
        Err(AssistantContractError::InvalidAnswer)
    );
    value.outcome = AnswerOutcome::Unknown("provider-error-as-answer".into());
    assert_eq!(
        validate_answer(&value, &context().scope),
        Err(AssistantContractError::UnknownState)
    );
}

#[test]
fn duplicate_evidence_and_provenance_identities_are_contract_errors() {
    let mut value = answer();
    value.evidence.push(value.evidence[0].clone());
    assert!(validate_answer(&value, &context().scope).is_err());
    value.evidence.pop();
    let item = AssistantItemRevision {
        id: "source-1".into(),
        revision: 1,
    };
    value.provenance.supplied_sources = vec![item.clone(), item];
    assert!(validate_answer(&value, &context().scope).is_err());
    value.provenance.supplied_sources.clear();
    value.provenance.mode = ProvenanceMode::Unknown("future-origin".into());
    assert_eq!(
        validate_answer(&value, &context().scope),
        Err(AssistantContractError::UnknownState)
    );
}

#[test]
fn all_ten_lifecycles_keep_cancellation_and_transport_truth_separate() {
    for lifecycle in [
        AttemptLifecycle::Admitted,
        AttemptLifecycle::Queued,
        AttemptLifecycle::Running,
        AttemptLifecycle::Validating,
        AttemptLifecycle::Completed(answer()),
        AttemptLifecycle::Denied { reason: reason() },
        AttemptLifecycle::Unavailable { reason: reason() },
        AttemptLifecycle::Failed { reason: reason() },
        AttemptLifecycle::Canceled { discarded: false },
        AttemptLifecycle::Canceled { discarded: true },
        AttemptLifecycle::Interrupted { reason: reason() },
    ] {
        let mut value = attempt();
        value.lifecycle = lifecycle.clone();
        value.transport = AssistantTransport::Disconnected {
            last_sequence: 4,
            reason: reason(),
        };
        assert_eq!(validate_attempt(&value), Ok(()), "{lifecycle:?}");
        assert_eq!(
            value.lifecycle, lifecycle,
            "validation cannot change durable state"
        );
    }
    assert_ne!(
        AttemptLifecycle::Canceled { discarded: true },
        AttemptLifecycle::Canceled { discarded: false }
    );
}

#[test]
fn unknown_transport_recovery_and_lifecycle_fail_closed() {
    let mut value = attempt();
    value.lifecycle = AttemptLifecycle::Unknown("future-terminal".into());
    assert_eq!(
        validate_attempt(&value),
        Err(AssistantContractError::UnknownState)
    );
    value = attempt();
    value.transport = AssistantTransport::Unknown("future-channel".into());
    assert_eq!(
        validate_attempt(&value),
        Err(AssistantContractError::UnknownState)
    );
    value.transport = AssistantTransport::Recovering {
        last_sequence: 4,
        cause: RecoveryCause::Unknown("future-recovery".into()),
    };
    assert_eq!(
        validate_attempt(&value),
        Err(AssistantContractError::UnknownState)
    );
}

#[test]
fn conversation_admission_must_match_one_exact_attempt_and_request() {
    let mut value = conversation_fixture();
    let draft = value.draft.clone();
    assert_eq!(validate_conversation(&value), Ok(()));
    assert_eq!(value.draft, draft, "admission cannot clear a newer draft");
    value.submission = SubmissionDisposition::Admitted {
        request_id: "other-request".into(),
        attempt_id: "attempt-1".into(),
    };
    assert_eq!(
        validate_conversation(&value),
        Err(AssistantContractError::InvalidReceipt)
    );
    value = conversation_fixture();
    value.attempts.push(value.attempts[0].clone());
    assert!(validate_conversation(&value).is_err());
    value.attempts[1].id = "attempt-2".into();
    assert!(
        validate_conversation(&value).is_err(),
        "one request cannot admit two attempts"
    );
    value = conversation_fixture();
    value.continuity = ConversationContinuity::Unknown("implicit-durable".into());
    assert_eq!(
        validate_conversation(&value),
        Err(AssistantContractError::UnknownState)
    );
}

#[test]
fn all_refusal_guidance_is_explicit_and_unknown_is_not_retryable() {
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

fn connection_fixture(state: ConnectionState, shape: SignInShape) -> AssistantConnection {
    let device = (state == ConnectionState::Pending && shape == SignInShape::Device).then(|| {
        AssistantDeviceFlow {
            verification_url: "https://example.invalid/verify".into(),
            user_code: "SYNTHETIC".into(),
            expires_at_ms: 2000,
            expires_at: "Expires at the host supplied time".into(),
        }
    });
    let reason = matches!(state, ConnectionState::Expired | ConnectionState::Rejected).then(reason);
    AssistantConnection {
        state,
        shape,
        device,
        account_label: None,
        verified_at: None,
        expires_at: None,
        reason,
    }
}

fn settings_fixture() -> AssistantSettings {
    AssistantSettings {
        owner_actor_id: "operator-a".into(),
        accepted_revision: 2,
        proposed_revision: 3,
        accepted: AssistantPreferences {
            engine_id: Some("engine-a".into()),
            memory_use: false,
            memory_capture: false,
        },
        proposed: AssistantPreferences {
            engine_id: Some("engine-a".into()),
            memory_use: true,
            memory_capture: false,
        },
        reasoning_tier: AssistantAccess::Granted,
        engines: vec![AssistantEngine {
            id: "engine-a".into(),
            revision: 1,
            label: "Host-configured engine".into(),
            availability: EngineAvailability::Enabled,
            connection: connection_fixture(ConnectionState::SignedIn, SignInShape::Device),
            capabilities: context().capabilities,
        }],
        budget: None,
        capabilities: context().capabilities,
    }
}

#[test]
fn every_named_connection_state_and_shape_has_a_truthful_projection() {
    for state in [
        ConnectionState::NotSignedIn,
        ConnectionState::Pending,
        ConnectionState::Probing,
        ConnectionState::SignedIn,
        ConnectionState::Expired,
        ConnectionState::Rejected,
        ConnectionState::Revoked,
        ConnectionState::NotApplicable,
    ] {
        for shape in [
            SignInShape::Paste,
            SignInShape::Device,
            SignInShape::Operator,
        ] {
            let connection = connection_fixture(state.clone(), shape.clone());
            assert_eq!(
                validate_connection(&connection),
                Ok(()),
                "{state:?}/{shape:?}"
            );
        }
    }
}

#[test]
fn device_open_requires_the_exact_unexpired_pending_device_flow() {
    let mut connection = connection_fixture(ConnectionState::Pending, SignInShape::Device);
    assert!(device_flow_is_current(&connection, 1999));
    assert!(!device_flow_is_current(&connection, 2000));
    assert!(!device_flow_is_current(&connection, 2001));
    connection.device = None;
    assert_eq!(
        validate_connection(&connection),
        Err(AssistantContractError::InvalidConnection)
    );
    assert!(!device_flow_is_current(&connection, 1000));
    for field in 0..3 {
        connection = connection_fixture(ConnectionState::Pending, SignInShape::Device);
        let device = connection.device.as_mut().unwrap();
        match field {
            0 => device.verification_url.clear(),
            1 => device.user_code.clear(),
            2 => device.expires_at.clear(),
            _ => unreachable!(),
        }
        assert_eq!(
            validate_connection(&connection),
            Err(AssistantContractError::InvalidConnection)
        );
        assert!(!device_flow_is_current(&connection, 1000));
    }
    connection = connection_fixture(ConnectionState::Pending, SignInShape::Device);
    connection.state = ConnectionState::SignedIn;
    assert!(
        !device_flow_is_current(&connection, 1000),
        "completed flow cannot be reopened"
    );
    connection.state = ConnectionState::Pending;
    connection.shape = SignInShape::Operator;
    assert_eq!(
        validate_connection(&connection),
        Err(AssistantContractError::InvalidConnection)
    );
}

#[test]
fn unknown_connection_shape_state_and_availability_never_enable_an_engine() {
    let mut settings = settings_fixture();
    assert!(engine_ready_for_ask(&settings, "operator-a").is_some());
    settings.engines[0].connection.state = ConnectionState::Unknown("new-state".into());
    assert!(validate_settings(&settings, "operator-a").is_err());
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
    settings = settings_fixture();
    settings.engines[0].connection.shape = SignInShape::Unknown("new-flow".into());
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
    settings = settings_fixture();
    settings.engines[0].availability = EngineAvailability::Unknown("new-engine-state".into());
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
}

#[test]
fn reasoning_tier_and_explicit_grants_outrank_a_signed_in_engine() {
    let mut settings = settings_fixture();
    for access in [
        AssistantAccess::Denied { reason: reason() },
        AssistantAccess::Unresolved { reason: reason() },
        AssistantAccess::Unknown("new-tier".into()),
    ] {
        settings.reasoning_tier = access;
        assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
    }
    settings.reasoning_tier = AssistantAccess::Granted;
    assert!(engine_ready_for_ask(&settings, "operator-a").is_some());
    settings.capabilities.granted.clear();
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
    settings.capabilities = context().capabilities;
    settings.engines[0].capabilities.granted.clear();
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
}

#[test]
fn disabled_cli_reason_is_preserved_without_a_fallback_engine() {
    let mut settings = settings_fixture();
    settings.engines[0].availability = EngineAvailability::Disabled {
        reason_code: "cli-not-tested".into(),
        text: "This host CLI has not been qualified".into(),
    };
    let before = settings.clone();
    assert_eq!(validate_settings(&settings, "operator-a"), Ok(()));
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
    assert_eq!(settings, before);
}

#[test]
fn settings_owner_and_engine_identities_are_exact_and_independent_of_subject() {
    let mut settings = settings_fixture();
    assert_eq!(validate_settings(&settings, "operator-a"), Ok(()));
    assert_eq!(
        validate_settings(&settings, "subject-b"),
        Err(AssistantContractError::ScopeMismatch)
    );
    settings.engines.push(settings.engines[0].clone());
    assert!(validate_settings(&settings, "operator-a").is_err());
    settings = settings_fixture();
    settings.accepted.engine_id = Some("unlisted-engine".into());
    assert!(validate_settings(&settings, "operator-a").is_err());
    settings = settings_fixture();
    settings.proposed.engine_id = Some("unlisted-engine".into());
    assert!(validate_settings(&settings, "operator-a").is_err());
    settings = settings_fixture();
    settings.accepted.engine_id = None;
    assert_eq!(validate_settings(&settings, "operator-a"), Ok(()));
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
}

#[test]
fn settings_validation_never_promotes_proposed_memory_or_engine_preferences() {
    let settings = settings_fixture();
    let before = settings.clone();
    assert_eq!(validate_settings(&settings, "operator-a"), Ok(()));
    assert_eq!(settings, before);
    assert!(!settings.accepted.memory_use);
    assert!(settings.proposed.memory_use);
    assert!(
        !settings.proposed.memory_capture,
        "memory use never silently enables capture"
    );
    assert_eq!(AssistantPreferences::default().engine_id, None);
    assert!(!AssistantPreferences::default().memory_capture);
}

#[test]
fn conversation_never_accepts_a_draft_bound_to_another_context() {
    let mut conversation = conversation_fixture();
    conversation.context_id = "another-page".into();
    let retained = conversation.draft.clone();
    assert_eq!(
        validate_conversation(&conversation),
        Err(AssistantContractError::ScopeMismatch)
    );
    assert_eq!(conversation.draft, retained);
}

#[test]
fn non_admitted_dispositions_cannot_reuse_an_admitted_request_identity() {
    let mut conversation = conversation_fixture();
    for submission in [
        SubmissionDisposition::Submitting {
            request_id: "request-1".into(),
        },
        SubmissionDisposition::Uncertain {
            request_id: "request-1".into(),
            reason: reason(),
        },
        SubmissionDisposition::Refused {
            request_id: "request-1".into(),
            reason: RefusalReason::EvidenceAdmissionUnavailable,
        },
    ] {
        conversation.submission = submission;
        assert_eq!(
            validate_conversation(&conversation),
            Err(AssistantContractError::InvalidReceipt)
        );
    }
    conversation.submission = SubmissionDisposition::Submitting {
        request_id: "fresh-request".into(),
    };
    assert_eq!(validate_conversation(&conversation), Ok(()));
}

#[test]
fn cancellation_requested_is_not_terminal_but_cannot_accept_a_late_completion() {
    let mut value = attempt();
    value.cancel_requested = true;
    assert_eq!(
        validate_attempt(&value),
        Err(AssistantContractError::InvalidAnswer)
    );
    value.lifecycle = AttemptLifecycle::Running;
    assert_eq!(
        validate_attempt(&value),
        Ok(()),
        "accepted cancellation request is not itself terminal"
    );
    value.lifecycle = AttemptLifecycle::Canceled { discarded: true };
    assert_eq!(validate_attempt(&value), Ok(()));
    value.transport = AssistantTransport::Recovering {
        last_sequence: 4,
        cause: RecoveryCause::TerminalConflict,
    };
    assert_eq!(
        validate_attempt(&value),
        Ok(()),
        "conflict preserves the accepted canceled terminal"
    );
}

#[test]
fn denial_and_ineligibility_carry_named_sanitized_explanations() {
    let mut value = answer();
    value.evidence[0].availability = AssistantAccess::Denied {
        reason: AssistantReason::default(),
    };
    assert!(validate_answer(&value, &context().scope).is_err());
    value.evidence[0].availability = AssistantAccess::Denied { reason: reason() };
    assert_eq!(validate_answer(&value, &context().scope), Ok(()));
    let mut attempt = attempt();
    attempt.eligibility = AssistantEligibility::Ineligible {
        reason: AssistantReason::default(),
    };
    assert!(validate_attempt(&attempt).is_err());
    attempt.eligibility = AssistantEligibility::Ineligible { reason: reason() };
    assert_eq!(validate_attempt(&attempt), Ok(()));
}

#[test]
fn accepted_freshness_and_omission_displays_cannot_be_silently_blank() {
    for field in 0..4 {
        let mut value = answer();
        match field {
            0 => value.as_of.clear(),
            1 => value.evidence[0].as_of.clear(),
            2 => value.provenance.built_at.clear(),
            3 => value.provenance.omissions.push(" \n".into()),
            _ => unreachable!(),
        }
        assert!(
            validate_answer(&value, &context().scope).is_err(),
            "blank field {field}"
        );
    }
}

#[test]
fn optional_settings_fields_never_disable_a_valid_accepted_engine() {
    for field in 0..3 {
        let mut settings = settings_fixture();
        match field {
            0 => {
                settings.budget = Some(AssistantBudget {
                    policy_label: "".into(),
                    limit_display: "".into(),
                    remaining_display: None,
                    as_of: "".into(),
                })
            }
            1 => settings.proposed.engine_id = Some("invalid-unsaved-choice".into()),
            2 => {
                let mut extra = settings.engines[0].clone();
                extra.id = "unselected-engine".into();
                extra.availability = EngineAvailability::Unknown("new-protocol".into());
                settings.engines.push(extra);
            }
            _ => unreachable!(),
        }
        assert!(
            validate_settings(&settings, "operator-a").is_err(),
            "Settings must report malformed field {field}"
        );
        assert!(
            engine_ready_for_ask(&settings, "operator-a").is_some(),
            "unrelated Settings field {field} disabled accepted Ask"
        );
    }
}

#[test]
fn expired_and_rejected_connections_require_named_reasons_in_every_shape() {
    for state in [ConnectionState::Expired, ConnectionState::Rejected] {
        for shape in [
            SignInShape::Paste,
            SignInShape::Device,
            SignInShape::Operator,
        ] {
            let mut connection = connection_fixture(state.clone(), shape);
            assert_eq!(validate_connection(&connection), Ok(()));
            connection.reason = None;
            assert_eq!(
                validate_connection(&connection),
                Err(AssistantContractError::InvalidConnection)
            );
            connection.reason = Some(AssistantReason::default());
            assert!(validate_connection(&connection).is_err());
        }
    }
}

#[test]
fn proposed_engine_and_ready_alternatives_never_become_accepted_fallbacks() {
    let mut settings = settings_fixture();
    let mut alternative = settings.engines[0].clone();
    alternative.id = "engine-b".into();
    settings.engines.push(alternative);
    settings.proposed.engine_id = Some("engine-b".into());
    assert_eq!(
        engine_ready_for_ask(&settings, "operator-a").unwrap().id,
        "engine-a"
    );
    settings.engines[0].availability = EngineAvailability::Disabled {
        reason_code: "disabled".into(),
        text: "Not qualified".into(),
    };
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
}

#[test]
fn accepted_engine_readiness_requires_exactly_one_identity_and_the_current_actor() {
    let mut settings = settings_fixture();
    assert!(engine_ready_for_ask(&settings, "subject-b").is_none());
    settings.engines.push(settings.engines[0].clone());
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
    settings = settings_fixture();
    settings.accepted.engine_id = Some("".into());
    assert!(engine_ready_for_ask(&settings, "operator-a").is_none());
}

fn proposal_fixture() -> AssistantProposal {
    AssistantProposal {
        id: "proposal-1".into(),
        revision: 2,
        source_id: "artifact-1".into(),
        source_revision: 3,
        context: context().stamp(),
        expires_at_ms: 2000,
        target: ProposalTarget::Composer(AssistantDestination {
            client_id: "client-1".into(),
            thread_id: "thread-1".into(),
            channel: "sms".into(),
            draft_id: "destination-1".into(),
            draft_revision: 4,
            current_text: String::new(),
            previous_suggestion: None,
            eligibility: AssistantEligibility::Eligible,
        }),
        content: "A reviewed suggestion".into(),
        policy: InsertionPolicy::EmptyOnly,
        reviewed_revision: Some(2),
        eligibility: AssistantEligibility::Eligible,
    }
}

fn artifact_fixture() -> AssistantArtifact {
    AssistantArtifact {
        id: "artifact-1".into(),
        revision: 3,
        context: context().stamp(),
        eligibility: AssistantEligibility::Eligible,
        kind: ArtifactKind::ClientDraft {
            recipient: "Synthetic client".into(),
            channel: "sms".into(),
            language: "English".into(),
            language_confirmed: true,
            intent: "Follow up".into(),
            subject: None,
        },
        provenance: answer().provenance,
        evidence: answer().evidence,
        draft: Some(AssistantArtifactDraft {
            accepted_text: "A reviewed suggestion".into(),
            proposed_text: "A reviewed suggestion".into(),
            revision: 4,
            language: "English".into(),
            human_edited: false,
            max_chars: 1000,
        }),
        proposal: Some(proposal_fixture()),
        capabilities: AssistantCapabilities::default(),
    }
}

fn proposal_source() -> AssistantItemRevision {
    AssistantItemRevision {
        id: "artifact-1".into(),
        revision: 3,
    }
}

#[test]
fn proposal_requires_exact_review_source_context_and_unexpired_authority() {
    let original = proposal_fixture();
    assert_eq!(validate_proposal(&original), Ok(()));
    assert!(proposal_is_current(
        &original,
        &proposal_source(),
        &context().stamp(),
        1999
    ));
    for mutation in 0..6 {
        let mut proposal = original.clone();
        match mutation {
            0 => proposal.reviewed_revision = None,
            1 => proposal.reviewed_revision = Some(1),
            2 => proposal.source_id = "another-source".into(),
            3 => proposal.source_revision += 1,
            4 => proposal.context.policy_generation += 1,
            5 => proposal.eligibility = AssistantEligibility::Ineligible { reason: reason() },
            _ => unreachable!(),
        }
        assert!(
            !proposal_is_current(&proposal, &proposal_source(), &context().stamp(), 1999),
            "mutation {mutation}"
        );
    }
    assert!(!proposal_is_current(
        &original,
        &proposal_source(),
        &context().stamp(),
        2000
    ));
}

#[test]
fn empty_only_and_unchanged_suggestion_policies_never_replace_human_text() {
    let mut proposal = proposal_fixture();
    assert!(proposal_is_current(
        &proposal,
        &proposal_source(),
        &context().stamp(),
        1000
    ));
    if let ProposalTarget::Composer(destination) = &mut proposal.target {
        destination.current_text = " ".into();
    }
    assert!(
        !proposal_is_current(&proposal, &proposal_source(), &context().stamp(), 1000),
        "typed whitespace is not an empty draft"
    );
    proposal.policy = InsertionPolicy::EmptyOrUnchangedSuggestion;
    if let ProposalTarget::Composer(destination) = &mut proposal.target {
        destination.previous_suggestion = Some("Previous suggestion".into());
        destination.current_text = "Previous suggestion".into();
    }
    assert!(proposal_is_current(
        &proposal,
        &proposal_source(),
        &context().stamp(),
        1000
    ));
    if let ProposalTarget::Composer(destination) = &mut proposal.target {
        destination.current_text.push('!');
    }
    let retained = proposal.clone();
    assert!(!proposal_is_current(
        &proposal,
        &proposal_source(),
        &context().stamp(),
        1000
    ));
    assert_eq!(
        proposal, retained,
        "a failed guard never alters a human draft"
    );
}

#[test]
fn workflow_handoffs_require_the_matching_closed_policy_and_target_eligibility() {
    let mut proposal = proposal_fixture();
    proposal.target = ProposalTarget::OwnerWorkflow {
        kind: OwnerWorkflowKind::Note,
        record_id: "record-1".into(),
        record_revision: 1,
        owner_workflow_id: "notes-1".into(),
        eligibility: AssistantEligibility::Eligible,
    };
    assert!(
        validate_proposal(&proposal).is_err(),
        "composer insertion policy cannot open an owner workflow"
    );
    proposal.policy = InsertionPolicy::OwnerWorkflow;
    assert_eq!(validate_proposal(&proposal), Ok(()));
    assert!(proposal_is_current(
        &proposal,
        &proposal_source(),
        &context().stamp(),
        1000
    ));
    if let ProposalTarget::OwnerWorkflow { eligibility, .. } = &mut proposal.target {
        *eligibility = AssistantEligibility::Ineligible { reason: reason() };
    }
    assert!(!proposal_is_current(
        &proposal,
        &proposal_source(),
        &context().stamp(),
        1000
    ));
    proposal.target = ProposalTarget::Unknown("new-business-write".into());
    assert_eq!(
        validate_proposal(&proposal),
        Err(AssistantContractError::UnknownState)
    );
}

#[test]
fn prepared_template_identity_cannot_survive_edits_or_replacement_policy() {
    let mut artifact = artifact_fixture();
    artifact.kind = ArtifactKind::PreparedTemplate {
        template_id: "template-1".into(),
        template_revision: 5,
        expires_at_ms: 2000,
    };
    assert_eq!(validate_artifact(&artifact), Ok(()));
    artifact.draft.as_mut().unwrap().human_edited = true;
    assert!(validate_artifact(&artifact).is_err());
    artifact.draft.as_mut().unwrap().human_edited = false;
    artifact.draft.as_mut().unwrap().proposed_text.push('!');
    assert!(validate_artifact(&artifact).is_err());
    artifact.draft.as_mut().unwrap().proposed_text.pop();
    artifact.proposal.as_mut().unwrap().policy = InsertionPolicy::EmptyOrUnchangedSuggestion;
    assert!(validate_artifact(&artifact).is_err());
}

#[test]
fn read_only_analysis_and_briefing_have_no_fabricated_editable_draft() {
    let mut artifact = artifact_fixture();
    artifact.kind = ArtifactKind::Analysis {
        objections: vec![],
        talking_points: vec!["A grounded point".into()],
        next_action: None,
        limitations: vec![],
    };
    assert!(validate_artifact(&artifact).is_err());
    artifact.draft = None;
    artifact.proposal = None;
    assert_eq!(validate_artifact(&artifact), Ok(()));
    artifact.kind = ArtifactKind::Briefing {
        facts: answer().facts,
        completeness: "Partial source coverage".into(),
    };
    assert_eq!(validate_artifact(&artifact), Ok(()));
}

#[test]
fn artifact_drafts_enforce_scalar_bounds_without_promoting_proposed_text() {
    let mut artifact = artifact_fixture();
    let draft = artifact.draft.as_mut().unwrap();
    draft.accepted_text = "é".into();
    draft.proposed_text = "🦀é".into();
    draft.max_chars = 2;
    let before = artifact.clone();
    assert_eq!(validate_artifact(&artifact), Ok(()));
    assert_eq!(artifact, before);
    artifact.draft.as_mut().unwrap().proposed_text.push('x');
    assert!(validate_artifact(&artifact).is_err());
}

#[test]
fn rich_call_preparation_preserves_missing_facts_examples_and_channel_suggestions() {
    let mut artifact = artifact_fixture();
    artifact.kind = ArtifactKind::CallPreparation(AssistantCallPreparation {
        guidance: crate::components::ClientCallGuidance {
            title: "Call preparation".into(),
            body: "A reviewed opening".into(),
            ..Default::default()
        },
        beats: vec![AssistantCallBeat {
            id: "beat-1".into(),
            title: "Establish purpose".into(),
            intent: "Confirm the next step".into(),
            fact: None,
            say: "Could we confirm the next step?".into(),
            avoid: vec!["Do not assert an unverified balance".into()],
            missing_fact: Some("Current balance not supplied".into()),
            worked_example: Some("Ask the client to confirm the available detail".into()),
        }],
        framing_hazards: vec!["No redundant balance banner".into()],
        permitted_closes: vec!["Confirm a next step".into()],
        facts: answer().facts,
        rehearsal_turns: vec![AssistantRehearsalTurn {
            speaker: "Client".into(),
            text: "What is the next step?".into(),
        }],
        sms_suggestion: Some("A draft SMS".into()),
        email_subject: Some("A draft subject".into()),
        email_body: Some("A draft email".into()),
        language: "Spanish".into(),
        language_confirmed: false,
        rubric_version: Some("rubric-1".into()),
        freshness: "Current synthetic snapshot".into(),
    });
    let before = artifact.clone();
    assert_eq!(validate_artifact(&artifact), Ok(()));
    assert_eq!(artifact, before);
    if let ArtifactKind::CallPreparation(preparation) = &mut artifact.kind {
        preparation.beats.push(preparation.beats[0].clone());
    }
    assert!(
        validate_artifact(&artifact).is_err(),
        "rich beats require stable unique identities"
    );
}

#[test]
fn proposal_unknowns_and_blank_target_identities_fail_closed() {
    for field in 0..8 {
        let mut proposal = proposal_fixture();
        match field {
            0 => proposal.id.clear(),
            1 => proposal.source_id.clear(),
            2 => proposal.context.actor_id.clear(),
            3 => proposal.policy = InsertionPolicy::Unknown("replace-human-text".into()),
            4 => proposal.eligibility = AssistantEligibility::Unknown("future-grant".into()),
            5 => {
                if let ProposalTarget::Composer(destination) = &mut proposal.target {
                    destination.client_id.clear();
                }
            }
            6 => {
                if let ProposalTarget::Composer(destination) = &mut proposal.target {
                    destination.draft_id.clear();
                }
            }
            7 => proposal.content.clear(),
            _ => unreachable!(),
        }
        assert!(validate_proposal(&proposal).is_err(), "field {field}");
        assert!(!proposal_is_current(
            &proposal,
            &proposal_source(),
            &context().stamp(),
            1000
        ));
    }
}

#[test]
fn reused_call_guidance_beats_cannot_bypass_stable_identity_and_display_validation() {
    for field in 0..4 {
        let mut artifact = artifact_fixture();
        let mut guidance = crate::components::ClientCallGuidance {
            title: "A reviewed script".into(),
            body: "A reviewed opening".into(),
            beats: vec![crate::components::ClientCallScriptBeat {
                id: "nested-beat-1".into(),
                title: "Establish purpose".into(),
                say: "Could we confirm the next step?".into(),
                no_file_detail: false,
            }],
            ..Default::default()
        };
        match field {
            0 => guidance.beats[0].id.clear(),
            1 => guidance.beats[0].title.clear(),
            2 => guidance.beats[0].say.clear(),
            3 => guidance.beats.push(guidance.beats[0].clone()),
            _ => unreachable!(),
        }
        artifact.kind = ArtifactKind::CallPreparation(AssistantCallPreparation {
            guidance,
            beats: vec![],
            framing_hazards: vec![],
            permitted_closes: vec![],
            facts: vec![],
            rehearsal_turns: vec![],
            sms_suggestion: None,
            email_subject: None,
            email_body: None,
            language: "English".into(),
            language_confirmed: true,
            rubric_version: None,
            freshness: "Current synthetic snapshot".into(),
        });
        assert!(
            validate_artifact(&artifact).is_err(),
            "nested guidance field {field}"
        );
    }
}
