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
mod texts;
pub mod transport;
mod types;

pub use component::{AiChat, ChatScopeOption};
pub use style::{
    ComposerAction, annotation_bubble_class, chat_state_attr, composer_hint, composer_hint_for,
    composer_key_action, composer_placeholder_for, default_composer_placeholder,
    effective_assistant_label, is_markdown, is_thinking, role_classes, role_data_attr, role_label,
    role_label_for, role_label_with, should_stick_to_bottom, show_welcome_chips,
    tool_phase_data_attr,
};
pub use texts::AiChatTexts;
pub use transport::{BridgeCommand, SseBridgeHandle, SseBridgeTransport, parse_sse_data};
pub use types::{
    AnnotationAnchor, AnnotationBody, AnnotationKind, ModelFieldMode, RetryOutcome, SettingsRowSet,
    TranscriptAnnotation, annotation_slots, format_allowed_tools, format_count, format_usage,
    format_usage_subtitle, model_field_mode, parse_allowed_tools, permission_selection,
    reconcile_model_for, retry_outcome, settings_from_form_fields, settings_rows_for,
};

// Re-export the shared model so consumers can build a session/transport and
// drive the component without a direct `ai-chat-core` dependency.
pub use ai_chat_core::{
    AttachmentRef, Capabilities, ChatError, ChatMessage, ChatRequest, ChatRole, ChatSession,
    ChatSettings, ChatTransport, Citation, HitlState, MessageMeta, StreamEvent, ToolCall,
    ToolPhase, Usage,
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
