//! Pure formatting/parsing helpers for [`super::AiChat`]'s optional usage
//! caption and settings popover. Kept separate from `style.rs` (which holds
//! purely presentational role/keystroke logic) because these shape the
//! `ai_chat_core` data model (`Usage`, `ChatSettings`) rather than daisyUI
//! classes — mirrors `data_table::types`' data-model role. No view code, so
//! it's unit-testable headlessly (see `tests.rs`).

use ai_chat_core::{ChatSettings, Usage};

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
