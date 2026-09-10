# ServerEntityTable verification — 2026-09-09

Bead: `ldui-9ke9`

Surface: the release-WASM History fixture at
`/components/data-table#server-entity-history`, using the repository's
PixelProof A/B/C/D methodology.

## Contract evidence

- Layer A: the History fixture owns a definite, viewport-relative table slot;
  the footer remains within that slot, the gear menu remains within the
  viewport on both axes, and the table retains its computed horizontal-scroll mechanism
  for any wide column state. The two audit-fixture wrappers normalize their 14
  native input/select shadows without changing production defaults; a browser
  assertion parses every computed shadow layer and requires every layer to be
  both transparent and zero-sized. The shared server chooser retains its
  right/bottom alignment while `max-h-80 overflow-y-auto` bounds a many-column
  menu to 320 px and makes the remaining items internally scrollable. The
  unchanged repository style and layout ceilings are exercised
  separately by `cargo xtask test-style` and `cargo xtask test-layout`.
- Layer B: the browser reads the accepted query, accepted row IDs,
  authoritative total, proposal token, request/failure state, and complete
  `EntityTablePreferences` independently from the rendered controls and row
  keys. It compares header order and column width geometry to that accepted
  model and counts each accepted preference replacement independently.
- Layer C: the native page-size Select has stable `id`, `name`, and accessible
  label; the Module resize separator has a name and ordered ARIA range; the
  fixture has zero serious or critical WCAG 2 A/AA and WCAG 2.1 AA axe
  findings.
- Layer D: real pointer input drives gear hide/restore/reorder, real keyboard
  input drives resize and the native Select, proposal tokens distinguish
  pending/accepted/failed requests, and buffered console errors, WASM panics,
  window errors, and unhandled rejections remain empty.

The comprehensive journey proves:

- gear hide and restore agree with accepted preferences, and Build moves
  exactly one position earlier in both the DOM and accepted order;
- Arrow Right, Arrow Left, Home, and End resize a named separator and the
  exact expected width, accepted preference, ARIA value, and rendered track
  remain identical; a real pointer drag previews locally and commits exactly
  one accepted replacement;
- the real native Select sequence (focus, Space, Home/Arrow, Enter) accepts a
  fixed size, fixed intent survives a desktop viewport resize, Auto accepts a
  measured capacity, and a second desktop resize accepts a different Auto
  capacity; the fixed value and visible label are re-read after resizing;
- Started sort and page-two navigation expose their matching pending tokens
  while accepted rows and controls remain locked, then replace rows only after
  the simulated server acknowledges the complete query; the page-two range is
  checked using exact numeric arithmetic; and
- a failed Mode filter proposal retains the accepted query, row IDs, total,
  range, page-size label, filter value, chooser expanded/visible state, column
  order, widths, and preference state in both pending and failed states.

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
5. **No-op reorder.** A capture listener swallowed Build's real move-earlier
   click. The comprehensive journey failed `Build must move exactly one
   position earlier`: actual order ended in `trigger, build`, while expected
   order ended in `build, trigger`. After removal, the exact test passed.
6. **No-op resize key.** A capture listener swallowed Arrow Left after Right,
   Home, and End had established non-edge starting widths. The exact keyboard
   oracle failed at 1200 px versus the required 1184 px. After removal, the
   exact test passed.
7. **Mixed visible shadow.** One History input received a transparent
   zero-sized shadow layer followed by a visible nonzero layer. The parsed
   all-layer oracle failed with one unexpected shadow versus zero. After
   removal, the exact test passed.
8. **Fixed Select drift.** After the fixed-size viewport resize, a runtime
   mutation changed only the native Select value to `auto`. Its readback failed
   at `auto` versus `10`. After removal, the exact test passed.
9. **Optimistic pending sort.** While token 4 was pending, a runtime mutation
   changed Started's rendered `aria-sort` to `ascending`. The lock assertion
   failed against accepted `none`. After removal, the exact test passed.
10. **Pending page DOM drift.** While page-two token 5 was pending, a runtime
    mutation removed the first accepted row. The pending lock failed with two
    DOM IDs versus the three accepted IDs. After removal, the exact test
    passed.
11. **Stale numeric range.** After page two was accepted, the range was changed
    to the plausible but stale `Showing 1–3 of 48`. Exact arithmetic required
    `Showing 4–6 of 48`. After removal, the exact test passed.
12. **Failed-filter control drift.** While the failing Mode proposal was
    pending, changing the rendered Select to `Replay` failed against the
    accepted empty value. Separately forcing the chooser open failed expanded
    state `true` versus accepted `false`. Both mutations were removed, and the
    exact test passed.

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
- Review-hardening RED against the pre-counter fixture: the official focused
  lane passed 14/15 and failed the comprehensive journey at
  `preferenceProposals` (`null` versus `0`).
- Review-hardening geometry RED at 1280x800: the unbounded 775 px chooser
  occupied y=415.5..1190.5. The bounded server chooser measures 320 px with
  computed `overflow-y: auto` and remains inside both viewport axes.
- Restored review-hardened exact journey: PASS, 1 passed, 0 failed, 17.24
  seconds. The attempted additional official focused rerun was stopped before
  test execution at the primary agent's request; the final candidate's single
  `cargo xtask verify-full` supplies the broad release-browser gate.
- Targeted native chooser reachability guard: PASS, 1 passed, 0 failed (3272
  filtered out).

## Remaining limitations

- The History adoption target is the repository's canonical desktop viewport;
  mobile behavior remains outside this specification.
- The fixture is a deterministic in-memory server simulator. It proves the
  controlled replacement boundary and correlation tokens, not transport-level
  cancellation or a production backend.

## Final integrated gate — 2026-09-10

Candidate `b85cf57` passed `cargo xtask verify-full`: **41/41 steps, exit 0**,
in 2793.2 seconds. All 16 native and 25 release-browser steps passed. The final
server-table column-tools lane passed 15/15 in 117.46 seconds, including both
History journeys and reactive facade validation. EntityTable passed 25/25,
including the `ldui-ova3` Auto-label/native-selection regression. Layout passed
15/15 in 68.67 seconds and style passed 13/13 in 58.10 seconds, with unchanged
ceilings. The library suite passed 3287 tests.

The first gate attempt exposed one redundant `clone()` of the `Copy`
multi-selection configuration. It was stopped before browsers; `b85cf57`
removed the clone, passed the exact failed clippy command, and then passed the
complete restarted gate above. This correction does not change selection
behavior. The canonical ten-field adoption example also passed
`cargo check -p leptos-daisyui-rs --example server_entity_history`.

The gate stopped both owned browser hosts. The post-gate Beads snapshot had
no additional ready, open, or blocked work. Consumer migration and deployment
remain outside this library qualification.
