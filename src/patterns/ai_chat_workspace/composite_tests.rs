//! Native proofs for the composite's decision logic (P4).
//!
//! Kept separate from `tests.rs` (P1's vocabulary proofs) so the two batches
//! stay legible; both are `#[cfg(test)]` modules of `ai_chat_workspace`.

use super::*;

use crate::components::ai_assistant_workspace::{
    AnswerOutcome, AssistantAccess, AssistantCapabilities, AssistantCapability,
    AssistantPreferences, AssistantReason, AssistantSettings, AttemptLifecycle, ConnectionState,
    EngineAvailability, engine_ready_for_ask,
};

/// Resolve one fixture future. Every backend future in this module is ready
/// on its first poll, exactly as `memory.rs`'s own tests assume — a real
/// executor would add nothing but a dependency.
#[cfg(feature = "test-mode")]
fn now<T>(mut fut: WorkspaceFuture<T>) -> Result<T, ChatWorkspaceError> {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    let waker = std::task::Waker::noop();
    let mut cx = Context::from_waker(waker);
    match Pin::new(&mut fut).poll(&mut cx) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("fixture futures are always ready"),
    }
}

/// One card from the catalogue, by engine id.
fn catalogue_card(engine_id: &str) -> ProviderCard {
    desktop_provider_catalogue()
        .into_iter()
        .find(|c| c.engine.id == engine_id)
        .expect("the catalogue publishes this engine")
}

// ── Per-card readiness ──────────────────────────────────────────────────────

#[test]
fn every_catalogue_card_is_ready_for_ask() {
    let cards = desktop_provider_catalogue();
    assert_eq!(cards.len(), 5);
    for card in &cards {
        assert!(
            card_ready_for_ask(card),
            "{} must be ready out of the catalogue",
            card.engine.id
        );
        assert_eq!(
            card_unready_reason(card),
            None,
            "a ready card must publish no reason"
        );
    }
}

#[test]
fn a_disabled_engine_is_not_ready_and_reports_the_hosts_own_code() {
    let mut card = catalogue_card("claude-code");
    card.engine.availability = EngineAvailability::Disabled {
        reason_code: AvailabilityReasonCode::CliMissing.as_code().to_owned(),
        text: "The CLI is not installed.".to_owned(),
    };
    assert!(!card_ready_for_ask(&card));
    assert_eq!(
        card_unready_reason(&card),
        Some(AvailabilityReasonCode::CliMissing),
        "the reason comes from the host's own code, it is not re-derived"
    );
}

#[test]
fn an_engine_without_the_ask_grant_is_not_ready_and_reads_as_unarmed() {
    let mut card = catalogue_card("codex-cli");
    card.engine.capabilities = AssistantCapabilities {
        granted: vec![AssistantCapability::Copy],
        details: vec![],
    };
    assert!(
        !card_ready_for_ask(&card),
        "availability alone never implies the Ask grant"
    );
    assert_eq!(
        card_unready_reason(&card),
        Some(AvailabilityReasonCode::NotArmed),
        "a missing grant is an environment fact, never a credential failure"
    );
    assert!(
        !AvailabilityReasonCode::NotArmed.is_credential_failure(),
        "so it must not route the actor at a sign-in screen"
    );
}

#[test]
fn a_signed_out_or_expired_engine_is_not_ready() {
    for (state, expected) in [
        (
            ConnectionState::NotSignedIn,
            AvailabilityReasonCode::NotSignedIn,
        ),
        (
            ConnectionState::Expired,
            AvailabilityReasonCode::SignInExpired,
        ),
        (
            ConnectionState::Pending,
            AvailabilityReasonCode::NotSignedIn,
        ),
        (
            ConnectionState::Revoked,
            AvailabilityReasonCode::NotSignedIn,
        ),
    ] {
        let mut card = catalogue_card("groq-gpt-oss-120b");
        card.engine.connection.state = state.clone();
        assert!(!card_ready_for_ask(&card), "{state:?} must not be ready");
        assert_eq!(card_unready_reason(&card), Some(expected), "{state:?}");
    }
    // The negative control: the two states that ARE ready.
    for state in [ConnectionState::SignedIn, ConnectionState::NotApplicable] {
        let mut card = catalogue_card("groq-gpt-oss-120b");
        card.engine.connection.state = state.clone();
        assert!(card_ready_for_ask(&card), "{state:?} must be ready");
    }
}

#[test]
fn card_readiness_does_not_consult_the_accepted_engine_id() {
    // `engine_ready_for_ask` answers a DIFFERENT question: it accepts only the
    // actor's single accepted engine. Pinning both answers against one
    // settings value is what stops a refactor collapsing the two.
    let cards = desktop_provider_catalogue();
    let accepted = cards[1].engine.id.clone();
    let preferences = AssistantPreferences {
        engine_id: Some(accepted.clone()),
        memory_use: true,
        memory_capture: true,
    };
    let settings = AssistantSettings {
        owner_actor_id: "actor-1".to_owned(),
        accepted_revision: 1,
        proposed_revision: 1,
        accepted: preferences.clone(),
        proposed: preferences,
        reasoning_tier: AssistantAccess::Granted,
        engines: cards.iter().map(|c| c.engine.clone()).collect(),
        budget: None,
        capabilities: AssistantCapabilities {
            granted: vec![AssistantCapability::Ask],
            details: vec![],
        },
    };
    assert_eq!(
        engine_ready_for_ask(&settings, "actor-1").map(|e| e.id.clone()),
        Some(accepted.clone()),
        "the accepted engine is the only one that function can return"
    );
    assert_eq!(
        cards.iter().filter(|c| c.engine.id != accepted).count(),
        4,
        "four cards are not the accepted one"
    );
    for card in &cards {
        assert!(
            card_ready_for_ask(card),
            "{} is ready regardless of which engine is accepted",
            card.engine.id
        );
    }
}

// ── A canceled turn is never a finished answer ──────────────────────────────

/// Drive one seeded turn to a cancel and return the record it left behind.
#[cfg(feature = "test-mode")]
fn canceled_record(discard: bool) -> TurnRecord {
    use crate::components::ai_chat::{ChatRequest, ChatSession};

    let backend = if discard {
        InMemoryChatWorkspaceBackend::seeded().with_fault(ChatWorkspaceFault::CancelDiscards)
    } else {
        InMemoryChatWorkspaceBackend::seeded()
    };
    let transport = now(backend.open_session(
        "claude-code",
        &KnowledgeSelection {
            posture: ChatPosture::Assistant,
            ..KnowledgeSelection::default()
        },
        Default::default(),
        ProviderTuning::default(),
    ))
    .expect("the seeded fixture opens a session");
    let mut session = ChatSession::new(transport);
    session
        .send(ChatRequest {
            prompt: "tell me about the intake checklist".to_owned(),
            attachments: Vec::new(),
            page_context: None,
        })
        .expect("send");
    // Two ladder polls, then four that each release one scripted event, so a
    // real prefix has streamed before the cancel lands.
    for _ in 0..6 {
        session.poll();
    }
    let id = backend
        .current_turn_id()
        .expect("a live turn before the cancel");
    session.cancel().expect("cancel");
    now(backend.turn(&id)).expect("the record survives the cancel")
}

#[test]
#[cfg(feature = "test-mode")]
fn a_canceled_turn_is_never_rendered_as_a_completed_answer() {
    let kept = canceled_record(false);
    assert!(
        matches!(
            kept.lifecycle,
            AttemptLifecycle::Canceled { discarded: false }
        ),
        "{:?}",
        kept.lifecycle
    );
    // The trap this guards: `outcome` IS populated on a canceled turn.
    assert!(
        matches!(kept.outcome, Some(AnswerOutcome::Answered { .. })),
        "the fixture writes the surviving partial onto the outcome: {:?}",
        kept.outcome
    );
    assert_eq!(
        completed_answer(&kept),
        None,
        "but a canceled turn must never present as a finished answer"
    );
    let partial = canceled_partial(&kept).expect("the kept partial is readable");
    assert!(!partial.is_empty(), "the kept leg keeps real text");

    let discarded = canceled_record(true);
    assert!(
        matches!(
            discarded.lifecycle,
            AttemptLifecycle::Canceled { discarded: true }
        ),
        "{:?}",
        discarded.lifecycle
    );
    assert_eq!(
        discarded.outcome,
        Some(AnswerOutcome::Answered {
            text: String::new()
        }),
        "a discarded cancel still writes an outcome — an EMPTY one"
    );
    assert_eq!(completed_answer(&discarded), None);
    assert_eq!(
        canceled_partial(&discarded),
        None,
        "and an empty partial must not reach a renderer as an empty bubble"
    );
}

#[test]
#[cfg(feature = "test-mode")]
fn a_completed_turn_is_the_negative_control_for_the_cancel_gate() {
    use crate::components::ai_chat::{ChatRequest, ChatSession};

    let backend = InMemoryChatWorkspaceBackend::seeded();
    let transport = now(backend.open_session(
        "codex-spark",
        &KnowledgeSelection {
            posture: ChatPosture::Assistant,
            ..KnowledgeSelection::default()
        },
        Default::default(),
        ProviderTuning::default(),
    ))
    .expect("open");
    let mut session = ChatSession::new(transport);
    assert_eq!(
        backend.current_turn_id(),
        None,
        "no turn before the first send"
    );
    session
        .send(ChatRequest {
            prompt: "hello".to_owned(),
            attachments: Vec::new(),
            page_context: None,
        })
        .expect("send");
    let id = backend.current_turn_id().expect("a live turn");
    for _ in 0..20 {
        session.poll();
    }
    let record = now(backend.turn(&id)).expect("record");
    assert!(
        matches!(record.lifecycle, AttemptLifecycle::Completed(_)),
        "{:?}",
        record.lifecycle
    );
    assert!(
        completed_answer(&record).is_some_and(|t| !t.is_empty()),
        "a completed turn DOES present as an answer: {:?}",
        record.outcome
    );
    assert_eq!(
        canceled_partial(&record),
        None,
        "and never as a canceled partial"
    );
}

// ── Schema gating, metering, notices, session settings ──────────────────────

#[test]
fn a_cards_tuning_payload_carries_only_the_levers_its_schema_declares() {
    let groq = catalogue_card("groq-gpt-oss-120b");
    let codex = catalogue_card("codex-cli");
    let ollama = catalogue_card("ollama");
    assert!(groq.tuning.temperature && !groq.tuning.codex_levers);
    assert!(codex.tuning.codex_levers && !codex.tuning.temperature);
    assert_eq!(ollama.tuning, TuningSchema::default());
}

#[test]
fn engine_metering_policy_splits_plan_from_metered() {
    assert!(engine_is_metered("groq-gpt-oss-120b"));
    assert!(engine_is_metered("ollama"));
    assert!(!engine_is_metered("claude-code"));
    assert!(!engine_is_metered("codex-cli"));
    assert!(!engine_is_metered("codex-spark"));
}

#[test]
fn a_notice_becomes_localized_wording_with_a_stable_kind_id() {
    let en = AiChatWorkspaceTexts::default();
    let es = AiChatWorkspaceTexts::es();
    let escalated = TurnNotice::Escalated {
        from: ReasoningEffort::Medium,
        to: ReasoningEffort::Low,
    };
    let (kind, body) = notice_text(&escalated, &en);
    assert_eq!(kind, "escalated");
    assert!(body.contains("medium") && body.contains("low"), "{body}");
    assert!(!body.contains("{from}") && !body.contains("{to}"), "{body}");
    assert_ne!(
        notice_text(&escalated, &es).1,
        body,
        "the wording is localized; the kind id is not"
    );
    assert_eq!(notice_text(&escalated, &es).0, "escalated");
    assert_eq!(notice_text(&TurnNotice::Truncated, &en).0, "truncated");
    assert_eq!(
        notice_text(&TurnNotice::Unknown("throttled".into()), &en),
        ("unknown", "throttled".to_owned()),
        "an unnamed notice keeps the host's own code rather than being dropped"
    );
}

#[test]
fn a_cards_session_settings_follow_what_the_engine_can_emit() {
    let ollama = settings_for(&catalogue_card("ollama"));
    assert!(
        !ollama.show_thinking && !ollama.show_tool_calls,
        "ollama emits neither, so its session must not ask for either"
    );
    assert_eq!(ollama.model, None, "two models is a choice, not a pin");
    let claude = settings_for(&catalogue_card("claude-code"));
    assert!(claude.show_thinking && claude.show_tool_calls);
    assert_eq!(claude.model, None, "an empty model list is free text");
    let groq = settings_for(&catalogue_card("groq-gpt-oss-120b"));
    assert_eq!(groq.model.as_deref(), Some("openai/gpt-oss-120b"));
}

#[test]
#[cfg(feature = "test-mode")]
fn the_terminal_lifecycles_are_the_ones_a_turn_can_never_leave() {
    let reason = AssistantReason {
        code: "x".to_owned(),
        message: "y".to_owned(),
    };
    for l in [
        AttemptLifecycle::Admitted,
        AttemptLifecycle::Queued,
        AttemptLifecycle::Running,
        AttemptLifecycle::Validating,
    ] {
        assert!(!is_terminal(&l), "{l:?} is still moving");
    }
    for l in [
        // Built by the fixture rather than by hand: `Completed` carries a
        // whole `AssistantAnswer`, and a hand-rolled one would pin this test
        // to that struct's shape instead of to the lifecycle question.
        canceled_record(false).lifecycle,
        completed_lifecycle(),
        AttemptLifecycle::Denied {
            reason: reason.clone(),
        },
        AttemptLifecycle::Unavailable {
            reason: reason.clone(),
        },
        AttemptLifecycle::Failed {
            reason: reason.clone(),
        },
        AttemptLifecycle::Canceled { discarded: false },
        AttemptLifecycle::Interrupted {
            reason: reason.clone(),
        },
    ] {
        assert!(is_terminal(&l), "{l:?} is terminal");
    }
}

/// A real `Completed` lifecycle, straight off a finished fixture turn.
#[cfg(feature = "test-mode")]
fn completed_lifecycle() -> AttemptLifecycle {
    use crate::components::ai_chat::{ChatRequest, ChatSession};

    let backend = InMemoryChatWorkspaceBackend::seeded();
    let transport = now(backend.open_session(
        "codex-spark",
        &KnowledgeSelection {
            posture: ChatPosture::Assistant,
            ..KnowledgeSelection::default()
        },
        Default::default(),
        ProviderTuning::default(),
    ))
    .expect("open");
    let mut session = ChatSession::new(transport);
    session
        .send(ChatRequest {
            prompt: "hello".to_owned(),
            attachments: Vec::new(),
            page_context: None,
        })
        .expect("send");
    let id = backend.current_turn_id().expect("a live turn");
    for _ in 0..20 {
        session.poll();
    }
    now(backend.turn(&id)).expect("record").lifecycle
}

// ── The knowledge rail's scope identity ─────────────────────────────────────

#[test]
fn two_folders_get_two_distinct_scope_values() {
    let a = CorpusScope::Folder("kb/intake".to_owned());
    let b = CorpusScope::Folder("kb/court".to_owned());
    assert_eq!(
        a.as_id(),
        b.as_id(),
        "as_id collapses every folder to one word, which is why it cannot be the select value"
    );
    assert_ne!(scope_value(&a), scope_value(&b));
    assert_eq!(scope_value(&CorpusScope::All), "all");
    let texts = AiChatWorkspaceTexts::default();
    assert_eq!(scope_label(&a, &texts), "kb/intake");
    assert_eq!(scope_label(&CorpusScope::All, &texts), texts.scope_all);
}
