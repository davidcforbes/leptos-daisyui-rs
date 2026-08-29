use super::*;
use crate::components::data_table::{clamp_page, page_bounds, page_count};
use leptos::prelude::{Callback, Get, RwSignal, Set, StoredValue};
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
