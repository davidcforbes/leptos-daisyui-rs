# ClientCallWorkspace verification — 2026-09-09

Scope: reusable library composition, simulated showcase and local Office source
research (`ldui-i1ui`). No Office files, provider sessions or contact records were
changed. The [component guide](../components/client_call_workspace.md) defines
the public contract; the [research report](../research/2026-09-09-office-softphone-workspace.md)
records the evidence and design choices.

## Evidence collected

| Check | Result |
|---|---|
| Native command and state contracts | 5 passed with `cargo test -p leptos-daisyui-rs --lib client_call_workspace` |
| Showcase type check | `cargo check -p leptos-daisyui-showcase` passed |
| Initial release softphone lane | 3/4 passed; fourth reached the visual audit after wrap-up and managed-call interactions |
| Initial visual audit | Failed on 14 inherited 6px gaps and 4 inherited inset shadows; no overlap, typography, shape or internal-spacing violations |
| Independent code review | Fixed orphaned simulator writes and missing required-control semantics |
| Independent research/contract cross-check | No material mismatch; refreshed a moving Office source reference |
| Final `cargo xtask verify-full` | Exit 0; runner reports 40/40, including the skipped Git-dependent sibling check described below |
| Final native library suite | 3,258 passed, including all five workspace contract tests |
| Final release softphone lane | 4/4 passed: two existing Softphone tests and two workspace tests |
| Regression browser suites | Reactivity 69/69, layout 15/15, style 13/13 and server-table column tools 7/7 passed; all other required browser suites passed |
| Workspace rendered checks | Zero-ceiling scoped visual audits, serious/critical axe checks, responsive geometry and desktop-panel negative control passed |
| Screenshot review | Wide, mobile and narrow desktop-panel captures reviewed; controls and help text fit their containers |

The first browser result is preserved in `.tmp/client-call-first-browser.log`.
It demonstrated that the new zero-ceiling visual oracle detects real rendered
differences before the composition applied its explicit field/button treatment.
The final gate log is `.tmp/client-call-verify-full.log`.
The wrapper's final `CLIENT-CALL-GATE-EXIT=0` and the completed process exit code
both confirm success. The command was run through
`pwsh -NoProfile -File .tmp/client-call-gate.ps1 -Gate verify-full`, which invokes
the existing xtask with Git excluded from its process environment.
After the gate, all 18 Rust files in the gate-start recovery archive still
matched their saved SHA-256 values. All three documentation screenshot copies
were byte-identical to the browser test output.

Reviewed release screenshots: [wide workspace](client-call-workspace-2026-09-09/workspace-wide.png),
[mobile workspace](client-call-workspace-2026-09-09/workspace-mobile.png) and
[375px desktop panel](client-call-workspace-2026-09-09/workspace-desktop-panel.png).

## Drawer-width failure and correction

An isolated Chrome DevTools page at
`http://127.0.0.1:59627/components/client-call-workspace` reproduced a narrow
desktop panel independently of the browser test process. At a 1442px viewport,
constraining the workspace to 375px left two approximately 155px columns and
horizontal overflow. This was a viewport-breakpoint error. A container query
restored one column; allowing the Field help label to wrap removed the remaining
overflow. Readback changed from width 373 / scroll width 487 to width 373 /
scroll width 373. The temporary DOM styling was used only to diagnose the issue;
the page was then closed and the actual Rust/Tailwind source was corrected.

The saved test source includes a desktop-panel assertion and a negative
control that forces two columns, observes the wrong column count, restores the
actual layout and rechecks one column. The source uses a 48rem component-width
threshold. Label/button gaps use the shared 8px step; inputs, selects and
textareas use flat surfaces. No audit ceilings were raised.

A fresh isolated page against the final release build confirmed the source fix
without a temporary stylesheet: a 375px workspace in an 1802px viewport had a
373px inner width and scroll width, one column, wrapped help labels and flat
field surfaces. A separate read after the simulated connection acknowledgment
confirmed managed/active state, an enabled End call control and a 02:05 timer.
Both isolated review pages were closed after inspection.

## Test boundary

The showcase counts every callback before its own command guard, and separately
counts confirmed writes. Browser tests compare controls, drafts, receipts and
acknowledgments. Keyboard outcome selection uses focus, Space, Home/Arrow and
Enter; synthetic events are reserved for adversarial disabled-control tests.

The tests establish controlled UI behavior only. They do not prove CTM, WebRTC,
Zoho, authorization or persistence integrations. The host must still correlate
asynchronous replies with context and operation tokens and preserve ambiguous
attempt evidence across dismissal.

## Preservation and exclusions

Source, research and documentation are saved in the working folder and in
hash-verified ZIP recovery archives under
`C:/Users/david/Documents/Codex Backups/leptos-daisyui-rs/`.
The archive script also includes the earlier recovered server-table footer work.

This session's injected no-Git rule prevents Git checks, commits and pushes.
The gate runs with Git unavailable; its `sibling-tokens` step reports a skip.
That skip is not evidence that a sibling repository lacks its remote metadata.
No completion claim here includes commit, push, branch-divergence verification
or a live Office deployment.
