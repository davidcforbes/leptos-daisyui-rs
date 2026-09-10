use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    EntityColumn, EntityColumnFilter, EntityTable, EntityTablePreferenceOwnership,
    EntityTablePreferences,
};
use std::rc::Rc;

#[derive(Clone)]
struct DefaultColumnRow {
    id: &'static str,
    client: &'static str,
    office: &'static str,
    rank: &'static str,
}

fn rows() -> Rc<Vec<DefaultColumnRow>> {
    Rc::new(vec![
        DefaultColumnRow {
            id: "row-1",
            client: "Ada Lovelace",
            office: "Mexico City",
            rank: "Senior",
        },
        DefaultColumnRow {
            id: "row-2",
            client: "Grace Hopper",
            office: "New Delhi",
            rank: "Lead",
        },
    ])
}

fn columns() -> Vec<EntityColumn<DefaultColumnRow>> {
    vec![
        EntityColumn::text("client", "Client", |row: &DefaultColumnRow| {
            row.client.to_owned()
        })
        .required(),
        EntityColumn::text("office", "Office", |row: &DefaultColumnRow| {
            row.office.to_owned()
        })
        .hidden_by_default(),
        EntityColumn::text("rank", "Rank", |row: &DefaultColumnRow| row.rank.to_owned()),
    ]
}

fn preference_json(preferences: &EntityTablePreferences) -> String {
    serde_json::to_string(preferences).expect("fixture preferences serialize")
}

/// Browser fixture for declaration-owned column defaults and controlled intent.
#[component]
pub fn OfficeEntityDefaultsFixture() -> impl IntoView {
    let data = RwSignal::new_local(rows());
    let seeded_preferences = RwSignal::new(EntityTablePreferences::new(1));
    let seeded_changes = RwSignal::new(0_u32);

    let mut explicit_all_visible = EntityTablePreferences::new(1);
    explicit_all_visible.column_order =
        vec!["client".to_owned(), "office".to_owned(), "rank".to_owned()];
    let explicit_preferences = RwSignal::new(explicit_all_visible);
    let rank_filter = RwSignal::new(String::new());

    let filters = vec![
        EntityColumnFilter::text(
            "rank",
            "office-defaults-rank-filter",
            "Rank",
            rank_filter,
            "Filter ranks",
            Callback::new(move |next| rank_filter.set(next)),
        )
        .with_description("Matches any part of the rank label"),
    ];

    view! {
        <section id="office-entity-defaults-fixture" class="mx-auto max-w-5xl space-y-6 bg-base-100 p-4">
            <h1 class="ld-text-display font-semibold">"Entity column defaults"</h1>

            <div data-testid="seeded-defaults-table">
                <EntityTable
                    data=data
                    columns=columns()
                    column_filters=filters
                    row_key=Rc::new(|row: &DefaultColumnRow| row.id.to_owned())
                    dataset_identity="seeded-defaults"
                    preference_ownership=EntityTablePreferenceOwnership::controlled(
                        seeded_preferences.into(),
                        Callback::new(move |replacement| {
                            seeded_changes.update(|count| *count += 1);
                            seeded_preferences.set(replacement);
                        }),
                    )
                    page_size_control_id="seeded-defaults-page-size"
                    show_reset_actions=true
                />
                <output data-testid="seeded-defaults-preferences">
                    {move || preference_json(&seeded_preferences.get())}
                </output>
                <output data-testid="seeded-defaults-change-count">
                    {move || seeded_changes.get().to_string()}
                </output>
            </div>

            <div data-testid="explicit-visible-table">
                <EntityTable
                    data=data
                    columns=columns()
                    row_key=Rc::new(|row: &DefaultColumnRow| row.id.to_owned())
                    dataset_identity="explicit-all-visible"
                    preference_ownership=EntityTablePreferenceOwnership::controlled(
                        explicit_preferences.into(),
                        Callback::new(move |replacement| explicit_preferences.set(replacement)),
                    )
                    page_size_control_id="explicit-visible-page-size"
                    show_reset_actions=true
                />
                <output data-testid="explicit-visible-preferences">
                    {move || preference_json(&explicit_preferences.get())}
                </output>
            </div>
        </section>
    }
}
