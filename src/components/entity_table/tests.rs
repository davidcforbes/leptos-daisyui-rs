use super::*;
use crate::components::badge::{BadgeColor, BadgeStyle};
use crate::components::data_table::{clamp_page, page_bounds, page_count};
use leptos::prelude::{Callback, Get, IntoAny, RwSignal, Set, Signal, StoredValue};
use leptos::reactive::owner::Owner;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    id: &'static str,
    name: &'static str,
    rank: u32,
}

fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "r1",
            name: "Zulu",
            rank: 2,
        },
        Row {
            id: "r2",
            name: "Alpha",
            rank: 1,
        },
        Row {
            id: "r3",
            name: "Bravo",
            rank: 1,
        },
    ]
}

fn columns() -> Vec<EntityColumn<Row>> {
    vec![
        EntityColumn::text("client", "Client", |row: &Row| row.name.to_owned()).required(),
        EntityColumn::new("rank", "Rank", |row: &Row| row.rank.to_string())
            .sortable_by(|left, right| left.rank.cmp(&right.rank)),
        EntityColumn::text("office", "Office", |row: &Row| row.id.to_owned()),
        EntityColumn::action("actions", "Actions", |_: &Row| "Claim".to_owned()).required(),
    ]
}

#[test]
fn system_order_and_stable_ties_are_preserved() {
    let rows = rows();
    let columns = columns();
    assert_eq!(
        sorted_indices(&rows, &columns, &EntitySort::System),
        [0, 1, 2]
    );
    assert_eq!(
        sorted_indices(&rows, &columns, &EntitySort::ascending("rank")),
        [1, 2, 0]
    );
    assert_eq!(
        sorted_indices(&rows, &columns, &EntitySort::descending("rank")),
        [0, 1, 2]
    );
}

#[test]
fn default_text_sort_extracts_one_normalized_key_per_row() {
    let calls = Rc::new(Cell::new(0));
    let calls_for_column = Rc::clone(&calls);
    let rows = rows();
    let columns = vec![EntityColumn::text("client", "Client", move |row: &Row| {
        calls_for_column.set(calls_for_column.get() + 1);
        row.name.to_owned()
    })];

    assert_eq!(
        sorted_indices(&rows, &columns, &EntitySort::ascending("client")),
        [1, 2, 0]
    );
    assert_eq!(
        calls.get(),
        rows.len(),
        "text sort keys must be normalized once per row, not once per comparison"
    );
}

#[test]
fn multi_sort_uses_clause_priority_before_dataset_order() {
    let rows = rows();
    let sort = EntitySort::multiple([
        EntitySortColumn::ascending("rank"),
        EntitySortColumn::descending("client"),
    ]);

    assert_eq!(sorted_indices(&rows, &columns(), &sort), [2, 1, 0]);
}

#[test]
fn multi_sort_extracts_each_text_key_once_per_row() {
    let client_calls = Rc::new(Cell::new(0));
    let office_calls = Rc::new(Cell::new(0));
    let client_calls_for_column = Rc::clone(&client_calls);
    let office_calls_for_column = Rc::clone(&office_calls);
    let rows = rows();
    let columns = vec![
        EntityColumn::text("client", "Client", move |row: &Row| {
            client_calls_for_column.set(client_calls_for_column.get() + 1);
            row.name.to_owned()
        }),
        EntityColumn::text("office", "Office", move |row: &Row| {
            office_calls_for_column.set(office_calls_for_column.get() + 1);
            row.id.to_owned()
        }),
    ];
    let sort = EntitySort::multiple([
        EntitySortColumn::ascending("client"),
        EntitySortColumn::descending("office"),
    ]);

    let _ = sorted_indices(&rows, &columns, &sort);

    assert_eq!(client_calls.get(), rows.len());
    assert_eq!(office_calls.get(), rows.len());
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TypedSortRow {
    id: &'static str,
    number: i64,
    date: (u16, u8, u8),
    label: &'static str,
    optional: Option<i64>,
}

fn typed_sort_rows() -> Vec<TypedSortRow> {
    vec![
        TypedSortRow {
            id: "ten",
            number: 10,
            date: (2026, 1, 2),
            label: "Zulu",
            optional: None,
        },
        TypedSortRow {
            id: "two-a",
            number: 2,
            date: (2025, 12, 31),
            label: "Alpha",
            optional: Some(10),
        },
        TypedSortRow {
            id: "negative",
            number: -3,
            date: (2026, 1, 1),
            label: "Bravo",
            optional: Some(2),
        },
        TypedSortRow {
            id: "two-b",
            number: 2,
            date: (2026, 2, 1),
            label: "Charlie",
            optional: Some(10),
        },
    ]
}

fn sorted_typed_ids(
    rows: &[TypedSortRow],
    column: EntityColumn<TypedSortRow>,
    sort: EntitySort,
) -> Vec<&'static str> {
    sorted_indices(rows, &[column], &sort)
        .into_iter()
        .map(|index| rows[index].id)
        .collect()
}

#[test]
fn typed_sort_keys_cover_signed_numeric_date_time_and_string_ordering() {
    let rows = typed_sort_rows();

    assert_eq!(
        sorted_typed_ids(
            &rows,
            EntityColumn::text("number", "Number", |row: &TypedSortRow| row
                .number
                .to_string())
            .sortable_by_key(|row| row.number),
            EntitySort::ascending("number"),
        ),
        ["negative", "two-a", "two-b", "ten"],
        "typed numeric ordering must not fall back to lexical display text"
    );
    assert_eq!(
        sorted_typed_ids(
            &rows,
            EntityColumn::text("number", "Number", |row: &TypedSortRow| row
                .number
                .to_string())
            .sortable_by_key(|row| row.number),
            EntitySort::descending("number"),
        ),
        ["ten", "two-a", "two-b", "negative"],
        "equal typed keys preserve stable source order under descending sort"
    );
    assert_eq!(
        sorted_typed_ids(
            &rows,
            EntityColumn::text("date", "Date", |row: &TypedSortRow| format!(
                "{:04}-{:02}-{:02}",
                row.date.0, row.date.1, row.date.2
            ))
            .sortable_by_key(|row| row.date),
            EntitySort::ascending("date"),
        ),
        ["two-a", "negative", "ten", "two-b"]
    );
    assert_eq!(
        sorted_typed_ids(
            &rows,
            EntityColumn::text("label", "Label", |row: &TypedSortRow| row.label.to_owned())
                .sortable_by_key(|row| row.label.to_owned()),
            EntitySort::ascending("label"),
        ),
        ["two-a", "negative", "two-b", "ten"]
    );
}

#[test]
fn optional_typed_sort_keys_keep_explicit_null_placement_in_both_directions() {
    let rows = typed_sort_rows();
    for direction in [
        EntitySortDirection::Ascending,
        EntitySortDirection::Descending,
    ] {
        let sort = match direction {
            EntitySortDirection::Ascending => EntitySort::ascending("optional"),
            EntitySortDirection::Descending => EntitySort::descending("optional"),
        };
        let first = EntityColumn::text("optional", "Optional", |row: &TypedSortRow| {
            row.optional
                .map_or_else(|| "None".to_owned(), |value| value.to_string())
        })
        .sortable_by_optional_key(EntityNullOrder::First, |row| row.optional);
        let last = EntityColumn::text("optional", "Optional", |row: &TypedSortRow| {
            row.optional
                .map_or_else(|| "None".to_owned(), |value| value.to_string())
        })
        .sortable_by_optional_key(EntityNullOrder::Last, |row| row.optional);

        assert_eq!(sorted_typed_ids(&rows, first, sort.clone())[0], "ten");
        assert_eq!(sorted_typed_ids(&rows, last, sort).last(), Some(&"ten"));
    }
}

#[test]
fn typed_sort_key_is_extracted_once_per_row_and_composes_in_multi_sort() {
    let rows = typed_sort_rows();
    let calls = Rc::new(Cell::new(0));
    let calls_for_key = Rc::clone(&calls);
    let columns = vec![
        EntityColumn::text("number", "Number", |row: &TypedSortRow| {
            row.number.to_string()
        })
        .sortable_by_key(move |row| {
            calls_for_key.set(calls_for_key.get() + 1);
            row.number
        }),
        EntityColumn::text("label", "Label", |row: &TypedSortRow| row.label.to_owned())
            .sortable_by_key(|row| row.label.to_ascii_lowercase()),
    ];
    let sort = EntitySort::multiple([
        EntitySortColumn::ascending("number"),
        EntitySortColumn::descending("label"),
    ]);

    assert_eq!(
        sorted_indices(&rows, &columns, &sort),
        [2, 3, 1, 0],
        "the typed primary key and normalized secondary key compose by clause priority"
    );
    assert_eq!(calls.get(), rows.len());
}

#[test]
fn sorted_index_cache_ignores_unrelated_preference_changes() {
    let calls = Rc::new(Cell::new(0));
    let calls_for_column = Rc::clone(&calls);
    let rows = Rc::new(rows());
    let columns = vec![EntityColumn::text("client", "Client", move |row: &Row| {
        calls_for_column.set(calls_for_column.get() + 1);
        row.name.to_owned()
    })];
    let sort = EntitySort::ascending("client");
    let mut cache = SortedIndexCache::new();

    let first = cache.indices(Rc::clone(&rows), &columns, &sort, 0);
    let first_call_count = calls.get();
    let mut unrelated_preferences = EntityTablePreferences::new(1);
    unrelated_preferences.page_size = 50;
    unrelated_preferences
        .hidden_columns
        .insert("status".to_owned());
    let second = cache.indices(Rc::clone(&rows), &columns, &sort, 0);

    assert_eq!(first.as_slice(), second.as_slice());
    assert_eq!(calls.get(), first_call_count);
    assert_eq!(unrelated_preferences.page_size, 50);

    cache.indices(
        Rc::clone(&rows),
        &columns,
        &EntitySort::descending("client"),
        0,
    );
    assert!(
        calls.get() > first_call_count,
        "a real sort change must recompute"
    );
}

#[test]
fn sorted_index_cache_invalidates_when_column_semantics_are_replaced() {
    let rows = Rc::new(rows());
    let ascending = vec![
        EntityColumn::new("rank", "Rank", |row: &Row| row.rank.to_string())
            .sortable_by(|left, right| left.rank.cmp(&right.rank)),
    ];
    let reversed = vec![
        EntityColumn::new("rank", "Rango", |row: &Row| row.rank.to_string())
            .sortable_by(|left, right| right.rank.cmp(&left.rank)),
    ];
    let sort = EntitySort::ascending("rank");
    let mut cache = SortedIndexCache::new();

    assert_eq!(
        cache
            .indices(Rc::clone(&rows), &ascending, &sort, 1)
            .as_slice(),
        [1, 2, 0]
    );
    assert_eq!(
        cache
            .indices(Rc::clone(&rows), &reversed, &sort, 2)
            .as_slice(),
        [0, 1, 2],
        "unchanged row Rc/sort must use the newest comparator generation"
    );
}

#[test]
fn focus_target_uses_visible_order_and_never_crosses_scope_or_hidden_rows() {
    let record = EntityFocusRecord {
        scope: "dataset-1/access-1".to_owned(),
        row_key: "r2".to_owned(),
        action_id: "delete".to_owned(),
        visible_position: 1,
    };
    let source = |keys: &[&str]| keys.iter().map(|key| (*key).to_owned()).collect::<Vec<_>>();

    assert_eq!(
        focus_target(
            &record,
            &source(&["r1", "r2", "r3"]),
            &source(&["r3", "r2", "r1"]),
            &record.scope,
            false,
            true,
        ),
        EntityFocusTarget::NoChange,
        "an unchanged visible row keeps native action focus"
    );
    assert_eq!(
        focus_target(
            &record,
            &source(&["r1", "r3"]),
            &source(&["r3", "r1"]),
            &record.scope,
            false,
            true,
        ),
        EntityFocusTarget::RowAction {
            row_key: "r1".to_owned(),
            action_id: "delete".to_owned(),
        },
        "the same visible position in sorted order wins after deletion"
    );
    assert_eq!(
        focus_target(
            &record,
            &source(&["r1", "r2", "r3"]),
            &source(&["r1", "r3"]),
            &record.scope,
            false,
            true,
        ),
        EntityFocusTarget::TableRegion,
        "filter/page hiding while the source row remains cannot choose a neighbor"
    );
    assert_eq!(
        focus_target(
            &record,
            &source(&["r1"]),
            &source(&["r1"]),
            &record.scope,
            false,
            false,
        ),
        EntityFocusTarget::TableRegion,
        "a missing, hidden, disabled, or unfocusable matching action falls back"
    );
    assert_eq!(
        focus_target(
            &record,
            &source(&["n1", "n2"]),
            &source(&["n1", "n2"]),
            "dataset-2/access-1",
            false,
            true,
        ),
        EntityFocusTarget::Clear,
        "dataset/access replacement cannot cross-focus"
    );
    assert_eq!(
        focus_target(
            &record,
            &source(&["r1", "r3"]),
            &source(&["r1", "r3"]),
            &record.scope,
            true,
            true,
        ),
        EntityFocusTarget::Clear,
        "a user who already moved focus is never interrupted"
    );

    let last = EntityFocusRecord {
        visible_position: 4,
        ..record
    };
    assert_eq!(
        focus_target(
            &last,
            &source(&["r1"]),
            &source(&["r1"]),
            &last.scope,
            false,
            true,
        ),
        EntityFocusTarget::RowAction {
            row_key: "r1".to_owned(),
            action_id: "delete".to_owned(),
        },
        "last-row/page collapse clamps to the preceding rendered row"
    );
}

#[test]
fn sort_clicks_cycle_system_ascending_descending_system() {
    let system = EntitySort::System;
    let ascending = next_sort(&system, "rank", true);
    assert_eq!(ascending, EntitySort::ascending("rank"));
    let descending = next_sort(&ascending, "rank", true);
    assert_eq!(descending, EntitySort::descending("rank"));
    assert_eq!(next_sort(&descending, "rank", true), EntitySort::System);
    assert_eq!(next_sort(&system, "actions", false), system);
    assert_eq!(
        next_sort(&EntitySort::descending("rank"), "client", true),
        EntitySort::ascending("client")
    );
}

#[test]
fn legacy_sort_variants_remain_constructible_and_matchable() {
    let ascending = EntitySort::ascending("rank");
    assert!(matches!(
        ascending,
        EntitySort::Ascending { ref column } if column == "rank"
    ));

    let descending = EntitySort::descending("client");
    assert!(matches!(
        descending,
        EntitySort::Descending { ref column } if column == "client"
    ));
    assert!(matches!(EntitySort::System, EntitySort::System));

    let multiple = EntitySort::multiple([
        EntitySortColumn::ascending("rank"),
        EntitySortColumn::descending("client"),
    ]);
    assert!(matches!(
        multiple,
        EntitySort::Multiple { ref clauses } if clauses.len() == 2
    ));
}

#[test]
fn additive_sort_clicks_append_cycle_and_remove_one_clause() {
    let rank = EntitySort::ascending("rank");
    let appended = next_sort_additive(&rank, "client", true);
    assert_eq!(
        appended.clauses(),
        [
            EntitySortColumn::ascending("rank"),
            EntitySortColumn::ascending("client"),
        ]
    );

    let descending = next_sort_additive(&appended, "client", true);
    assert_eq!(
        descending.clauses(),
        [
            EntitySortColumn::ascending("rank"),
            EntitySortColumn::descending("client"),
        ]
    );
    assert_eq!(
        next_sort_additive(&descending, "client", true),
        EntitySort::ascending("rank")
    );
    assert_eq!(
        next_sort_additive(&rank, "actions", false),
        rank,
        "non-sortable columns must be inert"
    );
}

#[test]
fn column_moves_use_canonical_order_and_stop_at_boundaries() {
    let columns = columns();
    let mut preferences = EntityTablePreferences::new(1);

    assert!(move_column(
        &mut preferences,
        &columns,
        "office",
        EntityColumnMove::Earlier,
    ));
    assert_eq!(
        preferences.column_order,
        ["client", "office", "rank", "actions"]
    );
    assert!(move_column(
        &mut preferences,
        &columns,
        "office",
        EntityColumnMove::Later,
    ));
    assert_eq!(
        preferences.column_order,
        ["client", "rank", "office", "actions"]
    );
    assert!(!move_column(
        &mut preferences,
        &columns,
        "client",
        EntityColumnMove::Earlier,
    ));
    assert!(!move_column(
        &mut preferences,
        &columns,
        "actions",
        EntityColumnMove::Later,
    ));
    assert!(!move_column(
        &mut preferences,
        &columns,
        "missing",
        EntityColumnMove::Earlier,
    ));
}

#[test]
fn pagination_bounds_and_last_page_are_clamped() {
    assert_eq!(page_count(0, 25), 0);
    assert_eq!(page_count(51, 25), 3);
    assert_eq!(page_bounds(0, 25, 51), 0..25);
    assert_eq!(page_bounds(2, 25, 51), 50..51);
    assert_eq!(page_bounds(9, 25, 51), 50..51);
    assert_eq!(clamp_page(9, 25, 51), 2);
    assert_eq!(clamp_page(9, 25, 0), 0);
}

#[test]
fn dataset_transition_resets_only_page_and_preserves_controlled_preferences() {
    let owner = Owner::new();
    owner.with(|| {
        let mut supplied = EntityTablePreferences::new(1);
        supplied.page_size = 50;
        supplied.sort = EntitySort::descending("rank");
        supplied.hidden_columns.insert("office".to_owned());
        supplied.column_widths.insert("office".to_owned(), 240);
        let current = RwSignal::new(supplied.clone());
        let emissions = Arc::new(Mutex::new(Vec::new()));
        let emissions_for_callback = Arc::clone(&emissions);
        let preferences = super::component::PreferenceState::new(
            EntityTablePreferenceOwnership::controlled(
                current.into(),
                Callback::new(move |replacement| {
                    emissions_for_callback
                        .lock()
                        .expect("controlled callback lock is available")
                        .push(replacement);
                }),
            ),
            StoredValue::new_local(columns()),
            1,
        );
        let current_page = RwSignal::new(3);
        let controller =
            super::component::DatasetTransitionController::new(current_page, preferences);

        controller.apply("office-1".to_owned(), "office-2".to_owned());

        assert_eq!(current_page.get(), 0, "a new dataset starts on page one");
        assert_eq!(
            current.get(),
            supplied,
            "dataset selection must preserve every supplied preference"
        );
        // Killed mutation: resetting preferences inside the real transition
        // controller made this callback collection non-empty.
        assert!(
            emissions
                .lock()
                .expect("controlled callback lock is available")
                .is_empty(),
            "dataset selection must not emit a preference replacement"
        );
    });
}

#[test]
fn row_deltas_preserve_a_valid_page_and_clamp_an_invalid_page() {
    assert_eq!(page_after_row_delta(2, 25, 74), 2);
    assert_eq!(page_after_row_delta(2, 25, 49), 1);
}

#[test]
fn controlled_page_size_change_reasserts_a_synchronously_accepted_value() {
    let owner = Owner::new();
    owner.with(|| {
        let current = RwSignal::new(EntityTablePreferences::new(1));
        let preferences = super::component::PreferenceState::new(
            EntityTablePreferenceOwnership::controlled(
                current.into(),
                Callback::new(move |replacement| current.set(replacement)),
            ),
            StoredValue::new_local(columns()),
            1,
        );
        let current_page = RwSignal::new(3);
        let live_value = RefCell::new(None::<String>);

        super::component::apply_page_size_change(preferences, current_page, "50", |value| {
            live_value.replace(Some(value));
        });

        assert_eq!(current.get().page_size, 50);
        assert_eq!(current_page.get(), 0);
        assert_eq!(live_value.into_inner().as_deref(), Some("50"));
    });
}

#[test]
fn controlled_page_size_change_restores_a_declined_or_delayed_value() {
    let owner = Owner::new();
    owner.with(|| {
        let current = RwSignal::new(EntityTablePreferences::new(1));
        let emitted = Arc::new(Mutex::new(Vec::new()));
        let emitted_for_callback = Arc::clone(&emitted);
        let preferences = super::component::PreferenceState::new(
            EntityTablePreferenceOwnership::controlled(
                current.into(),
                Callback::new(move |replacement| {
                    emitted_for_callback
                        .lock()
                        .expect("controlled callback lock is available")
                        .push(replacement);
                }),
            ),
            StoredValue::new_local(columns()),
            1,
        );
        let current_page = RwSignal::new(3);
        let live_value = RefCell::new(None::<String>);

        super::component::apply_page_size_change(preferences, current_page, "50", |value| {
            live_value.replace(Some(value));
        });

        assert_eq!(
            current.get().page_size,
            25,
            "the consumer remains the controlled source of truth"
        );
        assert_eq!(current_page.get(), 0);
        // Killed mutation: reasserting requested `50` instead of rereading the
        // controlled source changed this value to `Some("50")`.
        assert_eq!(
            live_value.into_inner().as_deref(),
            Some("25"),
            "the native select must immediately return to the supplied value"
        );
        let emitted = emitted
            .lock()
            .expect("controlled callback lock is available");
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].page_size, 50);
    });
}

#[test]
fn controlled_change_reads_current_signal_and_emits_one_normalized_replacement() {
    let owner = Owner::new();
    owner.with(|| {
        let mut initial = EntityTablePreferences::new(1);
        initial.page_size = 999;
        initial.sort = EntitySort::ascending("missing");
        initial.hidden_columns.insert("client".to_owned());
        initial.hidden_columns.insert("missing".to_owned());
        initial.column_widths.insert("missing".to_owned(), 300);
        let current = RwSignal::new(initial);
        let emitted = Arc::new(Mutex::new(Vec::new()));
        let emitted_for_callback = Arc::clone(&emitted);
        let ownership = EntityTablePreferenceOwnership::controlled(
            current.into(),
            Callback::new(move |replacement| {
                emitted_for_callback
                    .lock()
                    .expect("controlled callback lock is available")
                    .push(replacement);
            }),
        );
        let state =
            super::component::PreferenceState::new(ownership, StoredValue::new_local(columns()), 1);

        let normalized_initial = state.get();
        assert_eq!(normalized_initial.page_size, 25);
        assert_eq!(normalized_initial.sort, EntitySort::System);
        assert!(normalized_initial.hidden_columns.is_empty());
        assert!(normalized_initial.column_widths.is_empty());

        let mut supplied = EntityTablePreferences::new(1);
        supplied.page_size = 50;
        supplied.sort = EntitySort::ascending("rank");
        supplied.column_order = vec![
            "client".to_owned(),
            "rank".to_owned(),
            "office".to_owned(),
            "actions".to_owned(),
        ];
        current.set(supplied.clone());
        assert_eq!(
            state.get(),
            supplied,
            "controlled rendering follows the consumer signal"
        );

        state.update(|next| {
            next.sort = EntitySort::descending("rank");
            next.hidden_columns.insert("office".to_owned());
            next.hidden_columns.insert("missing".to_owned());
        });

        assert_eq!(
            current.get(),
            supplied,
            "controlled ownership never mutates the consumer signal"
        );
        assert_eq!(
            state.get(),
            supplied,
            "controlled rendering keeps the consumer value until it accepts a replacement"
        );
        let emitted = emitted
            .lock()
            .expect("controlled callback lock is available");
        assert_eq!(emitted.len(), 1, "one UI operation emits one replacement");
        assert_eq!(emitted[0].schema_version, 1);
        assert_eq!(emitted[0].page_size, 50);
        assert_eq!(emitted[0].sort, EntitySort::descending("rank"));
        assert_eq!(
            emitted[0]
                .hidden_columns
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["office"]
        );
        assert!(emitted[0].column_widths.is_empty());
    });
}

#[test]
fn declined_controlled_width_reset_restores_consumer_rendered_widths() {
    let owner = Owner::new();
    owner.with(|| {
        let mut supplied = EntityTablePreferences::new(1);
        supplied.column_widths.insert("client".to_owned(), 180);
        let current = RwSignal::new(supplied.clone());
        let emitted = Arc::new(Mutex::new(Vec::new()));
        let emitted_for_callback = Arc::clone(&emitted);
        let state = super::component::PreferenceState::new(
            EntityTablePreferenceOwnership::controlled(
                current.into(),
                Callback::new(move |replacement| {
                    emitted_for_callback
                        .lock()
                        .expect("controlled callback lock is available")
                        .push(replacement);
                }),
            ),
            StoredValue::new_local(columns()),
            1,
        );

        let rendered = state.update_and_rendered_widths(|preferences| {
            reset_columns(preferences);
        });

        assert_eq!(
            rendered.get("client"),
            Some(&180),
            "a declined controlled reset cannot leave an optimistic width mirror"
        );
        assert_eq!(current.get(), supplied);
        let emitted = emitted
            .lock()
            .expect("controlled callback lock is available");
        assert_eq!(emitted.len(), 1);
        assert!(emitted[0].column_widths.is_empty());
    });
}

#[test]
fn disabled_persistence_never_invokes_storage() {
    let reads = Cell::new(0);
    let writes = Cell::new(0);
    let persistence = EntityTablePreferencePersistence::Disabled;

    let loaded = super::storage::load_preferences_with(persistence, 1, &columns(), |_| {
        reads.set(reads.get() + 1);
        Some(String::new())
    });
    super::storage::save_preferences_with(persistence, &loaded, |_, _| {
        writes.set(writes.get() + 1);
    });

    assert_eq!(loaded, EntityTablePreferences::new(1));
    assert_eq!(reads.get(), 0, "persistence-off must not read localStorage");
    assert_eq!(
        writes.get(),
        0,
        "persistence-off must not write localStorage"
    );
}

#[test]
fn legacy_local_storage_keeps_prefixed_read_write_behavior() {
    let persistence = EntityTablePreferencePersistence::LegacyLocalStorage {
        storage_key: "compatibility",
    };
    let mut expected = EntityTablePreferences::new(4);
    expected.column_order = vec![
        "client".to_owned(),
        "rank".to_owned(),
        "office".to_owned(),
        "actions".to_owned(),
    ];
    let encoded = encode_preferences(&expected).unwrap();
    let read_key = RefCell::new(None::<String>);

    let loaded = super::storage::load_preferences_with(persistence, 4, &columns(), |key| {
        read_key.replace(Some(key.to_owned()));
        Some(encoded.clone())
    });
    let written = RefCell::new(None::<(String, String)>);
    super::storage::save_preferences_with(persistence, &loaded, |key, payload| {
        written.replace(Some((key.to_owned(), payload.to_owned())));
    });

    assert_eq!(loaded, expected);
    assert_eq!(
        read_key.into_inner().as_deref(),
        Some("ldui-entity-table:compatibility")
    );
    let (key, payload) = written.into_inner().expect("legacy write is retained");
    assert_eq!(key, "ldui-entity-table:compatibility");
    assert_eq!(decode_preferences(&payload, 4, &columns()), expected);
}

#[test]
fn legacy_storage_key_prop_resolves_to_uncontrolled_compatibility_mode() {
    let ownership =
        super::component::resolve_preference_ownership(None, Some("legacy-component-prop"));

    assert!(matches!(
        ownership,
        EntityTablePreferenceOwnership::Uncontrolled {
            persistence: EntityTablePreferencePersistence::LegacyLocalStorage {
                storage_key: "legacy-component-prop"
            }
        }
    ));
}

#[test]
#[should_panic(
    expected = "EntityTable configuration cannot combine preference_ownership with storage_key"
)]
fn explicit_ownership_and_legacy_storage_key_fail_closed() {
    let ownership = EntityTablePreferenceOwnership::Uncontrolled {
        persistence: EntityTablePreferencePersistence::Disabled,
    };
    let _ = super::component::resolve_preference_ownership(
        Some(ownership),
        Some("legacy-must-not-be-ignored"),
    );
}

#[test]
fn opinionated_page_sizes_are_fixed() {
    assert_eq!(ENTITY_PAGE_SIZE_CHOICES, [25, 50, 100]);
    assert!(valid_page_size(25));
    assert!(valid_page_size(50));
    assert!(valid_page_size(100));
    assert!(!valid_page_size(10));
}

#[test]
fn entity_column_text_overflow_builders_are_typed_and_source_local() {
    let wrapped = EntityColumn::text("name", "Name", |row: &Row| row.name.to_owned());
    let ellipsis = wrapped.clone().ellipsis();
    let clamped = wrapped.clone().line_clamp(2);

    assert_eq!(wrapped.text_overflow, EntityTextOverflow::Wrap);
    assert_eq!(ellipsis.text_overflow, EntityTextOverflow::Ellipsis);
    assert_eq!(
        clamped.text_overflow,
        EntityTextOverflow::LineClamp(std::num::NonZeroU8::new(2).unwrap())
    );
    assert!(entity_text_overflow_style(ellipsis.text_overflow).contains("text-overflow:ellipsis"));
    assert!(entity_text_overflow_style(clamped.text_overflow).contains("-webkit-line-clamp:2"));
}

#[test]
#[should_panic(expected = "EntityColumn line clamp must be positive")]
fn entity_column_line_clamp_rejects_zero() {
    let _ = EntityColumn::text("name", "Name", |row: &Row| row.name.to_owned()).line_clamp(0);
}

#[test]
fn rich_renderer_keeps_visual_precedence_over_text_overflow_metadata() {
    let column = EntityColumn::text("name", "Name", |row: &Row| row.name.to_owned())
        .line_clamp(2)
        .render_with(|row: &Row| row.name.to_owned().into_any());

    assert!(column.renderer.is_some());
    assert_eq!(column.text_overflow.as_str(), "line-clamp");
    assert_eq!((column.text)(&rows()[0]), "Zulu");
}

#[test]
fn entity_column_alignment_and_tabular_number_builders_are_typed() {
    let base = EntityColumn::text("amount", "Amount", |row: &Row| row.rank.to_string());
    assert_eq!(base.alignment, EntityColumnAlignment::Auto);
    assert!(!base.tabular_numbers);

    let centered = base.clone().align_center();
    assert_eq!(centered.alignment, EntityColumnAlignment::Center);
    let started = base.clone().align_start();
    assert_eq!(started.alignment, EntityColumnAlignment::Start);
    let numeric = base.align_end().tabular_numbers();
    assert_eq!(numeric.alignment, EntityColumnAlignment::End);
    assert!(numeric.tabular_numbers);
    assert_eq!(
        entity_alignment_class(EntityColumnAlignment::Start),
        "text-left"
    );
    assert_eq!(
        entity_alignment_class(EntityColumnAlignment::Center),
        "text-center"
    );
    assert_eq!(
        entity_alignment_class(EntityColumnAlignment::End),
        "text-right"
    );
    assert_eq!(
        entity_header_justify_class(EntityColumnAlignment::End),
        "justify-end"
    );
}

#[test]
fn rich_renderer_retains_wrapper_level_numeric_presentation() {
    let column = EntityColumn::text("amount", "Amount", |row: &Row| row.rank.to_string())
        .align_end()
        .tabular_numbers()
        .render_with(|_| ().into_any());

    assert_eq!(column.alignment, EntityColumnAlignment::End);
    assert!(column.tabular_numbers);
    assert!(column.renderer.is_some());
}

#[test]
fn semantic_badge_and_icon_builders_keep_canonical_text_source_local() {
    let badge =
        EntityColumn::text("status", "Status", |row: &Row| row.name.to_owned()).badge_with(|row| {
            (row.rank > 1).then(|| {
                EntityBadgePresentation::new(BadgeColor::Warning).with_style(BadgeStyle::Outline)
            })
        });
    let icon =
        EntityColumn::text("state", "State", |row: &Row| row.name.to_owned()).icon_with(|row| {
            (row.rank > 1)
                .then(|| EntityIconPresentation::new("triangle-alert", EntityIconColor::Warning))
        });

    assert_eq!((badge.text)(&rows()[0]), "Zulu");
    assert_eq!((icon.text)(&rows()[0]), "Zulu");
    assert!(matches!(
        badge.presentation,
        Some(EntityCellPresentation::Badge(_))
    ));
    assert!(matches!(
        icon.presentation,
        Some(EntityCellPresentation::Icon(_))
    ));
}

#[test]
fn semantic_cell_configuration_has_opinionated_defaults_and_rich_precedence() {
    let badge = EntityBadgePresentation::new(BadgeColor::Success);
    assert_eq!(badge.color, BadgeColor::Success);
    assert_eq!(badge.style, BadgeStyle::Soft);
    assert_eq!(EntityIconColor::Error.as_class(), "text-error");

    let column = EntityColumn::text("status", "Status", |row: &Row| row.name.to_owned())
        .badge_with(|_| Some(EntityBadgePresentation::new(BadgeColor::Info)))
        .render_with(|_| ().into_any());
    assert!(column.presentation.is_some());
    assert!(
        column.renderer.is_some(),
        "render_with remains the visual winner"
    );
}

#[test]
fn normalize_entity_secondary_text_folds_empty_and_whitespace_to_none() {
    assert_eq!(normalize_entity_secondary_text(None), None);
    assert_eq!(normalize_entity_secondary_text(Some(String::new())), None);
    assert_eq!(
        normalize_entity_secondary_text(Some("   ".to_owned())),
        None,
        "a whitespace-only secondary line must normalize away, leaving no line"
    );
    assert_eq!(
        normalize_entity_secondary_text(Some("Team lead".to_owned())),
        Some("Team lead".to_owned())
    );
}

#[test]
fn primary_secondary_builder_keeps_canonical_text_local_and_complete() {
    let column = EntityColumn::text("who", "Who", |row: &Row| {
        format!("{} ({})", row.name, row.id)
    })
    .primary_secondary(
        |row: &Row| row.name.to_owned(),
        |row: &Row| match row.id {
            "r1" => Some(row.id.to_owned()),
            "r2" => Some("   ".to_owned()),
            _ => None,
        },
    );

    let sample = rows();
    // The canonical `text` callback -- the accessible/export value -- stays
    // complete and untouched by the visual primary/secondary split.
    assert_eq!((column.text)(&sample[0]), "Zulu (r1)");
    assert_eq!((column.text)(&sample[1]), "Alpha (r2)");
    assert_eq!((column.text)(&sample[2]), "Bravo (r3)");

    let Some(EntityCellPresentation::PrimarySecondary { primary, secondary }) =
        column.presentation.as_ref()
    else {
        panic!("expected a PrimarySecondary presentation");
    };
    assert_eq!(primary(&sample[0]), "Zulu");
    assert_eq!(primary(&sample[1]), "Alpha");
    assert_eq!(secondary(&sample[0]), Some("r1".to_owned()));
    assert_eq!(
        normalize_entity_secondary_text(secondary(&sample[1])),
        None,
        "an empty or whitespace-only secondary line renders no secondary line"
    );
    assert_eq!(normalize_entity_secondary_text(secondary(&sample[2])), None);
}

#[test]
fn primary_secondary_does_not_disturb_default_or_typed_sort_keys() {
    // The default text-based sort key survives adding a primary/secondary presentation.
    let default_sorted = EntityColumn::text("who", "Who", |row: &Row| row.name.to_owned())
        .primary_secondary(|row: &Row| row.name.to_owned(), |_| None);
    assert!(default_sorted.sort_key.is_some());
    assert!(default_sorted.comparator.is_none());

    let rows = rows();
    let columns = vec![default_sorted];
    assert_eq!(
        sorted_indices(&rows, &columns, &EntitySort::ascending("who")),
        [1, 2, 0],
        "sort must still follow the canonical text key, not the primary/secondary split"
    );

    // A typed sort key set before primary_secondary must survive it.
    let typed = EntityColumn::text("rank", "Rank", |row: &Row| row.rank.to_string())
        .sortable_by_key(|row: &Row| row.rank)
        .primary_secondary(|row: &Row| row.rank.to_string(), |_| None);
    assert!(typed.sort_key.is_some());
    assert!(typed.comparator.is_none());
}

#[test]
fn primary_secondary_rich_renderer_still_wins() {
    let column = EntityColumn::text("who", "Who", |row: &Row| row.name.to_owned())
        .primary_secondary(|row: &Row| row.name.to_owned(), |_| None)
        .render_with(|_| ().into_any());
    assert!(column.presentation.is_some());
    assert!(
        column.renderer.is_some(),
        "render_with remains the visual winner over primary_secondary"
    );
}

/// Builds a column whose secondary line is relabeled by `role_label`,
/// mirroring the locale-style relabel the `entity-table-presentation` demo
/// fixture performs when its `columns` prop is replaced.
fn locale_labeled_column(role_label: &'static str) -> EntityColumn<Row> {
    EntityColumn::text("who", "Who", |row: &Row| row.name.to_owned()).primary_secondary(
        |row: &Row| row.name.to_owned(),
        move |row: &Row| Some(format!("{role_label}: {}", row.id)),
    )
}

#[test]
fn primary_secondary_reflects_a_columns_signal_replacement() {
    // The established meaning of "column replacement" in this codebase is
    // swapping the whole `columns` prop for a new `Signal<Vec<EntityColumn<T>>>`
    // value (`EntityColumns::Reactive`, consumed as `ColumnStore::Reactive`
    // in component.rs) -- not mutating row data through static columns. This
    // proves a primary_secondary presentation's closures come from whatever
    // Vec the columns Signal currently holds, so replacing it changes both
    // lines, exactly as `EntityColumn::text` header/text closures already do.
    let owner = Owner::new();
    owner.with(|| {
        let source = RwSignal::new_local(vec![locale_labeled_column("Role")]);
        let EntityColumns::Reactive(reactive) = EntityColumns::from(Signal::from(source)) else {
            panic!("Signal::from(source).into() must produce EntityColumns::Reactive");
        };

        let sample = rows();
        let read_secondary = |snapshot: &[EntityColumn<Row>]| {
            let Some(EntityCellPresentation::PrimarySecondary { secondary, .. }) =
                snapshot[0].presentation.as_ref()
            else {
                panic!("expected a PrimarySecondary presentation");
            };
            secondary(&sample[0])
        };

        assert_eq!(read_secondary(&reactive.get()), Some("Role: r1".to_owned()));

        source.set(vec![locale_labeled_column("Rol")]);

        assert_eq!(
            read_secondary(&reactive.get()),
            Some("Rol: r1".to_owned()),
            "replacing the columns Signal must be reflected in the presentation's secondary closure"
        );
    });
}

#[test]
fn column_chooser_trigger_defaults_to_localized_text() {
    assert_eq!(
        EntityColumnChooserTrigger::default(),
        EntityColumnChooserTrigger::Text
    );
}

#[test]
fn responsive_filter_metadata_keeps_label_activity_and_clear_intent_caller_owned() {
    let owner = Owner::new();
    owner.with(|| {
        let label = RwSignal::new("Workflow status".to_owned());
        let active = RwSignal::new(true);
        let clear_count = Arc::new(Mutex::new(0_u32));
        let observed_count = Arc::clone(&clear_count);
        let filter = EntityColumnFilter::new("status", || "filter".into_any()).with_responsive(
            label,
            active,
            Callback::new(move |_| *observed_count.lock().unwrap() += 1),
        );

        assert_eq!(filter.label("Fallback"), "Workflow status");
        assert!(filter.is_active());
        filter.clear();
        assert_eq!(*clear_count.lock().unwrap(), 1);

        label.set("Estado del flujo".to_owned());
        active.set(false);
        assert_eq!(filter.label("Fallback"), "Estado del flujo");
        assert!(!filter.is_active());
    });

    let compatibility = EntityColumnFilter::new("status", || "filter".into_any());
    assert_eq!(compatibility.label("Status"), "Status");
    assert!(!compatibility.is_active());
}

#[test]
fn controlled_text_filter_derives_activity_and_clear_from_the_supplied_value() {
    let owner = Owner::new();
    owner.with(|| {
        let label = RwSignal::new("Client".to_owned());
        let value = RwSignal::new("Ada".to_owned());
        let proposals = Arc::new(Mutex::new(Vec::<String>::new()));
        let observed = Arc::clone(&proposals);
        let filter = EntityColumnFilter::text(
            "client",
            "client-name-filter",
            label,
            value,
            "Filter clients",
            Callback::new(move |next| observed.lock().unwrap().push(next)),
        );

        assert_eq!(filter.control_id(), Some("client-name-filter"));
        assert_eq!(filter.label("Fallback"), "Client");
        assert!(filter.is_active());

        filter.clear();
        assert_eq!(&*proposals.lock().unwrap(), &[String::new()]);
        assert!(
            filter.is_active(),
            "a rejected clear proposal must not disagree with the controlled value"
        );

        value.set(String::new());
        assert!(!filter.is_active());
        label.set("Cliente".to_owned());
        assert_eq!(filter.label("Fallback"), "Cliente");
    });
}

#[test]
fn controlled_select_filter_keeps_value_identity_separate_from_reactive_labels() {
    let owner = Owner::new();
    owner.with(|| {
        let value = RwSignal::new("ready".to_owned());
        let options = RwSignal::new(vec![
            EntityColumnFilterOption::new("ready", "Ready"),
            EntityColumnFilterOption::new("urgent", "Urgent"),
        ]);
        let proposals = Arc::new(Mutex::new(Vec::<String>::new()));
        let observed = Arc::clone(&proposals);
        let filter = EntityColumnFilter::select(
            "status",
            "workflow-status-filter",
            "Status",
            value,
            "All statuses",
            options,
            Callback::new(move |next| observed.lock().unwrap().push(next)),
        );

        assert_eq!(filter.control_id(), Some("workflow-status-filter"));
        assert!(filter.is_active());
        options.set(vec![
            EntityColumnFilterOption::new("ready", "Listo"),
            EntityColumnFilterOption::new("urgent", "Urgente"),
        ]);
        assert_eq!(value.get(), "ready");

        filter.clear();
        assert_eq!(&*proposals.lock().unwrap(), &[String::new()]);
        assert_eq!(value.get(), "ready");
        value.set(String::new());
        assert!(!filter.is_active());
    });
}

#[test]
#[should_panic(expected = "EntityColumnFilter control_id must not be empty")]
fn controlled_filter_rejects_an_empty_dom_identity() {
    let owner = Owner::new();
    owner.with(|| {
        let _ = EntityColumnFilter::text(
            "client",
            "   ",
            "Client",
            RwSignal::new(String::new()),
            "Filter clients",
            Callback::new(|_| {}),
        );
    });
}

#[test]
fn display_projection_uses_render_order_visibility_sort_and_canonical_text() {
    let rows = rows();
    let mut columns = columns();
    columns[0] = columns[0]
        .clone()
        .render_with(|_| "decorative client markup".into_any());
    let mut preferences = EntityTablePreferences::new(1);
    preferences.page_size = 2;
    preferences.sort = EntitySort::ascending("rank");
    preferences.column_order = ["rank", "client", "office", "actions"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    preferences.hidden_columns.insert("office".to_owned());

    let projection = entity_table_display_projection(
        &rows,
        &columns,
        &preferences,
        0,
        2,
        &|row: &Row| row.id.to_owned(),
        EntityTableActionColumnPolicy::Exclude,
    );

    assert_eq!(
        projection.columns,
        [
            EntityTableDisplayColumn::new("rank", "Rank", false),
            EntityTableDisplayColumn::new("client", "Client", false),
        ]
    );
    assert_eq!(
        projection.rows(EntityTableProjectionScope::AllFiltered),
        [
            EntityTableDisplayRow::new("r2", ["1", "Alpha"]),
            EntityTableDisplayRow::new("r3", ["1", "Bravo"]),
            EntityTableDisplayRow::new("r1", ["2", "Zulu"]),
        ]
    );
    assert_eq!(
        projection.rows(EntityTableProjectionScope::CurrentPage),
        [
            EntityTableDisplayRow::new("r2", ["1", "Alpha"]),
            EntityTableDisplayRow::new("r3", ["1", "Bravo"]),
        ]
    );
}

#[test]
fn display_projection_current_page_and_action_opt_in_are_explicit() {
    let rows = rows();
    let columns = columns();
    let mut preferences = EntityTablePreferences::new(1);
    preferences.page_size = 2;

    let projection = entity_table_display_projection(
        &rows,
        &columns,
        &preferences,
        1,
        2,
        &|row: &Row| row.id.to_owned(),
        EntityTableActionColumnPolicy::Include,
    );

    assert_eq!(
        projection
            .columns
            .iter()
            .map(|column| (column.id.as_str(), column.is_action))
            .collect::<Vec<_>>(),
        [
            ("client", false),
            ("rank", false),
            ("office", false),
            ("actions", true),
        ]
    );
    assert_eq!(
        projection.rows(EntityTableProjectionScope::CurrentPage),
        [EntityTableDisplayRow::new(
            "r3",
            ["Bravo", "1", "r3", "Claim"]
        )]
    );
    assert_eq!(
        projection
            .rows(EntityTableProjectionScope::AllFiltered)
            .len(),
        3
    );
}

#[test]
fn viewport_fit_policy_is_explicit_and_does_not_replace_the_preference() {
    let bounded = EntityTableViewportFit::max_height("calc(100vh - 12rem)").with_min_rows(7);
    assert_eq!(bounded.height(), Some("calc(100vh - 12rem)"));
    assert_eq!(bounded.min_rows(), 7);

    let fill_parent = EntityTableViewportFit::fill_parent().with_min_rows(0);
    assert_eq!(fill_parent.height(), None);
    assert_eq!(fill_parent.min_rows(), 1);

    let preferences = EntityTablePreferences::new(1);
    assert_eq!(
        super::component::effective_page_size(Some(11), preferences.page_size),
        11
    );
    assert_eq!(
        super::component::effective_page_size(None, preferences.page_size),
        25
    );
    assert_eq!(preferences.page_size, 25);
}

#[test]
fn widths_are_clamped_by_shared_table_bounds() {
    let mut preferences = EntityTablePreferences::new(3);
    set_preferred_width(&mut preferences, "office", 12.0, Some(80));
    assert_eq!(preferences.column_widths["office"], 80);
    set_preferred_width(&mut preferences, "office", 20_000.0, None);
    assert_eq!(preferences.column_widths["office"], 1_200);
}

#[test]
fn required_columns_cannot_be_hidden() {
    let columns = columns();
    let mut preferences = EntityTablePreferences::new(1);
    assert!(!toggle_hidden_column(&mut preferences, &columns, "client"));
    assert!(!toggle_hidden_column(&mut preferences, &columns, "actions"));
    assert!(toggle_hidden_column(&mut preferences, &columns, "office"));
    assert!(preferences.hidden_columns.contains("office"));
    assert!(toggle_hidden_column(&mut preferences, &columns, "office"));
    assert!(!preferences.hidden_columns.contains("office"));
}

#[test]
fn explicit_resets_are_independent_and_preserve_page_size() {
    let mut preferences = EntityTablePreferences::new(7);
    preferences.page_size = 100;
    preferences.sort = EntitySort::descending("rank");
    preferences.hidden_columns.insert("office".to_owned());
    preferences.column_widths.insert("office".to_owned(), 320);

    assert!(reset_sort(&mut preferences));
    assert_eq!(preferences.sort, EntitySort::System);
    assert_eq!(preferences.page_size, 100);
    assert!(preferences.hidden_columns.contains("office"));
    assert_eq!(preferences.column_widths["office"], 320);
    assert!(!reset_sort(&mut preferences));

    assert!(reset_columns(&mut preferences));
    assert!(preferences.hidden_columns.is_empty());
    assert!(preferences.column_widths.is_empty());
    assert_eq!(preferences.page_size, 100);
    assert_eq!(preferences.sort, EntitySort::System);
    assert!(!reset_columns(&mut preferences));
}

#[test]
fn stored_preferences_are_versioned_and_normalized() {
    let columns = columns();
    let mut preferences = EntityTablePreferences::new(4);
    preferences.page_size = 50;
    preferences.sort = EntitySort::descending("rank");
    preferences.column_order = vec![
        "client".to_owned(),
        "rank".to_owned(),
        "office".to_owned(),
        "actions".to_owned(),
    ];
    preferences.hidden_columns.insert("office".to_owned());
    preferences.column_widths.insert("office".to_owned(), 280);

    let encoded = encode_preferences(&preferences).unwrap();
    assert_eq!(decode_preferences(&encoded, 4, &columns), preferences);

    let stale = decode_preferences(&encoded, 5, &columns);
    assert_eq!(stale.column_order, ["client", "rank", "office", "actions"]);
    assert_eq!(stale.schema_version, 5);
    let invalid = decode_preferences("not-json", 4, &columns);
    assert_eq!(
        invalid.column_order,
        ["client", "rank", "office", "actions"]
    );
    assert_eq!(invalid.schema_version, 4);
}

#[test]
fn unknown_and_required_stored_columns_are_pruned() {
    let payload = r#"{
        "schema_version":1,
        "page_size":999,
        "sort":{"Ascending":{"column":"missing"}},
        "hidden_columns":["client","office","missing"],
        "column_widths":{"office":1,"missing":2000}
    }"#;
    let decoded = decode_preferences(payload, 1, &columns());
    assert_eq!(decoded.page_size, 25);
    assert_eq!(decoded.sort, EntitySort::System);
    assert_eq!(
        decoded.hidden_columns.into_iter().collect::<Vec<_>>(),
        ["office"]
    );
    assert_eq!(decoded.column_widths["office"], 48);
    assert!(!decoded.column_widths.contains_key("missing"));
}

#[test]
fn normalization_is_pure_and_deterministic() {
    let columns = columns();
    let mut supplied = EntityTablePreferences::new(8);
    supplied.page_size = 999;
    supplied.sort = EntitySort::ascending("missing");
    supplied.hidden_columns.insert("client".to_owned());
    supplied.hidden_columns.insert("office".to_owned());
    supplied.hidden_columns.insert("missing".to_owned());
    supplied.column_widths.insert("office".to_owned(), 1);
    supplied.column_widths.insert("missing".to_owned(), 2_000);
    let original = supplied.clone();

    let first = normalize_preferences(&supplied, 8, &columns);
    let second = normalize_preferences(&supplied, 8, &columns);

    assert_eq!(
        supplied, original,
        "normalization must not mutate its input"
    );
    assert_eq!(first, second, "normalization must be deterministic");
    assert_eq!(first.schema_version, 8);
    assert_eq!(first.page_size, 25);
    assert_eq!(first.sort, EntitySort::System);
    assert_eq!(
        first
            .hidden_columns
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["office"]
    );
    assert_eq!(first.column_widths["office"], 48);
    assert!(!first.column_widths.contains_key("missing"));
}

#[test]
fn normalization_prunes_duplicate_and_unknown_sort_clauses_first_wins() {
    let mut supplied = EntityTablePreferences::new(1);
    supplied.sort = EntitySort::multiple([
        EntitySortColumn::ascending("rank"),
        EntitySortColumn::descending("rank"),
        EntitySortColumn::ascending("missing"),
        EntitySortColumn::descending("actions"),
        EntitySortColumn::descending("client"),
    ]);
    supplied.column_order = vec![
        "office".to_owned(),
        "office".to_owned(),
        "missing".to_owned(),
    ];
    supplied.hidden_columns.insert("client".to_owned());

    let normalized = normalize_preferences(&supplied, 1, &columns());

    assert_eq!(
        normalized.sort.clauses(),
        [
            EntitySortColumn::ascending("rank"),
            EntitySortColumn::descending("client"),
        ]
    );
    assert_eq!(
        normalized.column_order,
        ["office", "client", "rank", "actions"]
    );
    assert!(
        !normalized.hidden_columns.contains("client"),
        "required columns must be restored"
    );
}

#[test]
fn canonical_sort_clause_array_round_trips() {
    let canonical = serde_json::json!([
        {"column": "rank", "direction": "ascending"},
        {"column": "client", "direction": "descending"}
    ]);

    let decoded: EntitySort = serde_json::from_value(canonical.clone())
        .expect("canonical multi-column sort must deserialize");

    assert_eq!(serde_json::to_value(decoded).unwrap(), canonical);
}

#[test]
fn legacy_single_sort_payload_migrates_to_canonical_clause_array() {
    let decoded: EntitySort = serde_json::from_value(serde_json::json!({
        "Descending": {"column": "rank"}
    }))
    .expect("legacy single-column sort must remain readable");

    assert_eq!(
        serde_json::to_value(decoded).unwrap(),
        serde_json::json!([{"column": "rank", "direction": "descending"}])
    );
}

#[test]
fn legacy_preferences_without_column_order_normalize_to_declared_order() {
    let payload = r##"{
        "schema_version":1,
        "page_size":25,
        "sort":"System",
        "hidden_columns":[],
        "column_widths":{}
    }"##;

    let decoded = decode_preferences(payload, 1, &columns());
    let encoded = serde_json::to_value(decoded).unwrap();

    assert_eq!(
        encoded["column_order"],
        serde_json::json!(["client", "rank", "office", "actions"])
    );
}

// ldui-kl55: EntityTable's framework-owned page-size `<select>` had no
// id/name at all when the caller omitted `page_size_control_id`, and two
// tables on one page (the Office satellites' Setup page) could not be told
// apart. `next_entity_page_size_id` is the per-instance default generator
// wired into that select's `id` and `name`; these tests pin the generator's
// contract directly, since the reactive Select mount itself needs a browser
// (see tests/entity_table_smoke.rs's
// page_size_select_gets_unique_identity_without_an_override_and_honors_one
// for the DOM-level proof).
#[test]
fn page_size_default_ids_are_non_empty_and_unique() {
    let a = next_entity_page_size_id();
    let b = next_entity_page_size_id();
    assert!(!a.is_empty());
    assert!(!b.is_empty());
    assert_ne!(
        a, b,
        "two EntityTable instances must not share a default id"
    );
    assert!(a.starts_with("ldui-entity-page-size-"));
    assert!(b.starts_with("ldui-entity-page-size-"));
}

// ldui-mqb: typed summary-row emphasis. `EntityRowEmphasis` and its pure
// class/lookup functions live in `emphasis.rs` and carry their own focused
// unit tests; these two exercise the acceptance criterion this repo's
// `entity_table` fixture files are set up to prove directly -- that
// classification is keyed to a row's identity (via the table's mandatory
// `row_key`), not to whichever index a sort happens to put it at.

#[test]
fn row_classification_survives_a_sort_that_moves_the_row() {
    let rows = rows();
    let columns = columns();
    // `r1` is rank 2 (the other two rows are rank 1), so ascending and
    // descending sorts by `rank` put it at opposite ends -- a real change of
    // rendered position, not a no-op sort.
    let classifier: EntityRowEmphasisClassifier<Row> = Rc::new(|row: &Row| {
        if row.id == "r1" {
            EntityRowEmphasis::Summary
        } else {
            EntityRowEmphasis::Standard
        }
    });

    let ascending = sorted_indices(&rows, &columns, &EntitySort::ascending("rank"));
    let descending = sorted_indices(&rows, &columns, &EntitySort::descending("rank"));
    let ascending_position = ascending
        .iter()
        .position(|&index| rows[index].id == "r1")
        .expect("r1 must still be present after an ascending sort");
    let descending_position = descending
        .iter()
        .position(|&index| rows[index].id == "r1")
        .expect("r1 must still be present after a descending sort");
    assert_ne!(
        ascending_position, descending_position,
        "the fixture must actually move r1 between sorts, or this test proves nothing"
    );

    // Wherever r1 lands, it classifies Summary; every other row classifies
    // Standard -- in both sort orders, at every index.
    for indices in [&ascending, &descending] {
        for &index in indices {
            let row = &rows[index];
            let expected = if row.id == "r1" {
                EntityRowEmphasis::Summary
            } else {
                EntityRowEmphasis::Standard
            };
            assert_eq!(
                entity_row_emphasis_for(Some(&classifier), Some(row)),
                expected,
                "classification must follow row identity at index {index}, not rendered position"
            );
        }
    }
}

#[test]
fn no_row_emphasis_classifier_renders_every_row_identically_to_a_table_without_the_prop() {
    // Proven at the pure-function level, composed exactly as
    // `render_keyed_row`/`render_row_cells` use it: with no classifier at
    // all, every row resolves to `Standard`, which in turn contributes the
    // empty string to both the `<tr>` class and every `<td>` class -- the
    // same DOM a table predating `row_emphasis` renders.
    for row in rows() {
        assert_eq!(
            entity_row_emphasis_for::<Row>(None, Some(&row)),
            EntityRowEmphasis::Standard
        );
    }
    assert_eq!(
        entity_row_emphasis_row_class(EntityRowEmphasis::Standard),
        ""
    );
    assert_eq!(
        entity_row_emphasis_cell_class(EntityRowEmphasis::Standard),
        ""
    );
}
