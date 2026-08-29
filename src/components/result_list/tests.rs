use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CasePayload {
    case_number: &'static str,
    generation: u8,
}

fn keyed(
    key: &str,
    title: &str,
    case_number: &'static str,
    generation: u8,
) -> ResultListItem<CasePayload> {
    ResultListItem::new(
        key,
        ResultRow::new(title),
        CasePayload {
            case_number,
            generation,
        },
    )
}

// ── ResultRow ──

#[test]
fn result_row_new_leaves_subtitle_and_snippet_empty() {
    let row = ResultRow::new("index.md");
    assert_eq!(row.title, "index.md");
    assert_eq!(row.subtitle, "");
    assert_eq!(row.snippet, "");
}

#[test]
fn secondary_line_prefers_snippet_over_subtitle() {
    let row = ResultRow {
        title: "readme.md".into(),
        subtitle: "/docs".into(),
        snippet: "...matched text...".into(),
    };
    assert_eq!(row.secondary_line(), "...matched text...");
}

#[test]
fn secondary_line_falls_back_to_subtitle_when_snippet_empty() {
    let row = ResultRow {
        title: "readme.md".into(),
        subtitle: "/docs".into(),
        snippet: String::new(),
    };
    assert_eq!(row.secondary_line(), "/docs");
}

#[test]
fn secondary_line_empty_when_both_empty() {
    let row = ResultRow::new("a.md");
    assert_eq!(row.secondary_line(), "");
}

#[test]
fn result_row_default_is_all_empty() {
    let row = ResultRow::default();
    assert_eq!(row, ResultRow::new(""));
}

// ── ResultListItem identity and replacement semantics ──

#[test]
fn keyed_items_separate_duplicate_display_from_payload_identity() {
    let first = keyed("case-a", "Alex Morgan", "A-100", 1);
    let second = keyed("case-b", "Alex Morgan", "B-200", 1);
    assert_eq!(first.row, second.row);
    assert_ne!(first.key, second.key);
    assert_ne!(first.payload, second.payload);
}

#[test]
fn validation_rejects_blank_and_duplicate_keys() {
    let blank = vec![keyed("   ", "Blank", "A-100", 1)];
    assert_eq!(
        validate_result_list_items(&blank),
        Err(ResultListKeyError::EmptyKey { index: 0 })
    );

    let duplicate = vec![
        keyed("case-a", "Alex Morgan", "A-100", 1),
        keyed("case-a", "Alex Morgan", "B-200", 1),
    ];
    assert_eq!(
        validate_result_list_items(&duplicate),
        Err(ResultListKeyError::DuplicateKey {
            key: "case-a".to_owned(),
            first_index: 0,
            duplicate_index: 1,
        })
    );
}

#[test]
fn replacement_preserves_key_and_uses_latest_payload() {
    let replacement = vec![
        keyed("case-b", "Alejandro Morgan", "B-200", 2),
        keyed("case-a", "Alex Morgan", "A-100", 1),
    ];
    assert_eq!(
        reconcile_result_key(Some("case-b"), &replacement),
        Some("case-b".to_owned())
    );
    assert_eq!(
        current_result_item(&replacement, "case-b")
            .unwrap()
            .payload
            .generation,
        2
    );
}

#[test]
fn removal_falls_back_to_first_and_keyboard_uses_current_order() {
    let rows = vec![
        keyed("case-c", "Third", "C-300", 1),
        keyed("case-a", "First", "A-100", 1),
    ];
    assert_eq!(
        reconcile_result_key(Some("case-b"), &rows),
        Some("case-c".into())
    );
    assert_eq!(
        move_result_key(Some("case-c"), 1, &rows),
        Some("case-a".into())
    );
    assert_eq!(
        move_result_key(Some("case-a"), 1, &rows),
        Some("case-a".into())
    );
}

#[test]
fn option_dom_ids_are_collision_free_for_arbitrary_key_bytes() {
    assert_ne!(keyed_option_dom_id(7, "a b"), keyed_option_dom_id(7, "a-b"));
    assert_ne!(keyed_option_dom_id(7, "é"), keyed_option_dom_id(7, "e"));
    assert!(keyed_option_dom_id(7, "a b").starts_with("ld-result-list-7-option-"));
}

#[test]
fn legacy_result_list_remains_an_adapter_with_reset_first_policy() {
    let source = include_str!("component.rs");
    assert!(source.contains("pub fn ResultList("));
    assert!(source.contains("ResultReplacementPolicy::ResetFirst"));
    assert!(source.contains("Callback<ResultRow>"));
    assert!(source.contains("Callback<Option<usize>>"));
}

// ── move_selection ──

#[test]
fn move_selection_on_empty_list_is_none() {
    assert_eq!(move_selection(Some(0), 1, 0), None);
    assert_eq!(move_selection(None, 1, 0), None);
}

#[test]
fn move_selection_from_none_defaults_current_to_zero() {
    // Mirrors d2d: `self.selected.unwrap_or(0)` before applying delta.
    assert_eq!(move_selection(None, 1, 3), Some(1));
    assert_eq!(move_selection(None, 0, 3), Some(0));
}

#[test]
fn move_selection_moves_down_one_row() {
    assert_eq!(move_selection(Some(0), 1, 3), Some(1));
    assert_eq!(move_selection(Some(1), 1, 3), Some(2));
}

#[test]
fn move_selection_moves_up_one_row() {
    assert_eq!(move_selection(Some(2), -1, 3), Some(1));
    assert_eq!(move_selection(Some(1), -1, 3), Some(0));
}

#[test]
fn move_selection_clamps_at_last_row() {
    assert_eq!(move_selection(Some(1), 5, 3), Some(2));
    assert_eq!(move_selection(Some(2), 1, 3), Some(2));
}

#[test]
fn move_selection_clamps_at_first_row() {
    assert_eq!(move_selection(Some(1), -9, 3), Some(0));
    assert_eq!(move_selection(Some(0), -1, 3), Some(0));
}

#[test]
fn move_selection_single_item_list_stays_put() {
    assert_eq!(move_selection(Some(0), 1, 1), Some(0));
    assert_eq!(move_selection(Some(0), -1, 1), Some(0));
}

// ── select_first / select_last ──

#[test]
fn select_first_on_empty_is_none() {
    assert_eq!(select_first(0), None);
}

#[test]
fn select_first_returns_zero() {
    assert_eq!(select_first(1), Some(0));
    assert_eq!(select_first(10), Some(0));
}

#[test]
fn select_last_on_empty_is_none() {
    assert_eq!(select_last(0), None);
}

#[test]
fn select_last_returns_len_minus_one() {
    assert_eq!(select_last(1), Some(0));
    assert_eq!(select_last(10), Some(9));
}

// ── result_row_key (For-loop content-hash key) ──

#[test]
fn row_key_equal_rows_at_same_index_have_equal_keys() {
    let a = ResultRow {
        title: "readme.md".into(),
        subtitle: "/docs".into(),
        snippet: "...matched text...".into(),
    };
    let b = ResultRow {
        title: "readme.md".into(),
        subtitle: "/docs".into(),
        snippet: "...matched text...".into(),
    };
    assert_eq!(result_row_key(0, &a), result_row_key(0, &b));
}

#[test]
fn row_key_changes_when_title_changes_at_same_index() {
    let a = ResultRow::new("index.md");
    let b = ResultRow::new("readme.md");
    assert_ne!(
        result_row_key(0, &a),
        result_row_key(0, &b),
        "replacing the row at a fixed index with a different title must change the key so <For> re-renders"
    );
}

#[test]
fn row_key_changes_when_subtitle_changes_at_same_index() {
    let a = ResultRow {
        title: "readme.md".into(),
        subtitle: "/docs".into(),
        snippet: String::new(),
    };
    let b = ResultRow {
        title: "readme.md".into(),
        subtitle: "/src".into(),
        snippet: String::new(),
    };
    assert_ne!(result_row_key(0, &a), result_row_key(0, &b));
}

#[test]
fn row_key_changes_when_snippet_changes_at_same_index() {
    let a = ResultRow {
        title: "readme.md".into(),
        subtitle: "/docs".into(),
        snippet: "...old match...".into(),
    };
    let b = ResultRow {
        title: "readme.md".into(),
        subtitle: "/docs".into(),
        snippet: "...new match...".into(),
    };
    assert_ne!(result_row_key(0, &a), result_row_key(0, &b));
}

#[test]
fn row_key_differs_for_different_indices_with_same_content() {
    let row = ResultRow::new("index.md");
    assert_ne!(result_row_key(0, &row), result_row_key(1, &row));
}

// ── end-to-end nav sequence (mirrors d2d's `selection_moves_and_clamps`) ──

#[test]
fn keyboard_nav_sequence_matches_d2d_semantics() {
    let len = 3;
    let mut sel = select_first(len); // Some(0), as a freshly-loaded list would be
    sel = move_selection(sel, 1, len);
    assert_eq!(sel, Some(1));
    sel = move_selection(sel, 5, len);
    assert_eq!(sel, Some(2));
    sel = move_selection(sel, -9, len);
    assert_eq!(sel, Some(0));
    sel = select_last(len);
    assert_eq!(sel, Some(2));
    sel = select_first(len);
    assert_eq!(sel, Some(0));
}
