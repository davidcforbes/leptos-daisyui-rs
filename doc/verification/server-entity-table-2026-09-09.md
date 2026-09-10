# ServerEntityTable verification — 2026-09-09

Bead: `ldui-9ke9`

Surface: the release-WASM History fixture at
`/components/data-table#server-entity-history`, using the repository's
PixelProof A/B/C/D methodology.

## Contract evidence

- Layer A: the History fixture owns a definite, viewport-relative table slot;
  the footer remains within that slot, the gear menu remains within the
  viewport, and the table retains its computed horizontal-scroll mechanism
  for any wide column state. The two audit-fixture wrappers normalize their 14
  native input/select shadows without changing production defaults; a browser
  assertion requires all 14 controls to compute either `box-shadow: none` or
  Tailwind's transparent zero-sized shadow representation. The
  unchanged repository style and layout ceilings are exercised
  separately by `cargo xtask test-style` and `cargo xtask test-layout`.
- Layer B: the browser reads the accepted query, accepted row IDs,
  authoritative total, proposal token, request/failure state, and complete
  `EntityTablePreferences` independently from the rendered controls and row
  keys. It compares header order and column width geometry to that accepted
  model.
- Layer C: the native page-size Select has stable `id`, `name`, and accessible
  label; the Module resize separator has a name and ordered ARIA range; the
  fixture has zero serious or critical WCAG 2 A/AA and WCAG 2.1 AA axe
  findings.
- Layer D: real pointer input drives gear hide/restore/reorder, real keyboard
  input drives resize and the native Select, proposal tokens distinguish
  pending/accepted/failed requests, and buffered console errors, WASM panics,
  window errors, and unhandled rejections remain empty.

The comprehensive journey proves:

- gear hide, restore, and reorder agree with accepted preferences;
- Arrow Right, Arrow Left, Home, and End resize a named separator and the
  accepted width, ARIA value, and rendered track remain identical;
- the real native Select sequence (focus, Space, Home/Arrow, Enter) accepts a
  fixed size, fixed intent survives a desktop viewport resize, Auto accepts a
  measured capacity, and a second desktop resize accepts a different Auto
  capacity;
- Started sort and page-two navigation replace rows only after the simulated
  server acknowledges the complete query; and
- a failed Mode filter proposal retains the accepted query, row IDs, total,
  range, page-size label, column order, widths, and preference state.

## TDD RED and minimum fix

The first intended behavioral RED was
`server_entity_history_standard_controls_preserve_accepted_truth`: after the
native Auto choice, it timed out waiting for proposal token 2. The snapshot
showed `page_size_mode: "auto"` but `proposals: 1` and accepted page size 10.
The History fixture had mounted the canonical facade with
`viewport_fit=false`.

The minimum fix enabled `viewport_fit` and changed the definite fixture slot
from a fixed 34 rem height to `60vh`, allowing an actual desktop viewport
resize to produce a different Auto capacity. The exact journey then passed.

## Inject, catch, revert controls

All injections were temporary, ran against the same isolated release host,
were removed immediately after the named failure, and were followed by a
green exact-test rerun.

1. **Current-page-only filtering.** A runtime projection replaced every
   accepted filtered identity and rendered row key with identities from the
   original page-one slice. Test
   `server_entity_history_filters_use_accepted_population_truth` failed at
   `run must find a complete-population row outside page one` for the
   `run-037` filter (`acceptedIds` and `domIds` were both
   `["history-001"]`). After removal, the exact test passed.
2. **Bypassed missing-filter alert.** A runtime mutation removed the
   configuration alert and inserted a fake facade marker, modeling a validator
   bypass. Test
   `server_entity_table_column_validation_tracks_reactive_columns` failed with
   the rendered table present and alert code `null` instead of `name`. After
   removal, the exact test passed.
3. **Optimistic failed proposal.** A runtime mutation copied the proposed
   Mode=`Replay` query into the accepted-query marker before the simulated
   failure resolved. Test
   `server_entity_history_standard_controls_preserve_accepted_truth` failed at
   its retained accepted-query equality: the injected page-one filtered query
   differed from the real accepted page-two unfiltered query. After removal,
   the exact test passed.
4. **Detached width publication.** A runtime mutation stripped
   `column_widths` from the independently published preference model after a
   keyboard resize. Test
   `server_entity_history_standard_controls_preserve_accepted_truth` failed at
   `accepted Module width`. After removal, the exact test passed.

## Commands and results

- Exact comprehensive journey after the minimum fix: PASS, 1 passed, 0
  failed, 12.71 seconds.
- Restored population-truth exact test: PASS, 1 passed, 0 failed, 13.50
  seconds.
- Restored validation exact test: PASS, 1 passed, 0 failed, 6.19 seconds.
- Restored comprehensive journey after all controls: PASS, 1 passed, 0
  failed, 12.20 seconds.
- `cargo xtask test-server-table-column-tools`: PASS, 15 passed, 0 failed,
  136.26 seconds, xtask 1/1. A preceding run had 14/15 pass and rejected an
  over-strong test assertion that required active horizontal overflow; the
  corrected assertion verifies the computed scroll mechanism is available
  when needed.
- `cargo xtask test-style`: PASS, 13 passed, 0 failed, 65.88 seconds, xtask
  1/1. The first run correctly rejected 14 newly introduced audit-fixture
  shadows (`depth: 50 > ceiling 36`). Fixture-scoped input/select
  normalization removed exactly those 14 additions; the ceiling remained 36,
  and the browser readback proves all 14 fixture controls now use the
  transparent zero-shadow representation.
- `cargo xtask test-layout`: PASS, 15 passed, 0 failed, 77.45 seconds, xtask
  1/1 with all ceilings unchanged.
- Current exact comprehensive journey, including the 14-control shadow
  readback after the visual-audit fix: PASS, 1 passed, 0 failed, 12.25
  seconds.
