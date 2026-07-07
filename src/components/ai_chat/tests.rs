use super::style::{
    ComposerAction, composer_key_action, is_markdown, is_thinking, role_classes, role_label,
    should_stick_to_bottom, show_welcome_chips,
};
use ai_chat_core::ChatRole;

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
    assert_eq!(composer_key_action("Enter", false), ComposerAction::Send);
}

#[test]
fn shift_enter_inserts_newline() {
    assert_eq!(composer_key_action("Enter", true), ComposerAction::Newline);
}

#[test]
fn other_keys_are_ignored() {
    assert_eq!(composer_key_action("a", false), ComposerAction::Ignore);
    assert_eq!(composer_key_action("a", true), ComposerAction::Ignore);
    assert_eq!(composer_key_action("Escape", false), ComposerAction::Ignore);
    // A bare Shift press must not send.
    assert_eq!(composer_key_action("Shift", true), ComposerAction::Ignore);
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
