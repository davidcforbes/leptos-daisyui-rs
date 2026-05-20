use crate::components::data_table::body::DataTableBody;
use crate::components::data_table::header::DataTableHeader;
use crate::components::data_table::types::{
    Column, DataTableClasses, DataTableTexts, SortOrder, TableRow,
};
use crate::components::table::{Table, TableSize};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use web_sys::wasm_bindgen::JsCast;

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

    /// Custom text strings
    #[prop(optional)]
    texts: DataTableTexts,

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

    /// Node reference to container element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let container_class = merge_classes!(classes.container, class);
    let texts_for_body = texts.clone();
    let search_placeholder = texts.search_placeholder;

    // Search state with debounce (only used when on_search is provided)
    let (search_query, set_search_query) = signal(String::new());
    let (debounce_handle, set_debounce_handle) = signal(Option::<i32>::None);
    let has_search = on_search.is_some();

    let on_search_input = move |ev: leptos::ev::Event| {
        let target = ev.target().unwrap();
        let input: web_sys::HtmlInputElement = target.unchecked_into();
        let value = input.value();
        set_search_query.set(value.clone());

        // Clear previous timer
        if let Some(handle) = debounce_handle.get_untracked() {
            let window = web_sys::window().unwrap();
            window.clear_timeout_with_handle(handle);
        }

        // Set new 300ms debounce timer
        if let Some(cb) = on_search {
            let val = value.clone();
            let closure = wasm_bindgen::closure::Closure::once_into_js(move || {
                cb.run(val);
            });
            let window = web_sys::window().unwrap();
            let handle = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    300,
                )
                .unwrap();
            set_debounce_handle.set(Some(handle));
        }
    };

    // No-op sort callback — server tables don't sort client-side
    let noop_sort = Callback::new(move |_col_id: &'static str| {
        // Intentionally empty: server-side tables delegate sorting to the server
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
        <div class=container_class node_ref=node_ref style=container_style>
            {move || {
                if has_search {
                    Some(view! {
                        <div class="mb-3">
                            <input
                                type="text"
                                class="input input-bordered input-sm w-full max-w-xs"
                                placeholder=search_placeholder
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

            <div style=table_wrapper_style>
                <Table
                    size=table_size
                    zebra=zebra
                    pin_rows=pin_rows
                    pin_cols=pin_cols
                >
                    <DataTableHeader
                        columns=columns
                        sort_column=Signal::derive(move || None)
                        sort_order=Signal::derive(SortOrder::default)
                        on_sort=noop_sort
                        header_cell_class=classes.header_cell
                    />
                    <DataTableBody
                        columns=columns
                        rows=Signal::derive(move || {
                            rows.get().into_iter().enumerate().collect::<Vec<_>>()
                        })
                        loading=loading
                        texts=texts_for_body
                        body_cell_class=classes.body_cell
                        row_class=classes.row
                        loading_row_class=classes.loading_row
                        empty_row_class=classes.empty_row
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
                                on_page_change=on_page_change
                                texts=texts.clone()
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

    view! {
        <div class=classes.pagination>
            <span class=classes.page_indicator>
                {format!("Showing {}-{} of {}", start, end, total_count)}
            </span>

            <div class="join">
                // Previous button
                <button
                    class=merge_classes!(classes.pagination_button, "join-item")
                    disabled=current_page <= 1
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
                    disabled=current_page >= total_pages
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
