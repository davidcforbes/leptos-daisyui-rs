//! # AiChat
//!
//! A reusable Leptos chat panel over the shared, executor-free
//! [`ai_chat_core`] model. See the design at
//! `editmark/docs/superpowers/specs/2026-05-30-ai-chat-component-design.md`
//! (rollout step 3). The desktop analogue is `d2d_ui::controls::chat`.

mod component;
mod style;
#[cfg(test)]
mod tests;
pub mod transport;
mod types;

pub use component::{AiChat, ChatScopeOption};
pub use style::{
    ComposerAction, composer_key_action, is_markdown, is_thinking, role_classes, role_label,
    should_stick_to_bottom, show_welcome_chips,
};
pub use transport::{BridgeCommand, SseBridgeHandle, SseBridgeTransport, parse_sse_data};
pub use types::{
    format_allowed_tools, format_count, format_usage, parse_allowed_tools,
    settings_from_form_fields,
};

// Re-export the shared model so consumers can build a session/transport and
// drive the component without a direct `ai-chat-core` dependency.
pub use ai_chat_core::{
    AttachmentRef, Capabilities, ChatError, ChatMessage, ChatRequest, ChatRole, ChatSession,
    ChatSettings, ChatTransport, Citation, HitlState, MessageMeta, StreamEvent, ToolCall, Usage,
};

/// Wire types shared with the engine service and editmark-server
/// (`OpenSessionRequest`, `DocumentInput`, `SendRequest`, `SessionSnapshot`).
///
/// ```
/// use leptos_daisyui_rs::components::{Capabilities, ai_chat::wire::OpenSessionRequest};
/// ```
pub mod wire {
    pub use ai_chat_core::wire::*;
}
