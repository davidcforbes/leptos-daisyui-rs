//! The quick-action chips a chat composer offers, and the request prompt
//! each one seeds into the input box before the actor edits or sends it.

use super::texts::AiChatWorkspaceTexts;

/// A canned rewrite/expand-style request the composer can pre-fill.
/// `Translate` carries the target language because, unlike the rest, its
/// prompt is not static copy.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuickAction {
    /// Summarize the current answer or document.
    Summarize,
    /// Rewrite the current answer or document.
    Rewrite,
    /// Expand on the current answer or document.
    Expand,
    /// Fix grammar in the current answer or document.
    FixGrammar,
    /// Simplify the current answer or document.
    Simplify,
    /// Add more detail to the current answer or document.
    AddDetails,
    /// Convert the current answer or document to a table.
    ConvertToTable,
    /// Translate the current answer or document into an explicit language.
    Translate {
        /// The requested target language.
        language: String,
    },
}

impl QuickAction {
    /// Stable id for `data-*` hooks and telemetry.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Summarize => "summarize",
            Self::Rewrite => "rewrite",
            Self::Expand => "expand",
            Self::FixGrammar => "fix_grammar",
            Self::Simplify => "simplify",
            Self::AddDetails => "add_details",
            Self::ConvertToTable => "convert_to_table",
            Self::Translate { .. } => "translate",
        }
    }

    /// Every action that needs no free-form parameter, in a fixed-size
    /// array a composer can render as a static chip row. `Translate` is
    /// deliberately excluded — it needs a language, so it renders through
    /// its own control instead.
    pub fn all_static() -> [QuickAction; 7] {
        [
            Self::Summarize,
            Self::Rewrite,
            Self::Expand,
            Self::FixGrammar,
            Self::Simplify,
            Self::AddDetails,
            Self::ConvertToTable,
        ]
    }

    /// The composer prompt this action seeds, localized by `texts`.
    pub fn prompt(&self, texts: &AiChatWorkspaceTexts) -> String {
        match self {
            Self::Summarize => texts.qa_summarize.clone(),
            Self::Rewrite => texts.qa_rewrite.clone(),
            Self::Expand => texts.qa_expand.clone(),
            Self::FixGrammar => texts.qa_fix_grammar.clone(),
            Self::Simplify => texts.qa_simplify.clone(),
            Self::AddDetails => texts.qa_add_details.clone(),
            Self::ConvertToTable => texts.qa_convert_to_table.clone(),
            Self::Translate { language } => texts.qa_translate.replace("{language}", language),
        }
    }
}
