use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::patterns::*;
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
        filters: [search, status, case_type],
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

fn demo_columns(claim_count: RwSignal<usize>) -> Vec<EntityColumn<DemoRow>> {
    vec![
        EntityColumn::text("client", "Client", |row: &DemoRow| row.client.clone())
            .required()
            .with_min_width(240),
        EntityColumn::text("status", "Status", |row: &DemoRow| row.status.clone())
            .with_min_width(110),
        EntityColumn::text("case_type", "Case type", |row: &DemoRow| {
            row.case_type.clone()
        })
        .with_min_width(150),
        EntityColumn::text("received", "Received", |row: &DemoRow| row.received.clone())
            .with_min_width(125),
        EntityColumn::action("actions", "Actions", |_: &DemoRow| "Claim".to_owned())
            .required()
            .non_resizable()
            .render_with(move |row| {
                let row_id = row.id.clone();
                view! {
                    <Button
                        class="btn-primary btn-xs"
                        on_click=Callback::new(move |_| {
                            let _ = &row_id;
                            claim_count.update(|count| *count += 1);
                        })
                    >
                        "Claim"
                    </Button>
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
    let activate_count = RwSignal::new(0_usize);
    let last_activated = RwSignal::new(String::new());

    let snapshot = Signal::derive_local(move || Rc::new(demo_rows(&office.get())));
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
        let mut chips = Vec::new();
        if !search.get().is_empty() {
            chips.push(ActiveFilterChip::new("search", "Search", search.get()));
        }
        if !status.get().is_empty() {
            chips.push(ActiveFilterChip::new("status", "Status", status.get()));
        }
        if !case_type.get().is_empty() {
            chips.push(ActiveFilterChip::new(
                "case_type",
                "Case type",
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

    view! {
        <ListPage contract_id=CLIENT_SNAPSHOT_DEMO_PAGE.id>
            <PageHeader
                title="Client snapshot list"
                subtitle="One selected office downloads once; filtering, sorting, paging, and column preferences stay local."
                freshness=Box::new(move || view! {
                    <Badge color=BadgeColor::Success class="badge-sm">"Live"
                    </Badge>
                }.into_any())
                dataset=Box::new(move || view! {
                    <DatasetSelector
                        label="Office"
                        selected=office
                        options=Signal::stored(vec![
                            DatasetOption::new("office-mx", "Mexico City"),
                            DatasetOption::new("office-in", "New Delhi"),
                        ])
                        on_change=Callback::new(move |next| office.set(next))
                        status=Box::new(move || view! {
                            <span class="text-xs text-success">"Connected"</span>
                        }.into_any())
                    />
                }.into_any())
                actions=Box::new(move || view! {
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
                        <span class="text-xs font-medium">"Search"
                        </span>
                        <Input
                            input_type=InputType::Search
                            class="input-sm w-full"
                            placeholder="Client or record id"
                            value=search
                            on_input=Callback::new(move |value| search.set(value))
                        />
                    </label>
                }.into_any())
                actions=Box::new(move || view! {
                    <Button
                        class="btn-ghost btn-sm"
                        on_click=Callback::new(move |_| clear_filters.run(()))
                    >
                        "Reset filters"
                    </Button>
                }.into_any())
            >
                <label class="flex min-w-40 flex-col gap-1">
                    <span class="text-xs font-medium">"Status"
                    </span>
                    <Select
                        class="select-sm"
                        label=Signal::stored(Some("Status".to_owned()))
                        value=status
                        on_change=Callback::new(move |value| status.set(value))
                    >
                        <option value="">"All statuses"</option>
                        <option value="Ready">"Ready"</option>
                        <option value="Urgent">"Urgent"</option>
                    </Select>
                </label>
                <label class="flex min-w-44 flex-col gap-1">
                    <span class="text-xs font-medium">"Case type"
                    </span>
                    <Select
                        class="select-sm"
                        label=Signal::stored(Some("Case type".to_owned()))
                        value=case_type
                        on_change=Callback::new(move |value| case_type.set(value))
                    >
                        <option value="">"All case types"</option>
                        <option value="Family">"Family"</option>
                        <option value="Humanitarian">"Humanitarian"</option>
                    </Select>
                </label>
            </FilterBar>

            <ActiveFilterChips
                chips=active_chips
                on_remove=remove_filter
                on_clear=clear_filters
            />

            <div class="flex flex-wrap gap-4 text-xs text-base-content/60" aria-live="polite">
                <span>"Claims: " <strong data-testid="entity-claim-count">{move || claim_count.get()}</strong></span>
                <span>"Row activations: " <strong data-testid="entity-activate-count">{move || activate_count.get()}</strong></span>
                <span>"Last row: " <strong data-testid="entity-last-row">{move || last_activated.get()}</strong></span>
            </div>

            <AsyncDataSection
                state=retained_state
                on_retry=Callback::new(move |_| retained_state.set(PageState::Revalidating))
            >
                <EntityTable
                    data=filtered
                    columns=demo_columns(claim_count)
                    row_key=Rc::new(|row: &DemoRow| row.id.clone())
                    dataset_identity=Signal::derive(move || office.get())
                    storage_key="client-snapshot-demo"
                    preference_version=1
                    on_row_activate=Callback::new(move |key: String| {
                        activate_count.update(|count| *count += 1);
                        last_activated.set(key);
                    })
                />
            </AsyncDataSection>
        </ListPage>
    }
}
