# ServerDataTable footer verification — 2026-09-09

Issues: `ldui-gfk7` (footer and Auto/fixed row sizing), `ldui-y80u` (layout
audit profile). This is a historical verification receipt; current work and
status remain in Beads.

## Behavior verified

ServerDataTable uses the EntityTable footer convention: rows-per-page control
and range on the left, navigation on the right, wrapping at compact widths.
The Select retains its accessible identity through empty, loading, and query
changes. A retained Auto/fixed preference preserves explicit numeric intent
through resize and refetch. Labels, rendered rows, and pagination follow the
accepted server query while delayed or rejected proposals remain unaccepted.

The [public API and migration example](../components/data_table.md#footer-and-autofixed-rows-per-page)
cover host retention, readiness gating, localization, and definite parent height.
Fetching, request correlation, and deployment remain consumer responsibilities.

## Verification receipts

All commands ran from the Windows repository root. Browser tests ran serially
against release Wasm, with the runner checking stylesheet and asset freshness.

| Command | Observed result |
|---|---|
| `cargo xtask verify-full` | 39/40 steps; only `test-layout` failed on report truncation |
| `cargo xtask verify` after the audit correction | 16/16 in the runner summary; exit 0 |
| `cargo xtask test-layout` after the audit correction | 15/15 tests; exit 0; inject/catch/remove control passed |

The full run passed the focused server-table suite (7/7), reactivity (69/69),
and style audit (13/13), along with every other browser lane. Reactivity
included offset/cursor reset, fixed-slice rejection, and row-height convergence.
The server-table suite covered actual pointer and native Select keyboard
input, delayed/rejected proposals, fixed choice through resize, resumed Auto,
one-row filtering/restoration, empty/loading states, and stable footer identity.

The earlier negative-control run caught both original defects: the selector
was above the table, and resizing overwrote a manual row count with a measured
size that left the numeric selector blank.

## Audit correction

The layout profile omitted the intentional two-layer button shadow already
declared by the style profile and authored in `demo/input.css`. Additional
demo controls pushed DEPTH findings to the engine's 200-entry cap. The
truncation assertion correctly refused to trust the report.

The correction adds those same two shadow declarations to the layout profile.
No layout ceiling increased, no family was disabled, and the truncation guard
remains intact. The corrected DataTable audit and the injected overlap/grid
negative control both passed. Product source was unchanged during recovery.

## Reviewed captures

- [Wide footer, 1280px](server-table-footer-2026-09-09/footer-1280.png)
- [Compact footer, 375px](server-table-footer-2026-09-09/footer-375.png)

The compact footer stays inside its parent; declared-width table columns use
horizontal scrolling instead of overlapping adjacent cell text. These are
reviewed evidence captures, not replacements for PixelProof baselines.

## Limits and saved evidence

The session hook prohibited Git operations. `sibling-tokens` explicitly skipped
its branch check even though xtask labeled the step PASS. Therefore neither
the 16/16 summary nor the combined receipts establish sibling-branch parity.
No commit, push, source commit SHA, or ETL deployment is claimed.

The full gate was not rerun after the test-profile-only correction: its
39/40 receipt is supplemented by fresh native and full layout-lane results.
This record describes that evidence, not a single 40/40 full-gate run.

Local logs are `.tmp/gfk7-recovery-full.log`, `.tmp/gfk7-profile-native.log`,
and `.tmp/gfk7-profile-layout.log`; the original regression receipt is
`.tmp/gfk7-before.log`. The recovery archive includes these logs, the changed
files, reviewed captures, and a SHA-256 manifest. Its exact location is recorded
in Beads issue `ldui-c0qs`.
