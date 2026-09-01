use crate::debug_state;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::patterns::*;
use serde_json::json;
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Clone, Default, PartialEq)]
pub(crate) struct DemoFilters {
    search: String,
    status: String,
    case_type: String,
}

page_contract! {
    pub(crate) CLIENT_SNAPSHOT_DEMO_PAGE {
        id: "client-snapshot-demo",
        route: "/components/client-snapshot-list",
        pattern: PagePattern::ClientSnapshotList,
        dataset: DatasetBehavior::SelectorTriggersLoad { key: "office" },
        local_state: ["search", "status", "case_type", "sort", "page_size", "columns"],
        required_states: [
            InitialLoading, Ready, Revalidating, InitialError, RefreshError,
            NeverLoaded, Empty, FilteredEmpty, Stale, Claiming, ClaimSucceeded,
            ClaimConflict, ClaimFailed, LiveInterrupted
        ],
        breakpoints: [Compact, Wide],
    }
}

filter_schema! {
    pub(crate) CLIENT_SNAPSHOT_DEMO_FILTERS: DemoFilters {
        dataset_selector: "office",
        filters: [status, case_type],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DemoRow {
    id: String,
    client: String,
    status: String,
    case_type: String,
    received: String,
}

fn demo_rows(office: &str) -> Vec<DemoRow> {
    let office_label = if office == "office-in" {
        "Delhi"
    } else {
        "Mexico City"
    };
    (0..72)
        .map(|index| DemoRow {
            id: format!("{office}-{index:03}"),
            client: format!("{office_label} Client {:03}", index + 1),
            status: if index % 3 == 0 { "Urgent" } else { "Ready" }.to_owned(),
            case_type: if index % 2 == 0 {
                "Family"
            } else {
                "Humanitarian"
            }
            .to_owned(),
            received: format!("2026-08-{:02}", (index % 24) + 1),
        })
        .collect()
}

fn demo_columns(
    claim_count: RwSignal<usize>,
    retained_action_count: RwSignal<usize>,
    removed_rows: RwSignal<BTreeSet<String>>,
    spanish: bool,
) -> Vec<EntityColumn<DemoRow>> {
    let client = if spanish { "Cliente" } else { "Client" };
    let status = if spanish { "Estado" } else { "Status" };
    let case_type = if spanish { "Tipo de caso" } else { "Case type" };
    let received = if spanish { "Recibido" } else { "Received" };
    let actions = if spanish { "Acciones" } else { "Actions" };
    let claim = if spanish { "Reclamar" } else { "Claim" };
    let retain = if spanish { "Conservar" } else { "Keep" };
    vec![
        EntityColumn::text("client", client, |row: &DemoRow| row.client.clone())
            .required()
            .with_min_width(240),
        EntityColumn::text("status", status, |row: &DemoRow| row.status.clone())
            .with_min_width(110),
        EntityColumn::text("case_type", case_type, |row: &DemoRow| {
            row.case_type.clone()
        })
        .with_min_width(150),
        EntityColumn::text("received", received, |row: &DemoRow| row.received.clone())
            .with_min_width(125),
        EntityColumn::action("actions", actions, move |_: &DemoRow| claim.to_owned())
            .required()
            .non_resizable()
            .render_with(move |row| {
                let claim_row_id = row.id.clone();
                let retain_row_id = row.id.clone();
                let remove_id = row.id.clone();
                view! {
                    <div class="flex flex-wrap gap-1">
                        <EntityRowAction action_id="claim">
                            <Button
                                class="btn-primary btn-xs"
                                attr:data-claim-row=claim_row_id
                                on_click=Callback::new(move |_| {
                                    removed_rows.update(|rows| {
                                        rows.insert(remove_id.clone());
                                    });
                                    claim_count.update(|count| *count += 1);
                                })
                            >
                                {claim}
                            </Button>
                        </EntityRowAction>
                        <EntityRowAction action_id="retain">
                            <Button
                                class="btn-ghost btn-xs"
                                attr:data-retain-row=retain_row_id
                                on_click=Callback::new(move |_| {
                                    retained_action_count.update(|count| *count += 1);
                                })
                            >
                                {retain}
                            </Button>
                        </EntityRowAction>
                    </div>
                }
                .into_any()
            }),
    ]
}

#[component]
pub fn ClientSnapshotListDemo() -> impl IntoView {
    debug_assert!(CLIENT_SNAPSHOT_DEMO_PAGE.validate().is_ok());
    debug_assert!(CLIENT_SNAPSHOT_DEMO_FILTERS.validate().is_ok());

    let office = RwSignal::new("office-mx".to_owned());
    let search = RwSignal::new(String::new());
    let status = RwSignal::new(String::new());
    let case_type = RwSignal::new(String::new());
    let retained_state = RwSignal::new(PageState::Ready);
    let claim_count = RwSignal::new(0_usize);
    let retained_action_count = RwSignal::new(0_usize);
    let removed_rows = RwSignal::new(BTreeSet::<String>::new());
    let activate_count = RwSignal::new(0_usize);
    let toolbar_action_count = RwSignal::new(0_usize);
    let display_projection = RwSignal::new(EntityTableDisplayProjection::default());
    let exported_current_rows = RwSignal::new(0_usize);
    let exported_all_rows = RwSignal::new(0_usize);
    let exported_first_key = RwSignal::new(String::new());
    let last_activated = RwSignal::new(String::new());
    let spanish = RwSignal::new(false);
    let access_generation = RwSignal::new(0_u64);
    let save_count = RwSignal::new(0_usize);
    let save_state = RwSignal::new(SnapshotDefaultSaveState::Clean);
    let mut initial_preferences = EntityTablePreferences::new(1);
    initial_preferences.column_order = ["client", "status", "case_type", "received", "actions"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let table_preferences = RwSignal::new(initial_preferences);
    let preference_ownership = EntityTablePreferenceOwnership::controlled(
        table_preferences.into(),
        Callback::new(move |replacement| table_preferences.set(replacement)),
    );

    Effect::new(move |_| {
        debug_state::set("entity_table.preferences", table_preferences.get());
    });
    on_cleanup(move || debug_state::remove("entity_table.preferences"));
    Effect::new(move |_| {
        debug_state::set("entity_table.display_projection", display_projection.get());
    });
    on_cleanup(move || debug_state::remove("entity_table.display_projection"));

    let snapshot = Signal::derive_local(move || {
        let removed = removed_rows.get();
        Rc::new(
            demo_rows(&office.get())
                .into_iter()
                .filter(|row| !removed.contains(&row.id))
                .collect::<Vec<_>>(),
        )
    });
    let filtered = Signal::derive_local(move || {
        let query = search.get().trim().to_lowercase();
        let status_filter = status.get();
        let case_filter = case_type.get();
        Rc::new(
            snapshot
                .get()
                .iter()
                .filter(|row| {
                    (query.is_empty()
                        || row.client.to_lowercase().contains(&query)
                        || row.id.to_lowercase().contains(&query))
                        && (status_filter.is_empty() || row.status == status_filter)
                        && (case_filter.is_empty() || row.case_type == case_filter)
                })
                .cloned()
                .collect::<Vec<_>>(),
        )
    });

    let active_chips = Signal::derive(move || {
        let (search_label, status_label, case_label) = if spanish.get() {
            ("Buscar", "Estado", "Tipo de caso")
        } else {
            ("Search", "Status", "Case type")
        };
        let mut chips = Vec::new();
        if !search.get().is_empty() {
            chips.push(ActiveFilterChip::new("search", search_label, search.get()));
        }
        if !status.get().is_empty() {
            chips.push(ActiveFilterChip::new("status", status_label, status.get()));
        }
        if !case_type.get().is_empty() {
            chips.push(ActiveFilterChip::new(
                "case_type",
                case_label,
                case_type.get(),
            ));
        }
        chips
    });

    let remove_filter = Callback::new(move |id: String| match id.as_str() {
        "search" => search.set(String::new()),
        "status" => status.set(String::new()),
        "case_type" => case_type.set(String::new()),
        _ => {}
    });
    let clear_filters = Callback::new(move |_: ()| {
        search.set(String::new());
        status.set(String::new());
        case_type.set(String::new());
    });

    let columns = Signal::derive_local(move || {
        demo_columns(
            claim_count,
            retained_action_count,
            removed_rows,
            spanish.get(),
        )
    });
    let column_filter_definitions = StoredValue::new_local(vec![
        EntityColumnFilter::text(
            "client",
            "entity-client-filter",
            Signal::derive(move || {
                if spanish.get() {
                    "Cliente".to_owned()
                } else {
                    "Client".to_owned()
                }
            }),
            search,
            Signal::derive(move || {
                if spanish.get() {
                    "Filtrar clientes".to_owned()
                } else {
                    "Filter clients".to_owned()
                }
            }),
            Callback::new(move |next| search.set(next)),
        ),
        EntityColumnFilter::select(
            "status",
            "entity-status-filter",
            Signal::derive(move || {
                if spanish.get() {
                    "Estado".to_owned()
                } else {
                    "Status".to_owned()
                }
            }),
            status,
            Signal::derive(move || {
                if spanish.get() {
                    "Todos los estados".to_owned()
                } else {
                    "All statuses".to_owned()
                }
            }),
            Signal::derive(move || {
                if spanish.get() {
                    vec![
                        EntityColumnFilterOption::new("Ready", "Listo"),
                        EntityColumnFilterOption::new("Urgent", "Urgente"),
                        EntityColumnFilterOption::new("Rejected", "Rechazado"),
                    ]
                } else {
                    vec![
                        EntityColumnFilterOption::new("Ready", "Ready"),
                        EntityColumnFilterOption::new("Urgent", "Urgent"),
                        EntityColumnFilterOption::new("Rejected", "Rejected proposal"),
                    ]
                }
            }),
            Callback::new(move |next| {
                if next != "Rejected" {
                    status.set(next);
                }
            }),
        ),
        EntityColumnFilter::select(
            "case_type",
            "entity-case-filter",
            Signal::derive(move || {
                if spanish.get() {
                    "Tipo de caso".to_owned()
                } else {
                    "Case type".to_owned()
                }
            }),
            case_type,
            Signal::derive(move || {
                if spanish.get() {
                    "Todos los tipos".to_owned()
                } else {
                    "All case types".to_owned()
                }
            }),
            Signal::derive(move || {
                if spanish.get() {
                    vec![
                        EntityColumnFilterOption::new("Family", "Familia"),
                        EntityColumnFilterOption::new("Humanitarian", "Humanitario"),
                    ]
                } else {
                    vec![
                        EntityColumnFilterOption::new("Family", "Family"),
                        EntityColumnFilterOption::new("Humanitarian", "Humanitarian"),
                    ]
                }
            }),
            Callback::new(move |next| case_type.set(next)),
        ),
    ]);
    let column_filters = Signal::derive_local(move || column_filter_definitions.get_value());
    let filter_result = Signal::derive(move || {
        FilterResultSummary::new(filtered.get().len(), snapshot.get().len())
    });
    let filter_texts = Signal::derive(move || {
        if spanish.get() {
            FilterBarTexts {
                region_label: "Filtros".to_owned(),
                active_none: "Sin filtros activos".to_owned(),
                active_one: "1 filtro activo".to_owned(),
                active_many: "{count} filtros activos".to_owned(),
                remove_filter: "Quitar el filtro {label}".to_owned(),
                result_count: "{visible} de {total} resultados".to_owned(),
                reset: "Restablecer".to_owned(),
                save_default: "Guardar como predeterminado".to_owned(),
                clean_reason: "Los valores predeterminados ya están guardados".to_owned(),
                pending_reason: "Se está guardando la vista predeterminada".to_owned(),
                pending_feedback: "Guardando la vista predeterminada".to_owned(),
                saved_feedback: "Vista predeterminada guardada".to_owned(),
                conflict_feedback: "Conflicto de vista predeterminada: {message}".to_owned(),
                failure_feedback: "No se pudo guardar la vista: {message}".to_owned(),
            }
        } else {
            FilterBarTexts::default()
        }
    });
    let entity_texts = Signal::derive(move || {
        if spanish.get() {
            EntityTableTexts {
                region_label: "Tabla de clientes".to_owned(),
                rows_per_page: "Filas por página".to_owned(),
                rows_per_page_auto: "Automático ({rows})".to_owned(),
                choose_columns: "Elegir columnas".to_owned(),
                filters: "Filtros de columnas".to_owned(),
                filter_active: "Filtro activo".to_owned(),
                clear_filter: "Quitar el filtro {column}".to_owned(),
                column_order: "Orden de columnas".to_owned(),
                move_earlier: "Mover {column} antes desde la posición {position} de {total}"
                    .to_owned(),
                move_later: "Mover {column} después desde la posición {position} de {total}"
                    .to_owned(),
                resize_column: "Cambiar el ancho de la columna {column}".to_owned(),
                pixel_value: "{pixels} píxeles".to_owned(),
                sort_not_sorted: "Sin orden activo".to_owned(),
                sort_current: "Orden {direction}, prioridad {priority} de {total}".to_owned(),
                sort_plain_ascending: "Activar para ordenar ascendente solamente".to_owned(),
                sort_plain_descending: "Activar para ordenar descendente solamente".to_owned(),
                sort_plain_system: "Activar para restaurar el orden del sistema".to_owned(),
                sort_add: "Mayús+activar para añadir como prioridad {priority} ascendente"
                    .to_owned(),
                sort_change: "Mayús+activar para cambiar la prioridad {priority} a {direction}"
                    .to_owned(),
                sort_remove: "Mayús+activar para quitar la prioridad {priority}".to_owned(),
                ascending: "ascendente".to_owned(),
                descending: "descendente".to_owned(),
                system_order: "Orden del sistema".to_owned(),
                sort_summary: "Ordenado por {clauses}".to_owned(),
                sort_clause: "prioridad {priority}: {column} {direction}".to_owned(),
                reset_sort: "Restablecer orden".to_owned(),
                reset_columns: "Restablecer columnas".to_owned(),
                previous: "Anterior".to_owned(),
                next: "Siguiente".to_owned(),
                row_range: "Mostrando {start}-{end} de {total}".to_owned(),
                no_rows: "Sin filas".to_owned(),
                // Provider-empty and filtered-empty are different facts and
                // get different sentences (ldui-g4nw).
                no_matching_rows: "Ninguna fila coincide con los filtros".to_owned(),
            }
        } else {
            EntityTableTexts {
                region_label: "Client records".to_owned(),
                ..Default::default()
            }
        }
    });
    let dataset_texts = Signal::derive(move || {
        if spanish.get() {
            DatasetSelectorTexts {
                loading: "Cargando conjunto de datos".to_owned(),
                displayed: "Mostrando {dataset}".to_owned(),
                requested: "Cargando {dataset}".to_owned(),
                retained_error: "No se pudo reemplazar el conjunto de datos".to_owned(),
                retry: "Reintentar".to_owned(),
            }
        } else {
            DatasetSelectorTexts::default()
        }
    });
    let projected_defaults = Signal::derive(move || {
        CLIENT_SNAPSHOT_DEMO_FILTERS
            .project_defaults(
                [
                    ("status", json!(status.get())),
                    ("case_type", json!(case_type.get())),
                ],
                table_preferences.get(),
            )
            .expect("the demo projects only schema-declared defaults")
    });
    let default_save = SnapshotDefaultSave::new(
        projected_defaults,
        save_state,
        Callback::new(move |defaults: SnapshotViewDefaults| {
            save_count.update(|count| *count += 1);
            debug_state::set("snapshot_table.saved_defaults", defaults);
            save_state.set(SnapshotDefaultSaveState::Saved);
        }),
    );
    let first_dirty_check = StoredValue::new(true);
    Effect::new(move |_| {
        let _ = status.get();
        let _ = case_type.get();
        let _ = table_preferences.get();
        if first_dirty_check.get_value() {
            first_dirty_check.set_value(false);
        } else if !matches!(
            save_state.get_untracked(),
            SnapshotDefaultSaveState::Pending
        ) {
            save_state.set(SnapshotDefaultSaveState::Dirty);
        }
    });

    // ldui-3br: two standalone FilterBar fixtures with no `search` slot, for
    // the reactivity suite's `actions-only` and `columns-only` coverage.
    // Independent of the table above (own signals, no shared dataset
    // identity) since the bead is layout-only.
    let actions_only_reset_count = RwSignal::new(0_usize);
    let columns_only_priority = RwSignal::new(String::new());
    let columns_only_chips = Signal::derive(move || {
        let value = columns_only_priority.get();
        if value.is_empty() {
            Vec::new()
        } else {
            vec![ActiveFilterChip::new("priority", "Priority", value)]
        }
    });
    let columns_only_result = Signal::derive(move || {
        FilterResultSummary::new(
            if columns_only_priority.get().is_empty() {
                9
            } else {
                3
            },
            9,
        )
    });
    let columns_only_remove =
        Callback::new(move |_: String| columns_only_priority.set(String::new()));

    view! {
        <ListPage contract_id=CLIENT_SNAPSHOT_DEMO_PAGE.id>
            <PageHeader
                title="Client snapshot list"
                subtitle="One selected office downloads once; filtering, sorting, paging, and column preferences stay local."
                back=Box::new(|| view! {
                    <a
                        class="btn btn-ghost btn-sm"
                        href="#client-snapshot-table"
                        data-testid="client-snapshot-back"
                    >
                        "← Back"
                    </a>
                }.into_any())
                freshness=Box::new(move || view! {
                    <Badge color=BadgeColor::Success class="badge-sm">"Live"
                    </Badge>
                }.into_any())
                dataset=Box::new(move || view! {
                    <DatasetSelector
                        control_id="client-snapshot-dataset-selector"
                        label=Signal::derive(move || {
                            if spanish.get() { "Oficina".to_owned() } else { "Office".to_owned() }
                        })
                        selected=office
                        options=Signal::derive(move || vec![
                            DatasetOption::new(
                                "office-mx",
                                if spanish.get() { "Ciudad de México" } else { "Mexico City" },
                            ),
                            DatasetOption::new(
                                "office-in",
                                if spanish.get() { "Nueva Delhi" } else { "New Delhi" },
                            ),
                        ])
                        on_change=Callback::new(move |next| office.set(next))
                        loading=Signal::derive(move || retained_state.get() == PageState::Revalidating)
                        texts=dataset_texts
                    />
                }.into_any())
                actions=Box::new(move || view! {
                    <Button
                        class="btn-outline btn-sm"
                        attr:data-testid="toggle-client-locale"
                        on_click=Callback::new(move |_| spanish.update(|spanish| *spanish = !*spanish))
                    >
                        {move || if spanish.get() { "English" } else { "Español" }}
                    </Button>
                    <Button
                        class="btn-outline btn-sm"
                        attr:data-testid="change-access-generation"
                        on_click=Callback::new(move |_| access_generation.update(|value| *value += 1))
                    >
                        "Change access"
                    </Button>
                    <Button
                        class="btn-outline btn-sm"
                        attr:data-testid="toggle-revalidating"
                        on_click=Callback::new(move |_| {
                            retained_state.update(|state| {
                                *state = if *state == PageState::Ready {
                                    PageState::Revalidating
                                } else {
                                    PageState::Ready
                                };
                            });
                        })
                    >
                        "Toggle refresh"
                    </Button>
                }.into_any())
            />

            <FilterBar
                search=Box::new(move || view! {
                    <label class="flex w-full flex-col gap-1">
                        <span class="text-xs font-medium">
                            {move || if spanish.get() { "Buscar" } else { "Search" }}
                        </span>
                        <Input
                            input_type=InputType::Search
                            class="input-sm w-full"
                            placeholder=Signal::derive(move || {
                                if spanish.get() {
                                    "Cliente o id de registro".to_owned()
                                } else {
                                    "Client or record id".to_owned()
                                }
                            })
                            value=search
                            on_input=Callback::new(move |value| search.set(value))
                        />
                    </label>
                }.into_any())
                active_filters=active_chips
                on_remove=remove_filter
                on_reset=clear_filters
                result=filter_result
                default_save=default_save
                texts=filter_texts
            />

            // ldui-3br fixture: no `search` slot, framework actions only —
            // the reactivity suite asserts no `[data-filter-search]` wrapper
            // renders here and that Reset still works.
            <div data-testid="filter-bar-actions-only">
                <FilterBar
                    on_reset=Callback::new(move |()| {
                        actions_only_reset_count.update(|count| *count += 1);
                    })
                    actions=Box::new(|| view! {
                        <Button
                            class="btn-outline btn-sm"
                            attr:data-testid="filter-bar-actions-only-export"
                        >
                            "Export"
                        </Button>
                    }.into_any())
                />
                <p class="text-xs text-base-content/60">
                    "Actions-only resets: "
                    <strong data-testid="filter-bar-actions-only-reset-count">
                        {move || actions_only_reset_count.get()}
                    </strong>
                </p>
            </div>

            // ldui-3br fixture: no `search` slot, one column filter plus the
            // active-filter chip summary and result count, no actions at
            // all — the reactivity suite asserts no `[data-filter-search]`
            // and no `[data-filter-actions]` wrapper renders here.
            <div data-testid="filter-bar-columns-only">
                <FilterBar
                    active_filters=columns_only_chips
                    on_remove=columns_only_remove
                    result=columns_only_result
                >
                    <label class="flex flex-col gap-1">
                        <span class="text-xs font-medium">"Priority"</span>
                        <Select
                            class="select-sm"
                            attr:data-testid="filter-bar-columns-only-priority"
                            on:change=move |event| {
                                columns_only_priority.set(event_target_value(&event));
                            }
                        >
                            <option value="" selected=move || columns_only_priority.get().is_empty()>
                                "All priorities"
                            </option>
                            <option value="Urgent" selected=move || columns_only_priority.get() == "Urgent">
                                "Urgent"
                            </option>
                        </Select>
                    </label>
                </FilterBar>
            </div>

            <div class="flex flex-wrap gap-4 text-xs text-base-content/60" aria-live="polite">
                <span>"Claims: " <strong data-testid="entity-claim-count">{move || claim_count.get()}</strong></span>
                <span>"Kept actions: " <strong data-testid="entity-retain-count">{move || retained_action_count.get()}</strong></span>
                <span>"Default saves: " <strong data-testid="entity-save-count">{move || save_count.get()}</strong></span>
                <span>"Row activations: " <strong data-testid="entity-activate-count">{move || activate_count.get()}</strong></span>
                <span>"Toolbar actions: " <strong data-testid="entity-toolbar-action-count">{move || toolbar_action_count.get()}</strong></span>
                <span>"Exported page/all: " <strong data-testid="entity-export-counts">{move || format!("{}/{}", exported_current_rows.get(), exported_all_rows.get())}</strong></span>
                <span>"Exported first key: " <strong data-testid="entity-export-first-key">{move || exported_first_key.get()}</strong></span>
                <span>"Last row: " <strong data-testid="entity-last-row">{move || last_activated.get()}</strong></span>
            </div>
            <div class="flex flex-wrap gap-1" aria-label="Default save state fixture">
                <Button class="btn-ghost btn-xs" attr:data-testid="save-state-dirty" on_click=Callback::new(move |_| save_state.set(SnapshotDefaultSaveState::Dirty))>"Dirty"</Button>
                <Button class="btn-ghost btn-xs" attr:data-testid="save-state-pending" on_click=Callback::new(move |_| save_state.set(SnapshotDefaultSaveState::Pending))>"Pending"</Button>
                <Button class="btn-ghost btn-xs" attr:data-testid="save-state-conflict" on_click=Callback::new(move |_| save_state.set(SnapshotDefaultSaveState::Conflict("revision changed".to_owned())))>"Conflict"</Button>
                <Button class="btn-ghost btn-xs" attr:data-testid="save-state-failure" on_click=Callback::new(move |_| save_state.set(SnapshotDefaultSaveState::Failure("network unavailable".to_owned())))>"Failure"</Button>
            </div>

            <AsyncDataSection
                state=retained_state
                on_retry=Callback::new(move |_| retained_state.set(PageState::Revalidating))
            >
                <EntityTable
                    data=filtered
                    source_data=snapshot
                    page_size_control_id="client-snapshot-page-size"
                    columns=columns
                    column_filters=column_filters
                    row_key=Rc::new(|row: &DemoRow| row.id.clone())
                    dataset_identity=Signal::derive(move || office.get())
                    focus_scope=Signal::derive(move || format!("{}:{}", office.get(), access_generation.get()))
                    preference_ownership=preference_ownership
                    preference_version=1
                    texts=entity_texts
                    show_reset_actions=true
                    column_chooser_trigger=EntityColumnChooserTrigger::Icon
                    toolbar_actions=Box::new(move || {
                        view! {
                            <Button
                                class="btn-outline btn-sm"
                                attr:data-testid="entity-toolbar-export"
                                attr:aria-label="Export current rows"
                                on_click=Callback::new(move |_| {
                                    let projection = display_projection.get_untracked();
                                    let current = projection.rows(EntityTableProjectionScope::CurrentPage);
                                    let all = projection.rows(EntityTableProjectionScope::AllFiltered);
                                    exported_current_rows.set(current.len());
                                    exported_all_rows.set(all.len());
                                    exported_first_key.set(
                                        current.first().map_or_else(String::new, |row| row.key.clone()),
                                    );
                                    toolbar_action_count.update(|count| *count += 1);
                                })
                            >
                                "Export CSV"
                            </Button>
                        }
                        .into_any()
                    })
                    on_display_projection=Callback::new(move |projection| {
                        display_projection.set(projection);
                    })
                    on_row_activate=Callback::new(move |key: String| {
                        activate_count.update(|count| *count += 1);
                        last_activated.set(key);
                    })
                />
            </AsyncDataSection>
        </ListPage>
    }
}
