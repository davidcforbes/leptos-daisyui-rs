//! Reactive renderer for the typed client-side table model.

use super::model::{
    ENTITY_PAGE_SIZE_CHOICES, SortedIndexCache, clamp_page, next_sort, page_after_dataset_change,
    page_after_row_delta, page_bounds, page_count, reset_columns, reset_sort, set_preferred_width,
    toggle_hidden_column,
};
use super::storage::{load_preferences, save_preferences};
use super::types::{
    EntityColumn, EntityRowKey, EntityRowRenderer, EntitySort, EntityTablePreferences,
    EntityTableTexts,
};
use crate::components::button::Button;
use crate::components::data_table::{PageSlot, page_window, row_range};
use crate::components::dropdown::{Dropdown, DropdownContent, DropdownPlacement};
use crate::components::menu::{Menu, MenuCheckItem};
use crate::components::pagination::Pagination;
use crate::components::select::Select;
use crate::merge_classes;
use leptos::prelude::*;
use std::collections::BTreeMap;
use std::rc::Rc;
use web_sys::wasm_bindgen::JsCast;

const MAX_VISIBLE_PAGES: usize = 7;

#[derive(Clone, Debug, PartialEq)]
struct ResizeDrag {
    column_id: String,
    start_x: f64,
    start_width: f64,
    minimum_width: Option<u32>,
}

/// A typed, client-side table for complete dataset snapshots.
///
/// Ordering is represented as an index permutation, so source data and row
/// identity are never mutated. Only rows on the current page are cloned for
/// rendering. Wide and compact layouts share the same `<tr>` nodes, preventing
/// hidden duplicate pages in the DOM.
#[component]
pub fn EntityTable<T>(
    /// Complete, locally filterable dataset. Use a local signal when `T` is not `Send`.
    #[prop(into)]
    data: Signal<Rc<Vec<T>>, LocalStorage>,
    /// Typed column definitions in system order.
    columns: Vec<EntityColumn<T>>,
    /// Stable key callback used for DOM identity and row activation.
    row_key: EntityRowKey<T>,
    /// Identity of the selected dataset. Changing it resets pagination only.
    #[prop(into)]
    dataset_identity: Signal<String>,
    /// Optional view-state identity. Changing it resets pagination while
    /// preserving dataset-independent filters, sort, page size, and columns.
    /// Use this for immediate local-filter changes.
    #[prop(optional, into)]
    page_reset_key: Option<Signal<String>>,
    /// Optional renderer for the single-cell compact row layout.
    #[prop(optional)]
    compact_row: Option<EntityRowRenderer<T>>,
    /// Optional callback that makes rows mouse- and keyboard-operable.
    #[prop(optional)]
    on_row_activate: Option<Callback<String>>,
    /// Preference namespace appended to the framework storage prefix.
    #[prop(optional)]
    storage_key: Option<&'static str>,
    /// Consumer-controlled preference schema version.
    #[prop(default = 1)]
    preference_version: u16,
    /// Localizable labels for table controls.
    #[prop(into, default = Signal::stored(EntityTableTexts::default()))]
    texts: Signal<EntityTableTexts>,
    /// Shows separate reset-sort and reset-columns actions.
    #[prop(optional, default = false)]
    show_reset_actions: bool,
    /// Additional outer-container classes.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView
where
    T: Clone + 'static,
{
    let initial_preferences = load_preferences(storage_key, preference_version, &columns);
    let initial_widths = rendered_column_widths(&initial_preferences, &columns);

    let column_store = StoredValue::new_local(columns);
    let row_key = StoredValue::new_local(row_key);
    let compact_row = StoredValue::new_local(compact_row);
    let sorted_index_cache = StoredValue::new_local(SortedIndexCache::new());
    let preferences = RwSignal::new(initial_preferences);
    let column_widths = RwSignal::new(initial_widths);
    let current_page = RwSignal::new(0_usize);
    let previous_dataset = StoredValue::new(dataset_identity.get_untracked());
    let resize_drag = RwSignal::new(Option::<ResizeDrag>::None);

    Effect::new(move |_| {
        let next_dataset = dataset_identity.get();
        let previous = previous_dataset.get_value();
        let next_page =
            page_after_dataset_change(current_page.get_untracked(), previous, next_dataset.clone());
        current_page.set(next_page);
        previous_dataset.set_value(next_dataset);
    });

    if let Some(page_reset_key) = page_reset_key {
        let previous_page_reset = StoredValue::new(page_reset_key.get_untracked());
        Effect::new(move |_| {
            let next_key = page_reset_key.get();
            let previous = previous_page_reset.get_value();
            let next_page =
                page_after_dataset_change(current_page.get_untracked(), previous, next_key.clone());
            current_page.set(next_page);
            previous_page_reset.set_value(next_key);
        });
    }

    Effect::new(move |_| {
        let total_rows = data.get().len();
        let page_size = preferences.with(|preferences| preferences.page_size);
        let next_page = page_after_row_delta(current_page.get_untracked(), page_size, total_rows);
        if next_page != current_page.get_untracked() {
            current_page.set(next_page);
        }
    });

    Effect::new(move |_| {
        preferences.with(|preferences| save_preferences(storage_key, preferences));
    });

    let visible_columns = move || {
        let preferences = preferences.get();
        column_store.with_value(|columns| {
            columns
                .iter()
                .filter(|column| !preferences.hidden_columns.contains(column.id))
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    let total_rows = Signal::derive_local(move || data.get().len());
    let total_pages = Signal::derive(move || {
        page_count(
            total_rows.get(),
            preferences.with(|preferences| preferences.page_size),
        )
    });

    view! {
        <section
            class=merge_classes!("w-full min-w-0 space-y-3", class)
            data-entity-table="true"
        >
            <div class="flex flex-wrap items-center justify-end gap-2">
                <label class="flex items-center gap-2 text-sm text-base-content/75">
                    <span>{move || texts.with(|texts| texts.rows_per_page.clone())}</span>
                    <Select
                        class="select-sm w-20"
                        label=Signal::derive(move || {
                            Some(texts.with(|texts| texts.rows_per_page.clone()))
                        })
                        value=Signal::derive(move || {
                            preferences.with(|preferences| preferences.page_size.to_string())
                        })
                        on_change=Callback::new(move |value: String| {
                            if let Ok(page_size) = value.parse::<usize>()
                                && ENTITY_PAGE_SIZE_CHOICES.contains(&page_size)
                            {
                                preferences
                                    .update(|preferences| preferences.page_size = page_size);
                                current_page.set(0);
                            }
                        })
                    >
                        {ENTITY_PAGE_SIZE_CHOICES.into_iter().map(|page_size| view! {
                            <option value=page_size.to_string()>{page_size}</option>
                        }).collect_view()}
                    </Select>
                </label>

                <Dropdown class="dropdown-end" placement=DropdownPlacement::Bottom>
                    <div
                        tabindex="0"
                        role="button"
                        data-entity-column-chooser="true"
                        aria-label=move || texts.with(|texts| texts.choose_columns.clone())
                        class="btn btn-ghost btn-sm"
                    >
                        {move || texts.with(|texts| texts.choose_columns.clone())}
                    </div>
                    <DropdownContent class="bg-base-100 rounded-box z-[2] w-56 p-0 shadow-lg border border-base-300">
                        <Menu class="w-full">
                            {move || column_store.with_value(|columns| {
                                columns
                                    .iter()
                                    .filter(|column| !column.required)
                                    .cloned()
                                    .map(|column| {
                                        let column_id = column.id;
                                        let checked = Signal::derive(move || {
                                            !preferences.with(|preferences| {
                                                preferences.hidden_columns.contains(column_id)
                                            })
                                        });
                                        let on_toggle = Callback::new(move |_| {
                                            column_store.with_value(|columns| {
                                                preferences.update(|preferences| {
                                                    toggle_hidden_column(
                                                        preferences,
                                                        columns,
                                                        column_id,
                                                    );
                                                });
                                            });
                                        });
                                        view! {
                                            <MenuCheckItem
                                                checked=checked
                                                on_toggle=on_toggle
                                                attr:data-entity-column=column_id
                                            >
                                                {column.header}
                                            </MenuCheckItem>
                                        }
                                    })
                                    .collect_view()
                            })}
                        </Menu>
                    </DropdownContent>
                </Dropdown>

                {show_reset_actions.then(|| view! {
                    <Button
                        class="btn-ghost btn-sm"
                        attr:data-entity-reset-sort="true"
                        disabled=Signal::derive(move || {
                            preferences.with(|preferences| preferences.sort == EntitySort::System)
                        })
                        on_click=Callback::new(move |_| {
                            preferences.update(|preferences| {
                                reset_sort(preferences);
                            });
                            current_page.set(0);
                        })
                    >
                        {move || texts.with(|texts| texts.reset_sort.clone())}
                    </Button>
                    <Button
                        class="btn-ghost btn-sm"
                        attr:data-entity-reset-columns="true"
                        disabled=Signal::derive(move || preferences.with(|preferences| {
                            preferences.hidden_columns.is_empty()
                                && preferences.column_widths.is_empty()
                        }))
                        on_click=Callback::new(move |_| {
                            preferences.update(|preferences| {
                                reset_columns(preferences);
                            });
                            column_store.with_value(|columns| {
                                column_widths.set(rendered_column_widths(
                                    &EntityTablePreferences::new(preference_version),
                                    columns,
                                ));
                            });
                        })
                    >
                        {move || texts.with(|texts| texts.reset_columns.clone())}
                    </Button>
                })}
            </div>

            <div class="w-full overflow-x-auto rounded-box border border-base-300 bg-base-100">
                <table class="table table-sm table-zebra w-full" data-entity-table-grid="true">
                    <thead class="hidden lg:table-header-group">
                        <tr>
                            {move || visible_columns().into_iter().map(|column| {
                                let column_id = column.id;
                                let header = column.header.clone();
                                let sortable = column.sortable;
                                let resizable = column.resizable;
                                let minimum_width = column.min_width;
                                let width_style = move || {
                                    column_widths
                                        .with(|widths| widths.get(column_id).copied())
                                        .map(|width| format!(
                                            "width: {width}px; min-width: {width}px; max-width: {width}px"
                                        ))
                                        .or_else(|| minimum_width.map(|width| format!("min-width: {width}px")))
                                };
                                let sort_label = move || preferences.with(|preferences| {
                                    format!(
                                        "{}: {}",
                                        header,
                                        preferences.sort.next_label(column_id)
                                    )
                                });
                                view! {
                                    <th
                                        class="relative"
                                        scope="col"
                                        aria-sort=move || preferences.with(|preferences| {
                                            preferences.sort.aria_value_for(column_id)
                                        })
                                        style=width_style
                                    >
                                        {if sortable {
                                            let header = column.header.clone();
                                            Some(view! {
                                                <Button
                                                    class="btn-ghost btn-xs h-auto !min-h-0 w-full justify-start gap-1 rounded-sm px-0 py-1 text-left font-semibold !shadow-none"
                                                    attr:aria-label=sort_label
                                                    on_click=Callback::new(move |_| {
                                                        preferences.update(|preferences| {
                                                            preferences.sort = next_sort(
                                                                &preferences.sort,
                                                                column_id,
                                                                true,
                                                            );
                                                        });
                                                        current_page.set(0);
                                                    })
                                                >
                                                    <span>{header}</span>
                                                    <span aria-hidden="true" class="text-xs">
                                                        {move || preferences.with(|preferences| match &preferences.sort {
                                                            EntitySort::Ascending { column } if column == column_id => "▲",
                                                            EntitySort::Descending { column } if column == column_id => "▼",
                                                            _ => "↕",
                                                        })}
                                                    </span>
                                                </Button>
                                            })
                                        } else {
                                            None
                                        }}
                                        {(!sortable).then(|| view! {
                                            <span>{column.header.clone()}</span>
                                        })}
                                        {resizable.then(|| view! {
                                            <span
                                                class="absolute top-0 right-0 z-10 h-full w-2 cursor-col-resize select-none opacity-0 hover:opacity-100 hover:bg-primary/50 active:opacity-100 active:bg-primary/70"
                                                role="separator"
                                                aria-orientation="vertical"
                                                aria-label=format!("Resize {} column", column.header)
                                                on:click=move |event: web_sys::MouseEvent| event.stop_propagation()
                                                on:pointerdown=move |event: web_sys::PointerEvent| {
                                                    event.stop_propagation();
                                                    let rendered_width = event
                                                        .target()
                                                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                                                        .and_then(|element| element.parent_element())
                                                        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
                                                        .map(|element| f64::from(element.offset_width()));
                                                    let start_width = rendered_width
                                                        .or_else(|| column_widths.with_untracked(|widths| {
                                                            widths.get(column_id).copied().map(f64::from)
                                                        }))
                                                        .unwrap_or_else(|| f64::from(minimum_width.unwrap_or(48)));
                                                    resize_drag.set(Some(ResizeDrag {
                                                        column_id: column_id.to_owned(),
                                                        start_x: f64::from(event.client_x()),
                                                        start_width,
                                                        minimum_width,
                                                    }));
                                                    if let Some(target) = event.target()
                                                        && let Ok(element) = target.dyn_into::<web_sys::Element>()
                                                    {
                                                        let _ = element.set_pointer_capture(event.pointer_id());
                                                    }
                                                }
                                                on:pointermove=move |event: web_sys::PointerEvent| {
                                                    if let Some(drag) = resize_drag.get_untracked() {
                                                        let requested = drag.start_width
                                                            + (f64::from(event.client_x()) - drag.start_x);
                                                        let mut scratch = EntityTablePreferences::new(
                                                            preference_version,
                                                        );
                                                        set_preferred_width(
                                                            &mut scratch,
                                                            drag.column_id.clone(),
                                                            requested,
                                                            drag.minimum_width,
                                                        );
                                                        if let Some(width) = scratch.column_widths.get(&drag.column_id) {
                                                            column_widths.update(|widths| {
                                                                widths.insert(drag.column_id.clone(), *width);
                                                            });
                                                        }
                                                    }
                                                }
                                                on:pointerup=move |event: web_sys::PointerEvent| {
                                                    finish_resize(
                                                        event.target(),
                                                        event.pointer_id(),
                                                        resize_drag,
                                                        column_widths,
                                                        preferences,
                                                    );
                                                }
                                                on:pointercancel=move |event: web_sys::PointerEvent| {
                                                    finish_resize(
                                                        event.target(),
                                                        event.pointer_id(),
                                                        resize_drag,
                                                        column_widths,
                                                        preferences,
                                                    );
                                                }
                                            ></span>
                                        })}
                                    </th>
                                }
                            }).collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = data.get();
                            let columns_for_sort = column_store.get_value();
                            let preferences_value = preferences.get();
                            let indices = sorted_index_cache
                                .try_update_value(|cache| {
                                    cache.indices(
                                        Rc::clone(&rows),
                                        &columns_for_sort,
                                        &preferences_value.sort,
                                    )
                                })
                                .expect("entity-table sort cache is still mounted");
                            let bounds = page_bounds(
                                current_page.get(),
                                preferences_value.page_size,
                                indices.len(),
                            );
                            let visible_columns = columns_for_sort
                                .iter()
                                .filter(|column| {
                                    !preferences_value.hidden_columns.contains(column.id)
                                })
                                .cloned()
                                .collect::<Vec<_>>();
                            let page_rows = indices[bounds]
                                .iter()
                                .map(|index| rows[*index].clone())
                                .collect::<Vec<_>>();

                            if page_rows.is_empty() {
                                return view! {
                                    <tr>
                                        <td
                                            colspan=visible_columns.len().max(1)
                                            class="py-10 text-center text-base-content/65"
                                        >
                                            {texts.with(|texts| texts.no_rows.clone())}
                                        </td>
                                    </tr>
                                }.into_any();
                            }

                            page_rows
                                .into_iter()
                                .map(|row| render_row(
                                    row,
                                    visible_columns.clone(),
                                    row_key.get_value(),
                                    compact_row.get_value(),
                                    on_row_activate,
                                ))
                                .collect_view()
                                .into_any()
                        }}
                    </tbody>
                </table>
            </div>

            <div class="flex flex-wrap items-center justify-between gap-3">
                <span class="text-sm text-base-content/75">
                    {move || {
                        let total = total_rows.get();
                        if total == 0 {
                            return String::new();
                        }
                        let page_size = preferences.with(|preferences| preferences.page_size);
                        let (start, end) = row_range(current_page.get(), page_size, total);
                        texts
                            .with(|texts| texts.row_range.clone())
                            .replace("{start}", &start.to_string())
                            .replace("{end}", &end.to_string())
                            .replace("{total}", &total.to_string())
                    }}
                </span>
                <Pagination class="flex items-center gap-1">
                    <Button
                        class="join-item btn-sm"
                        attr:data-entity-page="previous"
                        disabled=Signal::derive(move || current_page.get() == 0)
                        on_click=Callback::new(move |_| {
                            current_page.update(|page| *page = page.saturating_sub(1));
                        })
                    >
                        {move || texts.with(|texts| texts.previous.clone())}
                    </Button>
                    {move || page_window(current_page.get(), total_pages.get(), MAX_VISIBLE_PAGES)
                        .into_iter()
                        .map(|slot| match slot {
                            PageSlot::Page(page) => view! {
                                <Button
                                    class="join-item btn-sm"
                                    attr:data-entity-page=(page + 1).to_string()
                                    active=page == current_page.get()
                                    disabled=page == current_page.get()
                                    on_click=Callback::new(move |_| current_page.set(page))
                                >
                                    {(page + 1).to_string()}
                                </Button>
                            }.into_any(),
                            PageSlot::Ellipsis => view! {
                                <span class="join-item btn btn-sm btn-disabled" aria-hidden="true">"…"</span>
                            }.into_any(),
                        })
                        .collect_view()}
                    <Button
                        class="join-item btn-sm"
                        attr:data-entity-page="next"
                        disabled=Signal::derive(move || {
                            current_page.get() + 1 >= total_pages.get()
                        })
                        on_click=Callback::new(move |_| {
                            current_page.update(|page| {
                                *page = clamp_page(
                                    page.saturating_add(1),
                                    preferences.with(|preferences| preferences.page_size),
                                    total_rows.get_untracked(),
                                );
                            });
                        })
                    >
                        {move || texts.with(|texts| texts.next.clone())}
                    </Button>
                </Pagination>
            </div>
        </section>
    }
}

fn render_row<T: Clone + 'static>(
    row: T,
    columns: Vec<EntityColumn<T>>,
    row_key: EntityRowKey<T>,
    compact_row: Option<EntityRowRenderer<T>>,
    on_row_activate: Option<Callback<String>>,
) -> AnyView {
    let key = row_key(&row);
    let interactive = on_row_activate.is_some();
    let compact_view = compact_row
        .map(|renderer| renderer(&row))
        .unwrap_or_else(|| render_default_compact_row(&row, &columns));
    let wide_cells = columns
        .iter()
        .cloned()
        .map(|column| {
            let cell = render_cell(&row, &column);
            view! {
                <td
                    class="hidden lg:table-cell"
                    data-entity-action=column.is_action.then_some("true")
                    on:click=move |event| {
                        if column.is_action {
                            event.stop_propagation();
                        }
                    }
                    on:keydown=move |event| {
                        if column.is_action {
                            event.stop_propagation();
                        }
                    }
                >
                    {cell}
                </td>
            }
        })
        .collect_view();
    let click_key = key.clone();
    let keydown_key = key.clone();

    view! {
        <tr
            data-row-key=key
            tabindex=interactive.then_some(0)
            class=interactive.then_some("cursor-pointer ld-focus-ring")
            on:click=move |event: web_sys::MouseEvent| {
                if !event_origin_is_action(event.target())
                    && let Some(callback) = on_row_activate
                {
                    callback.run(click_key.clone());
                }
            }
            on:keydown=move |event: web_sys::KeyboardEvent| {
                if (event.key() == "Enter" || event.key() == " ")
                    && !event_origin_is_action(event.target())
                    && let Some(callback) = on_row_activate
                {
                    event.prevent_default();
                    callback.run(keydown_key.clone());
                }
            }
        >
            <td colspan=columns.len().max(1) class="p-0 lg:hidden">
                <div class="p-3">{compact_view}</div>
            </td>
            {wide_cells}
        </tr>
    }
    .into_any()
}

fn rendered_column_widths<T>(
    preferences: &EntityTablePreferences,
    columns: &[EntityColumn<T>],
) -> BTreeMap<String, u32> {
    let mut widths = preferences.column_widths.clone();
    for column in columns {
        if let Some(width) = column.initial_width {
            widths.entry(column.id.to_owned()).or_insert(width);
        }
    }
    widths
}

fn render_default_compact_row<T: 'static>(row: &T, columns: &[EntityColumn<T>]) -> AnyView {
    columns
        .iter()
        .cloned()
        .map(|column| {
            let cell = render_cell(row, &column);
            view! {
                <div
                    class="flex items-start justify-between gap-3 py-1"
                    data-entity-action=column.is_action.then_some("true")
                >
                    <span class="text-xs font-medium uppercase tracking-wide text-base-content/60">
                        {column.header}
                    </span>
                    <span class="min-w-0 text-right">{cell}</span>
                </div>
            }
        })
        .collect_view()
        .into_any()
}

fn render_cell<T: 'static>(row: &T, column: &EntityColumn<T>) -> AnyView {
    column
        .renderer
        .as_ref()
        .map(|renderer| renderer(row))
        .unwrap_or_else(|| (column.text)(row).into_any())
}

fn event_origin_is_action(target: Option<web_sys::EventTarget>) -> bool {
    target
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| {
            element
                .closest(
                    "button, a, input, select, textarea, [role='button'], [data-entity-action='true']",
                )
                .ok()
                .flatten()
        })
        .is_some()
}

fn finish_resize(
    target: Option<web_sys::EventTarget>,
    pointer_id: i32,
    resize_drag: RwSignal<Option<ResizeDrag>>,
    column_widths: RwSignal<BTreeMap<String, u32>>,
    preferences: RwSignal<EntityTablePreferences>,
) {
    if let Some(target) = target
        && let Ok(element) = target.dyn_into::<web_sys::Element>()
    {
        let _ = element.release_pointer_capture(pointer_id);
    }
    if let Some(drag) = resize_drag.get_untracked()
        && let Some(width) =
            column_widths.with_untracked(|widths| widths.get(&drag.column_id).copied())
    {
        preferences.update(|preferences| {
            preferences.column_widths.insert(drag.column_id, width);
        });
    }
    resize_drag.set(None);
}
