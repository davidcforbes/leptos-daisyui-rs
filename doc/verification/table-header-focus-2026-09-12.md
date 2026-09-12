# Shared table header focus verification - 2026-09-12

Bead: `ldui-cxy8`. Input: `main` at
`0b0ae4e143030c2faeac0443daee248bd02841e6`, plus this local candidate.
No publication, deployment, commit, push or tracker closure is implied.

## Scope and ownership

The two shared sortable-header renderers own this defect: EntityTable's
`src/components/entity_table/component.rs` and DataTable's
`src/components/data_table/header.rs` (also used by ServerEntityTable).
The ETL consumer imports shared tokens correctly; it has no private override.
Generated tokens, the general Button component and the global palette are unchanged.

The focused and held-active background uses `table-header`, matching the existing
`table-header-content` ink. The header focus outline uses the same readable ink,
with a forced-colors CanvasText override. Sort callbacks and query semantics are
unchanged. Default baseline images are not promoted by these tests.

DataTable's idle marker retains its default opacity 0.6 but uses opacity 1 during
hover, keyboard focus and pressing through a named header-button group. Its idle
and sorted symbols, priority and accessible labels are unchanged.

## Observed RED and test corrections

The original ETL real-keyboard probe measured all nine Mirrors/Sources headers
at white/#F8FAFB, 1.0469355409295658:1. Its strict regression exited 101:
`C:/dev/4iiz-etl/.review/portal-focus-red-20260912.{sh,log,exit}`.

The first shared host run used the machine's dark theme and passed; this is not
claimed as RED. After explicitly selecting the demo's light theme, the actual
label and text-span marker measured white/rgb(248,248,248), 1.0620159366897584:1.
`header-focus-light-red-20260912.exit` is 101. This failure preceded the two
production class-string changes. Logs/scripts/exit files are under `.review/`.

The first post-fix targeted test reached correct production contrast but failed
its test-only bad-pair injection: unlayered important CSS could not override
Tailwind's layered important declaration. The final probe temporarily sets inline
important colors with transitions disabled, waits for the actual expected color
pair, then restores the exact previous style and waits for restoration. This
harness failure is not a second product defect. The earlier broad client run
captured that superseded test binary and exited 1; its 1/2 summary is not final
qualification evidence.

## Focused evidence established so far

`header-focus-targeted-green2-20260912.exit` is 0, with 1/1 test passing in 11.75s.
The typed EntityTable proof measured:

- Light/dark keyboard focus and held-pointer press: label and marker 9.894170324565742:1.
- Pure pointer hover: 6.64292456749162:1.
- Forced colors: 21:1, with a visible outline.
- Both left-aligned and numeric/right-aligned fixtures, plus 1280x800 to 1440x900 resize.
- Enter, Space and click each produce one expected sort transition; the typed
  model's displayed projection agrees with rendered rows.
- Restoring white/#F8FAFB fails at 1.0469355409295658:1; independently lowering
  the actual text marker's opacity to 0.01 also fails while label contrast passes.

Further test-only hardening adds explicit bounded color-pair waits, the existing
500ms pointer-settle delay plus computed zero-duration transition assertions, and
offscreen-fixture scroll setup before pointer input. Sequential keyboard evidence
seeds the header position, then requires actual Shift+Tab to leave and actual Tab
to return with `:focus-visible`; programmatic focus alone is not the assertion.

The hardened final typed-header test also passed on the frozen-source release
host: `header-focus-client-final-20260912.exit` is 0, 1/1 in 8.97s. The same
contrast values, both independent negative controls, native sort transitions,
model agreement, right-aligned fixture and resize checks passed with the final
helper. This supersedes the earlier targeted GREEN as the final client receipt.

## Server marker RED and frozen runtime

`header-focus-server-targeted-20260912.exit` is 101. The actual ServerEntityTable
idle marker on pointer-only hover measured 3.5121818812246026:1 at opacity 0.6,
while the label passed at 6.64292456749162:1. The browser reported the marker
present, actual hover true, focus-visible false and transitions disabled. This
observed RED preceded the narrow named-group opacity fix. The broad diagnostic
server run then reported 12 passed / 4 failed: the marker defect plus three
stylesheet-freshness refusals after the patch triggered Trunk's watcher. That
changing-host run is not frozen qualification evidence.

Runtime and ETL test source froze at 18:10 UTC:

- `src/components/entity_table/component.rs` SHA256
  `14D6F622517E9C270A8C2D8586B57E7C624F6151A173DD18088E76BED6DD7BD6`.
- `src/components/data_table/header.rs` SHA256
  `A20FD3C1EAAC5BBC302A95F22F50B4A2852C7379E7526D4ACFC544877537C6D5`.

The required `cargo xtask verify-full` passed on this frozen runtime:
`.review/header-focus-full-20260912.exit` reads 0 and the log's own summary
reads **41/41 passed**. The immutable launcher is the matching `.ps1`; the log
SHA256 is `C449BC891F4625F170092E127DF5BB53591440F8A6A83234C9E0429BB85019B2`.
All 16 native and 25 browser lanes passed, including the complete 32-test client
and 16-test server suites. The parent owns ETL's rebuilt consumer/full-gate
qualification separately; the ETL check now requires exactly five Mirrors and four Sources
headers and measures each label, actual marker and visible outline after native
Enter. It runs in the normal full matrix, not only the diagnostic branch. Shared
Tab/hover/pressed/forced-colors coverage remains distinct from that consumer proof.

## Final focused server receipt

`header-focus-server-final-20260912.exit` is 0: 1/1 passed in 9.34s on the full
gate's freshly built catalog host. The log SHA256 is
`6F825A2BB70A644704C392CA83A28CDC0E594EC90BA4DAFFB20D81E331E8E67E`.

The actual idle marker now has opacity 1 on pure hover and measures
6.64292456749162:1. Focus/held press measure 9.894170324565742:1 for label,
marker and outline; forced colors measure 21:1. Idle, ascending and descending
states pass. Enter and Space each produce exactly one accepted server query;
accepted IDs agree with rendered IDs. Focus, hover, canceled press and negative
controls do not alter accepted query/rows, totals, preferences or proposal counts.
Both the white/#F8FAFB and faint-marker negative controls fail the oracle, then
exact restoration passes.

Focused screenshots were visually inspected: the Started label/direction and
white focus outline are readable on blue, without changing header geometry or
the accepted row presentation. Current screenshots under `.review/header-focus/`:

- `server-history.png`: `E20CE381A4A2AE8DAE5F802EC812EC279307C2E2D5A14F7B124E3F5A44F8142A`.
- `server-history-ascending.png`: `8819F4DB30B27DE316A8194B2B12C129D425D495CB69B7767628FC1BD455DF23`.
- `server-history-descending.png`: `5CBBDC8BE7ECE7388C783B32643A960769694282D9AD9E0C1EF8E4BEA35933AE`.

Final client screenshots under the same directory:

- `entity-left.png`: `E7C373D923B93F400A7E026D28673795FE46FF44AE4BB4864DE86F0D4DC7E6A6`.
- `entity-sorted.png`: `5D37ABE52F9F3B0B6D9CBC001218619951AC15E061D50DB72E4C307438422C67`.
- `entity-right.png`: `71827496232A07E966533E8B5F64152406DB69FD3331240308D078E7A8FD8636`.
- `entity-right-desktop-resize.png`: `B008B3D3A4CE2CA8B02827425E0D72B3B7D53CD01D48C95A6C4F2B1C6BFD254B`.

The final client log SHA256 is
`244482A7831D555F80C50AF8D8E907A5514E91AA60CCE1CD2E6CF6C048AB2B52`.
These focused and full-library receipts remain separate from the parent's ETL
consumer qualification. No baseline files were changed by this shared fix.
Runtime/test source remained frozen throughout the final gate; this final
receipt-only prose update does not change the tested application.
