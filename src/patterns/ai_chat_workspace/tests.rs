use super::*;

use crate::components::ai_assistant_workspace::{
    AssistantAccess, AssistantCapabilities, AssistantCapability, AssistantContractError,
    AssistantFact, AssistantMemoryEntry, AssistantPreferences, AssistantReason, AssistantSettings,
    KnowledgeState, MemoryClass, MemoryState, RefusalNextAction, engine_ready_for_ask,
};
use crate::components::ai_chat::{Capabilities, Usage};

#[test]
fn availability_reason_codes_round_trip_and_preserve_unknown() {
    let codes = [
        "tier_effects_disabled",
        "not_armed",
        "cli_missing",
        "cli_version_unsupported",
        "cli_version_untested",
        "cli_protocol_unsupported",
        "credential_key_unavailable",
        "not_signed_in",
        "sign_in_expired",
        "budget_exhausted",
        "engine_busy",
        "sign_in_shape_unavailable",
    ];
    for code in codes {
        let parsed = AvailabilityReasonCode::parse(code);
        assert!(
            !matches!(parsed, AvailabilityReasonCode::Unknown(_)),
            "{code} parsed as Unknown"
        );
        assert_eq!(parsed.as_code(), code);
    }

    let unknown = AvailabilityReasonCode::parse("something_new_from_a_future_host");
    assert!(matches!(unknown, AvailabilityReasonCode::Unknown(_)));
    assert_eq!(unknown.as_code(), "something_new_from_a_future_host");
}

#[test]
fn cli_version_untested_is_not_an_expired_credential() {
    let code = AvailabilityReasonCode::CliVersionUntested;
    assert!(!code.is_credential_failure());
    assert_ne!(code.next_action(), RefusalNextAction::OpenSettings);

    for texts in [AiChatWorkspaceTexts::default(), AiChatWorkspaceTexts::es()] {
        let copy = texts.availability_reason(&code).to_lowercase();
        for forbidden in ["sign", "credential", "expired"] {
            assert!(
                !copy.contains(forbidden),
                "{copy:?} must not mention {forbidden:?}"
            );
        }
    }
}

#[test]
fn desktop_catalogue_matches_engine_truth() {
    let cards = desktop_provider_catalogue();
    assert_eq!(cards.len(), 5);

    let claude = cards
        .iter()
        .find(|c| c.engine.id == "claude-code")
        .expect("claude-code card");
    assert!(claude.capabilities.models.is_empty());
    assert_eq!(claude.capabilities.permission_modes.len(), 4);
    assert_eq!(claude.model_source, ModelSource::FreeText);

    let codex = cards
        .iter()
        .find(|c| c.engine.id == "codex-cli")
        .expect("codex-cli card");
    assert_eq!(codex.capabilities.models.len(), 7);
    assert_eq!(codex.model_source, ModelSource::Fixed);

    let groq = cards
        .iter()
        .find(|c| c.engine.id == "groq-gpt-oss-120b")
        .expect("groq card");
    assert!(groq.capabilities.needs_api_key);

    let ollama = cards
        .iter()
        .find(|c| c.engine.id == "ollama")
        .expect("ollama card");
    assert!(!ollama.capabilities.supports_tool_calls);

    let spark = cards
        .iter()
        .find(|c| c.engine.id == "codex-spark")
        .expect("codex-spark card");
    assert_eq!(
        spark.model_source,
        ModelSource::Pinned("gpt-5.3-codex-spark".into())
    );

    // Every fixture engine must actually be selectable through the real
    // `engine_ready_for_ask` gate, not merely carry plausible-looking
    // fields: a card with no `Ask` grant fails readiness silently, which
    // `AssistantCapabilities::allows` never surfaces as an error.
    for card in &cards {
        let settings = permissive_settings_for(card);
        let ready = engine_ready_for_ask(&settings, "actor-1");
        assert_eq!(
            ready.map(|e| e.id.as_str()),
            Some(card.engine.id.as_str()),
            "{} must be ready for Ask under a permissive, matching settings projection",
            card.engine.id
        );
    }
}

/// A minimal but fully valid `AssistantSettings` selecting exactly this
/// card's engine, granting `Ask` at both the settings and engine level, so
/// `engine_ready_for_ask` exercises the real gate rather than a stub.
fn permissive_settings_for(card: &ProviderCard) -> AssistantSettings {
    AssistantSettings {
        owner_actor_id: "actor-1".into(),
        accepted_revision: 1,
        proposed_revision: 1,
        accepted: AssistantPreferences {
            engine_id: Some(card.engine.id.clone()),
            memory_use: false,
            memory_capture: false,
        },
        proposed: AssistantPreferences::default(),
        reasoning_tier: AssistantAccess::Granted,
        engines: vec![card.engine.clone()],
        budget: None,
        capabilities: AssistantCapabilities {
            granted: vec![AssistantCapability::Ask],
            details: vec![],
        },
    }
}

fn capabilities_fixture(models: Vec<String>) -> Capabilities {
    Capabilities {
        id: "x".into(),
        label: "X".into(),
        needs_api_key: false,
        models,
        permission_modes: vec![],
        supports_thinking: false,
        supports_tool_calls: false,
    }
}

#[test]
fn model_source_derives_from_capabilities() {
    let empty = capabilities_fixture(vec![]);
    assert_eq!(
        ModelSource::from_capabilities(&empty, Some("pinned"), Some("2020")),
        ModelSource::FreeText,
        "no models always means free text, even with a pin/discovery hint"
    );

    let with_models = capabilities_fixture(vec!["a".into()]);
    assert_eq!(
        ModelSource::from_capabilities(&with_models, None, None),
        ModelSource::Fixed
    );
    assert_eq!(
        ModelSource::from_capabilities(&with_models, Some("a"), Some("2020")),
        ModelSource::Pinned("a".into()),
        "an explicit pin outranks a discovery timestamp"
    );
    assert_eq!(
        ModelSource::from_capabilities(&with_models, None, Some("2020")),
        ModelSource::Discovered {
            as_of: "2020".into()
        }
    );
}

fn evidence_fixture() -> TurnEvidence {
    TurnEvidence {
        citations: vec![],
        recall: None,
        facts: vec![],
        limitations: vec![],
        as_of: None,
        grounding: GroundingVerdict::AssistantOnly,
    }
}

#[test]
fn turn_evidence_fails_closed_on_an_empty_qualification() {
    let good = AssistantFact::new("Balance", Some("$100".into()), None, "as reported", None);
    let bad = AssistantFact::new("Balance", Some("$100".into()), None, "", None);
    assert!(good.is_ok());
    assert!(bad.is_err(), "an empty qualification must be rejected");

    let err = evidence_fixture()
        .with_facts(vec![good.clone(), bad])
        .unwrap_err();
    assert_eq!(err, AssistantContractError::InvalidFact);

    let ok = evidence_fixture().with_facts(vec![good]).unwrap();
    assert_eq!(ok.facts.len(), 1);
}

#[test]
fn usage_totals_never_sum_plan_and_metered() {
    let mut totals = UsageTotals::default();
    totals.add_plan(&Usage {
        output_tokens: 20,
        ..Default::default()
    });
    totals.add_plan(&Usage {
        output_tokens: 5,
        ..Default::default()
    });
    totals.add_metered(
        &Usage {
            output_tokens: 100,
            ..Default::default()
        },
        Some(1.0),
    );

    assert_eq!(
        totals.plan.output_tokens, 25,
        "plan accumulates across calls"
    );
    assert_eq!(
        totals.metered.output_tokens, 100,
        "metered stays in its own field, never folded into plan"
    );
}

#[test]
fn absent_cost_is_none_not_zero() {
    let mut totals = UsageTotals::default();
    assert_eq!(totals.metered_cost, None);
    assert!(!totals.cost_incomplete);

    totals.add_metered(&Usage::default(), None);
    assert_eq!(
        totals.metered_cost, None,
        "an absent cost stays None, never gets promoted to a reported zero"
    );
    assert!(totals.cost_incomplete);

    totals.add_metered(&Usage::default(), Some(2.5));
    assert_eq!(totals.metered_cost, Some(2.5));

    totals.add_metered(&Usage::default(), Some(0.5));
    assert_eq!(
        totals.metered_cost,
        Some(3.0),
        "two reported costs are summed"
    );

    // absent then reported: the flag stays set even though a real number is
    // now showing — that number is a floor, not the whole story, because an
    // earlier contribution's cost was never learned.
    assert!(
        totals.cost_incomplete,
        "one unreported contribution makes the running total incomplete forever, \
         even once later contributions do report a cost"
    );

    // reported then absent: a total that STARTS complete must not stay
    // looking complete once a later contribution comes in unpriced.
    let mut totals = UsageTotals::default();
    totals.add_metered(&Usage::default(), Some(5.0));
    assert_eq!(totals.metered_cost, Some(5.0));
    assert!(!totals.cost_incomplete);

    totals.add_metered(&Usage::default(), None);
    assert_eq!(
        totals.metered_cost,
        Some(5.0),
        "the known partial sum is kept, not discarded"
    );
    assert!(
        totals.cost_incomplete,
        "a later unpriced contribution must flip an already-complete total to incomplete"
    );
}

#[test]
fn tokens_per_sec_is_none_without_elapsed_time() {
    assert_eq!(UsageTotals::tokens_per_sec(100, 0), None);
    assert_eq!(UsageTotals::tokens_per_sec(100, -5), None);
    assert_eq!(UsageTotals::tokens_per_sec(100, 2_000), Some(50.0));
}

#[test]
fn guardrail_refuses_matter_email_phone_in_that_order_and_accepts_prose() {
    assert_eq!(
        guardrail_refusal("please check matter 24-8842 for me"),
        Some(MemoryRefusal::ContainsMatterNumber)
    );
    assert_eq!(
        guardrail_refusal("open ticket ZW-2301 today"),
        Some(MemoryRefusal::ContainsMatterNumber)
    );
    assert_eq!(
        guardrail_refusal("reach me at chris@example.com please"),
        Some(MemoryRefusal::ContainsEmail)
    );
    assert_eq!(
        guardrail_refusal("call me at 555-867-5309"),
        Some(MemoryRefusal::ContainsPhone)
    );
    assert_eq!(
        guardrail_refusal("I prefer concise answers in the morning"),
        None
    );

    assert_eq!(
        guardrail_refusal("matter 24-8842, email chris@example.com"),
        Some(MemoryRefusal::ContainsMatterNumber),
        "matter number wins over an email in the same text"
    );
    assert_eq!(
        guardrail_refusal("email chris@example.com, call 555-867-5309"),
        Some(MemoryRefusal::ContainsEmail),
        "email wins over a phone in the same text"
    );
}

#[test]
fn en_and_es_texts_are_complete_and_differ() {
    let en = AiChatWorkspaceTexts::default();
    let es = AiChatWorkspaceTexts::es();
    let en_fields = en.fields();
    let es_fields = es.fields();

    assert_eq!(en_fields.len(), AiChatWorkspaceTexts::FIELD_COUNT);
    assert_eq!(es_fields.len(), AiChatWorkspaceTexts::FIELD_COUNT);

    // Every field was deliberately translated; nothing shares copy between
    // EN and ES, so this allow-list stays empty.
    let allowed_identical: &[&str] = &[];

    for ((name, en_value), (es_name, es_value)) in en_fields.iter().zip(es_fields.iter()) {
        assert_eq!(name, es_name, "fields() must list names in the same order");
        assert!(!en_value.trim().is_empty(), "{name} is empty in EN");
        assert!(!es_value.trim().is_empty(), "{name} is empty in ES");
        if !allowed_identical.contains(name) {
            assert_ne!(
                en_value, es_value,
                "{name} is identical in EN and ES and not on the allow-list"
            );
        }
    }

    assert_eq!(
        en.grounded_not_found, "I could not find that in this folder.",
        "grounded_not_found's EN copy is pinned exactly"
    );
}

/// Guards against an unaccented-ASCII regression in the Spanish table (fix
/// round 1 of the P1 review): a prior draft translated every string but
/// dropped every diacritic (`sesion` instead of `sesion` with an accent,
/// etc.), which reads as a typo-riddled machine transliteration rather than
/// real Spanish. Checks both a structural floor (a real count of non-ASCII
/// characters across the table) and specific words that are simply wrong
/// without their accent.
#[test]
fn es_texts_carry_real_spanish_diacritics() {
    let es = AiChatWorkspaceTexts::es();

    let non_ascii_total: usize = es
        .fields()
        .iter()
        .map(|(_, value)| value.chars().filter(|c| !c.is_ascii()).count())
        .sum();
    assert!(
        non_ascii_total >= 40,
        "expected real Spanish diacritics across the table, found only \
         {non_ascii_total} non-ASCII characters"
    );

    assert!(es.sign_in.contains("sesión"), "{:?}", es.sign_in);
    assert!(es.sign_out.contains("sesión"), "{:?}", es.sign_out);
    assert!(
        es.reason_not_armed.contains("aún"),
        "{:?}",
        es.reason_not_armed
    );
    assert!(
        es.reason_cli_version_untested.contains("versión"),
        "{:?}",
        es.reason_cli_version_untested
    );
    assert!(
        es.switched_engine.contains("cambió"),
        "{:?}",
        es.switched_engine
    );
    assert!(
        es.action_retry_later.contains("Inténtalo"),
        "{:?}",
        es.action_retry_later
    );
    assert!(
        es.recall_words.contains("Búsqueda"),
        "{:?}",
        es.recall_words
    );
    assert!(
        es.refused_matter.contains("número") && es.refused_matter.contains("guardó"),
        "{:?}",
        es.refused_matter
    );
}

#[test]
fn knowledge_ids_round_trip() {
    assert_eq!(CorpusScope::File("a.md".into()).as_id(), "file");
    assert_eq!(CorpusScope::Folder("docs".into()).as_id(), "folder");
    assert_eq!(CorpusScope::All.as_id(), "all");
    assert_eq!(
        CorpusScope::Custom {
            label: "x".into(),
            paths: vec![]
        }
        .as_id(),
        "custom"
    );

    assert_eq!(IngestPhase::Idle.as_id(), "idle");
    assert_eq!(IngestPhase::Walking.as_id(), "walking");
    assert_eq!(IngestPhase::Indexing.as_id(), "indexing");
    assert_eq!(IngestPhase::Clustering.as_id(), "clustering");
    assert_eq!(IngestPhase::Ready.as_id(), "ready");
    assert_eq!(
        IngestPhase::Failed(AssistantReason::default()).as_id(),
        "failed"
    );

    for (mode, id) in [
        (CorpusQueryMode::FullText, "full_text"),
        (CorpusQueryMode::Similarity, "similarity"),
        (CorpusQueryMode::Llm, "llm"),
        (CorpusQueryMode::Fused, "fused"),
    ] {
        assert_eq!(mode.as_id(), id);
        assert_eq!(CorpusQueryMode::parse(id), Some(mode));
    }
    assert_eq!(CorpusQueryMode::parse("nope"), None);

    for (posture, id) in [
        (ChatPosture::Grounded, "grounded"),
        (ChatPosture::Assistant, "assistant"),
    ] {
        assert_eq!(posture.as_id(), id);
        assert_eq!(ChatPosture::parse(id), Some(posture));
    }
    assert_eq!(ChatPosture::parse("nope"), None);

    assert_eq!(
        OfficeKnowledgeScope::GroupImportant.as_id(),
        "group_important"
    );
    assert_eq!(OfficeKnowledgeScope::Foundation.as_id(), "foundation");
    assert_eq!(
        OfficeKnowledgeScope::parse("group_important"),
        OfficeKnowledgeScope::GroupImportant
    );
    assert_eq!(
        OfficeKnowledgeScope::parse("foundation"),
        OfficeKnowledgeScope::Foundation
    );
    let unknown = OfficeKnowledgeScope::parse("mystery");
    assert_eq!(unknown.as_id(), "mystery");
    assert!(matches!(unknown, OfficeKnowledgeScope::Unknown(_)));
}

#[test]
fn governance_unknown_arms_are_reachable() {
    let class = MemoryClass::Unknown("a-future-class".into());
    assert!(matches!(class, MemoryClass::Unknown(ref s) if s == "a-future-class"));

    let state = KnowledgeState::Unknown("a-future-state".into());
    assert!(matches!(state, KnowledgeState::Unknown(ref s) if s == "a-future-state"));
}

/// The shipped fixture must stay a pure in-process scripted backend: it may
/// never reach for the live SSE bridge, a host path helper, or the network.
/// Scanning the source is the only check that fails when someone adds one,
/// because such a reference compiles perfectly well. The needles live in this
/// file rather than inside `memory.rs` so the test cannot match itself.
#[test]
#[cfg(feature = "test-mode")]
fn memory_rs_never_references_the_live_bridge() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/patterns/ai_chat_workspace/memory.rs"
    );
    let source = std::fs::read_to_string(path).expect("the fixture source is readable");
    for needle in [
        "SseBridgeTransport",
        "paths::",
        "use_event_source_fetch",
        "http://",
        "fetch(",
    ] {
        assert!(
            !source.contains(needle),
            "memory.rs must not reference {needle}"
        );
    }
}

// ── P6: knowledge vocabularies, the ingest ladder and the curation gap ───────

/// Every id P6 renders into a `data-*` hook comes from an `as_str`/`as_id`,
/// never from `{:?}`. The asymmetry between the three `Unknown` arms is the
/// point of this test, not an accident of it.
#[test]
fn knowledge_string_conversions_round_trip() {
    for (corpus, id) in [
        (RecallCorpus::Words, "words"),
        (RecallCorpus::Meaning, "meaning"),
        (RecallCorpus::Graph, "graph"),
        (RecallCorpus::Thread, "thread"),
    ] {
        assert_eq!(corpus.as_str(), id);
        assert_eq!(RecallCorpus::parse(id), corpus);
    }
    // An unnamed lane keeps the host's id, so a hit still says WHICH lane.
    let lane = RecallCorpus::parse("lexical_v2");
    assert_eq!(lane.as_str(), "lexical_v2");
    assert!(matches!(lane, RecallCorpus::Unknown(_)));

    for (refusal, id) in [
        (
            MemoryRefusal::ContainsMatterNumber,
            "contains_matter_number",
        ),
        (MemoryRefusal::ContainsEmail, "contains_email"),
        (MemoryRefusal::ContainsPhone, "contains_phone"),
        (MemoryRefusal::CuratedKindOnly, "curated_kind_only"),
    ] {
        assert_eq!(refusal.as_str(), id);
        assert_eq!(MemoryRefusal::parse(id), refusal);
    }
    // The guardrail id set is CLOSED: a host reason this vocabulary does not
    // name collapses to `unknown` rather than leaking a string a proof
    // written for a typed guardrail would then match.
    let host = MemoryRefusal::Unknown("policy_denied".into());
    assert_eq!(host.as_str(), "unknown");
    assert_eq!(
        MemoryRefusal::parse("policy_denied"),
        MemoryRefusal::Unknown("policy_denied".into())
    );

    for (class, id) in [
        (MemoryClass::Language, "language"),
        (MemoryClass::Tone, "tone"),
        (MemoryClass::Verbosity, "verbosity"),
        (MemoryClass::WorkingMethod, "working_method"),
    ] {
        assert_eq!(class.as_str(), id);
        assert_eq!(MemoryClass::parse(id), class);
    }
    // The two curated classes must round-trip through their host ids, or the
    // kind select could not offer the choice that earns the refusal.
    for id in ["lesson", "principle"] {
        assert_eq!(MemoryClass::parse(id).as_str(), id);
    }

    for (state, id) in [
        (MemoryState::Candidate, "candidate"),
        (MemoryState::Confirmed, "confirmed"),
        (MemoryState::Withdrawn, "withdrawn"),
    ] {
        assert_eq!(state.as_str(), id);
        assert_eq!(MemoryState::parse(id), state);
    }

    // The verdict id must not carry the source COUNT, or a proof could never
    // assert on the verdict itself.
    assert_eq!(
        GroundingVerdict::Grounded { sources: 1 }.as_id(),
        "grounded"
    );
    assert_eq!(
        GroundingVerdict::Grounded { sources: 9 }.as_id(),
        "grounded"
    );
    assert_eq!(GroundingVerdict::NotFound.as_id(), "not_found");
    assert_eq!(GroundingVerdict::AssistantOnly.as_id(), "assistant_only");
}

fn corpus_with(scope: CorpusScope, phase: IngestPhase) -> KnowledgeSource {
    KnowledgeSource::Corpus {
        scope,
        ingest: IngestStatus {
            phase,
            files_seen: 3,
            files_indexed: 3,
            clusters: 2,
            as_of: "2027-01-01".into(),
        },
        query_mode: CorpusQueryMode::Fused,
        posture: ChatPosture::Grounded,
    }
}

#[test]
fn ingest_in_flight_covers_the_three_working_phases_and_not_idle() {
    for phase in [
        IngestPhase::Walking,
        IngestPhase::Indexing,
        IngestPhase::Clustering,
    ] {
        let sources = vec![corpus_with(CorpusScope::All, phase.clone())];
        assert!(any_ingest_in_flight(&sources), "{phase:?} is in flight");
    }
    // Idle and the two terminal phases are NOT in flight: a poll loop keyed
    // on `Idle` would never stop.
    for phase in [
        IngestPhase::Idle,
        IngestPhase::Ready,
        IngestPhase::Failed(AssistantReason::default()),
    ] {
        let sources = vec![corpus_with(CorpusScope::All, phase.clone())];
        assert!(!any_ingest_in_flight(&sources), "{phase:?} is settled");
    }
}

#[test]
fn with_ingest_replaces_one_scope_and_leaves_its_neighbour_alone() {
    let intake = CorpusScope::Folder("kb/intake".into());
    let court = CorpusScope::Folder("kb/court".into());
    let sources = vec![
        corpus_with(intake.clone(), IngestPhase::Ready),
        corpus_with(court.clone(), IngestPhase::Ready),
        KnowledgeSource::MemoryStore {
            enabled: true,
            capture_enabled: true,
        },
    ];
    let walking = IngestStatus {
        phase: IngestPhase::Walking,
        files_seen: 3,
        files_indexed: 0,
        clusters: 0,
        as_of: "2027-01-02".into(),
    };
    let next = with_ingest(sources, &intake, walking);
    let phases: Vec<&'static str> = corpus_ingests(&next)
        .iter()
        .map(|(_, s)| s.phase.as_id())
        .collect();
    // Two folders share `CorpusScope::as_id() == "folder"`, so a replacement
    // keyed on the id alone would hit both. This one is keyed on the scope.
    assert_eq!(phases, vec!["walking", "ready"]);
    assert_eq!(
        memory_flags(&next),
        Some((true, true)),
        "a non-corpus source is carried through untouched"
    );
    let _ = court;
}

#[test]
fn every_offered_memory_class_is_addressable_and_includes_the_curated_pair() {
    let choices = memory_class_choices();
    let ids: Vec<&str> = choices.iter().map(MemoryClass::as_str).collect();
    assert!(ids.contains(&"lesson"), "{ids:?}");
    assert!(ids.contains(&"principle"), "{ids:?}");
    for class in &choices {
        assert_eq!(
            MemoryClass::parse(class.as_str()).as_str(),
            class.as_str(),
            "every offered class survives the select's round trip"
        );
    }
}

#[test]
fn a_candidate_entry_is_the_one_that_awaits_curation() {
    let entry = |state: MemoryState| AssistantMemoryEntry {
        id: "mem-0001".into(),
        revision: 1,
        class: MemoryClass::Tone,
        text: "Keep it short.".into(),
        state,
        capabilities: AssistantCapabilities {
            granted: vec![AssistantCapability::MemoryConfirm],
            details: vec![],
        },
        retention: None,
    };
    assert!(awaits_curation(&entry(MemoryState::Candidate)));
    assert!(!awaits_curation(&entry(MemoryState::Confirmed)));
    assert!(!awaits_curation(&entry(MemoryState::Withdrawn)));
}

/// P4 left the knowledge-change reopen deliberately silent and pinned that
/// silence in its browser proof, so that whoever gave the reopen a sentence
/// had to change the assertion rather than route around it. This is that
/// sentence: three reasons, three answers, and the knowledge one names no
/// engine because no engine changed.
#[test]
fn reopen_notice_names_the_engine_only_for_an_engine_switch() {
    let t = AiChatWorkspaceTexts::default();
    assert_eq!(reopen_notice(ReopenReason::Boot, "Claude Code", &t), None);

    let switched =
        reopen_notice(ReopenReason::EngineSwitch, "Claude Code", &t).expect("a switch announces");
    assert!(switched.contains("Claude Code"), "{switched}");
    assert!(!switched.contains("{engine}"), "{switched}");

    let knowledge = reopen_notice(ReopenReason::KnowledgeChange, "Claude Code", &t)
        .expect("a knowledge change announces its reset");
    assert_eq!(knowledge, t.knowledge_changed);
    assert!(
        !knowledge.contains("Claude Code"),
        "a knowledge change must not name an engine that did not change: {knowledge}"
    );
    assert_ne!(knowledge, switched);

    // And the same three answers in Spanish, or the notice is a literal.
    let es = AiChatWorkspaceTexts::es();
    assert_ne!(
        reopen_notice(ReopenReason::KnowledgeChange, "Claude Code", &es),
        reopen_notice(ReopenReason::KnowledgeChange, "Claude Code", &t)
    );
}

/// Every vocabulary the knowledge rail renders has localized copy, and an
/// `Unknown` arm shows the host's own id rather than a blank.
#[test]
fn knowledge_labels_are_localized_and_never_blank() {
    for t in [AiChatWorkspaceTexts::default(), AiChatWorkspaceTexts::es()] {
        for phase in [
            IngestPhase::Idle,
            IngestPhase::Walking,
            IngestPhase::Indexing,
            IngestPhase::Clustering,
            IngestPhase::Ready,
            IngestPhase::Failed(AssistantReason::default()),
        ] {
            assert!(!t.ingest_phase_label(&phase).trim().is_empty(), "{phase:?}");
        }
        for mode in [
            CorpusQueryMode::FullText,
            CorpusQueryMode::Similarity,
            CorpusQueryMode::Llm,
            CorpusQueryMode::Fused,
        ] {
            assert!(!t.query_mode_name(mode).trim().is_empty(), "{mode:?}");
        }
        for corpus in [
            RecallCorpus::Words,
            RecallCorpus::Meaning,
            RecallCorpus::Graph,
            RecallCorpus::Thread,
        ] {
            assert!(!t.recall_corpus_label(&corpus).trim().is_empty());
        }
        assert_eq!(
            t.recall_corpus_label(&RecallCorpus::Unknown("lexical_v2".into())),
            "lexical_v2",
            "an unnamed lane is never narrated as one of the four named ones"
        );
        for refusal in [
            MemoryRefusal::ContainsMatterNumber,
            MemoryRefusal::ContainsEmail,
            MemoryRefusal::ContainsPhone,
            MemoryRefusal::CuratedKindOnly,
            MemoryRefusal::Unknown("policy_denied".into()),
        ] {
            assert!(!t.memory_refusal_label(&refusal).trim().is_empty());
        }
        for class in memory_class_choices() {
            assert!(!t.memory_class_label(&class).trim().is_empty());
        }
        for verdict in [
            GroundingVerdict::Grounded { sources: 1 },
            GroundingVerdict::NotFound,
            GroundingVerdict::AssistantOnly,
        ] {
            assert!(!t.grounding_label(&verdict).trim().is_empty());
        }
        let counts = t.ingest_counts_line(&IngestStatus {
            phase: IngestPhase::Ready,
            files_seen: 6,
            files_indexed: 4,
            clusters: 3,
            as_of: "2027-01-01".into(),
        });
        assert!(
            counts.contains('6') && counts.contains('4') && counts.contains('3'),
            "{counts}"
        );
        assert!(
            !counts.contains('{'),
            "every placeholder is substituted: {counts}"
        );
    }
}

/// The showcase's own copy must never imply that anything EXTRACTS memories.
/// The extraction jobs exist and have never run against either database, so
/// a workspace that hinted at them would be describing a capability the
/// actor does not have.
#[test]
fn no_text_claims_memories_are_extracted_automatically() {
    for t in [AiChatWorkspaceTexts::default(), AiChatWorkspaceTexts::es()] {
        for (name, value) in t.fields() {
            let lower = value.to_lowercase();
            for needle in ["extract", "automatic", "autom\u{e1}tic"] {
                assert!(
                    !lower.contains(needle),
                    "{name} implies extraction or automation: {value:?}"
                );
            }
        }
    }
}

/// Every text field this crate compares BYTE FOR BYTE against what the
/// markdown renderer produced, with the field name a failure should print.
///
/// Membership is the whole point, so it is stated rather than inferred.
/// A field belongs here when BOTH are true: something asserts it with
/// equality (not `contains`) against rendered DOM text, AND the element it
/// lands in is rendered through `crate::markdown::MarkdownView`.
///
/// * `grounded_not_found` qualifies: `ai_chat_knowledge_smoke`'s test 4
///   asserts the whole trimmed answer bubble equals it, and an assistant
///   message is markdown (`is_markdown(&msg.role)` in
///   `components/ai_chat/component.rs`).
/// * `knowledge_changed` is byte-pinned too (`ai_chat_showcase_smoke`'s
///   test 9) but is deliberately ABSENT: a `TranscriptAnnotation` renders
///   through `AnnotationRow`'s plain `<span class="whitespace-pre-wrap">`,
///   never through markdown, so nothing substitutes anything in it. Adding
///   it would forbid punctuation that is perfectly safe there. See
///   [`BYTE_PINNED_NON_MARKDOWN_FIELD_NAMES`].
/// * Every other browser assertion on copy is `contains`, which survives a
///   substitution and therefore needs no guard.
///
/// The name list is the source of truth; [`markdown_pinned_fields`] looks
/// values up through `fields()` so a rename fails here instead of silently
/// pinning a stale string. [`markdown_pinned_fields_tracks_browser_byte_pins`]
/// mechanically cross-checks the browser suite so a new byte-exact pin cannot
/// land without updating one of the two lists.
const MARKDOWN_PINNED_FIELD_NAMES: &[&str] = &["grounded_not_found"];

/// Browser (or unit) byte-pins that deliberately do **not** go through
/// markdown, so they must stay out of [`MARKDOWN_PINNED_FIELD_NAMES`].
const BYTE_PINNED_NON_MARKDOWN_FIELD_NAMES: &[&str] = &["knowledge_changed"];

fn markdown_pinned_fields(t: &AiChatWorkspaceTexts) -> Vec<(&'static str, &str)> {
    let table = t.fields();
    MARKDOWN_PINNED_FIELD_NAMES
        .iter()
        .map(|name| {
            let value = table
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| *v)
                .unwrap_or_else(|| {
                    panic!(
                        "{name} is markdown-pinned but missing from AiChatWorkspaceTexts::fields()"
                    )
                });
            (*name, value)
        })
        .collect()
}

/// Why a candidate sentence cannot be compared byte-for-byte against its own
/// rendering, or `None` when it can.
///
/// `editmark-core` parses with `Options::ENABLE_SMART_PUNCTUATION`
/// (`editmark-core/src/layout.rs`), and pulldown-cmark's smart-punctuation
/// pass rewrites FIVE things, not one: single quotes, double quotes,
/// `...` to an ellipsis, `--` to an en dash and `---` to an em dash. The
/// first guard written here covered the quote families only, because the
/// defect that prompted it happened to be an apostrophe — which would have
/// left the next author to rediscover the same class eight minutes into a
/// browser lane. The rule, not the instance, is what belongs in a guard.
///
/// The second group is markdown STRUCTURE: characters the parser reads as
/// inline markup (so the text between them is emitted without them), and
/// line prefixes that make the sentence a heading, list item or quote rather
/// than a paragraph. Neither is substitution, but both change the rendered
/// text of a pinned sentence just as completely.
fn markdown_substitution_hazard(text: &str) -> Option<String> {
    // Rewritten by the smart-punctuation pass, or already its output.
    const SUBSTITUTED: [char; 9] = [
        '\'', '\u{2018}', '\u{2019}', '"', '\u{201c}', '\u{201d}', '\u{2026}', '\u{2013}',
        '\u{2014}',
    ];
    // Read as inline markup, so the delimiters do not survive to the DOM.
    const MARKUP: [char; 8] = ['*', '_', '`', '[', ']', '<', '>', '&'];

    for ch in SUBSTITUTED {
        if text.contains(ch) {
            return Some(format!("{ch:?} is rewritten by ENABLE_SMART_PUNCTUATION"));
        }
    }
    for ch in MARKUP {
        if text.contains(ch) {
            return Some(format!("{ch:?} is read as inline markdown markup"));
        }
    }
    for seq in ["...", "--"] {
        if text.contains(seq) {
            return Some(format!(
                "{seq:?} is rewritten by ENABLE_SMART_PUNCTUATION into a \
                 single character"
            ));
        }
    }
    let first = text.trim_start();
    for prefix in ["#", "- ", "* ", "+ ", "> ", "    "] {
        if first.starts_with(prefix) {
            return Some(format!(
                "a line starting {prefix:?} is parsed as a block, not a \
                 paragraph"
            ));
        }
    }
    let ordered = first
        .split_once(". ")
        .is_some_and(|(head, _)| !head.is_empty() && head.chars().all(|c| c.is_ascii_digit()));
    if ordered {
        return Some("a line starting \"<digits>. \" is parsed as an ordered list".to_owned());
    }
    None
}

/// Copy pinned byte-for-byte against the markdown renderer must contain
/// nothing that renderer rewrites.
///
/// The failure this converts is specific: a one-character copy edit passes
/// `cargo fmt`, `cargo clippy` and the whole unit lane, then fails
/// `test-ai-chat-knowledge` eight minutes later on a byte mismatch nobody can
/// see by reading either string. It is now a compile-time-fast unit failure
/// naming the character and the reason.
#[test]
fn markdown_pinned_copy_survives_smart_punctuation() {
    for t in [AiChatWorkspaceTexts::default(), AiChatWorkspaceTexts::es()] {
        for (name, value) in markdown_pinned_fields(&t) {
            assert!(
                markdown_substitution_hazard(value).is_none(),
                "{}: {name} = {value:?} \u{2014} {}",
                t.locale_id,
                markdown_substitution_hazard(value).unwrap_or_default()
            );
        }
    }

    // The guard's own positive control. Without it, a checker that quietly
    // stopped matching anything would look exactly like clean copy -- which
    // is the failure mode this whole test exists to prevent one level up.
    for bad in [
        "I couldn't find that in this folder.",
        "I could not find that \u{2019}here\u{2019}.",
        "I could not find that -- try another folder.",
        "I could not find that...",
        "I could not find that in *this* folder.",
        "# Not found",
        "- Not found",
        "1. Not found",
        "Not found <yet>",
        "Fish & chips were not found",
    ] {
        assert!(
            markdown_substitution_hazard(bad).is_some(),
            "the guard must reject {bad:?}"
        );
    }
    // ...and must not reject copy that is genuinely safe, or it would forbid
    // every sentence and prove nothing.
    for good in [
        "I could not find that in this folder.",
        "No pude encontrar eso en esta carpeta.",
        "Nada coincidi\u{f3} con esa b\u{fa}squeda.",
    ] {
        assert_eq!(markdown_substitution_hazard(good), None, "{good:?}");
    }

    // `knowledge_changed` is byte-pinned by the browser suite but renders
    // through a plain span, not markdown. Asserting its ABSENCE from the
    // pinned list keeps that distinction deliberate: if a later phase routes
    // annotations through markdown, this line is what has to change.
    let en = AiChatWorkspaceTexts::default();
    assert!(
        !markdown_pinned_fields(&en)
            .iter()
            .any(|(name, _)| *name == "knowledge_changed"),
        "annotations are not markdown; see markdown_pinned_fields"
    );
    assert!(
        BYTE_PINNED_NON_MARKDOWN_FIELD_NAMES.contains(&"knowledge_changed"),
        "knowledge_changed must stay on the non-markdown exclusion list"
    );
}

/// Every EN text value that a browser smoke suite pins with a string literal
/// must be declared in [`MARKDOWN_PINNED_FIELD_NAMES`] or
/// [`BYTE_PINNED_NON_MARKDOWN_FIELD_NAMES`].
///
/// Without this, a new `assert_eq!(bubble, "…")` against copy that flows
/// through `MarkdownView` can land without updating the smart-punctuation
/// guard — and the only failure is an eight-minute browser lane on a
/// one-character rewrite. The scan is the mechanical cross-check
/// `FIELD_COUNT` already has for field inventory.
#[test]
fn markdown_pinned_fields_tracks_browser_byte_pins() {
    let en = AiChatWorkspaceTexts::default();
    let pinned: std::collections::BTreeSet<&str> =
        MARKDOWN_PINNED_FIELD_NAMES.iter().copied().collect();
    let excluded: std::collections::BTreeSet<&str> = BYTE_PINNED_NON_MARKDOWN_FIELD_NAMES
        .iter()
        .copied()
        .collect();
    assert!(
        pinned.is_disjoint(&excluded),
        "a field cannot be both markdown-pinned and excluded: {:?}",
        pinned.intersection(&excluded).collect::<Vec<_>>()
    );

    // The function and the name list must agree exactly.
    let from_fn: std::collections::BTreeSet<&str> = markdown_pinned_fields(&en)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    assert_eq!(
        from_fn, pinned,
        "markdown_pinned_fields drifted from MARKDOWN_PINNED_FIELD_NAMES"
    );

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut smoke = String::new();
    for path in rust_sources(&root) {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name.starts_with("ai_chat_") && name.ends_with("_smoke.rs") {
            smoke.push_str(&std::fs::read_to_string(&path).expect("readable smoke source"));
            smoke.push('\n');
        }
    }
    assert!(
        !smoke.is_empty(),
        "found no tests/ai_chat_*_smoke.rs sources to cross-check"
    );

    let mut seen_literal: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for (name, value) in en.fields() {
        // Short labels ("en", "Ask", "Engine") appear as substrings all over
        // the smoke suites; only sentence-length copy is a byte-pin risk for
        // the smart-punctuation guard.
        if value.chars().count() < 24 {
            continue;
        }
        let literal = format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""));
        if smoke.contains(&literal) {
            seen_literal.insert(name);
            assert!(
                pinned.contains(name) || excluded.contains(name),
                "{name} appears as a byte-exact string literal in an ai-chat \
                 browser smoke suite but is neither markdown-pinned nor on the \
                 non-markdown exclusion list. Add it to \
                 MARKDOWN_PINNED_FIELD_NAMES (if MarkdownView renders it) or \
                 BYTE_PINNED_NON_MARKDOWN_FIELD_NAMES (if it does not)."
            );
        }
    }

    for name in &pinned {
        assert!(
            seen_literal.contains(name),
            "{name} is markdown-pinned but no ai-chat browser smoke suite \
             pins its EN value as a string literal anymore — remove it from \
             MARKDOWN_PINNED_FIELD_NAMES or restore the browser assert"
        );
    }
    for name in &excluded {
        assert!(
            seen_literal.contains(name),
            "{name} is on the non-markdown exclusion list but no ai-chat \
             browser smoke suite pins its EN value as a string literal \
             anymore — remove it from BYTE_PINNED_NON_MARKDOWN_FIELD_NAMES \
             or restore the browser assert"
        );
    }
}

/// Every quick action's prompt is real localized copy, not the same English
/// sentence in both tables.
///
/// The check is per ACTION rather than over the whole set: a table that
/// translated seven of eight would still differ overall, and the eighth
/// would ship English into a Spanish composer. `Translate` is included with
/// a language substituted, because its prompt is a template and a proof that
/// only compared templates would never exercise the substitution the bar
/// actually performs.
#[test]
fn quick_actions_prompts_localize() {
    let en = AiChatWorkspaceTexts::default();
    let es = AiChatWorkspaceTexts::es();
    let mut actions: Vec<QuickAction> = QuickAction::all_static().to_vec();
    actions.push(QuickAction::Translate {
        language: "portugués".to_owned(),
    });

    for action in &actions {
        let en_prompt = action.prompt(&en);
        let es_prompt = action.prompt(&es);
        assert!(
            !en_prompt.is_empty() && !es_prompt.is_empty(),
            "{} has an empty prompt",
            action.as_id()
        );
        assert_ne!(
            en_prompt,
            es_prompt,
            "{} renders the same sentence in both tables",
            action.as_id()
        );
        // No template placeholder may survive into a rendered prompt.
        assert!(
            !en_prompt.contains("{language}") && !es_prompt.contains("{language}"),
            "{} leaked its placeholder: {en_prompt:?} / {es_prompt:?}",
            action.as_id()
        );
    }

    // The substitution really used the language it was handed, in both
    // tables — a `replace` against the wrong placeholder would silently
    // leave the sentence intact and still differ EN from ES.
    let translate = QuickAction::Translate {
        language: "portugués".to_owned(),
    };
    assert!(translate.prompt(&en).contains("portugués"));
    assert!(translate.prompt(&es).contains("portugués"));

    // And every action's `texts_field` names a field that really exists, so
    // the `data-ai-chat-label` a chip publishes can be resolved against the
    // table by name.
    let names: Vec<&'static str> = en.fields().into_iter().map(|(name, _)| name).collect();
    for action in &actions {
        assert!(
            names.contains(&action.texts_field()),
            "{} claims the field {:?}, which is not on the struct",
            action.as_id(),
            action.texts_field()
        );
    }
}

/// The one file in the whole repo that is allowed to reach a live server,
/// named here so the exception is auditable rather than a hole.
///
/// P9 wired Live mode, so `demo/src/` can now reach `127.0.0.1:8099` where
/// before nothing could. Everything the exception buys is that the scan
/// below still fails for every OTHER page — including every page a browser
/// lane drives.
const LIVE_BACKEND_FILE: &str = "ai_chat_live.rs";

/// Markers that mean "this source can talk to the live server".
const LIVE_MARKERS: [&str; 4] = [
    "8099",
    "SseBridgeTransport",
    "use_event_source_fetch",
    "/api/chat",
];

/// Markers that mean "this source can REACH the live backend", even without
/// naming a URL itself.
///
/// [`LIVE_MARKERS`] catches a source that talks to the server directly; these
/// catch one that only has to mount the module that does. The showcase page
/// (`ai_chat.rs`) legitimately carries them — it owns the Connect button —
/// which is why they are checked against the LANE-DRIVEN sources only.
const LIVE_WIRE_MARKERS: [&str; 3] = [
    "ai_chat_live",
    "LiveChatWorkspaceBackend",
    "data-ai-chat-live-connect",
];

/// The page-scoped host binary the ai-chat (and other focused) browser lanes
/// actually compile. Lane-driven membership is derived from THIS file's own
/// `mod` / `#[path]` declarations — not from a filename heuristic — so a
/// newly-mounted fixture cannot slip past the live-wire rule by picking a
/// name that does not end in `_fixture.rs` (ldui-iilm.10 / I2).
const LANE_HOST_SOURCE: &str = "demo/src/client_snapshot_test_host.rs";

/// File names (basename only) the focused browser host compiles in, including
/// the host itself.
///
/// Deliberately NOT derived from `demo/src/main.rs`: that binary is the
/// catalog showcase and legitimately mounts Live mode. Computing the set
/// from the host's own module list is the shape that stays complete when a
/// fixture is added without quietly sweeping in the showcase page.
fn lane_driven_file_names() -> std::collections::BTreeSet<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let host = std::fs::read_to_string(root.join(LANE_HOST_SOURCE))
        .unwrap_or_else(|e| panic!("readable {LANE_HOST_SOURCE}: {e}"));
    let mut names = std::collections::BTreeSet::new();
    names.insert("client_snapshot_test_host.rs".to_owned());

    for line in host.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#[path = \"") {
            let path = rest.split('"').next().unwrap_or("");
            if let Some(file) = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
            {
                names.insert(file.to_owned());
            }
            continue;
        }
        // Bare `mod foo;` — skip `mod foo {` and anything already attributed
        // via `#[path]` on the previous line by only accepting the semicolon
        // form that names a sibling file under demo/src/.
        if let Some(rest) = trimmed.strip_prefix("mod ") {
            let name = rest.trim_end_matches(';').trim();
            if rest.contains('{') || name.is_empty() || name.contains(' ') {
                continue;
            }
            names.insert(format!("{name}.rs"));
        }
    }
    names
}

fn is_lane_driven(name: &str, lane_files: &std::collections::BTreeSet<String>) -> bool {
    lane_files.contains(name)
}

/// Every `.rs` file under one directory, recursively.
fn rust_sources(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(rust_sources(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            found.push(path);
        }
    }
    found
}

/// No browser lane in this repo may talk to the live SSE bridge.
///
/// Live mode is PRESENT in the showcase and the fixture — a choice an actor
/// can see and select — and reaching it takes a separate, explicit Connect.
/// The risk is that an edit wires a live call into a page a lane drives, at
/// which point the suite stops being reproducible and starts failing for
/// whoever does not happen to be running a server. A source scan is the only
/// check that can fail BEFORE such a lane is written; a network assertion
/// inside the harness could only fail after.
///
/// TWO trees are scanned, and the second is the one that matters now. The
/// lanes drive PAGES, and every page lives in `demo/src/` — a tree this
/// scan did not cover until P9, so wiring live mode into a lane-driven page
/// would have evaded it entirely. The single exception is
/// [`LIVE_BACKEND_FILE`], which IS the live backend.
#[test]
fn no_lane_touches_the_live_server() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lane_files = lane_driven_file_names();
    assert!(
        lane_files.len() >= 5,
        "host module sweep found too few lane-driven files ({lane_files:?}); \
         the parser is not reading {LANE_HOST_SOURCE}"
    );
    let mut scanned_tests = 0usize;
    let mut scanned_demo = 0usize;
    let mut lane_driven: Vec<String> = Vec::new();
    let mut exception_seen = false;

    for (dir, counter) in [
        (root.join("tests"), &mut scanned_tests),
        (root.join("demo").join("src"), &mut scanned_demo),
    ] {
        let sources = rust_sources(&dir);
        assert!(
            !sources.is_empty(),
            "{} matched no Rust sources, so the scan is reading nothing",
            dir.display()
        );
        for path in sources {
            let source = std::fs::read_to_string(&path).expect("readable source");
            *counter += 1;
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_owned();
            if name == LIVE_BACKEND_FILE {
                // The exception must stay REAL: a file that no longer
                // reaches the live server is a stale exception, and a stale
                // exception is how a hole survives a rename.
                exception_seen = true;
                assert!(
                    LIVE_MARKERS.iter().any(|m| source.contains(m)),
                    "{name} is the named live-server exception but names none \
                     of {LIVE_MARKERS:?}; remove the exception or the file"
                );
                continue;
            }
            for marker in LIVE_MARKERS {
                assert!(
                    !source.contains(marker),
                    "{name} names {marker:?}, so it can reach the live server. \
                     Only {LIVE_BACKEND_FILE} may; a lane drives pages in \
                     demo/src, and those must drive the fixture"
                );
            }
            // A lane-driven document may not even MOUNT the live backend:
            // the showcase page owns Live mode, and a fixture the suite
            // loads must offer no way to reach a server at all.
            if is_lane_driven(&name, &lane_files) {
                lane_driven.push(name.clone());
                for marker in LIVE_WIRE_MARKERS {
                    assert!(
                        !source.contains(marker),
                        "{name} is loaded by a browser lane and names \
                         {marker:?}, so the lane could reach a live server"
                    );
                }
            }
        }
    }

    // The scan's own positive controls: a walk that silently matched nothing
    // would pass this test forever while checking nothing at all.
    assert!(
        scanned_tests > 20,
        "only {scanned_tests} integration-test sources were scanned, so the \
         scan is not reading the suite it claims to cover"
    );
    assert!(
        scanned_demo > 100,
        "only {scanned_demo} demo sources were scanned, so the scan is not \
         reading the pages the lanes actually drive"
    );
    // Completeness, not presence: every module the host compiles must have
    // been recognised. A rename that stopped the parser matching would
    // otherwise leave this rule checking a stale subset.
    let recognised: std::collections::BTreeSet<_> = lane_driven.iter().cloned().collect();
    let missing: Vec<_> = lane_files.difference(&recognised).cloned().collect();
    assert!(
        missing.is_empty(),
        "host modules not recognised as lane-driven (parser incomplete): \
         {missing:?}; recognised={recognised:?}; expected={lane_files:?}"
    );
    assert!(
        exception_seen,
        "the named exception {LIVE_BACKEND_FILE} was never found, so the \
         scan is not covering the tree the live backend lives in"
    );
}
