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

### Row identity and absolute indices

Sorting reorders an **index permutation**, never `data` itself. Every index the component hands you — `selected_rows`, `on_row_activate`, `cell_renderers`, `row_class_fn` — is an **absolute index into `data`**, so it survives pagination and sorting. A row on page 5 of a descending sort reports the same index it had on page 1 unsorted.

Selection is cleared automatically when `data`, the sort column, or the sort order changes, because those indices may no longer point at the same row.

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
| `class` | `&'static str` | `""` | Additional container classes |
| `table_size` | `Signal<TableSize>` | `Md` | daisyUI table density |
| `zebra` | `Signal<bool>` | `false` | Zebra striping |
| `pin_rows` | `Signal<bool>` | `false` | Sticky header/footer rows |
| `pin_cols` | `Signal<bool>` | `false` | Sticky first column |
| `max_height` | `Option<String>` | `None` | Viewport-constrained scrolling, e.g. `"calc(100vh - 260px)"` |
| `selected_rows` | `Option<RwSignal<BTreeSet<usize>>>` | `None` | Multi-select state (absolute indices). Owned locally if omitted. Supplying it makes rows keyboard-operable |
| `selection_anchor` | `Option<RwSignal<Option<usize>>>` | `None` | Anchor for Shift-range selection |
| `on_row_activate` | `Option<Callback<usize>>` | `None` | Plain click/Enter/Space activates instead of selecting. Supplying it makes rows keyboard-operable |
| `on_sort_change` | `Option<Callback<(&'static str, SortOrder)>>` | `None` | Fired after a header click changes sort state |
| `cell_renderers` | `Vec<CellRenderer>` | `[]` | Custom cell views, indexed by `Column::renderer_index` |
| `typed_cells` | `Vec<TypedCellFn>` | `[]` | Lightweight Badge/Icon cells, indexed by `Column::typed_cell_index` |
| `row_class_fn` | `Option<Callback<(usize, TableRow), String>>` | `None` | Per-row extra classes from `(abs_idx, row)` |
| `node_ref` | `NodeRef<Div>` | — | Reference to the container `<div>` |

`TableRow` is `HashMap<&'static str, String>`.

## Column

```rust
Column::new("balance", "Balance")          // sortable
Column::new_non_sortable("actions", "")    // not sortable
```

| Builder | Effect |
|---------|--------|
| `.with_sort_as(SortAs)` | How cells compare when sorting (see below) |
| `.filterable()` | Give this column a dropdown in the filter row |
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

A `filterable()` column gets a dropdown of its distinct values beneath the header. Filters combine with each other **and** with the search box — all must match. A table with no `filterable` column renders no filter row at all.

Best on low-cardinality columns (status, owner, type); a dropdown of a thousand distinct ids is not a usable filter.

<details>
<summary>View Code</summary>

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn FilteredTable(data: Signal<Vec<TableRow>>) -> impl IntoView {
    let columns = vec![
        Column::new("name", "Name"),                      // no dropdown
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
| `row_range` | `"Showing {start}–{end} of {total}"` |
| `filter_all` | `"All"` |

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
| `current_page` | `Signal<i64>` | Current page index |
| `total_count` | `Signal<i64>` | Total rows across all pages |
| `page_size` | `Signal<i64>` | Rows per page |
| `on_page_change` | `Callback<i64>` | Fired when the user pages |
| `on_search` | `Option<Callback<String>>` | Debounced search box; the parent performs the query |
| `on_query_change` | `Option<Callback<TableQuery>>` | Reports the complete server query after paging, search, sort, or filter changes |
| `filter_options` | `Option<Signal<HashMap<&'static str, Vec<String>>>>` | Population-wide choices for filterable columns |
| `on_row_activate` | `Option<Callback<usize>>` | Plain click or keyboard activation with the current-page row index |
| `on_row_inspect` | `Option<Callback<usize>>` | Double-click or Shift+Enter inspection with the current-page row index |
| `loading`, `classes`, `texts`, `sort_texts`, `class`, `table_size`, `zebra`, `pin_rows`, `pin_cols`, `max_height`, `cell_renderers`, `typed_cells`, `row_class_fn`, `node_ref` | | As `DataTable` |

**Not available**: `selected_rows` / `selection_anchor` (the server variant
has no selection state machine), `auto_page_size`, `paginate`, `searchable`
(use `on_search`), `on_sort_change` (use `on_query_change`).

## Helper functions

All pure and independently usable:

| Function | Purpose |
|----------|---------|
| `compare_cells(a, b, sort_as, order)` | The comparator behind sorting |
| `parse_number(cell)` / `parse_date(cell)` | Cell parsers (`None` = unparseable) |
| `column_sort_as(columns, col_id)` | A column's declared `SortAs` |
| `distinct_values(data, col_id)` | A column's sorted, deduped, non-empty values |
| `row_matches_filters(row, filters)` | Whether a row satisfies active filters (AND) |
| `prune_stale_filters(filters, options)` | Drop selections whose value vanished |
| `has_filterable_columns(columns)` | Whether to render a filter row |
| `rows_per_page_for_height(viewport, header, row)` | The raw geometric fit used by `auto_page_size` |
| `auto_page_size_for_height(viewport, header, row, configured, min_rows)` | Responsive fit with the short-viewport fallback policy |
| `page_count(total, size)` / `clamp_page(...)` / `page_bounds(...)` | Shared zero-based client-pagination state used by both client table implementations |
| `handle_row_click(...)` / `row_click_kind(ctrl, shift, has_activate)` | Selection/activation state machine |
| `cell_text` / `row_text` / `row_with_headers_text` | Clipboard export (tab-separated) |

## Add to `input.css`

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
@source inline("select select-bordered select-xs w-full font-normal p-1");
```

## Accessibility

- Headers are `role="columnheader"` with `aria-sort` reflecting the current state (`ascending` / `descending` / `none`).
- Resize handles are `role="separator"` with `aria-orientation="vertical"` and an `aria-label` naming their column.
- The search box carries `aria-label="Search table"`; each filter dropdown is labelled `Filter by <column>`.
- Sort state changes are conveyed through `aria-sort` rather than the `▲`/`▼` glyph alone.
- **Keyboard operation.** When the table is interactive — `selected_rows` or `on_row_activate` supplied — each row is focusable (`tabindex=0`) and carries `aria-selected`. **Enter** and **Space** do exactly what a plain click does (activate, or select); **Ctrl/Cmd** and **Shift** with Enter/Space toggle and range-extend selection, mirroring the mouse. Space suppresses its default page-scroll. A plain display table (neither prop) adds no tab stops.

**Known gap**: rows are keyboard-operable but the table does not yet expose full `role="grid"` semantics (per-cell `role="gridcell"`, arrow-key roaming). Screen readers announce rows and their selected state, but not "row 3 of 20, column 2". For a spreadsheet-grade grid this is a larger, separate change.

## Best Practices

1. **Choose from data ownership first.** New typed snapshots use `EntityTable`; server slices use `ServerDataTable`; this dynamic client component is the compatibility path.
2. **Declare `SortAs::Number` on every money, percentage, duration or count column.** Text order puts `"$1,000"` before `"$900"`, and the bug looks like working software.
3. **Keep `filterable()` for low-cardinality columns.** Filter by status or owner, search by name or id.
4. **Pair `auto_page_size` with `max_height`** (or a definite-height parent). Tune `min_rows` only when five is inappropriate; `page_size` is the intentional scroll fallback below that threshold.
5. **Treat every index as absolute.** Don't add the page offset yourself — it's already applied.
6. **Re-check selection after mutating `data`.** It's cleared on data and sort changes by design.
7. **Prefer `typed_cells` over `cell_renderers`** for badges and icons; reach for a full renderer only when you need arbitrary views.
8. **Use `paginate=false` for small fixed tables**, not `page_size=1000`.
9. Add the classes above to `input.css` — Tailwind can't see class names built at runtime.
