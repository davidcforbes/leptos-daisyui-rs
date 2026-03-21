use crate::components::data_table::types::{Column, DataTableTexts, TableRow};
use crate::merge_classes;
use leptos::prelude::*;

/// DataTable body component with loading and empty states
#[component]
pub fn DataTableBody(
    /// Column definitions
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Current page rows to display
    #[prop(into)]
    rows: Signal<Vec<TableRow>>,

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

    /// Loading row class
    #[prop(optional, into)]
    loading_row_class: &'static str,

    /// Empty row class
    #[prop(optional, into)]
    empty_row_class: &'static str,
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

                    rows_vec.iter().map(|row| {
                        view! {
                            <tr class=row_class>
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

                                    let content = {
                                        let v = cell_value.clone();
                                        view! { {v} }
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
