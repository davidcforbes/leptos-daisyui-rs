# DataTable

A production-ready data table with sorting, pagination, filtering, search, selection, row activation, and column resizing. Handles large datasets (10,000+ rows) efficiently by operating on index permutations rather than copying data.

## Which table should I use?

Choose from data ownership before choosing features:

| Data ownership | Use | Runtime marker |
|---|---|---|
| Complete, typed client snapshot | [`EntityTable<T>`](./entity_table.md) | `data-table-data-mode="client-snapshot"` |
| Server-owned query and current-page slice | [`ServerDataTable`](#serverdatatable) | `data-table-data-mode="server-query"` |
| Existing dynamic `HashMap` client rows or DataTable-only features | `components::DataTable` | `data-table-data-mode="compatibility-client"` |
| Existing `Vec<Vec<String>>` table needing automatic link/badge columns or bulk select | `widgets::DataTable` | n/a |

`components::DataTable` remains supported, but it is the compatibility client
path rather than the default for new contracted snapshot pages. Do not feed a
server page to a client table: page-local sort/filter results are not a server
query. The [EntityTable guide](./entity_table.md) documents the migration and
preference-ownership path.

## Description

`DataTable` renders rows of `HashMap<&'static str, String>` against a list of `Column` definitions. Almost every feature is **opt-in**: a table declared with just `data` and `columns` sorts and paginates, and nothing else. Filtering, search, selection, activation and responsive paging each switch on via a single prop or `Column` builder, so adding one never changes the behaviour of a table that doesn't use it.

For server-side pagination (the parent owns the page and fetches each slice), use [ServerDataTable](#serverdatatable) below.

### Opinionated visual hierarchy and stable geometry

The shared client and server renderers emit the framework's semantic table
contract directly: a `#004578` header band with white content, a `#e5f1fb`
aligned filter band with dark content, and a faint `#e0e0e0` border around every
header, filter, body, empty, loading, and detail cell. Zebra striping remains an
explicit opt-in.

Both variants use a fixed table layout and a declared `colgroup`. Sort markers
occupy a reserved slot and mounted header/filter nodes remain keyed by column,
so sorting changes row order and accessible sort state without repainting the
shell, changing column widths, or moving the scroll origin.

### Row identity and absolute indices

Sorting reorders an **index permutation**, never `data` itself. Every index the component hands you — `selected_rows`, `on_row_activate`, `cell_renderers`, `row_class_fn` — is an **absolute index into `data`**, so it survives pagination and sorting. A row on page 5 of a descending sort reports the same index it had on page 1 unsorted.

Without `row_key`, selection is cleared automatically when `data`, the sort
column, or the sort order changes, because those indices may no longer point at
the same row. Supply `row_key` when rows have a stable business identity. The
selection is then remapped by key, and the shared body also reconciles the
mounted row DOM by that key so focus does not slide to a different entity after
replacement or reorder. Empty and duplicate keys fail closed with a visible
table-body alert.

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `data` | `Signal<Vec<TableRow>>` | *required* | Row data, keyed by column id |
| `columns` | `Signal<Vec<Column>>` | *required* | Column definitions |
| `page_size` | `Signal<usize>` | `10` | Fixed rows per page; also the short-viewport fallback for `auto_page_size` |
| `auto_page_size` | `Signal<bool>` | `false` | Derive rows-per-page from rendered height. **Requires a definite height** — see below |
| `min_rows` | `Signal<usize>` | `5` | Auto-sizing usability threshold; below it, retain at least `page_size` rows and scroll |
| `loading` | `Signal<bool>` | `false` | Show the loading row |
| `paginate` | `Signal<bool>` | `true` | `false` hides the pager and shows all rows |
| `searchable` | `Signal<bool>` | `false` | Show a global free-text search box (300ms debounce) |
| `classes` | `DataTableClasses` | `Default` | Per-element class overrides |
| `texts` | `DataTableTexts` | `Default` | User-facing strings (i18n) |
| `sort_texts` | `DataTableSortTexts` | `Default` | Reactive current-state and next-action names for sortable header controls |
| `text_filter_label` | `Signal<String>` | `"Filter {column} by text"` | Reactive accessible-name template for substring filter inputs |
| `class` | `&'static str` | `""` | Additional container classes |
| `table_size` | `Signal<TableSize>` | `Md` | daisyUI table density |
| `zebra` | `Signal<bool>` | `false` | Zebra striping |
| `pin_rows` | `Signal<bool>` | `false` | Sticky header/footer rows |
| `pin_cols` | `Signal<bool>` | `false` | Sticky first column |
| `max_height` | `Option<String>` | `None` | Viewport-constrained scrolling, e.g. `"calc(100vh - 260px)"` |
| `selected_rows` | `Option<RwSignal<BTreeSet<usize>>>` | `None` | Multi-select state (absolute indices). Owned locally if omitted. Supplying it makes rows keyboard-operable |
| `selection_anchor` | `Option<RwSignal<Option<usize>>>` | `None` | Anchor for Shift-range selection |
| `row_key` | `Option<Callback<TableRow, String>>` | `None` | Stable business identity for selection remapping and keyed row DOM |
| `on_row_activate` | `Option<Callback<usize>>` | `None` | Plain click/Enter/Space activates instead of selecting. Supplying it makes rows keyboard-operable |
| `on_sort_change` | `Option<Callback<(&'static str, SortOrder)>>` | `None` | Fired after a header click changes sort state |
| `cell_renderers` | `Vec<CellRenderer>` | `[]` | Custom cell views, indexed by `Column::renderer_index` |
| `typed_cells` | `Vec<TypedCellFn>` | `[]` | Lightweight Badge/Icon cells, indexed by `Column::typed_cell_index` |
| `detail_renderer` | `Option<RowDetailRenderer>` | `None` | Optional row-specific full-width detail content immediately after its source row |
| `row_class_fn` | `Option<Callback<(usize, TableRow), String>>` | `None` | Per-row extra classes from `(abs_idx, row)` |
| `node_ref` | `NodeRef<Div>` | — | Reference to the container `<div>` |

`TableRow` is `HashMap<&'static str, String>`.

## Column

```rust
Column::new("balance", "Balance")          // sortable
Column::new_non_sortable("actions", "")    // not sortable
```

Every sortable header reserves a fixed-width indicator slot. An idle column
shows a quiet bidirectional `⇅` affordance; the active column shows `↑` or `↓`
at full emphasis. Moving the active sort therefore changes neither column
geometry nor the discoverability of the other sortable columns.

| Builder | Effect |
|---------|--------|
| `.with_sort_as(SortAs)` | How cells compare when sorting (see below) |
| `.filterable()` | Give this column a dropdown in the filter row |
| `.filterable_text()` | Give this column a 150ms-debounced, case-insensitive substring input |
| `.with_min_width(u32)` / `.with_max_width(u32)` | Width bounds in px |
| `.with_truncate()` | Ellipsis overflow (pair with `with_max_width`) |
| `.with_class(&'static str)` | Extra classes for this column's cells |
| `.non_resizable()` | Disable drag-resizing (columns are resizable by default) |
| `.with_renderer(usize)` | Index into `cell_renderers` |
| `.with_typed_cell(usize)` | Index into `typed_cells` |

`renderer_index` always wins over `typed_cell_index`; a column with neither renders `row[col.id]` as text.

## Style Variants

### SortAs
- `Text` *(default)* — plain lexicographic comparison
- `Number` — parses formatted numbers, ignoring currency symbols, thousands separators and `%`
- `Date` — parses dates

`SortAs::Number` handles more than plain digits:

| Cell | Parses as |
|------|-----------|
| `"$1,000.50"` | `1000.5` |
| `"9%"` | `9.0` |
| `"-12"` | `-12.0` |
| `"(1,234)"` | `-1234.0` (accounting negative) |
| `"—"` | *missing* |

A leading ASCII `-` or Unicode minus (`−`) also negates. Values that would parse to infinity or NaN are reported missing rather than sorted to an end.

### TableSize
- `Xs`, `Sm`, `Md` *(default)*, `Lg`, `Xl`

### TypedCell
- `Text(String)` — identical to the default text path
- `Badge { text, color }` — a daisyUI badge pill
- `Icon { name, color }` — a Lucide icon by name

## Sorting

Sorting is built in, not enabled by `on_sort_change`: `Column::new` is
sortable by default, and its header contains a native framework `Button`.
Pointer click, Enter, and Space share the same activation path and cycle the
internal sort exactly once. `on_sort_change` only observes that new state. Use
`Column::new_non_sortable` for action or display-only columns; it renders no
sort control or sort tab stop. The narrow right-edge separator remains a
separate focusable resize control and deliberately stops its events from
sorting.

The parent `th` exposes canonical `aria-sort`. The focused Button also names
the localized column, current state, and next action directly. Override the
reactive `sort_texts` signal with `DataTableSortTexts` templates when locale
changes; each template uses `{column}`. This contract is shared by
`DataTable` and `ServerDataTable`.

Cells are strings and compare as **text by default**. A column holding formatted numbers must say so, or it sorts by first digit — `"$1,000"` before `"$900"`:

<details>
<summary>View Code</summary>

```rust
use leptos_daisyui_rs::components::{Column, SortAs};

let columns = vec![
    Column::new("account", "Account"),                              // text
    Column::new("balance", "Balance").with_sort_as(SortAs::Number), // $85 < $900 < $1,000
    Column::new("opened", "Opened").with_sort_as(SortAs::Date),
];
```

</details>

### Full-width per-row detail

Use `RowDetailRenderer` only when explanatory content genuinely varies by
row. The callback receives the same `(absolute_index, TableRow)` identity as a
cell renderer and returns `None` for a normal one-row record or `Some(view)`
for a sibling detail `<tr>`. Its single cell spans the currently rendered
columns and carries the same faint grid border. Because the source row and
detail are emitted from one sorted/filtered/paged item, they move together and
do not participate in column sizing.

```rust,no_run
let details: RowDetailRenderer = Callback::new(|(index, row): (usize, TableRow)| {
    row.get("explanation").filter(|text| !text.is_empty()).map(|text| {
        view! { <p>{format!("{} ({index})", text)}</p> }.into_any()
    })
});

view! {
    <DataTable data=data columns=columns detail_renderer=details />
}
```

Interactive content in the detail row is isolated from source-row activation.
If a sentence comes from a small enum and repeats across many rows, prefer a
legend above the table; repeating it as a detail row doubles height without
adding row-specific information. `ServerDataTable` supports the same renderer,
with its usual page-local index contract.

Cells that don't parse (an em dash for "not measured", say) sort **last in both directions**, the way a spreadsheet puts blanks at the bottom — so a descending sort opens on the largest real value rather than a wall of dashes. A missing value is not a zero.

## Examples

### Basic usage

<details>
<summary>View Code</summary>

```rust
use std::collections::HashMap;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn BasicTable() -> impl IntoView {
    let columns = vec![
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new_non_sortable("status", "Status"),
    ];

    let data = vec![HashMap::from([
        ("name", "Alice".to_string()),
        ("email", "alice@example.com".to_string()),
        ("status", "Active".to_string()),
    ])];

    view! {
        <DataTable
            columns=Signal::derive(move || columns.clone())
            data=Signal::derive(move || data.clone())
            page_size=10
        />
    }
}
```

</details>

### Search and per-column filters

Use `filterable()` for a finite, low-cardinality exact dropdown (status, owner,
type). Use `filterable_text()` for a high-cardinality value such as a job name,
matter number, or destination. Its aligned text input applies a
case-insensitive substring after 150ms: `mat` matches both `zoho-matters` and
`Matter_Timeline`. Empty input removes that filter.

Both kinds share `ColumnFilters`, combine with each other **and** with the
table-wide search, and use AND semantics. `Column::filter_kind()` returns
`Some(ColumnFilterKind::Exact | Contains)` so server consumers can interpret
the same string map without changing its transport shape. A table with neither
builder renders no filter row.

<details>
<summary>View Code</summary>

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn FilteredTable(data: Signal<Vec<TableRow>>) -> impl IntoView {
    let columns = vec![
        Column::new("name", "Name").filterable_text(),   // substring input
        Column::new("role", "Role").filterable(),
        Column::new("status", "Status").filterable(),
    ];

    view! {
        <DataTable
            columns=Signal::derive(move || columns.clone())
            data=data
            searchable=true
        />
    }
}
```

</details>

Option lists derive from the *unfiltered* data, so choosing one option never removes the others. Selections whose value disappears from new data are pruned automatically — a filter pinned to a vanished value would otherwise silently match zero rows.

### Responsive paging (`auto_page_size`)

Derives the row count from the table's rendered height, so a taller window shows more rows. A `ResizeObserver` re-measures on every resize.

<details>
<summary>View Code</summary>

```rust
view! {
    <DataTable
        columns=columns
        data=data
        auto_page_size=true
        max_height="calc(100vh - 260px)"
        min_rows=5
    />
}
```

</details>

**Requires a definite height.** The table must get its height from its layout context, not from its own rows — otherwise the height being measured is a function of the row count derived from it, and the count can never grow. Either:

- pass `max_height`, which `auto_page_size` promotes to a real `height` (a bare `max-height` is only a *ceiling* and would leave the table shrink-wrapping its rows); or
- give the table a parent with a definite height, which it fills.

The pager and search box are flex siblings of the scroll area, so they're excluded from the measurement automatically.

The measured header is the complete `<thead>`. A filter row or wrapped header
labels therefore consume real viewport height. If the resulting fit is below
`min_rows`, the table retains the configured `page_size` (never less than
`min_rows`) and the existing bounded wrapper scrolls. This avoids one-row
pagination for tall badge/wrapped rows while normal-height tables remain fully
responsive.

### Selection and row activation

Both interactions coexist on one table:

- **Plain click** → activates via `on_row_activate` (when provided), otherwise selects.
- **Ctrl/Cmd+click** → toggles selection.
- **Shift+click** → extends the range from the anchor.

Without `on_row_activate`, every click selects exactly as it did before that callback existed.

<details>
<summary>View Code</summary>

```rust
use std::collections::BTreeSet;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn ActivatableTable(data: Signal<Vec<TableRow>>, columns: Signal<Vec<Column>>) -> impl IntoView {
    let selected = RwSignal::new(BTreeSet::<usize>::new());
    let navigate = leptos_router::hooks::use_navigate();

    view! {
        <DataTable
            data=data
            columns=columns
            selected_rows=selected
            on_row_activate=Callback::new(move |abs_idx: usize| {
                // `abs_idx` indexes `data` directly — it already accounts for
                // the current page and sort order.
                navigate(&format!("/detail/{abs_idx}"), Default::default());
            })
        />
    }
}
```

</details>

### Typed cells and custom renderers

<details>
<summary>View Code</summary>

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn RichTable(data: Signal<Vec<TableRow>>) -> impl IntoView {
    let columns = vec![
        Column::new("name", "Name"),
        Column::new("status", "Status").with_typed_cell(0),
    ];

    let typed_cells: Vec<TypedCellFn> = vec![Callback::new(|(_idx, row): (usize, TableRow)| {
        let active = row.get("status").map(|s| s == "Active").unwrap_or(false);
        TypedCell::Badge {
            text: row.get("status").cloned().unwrap_or_default(),
            color: if active { BadgeColor::Success } else { BadgeColor::Neutral },
        }
    })];

    view! {
        <DataTable
            columns=Signal::derive(move || columns.clone())
            data=data
            typed_cells=typed_cells
        />
    }
}
```

</details>

## Custom text (i18n)

`DataTableTexts` supplies visible table chrome:

| Field | Default |
|-------|---------|
| `loading` | `"Loading..."` |
| `empty` | `"No data available"` |
| `previous` | `"Previous"` |
| `next` | `"Next"` |
| `page_indicator` | `"Page {current} of {total}"` |
| `search_placeholder` | `"Search..."` |
| `search_label` | `"Search table"` |
| `row_range` | `"Showing {start}–{end} of {total}"` |
| `filter_all` | `"All"` |
| `filter_label` | `"Filter by {column}"` |

`DataTableSortTexts` supplies the focused sort control's complete accessible
name:

| Field | Default |
|-------|---------|
| `unsorted` | `"{column}, not sorted. Activate to sort ascending."` |
| `ascending` | `"{column}, sorted ascending. Activate to sort descending."` |
| `descending` | `"{column}, sorted descending. Activate to sort ascending."` |

## ServerDataTable

When the server owns pagination, filtering, or sorting, the parent holds the
query and fetches each slice. `ServerDataTable` renders only what it is given;
it does no client-side sorting, filtering, or paging. Its root identifies this
contract as `data-table-data-mode="server-query"` for runtime audits.

| Prop | Type | Description |
|------|------|-------------|
| `rows` | `Signal<Vec<TableRow>>` | The current page's rows only |
| `columns` | `Signal<Vec<Column>>` | Column definitions |
| `current_page` | `Option<Signal<i64>>` | Legacy offset page index; supply with the other three legacy offset props |
| `total_count` | `Option<Signal<i64>>` | Legacy offset population total |
| `page_size` | `Option<Signal<i64>>` | Legacy offset rows per page |
| `on_page_change` | `Option<Callback<i64>>` | Legacy numbered-page callback |
| `pagination` | `Option<ServerTablePagination>` | Explicit offset or cursor strategy; mutually exclusive with legacy offset props |
| `query_capabilities` | `ServerQueryCapabilities` | Endpoint-supported search, page-size, sorting, and filtering controls; defaults to all enabled |
| `on_search` | `Option<Callback<String>>` | Debounced search box; the parent performs the query |
| `on_query_change` | `Option<Callback<TableQuery>>` | Reports the complete server query after paging, search, sort, or filter changes |
| `query_ownership` | `Option<ServerTableQueryOwnership>` | Preferred explicit controlled/uncontrolled ownership; controlled mode supplies displayed-query truth and receives full replacements |
| `query_reset_key` | `Option<Signal<String>>` | Combined dataset/access identity; a change proposes a clean first-page query while preserving page size |
| `page_size_options` | `Signal<Vec<i64>>` | Positive choices for the controlled server-query page-size selector |
| `filter_options` | `Option<Signal<HashMap<&'static str, Vec<String>>>>` | Population-wide choices for exact filterable columns |
| `filter_option_entries` | `Option<Signal<DataTableFilterOptions>>` | Population-wide typed choices with separate stable values and reactive display labels; mutually exclusive with `filter_options` |
| `filter_vocabulary` | `Option<ServerFilterVocabulary>` | Optional explicit vocabulary truth; required as `CurrentSlice` when authoritative `filter_options` are absent |
| `text_filter_label` | `Signal<String>` | Reactive accessible-name template for substring inputs |
| `row_key` | `Option<Callback<TableRow, String>>` | Stable business identity used to reconcile row DOM across server-slice replacement |
| `selection` | `Option<ServerTableSelection>` | Controlled zero-or-one selected business key and replacement callback; requires `row_key` |
| `on_row_activate` | `Option<Callback<usize>>` | Plain click or keyboard activation with the current-page row index |
| `on_row_activate_keyed` | `Option<Callback<ServerTableRowAction>>` | Plain click or keyboard activation with a stable key, page-local index, and displayed row snapshot; requires `row_key` |
| `on_row_inspect` | `Option<Callback<usize>>` | Double-click or Shift+Enter inspection with the current-page row index |
| `on_row_inspect_keyed` | `Option<Callback<ServerTableRowAction>>` | Double-click or Shift+Enter inspection with the same keyed snapshot; requires `row_key` |
| `column_tools` | `Option<ServerTableColumnTools>` | Opt-in compact gear chooser, column visibility/order preferences, and a toolbar-actions slot beside it; see [Column tools](#column-tools-chooser-toolbar-actions-displayed-slice-projection) below |
| `on_displayed_slice` | `Option<Callback<ServerTableDisplayedSlice>>` | Atomic snapshot of exactly the currently displayed columns/rows -- the accepted current server slice only, never a complete-result-set projection |
| `loading`, `classes`, `texts`, `sort_texts`, `class`, `table_size`, `zebra`, `pin_rows`, `pin_cols`, `max_height`, `cell_renderers`, `typed_cells`, `detail_renderer`, `row_class_fn`, `node_ref` | | As `DataTable` |

**Not available**: `selected_rows` / `selection_anchor` (the server variant
has no selection state machine), `auto_page_size`, `paginate`, `searchable`
(use `on_search`), `on_sort_change` (use `on_query_change`).

### Server query ownership

For new server-owned pages, use
`ServerTableQueryOwnership::controlled(current, on_change)`. The supplied
`TableQuery` is the query represented by the displayed slice: its search,
single sort, column filters, page size, and 1-based page drive every
visible control and `aria-sort`. Each user transition proposes one complete
replacement. The component does not optimistically overwrite supplied truth;
a synchronous accepted or normalized replacement is rendered, while a
declined or delayed search, filter, or page-size proposal is reasserted in the
DOM. External Reset or route restoration is therefore an ordinary replacement
of `current` and causes no callback loop.

Bind `query_reset_key` to a collision-safe combination of dataset identity and
access generation. When it changes, the component proposes page one with empty
search/sort/filters and the current page-size choice. This prevents a query
from one authorization or dataset scope leaking into the next. Runtime
localization changes only `texts`, `sort_texts`, and live column headings; it
does not rewrite query state.

The historical `current_page`, `page_size`, `on_page_change`, and
`on_query_change` props remain the uncontrolled offset compatibility path.
Controlled ownership and `on_query_change` are mutually exclusive so a single
gesture cannot be delivered through two query callbacks.

### Server query capabilities

Pagination strategy and query capability are separate contracts. A keyset
endpoint can accept opaque Previous/Next cursors while rejecting search,
page-size, sorting, or filtering. Declare that truth with one cohesive value:

```rust,no_run
let capabilities = ServerQueryCapabilities::navigation_only()
    .with_search(true)
    .with_sorting(true);

view! {
    <ServerDataTable
        rows=rows
        columns=columns
        pagination=cursor_pagination
        query_capabilities=capabilities
    />
}
```

`ServerQueryCapabilities::default()` and `::all()` preserve the historical
full-query behavior. `::navigation_only()` renders only the pagination
strategy's navigation. The four `with_*` builders independently enable search,
page-size selection, sortable headers, and the aligned column-filter row.
Disabling a capability removes its native control and its proposal path;
sortable/filterable column declarations remain reusable and are projected as
inert for that table instance.

A disabled search capability cannot be combined with `on_search` or a
non-empty supplied search. Likewise, disabled sorting/filtering reject a
supplied active sort/filter instead of silently representing a query the
endpoint says it cannot execute. Page size remains part of every query because
the fixed server slice still has a size; disabling page-size capability removes
only the user's ability to change it. Contradictions render `role="alert"`
with `data-server-query-capability-config-error`.

Runtime audits can read `data-server-query-search`,
`data-server-query-page-size`, `data-server-query-sorting`, and
`data-server-query-filtering` (`enabled` or `disabled`) from the table root.

### Truthful server filter vocabularies

A server page is a window, not a population. When filtering is enabled and an
exact column is `filterable()`, supply population-authoritative `filter_options` or
`filter_option_entries`. Existing string maps remain a shorthand where value
equals label. Typed `DataTableFilterOption::new(value, label)` entries keep the
stable query/transport value separate from localized visible copy. `TableQuery`
continues to carry only the value; replacing the entries signal for a locale
change replaces labels without changing accepted query state.

```rust,no_run
let role_options = Signal::derive(move || HashMap::from([(
    "role",
    vec![
        DataTableFilterOption::new("provider.desk", t("Desk provider")),
        DataTableFilterOption::new("provider.crm", t("CRM provider")),
    ],
)]));

view! {
    <ServerDataTable
        rows=rows
        columns=columns
        pagination=pagination
        filter_option_entries=role_options
    />
}
```

The table keeps an accepted query value in the dropdown even if a later option
refresh temporarily omits it, so it remains visible and removable while
metadata and rows load independently. Because no label accompanied that
missing entry, the retained fallback displays the stable value until the
authoritative entry returns. Duplicate stable values or a real option using
the reserved empty value render one fail-closed `role="alert"` header row with
`data-table-filter-options-error="true"`; no aliased dropdown is rendered.
Supplying both string and typed option props is also rejected.

If an endpoint intentionally filters only the displayed slice, declare and
label that narrower truth explicitly:

```rust,no_run
let current_slice_copy = Signal::derive(move || {
    if spanish.get() {
        ServerCurrentSliceFilterTexts::new(
            "Todos en esta página",
            "Filtrar la página actual por {column}",
        )
    } else {
        ServerCurrentSliceFilterTexts::default()
    }
});

view! {
    <ServerDataTable
        rows=rows
        columns=columns
        pagination=pagination
        filter_vocabulary=ServerFilterVocabulary::current_slice(current_slice_copy)
    />
}
```

Current-slice mode derives distinct values from the displayed rows and uses
its dedicated reactive labels for both the empty option and accessible select
name. Combining it with authoritative `filter_options` is contradictory and
fails visibly. Omitting both contracts also renders a `role="alert"` with
`data-server-filter-vocabulary-config-error` and suppresses the filter row; it
never presents one page's values as if they covered the server population.

`filterable_text()` does not consume or derive a finite vocabulary. A
text-only server filter row therefore needs no `filter_options`; the parent
reads its own matching `Column` definition and applies a case-insensitive
contains predicate when handling `TableQuery.filters`. In a mixed row, only
the exact columns require authoritative or explicitly current-slice options.
When filtering capability is disabled or no column is filterable, no
vocabulary is required and no filter control or proposal path exists.

Runtime audits can read `data-server-filter-vocabulary` as `authoritative`,
`current-slice`, `disabled`, or `invalid`.

### Cursor pagination

Use `ServerTablePagination::cursor(ServerCursorPagination::controlled(...))`
for keyset APIs that return opaque previous/next cursors without a population
total. The caller supplies a controlled `ServerCursorQuery`, the accepted
`ServerCursorPage` metadata for the rows currently displayed, and one
full-query replacement callback. No page number, total, or offset range is
rendered. The root exposes `data-server-pagination-strategy="cursor"` for
runtime audits.

`ServerCursorToken` is deliberately opaque: the component only returns a
cloned token through `ServerCursorRequest::Previous` or `Next`; it never parses
or exposes the value. Enabled search, sort, filter, page-size, Reset, and
reset-key changes all set the request to `ServerCursorRequest::First`.
Previous and Next each emit exactly one replacement. For a fixed-slice
endpoint, pass `ServerQueryCapabilities::navigation_only()`; no unsupported
query-shape control is rendered or armed.

The page metadata also carries a typed `ServerCursorSliceState`. Use
`RetainedWhileLoading` while a new request is pending and
`RetainedAfterFailure` when the latest request failed. Existing rows remain
visible, the status caption describes them as retained, and navigation is
disabled while loading. `ServerCursorTexts` localizes those captions without
adding cursor-only fields to `DataTableTexts`.

The explicit `pagination` prop rejects any simultaneous legacy offset prop,
and cursor pagination rejects the offset-only `query_ownership` and
`on_query_change` props. A configuration error renders a `role="alert"`
instead of silently picking one strategy. Existing offset callers remain
source-compatible: omitting `pagination` and supplying the historical four
offset props constructs the offset strategy automatically.

### Stable server-row identity

Server pages should supply `row_key` whenever a record has a canonical id. It
is more than a convenient `data-*` attribute: the key is the keyed-rendering
identity for the source row and its optional detail row. Reordering a retained
slice moves the existing node; insertion/removal preserves unaffected nodes;
and cursor replacement cannot recycle page position zero for a different
entity. A same-key row update refreshes its cells and callback snapshot while
keeping the source row mounted.

Prefer the keyed callbacks for navigation or inspection that may cross an
asynchronous boundary. `ServerTableRowAction` contains `key`, `page_index`, and
the exact displayed `row`, so the consumer never has to look up an index in a
newer server slice. The historical index callbacks remain source-compatible
and may be supplied alongside them.

```rust,no_run
view! {
    <ServerDataTable
        rows=rows
        columns=columns
        pagination=pagination
        row_key=Callback::new(|row: TableRow| row["matter_id"].clone())
        selection=ServerTableSelection::controlled(
            selected_matter.into(),
            Callback::new(move |proposed| selected_matter.set(proposed)),
        )
        on_row_activate_keyed=Callback::new(|action: ServerTableRowAction| {
            open_matter(action.key, action.row);
        })
    />
}
```

The contract rejects whitespace-only keys and duplicate keys within a
displayed slice. In either case the body renders one `role="alert"` row with
`data-table-row-key-error="true"` and no data rows. Supplying a keyed callback
without `row_key` renders a root configuration alert with
`data-server-row-key-config-error`; it never falls back to a racing index
lookup.

`ServerTableSelection` is strictly controlled. The supplied `selected_key`
signal alone determines selected styling and `aria-selected`; a click, Enter,
or Space emits `Some(clicked_key)` without mutating it. A rejected or delayed
proposal therefore leaves the accepted row painted, with no feedback loop.
When the accepted key is absent from a cursor slice, every displayed row is
unselected; returning to a slice containing it paints that key again. The
caller decides when to clear a selection that leaves the current slice.

Ctrl/Meta/Shift-modified row gestures are inert because this is a
single-selection contract, not a hidden multi-select state machine. When
selection and activation callbacks are both supplied, the selection proposal
is emitted first and the explicit activation callback then fires from the same
plain gesture. Double-click keeps the existing contract: its first click takes
the plain path once, its repeat click is swallowed, and inspection fires once.

### Controlled checkbox multi-selection (`ldui-px06`)

`ServerTableSelection` above is deliberately single-select. For a bulk
workflow — select several visible rows, then act on all of them through one
caller-owned mutation — supply `multi_selection` instead. It renders a leading
checkbox column plus a header checkbox, keyed by the `row_key` the table
already requires.

```rust,no_run
let accepted = RwSignal::new(BTreeSet::<String>::new());

view! {
    <ServerDataTable
        rows=rows
        columns=columns
        pagination=pagination
        row_key=Callback::new(|row: TableRow| row["conversation_id"].clone())
        multi_selection=ServerTableMultiSelection::controlled(
            accepted.into(),
            Callback::new(move |proposal: ServerTableSelectionProposal| {
                if proposal.scope != current_dataset.get_untracked() {
                    return; // a proposal minted against the previous dataset
                }
                accepted.set(proposal.keys);
            }),
        )
            .with_scope(current_dataset.into())
            .with_row_label(Callback::new(|row: TableRow| row["subject"].clone()))
            .with_row_selectable(Callback::new(|row: TableRow| {
                if row["status"] == "Archived" {
                    ServerTableRowSelectability::blocked("Archived conversations cannot be reassigned")
                } else {
                    ServerTableRowSelectability::Selectable
                }
            }))
    />
}
```

#### What the header checkbox means

It means **the rows on this page**, and nothing else. A server table only ever
holds one slice, so a "select all" that appeared to reach rows the client has
never seen — followed by a bulk mutation applied to them — is the hazard this
model exists to make unrepresentable. The behaviour, the state machine
(`ServerTableSliceSelectionState`), the default copy, and the emitted
`ServerTableSelectionCause::CurrentSlice { keys }` payload all say *current
slice*; nothing in the component can name a row the caller did not render.

| Header state | Meaning | DOM |
|---|---|---|
| `NoSelectableRows` | the slice has no selectable rows at all | unchecked, `disabled` |
| `None` | no selectable displayed row is accepted | unchecked |
| `Partial` | *some but not all* selectable displayed rows are accepted | `indeterminate` |
| `All` | *every* selectable displayed row is accepted | checked |

**Accepted keys that are not on the current page never affect this.** They do
not force `Partial`, and they do not stop a fully-selected page reading as
`All`. Letting unseen rows tint the header would be the component asserting
something about a population it does not hold. They are surfaced instead as
their own explicit line of copy in a `role="status"` region
(`data-server-selection-off-slice-notice`), defaulting to *"N selected rows are
not on this page"*, alongside `data-server-selection-off-slice` on the
selection status wrapper. Every default string names *this page*; none of them
says "all".

Activating the header checkbox adds the slice's selectable keys when the state
is anything but `All`, and removes exactly those keys when it is `All`. Either
way, keys outside the slice are carried through untouched — which is how a
selection built across several pages survives paging without a preservation
step that could be forgotten.

#### Accepted truth is caller-owned

The component holds no selection state. `selected_keys` is displayed truth and
every gesture emits one `ServerTableSelectionProposal` carrying the **complete**
proposed set (not a delta), which the caller applies or declines wholesale.
Both checkboxes re-assert the accepted value onto the DOM element the browser
just toggled *before* emitting, so a declined or delayed proposal leaves no
optimistic divergence: the rendered checkbox, `aria-selected`, and the row
styling all stay aligned with the caller's signal.

#### Dataset identity

`with_scope` stamps every proposal with the dataset identity it was computed
against (`ServerTableSelectionProposal::scope`, also mirrored to
`data-server-selection-scope`). When the meaning of a key changes — a different
tenant, a re-scoped query, a new cursor stream — move the scope and clear the
accepted set in the **same** caller update, and reject any proposal whose
`scope` no longer matches. The component never clears the caller's set itself,
because clearing is exactly the atomic decision the caller owns.

Selection is keyed by stable business identity, never by page position, so a
replaced page whose row 0 is a different entity cannot inherit "row 0 is
selected". An accepted key for a row the server no longer returns simply stops
matching anything on screen and is counted in the off-slice notice; it is never
silently dropped and never aliased onto another row.

#### Accessibility

The header checkbox exposes checked / unchecked / `indeterminate` (a DOM
property, not an attribute — it is written as one), is disabled only when the
slice has nothing selectable, and is named for the slice
(*"Select all N rows on this page"*). Every row checkbox is named for its row
via `with_row_label` (falling back to the stable key), and toggles with `Space`
like any native checkbox — the leading cell stops click/keydown propagation, so
a checkbox gesture never also fires `on_row_activate`. A blocked row's
checkbox uses `aria-disabled` rather than the native `disabled` attribute, so
it stays in the tab order and its reason (`title`, and folded into the
accessible name) is reachable by keyboard as well as by pointer. Multi-selection
does not make rows themselves focusable: the gesture already lives on a real
focusable control inside the row, and a second tab stop per row would double the
keyboard cost of the table without adding a reachable action.

#### Configuration errors are rejected, not resolved

Supplying `selection` and `multi_selection` together renders a `role="alert"`
panel (`data-server-row-key-config-error`) instead of quietly honouring one of
them — silently picking would make a bulk-assignment gesture act on a single
row, or the reverse. `multi_selection` without `row_key` is rejected the same
way. Omitting `multi_selection` leaves rendering and single-selection behaviour
byte-for-byte as they were: no checkbox column, no status region, no extra
column track.

### Column tools (chooser, toolbar actions, displayed-slice projection)

`ServerDataTable` can opt into `EntityTable`-style presentation without
becoming a client-snapshot table: a compact gear column chooser, stable
column visibility/order preferences under a caller-owned namespace, and a
toolbar-actions slot beside the chooser (the natural home for an Export
button). It is entirely optional and source-compatible: omit `column_tools`
and rendering is exactly as before.

```rust,no_run
view! {
    <ServerDataTable
        rows=rows
        columns=columns
        pagination=pagination
        column_tools=ServerTableColumnTools::new(
            EntityTablePreferenceOwnership::uncontrolled(
                EntityTablePreferencePersistence::LegacyLocalStorage {
                    storage_key: "matters-table",
                },
            ),
            1, // schema_version
        )
            .with_chooser_trigger(EntityColumnChooserTrigger::Icon)
            .with_toolbar_actions(move || view! {
                <Button on_click=export_current_slice>"Export this page"</Button>
            }.into_any())
        on_displayed_slice=Callback::new(move |slice: ServerTableDisplayedSlice| {
            displayed_slice.set(slice);
        })
    />
}
```

Mark a column `Column::new(id, header).required()` to forbid the chooser from
ever hiding it; a required column is not even offered as a toggle. Reordering
and hide/show both reach `effective_columns` immediately, so the rendered
header, body, and stable column tracks always agree with what the chooser
shows checked. `EntityColumnChooserTrigger`, `EntityTablePreferenceOwnership`,
`EntityTablePreferencePersistence`, and the pure column-order/visibility
functions behind the chooser are all reused directly from `EntityTable` —
this is the same accessible, viewport-safe dropdown (`Escape` closes and
returns focus to the trigger; the menu never spills outside the viewport),
not a reimplementation.

**`ServerTableDisplayedSlice` is not `EntityTableDisplayProjection`.** It has
no `AllFiltered` scope and never will: a server-paginated table holds only
the page or cursor slice its caller fetched into `rows`, and `on_displayed_slice`
fires exactly that — the columns and rows currently rendered, nothing more.
Building an "export all" or "export filtered" feature from this value is a
type error waiting to happen turned into a naming impossibility instead: read
the type's own doc comment before wiring an export action, and label any UI
built from it against the visible page/slice ("Export this page"), never
"all" or "filtered".

## Helper functions

All pure and independently usable:

| Function | Purpose |
|----------|---------|
| `compare_cells(a, b, sort_as, order)` | The comparator behind sorting |
| `parse_number(cell)` / `parse_date(cell)` | Cell parsers (`None` = unparseable) |
| `column_sort_as(columns, col_id)` | A column's declared `SortAs` |
| `distinct_values(data, col_id)` | A column's sorted, deduped, non-empty values |
| `row_matches_filters(row, filters)` | Whether a row satisfies active filters (AND) |
| `row_matches_column_filters(row, columns, filters)` | Exact/contains matching using each column's declared kind |
| `prune_stale_filters(filters, options)` | Drop selections whose value vanished |
| `prune_stale_column_filters(filters, options, columns)` | Drop stale exact values while retaining free-form contains text |
| `has_filterable_columns(columns)` | Whether to render a filter row |
| `has_exact_filterable_columns(columns)` | Whether a server filter row needs a finite vocabulary |
| `rows_per_page_for_height(viewport, header, row)` | The raw geometric fit used by `auto_page_size` |
| `auto_page_size_for_height(viewport, header, row, configured, min_rows)` | Responsive fit with the short-viewport fallback policy |
| `page_count(total, size)` / `clamp_page(...)` / `page_bounds(...)` | Shared zero-based client-pagination state used by both client table implementations |
| `handle_row_click(...)` / `row_click_kind(ctrl, shift, has_activate)` | Selection/activation state machine |
| `cell_text` / `row_text` / `row_with_headers_text` | Clipboard export (tab-separated) |

## Consumer `input.css`

A Rust dependency does not deliver compiled CSS. The consuming application
must import the generated token sheet and make Tailwind scan this crate's Rust
source. For sibling checkouts, adjust these paths relative to the consumer's
stylesheet:

```css
@import "../leptos-daisyui-rs/styles/tokens.css";
@source "../leptos-daisyui-rs/src/**/*.rs";
```

Do not copy the palette into app-local CSS or rely on the library demo output.
The component source emits the semantic classes; the generated token import
defines their shared values. The following inline sources are needed only when
the consumer does not already scan the library source or for classes assembled
dynamically:

```css
@source inline("table table-zebra table-pin-rows table-pin-cols table-xs table-sm table-md table-lg");
@source inline("btn btn-sm animate-pulse");
/* Column-resize divider */
@source inline("relative absolute top-0 right-0 z-10 h-full w-1.5 cursor-col-resize select-none");
@source inline("opacity-0 hover:opacity-100 hover:bg-primary/50 active:opacity-100 active:bg-primary/70");
/* Typed cells */
@source inline("badge badge-neutral badge-primary badge-secondary badge-accent badge-info badge-success badge-warning badge-error");
@source inline("inline-block w-4 h-4 w-5 h-5 w-6 h-6 w-8 h-8 w-12 h-12");
/* Pagination */
@source inline("flex justify-between items-center mt-4 gap-2");
@source inline("btn btn-sm join join-item btn-active btn-disabled");
@source inline("text-sm text-base-content/60");
/* Per-column filter row */
@source inline("select select-bordered select-xs input input-bordered input-xs w-full font-normal p-1");
```

### `multi_selection` classes

The leading selection column reuses the `Checkbox` component, so a consumer
that already lists the checkbox classes needs nothing new:

```css
@source inline("checkbox checkbox-sm");
```

The status region's type step comes from `.ld-text-small`, which is an
authored rule in `styles/tokens.css` — do **not** add `ld-text-*` to
`@source inline(...)`.

## Accessibility

- Headers are `role="columnheader"` with `aria-sort` reflecting the current state (`ascending` / `descending` / `none`).
- Resize handles are focusable `role="separator"` controls with
  `aria-orientation="vertical"`, a unique localized column name, and ordered
  `aria-valuemin`/`aria-valuenow`/`aria-valuemax` plus value text. Left/Right
  resize by 16 pixels; Home/End select the allowed bounds. Keyboard and pointer
  paths share the same clamp logic and never activate sorting or scroll the
  page.
- The search box and every exact or substring filter control have both a localized accessible name and a real associated visually-hidden `<label>`. Placeholder text and physical column position are never the naming mechanism.
- Sort state changes are conveyed through `aria-sort` rather than the `▲`/`▼` glyph alone.
- **Keyboard operation.** When the table is interactive — `selected_rows` or `on_row_activate` supplied — each row is focusable (`tabindex=0`) and carries `aria-selected`. **Enter** and **Space** do exactly what a plain click does (activate, or select); **Ctrl/Cmd** and **Shift** with Enter/Space toggle and range-extend selection, mirroring the mouse. Space suppresses its default page-scroll. A plain display table (neither prop) adds no tab stops.

**Known gap**: rows are keyboard-operable but the table does not yet expose full `role="grid"` semantics (per-cell `role="gridcell"`, arrow-key roaming). Screen readers announce rows and their selected state, but not "row 3 of 20, column 2". For a spreadsheet-grade grid this is a larger, separate change.

## Best Practices

1. **Choose from data ownership first.** New typed snapshots use `EntityTable`; server slices use `ServerDataTable`; this dynamic client component is the compatibility path.
2. **Declare `SortAs::Number` on every money, percentage, duration or count column.** Text order puts `"$1,000"` before `"$900"`, and the bug looks like working software.
3. **Match filter control to cardinality.** Use `filterable()` for finite status/owner choices and `filterable_text()` for names, ids, and other high-cardinality substrings.
4. **Pair `auto_page_size` with `max_height`** (or a definite-height parent). Tune `min_rows` only when five is inappropriate; `page_size` is the intentional scroll fallback below that threshold.
5. **Treat every index as absolute.** Don't add the page offset yourself — it's already applied.
6. **Re-check selection after mutating `data`.** It's cleared on data and sort changes by design.
7. **Prefer `typed_cells` over `cell_renderers`** for badges and icons; reach for a full renderer only when you need arbitrary views.
8. **Use `paginate=false` for small fixed tables**, not `page_size=1000`.
9. Import the generated tokens and scan the library source in `input.css`; add
   inline sources only for classes Tailwind cannot see because they are built at
   runtime.
