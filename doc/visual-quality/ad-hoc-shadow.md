# Ad-hoc shadow

**Status:** automated (depth, since 2026-08-08)
**Seen in:** 4Ease parity work

## What it looks like

A card, a dropdown, or a popover carries a shadow that's *close* to the
product's elevation system but not from it — usually Tailwind's stock
`shadow-md`/`shadow-lg` grabbed because it was the nearest utility class,
rather than one of the declared elevation levels. Individually it looks
fine; next to a screen that correctly uses the declared levels, the two
surfaces stop reading as the same depth vocabulary, and the app stops
reading as one product. This was most visible in the 4Ease parity work,
where every mismatched shadow was a small tell that a screen had been built
without checking the desktop reference.

## Root cause

`ui_tokens::elevation::LEVELS` declares five shadow specs (offset-x,
offset-y, blur, opacity — `ldui_audit::from_ui_tokens`'s `shadows`), shared
with the Direct2D desktop face. A `shadow-md` (or any other value not in that
set) doesn't match any declared `ShadowSpec` within tolerance, so it's a rogue
shadow even when it looks superficially similar.

## The fix, for a framework-owned card (ldui-k4fn)

Reaching for the nearest utility is only half the failure; the other half is
that a *consumer* of an opinionated component cannot fix it from outside.
`KpiStrip`/`KpiCard` hard-coded `shadow-sm`, and no element it rendered
carried a class a product could safelist — the only ways out were a
descendant selector into someone else's markup or a page-local fork, both of
which break the ownership boundary the pattern exists to hold.

The answer is a framework-owned semantic class carrying a declared level,
with a custom property as the theme hook. It ships in the **generated**
`styles/tokens.css`:

```css
.ld-card-depth { box-shadow: var(--ld-card-shadow, var(--ld-elevation-4)); }
```

Three things make it work, and each is easy to get wrong:

- **The level is declared.** `--ld-elevation-4` is
  `ui_tokens::elevation::LEVEL_4`, documented as the card resting elevation,
  so `PROFILE.shadows` already accepts it and the depth family stops
  reporting the card. Picking a level "near" `shadow-sm` by eye would just
  move the rogue value.
- **It is static.** `.ld-elevated` (`src/tokens/animations.rs`) also rests at
  LEVEL_4, but lifts to LEVEL_8 with a transform on hover — an interaction
  affordance. Applying it to a read-only tile trades one defect for another.
  A non-interactive surface needs its own class that sets one property.
- **It ships statically, not only at runtime.** A class defined only by
  `UiTokensPreamble`'s `<style>` is undefined for any consumer who never
  mounts it — so replacing `shadow-sm` that way yields *no shadow*, silently.
  That is the ldui-h7tw defect class; `tests/ld_class_stylesheet_coverage.rs`
  guards against repeating it.

`--ld-card-shadow` is left undeclared by the framework, which is what makes
`:root { --ld-card-shadow: … }` in a product theme win with no `!important`
and no selector reaching into the component.

## How to check (manual)

`getComputedStyle(el).boxShadow` on the element; compare offsets/blur/opacity
against the five declared levels. A shadow with the right blur but a
different color or opacity is still a violation — the comparison is
geometry *and* color, not just "looks similar".

## Automation

The engine sweep parses each element's computed `box-shadow` into components
(offsets, blur, spread, opacity, color, inset) and compares against
`PROFILE.shadows` with fixed epsilons — a same-geometry, different-hue
shadow still fails. A miss pushes a `family::DEPTH` violation. This repo's
own negative-control test (`sweep_detects_injected_style_and_drift_violations`
in `tests/style_audit_smoke.rs`) deliberately skips injecting a depth
violation because the engine's own negative controls already prove that
family upstream (`pixelproof-style-audit`) — the ldui-audit integration
still reports it, it's just not re-proven here. Caught by
`cargo xtask test-style`.
