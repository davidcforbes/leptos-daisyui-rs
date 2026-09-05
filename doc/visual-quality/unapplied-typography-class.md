# Unapplied typography class

**Status:** rulebook-only in general; component-specific browser assertion for Softphone
**Seen in:** Softphone's initial duration display, corrected in `f730e66`

## What it looks like

A prominent timer or heading renders at body-text size even though its class
attribute appears to request special typography. The layout fits and a generic
style audit can pass, because the inherited size is itself an allowed size.

## Root cause

The intended class has no applicable CSS declaration, or the declaration loses
in the cascade. In Softphone, `ld-text-page-title` supplied no font-size rule.
The fix used the existing `text-2xl` utility and checked the rendered result.
A Rust compile or a class-name assertion cannot establish that CSS took effect.

## How to check (manual)

Compare the element with its intended visual role, then inspect
`getComputedStyle(element).fontSize` and the winning CSS rule. Confirm that the
utility exists in the freshly built stylesheet and its source is scanned.
Check a screenshot at both desktop and compact widths. Inspect computed
font weight, line height or shadow too when those are part of the intended change;
the presence of a utility name alone does not prove any of them.

## Automation

`tests/softphone_smoke.rs` checks a 24px computed timer size in the default
showcase environment at 1280px and 375px. Run `cargo xtask test-softphone`.
This catches regression of that specific intended emphasis. It is not a new
general `ldui-audit` rule: a global type-ramp check cannot infer which allowed
size a particular timer or heading was meant to use. Assert the intended
rendered property where a component has such a concrete visual contract.
