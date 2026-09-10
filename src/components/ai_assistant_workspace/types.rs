//! Owned presentation values for the controlled assistant workspace.

use super::settings::AssistantSettings;
use super::{AssistantEvidence, AssistantProvenance};

/// Display-safe host explanation; the code is opaque, never provider diagnostics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssistantReason {
    /// Opaque reason identity for adapters, not user-facing copy.
    pub code: String,
    /// Sanitized, host-localized explanation.
    pub message: String,
}

/// The host's accepted access verdict, independent of whether data was loaded.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantAccess {
    /// The host has granted this particular access.
    Granted,
    /// The host explicitly refused access.
    Denied {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// The host has not established access yet.
    Unresolved {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// Unrecognized adapter vocabulary; never a grant.
    Unknown(String),
}

impl Default for AssistantAccess {
    fn default() -> Self {
        Self::Unresolved {
            reason: AssistantReason::default(),
        }
    }
}

/// Loading truth for an independently authorized section.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantLoad<T> {
    /// Section access is forbidden; content must not be mounted.
    Denied {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// The service or retention projection is unavailable, not empty.
    Unavailable {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// An explicitly requested host read is in progress.
    Loading,
    /// Accepted data; only a ready empty collection means genuinely empty.
    Ready(T),
    /// The section failed its structural boundary validation.
    ContractError(AssistantContractError),
}

impl<T> Default for AssistantLoad<T> {
    fn default() -> Self {
        Self::Unavailable {
            reason: AssistantReason::default(),
        }
    }
}

/// Eligibility is host evidence, not an inference from visible content.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantEligibility {
    /// The host revalidated this projection for its stamped context.
    Eligible,
    /// The projection is not authorized for an interaction.
    Ineligible {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// Unrecognized adapter vocabulary; never eligible.
    Unknown(String),
}

impl Default for AssistantEligibility {
    fn default() -> Self {
        Self::Ineligible {
            reason: AssistantReason::default(),
        }
    }
}

/// Application-owned continuity; a component does not choose an adoption mode.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConversationContinuity {
    /// Each admitted question stands alone.
    OneShot,
    /// Continuity lasts for the host session.
    Session,
    /// The host provides durable, authorized application history.
    Durable,
    /// An unsupported continuity mode cannot admit work.
    Unknown(String),
}

/// The sole question draft. Edits and admission remain controlled by the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantDraft {
    /// Stable draft identity.
    pub id: String,
    /// Context to which the host explicitly bound this draft.
    pub context_id: String,
    /// Accepted context revision at binding.
    pub context_revision: u64,
    /// Exact host-owned question text.
    pub text: String,
    /// Revision advanced by every accepted edit.
    pub revision: u64,
    /// Host question limit in Rust Unicode scalar values, not bytes or UTF-16.
    pub max_chars: usize,
}

/// Distinct pre-admission refusals, not failed or completed answers.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefusalReason {
    /// Evidence admission is unavailable; the question was not admitted.
    EvidenceAdmissionUnavailable,
    /// The required assistant preference is unavailable.
    AssistantPreferenceUnavailable,
    /// This continuity identity no longer accepts questions.
    ConversationClosed,
    /// An unknown refusal must not invite a blind retry.
    Unknown(String),
}

/// Safe navigation guidance for a known refusal, never an automatic operation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefusalNextAction {
    /// Wait for the host to re-establish evidence admission.
    RetryLater,
    /// Inspect assistant preferences.
    OpenSettings,
    /// Explicitly request another conversation.
    NewConversation,
}

impl RefusalReason {
    /// Returns distinct guidance; unknown refusals deliberately have none.
    pub fn next_action(&self) -> Option<RefusalNextAction> {
        match self {
            Self::EvidenceAdmissionUnavailable => Some(RefusalNextAction::RetryLater),
            Self::AssistantPreferenceUnavailable => Some(RefusalNextAction::OpenSettings),
            Self::ConversationClosed => Some(RefusalNextAction::NewConversation),
            Self::Unknown(_) => None,
        }
    }
}

/// The host's admission result, separate from the admitted attempt lifecycle.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SubmissionDisposition {
    /// No outstanding admission request.
    #[default]
    Idle,
    /// The host reserved this request before asynchronous admission.
    Submitting {
        /// Original allocated request identity.
        request_id: String,
    },
    /// Admission is uncertain; recover this request rather than ask again.
    Uncertain {
        /// Original request identity retained across recovery.
        request_id: String,
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// The host accepted an attempt; this does not mean it completed.
    Admitted {
        /// Exactly correlated admission request.
        request_id: String,
        /// Accepted attempt identity.
        attempt_id: String,
    },
    /// Admission was refused; retain the question and distinguish the reason.
    Refused {
        /// Exactly correlated admission request.
        request_id: String,
        /// Typed pre-admission refusal.
        reason: RefusalReason,
    },
    /// Unsupported admission state, never idle or retryable.
    Unknown(String),
}

/// The host owns duplicate suppression, ordering and terminal immutability.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryCause {
    /// The delivery channel disconnected.
    Reconnect,
    /// The host observed a sequence gap.
    SequenceGap,
    /// A duplicate sequence contained conflicting data.
    DuplicateConflict,
    /// A late result conflicted with accepted terminal state.
    TerminalConflict,
    /// The stream could not be interpreted safely.
    StreamError,
    /// Unknown recovery causes cannot make provisional output actionable.
    Unknown(String),
}

/// Transport continuity does not replace the accepted durable lifecycle.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantTransport {
    /// Delivery is healthy at the last accepted sequence.
    Connected {
        /// Last sequence accepted by the host adapter.
        last_sequence: u64,
    },
    /// Delivery disconnected; no implicit resubmission.
    Disconnected {
        /// Last sequence accepted by the host adapter.
        last_sequence: u64,
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// The host is rereading this original attempt.
    Recovering {
        /// Last sequence accepted before uncertainty.
        last_sequence: u64,
        /// Typed continuity problem, not provider diagnostics.
        cause: RecoveryCause,
    },
    /// The host cannot safely interpret the current projection.
    Unreadable {
        /// Last sequence known to have been accepted.
        last_sequence: u64,
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// Unsupported transport vocabulary, never connected.
    Unknown(String),
}

/// A healthy completion can answer or explicitly decline on grounding limits.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnswerOutcome {
    /// Grounded accepted narrative, rendered as escaped plain text.
    Answered {
        /// Exact display-safe accepted narrative.
        text: String,
    },
    /// Healthy engine execution declined to answer; not a provider failure.
    Declined {
        /// Nonempty, material grounding limitations.
        limitations: Vec<String>,
        /// Host-localized evidence freshness.
        as_of: String,
    },
    /// An unknown outcome cannot establish completion content.
    Unknown(String),
}

/// A completed, scope-bound answer with individually qualified facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantAnswer {
    /// Stable accepted answer identity.
    pub id: String,
    /// Accepted answer revision.
    pub revision: u64,
    /// Actual answered scope, which must equal the attempt's accepted scope.
    pub answered_scope: AssistantScope,
    /// Host-localized evidence freshness.
    pub as_of: String,
    /// Answered or deliberately declined, never an error bubble.
    pub outcome: AnswerOutcome,
    /// Validated facts with per-fact qualifications and availability.
    pub facts: Vec<AssistantFact>,
    /// Exact source identities supplied by the host.
    pub evidence: Vec<AssistantEvidence>,
    /// Actual origin rather than the user's selected preference.
    pub provenance: AssistantProvenance,
    /// Material limitations kept beside the claims.
    pub limitations: Vec<String>,
}

/// Durable attempt lifecycle. Transport delivery is independently projected.
#[non_exhaustive]
// The lifecycle is intentionally value-shaped so callers can pattern-match
// accepted answers without an allocation or pointer-specific API change.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttemptLifecycle {
    /// Admission was acknowledged; execution has not been queued yet.
    Admitted,
    /// Accepted work is queued.
    Queued,
    /// Execution is in progress; any preview remains provisional.
    Running,
    /// Execution output is undergoing host validation.
    Validating,
    /// Only this variant carries accepted answer content.
    Completed(AssistantAnswer),
    /// The admitted operation was denied.
    Denied {
        /// Sanitized explanation, not answer text.
        reason: AssistantReason,
    },
    /// Execution is unavailable.
    Unavailable {
        /// Sanitized explanation, not answer text.
        reason: AssistantReason,
    },
    /// Execution failed; never treated as a healthy decline.
    Failed {
        /// Sanitized explanation, not answer text.
        reason: AssistantReason,
    },
    /// Terminal cancellation, including results discarded after cancellation.
    Canceled {
        /// The host discarded output; false denotes ordinary cancellation.
        discarded: bool,
    },
    /// Execution was interrupted and requires a host recovery decision.
    Interrupted {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// Unsupported lifecycle vocabulary, never a completed answer.
    Unknown(String),
}

/// One admitted host request, preserving its original context and provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantAttempt {
    /// Stable accepted attempt identity.
    pub id: String,
    /// Accepted projection revision.
    pub revision: u64,
    /// Original admission request identity, never minted on reconnect.
    pub request_id: String,
    /// Exact accepted question.
    pub question: String,
    /// Context accepted at admission, not silently relabeled on navigation.
    pub accepted_context: AssistantContext,
    /// Durable host-owned execution state.
    pub lifecycle: AttemptLifecycle,
    /// Last accepted delivery sequence and any continuity uncertainty.
    pub transport: AssistantTransport,
    /// Provisional display text; never an insertion or copy source.
    pub preview: Option<String>,
    /// Host-acknowledged cancel request, independent of terminal cancellation.
    pub cancel_requested: bool,
    /// Current host access to retained content.
    pub eligibility: AssistantEligibility,
    /// Requested preference at admission, not actual execution provenance.
    pub requested_engine: Option<String>,
}

/// Accepted conversation and its one canonical controlled composer draft.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantConversation {
    /// Stable application continuity identity.
    pub id: String,
    /// Accepted conversation revision.
    pub revision: u64,
    /// Current bound context identity.
    pub context_id: String,
    /// Current bound scope identity.
    pub scope_id: String,
    /// Closed conversations cannot admit new work.
    pub closed: bool,
    /// Explicit host-chosen adoption mode.
    pub continuity: ConversationContinuity,
    /// Conversation-level grants, required in addition to page grants.
    pub capabilities: AssistantCapabilities,
    /// Sole host-owned draft, preserved through refusals and uncertainty.
    pub draft: AssistantDraft,
    /// Host suggestions edit the draft; choosing one never asks automatically.
    pub suggestions: Vec<String>,
    /// Request admission state, independently correlated to an attempt.
    pub submission: SubmissionDisposition,
    /// Accepted historical and current attempts.
    pub attempts: Vec<AssistantAttempt>,
}

/// Explicit host grants. A component never derives grants from labels or data.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantCapability {
    /// Submit an explicit question.
    Ask,
    /// Request cancellation of admitted work.
    Cancel,
    /// Record an eligible feedback outcome.
    Feedback,
    /// Request new application continuity without asking a question.
    NewConversation,
    /// Load authorized application history.
    History,
    /// Open an opaque evidence identity through the host.
    Evidence,
    /// Copy only permitted visible answer content.
    Copy,
    /// Explicitly prepare an artifact.
    Prepare,
    /// Edit an artifact's host-local draft.
    EditArtifact,
    /// Persist an artifact edit.
    SaveArtifact,
    /// Request an artifact reset.
    ResetArtifact,
    /// Request a separately acknowledged artifact regeneration.
    Regenerate,
    /// Request a rewrite.
    Rewrite,
    /// Request translation into an explicit language.
    Translate,
    /// Review an exact handoff proposal revision.
    ReviewProposal,
    /// Insert a reviewed suggestion, never send it.
    Insert,
    /// Open a reviewed owner workflow.
    OpenWorkflow,
    /// Propose settings edits.
    Settings,
    /// Persist the exact proposed settings revision.
    SaveSettings,
    /// Request a host-owned sign-in flow.
    SignIn,
    /// Request sign-out, independently of governed memory retention.
    SignOut,
    /// Create an actor-owned memory candidate.
    MemoryCreate,
    /// Confirm an exact memory candidate.
    MemoryConfirm,
    /// Correct an owned memory revision.
    MemoryCorrect,
    /// Withdraw a memory from assistance under the host's retention policy.
    MemoryForget,
    /// Request an authorized memory export.
    MemoryExport,
    /// Submit Group Important content for review.
    KnowledgeSubmit,
    /// Perform a curator review, independently of publication.
    KnowledgeReview,
    /// Publish a host-approved exact candidate revision.
    KnowledgePublish,
    /// Withdraw published knowledge.
    KnowledgeWithdraw,
    /// Reject a candidate in an authorized curator workflow.
    KnowledgeReject,
    /// Request changes to a candidate.
    KnowledgeRequestChanges,
    /// View separately permissioned operational learning review.
    LearningReview,
    /// Request suspension, never execute jobs directly.
    Suspend,
    /// Request resumption of learning capacity.
    Resume,
    /// Unknown grants are never honored.
    Unknown(String),
}

/// A reason attached to one grant; details may only restrict it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantCapabilityState {
    /// Does not restrict an existing grant; cannot create one.
    Granted,
    /// The host refuses this operation.
    Denied {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// Required service support is unavailable.
    Unavailable {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// A required owner policy has not been configured.
    PolicyNotConfigured {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// An incompatible operation is in progress.
    Busy {
        /// Sanitized explanation.
        reason: AssistantReason,
    },
    /// An unknown restriction is never treated as a grant.
    Unknown(String),
}

/// One capability's optional restriction in the host projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantCapabilityDetail {
    /// The capability being explained.
    pub capability: AssistantCapability,
    /// Restriction or corroborating grant evidence.
    pub state: AssistantCapabilityState,
}

/// Explicit grants plus optional restrictive details; defaults grant nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssistantCapabilities {
    /// Capabilities explicitly granted by this scope.
    pub granted: Vec<AssistantCapability>,
    /// At most one detail per capability; duplicates are invalid.
    pub details: Vec<AssistantCapabilityDetail>,
}

impl AssistantCapabilities {
    /// Checks for duplicate or unknown capability vocabulary.
    pub fn validate(&self) -> Result<(), AssistantContractError> {
        for (index, capability) in self.granted.iter().enumerate() {
            if matches!(capability, AssistantCapability::Unknown(_))
                || self.granted[..index].contains(capability)
            {
                return Err(AssistantContractError::UnknownState);
            }
        }
        for (index, detail) in self.details.iter().enumerate() {
            if matches!(detail.capability, AssistantCapability::Unknown(_))
                || matches!(detail.state, AssistantCapabilityState::Unknown(_))
                || self.details[..index]
                    .iter()
                    .any(|prior| prior.capability == detail.capability)
            {
                return Err(AssistantContractError::UnknownState);
            }
        }
        Ok(())
    }

    /// Requires an explicit grant and no restrictive detail in a valid scope.
    pub fn allows(&self, capability: &AssistantCapability) -> bool {
        self.validate().is_ok()
            && self.granted.contains(capability)
            && self
                .details
                .iter()
                .filter(|detail| &detail.capability == capability)
                .all(|detail| matches!(detail.state, AssistantCapabilityState::Granted))
    }
}

/// Authenticated actor or viewed subject. These roles must never be conflated.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssistantPerson {
    /// Opaque host identity.
    pub id: String,
    /// Host-localized display label.
    pub label: String,
}

/// The host-admitted population; prompt text cannot widen it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeBasis {
    /// The current page's complete visible population.
    PagePopulation {
        /// Opaque population identity.
        population_id: String,
        /// Accepted population revision.
        population_revision: u64,
        /// Host-localized description of this population.
        description: String,
    },
    /// An independently authorized expansion beyond the current page.
    Expanded {
        /// Explicit authorization identity, not inferred from the question.
        authorization_id: String,
        /// Accepted authorization revision.
        authorization_revision: u64,
        /// Host-localized explanation of the expanded population.
        description: String,
    },
    /// Unknown scope is not actionable.
    Unknown(String),
}

/// A scope's stable identity, version and visible meaning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantScope {
    /// Opaque scope identity.
    pub id: String,
    /// Host-accepted scope revision.
    pub revision: u64,
    /// Visible scope label, including any expansion.
    pub label: String,
    /// Explicit population and authorization basis.
    pub basis: ScopeBasis,
}

/// Host-accepted context displayed by the workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantContext {
    /// Opaque context identity.
    pub id: String,
    /// Context revision; zero is a valid host revision.
    pub revision: u64,
    /// Replacement epoch, even if the context identity is reused.
    pub epoch: u64,
    /// Authenticated operator and memory owner.
    pub actor: AssistantPerson,
    /// Viewed subject, never implicitly the operator or memory owner.
    pub subject: Option<AssistantPerson>,
    /// Current authorized population.
    pub scope: AssistantScope,
    /// Accepted source data generation.
    pub data_revision: u64,
    /// Accepted permission generation.
    pub permission_revision: u64,
    /// Host-localized as-of display, not parsed for authority.
    pub as_of: String,
    /// Host-localized freshness explanation, not an authorization grant.
    pub freshness: String,
    /// Accepted owner-policy generation.
    pub policy_generation: u64,
    /// Page-level grants, further restricted by conversation/item scopes.
    pub capabilities: AssistantCapabilities,
}

/// Captured authority identities attached to every interaction, including edits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantContextStamp {
    /// Context identity.
    pub context_id: String,
    /// Expected context revision.
    pub context_revision: u64,
    /// Expected replacement epoch.
    pub epoch: u64,
    /// Expected authenticated operator.
    pub actor_id: String,
    /// Expected population identity.
    pub scope_id: String,
    /// Expected population scope revision.
    pub scope_revision: u64,
    /// Expected data generation.
    pub data_revision: u64,
    /// Expected permission generation.
    pub permission_revision: u64,
    /// Expected policy generation.
    pub policy_generation: u64,
}

/// The bounded interaction kinds understood by the workspace.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantActionKind {
    /// Submit the conversation's current controlled draft.
    Ask,
    /// Unknown actions are never dispatchable.
    Unknown(String),
}

/// The only destination currently supported by the assistant composer.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantIntentTarget {
    /// A specific conversation and draft revision.
    Conversation {
        /// Conversation identity and expected revision.
        conversation: AssistantItemRevision,
        /// Draft identity and expected revision.
        draft: AssistantItemRevision,
    },
    /// Unknown destinations are never dispatchable.
    Unknown(String),
}

/// An explicit user intent carrying the authority snapshot it was created from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantAction {
    /// Captured context and authority generations.
    pub context: AssistantContextStamp,
    /// Exact destination identities.
    pub target: AssistantIntentTarget,
    /// Requested bounded operation.
    pub kind: AssistantActionKind,
}

/// Host allocation and identity context attached to an admitted command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantRequest {
    /// Host-allocated request identity.
    pub id: String,
    /// Authority snapshot copied from the action.
    pub context: AssistantContextStamp,
    /// Exact destination identities.
    pub target: AssistantIntentTarget,
}

/// The typed payload sent for a validated assistant command.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantCommandPayload {
    /// Submit the exact controlled question and accepted scope.
    Ask {
        /// Exact draft text at admission.
        question: String,
        /// Accepted selected preference, never execution provenance.
        selected_engine_id: String,
        /// Exact accepted scope snapshot.
        scope: AssistantScope,
    },
    /// Unknown payloads are never accepted.
    Unknown(String),
}

/// A host-owned command proposal; creating it mutates no workspace state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantCommand {
    /// Original explicit intent.
    pub action: AssistantAction,
    /// Present only when the host allocated a request identity.
    pub request: Option<AssistantRequest>,
    /// Typed bounded payload.
    pub payload: AssistantCommandPayload,
}

/// Complete controlled workspace projection, with independently loaded services.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssistantWorkspaceState {
    /// Page-level access verdict.
    pub access: AssistantAccess,
    /// Current page context, when accepted by the host.
    pub context: Option<AssistantContext>,
    /// Next host request allocation.
    pub next_request_id: Option<String>,
    /// Conversation projection.
    pub conversation: AssistantLoad<AssistantConversation>,
    /// Accepted/proposed settings projection.
    pub settings: AssistantLoad<AssistantSettings>,
    /// Optional actor-owned memory projection.
    pub memory: AssistantLoad<()>,
    /// Optional learning projection.
    pub learning: AssistantLoad<()>,
    /// Host clock used for time-sensitive projections.
    pub now_ms: u64,
}

impl AssistantContext {
    /// Captures identities only; obtaining a stamp does not grant an action.
    pub fn stamp(&self) -> AssistantContextStamp {
        AssistantContextStamp {
            context_id: self.id.clone(),
            context_revision: self.revision,
            epoch: self.epoch,
            actor_id: self.actor.id.clone(),
            scope_id: self.scope.id.clone(),
            scope_revision: self.scope.revision,
            data_revision: self.data_revision,
            permission_revision: self.permission_revision,
            policy_generation: self.policy_generation,
        }
    }
}

/// A versioned item identity, preserved in requests and their receipts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantItemRevision {
    /// Opaque, nonblank item identity.
    pub id: String,
    /// Accepted revision expected by the interaction.
    pub revision: u64,
}

/// A structural boundary error, never raw provider diagnostics or private text.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssistantContractError {
    /// A required identity is blank or duplicated.
    MissingIdentity,
    /// A fact lacks its required qualification or availability invariant.
    InvalidFact,
    /// A comparison lacks its display or comparison basis.
    InvalidComparison,
    /// A completed answer has no valid content or limitations.
    InvalidAnswer,
    /// Answered scope differs from the scope admitted by the host.
    ScopeMismatch,
    /// A connection projection is incomplete or contradictory.
    InvalidConnection,
    /// A receipt does not match its request and accepted projection.
    InvalidReceipt,
    /// An unknown host vocabulary cannot be interpreted safely.
    UnknownState,
}

/// A comparison whose displayed value always travels with its basis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantComparison {
    display: String,
    basis: String,
}

impl AssistantComparison {
    /// Validates nonblank display and basis without rewriting host content.
    pub fn new(
        display: impl Into<String>,
        basis: impl Into<String>,
    ) -> Result<Self, AssistantContractError> {
        let display = display.into();
        let basis = basis.into();
        if display.trim().is_empty() || basis.trim().is_empty() {
            return Err(AssistantContractError::InvalidComparison);
        }
        Ok(Self { display, basis })
    }

    /// Host-formatted comparison value.
    pub fn display(&self) -> &str {
        &self.display
    }

    /// The period, denominator or other basis qualifying the comparison.
    pub fn basis(&self) -> &str {
        &self.basis
    }
}

/// One qualified fact; unavailable is distinct from a present value of zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantFact {
    label: String,
    current: Option<String>,
    baseline: Option<AssistantComparison>,
    qualification: String,
    availability: Option<String>,
}

impl AssistantFact {
    /// Requires a qualification and availability exactly when current is absent.
    pub fn new(
        label: impl Into<String>,
        current: Option<String>,
        baseline: Option<AssistantComparison>,
        qualification: impl Into<String>,
        availability: Option<String>,
    ) -> Result<Self, AssistantContractError> {
        let label = label.into();
        let qualification = qualification.into();
        if label.trim().is_empty()
            || qualification.trim().is_empty()
            || current
                .as_ref()
                .is_some_and(|value| value.trim().is_empty())
            || availability
                .as_ref()
                .is_some_and(|value| value.trim().is_empty())
            || current.is_none() != availability.is_some()
        {
            return Err(AssistantContractError::InvalidFact);
        }
        Ok(Self {
            label,
            current,
            baseline,
            qualification,
            availability,
        })
    }

    /// Visible fact name.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Host-formatted current value; absence is not zero.
    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// Optional, independently qualified comparison.
    pub fn baseline(&self) -> Option<&AssistantComparison> {
        self.baseline.as_ref()
    }

    /// Required material caveat displayed beside the value.
    pub fn qualification(&self) -> &str {
        &self.qualification
    }

    /// Per-fact explanation, present exactly when current is absent.
    pub fn availability(&self) -> Option<&str> {
        self.availability.as_deref()
    }
}
