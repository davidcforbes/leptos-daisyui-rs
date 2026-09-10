# Native select label overlaps its arrow

**Status:** automated (browser geometry assertion, since 2026-09-09)
**Seen in:** 4iiz-etl

## What it looks like

The end of a selected native-option label touches or paints beneath the browser's
dropdown arrow. Numeric choices may look correct while a longer dynamic label,
such as `Auto (11)`, loses its closing characters.

## Root cause

A fixed select width accounted for short numeric options but not the longest
bounded localized label after the browser-reserved trailing padding was removed.
The arrow is native paint rather than a normal DOM child, so ordinary overlap
sweeps cannot see the collision.

## How to check (manual)

Exercise every bounded label length in the actual browser and inspect the gap
between the final glyph and the native arrow. Include the longest localized or
dynamic value, not only the default numeric choice.

## Automation

`tests/entity_table_smoke.rs::auto_page_size_labels_leave_native_arrow_clearance`
measures rendered label text using the select's computed font, subtracts its
computed inline padding from the actual control width, and requires a remaining
gap. It covers one-, two-, and three-digit Auto counts, real keyboard selection,
accepted state readback, and an 80px inject/catch/revert negative control.
