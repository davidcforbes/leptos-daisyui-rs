# Native select label overlaps its arrow

**Status:** automated (browser geometry assertion, since 2026-09-09)
**Seen in:** 4iiz-etl, leptos-daisyui-rs, 4iiz-Office adoption

## What it looks like

The end of a selected native-option label touches or paints beneath the browser's
dropdown arrow. Numeric choices may look correct while a longer dynamic label,
such as `Auto (11)`, loses its closing characters.

## Root cause

A fixed select width accounted for short numeric options but not the longest
bounded localized label after the browser-reserved trailing padding was removed.
The arrow is native paint rather than a normal DOM child, so ordinary overlap
sweeps cannot see the collision.

### The second cause: `appearance: base-select` (ldui-tfx7, 2026-09-23)

Widening fixes a control whose width you own. It cannot fix a select in a
narrow table column, and there a second mechanism applies. daisyUI 5.5 sets
`appearance: base-select` on `.select` wherever Chrome supports customizable
selects, and in that mode the selected label is drawn by an internal part that
ignores the select's `text-overflow`. daisyUI's `overflow: hidden` clips at the
PADDING box, so a label wider than the content box paints straight across the
28px trailing padding and under the arrow, with no ellipsis. A hand-copied
daisyUI rule WITHOUT the `@supports (appearance: base-select)` block does not
reproduce it, which is how the first probe of this bug looked clean.

`Select` now clips a non-`multiple` select at its CONTENT box
(`overflow: clip` + `overflow-clip-margin: content-box`), which keeps the label
clear of the arrow in both appearance modes. It truncates without an ellipsis:
a base-select label only accepts one through author `<button><selectedcontent>`
markup, which would change `Select`'s structure for every consumer.

## How to check (manual)

Exercise every bounded label length in the actual browser and inspect the gap
between the final glyph and the native arrow. Include the longest localized or
dynamic value, not only the default numeric choice.

## Automation

`tests/entity_table_smoke.rs::select_labels_never_paint_under_the_native_arrow_at_any_size`
compares PIXELS, since the arrow is background paint and the base-select label is
internal: for every `SelectSize`, a long-label and an empty select built from the
real filter control's classes must have matching trailing-padding strips (max
channel difference 48 -- anti-aliasing differs by 1/255, a glyph by 100+), and
restoring daisyUI's `overflow: hidden` must make them differ.


`tests/entity_table_smoke.rs::auto_page_size_labels_leave_native_arrow_clearance`
measures rendered label text using the select's computed font, subtracts its
computed inline padding from the actual control width, and requires a remaining
gap. It covers one-, two-, and three-digit Auto counts, real keyboard selection,
accepted state readback, and an 80px inject/catch/revert negative control.

Include the actual localized three-digit option, not just a canvas estimate:
`Automático (166)` needs more room than the English fixture. In the measured
Windows Chromium font, the old 112px fixed width left -21.55px of clearance;
intrinsic sizing with an 80px minimum left only 0.45px. The component now uses
intrinsic sizing with a 144px minimum, which measured 10.45px for that label.
These measurements explain the choice; the live computed-font assertion remains
the authority for a different browser or locale.

When injecting a narrow width, override `min-width` as well as `width` and restore
both afterwards. Otherwise the production minimum defeats the injection and a
failed negative control tells you nothing about the actual text detector.
