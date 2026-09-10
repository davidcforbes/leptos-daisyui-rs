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

A correct flex chain can still suffer a related feedback loop in its footer.
If a page-number window gains slots or wider two-digit buttons while navigating,
the footer can wrap onto another line. The smaller table region then reports a
different Auto capacity; the next page may repeat rows even though the dataset
has not changed. Filtering to one page can trigger the reverse transition.
The row-range caption can trigger the same transition when its total changes
from several digits to one. Stabilizing only the page buttons misses that case.
Keep the pager's slot count and digit width stable while Auto measurement is
active, including layout-only space when a local filter reduces the page count.
Reserve the caption's footprint with overlapping, invisible, accessibility-hidden
source-bound ordinary and custom-empty captions. The visible caption retains the
only row-range hook and always reports the actual displayed count. Tabular digits
make the source-bound digit count meaningful; no reserved text becomes announced
data. Reserved space must not introduce fake page numbers or focusable controls.

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

At a constrained width, also navigate from single-digit to double-digit pages.
Check consecutive row ranges and unchanged table-region/footer heights. Repeat
the filter-one/restore sequence there, where a footer wrap is most likely.

## Automation

`src/patterns/snapshot_table_page.rs` has a bounded topology guard for the
framework-owned table slot. `tests/snapshot_table_page_controls_smoke.rs` mounts
a fixed-height `fill_parent` fixture, asserts the slot's computed flex growth
and geometry, and verifies that filtering to one row and restoring all rows
does not latch Auto rows-per-page onto the transient subset.

The same browser test removes the slot's sizing classes while one row is
painted, requires the bottom-edge geometry check to detect the break, then
restores the classes and verifies the original height and Auto capacity return.
Run `cargo xtask test-snapshot-table-page-controls` for the focused release
browser proof, or `cargo xtask verify-full` for the final combined gate.

The grouped-pagination journey in `tests/entity_table_smoke.rs` separately checks
consecutive ranges through page 10 and stable Auto capacity/geometry through
full-to-one-to-empty-to-full filtering. It locates an actual footer wrap transition
and checks both sides, not only a convenient fixed width. Pure tests cover the
stable window, source-bound captions and Auto layout reservation. Its focused lane
is `cargo xtask test-client-snapshot`; native reservation tests alone are not
evidence that the rendered footer stays stable.
