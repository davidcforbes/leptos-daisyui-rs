use super::style::{
    ComposerAction, composer_key_action, is_markdown, is_thinking, role_classes, role_label,
    should_stick_to_bottom, show_welcome_chips,
};
use super::types::format_usage;
use ai_chat_core::{ChatRole, Usage};

#[test]
fn user_sits_end_agent_sits_start() {
    assert_eq!(role_classes(&ChatRole::User).0, "chat-end");
    for r in [
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
    assert_eq!(composer_key_action("a", false, false), ComposerAction::Ignore);
    assert_eq!(composer_key_action("a", true, false), ComposerAction::Ignore);
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
    assert_eq!(format_usage(Some(u)), Some("$0.0021 · 1,234 in · 567 out".to_string()));
}

#[test]
fn format_usage_small_token_counts_have_no_commas() {
    let u = Usage {
        cost_usd: 0.0001,
        input_tokens: 12,
        output_tokens: 3,
    };
    assert_eq!(format_usage(Some(u)), Some("$0.0001 · 12 in · 3 out".to_string()));
}
