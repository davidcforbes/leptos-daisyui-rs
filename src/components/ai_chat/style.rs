//! Role → daisyUI class / label mapping for [`super::AiChat`].

use super::texts::AiChatTexts;
use super::types::AnnotationKind;
use ai_chat_core::{ChatRole, MessageMeta, ToolPhase};

/// `(chat-side, chat-bubble-modifier)` daisyUI classes for a message role.
///
/// The user's bubbles sit `chat-start` (tail on the left); every other role --
/// the assistant, its thinking and tool rows, system notes -- sits `chat-end`
/// (tail on the right), a 4iiz-Office owner ruling (ldui-bay7). The side only
/// moves the tail, header and avatar: bubbles still span the full row (see
/// the `.lds-aichat .chat` override in `markdown::theme`), so a narrow rail
/// loses no width to the side.
pub fn role_classes(role: &ChatRole) -> (&'static str, &'static str) {
    match role {
        ChatRole::User => ("chat-start", "chat-bubble-primary"),
        ChatRole::Assistant => ("chat-end", ""),
        ChatRole::System => ("chat-end", "chat-bubble-info"),
        ChatRole::Thinking => ("chat-end", "chat-bubble-ghost"),
        ChatRole::Tool => ("chat-end", "chat-bubble-neutral"),
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

/// [`role_label`] with the assistant attributed to the CONFIGURED backend's
/// label — parity with ai-chat-engine (em-c7w1): a Codex session must read as
/// Codex, never a hardcoded "Claude". An empty label keeps the historical
/// default so label-less callers are unchanged.
///
/// A thin wrapper over [`role_label_for`] with the default (English) texts,
/// kept so label-less callers and their tests are unchanged.
pub fn role_label_with(role: &ChatRole, assistant_label: &str) -> String {
    role_label_for(role, assistant_label, &AiChatTexts::default())
}

/// [`role_label_with`] over a localized [`AiChatTexts`] table.
///
/// A non-empty `assistant_label` still wins for the assistant — it is the
/// configured backend's own name, which is never translated.
pub fn role_label_for(role: &ChatRole, assistant_label: &str, texts: &AiChatTexts) -> String {
    match role {
        ChatRole::Assistant if !assistant_label.is_empty() => assistant_label.to_string(),
        ChatRole::User => texts.role_user.clone(),
        ChatRole::Assistant => texts.role_assistant.clone(),
        ChatRole::System => texts.role_system.clone(),
        ChatRole::Thinking => texts.role_thinking.clone(),
        ChatRole::Tool => texts.role_tool.clone(),
    }
}

/// The assistant attribution the panel should use everywhere: the configured
/// backend's own label, or — when the host supplied none — the localized
/// `AiChatTexts::role_assistant`.
///
/// This is the ONE place the fallback is decided. It used to be an inline
/// `"Claude"` literal in the component, which made `role_assistant`
/// unreachable: the derived signal was the only value ever passed as
/// `assistant_label`, so it was never empty at any call site and every
/// texts-aware branch below was dead in the render path. A host that
/// translated the field saw no effect and had no way to discover why.
pub fn effective_assistant_label(assistant_label: &str, texts: &AiChatTexts) -> String {
    if assistant_label.is_empty() {
        texts.role_assistant.clone()
    } else {
        assistant_label.to_string()
    }
}

/// The `data-chat-role` hook for a message role, mirroring the attribute
/// 4iiz-Office's own chat surfaces expose so a single browser assertion
/// reads both.
pub fn role_data_attr(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::System => "system",
        ChatRole::Thinking => "thinking",
        ChatRole::Tool => "tool",
    }
}

/// The `data-chat-tool-phase` hook for a message, or `None` when the message
/// carries no phase.
///
/// A tool turn reaches the transcript as TWO `ChatRole::Tool` messages, the
/// call and its result, which `data-chat-role` alone cannot tell apart. A
/// consumer wanting the RESULT therefore had to reach for it by document
/// position, and a positional selector does not fail when the transcript
/// changes shape: it silently starts describing something else. `ai_chat_core`
/// stamps `MessageMeta::tool_phase` on both messages, and this maps that field
/// onto a hook a selector can name.
///
/// `None` means the attribute is not emitted at all, which is the whole
/// contract: an empty `data-chat-tool-phase=""` would still match the
/// `[data-chat-tool-phase]` selector and make every ordinary bubble look like
/// half of a tool turn.
pub fn tool_phase_data_attr(meta: &MessageMeta) -> Option<&'static str> {
    match meta.tool_phase? {
        ToolPhase::Call => Some("call"),
        ToolPhase::Result => Some("result"),
    }
}

/// The root's `data-ai-chat-state` hook.
///
/// A consumed terminal error outranks the rest because it is the only state
/// the user must act on; it can only be set while the turn is over, since
/// `StreamEvent::Error` ends the turn, and it is cleared on the next send,
/// retry or restart.
pub fn chat_state_attr(waiting: bool, streaming: bool, has_error: bool) -> &'static str {
    if has_error {
        "error"
    } else if streaming {
        "streaming"
    } else if waiting {
        "waiting"
    } else {
        "idle"
    }
}

/// daisyUI bubble modifier for an annotation row's tone.
pub fn annotation_bubble_class(kind: AnnotationKind) -> &'static str {
    match kind {
        AnnotationKind::Notice => "chat-bubble-info",
        AnnotationKind::Warning => "chat-bubble-warning",
        AnnotationKind::Citations => "chat-bubble-ghost",
        AnnotationKind::Evidence => "chat-bubble-neutral",
    }
}

/// [`role_avatar_initial`] with the assistant's initial taken from the
/// configured label (uppercased first char; empty label falls back).
pub fn role_avatar_initial_with(role: &ChatRole, assistant_label: &str) -> String {
    match role {
        ChatRole::Assistant => assistant_label
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| role_avatar_initial(role).to_string()),
        r => role_avatar_initial(r).to_string(),
    }
}

/// Composer placeholder when the host didn't set one, addressed to the
/// configured assistant over a localized [`AiChatTexts`] table.
///
/// `AiChatTexts::composer_placeholder` is a template: its `{assistant}`
/// marker is replaced with `assistant_label`, or with
/// `AiChatTexts::role_assistant` when the label is empty.
pub fn composer_placeholder_for(assistant_label: &str, texts: &AiChatTexts) -> String {
    let who = if assistant_label.is_empty() {
        texts.role_assistant.as_str()
    } else {
        assistant_label
    };
    texts.composer_placeholder.replace("{assistant}", who)
}

/// Composer placeholder when the host didn't set one, addressed to the
/// configured assistant (empty label keeps the historical "Claude").
///
/// A thin wrapper over [`composer_placeholder_for`] with the default
/// (English) texts, kept so existing callers and their tests are unchanged.
pub fn default_composer_placeholder(assistant_label: &str) -> String {
    composer_placeholder_for(assistant_label, &AiChatTexts::default())
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

/// Composer height for one auto-grow pass, honouring a height the user
/// dragged the composer to with its resize grip (ldui-ms9u).
///
/// `content_px` is the textarea's natural `scrollHeight`; the post-send
/// collapse passes `0.0`. With no drag this is exactly
/// `clamp_composer_height`: the content, at least `base_px`, at most
/// `max_px`. A dragged height is a FLOOR from then on: the composer still
/// grows with its content up to `max_px`, never shrinks below the floor, and
/// collapses back to the floor -- not to `base_px` -- after a send. A floor
/// above `max_px` is kept as it is, because `max_px` caps only what the
/// auto-grow chooses; the drag itself is bounded by the textarea's CSS
/// `max-height`. A floor below `base_px` is lifted to `base_px`.
pub fn composer_height(
    content_px: f64,
    user_floor_px: Option<f64>,
    base_px: f64,
    max_px: f64,
) -> f64 {
    let floor = user_floor_px.map_or(base_px, |floor| floor.max(base_px));
    clamp_composer_height(content_px, floor, max_px.max(floor))
}

/// How far (px) the composer's inline height may sit from the height the
/// auto-grow last wrote and still count as that write rather than a drag.
/// Absorbs a browser re-serialising a fractional value; no deliberate drag is
/// that small.
pub const COMPOSER_DRAG_TOLERANCE_PX: f64 = 1.0;

/// The composer's INLINE `height` (its `style` attribute's value, e.g.
/// `"213px"`) in pixels. `None` for no inline height, `auto`, a non-px unit
/// or a non-positive value, none of which a resize grip writes.
pub fn composer_inline_height_px(value: &str) -> Option<f64> {
    value
        .trim()
        .strip_suffix("px")
        .and_then(|number| number.trim().parse::<f64>().ok())
        .filter(|px| px.is_finite() && *px > 0.0)
}

/// Whether the user dragged the composer's resize grip since the auto-grow
/// last wrote its height (ldui-ms9u).
///
/// A textarea fires no resize event, but the grip writes the element's
/// INLINE `height`, which nothing but the auto-grow otherwise writes. So an
/// inline height that differs from `last_written_px` is the user's, and an
/// inline height before anything was written is a drag before the first
/// keystroke. The comparison is on the inline value, never the rendered
/// `offsetHeight`: daisyUI's `.textarea` has `min-height: 5rem`, so the
/// rendered height already differs from the 48px base the auto-grow writes,
/// and a CSS `max-height` clamp would make it differ too. Neither is a drag,
/// and neither changes the inline value.
pub fn composer_was_dragged(inline_height_px: Option<f64>, last_written_px: Option<f64>) -> bool {
    match (inline_height_px, last_written_px) {
        (None, _) => false,
        (Some(_), None) => true,
        (Some(inline), Some(written)) => (inline - written).abs() > COMPOSER_DRAG_TOLERANCE_PX,
    }
}

/// Composer hint caption over a localized [`AiChatTexts`] table: the
/// keybinding reminder while idle, the in-flight hint while a turn is busy.
pub fn composer_hint_for(waiting: bool, texts: &AiChatTexts) -> String {
    if waiting {
        texts.composer_hint_busy.clone()
    } else {
        texts.composer_hint.clone()
    }
}

/// Composer hint caption text: the keybinding reminder while idle, switching
/// to the in-flight hint (mirroring the Stop button) while a turn is busy.
///
/// The `&'static str` shape of [`composer_hint_for`] over the default
/// (English) texts, kept so existing callers and their tests are unchanged.
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
