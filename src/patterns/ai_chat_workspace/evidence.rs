//! What backs one turn's answer: citations, the recall search that fed it,
//! qualified facts, and an overall grounding verdict.

use crate::components::ai_assistant_workspace::{AssistantContractError, AssistantFact};
use crate::components::ai_chat::Citation;

use super::knowledge::RecallReceipt;

/// The overall grounding outcome of a turn, independent of whether it
/// answered or declined (see `AnswerOutcome` in `ai_assistant_workspace`).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroundingVerdict {
    /// The answer drew on at least one corpus source.
    Grounded {
        /// How many distinct sources were used.
        sources: u32,
    },
    /// A grounded search ran but found nothing to answer from.
    NotFound,
    /// The engine answered from general knowledge, with no corpus involved.
    AssistantOnly,
}

impl GroundingVerdict {
    /// Stable id for the evidence rail's `data-ai-chat-grounding` hook.
    ///
    /// Never `{:?}`: `Grounded { sources }` would print its payload, so the
    /// hook would change value with the number of sources and no proof could
    /// assert on the verdict itself. The source COUNT is a separate hook.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Grounded { .. } => "grounded",
            Self::NotFound => "not_found",
            Self::AssistantOnly => "assistant_only",
        }
    }
}

/// Everything backing one turn's answer, assembled independently of the
/// answer text itself so a consumer can render "why" separately from
/// "what".
#[derive(Clone, Debug, PartialEq)]
pub struct TurnEvidence {
    /// Grounding citations shown alongside the answer.
    pub citations: Vec<Citation>,
    /// The recall search that fed this turn, if any.
    pub recall: Option<RecallReceipt>,
    /// Individually qualified facts backing the answer.
    pub facts: Vec<AssistantFact>,
    /// Material limitations kept beside the claims.
    pub limitations: Vec<String>,
    /// Host-localized evidence freshness.
    pub as_of: Option<String>,
    /// The overall grounding outcome.
    pub grounding: GroundingVerdict,
}

impl TurnEvidence {
    /// Replaces `facts` with the given results, failing closed on any
    /// single invalid fact rather than silently dropping it: partial
    /// evidence that looks complete is worse than no evidence at all.
    pub fn with_facts(
        mut self,
        facts: Vec<Result<AssistantFact, AssistantContractError>>,
    ) -> Result<Self, AssistantContractError> {
        let mut checked = Vec::with_capacity(facts.len());
        for fact in facts {
            checked.push(fact?);
        }
        self.facts = checked;
        Ok(self)
    }
}
