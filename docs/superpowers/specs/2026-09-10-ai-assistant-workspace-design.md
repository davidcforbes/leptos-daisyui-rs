# AiAssistantWorkspace: Office research and shared-component design

Status: **Draft for the 4iiz-office agent's review and amendment. Not approved for implementation.**

Date: 2026-09-10. Proposed owner: `leptos-daisyui-rs` for the reusable UI;
`4iiz-office` for its application adapter and services.

The user requested research and this specification first, then review by the
Office agent, **before Beads creation or implementation**. This document is not
an implementation plan or a change to Office's approved backlog. No Beads,
application code, provider sessions, or deployments were created for this research.

## 1. Recommendation

Create **`AiAssistantWorkspace`**, one highly opinionated, controlled Leptos
composition for contextual assistance. It should combine the common interaction
model of Office's assistants with the presentation needed by Office's newer
assistant-platform design: grounded answers, contextual artifacts, safe draft
handoffs, durable-turn status, evidence, history, engine configuration, governed
memory, and permissioned learning review.

This is feasible as a shared **UI** component. It is not feasible as a standalone
component that makes all of Office's assistant capabilities operational merely
by mounting it. Several required capabilities are backend-exposed but not adopted
by the current screens; others remain design targets. The component must render
those distinctions honestly, and Office must provide the missing contracts.

Do not turn the existing `AiChat` into an Office-specific agent framework.
Preserve it for its current consumers. Reuse appropriate presentation mechanics
and LDUI primitives, but introduce an explicit controlled boundary rather than
adapting Office's durable attempts to a provider-session chat protocol. Existing
`AiChat` owns a `ChatSession` drive loop and exposes model/system-prompt/tool
settings; Office needs server-validated facts, action proposals, memory governance,
and a stronger acknowledgment contract.[^ldui-chat][^turn-contract]

The defining rule is:

> The workspace may propose an interaction and display its acknowledged result.
> It never grants access, assembles business facts, starts a provider directly,
> publishes knowledge, or performs a business write on its own.

“Includes all features” means one coherent UI family with every researched
feature assigned a home, a typed availability state, and an owner. It does not
mean showing every control on every page or absorbing telephony, CRM, scheduling,
model execution, and background maintenance into the component.

## 2. Evidence, authority, and limits

### 2.1 Research snapshot

| Repository | Inspected revision | Evidence used |
|---|---|---|
| `C:/dev/4iiz-office` | `a27d3fccdd02b6d3b73c6e97e60352cc9c460cf8` | Active satellite UI, shared satellite UI, DTOs, routes, assistant services, engine adapters, and architecture/design corpus |
| `C:/dev/leptos-daisyui-rs` | `c3c32cf283445790fc0b288c6c7caee8bba585ea` | Existing `AiChat`, controlled call-workspace pattern, component conventions, and visual-quality rules |

Both inspected HEADs equaled their local `main` at research time. Office's
pre-existing `.beads/issues.jsonl` change and its detached short-path build
worktree were not part of this work. LDUI's pre-existing `.tmp/` was preserved.

This is a **static source and document audit**, not a production certification.
No live client records, prompts, credentials, subscription entitlements, provider
responses, or operational metrics were inspected. No application tests or long
builds were run. “Implemented” below means found in this source snapshot; it does
not establish deployed behavior, business approval, or passing runtime evidence.

### 2.2 Which documents control the design

Office's September 9 business requirements remain a draft business baseline.
Its four core contexts and owner-directed memory program must nevertheless be
preserved; this specification does not reopen the recorded trial selection,
Group Important, foundation seeding, or daily full-inventory curation decisions.
Detailed acceptance and the business decision register still need the designated
owners.[^brd]

Office's controlling V1 requirements additionally reserve comprehensive AI,
automated coaching and manager journaling for written Bert approval. Technical
review of this component cannot reprioritize the V1 backlog or substitute for
that approval. The existing approved work and workflow/data prerequisites remain
controlling.[^v1]

The revised September 9 platform design is the current implementation-design
authority beneath Office's business and architecture rules. It supersedes older
engine-design assumptions about per-worker provider context, token-paste as the
default sign-in experience, preference-only memory, and a presumed daily USD cap.
The comparison document is useful evidence of the review, not a second active
list of unresolved engineering decisions.[^platform][^comparison]

Two dated descriptions must not be repeated as present facts:

- The BRD's starting-point statement that engine choice, memory, and reporting
  are absent predates source now containing engine/configuration, memory, and
  durable-attempt modules. End-user reporting and full UI adoption are separate.
- Older documents name `crates/office-perf-web`; the active application consists
  of independently built `surfaces/` satellites and shared satellite UI. Office's
  vendoring document also retains historical consumer names. Current source and
  the immutable vendor provenance are the relevant adoption evidence.[^vendor]

### 2.3 Terminology that the UI must keep separate

| Term | Meaning |
|---|---|
| Office assistant conversation | Application-owned continuity and permitted history |
| Assistant attempt/turn | One durably identified request, including a refused, canceled, or unanswered request |
| Provider connection/process | Host-managed authentication and execution resource; not the conversation's authority |
| Provider reasoning context | Fresh logical model context assembled for an authorized reasoning step |
| Client conversation | SMS, WhatsApp, email, or phone engagement with a client; never the assistant transcript |
| Generated artifact | Analysis, script, reply draft, or briefing with its own provenance and validity |
| Proposal | A reviewable suggestion; neither an approved business command nor a completed write |

## 3. Current implementation inventory

These are distinct experiences, not instances of one existing chat component.

| Surface or capability | Observed source behavior | Required preservation or gap |
|---|---|---|
| Leader dashboard | One-question coach; 1,000-character bound; current office and selected KPI; answer, grounded KPI facts, current/baseline values, qualifications, limitations, as-of and model/prompt fields | Preserve structured facts and unavailable values. The response model field is currently Groq-labeled, not proof of actual executor identity. No shared settings mount observed.[^dashboard][^provenance-gap] |
| Coordinator | Locally retained `Vec<ChatMessage>`; selected office/worker scope; grounded answer details; explicit not-enabled versus failure; collapsible right rail | Local transcript is not durable Office history. This is the observed production callsite of the shared settings dialog; its engine label/gear are manually wired.[^coordinator] |
| Journal | One-question assistant over the actor's journal, lessons, and application primers; answer, as-of and limitations; editing the question clears the old answer | Preserve own-journal scope and visible errors. Do not relabel old questions as persisted history or assume journal text is free of client information.[^journal] |
| Existing client conversation | Stored/refreshed transcript analysis, objections, grounded talking points, suggested next action, caveats; separate one-off coaching; separate reply generation and shorter rewrite | Analysis, internal coaching, and client-facing drafts need different artifact types. Numeric Desk-ticket/capability restrictions remain host-owned.[^conversation] |
| Prepared client contact | One initial opener request when valid prepared mode is entered; review preview; explicit use into an empty composer; exact-template identity, expiry, changed-context and uncertain-attempt states | SMS/WhatsApp preparation does not imply email drafting support: prepared email drafting is intentionally unsupported in this snapshot. No automatic retry after uncertain paid work; human Send remains separate.[^prepared] |
| AI call preparation / No-Hire | Script loading/ready/preparing/failed/missing/unavailable is independent of page readiness; script suggestion seeds editable SMS; ordered beats, editable opening, Save edit, Reset and simulated call | Preserve capability-gated/read-back-backed script editing and keep Call/Text usable when the script fails. A cached artifact is not a chat turn; no hidden model call on its read GET.[^call-script] |
| Active Account detail | `surfaces/account` loads payment-call-script suggestions, seeds SMS/email without replacing human edits, bounds script polling, offers regeneration and prepares contact in the existing workflow | Keep script generation, editable message drafts, Place call and explicit Queue actions separate. The surrounding softphone is host-owned, not a second assistant implementation.[^account] |
| Account Conversations | Independently reads the payment-call script to seed SMS/email; preserves typed drafts on refresh; offers channel preparation, editable composers and softphone/contact workflows | Consolidate assistant-artifact presentation and suggestion reconciliation without replacing channel consent, contact updates, softphone or human dispatch.[^account-conversations] |
| Shared assistant settings | Reads one engine projection; changes common engine preference and memory switch; handles 204 then re-reads; shows availability reasons and optional budget information; best-effort warm | Retain read-back verification. Sign-in/out and export/forget controls are disabled placeholders in this UI snapshot, even though related backend routes exist.[^settings] |
| Durable turns | Conversation creation, typed admission/lifecycle, replayable SSE, snapshot, cancel, feedback, restart recovery | Source-exposed platform API; current coach screens still use legacy requests. Do not claim the current screens stream, resume, or browse durable conversations.[^turn-contract][^turn-runtime][^routes] |
| Governed memory | Personal memory/policy, Group Important candidate, knowledge revision and curator APIs; recall in several coach/artifact paths | Recall is not universal: simple conversation Q&A does not adopt it. Backend source is not a completed memory drawer, full publication workbench, export workflow, or daily learning operation.[^memory] |
| Provider engines | Closed Groq/Claude Code/Codex choices, preference/availability projection, bounded execution and journal attribution | Engine existence does not establish usable employee sign-in, entitlement, isolation proof, or a selected-engine guarantee for every generated artifact.[^engines] |

### 3.1 Migration hazards found in the audit

1. **Turn and memory integration is incomplete.** Typed-turn admission currently
   supplies an empty memory fingerprint, with a comment deferring integration;
   coach execution separately recalls memory. A UI must not present the admitted
   context key as proof of complete memory-bound answer validity. Office must
   reconcile admission, assembled-context evidence, cache dependencies, and
   response-release invalidation before making that promise.[^memory-gap]
2. **Replay is not a history browser.** The examined route family exposes create,
   per-turn events/snapshot/cancel/feedback, not a complete authorized conversation
   list/load/search/close journey. Durable storage alone cannot populate History.
   Require explicit host projections for the designed History experience.[^routes]
3. **Published knowledge is not a review preview.** The current knowledge DTO
   returns `published_text`, not the unpublished candidate body. A curator must
   see the actual proposed revision and its sources before approving it; the
   workspace cannot manufacture a review screen from revision metadata.[^memory]
4. **The old configuration HTTP mismatch has been repaired in source.** The
   current dialog uses the engine projection and 204-plus-read-back workflow.
   Do not re-file the comparison document's old GET-preferences/usage-route
   mismatch as an unfixed defect. Remaining gaps are adoption and disabled
   journeys, not that historical request shape.[^settings][^comparison]
5. **No silent change of engine provenance.** Chat preference, pre-generated call
   scripts, transcript analysis, and background reasoning can have different
   generation paths. Actual artifact attribution wins over the currently selected
   chat-engine label; BRD D-09 decides the eventual settings scope.[^brd]
6. **Requested engine is not actual engine evidence.** Admission records the
   preference, but execution later re-reads it. The typed runner serializes the
   coach response and drops the returned `EngineTurn`; Leader's response model
   field is unconditionally Groq-labeled. Consequently, a preference-change race
   can make requested and actual execution differ, and the typed completion lacks
   an actual-engine/usage projection. This is a source-supported risk, not an
   observed production incident. Office must pin/reconcile execution identity
   and project actual provenance, including cache-only/no-provider execution,
   before the shared UI can claim it.[^provenance-gap]

## 4. Complete feature coverage

The following maps all 39 numbered CO/LD/AC/CV/SH/MEM requirements to a UI home.
It is coverage of the proposed design, not a claim that the requirements are
already implemented or approved. Page data, permissions, and workflow policy
remain Office dependencies.[^brd]

| Requirements | User-facing feature | Workspace home and Office responsibility |
|---|---|---|
| CO-01, CO-02 | What to work on next; why it matters | Contextual prompts, ranked recommendation/fact cards, evidence and caveats; host supplies approved work population and ranking rationale |
| CO-03 | Prepare combined client contact | Contact-preparation artifact; host resolves client/matter/channel and existing preparation workflow |
| CO-04 | Follow-through and next action | Reviewable note/next-action proposal; host owns validation, save and acknowledged outcome |
| CO-05 | Honest progress | Progress facts with period, denominator and missing completion data; no inferred completion from chat |
| LD-01, LD-02 | Daily briefing; explain a change | Briefing and comparative fact blocks with baseline, qualification, period and source |
| LD-03, LD-04 | Delegation/workload; evidence-based coaching | Prepared recommendation and supporting evidence; no new cross-worker visibility or automatic assignment |
| LD-05 | Weekly review | Period-scoped briefing/answer; host supplies review population and outcomes; persistence is a separate platform capability |
| AC-01, AC-02 | Factual client briefing; sourced Q&A | Account-context artifact and evidence-backed answers; document metadata is not evidence that document text was read |
| AC-03 | Unresolved items | Typed open-issue/unknown-data blocks; host owns completeness and identity resolution |
| AC-04 | Contact preparation | Call/script/contact-preparation artifact alongside the existing host communication workflow |
| AC-05 | Durable note/next action | Explicit prepare/review/handoff; saved state appears only from the record owner's receipt |
| CV-01, CV-02 | Queue attention; understand a thread | Queue recommendations and separately versioned conversation analysis; host authorizes the population/thread |
| CV-03 | Initiate SMS/WhatsApp/email/phone contact | Prepare an authorized channel workflow; the component does not send, dial, grant channel access, or infer consent |
| CV-04, CV-05 | Draft; rewrite/translate | Dedicated client-draft card, language/intent controls, comparison and explicit insertion; source and draft remain distinct |
| CV-06 | Escalate | Escalation proposal linked to the existing owner workflow, never an autonomous notification |
| CV-07 | Honest send/follow-up state | Host-projected unsent/pending/confirmed/failed/unknown state, separate from AI generation status |
| SH-01, SH-02 | Common engine settings; individual connection | Shared Settings view with actual engine/connection status and host-managed sign-in/out intents |
| SH-03, SH-04 | Controlled personal memory; scoped continuity | My Memory, context strip, historical-scope labeling, evidence receipt and authorized History |
| SH-05 | Honest failure without blocking ordinary work | Typed errors and availability, retained drafts, independent workspace dismissal, manual workflow remains usable |
| SH-06 | Usage/evaluation/admin reporting | Per-attempt provenance/usage plus permissioned Activity and Learning Review; null is not zero |
| SH-07 | EN/ES, keyboard and assistive technology | Complete localizable text contract, labeled controls, focus discipline, quiet stream announcements |
| SH-08 | Availability on related pages | Host page-capability policy controls placement; preserve existing Journal without creating an assistant on every satellite |
| SH-09 | Interactive work before maintenance | Visible busy/capacity/maintenance states; host scheduler owns reservation, yielding and quota policy |
| MEM-01 | Office-owned layers; cross-engine memory | Distinct personal, shared, foundation and supplied-context evidence; no provider-native memory UI masquerading as Office memory |
| MEM-02 | Explicit Group Important | Preview scope/audience, submit, pending review, review/publication receipts; submitted is not shared |
| MEM-03 | Foundations for Office, 4Ease, related apps, DienerLaw | Foundation domains, source/version/review status, incomplete/withdrawn states; no hardcoded substitute facts |
| MEM-04 | Inspect/correct/govern memory and evidence | My Memory forms, revision/conflict states, authorized knowledge detail and evidence access; host retention/holds apply |
| MEM-05 | Daily complete inventory and answer review | Coverage by inventory/attempt/stage, partial/missed/unassessable states, cutoff and catch-up evidence |
| MEM-06 | Curate, summarize, consolidate and clean up | Candidate comparison, lineage, conflicts, reviewable merges/corrections/retirements and cleanup dispositions |
| MEM-07 | Verified remediation | Classification, owner/disposition and linked improvement, released version, replay/regression and monitoring evidence |
| MEM-08 | Learning health and operational follow-through | Permissioned Learning Review with recurring gaps, overdue work, publication/verification/monitoring state and suspend proposal |

Account contextual Q&A, richer delegation, multi-channel expansion, complete
history, curation, and operational reporting are **target capabilities**, not
just missing LDUI renderers. The Office review must identify their approved host
contracts and existing work before any implementation Beads are created.

Attachments/uploads, microphone dictation, voice-agent operation, unrestricted
web search, arbitrary file/folder scopes, tool consoles and plugin/MCP management
were not required by the reviewed assistant requirements. They are not added by
this proposal. Phone contact remains a host workflow; it is not permission to
turn this workspace into a voice assistant.[^brd][^platform]

## 5. Architectural choice and ownership

### 5.1 Alternatives

| Approach | Benefit | Cost / reason not selected |
|---|---|---|
| Add flags and slots to `AiChat` | Small initial callsite change; retains current session wiring | Mixes two different lifecycle/authority models and encourages flattening evidence, drafts and receipts into messages |
| **New controlled workspace facade** | One standard experience, explicit typed invariants, reuse of LDUI rendering primitives, host-independent tests | Requires a deliberate Office adapter and new public presentation types; recommended |
| Move the complete Office assistant platform into LDUI | Superficially one integration dependency | Wrong ownership: brings application authorization, storage, providers, retention and business policy into a UI library |

### 5.2 Boundary

| LDUI owns | Office adapter owns | Office core/services own |
|---|---|---|
| Fixed anatomy, controls, typography, responsive layout, keyboard/focus/scroll, typed-state rendering and intent guards | DTO validation/mapping, reactive read models, draft state, request correlation, replay/snapshot transport, route/base-URL handling, localized domain copy and page placement | Authentication/authorization, actor/subject resolution, approved readers, facts/context assembly, model execution, memory recall/persistence, evidence journal, publication, retention, scheduling, business mutations and verification |
| Intent payload shape and refusal presentation | Dispatch into existing record/composer/assignment workflows; feed accepted receipts back | Final permission/revision/idempotency checks; actual save/send/assign/escalate/cancel outcome |

The shared component has no dependency on `office-perf-dto`, Office routes,
provider SDKs, `ai-chat-engine`, a database, a filesystem, or a network client.
Its model is a validated **presentation projection**, not a second business DTO
layer or an authorization engine. Closed, typed variants cover interaction
semantics; the adapter retains opaque domain IDs and source references.

Prefer existing LDUI `Button`, `Field`, `Textarea`, `Select`, `Tabs`, `Badge`,
`Alert`, `Modal`, `Drawer`, `MarkdownView` and layout primitives. Reuse the
controlled-state/command pattern already used by `ClientCallWorkspace`.
Inspect and test any extracted `AiChat` scroll/composer helpers without changing
the existing component's public behavior as an incidental migration.[^ldui-call]

## 6. Opinionated user experience

### 6.1 Fixed anatomy

The contextual panel and full workspace use the same regions and state model.
The host chooses placement; the component adapts to **container width**, not a
guess based solely on browser width.

```text
+---------------------------------------------------------------+
| Assistant       [actual engine/status]       New  Expand  Close |
| Context: page / subject / office / period     [Why this scope]  |
| [stale / expanded-scope / policy / availability notice]         |
| Assist | Preparation | History | Memory | Activity | Settings   |
+----------------------------------------+----------------------+
| Grounded answer / transcript           | Selected evidence,   |
| - facts and material qualifications    | artifact detail, or  |
| - analysis / client draft cards        | proposal review      |
| - source and action receipts           |                      |
+----------------------------------------+----------------------+
| Ask about this context ...                         Ask / Stop  |
| Question bound / attempt status / draft-retention notice       |
+---------------------------------------------------------------+
```

The diagram describes region order, not a final pixel mockup. In a narrow rail,
detail replaces the body with a labeled Back control; it does not squeeze two
columns into a 400-pixel panel. Full-width mode can show body and inspector
together. Navigation labels may collapse into a labeled view selector. Only
one active variant is interactive; no hidden duplicate Send/Save/Ask controls.

Preparation appears when the host supplies an analysis, script, draft or other
supported artifact. Activity contains ordinary usage and, for expressly granted
review roles, Learning Review. Permission-denied sections are omitted without
leaking their contents. A supported but temporarily unavailable feature has a
named unavailable state, not an empty tab or a fabricated empty collection.

The contextual header, scope and availability rules are mandatory. A question
composer is present only when the host declares interactive Ask support; a
call-preparation-only context remains explicitly artifact-only. The component
must not create a new chat capability merely because it can render a composer.
Consumers may not omit evidence/caveats or relabel a proposal as completion.
Customization is limited to localized copy, context summaries, prompt suggestions,
validated content, theme tokens, and bounded host-owned artifact-detail/navigation
handoffs. No arbitrary toolbar or runtime plugin registry is proposed.

### 6.2 Context and scope changes

The context strip distinguishes the signed-in actor from the worker/client/matter
being viewed. Personal preferences and memory belong to the actor, not the
view-as subject. The host supplies an opaque accepted context key plus visible
scope, permissions revision, data revision/as-of, and relevant office, period,
filter and client/thread descriptors. The component never constructs authority
by concatenating labels.

Changing office, subject, period, filters, matter, thread or relevant permissions
invalidates actionable reuse under the old context. The old answer may remain
visible as **historical**, labeled with its original scope; its proposals are
locked until the host revalidates them. Pending work keeps its original IDs and
cannot land in the new context's transcript. A context change does not silently
reinterpret an unsent question: the user reviews the new scope before asking.

Questions such as “these rows” preserve the page population. Broader questions
may use only host-authorized typed readers and display a separate expanded-answer
scope with resolved population, sources and missing-data notices. The UI does not
expand an office or client scope because a prompt asks it to.[^platform]

### 6.3 Conversation and composer

Use a plain-language contextual prompt and a small set of host-supplied suggested
questions. Selecting a suggestion **fills the draft for review**; it does not
spend provider capacity. Ordinary mount, opening Settings, switching tabs, reading
history or resizing never starts a model call.

The Office adapter may deliberately preserve prepared contact's existing single
initial opener request. That is an explicit, validated host workflow, not a
generic component-on-mount effect. Cached call-script reads remain reads.

Enter asks; Shift+Enter inserts a newline; IME composition must not submit.
Show the host's actual limit and counting semantics (current question API:
1,000 Rust `char` values, not a guessed byte limit). Blank/over-limit input,
unavailable capability and an incompatible in-flight operation cannot dispatch.
Do not clear the draft because a request was locally queued. Clear or archive it
only after correlated admission; retain it on refusal and uncertain admission.

New conversation asks the host for new application continuity; it does not erase
durable history, forget memory, restart a provider account, or start a paid turn.
Do not advertise persisted history when the adapter supports only a local or
single-answer experience. Those temporary migration modes must be named.

Follow the latest output only while the reader is at the bottom. Scrolling up
preserves position and offers “New output” navigation. Copy reports success or
failure honestly and copies only permitted visible content, never hidden evidence
or provider diagnostics. Assistant text is not internal reasoning trace.

## 7. Public presentation contract

The following is an API-shape proposal, not compiled code or an approved final
Rust interface. The Office agent should review the mapping before implementation.

```rust
AiAssistantWorkspace {
    control_id: String,
    state: Signal<AssistantWorkspaceState>,
    texts: Signal<AssistantWorkspaceTexts>,
    on_command: Callback<AssistantWorkspaceCommand>,
    class: &'static str,
}
```

`control_id` is stable and unique for focus/ARIA/test hooks. `state` contains the
accepted presentation truth. Commands carry the relevant context, item, request,
and revision identities; the component rechecks current `can_dispatch` rules at
the gesture boundary, following the existing controlled-composition convention.
The host repeats all meaningful validation at its trusted boundary.

### 7.1 Required model families

| Model family | Required semantics |
|---|---|
| Context | Opaque context key and epoch; actor/subject distinction; visible page/answer scope; data freshness; historical versus current |
| Capability | Supported/allowed, denied, unavailable, policy-not-configured and busy reasons; no permissive defaults or grants inferred from labels |
| Conversation | Application conversation identity, continuity mode, history availability, accepted selection and load/error state |
| Composer | Host-local draft ID, revision, text, context binding, input bound, pending submission correlation and acknowledged disposition |
| Attempt | Request/attempt/conversation identity, accepted context, lifecycle, transport condition, last sequence, timing and display-safe reason |
| Answer | Validated content blocks, facts/qualifications, limitations, evidence references, actual engine/model/version/as-of and permitted interactions |
| Artifact | Distinct analysis, client draft, call preparation, briefing, or proposed note/action; original provenance, validation and freshness |
| Proposal | Kind, target reference, source answer/artifact, expected context/record/composer revision, expiry, availability and acknowledgment |
| Engine/settings | Available engine projections, requested/accepted preference, connection state, memory-use/capture policy, save/read-back state |
| Memory/knowledge | Typed item/revision/state, lifecycle/eligibility, permitted audience, publication status, sources, lineage and operation receipt |
| Activity/review | Permissioned usage, billing meaning, coverage intervals and counts, partial/unassessable state, review/improvement/verification projections |

Use closed semantic variants instead of ambiguous combinations such as
`loading=true, error=Some, success=true`. Optional values mean unavailable or
inapplicable only where that meaning is explicit; an unloaded collection has a
load state, not a misleading empty `Vec`.

The shared API may expose non-exhaustive enums for evolution. The Office adapter
must still reject unknown wire state/kind strings into a visible contract-error
state. A fallback arm must never convert an unknown value into permission,
completion, or a side effect. Do not pass arbitrary `serde_json::Value` into the
renderer just because a transport DTO has a JSON payload.

### 7.2 Commands and acknowledgment

Command families cover local draft edits; ask/new/select-history; cancel/reconnect
or snapshot refresh; source/detail opening; feedback; draft rewrite/translation;
script-opening edit/save/reset and explicit regeneration; proposal
preparation/insertion/rejection; settings/connection requests; memory
create/confirm/correct/forget/export; Group Important submission; authorized
curator decisions; and authorized operational suspend/resume proposals.

Draft edits can be adopted synchronously by the host's local reducer; they do
not require a network round trip. Keep one canonical draft copy. LDUI owns focus,
scroll and disclosure state, not a second conflicting copy of accepted settings,
memory or mutation outcomes. Do not add browser persistence of client text by
default; navigation retention is an explicitly governed host responsibility.

Every asynchronous command has a correlation ID and pending/accepted/refused/
failed/unknown result. Stale acknowledgments cannot update another item or a newer
draft. A callback invocation, HTTP write echo, SSE connection, or clipboard intent
is not an acknowledgment that the requested result exists.

## 8. Turn lifecycle and transport truth

Office currently defines four progress states and six terminal states:
`admitted`, `queued`, `running`, `validating`, then `completed`, `denied`,
`unavailable`, `failed`, `canceled`, `interrupted`.[^turn-contract]

```text
Draft -> submitting -> admitted -> queued -> running -> validating -> completed
             |            |          |          |           |
             +------------+----------+----------+-----------+
                         named refusal/failure/cancellation/interruption

Transport: connected <-> disconnected/reconnecting -> snapshot/replay
           (transport state does not replace the durable attempt state)
```

This is the normal path, not permission for the browser to invent intermediate
events. Render only acknowledged states. The server may refuse before admission;
the UI can also have a local `submitting` state that is not a journal state.

- `preview` deltas are visibly provisional, not a final answer. No insertion,
  sharing, memory capture or business proposal is enabled from provisional text.
- A completed attempt requires the accepted final answer/evidence contract.
  An ended stream without a terminal record is **not** completion.
- Cancel first becomes **cancellation requested**. Only the server terminal
  establishes cancellation. A late provider answer cannot re-enable old actions.
- Reconnect follows the same attempt's acknowledged sequence or snapshot. It
  never reissues the question. Sequence gaps, duplicates and conflicting terminal
  payloads are adapter errors/recovery conditions, not concatenated new answers.
- A retry of uncertain submission first resolves the original idempotency key
  through the host contract. A deliberately new attempt requires explicit user
  intent. Neither network failure nor provider warm failure authorizes fallback
  to another engine or an extra potentially billable call.
- Boot-interrupted attempts remain interrupted; a mounted browser must not
  restart them. Show a new-attempt action only where host policy permits it.
- A stream error has a different meaning from a failed turn. Use the snapshot
  route to recover the record rather than showing the last partial text as done.

The Office adapter owns SSE parsing, sequence reconciliation, request disposal
and backoff. The workspace consumes a typed accepted model, not raw events.
The existing generic `SseBridgeTransport` protocol and permissive parser are not
a substitute for this adapter.[^ldui-chat][^turn-runtime]

## 9. Answers, artifacts, evidence, and safe actions

### 9.1 Grounded output

Keep answer text, factual cards, qualifiers, gaps, sources and attribution
structurally separate. Facts can be present, partial or unavailable; zero,
unknown and not applicable are different. Comparative cards carry baseline and
period/denominator. Material caveats are visible beside the claim, not only
inside a collapsed inspector.

“Why this answer?” shows what was supplied: permitted page facts, approved
lookups, as-of, memory/foundation revisions, prompt/skill/model versions and
omissions. It must not claim to expose the model's private reasoning or prove
that each supplied memory causally influenced a sentence. A hash alone is not
reconstructable source evidence.[^platform]

History and evidence are reauthorized by the host under current access rules.
Revoked or held content can become unavailable while disposition metadata remains
visible where permitted. Do not fetch an unrestricted source URL from model text;
use typed source-open intents and host-validated destinations. Markdown rendering
must not execute HTML, scripts, tools or automatic remote-resource loads.

### 9.2 Distinct artifact lanes

| Artifact | Presentation and action contract |
|---|---|
| Conversation analysis | Objections, grounded talking points, proposed next action, limitations and built-at; refreshing is a separate acknowledged operation |
| Client reply/opener | Explicit recipient/channel/language context; draft label; requested intent; optional subject; rewrite/translate creates a new reviewable revision |
| Prepared template | Exact approved-template reference, current draft revision, expiry and context guard; edited text must not retain a false approved-template identity |
| Call preparation | Ordered intent/fact/say/avoid material, worked example, language and confirmation state, missing-fact marker and generation provenance where supplied |
| Briefing / progress | Factual summary and comparisons with provenance and completeness, not a fabricated schedule or performance rating |
| Note / assignment / escalation proposal | Explicit target, proposed content, sources and expected revision; open the existing owner workflow for confirmation |

Call preparation also accommodates source-supplied framing hazards, permitted
close options, grounded facts, consultant/client rehearsal turns, SMS and email
subject/body suggestions, rubric/prompt version and freshness. Human-edited
content is labeled as edited; it is not reattributed wholesale to the model.
Opening edits, Save edit, Reset and Regenerate are distinct, capability-gated
commands with independent pending/failure/read-back states. Script failure does
not disable otherwise permitted manual call/text work.

Do not lead a call script with a redundant balance banner or conceal a beat's
missing file detail. Those are documented/DTO-supported safeguards, **not all
currently rendered by No-Hire**: its present beat renderer reads title, intent
and say. The richer renderer is an explicit target needing review, not a claim
of current UI parity. Preserve EN/ES and unconfirmed-language distinctions; do
not imply the selected chat engine generated a cached script.[^call-script]

Preserve the existing Account/Account Conversations suggestion-reconciliation
rule: a refreshed suggestion can replace an empty draft or a draft still exactly
equal to the previous suggestion; user-edited text survives refresh, language
change, regeneration and refusal. This controlled seed rule is different from
prepared opener's **empty-only** insertion rule. The artifact kind and host
contract select the applicable rule; there is no universal replace-text action.
Manual message bounds, channel availability and Send/Queue remain host-owned.
[^account][^account-conversations]

### 9.3 Draft handoff and business writes

The standard client-draft action is **Insert into composer**, not Send. Emit a
proposal carrying artifact/answer ID, target client/thread/channel, source and
expected destination draft revision, context revision and expiry. For the
existing prepared opener, insertion must additionally require an empty destination.
If the human has typed meanwhile, preserve their work and ask them to resolve the
conflict; do not replace it silently.

The host inserts into its existing editable composer and acknowledges the new
draft revision. Only that acknowledgment permits an “Inserted” receipt. Subsequent
editing is distinct from saving or sending. Existing consent, recipient validation,
template, channel-readiness, idempotency and unknown-delivery locks remain in the
host workflow. No “sent” receipt is inferred from an insertion or generated text.

Similarly, prepare-assignment, note, next-action and escalation commands are
handoffs to existing workflows, not generic action executors. Keep unsupported
proposal kinds non-actionable and explain why. The workspace never directly calls
a CRM/channel API or bypasses its human confirmation/read-back path.

Feedback separates inserted, edited, discarded and flagged outcomes currently
supported by Office from future rating/adjudication projections. An insertion is
not proof of usefulness, a flag is not an established factual error, and feedback
does not automatically create a memory or an implementation Bead.[^turn-contract]

## 10. Engines, connection, and settings

Use one standard Settings view across adopting assistant surfaces. Display the
host's engine list, selected and effective preference, connection condition and
precise availability reason. Do not infer readiness from an engine name or a
stored preference. Current Office IDs map to Groq, Claude Code and Codex, but the
LDUI UI must not hardcode subscription products or account entitlements.

Save follows **proposal -> pending -> host write -> read-back -> accepted**.
On failure, preserve the previous accepted preference and the unsaved proposal.
The existing dialog's 204 response is not JSON. Optional warming after a successful
save is not permission to roll back a confirmed preference or fail over engines.
The legacy dialog writes `preferences.memoryEnabled`; the newer memory-policy
contract separately exposes use and capture. Office must supply one reconciled
accepted projection instead of assuming those fields are interchangeable or
treating the old toggle as consent to capture.
Changing a preference cannot rewrite the attribution of an in-flight or old turn.
The source provenance gap in section 3.1 must be resolved by the host; the adapter
must not populate an “actual engine” field from the requested/current preference
or Leader's legacy model label. Cache reuse also needs an explicit provenance
mode rather than a fabricated provider call.[^provenance-gap]

Sign-in/out are typed host intents and status projections. Office chooses a
permitted provider-managed browser flow or an honestly labeled operator-assisted
alternative. This component never accepts or stores provider tokens/passwords,
launches a CLI, or treats one trial subscription as a shared employee credential.
Provider auth, privacy, entitlement and retention decisions remain D-02–04.

Connection success, per-engine availability, reasoning authorization and available
capacity are separate conditions. The observed disabled sign-in and export/forget
controls must not become apparently functional merely because a new UI renders
them. They need completed host contracts and tested read-back journeys first.

Usage distinguishes reported tokens, unknown values, estimated/equivalent value,
verified incremental charge, fixed subscription cost and allocation. Never add
API-equivalent usage to subscription cost and label it actual spending. Show a
budget/cap only when the host supplies a configured policy; do not introduce an
unapproved daily cap or upgrade/overage purchase action.[^settings][^platform]

## 11. Memory, Group Important, and foundations

Memory is a governed workspace, not a switch labeled “the AI remembers you.”
Use three named views: **My Memory**, **Group Important**, **Foundations**.
Supplied page context and retained answer evidence are separate from all three.

### 11.1 My Memory

Show typed language, tone, verbosity and working-method entries, their state and
revision. Memory use and capture are separate policies; use-enabled is not
consent to automatic extraction. Initial automatic extraction remains off unless
the approved host policy and implementation explicitly support it.

Create/confirm/correct/forget operations require named pending and read-back
states. Candidate is not confirmed; conflicting revisions require refresh/review.
Current manual create produces a confirmed item; the existence of a candidate
state and capture-policy fields does not prove an automatic extraction pipeline.
Current DTOs expose create/confirm/forget, not a complete edit/export/erase-all
contract, so the adapter must declare each supported journey independently.

Do not offer unrestricted “remember this client fact.” The current boundary
rejects contact details, financial facts, legal identifiers and psychological
inferences; display the typed class without echoing prohibited content into
diagnostics. The UI's validation is assistance, not proof the server has accepted
the content.[^memory]

Current per-item forgetting withdraws the item and advances epochs; it is not
physical erasure. The engine sign-out request's separately named `forgetMemory`
means CLI-worker-directory cleanup, **not** forgetting Office-governed memory.
Do not combine these controls or reuse the same success message.[^memory]

Forgetting must distinguish “removed from assistance” from physical purge,
legal hold, backup expiry or provider retention. Display host receipts and any
remaining restrictions. The browser cannot promise “deleted everywhere.”

### 11.2 Group Important

Provide an explicit review step showing proposed text, title/canonical question,
EN/ES aliases, source/lineage and the intended all-Office audience. Submission
creates **pending review**, not publication or immediate cross-user recall.

Separate submitter, reviewer and publisher capabilities. Before curator approval,
the host must provide the exact proposed revision, sources and comparison to
current published content. Metadata-only or published-text-only projections are
insufficient. Publish, withdraw, reject and request-changes each require a
supported host action and conflict-safe receipt; do not present an unsupported
backend mutation as available.

Never silently turn a private memory or client-containing answer into shared
knowledge. Publication and redaction policy, permitted audience, source authority
and retention are Office responsibilities. The component provides the explicit
human review and truthful status rendering.

### 11.3 Foundations and invalidation

Display required domains, source owners, published revision/effective date,
review dates and incomplete coverage. Missing or withdrawn foundation content
is a visible condition; no embedded fallback may silently resurrect it.

Memory withdrawal/forgetting can invalidate cached answers and in-flight output.
The host must recheck relevant generations before releasing an answer or enabling
a proposal. The workspace immediately respects a newly supplied invalidation or
access revocation. Merely hiding a memory row is not evidence that derived answers
have stopped using it.[^platform][^memory-gap]

## 12. History, Activity, and Learning Review

History is an authorized application projection, never a provider transcript
browser. Show original scope, actual engine, time, outcome and retained-evidence
availability. Distinguish unavailable retention/access from no conversations.
Loading a past conversation does not load its old facts as current authority or
silently start another model turn. History pagination/search/filter/close behavior
requires explicit Office contracts; it cannot be inferred from per-attempt replay.

Activity separates employee-visible usage from reviewer/operator evidence. A
content publisher does not automatically gain raw questions, private memories or
personnel drill-through. Ordinary operator health can use metadata without text.

The permissioned **Learning Review** area covers the required platform target:

- Daily inventory coverage, answer/attempt assessment coverage, cutoff/time zone,
  late-arrival catch-up and named missing partitions; failure/refusal/cancellation
  counts are included, not only successful answers.
- Separate deterministic inventory completion, reused evaluation evidence,
  model assessment, human-review backlog and publication completion.
- Partial, missed, unconfigured-policy, unassessable and capacity-paused states
  with cause, oldest outstanding work, owner and proposed recovery action.
- Correct/incorrect/incomplete/unanswerable/appropriate-refusal/unassessable
  classifications, while keeping user feedback, model grading and human
  adjudication separately attributed.
- Root-cause candidates, contradictions, proposed corrections/merges/retirements,
  source gaps and foundation gaps; curator comparison before semantic changes.
- Improvement progression from detected through triage, ownership, proposal,
  approval, implemented/published, verified and monitored, or a recorded
  rejected/deferred disposition; released version and regression evidence are
  distinct from an issue being filed.
- Recurrence with opportunity counts and use-case/language/engine scope; no
  comparable new questions is not proof that a defect is fixed.
- Maintenance usage and foreground capacity protection; authorized suspend/resume
  proposals with actual host acknowledgment and preserved ordinary Office work.

The UI does not run daily review, schedule a job, query Postgres, rank memories,
publish content, create Beads, send external alerts or purchase capacity. Office's
supervised job, permitted local reporting projections, review/records policy and
operational owners must exist. A view cannot satisfy MEM-05–08 through placeholder
counts. These are target capabilities requiring Office review.[^platform]

## 13. Failure, privacy, localization, and accessibility

### 13.1 Required failure presentation

| Condition | Required behavior |
|---|---|
| Capability denied / tier reasoning off | Named unavailable explanation; no model request; ordinary page work remains usable |
| Identity, scope or policy unresolved | Fail closed for the affected operation; do not guess a person, office, matter or retention rule |
| Evidence admission unavailable | Retain question; show no admitted attempt; never claim a completed answer from a partial preview |
| Submission outcome unknown | Preserve draft and correlation; resolve original request before offering a new potentially billable attempt |
| Stream disconnected / journal unreadable | Retain acknowledged content, mark recovery status, use snapshot/replay; do not re-ask automatically |
| Validation failure / stale generation | No actionable final output; show display-safe reason and preserved historical content where permitted |
| Cancel requested / boot interrupted | Separate pending cancellation from terminal cancellation and interruption; ignore stale completion |
| Provider unavailable / quota or capacity refusal | Show actual engine and reason; no alternate provider, automatic upgrade or hidden maintenance priority |
| Settings/memory/proposal conflict | Keep accepted state and unsaved proposal; refresh exact item/revision and require human reconciliation |
| Draft target edited or expired | Refuse insertion; preserve destination text, expose review/refresh path, do not overwrite |
| Send/save result unknown | Keep the host's uncertainty lock; do not label success or expose a duplicate business action |
| Source withdrawn / history restricted | Remove ineligible content and actions; preserve only permitted disposition metadata |
| Learning coverage partial / metrics absent | Name incomplete/unassessable/unknown; never display an all-clear or zero from absent evidence |

Errors use typed codes plus sanitized, actionable user-facing reasons. Do not
send raw provider stderr, prompt payloads, secrets or client text to browser logs,
telemetry, screenshots or an unrestricted error inspector. Authorized detailed
diagnostics belong to the host's governed support path.

### 13.2 EN/ES and accessible interaction

All UI copy, state labels, errors, help, evidence captions, dates/numbers and
screen-reader announcements use a complete EN/ES text contract. Engine/source
identities and opaque references are not translated. Preserve language changes
for the current interface without rewriting historical source text; a requested
translation is a separately attributed draft revision.

Use labeled sections and a sequential transcript/log pattern, but do not announce
every token. Announce concise progress and completed-response status without
moving focus; allow the reader to inspect the response deliberately. Status
messages must be programmatically exposed, and modal detail views must manage
focus, keyboard containment and return appropriately.[^a11y-status][^a11y-dialog][^a11y-log]

Escape closes the top disclosure/modal according to LDUI's controlled modal
contract; do not make it silently cancel a paid turn while closing an inspector.
Provide an explicit accessible Stop control. Preserve a visible focus indicator,
logical tab order, native Select keyboard behavior, reduced motion, scalable
text, non-color-only states and sufficient contrast. Collapsing a panel with
pending work must not lose the draft or imply cancellation.

Use one active layout tree or rigorously inert hidden variants. Verify narrow
containers, expanded workspace, EN/ES wrapping, long identifiers, long errors,
large type/zoom and mobile keyboard space. Scope/evidence cannot disappear just
to make the header fit. Use semantic theme tokens and existing visual-quality
rules; do not hand-edit generated `styles/tokens.css`.[^visual]

## 14. Acceptance evidence required after approval

This section specifies future proof obligations. **None of these application
tests was run or claimed passing during this research.** Follow the existing
LDUI and Office harnesses and A/B/C/D methodology; do not introduce a parallel
test runner or a long unbounded “prove it works” loop.[^visual][^testing]

| Layer | Required evidence |
|---|---|
| A — Visual | Reviewed narrow/wide, EN/ES, theme/zoom and long-content captures for each meaningful state; intentional visual changes reviewed, not blindly accepted |
| B — Model and DOM | Pure model/guard tests and browser DOM checks for exact state, count, scope, selection, draft, artifact and acknowledgment semantics |
| C — Accessibility | Accessible names/roles, focus order/return, keyboard-only journeys, announcement behavior and no hidden duplicate controls |
| D — Effects and callbacks | Exact command payload, request/attempt/context/revision identity, callback counts, prohibited effects absent, host receipt/read-back and real adapter correlation |

Critical journeys include: refused admission retaining a question; snapshot/replay
without a second model call; duplicate/out-of-order events; context and permission
changes during a turn; memory withdrawal before release; late answer after cancel;
unknown wire enum; terminal-without-valid-answer; settings save/read-back failure;
provisional text with no actions; draft insertion conflict; expired/exact-template
identity; uncertain send; Group Important pending versus published; curator
metadata without reviewable content; retained-under-policy forgetting; and partial
daily coverage without a green completion claim.

Use inject/catch/revert negative controls for stale-action dispatch, duplicate
model-call retries, optimistic publication, absent-as-zero reporting, hidden
duplicate controls and unverified preview insertion. Verify each detector catches
the injected defect and then returns clean after revert. Source tests alone do
not prove rendered controls or provider isolation.

The shared-library demo/harness uses synthetic facts and a deterministic fake
host that can delay, refuse, reorder and invalidate acknowledgments. It must not
require paid provider access. Office separately qualifies its real adapter,
authorization, server contracts, evidence lifecycle and permitted provider paths
using approved data. Test plan and execution stay within the relevant owner's
authorization and are proportional to the eventual changed surface.

## 15. Integration and compatibility constraints

The proposed shared location is `src/components/ai_assistant_workspace/`, following
the repository's component/type/text/test conventions. This is a proposed boundary,
not authorization to create those source files now. Keep the public facade small;
internal subviews are not automatically independent public APIs.

Office's mapping belongs in shared satellite UI or a narrowly scoped adapter
module, with each satellite retaining its page context and business workflow.
Do not rebuild a universal page registry or move Office business decisions into
LDUI. Mount the facade against typed page capabilities, including availability
and role/scope restrictions, rather than merely copying a gear to four screens.

Migration must account for every inventory row: one-shot Dashboard/Journal,
local Coordinator transcript, conversation analysis/Q&A/drafting, prepared
opener, No-Hire script editing, and Account/Account Conversations call preparation
and suggestion reconciliation. An old surface should not be removed until
its specific interaction and safety contracts have replacement evidence. Whether
Account contextual chat and Learning Review join the first release is an Office
approval/dependency question; they must not be claimed delivered by the initial
UI facade alone.

Office vendors LDUI. After approved source implementation, Office adoption must
pin a committed shared-library revision, update machine-readable provenance,
preserve declared Office deltas, and requalify the affected satellites. A clean
sibling build does not prove the vendored consumer has changed. No vendor patch
or re-vendoring was performed for this document.[^vendor]

## 16. Office-agent review and return contract

The Office agent should amend this file in place and preserve its source/proposal
distinction. A review reply is not permission to create Beads unless the user
then approves proceeding. Record the reviewed Office revision and distinguish
technical acceptance from unresolved business-owner decisions.

| Review item | Required disposition before dependent work |
|---|---|
| Shared boundary and name | Accept/amend `AiAssistantWorkspace`, fixed anatomy, controlled state/command API and the prohibition on shared business/provider execution |
| Inventory and source drift | Confirm current mount points, Account/call-preparation coverage, generated-artifact ownership and any changes since `a27d3fcc` |
| Replacement fidelity | Confirm the one-shot/local-history migration policy and preservation of every conversation/prepared-contact safety contract |
| Host API readiness | Identify accepted contracts or existing work for history, typed-turn adoption, memory/context fingerprints, pending-candidate review, edit/export/forget, auth and learning projections |
| Requirement coverage | Under D-01, ratify/amend CO/LD/AC/CV/SH/MEM mapping and first adoption boundary without silently removing target capabilities |
| Existing versus new scope | Under D-11, preserve approved Journal behavior without adding new contextual/day-planner scope; under D-16, confirm reuse/adopt/build conclusions against current code and actual UI/runtime evidence |
| Authority and data | Carry forward BRD D-02–04, D-07–08, D-12, D-17–20: named provider users, data recipients, channel rights, completeness, evidence access, worker/matter separation, records/holds and disclosure |
| Memory governance | Resolve responsible owners/defaults/rights under D-05, D-14–15, D-18; distinguish backend DTO states from the full target lifecycle |
| Generated-content settings | Resolve D-09 scope of selected engine; preserve actual provenance regardless of that decision |
| Pilot and operations | Carry D-06, D-10, D-13, D-21: participants/capacity, operational priority, quality/latency/benefit/harm thresholds and suspension authority |
| Proof and vendoring | Accept/amend A/B/C/D acceptance cases, responsible repo for each gate, synthetic-data policy and immutable vendor adoption |

Return a concise accepted/amended/deferred disposition, the remaining contract
gaps with source references, and any business decisions that still gate particular
features. Link existing Office work where available; do not duplicate it. Only
after this review and the user's approval should the primary LDUI agent turn the
agreed design into implementation-ready Beads and begin coding.

## 17. Source register

Local source links are pinned by the repository revisions in section 2; line
anchors describe that snapshot and may move during the Office review. Office
specifications are requirements/design evidence, not runtime proof. External
sources were limited to primary W3C accessibility guidance; no private Office
content was submitted to external research services.

[^v1]: Office, [controlling V1 business requirements](C:/dev/4iiz-office/docs/business-requirements.md:79), §§3.2–4.5. Written business approval, no unapproved reprioritization, reliable workflow/data prerequisites and user-observable completion.

[^brd]: Office, [AI Assistant business requirements](C:/dev/4iiz-office/docs/superpowers/specs/2026-09-09-ai-assistant-business-requirements.md:9), draft dated 2026-09-09. Authority/owner amendment at lines 20–34 and 115–121; CO 174–224; LD 230–280; AC 286–336; CV 342–421; SH 427–489; MEM 495–588; decision register 681–708. Requirement coverage and approval boundaries, not current production certification.

[^platform]: Office, [AI Assistant platform design](C:/dev/4iiz-office/docs/superpowers/specs/2026-09-09-ai-assistant-platform-design.md:36), revised 2026-09-09. Authority/selected decisions §§1.2–1.3; ownership/context §§4–5; connections/lifecycle §6; memory/recall §§7–9; evidence §10; daily review §11; forgetting/holds §12; usage/operations §13; API/acceptance §§14–17.

[^comparison]: Office, [AI Assistant design comparison](C:/dev/4iiz-office/docs/superpowers/specs/2026-09-09-ai-assistant-design-comparison.md), 2026-09-09. Historical comparative review; subsequent revised platform decisions and current code take precedence over its earlier repair findings.

[^dashboard]: Office, [dashboard coach](C:/dev/4iiz-office/surfaces/dashboard/src/dashboard.rs:2040) and [request/response adapter](C:/dev/4iiz-office/surfaces/dashboard/src/api.rs:196). State/request at dashboard lines 1557–1561 and 1714–1750; UI 2040–2184; response parsing api lines 504–554.

[^coordinator]: Office, [Coordinator assistant panel](C:/dev/4iiz-office/surfaces/coordinator/src/coordinator.rs:4356) and [assistant request](C:/dev/4iiz-office/surfaces/coordinator/src/api.rs:494). Local history 1234–1239; scoped ask 2920–2966; grounded response 3174–3237; availability 3871–3947; panel/config 4356–4519.

[^journal]: Office, [Journal assistant](C:/dev/4iiz-office/surfaces/journal/src/journal.rs:1268) and [request adapter](C:/dev/4iiz-office/surfaces/journal/src/api.rs:188). Input/answer model 169–202; scope/response 560–617; ask flow 862–896; editing 949–963; UI 1268–1349.

[^conversation]: Office, [conversation-detail assistant and draft UI](C:/dev/4iiz-office/surfaces/conversation-detail/src/conversation_detail.rs:3427) and [API adapter](C:/dev/4iiz-office/surfaces/conversation-detail/src/api.rs:1081). Human-dispatch boundary 448–452 and 518–526; generated-artifact handlers 1939–2104; analysis/Q&A/draft rendering 3427–3736.

[^prepared]: Office, [prepared-contact state and interaction](C:/dev/4iiz-office/surfaces/conversation-detail/src/prepared.rs:163). State guards 163–241; single initial opener/request handling 1217–1311; use/edit/expiry interaction 1534–1597. Prepared suggestions and the human-send workflow are separate.

[^call-script]: Office, [No-Hire script state](C:/dev/4iiz-office/surfaces/no-hire-detail/src/no_hire_detail.rs:142), edit/read-back handlers 1040–1110 and rendered script 1755–1924; [rich call-script DTO](C:/dev/4iiz-office/crates/office-perf-dto/src/calls.rs:108); [AI architecture: cache API and script UI contracts](C:/dev/4iiz-office/docs/ai-architecture.md:758). Architecture §§13–14 records read-only cache GET, ordered script beats, language and missing-fact safeguards; historical UI paths in that document are not active mount-point evidence. DTO/documented richness is not proof every field is currently rendered.

[^account]: Office, [active Account page](C:/dev/4iiz-office/surfaces/account/src/account.rs:1582), script/draft reconciliation 1582–1617 and 1681–1716, polling/regeneration 1801–1888, preparation 1896–1940, call/composers 3151–3412; [draft reconciliation model](C:/dev/4iiz-office/surfaces/account/src/model.rs:713). Mounted by `surfaces/account/src/main.rs`, registered at `surface_portfolio.rs:618`.

[^account-conversations]: Office, [Account Conversations](C:/dev/4iiz-office/surfaces/account-conversations/src/account_conversations.rs:2355), preparation/composers 2355–2548 and softphone/contact area 2573–2734; [draft reconciliation model](C:/dev/4iiz-office/surfaces/account-conversations/src/model.rs:372); call-script API read at `surfaces/account-conversations/src/api.rs:249`.

[^settings]: Office, [shared AssistantConfigDialog](C:/dev/4iiz-office/crates/office-perf-satellite-ui/src/assistant_config_dialog.rs:1). Read/204/read-back and disabled journeys 1–25; pure model and state 302–625; request wiring 657–1042; presentation 1097–1470. Existing gear/indicator helpers at 1047/1071 are not proof of adoption by all surfaces.

[^turn-contract]: Office, [assistant turn DTO contract](C:/dev/4iiz-office/crates/office-perf-dto/src/assistant_turns.rs:37). States/events/capabilities 37–126; bounds 128–151; conversation/turn submission 155–203; admission/events/snapshot/feedback 205–284. Consult constants rather than inaccurate count prose in nearby comments.

[^turn-runtime]: Office, [durable attempt runtime](C:/dev/4iiz-office/crates/office-perf-api/src/assistant_turns.rs:497) and [turn SSE/snapshot routes](C:/dev/4iiz-office/crates/office-perf-api/src/routes_workspace/assistant_turns.rs:537). Journal-first publication, execution/cancel 497–742; interruption recovery 1204–1235; SSE replay/lag handling and snapshot routes 537–722.

[^routes]: Office, [registered assistant routes](C:/dev/4iiz-office/crates/office-perf-api/src/app_router.rs:796) and [satellite operation allowlist](C:/dev/4iiz-office/crates/office-perf-api/src/surface_portfolio.rs:28). Engine/config 796–825; memory 826–867; typed conversations/turns 868–895; legacy coach routes remain separately registered.

[^memory]: Office, [memory DTOs](C:/dev/4iiz-office/crates/office-perf-dto/src/assistant_memory.rs:23), [memory service](C:/dev/4iiz-office/crates/office-perf-api/src/assistant_memory.rs), and [memory routes](C:/dev/4iiz-office/crates/office-perf-api/src/routes_workspace/assistant_memory.rs). Typed kinds/states/rejection classes 23–69; policy/item projections 85–162; writes/receipts/knowledge content 164–279. `KnowledgeEntryDto` exposes published text, not a pending-review body. Manual confirmed creation is at route lines 601–671; withdrawal 765–804; published-only read 1116–1204. Compare [simple conversation Q&A](C:/dev/4iiz-office/crates/office-perf-api/src/conversation_assistant.rs:86) with recall in draft/analysis/opener paths. [Engine sign-out](C:/dev/4iiz-office/crates/office-perf-api/src/routes_workspace/assistant_engines.rs:674) has a different CLI-directory cleanup contract.

[^memory-gap]: Office, [typed-turn admission fingerprint](C:/dev/4iiz-office/crates/office-perf-api/src/routes_workspace/assistant_turns.rs:403), explicit empty memory component at lines 403–415; compare [Coordinator memory recall](C:/dev/4iiz-office/crates/office-perf-api/src/coordinator_assistant.rs:109). This is an observed integration gap, not proof that the memory subsystem is absent.

[^provenance-gap]: Office, [typed runner drops `EngineTurn`](C:/dev/4iiz-office/crates/office-perf-api/src/assistant_turns.rs:903), [executor re-reads preference](C:/dev/4iiz-office/crates/office-perf-api/src/state.rs:404), and [Leader's fixed model label](C:/dev/4iiz-office/crates/office-perf-api/src/leader_assistant.rs:411). Admission records requested engine at `routes_workspace/assistant_turns.rs:379`; the older [append journal](C:/dev/4iiz-office/crates/office-perf-queries/src/assistant_journal.rs:232) can record actual execution details but does not supply the missing typed-turn/UI projection.

[^engines]: Office, [engine contract](C:/dev/4iiz-office/crates/office-perf-conn/src/assistant_engine.rs:4), [engine/config DTO](C:/dev/4iiz-office/crates/office-perf-dto/src/assistant.rs:62), and [bounded engine sessions](C:/dev/4iiz-office/crates/office-perf-api/src/assistant_engine_sessions.rs:1). Source boundaries and availability do not establish live provider entitlements or runtime isolation certification.

[^ldui-chat]: LDUI, [AiChat](../../../src/components/ai_chat/component.rs), especially lines 48–153, 196–240 and 246–280; [SSE bridge](../../../src/components/ai_chat/transport.rs), especially lines 42–95 and 128–169. Session ownership/settings/send/cancel/restart semantics differ from Office's durable-attempt protocol.

[^ldui-call]: LDUI, [ClientCallWorkspace controlled command boundary](../../../src/components/client_call_workspace/component.rs:8) and [host contract documentation](../../../doc/components/client_call_workspace.md). Reuse the state/guard/command/acknowledgment pattern, not its telephony business model.

[^vendor]: Office, [vendor update/provenance contract](C:/dev/4iiz-office/vendor/README.md:76) and [machine-readable provenance](C:/dev/4iiz-office/vendor/PROVENANCE.json). The latter is authoritative for pinned files and declared deltas; historical consumer lists are not current satellite mount points.

[^visual]: LDUI, [visual-quality rulebook](../../../doc/visual-quality/README.md) and [CI/CD gate documentation](../../../doc/ci-cd.md). Follow scoped release browser hosts, zero-slack ceilings, negative controls and the final-candidate gate cadence.

[^testing]: PixelProof, [central A/B/C/D application testing methodology](C:/dev/PixelProof/docs/methodology/README.md) and [web surface methodology](C:/dev/PixelProof/docs/methodology/surface-web.md). Applied here to future acceptance design, not as evidence of tests run.

[^a11y-dialog]: W3C WAI, [Dialog (Modal) Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/), accessed 2026-09-10. Focus placement/containment, Escape behavior and logical focus return for actual modal dialogs.

[^a11y-status]: W3C WAI, [Understanding SC 4.1.3: Status Messages](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html), accessed 2026-09-10. Programmatic exposure of waiting/results/error status without requiring focus movement.

[^a11y-log]: W3C WAI, [ARIA23: Using role=log to identify sequential information updates](https://www.w3.org/WAI/WCAG22/Techniques/aria/ARIA23), accessed 2026-09-10. Reference for sequential transcript semantics, not a requirement to announce individual stream tokens.
