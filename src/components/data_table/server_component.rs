use crate::components::data_table::TABLE_SCROLL_WRAPPER_CLASS;
use crate::components::data_table::body::DataTableBody;
use crate::components::data_table::filter::{
    ColumnFilters, DataTableFilterRow, distinct_values, has_filterable_columns,
};
use crate::components::data_table::header::DataTableHeader;
use crate::components::data_table::selection::{RowClickKind, row_click_kind, row_is_interactive};
use crate::components::data_table::types::{
    CellRenderer, Column, DataTableClasses, DataTableTexts, SortOrder, TableRow, TypedCellFn,
};
use crate::components::table::{Table, TableSize};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use std::collections::HashMap;
use web_sys::wasm_bindgen::JsCast;

/// The full query a server-owned table is currently displaying: everything a
/// backend needs to produce the matching page. Emitted through
/// [`ServerDataTable`]'s `on_query_change` on every user change, so the query
/// round-trips instead of the server seeing only page numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct TableQuery {
    /// 1-based page number, matching `ServerDataTable`'s `current_page`.
    pub page: i64,
    /// Rows per page.
    pub page_size: i64,
    /// Debounced free-text search ("" when the box is empty or absent).
    pub search: String,
    /// Active sort as `(column id, order)`; `None` means the server's
    /// default order.
    pub sort: Option<(&'static str, SortOrder)>,
    /// Active per-column filter selections (column id -> exact value), from
    /// the filter row's dropdowns ([`Column::filterable`]).
    pub filters: ColumnFilters,
}

/// # ServerDataTable Component
///
/// A server-side paginated data table where the parent component controls
/// data fetching. Unlike `DataTable`, this component does NOT perform
/// client-side sorting, filtering, or pagination. The `rows` prop contains
/// pre-fetched data for the current page, and `on_page_change` is called
/// when the user navigates to a different page.
///
/// ## Example
/// ```rust,no_run
/// use std::collections::HashMap;
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::*;
///
/// #[component]
/// fn MyServerTable() -> impl IntoView {
///     let columns = vec![
///         Column::new("name", "Name"),
///         Column::new("email", "Email"),
///     ];
///
///     let (rows, set_rows) = signal(vec![]);
///     let (current_page, set_current_page) = signal(1_i64);
///     let (total_count, _) = signal(100_i64);
///     let (page_size, _) = signal(10_i64);
///     let (loading, set_loading) = signal(false);
///
///     let on_page_change = Callback::new(move |page: i64| {
///         set_current_page.set(page);
///         // Fetch data from server for the new page...
///     });
///
///     view! {
///         <ServerDataTable
///             rows=Signal::derive(move || rows.get())
///             columns=Signal::derive(move || columns.clone())
///             current_page=Signal::derive(move || current_page.get())
///             total_count=Signal::derive(move || total_count.get())
///             page_size=Signal::derive(move || page_size.get())
///             on_page_change=on_page_change
///             loading=Signal::derive(move || loading.get())
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("table table-zebra table-pin-rows table-pin-cols table-xs table-sm table-md table-lg");
/// @source inline("btn btn-sm btn-ghost btn-active animate-pulse join join-item");
/// // Column-resize divider (header.rs)
/// @source inline("relative absolute top-0 right-0 z-10 h-full w-2 cursor-col-resize select-none");
/// @source inline("opacity-0 hover:opacity-100 hover:bg-primary/50 active:opacity-100 active:bg-primary/70");
/// // Typed cells (Column::with_typed_cell -> TypedCell::Badge / TypedCell::Icon)
/// @source inline("badge badge-neutral badge-primary badge-secondary badge-accent badge-info badge-success badge-warning badge-error");
/// @source inline("inline-block w-4 h-4 w-5 h-5 w-6 h-6 w-8 h-8 w-12 h-12");
/// ```
///
/// ## Node References
/// - `node_ref` - References the container div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ServerDataTable(
    /// Pre-fetched rows for the current page (from server API)
    #[prop(into)]
    rows: Signal<Vec<TableRow>>,

    /// Column definitions
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Current page number (1-based)
    #[prop(into)]
    current_page: Signal<i64>,

    /// Total number of items across all pages
    #[prop(into)]
    total_count: Signal<i64>,

    /// Items per page
    #[prop(into)]
    page_size: Signal<i64>,

    /// Callback when user clicks a page
    #[prop(into)]
    on_page_change: Callback<i64>,

    /// Loading state
    #[prop(optional, into)]
    loading: Signal<bool>,

    /// Custom CSS classes
    #[prop(optional)]
    classes: DataTableClasses,

    /// Custom text strings. A `Signal` so table chrome can be localized at
    /// runtime -- see [`DataTableTexts`].
    #[prop(into, default = Signal::stored(DataTableTexts::default()))]
    texts: Signal<DataTableTexts>,

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

    /// Callback for server-side search (fires after 300ms debounce)
    #[prop(optional, into)]
    on_search: Option<Callback<String>>,

    /// Typed query/change API: fired with the full [`TableQuery`] (page, page
    /// size, search, sort, filters) on **every** user change -- page
    /// navigation, debounced search, a sort toggle on a sortable header, or a
    /// filter-row selection. Query-shape changes (search/sort/filters) report
    /// `page: 1`, since the old page number is meaningless against a new
    /// result set.
    ///
    /// This is the server-owned contract: the table renders whatever `rows`
    /// holds and never sorts or filters client-side; it only *reports* the
    /// query for the caller to re-fetch with. Supplying this callback is also
    /// what arms header sorting (without it a header click has nowhere to
    /// go, and sortable headers stay inert as before).
    #[prop(optional, into)]
    on_query_change: Option<Callback<TableQuery>>,

    /// Option lists for [`Column::filterable`] columns, keyed by column id --
    /// supply the *population-wide* distinct values from the server. When
    /// absent, the dropdowns fall back to the distinct values of the current
    /// page, which silently understates a paged population (the LIMIT-20
    /// problem) -- fine for a demo, wrong for production.
    #[prop(optional, into)]
    filter_options: Option<Signal<HashMap<&'static str, Vec<String>>>>,

    /// Node reference to container element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Per-cell renderers indexed by `Column::renderer_index`. A column with
    /// `renderer_index = Some(i)` invokes `cell_renderers[i]` with
    /// `(abs_idx, row)` to produce its cell view; columns without an index
    /// render `row[col.id]` as text. Out-of-bounds indices fall back to text.
    #[prop(optional)]
    cell_renderers: Vec<CellRenderer>,

    /// Per-cell typed-cell resolvers indexed by `Column::typed_cell_index`,
    /// for lightweight `Badge`/`Icon` rendering without a full custom
    /// `CellRenderer`. Additive alongside `cell_renderers` -- a column's
    /// `renderer_index` (when set) always takes precedence.
    #[prop(optional)]
    typed_cells: Vec<TypedCellFn>,

    /// Optional per-row extra CSS classes (e.g. a background tint) computed
    /// from the row's absolute index and data. Merged with `classes.row`.
    #[prop(optional)]
    row_class_fn: Option<Callback<(usize, TableRow), String>>,

    /// Optional callback fired on a **plain** row click (no Ctrl/Shift) or a
    /// keyboard Enter/Space, receiving the row's index **within the current
    /// page** (the server variant renders one page at a time; combine with
    /// `current_page`/`page_size` for a global position). Same contract as
    /// [`DataTable`](super::DataTable)'s `on_row_activate` (`ldui-1gp`) —
    /// e.g. navigate to the row's detail page. A modified click stays inert
    /// here (the server variant has no selection state machine).
    #[prop(optional, into)]
    on_row_activate: Option<Callback<usize>>,

    /// Optional secondary activation fired on a row **double-click** or
    /// Shift+Enter, receiving the page-local row index — same dblclick
    /// discrimination as the client-paged table (`ldui-tmr`/`ldui-1gp`): the
    /// first click still activates once, the repeat click is swallowed so
    /// activation never fires twice, and the inspector fires exactly once.
    #[prop(optional, into)]
    on_row_inspect: Option<Callback<usize>>,
) -> impl IntoView {
    // Column-width overrides from dragging a header divider, keyed by
    // column id. Shared between the header (writer) and body (reader) so
    // resized columns stay aligned.
    let column_widths = RwSignal::new(HashMap::<&'static str, f64>::new());

    // Row activation, forwarded to the shared body exactly like the
    // client-paged DataTable (ldui-1gp): a plain click/Enter/Space activates;
    // a modified click is inert here because the server variant has no
    // selection state machine to feed. `on_row_inspect` rides the body's own
    // dblclick discrimination (detail > 1 swallow), so the timing matches
    // the client table by construction.
    // A bare Callback (not Option) so the body's `optional, into` prop takes
    // it the same way DataTable's own forwarding does — passing an Option
    // here is the E0308 trap the ldui-tmr CI fix documented.
    let on_row_click = Callback::new(move |(index, ctrl, shift): (usize, bool, bool)| {
        let Some(callback) = on_row_activate else {
            return;
        };
        if matches!(row_click_kind(ctrl, shift, true), RowClickKind::Activate) {
            callback.run(index);
        }
    });
    // Inspect alone still needs focusable rows for its Shift+Enter path.
    let row_interactive =
        row_is_interactive(false, on_row_activate.is_some() || on_row_inspect.is_some());
    let container_class = merge_classes!(classes.container, class);

    // Search state with debounce (only used when on_search is provided).
    // Mirrors the debounce pattern in `data_table::component::DataTable`
    // (see ldui-1ub) -- keep the two in sync if either changes.
    let (search_query, set_search_query) = signal(String::new());
    let (debounce_handle, set_debounce_handle) = signal(Option::<TimeoutHandle>::None);
    // The typed query API is an equally good reason to render the search box.
    let has_search = on_search.is_some() || on_query_change.is_some();

    // Sort state. The server owns the actual ordering -- these signals only
    // drive the header indicators and ride along in the emitted query.
    let (sort_column, set_sort_column) = signal(Option::<&'static str>::None);
    let (sort_order, set_sort_order) = signal(SortOrder::default());

    // Filter-row selections (only rendered when a column is `filterable`).
    // Selections are never applied to `rows` client-side: on a server table
    // the current page is a window, and filtering the window would lie about
    // the population. They ride along in the emitted query instead.
    let column_filters = RwSignal::new(ColumnFilters::new());

    // Assemble the current query. Untracked reads: emission points are event
    // handlers/effects that already know *when* to fire.
    let emit_query = move |page: i64| {
        if let Some(cb) = on_query_change {
            cb.run(TableQuery {
                page,
                page_size: page_size.get_untracked(),
                search: search_query.get_untracked(),
                sort: sort_column
                    .get_untracked()
                    .map(|c| (c, sort_order.get_untracked())),
                filters: column_filters.get_untracked(),
            });
        }
    };

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
        let fire = move |v: String| {
            if let Some(cb) = on_search {
                cb.run(v);
            }
            // A changed search invalidates the old page number.
            emit_query(1);
        };
        let value_for_timeout = value.clone();
        match set_timeout_with_handle(
            move || {
                // Late-firing guard (ldui-d54): `fire` reaches emit_query,
                // which untracked-reads several of this owner's signals. A
                // debounce that outlives the table must be a no-op, not a
                // process-wide disposed-signal panic.
                if search_query.try_get_untracked().is_none() {
                    return;
                }
                fire(value_for_timeout);
            },
            std::time::Duration::from_millis(300),
        ) {
            Ok(handle) => set_debounce_handle.set(Some(handle)),
            Err(_) => {
                fire(value);
                set_debounce_handle.set(None);
            }
        }
    };
    // Cancel a pending debounce on unmount — mirrors DataTable's own search
    // debounce cleanup; keep the two in sync (ldui-d54).
    on_cleanup(move || {
        if let Some(handle) = debounce_handle.try_get_untracked().flatten() {
            handle.clear();
        }
    });

    // Header sort: toggle the indicator state and report through the typed
    // query (the server does the actual ordering). Without `on_query_change`
    // there is nowhere to report to, and headers stay inert exactly as when
    // this callback was a no-op.
    let on_sort = Callback::new(move |col_id: &'static str| {
        if on_query_change.is_none() {
            return;
        }
        if sort_column.get_untracked() == Some(col_id) {
            set_sort_order.set(sort_order.get_untracked().toggle());
        } else {
            set_sort_column.set(Some(col_id));
            set_sort_order.set(SortOrder::Asc);
        }
        emit_query(1);
    });

    // Page navigation: keep the plain `on_page_change` contract and mirror it
    // through the typed query.
    let page_change = Callback::new(move |page: i64| {
        on_page_change.run(page);
        emit_query(page);
    });

    // Filter row plumbing (rendered only when a column is `filterable`).
    let show_filter_row = Memo::new(move |_| columns.with(|c| has_filterable_columns(c)));
    let effective_filter_options = Signal::derive(move || match filter_options {
        Some(opts) => opts.get(),
        // Fallback: distinct values of the current page. Understates a paged
        // population -- see the `filter_options` prop docs.
        None => rows.with(|r| {
            columns.with(|cols| {
                cols.iter()
                    .filter(|c| c.filterable)
                    .map(|c| (c.id, distinct_values(r, c.id)))
                    .collect::<HashMap<&'static str, Vec<String>>>()
            })
        }),
    });
    Effect::new(move |prev: Option<()>| {
        let _ = column_filters.get();
        if prev.is_some() {
            emit_query(1);
        }
    });

    // Container style for viewport-constrained scrolling
    let container_style =
        max_height.map(|h| format!("display: flex; flex-direction: column; max-height: {}", h));

    let has_max_height = container_style.is_some();
    let table_wrapper_style = if has_max_height {
        Some("flex: 1; overflow-y: auto; min-height: 0")
    } else {
        None
    };
    let controls_style = if has_max_height {
        Some("flex-shrink: 0; padding: 12px 0")
    } else {
        None
    };

    view! {
        <div
            class=container_class
            node_ref=node_ref
            style=container_style
            data-table-data-mode="server-query"
        >
            {move || {
                if has_search {
                    Some(view! {
                        <div class="mb-3">
                            <input
                                type="text"
                                class="input input-bordered input-sm w-full max-w-xs"
                                placeholder=move || texts.with(|t| t.search_placeholder.clone())
                                aria-label="Search table"
                                prop:value=move || search_query.get()
                                on:input=on_search_input
                            />
                        </div>
                    })
                } else {
                    None
                }
            }}

            <div class=TABLE_SCROLL_WRAPPER_CLASS style=table_wrapper_style>
                <Table
                    size=table_size
                    zebra=zebra
                    pin_rows=pin_rows
                    pin_cols=pin_cols
                >
                    <DataTableHeader
                        columns=columns
                        sort_column=Signal::derive(move || sort_column.get())
                        sort_order=Signal::derive(move || sort_order.get())
                        on_sort=on_sort
                        header_cell_class=classes.header_cell
                        column_widths=column_widths
                    >
                        {move || {
                            show_filter_row.get().then(|| view! {
                                <DataTableFilterRow
                                    columns=columns
                                    options=effective_filter_options
                                    filters=column_filters
                                    all_label=Signal::derive(move || {
                                        texts.with(|t| t.filter_all.clone())
                                    })
                                />
                            })
                        }}
                    </DataTableHeader>
                    <DataTableBody
                        columns=columns
                        rows=Signal::derive(move || {
                            rows.get().into_iter().enumerate().collect::<Vec<_>>()
                        })
                        loading=loading
                        texts=texts
                        body_cell_class=classes.body_cell
                        row_class=classes.row
                        loading_row_class=classes.loading_row
                        empty_row_class=classes.empty_row
                        cell_renderers=cell_renderers
                        column_widths=Signal::derive(move || column_widths.get())
                        typed_cells=typed_cells
                        row_class_fn=row_class_fn
                        on_row_click=on_row_click
                        on_row_inspect=on_row_inspect
                        interactive=row_interactive
                    />
                </Table>
            </div>

            {move || {
                let total = total_count.get();
                let size = page_size.get().max(1);
                let page = current_page.get();
                let total_pages = if total == 0 { 1 } else { ((total as f64) / (size as f64)).ceil() as i64 };

                if total > 0 && !loading.get() {
                    let start = ((page - 1) * size) + 1;
                    let end = (page * size).min(total);

                    Some(view! {
                        <div style=controls_style>
                            <ServerPaginationControls
                                current_page=page
                                total_pages=total_pages
                                total_count=total
                                start=start
                                end=end
                                on_page_change=page_change
                                texts=texts.get()
                                classes=classes.clone()
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

/// Server-side pagination controls with page numbers, ellipsis, and Showing X-Y of Z
#[component]
fn ServerPaginationControls(
    current_page: i64,
    total_pages: i64,
    total_count: i64,
    start: i64,
    end: i64,
    on_page_change: Callback<i64>,
    texts: DataTableTexts,
    classes: DataTableClasses,
) -> impl IntoView {
    // Build page number list with ellipsis
    let page_numbers = build_page_range(current_page, total_pages);

    // Extracted from the view because RSX parses an inline `>=` as a tag
    // close (same trap `DataTableControls` documents). Worse than a parse
    // error: `disabled=current_page >` truncated the attribute, demoted the
    // rest of the tag to text, and turned the Next button's click-guard block
    // into a *render-time child expression* — every render "clicked Next"
    // until the last page, silently walking a freshly mounted table to page
    // N. Unexercised until the typed-query demo made the walk visible.
    let prev_disabled = current_page <= 1;
    let next_disabled = current_page >= total_pages;

    view! {
        <div class=classes.pagination>
            <span class=classes.page_indicator>
                {texts
                    .row_range
                    .replace("{start}", &start.to_string())
                    .replace("{end}", &end.to_string())
                    .replace("{total}", &total_count.to_string())}
            </span>

            <div class="join">
                // Previous button
                <button
                    class=merge_classes!(classes.pagination_button, "join-item")
                    disabled=prev_disabled
                    on:click=move |_| {
                        if current_page > 1 {
                            on_page_change.run(current_page - 1);
                        }
                    }
                >
                    {texts.previous}
                </button>

                // Page number buttons
                {page_numbers.into_iter().map(|item| {
                    match item {
                        PageItem::Page(num) => {
                            let is_active = num == current_page;
                            let active_class = if is_active { "btn-active" } else { "" };
                            view! {
                                <button
                                    class=merge_classes!(classes.pagination_button, "join-item", active_class)
                                    disabled=is_active
                                    on:click=move |_| {
                                        on_page_change.run(num);
                                    }
                                >
                                    {num.to_string()}
                                </button>
                            }.into_any()
                        }
                        PageItem::Ellipsis => {
                            view! {
                                <button
                                    class=merge_classes!(classes.pagination_button, "join-item btn-ghost")
                                    disabled=true
                                >
                                    "..."
                                </button>
                            }.into_any()
                        }
                    }
                }).collect_view()}

                // Next button
                <button
                    class=merge_classes!(classes.pagination_button, "join-item")
                    disabled=next_disabled
                    on:click=move |_| {
                        if current_page < total_pages {
                            on_page_change.run(current_page + 1);
                        }
                    }
                >
                    {texts.next}
                </button>
            </div>
        </div>
    }
}

/// Represents a page number or ellipsis in pagination
#[derive(Clone, Copy, Debug, PartialEq)]
enum PageItem {
    Page(i64),
    Ellipsis,
}

/// Build a page range with ellipsis for large ranges.
///
/// Shows: first page, last page, current page, and 1 page on each side of current.
/// Ellipsis fills gaps larger than 1 page.
///
/// Examples:
/// - 7 pages, current=4: [1, 2, 3, 4, 5, 6, 7]
/// - 20 pages, current=1: [1, 2, ..., 20]
/// - 20 pages, current=10: [1, ..., 9, 10, 11, ..., 20]
/// - 20 pages, current=20: [1, ..., 19, 20]
fn build_page_range(current: i64, total: i64) -> Vec<PageItem> {
    if total <= 7 {
        return (1..=total).map(PageItem::Page).collect();
    }

    let mut pages: Vec<i64> = Vec::new();

    // Always include first and last
    pages.push(1);
    pages.push(total);

    // Include current and neighbors
    for p in (current - 1)..=(current + 1) {
        if p >= 1 && p <= total {
            pages.push(p);
        }
    }

    // Sort and deduplicate
    pages.sort();
    pages.dedup();

    // Build result with ellipsis
    let mut result = Vec::new();
    let mut prev = 0_i64;
    for &p in &pages {
        if prev > 0 && p - prev > 1 {
            result.push(PageItem::Ellipsis);
        }
        result.push(PageItem::Page(p));
        prev = p;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract page numbers from PageItem vec, using -1 for Ellipsis
    fn to_nums(items: &[PageItem]) -> Vec<i64> {
        items
            .iter()
            .map(|item| match item {
                PageItem::Page(n) => *n,
                PageItem::Ellipsis => -1,
            })
            .collect()
    }

    // ── Single page ──

    #[test]
    fn one_total_page() {
        let result = build_page_range(1, 1);
        assert_eq!(to_nums(&result), vec![1]);
    }

    // ── Small ranges (total <= 7, no ellipsis) ──

    #[test]
    fn five_pages_current_1() {
        let result = build_page_range(1, 5);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn five_pages_current_3() {
        let result = build_page_range(3, 5);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn five_pages_current_5() {
        let result = build_page_range(5, 5);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn seven_pages_all_shown() {
        let result = build_page_range(4, 7);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, 5, 6, 7]);
    }

    // ── Large ranges with ellipsis (total > 7) ──

    #[test]
    fn ten_pages_current_1() {
        // current=1, neighbors=[1,2], always first=1 last=10
        // pages: [1, 2, 10] -> [1, 2, ..., 10]
        let result = build_page_range(1, 10);
        assert_eq!(to_nums(&result), vec![1, 2, -1, 10]);
    }

    #[test]
    fn ten_pages_current_5() {
        // current=5, neighbors=[4,5,6], always first=1 last=10
        // pages: [1, 4, 5, 6, 10] -> [1, ..., 4, 5, 6, ..., 10]
        let result = build_page_range(5, 10);
        assert_eq!(to_nums(&result), vec![1, -1, 4, 5, 6, -1, 10]);
    }

    #[test]
    fn ten_pages_current_10() {
        // current=10, neighbors=[9,10], always first=1 last=10
        // pages: [1, 9, 10] -> [1, ..., 9, 10]
        let result = build_page_range(10, 10);
        assert_eq!(to_nums(&result), vec![1, -1, 9, 10]);
    }

    #[test]
    fn ten_pages_current_2() {
        // current=2, neighbors=[1,2,3], always first=1 last=10
        // pages: [1, 2, 3, 10] -> [1, 2, 3, ..., 10]
        let result = build_page_range(2, 10);
        assert_eq!(to_nums(&result), vec![1, 2, 3, -1, 10]);
    }

    #[test]
    fn ten_pages_current_9() {
        // current=9, neighbors=[8,9,10], always first=1 last=10
        // pages: [1, 8, 9, 10] -> [1, ..., 8, 9, 10]
        let result = build_page_range(9, 10);
        assert_eq!(to_nums(&result), vec![1, -1, 8, 9, 10]);
    }

    #[test]
    fn twenty_pages_current_10() {
        // current=10, neighbors=[9,10,11], always first=1 last=20
        // pages: [1, 9, 10, 11, 20] -> [1, ..., 9, 10, 11, ..., 20]
        let result = build_page_range(10, 20);
        assert_eq!(to_nums(&result), vec![1, -1, 9, 10, 11, -1, 20]);
    }

    // ── Edge: current near first or last with gap of exactly 1 ──

    #[test]
    fn ten_pages_current_3_no_left_ellipsis() {
        // current=3, neighbors=[2,3,4], always first=1 last=10
        // pages: [1, 2, 3, 4, 10] -> [1, 2, 3, 4, ..., 10]
        let result = build_page_range(3, 10);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, -1, 10]);
    }

    #[test]
    fn ten_pages_current_8_no_right_ellipsis() {
        // current=8, neighbors=[7,8,9], always first=1 last=10
        // pages: [1, 7, 8, 9, 10] -> [1, ..., 7, 8, 9, 10]
        let result = build_page_range(8, 10);
        assert_eq!(to_nums(&result), vec![1, -1, 7, 8, 9, 10]);
    }

    // ── Two pages ──

    #[test]
    fn two_pages_current_1() {
        let result = build_page_range(1, 2);
        assert_eq!(to_nums(&result), vec![1, 2]);
    }

    #[test]
    fn two_pages_current_2() {
        let result = build_page_range(2, 2);
        assert_eq!(to_nums(&result), vec![1, 2]);
    }

    // ── PageItem PartialEq ──

    #[test]
    fn page_item_page_equality() {
        assert_eq!(PageItem::Page(5), PageItem::Page(5));
        assert_ne!(PageItem::Page(5), PageItem::Page(6));
    }

    #[test]
    fn page_item_ellipsis_equality() {
        assert_eq!(PageItem::Ellipsis, PageItem::Ellipsis);
    }

    #[test]
    fn page_item_page_ne_ellipsis() {
        assert_ne!(PageItem::Page(1), PageItem::Ellipsis);
    }
}
