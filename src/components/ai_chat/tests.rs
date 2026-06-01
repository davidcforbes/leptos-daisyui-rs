use super::style::{is_markdown, role_classes, role_label};
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
        assert_eq!(role_classes(&r).0, "chat-start", "{r:?} should be chat-start");
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
