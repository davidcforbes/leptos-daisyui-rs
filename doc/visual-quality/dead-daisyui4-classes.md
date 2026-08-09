# Dead daisyUI 4 classes

**Status:** automated (test-daisyui5 gate, since 2026-08-08)
**Seen in:** this repo (ldui-mai.3 — 206 sites across 28 files)

## What it looks like

A form label sits to the left of its input instead of stacking above it, or
a whole labelled field row collapses onto one line where every other field
on the page stacks correctly. `w-full` on the label appears to do nothing.
Nobody wrote a bug for it because it doesn't look broken so much as
inconsistent — one field among many just lays out differently.

## Root cause

`.form-control`, `.label-text`, and `.label-text-alt` were **removed in
daisyUI 5** and are complete no-ops today — verify with
`grep -c 'form-control' demo/node_modules/daisyui/daisyui.css` (returns 0).
They are not harmless leftovers: `.form-control` supplied
`display:flex; flex-direction:column`. Without it, a
`<label class="form-control w-full">` falls back to `display:inline`,
`w-full` goes inert on an inline box, and the label and its input flow
inline instead of stacking. `ldui-mai.3` found 206 call sites still carrying
these classes across 28 files — remnants of pre-daisyUI-5 code that compiled
and rendered *almost* right, so nothing failed loudly.

## How to check (manual)

`grep -rn 'form-control\|label-text' src/ demo/src/` — any hit outside a
migration-guidance comment is a live defect, not a style choice.

## Automation

**Not caught by `ldui-audit`** — this is a static source-text defect, not a
computed-style or DOM-structure one, so no rendered page carries the
signal ldui-audit is built to sweep. It's caught by a dedicated grep-based
test, `tests/no_dead_daisyui4_classes.rs`, which scans source files for the
three dead class names (skipping lines that are themselves migration
comments) and fails if any survive. That test runs as the `test-daisyui5`
step of the main gate (`cargo xtask verify`), not through
`cargo xtask test-style`/`test-layout`. It's the deliberate example in this
rulebook that automation for a defect can live outside the audit crate
entirely — a plain grep is the right shape of test when the failure mode is
a dead class silently doing nothing, which no computed-style sweep or
component-behavior test would ever observe.
