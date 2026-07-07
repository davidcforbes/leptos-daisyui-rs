//! Pure clipboard-export helpers for [`DataTable`](super::DataTable) --
//! plain-text serialization of a cell, a row, or a row prefixed with column
//! headers (paste-into-spreadsheet friendly). Mirrors d2d-ui's
//! `cell_clipboard_text` / `row_clipboard_text` / `row_with_headers_clipboard_text`
//! (`Rust-DeskApp/crates/d2d-ui/src/controls/data_table.rs`), adapted to this
//! crate's `TableRow` (`HashMap<&'static str, String>`) data model.
//!
//! These operate on the raw `TableRow` string data (not typed-cell overrides
//! from `Column::with_typed_cell`) since the row's string data is the
//! underlying source of truth regardless of how a cell is displayed.

use crate::components::data_table::types::{Column, TableRow};

/// Text of a single cell: `row[col.id]`, or `""` if the row has no entry for
/// that column.
pub fn cell_text(row: &TableRow, col: &Column) -> String {
    row.get(col.id).cloned().unwrap_or_default()
}

/// Tab-separated text of a whole row across the given columns, in column order.
pub fn row_text(row: &TableRow, columns: &[Column]) -> String {
    columns
        .iter()
        .map(|c| cell_text(row, c))
        .collect::<Vec<_>>()
        .join("\t")
}

/// A row's text prefixed with a tab-separated header line -- two
/// newline-joined lines, ready to paste into a spreadsheet.
pub fn row_with_headers_text(row: &TableRow, columns: &[Column]) -> String {
    let headers = columns
        .iter()
        .map(|c| c.header)
        .collect::<Vec<_>>()
        .join("\t");
    format!("{headers}\n{}", row_text(row, columns))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cols() -> Vec<Column> {
        vec![Column::new("name", "Name"), Column::new("email", "Email")]
    }

    fn row() -> TableRow {
        HashMap::from([
            ("name", "Alice".to_string()),
            ("email", "alice@example.com".to_string()),
        ])
    }

    // ── cell_text ──

    #[test]
    fn cell_text_returns_value() {
        let r = row();
        assert_eq!(cell_text(&r, &Column::new("name", "Name")), "Alice");
    }

    #[test]
    fn cell_text_missing_column_is_empty() {
        let r = row();
        assert_eq!(cell_text(&r, &Column::new("missing", "Missing")), "");
    }

    // ── row_text ──

    #[test]
    fn row_text_joins_columns_with_tabs_in_order() {
        let r = row();
        assert_eq!(row_text(&r, &cols()), "Alice\talice@example.com");
    }

    #[test]
    fn row_text_empty_columns_is_empty_string() {
        let r = row();
        assert_eq!(row_text(&r, &[]), "");
    }

    #[test]
    fn row_text_missing_values_render_as_empty_cells() {
        let r: TableRow = HashMap::from([("name", "Alice".to_string())]);
        assert_eq!(row_text(&r, &cols()), "Alice\t");
    }

    #[test]
    fn row_text_single_column() {
        let r = row();
        let single = vec![Column::new("email", "Email")];
        assert_eq!(row_text(&r, &single), "alice@example.com");
    }

    // ── row_with_headers_text ──

    #[test]
    fn row_with_headers_text_prefixes_header_line() {
        let r = row();
        assert_eq!(
            row_with_headers_text(&r, &cols()),
            "Name\tEmail\nAlice\talice@example.com"
        );
    }

    #[test]
    fn row_with_headers_text_empty_columns() {
        let r = row();
        assert_eq!(row_with_headers_text(&r, &[]), "\n");
    }

    #[test]
    fn row_with_headers_text_uses_header_not_id() {
        let r: TableRow = HashMap::from([("email_addr", "bob@example.com".to_string())]);
        let cols = vec![Column::new("email_addr", "E-Mail Address")];
        assert_eq!(
            row_with_headers_text(&r, &cols),
            "E-Mail Address\nbob@example.com"
        );
    }
}
