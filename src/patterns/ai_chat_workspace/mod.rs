//! Opinionated AI-chat composite driven by a host-implemented backend, built
//! over the generic `components::ai_chat` / `components::ai_assistant_workspace`
//! primitives — shaped like `crate::patterns::helpdesk`.
//!
//! The pure vocabulary (`backend`, `provider`, `knowledge`, `evidence`,
//! `status`, `quick_actions`, `texts`) is what every other file here compiles
//! against; `component` assembles it into [`AiChatWorkspace`], with the
//! engine header and the knowledge/evidence rails beside it, and `memory`
//! ships a scripted fixture backend behind the `test-mode` feature.

mod backend;
mod component;
mod engine_header;
mod evidence;
mod evidence_rail;
mod knowledge;
mod knowledge_rail;
#[cfg(feature = "test-mode")]
mod memory;
mod provider;
mod quick_actions;
mod settings_rows;
mod status;
mod texts;

#[cfg(test)]
mod composite_tests;
#[cfg(test)]
mod tests;

pub use backend::{
    ChatWorkspaceBackend, ChatWorkspaceError, ChatWorkspaceErrorKind, WorkspaceFuture,
};
pub use component::{AiChatWorkspace, WATCHDOG_MS, is_terminal, notice_text, settings_for};
pub use engine_header::{EngineHeader, cost_line, engine_is_metered, usage_line};
pub use evidence::{GroundingVerdict, TurnEvidence};
pub use evidence_rail::{EvidenceRail, citations_of};
pub use knowledge::{
    ChatPosture, CorpusQueryMode, CorpusScope, IngestPhase, IngestStatus, KnowledgeSelection,
    KnowledgeSource, MemoryDraft, MemoryRefusal, OfficeKnowledgeScope, RecallCorpus, RecallHit,
    RecallReceipt, guardrail_refusal,
};
pub use knowledge_rail::{KnowledgeSourceRail, corpus_choices, scope_label, scope_value};
#[cfg(feature = "test-mode")]
pub use memory::{
    BackendCall, ChatWorkspaceFault, FixtureClock, InMemoryChatWorkspaceBackend, PromptMatcher,
    SEED_ACTOR, SEED_FOLDER_COURT, SEED_FOLDER_INTAKE, SEED_NOW_MS, SEED_RECALL_QUERY,
    SEED_RECALL_SEARCH, ScriptedChatTransport, TurnScript,
};
pub use provider::{
    AvailabilityReasonCode, CodexLevers, GroqTuning, ModelSource, ProviderCard, ProviderTuning,
    ReasoningEffort, TuningSchema, card_ready_for_ask, card_unready_reason,
    desktop_provider_catalogue,
};
pub use quick_actions::QuickAction;
pub use settings_rows::{
    CodexLever, DEFAULT_TEMPERATURE, EFFORT_CHOICES, ProviderSettingsRows, TuningDraft,
};
pub use status::{
    TurnNotice, TurnRecord, UsageTotals, canceled_partial, completed_answer, lifecycle_id,
};
pub use texts::AiChatWorkspaceTexts;
