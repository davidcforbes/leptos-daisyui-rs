use crate::components::data_table::body::DataTableBody;
use crate::components::data_table::controls::DataTableControls;
use crate::components::data_table::header::DataTableHeader;
use crate::components::data_table::selection::handle_row_click;
use crate::components::data_table::types::{
    CellRenderer, Column, DataTableClasses, DataTableTexts, SortOrder, TableRow,
};
use crate::components::table::{Table, TableSize};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use std::collections::BTreeSet;
use web_sys::wasm_bindgen::JsCast;

/// # DataTable Component
///
/// A production-ready data table with sorting, pagination, loading states,
/// and efficient handling of large datasets (10,000+ rows).
///
/// ## Features
/// - Column-based sorting (click headers to toggle Asc/Desc)
/// - Pagination with customizable page size
/// - Loading and empty states
/// - Fully themed with daisyUI
/// - Efficient index-based operations for large datasets
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

    /// Number of rows per page (default: 10)
    #[prop(optional, into)]
    page_size: Signal<usize>,

    /// Loading state
    #[prop(optional, into)]
    loading: Signal<bool>,

    /// Enable pagination (default: true)
    #[prop(optional, into)]
    paginate: Signal<bool>,

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

    /// Node reference to container element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Per-cell renderers indexed by `Column::renderer_index`. A column with
    /// `renderer_index = Some(i)` invokes `cell_renderers[i]` with
    /// `(abs_idx, row)` to produce its cell view; columns without an index
    /// render `row[col.id]` as text. Out-of-bounds indices fall back to text.
    #[prop(optional)]
    cell_renderers: Vec<CellRenderer>,
) -> impl IntoView {
    // Default page size to 10 if not set
    let page_size = Signal::derive(move || {
        let size = page_size.get();
        if size == 0 { 10 } else { size }
    });

    // Default paginate to true
    let paginate = Signal::derive(move || {
        let p = paginate.get();
        if p { p } else { true }
    });

    // Pagination state
    let (current_page, set_current_page) = signal(0_usize);

    // Sorting state
    let (sort_column, set_sort_column) = signal(Option::<&'static str>::None);
    let (sort_order, set_sort_order) = signal(SortOrder::default());

    // Search state with debounce
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
                set_debounced_search.set(value_for_timeout);
                set_current_page.set(0);
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

    // Reset to page 1 when data changes
    Effect::new(move |_| {
        let _ = data.get();
        set_current_page.set(0);
    });

    // Filtered indices — applies search filter when searchable
    let filtered_indices = Memo::new(move |_| {
        let all_data = data.get();
        let query = debounced_search.get();
        if query.is_empty() {
            (0..all_data.len()).collect::<Vec<usize>>()
        } else {
            let q = query.to_lowercase();
            (0..all_data.len())
                .filter(|&i| {
                    if let Some(row) = all_data.get(i) {
                        row.values().any(|v| v.to_lowercase().contains(&q))
                    } else {
                        false
                    }
                })
                .collect::<Vec<usize>>()
        }
    });

    // Sorted indices
    let sorted_indices = Memo::new(move |_| {
        let mut indices = filtered_indices.get();
        if let Some(col_id) = sort_column.get() {
            let data_vec = data.get();
            indices.sort_by(|&a, &b| {
                let a_val = data_vec
                    .get(a)
                    .and_then(|row| row.get(col_id))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let b_val = data_vec
                    .get(b)
                    .and_then(|row| row.get(col_id))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                match sort_order.get() {
                    SortOrder::Asc => a_val.cmp(b_val),
                    SortOrder::Desc => b_val.cmp(a_val),
                }
            });
        }
        indices
    });

    // Total pages calculation with safety guards
    let total_pages = Memo::new(move |_| {
        let safe_page_size = page_size.get().max(1); // Prevent division by zero
        let total_items = sorted_indices.get().len();
        if total_items == 0 {
            1 // Always show at least "Page 1 of 1"
        } else {
            ((total_items as f64 / safe_page_size as f64).ceil() as usize).max(1)
        }
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
        if sort_column.get() == Some(col_id) {
            // Same column: toggle order
            set_sort_order.set(sort_order.get().toggle());
        } else {
            // New column: set to Asc
            set_sort_column.set(Some(col_id));
            set_sort_order.set(SortOrder::Asc);
        }
    });

    // Selection state — owned locally if the consumer didn't pass their own.
    let selected_rows = selected_rows.unwrap_or_else(|| RwSignal::new(BTreeSet::new()));
    let selection_anchor = selection_anchor.unwrap_or_else(|| RwSignal::new(None));

    // Clear selection on data refresh or sort change; indices may no
    // longer point at the same row after either of those events.
    Effect::new(move |prev: Option<()>| {
        let _ = data.get();
        let _ = sort_column.get();
        let _ = sort_order.get();
        if prev.is_some() {
            selected_rows.update(|s| s.clear());
            selection_anchor.set(None);
        }
    });

    // Row-click callback wiring multi-select semantics.
    let on_row_click = Callback::new(move |(abs_idx, ev): (usize, web_sys::MouseEvent)| {
        let total = data.with(|d| d.len());
        let ctrl = ev.ctrl_key() || ev.meta_key();
        let shift = ev.shift_key();
        let mut next = selected_rows.get_untracked();
        let mut anchor = selection_anchor.get_untracked();
        handle_row_click(abs_idx, ctrl, shift, &mut next, &mut anchor, total);
        selected_rows.set(next);
        selection_anchor.set(anchor);
    });

    let container_class = merge_classes!(classes.container, class);
    let texts_for_body = texts.clone();
    let texts_for_controls = texts.clone();
    let search_placeholder = texts.search_placeholder;

    // Container style for viewport-constrained scrolling
    let container_style =
        max_height.map(|h| format!("display: flex; flex-direction: column; max-height: {}", h));

    // Table wrapper style when max_height is set
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
                if searchable.get() {
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
                        sort_column=Signal::derive(move || sort_column.get())
                        sort_order=Signal::derive(move || sort_order.get())
                        on_sort=on_sort
                        header_cell_class=classes.header_cell
                    />
                    <DataTableBody
                        columns=columns
                        rows=Signal::derive(move || current_page_rows.get())
                        loading=loading
                        texts=texts_for_body
                        body_cell_class=classes.body_cell
                        row_class=classes.row
                        selected_row_class=classes.selected_row
                        selected_rows=Signal::derive(move || selected_rows.get())
                        loading_row_class=classes.loading_row
                        empty_row_class=classes.empty_row
                        on_row_click=on_row_click
                        cell_renderers=cell_renderers
                    />
                </Table>
            </div>

            {move || {
                if paginate.get() && !loading.get() && !data.get().is_empty() {
                    Some(view! {
                        <div style=controls_style>
                            <DataTableControls
                                current_page=Signal::derive(move || current_page.get())
                                set_current_page=set_current_page
                                total_pages=Signal::derive(move || total_pages.get())
                                texts=texts_for_controls.clone()
                                pagination_class=classes.pagination
                                button_class=classes.pagination_button
                                indicator_class=classes.page_indicator
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
