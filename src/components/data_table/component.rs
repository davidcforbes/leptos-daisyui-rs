use crate::components::data_table::auto_page::{
    DEFAULT_AUTO_MIN_ROWS, FALLBACK_HEADER_HEIGHT, FALLBACK_ROW_HEIGHT, RowHeightEra,
    auto_page_size_for_height, max_row_height, overflow_check_floor, rows_per_page_for_height,
};
use crate::components::data_table::body::{DataTableBody, DataTableBodyClick, DataTableBodyRow};
use crate::components::data_table::chooser::{
    CHOOSER_STORAGE_PREFIX, DataTableColumnChooser, parse_hidden, serialize_hidden, visible_columns,
};
use crate::components::data_table::controls::DataTableControls;
use crate::components::data_table::filter::{
    ColumnFilters, DataTableFilterRow, distinct_values, filter_options_from_strings,
    has_filterable_columns, prune_stale_column_filters, row_matches_column_filters,
    row_matches_search,
};
use crate::components::data_table::geometry::{
    StableColumnTrack, StableTableColGroup, stable_column_width, stable_table_content_style,
};
use crate::components::data_table::header::DataTableHeader;
use crate::components::data_table::pagination::page_count;
use crate::components::data_table::selection::{
    RowClickKind, handle_row_click, index_of_key, remap_selection, row_click_kind,
    row_is_interactive, selection_keys,
};
use crate::components::data_table::sort::{column_sort_as, compare_cells};
use crate::components::data_table::types::{
    CellRenderer, Column, ColumnFilterKind, DataTableClasses, DataTableSortTexts, DataTableTexts,
    RowDetailRenderer, SortOrder, TableRow, TypedCellFn,
};
use crate::components::data_table::{TABLE_SCROLL_WRAPPER_CLASS, next_data_table_search_id};
use crate::components::table::{Table, TableSize};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use std::collections::{BTreeSet, HashMap, HashSet};
use web_sys::wasm_bindgen::JsCast;

/// Read a `localStorage` value, returning `None` in any non-browser context
/// (no `window`, storage disabled) rather than panicking.
fn local_storage_get(key: &str) -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(key)
        .ok()?
}

/// Write a `localStorage` value, silently no-op'ing when storage is
/// unavailable.
fn local_storage_set(key: &str, value: &str) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
        let _ = storage.set_item(key, value);
    }
}

/// # DataTable Component
///
/// A production-ready data table with sorting, pagination, loading states,
/// and efficient handling of large datasets (10,000+ rows).
///
/// ## Features
/// - Column-based sorting (click headers to toggle Asc/Desc), typed per column
///   via [`Column::with_sort_as`] -- see below
/// - Pagination with customizable page size, or `auto_page_size` to grow the
///   row count with the window -- see below
/// - Loading and empty states
/// - Fully themed with daisyUI
/// - Efficient index-based operations for large datasets
///
/// ## Which DataTable -- this one, or `widgets::DataTable`?
///
/// This is the full column-model table: reach for it when you need
/// `cell_renderers`, `typed_cells`, `row_class_fn`, `Column::action()` action
/// cells, `row_key`-based selection identity that survives a data
/// replacement, the column chooser, per-column filter dropdowns
/// (`Column::filterable`), `extra_filter` plus `toolbar` composition, or
/// `auto_page_size` responsive paging -- plus a server-driven variant for
/// page/size/search/sort/filter pushed to the backend.
///
/// The simpler `widgets::DataTable` takes plain `Vec<Vec<String>>` rows and
/// has none of that renderer/selection/filter surface, but it is the only
/// place for three things this component does not have: `badge_column_keys`
/// and `link_column_keys` (automatic badge/link styling resolved by column
/// key) and `bulk_select` (a leading checkbox column keyed by the first
/// cell's `String` id).
///
/// ## Sorting
///
/// Sorting reorders an index permutation, never the `data` itself, so row
/// identity (and therefore `selected_rows`) survives a sort by construction.
///
/// Cells are `String`s and compare as text by default. A column holding
/// formatted numbers must say so with [`SortAs::Number`], or it sorts by first
/// digit (`"$1,000"` before `"$900"`):
///
/// ```rust
/// use leptos_daisyui_rs::components::{Column, SortAs};
///
/// let columns = vec![
///     Column::new("account", "Account"),                             // text
///     Column::new("balance", "Balance").with_sort_as(SortAs::Number), // $85 < $900 < $1,000
///     Column::new("opened", "Opened").with_sort_as(SortAs::Date),
/// ];
/// ```
///
/// ## Responsive paging
///
/// By default `page_size` is fixed. Pass `auto_page_size=true` *together with*
/// `max_height` to derive the row count from the rendered height instead, so a
/// taller window shows more rows. If fewer than `min_rows` fit, the table keeps
/// the configured `page_size` (never below the minimum) and scrolls:
///
/// ```rust,no_run
/// # use leptos::prelude::*;
/// # use leptos_daisyui_rs::components::*;
/// # fn f(columns: Signal<Vec<Column>>, data: Signal<Vec<TableRow>>) -> impl IntoView {
/// view! {
///     <DataTable
///         columns=columns
///         data=data
///         auto_page_size=true
///         max_height="calc(100vh - 260px)"
///         min_rows=5
///     />
/// }
/// # }
/// ```
///
/// The table needs a definite height -- `max_height` (promoted to `height`
/// here) or a parent that fixes it. See the `auto_page_size` prop docs for why.
/// The complete `<thead>` is measured, so a filter row or wrapped labels count
/// toward the available height.
///
/// ## Example
/// ```rust,no_run
/// use std::collections::HashMap;
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::*;
///
/// #[component]
/// fn MyComponent() -> impl IntoView {
///     let columns = vec![
///         Column::new("name", "Name"),
///         Column::new("email", "Email"),
///         Column::new_non_sortable("status", "Status"),
///     ];
///
///     let data = vec![
///         HashMap::from([
///             ("name", "Alice".to_string()),
///             ("email", "alice@example.com".to_string()),
///             ("status", "Active".to_string()),
///         ]),
///     ];
///
///     view! {
///         <DataTable
///             columns=Signal::derive(move || columns.clone())
///             data=Signal::derive(move || data.clone())
///             page_size=10
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("table table-zebra table-pin-rows table-pin-cols table-xs table-sm table-md table-lg");
/// @source inline("btn btn-sm animate-pulse");
/// // Column-resize divider (header.rs)
/// @source inline("relative absolute top-0 right-0 z-10 h-full w-2 cursor-col-resize select-none");
/// @source inline("opacity-0 hover:opacity-100 hover:bg-primary/50 focus:opacity-100 focus:bg-primary/50 focus:outline focus:outline-2 focus:outline-primary active:opacity-100 active:bg-primary/70");
/// // Typed cells (Column::with_typed_cell -> TypedCell::Badge / TypedCell::Icon)
/// @source inline("badge badge-neutral badge-primary badge-secondary badge-accent badge-info badge-success badge-warning badge-error");
/// @source inline("inline-block w-4 h-4 w-5 h-5 w-6 h-6 w-8 h-8 w-12 h-12");
/// // Pagination: numbered page buttons (join) + row-range caption (controls.rs)
/// @source inline("flex justify-between items-center mt-4 gap-2");
/// @source inline("btn btn-sm join join-item btn-active btn-disabled");
/// @source inline("text-sm text-base-content/75");
/// // Per-column filter row (Column::filterable/filterable_text -> filter.rs)
/// @source inline("select select-bordered select-xs input input-bordered input-xs w-full font-normal p-1");
/// ```
///
/// ## Node References
/// - `node_ref` - References the container div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn DataTable(
    /// Table data (HashMap with column IDs as keys)
    #[prop(into)]
    data: Signal<Vec<TableRow>>,

    /// Column definitions
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Number of rows per page (default: 10). With `auto_page_size`, this is
    /// also the fallback when fewer than `min_rows` fit in the viewport.
    #[prop(optional, into)]
    page_size: Signal<usize>,

    /// Opt-in responsive paging: derive rows-per-page from the table's rendered
    /// height instead of the fixed `page_size`, so a taller window shows more
    /// rows (default: `false`, i.e. `page_size` is authoritative).
    ///
    /// A `ResizeObserver` on the table's scroll wrapper re-measures on every
    /// resize and feeds [`rows_per_page_for_height`] — the same arithmetic
    /// d2d-ui's desktop table uses.
    ///
    /// ## Requires a definite height
    ///
    /// The table must get its height from its layout context, not from its own
    /// rows -- otherwise the height being measured is a function of the row
    /// count being derived from it, and the count can never grow. Either:
    ///
    /// - pass `max_height`, which this prop promotes to a definite `height`
    ///   (a bare `max-height` is only a ceiling and would leave the table
    ///   shrink-wrapping its rows); or
    /// - give the table a parent with a definite height, which it then fills.
    ///
    /// The pager and search box are laid out as flex siblings of the scroll
    /// area, so they are excluded from the measurement automatically.
    #[prop(optional, into)]
    auto_page_size: Signal<bool>,

    /// Usability floor for `auto_page_size` (default: 5). When the measured
    /// fit is below this threshold, the configured `page_size` is retained
    /// (never below this floor) and the bounded table viewport scrolls.
    #[prop(into, default = Signal::derive(|| DEFAULT_AUTO_MIN_ROWS))]
    min_rows: Signal<usize>,

    /// Loading state
    #[prop(optional, into)]
    loading: Signal<bool>,

    /// Enable pagination (default: true). Pass `paginate=false` to hide the
    /// pagination controls entirely (e.g. a 1–2 row table that needs no pager).
    #[prop(into, default = Signal::derive(|| true))]
    paginate: Signal<bool>,

    /// Opt-in gear-icon column chooser (default `false`). When on, a gear
    /// dropdown appears in the toolbar letting the user toggle which columns
    /// are visible; the last visible column can't be hidden. Pair with
    /// `chooser_key` to persist the choice per table.
    #[prop(optional, into)]
    column_chooser: Signal<bool>,

    /// `localStorage` key that persists this table's hidden-column set across
    /// sessions (only meaningful with `column_chooser`). Ids that no longer
    /// exist in the column set are dropped on load, so a renamed column can't
    /// leave a phantom entry. When `None`, the choice is in-memory only.
    #[prop(optional)]
    chooser_key: Option<&'static str>,

    /// Custom CSS classes
    #[prop(optional)]
    classes: DataTableClasses,

    /// Custom text strings. A `Signal` so table chrome can be localized at
    /// runtime: derive the struct from your translation function inside a
    /// `Signal::derive` that reads the active locale, and every string
    /// re-renders on a language switch.
    #[prop(into, default = Signal::stored(DataTableTexts::default()))]
    texts: Signal<DataTableTexts>,

    /// Localized accessible-name templates for sortable header controls.
    /// The signal is read live, so current-state and next-action copy can
    /// relocalize without remounting or resetting table state.
    #[prop(into, default = Signal::stored(DataTableSortTexts::default()))]
    sort_texts: Signal<DataTableSortTexts>,

    /// Localized accessible-name template for substring column filters;
    /// `{column}` is replaced with the live header.
    #[prop(into, default = Signal::stored("Filter {column} by text".to_owned()))]
    text_filter_label: Signal<String>,

    /// Additional CSS classes for container
    #[prop(optional, into)]
    class: &'static str,

    /// Table size variant
    #[prop(optional, into)]
    table_size: Signal<TableSize>,

    /// Enable zebra striping
    #[prop(optional, into)]
    zebra: Signal<bool>,

    /// Pin header and footer rows
    #[prop(optional, into)]
    pin_rows: Signal<bool>,

    /// Pin first column
    #[prop(optional, into)]
    pin_cols: Signal<bool>,

    /// Maximum height for viewport-constrained scrolling (e.g. "calc(100vh - 260px)")
    #[prop(optional, into)]
    max_height: Option<String>,

    /// Enable client-side search filtering
    #[prop(optional, into)]
    searchable: Signal<bool>,

    /// Optional selected-row state. When provided, rows respond to
    /// Ctrl/Shift-click with multi-select semantics matching d2d-ui's
    /// desktop table. Indices are absolute (into the underlying `data`)
    /// so they survive pagination. Cleared automatically when `data` or
    /// the sort column/order changes.
    #[prop(optional)]
    selected_rows: Option<RwSignal<BTreeSet<usize>>>,

    /// Optional anchor index for Shift-range selection. Defaults to a
    /// locally-owned signal if not provided.
    #[prop(optional)]
    selection_anchor: Option<RwSignal<Option<usize>>>,

    /// Optional stable row identity. Without it, row identity is positional
    /// (the absolute index into `data`), which is fine for one immutable
    /// client window but loses selection continuity when the data vec is
    /// *replaced* -- a server page swap, or a live pool dropping rows (e.g.
    /// SSE hiding claimed items) -- because the same index now points at a
    /// different row.
    ///
    /// When set, selection state keys off the row's identity instead: the
    /// keys of the selected rows are captured at click time and remapped onto
    /// every new data vec, so a selected row stays selected wherever it moves,
    /// deselects (rather than sliding onto a neighbour) when it disappears,
    /// and re-selects if it returns. A sort no longer clears the selection
    /// either -- keyed identity makes the conservative clear unnecessary.
    #[prop(optional, into)]
    row_key: Option<Callback<TableRow, String>>,

    /// Optional callback fired on a **plain** row click (no Ctrl/Shift),
    /// receiving the row's absolute index (same index space as
    /// `selected_rows` -- survives pagination/sort). Opt-in: when unset,
    /// every click feeds the existing Ctrl/Shift selection semantics
    /// unchanged. When set, a plain click calls this instead of selecting
    /// (e.g. to navigate to a detail page); a modified click still selects.
    #[prop(optional, into)]
    on_row_activate: Option<Callback<usize>>,

    /// Optional secondary activation fired with the row's absolute index on a
    /// **double-click** (or Shift+Enter from the keyboard), so a list page can
    /// use plain click = drilldown navigation and double-click = raw record
    /// inspector (`ldui-tmr`). Standard dblclick discrimination: the first
    /// click of a double-click still runs `on_row_activate` once (single-click
    /// navigation is non-destructive), and the repeat click is swallowed so
    /// activation never fires twice. Opt-in: when unset, nothing changes.
    #[prop(optional, into)]
    on_row_inspect: Option<Callback<usize>>,

    /// Node reference to container element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Per-cell renderers indexed by `Column::renderer_index`. A column with
    /// `renderer_index = Some(i)` invokes `cell_renderers[i]` with
    /// `(abs_idx, row)` to produce its cell view; columns without an index
    /// render `row[col.id]` as text. Out-of-bounds indices fall back to text.
    #[prop(optional)]
    cell_renderers: Vec<CellRenderer>,

    /// Optional callback fired after a sortable header click changes the sort
    /// state. Receives the new `(column_id, order)` pair. Useful for syncing
    /// external state or exposing the (otherwise internal) sort state to a
    /// test oracle/debug bridge.
    #[prop(optional)]
    on_sort_change: Option<Callback<(&'static str, SortOrder)>>,

    /// Per-cell typed-cell resolvers indexed by `Column::typed_cell_index`,
    /// for lightweight `Badge`/`Icon` rendering without a full custom
    /// `CellRenderer`. Additive alongside `cell_renderers` -- a column's
    /// `renderer_index` (when set) always takes precedence.
    #[prop(optional)]
    typed_cells: Vec<TypedCellFn>,

    /// Optional full-width detail content rendered immediately after a row.
    #[prop(optional)]
    detail_renderer: Option<RowDetailRenderer>,

    /// Optional per-row extra CSS classes (e.g. a background tint) computed
    /// from the row's absolute index and data. Merged with `classes.row` /
    /// `classes.selected_row`.
    #[prop(optional)]
    row_class_fn: Option<Callback<(usize, TableRow), String>>,

    /// Optional controlled filter predicate, invoked with `(abs_idx, row)` and
    /// ANDed with the built-in per-column dropdowns and the `searchable` box.
    /// This is the home for *derived* domain filters the distinct-value
    /// dropdowns cannot express -- SLA buckets, new-arrival windows -- without
    /// rebuilding the table's toolbar.
    ///
    /// The predicate is controlled by the caller's own state: read your
    /// signals inside it and the table re-filters when they change (the
    /// filtering memo tracks them automatically). Pair with [`toolbar`] to
    /// put the driving UI inline with the built-in toolbar.
    ///
    /// [`toolbar`]: #structfield.toolbar
    #[prop(optional, into)]
    extra_filter: Option<Callback<(usize, TableRow), bool>>,

    /// Optional extra toolbar content, rendered in the table's toolbar row
    /// between the search box and the column chooser -- so caller-supplied
    /// filter controls (the UI driving [`extra_filter`]) compose with the
    /// built-in toolbar instead of replacing it. Supplying it forces the
    /// toolbar row to render even when `searchable`/`column_chooser` are off.
    ///
    /// [`extra_filter`]: #structfield.extra_filter
    #[prop(optional)]
    toolbar: Option<ViewFn>,
) -> impl IntoView {
    // Column-width overrides from dragging a header divider, keyed by
    // column id. Shared between the header (writer) and body (reader) so
    // resized columns stay aligned.
    let column_widths = RwSignal::new(HashMap::<&'static str, f64>::new());

    // Rows measured to fit the scroll wrapper, written by the `ResizeObserver`
    // below. `None` until the first measurement lands, so the first paint uses
    // the caller's `page_size` rather than flashing a guessed row count.
    let auto_rows = RwSignal::new(Option::<usize>::None);
    // The table's scroll wrapper -- the element whose height *is* the space
    // available to rows (the search box and pager are its flex siblings, not
    // its children, so they're already excluded).
    let table_wrapper_ref = NodeRef::<Div>::new();

    let configured_page_size = page_size;

    // Effective rows per page: the measured/fallback result when
    // `auto_page_size` is on and a measurement exists, else the configured
    // `page_size` prop (defaulting to 10).
    let page_size = Signal::derive(move || {
        if auto_page_size.get()
            && let Some(rows) = auto_rows.get()
        {
            return rows;
        }
        let size = configured_page_size.get();
        if size == 0 { 10 } else { size }
    });

    // `paginate` comes straight from the prop (default `true` — see its
    // declaration). The old `if p { p } else { true }` re-derive here forced
    // it ALWAYS true, silently ignoring an explicit `paginate=false`
    // (bd_4iiz-inventory-toe.6) — removed.

    // Pagination state
    let (current_page, set_current_page) = signal(0_usize);

    // Sorting state
    let (sort_column, set_sort_column) = signal(Option::<&'static str>::None);
    let (sort_order, set_sort_order) = signal(SortOrder::default());

    // Search state with debounce. Mirrors the debounce pattern in
    // `data_table::server_component::ServerDataTable` -- keep the two in
    // sync if either changes.
    let (search_query, set_search_query) = signal(String::new());
    let (debounced_search, set_debounced_search) = signal(String::new());
    let (debounce_handle, set_debounce_handle) = signal(Option::<TimeoutHandle>::None);

    let on_search_input = move |ev: leptos::ev::Event| {
        // The event always has an `HtmlInputElement` target in practice, but
        // if the browser ever hands us something that doesn't cast cleanly,
        // skip this keystroke rather than panicking the whole WASM app.
        let Some(target) = ev.target() else {
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            return;
        };
        let value = input.value();
        set_search_query.set(value.clone());

        // Clear previous timer, if any.
        if let Some(handle) = debounce_handle.get_untracked() {
            handle.clear();
        }

        // Set new 300ms debounce timer. If scheduling fails (no `window`,
        // e.g. outside a browser context), fall back to running the search
        // immediately instead of silently dropping the keystroke.
        let value_for_timeout = value.clone();
        match set_timeout_with_handle(
            move || {
                // try-forms: a debounce that outlives the table (navigate
                // within 300 ms of a keystroke) degrades to a no-op instead
                // of setting disposed signals (ldui-d54).
                let _ = set_debounced_search.try_set(value_for_timeout);
                let _ = set_current_page.try_set(0);
            },
            std::time::Duration::from_millis(300),
        ) {
            Ok(handle) => set_debounce_handle.set(Some(handle)),
            Err(_) => {
                set_debounced_search.set(value);
                set_current_page.set(0);
                set_debounce_handle.set(None);
            }
        }
    };
    // Cancel a pending debounce on unmount — its closure writes this owner's
    // signals (ldui-d54).
    on_cleanup(move || {
        if let Some(handle) = debounce_handle.try_get_untracked().flatten() {
            handle.clear();
        }
    });

    // ── Column chooser (opt-in via `column_chooser`) ──

    // Hidden column ids. When `chooser_key` is set, hydrated once from
    // localStorage at mount and persisted on every change thereafter.
    let hidden_columns = RwSignal::new(HashSet::<&'static str>::new());
    if let Some(key) = chooser_key {
        let storage_key = format!("{CHOOSER_STORAGE_PREFIX}{key}");
        // One-time synchronous hydrate. Only keep ids that still exist.
        if let Some(stored) = local_storage_get(&storage_key) {
            let valid: Vec<&'static str> = columns.get_untracked().iter().map(|c| c.id).collect();
            hidden_columns.set(parse_hidden(&stored, &valid));
        }
        // Persist on change (the first run rewrites the just-hydrated value —
        // idempotent, so no data loss regardless of effect ordering).
        Effect::new(move |_| {
            let serialized = hidden_columns.with(serialize_hidden);
            local_storage_set(&storage_key, &serialized);
        });
    }

    // Columns actually rendered: the full set, minus hidden ones when the
    // chooser is on. Sorting/filtering logic keeps using the full `columns`
    // (keyed by id), so hiding a column only drops it from the view.
    let display_columns = Signal::derive(move || {
        if column_chooser.get() {
            visible_columns(&columns.get(), &hidden_columns.get())
        } else {
            columns.get()
        }
    });
    let stable_tracks = Signal::derive(move || {
        let widths = column_widths.get();
        let columns = display_columns.get();
        let flexible_column = columns
            .iter()
            .rev()
            .find(|column| !column.resizable)
            .map(|column| column.id);
        columns
            .into_iter()
            .map(|column| {
                let track = StableColumnTrack::new(
                    column.id,
                    stable_column_width(widths.get(column.id).copied(), column.min_width),
                );
                if flexible_column == Some(column.id) {
                    track.flexible()
                } else {
                    track
                }
            })
            .collect::<Vec<_>>()
    });

    // ── Per-column filter row (exact dropdown or substring input) ──

    // Active dropdown selections, shared with `DataTableFilterRow`.
    let column_filters = RwSignal::new(ColumnFilters::new());
    // Whether to render the filter row at all — no *visible* `filterable`
    // column means no row, so callers that never opt in are untouched.
    let show_filter_row = Memo::new(move |_| display_columns.with(|c| has_filterable_columns(c)));

    // Option lists per filterable column, derived from the *unfiltered* data so
    // that choosing one option never removes the others from their dropdowns.
    let raw_filter_options = Memo::new(move |_| {
        let all_data = data.get();
        columns.with(|cols| {
            cols.iter()
                .filter(|column| column.filter_kind() == Some(ColumnFilterKind::Exact))
                .map(|c| (c.id, distinct_values(&all_data, c.id)))
                .collect::<HashMap<&'static str, Vec<String>>>()
        })
    });
    let filter_options = Memo::new(move |_| filter_options_from_strings(raw_filter_options.get()));

    // Drop selections whose value disappeared from the new data; a filter
    // pinned to a value that no longer exists silently matches zero rows.
    Effect::new(move |_| {
        let options = raw_filter_options.get();
        column_filters.update(|f| {
            if prune_stale_column_filters(f, &options, &columns.get_untracked()) {
                set_current_page.set(0);
            }
        });
    });

    // Reset to page 1 when data changes
    Effect::new(move |_| {
        let _ = data.get();
        set_current_page.set(0);
    });

    // Back to page 1 whenever a filter selection changes — the row the user was
    // looking at on page 5 probably isn't there any more.
    Effect::new(move |prev: Option<()>| {
        let _ = column_filters.get();
        if prev.is_some() {
            set_current_page.set(0);
        }
    });

    // Filtered indices — applies the per-column filters and, when `searchable`,
    // the free-text query. The two combine with AND: the search box narrows
    // what the dropdowns already selected.
    let filtered_indices = Memo::new(move |_| {
        let all_data = data.get();
        let query = debounced_search.get();
        let filters = column_filters.get();
        let q = query.to_lowercase();
        // Search is column-scoped (`row_matches_search`): renderer-only
        // metadata in a TableRow — state codes, route ids, epoch instants —
        // must never match what a user types.
        let search_columns = columns.get();

        (0..all_data.len())
            .filter(|&i| {
                let Some(row) = all_data.get(i) else {
                    return false;
                };
                if !filters.is_empty()
                    && !row_matches_column_filters(row, &search_columns, &filters)
                {
                    return false;
                }
                // Caller-controlled predicate, ANDed with the built-ins.
                // Signals the predicate reads are tracked by this memo, so
                // the caller's own filter state re-filters reactively.
                if let Some(f) = extra_filter
                    && !f.run((i, row.clone()))
                {
                    return false;
                }
                row_matches_search(row, &search_columns, &q)
            })
            .collect::<Vec<usize>>()
    });

    // Back to page 1 when the caller's controlled `extra_filter` changes the
    // visible set, mirroring the built-in dropdowns. Scoped to tables that
    // supply the predicate so every other table keeps today's exact paging
    // behaviour.
    if extra_filter.is_some() {
        Effect::new(move |prev: Option<()>| {
            let _ = filtered_indices.get();
            if prev.is_some() {
                set_current_page.set(0);
            }
        });
    }

    // Sorted indices. Sorting an index permutation (never the data) is what
    // keeps row identity -- and therefore `selected_rows` -- intact.
    let sorted_indices = Memo::new(move |_| {
        let mut indices = filtered_indices.get();
        if let Some(col_id) = sort_column.get() {
            let data_vec = data.get();
            let order = sort_order.get();
            // Columns declare how their cells compare; the default is Text,
            // which is the plain lexicographic order. Money and duration
            // columns opt into `SortAs::Number` so "$900" does not outrank
            // "$1,000" on its first digit.
            let sort_as = column_sort_as(&columns.get(), col_id);
            indices.sort_by(|&a, &b| {
                let cell = |i: usize| {
                    data_vec
                        .get(i)
                        .and_then(|row| row.get(col_id))
                        .map(|s| s.as_str())
                        .unwrap_or("")
                };
                compare_cells(cell(a), cell(b), sort_as, order)
            });
        }
        indices
    });

    // Total pages calculation with safety guards
    let total_pages = Memo::new(move |_| {
        let total_items = sorted_indices.get().len();
        page_count(total_items, page_size.get()).max(1)
    });

    // Current page rows paired with absolute indices into `data`, with safety guards
    let current_page_rows = Memo::new(move |_| {
        if !paginate.get() {
            // No pagination: return all rows
            return sorted_indices
                .get()
                .iter()
                .filter_map(|&idx| data.get().get(idx).cloned().map(|row| (idx, row)))
                .collect::<Vec<_>>();
        }

        let safe_page = current_page.get().min(total_pages.get().saturating_sub(1));
        let start = safe_page * page_size.get();
        let end = ((safe_page + 1) * page_size.get()).min(sorted_indices.get().len());

        sorted_indices.get()[start..end]
            .iter()
            .filter_map(|&idx| data.get().get(idx).cloned().map(|row| (idx, row)))
            .collect::<Vec<_>>()
    });

    // Sort callback
    let on_sort = Callback::new(move |col_id: &'static str| {
        let new_order = if sort_column.get() == Some(col_id) {
            // Same column: toggle order
            let toggled = sort_order.get().toggle();
            set_sort_order.set(toggled);
            toggled
        } else {
            // New column: set to Asc
            set_sort_column.set(Some(col_id));
            set_sort_order.set(SortOrder::Asc);
            SortOrder::Asc
        };
        if let Some(cb) = on_sort_change {
            cb.run((col_id, new_order));
        }
    });

    // A row is keyboard-operable only when the consumer opted into interaction,
    // so plain display tables don't sprout a tab stop per row. Captured before
    // `selected_rows` is unwrapped into a local signal below, which would erase
    // whether the consumer supplied one. `row_has_selection` is captured
    // alongside it for the same reason -- it gates `aria-selected`
    // independently of `row_interactive` (ldui-cyhz): an activate-only table
    // is interactive but has no selection concept to report.
    // `on_row_inspect` counts as an activation callback here: an inspect-only
    // table still needs focusable rows for its Shift+Enter equivalent.
    let row_has_selection = selected_rows.is_some();
    let row_interactive = row_is_interactive(
        row_has_selection,
        on_row_activate.is_some() || on_row_inspect.is_some(),
    );

    // Selection state — owned locally if the consumer didn't pass their own.
    let selected_rows = selected_rows.unwrap_or_else(|| RwSignal::new(BTreeSet::new()));
    let selection_anchor = selection_anchor.unwrap_or_else(|| RwSignal::new(None));

    // Keyed row identity (`row_key`): the source of truth for what is
    // selected is a set of row *keys*, captured at click time and remapped
    // onto every new data vec below. `anchor_key` carries the Shift-range
    // anchor across replacements the same way.
    let selected_keys = RwSignal::new(BTreeSet::<String>::new());
    let anchor_key = RwSignal::new(Option::<String>::None);

    // On data replacement: positional identity can only clear (the same index
    // now points at a different row -- and so might a sort). Keyed identity
    // instead remaps the stored keys onto the new rows, so selection follows
    // the row wherever it moves and a sort clears nothing.
    Effect::new(move |prev: Option<()>| {
        if let Some(key_of) = row_key {
            let d = data.get();
            if prev.is_none() {
                // Adopt a consumer-pre-seeded selection as keys.
                let seeded = selected_rows.get_untracked();
                if !seeded.is_empty() {
                    selected_keys.set(selection_keys(&d, &seeded, |r| key_of.run(r.clone())));
                }
                return;
            }
            let keys = selected_keys.get_untracked();
            selected_rows.set(remap_selection(&d, &keys, |r| key_of.run(r.clone())));
            selection_anchor.set(
                anchor_key
                    .get_untracked()
                    .and_then(|k| index_of_key(&d, &k, |r| key_of.run(r.clone()))),
            );
        } else {
            let _ = data.get();
            let _ = sort_column.get();
            let _ = sort_order.get();
            if prev.is_some() {
                selected_rows.update(|s| s.clear());
                selection_anchor.set(None);
            }
        }
    });

    // Row-interaction callback, driven by a mouse click or a keyboard
    // Enter/Space (modifiers passed as bools, not an event). A plain
    // interaction activates when the consumer opted in via `on_row_activate`; a
    // modified one always feeds the existing Ctrl/Shift multi-select semantics.
    let on_row_click = Callback::new(move |event: DataTableBodyClick| {
        let abs_idx = event.row.index;
        let ctrl = event.ctrl;
        let shift = event.shift;
        match row_click_kind(ctrl, shift, on_row_activate.is_some()) {
            RowClickKind::Activate => {
                if let Some(cb) = on_row_activate {
                    cb.run(abs_idx);
                }
            }
            RowClickKind::Select => {
                let total = data.with(|d| d.len());
                let mut next = selected_rows.get_untracked();
                let mut anchor = selection_anchor.get_untracked();
                handle_row_click(abs_idx, ctrl, shift, &mut next, &mut anchor, total);
                // Keyed identity: capture the keys of what just got selected,
                // so the selection can be remapped when `data` is replaced.
                if let Some(key_of) = row_key {
                    data.with_untracked(|d| {
                        selected_keys.set(selection_keys(d, &next, |r| key_of.run(r.clone())));
                        anchor_key
                            .set(anchor.and_then(|i| d.get(i)).map(|r| key_of.run(r.clone())));
                    });
                }
                selected_rows.set(next);
                selection_anchor.set(anchor);
            }
        }
    });
    let body_on_row_inspect = on_row_inspect
        .map(|callback| Callback::new(move |row: DataTableBodyRow| callback.run(row.index)));

    // ── Responsive paging (`auto_page_size`) ──
    //
    // Measure the scroll wrapper and the *rendered* header/row heights, rather
    // than assuming d2d-ui's fixed pixel constants: on the web a row's height
    // depends on the daisyUI table size, theme and cell content. The constants
    // are only fallbacks for when there is nothing to measure (empty table, or
    // the first paint before rows exist).
    // The pending belt-and-braces overflow-check timer (ldui-89rp), kept so
    // unmount can cancel it -- same rationale and pattern as `measure_handle`
    // below and the search debounce's handle (ldui-d54).
    let overflow_handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);

    // Row-height "era" identity (ldui-89rp CRITICAL fix): `page_size` derives
    // from `auto_rows`, `current_page_rows` slices by `page_size`, and the
    // Effect below re-measures whenever `page_size` changes -- so the
    // measured row set is a function of the *previous* pass's derived count.
    // Undamped, that loop can fail to converge (a short render derives a
    // large count; the larger render reveals a tall row and derives a small
    // one; the smaller render excludes the tall row again and derives the
    // large one again -- forever, since successive measured maxes genuinely
    // differ and the write-only-on-change guard below never sees a repeat).
    // `RowHeightEra` ratchets the row height used for derivation so it can
    // only grow within one era (same dataset, same container width),
    // guaranteeing the derived count reaches a fixed point. `data_revision`
    // bumps -- and therefore starts a fresh era -- only on a genuine `data`
    // change, never on the `page_size`/`auto_rows` churn a measurement pass
    // itself causes.
    let data_revision: StoredValue<u64> = StoredValue::new(0);
    Effect::new(move |ran_before: Option<()>| {
        let _ = data.get();
        if ran_before.is_some() {
            data_revision.update_value(|revision| *revision = revision.wrapping_add(1));
        }
    });
    let row_era: StoredValue<Option<RowHeightEra>> = StoredValue::new(None);

    let measure_rows = move || {
        // Late-firing guard (ldui-d54): this runs from a zero-delay macrotask,
        // so a router navigation scheduled-then-navigated in one task disposes
        // this table's reactive owner before the timer fires. The try-read
        // doubles as the auto_page_size check — on a disposed owner it yields
        // None and the whole measurement degrades to a no-op instead of
        // panicking the entire wasm app.
        if !auto_page_size.try_get_untracked().unwrap_or(false) {
            return;
        }
        let Some(wrapper) = table_wrapper_ref.get_untracked() else {
            return;
        };

        let measure = |selector: &str, fallback: f64| -> f64 {
            wrapper
                .query_selector(selector)
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                .map(|el| el.offset_height() as f64)
                .filter(|h| *h > 0.0)
                .unwrap_or(fallback)
        };

        // `offset_height`, deliberately, not `client_height`: the wrapper is an
        // `overflow-y: auto` box, so a *horizontal* scrollbar -- which appears
        // when the widest rendered cell makes the table wider than the wrapper
        // -- is subtracted from `client_height`. That made the viewport a
        // function of which rows were on screen, and therefore of the row count
        // being derived from it. The two then chase each other with no fixed
        // point: a container of 124px oscillated forever between 5 rows @ 37px
        // and 1 row @ 52px (the 15px delta being exactly the scrollbar).
        // `offset_height` is the wrapper's flex-allocated border-box height,
        // which no scrollbar or cell content can move, so the measurement
        // depends only on the layout context and the fixed point is unique.
        let viewport = wrapper.offset_height() as f64;
        let header_height = measure("thead", FALLBACK_HEADER_HEIGHT);

        // The MAX across every currently rendered `<tbody> <tr>`, not just the
        // first (ldui-89rp): with variable-height rows -- a wrapped cell, a
        // multi-line badge stack -- a short first row derives a page size
        // that fits the short row and overflows a taller one further down
        // the page. `max_row_height` is the pure, unit-tested reduction; this
        // is only the DOM-side gathering of what to feed it. `0.0` (not
        // `FALLBACK_ROW_HEIGHT`) is the "nothing measured this pass" sentinel
        // fed to the era ratchet below, which applies its own fallback.
        let measured_max = wrapper
            .query_selector_all("tbody tr")
            .map(|rows| {
                let heights: Vec<f64> = (0..rows.length())
                    .filter_map(|i| rows.item(i))
                    .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                    .map(|el| el.offset_height() as f64)
                    .collect();
                max_row_height(&heights, 0.0)
            })
            .unwrap_or(0.0);

        // Ratchet through the era (ldui-89rp CRITICAL fix -- see the comment
        // above `row_era`'s declaration): the row height actually fed to the
        // derivation below is the high-water mark of every measurement this
        // pass and every prior pass in the same era, never just this pass's
        // raw reading. `offset_width`, like `offset_height` above, is the
        // wrapper's own border-box width, unaffected by its own scrollbar.
        // `table_size` is part of the key too (ldui-wgc3): a density change
        // moves the row-height ceiling itself, so a high-water mark measured
        // under a taller density must not survive into a shorter one.
        let era_key = (
            data_revision.get_value(),
            wrapper.offset_width(),
            table_size.try_get_untracked().unwrap_or_default().as_str(),
        );
        let era = row_era
            .get_value()
            .unwrap_or(RowHeightEra::empty(era_key))
            .observe(era_key, measured_max);
        row_era.set_value(Some(era));
        let row_height = era.effective_row_height(FALLBACK_ROW_HEIGHT);

        let configured_page_size = configured_page_size
            .try_get_untracked()
            .filter(|size| *size > 0)
            .unwrap_or(10);
        let min_rows = min_rows
            .try_get_untracked()
            .unwrap_or(DEFAULT_AUTO_MIN_ROWS);
        let rows = auto_page_size_for_height(
            viewport,
            header_height,
            row_height,
            configured_page_size,
            min_rows,
        );
        // Write only on a real change. This is what ends the settle pass below:
        // with `viewport` independent of the row count, the second measurement
        // agrees with the first, writes nothing, and nothing re-renders.
        // try-forms, not plain get/set (ldui-d54 belt-and-braces): a straggler
        // that slipped past the entry guard still must not panic. A disposed
        // try_get_untracked returns None, which never equals Some(Some(rows)),
        // and the try_set below is then a no-op.
        if auto_rows.try_get_untracked() != Some(Some(rows)) {
            let _ = auto_rows.try_set(Some(rows));
        }

        // Belt-and-braces (ldui-89rp): even the max of the currently
        // rendered rows can miss growth that only appears once the derived
        // count actually renders (e.g. a taller row that wasn't part of the
        // previous page's set). Check once more, on the next frame, whether
        // the wrapper still overflows its own allocated box; if so, drop the
        // count by exactly one row and stop -- deliberately no loop, so a
        // table that genuinely cannot fit still shows *a* page rather than
        // shrinking toward nothing.
        if let Some(handle) = overflow_handle.try_get_value().flatten() {
            handle.clear();
        }
        let tolerance = row_height;
        // See `overflow_check_floor`'s own docs (ldui-89rp regression, caught
        // by `auto_page_size_keeps_a_usable_page_and_scrolls_short_viewports`):
        // below `min_rows`, `rows` above is the retained configured page
        // size, not a responsive fit, and the floor must be `rows` itself so
        // this belt-and-braces check can never shave a row off it.
        let measured_rows = rows_per_page_for_height(viewport, header_height, row_height);
        let floor = overflow_check_floor(measured_rows, min_rows, rows);
        let check_overflow = move || {
            // Same late-firing guard as `measure_rows` above: the
            // `auto_page_size` try-read doubles as the disposed-owner check,
            // so `table_wrapper_ref`/`auto_rows` are safe to read/write
            // (still via try-forms, belt-and-braces) past this point.
            if !auto_page_size.try_get_untracked().unwrap_or(false) {
                return;
            }
            let Some(wrapper) = table_wrapper_ref.get_untracked() else {
                return;
            };
            let scroll_height = wrapper.scroll_height() as f64;
            let offset_height = wrapper.offset_height() as f64;
            if scroll_height > offset_height + tolerance
                && let Some(Some(current)) = auto_rows.try_get_untracked()
                && current > floor
            {
                let _ = auto_rows.try_set(Some(current - 1));
            }
        };
        match set_timeout_with_handle(check_overflow, std::time::Duration::ZERO) {
            Ok(handle) => {
                overflow_handle.try_update_value(|slot| *slot = Some(handle));
            }
            // No `window` to schedule against: checking now is better than
            // not at all.
            Err(_) => check_overflow(),
        }
    };

    // Measure on a fresh macrotask rather than inline: a `ResizeObserver`
    // callback can run before the surrounding layout has settled, and latching
    // a mid-reflow height leaves the table a row short with no further resize
    // to correct it.
    // The pending measure timer, kept so unmount can cancel it (ldui-d54).
    // The search debounce a few hundred lines up stores its handle the same
    // way — keep the two consistent.
    let measure_handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let schedule_measure = move || {
        // One pending measure at a time: cancel a not-yet-fired timer so the
        // Effect + ResizeObserver double-schedule collapses to one macrotask
        // and the stored handle is always the live one.
        if let Some(handle) = measure_handle.try_get_value().flatten() {
            handle.clear();
        }
        match set_timeout_with_handle(measure_rows, std::time::Duration::ZERO) {
            Ok(handle) => {
                measure_handle.try_update_value(|slot| *slot = Some(handle));
            }
            // No `window` to schedule against (non-browser context): measuring
            // now is better than not at all.
            Err(_) => measure_rows(),
        }
    };
    // A zero-delay macrotask must not outlive the reactive owner: without
    // this, any navigation immediately after mount or a data change fired
    // measure_rows into disposed signals and panicked the whole wasm app
    // (ldui-d54; caught by 4iiz-etl's visual gate on a History->Errors
    // transition, which had to pace navigations 150 ms apart to dodge it).
    on_cleanup(move || {
        if let Some(handle) = measure_handle.try_get_value().flatten() {
            handle.clear();
        }
        // Same rationale, for the belt-and-braces overflow-check timer
        // (ldui-89rp): it must not outlive the reactive owner either.
        if let Some(handle) = overflow_handle.try_get_value().flatten() {
            handle.clear();
        }
    });

    // Re-measure when anything that moves the arithmetic changes: the opt-in
    // itself, the usability/configured fallbacks, row height (table size /
    // density), and the rows available to measure. Reading the effective
    // `page_size` also re-measures once after the count changes, which corrects
    // a height latched from an unsettled layout.
    Effect::new(move |_| {
        let _ = auto_page_size.get();
        let _ = min_rows.get();
        let _ = configured_page_size.get();
        let _ = table_size.get();
        let _ = data.get();
        let _ = page_size.get();
        schedule_measure();
    });

    // Attach the `ResizeObserver` once, when the wrapper first enters the DOM
    // (CSR-only -- a browser is always present). Reading `table_wrapper_ref.get()`
    // is what makes this effect re-run the one time it flips `None` -> `Some`;
    // it never flips back, so the setup branch never runs twice.
    //
    // No-feedback-loop precondition: the measurement only *reads* the wrapper's
    // height and changes how many rows are inside it. That is safe exactly when
    // the wrapper's height comes from its layout context rather than its own
    // content -- which is why `container_style` gives the container a definite
    // `height` (not just `max-height`) whenever `auto_page_size` is on.
    Effect::new(move |_| {
        let Some(wrapper) = table_wrapper_ref.get() else {
            return;
        };

        schedule_measure();

        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
            move |_entries: js_sys::Array, _observer: web_sys::ResizeObserver| {
                // A real resize: start a fresh settle episode, so revisiting a
                // height the loop saw earlier is not mistaken for a cycle.
                schedule_measure();
            },
        )
            as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);

        match web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) {
            Ok(observer) => {
                observer.observe(wrapper.unchecked_ref::<web_sys::Element>());
                // `Closure`/`ResizeObserver` wrap JS values and are neither
                // `Send` nor `Sync`, but `on_cleanup` requires both (the
                // reactive graph is generic over native multithreaded use).
                // This component only ever runs single-threaded (wasm32 in the
                // browser); `SendWrapper` encodes that assumption explicitly
                // rather than working around it silently. Same rationale as
                // `toolbar::Toolbar`.
                let guard = send_wrapper::SendWrapper::new((closure, observer));
                on_cleanup(move || {
                    let (closure, observer) = guard.take();
                    observer.disconnect();
                    drop(closure);
                });
            }
            Err(_) => drop(closure),
        }
    });

    let container_class = merge_classes!(classes.container, class);
    let filter_all_label = Signal::derive(move || texts.with(|t| t.filter_all.clone()));

    // Container style for viewport-constrained scrolling.
    //
    // `auto_page_size` needs a *definite* height here, not just a ceiling.
    // `max-height` alone leaves the flex column shrink-wrapping its rows, so
    // the wrapper's measured height would be a function of the row count we are
    // deriving from it: the count can only ever stay where it is (it settles at
    // 1 row and never grows). Promoting `max_height` to `height` breaks that
    // circularity -- the wrapper's height then comes from the layout context,
    // independent of how many rows are inside it.
    //
    // With no `max_height` to promote, fall back to `height: 100%` so the table
    // fills whatever definite height its parent provides.
    let has_max_height = max_height.is_some();
    let container_style = move || match (auto_page_size.get(), max_height.as_deref()) {
        (true, Some(h)) => Some(format!(
            "display: flex; flex-direction: column; height: {h}; max-height: {h}"
        )),
        (true, None) => Some("display: flex; flex-direction: column; height: 100%".to_string()),
        (false, Some(h)) => Some(format!(
            "display: flex; flex-direction: column; max-height: {h}"
        )),
        (false, None) => None,
    };

    // The wrapper is the scroll viewport whenever the container is a flex
    // column -- i.e. when `max_height` is set, or `auto_page_size` made the
    // container definite-height on its own.
    let is_flex_column = move || has_max_height || auto_page_size.get();
    let table_wrapper_style =
        move || is_flex_column().then_some("flex: 1; overflow-y: auto; min-height: 0");
    let controls_style = move || is_flex_column().then_some("flex-shrink: 0; padding: 12px 0");
    let search_input_id = next_data_table_search_id();

    view! {
        <div
            class=container_class
            node_ref=node_ref
            style=container_style
            data-table-data-mode="compatibility-client"
        >
            {move || {
                let show_search = searchable.get();
                let show_chooser = column_chooser.get();
                let extra_toolbar = toolbar.clone();
                let search_input_id = search_input_id.clone();
                (show_search || show_chooser || extra_toolbar.is_some()).then(|| view! {
                    <div class="mb-3 flex items-center gap-2">
                        {show_search.then(|| {
                            let label_target = search_input_id.clone();
                            let control_id = search_input_id.clone();
                            view! {
                            <label class="sr-only" r#for=label_target>
                                {move || texts.with(|t| t.search_label.clone())}
                            </label>
                            <input
                                id=control_id
                                type="text"
                                class="input input-bordered input-sm w-full max-w-xs"
                                placeholder=move || texts.with(|t| t.search_placeholder.clone())
                                aria-label=move || texts.with(|t| t.search_label.clone())
                                prop:value=move || search_query.get()
                                on:input=on_search_input
                            />
                            }
                        })}
                        {extra_toolbar.map(|t| t.run())}
                        <div class="flex-1"></div>
                        {show_chooser.then(|| view! {
                            <DataTableColumnChooser columns=columns hidden=hidden_columns />
                        })}
                    </div>
                })
            }}

            <div
                class=TABLE_SCROLL_WRAPPER_CLASS
                style=table_wrapper_style
                node_ref=table_wrapper_ref
            >
                <div style=move || stable_table_content_style(&stable_tracks.get())>
                    <Table
                        size=table_size
                        zebra=zebra
                        pin_rows=pin_rows
                        pin_cols=pin_cols
                        class="table-fixed w-full border-collapse border border-table-grid"
                        attr:data-table-layout="stable"
                    >
                        <StableTableColGroup tracks=stable_tracks />
                        <DataTableHeader
                            columns=display_columns
                            sort_column=Signal::derive(move || sort_column.get())
                            sort_order=Signal::derive(move || sort_order.get())
                            sort_texts=sort_texts
                            on_sort=on_sort
                            header_cell_class=classes.header_cell
                            column_widths=column_widths
                        >
                            {move || {
                                show_filter_row.get().then(|| view! {
                                    <DataTableFilterRow
                                        columns=display_columns
                                        options=Signal::derive(move || filter_options.get())
                                        filters=column_filters
                                        all_label=filter_all_label
                                        filter_label=Signal::derive(move || {
                                            texts.with(|texts| texts.filter_label.clone())
                                        })
                                        text_filter_label=text_filter_label
                                    />
                                })
                            }}
                        </DataTableHeader>
                        <DataTableBody
                            columns=display_columns
                            rows=Signal::derive(move || current_page_rows.get())
                            loading=loading
                            texts=texts
                            body_cell_class=classes.body_cell
                            row_class=classes.row
                            selected_row_class=classes.selected_row
                            selected_rows=Signal::derive(move || selected_rows.get())
                            loading_row_class=classes.loading_row
                            empty_row_class=classes.empty_row
                            on_row_click=Some(on_row_click)
                            on_row_inspect=body_on_row_inspect
                            row_key=row_key
                            interactive=row_interactive
                            has_selection=row_has_selection
                            cell_renderers=cell_renderers
                            column_widths=Signal::derive(move || column_widths.get())
                            typed_cells=typed_cells
                            detail_renderer=detail_renderer
                            row_class_fn=row_class_fn
                        />
                    </Table>
                </div>
            </div>

            {move || {
                if paginate.get() && !loading.get() && !data.get().is_empty() {
                    Some(view! {
                        <div style=controls_style>
                            <DataTableControls
                                current_page=Signal::derive(move || current_page.get())
                                set_current_page=set_current_page
                                total_pages=Signal::derive(move || total_pages.get())
                                total_items=Signal::derive(move || sorted_indices.get().len())
                                page_size=page_size
                                texts=texts
                                pagination_class=classes.pagination
                                button_class=classes.pagination_button
                                page_button_class=classes.pagination_page_button
                                active_page_button_class=classes.pagination_active_page_button
                                range_class=classes.row_range
                            />
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}
