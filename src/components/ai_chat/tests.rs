use super::style::{
    ComposerAction, chat_state_attr, clamp_composer_height, composer_hint, composer_hint_for,
    composer_key_action, composer_placeholder_for, default_composer_placeholder,
    effective_assistant_label, is_markdown, is_thinking, role_avatar_bg, role_avatar_initial,
    role_avatar_initial_with, role_classes, role_data_attr, role_label, role_label_for,
    role_label_with, should_stick_to_bottom, show_welcome_chips,
};
use super::texts::AiChatTexts;
use super::types::{
    AnnotationAnchor, AnnotationBody, AnnotationKind, ModelFieldMode, RetryOutcome,
    TranscriptAnnotation, annotation_slots, format_allowed_tools, format_usage,
    format_usage_subtitle, model_field_mode, parse_allowed_tools, permission_selection,
    reconcile_model_for, retry_outcome, settings_from_form_fields, settings_rows_for,
};
use ai_chat_core::{
    Capabilities, ChatError, ChatRequest, ChatRole, ChatSession, ChatSettings, ChatTransport,
    StreamEvent, Usage,
};

#[test]
fn every_role_sits_start_for_full_width_rows() {
    // Bubbles span the full row (the .lds-aichat .chat override in theme.rs
    // stretches them to width:100%), so no role sits on the right. Colour, not
    // side, distinguishes the speaker -- see user_bubble_is_primary.
    for r in [
        ChatRole::User,
        ChatRole::Assistant,
        ChatRole::System,
        ChatRole::Thinking,
        ChatRole::Tool,
    ] {
        assert_eq!(
            role_classes(&r).0,
            "chat-start",
            "{r:?} should be chat-start"
        );
    }
}

#[test]
fn user_bubble_is_primary() {
    assert_eq!(role_classes(&ChatRole::User).1, "chat-bubble-primary");
    assert_eq!(role_classes(&ChatRole::Assistant).1, "");
}

#[test]
fn labels_match_roles() {
    assert_eq!(role_label(&ChatRole::User), "You");
    assert_eq!(role_label(&ChatRole::Assistant), "Claude");
}

/// beads-xqmi (parity with ai-chat-engine em-c7w1): assistant bubbles
/// attribute to the CONFIGURED backend's label — a Codex session reads as
/// Codex — while an empty label keeps every historical default.
#[test]
fn assistant_attribution_follows_the_configured_backend() {
    assert_eq!(
        role_label_with(&ChatRole::Assistant, "Codex CLI (OpenAI)"),
        "Codex CLI (OpenAI)"
    );
    assert_eq!(role_label_with(&ChatRole::Assistant, ""), "Claude");
    assert_eq!(role_label_with(&ChatRole::User, "Codex"), "You");

    assert_eq!(role_avatar_initial_with(&ChatRole::Assistant, "codex"), "C");
    assert_eq!(role_avatar_initial_with(&ChatRole::Assistant, ""), "C");
    assert_eq!(
        role_avatar_initial_with(&ChatRole::Tool, "Codex"),
        "\u{2699}"
    );

    assert_eq!(
        default_composer_placeholder("Codex"),
        "Ask Codex about this document\u{2026}"
    );
    assert_eq!(
        default_composer_placeholder(""),
        "Ask Claude about this document\u{2026}"
    );
}

#[test]
fn only_user_and_assistant_render_markdown() {
    assert!(is_markdown(&ChatRole::User));
    assert!(is_markdown(&ChatRole::Assistant));
    assert!(!is_markdown(&ChatRole::Thinking));
    assert!(!is_markdown(&ChatRole::Tool));
    assert!(!is_markdown(&ChatRole::System));
}

// --- Composer key handling (Enter=send, Shift+Enter=newline) ---

#[test]
fn enter_without_shift_sends() {
    assert_eq!(
        composer_key_action("Enter", false, false),
        ComposerAction::Send
    );
}

#[test]
fn shift_enter_inserts_newline() {
    assert_eq!(
        composer_key_action("Enter", true, false),
        ComposerAction::Newline
    );
}

#[test]
fn other_keys_are_ignored() {
    assert_eq!(
        composer_key_action("a", false, false),
        ComposerAction::Ignore
    );
    assert_eq!(
        composer_key_action("a", true, false),
        ComposerAction::Ignore
    );
    assert_eq!(
        composer_key_action("Escape", false, false),
        ComposerAction::Ignore
    );
    // A bare Shift press must not send.
    assert_eq!(
        composer_key_action("Shift", true, false),
        ComposerAction::Ignore
    );
}

// --- IME composition: Enter that commits a composition must never send ---

#[test]
fn enter_while_composing_is_ignored() {
    // CJK/Korean IME: the Enter that commits the composed text fires a
    // "Enter" keydown too. It must not be treated as "send".
    assert_eq!(
        composer_key_action("Enter", false, true),
        ComposerAction::Ignore
    );
}

#[test]
fn shift_enter_while_composing_is_ignored() {
    // Match standard chat behavior: while composing, Enter combinations are
    // swallowed rather than inserting a newline.
    assert_eq!(
        composer_key_action("Enter", true, true),
        ComposerAction::Ignore
    );
}

// --- Auto-stick-to-bottom scroll decision ---

#[test]
fn sticks_when_at_bottom() {
    // scroll_top=900, height=1000, viewport=100 -> distance 0.
    assert!(should_stick_to_bottom(900.0, 1000.0, 100.0, 40.0));
}

#[test]
fn sticks_within_threshold() {
    // distance = 1000 - 870 - 100 = 30, threshold 40 -> stick.
    assert!(should_stick_to_bottom(870.0, 1000.0, 100.0, 40.0));
}

#[test]
fn sticks_exactly_at_threshold() {
    // distance = 1000 - 860 - 100 = 40 == threshold -> inclusive, still sticks.
    assert!(should_stick_to_bottom(860.0, 1000.0, 100.0, 40.0));
}

#[test]
fn does_not_stick_when_scrolled_up() {
    // distance = 1000 - 500 - 100 = 400 > 40 -> user scrolled up, don't yank.
    assert!(!should_stick_to_bottom(500.0, 1000.0, 100.0, 40.0));
}

#[test]
fn short_content_always_sticks() {
    // Content fits the viewport (no scroll): distance is <= 0.
    assert!(should_stick_to_bottom(0.0, 80.0, 100.0, 40.0));
}

// --- Thinking indicator ---

#[test]
fn thinking_shown_while_waiting_before_first_token() {
    assert!(is_thinking(true, false));
}

#[test]
fn thinking_hidden_once_streaming_starts() {
    assert!(!is_thinking(true, true));
}

#[test]
fn thinking_hidden_when_idle() {
    assert!(!is_thinking(false, false));
    assert!(!is_thinking(false, true));
}

// --- Welcome/prompt chips visibility ---

#[test]
fn chips_shown_only_on_empty_transcript_with_prompts() {
    assert!(show_welcome_chips(0, 3));
}

#[test]
fn chips_hidden_once_conversation_starts() {
    assert!(!show_welcome_chips(1, 3));
}

#[test]
fn chips_hidden_without_configured_prompts() {
    assert!(!show_welcome_chips(0, 0));
}

// --- Usage/cost caption formatting ---

#[test]
fn format_usage_none_renders_nothing() {
    assert_eq!(format_usage(None), None);
}

#[test]
fn format_usage_all_zero_renders_nothing() {
    assert_eq!(format_usage(Some(Usage::default())), None);
}

#[test]
fn format_usage_formats_cost_and_tokens() {
    let u = Usage {
        cost_usd: 0.0021,
        input_tokens: 1234,
        output_tokens: 567,
        // `..Default::default()` rather than naming every field: `Usage` is a
        // telemetry struct on a sibling path dep that gains fields over time,
        // and a struct literal breaks every consumer each time one lands.
        ..Default::default()
    };
    assert_eq!(
        format_usage(Some(u)),
        Some("$0.0021 · 1,234 in · 567 out".to_string())
    );
}

#[test]
fn format_usage_small_token_counts_have_no_commas() {
    let u = Usage {
        cost_usd: 0.0001,
        input_tokens: 12,
        output_tokens: 3,
        ..Default::default()
    };
    assert_eq!(
        format_usage(Some(u)),
        Some("$0.0001 · 12 in · 3 out".to_string())
    );
}

// --- Composer auto-grow clamp math ---

#[test]
fn clamp_composer_height_below_base_uses_base() {
    assert_eq!(clamp_composer_height(20.0, 48.0, 320.0), 48.0);
}

#[test]
fn clamp_composer_height_within_range_passes_through() {
    assert_eq!(clamp_composer_height(150.0, 48.0, 320.0), 150.0);
}

#[test]
fn clamp_composer_height_above_max_uses_max() {
    assert_eq!(clamp_composer_height(500.0, 48.0, 320.0), 320.0);
}

#[test]
fn clamp_composer_height_exactly_at_bounds_is_inclusive() {
    assert_eq!(clamp_composer_height(48.0, 48.0, 320.0), 48.0);
    assert_eq!(clamp_composer_height(320.0, 48.0, 320.0), 320.0);
}

// --- Settings popover: allowed-tools comma-split parse/format ---

#[test]
fn parse_allowed_tools_splits_and_trims() {
    assert_eq!(
        parse_allowed_tools(" read, write , search"),
        Some(vec![
            "read".to_string(),
            "write".to_string(),
            "search".to_string()
        ])
    );
}

#[test]
fn parse_allowed_tools_drops_empty_entries() {
    assert_eq!(
        parse_allowed_tools("read,, write,"),
        Some(vec!["read".to_string(), "write".to_string()])
    );
}

#[test]
fn parse_allowed_tools_blank_input_is_none() {
    assert_eq!(parse_allowed_tools(""), None);
    assert_eq!(parse_allowed_tools("   "), None);
    assert_eq!(parse_allowed_tools(" , , "), None);
}

#[test]
fn format_allowed_tools_joins_with_comma_space() {
    assert_eq!(
        format_allowed_tools(&Some(vec!["read".to_string(), "write".to_string()])),
        "read, write"
    );
}

#[test]
fn format_allowed_tools_none_is_empty_string() {
    assert_eq!(format_allowed_tools(&None), "");
}

#[test]
fn allowed_tools_roundtrips_through_parse_and_format() {
    let original = Some(vec!["a".to_string(), "b".to_string()]);
    let text = format_allowed_tools(&original);
    assert_eq!(parse_allowed_tools(&text), original);
}

// --- Settings popover: building ChatSettings from raw form fields ---

#[test]
fn settings_from_form_fields_blank_text_becomes_none() {
    let s = settings_from_form_fields("", "  ", "", false, false);
    assert_eq!(s.model, None);
    assert_eq!(s.system_prompt, None);
    assert_eq!(s.allowed_tools, None);
    assert!(!s.show_thinking);
    assert!(!s.show_tool_calls);
}

#[test]
fn settings_from_form_fields_trims_and_populates() {
    let s = settings_from_form_fields(" claude-x ", " be terse ", "read, write", true, true);
    assert_eq!(s.model, Some("claude-x".to_string()));
    assert_eq!(s.system_prompt, Some("be terse".to_string()));
    assert_eq!(
        s.allowed_tools,
        Some(vec!["read".to_string(), "write".to_string()])
    );
    assert!(s.show_thinking);
    assert!(s.show_tool_calls);
}

// --- Composer hint caption ---

#[test]
fn composer_hint_idle_shows_keybindings() {
    assert_eq!(
        composer_hint(false),
        "Enter to send · Shift+Enter for newline"
    );
}

#[test]
fn composer_hint_busy_shows_generating() {
    assert_eq!(composer_hint(true), "Generating… · Esc to stop");
}

// --- Per-role avatar background color ---

#[test]
fn avatar_bg_distinguishes_user_from_agent() {
    assert_ne!(
        role_avatar_bg(&ChatRole::User),
        role_avatar_bg(&ChatRole::Assistant)
    );
}

#[test]
fn avatar_bg_is_a_daisyui_bg_class_for_every_role() {
    for r in [
        ChatRole::User,
        ChatRole::Assistant,
        ChatRole::System,
        ChatRole::Thinking,
        ChatRole::Tool,
    ] {
        assert!(
            role_avatar_bg(&r).starts_with("bg-"),
            "{r:?} avatar bg class should start with bg-"
        );
    }
}

// --- Per-role avatar initial glyph ---

#[test]
fn avatar_initial_is_unique_per_role() {
    let roles = [
        ChatRole::User,
        ChatRole::Assistant,
        ChatRole::System,
        ChatRole::Thinking,
        ChatRole::Tool,
    ];
    for (i, a) in roles.iter().enumerate() {
        for b in &roles[i + 1..] {
            assert_ne!(
                role_avatar_initial(a),
                role_avatar_initial(b),
                "{a:?} and {b:?} should not share an avatar initial"
            );
        }
    }
}

#[test]
fn avatar_initial_distinguishes_thinking_from_tool() {
    assert_ne!(
        role_avatar_initial(&ChatRole::Thinking),
        role_avatar_initial(&ChatRole::Tool)
    );
}

// --- Capability-driven settings rows ---

/// One `Capabilities` fixture. Built through a helper rather than inline
/// literals so a field added upstream breaks one line, not five tests.
fn caps(
    id: &str,
    models: &[&str],
    permission_modes: &[&str],
    supports_thinking: bool,
    supports_tool_calls: bool,
) -> Capabilities {
    Capabilities {
        id: id.to_string(),
        label: id.to_string(),
        needs_api_key: false,
        models: models.iter().map(|m| m.to_string()).collect(),
        permission_modes: permission_modes.iter().map(|m| m.to_string()).collect(),
        supports_thinking,
        supports_tool_calls,
    }
}

#[test]
fn settings_rows_for_a_claude_like_backend_stay_free_text_with_a_permission_row() {
    // Claude Code publishes no model list but does publish permission modes.
    let c = caps(
        "claude-code",
        &[],
        &["default", "acceptEdits", "plan", "bypassPermissions"],
        true,
        true,
    );
    let rows = settings_rows_for(Some(&c));
    assert_eq!(rows.model, ModelFieldMode::FreeText);
    assert!(rows.permission, "four permission modes must show the row");
    assert!(rows.thinking);
    assert!(rows.tool_calls);
}

#[test]
fn settings_rows_for_a_codex_like_backend_select_from_its_models_without_a_permission_row() {
    // Codex has sandbox semantics instead of CLI permission modes, so the
    // permission row must not appear even though it has plenty of models.
    let models = [
        "gpt-5.6",
        "gpt-5.6-mini",
        "gpt-5.5",
        "gpt-5.4",
        "o4",
        "o4-mini",
        "o3",
    ];
    let c = caps("codex-cli", &models, &[], true, true);
    let rows = settings_rows_for(Some(&c));
    match rows.model {
        ModelFieldMode::Select(list) => assert_eq!(list.len(), 7),
        other => panic!("seven models should select, got {other:?}"),
    }
    assert!(
        !rows.permission,
        "no permission modes means no permission row"
    );
}

#[test]
fn settings_rows_for_a_groq_like_backend_pin_its_single_model() {
    let c = caps("groq", &["llama-3.3-70b"], &[], true, true);
    let rows = settings_rows_for(Some(&c));
    assert_eq!(
        rows.model,
        ModelFieldMode::Pinned("llama-3.3-70b".to_string())
    );
    assert!(rows.thinking);
    assert!(!rows.permission);
}

#[test]
fn settings_rows_for_an_ollama_like_backend_disable_the_tool_calls_toggle() {
    let c = caps("ollama", &["llama3", "mistral"], &[], true, false);
    let rows = settings_rows_for(Some(&c));
    match rows.model {
        ModelFieldMode::Select(list) => assert_eq!(list.len(), 2),
        other => panic!("two models should select, got {other:?}"),
    }
    assert!(
        !rows.tool_calls,
        "the toggle is still rendered, but disabled and explained"
    );
    assert!(rows.thinking);
}

#[test]
fn settings_rows_without_capabilities_are_the_legacy_shape() {
    // A host that never fetched a capability list must see exactly the
    // popover it saw before this feature existed.
    let rows = settings_rows_for(None);
    assert_eq!(rows.model, ModelFieldMode::FreeText);
    assert!(!rows.permission);
    assert!(rows.thinking);
    assert!(rows.tool_calls);
}

#[test]
fn model_field_mode_handles_the_boundary_between_its_three_shapes() {
    assert_eq!(model_field_mode(None), ModelFieldMode::FreeText);
    assert_eq!(
        model_field_mode(Some(&caps("empty", &[], &[], true, true))),
        ModelFieldMode::FreeText,
        "an empty model list is indistinguishable from no list"
    );
    assert_eq!(
        model_field_mode(Some(&caps("one", &["only"], &[], true, true))),
        ModelFieldMode::Pinned("only".to_string())
    );
    assert_eq!(
        model_field_mode(Some(&caps("two", &["a", "b"], &[], true, true))),
        ModelFieldMode::Select(vec!["a".to_string(), "b".to_string()]),
        "two models is the smallest list worth a select"
    );
}

// --- Header usage subtitle ---

#[test]
fn format_usage_subtitle_never_adds_reasoning_to_output() {
    let u = Usage {
        input_tokens: 1234,
        output_tokens: 567,
        reasoning_tokens: 500,
        cache_read_tokens: 9000,
        ..Default::default()
    };
    let line = format_usage_subtitle(&u);
    assert_eq!(line, "1,234 in · 9,000 cached · 567 out · 500 rsn");
    // The trap this pins: reasoning is a SUBSET of output, so 567 + 500 must
    // never appear as the output figure.
    assert!(
        !line.contains("1,067"),
        "reasoning must never be summed into output: {line}"
    );
}

// --- Transcript annotation anchoring ---

fn annotation(anchor: AnnotationAnchor, body: &str) -> TranscriptAnnotation {
    TranscriptAnnotation {
        anchor,
        kind: AnnotationKind::Notice,
        body: AnnotationBody::Text(body.to_string()),
    }
}

#[test]
fn annotations_land_at_start_after_a_message_and_at_the_end() {
    let anns = [
        annotation(AnnotationAnchor::AtEnd, "end"),
        annotation(AnnotationAnchor::AtStart, "start"),
        annotation(AnnotationAnchor::AfterMessage(1), "after 1"),
    ];
    // Three messages -> four slots.
    let slots = annotation_slots(&anns, 3);
    assert_eq!(slots.len(), 4);
    assert_eq!(slots[0], vec![1], "AtStart renders before message 0");
    assert_eq!(slots[1], Vec::<usize>::new());
    assert_eq!(slots[2], vec![2], "AfterMessage(1) renders after message 1");
    assert_eq!(slots[3], vec![0], "AtEnd renders after the last message");
}

#[test]
fn an_out_of_range_after_message_annotation_clamps_to_the_end() {
    // An annotation minted against a longer transcript must still be shown,
    // not silently dropped, after the transcript is trimmed.
    let anns = [annotation(AnnotationAnchor::AfterMessage(99), "stale")];
    let slots = annotation_slots(&anns, 2);
    assert_eq!(slots[2], vec![0]);
    assert!(slots[0].is_empty());
    assert!(slots[1].is_empty());
}

#[test]
fn on_an_empty_transcript_start_annotations_precede_end_annotations() {
    // Start and end are the same slot when there are no messages; declaration
    // order alone would render "end" first here.
    let anns = [
        annotation(AnnotationAnchor::AtEnd, "end"),
        annotation(AnnotationAnchor::AtStart, "start"),
    ];
    let slots = annotation_slots(&anns, 0);
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0], vec![1, 0]);
}

#[test]
fn annotation_kind_data_attributes_are_stable_and_distinct() {
    let kinds = [
        AnnotationKind::Notice,
        AnnotationKind::Warning,
        AnnotationKind::Citations,
        AnnotationKind::Evidence,
    ];
    assert_eq!(
        kinds.map(|k| k.as_str()),
        ["notice", "warning", "citations", "evidence"]
    );
}

// --- DOM hooks ---

#[test]
fn role_data_attributes_are_unique_per_role() {
    let roles = [
        ChatRole::User,
        ChatRole::Assistant,
        ChatRole::System,
        ChatRole::Thinking,
        ChatRole::Tool,
    ];
    for (i, a) in roles.iter().enumerate() {
        for b in &roles[i + 1..] {
            assert_ne!(role_data_attr(a), role_data_attr(b), "{a:?} vs {b:?}");
        }
    }
    assert_eq!(role_data_attr(&ChatRole::User), "user");
    assert_eq!(role_data_attr(&ChatRole::Assistant), "assistant");
    assert_eq!(role_data_attr(&ChatRole::System), "system");
}

#[test]
fn panel_state_prefers_error_then_streaming_then_waiting() {
    assert_eq!(chat_state_attr(false, false, false), "idle");
    assert_eq!(chat_state_attr(true, false, false), "waiting");
    assert_eq!(chat_state_attr(true, true, false), "streaming");
    assert_eq!(
        chat_state_attr(false, false, true),
        "error",
        "a consumed terminal error is the state the user must act on"
    );
}

// --- Localized texts ---

#[test]
fn ai_chat_texts_en_and_es_complete_and_differ() {
    let en = AiChatTexts::default();
    let es = AiChatTexts::es();
    let en_fields = en.fields();
    let es_fields = es.fields();

    assert_eq!(en_fields.len(), AiChatTexts::FIELD_COUNT);
    assert_eq!(es_fields.len(), AiChatTexts::FIELD_COUNT);

    // "Claude" is the assistant's proper name, not a translatable noun: it is
    // the fallback shown when the host supplies no `assistant_label`, and
    // translating it would rename the product. Keeping it on the allow-list
    // means this test cannot prove the field is READ, which is exactly how the
    // hard-coded "Claude" literal survived fix round 0 — so
    // `the_assistant_fallback_comes_from_the_texts_table` exercises the field
    // with a table whose value is not "Claude". That test, not this one, is
    // the coverage for `role_assistant`.
    let allowed_identical: &[&str] = &["role_assistant"];

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

    // Both placeholder templates must keep the marker the panel substitutes,
    // or the assistant's name silently disappears from the composer.
    assert!(en.composer_placeholder.contains("{assistant}"));
    assert!(es.composer_placeholder.contains("{assistant}"));
}

#[test]
fn the_legacy_text_helpers_match_the_default_table() {
    // `composer_hint`, `default_composer_placeholder` and `role_label_with`
    // are thin wrappers kept for existing callers. They hold their own copy
    // of the English strings, so this pins them to the table they claim to
    // mirror -- otherwise the two can drift apart silently.
    let en = AiChatTexts::default();
    assert_eq!(composer_hint(false), composer_hint_for(false, &en));
    assert_eq!(composer_hint(true), composer_hint_for(true, &en));
    assert_eq!(
        default_composer_placeholder("Codex"),
        composer_placeholder_for("Codex", &en)
    );
    assert_eq!(
        default_composer_placeholder(""),
        composer_placeholder_for("", &en)
    );
    for r in [
        ChatRole::User,
        ChatRole::Assistant,
        ChatRole::System,
        ChatRole::Thinking,
        ChatRole::Tool,
    ] {
        assert_eq!(role_label_with(&r, ""), role_label_for(&r, "", &en));
        assert_eq!(role_label(&r), role_label_with(&r, ""));
    }
}

// --- Fix round 1 ---

/// C1: `Select` is a closed list, so a form value carried over from another
/// backend names nothing the control can display. Without a snap, no option
/// is `selected`, the browser shows the FIRST one, and Apply ships the hidden
/// stale id — which the user cannot even correct by choosing the option they
/// can already see, because that fires no `change` event.
#[test]
fn a_model_absent_from_the_new_backends_list_is_reconciled_onto_it() {
    let models = vec!["a".to_string(), "b".to_string()];
    assert_eq!(
        reconcile_model_for(&ModelFieldMode::Select(models.clone()), "z"),
        Some("a".to_string()),
        "a carried-over model must snap onto the closed list"
    );
    assert_eq!(
        reconcile_model_for(&ModelFieldMode::Select(models.clone()), "b"),
        None,
        "a listed model is already displayable: no churn"
    );
    assert_eq!(
        reconcile_model_for(&ModelFieldMode::Select(models), "a"),
        None,
        "the first model is not a special case"
    );
    assert_eq!(
        reconcile_model_for(&ModelFieldMode::Pinned("x".to_string()), "y"),
        Some("x".to_string())
    );
    assert_eq!(
        reconcile_model_for(&ModelFieldMode::Pinned("x".to_string()), "x"),
        None
    );
    for current in ["", "anything", "a"] {
        assert_eq!(
            reconcile_model_for(&ModelFieldMode::FreeText, current),
            None,
            "free text has no list to disagree with"
        );
    }
    // An empty list cannot be snapped onto, and must not panic.
    assert_eq!(
        reconcile_model_for(&ModelFieldMode::Select(Vec::new()), "z"),
        None
    );
}

/// I1: the fallback attribution must come from the texts table. It used to be
/// an inline `"Claude"` literal, which made `role_assistant` unreachable —
/// the derived signal was the only value ever passed as `assistant_label`, so
/// it was never empty and every texts-aware branch was dead.
#[test]
fn the_assistant_fallback_comes_from_the_texts_table() {
    let en = AiChatTexts::default();
    let es = AiChatTexts::es();
    let custom = AiChatTexts {
        role_assistant: "Asistente".into(),
        ..AiChatTexts::es()
    };

    assert_eq!(effective_assistant_label("", &en), "Claude");
    assert_eq!(effective_assistant_label("", &es), "Claude");
    assert_eq!(
        effective_assistant_label("", &custom),
        "Asistente",
        "a host that translates role_assistant must see it"
    );
    // A configured backend label always wins: it is the backend's own name.
    assert_eq!(
        effective_assistant_label("Codex CLI (OpenAI)", &custom),
        "Codex CLI (OpenAI)"
    );

    // And the branch the component used to bypass entirely.
    assert_eq!(
        role_label_for(&ChatRole::Assistant, "", &custom),
        "Asistente"
    );
    assert_eq!(
        composer_placeholder_for("", &custom),
        "Pregúntale a Asistente sobre este documento…",
        "the composer placeholder takes the same fallback"
    );
}

/// I2: on a control describing what the agent may do WITHOUT asking, an
/// unchosen first option displayed as chosen is a lie the host is never told
/// about — no `change` event fires.
#[test]
fn an_unchosen_permission_mode_selects_nothing_rather_than_the_first_option() {
    let modes = [
        "default".to_string(),
        "acceptEdits".to_string(),
        "plan".to_string(),
        "bypassPermissions".to_string(),
    ];
    assert_eq!(
        permission_selection(None, &modes),
        None,
        "nothing chosen must not read as 'default'"
    );
    assert_eq!(permission_selection(Some("default"), &modes), Some(0));
    assert_eq!(permission_selection(Some("plan"), &modes), Some(2));
    assert_eq!(
        permission_selection(Some("sandbox-write"), &modes),
        None,
        "a mode this backend does not publish is also 'nothing chosen'"
    );
    assert_eq!(permission_selection(Some("default"), &[]), None);
}

/// A transport whose `send` fails after `fails_after` successful sends, so a
/// test can let the first turn through and fail the retry.
struct FlakyTransport {
    sends: usize,
    fails_after: usize,
    queue: std::collections::VecDeque<StreamEvent>,
}

impl ChatTransport for FlakyTransport {
    fn send(&mut self, _req: ChatRequest) -> Result<(), ChatError> {
        self.sends += 1;
        if self.sends > self.fails_after {
            return Err(ChatError::Transport("connection lost".into()));
        }
        Ok(())
    }
    fn try_recv(&mut self) -> Option<StreamEvent> {
        self.queue.pop_front()
    }
    fn restart(&mut self) -> Result<(), ChatError> {
        Ok(())
    }
    fn cancel(&mut self) -> Result<(), ChatError> {
        Ok(())
    }
    fn configure(&mut self, _settings: ChatSettings) {}
}

fn flaky_session(fails_after: usize, queue: Vec<StreamEvent>) -> ChatSession {
    ChatSession::new(Box::new(FlakyTransport {
        sends: 0,
        fails_after,
        queue: queue.into(),
    }))
}

/// I3: clearing the strip before attempting the retry deleted the user's only
/// signal that anything was wrong the moment the retry itself failed — the
/// panel went back to `idle`, `on_retry` still fired, and the button they
/// would press again was gone.
#[test]
fn a_failed_retry_keeps_the_error_strip_and_does_not_notify_the_host() {
    let en = AiChatTexts::default();
    // First send succeeds, then the transport reports the error that raised
    // the strip; the retry's send is the one that fails.
    let mut session = flaky_session(1, vec![StreamEvent::Error("stream dropped".into())]);
    session
        .send(ChatRequest {
            prompt: "hello".into(),
            attachments: Vec::new(),
            page_context: None,
        })
        .expect("first send succeeds");
    assert!(session.poll(), "the error event lands");
    assert!(!session.is_waiting(), "a terminal error ends the turn");

    let outcome = retry_outcome(session.retry(), session.is_waiting(), &en);
    match outcome {
        RetryOutcome::Failed(message) => {
            assert!(
                message.starts_with("Retry failed"),
                "the strip must be texts-prefixed: {message}"
            );
            assert!(
                message.contains("connection lost"),
                "and must name the retry's own error: {message}"
            );
        }
        other => panic!("a failed re-send must keep the strip up, got {other:?}"),
    }
    assert!(
        !session.is_waiting(),
        "nothing is in flight after a failed retry"
    );
}

#[test]
fn a_successful_retry_clears_the_strip_and_notifies_the_host() {
    let en = AiChatTexts::default();
    let mut session = flaky_session(99, vec![StreamEvent::Error("stream dropped".into())]);
    session
        .send(ChatRequest {
            prompt: "hello".into(),
            attachments: Vec::new(),
            page_context: None,
        })
        .expect("first send succeeds");
    assert!(session.poll());

    assert_eq!(
        retry_outcome(session.retry(), session.is_waiting(), &en),
        RetryOutcome::Sent
    );
    assert!(session.is_waiting(), "the turn is back in flight");
}

#[test]
fn retrying_with_nothing_to_resend_leaves_the_strip_exactly_as_it_was() {
    // `ChatSession::retry` returns Ok(()) WITHOUT sending anything when there
    // is no `last_sent`. Treating that as success would clear the strip while
    // achieving nothing at all.
    let en = AiChatTexts::default();
    let mut session = flaky_session(99, Vec::new());
    assert_eq!(
        retry_outcome(session.retry(), session.is_waiting(), &en),
        RetryOutcome::NothingToRetry
    );

    // And a Spanish table prefixes the failure in Spanish.
    let es = AiChatTexts::es();
    let failed = retry_outcome(Err(ChatError::NotConnected), false, &es);
    match failed {
        RetryOutcome::Failed(message) => {
            assert!(message.starts_with("El reintento falló"), "got {message}")
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn localized_labels_follow_the_texts_table_but_never_rename_the_backend() {
    let es = AiChatTexts::es();
    assert_eq!(role_label_for(&ChatRole::User, "", &es), "Tú");
    assert_eq!(role_label_for(&ChatRole::System, "", &es), "Sistema");
    // A configured backend label is its own name and is never translated.
    assert_eq!(
        role_label_for(&ChatRole::Assistant, "Codex CLI (OpenAI)", &es),
        "Codex CLI (OpenAI)"
    );
    assert_eq!(
        composer_placeholder_for("Codex", &es),
        "Pregúntale a Codex sobre este documento…"
    );
}
