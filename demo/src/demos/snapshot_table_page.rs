use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    BadgeColor, Button, EntityBadgePresentation, EntityColumn, EntityColumnFilter, EntityIconColor,
    EntityIconPresentation, EntityNullOrder, EntityTable, EntityTablePreferenceOwnership,
    EntityTablePreferencePersistence, EntityTableTexts, EntityTableViewportFit,
};
use leptos_daisyui_rs::patterns::{
    PageHeader, PageHeaderNavigationLayout, SnapshotData, SnapshotDatasetOption,
    SnapshotDatasetSelectorConfig, SnapshotEntityTableConfig, SnapshotLocalRowProjection,
    SnapshotRequestHandle, SnapshotTablePage, SnapshotTableState,
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
                    navigation_layout=PageHeaderNavigationLayout::DedicatedRow
                    navigation_label=Signal::stored("Snapshot navigation".to_owned())
                    back=Box::new(|| view! {
                        <a
                            class="btn btn-ghost btn-sm"
                            href="#snapshot-page-table"
                            data-testid="snapshot-page-back"
                        >
                            "← Back to reports"
                        </a>
                    }.into_any())
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

/// Real-WASM fixture for framework-owned EntityTable viewport-fit paging.
#[component]
pub fn EntityTableViewportFitFixture() -> impl IntoView {
    let height = RwSignal::new(520_u32);
    let alternate_copy = RwSignal::new(false);
    let data: Signal<Rc<Vec<FixtureRow>>, LocalStorage> = RwSignal::new_local(Rc::new(
        (1..=60)
            .map(|index| FixtureRow {
                id: format!("fit-{index:02}"),
                client: format!("Viewport client {index:02}"),
                status: if index % 3 == 0 { "Urgent" } else { "Ready" }.to_owned(),
            })
            .collect::<Vec<_>>(),
    ))
    .into();
    let columns = Signal::derive_local(move || {
        vec![
            EntityColumn::text(
                "client",
                if alternate_copy.get() {
                    "Client account with localized heading"
                } else {
                    "Client"
                },
                |row: &FixtureRow| row.client.clone(),
            )
            .required()
            .with_min_width(220),
            EntityColumn::text(
                "status",
                if alternate_copy.get() {
                    "Localized workflow status"
                } else {
                    "Status"
                },
                |row: &FixtureRow| row.status.clone(),
            )
            .with_min_width(150),
        ]
    });
    let texts = Signal::derive(move || EntityTableTexts {
        rows_per_page: if alternate_copy.get() {
            "Rows per page used when the viewport is too short".to_owned()
        } else {
            "Rows per page".to_owned()
        },
        ..EntityTableTexts::default()
    });
    let filters = vec![EntityColumnFilter::new("status", || {
        view! {
            <span class="block min-h-8 py-1 text-xs" data-testid="viewport-fit-filter-row">
                "Controlled status filter"
            </span>
        }
        .into_any()
    })];

    view! {
        <section id="entity-viewport-fit-fixture" class="space-y-3">
            <div class="flex flex-wrap gap-2" aria-label="Viewport fit controls">
                <Button attr:data-testid="viewport-fit-default" on_click=Callback::new(move |_| height.set(520))>
                    "Default height"
                </Button>
                <Button attr:data-testid="viewport-fit-tall" on_click=Callback::new(move |_| height.set(800))>
                    "Tall height"
                </Button>
                <Button attr:data-testid="viewport-fit-short" on_click=Callback::new(move |_| height.set(180))>
                    "Short height"
                </Button>
                <Button attr:data-testid="viewport-fit-locale" on_click=Callback::new(move |_| alternate_copy.update(|value| *value = !*value))>
                    "Toggle localized copy"
                </Button>
            </div>
            <div
                class="min-h-0 w-full"
                style=move || format!("height: {}px", height.get())
                data-testid="viewport-fit-budget"
            >
                <EntityTable
                    data=data
                    columns=columns
                    column_filters=filters
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="viewport-fit-fixture"
                    viewport_fit=EntityTableViewportFit::fill_parent().with_min_rows(3)
                    preference_ownership=EntityTablePreferenceOwnership::uncontrolled(
                        EntityTablePreferencePersistence::Disabled,
                    )
                    texts=texts
                    page_size_control_id="viewport-fit-page-size"
                />
            </div>
        </section>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityTablePresentationRow {
    id: String,
    reference: String,
    narrative: String,
    rich: String,
    number: i64,
    date_key: (u16, u8, u8),
    optional_rank: Option<i64>,
    currency: String,
    percentage: String,
    status: String,
    icon_label: String,
}

/// Focused browser fixture for framework-owned EntityColumn presentation.
#[component]
pub fn EntityTablePresentationFixture() -> impl IntoView {
    let data = RwSignal::new_local(Rc::new(vec![
        EntityTablePresentationRow {
            id: "presentation-1".to_owned(),
            reference: "REFERENCE-WITH-ONE-UNBROKEN-VALUE-0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .to_owned(),
            narrative: "This canonical narrative is intentionally long enough to occupy more than two visual lines while remaining complete in the DOM for assistive technology and export."
                .to_owned(),
            rich: "Canonical rich-renderer fallback text".to_owned(),
            number: 10,
            date_key: (2026, 1, 2),
            optional_rank: None,
            currency: "-$12,345,678,901.25".to_owned(),
            percentage: "100.00%".to_owned(),
            status: "Needs review".to_owned(),
            icon_label: "Enabled".to_owned(),
        },
        EntityTablePresentationRow {
            id: "presentation-2".to_owned(),
            reference: "SHORT-2".to_owned(),
            narrative: "A short narrative.".to_owned(),
            rich: "Second rich value".to_owned(),
            number: 2,
            date_key: (2025, 12, 31),
            optional_rank: Some(10),
            currency: "$24.00".to_owned(),
            percentage: "20.50%".to_owned(),
            status: "Ready".to_owned(),
            icon_label: "Attention".to_owned(),
        },
        EntityTablePresentationRow {
            id: "presentation-3".to_owned(),
            reference: "NEGATIVE-3".to_owned(),
            narrative: "Signed numeric ordering fixture.".to_owned(),
            rich: "Third rich value".to_owned(),
            number: -3,
            date_key: (2026, 1, 1),
            optional_rank: Some(2),
            currency: "-$3.00".to_owned(),
            percentage: "-3.00%".to_owned(),
            status: "Unknown status".to_owned(),
            icon_label: "Unknown state".to_owned(),
        },
        EntityTablePresentationRow {
            id: "presentation-4".to_owned(),
            reference: "SECOND-TWO".to_owned(),
            narrative: "Stable equal-key ordering fixture.".to_owned(),
            rich: "Fourth rich value".to_owned(),
            number: 2,
            date_key: (2026, 2, 1),
            optional_rank: Some(10),
            currency: "$2.00".to_owned(),
            percentage: "2.00%".to_owned(),
            status: String::new(),
            icon_label: String::new(),
        },
    ]));
    let semantic_localized = RwSignal::new(false);
    let toggle_semantic_locale = Callback::new(move |_| {
        let localized = !semantic_localized.get_untracked();
        semantic_localized.set(localized);
        data.update(|rows| {
            let mut replacement = rows.as_ref().clone();
            replacement[0].status = if localized {
                "Revisión necesaria"
            } else {
                "Needs review"
            }
            .to_owned();
            replacement[0].icon_label = if localized { "Habilitado" } else { "Enabled" }.to_owned();
            *rows = Rc::new(replacement);
        });
    });
    let columns = vec![
        EntityColumn::text(
            "reference",
            "Reference",
            |row: &EntityTablePresentationRow| row.reference.clone(),
        )
        .ellipsis()
        .with_min_width(110)
        .with_width(150),
        EntityColumn::text(
            "narrative",
            "Narrative",
            |row: &EntityTablePresentationRow| row.narrative.clone(),
        )
        .line_clamp(2)
        .with_min_width(150)
        .with_width(220),
        EntityColumn::text(
            "rich",
            "Rich precedence",
            |row: &EntityTablePresentationRow| row.rich.clone(),
        )
        .line_clamp(2)
        .with_width(180)
        .align_end()
        .tabular_numbers()
        .render_with(|row: &EntityTablePresentationRow| {
            let text = row.rich.clone();
            view! { <em data-entity-presentation-rich="true">{text}</em> }.into_any()
        }),
        EntityColumn::text(
            "number",
            "Typed number",
            |row: &EntityTablePresentationRow| row.number.to_string(),
        )
        .sortable_by_key(|row| row.number)
        .align_end()
        .tabular_numbers()
        .with_width(130),
        EntityColumn::text("date", "Typed date", |row: &EntityTablePresentationRow| {
            format!(
                "{:04}-{:02}-{:02}",
                row.date_key.0, row.date_key.1, row.date_key.2
            )
        })
        .sortable_by_key(|row| row.date_key)
        .align_center()
        .tabular_numbers()
        .with_width(135),
        EntityColumn::text(
            "optional",
            "Optional rank",
            |row: &EntityTablePresentationRow| {
                row.optional_rank
                    .map_or_else(|| "Not ranked".to_owned(), |rank| rank.to_string())
            },
        )
        .sortable_by_optional_key(EntityNullOrder::Last, |row| row.optional_rank)
        .align_end()
        .tabular_numbers()
        .with_width(140),
        EntityColumn::text(
            "currency",
            "Currency-like",
            |row: &EntityTablePresentationRow| row.currency.clone(),
        )
        .align_end()
        .tabular_numbers()
        .ellipsis()
        .with_width(165),
        EntityColumn::text(
            "percentage",
            "Percentage-like",
            |row: &EntityTablePresentationRow| row.percentage.clone(),
        )
        .align_end()
        .tabular_numbers()
        .with_width(145),
        EntityColumn::text(
            "status_badge",
            "Semantic badge",
            |row: &EntityTablePresentationRow| row.status.clone(),
        )
        .badge_with(|row| match row.id.as_str() {
            "presentation-1" => Some(EntityBadgePresentation::new(BadgeColor::Warning)),
            "presentation-2" => Some(EntityBadgePresentation::new(BadgeColor::Success)),
            "presentation-3" => None,
            _ => Some(EntityBadgePresentation::new(BadgeColor::Info)),
        })
        .with_width(155),
        EntityColumn::text(
            "state_icon",
            "Semantic icon",
            |row: &EntityTablePresentationRow| row.icon_label.clone(),
        )
        .align_center()
        .icon_with(|row| match row.id.as_str() {
            "presentation-1" => Some(EntityIconPresentation::new(
                "circle-check",
                EntityIconColor::Success,
            )),
            "presentation-2" => Some(EntityIconPresentation::new(
                "triangle-alert",
                EntityIconColor::Warning,
            )),
            "presentation-3" => None,
            _ => Some(EntityIconPresentation::new("check", EntityIconColor::Info)),
        })
        .with_width(145),
    ];

    view! {
        <section
            id="entity-table-presentation-fixture"
            class="mx-auto max-w-3xl space-y-3 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Entity column presentation"</h1>
            <p class="text-sm text-base-content/70">
                "Plain canonical values demonstrate ellipsis and two-line clipping; the final column proves rich-renderer precedence."
            </p>
            <Button
                attr:data-testid="entity-presentation-locale"
                on_click=toggle_semantic_locale
            >
                "Toggle semantic cell locale"
            </Button>
            <EntityTable
                data=data
                columns=columns
                row_key=Rc::new(|row: &EntityTablePresentationRow| row.id.clone())
                dataset_identity=Signal::stored("presentation-fixture".to_owned())
            />
        </section>
    }
}

/// Focused browser fixture for the framework-owned page-size select identity
/// (ldui-kl55). Table A and table B both omit `page_size_control_id` — the
/// bug was that this left the rows-per-page `<select>` with no `id`/`name` at
/// all, and Office satellites mounting several `EntityTable`s on one Setup
/// page had no way to tell the controls apart. Table C supplies an explicit
/// override, which must be honored verbatim rather than replaced by a
/// generated default.
#[component]
pub fn EntityTablePageSizeIdentityFixture() -> impl IntoView {
    let data_a = RwSignal::new_local(rows("office-mx"));
    let data_b = RwSignal::new_local(rows("office-in"));
    let data_c = RwSignal::new_local(rows("office-mx"));

    view! {
        <section
            id="entity-table-page-size-identity-fixture"
            class="mx-auto max-w-3xl space-y-6 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Page-size select identity"</h1>
            <div data-testid="page-size-identity-table-a">
                <EntityTable
                    data=data_a
                    columns=columns()
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="page-size-identity-a"
                />
            </div>
            <div data-testid="page-size-identity-table-b">
                <EntityTable
                    data=data_b
                    columns=columns()
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="page-size-identity-b"
                />
            </div>
            <div data-testid="page-size-identity-table-c">
                <EntityTable
                    data=data_c
                    columns=columns()
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="page-size-identity-c"
                    page_size_control_id="page-size-identity-explicit-override"
                />
            </div>
        </section>
    }
}
