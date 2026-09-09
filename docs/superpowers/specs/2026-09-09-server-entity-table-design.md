# Server EntityTable Design

**Status:** Approved in chat on 2026-09-09; written contract pending final
user review.

**Scope:** Beads `ldui-9ke9`. Consumer reference:
`C:/dev/4iiz-etl`, Beads `bd_4iiz-etl-namz.1`, History screen
`crates/etl-web-ui/src/screens/history.rs`.

## Outcome

The framework will export `ServerEntityTable`, the canonical opinionated table
for an offset-paged population whose filtering, sorting, paging, total count,
and fetch lifecycle remain server-owned. It will compose the existing
`ServerDataTable` implementation with required controlled query ownership,
stable control identity, standard column preferences, the icon column chooser,
footer page-size/Auto behavior, and fill-parent geometry.

`EntityTable<T>` remains the canonical complete-client-snapshot component.
`ServerDataTable` remains the lower-level compatibility and cursor-paging
primitive. The new facade will not add a server mode to `EntityTable<T>` and
will not download, sort, filter, or page a server population locally.

## Why a Facade

History currently selects and wires the correct lower-level pieces itself:
controlled `TableQuery`, `ServerQueryCapabilities`, authoritative filter
options, `ServerTablePageSizePreference`, viewport fit, column tools,
preference ownership, and a no-op legacy page callback. That composition is
truthful, but it lets every consumer independently decide which pieces of the
"standard EntityTable" experience to omit.

A configuration builder would shorten that call site without making the
standard observable. Adding server behavior to generic `EntityTable<T>` would
erase its defining guarantee that the complete typed snapshot is present in
the browser. A bounded facade over `ServerDataTable` makes the standard path
explicit while preserving the existing server renderer and query state
machine.

## Public Boundary

`ServerEntityTable` will initially support numbered offset pagination with a
known authoritative total. Cursor endpoints continue to use
`ServerDataTable` directly because they cannot truthfully expose numbered
pages or a population range and already carry a distinct controlled cursor
contract.

The facade requires these inputs rather than silently manufacturing them:

- `rows: Signal<Vec<TableRow>>`: the accepted current server page only;
- `columns: Signal<Vec<Column>>`: domain declarations and render metadata;
- `query: Signal<TableQuery>` and `on_query_change: Callback<TableQuery>`:
  one controlled accepted query and full-replacement proposals;
- `total_count: Signal<i64>`: authoritative count for that accepted query;
- `loading: Signal<bool>`: initial or replacement request activity;
- `control_id: String`: stable prefix for every framework-owned control;
- `preference_ownership: EntityTablePreferenceOwnership` and
  `preference_version: u16`: explicit visibility, order, and width ownership;
- `page_size_preference: RwSignal<ServerTablePageSizePreference>`: explicit
  Auto/fixed intent, separate from the server's accepted numeric page size;
- `filter_options` or typed `filter_option_entries` whenever an exact filter
  needs an authoritative vocabulary; and
- `query_capabilities: ServerQueryCapabilities`: explicit endpoint support,
  including History's `with_search(false)` declaration.

The facade keeps bounded extension points already supported by
`ServerDataTable`: cell and typed-cell renderers, stable keyed row activation
and inspection, row classes, selection, toolbar actions, localized text, page
size choices, and displayed-slice observation. These extension points do not
change data ownership. In particular, a displayed-slice callback can never
mean "all filtered rows."

The implementation may group presentation-only inputs into a focused public
configuration type when that avoids duplicating a long prop list, but the
required ownership values above must remain explicit at the call site and
must not acquire permissive defaults.

## Opinionated Behavior

The facade always applies the following standard presentation:

- `data-table-data-mode="server-query"` remains the underlying ownership
  marker, with an additional facade marker for browser audits;
- controlled query state is the sole visible truth; gestures emit proposals
  but do not optimistically change controls, rows, totals, or captions;
- the compact icon gear and its accessible name are always present;
- visibility, order, and column widths use the supplied EntityTable preference
  ownership and schema version;
- the footer contains Rows per page, the authoritative displayed range, and
  numbered navigation in the existing order;
- Auto sizing measures a definite fill-parent viewport and proposes a new
  server page size; a fixed choice remains fixed through resize;
- stable column tracks, named keyboard resize separators, pinned headers, and
  horizontal overflow remain owned by the existing renderer; and
- zebra striping stays off unless explicitly requested, matching the current
  opinionated table hierarchy.

The facade accepts a reactive `viewport_fit` availability signal because an
initial failure has no row geometry from which to authorize another request.
Auto remains a visible retained preference while measurement is temporarily
unavailable. The default minimum is one usable row for a page that explicitly
fills its available height; callers may raise that floor.

## Standard-Column Validation

Every declared domain column in `ServerEntityTable` must declare an exact or
text filter. This includes an action-bearing column such as History's Run
column. A missing filter is a configuration error rendered as a named
`role="alert"` panel with a deterministic data attribute; the facade will not
quietly present a partial standard table.

Exact filters require server-authoritative option entries. Text filters need
no finite vocabulary. Active exact values absent from a newly returned option
list remain visible until the server accepts a replacement query, preserving
the existing controlled behavior.

Column identifiers remain stable `&'static str` values. Required columns may
not be hidden; the last visible column may not be hidden; unknown or stale
preference entries are normalized by the shared EntityTable preference model.

## Server and Failure Semantics

The facade does not fetch data and does not infer request completion. The host
starts transport only after receiving a query proposal and updates `query`,
`rows`, and `total_count` only when that response is accepted. A stale,
failed, or declined request therefore leaves all three accepted values and all
visible controls unchanged.

During revalidation the host may keep accepted rows mounted and set `loading`.
On failure it keeps those rows and accepted values mounted and renders its
failure notice in the surrounding page. Initial failure may hide the table and
gate `viewport_fit` off. This preserves the existing History `Feed` lifecycle
without moving transport strings, retry policy, or request correlation into a
table component.

The browser fixture will make this boundary observable: it will publish the
accepted query, page IDs, total, proposal count, and failure state separately.
A failed proposal must not change the DOM's accepted query markers, row IDs,
range caption, page-size label, chooser state, or preference payload.

## History Adoption Example

The library demo and component guide will include a History-shaped supported
example with more rows than one page. The example will define Run, Module,
Mode, Started, Duration, Verdict, Captured, Rejected, Trigger, and Build
columns, with each filter mapped to the simulated server query rather than to
the displayed slice.

The example will clearly identify the consumer-owned work that remains:

- convert domain records to `TableRow` and provide stable row keys;
- declare column labels, filter kinds, sortable fields, widths, renderers, and
  required/action status;
- translate `TableQuery` into endpoint-specific parameters, including
  substring versus exact semantics and date syntax;
- provide authoritative exact-filter vocabularies;
- correlate requests and accept or reject responses;
- retain the last accepted page during refresh failure; and
- own navigation, authorization, error copy, persistence transport, and
  backend capability enforcement.

The example is framework-owned proof, not an edit or deployment of the ETL
consumer.

## Verification

This web/WASM work follows the existing PixelProof A/B/C/D methodology.

### Native contract tests

Pure tests will prove that standard-column validation rejects any missing
filter, accepts exact and text filters, and never treats an action column as an
exemption. Source guards will prove that `ServerEntityTable` composes
`ServerDataTable` rather than implementing local sort/filter/page operations.
Public rustdoc examples must compile in the ordinary library gate.

### Browser structure and behavior

One release-WASM fixture will use a multi-page in-memory server simulator whose
accepted query is separately inspectable. It will:

- filter every supported column and prove matching rows can come from outside
  the previously displayed page;
- sort and page through accepted server replacements;
- use the real native page-size Select sequence: focus, Space, Home/Arrow,
  Enter, then compare the selected value and accepted query;
- switch Auto/fixed intent and resize the desktop viewport, proving only Auto
  proposes a new server capacity;
- hide, restore, and reorder columns through the gear, comparing the rendered
  table and authoritative preference state;
- resize a column by pointer and Left/Right/Home/End, asserting its named
  separator range and preference readback;
- force a query failure and prove retained rows, accepted query, range,
  controls, and preferences do not move; and
- capture browser errors, WASM panics, axe critical/serious findings, overlap,
  clipping, and the existing zero-slack style/layout reports.

The fixture targets the repository's canonical desktop viewport. Mobile is
not required by the History consumer, but ordinary horizontal-overflow source
guards remain intact.

### Negative controls

Each new oracle must be observed failing and then restored. At minimum the
implementation will temporarily make one filter operate on the current slice,
bypass the missing-filter validator, optimistically apply a failed proposal,
and detach one preference readback. The exact failing assertion and restored
result will be recorded in the verification report.

Focused verification will use the server-table browser lane while iterating.
The final candidate runs `cargo xtask verify-full` from the repository root,
using the actual summary and real process exit code as authority.

## Compatibility

This is additive. Existing `EntityTable`, `DataTable`, and `ServerDataTable`
call sites remain source-compatible. Lower-level callers may continue choosing
only the mechanics they need. Documentation will change the selection table to
name `ServerEntityTable` as the preferred offset-paged server standard and
identify `ServerDataTable` as the lower-level/cursor/compatibility path.

No consumer repository, network endpoint, database, authentication state, or
stored preference payload is changed by this issue.

## Acceptance

The work is complete when the public facade makes the complete standard table
composition the shortest supported server-offset path; the History-shaped
example and docs distinguish framework versus domain ownership; all native,
browser, negative-control, visual, and accessibility evidence passes; Beads is
closed; and the verified commit plus Dolt state are pushed.
