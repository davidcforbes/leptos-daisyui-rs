# Resizing one column changes its neighbors

**Status:** automated browser regression
**Seen in:** EntityTable group-aware pagination fixture

## What it looks like

Focusing a resize separator moves several borders. Home announces 48px while
the column remains wider, and dragging it changes neighboring widths.

## Root cause

A full-width fixed-layout table stretches an entirely explicit colgroup.
Copying that rendered width into the resize model on focus compounds the
stretch. Initial widths expressed as consumer minima also prevent narrowing.

## How to check (manual)

Record all header widths, focus a separator, then use Home and drag it. Focus
must leave geometry and preferences unchanged. The changed column must agree
with its accessible value and accepted preference; neighboring widths must
remain unchanged. Check tables with and without a flexible track separately.

## Automation

`resizing_one_column_preserves_every_neighbor_width` in
`tests/entity_table_smoke.rs` drives keyboard and real CDP pointer input,
reads accepted fixture preferences independently, and measures every neighbor.
The original focus mutation and subsequent stretched-minimum failures were
both observed before the corresponding fixes. The registered lane is
`cargo xtask test-client-snapshot`.
