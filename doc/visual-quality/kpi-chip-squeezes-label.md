# KPI status chip squeezes its label

**Status:** automated (browser geometry assertion, since 2026-09-10)
**Seen in:** leptos-daisyui-rs Office-delta fixture

## What it looks like

A short KPI heading collapses into a near-character-wide column beside its
status chip. The chip itself can remain fully inside the card, so a bounds-only
check passes while the heading becomes a tall stack of clipped letters.

At a 390px viewport, a roughly 140px card carrying `Needs review` squeezed the
`Weekly hires` span to 7.609px (`clientWidth` 8 versus `scrollWidth` 15). Its
reserved 32px two-line box concealed 176px of scroll height. The status-chip
bounds assertion still passed.

## Root cause

The heading and chip shared one non-wrapping flex row. The chip was
`shrink-0`, so the heading absorbed all width pressure when their combined
intrinsic widths exceeded the card body. Checking only the chip's rectangle
missed the damage transferred to its sibling.

The corrected composition wraps the heading-and-chip row when a chip is
present and lets long chip copy shrink and wrap within the card. Cards without
a chip retain their historical title-row layout.

## How to check (manual)

Render a chipped card at the narrowest supported viewport and inspect both
siblings, not only the badge. A short heading such as `Weekly hires` must remain
readable within its reserved two-line box, and the chip must wrap to another
line when the two intrinsic widths do not fit. Also try longer localized chip
copy and confirm that its badge grows vertically without crossing the card.

In DevTools, compare the short heading's `scrollWidth` with `clientWidth` and
`scrollHeight` with `clientHeight`; both scroll dimensions must fit. Then check
the chip's bounds independently.

## Automation

`tests/entity_table_smoke.rs::office_status_chip_and_wrapped_summary_remain_reactive`
sets the fixture to a 390px viewport and checks both sides of the layout: the
chip remains within the card and the short `Weekly hires` heading has no hidden
horizontal or vertical overflow. The label oracle recorded the pre-fix RED
dimensions above; final post-fix browser GREEN verification is pending.
