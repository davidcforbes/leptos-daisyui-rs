use crate::components::data_table::resize::{effective_min_width, resized_width};
use crate::components::data_table::types::{Column, SortOrder};
use crate::merge_classes;
use leptos::prelude::*;
use std::collections::HashMap;
use web_sys::wasm_bindgen::JsCast;

/// State of an in-progress column-resize drag. Only one drag can be active
/// at a time per `DataTableHeader` instance.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ResizeDrag {
    col_id: &'static str,
    start_x: f64,
    start_width: f64,
}

/// DataTable header component with sortable columns and draggable
/// column-width dividers.
#[component]
pub fn DataTableHeader(
    /// Column definitions
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Currently sorted column ID
    #[prop(into)]
    sort_column: Signal<Option<&'static str>>,

    /// Current sort order
    #[prop(into)]
    sort_order: Signal<SortOrder>,

    /// Callback when column header is clicked
    on_sort: Callback<&'static str>,

    /// Custom header cell class
    #[prop(optional, into)]
    header_cell_class: &'static str,

    /// Column-width overrides set by dragging a header divider, keyed by
    /// column id. Owned by the parent `DataTable`/`ServerDataTable` and
    /// shared with `DataTableBody` so `<td>` widths stay in sync with the
    /// header regardless of the browser's table layout algorithm. Must be
    /// the *same* signal instance passed to `DataTableBody`'s
    /// `column_widths` prop.
    column_widths: RwSignal<HashMap<&'static str, f64>>,
) -> impl IntoView {
    // Local drag state -- not shared outside this header instance.
    let resize_drag = RwSignal::new(Option::<ResizeDrag>::None);

    view! {
        <thead>
            <tr>
                {move || {
                    columns.get().iter().map(|col| {
                        let col_id = col.id;
                        let header_label = col.header;
                        let is_sorted = sort_column.get() == Some(col_id);
                        let is_sortable = col.sortable;
                        let is_resizable = col.resizable;
                        let min_width_opt = col.min_width;
                        let min_w = effective_min_width(min_width_opt);

                        let cell_class = if is_sortable {
                            merge_classes!(header_cell_class, col.class.unwrap_or(""), "relative")
                        } else {
                            merge_classes!(header_cell_class, col.class.unwrap_or(""), "opacity-50 relative")
                        };

                        let aria_sort = if is_sorted {
                            Some(sort_order.get().as_aria_str())
                        } else {
                            Some("none")
                        };

                        // Explicit width (set by a prior resize drag) always
                        // wins; otherwise fall back to the original
                        // `min_width`-only style so unresized columns are
                        // unaffected.
                        let width_style = column_widths
                            .with(|m| m.get(col_id).copied())
                            .map(|w| format!("width: {}px; min-width: {}px; max-width: {}px", w.round(), w.round(), w.round()))
                            .or_else(|| min_width_opt.map(|w| format!("min-width: {}px", w)));

                        view! {
                            <th
                                class=cell_class
                                role="columnheader"
                                aria-sort=aria_sort
                                style=width_style
                                on:click=move |_| {
                                    if is_sortable {
                                        on_sort.run(col_id);
                                    }
                                }
                            >
                                <span class="flex items-center gap-1">
                                    {col.header}
                                    {move || {
                                        if is_sorted {
                                            Some(view! {
                                                <span class="text-xs">
                                                    {sort_order.get().as_symbol()}
                                                </span>
                                            })
                                        } else {
                                            None
                                        }
                                    }}
                                </span>
                                {is_resizable.then(|| view! {
                                    <span
                                        class="absolute top-0 right-0 z-10 h-full w-1.5 cursor-col-resize select-none opacity-0 hover:opacity-100 hover:bg-primary/50 active:opacity-100 active:bg-primary/70"
                                        role="separator"
                                        aria-orientation="vertical"
                                        aria-label=format!("Resize {} column", header_label)
                                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                        on:pointerdown=move |ev: web_sys::PointerEvent| {
                                            ev.stop_propagation();
                                            // Start from the divider's own <th> rendered width when
                                            // available (most accurate), else the last override, else
                                            // the effective minimum.
                                            let rendered_width = ev
                                                .target()
                                                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                                .and_then(|el| el.parent_element())
                                                .and_then(|th| th.dyn_into::<web_sys::HtmlElement>().ok())
                                                .map(|th| th.offset_width() as f64);
                                            let start_width = rendered_width
                                                .or_else(|| column_widths.with_untracked(|m| m.get(col_id).copied()))
                                                .unwrap_or(min_w);
                                            resize_drag.set(Some(ResizeDrag {
                                                col_id,
                                                start_x: ev.client_x() as f64,
                                                start_width,
                                            }));
                                            if let Some(target) = ev.target()
                                                && let Ok(el) = target.dyn_into::<web_sys::Element>()
                                            {
                                                let _ = el.set_pointer_capture(ev.pointer_id());
                                            }
                                        }
                                        on:pointermove=move |ev: web_sys::PointerEvent| {
                                            if let Some(drag) = resize_drag.get_untracked() {
                                                let new_width = resized_width(
                                                    drag.start_width,
                                                    drag.start_x,
                                                    ev.client_x() as f64,
                                                    min_w,
                                                );
                                                column_widths.update(|m| {
                                                    m.insert(drag.col_id, new_width);
                                                });
                                            }
                                        }
                                        on:pointerup=move |_ev: web_sys::PointerEvent| {
                                            resize_drag.set(None);
                                        }
                                        on:pointercancel=move |_ev: web_sys::PointerEvent| {
                                            resize_drag.set(None);
                                        }
                                    ></span>
                                })}
                            </th>
                        }
                    }).collect_view()
                }}
            </tr>
        </thead>
    }
}
