//! Every string the `AiChatWorkspace` composite, its header, its knowledge
//! panel and its quick actions render, in one struct built by literal.
//!
//! `Default` is English; [`AiChatWorkspaceTexts::es`] is the Spanish table. A
//! host overrides individual fields by struct-update syntax, exactly like
//! `crate::patterns::helpdesk::HelpdeskTexts`.

use super::evidence::GroundingVerdict;
use super::knowledge::{CorpusQueryMode, IngestPhase, IngestStatus, MemoryRefusal, RecallCorpus};
use super::provider::AvailabilityReasonCode;
use crate::components::ai_assistant_workspace::{
    AnswerOutcome, AttemptLifecycle, MemoryClass, MemoryState,
};

/// Every string the workspace composite renders. See the module doc for the
/// `Default`/[`Self::es`] convention.
#[derive(Clone, Debug, PartialEq)]
pub struct AiChatWorkspaceTexts {
    /// Stable id of the locale this table is written in, for the composite's
    /// `data-ai-chat-workspace-locale` hook and for a host that mixes tables.
    /// Not display copy: it is the discriminator a proof reads to tell which
    /// table is mounted without matching on a translated sentence.
    pub locale_id: String,
    /// Label for the engine picker control.
    pub engine: String,
    /// Badge shown when the turn is grounded in a corpus.
    pub grounded: String,
    /// Badge shown when the turn ran in general-assistant posture.
    pub assistant: String,
    /// Shown when the selected engine is configured but not enabled.
    pub not_enabled: String,
    /// Shown when the selected engine cannot currently be reached.
    pub unavailable: String,
    /// The header's sign-in button.
    pub sign_in: String,
    /// The header's sign-out button.
    pub sign_out: String,
    /// Label in front of the engine's remaining budget display.
    pub budget_remaining: String,
    /// Label for plan-covered usage, kept apart from metered usage.
    pub usage_plan: String,
    /// Label for metered (pay-as-you-go) usage.
    pub usage_metered: String,
    /// Label for the reasoning-token figure. Without it the usage row reads
    /// `Plan: 0  Metered: 908  41  96` and the last two numbers look like
    /// continuations of the metered one.
    pub usage_reasoning: String,
    /// Label for the output-token figure; see [`Self::usage_reasoning`].
    pub usage_output: String,
    /// Label for the tokens-per-second throughput readout.
    pub tokens_per_sec: String,
    /// Shown in place of a metered cost the backend did not report.
    pub cost_absent: String,
    /// Label for `AttemptLifecycle::Admitted`.
    pub state_admitted: String,
    /// Label for `AttemptLifecycle::Queued`.
    pub state_queued: String,
    /// Label for `AttemptLifecycle::Running`.
    pub state_running: String,
    /// Label for `AttemptLifecycle::Validating`.
    pub state_validating: String,
    /// Label for a completed attempt whose outcome answered the question.
    pub state_completed: String,
    /// Label for a completed attempt that healthily declined to answer.
    pub state_declined: String,
    /// Label for `AttemptLifecycle::Denied`.
    pub state_denied: String,
    /// Label for `AttemptLifecycle::Unavailable`.
    pub state_unavailable: String,
    /// Label for `AttemptLifecycle::Failed`.
    pub state_failed: String,
    /// Label for an ordinary cancellation that kept whatever output existed.
    pub state_canceled_kept: String,
    /// Label for a cancellation whose output the host discarded.
    pub state_canceled_discarded: String,
    /// Label for `AttemptLifecycle::Interrupted`.
    pub state_interrupted: String,
    /// Copy for `AvailabilityReasonCode::TierEffectsDisabled`.
    pub reason_tier_effects_disabled: String,
    /// Copy for `AvailabilityReasonCode::NotArmed`.
    pub reason_not_armed: String,
    /// Copy for `AvailabilityReasonCode::CliMissing`.
    pub reason_cli_missing: String,
    /// Copy for `AvailabilityReasonCode::CliVersionUnsupported`.
    pub reason_cli_version_unsupported: String,
    /// Copy for `AvailabilityReasonCode::CliVersionUntested`. Must never mention sign-in, credentials or expiry: an untested CLI is an environment fact, not a stale credential.
    pub reason_cli_version_untested: String,
    /// Copy for `AvailabilityReasonCode::CliProtocolUnsupported`.
    pub reason_cli_protocol_unsupported: String,
    /// Copy for `AvailabilityReasonCode::CredentialKeyUnavailable`.
    pub reason_credential_key_unavailable: String,
    /// Copy for `AvailabilityReasonCode::NotSignedIn`.
    pub reason_not_signed_in: String,
    /// Copy for `AvailabilityReasonCode::SignInExpired`.
    pub reason_sign_in_expired: String,
    /// Copy for `AvailabilityReasonCode::BudgetExhausted`.
    pub reason_budget_exhausted: String,
    /// Copy for `AvailabilityReasonCode::EngineBusy`.
    pub reason_engine_busy: String,
    /// Copy for `AvailabilityReasonCode::SignInShapeUnavailable`.
    pub reason_sign_in_shape_unavailable: String,
    /// Copy for `RefusalNextAction::RetryLater`.
    pub action_retry_later: String,
    /// Copy for `RefusalNextAction::OpenSettings`.
    pub action_open_settings: String,
    /// Copy for `RefusalNextAction::NewConversation`.
    pub action_new_conversation: String,
    /// Label for `CorpusScope::File`.
    pub scope_file: String,
    /// Label for `CorpusScope::Folder`.
    pub scope_folder: String,
    /// Label for `CorpusScope::All`.
    pub scope_all: String,
    /// Label for `CorpusScope::Custom`.
    pub scope_custom: String,
    /// The reindex-this-scope button.
    pub reindex: String,
    /// Label for `IngestPhase::Idle`.
    pub ingest_idle: String,
    /// Label for `IngestPhase::Walking`.
    pub ingest_walking: String,
    /// Label for `IngestPhase::Indexing`.
    pub ingest_indexing: String,
    /// Label for `IngestPhase::Clustering`.
    pub ingest_clustering: String,
    /// Label for `IngestPhase::Ready`.
    pub ingest_ready: String,
    /// Label for `IngestPhase::Failed`.
    pub ingest_failed: String,
    /// Label for `CorpusQueryMode::FullText`.
    pub query_full_text: String,
    /// Label for `CorpusQueryMode::Similarity`.
    pub query_similarity: String,
    /// Label for `CorpusQueryMode::Llm`.
    pub query_llm: String,
    /// Label for `CorpusQueryMode::Fused`.
    pub query_fused: String,
    /// Label for `ChatPosture::Grounded`.
    pub posture_grounded: String,
    /// Label for `ChatPosture::Assistant`.
    pub posture_assistant: String,
    /// Shown when a grounded turn found no supporting evidence. The EN copy is pinned exactly (not just non-empty) because it is the visible proof that a grounded posture withheld an ungrounded guess rather than making one up.
    ///
    /// It carries NO apostrophe, and that is load-bearing rather than a style
    /// choice. The transcript renders an assistant message through
    /// `editmark_core`'s markdown pipeline, which typographically substitutes
    /// a straight `'` for a right single quotation mark — so a sentence
    /// pinned for BYTE-EXACT comparison against what the actor reads cannot
    /// contain one. The first run of the knowledge browser lane caught this:
    /// the model held one character and the DOM held another, and only the
    /// exact comparison could see it. Pinned by
    /// `grounded_not_found_survives_the_markdown_renderer` in `tests.rs`.
    pub grounded_not_found: String,
    /// Display name for `MemoryClass::Language`.
    pub kind_language: String,
    /// Display name for `MemoryClass::Tone`.
    pub kind_tone: String,
    /// Display name for `MemoryClass::Verbosity`.
    pub kind_verbosity: String,
    /// Display name for `MemoryClass::WorkingMethod`.
    pub kind_working_method: String,
    /// Display name for `MemoryState::Candidate`.
    pub state_candidate: String,
    /// Display name for `MemoryState::Confirmed`.
    pub state_confirmed: String,
    /// Display name for `MemoryState::Withdrawn`.
    pub state_withdrawn: String,
    /// The use-memory toggle.
    pub memory_use: String,
    /// The capture-memory toggle.
    pub memory_capture: String,
    /// The remember-this button.
    pub remember: String,
    /// Shown when `guardrail_refusal` returns `ContainsMatterNumber`.
    pub refused_matter: String,
    /// Shown when `guardrail_refusal` returns `ContainsEmail`.
    pub refused_email: String,
    /// Shown when `guardrail_refusal` returns `ContainsPhone`.
    pub refused_phone: String,
    /// Label for `RecallCorpus::Words`.
    pub recall_words: String,
    /// Label for `RecallCorpus::Meaning`.
    pub recall_meaning: String,
    /// Label for `RecallCorpus::Graph`.
    pub recall_graph: String,
    /// Label for `RecallCorpus::Thread`.
    pub recall_thread: String,
    /// Label in front of a recall receipt id.
    pub receipt: String,
    /// Label for `OfficeKnowledgeScope::GroupImportant`.
    pub kb_scope_group_important: String,
    /// Label for `OfficeKnowledgeScope::Foundation`.
    pub kb_scope_foundation: String,
    /// Prompt seeded by `QuickAction::Summarize`.
    pub qa_summarize: String,
    /// Prompt seeded by `QuickAction::Rewrite`.
    pub qa_rewrite: String,
    /// Prompt seeded by `QuickAction::Expand`.
    pub qa_expand: String,
    /// Prompt seeded by `QuickAction::FixGrammar`.
    pub qa_fix_grammar: String,
    /// Prompt seeded by `QuickAction::Simplify`.
    pub qa_simplify: String,
    /// Prompt seeded by `QuickAction::AddDetails`.
    pub qa_add_details: String,
    /// Prompt seeded by `QuickAction::ConvertToTable`.
    pub qa_convert_to_table: String,
    /// Prompt template seeded by `QuickAction::Translate`; `{language}` is substituted.
    pub qa_translate: String,
    /// Toast template shown after switching engines; `{engine}` is substituted.
    pub switched_engine: String,
    /// Toast shown after starting a new conversation.
    pub conversation_reset: String,
    /// Toast shown after canceling an in-flight turn.
    pub stopped: String,
    /// Toast template shown after an escalation; `{from}`/`{to}` are substituted.
    pub escalated: String,
    /// Toast shown when a response was cut short.
    pub truncated: String,
    /// Toast shown when a turn timed out without completing.
    pub turn_timed_out: String,
    /// Toast shown after settings are reopened from a refusal action.
    pub settings_reopened: String,
    /// Shown when live/streaming updates cannot currently be delivered.
    pub live_not_available: String,
    /// Field label for the knowledge rail's corpus-scope select. A FIELD
    /// label, never one of the control's own options: a select whose
    /// accessible name repeats an option tells a screen-reader user what is
    /// currently chosen and nothing about what the control decides.
    pub corpus_scope_label: String,
    /// The corpus-scope select's "no corpus at all" option.
    pub scope_none: String,
    /// Field label for the knowledge rail's query-mode select.
    pub query_mode_label: String,
    /// Field label for the knowledge rail's posture select.
    pub posture_label: String,
    /// Field label for the reasoning-effort select.
    pub effort_label: String,
    /// The reasoning-effort select's leading option, chosen when the host has
    /// picked no effort and the engine runs at its own default. It exists so
    /// the control can never display a level the host never chose: with three
    /// options and nothing selected, a browser shows the FIRST one, and the
    /// actor reads "low" on a turn that will run at the engine's default.
    pub effort_engine_default: String,
    /// Field label for the temperature slider; the value is appended.
    pub temperature_label: String,
    /// Label for the Codex CLI's built-in web-search lever.
    pub codex_web_search: String,
    /// Label for the Codex CLI's suppress-plugins lever.
    pub codex_suppress_plugins: String,
    /// Label for the Codex CLI's disable-code-mode lever.
    pub codex_disable_code_mode: String,
    /// Immutable statement that the Codex CLI offers no MCP tool access. Not
    /// a control: there is nothing here for an actor to change.
    pub codex_no_mcp: String,
    /// Note shown for an engine that authenticates with a host-held key,
    /// in place of any input. A key must never be typeable in this panel.
    pub credential_note: String,
    /// Copy for `AvailabilityReasonCode::EngineProcessNotRunning`. Must name
    /// the runtime the actor starts, because "unavailable" alone leaves them
    /// with nothing to do.
    pub reason_engine_process_not_running: String,
    /// Copy for `AvailabilityReasonCode::ModelNotInstalled`. Must be about
    /// fetching a model, never about starting a process: those are two
    /// different actions and telling an actor the wrong one wastes their time.
    pub reason_model_not_installed: String,
    /// The honesty strip's copy when nothing is wrong. A strip that only
    /// ever appears on trouble teaches an actor that its absence means
    /// nothing was checked.
    pub honesty_ready: String,
    /// Transcript notice for a cancellation that KEPT whatever had streamed.
    pub canceled_kept_notice: String,
    /// Transcript notice for a cancellation whose partial output the host
    /// discarded. Distinct copy, because the two cancels leave the actor with
    /// different things.
    pub canceled_discarded_notice: String,
    /// Heading for the list of a declined answer's material limitations.
    pub limitations_label: String,
    /// Heading for the knowledge rail's per-corpus ingest list.
    pub corpus_label: String,
    /// Template for one corpus's ingest counts. `{indexed}`, `{seen}` and
    /// `{clusters}` are substituted.
    pub ingest_counts: String,
    /// Heading for the memory-store block (flags and recall).
    pub memory_store_label: String,
    /// Honesty row shown when the memory store did not answer. It must say
    /// that nothing was recalled AND nothing was written, because a store
    /// that is merely unreadable and one that dropped a write leave the
    /// actor in different positions.
    pub memory_offline: String,
    /// Field label for the recall query input.
    pub recall_label: String,
    /// The run-a-recall button.
    pub recall: String,
    /// Shown when a recall ran and matched nothing. Distinct from
    /// [`Self::memory_offline`]: a search that found nothing is an answer.
    pub recall_no_hits: String,
    /// Field label for the proposed memory's class select.
    pub remember_kind_label: String,
    /// Field label for the proposed memory's text.
    pub remember_label: String,
    /// Option label for the curated `lesson` class, offered so an actor can
    /// see the refusal it earns rather than being quietly prevented from
    /// asking.
    pub kind_lesson: String,
    /// Option label for the curated `principle` class.
    pub kind_principle: String,
    /// Shown when the host refuses a class only a curator may create.
    pub refused_curated_kind: String,
    /// Shown for a host refusal reason this vocabulary does not name; the
    /// host's own sentence is rendered beside it.
    pub refused_unknown: String,
    /// The curation-gap honesty line carried by every candidate entry. It
    /// says the entry is written and NOT yet findable by recall — the one
    /// property a write-then-read-back would otherwise hide.
    pub awaiting_curation: String,
    /// Heading for the actor's own memory entries.
    pub personal_memory_label: String,
    /// Heading for the curated organizational-knowledge collections.
    pub office_knowledge_label: String,
    /// The confirm-this-entry button.
    pub confirm: String,
    /// The withdraw-this-entry button.
    pub withdraw: String,
    /// Transcript notice for a reopen caused by a KNOWLEDGE change. The
    /// session really is rebuilt, so silence would hide a reset transcript;
    /// the engine did not change, so the switch wording would be false.
    pub knowledge_changed: String,
    /// Label for `GroundingVerdict::Grounded`.
    pub grounding_grounded: String,
    /// Label for `GroundingVerdict::NotFound`. A short verdict label, NOT
    /// [`Self::grounded_not_found`], which is the answer sentence itself.
    pub grounding_not_found: String,
    /// Label for `GroundingVerdict::AssistantOnly`.
    pub grounding_assistant_only: String,
    /// Heading for the evidence rail's qualified facts.
    pub facts_label: String,
    /// Label in front of the evidence's freshness stamp.
    pub as_of_label: String,
    /// Field label in front of the evidence rail's query-mode attribution.
    pub query_attribution_label: String,
}

impl Default for AiChatWorkspaceTexts {
    /// English copy.
    fn default() -> Self {
        Self {
            locale_id: "en".into(),
            engine: "Engine".into(),
            grounded: "Grounded".into(),
            assistant: "Assistant".into(),
            not_enabled: "Not enabled".into(),
            unavailable: "Unavailable".into(),
            sign_in: "Sign in".into(),
            sign_out: "Sign out".into(),
            budget_remaining: "Budget remaining".into(),
            usage_plan: "Plan usage".into(),
            usage_metered: "Metered usage".into(),
            usage_reasoning: "Reasoning tokens".into(),
            usage_output: "Output tokens".into(),
            tokens_per_sec: "Tokens per second".into(),
            cost_absent: "Cost not available".into(),
            state_admitted: "Admitted".into(),
            state_queued: "Queued".into(),
            state_running: "Running".into(),
            state_validating: "Validating".into(),
            state_completed: "Completed".into(),
            state_declined: "Answer declined".into(),
            state_denied: "Denied".into(),
            state_unavailable: "Unavailable".into(),
            state_failed: "Failed".into(),
            state_canceled_kept: "Canceled".into(),
            state_canceled_discarded: "Canceled (output discarded)".into(),
            state_interrupted: "Interrupted".into(),
            reason_tier_effects_disabled:
                "This effect isn't available on the account's current tier.".into(),
            reason_not_armed: "This engine hasn't been armed for use yet.".into(),
            reason_cli_missing: "The command-line tool for this engine isn't installed.".into(),
            reason_cli_version_unsupported:
                "The installed CLI version is too old to support this workspace.".into(),
            reason_cli_version_untested:
                "This CLI version has not yet been verified for this workspace.".into(),
            reason_cli_protocol_unsupported:
                "The CLI speaks a protocol this workspace doesn't support.".into(),
            reason_credential_key_unavailable: "No credential key is available for this engine."
                .into(),
            reason_not_signed_in: "You're not signed in to this engine.".into(),
            reason_sign_in_expired: "Your sign-in for this engine has expired.".into(),
            reason_budget_exhausted: "This engine's budget has been used up.".into(),
            reason_engine_busy: "This engine is busy with another request.".into(),
            reason_sign_in_shape_unavailable:
                "The sign-in method for this engine isn't available right now.".into(),
            action_retry_later: "Try again later".into(),
            action_open_settings: "Open settings".into(),
            action_new_conversation: "Start a new conversation".into(),
            scope_file: "This file".into(),
            scope_folder: "This folder".into(),
            scope_all: "Everything".into(),
            scope_custom: "Custom selection".into(),
            reindex: "Reindex".into(),
            ingest_idle: "Idle".into(),
            ingest_walking: "Scanning files".into(),
            ingest_indexing: "Indexing".into(),
            ingest_clustering: "Clustering".into(),
            ingest_ready: "Ready".into(),
            ingest_failed: "Failed".into(),
            query_full_text: "Full text".into(),
            query_similarity: "Similarity".into(),
            query_llm: "AI-generated".into(),
            query_fused: "Fused".into(),
            posture_grounded: "Grounded in this folder".into(),
            posture_assistant: "General assistant".into(),
            grounded_not_found: "I could not find that in this folder.".into(),
            kind_language: "Language".into(),
            kind_tone: "Tone".into(),
            kind_verbosity: "Verbosity".into(),
            kind_working_method: "Working method".into(),
            state_candidate: "Candidate".into(),
            state_confirmed: "Confirmed".into(),
            state_withdrawn: "Withdrawn".into(),
            memory_use: "Use memory".into(),
            memory_capture: "Capture memory".into(),
            remember: "Remember this".into(),
            refused_matter: "This looks like it includes a matter number, so it wasn't saved."
                .into(),
            refused_email: "This looks like it includes an email address, so it wasn't saved."
                .into(),
            refused_phone: "This looks like it includes a phone number, so it wasn't saved.".into(),
            recall_words: "Keyword search".into(),
            recall_meaning: "Meaning search".into(),
            recall_graph: "Graph search".into(),
            recall_thread: "Conversation search".into(),
            receipt: "Receipt".into(),
            kb_scope_group_important: "Group Important".into(),
            kb_scope_foundation: "Foundation".into(),
            qa_summarize: "Summarize this".into(),
            qa_rewrite: "Rewrite this".into(),
            qa_expand: "Expand on this".into(),
            qa_fix_grammar: "Fix the grammar".into(),
            qa_simplify: "Simplify this".into(),
            qa_add_details: "Add more details".into(),
            qa_convert_to_table: "Convert this to a table".into(),
            qa_translate: "Translate this to {language}".into(),
            switched_engine: "Switched to {engine}".into(),
            conversation_reset: "Conversation reset".into(),
            stopped: "Stopped".into(),
            escalated: "Escalated from {from} to {to}".into(),
            truncated: "Response truncated".into(),
            turn_timed_out: "The turn timed out".into(),
            settings_reopened: "Settings reopened".into(),
            live_not_available: "Live updates aren't available right now".into(),
            corpus_scope_label: "Knowledge source".into(),
            scope_none: "No corpus".into(),
            query_mode_label: "Query mode".into(),
            posture_label: "Answer posture".into(),
            effort_label: "Reasoning effort".into(),
            effort_engine_default: "Engine default".into(),
            temperature_label: "Temperature".into(),
            codex_web_search: "Web search".into(),
            codex_suppress_plugins: "Suppress plugins".into(),
            codex_disable_code_mode: "Disable code mode".into(),
            codex_no_mcp: "This engine never offers MCP tool access.".into(),
            credential_note: "This engine authenticates with a key the host holds. A key is never typed into this panel.".into(),
            reason_engine_process_not_running:
                "This engine's local runtime isn't running. Start it (for example, `ollama serve`) and try again."
                    .into(),
            reason_model_not_installed:
                "The selected model isn't installed on this machine. Fetch it first (for example, `ollama pull llama3.1:8b`)."
                    .into(),
            honesty_ready: "Ready to answer".into(),
            canceled_kept_notice: "You stopped this answer. What had already arrived is kept.".into(),
            canceled_discarded_notice: "You stopped this answer, and the partial output was discarded.".into(),
            limitations_label: "What this answer could not cover".into(),
            corpus_label: "Documents".into(),
            ingest_counts: "{indexed}/{seen} files \u{b7} {clusters} clusters".into(),
            memory_store_label: "Memory store".into(),
            memory_offline: "The memory store did not answer. Nothing was recalled and nothing was written.".into(),
            recall_label: "Search your memory".into(),
            recall: "Recall".into(),
            recall_no_hits: "That search matched nothing.".into(),
            remember_kind_label: "Kind".into(),
            remember_label: "Remember this".into(),
            kind_lesson: "Lesson".into(),
            kind_principle: "Principle".into(),
            refused_curated_kind: "Lessons and principles are written by curators, not saved directly.".into(),
            refused_unknown: "The memory store refused this.".into(),
            awaiting_curation: "Written, awaiting curation. A curator has to confirm it before recall can find it.".into(),
            personal_memory_label: "Your memory".into(),
            office_knowledge_label: "Office knowledge".into(),
            confirm: "Confirm".into(),
            withdraw: "Withdraw".into(),
            knowledge_changed: "Knowledge sources changed, so this conversation started over.".into(),
            grounding_grounded: "Grounded in the documents".into(),
            grounding_not_found: "Nothing found in the documents".into(),
            grounding_assistant_only: "General assistance, no documents".into(),
            facts_label: "Facts behind this answer".into(),
            as_of_label: "As of".into(),
            query_attribution_label: "Queried by".into(),
        }
    }
}

impl AiChatWorkspaceTexts {
    /// The number of fields on this struct; kept in sync with the struct and
    /// [`Self::fields`] by hand, and asserted equal to both by
    /// `en_and_es_texts_are_complete_and_differ` in `tests.rs`.
    pub const FIELD_COUNT: usize = 139;

    /// Spanish copy, with full orthographic accents (not a transliteration).
    pub fn es() -> Self {
        Self {
            locale_id: "es".into(),
            engine: "Motor".into(),
            grounded: "Fundamentado".into(),
            assistant: "Asistente".into(),
            not_enabled: "No habilitado".into(),
            unavailable: "No disponible".into(),
            sign_in: "Iniciar sesión".into(),
            sign_out: "Cerrar sesión".into(),
            budget_remaining: "Presupuesto restante".into(),
            usage_plan: "Uso del plan".into(),
            usage_metered: "Uso medido".into(),
            usage_reasoning: "Tokens de razonamiento".into(),
            usage_output: "Tokens de salida".into(),
            tokens_per_sec: "Tokens por segundo".into(),
            cost_absent: "Costo no disponible".into(),
            state_admitted: "Admitido".into(),
            state_queued: "En cola".into(),
            state_running: "En ejecución".into(),
            state_validating: "Validando".into(),
            state_completed: "Completado".into(),
            state_declined: "Respuesta declinada".into(),
            state_denied: "Denegado".into(),
            state_unavailable: "No disponible".into(),
            state_failed: "Fallido".into(),
            state_canceled_kept: "Cancelado".into(),
            state_canceled_discarded: "Cancelado (resultado descartado)".into(),
            state_interrupted: "Interrumpido".into(),
            reason_tier_effects_disabled:
                "Este efecto no está disponible en el nivel actual de la cuenta.".into(),
            reason_not_armed: "Este motor aún no se ha activado para su uso.".into(),
            reason_cli_missing:
                "No está instalada la herramienta de línea de comandos para este motor.".into(),
            reason_cli_version_unsupported:
                "La versión instalada de la CLI es demasiado antigua para este espacio de trabajo."
                    .into(),
            reason_cli_version_untested:
                "Esta versión de la CLI aún no se ha verificado para este espacio de trabajo."
                    .into(),
            reason_cli_protocol_unsupported:
                "La CLI utiliza un protocolo que este espacio de trabajo no admite.".into(),
            reason_credential_key_unavailable:
                "No hay una clave de credencial disponible para este motor.".into(),
            reason_not_signed_in: "No has iniciado sesión en este motor.".into(),
            reason_sign_in_expired: "Tu sesión en este motor ha caducado.".into(),
            reason_budget_exhausted: "El presupuesto de este motor se ha agotado.".into(),
            reason_engine_busy: "Este motor está ocupado con otra solicitud.".into(),
            reason_sign_in_shape_unavailable:
                "El método de inicio de sesión de este motor no está disponible en este momento."
                    .into(),
            action_retry_later: "Inténtalo más tarde".into(),
            action_open_settings: "Abrir configuración".into(),
            action_new_conversation: "Iniciar una conversación nueva".into(),
            scope_file: "Este archivo".into(),
            scope_folder: "Esta carpeta".into(),
            scope_all: "Todo".into(),
            scope_custom: "Selección personalizada".into(),
            reindex: "Reindexar".into(),
            ingest_idle: "Inactivo".into(),
            ingest_walking: "Explorando archivos".into(),
            ingest_indexing: "Indexando".into(),
            ingest_clustering: "Agrupando".into(),
            ingest_ready: "Listo".into(),
            ingest_failed: "Fallido".into(),
            query_full_text: "Texto completo".into(),
            query_similarity: "Similitud".into(),
            query_llm: "Generado por IA".into(),
            query_fused: "Combinado".into(),
            posture_grounded: "Fundamentado en esta carpeta".into(),
            posture_assistant: "Asistente general".into(),
            grounded_not_found: "No pude encontrar eso en esta carpeta.".into(),
            kind_language: "Idioma".into(),
            kind_tone: "Tono".into(),
            kind_verbosity: "Nivel de detalle".into(),
            kind_working_method: "Método de trabajo".into(),
            state_candidate: "Candidato".into(),
            state_confirmed: "Confirmado".into(),
            state_withdrawn: "Retirado".into(),
            memory_use: "Usar memoria".into(),
            memory_capture: "Capturar memoria".into(),
            remember: "Recordar esto".into(),
            refused_matter: "Esto parece incluir un número de asunto, por lo que no se guardó."
                .into(),
            refused_email:
                "Esto parece incluir una dirección de correo electrónico, por lo que no se guardó."
                    .into(),
            refused_phone: "Esto parece incluir un número de teléfono, por lo que no se guardó."
                .into(),
            recall_words: "Búsqueda por palabras clave".into(),
            recall_meaning: "Búsqueda por significado".into(),
            recall_graph: "Búsqueda por grafo".into(),
            recall_thread: "Búsqueda en la conversación".into(),
            receipt: "Comprobante".into(),
            kb_scope_group_important: "Importante del grupo".into(),
            kb_scope_foundation: "Fundamento".into(),
            qa_summarize: "Resumir esto".into(),
            qa_rewrite: "Reescribir esto".into(),
            qa_expand: "Ampliar esto".into(),
            qa_fix_grammar: "Corregir la gramática".into(),
            qa_simplify: "Simplificar esto".into(),
            qa_add_details: "Agregar más detalles".into(),
            qa_convert_to_table: "Convertir esto en una tabla".into(),
            qa_translate: "Traducir esto a {language}".into(),
            switched_engine: "Se cambió a {engine}".into(),
            conversation_reset: "Conversación reiniciada".into(),
            stopped: "Detenido".into(),
            escalated: "Escalado de {from} a {to}".into(),
            truncated: "Respuesta truncada".into(),
            turn_timed_out: "El turno superó el tiempo de espera".into(),
            settings_reopened: "Configuración reabierta".into(),
            live_not_available: "Las actualizaciones en vivo no están disponibles en este momento"
                .into(),
            corpus_scope_label: "Fuente de conocimiento".into(),
            scope_none: "Sin corpus".into(),
            query_mode_label: "Modo de consulta".into(),
            posture_label: "Postura de respuesta".into(),
            effort_label: "Esfuerzo de razonamiento".into(),
            effort_engine_default: "Valor predeterminado del motor".into(),
            temperature_label: "Temperatura".into(),
            codex_web_search: "Búsqueda web".into(),
            codex_suppress_plugins: "Suprimir los complementos".into(),
            codex_disable_code_mode: "Desactivar el modo de código".into(),
            codex_no_mcp: "Este motor nunca ofrece acceso a herramientas MCP.".into(),
            credential_note: "Este motor se autentica con una clave que guarda el anfitrión. Nunca se escribe una clave en este panel.".into(),
            reason_engine_process_not_running:
                "El entorno local de este motor no está en ejecución. Inícialo (por ejemplo, `ollama serve`) e inténtalo de nuevo."
                    .into(),
            reason_model_not_installed:
                "El modelo seleccionado no está instalado en esta máquina. Descárgalo primero (por ejemplo, `ollama pull llama3.1:8b`)."
                    .into(),
            honesty_ready: "Listo para responder".into(),
            canceled_kept_notice: "Detuviste esta respuesta. Se conserva lo que ya había llegado.".into(),
            canceled_discarded_notice: "Detuviste esta respuesta y se descartó el resultado parcial.".into(),
            limitations_label: "Lo que esta respuesta no pudo cubrir".into(),
            corpus_label: "Documentos".into(),
            ingest_counts: "{indexed}/{seen} archivos \u{b7} {clusters} gr\u{fa}pos".into(),
            memory_store_label: "Almac\u{e9}n de memoria".into(),
            memory_offline: "El almac\u{e9}n de memoria no respondi\u{f3}. No se recuper\u{f3} ni se guard\u{f3} nada.".into(),
            recall_label: "Buscar en tu memoria".into(),
            recall: "Recuperar".into(),
            recall_no_hits: "Esa b\u{fa}squeda no coincidi\u{f3} con nada.".into(),
            remember_kind_label: "Tipo".into(),
            remember_label: "Recordar esto".into(),
            kind_lesson: "Lecci\u{f3}n".into(),
            kind_principle: "Principio".into(),
            refused_curated_kind: "Las lecciones y los principios los escriben los curadores, no se guardan directamente.".into(),
            refused_unknown: "El almac\u{e9}n de memoria rechaz\u{f3} esto.".into(),
            awaiting_curation: "Guardado, pendiente de curaci\u{f3}n. Un curador debe confirmarlo antes de que la b\u{fa}squeda pueda encontrarlo.".into(),
            personal_memory_label: "Tu memoria".into(),
            office_knowledge_label: "Conocimiento del despacho".into(),
            confirm: "Confirmar".into(),
            withdraw: "Retirar".into(),
            knowledge_changed: "Las fuentes de conocimiento cambiaron, as\u{ed} que esta conversaci\u{f3}n empez\u{f3} de nuevo.".into(),
            grounding_grounded: "Fundamentado en los documentos".into(),
            grounding_not_found: "No se encontr\u{f3} nada en los documentos".into(),
            grounding_assistant_only: "Asistencia general, sin documentos".into(),
            facts_label: "Datos detr\u{e1}s de esta respuesta".into(),
            as_of_label: "Al d\u{ed}a".into(),
            query_attribution_label: "Consultado por".into(),
        }
    }

    /// Localized copy for one availability reason code; `Unknown` shows the
    /// code itself, exactly like a host reason with no committed translation.
    pub fn availability_reason(&self, code: &AvailabilityReasonCode) -> String {
        match code {
            AvailabilityReasonCode::TierEffectsDisabled => {
                self.reason_tier_effects_disabled.clone()
            }
            AvailabilityReasonCode::NotArmed => self.reason_not_armed.clone(),
            AvailabilityReasonCode::CliMissing => self.reason_cli_missing.clone(),
            AvailabilityReasonCode::CliVersionUnsupported => {
                self.reason_cli_version_unsupported.clone()
            }
            AvailabilityReasonCode::CliVersionUntested => self.reason_cli_version_untested.clone(),
            AvailabilityReasonCode::CliProtocolUnsupported => {
                self.reason_cli_protocol_unsupported.clone()
            }
            AvailabilityReasonCode::CredentialKeyUnavailable => {
                self.reason_credential_key_unavailable.clone()
            }
            AvailabilityReasonCode::NotSignedIn => self.reason_not_signed_in.clone(),
            AvailabilityReasonCode::SignInExpired => self.reason_sign_in_expired.clone(),
            AvailabilityReasonCode::BudgetExhausted => self.reason_budget_exhausted.clone(),
            AvailabilityReasonCode::EngineBusy => self.reason_engine_busy.clone(),
            AvailabilityReasonCode::SignInShapeUnavailable => {
                self.reason_sign_in_shape_unavailable.clone()
            }
            AvailabilityReasonCode::EngineProcessNotRunning => {
                self.reason_engine_process_not_running.clone()
            }
            AvailabilityReasonCode::ModelNotInstalled => self.reason_model_not_installed.clone(),
            AvailabilityReasonCode::Unknown(code) => code.clone(),
        }
    }

    /// Human copy for one attempt lifecycle state. A completed attempt reads
    /// its nested outcome to distinguish an answer from a healthy decline; an
    /// unrecognized lifecycle shows the host's own code, exactly like
    /// [`Self::availability_reason`]'s `Unknown` case.
    pub fn lifecycle_label(&self, l: &AttemptLifecycle) -> String {
        match l {
            AttemptLifecycle::Admitted => self.state_admitted.clone(),
            AttemptLifecycle::Queued => self.state_queued.clone(),
            AttemptLifecycle::Running => self.state_running.clone(),
            AttemptLifecycle::Validating => self.state_validating.clone(),
            AttemptLifecycle::Completed(answer) => match &answer.outcome {
                AnswerOutcome::Declined { .. } => self.state_declined.clone(),
                AnswerOutcome::Answered { .. } | AnswerOutcome::Unknown(_) => {
                    self.state_completed.clone()
                }
            },
            AttemptLifecycle::Denied { .. } => self.state_denied.clone(),
            AttemptLifecycle::Unavailable { .. } => self.state_unavailable.clone(),
            AttemptLifecycle::Failed { .. } => self.state_failed.clone(),
            AttemptLifecycle::Canceled { discarded: true } => self.state_canceled_discarded.clone(),
            AttemptLifecycle::Canceled { discarded: false } => self.state_canceled_kept.clone(),
            AttemptLifecycle::Interrupted { .. } => self.state_interrupted.clone(),
            AttemptLifecycle::Unknown(code) => code.clone(),
        }
    }

    /// Human copy for one ingest phase. `Failed` carries the host's own
    /// reason SEPARATELY (the rail renders it on
    /// `data-ai-chat-ingest-error`), so this is the phase name only.
    pub fn ingest_phase_label(&self, phase: &IngestPhase) -> String {
        match phase {
            IngestPhase::Idle => self.ingest_idle.clone(),
            IngestPhase::Walking => self.ingest_walking.clone(),
            IngestPhase::Indexing => self.ingest_indexing.clone(),
            IngestPhase::Clustering => self.ingest_clustering.clone(),
            IngestPhase::Ready => self.ingest_ready.clone(),
            IngestPhase::Failed(_) => self.ingest_failed.clone(),
        }
    }

    /// Human copy for one corpus query mode.
    pub fn query_mode_name(&self, mode: CorpusQueryMode) -> String {
        match mode {
            CorpusQueryMode::FullText => self.query_full_text.clone(),
            CorpusQueryMode::Similarity => self.query_similarity.clone(),
            CorpusQueryMode::Llm => self.query_llm.clone(),
            CorpusQueryMode::Fused => self.query_fused.clone(),
        }
    }

    /// Human copy for one recall corpus. An `Unknown` lane shows the host's
    /// own id rather than being narrated as one of the four named lanes —
    /// claiming a hit "matched by meaning" when the service never said so is
    /// precisely the overreach the attribution exists to prevent.
    pub fn recall_corpus_label(&self, corpus: &RecallCorpus) -> String {
        match corpus {
            RecallCorpus::Words => self.recall_words.clone(),
            RecallCorpus::Meaning => self.recall_meaning.clone(),
            RecallCorpus::Graph => self.recall_graph.clone(),
            RecallCorpus::Thread => self.recall_thread.clone(),
            RecallCorpus::Unknown(id) => id.clone(),
        }
    }

    /// Human copy for one memory class; an unnamed host class shows its own
    /// id, except for the two curated classes this table does name.
    pub fn memory_class_label(&self, class: &MemoryClass) -> String {
        match class {
            MemoryClass::Language => self.kind_language.clone(),
            MemoryClass::Tone => self.kind_tone.clone(),
            MemoryClass::Verbosity => self.kind_verbosity.clone(),
            MemoryClass::WorkingMethod => self.kind_working_method.clone(),
            MemoryClass::Unknown(id) if id == "lesson" => self.kind_lesson.clone(),
            MemoryClass::Unknown(id) if id == "principle" => self.kind_principle.clone(),
            MemoryClass::Unknown(id) => id.clone(),
        }
    }

    /// Human copy for one memory state.
    pub fn memory_state_label(&self, state: &MemoryState) -> String {
        match state {
            MemoryState::Candidate => self.state_candidate.clone(),
            MemoryState::Confirmed => self.state_confirmed.clone(),
            MemoryState::Withdrawn => self.state_withdrawn.clone(),
            MemoryState::Unknown(id) => id.clone(),
        }
    }

    /// Human copy for one memory refusal. `Unknown` shows this table's
    /// generic sentence; the host's own reason is rendered beside it by the
    /// rail, never in place of it.
    pub fn memory_refusal_label(&self, refusal: &MemoryRefusal) -> String {
        match refusal {
            MemoryRefusal::ContainsMatterNumber => self.refused_matter.clone(),
            MemoryRefusal::ContainsEmail => self.refused_email.clone(),
            MemoryRefusal::ContainsPhone => self.refused_phone.clone(),
            MemoryRefusal::CuratedKindOnly => self.refused_curated_kind.clone(),
            MemoryRefusal::Unknown(_) => self.refused_unknown.clone(),
        }
    }

    /// Human copy for one grounding verdict.
    pub fn grounding_label(&self, verdict: &GroundingVerdict) -> String {
        match verdict {
            GroundingVerdict::Grounded { .. } => self.grounding_grounded.clone(),
            GroundingVerdict::NotFound => self.grounding_not_found.clone(),
            GroundingVerdict::AssistantOnly => self.grounding_assistant_only.clone(),
        }
    }

    /// One corpus's ingest counts, substituted into [`Self::ingest_counts`].
    pub fn ingest_counts_line(&self, status: &IngestStatus) -> String {
        self.ingest_counts
            .replace("{indexed}", &status.files_indexed.to_string())
            .replace("{seen}", &status.files_seen.to_string())
            .replace("{clusters}", &status.clusters.to_string())
    }

    /// Every field, in declaration order, for the completeness test. Keep in
    /// sync with the struct and [`Self::FIELD_COUNT`] by hand.
    pub fn fields(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("locale_id", self.locale_id.as_str()),
            ("engine", self.engine.as_str()),
            ("grounded", self.grounded.as_str()),
            ("assistant", self.assistant.as_str()),
            ("not_enabled", self.not_enabled.as_str()),
            ("unavailable", self.unavailable.as_str()),
            ("sign_in", self.sign_in.as_str()),
            ("sign_out", self.sign_out.as_str()),
            ("budget_remaining", self.budget_remaining.as_str()),
            ("usage_plan", self.usage_plan.as_str()),
            ("usage_metered", self.usage_metered.as_str()),
            ("usage_reasoning", self.usage_reasoning.as_str()),
            ("usage_output", self.usage_output.as_str()),
            ("tokens_per_sec", self.tokens_per_sec.as_str()),
            ("cost_absent", self.cost_absent.as_str()),
            ("state_admitted", self.state_admitted.as_str()),
            ("state_queued", self.state_queued.as_str()),
            ("state_running", self.state_running.as_str()),
            ("state_validating", self.state_validating.as_str()),
            ("state_completed", self.state_completed.as_str()),
            ("state_declined", self.state_declined.as_str()),
            ("state_denied", self.state_denied.as_str()),
            ("state_unavailable", self.state_unavailable.as_str()),
            ("state_failed", self.state_failed.as_str()),
            ("state_canceled_kept", self.state_canceled_kept.as_str()),
            (
                "state_canceled_discarded",
                self.state_canceled_discarded.as_str(),
            ),
            ("state_interrupted", self.state_interrupted.as_str()),
            (
                "reason_tier_effects_disabled",
                self.reason_tier_effects_disabled.as_str(),
            ),
            ("reason_not_armed", self.reason_not_armed.as_str()),
            ("reason_cli_missing", self.reason_cli_missing.as_str()),
            (
                "reason_cli_version_unsupported",
                self.reason_cli_version_unsupported.as_str(),
            ),
            (
                "reason_cli_version_untested",
                self.reason_cli_version_untested.as_str(),
            ),
            (
                "reason_cli_protocol_unsupported",
                self.reason_cli_protocol_unsupported.as_str(),
            ),
            (
                "reason_credential_key_unavailable",
                self.reason_credential_key_unavailable.as_str(),
            ),
            ("reason_not_signed_in", self.reason_not_signed_in.as_str()),
            (
                "reason_sign_in_expired",
                self.reason_sign_in_expired.as_str(),
            ),
            (
                "reason_budget_exhausted",
                self.reason_budget_exhausted.as_str(),
            ),
            ("reason_engine_busy", self.reason_engine_busy.as_str()),
            (
                "reason_sign_in_shape_unavailable",
                self.reason_sign_in_shape_unavailable.as_str(),
            ),
            ("action_retry_later", self.action_retry_later.as_str()),
            ("action_open_settings", self.action_open_settings.as_str()),
            (
                "action_new_conversation",
                self.action_new_conversation.as_str(),
            ),
            ("scope_file", self.scope_file.as_str()),
            ("scope_folder", self.scope_folder.as_str()),
            ("scope_all", self.scope_all.as_str()),
            ("scope_custom", self.scope_custom.as_str()),
            ("reindex", self.reindex.as_str()),
            ("ingest_idle", self.ingest_idle.as_str()),
            ("ingest_walking", self.ingest_walking.as_str()),
            ("ingest_indexing", self.ingest_indexing.as_str()),
            ("ingest_clustering", self.ingest_clustering.as_str()),
            ("ingest_ready", self.ingest_ready.as_str()),
            ("ingest_failed", self.ingest_failed.as_str()),
            ("query_full_text", self.query_full_text.as_str()),
            ("query_similarity", self.query_similarity.as_str()),
            ("query_llm", self.query_llm.as_str()),
            ("query_fused", self.query_fused.as_str()),
            ("posture_grounded", self.posture_grounded.as_str()),
            ("posture_assistant", self.posture_assistant.as_str()),
            ("grounded_not_found", self.grounded_not_found.as_str()),
            ("kind_language", self.kind_language.as_str()),
            ("kind_tone", self.kind_tone.as_str()),
            ("kind_verbosity", self.kind_verbosity.as_str()),
            ("kind_working_method", self.kind_working_method.as_str()),
            ("state_candidate", self.state_candidate.as_str()),
            ("state_confirmed", self.state_confirmed.as_str()),
            ("state_withdrawn", self.state_withdrawn.as_str()),
            ("memory_use", self.memory_use.as_str()),
            ("memory_capture", self.memory_capture.as_str()),
            ("remember", self.remember.as_str()),
            ("refused_matter", self.refused_matter.as_str()),
            ("refused_email", self.refused_email.as_str()),
            ("refused_phone", self.refused_phone.as_str()),
            ("recall_words", self.recall_words.as_str()),
            ("recall_meaning", self.recall_meaning.as_str()),
            ("recall_graph", self.recall_graph.as_str()),
            ("recall_thread", self.recall_thread.as_str()),
            ("receipt", self.receipt.as_str()),
            (
                "kb_scope_group_important",
                self.kb_scope_group_important.as_str(),
            ),
            ("kb_scope_foundation", self.kb_scope_foundation.as_str()),
            ("qa_summarize", self.qa_summarize.as_str()),
            ("qa_rewrite", self.qa_rewrite.as_str()),
            ("qa_expand", self.qa_expand.as_str()),
            ("qa_fix_grammar", self.qa_fix_grammar.as_str()),
            ("qa_simplify", self.qa_simplify.as_str()),
            ("qa_add_details", self.qa_add_details.as_str()),
            ("qa_convert_to_table", self.qa_convert_to_table.as_str()),
            ("qa_translate", self.qa_translate.as_str()),
            ("switched_engine", self.switched_engine.as_str()),
            ("conversation_reset", self.conversation_reset.as_str()),
            ("stopped", self.stopped.as_str()),
            ("escalated", self.escalated.as_str()),
            ("truncated", self.truncated.as_str()),
            ("turn_timed_out", self.turn_timed_out.as_str()),
            ("settings_reopened", self.settings_reopened.as_str()),
            ("live_not_available", self.live_not_available.as_str()),
            ("corpus_scope_label", self.corpus_scope_label.as_str()),
            ("scope_none", self.scope_none.as_str()),
            ("query_mode_label", self.query_mode_label.as_str()),
            ("posture_label", self.posture_label.as_str()),
            ("effort_label", self.effort_label.as_str()),
            ("effort_engine_default", self.effort_engine_default.as_str()),
            ("temperature_label", self.temperature_label.as_str()),
            ("codex_web_search", self.codex_web_search.as_str()),
            (
                "codex_suppress_plugins",
                self.codex_suppress_plugins.as_str(),
            ),
            (
                "codex_disable_code_mode",
                self.codex_disable_code_mode.as_str(),
            ),
            ("codex_no_mcp", self.codex_no_mcp.as_str()),
            ("credential_note", self.credential_note.as_str()),
            (
                "reason_engine_process_not_running",
                self.reason_engine_process_not_running.as_str(),
            ),
            (
                "reason_model_not_installed",
                self.reason_model_not_installed.as_str(),
            ),
            ("honesty_ready", self.honesty_ready.as_str()),
            ("canceled_kept_notice", self.canceled_kept_notice.as_str()),
            (
                "canceled_discarded_notice",
                self.canceled_discarded_notice.as_str(),
            ),
            ("limitations_label", self.limitations_label.as_str()),
            ("corpus_label", self.corpus_label.as_str()),
            ("ingest_counts", self.ingest_counts.as_str()),
            ("memory_store_label", self.memory_store_label.as_str()),
            ("memory_offline", self.memory_offline.as_str()),
            ("recall_label", self.recall_label.as_str()),
            ("recall", self.recall.as_str()),
            ("recall_no_hits", self.recall_no_hits.as_str()),
            ("remember_kind_label", self.remember_kind_label.as_str()),
            ("remember_label", self.remember_label.as_str()),
            ("kind_lesson", self.kind_lesson.as_str()),
            ("kind_principle", self.kind_principle.as_str()),
            ("refused_curated_kind", self.refused_curated_kind.as_str()),
            ("refused_unknown", self.refused_unknown.as_str()),
            ("awaiting_curation", self.awaiting_curation.as_str()),
            ("personal_memory_label", self.personal_memory_label.as_str()),
            (
                "office_knowledge_label",
                self.office_knowledge_label.as_str(),
            ),
            ("confirm", self.confirm.as_str()),
            ("withdraw", self.withdraw.as_str()),
            ("knowledge_changed", self.knowledge_changed.as_str()),
            ("grounding_grounded", self.grounding_grounded.as_str()),
            ("grounding_not_found", self.grounding_not_found.as_str()),
            (
                "grounding_assistant_only",
                self.grounding_assistant_only.as_str(),
            ),
            ("facts_label", self.facts_label.as_str()),
            ("as_of_label", self.as_of_label.as_str()),
            (
                "query_attribution_label",
                self.query_attribution_label.as_str(),
            ),
        ]
    }
}
