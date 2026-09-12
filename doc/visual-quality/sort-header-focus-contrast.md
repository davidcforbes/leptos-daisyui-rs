# Sortable header ink disappears on keyboard focus

**Status:** automated (focused release-browser contract, since 2026-09-12)
**Seen in:** ETL Mirrors/Sources; shared EntityTable and DataTable/ServerEntityTable

## What it looks like

A blue table header is readable until Tab reaches its sort button. The label
and direction marker become white on near-white even though sorting works and
the default screenshot passes. Dark-mode-only testing can conceal the defect.

## Root cause

The shared header declares table-header-content ink, but a Ghost Button's
focused/pressed background falls back to the general theme's base color.
The foreground and background therefore stop being a coherent semantic pair.
The generic animated focus ring can also outrank a non-important header outline.

## How to check (manual)

Use an actual Tab key in a light-theme desktop browser. Inspect left- and
right-aligned headers before and after sorting, then hover and hold a pointer
press. Labels, direction markers and the focus outline must remain readable.
Repeat in forced colors and after a desktop resize. Enter, Space and clicking
must each advance sorting exactly once; focus/hover must not change rows,
accepted queries, preferences, saves or selection.

## Automation

`tests/common/table_header_focus.rs` computes actual rendered foreground against
composited backgrounds, including marker/ancestor opacity; the marker is required,
not an optional SVG. Both labels and markers must meet 4.5:1. Real Tab navigation,
pointer events and forced-color emulation exercise interactive states. A temporary
rendered white/#F8FAFB pair must fail the same oracle, then removal must restore
the passing state. This does not mutate release bundles or approve image baselines.

`shared_entity_headers_keep_keyboard_focus_readable` and
`shared_server_headers_keep_keyboard_focus_readable` run in the existing
client-snapshot and server-table-column-tools release browser lanes, respectively,
including `cargo xtask verify-full`. Default-state style/axe audits alone are not
evidence for this interaction defect. ETL separately rebuilds and checks all nine
Mirrors/Sources headers in its full browser matrix.
