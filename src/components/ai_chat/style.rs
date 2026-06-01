//! Role → daisyUI class / label mapping for [`super::AiChat`].

use ai_chat_core::ChatRole;

/// `(chat-side, chat-bubble-modifier)` daisyUI classes for a message role.
///
/// User messages sit on the right (`chat-end`); everything the agent emits
/// sits on the left (`chat-start`).
pub fn role_classes(role: &ChatRole) -> (&'static str, &'static str) {
    match role {
        ChatRole::User => ("chat-end", "chat-bubble-primary"),
        ChatRole::Assistant => ("chat-start", ""),
        ChatRole::System => ("chat-start", "chat-bubble-info"),
        ChatRole::Thinking => ("chat-start", "chat-bubble-ghost"),
        ChatRole::Tool => ("chat-start", "chat-bubble-neutral"),
    }
}

/// Short header label for a message role.
pub fn role_label(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::User => "You",
        ChatRole::Assistant => "Claude",
        ChatRole::System => "System",
        ChatRole::Thinking => "Thinking",
        ChatRole::Tool => "Tool",
    }
}

/// True when a role's `content` is markdown (rendered via `MarkdownView`); the
/// agent's reasoning/tool messages are plain text.
pub fn is_markdown(role: &ChatRole) -> bool {
    matches!(role, ChatRole::User | ChatRole::Assistant)
}
