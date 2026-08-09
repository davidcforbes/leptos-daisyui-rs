# Off-ramp font size

**Status:** automated (typography, since 2026-08-08)
**Seen in:** office-perf-web (ad-hoc size utilities from the office-perf audit
stream); reproduced synthetically in this repo's own negative-control test.

## What it looks like

A label, a stat value, or a table cell is *almost* the right size — 13px
where the ramp has 12 or 14, 15px where it has 14 or 16. In isolation it
looks like nothing; next to a page that otherwise uses six disciplined sizes,
it reads as a rhythm break the eye notices before the mind can name it.

## Root cause

Someone reached for an arbitrary Tailwind value — `text-[13px]`, or a
one-off inline `style="font-size: 13.37px"` — instead of the nearest step on
the declared type ramp. This repo's ramp is
`ui_tokens::typography::RAMP` = **28 / 20 / 16 / 14 / 12 / 11 px**
(`ldui_audit::from_ui_tokens`'s `type_ramp`), the same ramp the desktop face
draws from. An off-ramp value also breaks WCAG 1.4.4 (resize text) when it's
px-pinned rather than expressed in a way that scales with the user's browser
font-size preference — see the DIP-to-rem rule in `doc/ci-cd.md`'s token
generation section.

## How to check (manual)

`getComputedStyle(el).fontSize` on the suspect element; compare against the
six values above. Faster in bulk: eyeball a page at 2x zoom — off-ramp sizes
cluster visibly once magnified, where on-ramp text falls into clean bands.

## Automation

The engine sweep walks every visible element with its own text node and
checks `getComputedStyle(el).fontSize` against `PROFILE.type_ramp`
(`onSet`), pushing a `family::TYPOGRAPHY` violation (`font-size N.Npx off the
type ramp`) for any miss. This is one of the three injected violations in
`sweep_detects_injected_style_and_drift_violations`
(`tests/style_audit_smoke.rs`) — a `<p style="font-size:13.37px">` is
injected and the suite asserts the typography count rises. Caught by
`cargo xtask test-style`.
