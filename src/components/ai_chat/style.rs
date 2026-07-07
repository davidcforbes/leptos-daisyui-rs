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

/// What a keystroke in the multiline composer should do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposerAction {
    /// Submit the current input (Enter without Shift).
    Send,
    /// Insert a literal newline (Shift+Enter) — the textarea's default.
    Newline,
    /// Not composer-relevant; let the browser handle it (normal typing).
    Ignore,
}

/// Decide what a keydown means in the `<textarea>` composer: Enter (no Shift)
/// sends, Shift+Enter inserts a newline, everything else is ordinary input.
pub fn composer_key_action(key: &str, shift: bool) -> ComposerAction {
    match key {
        "Enter" if !shift => ComposerAction::Send,
        "Enter" => ComposerAction::Newline,
        _ => ComposerAction::Ignore,
    }
}

/// Whether the transcript should stay pinned to the bottom on new content.
///
/// `distance_from_bottom = scroll_height - scroll_top - client_height`. We keep
/// sticking while the user is within `threshold` px of the bottom, so streaming
/// content follows the view; once they scroll further up we stop yanking them
/// back down.
pub fn should_stick_to_bottom(
    scroll_top: f64,
    scroll_height: f64,
    client_height: f64,
    threshold: f64,
) -> bool {
    (scroll_height - scroll_top - client_height) <= threshold
}

/// Whether to show the ephemeral "Thinking…" indicator: a turn is in flight
/// (`waiting`) but no assistant text has begun streaming yet. Mirrors d2d-ui's
/// `is_thinking` (`busy && streaming_idx.is_none()`).
pub fn is_thinking(waiting: bool, has_streaming_message: bool) -> bool {
    waiting && !has_streaming_message
}

/// Whether welcome/prompt chips should be shown: only on an empty transcript
/// and only when the host configured at least one prompt.
pub fn show_welcome_chips(message_count: usize, prompt_count: usize) -> bool {
    message_count == 0 && prompt_count > 0
}
