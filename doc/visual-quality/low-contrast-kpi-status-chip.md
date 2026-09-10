# Low-contrast KPI status chip

**Status:** automated (browser computed-contrast assertion; fresh-build GREEN pending)
**Seen in:** leptos-daisyui-rs Office-delta fixture

## What it looks like

A soft semantic KPI chip has a recognizable warning tint but its small status
text nearly disappears into the pale fill. Shape, bounds, wrapping, and the
semantic badge class can all be correct while the words remain unreadable.

The settled light-theme `Needs review` chip rendered 12px semibold warning text
on its soft warning surface at 1.6934:1 (`rgb(252, 183, 0)` over
`rgb(255, 250, 239)`). Small text requires at least 4.5:1.

## Root cause

daisyUI's soft badge treatment uses the semantic badge color for both its text
and a low-percentage mix in its background. That treatment does not guarantee
text contrast for every semantic hue and theme; warning amber on the light
soft-warning surface is the concrete failure.

The KPI status chip keeps the existing semantic soft background and border but
uses the theme's `base-content` token for its caller-owned status words. This is
scoped to `KpiStatusChip`; changing `Badge` globally would silently redesign
unrelated badges.

## How to check (manual)

Inspect every KPI status-chip color in both light and dark themes. Read the
settled computed foreground and effective composited background, convert them
to sRGB, and calculate WCAG contrast. Require at least 4.5:1 because the chip's
12px text is not large text. Do not infer contrast from utility names or sample
only the border color.

## Automation

`tests/entity_table_smoke.rs::office_status_chip_and_wrapped_summary_remain_reactive`
measures the real rendered chip in `light` and `dark`, composites translucent
backgrounds through its ancestor chain, converts CSS Color 4 values through a
browser canvas to sRGB, and requires a ratio of at least 4.5:1. Its
inject/catch/revert control temporarily makes the foreground equal the
background, observes a 1:1 failure, restores the original inline style, and
requires the original ratio to return before judging it.

The pre-fix frozen artifact failed at 1.6934:1 in light mode and the negative
control failed at exactly 1:1. Final GREEN requires a fresh release build; the
already-served artifact cannot contain the source fix.
