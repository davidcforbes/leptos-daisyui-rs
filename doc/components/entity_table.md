# EntityTable

`EntityTable<T>` is the opinionated table for a complete, typed client
snapshot. The caller downloads the whole selected dataset, supplies stable row
keys and typed column functions, and the component owns transient paging while
sorting, resizing, column visibility, and column order stay local. When the snapshot needs
filtering, the caller supplies the locally filtered snapshot and changes
`page_reset_key` to return paging to the first page.

## Choose the table from data ownership

| Data ownership | Component | Observable mode | Rule |
|---|---|---|---|
| Complete typed snapshot is already in the browser | `EntityTable<T>` | `client-snapshot` | Preferred for new contracted snapshot pages. |
| Server owns filtering, sorting, paging, and total count | `ServerDataTable` | `server-query` | Pass only the current slice and round-trip every query change. |
| Existing client table uses dynamic `HashMap` rows or DataTable-only features | `components::DataTable` | `compatibility-client` | Compatibility path; do not choose it for a new contracted snapshot by habit. |
| Existing simple table needs automatic link/badge columns or bulk selection | `widgets::DataTable` | n/a | Retained legacy widget with a different row model. |

Do not pass a server page to `EntityTable` and let it sort or filter that slice.
That silently changes a server query into page-local behavior. Do not download a
complete dataset merely to satisfy `EntityTable` when the server must own the
query. The component roots expose `data-table-data-mode` so browser audits can
detect an ownership mismatch on the running page.

## Shared mechanics, separate data models

`EntityTable` and the DataTable family intentionally keep different row and
column types, but they do not carry separate behavior for common mechanics:

| Mechanic | Shared owner |
|---|---|
| Page count, clamping, bounds, numbered window, and row-range caption | `components::data_table::pagination` |
| Resize minimum, maximum, and drag-delta bounds | `components::data_table::resize` |
| Show/hide transition, required-column guard, and last-visible guard | `components::data_table::chooser` |

The renderers remain separate because `EntityColumn<T>` resolves typed rows,
whereas `Column` resolves dynamic `TableRow` maps. Sharing those renderers would
erase the compile-time distinction the snapshot component exists to provide.

## Core inputs

| Prop | Purpose |
|---|---|
| `data: Signal<Rc<Vec<T>>, LocalStorage>` | The complete selected snapshot. |
| `columns: EntityColumns<T>` | Static `Vec` compatibility or local reactive typed declarations; stable IDs own preference identity. |
| `row_key: EntityRowKey<T>` | Stable identity used for keyed DOM rows and activation. |
| `dataset_identity: Signal<String>` | Identifies the downloaded dataset; a change resets only the current page. |
| `page_reset_key` | Optional identity for local view-state changes that should reset only paging. |
| `compact_row: EntityCompactRow<T>` | Default, static, or reactive single-cell renderer used at compact breakpoints without duplicating rows. |
| `column_filters: EntityColumnFilters` | Controlled one-to-one filter controls aligned beneath stable desktop columns. |
| `source_data` | Optional authoritative source membership used only for safe post-removal focus recovery. |
| `focus_scope` | Optional opaque dataset/access generation; recovery never crosses a change. |
| `preference_ownership` | Controlled or uncontrolled preference policy. |
| `storage_key` | Legacy local-storage compatibility prop; mutually exclusive with `preference_ownership`. |

Page number, free-text search, selected dataset, row data, and snapshot revision
are transient state and do not belong in `EntityTablePreferences`.

## Reactive column semantics

Existing `columns=vec![...]` calls remain source-compatible. A localized or
otherwise runtime-defined table can instead pass
`Signal<Vec<EntityColumn<T>>, LocalStorage>`. Each replacement advances an
internal semantic generation: headers, chooser copy, default compact labels,
sort/accessibility names, and comparator or sort-key behavior all switch to
the new declarations. The sorted-index cache includes that generation, so an
unchanged row `Rc` and sort value can never reuse indices from an obsolete
comparator.

Preferences normalize by stable column ID after replacement. Surviving order,
visibility, widths, sort clauses, and page size remain intact; removed IDs and
newly non-sortable sort clauses disappear, and new IDs append in system order.
A label-only locale update therefore updates mounted header nodes without
resetting consumer state.

`EntityTableTexts` is a live signal and covers every framework-owned visible
or accessible string, including column-order actions, resize names/value text,
sort state/action/summary copy, region name, pagination, and empty state.

## Hybrid aligned filters

Use `EntityColumnFilter::new("stable_column_id", renderer)` for a controlled
filter that maps one-to-one to a column. `EntityColumnFilters` renders one
second-header cell for every visible column and places the control only in its
target cell. Reorder and visibility use the same ordered descriptor list as
the header and body, so tracks cannot drift. Filter cells stop pointer,
keyboard, and pointer-down propagation; selecting a value cannot sort, resize,
or activate a row.

Keep global search and controls that do not map to one column in the utility
`FilterBar`. The complete pattern has exactly one Reset and one Save as Default
there; it does not duplicate column controls above the table. See
[`client-snapshot-list.md`](../patterns/client-snapshot-list.md).

## Visual hierarchy and shell stability

`EntityTable` emits the canonical `#004578` header with white content, the
aligned `#e5f1fb` filter band with dark content, and a collapsed faint `#e0e0e0`
grid across every table cell. Zebra striping is off unless requested. Its fixed
layout and declared `colgroup` keep column tracks stable; sorting updates rows,
markers, and accessibility state in place without replacing the table/header
nodes or moving the shell.

These are generated semantic utilities, not demo-only CSS. Every consuming
Tailwind build must import `leptos-daisyui-rs/styles/tokens.css` and scan
`leptos-daisyui-rs/src/**/*.rs`, with paths resolved from its own `input.css`.
See the [DataTable consumer CSS setup](./data_table.md#consumer-inputcss).

## Row-action focus recovery

Wrap repeatable action controls in `EntityRowAction action_id="..."`. When a
focused source row is actually removed within the same `focus_scope`, the
table uses its real filtered/sorted/paged order and visible position to focus
the same enabled, visible action on the row that moved into that position. A
last-row removal clamps to the preceding visible row.

If filtering or paging merely hides a row that remains in `source_data`, if
the matching action is absent/disabled/hidden, or if no neighbor remains,
focus goes to the named programmatically focusable table region. Dataset or
access-generation changes clear recovery without cross-focusing. A declined
or failed action that leaves the row present retains native focus, and focus
already moved by the user is never stolen. Consumers must not reproduce this
with DOM queries or source-order guesses.

## Preference ownership

### Controlled: recommended for governed persistence

Controlled mode makes the consumer the only source of truth. Every table UI
operation emits one normalized, complete `EntityTablePreferences` replacement.
The component performs no browser-storage I/O and keeps rendering the supplied
signal until the consumer accepts a replacement.

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    EntityTablePreferenceOwnership, EntityTablePreferences,
};

let preferences = RwSignal::new(EntityTablePreferences::new(3));
let ownership = EntityTablePreferenceOwnership::controlled(
    preferences.into(),
    Callback::new(move |replacement| {
        // Validate/save through the page's governed persistence path, then
        // accept the returned value. This in-memory assignment illustrates
        // the acceptance step.
        preferences.set(replacement);
    }),
);

// Pass `ownership` through the EntityTable `preference_ownership` prop and
// keep `preference_version=3` aligned with the DTO schema.
```

Normalization is pure and deterministic. A schema-version mismatch resets to
defaults. Ordered columns keep the first occurrence of each known id and append
missing declarations in system order. Sort clauses likewise keep the first
occurrence of each sortable known id while removing duplicates, unknown ids,
and non-sortable columns. Required ids are removed from `hidden_columns`, and
widths use the shared DataTable bounds.

## Ordered columns and multi-column sort

`EntityTablePreferences::column_order` is the complete ordered list of stable
column ids. The table renders wide headers, wide cells, and the default compact
row in that same order. The chooser exposes an ordered list with named “move
earlier” and “move later” buttons. Boundary buttons are disabled, and focus is
restored to the corresponding control after a move.

`EntityTablePreferences::sort` is an `EntitySort`. The historical public
`System`, `Ascending`, and `Descending` variants remain available for
single-column source code; `Multiple` carries two or more ordered clauses:

```rust,no_run
use leptos_daisyui_rs::components::{EntitySort, EntitySortColumn};

let sort = EntitySort::multiple([
    EntitySortColumn::ascending("status"),
    EntitySortColumn::descending("client"),
]);
```

Clauses are compared lexicographically, with stable dataset order breaking full
ties. Text keys are extracted once per row for each active text clause rather
than allocated inside the comparison loop.

A plain header activation keeps the compatibility single-sort cycle: ascending,
descending, then system order. Shift+activation edits one clause without
discarding the others: absent becomes ascending, ascending becomes descending,
and descending is removed. Markers and accessible labels expose every clause's
direction and one-based priority. Only the primary header carries `aria-sort`,
as required for a table with one primary ordering; a polite live summary
announces the complete clause sequence.

Resizable headers expose focusable vertical separators with `aria-valuemin`,
`aria-valuemax`, and live `aria-valuenow`/`aria-valuetext` semantics. Left and
Right resize by 16 pixels, while Home and End select the allowed minimum and
maximum. Keyboard and pointer changes use the same shared bounds; in controlled
mode each completed keyboard action emits one normalized preference
replacement and performs no browser storage I/O. The shared `DataTable` header
uses the same keyboard resize math and separator semantics.

### Interaction and accessibility release evidence

A green screenshot, style audit, or layout audit is Layer A evidence; it does
not prove that an interactive table is keyboard-operable or exposes accurate
semantics. Before approving a table change, enumerate every supported operation
and attach browser evidence for each one:

| Operation | Required browser evidence |
|---|---|
| Sort | Real Tab plus Enter/Space and Shift-modified activation; one activation per key; accurate current/next-action labels; primary-only `aria-sort`; priorities, live summary, rendered rows, and controlled model agree. |
| Resize | A visibly focusable, uniquely named vertical `separator`; ordered `aria-valuemin <= aria-valuenow <= aria-valuemax` plus value text; real Left/Right/Home/End input; no scroll or sort activation; rendered width and controlled model agree. |
| Reorder | Named earlier/later controls include position; boundary state is disabled; focus follows the moved column or its enabled opposite at a boundary; DOM and model order agree. |
| Visibility | Keyboard-operable chooser state agrees with rendered columns; required-column and last-visible guards remain enforced. |
| Paging | Current page and boundary-disabled states are exposed; keyboard button activation changes the row range and rendered rows exactly once. |
| Compact rows | Compact content follows the same normalized column order and preserves names/actions without duplicate hidden focus targets. |

The shared minimum-width helper caps every public `min_width`, including
`u32::MAX`, at the global maximum. That keeps the ARIA range ordered and every
direct `f64::clamp` range valid. Prove the extreme input in a pure regression
test and prove focus plus key operation in the real DOM; either test alone is
insufficient. An axe run complements these checks but cannot prove keyboard
operation. Keep an inject/catch/revert negative control for a key behavior or
focusability assertion so a green journey demonstrates detection, not merely
execution.

### Uncontrolled without persistence

This is the default when neither ownership nor `storage_key` is supplied. The
component owns an in-memory signal for its lifetime and never reads or writes
`localStorage`.

```rust,no_run
use leptos_daisyui_rs::components::{
    EntityTablePreferenceOwnership, EntityTablePreferencePersistence,
};

let ownership = EntityTablePreferenceOwnership::uncontrolled(
    EntityTablePreferencePersistence::Disabled,
);
```

### Legacy local storage

Existing callers can keep the historical prop unchanged:

```rust,ignore
<EntityTable
    // required snapshot props omitted
    storage_key="no-hires"
    preference_version=2
/>
```

It resolves to uncontrolled `LegacyLocalStorage` with the existing
`ldui-entity-table:<storage_key>` key. Supplying both `storage_key` and
`preference_ownership` fails closed instead of silently allowing a controlled
table to perform browser I/O.

Legacy serialized single-sort values (`System`, `Ascending`, and `Descending`)
remain readable. The next write uses the canonical sort-clause array, and a
legacy payload without `column_order` normalizes to the declared system order.

This schema addition is an intentional source migration for consumers that use
an `EntityTablePreferences` struct literal or exhaustively match `EntitySort`.
Add `column_order: Vec::new()` to legacy literals, and handle
`EntitySort::Multiple { clauses }` (or consume `sort.clauses()`) in exhaustive
matches. Vendor the framework and those consumer changes atomically.

## Migration path

For an existing `EntityTable` using `storage_key`, preserve that prop until the
consumer has a governed persistence endpoint. Then load the saved preference
DTO into a signal, pass controlled ownership, handle each full replacement in
the page, and remove `storage_key`. Keep `preference_version` stable unless the
preference schema actually changes.

Existing `components::DataTable` call sites remain compatible. Migrate a call
site to `EntityTable<T>` only when it represents a complete snapshot and its
required DataTable-only features have typed equivalents. Existing server-owned
pages migrate directly to `ServerDataTable`; they must not pass one fetched page
through either client table.

## Verification

The focused inner and browser lanes are:

```powershell
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser
```

The browser lane checks real keyboard reorder with boundary-safe focus
restoration, real Enter/Shift+Enter and Shift-click multi-sort, real keyboard
resize with controlled-model readback, paging, chooser behavior, controlled
preference mount behavior without browser storage reads or writes, compact
rendering, row/action activation, a vendored axe-core audit with the chooser
open, style oracles, and the `client-snapshot` ownership marker. Native tests
retain explicit coverage of the legacy local-storage compatibility path.
