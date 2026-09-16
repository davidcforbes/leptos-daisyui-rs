//! Pure formatting/parsing helpers for [`super::AiChat`]'s optional usage
//! caption and settings popover. Kept separate from `style.rs` (which holds
//! purely presentational role/keystroke logic) because these shape the
//! `ai_chat_core` data model (`Usage`, `ChatSettings`) rather than daisyUI
//! classes — mirrors `data_table::types`' data-model role. No view code, so
//! it's unit-testable headlessly (see `tests.rs`).

use ai_chat_core::{Capabilities, ChatSettings, Citation, Usage};

/// Render a turn's [`Usage`] for the caption line, e.g.
/// `"$0.0021 · 1,234 in · 567 out"`. Returns `None` when there is no usage
/// yet, or it is all-zero (nothing meaningful to show yet) — the caller
/// should render nothing in that case.
pub fn format_usage(usage: Option<Usage>) -> Option<String> {
    let u = usage?;
    if u.cost_usd == 0.0 && u.input_tokens == 0 && u.output_tokens == 0 {
        return None;
    }
    Some(format!(
        "${:.4} \u{b7} {} in \u{b7} {} out",
        u.cost_usd,
        format_count(u.input_tokens),
        format_count(u.output_tokens)
    ))
}

/// Group a token count's digits with thousands separators (`1234` ->
/// `"1,234"`).
pub fn format_count(n: u64) -> String {
    let digits = n.to_string();
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

/// Render a turn's [`Usage`] as the header subtitle's token breakdown, e.g.
/// `"1,234 in · 900 cached · 567 out · 120 rsn"`.
///
/// `reasoning_tokens` is a SUBSET of `output_tokens`, never an addition to
/// it, so the two are shown side by side and never summed — a turn that spent
/// most of its completion budget thinking is otherwise indistinguishable from
/// a cheap one. `cached` is `cache_read_tokens`: prompt tokens served from the
/// provider's cache, which `input_tokens` (new prompt tokens only) excludes
/// even though both are billed.
pub fn format_usage_subtitle(u: &Usage) -> String {
    format!(
        "{} in \u{b7} {} cached \u{b7} {} out \u{b7} {} rsn",
        format_count(u.input_tokens),
        format_count(u.cache_read_tokens),
        format_count(u.output_tokens),
        format_count(u.reasoning_tokens),
    )
}

/// How the settings popover should render its Model field for the currently
/// selected backend.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModelFieldMode {
    /// No model list is published: a free-text box, the historical shape.
    FreeText,
    /// Several models are published: a native HTML `select` over them.
    Select(Vec<String>),
    /// Exactly one model is published: show it, but do not invite editing.
    Pinned(String),
}

/// Decide the Model field's shape from a backend's [`Capabilities`].
///
/// `None` (no capability list at all, e.g. a host that never fetched one)
/// and an empty `models` both mean FreeText — the legacy behaviour, which
/// every existing consumer keeps.
pub fn model_field_mode(caps: Option<&Capabilities>) -> ModelFieldMode {
    match caps {
        Some(c) if c.models.len() == 1 => ModelFieldMode::Pinned(c.models[0].clone()),
        Some(c) if !c.models.is_empty() => ModelFieldMode::Select(c.models.clone()),
        _ => ModelFieldMode::FreeText,
    }
}

/// Which settings rows a backend can honour, and in what shape.
///
/// `permission`, `thinking` and `tool_calls` describe SUPPORT, not
/// visibility: an unsupported toggle is still rendered, disabled and
/// explained, rather than hidden.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsRowSet {
    /// The Model field's shape — see [`model_field_mode`].
    pub model: ModelFieldMode,
    /// Whether this backend has CLI-style permission modes to pick from.
    pub permission: bool,
    /// Whether this backend can emit reasoning the panel could show.
    pub thinking: bool,
    /// Whether this backend can emit tool calls the panel could show.
    pub tool_calls: bool,
}

/// Derive the settings popover's row set from the selected backend's
/// [`Capabilities`].
///
/// `None` yields the legacy shape — free-text model, no permission row, both
/// toggles enabled — so a host that supplies no `backends` sees exactly the
/// popover it saw before.
pub fn settings_rows_for(caps: Option<&Capabilities>) -> SettingsRowSet {
    SettingsRowSet {
        model: model_field_mode(caps),
        permission: caps.is_some_and(|c| !c.permission_modes.is_empty()),
        thinking: caps.is_none_or(|c| c.supports_thinking),
        tool_calls: caps.is_none_or(|c| c.supports_tool_calls),
    }
}

/// Where a [`TranscriptAnnotation`] sits relative to the transcript.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnnotationAnchor {
    /// Before the first message.
    AtStart,
    /// Immediately after the message at this index. An index past the end
    /// clamps to the end rather than dropping the annotation.
    AfterMessage(usize),
    /// After the last message.
    AtEnd,
}

/// What an annotation is saying, which drives its tone and its
/// `data-chat-annotation-kind` hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnnotationKind {
    /// Neutral information ("Switched to {engine}", "Conversation reset").
    Notice,
    /// Something the user should notice ("Stopped", "Response truncated").
    Warning,
    /// Sources backing the preceding answer.
    Citations,
    /// Retrieved evidence shown alongside an answer.
    Evidence,
}

impl AnnotationKind {
    /// The `data-chat-annotation-kind` value for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationKind::Notice => "notice",
            AnnotationKind::Warning => "warning",
            AnnotationKind::Citations => "citations",
            AnnotationKind::Evidence => "evidence",
        }
    }
}

/// An annotation's content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum AnnotationBody {
    /// A line of prose, rendered as plain text (never markdown).
    Text(String),
    /// A list of sources, rendered as links where a `href` is present.
    Citations(Vec<Citation>),
}

/// A host-owned row interleaved into the transcript.
///
/// This is how "Switched to {engine} — conversation reset", "Stopped",
/// escalation, truncation and citations enter the transcript: `ChatSession`
/// has no `announce()`, and the panel must not invent provider vocabulary of
/// its own. A pattern above the panel builds these; the panel only places and
/// paints them.
#[derive(Clone, Debug, PartialEq)]
pub struct TranscriptAnnotation {
    /// Where the row sits relative to `ChatSession::messages`.
    pub anchor: AnnotationAnchor,
    /// The row's tone and DOM hook.
    pub kind: AnnotationKind,
    /// The row's content.
    pub body: AnnotationBody,
}

/// Bucket `annotations` into the `message_count + 1` slots of a transcript:
/// slot `0` renders before message `0`, slot `i + 1` renders after message
/// `i`, and slot `message_count` renders after the last message.
///
/// Each bucket holds indices INTO `annotations`, in render order.
/// [`AnnotationAnchor::AtStart`] entries are placed before any other entry
/// that lands in the same slot, which only matters on an empty transcript
/// where start and end are the same slot. An out-of-range
/// [`AnnotationAnchor::AfterMessage`] clamps to the end slot, so an
/// annotation minted against a transcript that has since been trimmed is
/// still shown rather than silently dropped.
pub fn annotation_slots(
    annotations: &[TranscriptAnnotation],
    message_count: usize,
) -> Vec<Vec<usize>> {
    let mut slots: Vec<Vec<usize>> = vec![Vec::new(); message_count + 1];
    for (i, a) in annotations.iter().enumerate() {
        if matches!(a.anchor, AnnotationAnchor::AtStart) {
            slots[0].push(i);
        }
    }
    for (i, a) in annotations.iter().enumerate() {
        match a.anchor {
            AnnotationAnchor::AtStart => {}
            AnnotationAnchor::AfterMessage(m) => {
                slots[m.saturating_add(1).min(message_count)].push(i);
            }
            AnnotationAnchor::AtEnd => slots[message_count].push(i),
        }
    }
    slots
}

/// Parse a comma-separated `allowed_tools` field into
/// `ChatSettings::allowed_tools`: trims each entry, drops empties, and
/// collapses an all-empty input to `None` (the engine's "no restriction"
/// default).
pub fn parse_allowed_tools(text: &str) -> Option<Vec<String>> {
    let tools: Vec<String> = text
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if tools.is_empty() { None } else { Some(tools) }
}

/// Format `ChatSettings::allowed_tools` back into the comma-separated text
/// used by the settings popover's text field.
pub fn format_allowed_tools(tools: &Option<Vec<String>>) -> String {
    tools.as_ref().map(|t| t.join(", ")).unwrap_or_default()
}

/// Build a [`ChatSettings`] from the settings popover's raw form field
/// values: blank text fields collapse to `None`, `tools_text` is
/// comma-split via [`parse_allowed_tools`].
pub fn settings_from_form_fields(
    model: &str,
    system_prompt: &str,
    tools_text: &str,
    show_thinking: bool,
    show_tool_calls: bool,
) -> ChatSettings {
    let model = model.trim();
    let system_prompt = system_prompt.trim();
    ChatSettings {
        model: (!model.is_empty()).then(|| model.to_string()),
        system_prompt: (!system_prompt.is_empty()).then(|| system_prompt.to_string()),
        allowed_tools: parse_allowed_tools(tools_text),
        show_thinking,
        show_tool_calls,
    }
}
