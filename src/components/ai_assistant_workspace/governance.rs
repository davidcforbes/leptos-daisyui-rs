//! Actor-owned memory and independently reviewed organizational knowledge.

use super::*;

/// Personal working preferences, never an unrestricted client-fact category.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryClass {
    /// Preferred communication language.
    Language,
    /// Preferred tone.
    Tone,
    /// Preferred amount of detail.
    Verbosity,
    /// A personal working method.
    WorkingMethod,
    /// Unsupported memory classes cannot be captured or confirmed.
    Unknown(String),
}

/// Candidate, confirmed and withdrawn are distinct host-owned states.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryState {
    /// Proposed personal memory, not yet confirmed for assistance.
    Candidate,
    /// Host-confirmed personal working preference.
    Confirmed,
    /// Removed from assistance; not a claim of physical erasure everywhere.
    Withdrawn,
    /// Unsupported memory state cannot authorize an interaction.
    Unknown(String),
}

/// Explicit retention facts attached to a withdrawal acknowledgment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantRetention {
    /// Whether the host actually removed the item from assistance.
    pub removed_from_assistance: bool,
    /// Known physical-purge status; absence means unknown.
    pub physical_purge: Option<String>,
    /// Known legal-hold status; absence means unknown.
    pub legal_hold: Option<String>,
    /// Known backup-retention status; absence means unknown.
    pub backup_expiry: Option<String>,
    /// Known provider-retention status; absence means unknown.
    pub provider_retention: Option<String>,
}

/// Versioned personal memory, authorized independently of the viewed subject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantMemoryEntry {
    /// Stable item identity.
    pub id: String,
    /// Accepted item revision.
    pub revision: u64,
    /// Explicit allowed personal-memory class.
    pub class: MemoryClass,
    /// Accepted plain-text working preference.
    pub text: String,
    /// Current candidate/confirmation/withdrawal state.
    pub state: MemoryState,
    /// Item-specific grants, additionally restricted by collection and context.
    pub capabilities: AssistantCapabilities,
    /// Actual retention facts, especially after withdrawal.
    pub retention: Option<AssistantRetention>,
}

/// Host-controlled memory form, distinct from accepted entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantMemoryDraft {
    /// Current form revision, advanced on every accepted edit.
    pub revision: u64,
    /// Explicit personal-memory class.
    pub class: MemoryClass,
    /// Proposed plain text; a blank form is not a saved memory.
    pub text: String,
    /// Host-supplied scalar-value bound.
    pub max_chars: usize,
}

/// Actor-owned memory with separate policy use and capture permissions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantMemory {
    /// Authenticated actor identity; never the viewed subject.
    pub owner_actor_id: String,
    /// Accepted collection revision.
    pub revision: u64,
    /// Whether confirmed memory can be used in assistance.
    pub use_enabled: bool,
    /// Whether memory capture is allowed; independent of use.
    pub capture_enabled: bool,
    /// Authorized accepted entries, not a inferred empty collection.
    pub entries: Vec<AssistantMemoryEntry>,
    /// Current host-controlled create/correct form.
    pub draft: AssistantMemoryDraft,
    /// Collection-level grants.
    pub capabilities: AssistantCapabilities,
}

/// Organizational candidate lifecycle, separate from curator review receipts.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KnowledgeState {
    /// An unpublished author draft.
    Draft,
    /// Submitted for review; approval does not itself publish it.
    PendingReview,
    /// Publication has been independently confirmed by the host.
    Published,
    /// The candidate was rejected.
    Rejected,
    /// The curator requested a new proposed revision.
    ChangesRequested,
    /// Knowledge was withdrawn from active assistance.
    Withdrawn,
    /// A later accepted revision superseded this one.
    Superseded,
    /// The host archived this item under its retention policy.
    Archived,
    /// This knowledge projection is unavailable, not an empty published body.
    Unavailable,
    /// Unsupported state, never an implicit publication or review approval.
    Unknown(String),
}

/// Curator review is independent of the submitting author's local review.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KnowledgeReviewState {
    /// No accepted curator review for the candidate.
    NotReviewed,
    /// The host reserved a review of an exact proposed revision.
    Pending {
        /// Correlated review request.
        request_id: String,
        /// Candidate revision under review.
        proposed_revision: u64,
    },
    /// Host-confirmed curator approval; publication is still a separate action.
    Approved {
        /// Correlated review request.
        request_id: String,
        /// Exact candidate revision approved by the curator.
        proposed_revision: u64,
        /// Authenticated reviewing actor.
        reviewer_id: String,
        /// Accepted review receipt revision.
        receipt_revision: u64,
    },
    /// The curator explicitly rejected the reviewed candidate.
    Rejected {
        /// Named sanitized explanation.
        reason: AssistantReason,
    },
    /// Unsupported review state, never an approval.
    Unknown(String),
}

/// Reviewable organizational candidate, never a silently shared personal answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantKnowledgeEntry {
    /// Stable candidate identity.
    pub id: String,
    /// Current proposed/accepted item revision.
    pub revision: u64,
    /// Host-projected title.
    pub title: String,
    /// Canonical question answered by the knowledge.
    pub canonical_question: String,
    /// English aliases, kept distinct from translated interface text.
    pub aliases_en: Vec<String>,
    /// Spanish aliases.
    pub aliases_es: Vec<String>,
    /// Host-authorized audience display, not a permission grant.
    pub audience: String,
    /// Lifecycle independent of curator review.
    pub state: KnowledgeState,
    /// Current reviewable proposed body; absence disables candidate review.
    pub proposed_body: Option<String>,
    /// Separately accepted published body, never substituted for review text.
    pub published_body: Option<String>,
    /// Exact source revisions offered for review.
    pub sources: Vec<AssistantItemRevision>,
    /// Preceding knowledge revisions and lineage.
    pub lineage: Vec<AssistantItemRevision>,
    /// Host-projected comparison required by curator decisions.
    pub comparison: Option<String>,
    /// Independently acknowledged curator review.
    pub review: KnowledgeReviewState,
    /// Per-item grants, independent of submitting and publishing permissions.
    pub capabilities: AssistantCapabilities,
}

/// Explicit Group Important authoring form, not an automatically shared answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantKnowledgeDraft {
    /// Current form revision.
    pub revision: u64,
    /// Proposed title.
    pub title: String,
    /// Proposed canonical question.
    pub canonical_question: String,
    /// Proposed English aliases.
    pub aliases_en: Vec<String>,
    /// Proposed Spanish aliases.
    pub aliases_es: Vec<String>,
    /// Explicit intended audience.
    pub audience: String,
    /// Proposed organizational body.
    pub body: String,
    /// Exact proposed source revisions.
    pub sources: Vec<AssistantItemRevision>,
    /// Author-reviewed form revision; never a curator approval.
    pub reviewed_revision: Option<u64>,
}

/// Explicit foundation coverage, never substituted by hidden fallback facts.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FoundationCoverage {
    /// Host-confirmed complete coverage for the stated foundation.
    Complete,
    /// The foundation covers only part of its declared scope.
    Partial {
        /// Named gap explanation.
        reason: AssistantReason,
    },
    /// Required foundation material is absent.
    Missing {
        /// Named missing-source explanation.
        reason: AssistantReason,
    },
    /// The foundation was withdrawn from use.
    Withdrawn {
        /// Named withdrawal explanation.
        reason: AssistantReason,
    },
    /// Unknown coverage cannot establish completeness.
    Unknown(String),
}

/// Versioned shared foundation with visible ownership and review dates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantFoundation {
    /// Stable foundation identity.
    pub id: String,
    /// Host-defined business domain.
    pub domain: String,
    /// Accountable owner display.
    pub owner: String,
    /// Accepted foundation revision.
    pub revision: u64,
    /// Host-localized effective date.
    pub effective_at: String,
    /// Host-localized review date or explicit unavailable-date explanation.
    pub review_at: String,
    /// Explicit coverage and gaps.
    pub coverage: FoundationCoverage,
    /// Exact source identities supplied by the host.
    pub source_ids: Vec<String>,
}

/// Independently available shared knowledge and foundations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantKnowledge {
    /// Accepted collection revision.
    pub revision: u64,
    /// Authorized candidate and published projections.
    pub entries: Vec<AssistantKnowledgeEntry>,
    /// Sole host-controlled authoring form.
    pub draft: AssistantKnowledgeDraft,
    /// Explicit foundation projections and coverage gaps.
    pub foundations: Vec<AssistantFoundation>,
    /// Collection-level grants.
    pub capabilities: AssistantCapabilities,
}

/// Metadata-only disposition for history and usage, never an answer body.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantRecordedOutcome {
    /// Work was admitted.
    Admitted,
    /// Work is queued.
    Queued,
    /// Work is running.
    Running,
    /// Output is being validated.
    Validating,
    /// A grounded answer was accepted.
    Answered,
    /// Healthy execution declined on grounding limits.
    Declined,
    /// The admitted operation was denied.
    Denied,
    /// Execution was unavailable.
    Unavailable,
    /// Execution failed; this metadata is not a transcript error bubble.
    Failed,
    /// Cancellation may still have incurred work discarded by the host.
    Canceled {
        /// Whether paid/generated output was discarded.
        discarded: bool,
    },
    /// The host interrupted the work.
    Interrupted,
    /// The request was refused before admission.
    Refused {
        /// Distinct pre-admission refusal reason.
        reason: RefusalReason,
    },
    /// Unsupported metadata remains explicitly unknown.
    Unknown(String),
}

/// One authorized application-history item, not a replay URL or provider session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantHistoryEntry {
    /// Stable history identity.
    pub id: String,
    /// Accepted retained-item revision.
    pub revision: u64,
    /// Safe host-projected title.
    pub title: String,
    /// Original accepted scope, not relabeled to the current page.
    pub scope: AssistantScope,
    /// Actual executor when known, not the current selected preference.
    pub actual_engine: Option<String>,
    /// Host-localized historical time.
    pub at: String,
    /// Metadata-only original disposition.
    pub outcome: AssistantRecordedOutcome,
    /// Independent current access to retained evidence.
    pub retained_evidence: AssistantAccess,
    /// Item-specific history grant.
    pub capabilities: AssistantCapabilities,
}

/// Explicit application history; unavailable retention is not an empty list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantHistory {
    /// Accepted history collection revision.
    pub revision: u64,
    /// Authorized metadata only; loading an entry never emits Ask.
    pub entries: Vec<AssistantHistoryEntry>,
    /// Collection-level history permissions.
    pub capabilities: AssistantCapabilities,
}

/// Accounting classes are not additive descriptions of the same actual charge.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UsageCharge {
    /// Charge is not known; the opaque code never becomes a zero dollar amount.
    Unknown(String),
    /// A hypothetical API-equivalent value, not verified incremental spending.
    EstimatedApiEquivalent {
        /// Host-localized equivalent-value display.
        display: String,
    },
    /// Independently verified incremental cost.
    VerifiedIncremental {
        /// Host-localized verified charge display.
        display: String,
    },
    /// A fixed subscription and optional allocation, never summed with estimates.
    SubscriptionAllocation {
        /// Host-localized fixed subscription cost.
        fixed_cost: String,
        /// Known allocation display; absence remains unknown.
        allocation: Option<String>,
    },
}

/// Optional measured tokens and a separately classified charge projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantUsage {
    /// Reported input tokens; absence is unknown and zero is a real measurement.
    pub input_tokens: Option<u64>,
    /// Reported output tokens; absence is unknown and zero is a real measurement.
    pub output_tokens: Option<u64>,
    /// Explicit accounting class; this component performs no mixed-cost sum.
    pub charge: UsageCharge,
}

/// Ordinary activity exposes no raw question, private memory or provider error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantActivityEntry {
    /// Stable activity identity.
    pub id: String,
    /// Host-localized event time.
    pub at: String,
    /// Original attempt or refusal correlation identity.
    pub attempt_id: String,
    /// Original host-authorized scope.
    pub scope: AssistantScope,
    /// Actual executor when known.
    pub engine: Option<String>,
    /// Includes refusals, failures and canceled/discarded outcomes.
    pub outcome: AssistantRecordedOutcome,
    /// Measured/unknown usage and honest accounting class.
    pub usage: AssistantUsage,
    /// Independent current access to further detail.
    pub detail_availability: AssistantAccess,
}

/// Ordinary activity is separate from the permissioned learning-review surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantActivity {
    /// Accepted activity revision.
    pub revision: u64,
    /// Authorized metadata-only usage entries.
    pub entries: Vec<AssistantActivityEntry>,
    /// Host-configured budget only.
    pub budget: Option<AssistantBudget>,
}

/// Coverage truth, including missing data and explicitly paused capacity.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoverageState {
    /// Host-confirmed complete coverage of the stated interval and stage.
    Complete,
    /// Only part of the required inventory was covered.
    Partial { /// Named cause.
        reason: AssistantReason },
    /// The required inventory was missed.
    Missed { /// Named cause.
        reason: AssistantReason },
    /// This coverage lane has not been configured.
    Unconfigured { /// Named cause.
        reason: AssistantReason },
    /// Available evidence cannot establish coverage.
    Unassessable { /// Named cause.
        reason: AssistantReason },
    /// Coverage is paused under the stated capacity policy.
    CapacityPaused { /// Named cause.
        reason: AssistantReason },
    /// Unsupported coverage cannot imply an all-clear.
    Unknown(String),
}

/// A single interval/stage coverage projection, with unknown counts preserved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantCoverage {
    /// Host-confirmed coverage disposition.
    pub state: CoverageState,
    /// Known total inventory; absence is not zero.
    pub total: Option<u64>,
    /// Known covered inventory.
    pub covered: Option<u64>,
    /// Known refusal count.
    pub refused: Option<u64>,
    /// Known failure count.
    pub failed: Option<u64>,
    /// Known canceled count, including discarded outcomes where the host counts them.
    pub canceled: Option<u64>,
    /// Named missing partitions, not silently omitted from coverage.
    pub missing_partitions: Vec<String>,
    /// Host-localized oldest outstanding item/time when known.
    pub oldest_outstanding: Option<String>,
    /// Accountable owner when assigned.
    pub owner: Option<String>,
    /// Proposed recovery, never a job started by this component.
    pub proposed_recovery: Option<String>,
}

/// Review stages are independently covered, not one combined success counter.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LearningStage {
    /// Complete inventory capture.
    Inventory,
    /// Reused evaluation coverage.
    ReusedEvaluation,
    /// Model-produced assessment coverage.
    ModelAssessment,
    /// Human review coverage.
    HumanReview,
    /// Publication coverage.
    Publication,
    /// Unsupported stages do not imply completeness.
    Unknown(String),
}

/// One named stage and its independently accepted coverage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantStageCoverage {
    /// Explicit stage identity.
    pub stage: LearningStage,
    /// Stage-local coverage and causes.
    pub coverage: AssistantCoverage,
}

/// Typed assessment result; a flag is not automatically an established error.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssessmentOutcome {
    /// The attributed reviewer judged the output correct.
    Correct,
    /// The attributed reviewer judged the output incorrect.
    Incorrect,
    /// The output was incomplete.
    Incomplete,
    /// Supplied evidence could not answer the question.
    Unanswerable,
    /// A refusal was judged appropriate.
    AppropriateRefusal,
    /// The available material could not be assessed.
    Unassessable,
    /// Unsupported assessment remains unknown.
    Unknown(String),
}

/// Origin of an assessment, never silently upgraded to human adjudication.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssessmentAttribution {
    /// End-user feedback, not an independently adjudicated fact.
    UserFeedback,
    /// A model-generated grade.
    ModelGrade,
    /// An actual human adjudication.
    HumanAdjudication,
    /// Unknown attribution cannot establish adjudication.
    Unknown(String),
}

/// An explicitly attributed assessment with its own evaluated scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantAssessment {
    /// Typed assessment result.
    pub outcome: AssessmentOutcome,
    /// Actual assessment source.
    pub attribution: AssessmentAttribution,
    /// Display-safe rationale supplied by the reviewer/host.
    pub rationale: String,
    /// Evaluated use-case or population scope.
    pub scope: String,
    /// Evaluated language, unknown when not supplied.
    pub language: Option<String>,
    /// Evaluated actual engine, unknown when not supplied.
    pub engine: Option<String>,
}

/// Improvement progression is not collapsed into a generic completed flag.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImprovementState {
    /// A gap or contradiction was detected.
    Detected,
    /// The finding was triaged.
    Triaged,
    /// An accountable owner accepted it.
    Owned,
    /// A change was proposed.
    Proposed,
    /// The proposed change was approved, not implemented.
    Approved,
    /// A change was implemented, not necessarily released.
    Implemented,
    /// A version was published, not necessarily verified.
    Published,
    /// The released change has explicit verification evidence.
    Verified,
    /// The released change is being monitored for comparable opportunities.
    Monitored,
    /// The finding/change was rejected.
    Rejected,
    /// The finding/change was deliberately deferred.
    Deferred,
    /// Unsupported progression cannot mean fixed.
    Unknown(String),
}

/// A tracked improvement projection; this UI creates no issue, job or deployment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantImprovement {
    /// Stable improvement identity.
    pub id: String,
    /// Accepted improvement revision.
    pub revision: u64,
    /// Explicit source/foundation gap when present.
    pub source_gap: Option<String>,
    /// Known root cause when established.
    pub root_cause: Option<String>,
    /// Known contradiction requiring reconciliation.
    pub contradiction: Option<String>,
    /// Full accepted progression state.
    pub state: ImprovementState,
    /// Accountable owner when assigned.
    pub owner: Option<String>,
    /// Deliberate current disposition.
    pub disposition: String,
    /// Actual released version, absent before release or when unknown.
    pub released_version: Option<String>,
    /// Explicit host-projected verification evidence, not just a version string.
    pub verification_evidence: Vec<String>,
    /// Host-localized monitoring time when known.
    pub monitored_at: Option<String>,
    /// Comparable recurrence count; absent is unknown.
    pub recurrence_count: Option<u64>,
    /// Comparable opportunities; zero cannot demonstrate a fix.
    pub opportunity_count: Option<u64>,
    /// Explicit evaluated use-case scope.
    pub use_case: String,
    /// Evaluated language when supplied.
    pub language: Option<String>,
    /// Evaluated engine when supplied.
    pub engine: Option<String>,
}

/// Host-owned learning capacity disposition, not a local scheduling decision.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapacityState {
    /// The configured capacity lane is active.
    Active,
    /// The host paused the lane under its policy.
    Paused,
    /// Unsupported capacity never grants suspend or resume.
    Unknown(String),
}

/// Capacity status; command receipts remain in the workspace operation journal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantCapacity {
    /// Host-accepted capacity state.
    pub state: CapacityState,
    /// Named policy/status explanation.
    pub reason: AssistantReason,
    /// Explicit capacity policy display.
    pub policy: String,
    /// Known next recovery, not a scheduled job created here.
    pub next_recovery: Option<String>,
}

/// Separately authorized full-inventory learning review, not an ordinary dashboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantLearningReview {
    /// Stable review projection identity.
    pub id: String,
    /// Accepted review revision.
    pub revision: u64,
    /// Independent operator access; denied content must not be mounted.
    pub access: AssistantAccess,
    /// Host-localized interval start.
    pub interval_start: String,
    /// Host-localized interval end.
    pub interval_end: String,
    /// Explicit inventory cutoff display.
    pub cutoff: String,
    /// Named reporting time zone.
    pub time_zone: String,
    /// Explicit late-arrival/catch-up projection.
    pub late_arrival_catch_up: String,
    /// Named inventory partitions and coverage gaps.
    pub partitions: Vec<AssistantCoverage>,
    /// Independent inventory/evaluation/review/publication stages.
    pub stages: Vec<AssistantStageCoverage>,
    /// Attributed assessment results.
    pub assessments: Vec<AssistantAssessment>,
    /// Improvements with release/verification/monitoring evidence.
    pub improvements: Vec<AssistantImprovement>,
    /// Actual configured capacity state and policy.
    pub capacity: AssistantCapacity,
    /// Explicit learning-review and suspend/resume grants.
    pub capabilities: AssistantCapabilities,
}
