# Softphone UI implementation plan

Goal: provide the controlled Softphone composition described in
[the design](./2026-09-04-softphone-ui-design.md). Implementation progress is
tracked only by ldui-xmhn in Beads.

Architecture: pure Rust state/command contracts, Leptos rendering and a
simulated showcase. No provider dependency. Use existing Button/Select/Persona
and daisyUI 5 semantic tokens. Keep browser clock code behind a Wasm boundary.

The pure contract lives in `src/components/softphone/types.rs`, including
co-located native tests. It exports SoftphoneState, SoftphoneClient,
SoftphoneNumber, SoftphonePhase, SoftphoneTimer, SoftphoneCapabilities,
SoftphoneActionKind, SoftphoneAction and SoftphoneCommand. Test guards and
timer boundaries red/green before accepting behavior. Run its standalone Rust
tests while rendering is under development, then the scoped library tests.

`component.rs` renders the controlled state, native number selector, timer,
action tiles and keypad. `texts.rs` supplies complete localized copy.
`mod.rs` and `src/components/mod.rs` export the public API. Handlers must
re-read current state and apply can_dispatch before emitting; number changes
must restore the caller-owned DOM value when the host declines the request.

`demo/src/demos/softphone.rs`, the demo route/module registry and navigation
provide an interactive simulation. Add `tests/softphone_smoke.rs` and register
`cargo xtask test-softphone` in standalone and full gates. Browser tests read
the host's command/state evidence independently of visible text and exercise
real native keyboard selection. Capture representative wide/narrow screenshots
for visual review and verify no layout overlap or unresolved icons.

`doc/components/softphone.md` documents all props/types, a working controlled
usage example, action-state policy, pending/rejection behavior, timer ownership,
localization and reproducible tests. Link from doc/README.md and annotate the
research report with the accepted UI-only scope. Update gate counts wherever
the new browser lane changes them.

Final verification is `cargo xtask verify-full`, with focused checks during
development. Re-read Beads queue after the long gate and immediately before
landing; review changes, close the bead, synchronize Beads and commit/push.
