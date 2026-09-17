//! Every string [`super::AiChat`] renders, in one struct built by literal.
//!
//! `Default` is English; [`AiChatTexts::es`] is the Spanish table. A host
//! overrides individual fields by struct-update syntax, exactly like
//! `crate::patterns::helpdesk::HelpdeskTexts` and
//! `crate::patterns::ai_chat_workspace::AiChatWorkspaceTexts`.
//!
//! The panel stays GENERIC: nothing here names a provider, a reason code or
//! an Office screen. Provider copy reaches the panel through the
//! `settings_extra` slot and the `annotations` prop instead.

/// Every label, placeholder and caption [`super::AiChat`] renders. See the
/// module doc for the `Default`/[`Self::es`] convention.
#[derive(Clone, Debug, PartialEq)]
pub struct AiChatTexts {
    /// Label for the settings popover's backend picker.
    pub backend: String,
    /// Label for the settings popover's model field, in all three of its
    /// shapes (free text, select, pinned).
    pub model: String,
    /// Placeholder shown in the free-text model field when the host has not
    /// chosen a model and the transport's own default applies.
    pub model_free_text_hint: String,
    /// Label for the CLI permission-mode picker.
    pub permission_mode: String,
    /// The permission picker's disabled placeholder option, shown whenever the
    /// host has chosen no mode — or has chosen one this backend does not
    /// publish. It must read as "nothing chosen", never as a mode.
    pub permission_mode_unset: String,
    /// Label for the system-prompt textarea.
    pub system_prompt: String,
    /// Label for the comma-separated allowed-tools field.
    pub allowed_tools: String,
    /// Label for the show-thinking toggle.
    pub show_thinking: String,
    /// Label for the show-tool-calls toggle.
    pub show_tool_calls: String,
    /// `title` on a settings control the selected backend cannot honour. The
    /// control is rendered DISABLED rather than hidden, so this string is the
    /// only explanation the user gets — honesty over tidiness.
    pub unsupported_here: String,
    /// The settings popover's apply button.
    pub apply: String,
    /// The header's new-session (restart) button.
    pub new_session: String,
    /// Accessible name for the header's settings gear.
    pub settings: String,
    /// The composer's send button.
    pub send: String,
    /// The composer's stop button, which cancels the in-flight turn.
    pub stop: String,
    /// Accessible name for the ephemeral pre-first-token indicator.
    pub thinking: String,
    /// The error strip's retry button.
    pub retry: String,
    /// Prefix for the message shown when the retry itself failed. The strip
    /// stays up carrying this instead of the original error, so the only
    /// signal that something is wrong is never removed by trying to fix it.
    pub retry_failed: String,
    /// The per-message copy button.
    pub copy: String,
    /// `title` on the per-message copy button.
    pub copy_message: String,
    /// Accessible name for the header's chat-scope picker.
    pub scope: String,
    /// Fallback shown on the scope picker when the active scope id matches no
    /// supplied option.
    pub scope_unknown: String,
    /// Placeholder example in the allowed-tools field. A value example rather
    /// than a label, but it is still visible English text.
    pub allowed_tools_hint: String,
    /// Composer placeholder used when the host supplied none. `{assistant}`
    /// is replaced with the configured assistant label.
    pub composer_placeholder: String,
    /// Composer caption while idle (the keybinding reminder).
    pub composer_hint: String,
    /// Composer caption while a turn is in flight.
    ///
    /// Not in the original field list, and deliberately added: the busy hint
    /// is the panel's other hard-coded English caption, so localizing only
    /// the idle one would leave a visibly untranslated panel the moment a
    /// turn starts.
    pub composer_hint_busy: String,
    /// Bubble header for the user's own messages.
    pub role_user: String,
    /// Bubble header for assistant messages when the host supplied no
    /// `assistant_label` (the historical "Claude" fallback).
    pub role_assistant: String,
    /// Bubble header for system messages.
    pub role_system: String,
    /// Bubble header for reasoning messages.
    pub role_thinking: String,
    /// Bubble header for tool-call/tool-result messages.
    pub role_tool: String,
}

impl Default for AiChatTexts {
    fn default() -> Self {
        Self {
            backend: "Backend".into(),
            model: "Model".into(),
            model_free_text_hint: "(transport default)".into(),
            permission_mode: "Permission mode".into(),
            permission_mode_unset: "Not set".into(),
            system_prompt: "System prompt".into(),
            allowed_tools: "Allowed tools (comma-separated)".into(),
            show_thinking: "Show thinking".into(),
            show_tool_calls: "Show tool calls".into(),
            unsupported_here: "This backend does not support this setting.".into(),
            apply: "Apply".into(),
            new_session: "New session".into(),
            settings: "Chat settings".into(),
            send: "Send".into(),
            stop: "Stop".into(),
            thinking: "Thinking\u{2026}".into(),
            retry: "Retry".into(),
            retry_failed: "Retry failed".into(),
            copy: "Copy".into(),
            copy_message: "Copy message".into(),
            scope: "Chat scope".into(),
            scope_unknown: "Scope".into(),
            allowed_tools_hint: "read, write".into(),
            composer_placeholder: "Ask {assistant} about this document\u{2026}".into(),
            composer_hint: "Enter to send \u{b7} Shift+Enter for newline".into(),
            composer_hint_busy: "Generating\u{2026} \u{b7} Esc to stop".into(),
            role_user: "You".into(),
            role_assistant: "Claude".into(),
            role_system: "System".into(),
            role_thinking: "Thinking".into(),
            role_tool: "Tool".into(),
        }
    }
}

impl AiChatTexts {
    /// The number of fields on this struct; kept in sync with the struct and
    /// [`Self::fields`] by hand, and asserted equal to both by
    /// `ai_chat_texts_en_and_es_complete_and_differ` in `tests.rs`.
    pub const FIELD_COUNT: usize = 31;

    /// Spanish copy, with full orthographic accents (not a transliteration).
    pub fn es() -> Self {
        Self {
            backend: "Motor".into(),
            model: "Modelo".into(),
            model_free_text_hint: "(predeterminado del transporte)".into(),
            permission_mode: "Modo de permisos".into(),
            permission_mode_unset: "Sin definir".into(),
            system_prompt: "Instrucci\u{f3}n del sistema".into(),
            allowed_tools: "Herramientas permitidas (separadas por comas)".into(),
            show_thinking: "Mostrar razonamiento".into(),
            show_tool_calls: "Mostrar llamadas a herramientas".into(),
            unsupported_here: "Este motor no admite esta opci\u{f3}n.".into(),
            apply: "Aplicar".into(),
            new_session: "Sesi\u{f3}n nueva".into(),
            settings: "Ajustes del chat".into(),
            send: "Enviar".into(),
            stop: "Detener".into(),
            thinking: "Pensando\u{2026}".into(),
            retry: "Reintentar".into(),
            retry_failed: "El reintento fall\u{f3}".into(),
            copy: "Copiar".into(),
            copy_message: "Copiar el mensaje".into(),
            scope: "\u{c1}mbito del chat".into(),
            scope_unknown: "\u{c1}mbito".into(),
            allowed_tools_hint: "leer, escribir".into(),
            composer_placeholder: "Preg\u{fa}ntale a {assistant} sobre este documento\u{2026}"
                .into(),
            composer_hint: "Intro para enviar \u{b7} May\u{fa}s+Intro para salto de l\u{ed}nea"
                .into(),
            composer_hint_busy: "Generando\u{2026} \u{b7} Esc para detener".into(),
            role_user: "T\u{fa}".into(),
            role_assistant: "Claude".into(),
            role_system: "Sistema".into(),
            role_thinking: "Pensando".into(),
            role_tool: "Herramienta".into(),
        }
    }

    /// Every field as `(name, value)`, in declaration order, so a test can
    /// assert completeness without reflection. Kept in sync with the struct
    /// and [`Self::FIELD_COUNT`] by hand.
    pub fn fields(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("backend", self.backend.as_str()),
            ("model", self.model.as_str()),
            ("model_free_text_hint", self.model_free_text_hint.as_str()),
            ("permission_mode", self.permission_mode.as_str()),
            ("permission_mode_unset", self.permission_mode_unset.as_str()),
            ("system_prompt", self.system_prompt.as_str()),
            ("allowed_tools", self.allowed_tools.as_str()),
            ("show_thinking", self.show_thinking.as_str()),
            ("show_tool_calls", self.show_tool_calls.as_str()),
            ("unsupported_here", self.unsupported_here.as_str()),
            ("apply", self.apply.as_str()),
            ("new_session", self.new_session.as_str()),
            ("settings", self.settings.as_str()),
            ("send", self.send.as_str()),
            ("stop", self.stop.as_str()),
            ("thinking", self.thinking.as_str()),
            ("retry", self.retry.as_str()),
            ("retry_failed", self.retry_failed.as_str()),
            ("copy", self.copy.as_str()),
            ("copy_message", self.copy_message.as_str()),
            ("scope", self.scope.as_str()),
            ("scope_unknown", self.scope_unknown.as_str()),
            ("allowed_tools_hint", self.allowed_tools_hint.as_str()),
            ("composer_placeholder", self.composer_placeholder.as_str()),
            ("composer_hint", self.composer_hint.as_str()),
            ("composer_hint_busy", self.composer_hint_busy.as_str()),
            ("role_user", self.role_user.as_str()),
            ("role_assistant", self.role_assistant.as_str()),
            ("role_system", self.role_system.as_str()),
            ("role_thinking", self.role_thinking.as_str()),
            ("role_tool", self.role_tool.as_str()),
        ]
    }
}
