//! Opinionated AI-chat composite driven by a host-implemented backend, built
//! over the generic `components::ai_chat` / `components::ai_assistant_workspace`
//! primitives — shaped like `crate::patterns::helpdesk`.
//!
//! This module lands the pure vocabulary (`backend`, `provider`, `knowledge`,
//! `evidence`, `status`, `quick_actions`, `texts`) every later phase of the
//! showcase composite compiles against. The `AiChatWorkspace` component
//! itself, its fixture backend and the demo page land in later tasks; see
//! the epic's design plan under `doc/plans/` and the consumer guide at
//! `doc/components/ai_chat_workspace.md` (written once the composite lands).

mod backend;
mod evidence;
mod knowledge;
mod provider;
mod quick_actions;
mod status;
mod texts;

#[cfg(test)]
mod tests;

pub use backend::{
    ChatWorkspaceBackend, ChatWorkspaceError, ChatWorkspaceErrorKind, WorkspaceFuture,
};
pub use evidence::{GroundingVerdict, TurnEvidence};
pub use knowledge::{
    ChatPosture, CorpusQueryMode, CorpusScope, IngestPhase, IngestStatus, KnowledgeSelection,
    KnowledgeSource, MemoryDraft, MemoryRefusal, OfficeKnowledgeScope, RecallCorpus, RecallHit,
    RecallReceipt, guardrail_refusal,
};
pub use provider::{
    AvailabilityReasonCode, CodexLevers, GroqTuning, ModelSource, ProviderCard, ProviderTuning,
    ReasoningEffort, TuningSchema, desktop_provider_catalogue,
};
pub use quick_actions::QuickAction;
pub use status::{TurnRecord, UsageTotals, lifecycle_id};
pub use texts::AiChatWorkspaceTexts;
