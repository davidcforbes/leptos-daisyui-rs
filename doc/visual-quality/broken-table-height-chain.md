# Broken table height chain

**Status:** automated (native topology and browser assertions, since 2026-09-04)
**Seen in:** leptos-daisyui-rs, 4iiz-Office No Hires

## What it looks like

A table configured to fill a page stops after its currently rendered rows and
leaves unused space below it, even though the application shell has a definite
height. Its Auto rows-per-page value can then change after filtering to a small
subset and remain stuck when the full result set returns.

## Root cause

CSS percentage height must resolve through every framework-owned wrapper between
the definite-height flex surface and `EntityTable`. A content-sized wrapper in
that chain makes the table's `height: 100%` depend on the rows it already
painted. In `SnapshotTablePage`, the table slot must therefore be a shrinkable
remaining-space flex child: `min-h-0 flex-1`.

The visual gap and the Auto-page-size latch are two consequences of that same
feedback loop. Adding height to an already-correct application shell only hides
the missing framework boundary and leaves other consumers exposed.

## How to check (manual)

Place `SnapshotTablePage` in a definite-height flex surface and configure its
table with `EntityTableViewportFit::fill_parent()`:

- verify the page consumes the supplied height and the table slot consumes the
  remaining height below headers and filters;
- verify the table's internal region, rather than the page, owns overflow;
- record the Auto row count, filter to one row, restore all rows, and verify the
  original Auto count returns; and
- repeat at a shorter and taller surface height so the result cannot be a
  content-sized coincidence.

## Automation

`src/patterns/snapshot_table_page.rs` has a bounded topology guard for the
framework-owned table slot. `tests/snapshot_table_page_controls_smoke.rs` mounts
a fixed-height `fill_parent` fixture, asserts the slot's computed flex growth
and geometry, and verifies that filtering to one row and restoring all rows
does not latch Auto rows-per-page onto the transient subset.
