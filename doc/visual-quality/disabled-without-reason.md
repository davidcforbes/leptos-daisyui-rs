# Disabled control with no reason

**Status:** automated (component-drift, since 2026-09-22, ldui-p82h)
**Seen in:** 4iiz-Office (Account page, ClientCallWorkspace: nine controls), the
Button showcase, every pager

## What it looks like

A greyed-out button that says nothing about why. A sighted user hovers and
gets no tooltip; a screen-reader user hears "Save, dimmed" and has no idea what
would enable it. The owner's rule (Office guidelines 12.4/12.6): **a disabled
action must say why**. Office wired `aria-describedby` hints by hand on one
page, then found nine more reasonless disabled controls on the next.

## Root cause

`disabled=true` is a one-bit prop; nothing forces the explanation to travel
with it, so the reason lives in a code comment or the author's head. The
framework fix is `Button::disabled_reason` (and `RowActionButton`,
`EntityExportAction`, `KpiAction::disabled_reason`, `EntitySavedFilterTexts::save_button_reason`):
non-blank text disables the control AND renders the reason as a hidden hint the
button references by `aria-describedby`, plus the native `title`. Blank is "no
reason" and leaves the control enabled, so one reactive string drives both.

## What is NOT a finding

A control disabled by **state** rather than a withheld action carries no
reason, and the rule exempts it explicitly:

- the current page of a pager and its boundary Previous/Next (inside
  `[data-pagination]`, or `[aria-current="page"]`) -- "you are here";
- a loading control (`.loading`, `[aria-busy="true"]`) -- it is busy, not
  refused;
- the already-selected option of a single-select group (`[aria-checked="true"]`,
  `[aria-pressed="true"]`, `[aria-selected="true"]`).

Anything else that is disabled needs a reason.

## How to check (manual)

Tab to every disabled control (they stay in the accessibility tree). For each:
does it have an accessible name, and does the accessibility inspector show a
description that says why? Hover: is there a tooltip?

## Automation

`ldui-audit` rule 6 `disabled-without-reason` (`audit/src/drift.js`), family
`component-drift`: a `button[disabled]`, `.btn-disabled` or
`[role=button][aria-disabled=true]` must have an accessible name (aria-label,
aria-labelledby, own text minus `aria-hidden` subtrees, title) AND an
`aria-describedby` resolving to non-empty text. `Button` stamps
`data-disabled-without-reason="true"` when rendered `disabled=true` with no
reason, and the finding quotes it so the fix is named. Negative control:
`disabled_without_reason_name_excludes_aria_hidden_descendants` in
`audit/src/drift.rs`. Browser proof: `cargo xtask test-row-action-presets`.
