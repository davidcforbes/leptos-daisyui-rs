# Office calling adoption extensions — 2026-09-09

This record covers `ldui-26tv`, the framework response to Office `op-zhksn`.
The [component guide](../components/client_call_workspace.md) defines the updated
API and migration. The [research addendum](../research/2026-09-09-office-softphone-workspace.md#office-adoption-review-addendum)
links the Office source evidence. Office source and vendor pins are unchanged.

## Candidate scope

The candidate adds typed refusal evidence, blocked saved numbers, a controlled
contact target selector, ordered script beats and preparation state, an optional
regeneration mutation, and independent provider talk time. Calling enums are
non-exhaustive, with downstream command matches rejecting unknown operations.
The original simple guidance and single-field contact mode remain available.

Native tests exercise evidence, payload and capability guards. Release browser
tests exercise real selection and acknowledgment, blocked callbacks, script
states, regeneration concurrency/stale context, measured time and responsive
geometry. Scoped visual ceilings remain zero; screenshots require human-style
visual review as well as structural assertions.

## Adoption evidence

| Office ask | Implemented contract and proof |
|---|---|
| Distinct refusal evidence | Typed preflight/provider/not-submitted refusals retain separate status values; browser retries each and verifies that Uncertain still locks dialing |
| Number eligibility | Disabled saved-number choice retains its explanation; native and forced-click browser probes reject blocked selection and dialing |
| Structured script | Ordered beats retain titles, spoken text and the no-file-detail marker; browser observes Ready, measured Preparing and Failed states |
| Regeneration | Separate request state rejects duplicates, keeps Accepted distinct from completion, handles comparison conflict and rejects stale-context completion |
| Contact target choice | Native Select keyboard interaction chooses an authorized target; rejection restores controlled selection and acknowledgment updates the exact saved field |
| Provider talk time | Browser distinguishes unknown, confirmed zero and 125 seconds; a reversible readout corruption exercises the oracle and manual minutes remain independent |
| Extensible enums | Public calling enums are non-exhaustive; downstream demo matches compile and unknown commands do not reserve requests; migration is documented |

## Verification status

| Check | Result |
|---|---|
| Workspace native contracts | 13/13 passed with `cargo test -p leptos-daisyui-rs --lib client_call_workspace` |
| Softphone native contracts | 10/10 passed with `cargo test -p leptos-daisyui-rs --lib components::softphone::types::tests` |
| Blocked-Dial negative control | Removing its predicate caused the targeted regression test to fail; restoring it returned the suite to green |
| Showcase type check | `cargo check -p leptos-daisyui-showcase` passed after correcting draft-duration introspection |
| First release browser lane | 6/7 passed; target/refusal/duration test stopped on a duplicated JavaScript `const` binding |
| Browser harness corrections | Scoped repeated JavaScript bindings in local functions; preserved and restored the original reactive Text node in the duration negative control |
| Rendered evidence | Wide/mobile rich-script audits and axe checks passed; both screenshots reviewed |
| Independent review | Fixed unknown-command reservations and absent regeneration capability handling after client switch; no unresolved findings |
| Final `cargo xtask verify-full` | 40/40 passed on the corrected test candidate; wrapper exit `0` |

The final gate passed all 40 steps. Its native stages included 3,267 library
tests, 79 xtask tests, 17 audit tests and all source guards. Formatting, four
scoped clippy checks, build/check and the ordinary sibling-token check passed;
the latter resolved all 41 referenced tokens on the sibling's default branch.
The corrected release Softphone lane passed 7/7.

The first browser log is `.tmp/call-adoption-first-browser.log`. The final gate
log is `.tmp/call-adoption-verify-full.log`; its wrapper records the real process
exit as `CALL-ADOPTION-GATE-EXIT`. The gate uses the existing xtask orchestration;
no stylesheet or audit ceiling bypass is introduced. A pre-gate manifest records
775 Rust source/build inputs for comparison after the run. During the unrelated
browser suites, independent review identified one test-only amendment: assigning
the duration element's `textContent` detached Leptos's reactive Text node. The
negative control now mutates and restores that same Text node, then checks the
next host duration update. This amendment was formatted and checked before the
softphone suite compiled. The source proof records its exact hash separately;
the other 774 inputs, including application source, must remain unchanged.

Reviewed captures: [wide Office scenario](client-call-workspace-2026-09-09/office-wide.png)
and [mobile Office scenario](client-call-workspace-2026-09-09/office-mobile.png).
The disabled mobile choice retains a visible reason, script beats remain in
order, and the contact target and call record fit their narrow container.

The fixture proves independent concurrent call/regeneration requests and stale
context rejection. It does not simulate old same-context operation-token replay;
production adapters must implement that additional correlation contract.

## Operational boundary

These are controlled UI tests against a simulated host. They do not establish
Office vendor adoption, CTM integration, microphone/media permissions, CRM
writes, background polling or durable request correlation. A provider-received
refusal is distinguished from a preflight that attempted no POST. Regeneration
acceptance is distinguished from a completed script. Talk time is distinguished
from elapsed session time and manually recorded minutes.

The earlier verification record documents a historical no-Git session limit.
The current hook explicitly permits ordinary project Git and only excludes
Beads Git/GitHub features. Current verification therefore runs the normal
sibling-token check. The earlier recovered footer work remains in this working
tree and is included in final verification and preservation.
