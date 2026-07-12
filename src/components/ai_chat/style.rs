//! Role → daisyUI class / label mapping for [`super::AiChat`].

use ai_chat_core::ChatRole;

/// `(chat-side, chat-bubble-modifier)` daisyUI classes for a message role.
///
/// Every role sits `chat-start`: bubbles span the full row (see the
/// `.lds-aichat .chat` override in `markdown::theme`), so no message sits on
/// the right. The bubble colour, not the side, distinguishes the speaker.
pub fn role_classes(role: &ChatRole) -> (&'static str, &'static str) {
    match role {
        ChatRole::User => ("chat-start", "chat-bubble-primary"),
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
///
/// `is_composing` is `true` while an IME composition is in progress (CJK/Korean
/// input methods, etc.). The Enter that commits a composition must never be
/// treated as "send" — it's `KeyboardEvent.isComposing` in the browser, which
/// callers should forward via `ev.is_composing()`.
pub fn composer_key_action(key: &str, shift: bool, is_composing: bool) -> ComposerAction {
    if is_composing {
        return ComposerAction::Ignore;
    }
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

/// Composer auto-grow: clamp the textarea's natural `scrollHeight` between a
/// collapsed `base_height` and a `max_height`, beyond which the textarea
/// stops growing and scrolls internally instead (`overflow-y-auto`).
pub fn clamp_composer_height(scroll_height: f64, base_height: f64, max_height: f64) -> f64 {
    scroll_height.max(base_height).min(max_height)
}

/// Composer hint caption text: the keybinding reminder while idle, switching
/// to the in-flight hint (mirroring the Stop button) while a turn is busy.
pub fn composer_hint(waiting: bool) -> &'static str {
    if waiting {
        "Generating\u{2026} \u{b7} Esc to stop"
    } else {
        "Enter to send \u{b7} Shift+Enter for newline"
    }
}

/// Background daisyUI class for a role's avatar dot, paired with
/// [`role_label`]'s first character as the initial. User gets the primary
/// color (matching its `chat-bubble-primary`); everything else gets a
/// neutral/info tone matching its bubble family.
pub fn role_avatar_bg(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::User => "bg-primary text-primary-content",
        ChatRole::Assistant => "bg-neutral text-neutral-content",
        ChatRole::System => "bg-info text-info-content",
        ChatRole::Thinking => "bg-base-300 text-base-content",
        ChatRole::Tool => "bg-neutral text-neutral-content",
    }
}

/// Avatar glyph for a role. Mirrors [`role_label`]'s first letter for most
/// roles, except `Thinking`/`Tool` — both of which would otherwise collide on
/// "T" — which get distinct glyphs instead (an ellipsis for the ephemeral
/// reasoning indicator, a gear for tool-call output).
pub fn role_avatar_initial(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::User => "Y",
        ChatRole::Assistant => "C",
        ChatRole::System => "S",
        ChatRole::Thinking => "\u{2026}",
        ChatRole::Tool => "\u{2699}",
    }
}
