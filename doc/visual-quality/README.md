# Visual-quality defect-pattern rulebook

This is the growing index of visual-quality defect patterns for
leptos-daisyui-rs and its consumer apps — the concrete, named things that go
wrong (a silently-fallen-back font, a corner radius that's 15px instead of
8px, a hand-rolled button next to real ones) rather than the abstract rule
categories (`typography`, `shape`, `depth`, ...) that `ldui-audit` reports
under. A defect pattern starts here, described in plain language with a
manual check, whether or not anything can mechanically catch it yet. When
one becomes checkable against computed styles or DOM structure, it graduates
to a rule in `ldui-audit` (`../../audit/`) and this entry is updated to point
at the check that now catches it — the entry itself never goes away, because
the manual check and the "what it looks like" description stay useful for
anyone triaging a screen before the automation runs.

Where a pattern sits in the wider test model is described in the sibling
PixelProof repo's methodology doc,
`PixelProof/docs/methodology/principles.md`: Layer A (visual — computed
styles, pixels), Layer B (structure & state), Layer C (accessibility), Layer
D (behavioral & side-effect observability). Every entry here is a Layer A
pattern; ldui-audit is this repo's Layer A tooling.

## Lifecycle: rulebook-only → automated

1. **Rulebook-only.** A defect is real and recognizable but nothing checks
   for it mechanically yet — either nobody's built the rule, or (as with
   `default-component-not-specced.md`) it fundamentally can't be checked
   against computed output because it's a judgment call against design
   intent. These entries carry a manual check and nothing else.
2. **Automated.** A rule in `ldui-audit` (or, occasionally, a different
   mechanism entirely — see `dead-daisyui4-classes.md`) now catches the
   pattern on every run of the relevant suite. The entry's `## Automation`
   section names the family, the rule, and the test that exercises it.

A pattern can also move the other direction in spirit, if a family's
ceiling is ratcheted down but not yet to zero — the rule *catches* it, the
gate doesn't yet *block* on every instance. That distinction lives in the
suite's ceiling table (`tests/style_audit_smoke.rs`,
`tests/layout_audit_smoke.rs`), not in this rulebook.

## Entry template

Every file in this directory follows the same shape:

```markdown
# <defect name>

**Status:** automated (<family>, since <date>) | rulebook-only
**Seen in:** <app(s)>

## What it looks like
## Root cause
## How to check (manual)
## Automation
<which ldui-audit family/rule catches it, or why it cannot be automated>
```

## Index

| Entry | Status | Family | What it looks like |
|---|---|---|---|
| [`fallback-font.md`](./fallback-font.md) | automated | typography | Text renders in the platform default font instead of the declared one — looks "slightly off", nobody files it. Both failure modes caught since 2026-08-09 — but only if the profile pins a **real family name**, never a CSS generic |
| [`off-ramp-font-size.md`](./off-ramp-font-size.md) | automated | typography | An ad-hoc pixel size (e.g. `text-[13px]`) sits off the six-step type ramp |
| [`ad-hoc-shadow.md`](./ad-hoc-shadow.md) | automated | depth | A stock Tailwind `shadow-md` instead of a declared elevation level — cards stop reading as one product |
| [`undeclared-radius.md`](./undeclared-radius.md) | automated | shape | 8px vs 15px card corners — a small number mismatch that makes two apps look unrelated |
| [`hand-rolled-button.md`](./hand-rolled-button.md) | automated | component-drift | A raw `<button>` styled to look like `Button`, missing its focus ring, states, and theme awareness |
| [`dead-daisyui4-classes.md`](./dead-daisyui4-classes.md) | automated (separate gate) | — | `.form-control`/`.label-text` are no-ops in daisyUI 5; a labelled field silently collapses to inline layout |
| [`default-component-not-specced.md`](./default-component-not-specced.md) | rulebook-only | — | Every computed style is individually valid, but the screen ships on library defaults instead of the specced variants |

See `../ci-cd.md` for how `cargo xtask test-style` / `test-layout` / `verify`
run these suites, and `../../audit/src/lib.rs` for the `ldui-audit` crate
these rules live in.
