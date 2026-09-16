//! In-memory [`ChatWorkspaceBackend`] and scripted [`ChatTransport`] for the
//! showcase demo page and the browser proofs, shaped like
//! `crate::patterns::helpdesk::memory`: a deterministic seed, injectable
//! faults, and a call log the fixture exposes.
//!
//! The demo runs as CSR wasm and cannot spawn a CLI or reach a hosted engine,
//! so every provider and knowledge source here is scripted. The fixture ships
//! in the library rather than in the demo crate so a consumer can drive its
//! own tests against the same seed.
//!
//! Two properties are load-bearing and deliberately not conveniences:
//!
//! * **No wall clock and no randomness.** Time comes from [`FixtureClock`],
//!   which only ever advances inside [`ScriptedChatTransport::try_recv`], so
//!   two constructions driven by the same script are byte-identical.
//! * **Written is not findable.** [`InMemoryChatWorkspaceBackend::remember`]
//!   lands an entry in [`MemoryState::Candidate`], mirroring the real memory
//!   service's `proposed` status, and
//!   [`InMemoryChatWorkspaceBackend::recall`] does not return candidates. A
//!   curator (here, `set_memory_state`) closes that gap.
//!
//! Two of this module's names collide with `crate::patterns::helpdesk`'s
//! fixture, which the `patterns` module re-exports by glob, so at the
//! `patterns` level they are aliased: [`BackendCall`] is
//! `patterns::ChatWorkspaceCall` and [`SEED_NOW_MS`] is
//! `patterns::SEED_CHAT_NOW_MS`. Plain `patterns::BackendCall` and
//! `patterns::SEED_NOW_MS` are helpdesk's, and are a different type
//! entirely — reaching for the brief's name gets the wrong type with no
//! error until a match arm fails to compile. The module itself is private,
//! so the aliases are the only path to these two.
//!
//! State is shared between the backend and the transport through one
//! `Arc<Mutex<FixtureCore>>`. The backend's own futures are `!Send` (wasm is
//! single-threaded), but [`ChatTransport`] is `Send`, so the shared core is
//! std-only: `Arc` and `Mutex`, never `Rc` or `RefCell`.

use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use crate::components::ai_assistant_workspace::{
    AnswerOutcome, AssistantAccess, AssistantAnswer, AssistantBudget, AssistantCapabilities,
    AssistantCapability, AssistantConnection, AssistantEngine, AssistantKnowledgeEntry,
    AssistantMemory, AssistantMemoryDraft, AssistantMemoryEntry, AssistantPreferences,
    AssistantProvenance, AssistantReason, AssistantScope, AssistantSettings, AttemptLifecycle,
    ConnectionState, EngineAvailability, KnowledgeReviewState, KnowledgeState, MemoryClass,
    MemoryState, ProvenanceMode, ScopeBasis, SignInShape,
};
use crate::components::ai_chat::{
    AnnotationAnchor, AnnotationBody, AnnotationKind, ChatError, ChatRequest, ChatSettings,
    ChatTransport, Citation, StreamEvent, TranscriptAnnotation, Usage,
};
use ai_chat_core::WakeFn;

use super::backend::{
    ChatWorkspaceBackend, ChatWorkspaceError, ChatWorkspaceErrorKind, WorkspaceFuture,
};
use super::evidence::{GroundingVerdict, TurnEvidence};
use super::knowledge::{
    ChatPosture, CorpusQueryMode, CorpusScope, IngestPhase, IngestStatus, KnowledgeSelection,
    KnowledgeSource, MemoryDraft, MemoryRefusal, OfficeKnowledgeScope, RecallCorpus, RecallHit,
    RecallReceipt, guardrail_refusal,
};
use super::provider::{
    AvailabilityReasonCode, ProviderCard, ProviderTuning, ReasoningEffort,
    desktop_provider_catalogue,
};
use super::status::{TurnNotice, TurnRecord, UsageTotals};
use super::texts::AiChatWorkspaceTexts;

/// The fixed "now" every fixture timestamp derives from, so ages and
/// receipts are deterministic in proofs. Same epoch as
/// `crate::patterns::helpdesk::SEED_NOW_MS`.
pub const SEED_NOW_MS: i64 = 1_800_000_000_000;

/// The memory service's own presentable description of the search it runs,
/// carried verbatim on every [`RecallReceipt`] this fixture mints. It is
/// quoted, never paraphrased: a workspace that reworded it would be claiming
/// a retrieval strategy it cannot observe.
pub const SEED_RECALL_SEARCH: &str = "hybrid: full text + title similarity and Titan v2 cosine similarity, fused by reciprocal rank, curated items boosted, under row-level security";

/// A query that matches at least one seeded item in every one of the four
/// recall corpora, for proofs that assert corpus coverage.
pub const SEED_RECALL_QUERY: &str = "court date reminder tone for client email drafts";

/// The actor the seed belongs to.
pub const SEED_ACTOR: &str = "w-chris";

/// The first seeded corpus folder.
pub const SEED_FOLDER_INTAKE: &str = "kb/intake";

/// The second seeded corpus folder.
pub const SEED_FOLDER_COURT: &str = "kb/court";

const RRF_K: f32 = 60.0;

// ---------------------------------------------------------------------------
// Clock
// ---------------------------------------------------------------------------

/// The fixture's only source of time. It never reads a wall clock: it starts
/// at [`SEED_NOW_MS`] and advances by [`Self::ms_per_poll`] once per call to
/// [`ScriptedChatTransport::try_recv`], so a host watchdog comparing
/// `now_ms()` against a turn's start sees a schedule it fully controls. A
/// script built with [`TurnScript::stall`] never completes, so at the default
/// 100 ms per poll a 120 000 ms watchdog fires on the 1200th poll.
#[derive(Clone, Debug)]
pub struct FixtureClock {
    now_ms: Arc<AtomicI64>,
    /// Milliseconds added to the clock by each `try_recv` call.
    pub ms_per_poll: i64,
}

impl Default for FixtureClock {
    fn default() -> Self {
        Self::new(SEED_NOW_MS, 100)
    }
}

impl FixtureClock {
    /// A clock starting at `start_ms` that advances `ms_per_poll` per poll.
    pub fn new(start_ms: i64, ms_per_poll: i64) -> Self {
        Self {
            now_ms: Arc::new(AtomicI64::new(start_ms)),
            ms_per_poll,
        }
    }

    /// The current fixture time in epoch milliseconds.
    pub fn now_ms(&self) -> i64 {
        self.now_ms.load(Ordering::SeqCst)
    }

    /// Advances the clock by one poll and returns the new time.
    pub fn advance(&self) -> i64 {
        self.now_ms.fetch_add(self.ms_per_poll, Ordering::SeqCst) + self.ms_per_poll
    }

    /// Milliseconds elapsed since [`SEED_NOW_MS`].
    pub fn elapsed_ms(&self) -> i64 {
        self.now_ms() - SEED_NOW_MS
    }
}

/// Formats an epoch-millisecond instant as `YYYY-MM-DDTHH:MM:SSZ` with no
/// date library and no locale, so every seeded `as_of` string is stable.
fn fixture_as_of(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Howard Hinnant's civil-from-days, shifted to a 0000-03-01 era.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

// ---------------------------------------------------------------------------
// Scripts
// ---------------------------------------------------------------------------

/// Which prompts one registered [`TurnScript`] answers.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromptMatcher {
    /// Every prompt sent to the registered engine.
    Any,
    /// Prompts containing this text, compared case-insensitively.
    Contains(String),
    /// Prompts equal to this text after trimming, compared
    /// case-insensitively.
    Exact(String),
}

impl PromptMatcher {
    /// Whether this matcher accepts `prompt`.
    pub fn matches(&self, prompt: &str) -> bool {
        let prompt = prompt.trim().to_lowercase();
        match self {
            Self::Any => true,
            Self::Contains(needle) => prompt.contains(&needle.trim().to_lowercase()),
            Self::Exact(text) => prompt == text.trim().to_lowercase(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ScriptOp {
    Thinking(String),
    TextWords(String),
    ToolCall {
        name: String,
        input: String,
    },
    ToolResult {
        content: String,
        is_error: bool,
    },
    Usage(Usage),
    Evidence(TurnEvidence),
    Escalate(ReasoningEffort, ReasoningEffort),
    Truncated,
    Declined {
        limitations: Vec<String>,
        as_of: String,
    },
    Fail(AvailabilityReasonCode),
    Error(String),
    Stall,
}

/// One scripted turn: the exact sequence of stream events an engine emits,
/// plus the turn metadata (evidence, escalation, truncation, decline,
/// failure) the composite renders beside them.
///
/// Ops run in the order they were added. A script that ends without
/// [`Self::stall`], [`Self::error`] or [`Self::fail`] finishes with a
/// `Done` event, which is what drives the turn's lifecycle to completed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurnScript {
    ops: Vec<ScriptOp>,
}

impl TurnScript {
    /// An empty script. Sent as-is it produces a turn with no text that
    /// still completes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Emits one reasoning block.
    pub fn thinking(mut self, s: &str) -> Self {
        self.ops.push(ScriptOp::Thinking(s.to_owned()));
        self
    }

    /// Emits one `TextDelta` per whitespace-separated word, so a proof sees
    /// real incremental streaming rather than one atomic answer. The
    /// accumulated transcript text is the words rejoined by single spaces.
    pub fn text_words(mut self, s: &str) -> Self {
        self.ops.push(ScriptOp::TextWords(s.to_owned()));
        self
    }

    /// Emits a tool invocation with a raw JSON input string.
    pub fn tool_call(mut self, name: &str, input_json: &str) -> Self {
        self.ops.push(ScriptOp::ToolCall {
            name: name.to_owned(),
            input: input_json.to_owned(),
        });
        self
    }

    /// Emits a tool result, optionally an error one.
    pub fn tool_result(mut self, content: &str, is_error: bool) -> Self {
        self.ops.push(ScriptOp::ToolResult {
            content: content.to_owned(),
            is_error,
        });
        self
    }

    /// Emits a usage report for the turn.
    pub fn usage(mut self, u: Usage) -> Self {
        self.ops.push(ScriptOp::Usage(u));
        self
    }

    /// Attaches evidence to the turn record, overriding whatever the
    /// grounded-posture rule would otherwise have derived.
    pub fn evidence(mut self, e: TurnEvidence) -> Self {
        self.ops.push(ScriptOp::Evidence(e));
        self
    }

    /// Records a reasoning-effort escalation as a
    /// [`AnnotationKind::Warning`] transcript annotation, localized through
    /// `AiChatWorkspaceTexts::escalated`.
    pub fn escalate(mut self, from: ReasoningEffort, to: ReasoningEffort) -> Self {
        self.ops.push(ScriptOp::Escalate(from, to));
        self
    }

    /// Marks the answer truncated, as a [`AnnotationKind::Warning`]
    /// transcript annotation localized through
    /// `AiChatWorkspaceTexts::truncated`.
    pub fn truncated(mut self) -> Self {
        self.ops.push(ScriptOp::Truncated);
        self
    }

    /// Completes the turn as a healthy decline rather than an answer,
    /// carrying its material limitations and evidence freshness.
    pub fn declined(mut self, limitations: Vec<String>, as_of: &str) -> Self {
        self.ops.push(ScriptOp::Declined {
            limitations,
            as_of: as_of.to_owned(),
        });
        self
    }

    /// Ends the turn `Unavailable` with this availability reason, which is
    /// a refused execution rather than a crash.
    pub fn fail(mut self, code: AvailabilityReasonCode) -> Self {
        self.ops.push(ScriptOp::Fail(code));
        self
    }

    /// Ends the turn `Failed` with this message.
    pub fn error(mut self, msg: &str) -> Self {
        self.ops.push(ScriptOp::Error(msg.to_owned()));
        self
    }

    /// Never emits `Done`, so the turn stays `Running` forever and a host
    /// watchdog comparing [`FixtureClock::now_ms`] against the turn's start
    /// is the only thing that can end it.
    pub fn stall(mut self) -> Self {
        self.ops.push(ScriptOp::Stall);
        self
    }
}

// ---------------------------------------------------------------------------
// Faults
// ---------------------------------------------------------------------------

/// An error condition to inject into an [`InMemoryChatWorkspaceBackend`].
/// One fault is active at a time, and it shapes every subsequent call.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatWorkspaceFault {
    /// The actor's reasoning tier denies every effect; the engine list is
    /// still returned so the picker can explain itself.
    TierDisabled,
    /// One engine reports itself unavailable for this reason.
    EngineUnavailable {
        /// The affected engine.
        engine_id: String,
        /// Why it is unavailable.
        code: AvailabilityReasonCode,
    },
    /// One engine has no accepted sign-in.
    NotSignedIn {
        /// The affected engine.
        engine_id: String,
    },
    /// One engine's sign-in has expired.
    SignInExpired {
        /// The affected engine.
        engine_id: String,
    },
    /// One engine has no configured credential key.
    KeyMissing {
        /// The affected engine.
        engine_id: String,
    },
    /// The budget is present with nothing remaining and every session is
    /// refused.
    BudgetExhausted,
    /// Cancelling discards the partial answer instead of keeping it.
    CancelDiscards,
    /// Re-indexing this scope ends in [`IngestPhase::Failed`].
    IngestFails {
        /// The scope that fails to index.
        scope: CorpusScope,
    },
    /// Recall and remember report a network failure.
    MemoryStoreOffline,
    /// `try_recv` releases at most this many events per poll, instead of
    /// the default one.
    SlowStream {
        /// Events released per poll.
        events_per_poll: u32,
    },
}

// ---------------------------------------------------------------------------
// Call log
// ---------------------------------------------------------------------------

/// One call the fixture recorded, for proofs to assert on. Transport-side
/// calls ([`Self::Cancel`], [`Self::Restart`]) share the same ordered log as
/// the backend's own, which is why the log lives in the shared core.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendCall {
    /// A call to [`ChatWorkspaceBackend::settings`].
    Settings,
    /// A call to [`ChatWorkspaceBackend::providers`].
    Providers,
    /// A call to [`ChatWorkspaceBackend::knowledge`].
    Knowledge,
    /// A call to [`ChatWorkspaceBackend::reindex`].
    Reindex(CorpusScope),
    /// A call to [`ChatWorkspaceBackend::open_session`].
    OpenSession {
        /// The requested engine.
        engine_id: String,
        /// The requested grounding posture.
        posture: ChatPosture,
        /// The requested corpus query mode.
        query_mode: CorpusQueryMode,
    },
    /// A call to [`ChatWorkspaceBackend::set_tuning`], by engine id.
    SetTuning(String),
    /// A call to [`ChatWorkspaceBackend::turn`], by turn id.
    Turn(String),
    /// A call to [`ChatWorkspaceBackend::recall`], by query.
    Recall(String),
    /// A call to [`ChatWorkspaceBackend::remember`], by proposed class.
    Remember(MemoryClass),
    /// A call to [`ChatWorkspaceBackend::set_memory_flags`], carrying the
    /// requested use and capture flags in that order.
    SetMemoryFlags(bool, bool),
    /// A call to [`ChatWorkspaceBackend::set_memory_state`].
    SetMemoryState(String, MemoryState),
    /// A call to [`ChatWorkspaceBackend::sign_in`], by engine id.
    SignIn(String),
    /// A call to [`ChatWorkspaceBackend::sign_out`], by engine id.
    SignOut(String),
    /// A call to `ChatTransport::cancel`.
    Cancel {
        /// The cancelled turn.
        turn_id: String,
        /// Whether the partial answer was discarded.
        discarded: bool,
    },
    /// A call to `ChatTransport::restart`.
    Restart,
}

// ---------------------------------------------------------------------------
// Seed data
// ---------------------------------------------------------------------------

struct SeedDoc {
    path: &'static str,
    folder: &'static str,
    title: &'static str,
    body: &'static str,
}

fn seed_docs() -> Vec<SeedDoc> {
    vec![
        SeedDoc {
            path: "kb/intake/intake-checklist.md",
            folder: SEED_FOLDER_INTAKE,
            title: "Intake checklist",
            body: "Confirm identity and conflict check before opening a matter. Record the referral source. Collect the signed engagement letter. Open the billing record last.",
        },
        SeedDoc {
            path: "kb/intake/fee-agreement.md",
            folder: SEED_FOLDER_INTAKE,
            title: "Fee agreement template",
            body: "The flat fee covers the initial hearing only. Additional hearings are billed hourly. The agreement is void until both signatures are recorded.",
        },
        SeedDoc {
            path: "kb/intake/client-communication.md",
            folder: SEED_FOLDER_INTAKE,
            title: "Client communication guideline",
            body: "Answer every client email within one business day. Keep the tone plain and warm. Never promise an outcome. Summarize next steps at the end of each message.",
        },
        SeedDoc {
            path: "kb/court/court-date-continuance.md",
            folder: SEED_FOLDER_COURT,
            title: "Court date memo: continuance",
            body: "A continuance request must reach the clerk seven days before the scheduled court date. Attach the proposed order. Notify the client the same day.",
        },
        SeedDoc {
            path: "kb/court/court-date-remote.md",
            folder: SEED_FOLDER_COURT,
            title: "Court date memo: remote appearance",
            body: "Remote appearance is granted by standing order for status hearings. Test the video link the morning of the court date. Keep a phone number on file as the fallback.",
        },
        SeedDoc {
            path: "kb/court/payment-plan.md",
            folder: SEED_FOLDER_COURT,
            title: "Payment plan note",
            body: "A payment plan requires a signed addendum and a first installment before the next hearing. Missed installments pause work rather than closing the matter.",
        },
    ]
}

/// How many seeded documents a scope covers, so the ingest counts a consumer
/// renders are derived from the seed rather than repeated beside it.
fn docs_in(scope: &CorpusScope) -> u32 {
    seed_docs()
        .iter()
        .filter(|d| match scope {
            CorpusScope::All => true,
            CorpusScope::Folder(path) => d.folder == path,
            CorpusScope::File(path) => d.path == path,
            CorpusScope::Custom { paths, .. } => paths.iter().any(|p| p == d.path),
        })
        .count() as u32
}

/// Lowercased words of four or more alphanumeric characters. The floor drops
/// articles and prepositions without maintaining a stopword list, which keeps
/// the grounded-posture rule readable and stable.
fn significant_words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4)
        .map(|w| w.to_lowercase())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Documents whose title-plus-body shares at least two significant words
/// with the prompt. Two is the smallest overlap that is not a coincidence of
/// one shared noun, and it is deliberately a rule a proof can reproduce by
/// hand rather than a similarity score.
fn grounded_hits(prompt: &str) -> Vec<SeedDoc> {
    let asked = significant_words(prompt);
    seed_docs()
        .into_iter()
        .filter(|d| {
            let doc = significant_words(&format!("{} {}", d.title, d.body));
            asked.iter().filter(|w| doc.contains(w)).count() >= 2
        })
        .collect()
}

struct SeedRecallItem {
    title: &'static str,
    snippet: &'static str,
    corpus: RecallCorpus,
    keywords: &'static [&'static str],
}

fn seed_recall_items() -> Vec<SeedRecallItem> {
    vec![
        SeedRecallItem {
            title: "Reminder emails go out two days ahead",
            snippet: "The office sends a court date reminder two days ahead, never the morning of.",
            corpus: RecallCorpus::Words,
            keywords: &["court", "date", "reminder"],
        },
        SeedRecallItem {
            title: "Plain English in client email",
            snippet: "Client email avoids Latin terms and explains each next step in plain English.",
            corpus: RecallCorpus::Words,
            keywords: &["client", "email", "plain"],
        },
        SeedRecallItem {
            title: "Warm but never reassuring about outcomes",
            snippet: "Tone stays warm; an outcome is never predicted, however likely it looks.",
            corpus: RecallCorpus::Meaning,
            keywords: &["tone", "warm", "email", "drafts"],
        },
        SeedRecallItem {
            title: "Short paragraphs read better on a phone",
            snippet: "Drafts use short paragraphs because most clients read on a phone.",
            corpus: RecallCorpus::Meaning,
            keywords: &["drafts", "paragraphs", "phone"],
        },
        SeedRecallItem {
            title: "Reminder relates to the continuance memo",
            snippet: "The reminder practice is derived from the continuance memo's seven-day rule.",
            corpus: RecallCorpus::Graph,
            keywords: &["reminder", "continuance", "court"],
        },
        SeedRecallItem {
            title: "Fee agreement supersedes the 2024 template",
            snippet: "The current fee agreement supersedes the 2024 template it was derived from.",
            corpus: RecallCorpus::Graph,
            keywords: &["agreement", "template", "supersedes"],
        },
        SeedRecallItem {
            title: "Earlier in this conversation: reminder wording",
            snippet: "You asked for the reminder wording to name the courtroom, not the judge.",
            corpus: RecallCorpus::Thread,
            keywords: &["reminder", "wording", "court"],
        },
        SeedRecallItem {
            title: "Earlier in this conversation: draft length",
            snippet: "You asked to keep email drafts under one screen.",
            corpus: RecallCorpus::Thread,
            keywords: &["email", "drafts", "length"],
        },
    ]
}

/// A per-corpus multiplier applied to the reciprocal-rank score so two hits
/// holding the same rank in different corpora still order strictly, and a
/// proof can assert a total order rather than a merely non-increasing one.
fn corpus_weight(corpus: &RecallCorpus) -> f32 {
    match corpus {
        RecallCorpus::Words => 1.0,
        RecallCorpus::Meaning => 0.97,
        RecallCorpus::Graph => 0.94,
        RecallCorpus::Thread => 0.91,
        RecallCorpus::Unknown(_) => 0.88,
    }
}

fn memory_grants() -> AssistantCapabilities {
    AssistantCapabilities {
        granted: vec![
            AssistantCapability::MemoryCreate,
            AssistantCapability::MemoryConfirm,
            AssistantCapability::MemoryCorrect,
            AssistantCapability::MemoryForget,
        ],
        details: vec![],
    }
}

fn settings_grants() -> AssistantCapabilities {
    AssistantCapabilities {
        granted: vec![
            AssistantCapability::Settings,
            AssistantCapability::SaveSettings,
            AssistantCapability::SignIn,
            AssistantCapability::SignOut,
            AssistantCapability::MemoryCreate,
            AssistantCapability::MemoryConfirm,
            AssistantCapability::MemoryForget,
        ],
        details: vec![],
    }
}

fn memory_entry(
    id: &str,
    class: MemoryClass,
    text: &str,
    state: MemoryState,
) -> AssistantMemoryEntry {
    AssistantMemoryEntry {
        id: id.to_owned(),
        revision: 1,
        class,
        text: text.to_owned(),
        state,
        capabilities: memory_grants(),
        retention: None,
    }
}

fn seed_memory() -> AssistantMemory {
    AssistantMemory {
        owner_actor_id: SEED_ACTOR.to_owned(),
        revision: 1,
        use_enabled: true,
        capture_enabled: true,
        entries: vec![
            memory_entry(
                "mem-0001",
                MemoryClass::Language,
                "Write to this client in Spanish unless they write in English first.",
                MemoryState::Confirmed,
            ),
            memory_entry(
                "mem-0002",
                MemoryClass::Tone,
                "Keep reminder emails warm and short.",
                MemoryState::Candidate,
            ),
            memory_entry(
                "mem-0003",
                MemoryClass::WorkingMethod,
                "Draft the court date reminder before the status call, not after.",
                MemoryState::Withdrawn,
            ),
        ],
        draft: AssistantMemoryDraft {
            revision: 1,
            class: MemoryClass::Tone,
            text: String::new(),
            max_chars: 280,
        },
        capabilities: memory_grants(),
    }
}

fn knowledge_entry(
    id: &str,
    title: &str,
    question: &str,
    audience: &str,
    body: &str,
) -> AssistantKnowledgeEntry {
    AssistantKnowledgeEntry {
        id: id.to_owned(),
        revision: 2,
        title: title.to_owned(),
        canonical_question: question.to_owned(),
        aliases_en: vec![],
        aliases_es: vec![],
        audience: audience.to_owned(),
        state: KnowledgeState::Published,
        proposed_body: None,
        published_body: Some(body.to_owned()),
        sources: vec![],
        lineage: vec![],
        comparison: None,
        review: KnowledgeReviewState::NotReviewed,
        capabilities: AssistantCapabilities {
            granted: vec![AssistantCapability::Evidence],
            details: vec![],
        },
    }
}

fn seed_office_knowledge(scope: &OfficeKnowledgeScope) -> Vec<AssistantKnowledgeEntry> {
    match scope {
        OfficeKnowledgeScope::Foundation => vec![
            knowledge_entry(
                "kn-f-001",
                "How a matter is opened",
                "What has to happen before a matter is opened?",
                "Everyone",
                "Conflict check, engagement letter, then the billing record.",
            ),
            knowledge_entry(
                "kn-f-002",
                "Who may speak to the court",
                "Who is allowed to contact the clerk?",
                "Everyone",
                "Only the attorney of record or a named paralegal contacts the clerk.",
            ),
        ],
        OfficeKnowledgeScope::GroupImportant => vec![
            knowledge_entry(
                "kn-g-001",
                "Reminder email wording",
                "What wording do we use for a court date reminder?",
                "Client-facing staff",
                "Name the courtroom and the time; never the judge.",
            ),
            knowledge_entry(
                "kn-g-002",
                "When a payment plan pauses work",
                "What happens when an installment is missed?",
                "Billing",
                "Work pauses; the matter is not closed.",
            ),
        ],
        // A scope this seed does not know is empty, not silently the
        // group-important set.
        _ => vec![],
    }
}

fn scope_key(scope: &CorpusScope) -> String {
    match scope {
        CorpusScope::File(p) | CorpusScope::Folder(p) => p.clone(),
        CorpusScope::All => "*".to_owned(),
        CorpusScope::Custom { label, .. } => format!("custom:{label}"),
    }
}

fn ready_ingest(files: u32, clusters: u32, as_of: &str) -> IngestStatus {
    IngestStatus {
        phase: IngestPhase::Ready,
        files_seen: files,
        files_indexed: files,
        clusters,
        as_of: as_of.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Core
// ---------------------------------------------------------------------------

struct TurnState {
    id: String,
    engine_id: String,
    queue: VecDeque<StreamEvent>,
    stage: TurnStage,
    partial: String,
    started_ms: i64,
    usage: Option<Usage>,
    evidence: Option<TurnEvidence>,
    declined: Option<(Vec<String>, String)>,
    fail_code: Option<AvailabilityReasonCode>,
    terminal: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TurnStage {
    Admitted,
    Queued,
    Streaming,
}

struct FixtureCore {
    clock: FixtureClock,
    fault: Option<ChatWorkspaceFault>,
    scripts: Vec<(String, PromptMatcher, TurnScript)>,
    calls: Vec<BackendCall>,
    texts: AiChatWorkspaceTexts,
    engine_id: String,
    selection: KnowledgeSelection,
    chat_settings: ChatSettings,
    /// The settings last handed to [`ChatTransport::configure`], which is
    /// a different event from the ones `open_session` was opened with.
    configured_settings: Option<ChatSettings>,
    /// The tuning last applied for each engine id, keyed per engine. One
    /// slot would answer "what was applied last" but not "what is codex's
    /// tuning now that groq's temperature was changed", which is the
    /// question a tuning panel has to get right.
    tuning: BTreeMap<String, ProviderTuning>,
    connections: BTreeMap<String, AssistantConnection>,
    ingest: BTreeMap<String, IngestStatus>,
    memory: AssistantMemory,
    /// The recall sequence number in force when each entry last became
    /// eligible for recall (written, then confirmed), so the first recall in
    /// which an entry can appear at all withholds its `Meaning` hit exactly
    /// the way the real store does while its vector is still being computed.
    written_at_seq: BTreeMap<String, u32>,
    turn: Option<TurnState>,
    records: BTreeMap<String, TurnRecord>,
    wake: Option<WakeFn>,
    released_in_poll: u32,
    next_turn: u32,
    next_memory: u32,
    recall_seq: u32,
}

impl FixtureCore {
    fn seeded() -> Self {
        let clock = FixtureClock::default();
        let as_of = fixture_as_of(clock.now_ms());
        let mut ingest = BTreeMap::new();
        for scope in [
            CorpusScope::Folder(SEED_FOLDER_INTAKE.to_owned()),
            CorpusScope::Folder(SEED_FOLDER_COURT.to_owned()),
            CorpusScope::All,
        ] {
            let files = docs_in(&scope);
            ingest.insert(
                scope_key(&scope),
                ready_ingest(files, files.div_ceil(2), &as_of),
            );
        }
        Self {
            clock,
            fault: None,
            scripts: Vec::new(),
            calls: Vec::new(),
            texts: AiChatWorkspaceTexts::default(),
            engine_id: "claude-code".to_owned(),
            selection: KnowledgeSelection::default(),
            chat_settings: ChatSettings::default(),
            configured_settings: None,
            tuning: BTreeMap::new(),
            connections: BTreeMap::new(),
            ingest,
            memory: seed_memory(),
            written_at_seq: BTreeMap::new(),
            turn: None,
            records: BTreeMap::new(),
            wake: None,
            released_in_poll: 0,
            next_turn: 1,
            next_memory: 4,
            recall_seq: 0,
        }
    }

    fn now(&self) -> i64 {
        self.clock.now_ms()
    }

    fn as_of(&self) -> String {
        fixture_as_of(self.now())
    }

    fn events_per_poll(&self) -> u32 {
        match &self.fault {
            Some(ChatWorkspaceFault::SlowStream { events_per_poll }) => (*events_per_poll).max(1),
            _ => 1,
        }
    }

    fn engine_fault_code(&self, engine_id: &str) -> Option<AvailabilityReasonCode> {
        match &self.fault {
            Some(ChatWorkspaceFault::EngineUnavailable {
                engine_id: id,
                code,
            }) if id == engine_id => Some(code.clone()),
            Some(ChatWorkspaceFault::NotSignedIn { engine_id: id }) if id == engine_id => {
                Some(AvailabilityReasonCode::NotSignedIn)
            }
            Some(ChatWorkspaceFault::SignInExpired { engine_id: id }) if id == engine_id => {
                Some(AvailabilityReasonCode::SignInExpired)
            }
            Some(ChatWorkspaceFault::KeyMissing { engine_id: id }) if id == engine_id => {
                Some(AvailabilityReasonCode::CredentialKeyUnavailable)
            }
            Some(ChatWorkspaceFault::BudgetExhausted) => {
                Some(AvailabilityReasonCode::BudgetExhausted)
            }
            _ => None,
        }
    }

    fn engines(&self) -> Vec<AssistantEngine> {
        desktop_provider_catalogue()
            .into_iter()
            .map(|card| self.apply_engine_fault(card.engine))
            .collect()
    }

    fn apply_engine_fault(&self, mut engine: AssistantEngine) -> AssistantEngine {
        if let Some(connection) = self.connections.get(&engine.id) {
            engine.connection = connection.clone();
        }
        if let Some(code) = self.engine_fault_code(&engine.id) {
            let text = fault_message(&code);
            engine.availability = EngineAvailability::Disabled {
                reason_code: code.as_code().to_owned(),
                text: text.clone(),
            };
            let reason = AssistantReason {
                code: code.as_code().to_owned(),
                message: text,
            };
            match code {
                AvailabilityReasonCode::NotSignedIn => {
                    engine.connection.state = ConnectionState::NotSignedIn;
                    engine.connection.shape = SignInShape::Paste;
                }
                AvailabilityReasonCode::SignInExpired => {
                    engine.connection.state = ConnectionState::Expired;
                    engine.connection.shape = SignInShape::Paste;
                    engine.connection.reason = Some(reason);
                }
                AvailabilityReasonCode::CredentialKeyUnavailable => {
                    engine.connection.state = ConnectionState::NotSignedIn;
                    engine.connection.shape = SignInShape::Paste;
                }
                _ => {}
            }
        }
        engine
    }

    fn log(&mut self, call: BackendCall) {
        self.calls.push(call);
    }

    fn default_script(&self, engine_id: &str) -> TurnScript {
        let model = self
            .chat_settings
            .model
            .clone()
            .unwrap_or_else(|| "gpt-5.6".to_owned());
        match engine_id {
            "claude-code" => TurnScript::new()
                .thinking("Reading the intake checklist before answering.")
                .tool_call("Read", "{\"path\":\"kb/intake/intake-checklist.md\"}")
                .tool_result(
                    "Confirm identity and conflict check before opening a matter.",
                    false,
                )
                .text_words(
                    "The checklist opens with the conflict check, then the engagement letter.",
                )
                .usage(Usage {
                    cost_usd: 0.0143,
                    input_tokens: 812,
                    output_tokens: 96,
                    reasoning_tokens: 41,
                    cache_read_tokens: 6_144,
                    cache_creation_tokens: 512,
                }),
            "codex-cli" => TurnScript::new()
                .text_words(&format!(
                    "Codex CLI answered with {model} and changed nothing on disk."
                ))
                .usage(Usage {
                    cost_usd: 0.0,
                    input_tokens: 240,
                    output_tokens: 44,
                    reasoning_tokens: 12,
                    ..Usage::default()
                }),
            "codex-spark" => TurnScript::new()
                .text_words("Spark replied from gpt-5.3-codex-spark.")
                .usage(Usage {
                    cost_usd: 0.0,
                    input_tokens: 96,
                    output_tokens: 12,
                    ..Usage::default()
                }),
            "groq-gpt-oss-120b" => TurnScript::new()
                .escalate(ReasoningEffort::Medium, ReasoningEffort::Low)
                .text_words("Groq answered at low effort and ran out of room before the last step")
                .truncated()
                .usage(Usage {
                    cost_usd: 0.0007,
                    input_tokens: 310,
                    output_tokens: 128,
                    reasoning_tokens: 64,
                    ..Usage::default()
                }),
            _ => TurnScript::new()
                .text_words(&format!(
                    "Ollama answered locally with {}.",
                    self.chat_settings
                        .model
                        .clone()
                        .unwrap_or_else(|| "llama3.1:8b".to_owned())
                ))
                .usage(Usage {
                    cost_usd: 0.0,
                    input_tokens: 128,
                    output_tokens: 22,
                    reasoning_tokens: 0,
                    ..Usage::default()
                }),
        }
    }

    fn resolve_script(&self, engine_id: &str, prompt: &str) -> TurnScript {
        self.scripts
            .iter()
            .find(|(id, matcher, _)| id == engine_id && matcher.matches(prompt))
            .map(|(_, _, script)| script.clone())
            .unwrap_or_else(|| self.default_script(engine_id))
    }
}

fn fault_message(code: &AvailabilityReasonCode) -> String {
    match code {
        AvailabilityReasonCode::NotSignedIn => "Sign in to use this engine.".to_owned(),
        AvailabilityReasonCode::SignInExpired => "This sign-in has expired.".to_owned(),
        AvailabilityReasonCode::CredentialKeyUnavailable => {
            "No key is configured for this engine.".to_owned()
        }
        AvailabilityReasonCode::BudgetExhausted => "This budget has been used up.".to_owned(),
        other => format!("This engine is unavailable ({}).", other.as_code()),
    }
}

fn workspace_error(kind: ChatWorkspaceErrorKind, message: impl Into<String>) -> ChatWorkspaceError {
    ChatWorkspaceError {
        kind,
        message: message.into(),
    }
}

fn fixture_scope() -> AssistantScope {
    AssistantScope {
        id: "fixture-scope".to_owned(),
        revision: 1,
        label: "Seeded office knowledge".to_owned(),
        basis: ScopeBasis::PagePopulation {
            population_id: "fixture-population".to_owned(),
            population_revision: 1,
            description: "The six seeded documents.".to_owned(),
        },
    }
}

fn fixture_provenance(engine_id: &str, model: Option<String>, as_of: &str) -> AssistantProvenance {
    AssistantProvenance {
        mode: ProvenanceMode::Generated,
        actual_engine: Some(engine_id.to_owned()),
        model_version: model,
        prompt_version: None,
        skill_version: None,
        memory_revisions: vec![],
        foundation_revisions: vec![],
        supplied_sources: vec![],
        omissions: vec![],
        built_at: as_of.to_owned(),
    }
}

/// Renders one [`TurnNotice`] as the transcript annotation a composite would
/// interleave, localized through the fixture's texts. The notice is the
/// record; this is only its wording.
fn notice_annotation(notice: &TurnNotice, texts: &AiChatWorkspaceTexts) -> TranscriptAnnotation {
    let body = match notice {
        TurnNotice::Escalated { from, to } => texts
            .escalated
            .replace("{from}", from.as_str())
            .replace("{to}", to.as_str()),
        TurnNotice::Truncated => texts.truncated.clone(),
        TurnNotice::Unknown(code) => code.clone(),
    };
    TranscriptAnnotation {
        anchor: AnnotationAnchor::AtEnd,
        kind: AnnotationKind::Warning,
        body: AnnotationBody::Text(body),
    }
}

// ---------------------------------------------------------------------------
// Transport
// ---------------------------------------------------------------------------

/// The scripted [`ChatTransport`] an [`InMemoryChatWorkspaceBackend`] hands
/// out from `open_session`. It shares the backend's core, so a proof can send
/// a turn through a real `ChatSession` and then read the same turn's record
/// and call log back off the backend.
pub struct ScriptedChatTransport {
    core: Arc<Mutex<FixtureCore>>,
}

impl ScriptedChatTransport {
    fn new(core: Arc<Mutex<FixtureCore>>) -> Self {
        Self { core }
    }

    fn build_queue(script: &TurnScript, turn: &mut TurnState) -> bool {
        let mut stalled = false;
        for op in &script.ops {
            match op {
                ScriptOp::Thinking(s) => turn.queue.push_back(StreamEvent::Thinking(s.clone())),
                ScriptOp::TextWords(s) => {
                    for (i, word) in s.split_whitespace().enumerate() {
                        let chunk = if i == 0 {
                            word.to_owned()
                        } else {
                            format!(" {word}")
                        };
                        turn.queue.push_back(StreamEvent::TextDelta(chunk));
                    }
                }
                ScriptOp::ToolCall { name, input } => turn.queue.push_back(StreamEvent::ToolCall {
                    name: name.clone(),
                    input: input.clone(),
                }),
                ScriptOp::ToolResult { content, is_error } => {
                    turn.queue.push_back(StreamEvent::ToolResult {
                        content: content.clone(),
                        is_error: *is_error,
                    })
                }
                ScriptOp::Usage(u) => turn.queue.push_back(StreamEvent::Usage(*u)),
                ScriptOp::Evidence(e) => turn.evidence = Some(e.clone()),
                ScriptOp::Escalate(_, _) | ScriptOp::Truncated => {}
                ScriptOp::Declined { limitations, as_of } => {
                    turn.declined = Some((limitations.clone(), as_of.clone()));
                }
                ScriptOp::Fail(code) => {
                    turn.fail_code = Some(code.clone());
                    turn.queue
                        .push_back(StreamEvent::Error(fault_message(code)));
                }
                ScriptOp::Error(msg) => turn.queue.push_back(StreamEvent::Error(msg.clone())),
                ScriptOp::Stall => stalled = true,
            }
        }
        stalled
    }

    /// The turn notices a script mints, in script order. These are the
    /// data — [`InMemoryChatWorkspaceBackend::annotations`] is the
    /// presentation derived from them.
    fn notices_for(script: &TurnScript) -> Vec<TurnNotice> {
        script
            .ops
            .iter()
            .filter_map(|op| match op {
                ScriptOp::Escalate(from, to) => Some(TurnNotice::Escalated {
                    from: from.clone(),
                    to: to.clone(),
                }),
                ScriptOp::Truncated => Some(TurnNotice::Truncated),
                _ => None,
            })
            .collect()
    }

    fn base_record(id: &str, engine_id: &str, notices: Vec<TurnNotice>) -> TurnRecord {
        TurnRecord {
            id: id.to_owned(),
            engine_id: engine_id.to_owned(),
            lifecycle: AttemptLifecycle::Admitted,
            usage: None,
            tokens_per_sec: None,
            evidence: None,
            outcome: None,
            notices,
        }
    }

    fn set_lifecycle(core: &mut FixtureCore, id: &str, lifecycle: AttemptLifecycle) {
        if let Some(record) = core.records.get_mut(id) {
            record.lifecycle = lifecycle;
        }
    }

    fn complete(core: &mut FixtureCore) {
        let Some(turn) = core.turn.as_mut() else {
            return;
        };
        turn.terminal = true;
        let as_of = fixture_as_of(core.clock.now_ms());
        let outcome = match turn.declined.clone() {
            Some((limitations, declined_as_of)) => AnswerOutcome::Declined {
                limitations,
                as_of: declined_as_of,
            },
            None => AnswerOutcome::Answered {
                text: turn.partial.clone(),
            },
        };
        let limitations = match &outcome {
            AnswerOutcome::Declined { limitations, .. } => limitations.clone(),
            _ => vec![],
        };
        let mut totals = UsageTotals::default();
        if let Some(u) = turn.usage {
            if u.cost_usd > 0.0 {
                totals.add_metered(&u, Some(u.cost_usd));
            } else {
                totals.add_plan(&u);
            }
        }
        let tokens_per_sec = turn.usage.and_then(|u| {
            UsageTotals::tokens_per_sec(u.output_tokens, core.clock.now_ms() - turn.started_ms)
        });
        let answer = AssistantAnswer {
            id: turn.id.clone(),
            revision: 1,
            answered_scope: fixture_scope(),
            as_of: as_of.clone(),
            outcome: outcome.clone(),
            facts: vec![],
            evidence: vec![],
            provenance: fixture_provenance(
                &turn.engine_id,
                core.chat_settings.model.clone(),
                &as_of,
            ),
            limitations,
        };
        let id = turn.id.clone();
        let evidence = turn.evidence.clone();
        if let Some(record) = core.records.get_mut(&id) {
            record.lifecycle = AttemptLifecycle::Completed(answer);
            record.outcome = Some(outcome);
            record.usage = Some(totals);
            record.tokens_per_sec = tokens_per_sec;
            record.evidence = evidence;
        }
    }
}

impl ChatTransport for ScriptedChatTransport {
    fn send(&mut self, req: ChatRequest) -> Result<(), ChatError> {
        let mut core = self.core.lock().expect("fixture core poisoned");
        let engine_id = core.engine_id.clone();
        let mut script = core.resolve_script(&engine_id, &req.prompt);

        // Grounded posture is a property of the SESSION, not of the script:
        // a grounded turn that matches no seeded document answers with the
        // localized not-found sentence and nothing else, whatever the script
        // would have said.
        let hits = grounded_hits(&req.prompt);
        let grounding = match core.selection.posture {
            ChatPosture::Grounded if hits.is_empty() => {
                script = TurnScript::new().text_words(&core.texts.grounded_not_found);
                GroundingVerdict::NotFound
            }
            ChatPosture::Grounded => GroundingVerdict::Grounded {
                sources: hits.len() as u32,
            },
            ChatPosture::Assistant => GroundingVerdict::AssistantOnly,
        };

        let id = format!("turn-{:04}", core.next_turn);
        core.next_turn += 1;
        let mut turn = TurnState {
            id: id.clone(),
            engine_id: engine_id.clone(),
            queue: VecDeque::new(),
            stage: TurnStage::Admitted,
            partial: String::new(),
            started_ms: core.clock.now_ms(),
            usage: None,
            evidence: None,
            declined: None,
            fail_code: None,
            terminal: false,
        };
        let stalled = Self::build_queue(&script, &mut turn);
        // An error event is terminal, so a script that ends in one never also
        // gets a `Done` that would complete the turn it just failed.
        let errored = matches!(turn.queue.back(), Some(StreamEvent::Error(_)));
        if !stalled && !errored {
            turn.queue.push_back(StreamEvent::Done);
        }
        if turn.evidence.is_none() {
            turn.evidence = Some(TurnEvidence {
                citations: hits
                    .iter()
                    .map(|d| Citation {
                        label: d.title.to_owned(),
                        href: Some(d.path.to_owned()),
                    })
                    .collect(),
                recall: None,
                facts: vec![],
                limitations: vec![],
                as_of: Some(fixture_as_of(core.clock.now_ms())),
                grounding,
            });
        }
        let notices = Self::notices_for(&script);
        core.records
            .insert(id.clone(), Self::base_record(&id, &engine_id, notices));
        core.turn = Some(turn);
        core.released_in_poll = 0;
        if let Some(wake) = core.wake.clone() {
            drop(core);
            wake();
        }
        Ok(())
    }

    fn try_recv(&mut self) -> Option<StreamEvent> {
        let mut core = self.core.lock().expect("fixture core poisoned");
        core.clock.advance();
        if core.released_in_poll >= core.events_per_poll() {
            core.released_in_poll = 0;
            return None;
        }
        let stage = core.turn.as_ref().map(|t| t.stage)?;
        let id = core.turn.as_ref().map(|t| t.id.clone())?;
        match stage {
            TurnStage::Admitted => {
                if let Some(turn) = core.turn.as_mut() {
                    turn.stage = TurnStage::Queued;
                }
                Self::set_lifecycle(&mut core, &id, AttemptLifecycle::Queued);
                core.released_in_poll = 0;
                None
            }
            TurnStage::Queued => {
                if let Some(turn) = core.turn.as_mut() {
                    turn.stage = TurnStage::Streaming;
                }
                Self::set_lifecycle(&mut core, &id, AttemptLifecycle::Running);
                core.released_in_poll = 0;
                None
            }
            TurnStage::Streaming => {
                let Some(event) = core.turn.as_mut().and_then(|t| t.queue.pop_front()) else {
                    core.released_in_poll = 0;
                    return None;
                };
                core.released_in_poll += 1;
                if let Some(turn) = core.turn.as_mut() {
                    match &event {
                        StreamEvent::TextDelta(s) => turn.partial.push_str(s),
                        StreamEvent::Usage(u) => turn.usage = Some(*u),
                        _ => {}
                    }
                }
                let fail_code = core.turn.as_ref().and_then(|t| t.fail_code.clone());
                let next_is_done = core
                    .turn
                    .as_ref()
                    .map(|t| t.queue.len() == 1 && t.queue.front() == Some(&StreamEvent::Done))
                    .unwrap_or(false);
                match &event {
                    StreamEvent::Done => Self::complete(&mut core),
                    StreamEvent::Error(message) => {
                        if let Some(turn) = core.turn.as_mut() {
                            turn.terminal = true;
                            turn.queue.clear();
                        }
                        let reason = AssistantReason {
                            code: fail_code
                                .as_ref()
                                .map(|c| c.as_code().to_owned())
                                .unwrap_or_else(|| "engine_error".to_owned()),
                            message: message.clone(),
                        };
                        let lifecycle = if fail_code.is_some() {
                            AttemptLifecycle::Unavailable { reason }
                        } else {
                            AttemptLifecycle::Failed { reason }
                        };
                        Self::set_lifecycle(&mut core, &id, lifecycle);
                    }
                    _ if next_is_done => {
                        Self::set_lifecycle(&mut core, &id, AttemptLifecycle::Validating)
                    }
                    _ => Self::set_lifecycle(&mut core, &id, AttemptLifecycle::Running),
                }
                Some(event)
            }
        }
    }

    fn restart(&mut self) -> Result<(), ChatError> {
        let mut core = self.core.lock().expect("fixture core poisoned");
        core.log(BackendCall::Restart);
        core.turn = None;
        core.released_in_poll = 0;
        Ok(())
    }

    fn cancel(&mut self) -> Result<(), ChatError> {
        let mut core = self.core.lock().expect("fixture core poisoned");
        let discarded = core.fault == Some(ChatWorkspaceFault::CancelDiscards);
        let Some(turn) = core.turn.as_mut() else {
            return Ok(());
        };
        let id = turn.id.clone();
        turn.queue.clear();
        turn.terminal = true;
        if discarded {
            turn.partial.clear();
        }
        // A cancelled turn never reaches `complete`, so the streamed prefix
        // would otherwise leave no trace anywhere a consumer can read: the
        // lifecycle's `Canceled` carries only a boolean. Write the partial
        // onto the record's outcome, AFTER the discard, so "kept" and
        // "discarded" are two observably different records rather than the
        // same record with a different flag.
        let partial = turn.partial.clone();
        if let Some(record) = core.records.get_mut(&id) {
            record.outcome = Some(AnswerOutcome::Answered { text: partial });
        }
        core.log(BackendCall::Cancel {
            turn_id: id.clone(),
            discarded,
        });
        Self::set_lifecycle(&mut core, &id, AttemptLifecycle::Canceled { discarded });
        core.released_in_poll = 0;
        Ok(())
    }

    fn configure(&mut self, settings: ChatSettings) {
        let mut core = self.core.lock().expect("fixture core poisoned");
        core.chat_settings = settings.clone();
        core.configured_settings = Some(settings);
    }

    fn set_wake(&mut self, wake: WakeFn) {
        let mut core = self.core.lock().expect("fixture core poisoned");
        core.wake = Some(wake);
    }
}

// ---------------------------------------------------------------------------
// Backend
// ---------------------------------------------------------------------------

/// A [`ChatWorkspaceBackend`] backed by seeded, in-process state. Powers the
/// showcase demo page and the browser proofs, and ships only under
/// `test-mode`.
///
/// Clone it freely: every clone shares one core, so a proof can keep a handle
/// for [`Self::calls`] while the composite owns another.
#[derive(Clone)]
pub struct InMemoryChatWorkspaceBackend {
    core: Arc<Mutex<FixtureCore>>,
}

impl Default for InMemoryChatWorkspaceBackend {
    fn default() -> Self {
        Self::seeded()
    }
}

impl InMemoryChatWorkspaceBackend {
    /// Five engines, six documents across two folders, four recall corpora,
    /// office knowledge in both scopes and three personal-memory entries.
    pub fn seeded() -> Self {
        Self {
            core: Arc::new(Mutex::new(FixtureCore::seeded())),
        }
    }

    /// Inject a fault: every subsequent call is shaped by it.
    pub fn with_fault(self, fault: ChatWorkspaceFault) -> Self {
        self.lock().fault = Some(fault);
        self
    }

    /// Register a script for one engine. The first registered script whose
    /// engine and matcher both accept a prompt wins; with none matching, the
    /// engine's own default script runs.
    pub fn with_script(self, engine_id: &str, matcher: PromptMatcher, script: TurnScript) -> Self {
        self.lock()
            .scripts
            .push((engine_id.to_owned(), matcher, script));
        self
    }

    /// Replace the fixture clock, for a proof that needs a different poll
    /// step than the default 100 ms.
    pub fn with_clock(self, clock: FixtureClock) -> Self {
        self.lock().clock = clock;
        self
    }

    /// Every call made so far, in order, including the transport's own
    /// `cancel` and `restart`.
    pub fn calls(&self) -> Vec<BackendCall> {
        self.lock().calls.clone()
    }

    /// The transcript annotations for one turn, rendered from that turn's
    /// `TurnRecord::notices` through this fixture's texts. Purely a
    /// convenience: the notices themselves are on the record, which is what
    /// a composite holding a `dyn ChatWorkspaceBackend` can actually reach.
    pub fn annotations(&self, turn_id: &str) -> Vec<TranscriptAnnotation> {
        let core = self.lock();
        let Some(record) = core.records.get(turn_id) else {
            return vec![];
        };
        record
            .notices
            .iter()
            .map(|notice| notice_annotation(notice, &core.texts))
            .collect()
    }

    /// The [`ProviderTuning`] most recently applied for one engine — by
    /// `set_tuning`, or by the `open_session` that selected it — and `None`
    /// for an engine neither tuned nor opened. Per engine, not one slot: a
    /// proof has to be able to show that tuning groq left codex's own
    /// tuning alone.
    pub fn applied_tuning(&self, engine_id: &str) -> Option<ProviderTuning> {
        self.lock().tuning.get(engine_id).cloned()
    }

    /// The [`ChatSettings`] most recently handed to
    /// [`ChatTransport::configure`], and `None` before the first one. This
    /// is the payload that actually crossed the transport seam, which is a
    /// stronger claim than "a configure call happened".
    pub fn applied_chat_settings(&self) -> Option<ChatSettings> {
        self.lock().configured_settings.clone()
    }

    /// The current fixture time in epoch milliseconds, for a host watchdog.
    pub fn now_ms(&self) -> i64 {
        self.lock().clock.now_ms()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, FixtureCore> {
        self.core.lock().expect("fixture core poisoned")
    }

    fn ready<T: 'static>(v: Result<T, ChatWorkspaceError>) -> WorkspaceFuture<T> {
        Box::pin(std::future::ready(v))
    }

    fn corpus_source(&self, core: &FixtureCore, scope: CorpusScope) -> KnowledgeSource {
        let key = scope_key(&scope);
        let ingest = core
            .ingest
            .get(&key)
            .cloned()
            .unwrap_or_else(|| ready_ingest(0, 0, &core.as_of()));
        KnowledgeSource::Corpus {
            scope,
            ingest,
            query_mode: core.selection.query_mode,
            posture: core.selection.posture,
        }
    }

    /// Advances one mid-ingest scope by exactly one phase. Called from
    /// `knowledge()`, so the phase ladder is driven by the consumer's own
    /// polling rather than by elapsed time.
    fn advance_ingest(core: &mut FixtureCore) {
        let failing = match &core.fault {
            Some(ChatWorkspaceFault::IngestFails { scope }) => Some(scope_key(scope)),
            _ => None,
        };
        let as_of = core.as_of();
        for (key, status) in core.ingest.iter_mut() {
            let next = match status.phase {
                IngestPhase::Walking => IngestPhase::Indexing,
                IngestPhase::Indexing => IngestPhase::Clustering,
                IngestPhase::Clustering => {
                    if failing.as_deref() == Some(key.as_str()) {
                        IngestPhase::Failed(AssistantReason {
                            code: "ingest_failed".to_owned(),
                            message: "Indexing stopped on an unreadable file.".to_owned(),
                        })
                    } else {
                        IngestPhase::Ready
                    }
                }
                _ => continue,
            };
            status.files_indexed = match next {
                IngestPhase::Indexing => status.files_seen / 2,
                _ => status.files_seen,
            };
            status.clusters = match next {
                IngestPhase::Clustering | IngestPhase::Ready => status.files_seen.div_ceil(2),
                _ => 0,
            };
            status.phase = next;
            status.as_of = as_of;
            return;
        }
    }
}

impl ChatWorkspaceBackend for InMemoryChatWorkspaceBackend {
    fn settings(&self) -> WorkspaceFuture<AssistantSettings> {
        let mut core = self.lock();
        core.log(BackendCall::Settings);
        let as_of = core.as_of();
        let tier = if core.fault == Some(ChatWorkspaceFault::TierDisabled) {
            AssistantAccess::Denied {
                reason: AssistantReason {
                    code: AvailabilityReasonCode::TierEffectsDisabled
                        .as_code()
                        .to_owned(),
                    message: "This plan does not include assistant effects.".to_owned(),
                },
            }
        } else {
            AssistantAccess::Granted
        };
        let budget = if core.fault == Some(ChatWorkspaceFault::BudgetExhausted) {
            Some(AssistantBudget {
                policy_label: "Monthly assistant budget".to_owned(),
                limit_display: "$25.00".to_owned(),
                remaining_display: Some("$0.00".to_owned()),
                as_of: as_of.clone(),
            })
        } else {
            None
        };
        let preferences = AssistantPreferences {
            engine_id: Some(core.engine_id.clone()),
            memory_use: core.memory.use_enabled,
            memory_capture: core.memory.capture_enabled,
        };
        let settings = AssistantSettings {
            owner_actor_id: SEED_ACTOR.to_owned(),
            accepted_revision: 1,
            proposed_revision: 1,
            accepted: preferences.clone(),
            proposed: preferences,
            reasoning_tier: tier,
            engines: core.engines(),
            budget,
            capabilities: settings_grants(),
        };
        Self::ready(Ok(settings))
    }

    fn providers(&self) -> WorkspaceFuture<Vec<ProviderCard>> {
        let mut core = self.lock();
        core.log(BackendCall::Providers);
        let cards = desktop_provider_catalogue()
            .into_iter()
            .map(|mut card| {
                card.engine = core.apply_engine_fault(card.engine);
                card
            })
            .collect();
        Self::ready(Ok(cards))
    }

    fn knowledge(&self) -> WorkspaceFuture<Vec<KnowledgeSource>> {
        let mut core = self.lock();
        core.log(BackendCall::Knowledge);
        Self::advance_ingest(&mut core);
        let mut sources = vec![
            self.corpus_source(&core, CorpusScope::Folder(SEED_FOLDER_INTAKE.to_owned())),
            self.corpus_source(&core, CorpusScope::Folder(SEED_FOLDER_COURT.to_owned())),
            self.corpus_source(&core, CorpusScope::All),
        ];
        sources.push(KnowledgeSource::MemoryStore {
            enabled: core.memory.use_enabled,
            capture_enabled: core.memory.capture_enabled,
        });
        for scope in [
            OfficeKnowledgeScope::GroupImportant,
            OfficeKnowledgeScope::Foundation,
        ] {
            let entries = seed_office_knowledge(&scope);
            sources.push(KnowledgeSource::OfficeKnowledge { scope, entries });
        }
        sources.push(KnowledgeSource::PersonalMemory(core.memory.clone()));
        Self::ready(Ok(sources))
    }

    fn reindex(&self, scope: &CorpusScope) -> WorkspaceFuture<IngestStatus> {
        let mut core = self.lock();
        core.log(BackendCall::Reindex(scope.clone()));
        let as_of = core.as_of();
        let key = scope_key(scope);
        let files_seen = docs_in(scope);
        let status = IngestStatus {
            phase: IngestPhase::Walking,
            files_seen,
            files_indexed: 0,
            clusters: 0,
            as_of,
        };
        core.ingest.insert(key, status.clone());
        Self::ready(Ok(status))
    }

    fn open_session(
        &self,
        engine_id: &str,
        knowledge: &KnowledgeSelection,
        settings: ChatSettings,
        tuning: ProviderTuning,
    ) -> WorkspaceFuture<Box<dyn ChatTransport>> {
        let mut core = self.lock();
        core.log(BackendCall::OpenSession {
            engine_id: engine_id.to_owned(),
            posture: knowledge.posture,
            query_mode: knowledge.query_mode,
        });
        if let Some(code) = core.engine_fault_code(engine_id) {
            return Self::ready(Err(workspace_error(
                ChatWorkspaceErrorKind::Unavailable,
                fault_message(&code),
            )));
        }
        core.engine_id = engine_id.to_owned();
        core.selection = knowledge.clone();
        core.chat_settings = settings;
        core.tuning.insert(engine_id.to_owned(), tuning);
        let transport: Box<dyn ChatTransport> =
            Box::new(ScriptedChatTransport::new(Arc::clone(&self.core)));
        Self::ready(Ok(transport))
    }

    fn set_tuning(&self, engine_id: &str, tuning: ProviderTuning) -> WorkspaceFuture<()> {
        let mut core = self.lock();
        core.log(BackendCall::SetTuning(engine_id.to_owned()));
        core.tuning.insert(engine_id.to_owned(), tuning);
        Self::ready(Ok(()))
    }

    fn turn(&self, turn_id: &str) -> WorkspaceFuture<TurnRecord> {
        let mut core = self.lock();
        core.log(BackendCall::Turn(turn_id.to_owned()));
        let record = core.records.get(turn_id).cloned().ok_or_else(|| {
            workspace_error(
                ChatWorkspaceErrorKind::NotFound,
                format!("No turn {turn_id}"),
            )
        });
        Self::ready(record)
    }

    fn current_turn_id(&self) -> Option<String> {
        let core = self.lock();
        core.turn
            .as_ref()
            .filter(|t| !t.terminal)
            .map(|t| t.id.clone())
    }

    fn recall(&self, query: &str) -> WorkspaceFuture<RecallReceipt> {
        let mut core = self.lock();
        core.log(BackendCall::Recall(query.to_owned()));
        if core.fault == Some(ChatWorkspaceFault::MemoryStoreOffline) {
            return Self::ready(Err(workspace_error(
                ChatWorkspaceErrorKind::Network,
                "The memory store did not answer.",
            )));
        }
        // One counter, not two: the receipt number and the recall sequence
        // the meaning-vector delay is keyed to are the same count, and a
        // second counter is only an opportunity to bump one of them.
        core.recall_seq += 1;
        let seq = core.recall_seq;
        let receipt = RecallReceipt {
            receipt_id: format!("rcpt-{seq:04}"),
            as_of: core.as_of(),
            since: None,
            hits: if core.memory.use_enabled {
                recall_hits(&core, query)
            } else {
                vec![]
            },
            search: SEED_RECALL_SEARCH.to_owned(),
        };
        Self::ready(Ok(receipt))
    }

    fn remember(
        &self,
        draft: MemoryDraft,
    ) -> WorkspaceFuture<Result<AssistantMemoryEntry, MemoryRefusal>> {
        let mut core = self.lock();
        core.log(BackendCall::Remember(draft.kind.clone()));
        if core.fault == Some(ChatWorkspaceFault::MemoryStoreOffline) {
            return Self::ready(Err(workspace_error(
                ChatWorkspaceErrorKind::Network,
                "The memory store did not answer.",
            )));
        }
        // A curated class is refused outright, before the text is examined:
        // lessons and principles are distilled by a curator out of accepted
        // items, never written directly.
        if is_curated_kind(&draft.kind) {
            return Self::ready(Ok(Err(MemoryRefusal::CuratedKindOnly)));
        }
        if let Some(refusal) = guardrail_refusal(&draft.body) {
            return Self::ready(Ok(Err(refusal)));
        }
        let id = format!("mem-{:04}", core.next_memory);
        core.next_memory += 1;
        let entry = memory_entry(&id, draft.kind, &draft.body, MemoryState::Candidate);
        let seq = core.recall_seq;
        core.written_at_seq.insert(id, seq);
        core.memory.entries.push(entry.clone());
        core.memory.revision += 1;
        Self::ready(Ok(Ok(entry)))
    }

    fn set_memory_flags(
        &self,
        use_enabled: bool,
        capture_enabled: bool,
    ) -> WorkspaceFuture<AssistantMemory> {
        let mut core = self.lock();
        core.log(BackendCall::SetMemoryFlags(use_enabled, capture_enabled));
        core.memory.use_enabled = use_enabled;
        core.memory.capture_enabled = capture_enabled;
        core.memory.revision += 1;
        let memory = core.memory.clone();
        Self::ready(Ok(memory))
    }

    fn set_memory_state(
        &self,
        id: &str,
        state: MemoryState,
    ) -> WorkspaceFuture<AssistantMemoryEntry> {
        let mut core = self.lock();
        core.log(BackendCall::SetMemoryState(id.to_owned(), state.clone()));
        let found = core.memory.entries.iter_mut().find(|e| e.id == id);
        let result = match found {
            Some(entry) => {
                entry.state = state;
                entry.revision += 1;
                Ok(entry.clone())
            }
            None => Err(workspace_error(
                ChatWorkspaceErrorKind::NotFound,
                format!("No memory entry {id}"),
            )),
        };
        // Confirmation is the moment an entry becomes eligible for recall, so
        // it restarts the meaning-vector delay.
        if let Ok(entry) = &result {
            core.memory.revision += 1;
            if entry.state == MemoryState::Confirmed {
                let seq = core.recall_seq;
                core.written_at_seq.insert(entry.id.clone(), seq);
            }
        }
        Self::ready(result)
    }

    fn sign_in(&self, engine_id: &str) -> WorkspaceFuture<AssistantConnection> {
        let mut core = self.lock();
        core.log(BackendCall::SignIn(engine_id.to_owned()));
        let as_of = core.as_of();
        let connection = AssistantConnection {
            state: ConnectionState::SignedIn,
            shape: SignInShape::Paste,
            device: None,
            account_label: Some("chris@example.test".to_owned()),
            verified_at: Some(as_of),
            expires_at: None,
            reason: None,
        };
        core.connections
            .insert(engine_id.to_owned(), connection.clone());
        Self::ready(Ok(connection))
    }

    fn sign_out(&self, engine_id: &str) -> WorkspaceFuture<AssistantConnection> {
        let mut core = self.lock();
        core.log(BackendCall::SignOut(engine_id.to_owned()));
        let connection = AssistantConnection {
            state: ConnectionState::NotSignedIn,
            shape: SignInShape::Paste,
            device: None,
            account_label: None,
            verified_at: None,
            expires_at: None,
            reason: None,
        };
        core.connections
            .insert(engine_id.to_owned(), connection.clone());
        Self::ready(Ok(connection))
    }
}

/// Whether this class is one the memory service only ever creates by
/// curation. The vocabulary has no named variant for either, so they arrive
/// as host strings.
fn is_curated_kind(kind: &MemoryClass) -> bool {
    matches!(kind, MemoryClass::Unknown(s) if s.eq_ignore_ascii_case("lesson") || s.eq_ignore_ascii_case("principle"))
}

/// Builds one recall's hits: the seeded corpus items whose keywords the query
/// mentions, plus the actor's CONFIRMED memory entries. Candidates are
/// deliberately absent — the real service's recall returns curated items
/// only, so a write is not findable until a curator approves it.
fn recall_hits(core: &FixtureCore, query: &str) -> Vec<RecallHit> {
    let asked: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();
    let mentions = |words: &[&str]| words.iter().any(|k| asked.iter().any(|a| a == k));

    let mut per_corpus: BTreeMap<String, u32> = BTreeMap::new();
    let mut hits: Vec<RecallHit> = Vec::new();
    let mut push =
        |corpus: RecallCorpus, title: String, snippet: String, per: &mut BTreeMap<String, u32>| {
            let key = format!("{corpus:?}");
            let rank = per.entry(key).and_modify(|r| *r += 1).or_insert(1);
            hits.push(RecallHit {
                rrf_score: corpus_weight(&corpus) / (RRF_K + *rank as f32),
                rank_in_corpus: *rank,
                corpus,
                title,
                snippet,
            });
        };

    for item in seed_recall_items() {
        if mentions(item.keywords) {
            push(
                item.corpus,
                item.title.to_owned(),
                item.snippet.to_owned(),
                &mut per_corpus,
            );
        }
    }

    for entry in &core.memory.entries {
        if entry.state != MemoryState::Confirmed {
            continue;
        }
        let words = significant_words(&entry.text);
        if !words.iter().any(|w| asked.iter().any(|a| a == w)) {
            continue;
        }
        push(
            RecallCorpus::Words,
            entry.text.clone(),
            entry.text.clone(),
            &mut per_corpus,
        );
        // The meaning vector lands a moment after the item does, so the
        // first recall in which an entry is eligible at all reports a words
        // match and never a meaning one.
        let fresh = core
            .written_at_seq
            .get(&entry.id)
            .is_some_and(|seq| *seq + 1 >= core.recall_seq);
        if !fresh {
            push(
                RecallCorpus::Meaning,
                entry.text.clone(),
                entry.text.clone(),
                &mut per_corpus,
            );
        }
    }

    hits.sort_by(|a, b| {
        b.rrf_score
            .partial_cmp(&a.rrf_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.title.cmp(&b.title))
    });
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ai_chat::{ChatRole, ChatSession};
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    fn now<T>(mut fut: WorkspaceFuture<T>) -> Result<T, ChatWorkspaceError> {
        let waker = std::task::Waker::noop();
        let mut cx = Context::from_waker(waker);
        match Pin::new(&mut fut).poll(&mut cx) {
            Poll::Ready(v) => v,
            Poll::Pending => panic!("fixture futures are always ready"),
        }
    }

    fn session(backend: &InMemoryChatWorkspaceBackend, engine: &str) -> ChatSession {
        let selection = KnowledgeSelection {
            posture: ChatPosture::Assistant,
            ..KnowledgeSelection::default()
        };
        let transport = now(backend.open_session(
            engine,
            &selection,
            ChatSettings {
                show_thinking: true,
                show_tool_calls: true,
                ..ChatSettings::default()
            },
            ProviderTuning::default(),
        ))
        .expect("session opens");
        let mut s = ChatSession::new(transport);
        s.configure(ChatSettings {
            show_thinking: true,
            show_tool_calls: true,
            ..ChatSettings::default()
        });
        s
    }

    /// Polls until the session stops changing or `max` polls elapse.
    fn drain(s: &mut ChatSession, max: usize) -> usize {
        let mut polls = 0;
        while polls < max {
            polls += 1;
            let changed = s.poll();
            if !changed && !s.is_waiting() {
                break;
            }
        }
        polls
    }

    fn ask(s: &mut ChatSession, prompt: &str) {
        s.send(ChatRequest {
            prompt: prompt.to_owned(),
            ..ChatRequest::default()
        })
        .expect("send");
    }

    #[test]
    fn every_script_op_streams_through_a_real_chat_session() {
        let script = TurnScript::new()
            .thinking("weighing the options")
            .tool_call("Read", "{\"path\":\"a.md\"}")
            .tool_result("file body", false)
            .text_words("one two three")
            .usage(Usage {
                cost_usd: 0.5,
                output_tokens: 3,
                ..Usage::default()
            })
            .evidence(TurnEvidence {
                citations: vec![],
                recall: None,
                facts: vec![],
                limitations: vec![],
                as_of: None,
                grounding: GroundingVerdict::AssistantOnly,
            })
            .escalate(ReasoningEffort::Medium, ReasoningEffort::Low)
            .truncated();
        let backend = InMemoryChatWorkspaceBackend::seeded().with_script(
            "claude-code",
            PromptMatcher::Contains("script".into()),
            script,
        );
        // Pin the public path a consumer reaches the fixture by.
        let _: crate::patterns::InMemoryChatWorkspaceBackend = backend.clone();
        let mut s = session(&backend, "claude-code");
        ask(&mut s, "run the script please");
        let turn_id = backend.current_turn_id().expect("a turn is live");
        drain(&mut s, 60);

        let events = s.drain_events();
        assert!(events.contains(&StreamEvent::Thinking("weighing the options".into())));
        assert!(events.contains(&StreamEvent::ToolCall {
            name: "Read".into(),
            input: "{\"path\":\"a.md\"}".into()
        }));
        assert!(events.contains(&StreamEvent::ToolResult {
            content: "file body".into(),
            is_error: false
        }));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, StreamEvent::TextDelta(_)))
                .count(),
            3,
            "one delta per word"
        );
        assert!(events.iter().any(|e| matches!(e, StreamEvent::Usage(_))));
        assert_eq!(events.last(), Some(&StreamEvent::Done));

        let roles: Vec<ChatRole> = s.messages().iter().map(|m| m.role).collect();
        assert_eq!(
            roles,
            vec![
                ChatRole::User,
                ChatRole::Thinking,
                ChatRole::Tool,
                ChatRole::Tool,
                ChatRole::Assistant
            ]
        );
        assert_eq!(s.messages()[4].content, "one two three");

        let record = now(backend.turn(&turn_id)).expect("record");
        assert!(matches!(record.lifecycle, AttemptLifecycle::Completed(_)));
        assert_eq!(
            record.evidence.expect("evidence").grounding,
            GroundingVerdict::AssistantOnly
        );
        let totals = record.usage.expect("usage");
        assert_eq!(totals.metered_cost, Some(0.5));
        assert_eq!(
            backend.annotations(&turn_id).len(),
            2,
            "escalate + truncated"
        );
    }

    #[test]
    fn terminal_script_ops_end_the_turn_their_own_way() {
        let backend = InMemoryChatWorkspaceBackend::seeded()
            .with_script(
                "claude-code",
                PromptMatcher::Exact("decline".into()),
                TurnScript::new()
                    .text_words("Not from this folder.")
                    .declined(vec!["No fee schedule is indexed.".into()], "2027-01-15"),
            )
            .with_script(
                "claude-code",
                PromptMatcher::Exact("fail".into()),
                TurnScript::new().fail(AvailabilityReasonCode::EngineBusy),
            )
            .with_script(
                "claude-code",
                PromptMatcher::Exact("error".into()),
                TurnScript::new().error("the engine hung up"),
            );
        let mut s = session(&backend, "claude-code");

        ask(&mut s, "decline");
        let id = backend.current_turn_id().expect("turn");
        drain(&mut s, 60);
        let record = now(backend.turn(&id)).expect("record");
        assert!(matches!(record.lifecycle, AttemptLifecycle::Completed(_)));
        assert_eq!(
            record.outcome,
            Some(AnswerOutcome::Declined {
                limitations: vec!["No fee schedule is indexed.".into()],
                as_of: "2027-01-15".into(),
            })
        );

        ask(&mut s, "fail");
        let id = backend.current_turn_id().expect("turn");
        drain(&mut s, 60);
        assert!(matches!(
            now(backend.turn(&id)).expect("record").lifecycle,
            AttemptLifecycle::Unavailable { reason } if reason.code == "engine_busy"
        ));

        ask(&mut s, "error");
        let id = backend.current_turn_id().expect("turn");
        drain(&mut s, 60);
        assert!(matches!(
            now(backend.turn(&id)).expect("record").lifecycle,
            AttemptLifecycle::Failed { reason } if reason.message == "the engine hung up"
        ));
        assert_eq!(backend.current_turn_id(), None, "a failed turn is not live");
    }

    #[test]
    fn the_wake_callback_fires_when_a_turn_is_queued() {
        let backend = InMemoryChatWorkspaceBackend::seeded();
        let mut transport = now(backend.open_session(
            "codex-spark",
            &KnowledgeSelection {
                posture: ChatPosture::Assistant,
                ..KnowledgeSelection::default()
            },
            ChatSettings::default(),
            ProviderTuning::default(),
        ))
        .expect("session");
        let woken = Arc::new(AtomicI64::new(0));
        let counter = Arc::clone(&woken);
        transport.set_wake(Arc::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
        transport
            .send(ChatRequest {
                prompt: "hello".into(),
                ..ChatRequest::default()
            })
            .expect("send");
        assert_eq!(woken.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn each_fault_yields_its_lifecycle_or_availability() {
        // TierDisabled: the tier denies, the engine list still arrives.
        let b = InMemoryChatWorkspaceBackend::seeded().with_fault(ChatWorkspaceFault::TierDisabled);
        let s = now(b.settings()).expect("settings");
        assert!(matches!(s.reasoning_tier, AssistantAccess::Denied { .. }));
        assert_eq!(s.engines.len(), 5);

        // EngineUnavailable: that engine is disabled and refuses a session.
        let b = InMemoryChatWorkspaceBackend::seeded().with_fault(
            ChatWorkspaceFault::EngineUnavailable {
                engine_id: "codex-cli".into(),
                code: AvailabilityReasonCode::CliMissing,
            },
        );
        let engines = now(b.settings()).expect("settings").engines;
        let codex = engines.iter().find(|e| e.id == "codex-cli").expect("codex");
        assert!(matches!(
            &codex.availability,
            EngineAvailability::Disabled { reason_code, .. } if reason_code == "cli_missing"
        ));
        let err = now(b.open_session(
            "codex-cli",
            &KnowledgeSelection::default(),
            ChatSettings::default(),
            ProviderTuning::default(),
        ))
        .err()
        .expect("refused");
        assert_eq!(err.kind, ChatWorkspaceErrorKind::Unavailable);

        // NotSignedIn / SignInExpired / KeyMissing carry their own codes.
        for (fault, code) in [
            (
                ChatWorkspaceFault::NotSignedIn {
                    engine_id: "groq-gpt-oss-120b".into(),
                },
                "not_signed_in",
            ),
            (
                ChatWorkspaceFault::SignInExpired {
                    engine_id: "groq-gpt-oss-120b".into(),
                },
                "sign_in_expired",
            ),
            (
                ChatWorkspaceFault::KeyMissing {
                    engine_id: "groq-gpt-oss-120b".into(),
                },
                "credential_key_unavailable",
            ),
        ] {
            let b = InMemoryChatWorkspaceBackend::seeded().with_fault(fault);
            let engines = now(b.settings()).expect("settings").engines;
            let groq = engines
                .iter()
                .find(|e| e.id == "groq-gpt-oss-120b")
                .expect("groq");
            assert!(matches!(
                &groq.availability,
                EngineAvailability::Disabled { reason_code, .. } if reason_code == code
            ));
        }

        // BudgetExhausted: a budget with nothing left, every session refused.
        let b =
            InMemoryChatWorkspaceBackend::seeded().with_fault(ChatWorkspaceFault::BudgetExhausted);
        let settings = now(b.settings()).expect("settings");
        let budget = settings.budget.clone().expect("budget");
        assert_eq!(budget.remaining_display.as_deref(), Some("$0.00"));
        // Deliberate, and beyond the brief's letter: an exhausted budget is
        // not one engine's problem, so every engine reads disabled rather
        // than enabled-but-refusing.
        assert!(
            settings.engines.iter().all(|e| matches!(
                &e.availability,
                EngineAvailability::Disabled { reason_code, .. } if reason_code == "budget_exhausted"
            )),
            "an exhausted budget disables all five engines"
        );
        assert!(
            now(b.open_session(
                "claude-code",
                &KnowledgeSelection::default(),
                ChatSettings::default(),
                ProviderTuning::default(),
            ))
            .is_err()
        );

        // IngestFails: the ladder ends Failed rather than Ready.
        let scope = CorpusScope::Folder(SEED_FOLDER_COURT.into());
        let b =
            InMemoryChatWorkspaceBackend::seeded().with_fault(ChatWorkspaceFault::IngestFails {
                scope: scope.clone(),
            });
        assert_eq!(
            now(b.reindex(&scope)).expect("reindex").phase,
            IngestPhase::Walking
        );
        let mut phase = IngestPhase::Walking;
        for _ in 0..3 {
            let sources = now(b.knowledge()).expect("knowledge");
            phase = sources
                .iter()
                .find_map(|s| match s {
                    KnowledgeSource::Corpus {
                        scope: sc, ingest, ..
                    } if sc == &scope => Some(ingest.phase.clone()),
                    _ => None,
                })
                .expect("court corpus");
        }
        assert!(matches!(phase, IngestPhase::Failed(_)));

        // MemoryStoreOffline: recall and remember are network failures.
        let b = InMemoryChatWorkspaceBackend::seeded()
            .with_fault(ChatWorkspaceFault::MemoryStoreOffline);
        assert_eq!(
            now(b.recall("anything")).expect_err("offline").kind,
            ChatWorkspaceErrorKind::Network
        );
        assert_eq!(
            now(b.remember(MemoryDraft {
                kind: MemoryClass::Tone,
                title: "t".into(),
                body: "warm".into(),
                scope: "office".into(),
            }))
            .expect_err("offline")
            .kind,
            ChatWorkspaceErrorKind::Network
        );

        // SlowStream: a poll releases EXACTLY the configured event count. The
        // unfaulted run on the identical script is the control — an upper
        // bound alone is satisfied by the default, so it would pass against a
        // fixture that ignored the fault entirely.
        const SEVEN_EVENTS: &str = "one two three four five six";
        fn events_per_poll_sequence(b: &InMemoryChatWorkspaceBackend, polls: usize) -> Vec<usize> {
            let mut s = session(b, "codex-spark");
            ask(&mut s, "hello");
            (0..polls)
                .map(|_| {
                    s.poll();
                    s.drain_events().len()
                })
                .collect()
        }
        let b = InMemoryChatWorkspaceBackend::seeded()
            .with_fault(ChatWorkspaceFault::SlowStream { events_per_poll: 3 })
            .with_script(
                "codex-spark",
                PromptMatcher::Any,
                TurnScript::new().text_words(SEVEN_EVENTS),
            );
        assert_eq!(
            events_per_poll_sequence(&b, 5),
            vec![0, 0, 3, 3, 1],
            "two ladder polls, then six deltas and a Done three at a time"
        );
        let b = InMemoryChatWorkspaceBackend::seeded().with_script(
            "codex-spark",
            PromptMatcher::Any,
            TurnScript::new().text_words(SEVEN_EVENTS),
        );
        assert_eq!(
            events_per_poll_sequence(&b, 5),
            vec![0, 0, 1, 1, 1],
            "the unfaulted control releases exactly one per poll"
        );

        // CancelDiscards: the discard reaches the log. Which text survives is
        // proven by `cancel_discards_when_faulted_and_keeps_partial_otherwise`.
        let b =
            InMemoryChatWorkspaceBackend::seeded().with_fault(ChatWorkspaceFault::CancelDiscards);
        let mut s = session(&b, "claude-code");
        ask(&mut s, "hello");
        for _ in 0..4 {
            s.poll();
        }
        s.cancel().expect("cancel");
        assert!(
            b.calls().iter().any(|c| matches!(
                c,
                BackendCall::Cancel {
                    discarded: true,
                    ..
                }
            )),
            "the fault makes the cancel a discarding one"
        );
    }

    #[test]
    fn restart_clears_the_live_turn_and_leaves_the_fixture_usable() {
        let b = InMemoryChatWorkspaceBackend::seeded().with_script(
            "claude-code",
            PromptMatcher::Any,
            TurnScript::new().text_words("one two three four five"),
        );
        let mut s = session(&b, "claude-code");
        ask(&mut s, "start over in a moment");
        let first = b.current_turn_id().expect("a turn is live");
        for _ in 0..4 {
            s.poll();
        }
        now(b.turn(&first)).expect("record");

        s.restart().expect("restart");
        assert!(!s.poll(), "nothing is left to drain after a restart");
        assert_eq!(b.current_turn_id(), None, "no turn is live after a restart");
        let calls = b.calls();
        assert_eq!(
            calls.last(),
            Some(&BackendCall::Restart),
            "the transport's restart shares the backend's ordered log"
        );
        let turn_at = calls
            .iter()
            .position(|c| matches!(c, BackendCall::Turn(id) if id == &first))
            .expect("the turn lookup was logged");
        assert_eq!(
            turn_at,
            calls.len() - 2,
            "Restart lands immediately after the Turn lookup"
        );

        // And the fixture still works: a second turn gets a fresh id and
        // completes.
        ask(&mut s, "and again");
        let second = b.current_turn_id().expect("a second turn is live");
        assert_ne!(second, first, "a restart does not reuse the turn id");
        drain(&mut s, 40);
        assert!(matches!(
            now(b.turn(&second)).expect("record").lifecycle,
            AttemptLifecycle::Completed(_)
        ));
    }

    #[test]
    fn applied_tuning_is_per_engine_and_configure_is_observable() {
        let b = InMemoryChatWorkspaceBackend::seeded();
        assert_eq!(
            b.applied_chat_settings(),
            None,
            "nothing was configured yet"
        );
        assert_eq!(
            b.applied_tuning("ollama"),
            None,
            "never tuned, never opened"
        );

        let groq = ProviderTuning {
            groq: Some(crate::patterns::GroqTuning { temperature: 0.2 }),
            ..ProviderTuning::default()
        };
        now(b.set_tuning("groq-gpt-oss-120b", groq.clone())).expect("tuned");
        let codex = ProviderTuning {
            codex: Some(crate::patterns::CodexLevers {
                web_search: true,
                ..crate::patterns::CodexLevers::default()
            }),
            ..ProviderTuning::default()
        };
        now(b.set_tuning("codex-cli", codex.clone())).expect("tuned");

        // Per engine, not one slot: tuning groq did not give codex a
        // temperature, and the later call did not overwrite the earlier one.
        assert_eq!(b.applied_tuning("groq-gpt-oss-120b"), Some(groq));
        assert_eq!(b.applied_tuning("codex-cli"), Some(codex));
        assert!(
            b.applied_tuning("codex-cli")
                .expect("codex tuning")
                .groq
                .is_none(),
            "codex carries no groq sampling"
        );
        assert_eq!(b.applied_tuning("ollama"), None);

        // `configure` is the seam the composite's toggles cross, and the
        // payload is readable rather than merely counted.
        let mut s = session(&b, "claude-code");
        s.configure(ChatSettings {
            show_thinking: false,
            show_tool_calls: true,
            model: Some("claude-sonnet-4-6".into()),
            ..ChatSettings::default()
        });
        let applied = b.applied_chat_settings().expect("configured");
        assert!(!applied.show_thinking);
        assert!(applied.show_tool_calls);
        assert_eq!(applied.model.as_deref(), Some("claude-sonnet-4-6"));
    }

    #[test]
    fn two_seeded_backends_are_identical() {
        fn run() -> String {
            let b = InMemoryChatWorkspaceBackend::seeded();
            let mut s = session(&b, "claude-code");
            ask(&mut s, "what does the intake checklist say");
            let id = b.current_turn_id().expect("turn");
            drain(&mut s, 80);
            let record = now(b.turn(&id)).expect("record");
            let receipt = now(b.recall(SEED_RECALL_QUERY)).expect("receipt");
            format!("{:?}", (b.calls(), s.messages(), record, receipt))
        }
        assert_eq!(run(), run());
    }

    #[test]
    fn cancel_discards_when_faulted_and_keeps_partial_otherwise() {
        const ANSWER: &str = "the fee agreement template lives in the intake folder";
        for discards in [false, true] {
            let b = InMemoryChatWorkspaceBackend::seeded().with_script(
                "claude-code",
                PromptMatcher::Any,
                TurnScript::new().text_words(ANSWER),
            );
            let b = if discards {
                b.with_fault(ChatWorkspaceFault::CancelDiscards)
            } else {
                b
            };
            let mut s = session(&b, "claude-code");
            ask(&mut s, "tell me about the fee agreement");
            let id = b.current_turn_id().expect("turn");
            // Two ladder polls, then one word per poll: four words streamed.
            for _ in 0..6 {
                s.poll();
            }
            s.cancel().expect("cancel");
            let logged = b
                .calls()
                .into_iter()
                .find_map(|c| match c {
                    BackendCall::Cancel { turn_id, discarded } => Some((turn_id, discarded)),
                    _ => None,
                })
                .expect("cancel was logged");
            assert_eq!(logged, (id.clone(), discards));
            let record = now(b.turn(&id)).expect("record");
            assert_eq!(
                record.lifecycle,
                AttemptLifecycle::Canceled {
                    discarded: discards
                }
            );
            // The partial TEXT, not just the flag that claims to govern it.
            // The two legs are each other's negative control, so neither can
            // pass while `cancel` ignores the fault.
            let Some(AnswerOutcome::Answered { text, .. }) = record.outcome else {
                panic!("a cancelled turn carries the partial it kept, if any");
            };
            if discards {
                assert_eq!(text, "", "a discarding cancel keeps nothing");
            } else {
                assert_eq!(
                    text, "the fee agreement template",
                    "four words streamed in six polls"
                );
                assert!(ANSWER.starts_with(&text), "and they are the script's own");
            }
        }
    }

    #[test]
    fn guardrail_refusals_and_accepted_prose_reach_the_log() {
        let b = InMemoryChatWorkspaceBackend::seeded();
        let draft = |kind: MemoryClass, body: &str| MemoryDraft {
            kind,
            title: "note".into(),
            body: body.into(),
            scope: "office".into(),
        };

        let refused = [
            (
                draft(MemoryClass::WorkingMethod, "Always cite matter 24-88421."),
                MemoryRefusal::ContainsMatterNumber,
            ),
            (
                draft(MemoryClass::Tone, "Copy chris@example.test on drafts."),
                MemoryRefusal::ContainsEmail,
            ),
            (
                draft(MemoryClass::Tone, "Call me on 555 867 5309 0."),
                MemoryRefusal::ContainsPhone,
            ),
            (
                draft(MemoryClass::Unknown("lesson".into()), "Plain prose."),
                MemoryRefusal::CuratedKindOnly,
            ),
            (
                draft(MemoryClass::Unknown("principle".into()), "Plain prose."),
                MemoryRefusal::CuratedKindOnly,
            ),
        ];
        for (d, expected) in refused {
            let kind = d.kind.clone();
            let out = now(b.remember(d)).expect("transport ok");
            assert_eq!(out.expect_err("refused"), expected, "for {kind:?}");
        }

        let accepted = now(b.remember(draft(
            MemoryClass::Tone,
            "Keep status updates to three sentences.",
        )))
        .expect("transport ok")
        .expect("accepted");
        assert_eq!(accepted.state, MemoryState::Candidate);

        let logged: Vec<MemoryClass> = b
            .calls()
            .into_iter()
            .filter_map(|c| match c {
                BackendCall::Remember(k) => Some(k),
                _ => None,
            })
            .collect();
        assert_eq!(logged.len(), 6, "every attempt, refused or not, is logged");
    }

    #[test]
    fn recall_hits_are_rrf_sorted_and_cover_four_corpora() {
        let b = InMemoryChatWorkspaceBackend::seeded();
        let receipt = now(b.recall(SEED_RECALL_QUERY)).expect("receipt");
        assert_eq!(receipt.receipt_id, "rcpt-0001");
        assert_eq!(receipt.search, SEED_RECALL_SEARCH);
        assert!(receipt.as_of.starts_with("2027-"), "{}", receipt.as_of);

        for corpus in [
            RecallCorpus::Words,
            RecallCorpus::Meaning,
            RecallCorpus::Graph,
            RecallCorpus::Thread,
        ] {
            assert!(
                receipt.hits.iter().any(|h| h.corpus == corpus),
                "no hit for {corpus:?}"
            );
        }
        let scores: Vec<f32> = receipt.hits.iter().map(|h| h.rrf_score).collect();
        let mut sorted = scores.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert_eq!(scores, sorted, "hits are ordered by rrf_score descending");
        for corpus in [RecallCorpus::Words, RecallCorpus::Thread] {
            let ranks: Vec<u32> = receipt
                .hits
                .iter()
                .filter(|h| h.corpus == corpus)
                .map(|h| h.rank_in_corpus)
                .collect();
            assert!(ranks.contains(&1), "{corpus:?} ranks start at 1");
        }

        // Disabling use empties the receipt without hiding the call.
        now(b.set_memory_flags(false, true)).expect("flags");
        let off = now(b.recall(SEED_RECALL_QUERY)).expect("receipt");
        assert!(off.hits.is_empty());
        assert_eq!(
            b.calls()
                .into_iter()
                .filter(|c| matches!(c, BackendCall::Recall(_)))
                .count(),
            2
        );
    }

    #[test]
    fn remembered_items_are_not_recallable_until_confirmed() {
        let b = InMemoryChatWorkspaceBackend::seeded();
        let entry = now(b.remember(MemoryDraft {
            kind: MemoryClass::WorkingMethod,
            title: "Cadence".into(),
            body: "Send the courtroom cadence summary every Friday.".into(),
            scope: "office".into(),
        }))
        .expect("transport ok")
        .expect("accepted");
        assert_eq!(entry.state, MemoryState::Candidate);

        let before = now(b.recall("courtroom cadence summary")).expect("receipt");
        assert!(
            !before.hits.iter().any(|h| h.title.contains("cadence")),
            "a candidate is written but not findable"
        );

        now(b.set_memory_state(&entry.id, MemoryState::Confirmed)).expect("confirmed");
        let after = now(b.recall("courtroom cadence summary")).expect("receipt");
        let mine: Vec<&RecallHit> = after
            .hits
            .iter()
            .filter(|h| h.title.contains("cadence"))
            .collect();
        assert!(!mine.is_empty(), "confirmation makes it recallable");
        assert!(
            mine.iter().any(|h| h.corpus == RecallCorpus::Words),
            "words match immediately"
        );
        assert!(
            !mine.iter().any(|h| h.corpus == RecallCorpus::Meaning),
            "the meaning vector has not landed on the first recall after the write"
        );
    }

    #[test]
    fn grounded_not_found_sentence_is_exact_and_assistant_never_emits_it() {
        let sentence = AiChatWorkspaceTexts::default().grounded_not_found;
        let open = |b: &InMemoryChatWorkspaceBackend, posture: ChatPosture| {
            let selection = KnowledgeSelection {
                corpus: Some(CorpusScope::All),
                posture,
                ..KnowledgeSelection::default()
            };
            let t = now(b.open_session(
                "claude-code",
                &selection,
                ChatSettings::default(),
                ProviderTuning::default(),
            ))
            .expect("session");
            ChatSession::new(t)
        };

        // Grounded + no corpus overlap: the exact sentence, NotFound.
        let b = InMemoryChatWorkspaceBackend::seeded();
        let mut s = open(&b, ChatPosture::Grounded);
        ask(&mut s, "zqxjv wibble frobnicate");
        let id = b.current_turn_id().expect("turn");
        drain(&mut s, 80);
        let answer = s
            .messages()
            .iter()
            .find(|m| m.role == ChatRole::Assistant)
            .expect("an answer");
        assert_eq!(answer.content, sentence);
        assert_eq!(
            now(b.turn(&id))
                .expect("record")
                .evidence
                .expect("e")
                .grounding,
            GroundingVerdict::NotFound
        );

        // Grounded + a real overlap: grounded with citations.
        let b = InMemoryChatWorkspaceBackend::seeded();
        let mut s = open(&b, ChatPosture::Grounded);
        ask(
            &mut s,
            "what does the intake checklist require before opening a matter",
        );
        let id = b.current_turn_id().expect("turn");
        drain(&mut s, 80);
        let evidence = now(b.turn(&id)).expect("record").evidence.expect("e");
        assert!(matches!(
            evidence.grounding,
            GroundingVerdict::Grounded { sources } if sources >= 1
        ));
        assert!(!evidence.citations.is_empty());
        assert_ne!(
            s.messages()
                .iter()
                .find(|m| m.role == ChatRole::Assistant)
                .expect("answer")
                .content,
            sentence
        );

        // Assistant posture never emits it, even with no overlap at all.
        let b = InMemoryChatWorkspaceBackend::seeded();
        let mut s = open(&b, ChatPosture::Assistant);
        ask(&mut s, "zqxjv wibble frobnicate");
        let id = b.current_turn_id().expect("turn");
        drain(&mut s, 80);
        assert_ne!(
            s.messages()
                .iter()
                .find(|m| m.role == ChatRole::Assistant)
                .expect("answer")
                .content,
            sentence
        );
        assert_eq!(
            now(b.turn(&id))
                .expect("record")
                .evidence
                .expect("e")
                .grounding,
            GroundingVerdict::AssistantOnly
        );
    }

    #[test]
    fn default_scripts_differ_per_engine() {
        let mut transcripts: Vec<String> = Vec::new();
        let mut records: Vec<TurnRecord> = Vec::new();
        for engine in [
            "claude-code",
            "codex-cli",
            "codex-spark",
            "groq-gpt-oss-120b",
            "ollama",
        ] {
            let b = InMemoryChatWorkspaceBackend::seeded();
            let mut s = session(&b, engine);
            ask(&mut s, "hello");
            let id = b.current_turn_id().expect("turn");
            drain(&mut s, 80);
            let record = now(b.turn(&id)).expect("record");
            let annotations = b.annotations(&id);
            transcripts.push(format!(
                "{:?}|{:?}|{:?}",
                s.messages()
                    .iter()
                    .map(|m| (m.role, m.content.clone()))
                    .collect::<Vec<_>>(),
                record.usage,
                annotations
            ));
            records.push(record);
        }
        for i in 0..transcripts.len() {
            for j in (i + 1)..transcripts.len() {
                assert_ne!(transcripts[i], transcripts[j], "engines {i} and {j} match");
            }
        }
        // Claude alone reports a cache read and a metered cost.
        assert!(transcripts[0].contains("cache_read_tokens: 6144"));
        assert_eq!(
            records[0].usage.expect("claude usage").metered_cost,
            Some(0.0143)
        );
        // Spark answers from its pinned model, whatever the session selected.
        assert!(transcripts[2].contains("gpt-5.3-codex-spark"));
        // Ollama is local: plan-covered, unpriced, and no reasoning tokens.
        let ollama = records[4].usage.expect("ollama usage");
        assert_eq!(ollama.metered_cost, None, "a local engine bills nothing");
        assert_eq!(ollama.plan.reasoning_tokens, 0);
        // Groq alone carries the escalation and truncation annotations.
        assert!(transcripts[3].contains("Escalated from medium to low"));
        assert!(transcripts[3].contains("Response truncated"));
        // The notices themselves are on the record, which is what a composite
        // holding a `dyn ChatWorkspaceBackend` can reach; the annotations
        // above are only their wording.
        assert_eq!(
            records[3].notices,
            vec![
                TurnNotice::Escalated {
                    from: ReasoningEffort::Medium,
                    to: ReasoningEffort::Low
                },
                TurnNotice::Truncated
            ],
            "groq's default script escalates, then truncates, in that order"
        );
        assert!(
            records[0].notices.is_empty(),
            "an ordinary turn carries no notices"
        );

        // codex-cli echoes the model the SESSION selected, not a fallback.
        let b = InMemoryChatWorkspaceBackend::seeded();
        let transport = now(b.open_session(
            "codex-cli",
            &KnowledgeSelection {
                posture: ChatPosture::Assistant,
                ..KnowledgeSelection::default()
            },
            ChatSettings {
                model: Some("gpt-5.7-codex".into()),
                ..ChatSettings::default()
            },
            ProviderTuning::default(),
        ))
        .expect("session opens");
        let mut s = ChatSession::new(transport);
        ask(&mut s, "hello");
        drain(&mut s, 80);
        assert!(
            s.messages()
                .iter()
                .any(|m| m.content.contains("gpt-5.7-codex")),
            "codex-cli names the selected model: {:?}",
            s.messages()
        );
    }

    #[test]
    fn clock_advances_per_poll_and_stall_never_completes() {
        // The clock is cloned rather than only reached through the backend,
        // so `FixtureClock::elapsed_ms` — the accessor a host watchdog owns —
        // is exercised by the same run.
        let clock = FixtureClock::default();
        let b = InMemoryChatWorkspaceBackend::seeded()
            .with_clock(clock.clone())
            .with_script(
                "claude-code",
                PromptMatcher::Any,
                TurnScript::new().text_words("thinking about it").stall(),
            );
        assert_eq!(b.now_ms(), SEED_NOW_MS);
        assert_eq!(clock.elapsed_ms(), 0);
        let mut s = session(&b, "claude-code");
        ask(&mut s, "wait for me");
        let id = b.current_turn_id().expect("turn");

        // Two ladder polls, three word polls, then nothing but empty polls.
        for _ in 0..1200 {
            s.poll();
        }
        // Exactly, not `>=`: two ladder polls at one `try_recv` each, three
        // streaming polls that spend a second `try_recv` ending the burst,
        // then 1195 empty polls at one each. A `>=` would hide a change to
        // the burst accounting, and the brief's "1200 × 100 ms" arithmetic
        // holds only for the ladder and stalled polls.
        assert_eq!(b.now_ms(), SEED_NOW_MS + 120_300);
        assert_eq!(clock.elapsed_ms(), 120_300);
        let record = now(b.turn(&id)).expect("record");
        assert_eq!(record.lifecycle, AttemptLifecycle::Running);
        assert_eq!(b.current_turn_id().as_deref(), Some(id.as_str()));

        // A faster clock covers the same span in fewer polls.
        let b = InMemoryChatWorkspaceBackend::seeded()
            .with_clock(FixtureClock::new(SEED_NOW_MS, 1_000))
            .with_script("claude-code", PromptMatcher::Any, TurnScript::new().stall());
        let mut s = session(&b, "claude-code");
        ask(&mut s, "wait");
        for _ in 0..120 {
            s.poll();
        }
        // A script with no events at all: every poll is one empty `try_recv`,
        // so this one IS exactly 120 × 1000 ms.
        assert_eq!(b.now_ms(), SEED_NOW_MS + 120_000);
    }
}
