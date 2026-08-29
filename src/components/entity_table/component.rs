//! Reactive renderer for the typed client-side table model.

use super::model::{
    ENTITY_PAGE_SIZE_CHOICES, EntityColumnMove, SortedIndexCache,
    emit_normalized_preference_change, move_column, next_sort, next_sort_additive,
    normalize_preferences, ordered_columns, page_after_dataset_change, page_after_row_delta,
    reset_columns, reset_sort, set_preferred_width, toggle_hidden_column,
};
use super::storage::{load_preferences, save_preferences};
use super::types::{
    EntityColumn, EntityRowKey, EntityRowRenderer, EntitySort, EntitySortDirection,
    EntityTablePreferenceOwnership, EntityTablePreferencePersistence, EntityTablePreferences,
    EntityTableTexts,
};
use crate::components::button::Button;
use crate::components::data_table::{
    MAX_COLUMN_WIDTH, PageSlot, StableColumnTrack, StableTableColGroup, clamp_page,
    effective_min_width, keyboard_resized_width, page_bounds, page_count, page_window, row_range,
    stable_column_width, stable_table_content_style,
};
use crate::components::dropdown::{Dropdown, DropdownPlacement};
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

/// Send-safe header presentation split from an `EntityColumn<T>`'s local
/// `Rc` render/sort callbacks. Leptos's keyed `For` requires `Send` items;
/// keeping only header mechanics here lets the behavioral columns remain
/// deliberately local while sort-only preference changes preserve DOM nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct EntityHeaderDescriptor {
    id: &'static str,
    header: String,
    sortable: bool,
    resizable: bool,
    min_width: Option<u32>,
    initial_width: Option<u32>,
}

#[derive(Clone, Copy)]
enum PreferenceSource {
    Controlled {
        current: Signal<EntityTablePreferences>,
        on_change: Callback<EntityTablePreferences>,
    },
    Uncontrolled {
        current: RwSignal<EntityTablePreferences>,
        persistence: EntityTablePreferencePersistence,
    },
}

pub(super) struct PreferenceState<T: 'static> {
    source: PreferenceSource,
    columns: StoredValue<Vec<EntityColumn<T>>, LocalStorage>,
    schema_version: u16,
}

impl<T: 'static> Clone for PreferenceState<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for PreferenceState<T> {}

impl<T: 'static> PreferenceState<T> {
    pub(super) fn new(
        ownership: EntityTablePreferenceOwnership,
        columns: StoredValue<Vec<EntityColumn<T>>, LocalStorage>,
        schema_version: u16,
    ) -> Self {
        let source = match ownership {
            EntityTablePreferenceOwnership::Controlled { current, on_change } => {
                PreferenceSource::Controlled { current, on_change }
            }
            EntityTablePreferenceOwnership::Uncontrolled { persistence } => {
                let initial = columns
                    .with_value(|columns| load_preferences(persistence, schema_version, columns));
                PreferenceSource::Uncontrolled {
                    current: RwSignal::new(initial),
                    persistence,
                }
            }
        };
        Self {
            source,
            columns,
            schema_version,
        }
    }

    pub(super) fn get(self) -> EntityTablePreferences {
        let current = match self.source {
            PreferenceSource::Controlled { current, .. } => current.get(),
            PreferenceSource::Uncontrolled { current, .. } => current.get(),
        };
        self.columns
            .with_value(|columns| normalize_preferences(&current, self.schema_version, columns))
    }

    fn get_untracked(self) -> EntityTablePreferences {
        let current = match self.source {
            PreferenceSource::Controlled { current, .. } => current.get_untracked(),
            PreferenceSource::Uncontrolled { current, .. } => current.get_untracked(),
        };
        self.columns
            .with_value(|columns| normalize_preferences(&current, self.schema_version, columns))
    }

    fn with_untracked<R>(self, read: impl FnOnce(&EntityTablePreferences) -> R) -> R {
        read(&self.get_untracked())
    }

    fn with<R>(self, read: impl FnOnce(&EntityTablePreferences) -> R) -> R {
        read(&self.get())
    }

    fn rendered_widths(self) -> BTreeMap<String, u32> {
        let current = self.get_untracked();
        self.columns
            .with_value(|columns| rendered_column_widths(&current, columns))
    }

    pub(super) fn update_and_rendered_widths(
        self,
        update: impl FnOnce(&mut EntityTablePreferences),
    ) -> BTreeMap<String, u32> {
        self.update(update);
        self.rendered_widths()
    }

    pub(super) fn update(
        self,
        update: impl FnOnce(&mut EntityTablePreferences),
    ) -> EntityTablePreferences {
        let current = self.get_untracked();
        self.columns.with_value(|columns| {
            emit_normalized_preference_change(
                &current,
                self.schema_version,
                columns,
                update,
                |replacement| match self.source {
                    PreferenceSource::Controlled { on_change, .. } => on_change.run(replacement),
                    PreferenceSource::Uncontrolled { current, .. } => current.set(replacement),
                },
            )
        })
    }
}

pub(super) struct DatasetTransitionController<T: 'static> {
    current_page: RwSignal<usize>,
    preferences: PreferenceState<T>,
}

impl<T: 'static> Clone for DatasetTransitionController<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for DatasetTransitionController<T> {}

impl<T: 'static> DatasetTransitionController<T> {
    pub(super) const fn new(
        current_page: RwSignal<usize>,
        preferences: PreferenceState<T>,
    ) -> Self {
        Self {
            current_page,
            preferences,
        }
    }

    pub(super) fn apply(self, previous_dataset: String, next_dataset: String) {
        let supplied_preferences = self.preferences.get_untracked();
        let next_page = page_after_dataset_change(
            self.current_page.get_untracked(),
            previous_dataset,
            next_dataset,
        );
        self.current_page.set(next_page);
        debug_assert_eq!(
            self.preferences.get_untracked(),
            supplied_preferences,
            "dataset changes must preserve supplied EntityTable preferences"
        );
    }
}

pub(super) fn apply_page_size_change<T: 'static>(
    preferences: PreferenceState<T>,
    current_page: RwSignal<usize>,
    requested_value: &str,
    reassert_live_value: impl FnOnce(String),
) {
    if let Ok(page_size) = requested_value.parse::<usize>()
        && ENTITY_PAGE_SIZE_CHOICES.contains(&page_size)
    {
        preferences.update(|preferences| preferences.page_size = page_size);
        current_page.set(0);
    }

    let supplied_value =
        preferences.with_untracked(|preferences| preferences.page_size.to_string());
    reassert_live_value(supplied_value);
}

pub(super) fn resolve_preference_ownership(
    explicit: Option<EntityTablePreferenceOwnership>,
    legacy_storage_key: Option<&'static str>,
) -> EntityTablePreferenceOwnership {
    match (explicit, legacy_storage_key) {
        (Some(_), Some(_)) => {
            panic!("EntityTable configuration cannot combine preference_ownership with storage_key")
        }
        (Some(ownership), None) => ownership,
        (None, Some(storage_key)) => EntityTablePreferenceOwnership::Uncontrolled {
            persistence: EntityTablePreferencePersistence::LegacyLocalStorage { storage_key },
        },
        (None, None) => EntityTablePreferenceOwnership::Uncontrolled {
            persistence: EntityTablePreferencePersistence::Disabled,
        },
    }
}

/// A typed, client-side table for complete dataset snapshots.
///
/// Ordering is represented as an index permutation, so source data and row
/// identity are never mutated. Only rows on the current page are cloned for
/// rendering. Wide and compact layouts share the same `<tr>` nodes, preventing
/// hidden duplicate pages in the DOM. Wide tables use stable declared tracks,
/// a semantic dark-blue header, and a faint full-cell grid. Sorting updates the
/// body order and sort metadata without replacing header nodes or moving the
/// table shell; a non-resizable utility column absorbs spare full-width space.
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
    ///
    /// This compatibility prop selects `LegacyLocalStorage` when
    /// `preference_ownership` is omitted. Supplying both is a configuration
    /// error so controlled ownership can never silently perform browser I/O.
    #[prop(optional)]
    storage_key: Option<&'static str>,
    /// Typed preference ownership. Controlled mode performs no component I/O.
    #[prop(optional)]
    preference_ownership: Option<EntityTablePreferenceOwnership>,
    /// Consumer-controlled preference schema version.
    #[prop(default = 1)]
    preference_version: u16,
    /// Localizable labels for table controls.
    #[prop(into, default = Signal::stored(EntityTableTexts::default()))]
    texts: Signal<EntityTableTexts>,
    /// Shows separate reset-sort and reset-columns actions.
    #[prop(optional, default = false)]
    show_reset_actions: bool,
    /// Enable alternating body-row striping. The opinionated default is a
    /// clean faint grid without zebra banding.
    #[prop(optional, into)]
    zebra: Signal<bool>,
    /// Additional outer-container classes.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView
where
    T: Clone + 'static,
{
    let column_store = StoredValue::new_local(columns);
    let preference_ownership = resolve_preference_ownership(preference_ownership, storage_key);
    let preferences = PreferenceState::new(preference_ownership, column_store, preference_version);
    let initial_widths = column_store
        .with_value(|columns| rendered_column_widths(&preferences.get_untracked(), columns));
    let row_key = StoredValue::new_local(row_key);
    let compact_row = StoredValue::new_local(compact_row);
    let sorted_index_cache = StoredValue::new_local(SortedIndexCache::new());
    let column_widths = RwSignal::new(initial_widths);
    let header_descriptors =
        RwSignal::new(column_store.with_value(|columns| {
            entity_header_descriptors(&preferences.get_untracked(), columns)
        }));
    let current_page = RwSignal::new(0_usize);
    let previous_dataset = StoredValue::new(dataset_identity.get_untracked());
    let resize_drag = RwSignal::new(Option::<ResizeDrag>::None);
    let dataset_transition = DatasetTransitionController::new(current_page, preferences);
    let page_size_select = NodeRef::<leptos::html::Select>::new();

    Effect::new(move |_| {
        let next_dataset = dataset_identity.get();
        let previous = previous_dataset.get_value();
        dataset_transition.apply(previous, next_dataset.clone());
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

    if let PreferenceSource::Uncontrolled {
        current,
        persistence,
    } = preferences.source
    {
        Effect::new(move |_| {
            current.with(|preferences| save_preferences(persistence, preferences));
        });
    }

    Effect::new(move |_| {
        let next_widths = column_store.with_value(|columns| {
            preferences.with(|preferences| rendered_column_widths(preferences, columns))
        });
        if next_widths != column_widths.get_untracked() {
            column_widths.set(next_widths);
        }
    });

    Effect::new(move |_| {
        let next = column_store.with_value(|columns| {
            preferences.with(|preferences| entity_header_descriptors(preferences, columns))
        });
        if next != header_descriptors.get_untracked() {
            header_descriptors.set(next);
        }
    });

    let flexible_column_id = Signal::derive(move || {
        header_descriptors.with(|columns| entity_flexible_column_id(columns))
    });
    let stable_tracks = Signal::derive(move || {
        let widths = column_widths.get();
        header_descriptors
            .get()
            .into_iter()
            .map(|column| {
                let track = StableColumnTrack::new(
                    column.id,
                    widths.get(column.id).copied().unwrap_or_else(|| {
                        stable_column_width(None, column.initial_width.or(column.min_width))
                    }),
                );
                if flexible_column_id.get() == Some(column.id) {
                    track.flexible()
                } else {
                    track
                }
            })
            .collect::<Vec<_>>()
    });

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
            data-table-data-mode="client-snapshot"
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
                        node_ref=page_size_select
                        on_change=Callback::new(move |value: String| {
                            apply_page_size_change(
                                preferences,
                                current_page,
                                &value,
                                move |supplied_value| {
                                    if let Some(select) = page_size_select.get() {
                                        select.set_value(&supplied_value);
                                    }
                                },
                            );
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
                    <div class="dropdown-content bg-base-100 rounded-box z-[2] w-72 p-0 shadow-lg border border-base-300">
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
                        <div class="border-t border-base-300 p-2">
                            <p class="px-2 pb-1 text-xs font-semibold text-base-content/65">
                                "Column order"
                            </p>
                            <ol class="space-y-1" aria-label="Column order">
                                <For
                                    each=move || column_store.with_value(|columns| {
                                        ordered_columns(&preferences.get(), columns)
                                            .into_iter()
                                            .map(|column| (column.id, column.header))
                                            .collect::<Vec<_>>()
                                    })
                                    key=|column| column.0
                                    children=move |(column_id, header)| {
                                        let earlier_label = header.clone();
                                        let later_label = header.clone();
                                        view! {
                                            <li
                                                class="flex items-center gap-1 rounded-field px-2 py-1"
                                                data-entity-column-order=column_id
                                            >
                                                <span class="min-w-0 flex-1 truncate text-sm">{header}</span>
                                                <Button
                                                    class="btn-ghost btn-xs btn-square"
                                                    attr:data-entity-column-order=column_id
                                                    attr:data-entity-column-move="earlier"
                                                    attr:aria-label=move || {
                                                        let (position, total) = preferences.with(|preferences| {
                                                            (
                                                                preferences
                                                                    .column_order
                                                                    .iter()
                                                                    .position(|id| id == column_id)
                                                                    .map(|index| index + 1)
                                                                    .unwrap_or(1),
                                                                preferences.column_order.len(),
                                                            )
                                                        });
                                                        format!("Move {earlier_label} earlier from position {position} of {total}")
                                                    }
                                                    disabled=Signal::derive(move || {
                                                        preferences.with(|preferences| {
                                                            preferences.column_order.first().is_some_and(|id| id == column_id)
                                                        })
                                                    })
                                                    on_click=Callback::new(move |event: web_sys::MouseEvent| {
                                                        restore_column_move_focus(
                                                            event,
                                                            column_id,
                                                            EntityColumnMove::Earlier,
                                                        );
                                                        column_store.with_value(|columns| {
                                                            preferences.update(|preferences| {
                                                                move_column(
                                                                    preferences,
                                                                    columns,
                                                                    column_id,
                                                                    EntityColumnMove::Earlier,
                                                                );
                                                            });
                                                        });
                                                    })
                                                >
                                                    <span aria-hidden="true">"↑"</span>
                                                </Button>
                                                <Button
                                                    class="btn-ghost btn-xs btn-square"
                                                    attr:data-entity-column-order=column_id
                                                    attr:data-entity-column-move="later"
                                                    attr:aria-label=move || {
                                                        let (position, total) = preferences.with(|preferences| {
                                                            (
                                                                preferences
                                                                    .column_order
                                                                    .iter()
                                                                    .position(|id| id == column_id)
                                                                    .map(|index| index + 1)
                                                                    .unwrap_or(1),
                                                                preferences.column_order.len(),
                                                            )
                                                        });
                                                        format!("Move {later_label} later from position {position} of {total}")
                                                    }
                                                    disabled=Signal::derive(move || {
                                                        preferences.with(|preferences| {
                                                            preferences.column_order.last().is_some_and(|id| id == column_id)
                                                        })
                                                    })
                                                    on_click=Callback::new(move |event: web_sys::MouseEvent| {
                                                        restore_column_move_focus(
                                                            event,
                                                            column_id,
                                                            EntityColumnMove::Later,
                                                        );
                                                        column_store.with_value(|columns| {
                                                            preferences.update(|preferences| {
                                                                move_column(
                                                                    preferences,
                                                                    columns,
                                                                    column_id,
                                                                    EntityColumnMove::Later,
                                                                );
                                                            });
                                                        });
                                                    })
                                                >
                                                    <span aria-hidden="true">"↓"</span>
                                                </Button>
                                            </li>
                                        }
                                    }
                                />
                            </ol>
                        </div>
                    </div>
                </Dropdown>

                {show_reset_actions.then(|| view! {
                    <Button
                        class="btn-ghost btn-sm"
                        attr:data-entity-reset-sort="true"
                        disabled=Signal::derive(move || {
                            preferences.with(|preferences| preferences.sort.is_system())
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
                                && column_store.with_value(|columns| {
                                    preferences
                                        .column_order
                                        .iter()
                                        .map(String::as_str)
                                        .eq(columns.iter().map(|column| column.id))
                                })
                        }))
                        on_click=Callback::new(move |_| {
                            column_widths.set(
                                preferences.update_and_rendered_widths(|preferences| {
                                    reset_columns(preferences);
                                }),
                            );
                        })
                    >
                        {move || texts.with(|texts| texts.reset_columns.clone())}
                    </Button>
                })}
            </div>

            <p class="sr-only" aria-live="polite" data-entity-sort-summary="true">
                {move || column_store.with_value(|columns| {
                    preferences.with(|preferences| sort_summary(&preferences.sort, columns))
                })}
            </p>

            <div class="w-full overflow-x-auto rounded-box border border-table-grid bg-base-100">
                <div style=move || stable_table_content_style(&stable_tracks.get())>
                <table
                    class="table table-sm table-fixed w-full border-collapse border border-table-grid"
                    class:table-zebra=move || zebra.get()
                    data-entity-table-grid="true"
                    data-table-layout="stable"
                >
                    <StableTableColGroup tracks=stable_tracks />
                    <thead class="hidden lg:table-header-group">
                        <tr>
                            <For
                                each=move || header_descriptors.get()
                                key=|column| (
                                    column.id,
                                    column.header.clone(),
                                    column.sortable,
                                    column.resizable,
                                    column.min_width,
                                    column.initial_width,
                                )
                                children=move |column| {
                                let column_id = column.id;
                                let header = column.header.clone();
                                let sortable = column.sortable;
                                let resizable = column.resizable;
                                let minimum_width = column.min_width;
                                let minimum_value = effective_min_width(minimum_width);
                                let width_style = move || {
                                    if flexible_column_id.get() == Some(column_id) {
                                        return minimum_width.map(|_| {
                                            format!("min-width: {}px", minimum_value.round())
                                        });
                                    }
                                    column_widths
                                        .with(|widths| widths.get(column_id).copied())
                                        .map(|width| format!(
                                            "width: {width}px; min-width: {width}px; max-width: {width}px"
                                        ))
                                        .or_else(|| {
                                            minimum_width.map(|_| {
                                                format!("min-width: {}px", minimum_value.round())
                                            })
                                        })
                                };
                                let sort_label = move || preferences.with(|preferences| {
                                    format!(
                                        "{}: {}. {}. {}.",
                                        header,
                                        preferences.sort.current_label(column_id),
                                        preferences.sort.plain_action_label(column_id),
                                        preferences.sort.additive_action_label(column_id),
                                    )
                                });
                                view! {
                                    <th
                                        class="relative border border-table-grid bg-table-header text-table-header-content forced-colors:border-[CanvasText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                                        scope="col"
                                        data-entity-column=column_id
                                        aria-sort=move || preferences.with(|preferences| {
                                            preferences.sort.aria_value_for(column_id)
                                        })
                                        data-entity-sort-priority=move || preferences.with(|preferences| {
                                            preferences.sort.priority_for(column_id).map(|priority| priority.to_string())
                                        })
                                        data-entity-sort-direction=move || preferences.with(|preferences| {
                                            preferences.sort.direction_for(column_id).map(|direction| {
                                                direction.aria_value()
                                            })
                                        })
                                        style=width_style
                                    >
                                        {if sortable {
                                            let header = column.header.clone();
                                            Some(view! {
                                                <Button
                                                    class="btn-ghost btn-xs h-auto !min-h-0 w-full justify-start gap-1 rounded-sm px-0 py-1 text-left font-semibold text-table-header-content !shadow-none hover:bg-white/15 focus-visible:outline-white forced-colors:text-[CanvasText]"
                                                    attr:data-entity-sort-column=column_id
                                                    attr:aria-label=sort_label
                                                    on:keydown=move |event: web_sys::KeyboardEvent| {
                                                        if !event.shift_key()
                                                            || !matches!(event.key().as_str(), "Enter" | " " | "Spacebar")
                                                        {
                                                            return;
                                                        }
                                                        event.prevent_default();
                                                        event.stop_propagation();
                                                        preferences.update(|preferences| {
                                                            preferences.sort = next_sort_additive(
                                                                &preferences.sort,
                                                                column_id,
                                                                true,
                                                            );
                                                        });
                                                        current_page.set(0);
                                                    }
                                                    on_click=Callback::new(move |event: web_sys::MouseEvent| {
                                                        preferences.update(|preferences| {
                                                            preferences.sort = if event.shift_key() {
                                                                next_sort_additive(
                                                                    &preferences.sort,
                                                                    column_id,
                                                                    true,
                                                                )
                                                            } else {
                                                                next_sort(
                                                                    &preferences.sort,
                                                                    column_id,
                                                                    true,
                                                                )
                                                            };
                                                        });
                                                        current_page.set(0);
                                                    })
                                                >
                                                    <span>{header}</span>
                                                    <span
                                                        aria-hidden="true"
                                                        data-entity-sort-indicator="true"
                                                        class="inline-flex w-6 shrink-0 justify-center text-xs"
                                                    >
                                                        {move || preferences.with(|preferences| {
                                                            let Some(direction) = preferences.sort.direction_for(column_id) else {
                                                                return "↕".to_owned();
                                                            };
                                                            let marker = match direction {
                                                                EntitySortDirection::Ascending => "▲",
                                                                EntitySortDirection::Descending => "▼",
                                                            };
                                                            format!(
                                                                "{marker}{}",
                                                                preferences.sort.priority_for(column_id).unwrap_or(1)
                                                            )
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
                                                class="absolute top-0 right-0 z-10 h-full w-2 cursor-col-resize select-none opacity-0 hover:opacity-100 hover:bg-primary/50 focus:opacity-100 focus:bg-primary/50 focus:outline focus:outline-2 focus:outline-primary active:opacity-100 active:bg-primary/70"
                                                role="separator"
                                                tabindex="0"
                                                aria-orientation="vertical"
                                                aria-label=format!("Resize {} column", column.header)
                                                aria-valuemin=minimum_value.round() as u32
                                                aria-valuemax=MAX_COLUMN_WIDTH.round() as u32
                                                aria-valuenow=move || column_widths.with(|widths| {
                                                    widths
                                                        .get(column_id)
                                                        .copied()
                                                        .unwrap_or_else(|| {
                                                            minimum_value.round() as u32
                                                        })
                                                })
                                                aria-valuetext=move || column_widths.with(|widths| {
                                                    format!(
                                                        "{} pixels",
                                                        widths
                                                            .get(column_id)
                                                            .copied()
                                                            .unwrap_or_else(|| {
                                                                minimum_value.round() as u32
                                                            })
                                                    )
                                                })
                                                on:click=move |event: web_sys::MouseEvent| event.stop_propagation()
                                                on:focus=move |event: web_sys::FocusEvent| {
                                                    if let Some(rendered_width) = separator_parent_width(event.target()) {
                                                        let width = rendered_width
                                                            .clamp(minimum_value, MAX_COLUMN_WIDTH)
                                                            .round() as u32;
                                                        column_widths.update(|widths| {
                                                            widths.insert(column_id.to_owned(), width);
                                                        });
                                                    }
                                                }
                                                on:keydown=move |event: web_sys::KeyboardEvent| {
                                                    let current_width = separator_parent_width(
                                                        event.current_target().or_else(|| event.target()),
                                                    )
                                                    .or_else(|| column_widths.with_untracked(|widths| {
                                                        widths.get(column_id).copied().map(f64::from)
                                                    }))
                                                    .unwrap_or(minimum_value);
                                                    let Some(requested_width) = keyboard_resized_width(
                                                        current_width,
                                                        &event.key(),
                                                        minimum_value,
                                                    ) else {
                                                        return;
                                                    };
                                                    event.prevent_default();
                                                    event.stop_propagation();
                                                    column_widths.set(
                                                        preferences.update_and_rendered_widths(|preferences| {
                                                            set_preferred_width(
                                                                preferences,
                                                                column_id,
                                                                requested_width,
                                                                minimum_width,
                                                            );
                                                        }),
                                                    );
                                                }
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
                                }
                            />
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
                            let visible_columns = ordered_columns(
                                &preferences_value,
                                &columns_for_sort,
                            )
                                .into_iter()
                                .filter(|column| {
                                    !preferences_value.hidden_columns.contains(column.id)
                                })
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
                                            class="border border-table-grid py-10 text-center text-base-content/65 forced-colors:border-[CanvasText]"
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
                    class="hidden border border-table-grid forced-colors:border-[CanvasText] lg:table-cell"
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
            <td
                colspan=columns.len().max(1)
                class="border border-table-grid p-0 forced-colors:border-[CanvasText] lg:hidden"
            >
                <div class="p-3">{compact_view}</div>
            </td>
            {wide_cells}
        </tr>
    }
    .into_any()
}

fn sort_summary<T>(sort: &EntitySort, columns: &[EntityColumn<T>]) -> String {
    if sort.is_system() {
        return "System order".to_owned();
    }
    let clauses = sort
        .clauses()
        .iter()
        .enumerate()
        .filter_map(|(index, clause)| {
            let column = columns.iter().find(|column| column.id == clause.column)?;
            let direction = match clause.direction {
                EntitySortDirection::Ascending => "ascending",
                EntitySortDirection::Descending => "descending",
            };
            Some(format!(
                "priority {}: {} {direction}",
                index + 1,
                column.header
            ))
        })
        .collect::<Vec<_>>();
    format!("Sorted by {}", clauses.join(", then "))
}

fn entity_header_descriptors<T>(
    preferences: &EntityTablePreferences,
    columns: &[EntityColumn<T>],
) -> Vec<EntityHeaderDescriptor> {
    ordered_columns(preferences, columns)
        .into_iter()
        .filter(|column| !preferences.hidden_columns.contains(column.id))
        .map(|column| EntityHeaderDescriptor {
            id: column.id,
            header: column.header,
            sortable: column.sortable,
            resizable: column.resizable,
            min_width: column.min_width,
            initial_width: column.initial_width,
        })
        .collect()
}

fn entity_flexible_column_id(columns: &[EntityHeaderDescriptor]) -> Option<&'static str> {
    columns
        .iter()
        .rev()
        .find(|column| !column.resizable)
        .map(|column| column.id)
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

fn separator_parent_width(target: Option<web_sys::EventTarget>) -> Option<f64> {
    target
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| element.parent_element())
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        .map(|element| f64::from(element.offset_width()))
}

fn restore_column_move_focus(
    event: web_sys::MouseEvent,
    column_id: &'static str,
    direction: EntityColumnMove,
) {
    let Some(root) = event
        .target()
        .or_else(|| event.current_target())
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| element.closest("[data-entity-table]").ok().flatten())
    else {
        return;
    };
    if let Ok(Some(anchor)) = root.query_selector("[data-entity-column-chooser]")
        && let Ok(anchor) = anchor.dyn_into::<web_sys::HtmlElement>()
    {
        let _ = anchor.focus();
    }
    let preferred_direction = match direction {
        EntityColumnMove::Earlier => "earlier",
        EntityColumnMove::Later => "later",
    };
    let fallback_direction = match direction {
        EntityColumnMove::Earlier => "later",
        EntityColumnMove::Later => "earlier",
    };
    request_animation_frame(move || {
        let Ok(nodes) = root.query_selector_all("[data-entity-column-move]") else {
            return;
        };
        for direction in [preferred_direction, fallback_direction] {
            for index in 0..nodes.length() {
                let Some(node) = nodes.item(index) else {
                    continue;
                };
                let Ok(element) = node.dyn_into::<web_sys::Element>() else {
                    continue;
                };
                if element.get_attribute("data-entity-column-order").as_deref() == Some(column_id)
                    && element.get_attribute("data-entity-column-move").as_deref()
                        == Some(direction)
                    && !element.has_attribute("disabled")
                    && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
                {
                    let _ = element.focus();
                    return;
                }
            }
        }
    });
}

fn finish_resize<T: 'static>(
    target: Option<web_sys::EventTarget>,
    pointer_id: i32,
    resize_drag: RwSignal<Option<ResizeDrag>>,
    column_widths: RwSignal<BTreeMap<String, u32>>,
    preferences: PreferenceState<T>,
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
        column_widths.set(preferences.update_and_rendered_widths(|preferences| {
            preferences.column_widths.insert(drag.column_id, width);
        }));
    }
    resize_drag.set(None);
}
