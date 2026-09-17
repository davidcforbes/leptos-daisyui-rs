//! Knowledge sources, ingest status and query modes the chat workspace
//! mixes into a turn, plus the pure text guardrail the memory-capture flow
//! checks before a candidate ever reaches the host.

use crate::components::ai_assistant_workspace::{
    AssistantKnowledgeEntry, AssistantMemory, AssistantReason, MemoryClass,
};

/// One thing the composite can ground a turn against, or manage on its own.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum KnowledgeSource {
    /// A file/folder/custom corpus the host indexes for this workspace.
    Corpus {
        /// What the corpus covers.
        scope: CorpusScope,
        /// The corpus's current ingest progress.
        ingest: IngestStatus,
        /// How a turn queries this corpus.
        query_mode: CorpusQueryMode,
        /// Whether a turn against this corpus must stay grounded or may
        /// fall back to general assistance.
        posture: ChatPosture,
    },
    /// The actor's own personal memory store, use/capture only (no entries
    /// — see [`KnowledgeSource::PersonalMemory`] for those).
    MemoryStore {
        /// Whether confirmed memory may be used in assistance.
        enabled: bool,
        /// Whether new memory may be captured.
        capture_enabled: bool,
    },
    /// Curated organizational knowledge (Group Important / Foundation).
    OfficeKnowledge {
        /// Which office-knowledge collection this is.
        scope: OfficeKnowledgeScope,
        /// The entries currently available from this collection.
        entries: Vec<AssistantKnowledgeEntry>,
    },
    /// The actor's full personal-memory collection, capabilities and all.
    PersonalMemory(AssistantMemory),
}

/// What a corpus covers.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorpusScope {
    /// One file, by host-resolved path.
    File(String),
    /// One folder, by host-resolved path.
    Folder(String),
    /// Every indexed file the actor may see.
    All,
    /// An explicit, named set of paths.
    Custom {
        /// Display label for the selection.
        label: String,
        /// The selected paths.
        paths: Vec<String>,
    },
}

impl CorpusScope {
    /// Stable id for `data-*` hooks and telemetry.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::File(_) => "file",
            Self::Folder(_) => "folder",
            Self::All => "all",
            Self::Custom { .. } => "custom",
        }
    }
}

/// A corpus's indexing progress.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IngestPhase {
    /// No indexing has been requested (or is due).
    Idle,
    /// Enumerating files in scope.
    Walking,
    /// Building the search index.
    Indexing,
    /// Building semantic clusters over the index.
    Clustering,
    /// Indexing completed and the corpus is queryable.
    Ready,
    /// Indexing failed.
    Failed(AssistantReason),
}

impl IngestPhase {
    /// Stable id for `data-*` hooks and telemetry.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Walking => "walking",
            Self::Indexing => "indexing",
            Self::Clustering => "clustering",
            Self::Ready => "ready",
            Self::Failed(_) => "failed",
        }
    }
}

/// A corpus's current ingest progress and last-known counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngestStatus {
    /// Current phase.
    pub phase: IngestPhase,
    /// Files enumerated so far.
    pub files_seen: u32,
    /// Files actually indexed so far.
    pub files_indexed: u32,
    /// Semantic clusters built so far.
    pub clusters: u32,
    /// Host-localized freshness of this status.
    pub as_of: String,
}

/// How a turn queries a corpus.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorpusQueryMode {
    /// Plain keyword/full-text search.
    FullText,
    /// Embedding similarity search.
    Similarity,
    /// An LLM-generated query reformulation.
    Llm,
    /// Full-text and similarity results reciprocal-rank fused.
    Fused,
}

impl CorpusQueryMode {
    /// Stable id for `data-*` hooks and telemetry.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::FullText => "full_text",
            Self::Similarity => "similarity",
            Self::Llm => "llm",
            Self::Fused => "fused",
        }
    }

    /// Parses a stable id back into a mode, or `None` if unrecognized.
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "full_text" => Some(Self::FullText),
            "similarity" => Some(Self::Similarity),
            "llm" => Some(Self::Llm),
            "fused" => Some(Self::Fused),
            _ => None,
        }
    }
}

/// Whether a turn against a corpus must stay grounded in it, or may fall
/// back to general assistance.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatPosture {
    /// The answer must come from the corpus; an absent answer is reported
    /// as not-found rather than guessed.
    Grounded,
    /// The engine may answer generally when the corpus does not cover it.
    Assistant,
}

impl ChatPosture {
    /// Stable id for `data-*` hooks and telemetry.
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Grounded => "grounded",
            Self::Assistant => "assistant",
        }
    }

    /// Parses a stable id back into a posture, or `None` if unrecognized.
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "grounded" => Some(Self::Grounded),
            "assistant" => Some(Self::Assistant),
            _ => None,
        }
    }
}

/// Which curated organizational-knowledge collection a
/// [`KnowledgeSource::OfficeKnowledge`] draws from.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OfficeKnowledgeScope {
    /// Group Important: reviewed, high-visibility answers.
    GroupImportant,
    /// Foundation: the shared organizational baseline.
    Foundation,
    /// A host scope this vocabulary does not name.
    Unknown(String),
}

impl OfficeKnowledgeScope {
    /// The wire id this variant round-trips to; `Unknown` returns the
    /// original id it was parsed from.
    pub fn as_id(&self) -> &str {
        match self {
            Self::GroupImportant => "group_important",
            Self::Foundation => "foundation",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// Parses the host's wire id, preserving an unrecognized one verbatim.
    pub fn parse(id: &str) -> Self {
        match id {
            "group_important" => Self::GroupImportant,
            "foundation" => Self::Foundation,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// What knowledge a turn should be opened against. `Default` mixes in
/// nothing but the actor's own memory, fused-queried and grounded — the
/// most conservative useful starting point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeSelection {
    /// The corpus to ground against, if any.
    pub corpus: Option<CorpusScope>,
    /// How to query it.
    pub query_mode: CorpusQueryMode,
    /// Whether the turn must stay grounded or may fall back to general
    /// assistance.
    pub posture: ChatPosture,
    /// Whether personal-memory recall should be mixed in.
    pub memory_recall: bool,
    /// Which office-knowledge collections should be mixed in.
    pub office_scopes: Vec<OfficeKnowledgeScope>,
}

impl Default for KnowledgeSelection {
    fn default() -> Self {
        Self {
            corpus: None,
            query_mode: CorpusQueryMode::Fused,
            posture: ChatPosture::Grounded,
            memory_recall: true,
            office_scopes: Vec::new(),
        }
    }
}

/// Which recall corpus one [`RecallHit`] came from. `Unknown` preserves a
/// host corpus this vocabulary does not yet name.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecallCorpus {
    /// Keyword/full-text recall.
    Words,
    /// Embedding/semantic recall.
    Meaning,
    /// Knowledge-graph recall.
    Graph,
    /// Recall over the current conversation's own history.
    Thread,
    /// A host corpus this vocabulary does not name.
    Unknown(String),
}

impl RecallCorpus {
    /// The stable wire string this corpus round-trips to, rendered on a
    /// hit's `data-ai-chat-memory-attribution` hook.
    ///
    /// Never `{:?}`: `Debug` prints a Rust variant name a rename is free to
    /// change. `Unknown` returns the host's own lane id verbatim, because a
    /// hit attributed to a lane this vocabulary has not learned yet must
    /// still say WHICH lane — narrating "by meaning" for a lane the service
    /// never named is exactly the claim this attribution exists to prevent.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Words => "words",
            Self::Meaning => "meaning",
            Self::Graph => "graph",
            Self::Thread => "thread",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// Parses the host's wire id, preserving an unrecognized one verbatim.
    pub fn parse(id: &str) -> Self {
        match id {
            "words" => Self::Words,
            "meaning" => Self::Meaning,
            "graph" => Self::Graph,
            "thread" => Self::Thread,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// One recall result.
#[derive(Clone, Debug, PartialEq)]
pub struct RecallHit {
    /// Display title.
    pub title: String,
    /// A short excerpt.
    pub snippet: String,
    /// Which corpus produced it.
    pub corpus: RecallCorpus,
    /// Reciprocal-rank-fusion score, for relative ordering only — never
    /// compared across two different receipts.
    pub rrf_score: f32,
    /// Rank within its own corpus (1-based), before fusion.
    pub rank_in_corpus: u32,
}

/// A completed recall search: what was asked, when, and what came back.
#[derive(Clone, Debug, PartialEq)]
pub struct RecallReceipt {
    /// Stable id for this receipt, so a turn can cite it.
    pub receipt_id: String,
    /// Host-localized time the search ran.
    pub as_of: String,
    /// If this recall continues an earlier one, that receipt's id.
    pub since: Option<String>,
    /// The results, in host-decided display order.
    pub hits: Vec<RecallHit>,
    /// The host's own presentable description of the search that ran, shown
    /// verbatim rather than reconstructed from the query modes. The real
    /// memory service returns this string beside the hits, and a workspace
    /// that paraphrased it would claim a retrieval strategy it cannot see.
    pub search: String,
}

/// A proposed personal-memory candidate, before the guardrail or the host
/// has accepted it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryDraft {
    /// The proposed memory class.
    pub kind: MemoryClass,
    /// Short title.
    pub title: String,
    /// The proposed memory text.
    pub body: String,
    /// Host-defined scope this memory would apply within.
    pub scope: String,
}

/// Why [`guardrail_refusal`] refused a memory candidate. `Unknown` is not
/// produced by the pure guardrail itself; it exists so a host-side refusal
/// reason can travel through the same vocabulary.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryRefusal {
    /// The text appears to contain a matter number.
    ContainsMatterNumber,
    /// The text appears to contain an email address.
    ContainsEmail,
    /// The text appears to contain a phone number.
    ContainsPhone,
    /// The proposed class is one a curator creates, never a write. The real
    /// memory service refuses `lesson` and `principle` on write: those are
    /// distilled out of accepted items during curation, so accepting one
    /// directly would let a workspace mint organizational doctrine.
    CuratedKindOnly,
    /// A host refusal reason this vocabulary does not name.
    Unknown(String),
}

impl MemoryRefusal {
    /// The stable wire string this refusal round-trips to, rendered on the
    /// rail's `data-ai-chat-guardrail` hook.
    ///
    /// The id set is CLOSED at five, and `Unknown` collapses to `"unknown"`
    /// rather than leaking the host's reason string — unlike
    /// [`RecallCorpus::as_str`] and
    /// [`crate::components::ai_assistant_workspace::MemoryClass::as_str`],
    /// which preserve theirs. The difference is what the hook is for: a
    /// proof asserts that a specific guardrail fired, so an open id set
    /// would let an unrecognized host reason pass an assertion written for a
    /// typed one. The host's own sentence is still rendered, as the row's
    /// TEXT.
    ///
    /// The match is exhaustive on purpose even though the enum is
    /// `#[non_exhaustive]`: inside this crate a new refusal must break THIS
    /// function rather than quietly inherit `"unknown"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ContainsMatterNumber => "contains_matter_number",
            Self::ContainsEmail => "contains_email",
            Self::ContainsPhone => "contains_phone",
            Self::CuratedKindOnly => "curated_kind_only",
            Self::Unknown(_) => "unknown",
        }
    }

    /// Parses a wire string produced by [`Self::as_str`]; anything else —
    /// including `"unknown"` itself, which names no reason — is carried as
    /// [`Self::Unknown`] rather than guessed at.
    pub fn parse(id: &str) -> Self {
        match id {
            "contains_matter_number" => Self::ContainsMatterNumber,
            "contains_email" => Self::ContainsEmail,
            "contains_phone" => Self::ContainsPhone,
            "curated_kind_only" => Self::CuratedKindOnly,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

fn contains_matter_number(text: &str) -> bool {
    for segment in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        if segment.is_empty() {
            continue;
        }
        let parts: Vec<&str> = segment.split('-').collect();
        for pair in parts.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let a_is_two_digits = a.len() == 2 && a.chars().all(|c| c.is_ascii_digit());
            let a_is_letter_prefix =
                (2..=4).contains(&a.len()) && a.chars().all(|c| c.is_ascii_uppercase());
            let b_is_long_digits = b.len() >= 4 && b.chars().all(|c| c.is_ascii_digit());
            let b_is_short_digits = b.len() >= 3 && b.chars().all(|c| c.is_ascii_digit());
            if (a_is_two_digits && b_is_long_digits) || (a_is_letter_prefix && b_is_short_digits) {
                return true;
            }
        }
    }
    false
}

fn contains_email(text: &str) -> bool {
    for raw_token in text.split_whitespace() {
        let token: String = raw_token
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '-' | '+'))
            .collect();
        let Some(at) = token.find('@') else {
            continue;
        };
        let local = &token[..at];
        let domain = &token[at + 1..];
        if local.is_empty() || domain.is_empty() {
            continue;
        }
        let Some(dot) = domain.rfind('.') else {
            continue;
        };
        let tld = &domain[dot + 1..];
        if tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()) {
            return true;
        }
    }
    false
}

fn contains_phone_number(text: &str) -> bool {
    let mut digit_count = 0usize;
    for ch in text.chars() {
        let is_run_char = ch.is_ascii_digit() || matches!(ch, ' ' | '-' | '(' | ')' | '.' | '+');
        if is_run_char {
            if ch.is_ascii_digit() {
                digit_count += 1;
            }
        } else {
            if digit_count >= 10 {
                return true;
            }
            digit_count = 0;
        }
    }
    digit_count >= 10
}

/// Refuses obvious matter numbers, emails and phone numbers before a memory
/// candidate ever reaches the host. Deliberately conservative pattern
/// matching, not a full PII scrubber: it exists to stop obviously-scoped
/// identifiers from being memorized, not to promise privacy.
///
/// Checked in this order, first match wins:
/// 1. A matter number: two digits then a hyphen then four or more digits
///    (`24-8842`), or two to four uppercase letters then a hyphen then
///    three or more digits (`ZW-2301`) — both anchored so a plain phone
///    number like `555-867-5309` never matches (its digit groups are three
///    digits wide, not the required two).
/// 2. An email address: a `local@domain.tld` shape with a two-or-more
///    letter top-level domain.
/// 3. A phone number: ten or more digits within one contiguous run of
///    digits and the separators `+ - ( ) .` and spaces, so scattered,
///    unrelated numbers across a sentence never trip it.
pub fn guardrail_refusal(text: &str) -> Option<MemoryRefusal> {
    if contains_matter_number(text) {
        Some(MemoryRefusal::ContainsMatterNumber)
    } else if contains_email(text) {
        Some(MemoryRefusal::ContainsEmail)
    } else if contains_phone_number(text) {
        Some(MemoryRefusal::ContainsPhone)
    } else {
        None
    }
}
