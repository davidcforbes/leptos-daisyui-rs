use super::style::{
    ComposerAction, clamp_composer_height, composer_hint, composer_key_action,
    default_composer_placeholder, is_markdown, is_thinking, role_avatar_bg, role_avatar_initial,
    role_avatar_initial_with, role_classes, role_label, role_label_with, should_stick_to_bottom,
    show_welcome_chips,
};
use super::types::{
    format_allowed_tools, format_usage, parse_allowed_tools, settings_from_form_fields,
};
use ai_chat_core::{ChatRole, Usage};

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
