# Task 4 report

## Outcome

Implemented and committed the complete release-browser interaction evidence for
the History-shaped `ServerEntityTable` fixture.

Commit: `92b1d76 test(data-table): qualify canonical server EntityTable (ldui-9ke9)`

The review-hardening follow-up closes every blocking finding in
`task-4-review-report.md`: exact reorder and resize outcomes, canonical pointer
resize, fixed Select readback, pending sort/page/failure locks, numeric range,
failed filter and chooser retention, both-axis chooser geometry, and strict
all-layer shadow validation.

## Files committed

- `demo/src/demos/data_table.rs`
  - enabled the History fixture's viewport-fit contract;
  - changed its definite table slot to viewport-relative `60vh` so real
    desktop viewport changes exercise Auto capacity;
  - normalized only the two audit fixtures' input/select shadows so the new
    proof UI adds no depth violations while production defaults remain
    unchanged.
- `tests/server_table_column_tools_smoke.rs`
  - added one comprehensive, token-aware History journey covering gear
    hide/restore/reorder, controlled widths, fixed/Auto page size, desktop
    resize, sort, page two, retained failure, geometry, scoped axe, and browser
    errors;
  - added stable page-size and interaction snapshots plus native Select helper;
  - asserted all 14 new audit-fixture native controls use the transparent
    zero-shadow representation, rejecting any mixed visible layer;
  - added exact one-slot reorder and per-key/pointer width outcomes with one
    accepted preference replacement per gesture;
  - observed token-matched pending sort, page, and failed-filter states before
    acceptance/failure, including visible filter/chooser locks and exact range
    arithmetic.
- `src/components/data_table/server_component.rs`
  - bounded the shared server column chooser to 320 px with internal vertical
    scrolling while preserving its existing right/bottom alignment;
  - added a focused source guard for those reachability classes.
- `doc/verification/server-entity-table-2026-09-09.md`
  - recorded A/B/C/D evidence, the first RED, all four inject/catch/revert
    controls, and exact command results.

Concurrent `ldui-ova3` changes were not staged or modified.

## TDD and negative controls

First intended RED: after selecting Auto with real focus, Space, Home/Arrow,
Enter input, the new journey timed out waiting for token 2. The snapshot showed
the Auto preference but no query proposal because the fixture had
`viewport_fit=false`. The minimal fixture fix made the journey green.

Temporary runtime mutations, all removed and followed by green exact reruns:

1. Current-page-only identity projection made
   `server_entity_history_filters_use_accepted_population_truth` fail at its
   outside-page assertion for `run-037`.
2. Removing the missing-filter alert and inserting a facade marker made
   `server_entity_table_column_validation_tracks_reactive_columns` fail with
   alert code null instead of `name`.
3. Copying the failed proposal into the accepted-query marker made
   `server_entity_history_standard_controls_preserve_accepted_truth` fail its
   retained accepted-query equality.
4. Removing width publication from the preference marker made the same
   comprehensive journey fail at `accepted Module width`.
5. Swallowing Build's move-earlier click failed the exact one-position reorder
   assertion (original `trigger, build` versus expected `build, trigger`).
6. Swallowing Arrow Left failed exact resize math at 1200 versus 1184 px.
7. Adding a transparent zero shadow plus a visible layer failed the all-layer
   shadow count at one versus zero.
8. Drifting the fixed Select to Auto failed its post-resize value readback.
9. Publishing optimistic pending `aria-sort`, removing a pending page row,
   supplying a stale numeric range, retaining Mode=`Replay`, and forcing the
   chooser open each failed its corresponding lock/readback assertion.

## Verification

- `cargo xtask test-server-table-column-tools`: PASS, 15 passed, 0 failed,
  xtask 1/1, 136.26 seconds.
- `cargo xtask test-style`: PASS, 13 passed, 0 failed, xtask 1/1, 65.88
  seconds. Before normalization it correctly failed `depth: 50 > ceiling 36`;
  the 14-control normalization returned the page to the unchanged ceiling.
- `cargo xtask test-layout`: PASS, 15 passed, 0 failed, xtask 1/1, 77.45
  seconds.
- Current exact comprehensive journey after final audit-shadow assertion:
  PASS, 1 passed, 0 failed, 12.25 seconds.
- Review-hardening exact journey after all injections were removed: PASS, 1
  passed, 0 failed, 17.24 seconds.
- Targeted native server-chooser height/alignment guard: PASS, 1 passed, 0
  failed (3272 filtered out).
- Review-hardening official pre-fix lane: expected RED, 14 passed and 1 failed
  because the old fixture exposed no preference proposal counter.
- The post-fix broad focused rerun was stopped before test execution at the
  primary agent's request. The primary will run one final `cargo xtask
  verify-full` on the complete candidate, covering focused/style/layout lanes
  without another redundant release build.
- `git diff --check`: PASS before commit.

## Self-review

- Query transitions use proposal-token plus accepted-state predicates; no
  timing-only success condition was introduced.
- The facade still owns no local sort/filter/page operations; the fixture's
  simulated server remains the only population transformer.
- Header order is read from stable colgroup tracks so non-sortable columns are
  included.
- Width assertions compare the accepted preference model, ARIA value/text, and
  rendered track after Arrow Right, Arrow Left, Home, and End.
- Failure comparison covers accepted query, accepted/DOM IDs, total, range,
  page-size state, header order, resize state, and complete preferences.
- The first visual oracle was deliberately corrected from “must currently
  overflow” to “computed overflow mechanism is available”; requiring active
  overflow in every desktop state would reject a valid table whose tracks fit.
- No style/layout ceiling changed. The only production presentation change is
  the approved shared server-chooser height bound/internal y-scroll required
  to make all ten canonical History columns reachable at 1280x800.
