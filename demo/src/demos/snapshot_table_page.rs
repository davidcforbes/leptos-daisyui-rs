use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    Button, EntityColumn, EntityTablePreferenceOwnership, EntityTablePreferencePersistence,
};
use leptos_daisyui_rs::patterns::{
    PageHeader, SnapshotData, SnapshotDatasetOption, SnapshotDatasetSelectorConfig,
    SnapshotEntityTableConfig, SnapshotLocalRowProjection, SnapshotRequestHandle,
    SnapshotTablePage, SnapshotTableState,
};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct FixtureRow {
    id: String,
    client: String,
    status: String,
}

fn rows(dataset: &str) -> Rc<Vec<FixtureRow>> {
    let office = if dataset == "office-in" {
        "New Delhi"
    } else {
        "Mexico City"
    };
    Rc::new(
        (1..=3)
            .map(|index| FixtureRow {
                id: format!("{dataset}-{index}"),
                client: format!("{office} Client {index}"),
                status: if index == 1 { "Urgent" } else { "Ready" }.to_owned(),
            })
            .collect(),
    )
}

fn snapshot(dataset: &str, revision: &str) -> SnapshotData<FixtureRow, String, ()> {
    let rows = rows(dataset);
    SnapshotData::new(
        dataset.to_owned(),
        Rc::clone(&rows),
        revision,
        rows.len(),
        Some(()),
    )
    .expect("fixture snapshot is complete")
}

fn columns() -> Vec<EntityColumn<FixtureRow>> {
    vec![
        EntityColumn::text("client", "Client", |row: &FixtureRow| row.client.clone())
            .required()
            .with_min_width(220),
        EntityColumn::text("status", "Status", |row: &FixtureRow| row.status.clone())
            .with_min_width(120),
    ]
}

/// Small real-WASM fixture for the typed snapshot composition contract.
#[component]
pub fn SnapshotTablePageFixture() -> impl IntoView {
    type State = SnapshotTableState<FixtureRow, String, String, (), String>;

    let mut initial = State::new();
    let request = initial
        .start_request("office-mx".to_owned())
        .expect("initial fixture request");
    assert_eq!(
        initial.complete(request, snapshot("office-mx", "mx-r1")),
        leptos_daisyui_rs::patterns::SnapshotTransitionDisposition::Applied
    );

    let state = RwSignal::new_local(initial);
    let pending = RwSignal::new_local(Option::<SnapshotRequestHandle<String>>::None);
    let filter_mode = RwSignal::new("all");
    let local_rows = RwSignal::new_local(Option::<SnapshotLocalRowProjection<FixtureRow>>::None);

    Effect::new(move |_| {
        let mode = filter_mode.get();
        let projection = state.with(|state| {
            let displayed = state.view(None).displayed()?;
            let rows = displayed
                .rows()
                .iter()
                .filter(|row| match mode {
                    "urgent" => row.status == "Urgent",
                    "none" => false,
                    _ => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            state.local_row_projection(Rc::new(rows))
        });
        local_rows.set(projection);
    });

    let request_dataset = Callback::new(move |dataset: String| {
        let mut next = None;
        state.update(|state| next = state.start_request(dataset).ok());
        pending.set(next);
    });
    let selector = SnapshotDatasetSelectorConfig::new(
        "Office",
        Signal::stored(vec![
            SnapshotDatasetOption::new("office-mx".to_owned(), "Mexico City"),
            SnapshotDatasetOption::new("office-in".to_owned(), "New Delhi"),
        ]),
        Arc::new(|value: &String| value.clone()),
        request_dataset,
    );
    let table = SnapshotEntityTableConfig::new(
        columns(),
        Rc::new(|row: &FixtureRow| row.id.clone()),
        EntityTablePreferenceOwnership::uncontrolled(EntityTablePreferencePersistence::Disabled),
    );

    let start_replacement = Callback::new(move |_| request_dataset.run("office-in".to_owned()));
    let fail_replacement = Callback::new(move |_| {
        let Some(handle) = pending.get_untracked() else {
            return;
        };
        state.update(|state| {
            state.fail(handle, "Fixture replacement failed.".to_owned());
        });
        pending.set(None);
    });
    let complete_replacement = Callback::new(move |_| {
        let Some(handle) = pending.get_untracked() else {
            return;
        };
        state.update(|state| {
            state.complete(handle, snapshot("office-in", "in-r1"));
        });
        pending.set(None);
    });
    let retry = Callback::new(move |_| request_dataset.run("office-in".to_owned()));

    view! {
        <SnapshotTablePage
            contract_id="snapshot-page"
            state=state.into()
            local_rows=local_rows.into()
            header=Box::new(|| view! {
                <PageHeader
                    title="Snapshot table fixture"
                    subtitle="Typed identity, retained mounting, and slot-order proof."
                />
            }.into_any())
            dataset_selector=selector
            kpis=Box::new(|| view! {
                <div class="stats shadow-sm" data-testid="snapshot-kpi">
                    <div class="stat py-2">
                        <div class="stat-title">"Rows"</div>
                        <div class="stat-value text-2xl">"3"</div>
                    </div>
                </div>
            }.into_any())
            filters=Box::new(move || view! {
                <div class="flex flex-wrap gap-2" aria-label="Fixture controls">
                    <Button
                        attr:data-testid="snapshot-filter-all"
                        attr:aria-pressed=move || (filter_mode.get() == "all").to_string()
                        on_click=Callback::new(move |_| filter_mode.set("all"))
                    >
                        "All rows"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-filter-urgent"
                        attr:aria-pressed=move || (filter_mode.get() == "urgent").to_string()
                        on_click=Callback::new(move |_| filter_mode.set("urgent"))
                    >
                        "Urgent only"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-filter-none"
                        attr:aria-pressed=move || (filter_mode.get() == "none").to_string()
                        on_click=Callback::new(move |_| filter_mode.set("none"))
                    >
                        "No matches"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-start-replacement"
                        on_click=start_replacement
                    >
                        "Start replacement"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-fail-replacement"
                        on_click=fail_replacement
                    >
                        "Fail replacement"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-complete-replacement"
                        on_click=complete_replacement
                    >
                        "Complete replacement"
                    </Button>
                </div>
            }.into_any())
            entity_table=table
            on_retry=retry
            action_key_label=Rc::new(|key: &String| key.clone())
        />
    }
}
