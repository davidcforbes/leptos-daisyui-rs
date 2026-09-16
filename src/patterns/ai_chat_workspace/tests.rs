use super::*;

use crate::components::ai_assistant_workspace::{
    AssistantAccess, AssistantCapabilities, AssistantCapability, AssistantContractError,
    AssistantFact, AssistantPreferences, AssistantReason, AssistantSettings, KnowledgeState,
    MemoryClass, RefusalNextAction, engine_ready_for_ask,
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
        en.grounded_not_found, "I couldn't find that in this folder.",
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
