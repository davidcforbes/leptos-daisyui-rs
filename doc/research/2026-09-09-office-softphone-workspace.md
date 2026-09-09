# Office softphone research and reusable workspace decision

Research date: 2026-09-09. Reference checkout: `C:/dev/4iiz-office`.
Implementation target: `leptos-daisyui-rs`, Bead `ldui-i1ui`.
This report describes inspected local source, not a verified production deployment.
Office was not modified and no telephone calls were made. Repository history was
unavailable under this session's no-Git instruction; file paths and named symbols
identify the evidence. The component contract is [ClientCallWorkspace](../components/client_call_workspace.md).

Method: indexed the 189-document Office docs corpus, used fused searches for
softphone, call panel, client numbers, outcomes and coordinator workflows, then
traced the matching satellite UI, DTO/API adapters and contact-write route.
Code is primary evidence; older plans and the September 9 draft are explicitly
separated from implemented behavior. A second read-only review cross-checked the
final claims against current source. Office is an active working tree, so named
symbols are more durable references than approximate line numbers.

## Recommendation

Build `ClientCallWorkspace` as a controlled, opinionated composition above the
existing `Softphone`. It should join the work that an operator performs around a
call: identify the client, review the reason for contact, choose or correct the
destination, launch the call, understand what the provider actually confirmed,
and record a deliberate outcome. Keep provider sessions, authorization, contact
writes, call dispatch and persistence in the consumer. This is an operational
workspace, with a fixed information hierarchy rather than a collection of slots.

The most consequential design decision is to distinguish a bridge request from
a connected media session. Office's coordinator can receive an accepted launch
while only the consultant's phone is ringing. A missing response can mean that a
call was dispatched. Neither result establishes client connection, elapsed talk
time, recording availability or permission to retry. `AgentRinging` and
`Uncertain` must therefore lock the destination and the launch action until the
host reconciles the attempt. They must not silently become `Active`.[^1][^2]

Three approaches were considered. Copying the Office drawer would reproduce its
useful destination editor but also its stale contact-update affordance and tight
coupling to account and coordinator state. Expanding the existing `Softphone`
would burden every small call console with case context and work completion.
A separate composition preserves the existing console's tested live controls
while defining the richer workflow for consumers that need it. The third option
is the implementation choice. The existing `Softphone` API remains useful on its
own and is not a deprecated or incomplete version of this workspace.[^3]

## Where the UI actually lives

The current rich reference is `surfaces/coordinator/src/coordinator.rs`,
particularly `CoordinatorPage`, its call ownership and attempt helpers, and the
right-hand `Modal` around lines 4518–4677. The account satellite has a second
implementation in `surfaces/account/src/account.rs`, around its
`open_softphone`, `place_call`, `softphone_state`, and modal rendering. These
satellites are more relevant than the retired `office-perf-web` tree or archived
screens. Both drawers keep client context visible, accept a telephone destination,
offer known numbers and show a status explanation. Closing the drawer dismisses
the surface; it is not a telephony hang-up.[^1][^2]

The coordinator adds an actual twelve-key destination pad, backspace, a number
update row and route-specific call guards. `DIAL_NUMBER_MAX_LEN` is 16 in this
snapshot. Its pad edits the destination; it does not send in-call DTMF. The
account drawer is simpler and does not establish that every coordinator feature
is already shared. The two implementations are evidence of a composition gap,
not two independent telephony engines.[^1][^2]

In LDUI, the existing `Softphone` already provides client identity, stable number
selection, lifecycle status, duration, mute, hold/resume, voicemail routing,
recording, transcription, DTMF and end-call controls. Its state is caller-owned.
Capabilities hide unsupported operations; pending requests and lifecycle rules
disable unavailable operations. Its clock deliberately includes held and
reconnecting time, and a stopped duration is supplied separately from the ended
phase. None of these controls connects a microphone or provider SDK.[^3]

## Dispatch and identity are part of the UI contract

`CallAttemptState` has six states: `Submitting`, `RingingConsultant`,
`AgentNotReady`, `AgentNotAvailable`, `NotSubmitted`, and `Uncertain`.
`from_result` checks structured dispatch evidence rather than interpreting a
message string. Only the two explicit agent refusals and `NotSubmitted` permit
another submit. A transport failure maps to uncertainty. This is a stronger
contract than a generic red error banner followed by an enabled Retry button.[^1]

The coordinator's `reserve_call` inserts `Submitting` synchronously before
spawning the request. It keys attempts by actor and route-specific target, so
closing and reopening a drawer cannot erase evidence of an accepted or ambiguous
request. `CallOwner` additionally binds the request to the signed-in actor,
viewed worker, generation, row key and case number. The asynchronous path checks
ownership again before dispatch and preserves the result against its original
attempt key. The account satellite uses a scoped action identity and checks it
again when finishing the action.[^1][^2]

The reusable boundary cannot reconstruct those application identities. It
therefore requires an opaque, nonblank `context_id`, carried on every command,
and exposes a pure `accepts` guard. The host must mint a new context when actor,
client, authorization generation or attempt changes. It must reserve pending
state before its callback returns, correlate asynchronous responses with its own
operation token, and reject stale responses. The component's context guard is
useful defense against stale UI commands; it is not authentication, server-side
idempotency or a substitute for operation correlation.

## Contact numbers: an important contradiction

The coordinator's `Update number` row is disabled with a tooltip referring to
`op-924qx` and a missing route. However,
`crates/office-perf-api/src/routes_workspace/matter_contact.rs` now implements
`POST /api/matter/:matter_id/contact/phone`. Its module documentation explicitly
identifies the softphone update operation. The route resolves the primary
contact on the server, validates the requested field, normalizes the number,
writes Zoho first and then updates the mirror and invalidation signals.[^4]

Consequently, the disabled row cannot be used as proof that the capability does
not exist. It is evidence that the UI has not yet consumed the route in this
local snapshot. The reusable component should expose an optional contact-update
capability with the exact field identity and visible field label, the last
confirmed saved value, pending status, a permission explanation and a failure
message. Typing a number must never silently change a contact. Clicking Save
requests an explicit write; the displayed saved value changes only on host
acknowledgment. Dialing is blocked during that write so the contact and call
destination cannot race.

There is a second mapping detail: coordinator `allowed_call_numbers` uses the
record's phone and alternate phone and explicitly excludes its mobile field in
that contract. The update route accepts `mobile` or `phone`. A consumer must map
its business fields deliberately rather than infer them from a familiar label.
The library uses opaque saved-number and update-field IDs.[^1][^4]

`input type="tel"` provides a telephone input surface, not a universal telephone
validation algorithm. Office's normalization rule includes a North American
ten-digit assumption; embedding that rule in an international component would
change its meaning. The new destination editor preserves host-formatted text,
caps it at 64 characters, and requires host validation feedback. The pre-dial pad
is an editing aid. The existing live keypad continues to emit one DTMF command
per permitted click and is never attached to global keyboard input.[^3][^5]

## Completion belongs beside the call, with separate acknowledgment

Office's coordinator work-completion flow defines six outcomes: Interested,
Not interested, No response / busy, Requested more information, Requested a
callback, and Invalid number. Its completion form requires an explicit result,
allows a trimmed note and accepts an optional positive duration in minutes.
The source comments explain why defaulting to an invented result, canned note or
five-minute duration was incorrect. Different work types use different complete
routes; selecting an outcome is not itself proof that any work item was
completed.[^6]

The new workspace adopts that six-outcome vocabulary with no initial selection.
Notes can be prepared before or during a call. Saving the outcome is enabled
only after a host-confirmed finished attempt or an ended managed call. Blank
duration remains absent. Invalid, zero, negative, fractional and overflowing
minute values cannot be submitted. Callback outcomes require a written follow-up
instruction identifying when and who should act; this is recorded text, not a
calendar booking. No provider duration is converted into an asserted manual
duration. The command carries a complete immutable snapshot; stale snapshots
are rejected by the current-state guard.

Saving notes is independently acknowledged. Failure preserves the draft and
allows retry. Pending save freezes those fields and blocks duplicate submits;
confirmation changes the surface to Saved. A close request carries the context
but clears nothing. The host decides whether to persist a draft or show its
existing unsaved-change confirmation before dismissing. The workspace itself
has no localStorage or cross-client draft cache.

## Guidance and the edge of scope

The active-conversations satellite design dated 2026-09-09 describes a draft
omnichannel workspace with coordinator-initiated SMS, WhatsApp, email and calls,
plus AI-suggested text. Its status is a proposal; it is not evidence that those
features are shipped in the softphone. The reusable calling surface can accept
a read-only guidance block with a title, plain-text body and source label.
That is sufficient for an operator's call objective or reviewed script without
pretending to generate advice or automatically act on it.[^7]

The earlier LDUI research dated 2026-09-04 explored a broader client call
workspace, but recorded a subsequent scope decision to implement only the small
Softphone. This new request explicitly authorizes the richer reusable component.
The old proposals are useful design history; they are not existing public
exports and should not be presented as such.[^8]

Incoming-call answer/reject, transfer, conference, device enumeration, provider
registration, caller-ID selection, SMS/email composition, AI generation,
transcript storage, recording playback and CRM navigation remain consumer
features. They are neither inert buttons nor implied capabilities here. If a
consumer needs them, it must add an explicit supported contract rather than show
a control whose behavior is undefined. Recording and transcription *requests*
already supported by Softphone remain available in managed calls with host
capabilities; this does not imply permission, storage or playback support.

## Information hierarchy and accessibility

The workspace has a named region, client heading, supporting identity and a
visible Close button. Below that, a responsive two-column layout places the
destination, launch status and live controls in the call column; guidance and
the work record occupy the second column. At narrow widths the same DOM stacks
in that reading order. There are no duplicated hidden forms, draggable widths,
floating layers, minimization engine or guessed viewport offsets.

Use existing LDUI Button, Input, Select, Textarea and Softphone primitives,
semantic daisyUI colors and the established typography and spacing scale. Status
text must identify uncertainty in words. Each field has a visible associated
label; its error is associated through the field contract. The status uses a
polite live region, while an elapsed clock does not announce every second.
Buttons remain real keyboard-operable buttons, and supported controls report
their actual disabled or confirmed state.[^3][^9]

The component is an inline region. Applications needing a right-hand drawer
compose it with LDUI Modal and retain responsibility for the opener, focus
return and dismissal proposal. A true modal must make background content inert
and contain keyboard focus; setting `aria-modal` on this inline workspace would
be misleading. Dismissal remains distinct from End call, which is available
during another pending managed control request.[^3][^10]

## Evidence requirements and adoption limits

Native tests must exercise call replay guards, stale context and payload
rejection, explicit contact writes, capability loss, managed end-call priority,
and outcome validation. Browser proof must send real input and compare the
rendered form, callback receipts and authoritative host state. The simulated
showcase needs accepted bridge, uncertain, refused, managed connected/held,
finished and wrap-up save/reject states. Screenshots and computed bounds must
cover wide and compact layouts. A deliberate negative control must demonstrate
that the new oracle actually detects a broken behavior.

Office adoption is a separate consumer change. Its adapter should preserve
`reserve_call`, ownership generations and structured dispatch evidence; map
safe server explanations to status text; enable contact update only when
authorization and field mapping are available; retain drafts outside the
drawer; and map SaveWrapUp to the correct work-kind route. This library work
neither changes Office's routes nor proves a provider integration. The most
valuable result is one coherent UI with those distinctions made explicit.

## Office adoption review addendum

Office's `op-zhksn` review and library follow-up `ldui-26tv` identified information
that the first composition compressed too far. The
[updated component contract](../components/client_call_workspace.md) expands the
model without moving provider operations into the UI.

The first model could display refusal prose in `status_detail`, but did not
preserve the distinction between local `AgentNotReady` preflight and a provider
`AgentNotAvailable` refusal after receiving a request. These are separate typed
reasons in [Office's call DTO](C:/dev/4iiz-office/crates/office-perf-dto/src/calls.rs),
alongside explicit no-dispatch evidence. Unknown responses must still lock retry.

The same DTO defines ordered, bilingual script beats with suggested lines and
`no_file_detail`, and a preparation state with elapsed/typical seconds and a
failure reason. A plain guidance block cannot faithfully present that structure.
The library now models the ordered content and preparation separately from an
optional regeneration mutation. Office's
[regeneration route](C:/dev/4iiz-office/crates/office-perf-api/src/routes_pool/scripts.rs)
refuses comparison-owned scripts with 409, which maps to visible failure and a
blocked action, not success.

Number eligibility and update targets are distinct. Office's
[matter ownership guard](C:/dev/4iiz-office/crates/office-perf-queries/src/matter.rs)
accepts Phone/Alt phone, whereas its
[contact update DTO](C:/dev/4iiz-office/crates/office-perf-dto/src/matter_contact.rs)
allows Phone/Mobile writes. A disabled saved-number explanation and a controlled
write-target selector preserve both facts. Consumers could already filter the
number list; the library did not force the mobile rejection defect.

`CallStateDto.talk_time` is an optional signed count of seconds. A validated
nonnegative measurement belongs in a separate provider talk-time readout, not
the manual minutes field or the ring-inclusive duration. Reading call state
does not establish that an adapter can perform browser media controls.

Structured payment promises and omnichannel orchestration remain consumer
composition decisions. The voice workspace does not invent a promise write
route or claim to implement SMS/email lifecycle behavior.

## Sources

[^1]: Local primary source: [coordinator.rs](C:/dev/4iiz-office/surfaces/coordinator/src/coordinator.rs), `DIAL_NUMBER_MAX_LEN`, `CallOwner`, `CallAttemptState` (around 2836), `allowed_call_numbers`, `reserve_call`, `submit_call` (around 3683), and the softphone modal (around 4518).
[^2]: Local primary source: [account.rs](C:/dev/4iiz-office/surfaces/account/src/account.rs), `open_softphone`, `place_call`, softphone modal, `saved_call_numbers`, `softphone_state`. Line positions changed during the research session; symbol references identify the intended implementation.
[^3]: [Existing Softphone contract](../components/softphone.md), [types](../../src/components/softphone/types.rs), and [component](../../src/components/softphone/component.rs).
[^4]: Local primary source: [matter_contact.rs](C:/dev/4iiz-office/crates/office-perf-api/src/routes_workspace/matter_contact.rs), module contract, `normalize_phone`, `update_matter_contact_phone`, `update_phone`; compare coordinator `UPDATE_NUMBER_ROUTE_BEAD` and its disabled button near line 4625.
[^5]: [MDN: telephone input](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/input/tel), telephone validation and input semantics.
[^6]: Local primary source: [coordinator.rs](C:/dev/4iiz-office/surfaces/coordinator/src/coordinator.rs), outcome mapping around 1382 and `confirm_complete` around 3450; [coordinator API](C:/dev/4iiz-office/surfaces/coordinator/src/api.rs), call and work-completion helpers.
[^7]: Local draft: [Active conversations satellite design](C:/dev/4iiz-office/docs/superpowers/specs/2026-09-09-active-conversations-satellite-design.md). Its proposed status is material to this report.
[^8]: [Earlier client-call research](2026-09-04-client-call-workspace.md), especially its scope update.
[^9]: [WAI-ARIA APG: button pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/), keyboard interaction and state semantics.
[^10]: [WAI-ARIA APG: modal dialog pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/), focus containment, labeling and dismissal.
