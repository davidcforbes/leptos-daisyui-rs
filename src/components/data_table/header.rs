use crate::components::data_table::resize::{
    MAX_COLUMN_WIDTH, effective_min_width, keyboard_resized_width, resized_width,
};
use crate::components::data_table::types::{Column, DataTableSortTexts, SortOrder};
use crate::components::{Button, ButtonSize, ButtonStyle};
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

    /// Localized current-state and next-action copy for focused sort controls.
    #[prop(into)]
    sort_texts: Signal<DataTableSortTexts>,

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

    /// Extra rows rendered inside this `<thead>` beneath the sortable header
    /// row -- `DataTable` passes its filter row here when any column is
    /// [`filterable`](Column::filterable).
    ///
    /// Must resolve to `<tr>` elements: they are children of a `<thead>`, and a
    /// browser will hoist anything else out of the table.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    // Local drag state -- not shared outside this header instance.
    let resize_drag = RwSignal::new(Option::<ResizeDrag>::None);

    view! {
        <thead>
            <tr>
                <For
                    each=move || columns.get()
                    key=|col| (
                        col.id,
                        col.header.clone(),
                        col.sortable,
                        col.resizable,
                        col.min_width,
                    )
                    children=move |col| {
                        let col_id = col.id;
                        let header_label = col.header.clone();
                        let is_sortable = col.sortable;
                        let is_resizable = col.resizable;
                        let min_width_opt = col.min_width;
                        let min_w = effective_min_width(min_width_opt);

                        let cell_class =
                            merge_classes!(
                                header_cell_class,
                                col.class.unwrap_or(""),
                                "relative border border-table-grid bg-table-header text-table-header-content forced-colors:border-[CanvasText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                            );

                        let aria_sort = move || {
                            if !is_sortable {
                                None
                            } else if sort_column.get() == Some(col_id) {
                                Some(sort_order.get().as_aria_str())
                            } else {
                                Some("none")
                            }
                        };
                        let sort_header = header_label.clone();
                        let sort_label = move || {
                            let current = (sort_column.get() == Some(col_id))
                                .then(|| sort_order.get());
                            sort_texts.with(|texts| texts.control_label(&sort_header, current))
                        };

                        // Explicit width (set by a prior resize drag) always
                        // wins; otherwise fall back to the original
                        // `min_width`-only style so unresized columns are
                        // unaffected. Scoped to its own reactive closure bound
                        // to just this `<th>`'s `style` attribute (rather than
                        // read in this outer per-column map) so a
                        // column-width drag only re-renders this header
                        // cell's style instead of rebuilding the whole header
                        // row -- and never replaces the resize-handle DOM
                        // node mid-drag.
                        let width_style = move || {
                            column_widths
                                .with(|m| m.get(col_id).copied())
                                .map(|w| format!("width: {}px; min-width: {}px; max-width: {}px", w.round(), w.round(), w.round()))
                                .or_else(|| {
                                    min_width_opt
                                        .map(|_| format!("min-width: {}px", min_w.round()))
                                })
                        };

                        view! {
                            <th
                                class=cell_class
                                role="columnheader"
                                aria-sort=aria_sort
                                style=width_style
                            >
                                {is_sortable.then(|| {
                                    let header = col.header.clone();
                                    view! {
                                        <Button
                                            style=ButtonStyle::Ghost
                                            size=ButtonSize::Sm
                                            class="h-auto !min-h-0 w-full justify-start gap-1 rounded-sm px-0 py-1 text-left font-semibold text-table-header-content !shadow-none hover:bg-white/15 focus-visible:outline-white forced-colors:text-[CanvasText]"
                                            attr:data-table-sort-column=col_id
                                            attr:aria-label=sort_label
                                            on_click=Callback::new(move |_| on_sort.run(col_id))
                                        >
                                            <span>{header}</span>
                                            <span
                                                aria-hidden="true"
                                                data-table-sort-indicator="true"
                                                class="inline-flex w-4 shrink-0 justify-center text-xs"
                                            >
                                                {move || {
                                                    if sort_column.get() == Some(col_id) {
                                                        sort_order.get().as_symbol()
                                                    } else {
                                                        ""
                                                    }
                                                }}
                                            </span>
                                        </Button>
                                    }
                                })}
                                {(!is_sortable).then(|| view! {
                                    <span class="flex items-center gap-1">{col.header.clone()}</span>
                                })}
                                {is_resizable.then(|| view! {
                                    <span
                                        class="absolute top-0 right-0 z-10 h-full w-2 cursor-col-resize select-none opacity-0 hover:opacity-100 hover:bg-primary/50 focus:opacity-100 focus:bg-primary/50 focus:outline focus:outline-2 focus:outline-primary active:opacity-100 active:bg-primary/70"
                                        role="separator"
                                        tabindex="0"
                                        aria-orientation="vertical"
                                        aria-label=format!("Resize {} column", header_label)
                                        aria-valuemin=min_w.round() as u32
                                        aria-valuemax=MAX_COLUMN_WIDTH.round() as u32
                                        aria-valuenow=move || column_widths.with(|widths| {
                                            widths
                                                .get(col_id)
                                                .copied()
                                                .unwrap_or(min_w)
                                                .round() as u32
                                        })
                                        aria-valuetext=move || column_widths.with(|widths| {
                                            format!(
                                                "{} pixels",
                                                widths
                                                    .get(col_id)
                                                    .copied()
                                                    .unwrap_or(min_w)
                                                    .round() as u32
                                            )
                                        })
                                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                        on:focus=move |ev: web_sys::FocusEvent| {
                                            if let Some(rendered_width) = separator_parent_width(ev.target()) {
                                                column_widths.update(|widths| {
                                                    widths.insert(
                                                        col_id,
                                                        rendered_width
                                                            .clamp(min_w, MAX_COLUMN_WIDTH)
                                                            .round(),
                                                    );
                                                });
                                            }
                                        }
                                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                                            let current_width = separator_parent_width(
                                                ev.current_target().or_else(|| ev.target()),
                                            )
                                            .or_else(|| column_widths.with_untracked(|widths| {
                                                widths.get(col_id).copied()
                                            }))
                                            .unwrap_or(min_w);
                                            let Some(new_width) = keyboard_resized_width(
                                                current_width,
                                                &ev.key(),
                                                min_w,
                                            ) else {
                                                return;
                                            };
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            column_widths.update(|widths| {
                                                widths.insert(col_id, new_width.round());
                                            });
                                        }
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
                                        on:pointerup=move |ev: web_sys::PointerEvent| {
                                            if let Some(target) = ev.target()
                                                && let Ok(el) = target.dyn_into::<web_sys::Element>()
                                            {
                                                let _ = el.release_pointer_capture(ev.pointer_id());
                                            }
                                            resize_drag.set(None);
                                        }
                                        on:pointercancel=move |ev: web_sys::PointerEvent| {
                                            if let Some(target) = ev.target()
                                                && let Ok(el) = target.dyn_into::<web_sys::Element>()
                                            {
                                                let _ = el.release_pointer_capture(ev.pointer_id());
                                            }
                                            resize_drag.set(None);
                                        }
                                    ></span>
                                })}
                            </th>
                        }
                    }
                />
            </tr>
            {children.map(|c| c())}
        </thead>
    }
}

fn separator_parent_width(target: Option<web_sys::EventTarget>) -> Option<f64> {
    target
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| element.parent_element())
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        .map(|element| f64::from(element.offset_width()))
}
