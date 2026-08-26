use super::*;
use std::cell::Cell;
use std::rc::Rc;

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

    let first = cache.indices(Rc::clone(&rows), &columns, &sort);
    let first_call_count = calls.get();
    let mut unrelated_preferences = EntityTablePreferences::new(1);
    unrelated_preferences.page_size = 50;
    unrelated_preferences
        .hidden_columns
        .insert("status".to_owned());
    let second = cache.indices(Rc::clone(&rows), &columns, &sort);

    assert_eq!(first.as_slice(), second.as_slice());
    assert_eq!(calls.get(), first_call_count);
    assert_eq!(unrelated_preferences.page_size, 50);

    cache.indices(
        Rc::clone(&rows),
        &columns,
        &EntitySort::descending("client"),
    );
    assert!(
        calls.get() > first_call_count,
        "a real sort change must recompute"
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
fn dataset_changes_reset_page_but_row_deltas_preserve_valid_page() {
    assert_eq!(page_after_dataset_change(3, "office-1", "office-1"), 3);
    assert_eq!(page_after_dataset_change(3, "office-1", "office-2"), 0);
    assert_eq!(page_after_row_delta(2, 25, 74), 2);
    assert_eq!(page_after_row_delta(2, 25, 49), 1);
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
fn stored_preferences_are_versioned_and_normalized() {
    let columns = columns();
    let mut preferences = EntityTablePreferences::new(4);
    preferences.page_size = 50;
    preferences.sort = EntitySort::descending("rank");
    preferences.hidden_columns.insert("office".to_owned());
    preferences.column_widths.insert("office".to_owned(), 280);

    let encoded = encode_preferences(&preferences).unwrap();
    assert_eq!(decode_preferences(&encoded, 4, &columns), preferences);

    let stale = decode_preferences(&encoded, 5, &columns);
    assert_eq!(stale, EntityTablePreferences::new(5));
    assert_eq!(
        decode_preferences("not-json", 4, &columns),
        EntityTablePreferences::new(4)
    );
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
