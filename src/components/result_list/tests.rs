use super::*;

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
