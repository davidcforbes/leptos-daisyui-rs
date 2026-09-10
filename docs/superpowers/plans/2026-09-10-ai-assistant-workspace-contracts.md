# AiAssistantWorkspace implementation contracts

This is the typed-interface supplement to the approved design and implementation
plan, not a second execution ledger. Beads records implementation progress.
The design spec wins if an inconsistency is found. Names below are Rust names;
all strings are owned, display-safe host projections, not transport DTOs.

## Core types and ownership

All evolving enums are `#[non_exhaustive]` and include `Unknown(String)` when
they represent a host vocabulary. Unknown values cannot grant an action.
`Revision` is `u64`; `Id` is an owned nonblank string, validated at the affected
boundary. Timestamps used for guards are Unix milliseconds supplied by the host;
human date/number displays are separate host-localized strings. The component
does not interpret a localized date as authority or run its own policy clock.

| Type | Fields / variants |
|---|---|
| `AssistantWorkspaceState` | `context: Option<AssistantContext>`, `access: AssistantAccess`, `now_ms: u64`, `next_request_id: Option<String>`, `conversation: AssistantLoad<AssistantConversation>`, `artifacts: AssistantLoad<Vec<AssistantArtifact>>`, `settings: AssistantLoad<AssistantSettings>`, `memory: AssistantLoad<AssistantMemory>`, `knowledge: AssistantLoad<AssistantKnowledge>`, `history: AssistantLoad<AssistantHistory>`, `activity: AssistantLoad<AssistantActivity>`, `learning: AssistantLoad<AssistantLearningReview>`, `operations: Vec<AssistantOperation>` |
| `AssistantAccess` | `Granted`, `Denied { reason }`, `Unresolved { reason }`, `Unknown(code)`; default Unresolved |
| `AssistantLoad<T>` | `Denied { reason }`, `Unavailable { reason }`, `Loading`, `Ready(T)`, `ContractError(AssistantContractError)`; empty is a Ready empty collection, never the default |
| `AssistantContext` | `id`, `revision`, `epoch`, `actor: AssistantPerson`, `subject: Option<AssistantPerson>`, `scope: AssistantScope`, `data_revision`, `permission_revision`, `as_of`, `freshness`, `policy_generation`, `capabilities: AssistantCapabilities` |
| `AssistantPerson` | `id`, `label`; actor is authenticated operator, subject is the viewed person and never the memory owner |
| `AssistantScope` | `id`, `revision`, `label`, `basis: ScopeBasis`; `ScopeBasis` is `PagePopulation { population_id, population_revision, description }`, `Expanded { authorization_id, authorization_revision, description }`, or `Unknown(code)` |
| `AssistantCapability` | `Ask`, `Cancel`, `Feedback`, `NewConversation`, `History`, `Evidence`, `Copy`, `Prepare`, `EditArtifact`, `SaveArtifact`, `ResetArtifact`, `Regenerate`, `Rewrite`, `Translate`, `ReviewProposal`, `Insert`, `OpenWorkflow`, `Settings`, `SaveSettings`, `SignIn`, `SignOut`, `MemoryCreate`, `MemoryConfirm`, `MemoryCorrect`, `MemoryForget`, `MemoryExport`, `KnowledgeSubmit`, `KnowledgeReview`, `KnowledgePublish`, `KnowledgeWithdraw`, `KnowledgeReject`, `KnowledgeRequestChanges`, `LearningReview`, `Suspend`, `Resume`, `Unknown(code)` |
| `AssistantConversation` | `id`, `revision`, `context_id`, `scope_id`, `closed`, `continuity: ConversationContinuity`, `capabilities: AssistantCapabilities`, `draft: AssistantDraft`, `suggestions: Vec<String>`, `submission: SubmissionDisposition`, `attempts: Vec<AssistantAttempt>` |
| `ConversationContinuity` | `OneShot`, `Session`, `Durable`, `Unknown(code)`; host chooses migration mode |
| `AssistantDraft` | `id`, `context_id`, `context_revision`, `text`, `revision`, `max_chars`; no second component-owned text copy |
| `SubmissionDisposition` | `Idle`, `Submitting { request_id }`, `Uncertain { request_id, reason }`, `Admitted { request_id, attempt_id }`, `Refused { request_id, reason: RefusalReason }`, `Unknown(code)` |
| `RefusalReason` | `EvidenceAdmissionUnavailable`, `AssistantPreferenceUnavailable`, `ConversationClosed`, `Unknown(code)`; `next_action()` derives respectively `RetryLater`, `OpenSettings`, `NewConversation`; unknown returns no next action |
| `AssistantAttempt` | `id`, `revision`, `request_id`, `question`, `accepted_context: AssistantContext`, `lifecycle`, `transport`, `preview: Option<String>`, `cancel_requested`, `eligibility: AssistantEligibility`, `requested_engine: Option<String>` |
| `AttemptLifecycle` | `Admitted`, `Queued`, `Running`, `Validating`, `Completed(AssistantAnswer)`, `Denied { reason }`, `Unavailable { reason }`, `Failed { reason }`, `Canceled { discarded: bool }`, `Interrupted { reason }`, `Unknown(code)` |
| `AssistantTransport` | `Connected { last_sequence }`, `Disconnected { last_sequence, reason }`, `Recovering { last_sequence, cause: RecoveryCause }`, `Unreadable { last_sequence, reason }`, `Unknown(code)`; `RecoveryCause` is `Reconnect`, `SequenceGap`, `DuplicateConflict`, `TerminalConflict`, `StreamError`, or `Unknown(code)` |
| `AssistantAnswer` | `id`, `revision`, `answered_scope`, `as_of`, `outcome: AnswerOutcome`, `facts: Vec<AssistantFact>`, `evidence: Vec<AssistantEvidence>`, `provenance: AssistantProvenance`, `limitations: Vec<String>` |
| `AnswerOutcome` | `Answered { text }`, `Declined { limitations: Vec<String>, as_of }`, `Unknown(code)`; validation requires nonblank text or nonempty nonblank decline limitations |
| `AssistantFact` | Private `label`, `current: Option<String>`, `baseline: Option<AssistantComparison>`, `qualification`, `availability: Option<String>`; constructor and getters only |
| `AssistantComparison` | Private `display`, `basis`; nonblank validated constructor and getters |
| `AssistantContractError` | `MissingIdentity`, `InvalidFact`, `InvalidComparison`, `InvalidAnswer`, `ScopeMismatch`, `InvalidConnection`, `InvalidReceipt`, `UnknownState`, each carrying only sanitized boundary/identity metadata where useful, never raw prompts or provider errors |

Every `capabilities` field in this supplement has type `AssistantCapabilities`,
including engine, settings, artifact, memory/item, knowledge/item, history/item
and learning scopes. It stores `granted: Vec<AssistantCapability>` and
`details: Vec<AssistantCapabilityDetail>`. A detail contains capability and
Granted/Denied/Unavailable/PolicyNotConfigured/Busy/Unknown with named reasons.
Details can only restrict, never add a grant; absence from granted always denies.
All applicable scopes must grant and no detail may restrict. Duplicate/conflicting
details are a local contract error. Context/reasoning-tier restrictions outrank
engine/item details. `AssistantEligibility` is Eligible/Ineligible { reason }/
Unknown(code), with default Ineligible; no boolean-plus-optional-reason pair.

Core validation checks duplicate IDs, nonblank identities, fact/answer shape,
attempt-to-answer scope and contradictory terminal payloads. Optional collection
errors stay in that collection's `ContractError` state; unavailable memory or
learning does not disable otherwise authorized ordinary conversation work.
`validate_workspace` validates only core context/conversation. Separate
`validate_settings`, `validate_artifact`, `validate_memory`, `validate_knowledge`
and `validate_learning` return errors for their own sections. Every dispatcher
uses only relevant validation; a bad optional service cannot block Ask. Each
view renders its own error rather than hiding the entire facade.
Access/context resolution and current generations independently gate actions.
Content denied or invalidated by the host is not mounted, including previews,
artifact bodies and retained evidence. Permitted disposition metadata may remain.

Scope is authoritative host data, not inferred from question text. "These rows"
uses PagePopulation; Expanded requires its own authorization identity/revision
and visible label. New context/epoch, data revision, permission revision or policy
generation invalidates old action eligibility until the host supplies a newly
validated artifact/proposal. Historical attempts retain their accepted scope and
can be read only while eligible, but are not current authority. Actor changes
withhold old actor memory; merely changing the viewed subject does not move or
relabel actor memory. Memory carries `owner_actor_id` and validation compares it
to the authenticated actor, never the subject.
Composer context/revision must match before Ask. After a context change the host
must explicitly rebind the draft following scope review; merely rendering a new
header cannot make the old question actionable. The component never silently
clears a draft; the host may clear/archive only the draft revision named by a
correlated admission. A newer edit survives an earlier admission receipt.

No local lifecycle reducer consumes provider events. The host adapter owns
sequence acceptance, duplicate suppression and terminal immutability. It supplies
the last accepted sequence and a recovery cause when continuity is uncertain;
actions from provisional/recovery/conflicting output stay locked. An attempt
marked canceled remains terminal; a late result is either host-rejected or a
TerminalConflict projection, never a new completed answer. Fake-host tests must
exercise this contract rather than asking the component to parse raw events.

## Request reservation, commands and receipts

The host allocates `next_request_id` before advertising an asynchronous action.
It is a fresh opaque ID, never generated from client content, array position or
wall-clock display. On receiving a command, the host **synchronously** consumes
that ID and publishes a pending operation before starting asynchronous work.
This reservation is part of the controlled-host contract, just like adopting a
draft edit. An adapter which cannot reserve synchronously must leave the action
unavailable; the component does not create an optimistic completion latch.

`AssistantAction { context: AssistantContextStamp, target: AssistantIntentTarget,
kind: AssistantActionKind }` always carries the identity captured by the rendered
control, including synchronous edits. The dispatcher rejects a stale stamp or
wrong target/kind combination before constructing any command. A bare Ask below
means only the kind, never an identity-free callback.

`AssistantContextStamp` contains context ID/revision/epoch, actor ID, scope
ID/revision, data revision, permission revision and policy generation.
`AssistantItemRevision { id, revision }` is a nonblank item identity.
`AssistantIntentTarget` has these closed variants:

- Conversation { conversation: AssistantItemRevision, draft: AssistantItemRevision }.
- Attempt { conversation, attempt: AssistantItemRevision }.
- Answer { conversation, attempt, answer: AssistantItemRevision }.
- Evidence { answer: AssistantItemRevision, evidence: AssistantItemRevision }.
- Artifact { artifact: AssistantItemRevision, draft_revision: Option<u64> }.
- Proposal { proposal: AssistantItemRevision, source: AssistantItemRevision,
  target: ProposalTarget }.
- Settings { owner_actor_id, accepted_revision, proposed_revision }.
- Engine { engine: AssistantItemRevision }.
- Memory { owner_actor_id, collection_revision, item: Option<AssistantItemRevision>,
  draft_revision: Option<u64> }.
- Knowledge { collection_revision, item: Option<AssistantItemRevision>,
  draft_revision: Option<u64> }.
- History { entry: AssistantItemRevision }; Learning { review: AssistantItemRevision };
  Unknown(code). In the grouped declarations, unannotated item fields use
  AssistantItemRevision too. Every optional field has a kind-specific requirement.

`AssistantRequest { id, context: AssistantContextStamp, target:
AssistantIntentTarget }` preserves that complete identity. All async commands
require it; synchronous host-local edits use None. The emitted
`AssistantWorkspaceCommand { request: Option<AssistantRequest>, action:
AssistantAction, payload: AssistantCommandPayload }` uses a closed payload:
Ask { question, selected_engine_id, scope: AssistantScope },
SaveSettings { proposed: AssistantPreferences },
Insert { proposal: AssistantProposal }, OpenWorkflow { proposal: AssistantProposal },
Copy { visible_text }, or None for actions whose complete data is already in
the typed action. Thus Ask contains exact conversation/draft IDs/revisions and
question; settings contains both form/accepted revisions; insertion contains
source/proposal/target identities and revisions, expiry, policy and reviewed
content. No arbitrary JSON, string operation name or out-of-band callback data.
`accepts_command(state, command)` independently revalidates public commands.
Request context/target must equal action context/target exactly; Ask payload
scope must equal the stamped current scope; Insert/OpenWorkflow payload must
equal the currently validated proposal including target/source/reviewed content.
Settings/Copy payloads must likewise equal the eligible current projection.
Mismatches are rejected, never resolved by choosing one copy as more authoritative.
This protects hand-built commands as well as commands returned by command_for.

`AssistantOperation { request, kind: AssistantOperationKind, state }` is host-owned.
`OperationState` is `Pending`, `Accepted`, `Refused { reason }`, `Failed { reason }`,
`Uncertain { reason }`, `ConfirmedState { resulting_revision, receipt }`, or
`AcknowledgedEffect { receipt }`. ConfirmedState means an independently accepted
host projection/read-back, not a write echo. It can establish a versioned read
(such as recovered history) as well as a mutation without claiming that the read
performed a write. AcknowledgedEffect is Copied/Opened/ExportReady and means the
corresponding host platform API resolved successfully; it claims no persisted
record revision or independent clipboard read-back. Callback invocation alone
establishes neither.

`AssistantOperationKind` is the asynchronous subset of AssistantActionKind.
Only the corresponding closed outcome below is accepted; every other pairing is
InvalidReceipt. Local edits have no operation outcome. No generic Saved receipt
stands for unrelated operations:

| Operation kind | Required typed receipt outcome / authority |
|---|---|
| Ask | AttemptAdmitted; exact SubmissionDisposition admission and admitted attempt revision, never Completed |
| NewConversation | ConversationCreated; accepted new continuity identity/revision, no paid Ask |
| Cancel | CancellationRequested; accepted cancel-request projection only, terminal cancellation remains lifecycle-owned |
| RecoverSubmission / RecoverAttempt / LoadHistory | SubmissionResolved / AttemptRecovered / HistoryLoaded; exact re-read projection, never a new attempt |
| OpenEvidence / OpenVerification / OpenProposalWorkflow | Opened effect acknowledgment; typed target retained, no write receipt |
| CopyAnswer / ExportMemory | Copied / ExportReady effect acknowledgment; only permitted payload |
| Feedback | FeedbackRecorded |
| PrepareArtifact | ArtifactPrepared |
| SaveArtifact / ResetArtifact / RegenerateArtifact / RewriteArtifact / TranslateArtifact | ArtifactSaved / ArtifactReset / ArtifactRegenerated / ArtifactRewritten / ArtifactTranslated; independent operation keys |
| ReviewProposal / InsertProposal / RejectProposal | ProposalReviewed / Inserted / ProposalRejected |
| SaveSettings | SettingsSaved; exact accepted and proposed revisions |
| SignIn / SignOut | SignedIn / SignedOut; actual accepted connection state, not merely a launched flow |
| CreateMemory / ConfirmMemory / CorrectMemory / ForgetMemory | MemoryCreated / MemoryConfirmed / MemoryCorrected / MemoryWithdrawn; withdrawal includes retention facts |
| SubmitKnowledge / ReviewKnowledge / PublishKnowledge / WithdrawKnowledge / RejectKnowledge / RequestKnowledgeChanges | KnowledgeSubmittedForReview / KnowledgeReviewed / KnowledgePublished / KnowledgeWithdrawn / KnowledgeRejected / KnowledgeChangesRequested |
| SuspendLearning / ResumeLearning | LearningSuspended / LearningResumed |

All remaining outcomes use ConfirmedState with the actual accepted target
revision. Unknown operation/outcome codes never confirm anything. Sign-in pending
flows, regeneration progress and cancellation requests remain separate from their
eventual terminal results; each renders only its acknowledged stage.
Creation/admission outcomes additionally carry the newly accepted item identity:
AttemptAdmitted has attempt ID, ConversationCreated has conversation ID,
ArtifactPrepared has artifact ID, MemoryCreated has memory ID and
KnowledgeSubmittedForReview has candidate ID. Their resulting revision refers to
that new item; existing-item mutations refer to the requested item, and insertion
to the destination draft. Original request identity remains unchanged for matching.

Only the exact request/context/target and kind may establish a receipt. For a
durable mutation, resulting_revision must equal the current accepted target
revision; original expected revisions identify which proposal was submitted and
may not accept or clear a newer form revision. For an ephemeral effect the source
revision must still match. Insertion compares resulting destination revision, not
proposal revision, and never claims a sent message. Pending/Accepted/Uncertain lock the same operation; an uncertain
submission also locks Ask. `RecoverSubmission { request_id }` and
`RecoverAttempt { attempt_id, expected_revision }` address the original request,
never Ask. A new attempt always requires a fresh explicit Ask gesture after the
host has settled the original request.

`AssistantActionKind` has named variants, not string operation codes. The exact
context and typed target above are mandatory for every listed kind:

- `EditDraft { text, expected_revision }`, `UseSuggestion { text, expected_revision }`,
  `Ask`, `NewConversation`, `RecoverSubmission { request_id }`,
  `Cancel { attempt_id, expected_revision }`, `RecoverAttempt { attempt_id, expected_revision }`.
- `OpenEvidence { answer_id, evidence_id, expected_revision }`,
  `CopyAnswer { answer_id, expected_revision }`,
  `Feedback { attempt_id, expected_revision, outcome: FeedbackOutcome }`.
  FeedbackOutcome is Inserted/Edited/Discarded/Flagged/Unknown; Declined accepts
  only Discarded/Flagged. Feedback is not a memory or publication action.
- `PrepareArtifact`, `EditArtifact`, `SaveArtifact`, `ResetArtifact`, `RegenerateArtifact`,
  `RewriteArtifact`, `TranslateArtifact` with artifact ID/revision and typed
  text/language payload where applicable; independent operation kinds.
- `ReviewProposal`, `InsertProposal`, `RejectProposal`, `OpenProposalWorkflow`
  with proposal ID/revision; Insert/Open additionally expose exact target fields
  in the emitted command, never only an unscoped button event.
- `EditSettings { expected_revision, proposed }`, `SaveSettings { expected_revision }`,
  `SignIn`, `OpenVerification`, `SignOut` with engine ID/revision. SignOut carries
  an explicitly named worker-cleanup choice, not a governed-memory-forget flag.
- `EditMemoryDraft`, `CreateMemory`, `ConfirmMemory`, `CorrectMemory`, `ForgetMemory`,
  `ExportMemory`; edits carry class/body and revision, item actions carry ID/revision.
- `EditKnowledgeDraft`, `SubmitKnowledge`, `ReviewKnowledge`, `PublishKnowledge`, `WithdrawKnowledge`,
  `RejectKnowledge`, `RequestKnowledgeChanges`; exact candidate revision and
  reviewable body/source comparison are required for curator actions.
- `LoadHistory { id, expected_revision }`, `SuspendLearning`, `ResumeLearning`
  with exact item/revision. No close/search/pagination action without a future
  typed host contract. `Unknown(code)` dispatches nothing.

`can_dispatch` checks the same pure rules as `command_for`; the latter constructs
the full command from current accepted state. Render-time disabled state is not
the only guard. Item callbacks retain their rendered item revision and are
rejected after replacement. Ask/Cancel/Feedback additionally require the current
conversation's capability, never only a global page capability. Ask requires
resolved context, open conversation, nonblank in-bound draft, no live attempt or
uncertain submission, eligible selected engine and fresh request allocation.

## Artifact and evidence contracts

| Type | Fields / variants |
|---|---|
| `AssistantEvidence` | `id`, `label`, `source_kind`, `revision`, `as_of`, `availability`, `qualification`; host-open by opaque identity, no model URL navigation |
| `AssistantProvenance` | `mode: Generated / CacheReuse / HumanEdited / Unknown`, `actual_engine: Option<String>`, `model_version: Option<String>`, `prompt_version: Option<String>`, `skill_version: Option<String>`, `memory_revisions`, `foundation_revisions`, `supplied_sources`, `omissions`, `built_at`; absence stays unknown |
| `AssistantArtifact` | `id`, `revision`, `context: AssistantContextStamp`, `eligibility: AssistantEligibility`, `kind`, `provenance`, `evidence`, `draft: Option<AssistantArtifactDraft>`, `proposal: Option<AssistantProposal>`, `capabilities`; read-only Analysis/Briefing have no draft |
| `ArtifactKind` | `Analysis { objections, talking_points, next_action, limitations }`, `ClientDraft { recipient, channel, language, language_confirmed, intent, subject }`, `PreparedTemplate { template_id, template_revision, expires_at_ms }`, `CallPreparation(AssistantCallPreparation)`, `Briefing { facts, completeness }`, `WorkflowProposal { target, kind }`, `Unknown(code)` |
| `AssistantArtifactDraft` | `accepted_text`, `proposed_text`, `revision`, `language`, `human_edited`, `max_chars`; accepted and proposed text stay separate |
| `AssistantCallPreparation` | `guidance: ClientCallGuidance`, `beats: Vec<AssistantCallBeat>`, `framing_hazards`, `permitted_closes`, `facts`, `rehearsal_turns`, `sms_suggestion`, `email_subject`, `email_body`, `language`, `language_confirmed`, `rubric_version`, `freshness`; all rich optional content is explicit Option/collection, not a fake empty success |
| `AssistantCallBeat` | `id`, `title`, `intent`, `fact: Option<AssistantFact>`, `say`, `avoid`, `missing_fact: Option<String>`, `worked_example: Option<String>` |
| `AssistantProposal` | `id`, `revision`, `source_id`, `source_revision`, `context: AssistantContextStamp`, `expires_at_ms`, `target: ProposalTarget`, `content`, `policy: InsertionPolicy`, `reviewed_revision: Option<u64>`, `eligibility: AssistantEligibility` |
| `ProposalTarget` | `Composer(AssistantDestination)`, `OwnerWorkflow { kind: Note / Assignment / NextAction / Escalation / Unknown, record_id, record_revision, owner_workflow_id, eligibility: AssistantEligibility }`, `Unknown(code)` |
| `AssistantDestination` | `client_id`, `thread_id`, `channel`, `draft_id`, `draft_revision`, `current_text`, `previous_suggestion: Option<String>`, `eligibility: AssistantEligibility` |
| `InsertionPolicy` | `EmptyOnly`, `EmptyOrUnchangedSuggestion`, `OwnerWorkflow`, `Unknown(code)` |

Review is a host-acknowledged proposal revision, not a generic local "reviewed"
boolean surviving content replacement. PreparedTemplate requires exact unedited
template identity and EmptyOnly. EmptyOrUnchangedSuggestion compares the accepted
destination text byte-for-byte to the previous suggestion; human-edited text
blocks replacement. Source/context/policy generation, expiry, destination revision
and acknowledged review must still match at insertion time. Neither a preview nor
a Declined answer can be a proposal source. Call guidance reuse must not lose the
rich beat fields listed above; extract a crate-private read-only renderer only
where the existing and new contracts genuinely overlap.

## Settings and connection contracts

`AssistantSettings` contains `accepted: AssistantPreferences`,
`proposed: AssistantPreferences`, `accepted_revision`, `proposed_revision`,
`owner_actor_id`, the reasoning-tier `AssistantAccess`, engine list, optional
configured budget, and capabilities. `AssistantPreferences` contains `engine_id`,
`memory_use`, `memory_capture`; capture defaults false independently of use.
No engine is selected by the component. Header reads accepted selected preference;
unsaved preference remains in Settings, actual execution remains per answer.
Every settings edit advances proposed_revision. Save binds accepted_revision and
proposed_revision plus the exact proposed payload; its receipt may update accepted
preferences from read-back but must not clear/accept a newer proposed revision.

`AssistantEngine { id, revision, label, availability: EngineAvailability,
connection: AssistantConnection, capabilities }` uses Enabled or
Disabled { reason_code, text }; unknown availability disables the engine.
`AssistantConnection { state, shape, device: Option<AssistantDeviceFlow>,
account_label, verified_at, expires_at, reason }` represents all 8×3 combinations.
ConnectionState is NotSignedIn/Pending/Probing/SignedIn/Expired/Rejected/Revoked/
NotApplicable/Unknown. SignInShape is Paste/Device/Operator/Unknown.
`AssistantDeviceFlow { verification_url, user_code, expires_at_ms, expires_at }`
is required for pending/device; missing/blank/expired payload forbids opening it.
The URL is displayed but opened only through an explicit host-validated
OpenVerification command. Paste is a named host handoff, never a token input.
Operator is explicitly operator-assisted, never a simulated native flow.
Readiness precedence is tier verdict, then engine availability, then connection;
unsupported protocol and untested CLI keep their own reason codes and text.

## Governed collections

| Type | Fields / variants |
|---|---|
| `AssistantMemory` | `owner_actor_id`, `revision`, `use_enabled`, `capture_enabled`, `entries`, `draft`, `capabilities`; host reconciles these policy columns with accepted Settings |
| `AssistantMemoryEntry` | `id`, `revision`, `class: Language / Tone / Verbosity / WorkingMethod / Unknown`, `text`, `state: Candidate / Confirmed / Withdrawn / Unknown`, `capabilities`, `retention: Option<AssistantRetention>` |
| `AssistantMemoryDraft` | `revision`, typed `class`, `text`, `max_chars`; accepted host field edits only, no unrestricted client-fact category |
| `AssistantRetention` | `removed_from_assistance`, `physical_purge: Option<String>`, `legal_hold: Option<String>`, `backup_expiry: Option<String>`, `provider_retention: Option<String>`; absence means unknown, never erased everywhere |
| `AssistantKnowledge` | `revision`, `entries`, `draft`, `foundations`, `capabilities` |
| `AssistantKnowledgeEntry` | `id`, `revision`, `title`, `canonical_question`, `aliases_en`, `aliases_es`, `audience`, `state: Draft / PendingReview / Published / Rejected / ChangesRequested / Withdrawn / Superseded / Archived / Unavailable / Unknown`, `proposed_body: Option<String>`, `published_body: Option<String>`, `sources`, `lineage`, `comparison: Option<String>`, `review: KnowledgeReviewState`, `capabilities` |
| `KnowledgeReviewState` | `NotReviewed`, `Pending { request_id, proposed_revision }`, `Approved { request_id, proposed_revision, reviewer_id, receipt_revision }`, `Rejected { reason }`, `Unknown(code)`; host-owned independently of submitter draft review |
| `AssistantKnowledgeDraft` | `revision`, `title`, `canonical_question`, `aliases_en`, `aliases_es`, `audience`, `body`, `sources`, `reviewed_revision: Option<u64>` |
| `AssistantFoundation` | `id`, `domain`, `owner`, `revision`, `effective_at`, `review_at`, `coverage: Complete / Partial { reason } / Missing { reason } / Withdrawn { reason } / Unknown`, `source_ids` |
| `AssistantHistory` | `revision`, `entries`, `capabilities`; search/paging/close are not fabricated |
| `AssistantHistoryEntry` | `id`, `revision`, `title`, `scope`, `actual_engine: Option<String>`, `at`, `outcome`, `retained_evidence: AssistantAccess`, `capabilities` |
| `AssistantActivity` | `revision`, `entries`, `budget: Option<AssistantBudget>` |
| `AssistantUsage` | `input_tokens: Option<u64>`, `output_tokens: Option<u64>`, `charge: Unknown / EstimatedApiEquivalent { display } / VerifiedIncremental { display } / SubscriptionAllocation { fixed_cost, allocation: Option<String> }`; no arithmetic mixing these classes |
| `AssistantActivityEntry` | `id`, `at`, `attempt_id`, `scope`, `engine: Option<String>`, `outcome` (including refused/failed/canceled/discarded), `usage`, `detail_availability` |
| `AssistantBudget` | `policy_label`, `limit_display`, `remaining_display: Option<String>`, `as_of`; only host-configured policy |

Learning Review is a typed projection, not a free-form metric dashboard:
`AssistantLearningReview { id, revision, access, interval_start, interval_end,
cutoff, time_zone, late_arrival_catch_up, partitions, stages, assessments,
improvements, capacity, capabilities }`.

`CoverageState` is Complete/Partial/Missed/Unconfigured/Unassessable/CapacityPaused/
Unknown, with causes on non-complete states. `AssistantCoverage` carries
state, total/covered/refused/failed/canceled counts as `Option<u64>`, named missing
partitions, oldest outstanding, owner and proposed recovery. Stages are separate
Inventory/ReusedEvaluation/ModelAssessment/HumanReview/Publication projections.
`AssistantAssessment` carries outcome Correct/Incorrect/Incomplete/Unanswerable/
AppropriateRefusal/Unassessable/Unknown, attributed source UserFeedback/ModelGrade/
HumanAdjudication, rationale, scope, language and engine.
`AssistantImprovement` carries ID/revision, source gap/root-cause/contradiction,
state Detected/Triaged/Owned/Proposed/Approved/Implemented/Published/Verified/
Monitored/Rejected/Deferred/Unknown, owner, disposition, released_version,
verification_evidence, monitored_at, recurrence and opportunity counts (optional),
use-case/language/engine scope. Zero opportunities never demonstrates a fix.
`AssistantCapacity` has Active/Paused/Unknown, reason, policy, next_recovery and
matching suspend/resume operation receipts. No jobs, alerts, purchases or issues
are executed by this UI.

Knowledge publication requires KnowledgePublish and a host-confirmed Approved
review of the same candidate revision, in addition to the exact reviewable body,
sources and comparison. KnowledgeReview is a separate curator grant; the
submitter's local reviewed_revision never confers approval or publication.
Approval exists only in nested review, not a duplicate entry status. A candidate
awaiting publication remains PendingReview with an Approved review receipt.
Only that combination can Publish; a changed candidate revision invalidates its
older review. Published means host-confirmed publication, not just approval.

The initial shared handoff policies intentionally do not authorize replacing a
human-edited Office composer. That adoption path remains deferred until Office's
owner decides it; any later explicit-replace policy needs its own reviewed contract
and guard evidence. The rich call artifact renderer does not migrate No-Hire or
change ownership of ClientCallWorkspace's existing private renderer.

## Text and rendering contract

`AssistantWorkspaceTexts` owns a typed `AssistantText` key map with complete
English and Spanish constructors. Keys cover every fixed heading/action/state/
reason/announcement, including all refusal, connection, operation and governance
variants. Missing caller overrides fall back to the selected complete locale,
not a debug-format enum. Host-provided narrative and source text remain untouched.
Tests enumerate all keys and assert nonblank EN/ES values. Relative/date/number
copy is supplied by the host using the current locale, not parsed back for guards.

All source text uses escaped plain text and whitespace-preserving paragraphs;
there is no HTML injection, Markdown image fetch, live tool instruction or URL
autolink renderer in the initial facade. This intentionally favors a clear safe
boundary over inherited permissive AiChat Markdown behavior.

Errors use `AssistantReason { code, message }` at display boundaries. `code` is
an opaque identifier, not visible provider diagnostics; unknown codes get generic
localized contract-error copy. The host supplies only sanitized `message` text.
No Debug dump, console log or telemetry is emitted for state, commands, drafts,
sources, errors or receipts. CopyAnswer resolves only the permitted visible answer
text through the host, never hidden evidence/diagnostic fields. Source opening
passes an evidence ID to the host; strings in answer text never create navigation.
