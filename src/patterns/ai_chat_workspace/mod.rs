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
    WorkspaceRefusal,
};
pub use component::{
    AiChatWorkspace, AiChatWorkspaceLayout, ReopenReason, WATCHDOG_MS, cancel_notice, is_terminal,
    notice_text, published_capabilities, reopen_announces_switch, reopen_notice, settings_for,
    tick_turn_id, watchdog_should_fire,
};
pub use engine_header::{
    EngineHeader, UsageFigures, WATCHDOG_FAILURE_KIND, cost_line, engine_is_metered,
    header_cancel_discarded, header_failure_kind, header_outcome_id, honesty_for,
    honesty_state_for_code, honesty_tone, usage_figures, usage_line,
};
pub use evidence::{GroundingVerdict, TurnEvidence};
pub use evidence_rail::{EvidenceRail, citations_of, evidence_of, facts_of};
pub use knowledge::{
    ChatPosture, CorpusQueryMode, CorpusScope, IngestPhase, IngestStatus, KnowledgeSelection,
    KnowledgeSource, MemoryDraft, MemoryRefusal, OfficeKnowledgeScope, RecallCorpus, RecallHit,
    RecallReceipt, guardrail_refusal,
};
pub use knowledge_rail::{
    KnowledgeDraft, KnowledgeSourceRail, any_ingest_in_flight, awaits_curation, corpus_choices,
    corpus_ingests, label_is_distinct_from_options, memory_class_choices, memory_flags,
    office_collections, personal_memory, scope_label, scope_value, with_ingest,
};
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
pub use quick_actions::{QuickAction, QuickActionBar};
pub use settings_rows::{
    CodexLever, DEFAULT_TEMPERATURE, EFFORT_CHOICES, ProviderSettingsRows, TuningDraft,
    effort_selection,
};
pub use status::{
    TurnNotice, TurnRecord, UsageTotals, canceled_partial, completed_answer, declined_limitations,
    failure_kind, lifecycle_id, outcome_id,
};
pub use texts::AiChatWorkspaceTexts;
