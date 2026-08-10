# Default component not specced

**Status:** rulebook-only
**Seen in:** consumer app screen reviews generally (office-perf-web among
them) — a judgment pattern, not yet tied to one filed defect.

## What it looks like

A screen ships and every computed style is individually "valid" — on-ramp
font sizes, declared radii, declared shadows, `.btn` present on every
button — and it still looks wrong, because every component is sitting on
its *default* variant. A primary action button rendered as
`ButtonColor::default()` instead of `ButtonColor::Primary`; a card at
default elevation where the design calls for an elevated one; a badge in
neutral gray where the spec calls for a status color. Nothing here is a
style-system violation, so nothing an automated sweep looks at is wrong.

## Root cause

Nobody selected the intended color/size/style props for the component
against the design spec — usually because the component was wired up to get
something on screen, and the deliberate prop choice was deferred and then
forgotten. This differs from every other entry in this rulebook: those are
"the rendered value disagrees with the declared visual system"; this one is
"the rendered value agrees with the visual system but disagrees with the
*design intent* for this specific screen" — a comparison ldui-audit has no
input for, because it has no access to the design reference.

## How to check (manual)

Open the screen next to its design reference (Figma mock, desktop
screenshot, whatever the source of truth is) and compare component-by-
component: does every button/card/badge/input carry the *specific* variant
the design calls for, or the library default? This is a per-screen visual
diff against intent, not a rule that generalizes across screens — a
"default-looking" button is sometimes exactly what was designed.

## Automation

**Cannot be automated** by ldui-audit or any computed-style sweep — there is
no machine-readable "design intent" to check rendered output against, only
a human comparison to a reference artifact. This entry stays rulebook-only
by design; it is the counter-example that shows not every defect in this
index graduates to `ldui-audit`. If a future project defines design specs in
a structured, machine-readable form (e.g. per-screen expected variant
manifests), this could move to `automated (component-drift)` — until then,
it's a checklist item for screen review, not a gate.
