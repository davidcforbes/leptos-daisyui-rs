# Undeclared corner radius

**Status:** automated (shape, since 2026-08-08)
**Seen in:** office-perf-web (op-edag.2)

## What it looks like

Two apps that are supposed to share a visual system have cards with
different corner roundness — one at 8px, the other at 15px. Neither number
is "wrong" in isolation; the mismatch is what makes them look unrelated.
Corner radius is one of the highest-leverage, lowest-visibility signals in a
UI: most people can't name it as the cause, but everyone notices two apps
that "don't quite match."

## Root cause

`ui_tokens::radius` declares four values — `CONTROL` (4px), `CARD` (8px),
`BADGE` (12px), and `PILL` (a `9999px` sentinel meaning "fully rounded, any
magnitude") — pinned into `ldui_audit::from_ui_tokens`'s `radii`. A card
built with an arbitrary `rounded-[15px]` (or any value not in that set, and
not fully-pill-rounded relative to its own box) is off the declared set even
though 15px "looks like a reasonable card radius" in isolation.

## How to check (manual)

`getComputedStyle(el).borderTopLeftRadius` (and the other three corners) on
the element; compare against 4 / 8 / 12px, or confirm full pill-rounding
(radius ≥ half the shorter side) for anything claiming to be a pill.

## Automation

The engine sweep checks all four corners' computed radius against
`PROFILE.radii`, treating any declared radius ≥ 999 as a "pill" sentinel that
allows full rounding at any magnitude, and pushes one `family::SHAPE`
violation per offending element (`border-radius Npx not in the declared
radius set`). This is one of the three injected violations in
`sweep_detects_injected_style_and_drift_violations`
(`tests/style_audit_smoke.rs`) — a 40x40 box with `border-radius:17px` is
injected specifically below the min-side/2 pill threshold so it can't be
mistaken for a legitimate pill, and the suite asserts the shape count rises.
`audit/src/profile.rs`'s own unit tests pin the declared radii to
`[4.0, 8.0, 12.0, 9999.0]`; the audit engine's own tests
(`pixelproof-style-audit`'s `profile_matchers_apply_epsilon`) pin `15.0` as
the canonical undeclared-radius example (`assert!(!p.radius_ok(15.0),
"undeclared radius fails")`). Caught by `cargo xtask test-style`.
