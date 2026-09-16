//! The transport seam the `AiChatWorkspace` composite calls; the host
//! implements each method over its own engine adapters and knowledge store.
//! Futures are `'static` and not `Send`, matching
//! `crate::patterns::helpdesk::backend`: wasm is single-threaded and every
//! call is awaited from `spawn_local`.

use std::future::Future;
use std::pin::Pin;

use crate::components::ai_assistant_workspace::{
    AssistantConnection, AssistantMemory, AssistantMemoryEntry, AssistantSettings, MemoryState,
};
use crate::components::ai_chat::{ChatSettings, ChatTransport};

use super::knowledge::{
    CorpusScope, IngestStatus, KnowledgeSelection, KnowledgeSource, MemoryDraft, MemoryRefusal,
    RecallReceipt,
};
use super::provider::{ProviderCard, ProviderTuning};
use super::status::TurnRecord;

/// A boxed, local future resolving to the backend's typed result.
pub type WorkspaceFuture<T> =
    Pin<Box<dyn Future<Output = Result<T, ChatWorkspaceError>> + 'static>>;

/// A display-safe error the composite can render without inspecting a
/// provider diagnostic string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatWorkspaceError {
    /// The typed category.
    pub kind: ChatWorkspaceErrorKind,
    /// A scrubbed, display-safe message.
    pub message: String,
}

impl std::fmt::Display for ChatWorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ChatWorkspaceError {}

/// What kind of problem a [`ChatWorkspaceError`] reports.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatWorkspaceErrorKind {
    /// The requested engine or knowledge source is not currently reachable.
    Unavailable,
    /// The host declined the request (a policy or capability gate).
    Refused,
    /// The referenced turn, memory entry or knowledge item does not exist.
    NotFound,
    /// The request never reached an answer.
    Network,
    /// The host or an upstream engine refused the request for some other
    /// reason.
    Upstream,
    /// The composite asked for something this backend does not implement.
    Unsupported,
}

/// Everything the `AiChatWorkspace` composite asks of its host: engine and
/// knowledge configuration, session transport, turn history, recall and
/// governed personal memory, and sign-in.
pub trait ChatWorkspaceBackend {
    /// The actor's accepted assistant settings (tier, engines, budget).
    fn settings(&self) -> WorkspaceFuture<AssistantSettings>;
    /// Every engine the host offers, with its picker/tuning metadata.
    fn providers(&self) -> WorkspaceFuture<Vec<ProviderCard>>;
    /// Every knowledge source (corpora, memory, office knowledge) available
    /// to mix into a turn.
    fn knowledge(&self) -> WorkspaceFuture<Vec<KnowledgeSource>>;
    /// Ask the host to (re)index one corpus scope.
    fn reindex(&self, scope: &CorpusScope) -> WorkspaceFuture<IngestStatus>;
    /// Open a live session against one engine with the given knowledge mix,
    /// chat settings and provider tuning.
    fn open_session(
        &self,
        engine_id: &str,
        knowledge: &KnowledgeSelection,
        settings: ChatSettings,
        tuning: ProviderTuning,
    ) -> WorkspaceFuture<Box<dyn ChatTransport>>;
    /// Apply provider-specific tuning to an already-configured engine.
    fn set_tuning(&self, engine_id: &str, tuning: ProviderTuning) -> WorkspaceFuture<()>;
    /// Fetch one turn's accepted lifecycle, usage and evidence.
    fn turn(&self, turn_id: &str) -> WorkspaceFuture<TurnRecord>;
    /// The id of the turn currently in flight, if any.
    fn current_turn_id(&self) -> Option<String>;
    /// Search across the actor's available recall corpora.
    fn recall(&self, query: &str) -> WorkspaceFuture<RecallReceipt>;
    /// Propose a personal-memory candidate; the host may refuse it outright
    /// (the inner `Result`) independently of a transport-level failure (the
    /// outer one).
    fn remember(
        &self,
        draft: MemoryDraft,
    ) -> WorkspaceFuture<Result<AssistantMemoryEntry, MemoryRefusal>>;
    /// Toggle memory use/capture for the actor's whole collection.
    fn set_memory_flags(
        &self,
        use_enabled: bool,
        capture_enabled: bool,
    ) -> WorkspaceFuture<AssistantMemory>;
    /// Confirm or withdraw one memory entry.
    fn set_memory_state(
        &self,
        id: &str,
        state: MemoryState,
    ) -> WorkspaceFuture<AssistantMemoryEntry>;
    /// Start a host-owned sign-in flow for one engine.
    fn sign_in(&self, engine_id: &str) -> WorkspaceFuture<AssistantConnection>;
    /// Sign out of one engine.
    fn sign_out(&self, engine_id: &str) -> WorkspaceFuture<AssistantConnection>;
}
