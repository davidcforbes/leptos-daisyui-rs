use crate::components::data_table::types::{CellRenderer, Column, DataTableTexts, TableRow};
use crate::merge_classes;
use leptos::prelude::*;
use std::collections::BTreeSet;

/// DataTable body component with loading and empty states
#[component]
pub fn DataTableBody(
    /// Column definitions
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Current page rows paired with their absolute index into the underlying data
    #[prop(into)]
    rows: Signal<Vec<(usize, TableRow)>>,

    /// Loading state
    #[prop(into)]
    loading: Signal<bool>,

    /// Custom text strings
    texts: DataTableTexts,

    /// Custom body cell class
    #[prop(optional, into)]
    body_cell_class: &'static str,

    /// Custom row class
    #[prop(optional, into)]
    row_class: &'static str,

    /// Class applied when a row's absolute index is in `selected_rows`
    #[prop(optional, into)]
    selected_row_class: &'static str,

    /// Selected-row state, keyed by absolute index into the underlying data
    #[prop(optional, into)]
    selected_rows: Signal<BTreeSet<usize>>,

    /// Loading row class
    #[prop(optional, into)]
    loading_row_class: &'static str,

    /// Empty row class
    #[prop(optional, into)]
    empty_row_class: &'static str,

    /// Click callback invoked with the row's absolute index and the raw `MouseEvent`
    #[prop(optional, into)]
    on_row_click: Option<Callback<(usize, web_sys::MouseEvent)>>,

    /// Per-cell renderers. A column with `renderer_index = Some(i)` invokes
    /// `cell_renderers[i]` with `(abs_idx, row)`; otherwise the cell renders
    /// `row[col.id]` as plain text. Out-of-bounds indices fall back to text.
    #[prop(optional)]
    cell_renderers: Vec<CellRenderer>,
) -> impl IntoView {
    view! {
        <tbody>
            {move || {
                if loading.get() {
                    // Loading state
                    let col_count = columns.get().len();
                    view! {
                        <tr class=loading_row_class>
                            <td colspan=col_count class="text-center py-8">
                                {texts.loading}
                            </td>
                        </tr>
                    }.into_any()
                } else if rows.get().is_empty() {
                    // Empty state
                    let col_count = columns.get().len();
                    view! {
                        <tr class=empty_row_class>
                            <td colspan=col_count class="text-center py-8">
                                {texts.empty}
                            </td>
                        </tr>
                    }.into_any()
                } else {
                    // Data rows
                    let rows_vec = rows.get();
                    let cols = columns.get();
                    let renderers = cell_renderers.clone();

                    rows_vec.iter().map(|(abs_idx, row)| {
                        let abs_idx = *abs_idx;
                        let click_handler = on_row_click.map(|cb| {
                            move |ev: web_sys::MouseEvent| cb.run((abs_idx, ev))
                        });
                        let row_class_dyn = Signal::derive(move || {
                            if selected_rows.with(|s| s.contains(&abs_idx)) {
                                merge_classes!(row_class, selected_row_class).to_class()
                            } else {
                                merge_classes!(row_class).to_class()
                            }
                        });

                        view! {
                            <tr
                                class=move || row_class_dyn.get()
                                on:click=move |ev| {
                                    if let Some(h) = &click_handler {
                                        h(ev);
                                    }
                                }
                            >
                                {cols.iter().map(|col| {
                                    let cell_value = row.get(col.id).cloned().unwrap_or_default();
                                    let cell_class = merge_classes!(body_cell_class, col.class.unwrap_or(""));

                                    // Build truncation style if enabled
                                    let truncate_style = if col.truncate {
                                        let max_w = col.max_width.map(|w| format!("max-width: {}px; ", w)).unwrap_or_default();
                                        Some(format!("{}overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", max_w))
                                    } else {
                                        None
                                    };

                                    // Title attribute for native tooltip when truncated
                                    let title_attr = if col.truncate {
                                        Some(cell_value.clone())
                                    } else {
                                        None
                                    };

                                    // Custom renderer if column opts in and index is in range;
                                    // otherwise render the cell as plain text.
                                    let content = match col.renderer_index.and_then(|i| renderers.get(i)) {
                                        Some(renderer) => renderer.run((abs_idx, row.clone())),
                                        None => view! { {cell_value.clone()} }.into_any(),
                                    };

                                    view! {
                                        <td class=cell_class style=truncate_style title=title_attr>
                                            {content}
                                        </td>
                                    }
                                }).collect_view()}
                            </tr>
                        }
                    }).collect_view().into_any()
                }
            }}
        </tbody>
    }
}
