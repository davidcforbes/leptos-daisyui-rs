# AiAssistantWorkspace Implementation Plan

Execution uses the Beads dependency graph, not checkboxes in this document. The
primary agent owns integration and landing; every implementation slice receives
an independent review. Existing component fixes `ldui-qg0i` and `ldui-qacu` are
qualified independently; pure AI model work may proceed between their browser
builds. Source/demo stay frozen during each browser host. The six Office-review
Beads are incorporated, not duplicated.

**Goal:** Deliver the Office-reviewed, provider-independent controlled assistant
workspace with truthful facts, attempts, artifacts, settings and governed views.

**Architecture:** A small public Leptos facade consumes one accepted typed
projection and emits guarded typed requests. Pure models own structural
validation and interaction eligibility, never authorization, transport or
business execution. Internal subviews share one text contract and one dispatcher.

**Tech stack:** Rust 2024, Leptos 0.8, daisyUI 5, existing LDUI primitives,
PixelProof browser harness and the existing `cargo xtask` gate.

**Spec:** [Office-reviewed design](../specs/2026-09-10-ai-assistant-workspace-design.md).

**Required interface supplement:** [Exact contracts](2026-09-10-ai-assistant-workspace-contracts.md).
Read this before any implementation phase. It fixes field/variant shapes,
sectional validation, request reservation and command/receipt ownership.

**Required proof mapping:** [Named acceptance journeys](2026-09-10-ai-assistant-workspace-acceptance.md).
This maps every critical contract and negative control to synthetic states and
named tests; it is an obligation list, not a claim of current passing evidence.

## Summary

Build `src/components/ai_assistant_workspace/` without changing `AiChat` or
Office. The facade provides Assist, Preparation, History, Memory, Activity and
Settings, with named unavailable states when an authorized host contract is
absent. A deterministic synthetic host exercises accepted, refused, unknown,
stale and conflicting acknowledgments without a provider or network dependency.

## Motivation and decisions

Office's review accepted the shared boundary and identified five required model
gaps and four smaller contracts. Those are binding requirements, not optional
enhancements. The current component work also demonstrates why DOM truth,
controlled read-back, native keyboard behavior and cleanup require browser tests.

This implements a reusable library, not Office adoption. Owner questions in spec
16.1 remain Office gates. The library uses safe insertion/revision guards and
explicit continuity modes; an adapter must choose its approved migration mode.
Selected engine is never actual provenance. No engine product, sign-in method,
default memory capture, Office pilot or new page capability is chosen here.

Call preparation reuses `ClientCallGuidance` data in a richer typed artifact.
Do not extract ClientCallWorkspace's private renderer in this plan: its broader
ownership/migration decision remains with Office. The new artifact renderer owns
its own richer presentation and introduces no second telephony console.

## Global constraints

- The shared component has no dependency on `office-perf-dto`, Office routes,
  provider SDKs, `ai-chat-engine`, a database, a filesystem, or a network client.
- The workspace may propose an interaction and display its acknowledged result.
  It never grants access, assembles business facts, starts a provider directly,
  publishes knowledge, or performs a business write on its own.
- No arbitrary JSON presentation payload, plugin registry or caller-owned toolbar.
- All evolving public enums are non-exhaustive. Unknown wire states remain
  contract errors and cannot grant permission, completion or side effects.
- Rust scalar-value counting enforces the host-supplied question limit. Enter
  submits, Shift+Enter inserts a newline, and IME composition never submits.
- One canonical controlled draft; no automatic clearing, persistence, retry,
  fallback engine, model call on mount, or replay-as-new-request behavior.
- Every asynchronous request names correlation, context and applicable item,
  attempt, source and expected revision identities. Callbacks are not receipts.
- Material qualifications and per-fact unavailability remain beside the claim.
- Component copy has complete English and Spanish defaults; host content and
  opaque identifiers are not translated. Use existing tokens and container width.
- No production records, provider credentials, paid requests or Office mutations
  in fixtures or verification. No new build runner or screenshot-baseline refresh.
- Use scoped formatting and focused tests; freeze source/demo before each browser
  host. Final qualification is `cargo xtask verify-full`, with its actual summary.
- Beads are the sole execution ledger. Preserve unrelated work, including `.tmp/`.
  Work in the existing checkout, honoring the user's no-stray-worktrees preference.

## Files and responsibility

| File | Responsibility |
|---|---|
| `src/components/ai_assistant_workspace/mod.rs` | Public facade and typed exports |
| `types.rs` | Context, capability, draft, attempt, answer, validated fact and operation contracts |
| `artifact.rs` | Typed artifact, evidence, provenance and handoff contracts |
| `settings.rs` | Engine, sign-in flow, accepted/proposed settings and policy contracts |
| `governance.rs` | Memory, knowledge, history, usage and learning projections |
| `model.rs` | Fail-closed validation, eligibility and exact command construction |
| `texts.rs` | Typed text keys, complete EN/ES defaults and caller overrides |
| `component.rs` | Fixed anatomy, container layout, local view/detail/focus/scroll state |
| `answer_view.rs` | Attempt, qualified answer, provisional/refused/declined presentation |
| `artifact_view.rs` | Preparation, evidence and explicit handoff review |
| `settings_view.rs` | Accepted/proposed preferences and connection matrix |
| `memory_view.rs` | Actor-owned memory and retention receipts |
| `knowledge_view.rs` | Group Important and foundation review |
| `history_view.rs` | Authorized application history |
| `activity_view.rs` | Usage and separately permissioned learning projections |
| `tests.rs` | Native behavioral and guard tests |
| `tests/ai_assistant_workspace_contract.rs` | Native external-consumer command identity and payload proof |
| `demo/src/demos/ai_assistant_workspace.rs` | Synthetic controlled host and interaction receipts |
| `tests/ai_assistant_workspace_smoke.rs` | A/B/C/D browser evidence and negative controls |
| `doc/components/ai_assistant_workspace.md` | Public usage, migration and host acknowledgment contract |
| `src/components/mod.rs`, `demo/src/demos/mod.rs`, `demo/src/main.rs`, `demo/src/core/layout.rs`, `xtask/src/main.rs`, `doc/ci-cd.md` | Register facade/demo route/navigation/focused release lane |

Each view phase consumes the exact state/text/dispatcher contracts from Phase 1
and produces one crate-private Leptos component. Its RED is a named native guard
or view-model test under the module's `tests` subtree before the helper exists;
GREEN is `cargo test -p leptos-daisyui-rs --lib ai_assistant_workspace` plus a
scoped library check. Once the command foundation exists, also run
`cargo test -p leptos-daisyui-rs --test ai_assistant_workspace_contract` for the
external-consumer boundary. Rendered acceptance is exercised by the named fixture tests
below, not claimed from native serialization. Phase 1 and final integration are
primary-agent work; independent reviewers qualify each bounded slice. Source
ownership is sequential even where dependency prerequisites permit parallel work.

## Phase 1: Typed presentation foundation and guarded intents

**Description:** Create the model files, text contract and native tests. Incorporate
`ldui-6qrr`, `ldui-dmen`, `ldui-kll7`, `ldui-eimk`, `ldui-xak4` and `ldui-r14q`
at the model boundary; their rendered acceptance remains open until Phase 9.

**Design:** Public model names are `AssistantWorkspaceState`,
`AssistantWorkspaceCommand`, `AssistantAction`, `AssistantContext`,
`AssistantDraft`, `AssistantConversation`, `AssistantAttempt`,
`AttemptLifecycle`, `AssistantTransport`, `SubmissionDisposition`,
`AssistantFact`, `AssistantComparison`, `AnswerOutcome`, `AssistantAnswer`,
`AssistantArtifact`, `AssistantProposal`, `AssistantEvidence`,
`AssistantProvenance`, `AssistantSettings`, `AssistantEngine`,
`ConnectionState`, `SignInShape`, `AssistantMemory`, `AssistantKnowledge`,
`AssistantHistory`, `AssistantActivity`, `AssistantLearningReview`,
`AssistantAccess`, `AssistantLoad<T>`, `AssistantOperation`, and
`AssistantContractError`. Each family uses owned display-safe data, stable IDs,
explicit revisions, availability and operation receipts. A default state is
non-actionable, not an empty successful workspace.
The interface supplement is normative for fields/variants and constructors.
Unknown variants exist explicitly, including recovery and governance vocabularies.
Core validation is separate from section validation; absence or malformed optional
Memory/Learning data must not disable a valid Ask. Host-provided `next_request_id`
is required for async commands and is synchronously consumed/published pending by
the host before async work. Missing allocation disables the affected request.
The same request is recovered after uncertainty, never minted again by the UI.

Foundation deliverables are split internally into reviewed native slices: core
context/fact/attempt types first; artifact/settings contracts next; governance
types last; then dispatcher/text integration. These slices do not create more
execution trackers or allow placeholder default-allow guards between them.

Implement these pure entry points and use them in the eventual component:

```rust
pub fn validate_workspace(state: &AssistantWorkspaceState)
    -> Result<(), AssistantContractError>;
pub fn can_dispatch(state: &AssistantWorkspaceState, action: &AssistantAction)
    -> bool;
pub fn command_for(state: &AssistantWorkspaceState, action: AssistantAction)
    -> Option<AssistantWorkspaceCommand>;
```

`AssistantFact::new(label, current, baseline, qualification, availability)`
returns `Result<Self, AssistantContractError>`, with owned `String`/`Option<String>`
display values and `Option<AssistantComparison>`. Private fields and getters
prevent mutation around validation. Reject blank label/qualification, blank
present displays, missing current without nonblank availability, and availability
attached to a present current. A comparison requires both display and basis.

`AttemptLifecycle` models admitted/queued/running/validating/completed/denied/
unavailable/failed/canceled/interrupted, with `Canceled { discarded: bool }`.
Completed carries `AnswerOutcome::Answered` or `Declined`; the latter requires
nonempty limitations and permits only flagged/discarded feedback. Errors never
become answer content. Validate answered scope against the attempt's accepted
scope, while historical context remains visibly distinct from current context.

Refusal variants preserve evidence-admission-unavailable, preference-unavailable
and conversation-closed and derive RetryLater/OpenSettings/NewConversation.
Unknown codes produce contract errors and retain the draft. Transport disconnect
does not replace a durable lifecycle or permit Ask to resend uncertain work.

Commands are semantic families for draft edits/suggestions, Ask, new/history,
Stop/recovery, evidence/detail, feedback, artifact edits/rewrite/regeneration,
proposal review/insertion, settings/sign-in/out, memory/knowledge and operational
review requests. The dispatcher re-reads current state for every gesture.
Guards check capability, identity, lifecycle, context/revision, expiry and pending
or uncertain operation state. Unsupported or unknown variants dispatch nothing.

**Acceptance criteria:** Start with failing native tests for each invariant and
guard; run `cargo test -p leptos-daisyui-rs --lib ai_assistant_workspace` after
each model slice. Cover current zero versus unavailable, all refusal mappings,
discarded cancellation, healthy decline, scope mismatch, revoked capability,
stale context, expired proposal, edited destination, unknown enum, missing
terminal answer, uncertain submission and no permissive defaults. No new IO or
provider dependency. Export types through the component module and `components`.

**Parallel:** no.

**Depends on:** none.

## Phase 2: Accepted settings and truthful connection states

**Description:** Implement internal Settings presentation from Phase 1's models.

**Design:** `AssistantSettingsView` consumes state/text signals and the shared
typed dispatch callback. Render all eight connection states and all three shapes,
including pending device verification URL/code/expiry, operator-assisted flow,
and a named paste handoff without accepting tokens. Device navigation is an
explicit typed host intent; never execute an arbitrary URL from an answer.
Tier reasoning verdict takes precedence over engine availability, then connection.
Preserve precise host reason codes and display text, including untested CLI and
unsupported protocol. Requested edits and accepted preferences are visibly
separate until a matching read-back receipt; failed/unknown saves keep both.
Memory use and capture are independent, with capture default off. Sign-out's
worker cleanup is distinct from Office memory forgetting. Budget and usage have
configured/unknown states and no invented cap or purchase action.

**Acceptance criteria:** Native tests enumerate the 8-by-3 model matrix and
reject missing device payload, unknown shape and unauthorized actions. Browser
fixture cases in Phase 9 cover tier denial despite signed-in state, disabled CLI
reason, save/write/read-back separation, stale receipt, rejected save and explicit
device/operator handoffs. No token/password input or provider launch.

**Parallel:** yes, after the typed foundation.

**Depends on:** Phase 1.

## Phase 3: Grounded answers, artifacts and reviewable handoffs

**Description:** Implement the attempt/answer and Preparation/detail subviews.

**Design:** `AssistantAnswerView` renders lifecycle, transport, provisional text,
facts, limitations, evidence and per-answer actual provenance separately.
`AssistantArtifactView` renders typed analysis, reply/template, call preparation,
briefing and note/action proposals. Use the supplement's rich call-preparation
shape; reuse existing guidance data without extracting its private renderer or
introducing telephony. Material caveats stay visible. Unsupported and
revoked content is not mounted behind a hidden control.

Review shows original scope, target, expected draft/record revision, expiration
and proposed content before insertion/handoff. Empty-only prepared insertion and
unchanged-suggestion replacement are different typed policies. Human edits lock
replacement. Preview, declined, stale, expired and unknown results cannot expose
capture/share/insert actions. Receipt text follows host confirmation, never click.
Copy is a guarded host intent with a separately acknowledged platform result;
do not claim independent clipboard read-back or infer success from the callback.
Evidence uses opaque IDs and host-open intents, not model URLs.

**Acceptance criteria:** Native guards cover every disallowed source/target state.
Browser cases prove qualified facts, unavailable values, declined limitations,
distinct discarded-cancel copy, no error bubble, no provisional insertion, scope
mismatch withholding content, explicit proposal review, preserved human draft,
stale acknowledgment rejection and correct actual-versus-selected provenance.

**Parallel:** yes, after the typed foundation; source ownership remains serialized.

**Depends on:** Phase 1.

## Phase 4: Actor-owned memory and retention receipts

**Description:** Implement My Memory in `memory_view.rs` and its independently
rejectable guard tests, consuming Phase 1's memory/access/operation contracts.

**Design:** `AssistantMemoryView` uses explicit load/access states and typed
Language/Tone/Verbosity/WorkingMethod forms. Create/confirm/correct/forget/export
are separately granted and acknowledged. Candidate is not confirmed. Compare
owner_actor_id with the authenticated actor, not the viewed subject. Never mount
denied content or a previous actor's memory. Forget receipts distinguish withdrawal,
purge/hold/backups/provider retention; absence is unknown, never erased everywhere.
Use/capture remain separate policy fields, reconciled with accepted Settings.

**Acceptance criteria:** Native tests reject wrong-actor, stale-revision, unknown
class and unsupported actions. Browser `memory_ownership_and_retention_receipts`
covers candidate/confirmed/withdrawn, denied content absence, changed subject,
changed actor, pending/refused/confirmed forget and retained-under-policy wording.
An unavailable or invalid memory service does not disable authorized Ask.

**Parallel:** yes, after the typed foundation.

**Depends on:** Phase 1.

## Phase 5: Reviewable Group Important and foundations

**Description:** Implement `AssistantKnowledgeView` in `knowledge_view.rs` and
knowledge guard tests from the exact Phase 1 projections.

**Design:** Render title/canonical question, EN/ES aliases, all-Office audience,
lineage, proposed revision/body, sources and comparison. Submitter/reviewer/
publisher grants are independent. Submit creates pending review, not published.
Publish/withdraw/reject/request-changes require exact reviewable revision and
host confirmation. Metadata-only and published-only payloads cannot approve a
candidate. Foundations show domain, owner, revision/effective/review dates and
Complete/Partial/Missing/Withdrawn coverage; no hidden fallback source.

**Acceptance criteria:** Native tests reject stale candidate, absent proposed
body/comparison, missing grant and unknown state. Browser
`knowledge_review_never_optimistically_publishes` proves explicit draft review,
pending-versus-published, conflict preservation, denied content absence and
withdrawn foundations. Inject optimistic publication, catch it, then restore and
confirm clean. No private memory/answer becomes shared content automatically.

**Parallel:** yes, after the typed foundation.

**Depends on:** Phase 1.

## Phase 6: Authorized history and honest usage

**Description:** Implement `AssistantHistoryView` in `history_view.rs` and ordinary
`AssistantActivityView` in `activity_view.rs`, plus focused model tests.

**Design:** History is authorized application continuity with original scope,
actual engine/time/outcome/evidence availability. Ready empty differs from
unavailable retention. Loading history emits only LoadHistory, never Ask. Do not
invent paging/search/close APIs. Usage uses the supplement's exact charge variants;
unknown tokens stay unknown. Estimated API-equivalent, verified incremental and
fixed subscription/allocation are never summed into alleged actual spending.
Include refusals, failures and both cancellation/discarded outcomes.

**Acceptance criteria:** Native guard/format tests and browser
`history_loading_and_usage_are_truthful` prove no Ask on load, current/historical
scope separation, permission loss, unknown-versus-zero, both cancel accounting
states and configured-only budget. Absent-as-zero negative control catches and
clears. No raw questions/private memory appear in ordinary metadata-only Activity.

**Parallel:** yes, after the typed foundation.

**Depends on:** Phase 1.

## Phase 7: Permissioned learning review projections

**Description:** Add `AssistantLearningView` to `activity_view.rs` and independent
coverage/assessment/improvement guard tests.

**Design:** Use exact supplement types for interval/cutoff/timezone/late catch-up,
missing partitions, stage coverage, oldest work/owner/recovery, assessment class
and attribution, source/foundation gaps and contradictions, full improvement
progression/disposition, released version, verification and monitoring, scoped
recurrence and opportunity counts, capacity pause and suspend/resume receipts.
No jobs, provider calls, alerts, Beads or capacity purchases execute here.

**Acceptance criteria:** Native tests reject missing operator grants and invalid
coverage. Browser `learning_partial_is_not_all_clear` distinguishes Partial,
Missed, Unconfigured, Unassessable and CapacityPaused from Complete; zero
comparable opportunities never means fixed. Published does not mean verified;
filed does not mean implemented. Ordinary work remains usable when this section
is unavailable or malformed, and denied review content is absent from the DOM.

**Parallel:** no; shares `activity_view.rs` with Phase 6.

**Depends on:** Phase 6.

## Phase 8: Unified facade, composer and accessible container layout

**Description:** Compose the subviews behind the public `AiAssistantWorkspace`.

**Design:** Props are `control_id`, `state: Signal<AssistantWorkspaceState>`,
`texts: Signal<AssistantWorkspaceTexts>`, `on_command:
Callback<AssistantWorkspaceCommand>` and optional static `class`. Use the
existing Button/Field/Textarea/Select/Badge and approved safe text rendering.
One active view tree, mandatory header/context/availability regions, narrow
Back/detail navigation and wide inspector. No duplication of interactive controls.
Local state is limited to active view, disclosure, focus and scroll following.
Controlled drafts stay in the host; suggestions edit them without asking.

All asynchronous gestures go through `command_for` with current state. Enter
submits only outside IME and without Shift; character bounds use `chars().count()`.
Keep reader scroll position while inspecting earlier content; offer New output
instead of stealing focus. Escape closes only the top detail/disclosure, never
cancels an attempt. Closing the workspace preserves host draft and pending work.
Use container-driven layout and complete EN/ES labels and announcements.
Visual direction is a calm working console: the persistent context/evidence
strip is the defining element, left-aligned reading text stays below roughly
80 characters, and action outcomes use distinct semantic names and shapes.
Use the repository's existing base-100/base-200/base-content/primary/warning/error
tokens and type ramp rather than adding a palette or fonts. Avoid decorative
statistics, repeated card chrome, animated token streams and assistant mascots.
Review this direction against the spec's fixed anatomy before rendering code.

**Acceptance criteria:** Native defaults/text coverage plus real browser keyboard
journeys for Ask, Shift+Enter, IME, native view Select, Stop, settings, disclosure
Back/Escape/focus return. Assert exact callbacks and no on-mount/tab-switch model
request, no hidden duplicate controls, no unauthorized composer, correct long-
content wrapping and readable scope/evidence at narrow width and enlarged text.

**Parallel:** no; composes all prior subviews.

**Depends on:** Phases 2, 3, 4, 5, 6, 7.

## Phase 9: Synthetic host, focused lane and detection controls

**Description:** Register the demo and a focused release browser lane, included
in `verify-full` using the existing catalog host. Complete the six review Beads'
rendered acceptance and qualify the complete facade. Register the external
native contract test in the ordinary native gate too: the existing `test-lib`
step uses `--lib` and cannot execute integration tests by itself.

**Design:** The fixture owns accepted state, draft/record revisions, receipt log
and deterministic delayed/refused/out-of-order host responses. It never calls a
provider. Expose independent command counts and accepted state for the D oracle;
do not let the component's own DOM manufacture the expected result. Cover all
spec section 14 journeys with bounded tests, not a soak loop. Screenshots use
synthetic data and are reviewed directly at narrow/wide, EN/ES, theme and zoom.

**Acceptance criteria:** `cargo xtask test-ai-assistant-workspace` passes with
fresh assets. Inject/catch/revert controls detect stale-action dispatch, duplicate
Ask on recovery, optimistic publication, absent-as-zero usage, hidden duplicate
controls and preview insertion. Accessibility tests inspect names/roles, keyboard
operation, focus return and concise announcements. Capture browser errors and
assert none; verify repeated mount/unmount releases owned listeners/observers.

**Parallel:** no; depends on Phase 8.

**Depends on:** Phase 8.

## Phase 10: Public contract, review and final qualification

**Description:** Document the implemented API, validate the spec-to-code coverage,
review the complete change and land the reconciled work.

**Design:** The guide includes a compiling synthetic host example, capability and
command tables, correlation/read-back rules, locale use, security and privacy
boundary, bounded artifact handoffs, historical scope, migration modes and Office
adoption prerequisites. It states what the library proves and what only Office's
adapter/provider gates can prove. Update the registry/CI guide with the exact
focused command and preserve the current runner membership as authority.
Preserve Office's softphone re-vendor ordering/declared deltas, migration retirement
of source-shape tests, and reasoning-step version bumps when memory/expanded scope
changes prompt bundles. Keep each tag safely below Tachys's 26-attribute budget;
spread diagnostics onto a child model element, not the page main.

**Acceptance criteria:** Independent task and whole-change review findings are
resolved; native focused tests and `cargo xtask verify-full` pass on the frozen
candidate. Review regenerated screenshots, record exact commands/exit/elapsed
time and candidate, refresh all Beads queue states, close/read back completed
issues, commit and push to `fork/main`. Verify no new worktrees/branches, no
owned background hosts and no accidental Office or `.tmp/` changes.

**Parallel:** no; depends on Phase 9 and all six Office-review requirement Beads.

**Depends on:** Phase 9.
