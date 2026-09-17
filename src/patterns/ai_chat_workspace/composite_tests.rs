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

// ── An assistant-only verdict gates the evidence beside it ──────────────────

#[test]
#[cfg(feature = "test-mode")]
fn assistant_only_turn_with_contradictory_evidence_renders_no_citations_or_facts() {
    use crate::components::ai_assistant_workspace::AssistantFact;
    use crate::components::ai_chat::{ChatRequest, ChatSession, Citation};

    // A host can hand the rail the CONTRADICTORY pair: an assistant-only
    // verdict — "General assistance, no documents" — beside citations and
    // qualified facts as though the turn had read the corpus. The fixture
    // never produces this pair itself (its transport consults no corpus for
    // an assistant turn), so this script writes it onto the turn directly.
    let backend = InMemoryChatWorkspaceBackend::seeded().with_script(
        "claude-code",
        PromptMatcher::Contains("contradiction".into()),
        TurnScript::new()
            .text_words("answered from general knowledge")
            .evidence(TurnEvidence {
                citations: vec![Citation {
                    label: "Intake checklist".into(),
                    href: Some("kb/intake/checklist.md".into()),
                }],
                recall: None,
                facts: vec![
                    AssistantFact::new(
                        "Reminder lead time",
                        Some("Two days ahead".into()),
                        None,
                        "Quoted from kb/intake/checklist.md; the memo itself is the only source.",
                        None,
                    )
                    .expect("a well-formed fact"),
                ],
                limitations: vec![],
                as_of: None,
                grounding: GroundingVerdict::AssistantOnly,
            }),
    );
    let transport = now(backend.open_session(
        "claude-code",
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
            prompt: "a contradiction for the rail".to_owned(),
            attachments: Vec::new(),
            page_context: None,
        })
        .expect("send");
    let id = backend.current_turn_id().expect("a live turn");
    for _ in 0..20 {
        session.poll();
    }
    let record = now(backend.turn(&id)).expect("record");

    // Positive control: the contradictory evidence really did reach the
    // record — an empty fixture would pass the gate assertions vacuously.
    let evidence = record
        .evidence
        .clone()
        .expect("the script attached evidence");
    assert_eq!(evidence.grounding, GroundingVerdict::AssistantOnly);
    assert!(!evidence.citations.is_empty(), "{evidence:?}");
    assert!(!evidence.facts.is_empty(), "{evidence:?}");

    // The rail-side gate: an assistant-only verdict means NO corpus was
    // consulted, so neither citations nor qualified facts may be rendered
    // no matter what the evidence carries. `evidence_of` deliberately stays
    // verbatim — the grounding badge itself is read from it.
    assert_eq!(citations_of(Some(&record)), vec![]);
    assert_eq!(facts_of(Some(&record)), vec![]);
    assert!(
        evidence_of(Some(&record)).is_some(),
        "the verdict itself must still reach the grounding hook"
    );

    // The other verdicts are NOT empty, so the rail keeps showing grounded
    // evidence — the gate narrows on the assistant-only verdict only.
    for grounded in [
        GroundingVerdict::Grounded { sources: 1 },
        GroundingVerdict::NotFound,
    ] {
        let grounded_record = TurnRecord {
            evidence: Some(TurnEvidence {
                grounding: grounded.clone(),
                ..record.evidence.clone().expect("evidence")
            }),
            ..record.clone()
        };
        assert!(
            !citations_of(Some(&grounded_record)).is_empty(),
            "{grounded:?}"
        );
        assert!(!facts_of(Some(&grounded_record)).is_empty(), "{grounded:?}");
    }
}

/// The composite's tick, reduced to the two things that decide what the
/// header shows: which turn id it resolves, and the record it reads for it.
/// Returns the last record the tick managed to read, exactly as the header's
/// `turn` signal would hold it.
#[cfg(feature = "test-mode")]
fn drive_ticks(
    backend: &InMemoryChatWorkspaceBackend,
    session: &mut crate::components::ai_chat::ChatSession,
    watched: &mut Option<String>,
    ticks: usize,
) -> Option<TurnRecord> {
    let mut seen: Option<TurnRecord> = None;
    for _ in 0..ticks {
        // The panel's transport poll and the composite's record tick are
        // independent intervals; one beat of each is the tightest interleave
        // the browser can produce, and the slowest to hide a latch.
        session.poll();
        let Some(id) = tick_turn_id(backend.current_turn_id(), watched.as_deref()) else {
            continue;
        };
        *watched = Some(id.clone());
        if let Ok(record) = now(backend.turn(&id)) {
            seen = Some(record);
        }
    }
    seen
}

/// A turn that FINISHES must still be readable by the composite's tick.
///
/// The bug this pins: `current_turn_id()` stops reporting a turn the instant
/// it goes terminal, in the same locked call that writes `Completed` onto the
/// record — so there is no window in which the live id and the terminal
/// record coexist, and a tick keyed only on the live id can NEVER see a
/// finished turn. The header latched at `validating` forever, with no usage,
/// no outcome and no evidence, on a turn that had answered correctly.
#[test]
#[cfg(feature = "test-mode")]
fn the_tick_still_reads_a_turn_that_has_just_gone_terminal() {
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

    let record =
        drive_ticks(&backend, &mut session, &mut None, 40).expect("the tick read some record");
    assert!(
        matches!(record.lifecycle, AttemptLifecycle::Completed(_)),
        "the last record the tick could read is {:?}, not Completed — the \
         header latches there and the honesty surface never lights up",
        record.lifecycle
    );
    assert!(
        record.outcome.is_some(),
        "a completed turn's outcome must reach the header: {record:?}"
    );
    assert!(
        record.usage.is_some(),
        "a completed turn's usage must reach the header: {record:?}"
    );
    assert!(
        record.evidence.is_some(),
        "a completed turn's evidence must reach the header: {record:?}"
    );
}

/// The same latch, on the cancel path: `cancel()` marks the turn terminal in
/// the call that records `Canceled`, so the notice the composite announces
/// once per turn id was unreachable too.
#[test]
#[cfg(feature = "test-mode")]
fn the_tick_still_reads_a_turn_that_has_just_been_canceled() {
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
    // Enough ticks for a real prefix to stream, then stop it mid-answer.
    let mut watched: Option<String> = None;
    let _ = drive_ticks(&backend, &mut session, &mut watched, 6);
    session.cancel().expect("cancel");

    let record =
        drive_ticks(&backend, &mut session, &mut watched, 4).expect("the tick read some record");
    assert!(
        matches!(record.lifecycle, AttemptLifecycle::Canceled { .. }),
        "the last record the tick could read is {:?}, not Canceled",
        record.lifecycle
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
#[cfg(feature = "test-mode")]
fn the_usage_line_reports_the_turn_the_figures_beside_it_report() {
    // claude-code is NOT metered by `engine_is_metered`, but its seeded
    // script is priced, so the fixture accumulates every token onto the
    // METERED side. Selecting a bucket by engine id therefore rendered
    // `0 in · 0 cached · 0 out · 0 rsn` for a turn whose own figures say 908
    // tokens — the subtitle and the hooks beside it describing one turn and
    // disagreeing about it.
    let record = turn_record(None, "the intake checklist conflict check");
    let figures = usage_figures(&record).expect("the seeded script reports usage");
    assert!(!engine_is_metered("claude-code"));
    assert_eq!(figures.plan_tokens, 0, "the plan side really is empty");
    assert_eq!(figures.metered_tokens, 812 + 96);

    let line = usage_line(&record).expect("a turn with usage has a usage line");
    assert!(
        line.contains("96 out"),
        "the line must report the output the figures report: {line:?}"
    );
    assert!(line.contains("41 rsn"), "and the reasoning: {line:?}");
    assert!(line.contains("812 in"), "and the input: {line:?}");
    assert!(
        !line.starts_with("0 in"),
        "a priced turn must never read as an empty one: {line:?}"
    );

    // The negative control on the other side of the split: codex-cli reports
    // no cost, so its tokens land on the PLAN side, and the same line must
    // follow them there rather than to a fixed bucket.
    let plan = {
        use crate::components::ai_chat::{ChatRequest, ChatSession};
        let backend = InMemoryChatWorkspaceBackend::seeded();
        let transport = now(backend.open_session(
            "codex-cli",
            &KnowledgeSelection {
                posture: ChatPosture::Assistant,
                ..KnowledgeSelection::default()
            },
            Default::default(),
            ProviderTuning::default(),
        ))
        .expect("open codex-cli");
        let mut session = ChatSession::new(transport);
        session
            .send(ChatRequest {
                prompt: "anything".to_owned(),
                attachments: Vec::new(),
                page_context: None,
            })
            .expect("send");
        let id = backend.current_turn_id().expect("a live turn");
        for _ in 0..80 {
            session.poll();
        }
        now(backend.turn(&id)).expect("record")
    };
    let plan_line = usage_line(&plan).expect("codex reports usage too");
    assert!(plan_line.contains("44 out"), "{plan_line:?}");
    assert!(plan_line.contains("240 in"), "{plan_line:?}");

    // A turn with no usage has no line at all, rather than a row of zeroes.
    assert_eq!(
        usage_line(&TurnRecord {
            usage: None,
            ..record
        }),
        None
    );
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

// ── Fix round 1 ─────────────────────────────────────────────────────────────

/// I1: only an engine switch announces one.
///
/// The pre-fix code used "a session already exists" as the discriminator, so
/// every knowledge-rail change reopened the session and announced
/// `"Switched to {engine}"` for an engine that had not changed. Breaking this
/// (making `reopen_announces_switch` return `true` for `KnowledgeChange`)
/// fails the second assertion.
#[test]
fn only_an_engine_switch_announces_a_switch() {
    assert!(reopen_announces_switch(ReopenReason::EngineSwitch));
    assert!(
        !reopen_announces_switch(ReopenReason::KnowledgeChange),
        "a corpus/posture/query-mode change rebuilds the session but does NOT \
         change the engine, so the engine-switch wording would be false"
    );
    assert!(
        !reopen_announces_switch(ReopenReason::Boot),
        "a boot has no conversation to have reset"
    );
    // And the wording it would have used really does name an engine, which is
    // why announcing it on a knowledge change is a lie rather than noise.
    let en = AiChatWorkspaceTexts::default();
    assert!(
        en.switched_engine.contains("{engine}"),
        "{}",
        en.switched_engine
    );
    let es = AiChatWorkspaceTexts::es();
    assert!(
        es.switched_engine.contains("{engine}"),
        "{}",
        es.switched_engine
    );
}

/// I2: an unchosen reasoning effort selects the engine-default option, never
/// a level.
///
/// With three options and nothing selected a browser displays the FIRST one,
/// so the pre-fix control read "low" while `to_tuning` sent `None`. Breaking
/// this (returning `Some(0)` for `None`) fails the first assertion.
#[test]
fn an_unchosen_reasoning_effort_selects_no_level() {
    assert_eq!(
        effort_selection(None),
        None,
        "nothing chosen must select the leading engine-default option, not `low`"
    );
    assert_eq!(effort_selection(Some(&ReasoningEffort::Low)), Some(0));
    assert_eq!(effort_selection(Some(&ReasoningEffort::Medium)), Some(1));
    assert_eq!(effort_selection(Some(&ReasoningEffort::High)), Some(2));
    assert_eq!(
        effort_selection(Some(&ReasoningEffort::Unknown("blistering".into()))),
        None,
        "a level this vocabulary cannot offer falls back to engine-default \
         rather than silently selecting a level it is not"
    );
    // `TuningDraft::seed_from` leaves `effort` at `None` for every card, so the
    // `None` case above is the one a user meets on a freshly opened popover,
    // not a hypothetical. That draft holds signals and needs a reactive owner,
    // so the rendered consequence is proved in the browser lane instead —
    // `capabilities_drive_the_settings_form_shape` reads the effort select's
    // options and its selected value.
}

/// I3: the watchdog fires once per turn, never once per tick and never once
/// per watchdog period.
///
/// The pre-fix code cleared the watched entry, which re-seeded it on the next
/// tick (`current_turn_id()` is unchanged — `fail_turn` acts on the
/// `ChatSession`, not the backend) and reset the notice counter to zero,
/// replaying every notice as a fresh annotation forever. Breaking this
/// (dropping the `already_fired` gate) fails the third assertion.
#[test]
fn the_watchdog_fires_once_per_turn() {
    assert!(
        watchdog_should_fire(WATCHDOG_MS, false, false),
        "the boundary tick fires: the comparison is inclusive"
    );
    assert!(
        !watchdog_should_fire(WATCHDOG_MS - 1, false, false),
        "one millisecond short does not"
    );
    assert!(
        !watchdog_should_fire(WATCHDOG_MS * 10, false, true),
        "a turn the watchdog already failed is never failed again, however \
         long it stays current"
    );
    assert!(
        !watchdog_should_fire(WATCHDOG_MS * 10, true, false),
        "a turn that finished is never failed after the fact"
    );
    assert!(
        !watchdog_should_fire(0, false, false),
        "and a turn that just started is not failed on its first tick"
    );
}

/// I4: no rail control is labelled with one of its own options.
///
/// Pre-fix, all three were: the corpus select read "This folder", the
/// query-mode select "Fused", the posture select "Grounded in this folder".
/// A screen-reader user heard what was currently chosen and nothing about
/// what the control decides. Breaking this (pointing any label back at an
/// option field) fails that control's assertion, in both locales.
#[test]
fn knowledge_rail_label_is_never_one_of_its_own_options() {
    for texts in [AiChatWorkspaceTexts::default(), AiChatWorkspaceTexts::es()] {
        let scope_options = vec![
            texts.scope_none.clone(),
            texts.scope_file.clone(),
            texts.scope_folder.clone(),
            texts.scope_all.clone(),
            texts.scope_custom.clone(),
        ];
        let mode_options = vec![
            texts.query_full_text.clone(),
            texts.query_similarity.clone(),
            texts.query_llm.clone(),
            texts.query_fused.clone(),
        ];
        let posture_options = vec![
            texts.posture_grounded.clone(),
            texts.posture_assistant.clone(),
        ];
        for (label, options, which) in [
            (&texts.corpus_scope_label, &scope_options, "corpus scope"),
            (&texts.query_mode_label, &mode_options, "query mode"),
            (&texts.posture_label, &posture_options, "posture"),
        ] {
            assert!(
                label_is_distinct_from_options(label, options),
                "the {which} label {label:?} repeats one of its own options \
                 ({options:?}) in locale {:?}",
                texts.locale_id
            );
        }
        // The helper itself has to be able to fail, or the loop above proves
        // nothing.
        assert!(!label_is_distinct_from_options(
            &texts.query_fused,
            &mode_options
        ));
        assert!(!label_is_distinct_from_options("  ", &mode_options));
    }
}

/// I5: every visible string the settings rows render comes from the text
/// table, and every one of them is translated.
#[test]
fn every_settings_row_string_is_translated() {
    let en = AiChatWorkspaceTexts::default();
    let es = AiChatWorkspaceTexts::es();
    let pairs = [
        ("effort_label", &en.effort_label, &es.effort_label),
        (
            "effort_engine_default",
            &en.effort_engine_default,
            &es.effort_engine_default,
        ),
        (
            "temperature_label",
            &en.temperature_label,
            &es.temperature_label,
        ),
        (
            "codex_web_search",
            &en.codex_web_search,
            &es.codex_web_search,
        ),
        (
            "codex_suppress_plugins",
            &en.codex_suppress_plugins,
            &es.codex_suppress_plugins,
        ),
        (
            "codex_disable_code_mode",
            &en.codex_disable_code_mode,
            &es.codex_disable_code_mode,
        ),
        ("codex_no_mcp", &en.codex_no_mcp, &es.codex_no_mcp),
        ("credential_note", &en.credential_note, &es.credential_note),
    ];
    for (name, e, sp) in pairs {
        assert!(!e.trim().is_empty(), "{name} is empty in EN");
        assert!(!sp.trim().is_empty(), "{name} is empty in ES");
        assert_ne!(e, sp, "{name} is identical in EN and ES");
    }
    // And the levers really read from the table rather than from a literal.
    for lever in CodexLever::ALL {
        assert_ne!(
            lever.label(&en),
            lever.label(&es),
            "{} is not localized",
            lever.as_id()
        );
    }
}

/// M1 (folded in): a pinned card publishes exactly its pin.
///
/// `AiChat` derives the Model row's shape from `Capabilities::models` alone,
/// so a card that pins one of seven models has to publish one, or the panel
/// offers all seven on an engine the host pinned. Browser test 1 covers the
/// rendered result; this covers the projection itself.
#[test]
fn a_pinned_card_publishes_exactly_its_pinned_model() {
    let spark = catalogue_card("codex-spark");
    assert_eq!(
        spark.capabilities.models.len(),
        7,
        "the raw card still carries the whole catalogue"
    );
    let published = published_capabilities(&spark);
    assert_eq!(
        published.models,
        vec!["gpt-5.3-codex-spark".to_owned()],
        "but what reaches the panel is the pin"
    );
    assert_eq!(
        settings_for(&spark).model.as_deref(),
        Some("gpt-5.3-codex-spark"),
        "and the session opens on it"
    );
    // A discovered list is still a list of choices, so it must NOT collapse.
    let ollama = catalogue_card("ollama");
    assert_eq!(
        published_capabilities(&ollama).models,
        ollama.capabilities.models,
        "a discovery timestamp is freshness, not a pin"
    );
    // Neither does a plain fixed catalogue.
    let codex = catalogue_card("codex-cli");
    assert_eq!(
        published_capabilities(&codex).models.len(),
        7,
        "an unpinned catalogue is untouched"
    );
}

// ── P5: honesty, outcome, failure shape and usage ───────────────────────────

/// Drive one turn on claude-code to a terminal lifecycle and return its
/// record. `script` of `None` leaves the engine's own seeded script in place.
///
/// Uses the REAL fixture rather than a hand-built `TurnRecord`: the point of
/// every assertion below is what a host's own pipeline produces, and a
/// literal record would let a wrong fixture and a wrong renderer agree.
#[cfg(feature = "test-mode")]
fn turn_record(script: Option<TurnScript>, prompt: &str) -> TurnRecord {
    use crate::components::ai_chat::{ChatRequest, ChatSession};

    let mut backend = InMemoryChatWorkspaceBackend::seeded();
    if let Some(script) = script {
        backend = backend.with_script("claude-code", PromptMatcher::Any, script);
    }
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
            prompt: prompt.to_owned(),
            attachments: Vec::new(),
            page_context: None,
        })
        .expect("send");
    let id = backend.current_turn_id().expect("a live turn after send");
    for _ in 0..80 {
        session.poll();
    }
    now(backend.turn(&id)).expect("the record survives the turn")
}

#[test]
fn not_enabled_and_failed_are_two_different_answers_to_two_different_questions() {
    // The account withheld the effect: nothing was attempted.
    let (tier, reason) = honesty_for(true, None, None, false);
    assert_eq!(tier, "not_enabled");
    assert_eq!(reason.as_deref(), Some("tier_effects_disabled"));

    // Something was attempted and broke. Different hook value AND different
    // tone — a proof that compared only the hook could pass while both
    // states looked identical on screen.
    let failed = TurnRecord {
        id: "t".into(),
        engine_id: "claude-code".into(),
        lifecycle: AttemptLifecycle::Failed {
            reason: AssistantReason {
                code: "engine_error".into(),
                message: "the engine stopped".into(),
            },
        },
        usage: None,
        tokens_per_sec: None,
        evidence: None,
        outcome: None,
        notices: vec![],
    };
    let (state, code) = honesty_for(false, None, Some(&failed), false);
    assert_eq!(state, "failed");
    assert_eq!(code.as_deref(), Some("engine_error"));
    assert_ne!(honesty_tone("not_enabled"), honesty_tone("failed"));

    // And a tier denial outranks the engine's own verdict: a broken card
    // under a denied tier still reads `not_enabled`, because the account is
    // the thing the actor has to fix first.
    let (state, code) = honesty_for(true, None, Some(&failed), false);
    assert_eq!(state, "not_enabled");
    assert_eq!(code.as_deref(), Some("tier_effects_disabled"));
}

#[test]
fn every_availability_code_has_a_state_and_the_three_credential_ones_differ() {
    let codes = [
        AvailabilityReasonCode::TierEffectsDisabled,
        AvailabilityReasonCode::NotArmed,
        AvailabilityReasonCode::CliMissing,
        AvailabilityReasonCode::CliVersionUnsupported,
        AvailabilityReasonCode::CliVersionUntested,
        AvailabilityReasonCode::CliProtocolUnsupported,
        AvailabilityReasonCode::CredentialKeyUnavailable,
        AvailabilityReasonCode::NotSignedIn,
        AvailabilityReasonCode::SignInExpired,
        AvailabilityReasonCode::BudgetExhausted,
        AvailabilityReasonCode::EngineBusy,
        AvailabilityReasonCode::SignInShapeUnavailable,
        AvailabilityReasonCode::EngineProcessNotRunning,
        AvailabilityReasonCode::ModelNotInstalled,
        AvailabilityReasonCode::Unknown("future".into()),
    ];
    let texts = AiChatWorkspaceTexts::default();
    for code in &codes {
        let state = honesty_state_for_code(code);
        assert!(!state.is_empty(), "{code:?} has no honesty state");
        assert!(
            !honesty_tone(state).is_empty(),
            "{state} has no tone classes"
        );
        assert!(
            !texts.availability_reason(code).trim().is_empty(),
            "{code:?} has no copy"
        );
        let (reported, reason) = honesty_for(false, Some(code), None, false);
        assert_eq!(reported, state);
        assert_eq!(reason.as_deref(), Some(code.as_code()));
    }
    // The three credential fixes are three different actions, so they must
    // not collapse into one state.
    let credential = [
        honesty_state_for_code(&AvailabilityReasonCode::CredentialKeyUnavailable),
        honesty_state_for_code(&AvailabilityReasonCode::NotSignedIn),
        honesty_state_for_code(&AvailabilityReasonCode::SignInExpired),
    ];
    let mut sorted = credential.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 3, "{credential:?}");
    // And a local runtime that is not running is not the same problem as a
    // model that is not installed, even though both read `unavailable`.
    assert_ne!(
        AvailabilityReasonCode::EngineProcessNotRunning.as_code(),
        AvailabilityReasonCode::ModelNotInstalled.as_code()
    );
    assert_ne!(
        texts.reason_engine_process_not_running,
        texts.reason_model_not_installed
    );
}

#[test]
fn the_two_new_local_runtime_codes_round_trip_and_ask_for_no_credential() {
    for code in [
        AvailabilityReasonCode::EngineProcessNotRunning,
        AvailabilityReasonCode::ModelNotInstalled,
    ] {
        assert_eq!(AvailabilityReasonCode::parse(code.as_code()), code);
        assert!(
            !code.is_credential_failure(),
            "{code:?} is an environment fact, never a stale credential"
        );
        assert_eq!(
            code.next_action(),
            crate::components::ai_assistant_workspace::RefusalNextAction::RetryLater,
            "neither is fixed in a settings pane"
        );
    }
}

#[test]
#[cfg(feature = "test-mode")]
fn a_declined_answer_is_a_healthy_completion_that_lists_its_limitations() {
    let record = turn_record(
        Some(TurnScript::new().declined(
            vec![
                "The court calendar was not indexed.".to_owned(),
                "No filing deadline was published.".to_owned(),
            ],
            "2026-01-01T00:00:00Z",
        )),
        "decline",
    );
    assert_eq!(lifecycle_id(&record.lifecycle), "completed");
    assert_eq!(outcome_id(&record), Some("declined"));
    assert_eq!(
        failure_kind(&record),
        None,
        "a decline is not a failure and must never carry a failure kind"
    );
    assert_eq!(declined_limitations(&record).len(), 2);
    let (state, _) = honesty_for(false, None, Some(&record), false);
    assert_eq!(
        state, "ready",
        "declining healthily is not an honesty fault"
    );
}

#[test]
#[cfg(feature = "test-mode")]
fn a_truncated_completion_never_reads_as_a_finished_one() {
    let truncated = turn_record(
        Some(TurnScript::new().text_words("half an answer").truncated()),
        "truncate",
    );
    assert_eq!(lifecycle_id(&truncated.lifecycle), "completed");
    assert_eq!(outcome_id(&truncated), Some("truncated"));
    assert_eq!(declined_limitations(&truncated).len(), 0);

    // Negative control: the same script without the truncation mark.
    let whole = turn_record(
        Some(TurnScript::new().text_words("a whole answer")),
        "whole",
    );
    assert_eq!(outcome_id(&whole), Some("completed"));
}

#[test]
#[cfg(feature = "test-mode")]
fn a_failure_kind_is_the_records_own_code_and_untested_is_not_expired() {
    let untested = turn_record(
        Some(TurnScript::new().fail(AvailabilityReasonCode::CliVersionUntested)),
        "fail",
    );
    assert_eq!(failure_kind(&untested), Some("cli_version_untested"));
    assert_eq!(outcome_id(&untested), Some("unavailable"));
    assert!(
        !AvailabilityReasonCode::CliVersionUntested.is_credential_failure(),
        "an untested CLI version is an environment fact"
    );

    // The negative control on the same shape: an expired sign-in IS a
    // credential failure and carries a different code entirely.
    let expired = turn_record(
        Some(TurnScript::new().fail(AvailabilityReasonCode::SignInExpired)),
        "expired",
    );
    assert_eq!(failure_kind(&expired), Some("sign_in_expired"));
    assert_ne!(failure_kind(&untested), failure_kind(&expired));
    assert!(AvailabilityReasonCode::SignInExpired.is_credential_failure());

    // A crash is `Failed`, not `Unavailable`: we tried and it broke, rather
    // than it refusing to run.
    let errored = turn_record(Some(TurnScript::new().error("the engine stopped")), "error");
    assert_eq!(failure_kind(&errored), Some("engine_error"));
    assert_eq!(outcome_id(&errored), Some("failed"));
}

#[test]
#[cfg(feature = "test-mode")]
fn usage_keeps_plan_and_metered_apart_while_reasoning_stays_inside_output() {
    // claude-code's own seeded script reports a PRICED turn, so every token
    // lands on the metered side and the plan side stays empty.
    let record = turn_record(None, "the intake checklist conflict check");
    let figures = usage_figures(&record).expect("the seeded script reports usage");
    assert_eq!(figures.metered_tokens, 812 + 96);
    assert_eq!(
        figures.plan_tokens, 0,
        "nothing leaks across the split: {figures:?}"
    );
    assert_eq!(figures.reasoning_tokens, 41);
    assert_eq!(figures.output_tokens, 96);
    assert!(
        figures.reasoning_tokens <= figures.output_tokens,
        "{figures:?}"
    );

    // The negative control on the other side of the split: codex-cli's
    // script reports no cost, so the SAME four hooks move to the plan side
    // and the metered side is the empty one.
    let plan = {
        use crate::components::ai_chat::{ChatRequest, ChatSession};
        let backend = InMemoryChatWorkspaceBackend::seeded();
        let transport = now(backend.open_session(
            "codex-cli",
            &KnowledgeSelection {
                posture: ChatPosture::Assistant,
                ..KnowledgeSelection::default()
            },
            Default::default(),
            ProviderTuning::default(),
        ))
        .expect("open codex-cli");
        let mut session = ChatSession::new(transport);
        session
            .send(ChatRequest {
                prompt: "anything".to_owned(),
                attachments: Vec::new(),
                page_context: None,
            })
            .expect("send");
        let id = backend.current_turn_id().expect("a live turn");
        for _ in 0..80 {
            session.poll();
        }
        now(backend.turn(&id)).expect("record")
    };
    let plan_figures = usage_figures(&plan).expect("codex reports usage too");
    assert_eq!(plan_figures.plan_tokens, 240 + 44);
    assert_eq!(plan_figures.metered_tokens, 0);
    assert!(plan_figures.reasoning_tokens <= plan_figures.output_tokens);

    // A turn with no usage publishes nothing rather than zeroes, which would
    // read as "this turn cost nothing".
    let empty = TurnRecord {
        usage: None,
        ..record.clone()
    };
    assert_eq!(usage_figures(&empty), None);
}

#[test]
fn a_cancel_notice_tells_the_actor_which_cancel_they_got() {
    let texts = AiChatWorkspaceTexts::default();
    let kept = cancel_notice(&AttemptLifecycle::Canceled { discarded: false }, &texts)
        .expect("a kept cancel is announced");
    let discarded = cancel_notice(&AttemptLifecycle::Canceled { discarded: true }, &texts)
        .expect("a discarded cancel is announced");
    assert_ne!(kept, discarded, "the two cancels leave different things");
    assert_eq!(
        cancel_notice(&AttemptLifecycle::Running, &texts),
        None,
        "a turn that was not canceled announces no cancel"
    );
}

#[test]
fn the_watchdog_failure_kind_is_its_own_thing() {
    assert_eq!(WATCHDOG_FAILURE_KIND, "watchdog");
    assert_eq!(
        AvailabilityReasonCode::parse(WATCHDOG_FAILURE_KIND),
        AvailabilityReasonCode::Unknown("watchdog".into()),
        "the watchdog is not an availability reason: nothing said the engine \
         was unavailable, this workspace simply stopped waiting"
    );
    assert_eq!(
        header_failure_kind(None, true).as_deref(),
        Some("watchdog"),
        "a timed-out turn reports a failure even though its own record is \
         still Running — which is exactly why the flag cannot live on the \
         record"
    );
    assert_eq!(header_failure_kind(None, false), None);
    let (state, reason) = honesty_for(false, None, None, true);
    assert_eq!(state, "failed");
    assert_eq!(reason.as_deref(), Some("watchdog"));
}

#[test]
fn a_workspace_refusal_derives_its_next_step_and_promises_nothing_without_one() {
    use crate::components::ai_assistant_workspace::RefusalNextAction;

    let keyless = WorkspaceRefusal {
        kind: ChatWorkspaceErrorKind::Refused,
        code: Some(AvailabilityReasonCode::CredentialKeyUnavailable),
        message: "no key".into(),
        engine_id: Some("groq-gpt-oss-120b".into()),
    };
    assert_eq!(keyless.next_action(), RefusalNextAction::OpenSettings);
    assert_eq!(keyless.kind.as_str(), "refused");

    let untyped = WorkspaceRefusal {
        kind: ChatWorkspaceErrorKind::Network,
        code: None,
        message: "no answer".into(),
        engine_id: None,
    };
    assert_eq!(
        untyped.next_action(),
        RefusalNextAction::NewConversation,
        "with nothing identified, the honest offer is the one that neither \
         promises a retry will help nor implies settings can fix it"
    );
    assert_eq!(untyped.kind.as_str(), "network");
}

#[test]
fn every_error_kind_round_trips_its_wire_string() {
    let all = [
        ChatWorkspaceErrorKind::Unavailable,
        ChatWorkspaceErrorKind::Refused,
        ChatWorkspaceErrorKind::NotFound,
        ChatWorkspaceErrorKind::Network,
        ChatWorkspaceErrorKind::Upstream,
        ChatWorkspaceErrorKind::Unsupported,
    ];
    let mut codes: Vec<&str> = Vec::new();
    for kind in &all {
        assert_eq!(
            ChatWorkspaceErrorKind::parse(kind.as_str()).as_ref(),
            Some(kind)
        );
        codes.push(kind.as_str());
    }
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), all.len(), "each kind needs its own string");
    assert_eq!(ChatWorkspaceErrorKind::parse("Refused"), None);
}

#[test]
#[cfg(feature = "test-mode")]
fn a_watchdog_failure_never_renders_as_the_actor_pressing_stop() {
    // `ChatSession::fail_turn` reaches the transport through `cancel()`, so
    // the host's record of a timed-out turn comes back CANCELED. Reading
    // that record naively publishes `outcome=canceled` and
    // `cancel-discarded=false` — telling the actor they stopped a turn they
    // never touched.
    let canceled = canceled_record(false);
    assert!(matches!(
        canceled.lifecycle,
        AttemptLifecycle::Canceled { discarded: false }
    ));

    assert_eq!(
        header_outcome_id(Some(&canceled), true),
        Some("failed"),
        "the watchdog owns the outcome of a turn it failed"
    );
    assert_eq!(
        header_cancel_discarded(Some(&canceled), true),
        None,
        "and publishes no cancel at all, rather than `false`"
    );

    // The negative control on the same record: an actor's own cancel, with
    // the watchdog silent, still reports exactly what it is.
    assert_eq!(header_outcome_id(Some(&canceled), false), Some("canceled"));
    assert_eq!(header_cancel_discarded(Some(&canceled), false), Some(false));
    let discarded = canceled_record(true);
    assert_eq!(
        header_cancel_discarded(Some(&discarded), false),
        Some(true),
        "and the two cancels stay distinguishable"
    );
}
