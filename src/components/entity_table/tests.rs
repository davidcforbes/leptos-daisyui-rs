use super::*;
use crate::components::badge::{BadgeColor, BadgeStyle};
use crate::components::data_table::{clamp_page, page_bounds, page_count, row_range};
use leptos::prelude::{Callback, Get, IntoAny, RwSignal, Set, Signal, StoredValue, Update};
use leptos::reactive::owner::Owner;
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
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

        super::component::apply_page_size_change(preferences, current_page, false, "50", |value| {
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

        super::component::apply_page_size_change(preferences, current_page, false, "50", |value| {
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
    assert_eq!(base.kind, EntityColumnKind::Text);

    let centered = base.clone().align_center();
    assert_eq!(centered.alignment, EntityColumnAlignment::Center);
    let started = base.clone().align_start();
    assert_eq!(started.alignment, EntityColumnAlignment::Start);
    let numeric = base.align_end().tabular_numbers();
    assert_eq!(numeric.alignment, EntityColumnAlignment::End);
    assert_eq!(numeric.kind, EntityColumnKind::Numeric);
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
    assert_eq!(column.kind, EntityColumnKind::Numeric);
    assert!(column.renderer.is_some());
}

// ── EntityColumnKind / numeric / identifier (ldui-no94) ──

#[test]
fn numeric_sets_kind_and_right_alignment() {
    let col = EntityColumn::text("balance", "Balance", |row: &Row| row.rank.to_string()).numeric();
    assert_eq!(col.kind, EntityColumnKind::Numeric);
    assert_eq!(col.alignment, EntityColumnAlignment::End);
}

#[test]
fn numeric_does_not_change_sort_key_or_comparator() {
    // EntityColumn's sort model is typed, not a `SortAs` enum: `.numeric()`
    // must never silently replace a caller's exact typed comparator with a
    // re-parsed-text fallback, regardless of call order.
    let with_typed_key = EntityColumn::new("balance", "Balance", |row: &Row| row.rank.to_string())
        .sortable_by_key(|row: &Row| row.rank)
        .numeric();
    assert!(with_typed_key.sort_key.is_some());
    assert!(with_typed_key.comparator.is_none());

    let numeric_first = EntityColumn::new("balance", "Balance", |row: &Row| row.rank.to_string())
        .numeric()
        .sortable_by_key(|row: &Row| row.rank);
    assert!(numeric_first.sort_key.is_some());
}

#[test]
fn align_after_numeric_overrides_the_implied_alignment() {
    // Builder calls apply in order: a caller reaching for centered or
    // left-aligned numeric presentation can still say so.
    let col = EntityColumn::text("balance", "Balance", |row: &Row| row.rank.to_string())
        .numeric()
        .align_center();
    assert_eq!(col.alignment, EntityColumnAlignment::Center);
    assert_eq!(col.kind, EntityColumnKind::Numeric);
}

#[test]
fn identifier_sets_kind_without_touching_alignment_or_sort() {
    let col = EntityColumn::text("job", "Job", |row: &Row| row.name.to_owned()).identifier();
    assert_eq!(col.kind, EntityColumnKind::Identifier);
    assert_eq!(col.alignment, EntityColumnAlignment::Auto);
    assert!(col.comparator.is_none());
    assert!(col.sort_key.is_some());
}

#[test]
fn tabular_numbers_sets_kind_without_touching_alignment() {
    // The lower-level primitive `.numeric()` is built from: kind only,
    // alignment untouched -- e.g. a centered date column with tabular
    // figures but not right-aligned.
    let col = EntityColumn::text("opened", "Opened", |row: &Row| row.name.to_owned())
        .align_center()
        .tabular_numbers();
    assert_eq!(col.kind, EntityColumnKind::Numeric);
    assert_eq!(col.alignment, EntityColumnAlignment::Center);
}

#[test]
fn entity_column_kind_default_class_matches_data_table_tokens() {
    assert_eq!(EntityColumnKind::Text.default_class(), None);
    assert_eq!(
        EntityColumnKind::Numeric.default_class(),
        Some("tabular-nums")
    );
    assert_eq!(
        EntityColumnKind::Identifier.default_class(),
        Some("font-mono")
    );
}

#[test]
fn entity_column_kind_as_str_is_stable() {
    assert_eq!(EntityColumnKind::Text.as_str(), "text");
    assert_eq!(EntityColumnKind::Numeric.as_str(), "numeric");
    assert_eq!(EntityColumnKind::Identifier.as_str(), "identifier");
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

fn date(year: i32, month: u8, day: u8) -> EntityDate {
    EntityDate::from_ymd(year, month, day).expect("a real calendar day")
}

#[test]
fn a_civil_date_accepts_only_real_iso_calendar_days() {
    // The filter never compares rendered cell text, so the ONE place a string
    // becomes comparable is here, and every rejection is a named state rather
    // than a silent fallback.
    assert_eq!(EntityDate::parse("2026-08-04"), Ok(date(2026, 8, 4)));
    assert_eq!(
        EntityDate::parse("  2026-08-04\n"),
        Ok(date(2026, 8, 4)),
        "a value restored from a URL or a saved view often carries whitespace"
    );
    assert_eq!(EntityDate::parse("2024-02-29"), Ok(date(2024, 2, 29)));

    assert_eq!(EntityDate::parse(""), Err(EntityDateParseError::Empty));
    assert_eq!(EntityDate::parse("   "), Err(EntityDateParseError::Empty));

    for malformed in [
        "2026-8-4",
        "04/08/2026",
        "2026-08-04T09:30",
        "20260804",
        "+2026-08-04",
        "2026-08-0x",
        "202\u{00e9}-08-04",
    ] {
        assert_eq!(
            EntityDate::parse(malformed),
            Err(EntityDateParseError::Malformed),
            "{malformed} is not an ISO calendar date"
        );
    }

    for out_of_range in ["2026-13-01", "2026-00-10", "2026-02-30", "0000-01-01"] {
        assert_eq!(
            EntityDate::parse(out_of_range),
            Err(EntityDateParseError::OutOfRange),
            "{out_of_range} is shaped right but names no real day"
        );
    }
    assert_eq!(
        EntityDate::parse("2026-02-29"),
        Err(EntityDateParseError::OutOfRange),
        "2026 is not a leap year"
    );
    assert_eq!(
        EntityDate::from_ymd(1900, 2, 29),
        None,
        "1900 is not a leap year"
    );
    assert_eq!(EntityDate::from_ymd(2000, 2, 29), Some(date(2000, 2, 29)));
}

#[test]
fn civil_dates_order_by_calendar_and_round_trip_their_machine_text() {
    let mut days = vec![date(2026, 1, 9), date(2025, 12, 31), date(2026, 1, 10)];
    days.sort();
    assert_eq!(
        days,
        vec![date(2025, 12, 31), date(2026, 1, 9), date(2026, 1, 10)],
        "9 January must sort before 10 January, which text ordering also gets \
         right only because to_iso zero-pads"
    );
    assert_eq!(days[1].to_iso(), "2026-01-09");
    assert_eq!(EntityDate::parse(&days[1].to_iso()), Ok(days[1]));
    assert_eq!(days[1].year(), 2026);
    assert_eq!(days[1].month(), 1);
    assert_eq!(days[1].day(), 9);
}

#[test]
fn an_unbounded_date_filter_hides_nothing_at_all() {
    // "The user has not filtered" -- so an undated row must survive too.
    // Getting this backwards empties a table the moment the control renders.
    let filter = EntityDateFilter::unbounded();
    assert_eq!(filter.status(), EntityDateFilterStatus::Unconstrained);
    assert!(!filter.constrains());
    assert!(filter.matches(Some(date(1999, 1, 1))));
    assert!(filter.matches(None));
    assert_eq!(
        EntityDateFilter::parse_bounds("", "   "),
        EntityDateFilter::unbounded(),
        "empty control text on both ends is the identity filter"
    );
}

#[test]
fn both_date_range_ends_are_inclusive() {
    let filter = EntityDateFilter::between(date(2026, 8, 1), date(2026, 8, 4));
    assert_eq!(filter.status(), EntityDateFilterStatus::Constrained);
    assert!(
        filter.matches(Some(date(2026, 8, 1))),
        "the start day is in"
    );
    assert!(filter.matches(Some(date(2026, 8, 4))), "the end day is in");
    assert!(!filter.matches(Some(date(2026, 7, 31))));
    assert!(!filter.matches(Some(date(2026, 8, 5))));

    let single = EntityDateFilter::on(date(2026, 8, 4));
    assert!(single.matches(Some(date(2026, 8, 4))));
    assert!(!single.matches(Some(date(2026, 8, 3))));
}

#[test]
fn a_half_open_date_filter_compares_one_end_and_excludes_undated_rows() {
    // The Office "arrived on or before cutoff" shape: only the upper end is
    // bounded, and a record with no arrival date cannot satisfy it.
    let cutoff = EntityDateFilter::parse_on_or_before("2026-08-04");
    assert_eq!(cutoff.status(), EntityDateFilterStatus::Constrained);
    assert!(cutoff.constrains());
    assert!(cutoff.start().is_open());
    assert!(cutoff.matches(Some(date(1970, 1, 1))));
    assert!(cutoff.matches(Some(date(2026, 8, 4))));
    assert!(!cutoff.matches(Some(date(2026, 8, 5))));
    assert!(
        !cutoff.matches(None),
        "an undated row must not slip through a bounded filter"
    );

    let since = EntityDateFilter::parse_on_or_after("2026-08-04");
    assert!(since.end().is_open());
    assert!(since.matches(Some(date(2026, 8, 4))));
    assert!(!since.matches(Some(date(2026, 8, 3))));
    assert!(!since.matches(None));
}

#[test]
fn an_inverted_date_range_matches_nothing_and_reports_why() {
    let filter = EntityDateFilter::between(date(2026, 8, 5), date(2026, 8, 4));
    assert_eq!(filter.status(), EntityDateFilterStatus::Impossible);
    assert!(
        filter.constrains(),
        "an impossible filter is excluding everything, so the user must be \
         able to see and clear it"
    );
    assert!(!filter.matches(Some(date(2026, 8, 4))));
    assert!(!filter.matches(Some(date(2026, 8, 5))));
    assert!(!filter.matches(None));
    assert_eq!(filter.invalid_input(), None, "impossible is not unreadable");
}

#[test]
fn an_unreadable_date_filter_matches_nothing_and_keeps_the_offending_text() {
    // Degrading unreadable text to "no constraint" would silently WIDEN the
    // result set; degrading it to a quiet empty table would hide the cause.
    // Neither happens: nothing matches and the text is retrievable.
    let filter = EntityDateFilter::parse_on_or_before("last tuesday");
    assert_eq!(filter.status(), EntityDateFilterStatus::Invalid);
    assert!(filter.constrains());
    assert_eq!(filter.invalid_input(), Some("last tuesday"));
    assert!(!filter.matches(Some(date(2026, 8, 4))));
    assert!(!filter.matches(None));
    assert!(filter.end().is_invalid());
    assert_eq!(filter.end().date(), None);

    // Unreadable outranks impossible: the actionable message wins.
    let both = EntityDateFilter::parse_bounds("2026-08-05", "2026-02-30");
    assert_eq!(both.status(), EntityDateFilterStatus::Invalid);
    assert_eq!(both.invalid_input(), Some("2026-02-30"));

    assert_eq!(
        EntityDateBound::parse("2026-02-30"),
        EntityDateBound::Invalid("2026-02-30".to_owned())
    );
    assert_eq!(EntityDateBound::parse("  "), EntityDateBound::Open);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArrivalRow {
    client: &'static str,
    status: &'static str,
    arrived: Option<EntityDate>,
}

fn arrival_rows() -> Vec<ArrivalRow> {
    vec![
        ArrivalRow {
            client: "Ada Lovelace",
            status: "ready",
            arrived: Some(date(2026, 8, 1)),
        },
        ArrivalRow {
            client: "Ada Byron",
            status: "urgent",
            arrived: Some(date(2026, 8, 4)),
        },
        ArrivalRow {
            client: "Grace Hopper",
            status: "ready",
            arrived: Some(date(2026, 8, 9)),
        },
        ArrivalRow {
            client: "Ada Unknown",
            status: "ready",
            arrived: None,
        },
    ]
}

/// Applies the three surfaces the way a consumer must: conjunction, with the
/// date compared against the ROW's own value and never against cell text.
fn surviving_clients(search: &str, status: &str, cutoff: &EntityDateFilter) -> Vec<&'static str> {
    arrival_rows()
        .into_iter()
        .filter(|row| {
            search.is_empty() || row.client.to_lowercase().contains(&search.to_lowercase())
        })
        .filter(|row| status.is_empty() || row.status == status)
        .filter(|row| cutoff.matches(row.arrived))
        .map(|row| row.client)
        .collect()
}

#[test]
fn a_date_filter_composes_with_search_and_column_filters_by_conjunction() {
    // Alone, the free-text search keeps the undated row.
    assert_eq!(
        surviving_clients("ada", "", &EntityDateFilter::unbounded()),
        vec!["Ada Lovelace", "Ada Byron", "Ada Unknown"]
    );
    // Adding the cutoff narrows it and drops the undated row, because a
    // bounded date filter is a real constraint, not a preference.
    let cutoff = EntityDateFilter::parse_on_or_before("2026-08-04");
    assert_eq!(
        surviving_clients("ada", "", &cutoff),
        vec!["Ada Lovelace", "Ada Byron"]
    );
    // ANDed with a column filter, never ORed: each surface can only remove.
    assert_eq!(
        surviving_clients("ada", "ready", &cutoff),
        vec!["Ada Lovelace"]
    );
    assert_eq!(surviving_clients("", "urgent", &cutoff), vec!["Ada Byron"]);
    // An unreadable cutoff empties the result even where the other two
    // surfaces match -- and `status()` is what explains the empty table.
    let unreadable = EntityDateFilter::parse_on_or_before("2026-02-30");
    assert!(surviving_clients("ada", "ready", &unreadable).is_empty());
    assert_eq!(unreadable.status(), EntityDateFilterStatus::Invalid);
    // Clearing the date restores exactly the pre-date result set: the filter
    // surfaces are independent, so removing one never disturbs the others.
    assert_eq!(
        surviving_clients("ada", "ready", &EntityDateFilter::parse_on_or_before("")),
        vec!["Ada Lovelace", "Ada Unknown"]
    );
    assert_eq!(
        surviving_clients("", "ready", &EntityDateFilter::unbounded()),
        vec!["Ada Lovelace", "Grace Hopper", "Ada Unknown"]
    );
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
#[should_panic(expected = "EntityColumnFilter control_id must not be empty")]
fn a_controlled_date_filter_rejects_an_empty_dom_identity() {
    let owner = Owner::new();
    owner.with(|| {
        let _ = EntityColumnFilter::date(
            "arrived",
            "",
            "Arrived",
            RwSignal::new(String::new()),
            "Enter a date as YYYY-MM-DD",
            Callback::new(|_| {}),
        );
    });
}

#[test]
fn a_controlled_date_filter_only_proposes_and_never_applies() {
    let owner = Owner::new();
    owner.with(|| {
        let label = RwSignal::new("Arrived".to_owned());
        let value = RwSignal::new("2026-08-04".to_owned());
        let proposals = Arc::new(Mutex::new(Vec::<EntityDateFilterProposal>::new()));
        let observed = Arc::clone(&proposals);
        let filter = EntityColumnFilter::date(
            "arrived",
            "awaiting-arrived-filter",
            label,
            value,
            "Enter a date as YYYY-MM-DD",
            Callback::new(move |proposal| observed.lock().unwrap().push(proposal)),
        );

        assert_eq!(filter.control_id(), Some("awaiting-arrived-filter"));
        assert_eq!(filter.label("Arrival date"), "Arrived");
        assert!(filter.is_active());

        filter.clear();
        let cleared = proposals.lock().unwrap().last().cloned().expect("proposal");
        assert_eq!(cleared.raw, "");
        assert_eq!(cleared.bound, EntityDateBound::Open);
        assert_eq!(cleared.cause, EntityDateFilterCause::Cleared);
        assert_eq!(cleared.column_id, "arrived");
        assert_eq!(
            cleared.control_id, "awaiting-arrived-filter",
            "the scope stamp is the caller's own base ID, so header and \
             responsive copies of one filter propose under one identity"
        );
        assert_eq!(cleared.date(), None);
        assert_eq!(
            value.get(),
            "2026-08-04",
            "a proposal must not mutate the caller-owned value"
        );
        assert!(
            filter.is_active(),
            "a rejected clear proposal must not disagree with the controlled value"
        );

        // A proposal carries the complete resulting value ALREADY interpreted,
        // so a caller storing a parsed filter cannot disagree with the control
        // about whether the text was readable.
        let edited = EntityDateFilterProposal::new(
            "2026-08-09",
            EntityDateFilterCause::Edited,
            "arrived",
            "awaiting-arrived-filter",
        );
        assert_eq!(edited.date(), Some(date(2026, 8, 9)));
        assert_eq!(
            EntityDateFilterProposal::new(
                "2026-02-30",
                EntityDateFilterCause::Edited,
                "arrived",
                "awaiting-arrived-filter",
            )
            .bound,
            EntityDateBound::Invalid("2026-02-30".to_owned())
        );

        // Localized copy is reactive, and an unreadable accepted value stays
        // ACTIVE -- otherwise the responsive panel would hide the only control
        // that can recover from it.
        label.set("Recibido".to_owned());
        assert_eq!(filter.label("Arrival date"), "Recibido");
        value.set("last tuesday".to_owned());
        assert!(filter.is_active());
        value.set("   ".to_owned());
        assert!(
            !filter.is_active(),
            "whitespace-only text expresses no constraint"
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
        super::component::resolved_page_size(&preferences, true, Some(11)).rows(),
        11
    );
    assert_eq!(
        super::component::resolved_page_size(&preferences, true, None).rows(),
        25
    );
    assert_eq!(
        preferences.page_size, 25,
        "a measured capacity is presentation state and never edits the preference"
    );
}

// ── One resolved page size (ldui-5p06) ──
//
// The defect: a viewport-fitted body rendered five rows and a four-page pager
// while the rows-per-page control read `25`. These pin the single resolution
// every one of those four surfaces now derives from.

#[test]
fn auto_fit_resolution_reports_the_measured_rows_and_says_it_is_auto() {
    let resolved = resolve_entity_page_size(EntityPageSizeIntent::Auto, true, 25, Some(5));
    assert!(resolved.is_auto());
    assert_eq!(resolved.intent(), EntityPageSizeIntent::Auto);
    assert_eq!(resolved.rows(), 5);
}

#[test]
fn an_explicit_numeric_choice_renders_that_many_rows_despite_a_smaller_fit() {
    // The bead's headline case: 17 rows and a five-row viewport must not show
    // `25` over a five-row page. Choosing 25 means 25.
    let resolved = resolve_entity_page_size(EntityPageSizeIntent::Fixed, true, 25, Some(5));
    assert!(!resolved.is_auto());
    assert_eq!(resolved.rows(), 25);
    assert_eq!(resolved.control_value(), "25");
    assert_eq!(
        page_count(17, resolved.rows()),
        1,
        "17 rows at an explicit 25 is one page, never four"
    );
}

#[test]
fn auto_before_the_first_measurement_is_still_labeled_auto() {
    // First paint renders the configured fallback. Reporting that as `Fixed`
    // would make the control flip 25 -> Auto(5) as if the user had acted.
    let resolved = resolve_entity_page_size(EntityPageSizeIntent::Auto, true, 25, None);
    assert!(resolved.is_auto());
    assert_eq!(resolved.rows(), 25, "the body really does render 25 here");
    assert_eq!(resolved.control_value(), "auto");
}

#[test]
fn auto_is_unavailable_without_a_viewport_fit_policy() {
    // A preference restored from a fitting table must not label a table that
    // never measures anything `Auto`.
    for measured in [None, Some(5)] {
        let resolved = resolve_entity_page_size(EntityPageSizeIntent::Auto, false, 50, measured);
        assert!(!resolved.is_auto(), "measured={measured:?}");
        assert_eq!(resolved.rows(), 50);
        assert_eq!(resolved.control_value(), "50");
    }
}

#[test]
fn a_resolution_can_never_carry_zero_rows() {
    // A zero would render an empty page and divide by zero in page counting.
    assert_eq!(EntityPageSize::auto(0).rows(), 1);
    assert_eq!(EntityPageSize::fixed(0).rows(), 1);
    assert_eq!(
        resolve_entity_page_size(EntityPageSizeIntent::Auto, true, 0, Some(0)).rows(),
        1
    );
    let mut zeroed = EntityTablePreferences::new(1);
    zeroed.page_size = 0;
    assert_eq!(
        super::component::resolved_page_size(&zeroed, false, None).rows(),
        1
    );
}

#[test]
fn body_summary_control_and_pager_all_read_the_same_resolution() {
    // The four surfaces the bead names, derived from one value. The check that
    // matters is that no surface is allowed a second source: each is computed
    // here exactly the way `EntityTable` computes it.
    let texts = EntityTableTexts::default();
    let total_rows = 17;
    for (intent, measured, expected_rows, expected_value, expected_label) in [
        (EntityPageSizeIntent::Auto, Some(5), 5, "auto", "Auto (5)"),
        (EntityPageSizeIntent::Auto, Some(7), 7, "auto", "Auto (7)"),
        (EntityPageSizeIntent::Fixed, Some(5), 25, "25", "25"),
    ] {
        let resolved = resolve_entity_page_size(intent, true, 25, measured);
        let rows = resolved.rows();
        assert_eq!(rows, expected_rows);
        // Control: selected value and visible label.
        assert_eq!(resolved.control_value(), expected_value);
        assert_eq!(resolved.control_label(&texts), expected_label);
        // Pager.
        let pages = page_count(total_rows, rows);
        // Summary.
        let (start, end) = row_range(0, rows, total_rows);
        // Body.
        let rendered = page_bounds(0, rows, total_rows).len();
        assert_eq!(rendered, rows.min(total_rows), "body vs resolution: {rows}");
        assert_eq!(end - start + 1, rendered, "summary vs body: {rows}");
        assert_eq!(pages, total_rows.div_ceil(rows), "pager vs resolution");
        assert_eq!(
            pages == 1,
            rendered == total_rows,
            "a single page must mean every row is rendered: {rows}"
        );
    }
}

#[test]
fn the_control_label_is_localizable_rather_than_hardcoded_english() {
    let texts = EntityTableTexts {
        rows_per_page_auto: "Automatique ({rows} lignes)".to_owned(),
        ..EntityTableTexts::default()
    };
    assert_eq!(
        EntityPageSize::auto(7).control_label(&texts),
        "Automatique (7 lignes)"
    );
}

#[test]
fn choosing_auto_records_the_intent_without_touching_the_numeric_preference() {
    let owner = Owner::new();
    owner.with(|| {
        let current = RwSignal::new(EntityTablePreferences::new(1));
        current.update(|preferences| {
            preferences.page_size = 50;
            preferences.page_size_mode = EntityPageSizeIntent::Fixed;
        });
        let preferences = super::component::PreferenceState::new(
            EntityTablePreferenceOwnership::controlled(
                current.into(),
                Callback::new(move |replacement| current.set(replacement)),
            ),
            StoredValue::new_local(columns()),
            1,
        );
        let current_page = RwSignal::new(2);
        let live_value = RefCell::new(None::<String>);

        super::component::apply_page_size_change(
            preferences,
            current_page,
            true,
            "auto",
            |value| {
                live_value.replace(Some(value));
            },
        );

        assert_eq!(current.get().page_size_mode, EntityPageSizeIntent::Auto);
        assert_eq!(
            current.get().page_size,
            50,
            "the explicit numeric preference survives an Auto selection"
        );
        assert_eq!(current_page.get(), 0);
        assert_eq!(live_value.into_inner().as_deref(), Some("auto"));
    });
}

#[test]
fn choosing_a_number_leaves_auto_in_one_atomic_preference_replacement() {
    let owner = Owner::new();
    owner.with(|| {
        let current = RwSignal::new(EntityTablePreferences::new(1));
        let emitted = Arc::new(Mutex::new(Vec::new()));
        let emitted_for_callback = Arc::clone(&emitted);
        let preferences = super::component::PreferenceState::new(
            EntityTablePreferenceOwnership::controlled(
                current.into(),
                Callback::new(move |replacement: EntityTablePreferences| {
                    current.set(replacement.clone());
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

        super::component::apply_page_size_change(preferences, current_page, true, "50", |value| {
            live_value.replace(Some(value));
        });

        let emitted = emitted
            .lock()
            .expect("controlled callback lock is available");
        assert_eq!(
            emitted.len(),
            1,
            "the mode and the number must move together, not in two replacements"
        );
        assert_eq!(emitted[0].page_size, 50);
        assert_eq!(emitted[0].page_size_mode, EntityPageSizeIntent::Fixed);
        assert_eq!(live_value.into_inner().as_deref(), Some("50"));
    });
}

#[test]
fn an_auto_request_is_ignored_by_a_table_that_never_measures() {
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
        let current_page = RwSignal::new(2);
        let live_value = RefCell::new(None::<String>);

        super::component::apply_page_size_change(
            preferences,
            current_page,
            false,
            "auto",
            |value| {
                live_value.replace(Some(value));
            },
        );

        assert_eq!(current_page.get(), 2, "an ignored request pages nothing");
        assert_eq!(
            live_value.into_inner().as_deref(),
            Some("25"),
            "the control snaps back to the numeric value actually in force"
        );
    });
}

#[test]
fn a_stored_page_size_mode_survives_a_preference_round_trip() {
    let columns = columns();
    let mut preferences = EntityTablePreferences::new(1);
    preferences.page_size_mode = EntityPageSizeIntent::Fixed;
    let payload = encode_preferences(&preferences).expect("preferences encode");
    let decoded = decode_preferences(&payload, 1, &columns);
    assert_eq!(decoded.page_size_mode, EntityPageSizeIntent::Fixed);

    // A payload written before this field existed defaults to Auto, which is
    // the pre-ldui-5p06 behavior of every viewport-fit table.
    let legacy = decode_preferences(
        r#"{"schema_version":1,"page_size":25,"sort":"system","hidden_columns":[],"column_widths":{}}"#,
        1,
        &columns,
    );
    assert_eq!(legacy.page_size_mode, EntityPageSizeIntent::Auto);
    assert_eq!(legacy.page_size, 25);
}

#[test]
fn a_resize_changes_only_the_measured_half_of_the_resolution() {
    // Two desktop heights: the intent, the numeric preference, and the
    // control's selected value are all identical across them, so a resize
    // cannot move the user's selection or the control's focus.
    let preferences = EntityTablePreferences::new(1);
    let tall = super::component::resolved_page_size(&preferences, true, Some(14));
    let short = super::component::resolved_page_size(&preferences, true, Some(6));
    assert_eq!(tall.control_value(), short.control_value());
    assert_eq!(tall.intent(), short.intent());
    assert_ne!(tall.rows(), short.rows());
    assert_eq!(
        (tall.rows(), short.rows()),
        (14, 6),
        "both heights report their own fit"
    );
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

// ── ldui-nz6d: controlled checkbox multi-selection wiring ──
//
// The state machine itself is covered exhaustively in `multi_selection.rs`.
// These cover the parts only the component owns: the leading track, the
// empty-state colspan, the construction-time refusal, and the render-path
// facts that a pure function cannot express.

#[test]
fn the_selection_control_track_leads_without_becoming_the_flexible_sink() {
    use crate::components::data_table::StableColumnTrack;

    let data_tracks = vec![
        StableColumnTrack::new("client", 220),
        StableColumnTrack::new("rank", 120),
    ];
    let without = super::component::entity_stable_tracks(false, data_tracks.clone());
    let with = super::component::entity_stable_tracks(true, data_tracks.clone());

    assert_eq!(without, data_tracks);
    assert_eq!(with.len(), without.len() + 1);
    assert_eq!(with[0].id, "__ldui-entity-selection");
    assert_eq!(with[0].width, 48);
    assert!(
        !with[0].flexible,
        "a fixed-width control column must never absorb spare table width"
    );
    assert_eq!(
        with[1..].to_vec(),
        without,
        "declaring a selection column must not disturb any data column's track"
    );
}

#[test]
fn the_empty_state_message_spans_the_selection_cell_too() {
    // Without the extra span the message row is one cell short, leaving a
    // ragged grid line under the checkbox column.
    assert_eq!(super::component::entity_empty_state_colspan(3, false), 3);
    assert_eq!(super::component::entity_empty_state_colspan(3, true), 4);
    // The pre-existing `.max(1)` floor for a table with no visible columns
    // is preserved in both modes.
    assert_eq!(super::component::entity_empty_state_colspan(0, false), 1);
    assert_eq!(super::component::entity_empty_state_colspan(0, true), 2);
}

#[test]
fn the_selection_column_is_never_synthesized_as_a_data_column() {
    // Structurally absent from the chooser, the sort model, the filter
    // vocabulary and the display projection -- rather than filtered out of
    // four places that could each be forgotten.
    let source = include_str!("component.rs");
    // Assembled at runtime so this assertion's own text cannot satisfy the
    // search it performs.
    for forbidden in [
        format!("EntityColumn::new({}", "SELECTION_COLUMN_TRACK_ID"),
        format!("EntityColumn::text({}", "SELECTION_COLUMN_TRACK_ID"),
    ] {
        assert!(
            !source.contains(&forbidden),
            "the selection control column must never be synthesized as a data EntityColumn"
        );
    }
}

#[test]
fn the_header_checkbox_writes_indeterminate_as_a_dom_property() {
    // `indeterminate` has NO HTML attribute: `indeterminate="true"` in
    // markup does nothing at all. It must be written through `prop:` AND
    // re-written with `set_indeterminate` in the change handler, because the
    // browser clears the property the moment the user clicks the box.
    let source = include_str!("component.rs");
    assert!(
        source.contains("prop:indeterminate=move || displayed_page_state.get().is_indeterminate()"),
        "the header checkbox must bind indeterminate as a DOM property"
    );
    assert!(
        source.contains("input.set_indeterminate(state.is_indeterminate())"),
        "the change handler must re-assert the indeterminate property the browser just cleared"
    );
    assert!(
        !source.contains("attr:indeterminate"),
        "indeterminate is not an attribute and must never be written as one"
    );
}

#[test]
fn the_header_governs_the_same_keys_the_body_renders() {
    // The one-truthful-page-size rule (ldui-5p06) applied to selection: the
    // header state and the rendered rows both read `page_row_keys`, which is
    // itself derived from the single resolved `page_size` memo. Recomputing
    // a second page window for the header is exactly the bug 5p06 fixed.
    let source = include_str!("component.rs");
    let derivation = source
        .split_once("let displayed_page_state =")
        .expect("header state derivation")
        .1;
    let derivation = &derivation[..derivation.find("});").expect("derivation end")];
    assert!(
        derivation.contains("page_row_keys"),
        "header state must be derived from page_row_keys, not a recomputed page window"
    );
    assert!(
        !derivation.contains("page_bounds") && !derivation.contains("page_size"),
        "header state must not recompute its own page window"
    );
    assert!(
        source.contains("let keys = page_row_keys.get_untracked();"),
        "the header gesture must cover exactly the rendered keys"
    );
    assert!(
        source.contains("move || page_row_keys.get(),"),
        "the body must iterate the same signal the header state reads"
    );
}

#[test]
fn every_selection_checkbox_carries_an_accessible_name() {
    let source = include_str!("component.rs");
    assert_eq!(
        source
            .matches("data-entity-selection-toggle=\"page\"")
            .count(),
        1,
        "exactly one header checkbox"
    );
    assert!(
        source.contains("texts.page_label(state, count)"),
        "the header checkbox is named from the localized Texts struct"
    );
    assert!(
        source.contains("texts.row_label(&name, label_accepted())"),
        "each row checkbox is named after its own row, never \"checkbox\""
    );
}

#[test]
fn selection_state_is_never_conveyed_by_colour_alone() {
    // The native checked/indeterminate glyph is the non-colour indicator; a
    // row's `aria-selected` is the programmatic one. The row tint is purely
    // supplementary, and there is no colour-only affordance anywhere.
    let source = include_str!("component.rs");
    assert!(
        source.contains("prop:checked=checked_accepted"),
        "each row's state must render as a real checkbox glyph"
    );
    assert!(
        source.contains("aria-selected=move || entity_row_aria_selected("),
        "selected rows must expose aria-selected"
    );
}

#[test]
fn the_selection_checkbox_owns_its_own_gesture() {
    // Without stopping propagation the same click would reach the row's
    // `on_row_activate` handler as well, so ticking a box would also
    // navigate.
    let source = include_str!("component.rs");
    let cell = source
        .split("data-entity-selection-cell=\"true\"")
        .nth(1)
        .expect("selection cell markup");
    let cell = &cell[..cell.find("</td>").expect("selection cell end")];
    assert!(cell.contains("on:click=move |event: web_sys::MouseEvent| event.stop_propagation()"));
    assert!(
        cell.contains("on:keydown=move |event: web_sys::KeyboardEvent| event.stop_propagation()")
    );
}

#[test]
fn multi_selection_does_not_make_the_whole_row_a_click_target() {
    // Row interactivity stays exactly what it was: `on_row_activate` or
    // single `selection`. Adding multi-selection to that predicate would
    // mean a plain click both activated the row and toggled its checkbox.
    let source = include_str!("component.rs");
    assert!(
        source.contains("let interactive = on_row_activate.is_some() || selection.is_some();"),
        "multi_selection must not widen the row-interactivity predicate"
    );
}

#[test]
#[should_panic(
    expected = "EntityTable configuration cannot combine selection with multi_selection"
)]
fn combining_both_selection_models_fails_closed_at_construction() {
    // Construction-time refusal with both prop names in the message -- not a
    // precedence rule that silently picks one.
    let _ = super::multi_selection::resolve_entity_selection_mode(true, true)
        .unwrap_or_else(|message| panic!("{message}"));
}

// ── Controlled accessible row grouping (ldui-iyfa) ──

#[derive(Clone, Debug, PartialEq, Eq)]
struct ActivityRow {
    id: &'static str,
    coordinator: &'static str,
    kind: &'static str,
    /// Machine `YYYY-MM-DD` arrival text, exactly as a consumer's own model
    /// would carry it before a date filter interprets it.
    arrived: &'static str,
}

fn activity_rows() -> Vec<ActivityRow> {
    vec![
        ActivityRow {
            id: "a1",
            coordinator: "co-2",
            kind: "Task",
            arrived: "2026-08-09",
        },
        ActivityRow {
            id: "a2",
            coordinator: "co-1",
            kind: "Task",
            arrived: "2026-08-01",
        },
        ActivityRow {
            id: "a3",
            coordinator: "co-2",
            kind: "Goal",
            arrived: "2026-08-10",
        },
        ActivityRow {
            id: "a4",
            coordinator: "co-1",
            kind: "Goal",
            arrived: "2026-08-04",
        },
        ActivityRow {
            id: "a5",
            coordinator: "co-3",
            kind: "Actual",
            arrived: "",
        },
    ]
}

fn activity_groups() -> Vec<EntityRowGroup> {
    vec![
        EntityRowGroup::new("co-1", "Ana Ruiz"),
        EntityRowGroup::new("co-2", "Beto Cruz"),
        EntityRowGroup::new("co-3", "Ana Ruiz"),
    ]
}

fn grouped_order_of(
    rows: &[ActivityRow],
    groups: &[EntityRowGroup],
    order: EntityGroupOrder,
    collapsed: &BTreeSet<String>,
) -> EntityGroupedOrder {
    let sorted: Vec<usize> = (0..rows.len()).collect();
    let group_key_of = |index: usize| rows[index].coordinator.to_owned();
    entity_grouped_order(&sorted, &group_key_of, groups, order, collapsed)
}

fn grouped_keys_of(rows: &[ActivityRow], order: &EntityGroupedOrder) -> Vec<&'static str> {
    order.indices.iter().map(|index| rows[*index].id).collect()
}

fn grouped_page_of(
    order: &EntityGroupedOrder,
    rows: &[ActivityRow],
    bounds: std::ops::Range<usize>,
) -> (Vec<String>, Vec<String>) {
    let group_keys = order.group_keys[bounds.clone()].to_vec();
    let row_keys = order.indices[bounds]
        .iter()
        .map(|index| rows[*index].id.to_owned())
        .collect::<Vec<_>>();
    (group_keys, row_keys)
}

#[test]
fn grouping_partitions_by_stable_key_never_by_display_label() {
    // `co-1` and `co-3` deliberately carry the SAME label. Two groups that
    // read identically on screen must stay two groups, because identity is the
    // key -- were the label the partition, this dataset would collapse three
    // sections into two and silently merge two coordinators' records.
    let rows = activity_rows();
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    assert_eq!(
        order
            .runs
            .iter()
            .map(|run| run.key.as_str())
            .collect::<Vec<_>>(),
        vec!["co-1", "co-2", "co-3"]
    );
    assert_eq!(
        grouped_keys_of(&rows, &order),
        vec!["a2", "a4", "a1", "a3", "a5"]
    );
}

#[test]
fn relabelling_a_group_changes_nothing_but_the_rendered_heading() {
    // A localization change must not repartition the table, reorder a section,
    // or move a collapse flag onto a different group.
    let rows = activity_rows();
    let declared = activity_groups();
    let relabelled = vec![
        EntityRowGroup::new("co-1", "Ana Ruiz (Field)"),
        EntityRowGroup::new("co-2", "Beto Cruz (Field)"),
        EntityRowGroup::new("co-3", "Ana Ruiz (Office)"),
    ];
    let before = grouped_order_of(
        &rows,
        &declared,
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    let after = grouped_order_of(
        &rows,
        &relabelled,
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    assert_eq!(before, after);
    assert_eq!(entity_group_label(&relabelled, "co-3"), "Ana Ruiz (Office)");
    // An undeclared key still renders, under its own key, rather than
    // disappearing with its rows.
    assert_eq!(entity_group_label(&declared, "co-9"), "co-9");
}

#[test]
fn row_sorting_happens_within_groups_not_across_them() {
    // The incoming permutation is the table's own sort. Grouping is a STABLE
    // partition on top of it, so the within-group order must survive exactly.
    let rows = activity_rows();
    let sorted_by_kind: Vec<usize> = vec![4, 2, 3, 0, 1];
    let group_key_of = |index: usize| rows[index].coordinator.to_owned();
    let order = entity_grouped_order(
        &sorted_by_kind,
        &group_key_of,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    // co-1 keeps (a4, a2) and co-2 keeps (a3, a1) -- the order the sort put
    // them in, not the source order.
    assert_eq!(
        grouped_keys_of(&rows, &order),
        vec!["a4", "a2", "a3", "a1", "a5"]
    );
}

#[test]
fn an_explicit_group_sort_reorders_sections_and_leaves_rows_alone() {
    let rows = activity_rows();
    let groups = activity_groups();
    let ascending = grouped_order_of(
        &rows,
        &groups,
        EntityGroupOrder::LabelAscending,
        &BTreeSet::new(),
    );
    // Two groups share the label "Ana Ruiz"; declared order breaks the tie, so
    // the result is total rather than dependent on sort implementation.
    assert_eq!(
        ascending
            .runs
            .iter()
            .map(|run| run.key.as_str())
            .collect::<Vec<_>>(),
        vec!["co-1", "co-3", "co-2"]
    );
    let descending = grouped_order_of(
        &rows,
        &groups,
        EntityGroupOrder::LabelDescending,
        &BTreeSet::new(),
    );
    assert_eq!(
        descending
            .runs
            .iter()
            .map(|run| run.key.as_str())
            .collect::<Vec<_>>(),
        vec!["co-2", "co-1", "co-3"]
    );
    // Within co-2, source order (a1 before a3) survives every group sort.
    assert_eq!(
        grouped_keys_of(&rows, &descending),
        vec!["a1", "a3", "a2", "a4", "a5"]
    );
}

#[test]
fn undeclared_group_keys_rank_after_declared_ones_in_first_appearance_order() {
    let rows = activity_rows();
    let declared = vec![EntityRowGroup::new("co-2", "Beto Cruz")];
    let order = grouped_order_of(
        &rows,
        &declared,
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    assert_eq!(
        order
            .runs
            .iter()
            .map(|run| run.key.as_str())
            .collect::<Vec<_>>(),
        vec!["co-2", "co-1", "co-3"]
    );
    // Nothing is dropped for being undeclared: every row still reaches a page.
    assert_eq!(order.indices.len(), rows.len());
}

#[test]
fn filtering_out_a_group_removes_its_heading_with_its_rows() {
    // Filters apply to child rows; this component filters nothing itself. A
    // group whose rows are all gone has no run left, so its heading cannot
    // outlive its children.
    let rows: Vec<ActivityRow> = activity_rows()
        .into_iter()
        .filter(|row| row.coordinator != "co-2")
        .collect();
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    assert!(order.runs.iter().all(|run| run.key != "co-2"));
    assert_eq!(order.runs.len(), 2);
}

#[test]
fn collapsing_removes_rows_from_the_displayed_model_but_keeps_an_honest_count() {
    let rows = activity_rows();
    let collapsed = BTreeSet::from(["co-1".to_owned()]);
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &collapsed,
    );
    // The run survives with its true row count -- the heading can still say
    // how many records it hides -- while contributing no displayed rows, so
    // paging, the row-range summary and the selection population shrink
    // together and every count stays truthful.
    let run = order
        .runs
        .iter()
        .find(|run| run.key == "co-1")
        .expect("a collapsed group keeps its run");
    assert!(run.collapsed);
    assert_eq!(run.row_count, 2);
    assert_eq!(order.indices.len(), 3);
    assert!(!order.group_keys.iter().any(|key| key == "co-1"));
    assert_eq!(order.group_keys.len(), order.indices.len());
}

#[test]
fn a_date_filter_that_empties_a_group_removes_its_heading_with_its_rows() {
    // Composition with ldui-iyfa: a date filter is an ordinary CHILD-ROW
    // filter, applied before grouping. A group left with no surviving child
    // has no run, so its heading cannot outlive its rows -- and an undated
    // row is excluded by a bounded filter, which is what empties `co-3` here.
    let cutoff = EntityDateFilter::parse_on_or_before("2026-08-04");
    let rows: Vec<ActivityRow> = activity_rows()
        .into_iter()
        .filter(|row| cutoff.matches(EntityDate::parse(row.arrived).ok()))
        .collect();
    assert_eq!(
        grouped_keys_of(
            &rows,
            &grouped_order_of(
                &rows,
                &activity_groups(),
                EntityGroupOrder::Declared,
                &BTreeSet::new(),
            )
        ),
        vec!["a2", "a4"]
    );

    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    assert_eq!(
        order
            .runs
            .iter()
            .map(|run| run.key.as_str())
            .collect::<Vec<_>>(),
        vec!["co-1"],
        "co-2 (both rows after the cutoff) and co-3 (undated) lose their headings"
    );

    // Clearing the date restores every group, headings included.
    let restored = activity_rows();
    let cleared = EntityDateFilter::parse_on_or_before("");
    assert!(
        restored
            .iter()
            .all(|row| cleared.matches(EntityDate::parse(row.arrived).ok()))
    );
    let order = grouped_order_of(
        &restored,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    assert_eq!(order.runs.len(), 3);
}

#[test]
fn collapsing_never_resurrects_a_group_the_filter_emptied() {
    let rows: Vec<ActivityRow> = activity_rows()
        .into_iter()
        .filter(|row| row.coordinator != "co-2")
        .collect();
    let collapsed = BTreeSet::from(["co-2".to_owned()]);
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &collapsed,
    );
    assert!(order.runs.iter().all(|run| run.key != "co-2"));
}

#[test]
fn group_collapse_proposals_carry_the_complete_resulting_set() {
    // Same contract as ldui-nz6d's selection proposals: a complete set, never
    // a delta the caller has to reassemble.
    let current = BTreeSet::from(["co-1".to_owned(), "co-2".to_owned()]);
    let expanded = propose_entity_group_collapse(&current, "co-1", false);
    assert_eq!(expanded, BTreeSet::from(["co-2".to_owned()]));
    let collapsed = propose_entity_group_collapse(&current, "co-3", true);
    assert_eq!(
        collapsed,
        BTreeSet::from(["co-1".to_owned(), "co-2".to_owned(), "co-3".to_owned()])
    );
    // The caller's own accepted set is never mutated in place.
    assert_eq!(current.len(), 2);
}

#[test]
fn an_expanded_group_heading_is_never_stranded_as_the_last_visible_row() {
    // Headings are derived FROM the page's rows, so an orphan heading is
    // unrepresentable rather than merely avoided. Every page boundary of this
    // dataset is checked, at every page size.
    let rows = activity_rows();
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    for page_size in 1..=rows.len() {
        let pages = page_count(order.indices.len(), page_size);
        for page in 0..pages {
            let bounds = page_bounds(page, page_size, order.indices.len());
            let (group_keys, row_keys) = grouped_page_of(&order, &rows, bounds.clone());
            let previous = entity_previous_group_key(&order.group_keys, &bounds);
            let sections = entity_grouped_page_sections(
                &order.runs,
                &group_keys,
                &row_keys,
                previous.as_deref(),
                page + 1 >= pages,
            );
            assert!(
                sections
                    .iter()
                    .all(|section| section.collapsed || !section.row_keys.is_empty()),
                "expanded section without rows at page {page} size {page_size}"
            );
            // Every painted row belongs to exactly one section, in order.
            let painted = sections
                .iter()
                .flat_map(|section| section.row_keys.clone())
                .collect::<Vec<_>>();
            assert_eq!(painted, row_keys);
        }
    }
}

#[test]
fn a_group_spanning_a_page_boundary_resumes_with_a_continuation_heading() {
    let rows = activity_rows();
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    // Page size 1 splits co-1 (a2, a4) across pages 0 and 1.
    let bounds = page_bounds(1, 1, order.indices.len());
    let (group_keys, row_keys) = grouped_page_of(&order, &rows, bounds.clone());
    let previous = entity_previous_group_key(&order.group_keys, &bounds);
    assert_eq!(previous.as_deref(), Some("co-1"));
    let sections = entity_grouped_page_sections(
        &order.runs,
        &group_keys,
        &row_keys,
        previous.as_deref(),
        false,
    );
    assert_eq!(sections.len(), 1);
    assert!(sections[0].continued);
    assert_eq!(sections[0].group_key, "co-1");
    // The count is the group's TRUE size, not the page's slice of it.
    assert_eq!(sections[0].group_row_count, 2);
    assert_eq!(sections[0].first_row_position, 0);
}

#[test]
fn a_collapsed_group_heading_is_anchored_before_the_next_fresh_section() {
    let rows = activity_rows();
    let collapsed = BTreeSet::from(["co-2".to_owned()]);
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &collapsed,
    );
    let bounds = page_bounds(0, 25, order.indices.len());
    let (group_keys, row_keys) = grouped_page_of(&order, &rows, bounds.clone());
    let previous = entity_previous_group_key(&order.group_keys, &bounds);
    let sections = entity_grouped_page_sections(
        &order.runs,
        &group_keys,
        &row_keys,
        previous.as_deref(),
        true,
    );
    assert_eq!(
        sections
            .iter()
            .map(|section| section.group_key.as_str())
            .collect::<Vec<_>>(),
        vec!["co-1", "co-2", "co-3"]
    );
    let hidden = &sections[1];
    assert!(hidden.collapsed);
    assert!(hidden.row_keys.is_empty());
    assert_eq!(hidden.group_row_count, 2);
}

#[test]
fn a_trailing_collapsed_group_renders_on_the_last_page_and_only_there() {
    let rows = activity_rows();
    let collapsed = BTreeSet::from(["co-3".to_owned()]);
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &collapsed,
    );
    let pages = page_count(order.indices.len(), 2);
    let mut rendered = 0;
    for page in 0..pages {
        let bounds = page_bounds(page, 2, order.indices.len());
        let (group_keys, row_keys) = grouped_page_of(&order, &rows, bounds.clone());
        let previous = entity_previous_group_key(&order.group_keys, &bounds);
        let sections = entity_grouped_page_sections(
            &order.runs,
            &group_keys,
            &row_keys,
            previous.as_deref(),
            page + 1 >= pages,
        );
        rendered += sections
            .iter()
            .filter(|section| section.group_key == "co-3")
            .count();
    }
    assert_eq!(
        rendered, 1,
        "every non-empty collapsed group renders exactly once"
    );
}

#[test]
fn collapsing_every_group_still_renders_every_heading() {
    let rows = activity_rows();
    let collapsed = BTreeSet::from(["co-1".to_owned(), "co-2".to_owned(), "co-3".to_owned()]);
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &collapsed,
    );
    assert!(order.indices.is_empty());
    let sections = entity_grouped_page_sections(&order.runs, &[], &[], None, true);
    assert_eq!(sections.len(), 3);
    assert!(sections.iter().all(|section| section.collapsed));
}

#[test]
fn a_group_heading_spans_the_current_column_count_including_the_selection_cell() {
    // ldui-ibjk: the `<colgroup>` pins one `<col>` per column and the leading
    // selection cell claims its own track, so a heading short by one desyncs
    // the grid. Both full-width rows -- the empty state and every heading --
    // read the same arithmetic, which is why they cannot drift apart.
    assert_eq!(entity_group_header_colspan(4, false), 4);
    assert_eq!(entity_group_header_colspan(4, true), 5);
    // Hiding a column narrows the span with it.
    assert_eq!(entity_group_header_colspan(3, true), 4);
    // A table with every column hidden still spans one cell, never zero.
    assert_eq!(entity_group_header_colspan(0, false), 1);
    assert_eq!(entity_group_header_colspan(0, true), 2);
    assert_eq!(
        entity_group_header_colspan(4, true),
        super::component::entity_empty_state_colspan(4, true)
    );
}

#[test]
fn group_headings_are_presentation_and_never_join_the_displayed_page_selection() {
    // ldui-nz6d's invariant: the header checkbox governs the keys the table is
    // painting. A heading is not a record and has no row key, so it cannot
    // enter that population -- and there is deliberately no per-group
    // select-all, which would have to name rows on other pages (or, for a
    // collapsed group, rows nobody can see) and would reintroduce exactly the
    // "checked means something you cannot verify" defect nz6d refused.
    let rows = activity_rows();
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    let bounds = page_bounds(0, 25, order.indices.len());
    let (group_keys, row_keys) = grouped_page_of(&order, &rows, bounds);
    let sections = entity_grouped_page_sections(&order.runs, &group_keys, &row_keys, None, true);
    let displayed = EntityTableDisplayedPage::new(row_keys.clone());
    assert_eq!(displayed.len(), rows.len());
    assert_eq!(sections.len(), 3);
    let accepted = row_keys.iter().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        displayed.selection_state(&accepted),
        EntityTableDisplayedPageSelection::All,
        "selecting every painted row checks the header; headings never count"
    );
    let source = include_str!("component.rs");
    assert!(
        !source.contains("data-entity-selection-toggle=\"group\""),
        "no per-group select-all may exist"
    );
}

#[test]
fn collapsing_a_group_shrinks_the_selection_population_rather_than_clearing_keys() {
    let rows = activity_rows();
    let collapsed = BTreeSet::from(["co-1".to_owned()]);
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &collapsed,
    );
    let bounds = page_bounds(0, 25, order.indices.len());
    let (_, row_keys) = grouped_page_of(&order, &rows, bounds);
    let displayed = EntityTableDisplayedPage::new(row_keys);
    // a2/a4 belong to the collapsed group. They stay ACCEPTED and are counted
    // off-page, exactly like keys on another page -- never silently cleared.
    let accepted = BTreeSet::from([
        "a2".to_owned(),
        "a4".to_owned(),
        "a1".to_owned(),
        "a3".to_owned(),
        "a5".to_owned(),
    ]);
    assert_eq!(
        displayed.selection_state(&accepted),
        EntityTableDisplayedPageSelection::All
    );
    assert_eq!(off_page_selected_count(&accepted, displayed.keys()), 2);
}

#[test]
fn the_display_projection_carries_the_group_identity_the_table_stopped_repeating() {
    let rows = activity_rows();
    let columns = vec![
        EntityColumn::text("kind", "Kind", |row: &ActivityRow| row.kind.to_owned()).required(),
    ];
    let preferences = EntityTablePreferences::new(1);
    let order = grouped_order_of(
        &rows,
        &activity_groups(),
        EntityGroupOrder::Declared,
        &BTreeSet::new(),
    );
    let groups = activity_groups();
    let label_of = |key: &str| entity_group_label(&groups, key);
    let projection = super::model::entity_table_display_projection_from_indices(
        &rows,
        &columns,
        &preferences,
        &order.indices,
        0,
        25,
        &|row: &ActivityRow| row.id.to_owned(),
        EntityTableActionColumnPolicy::default(),
        Some(super::model::EntityProjectionGrouping {
            group_keys: &order.group_keys,
            label_of: &label_of,
            column_header: "Coordinator",
        }),
    );
    assert_eq!(projection.columns[0].id, ENTITY_GROUP_COLUMN_ID);
    assert_eq!(projection.columns[0].label, "Coordinator");
    assert_eq!(projection.columns[1].id, "kind");
    let exported = projection.rows(EntityTableProjectionScope::AllFiltered);
    assert_eq!(exported.len(), rows.len());
    // The LABEL is the exported cell (a person reads a CSV) and the KEY is the
    // stable identity beside it (a re-import joins on it). Two groups sharing
    // a label stay distinguishable because of the key.
    assert_eq!(exported[0].cells[0], "Ana Ruiz");
    assert_eq!(exported[0].group_key.as_deref(), Some("co-1"));
    assert_eq!(exported[4].cells[0], "Ana Ruiz");
    assert_eq!(exported[4].group_key.as_deref(), Some("co-3"));
    // The export is in the same grouped order the body paints.
    assert_eq!(
        exported
            .iter()
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        vec!["a2", "a4", "a1", "a3", "a5"]
    );
}

#[test]
fn an_ungrouped_projection_gains_no_group_column_and_no_group_key() {
    let rows = activity_rows();
    let columns = vec![
        EntityColumn::text("kind", "Kind", |row: &ActivityRow| row.kind.to_owned()).required(),
    ];
    let preferences = EntityTablePreferences::new(1);
    let projection = entity_table_display_projection(
        &rows,
        &columns,
        &preferences,
        0,
        25,
        &|row: &ActivityRow| row.id.to_owned(),
        EntityTableActionColumnPolicy::default(),
    );
    assert_eq!(projection.columns.len(), 1);
    assert_eq!(projection.columns[0].id, "kind");
    assert!(
        projection
            .rows(EntityTableProjectionScope::AllFiltered)
            .iter()
            .all(|row| row.group_key.is_none())
    );
}

#[test]
fn group_ranks_are_total_over_declared_and_encountered_keys() {
    let groups = activity_groups();
    let encountered = vec!["co-9".to_owned(), "co-2".to_owned(), "co-8".to_owned()];
    let ranks = entity_group_ranks(&groups, EntityGroupOrder::Declared, &encountered);
    assert_eq!(ranks["co-1"], 0);
    assert_eq!(ranks["co-2"], 1);
    assert_eq!(ranks["co-3"], 2);
    assert_eq!(ranks["co-9"], 3);
    assert_eq!(ranks["co-8"], 4);
}

#[test]
fn a_grouped_table_keeps_one_global_column_header_and_one_filter_row() {
    // The whole point of ldui-iyfa: one table instance, not one per group. The
    // `<thead>` and the controlled filter row sit outside the grouped body
    // branch entirely, so grouping structurally cannot duplicate either.
    let source = include_str!("component.rs");
    assert_eq!(source.matches("<thead").count(), 1);
    assert_eq!(source.matches("data-entity-column-filter-row=").count(), 1);
}

#[test]
fn the_group_heading_carries_row_group_and_colgroup_semantics() {
    // One `<tbody>` per section (already `role="rowgroup"`) named by its
    // heading, plus a spanning `<th scope="colgroup">` so HTML's own
    // header-association algorithm attributes every child cell to the heading
    // without the label being repeated in a data cell.
    let source = include_str!("component.rs");
    assert!(source.contains("aria-labelledby=labelled_by"));
    assert!(source.contains("scope=\"colgroup\""));
    assert!(source.contains("data-entity-group-header=group_key.clone()"));
    // Collapsed children are not rendered at all -- they leave the
    // accessibility tree instead of being painted and visually hidden.
    assert!(source.contains("{(!collapsed).then(move || local_for_enumerate("));
    // The disclosure control, not the heading row, carries `aria-expanded`.
    assert!(source.contains("aria-expanded=(!collapsed).to_string()"));
}

#[test]
fn a_group_heading_row_is_never_focusable_and_never_selectable() {
    // Presentation rows must not create a tab stop, must not report
    // `aria-selected`, and must not carry a row key that selection or focus
    // recovery could latch onto.
    let source = include_str!("component.rs");
    let heading = source
        .split("data-entity-group-header=group_key.clone()")
        .nth(1)
        .expect("group heading markup");
    let heading = &heading[..heading.find("</tr>").expect("group heading end")];
    assert!(!heading.contains("tabindex"));
    assert!(!heading.contains("aria-selected"));
    assert!(!heading.contains("data-entity-row-key"));
}

#[test]
fn focus_recovery_reads_the_painted_page_rather_than_recomputing_one() {
    // Grouping reorders rows and collapse removes them, so a second
    // independently recomputed page window would recover focus onto a row that
    // is not on screen. There is one displayed order and everything reads it.
    let source = include_str!("component.rs");
    assert!(source.contains("let visible_keys = page_row_keys.get();"));
    assert!(
        !source.contains("fn visible_row_keys"),
        "the duplicate page-window computation must be gone"
    );
}

#[test]
fn an_ungrouped_table_renders_the_body_it_always_did() {
    // Grouping costs a table that does not use it nothing: no extra `<tbody>`,
    // no heading markup, no `data-entity-group-*` attribute anywhere.
    let source = include_str!("component.rs");
    assert!(source.contains("{(!has_grouping).then(|| view! {"));
    assert!(source.contains("{has_grouping.then(|| {"));
    // Every piece of grouping markup lives behind that gate.
    let grouped = source
        .split("{has_grouping.then(|| {")
        .nth(1)
        .expect("grouped body branch");
    let ungrouped = source
        .split("{(!has_grouping).then(|| view! {")
        .nth(1)
        .expect("ungrouped body branch");
    let ungrouped = &ungrouped[..ungrouped
        .find("{has_grouping.then(|| {")
        .expect("ungrouped branch ends before the grouped one")];
    assert!(!ungrouped.contains("data-entity-group"));
    assert!(grouped.contains("render_group_section"));
}

#[test]
fn every_group_string_the_table_renders_comes_from_texts() {
    // A dedicated texts struct, like ldui-nz6d's selection copy: copy that
    // only exists when the feature is configured must not widen the always
    // required `EntityTableTexts` and break every consumer's literal.
    let texts = EntityGroupTexts::default();
    assert_eq!(texts.row_count_label(459), "459 rows");
    assert_eq!(texts.heading("Ana Ruiz", false), "Ana Ruiz");
    assert_eq!(texts.heading("Ana Ruiz", true), "Ana Ruiz (continued)");
    // The accessible name of the disclosure control contains the visible group
    // label, so it satisfies label-in-name rather than replacing it.
    assert_eq!(texts.toggle_label("Ana Ruiz", false), "Collapse Ana Ruiz");
    assert_eq!(texts.toggle_label("Ana Ruiz", true), "Expand Ana Ruiz");
    assert_eq!(texts.column_header, "Group");
    // Adding grouping left the base texts struct untouched.
    let base = EntityTableTexts::default();
    assert_eq!(base.no_rows, "No rows");
}

#[test]
fn a_declared_group_carries_optional_compact_metadata() {
    let groups = vec![EntityRowGroup::new("co-1", "Ana Ruiz").with_meta("3 of 5 complete")];
    assert_eq!(
        entity_group_meta(&groups, "co-1").as_deref(),
        Some("3 of 5 complete")
    );
    assert_eq!(entity_group_meta(&groups, "co-2"), None);
    assert_eq!(groups[0].key(), "co-1");
    assert_eq!(groups[0].label(), "Ana Ruiz");
    assert_eq!(groups[0].meta(), Some("3 of 5 complete"));
}
