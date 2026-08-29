# Table hierarchy drift

**Status:** automated (computed-style browser assertion, since 2026-08-28)
**Seen in:** leptos-daisyui-rs, 4iiz-etl

## What it looks like

An opinionated data table renders as one undifferentiated white surface, or its
filter controls look like a detached toolbar. The column headers no longer read
as the strong navigation band, the filter row no longer reads as its subordinate
control band, and faint cell boundaries disappear. A sort can also appear to
"redraw" the table when the shell or header tracks move with the body data.

## Root cause

The semantic table classes may be missing, or a consuming Tailwind build may
omit the generated `styles/tokens.css` import. Demo-only CSS can hide the latter:
the showcase looks correct while the consumer has class names with no token
definitions. Content-driven layout or keyed headers that include sort/presentation
text can independently replace or resize the shell during a sort.

## How to check (manual)

On both a client and server table, verify:

- the first header row is `#004578` with white text;
- the aligned filter row is `#e5f1fb` with near-black text;
- every header, body, filter, empty, loading, and detail cell uses the faint
  `#e0e0e0` grid;
- header/filter/body tracks align after resize, reorder, visibility, and narrow
  horizontal scrolling; and
- pointer, Enter, or Space sorting changes row order and sort state without
  replacing the table/header nodes, moving the shell, or changing scroll origin.

Consumer CSS must import the library token stylesheet before Tailwind scans the
library sources. Seeing the hex values only in the demo output is not proof that
a consumer receives them.

## Automation

`tests/reactivity_smoke.rs` reads the computed header/filter/grid colors and
asserts the exact RGB values on a real Wasm page. Its client/server geometry
journeys tag table/header/filter nodes, record boxes and scroll origin, activate
sort through pointer, Enter, and Space, and require identity plus sub-pixel-stable
geometry. `tests/entity_table_smoke.rs` applies the same shell oracle to the typed
client-snapshot table. Generated-token freshness remains enforced by
`cargo xtask verify`; no demo-only stylesheet is accepted as the source of truth.
