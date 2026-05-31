//! Table UI helpers for the graphic-mode editor (em-berj.3).
//!
//! Three small DOM operations the keydown handler in
//! `graphic_editor.rs` needs:
//!
//! 1. [`caret_cell`] — find the `<td>` / `<th>` containing the
//!    current Selection's anchor, or `None` if the caret is outside
//!    a table cell.
//! 2. [`is_html_table_cell`] — distinguish "raw `<table>` markdown
//!    cell" (no editable source range from pulldown-cmark, so CRUD
//!    is refused) from a normal GFM-table cell.
//! 3. [`next_cell`] / [`prev_cell`] — move the caret one cell over,
//!    wrapping to the next / previous row when at the row's edge.
//!    Returns `false` when there's no next / previous cell (caller
//!    can decide whether to insert a new row or let Tab escape).
//!
//! The Phase-3 cell-safety filter ([`editmark_core::filter_cell_insert`])
//! lives in editmark-core; this module is just the DOM glue.

use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Node, Window};

/// Identifying coordinates for one table cell within the source —
/// produced by [`resolve_cell_in_dom`] from a DOM hit (right-click,
/// caret position, …) and consumed by the table-edit splice helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellCoord {
    /// Byte range of the enclosing `<table>` block in the source
    /// (read from the `data-em-src` attribute stamped during
    /// the decorate-for-editing pass).
    pub table_source_start: usize,
    pub table_source_end: usize,
    /// Document-order index of the clicked row, counting transparently
    /// across `<thead>` / `<tbody>` wrappers.  0 = header row, 1+ =
    /// body rows.
    pub dom_row_idx: usize,
    /// Column index of the clicked cell within its row.
    pub col_idx: usize,
}

/// Walk up from `cell` to the enclosing `<table>` and resolve the
/// hit into a [`CellCoord`].  Returns `None` when the cell isn't
/// inside a table whose `<table>` element carries `data-em-src`.
pub fn resolve_cell_in_dom(cell: &Element) -> Option<CellCoord> {
    let table = ancestor_with_tag(cell, "table")?;
    let attr = table.get_attribute("data-em-src")?;
    let (start_s, end_s) = attr.split_once('-')?;
    let table_source_start: usize = start_s.parse().ok()?;
    let table_source_end: usize = end_s.parse().ok()?;
    let dom_row_idx = row_index_in_table(&table, cell)?;
    let col_idx = cell_column_index(cell)?;
    Some(CellCoord {
        table_source_start,
        table_source_end,
        dom_row_idx,
        col_idx,
    })
}

fn ancestor_with_tag(start: &Element, tag: &str) -> Option<Element> {
    let mut current: Option<Element> = Some(start.clone());
    while let Some(el) = current {
        if el.tag_name().eq_ignore_ascii_case(tag) {
            return Some(el);
        }
        current = el.parent_element();
    }
    None
}

/// Count which `<tr>` (in document order, transparent across
/// `<thead>` / `<tbody>`) the given `cell` lives in.
fn row_index_in_table(table: &Element, cell: &Element) -> Option<usize> {
    let target_row = cell.parent_element()?;
    let mut idx = 0usize;
    let mut found = false;
    walk_rows(table, &mut |row| {
        if row.is_same_node(Some(target_row.unchecked_ref::<Node>())) {
            found = true;
            return false; // stop
        }
        idx += 1;
        true
    });
    if found {
        Some(idx)
    } else {
        None
    }
}

fn walk_rows<F: FnMut(&Element) -> bool>(table: &Element, visit: &mut F) {
    let mut child = table.first_element_child();
    while let Some(el) = child {
        let tag = el.tag_name().to_ascii_lowercase();
        match tag.as_str() {
            "thead" | "tbody" | "tfoot" => {
                let mut row = el.first_element_child();
                while let Some(r) = row {
                    if r.tag_name().eq_ignore_ascii_case("tr") {
                        if !visit(&r) {
                            return;
                        }
                    }
                    row = r.next_element_sibling();
                }
            }
            "tr" => {
                if !visit(&el) {
                    return;
                }
            }
            _ => {}
        }
        child = el.next_element_sibling();
    }
}

/// Walk up the DOM from `node` looking for the first ancestor that is
/// a `<td>` or `<th>`.  Returns the cell element when found, or `None`.
pub fn cell_ancestor(node: &Node) -> Option<Element> {
    let mut current: Option<Node> = Some(node.clone());
    while let Some(n) = current {
        if let Some(el) = n.dyn_ref::<Element>() {
            let tag = el.tag_name().to_ascii_lowercase();
            if tag == "td" || tag == "th" {
                return Some(el.clone());
            }
        }
        current = n.parent_node();
    }
    None
}

/// The `<td>` / `<th>` containing the current window selection's
/// anchor, or `None` when no anchor or no enclosing cell.
pub fn caret_cell(window: &Window) -> Option<Element> {
    let selection = window.get_selection().ok().flatten()?;
    let anchor = selection.anchor_node()?;
    cell_ancestor(&anchor)
}

/// `true` when the table containing `cell` came from raw `<table>`
/// HTML in the source (pulldown-cmark surfaces these with empty
/// `cell_source_ranges`).  In v1 the decorate-for-editing pass marks
/// the cell with `data-em-cell-readonly="true"`; this helper just
/// checks for that flag so callers don't reach into the DOM tree
/// themselves.
pub fn is_html_table_cell(cell: &Element) -> bool {
    cell.has_attribute("data-em-cell-readonly")
}

/// Find the cell immediately after `cell` in document order — same
/// row's next sibling, or the first cell of the next row.  Returns
/// `None` at the end of the last row of the table.
pub fn next_cell(cell: &Element) -> Option<Element> {
    // Try a same-row sibling first.
    if let Some(next) = cell.next_element_sibling() {
        let tag = next.tag_name().to_ascii_lowercase();
        if tag == "td" || tag == "th" {
            return Some(next);
        }
    }
    // No more siblings — walk to the next <tr> and take its first cell.
    let row = cell.parent_element()?;
    let mut sibling = row.next_element_sibling();
    while let Some(s) = sibling {
        let tag = s.tag_name().to_ascii_lowercase();
        if tag == "tr" {
            return first_cell_of_row(&s);
        }
        // Skip over `<thead>` / `<tbody>` wrappers: descend into their
        // first `<tr>`.
        if tag == "thead" || tag == "tbody" {
            if let Some(inner) = s.first_element_child() {
                if inner.tag_name().eq_ignore_ascii_case("tr") {
                    return first_cell_of_row(&inner);
                }
            }
        }
        sibling = s.next_element_sibling();
    }
    // Also climb out of <thead> / <tbody> wrappers if the current row
    // was inside one.
    let section = row.parent_element()?;
    let section_tag = section.tag_name().to_ascii_lowercase();
    if section_tag == "thead" || section_tag == "tbody" {
        let mut next_section = section.next_element_sibling();
        while let Some(sec) = next_section {
            let tag = sec.tag_name().to_ascii_lowercase();
            if tag == "tbody" || tag == "thead" {
                if let Some(first_tr) = sec.first_element_child() {
                    if first_tr.tag_name().eq_ignore_ascii_case("tr") {
                        return first_cell_of_row(&first_tr);
                    }
                }
            }
            next_section = sec.next_element_sibling();
        }
    }
    None
}

/// Symmetric to [`next_cell`].
pub fn prev_cell(cell: &Element) -> Option<Element> {
    if let Some(prev) = cell.previous_element_sibling() {
        let tag = prev.tag_name().to_ascii_lowercase();
        if tag == "td" || tag == "th" {
            return Some(prev);
        }
    }
    let row = cell.parent_element()?;
    let mut sibling = row.previous_element_sibling();
    while let Some(s) = sibling {
        let tag = s.tag_name().to_ascii_lowercase();
        if tag == "tr" {
            return last_cell_of_row(&s);
        }
        if tag == "thead" || tag == "tbody" {
            if let Some(inner) = s.last_element_child() {
                if inner.tag_name().eq_ignore_ascii_case("tr") {
                    return last_cell_of_row(&inner);
                }
            }
        }
        sibling = s.previous_element_sibling();
    }
    let section = row.parent_element()?;
    let section_tag = section.tag_name().to_ascii_lowercase();
    if section_tag == "thead" || section_tag == "tbody" {
        let mut prev_section = section.previous_element_sibling();
        while let Some(sec) = prev_section {
            let tag = sec.tag_name().to_ascii_lowercase();
            if tag == "tbody" || tag == "thead" {
                if let Some(last_tr) = sec.last_element_child() {
                    if last_tr.tag_name().eq_ignore_ascii_case("tr") {
                        return last_cell_of_row(&last_tr);
                    }
                }
            }
            prev_section = sec.previous_element_sibling();
        }
    }
    None
}

/// Column index of `cell` within its `<tr>` parent — i.e. how many
/// `<td>` / `<th>` siblings precede it.  Returns `0` for the first
/// cell, `None` when the cell has no `<tr>` parent.
pub fn cell_column_index(cell: &Element) -> Option<usize> {
    let row = cell.parent_element()?;
    let mut i = 0usize;
    let mut sibling = row.first_element_child();
    while let Some(s) = sibling {
        if s.is_same_node(Some(cell.unchecked_ref::<Node>())) {
            return Some(i);
        }
        let tag = s.tag_name().to_ascii_lowercase();
        if tag == "td" || tag == "th" {
            i += 1;
        }
        sibling = s.next_element_sibling();
    }
    None
}

/// `<tr>` immediately after `cell`'s row in the same table (descending
/// through `<thead>` → `<tbody>` if needed).
pub fn next_row(cell: &Element) -> Option<Element> {
    let row = cell.parent_element()?;
    if let Some(sibling) = next_tr_sibling(&row) {
        return Some(sibling);
    }
    // Climb out of <thead> / <tbody> wrappers.
    let section = row.parent_element()?;
    let stag = section.tag_name().to_ascii_lowercase();
    if stag == "thead" || stag == "tbody" {
        let mut next_section = section.next_element_sibling();
        while let Some(sec) = next_section {
            let t = sec.tag_name().to_ascii_lowercase();
            if t == "tbody" || t == "thead" {
                if let Some(first_tr) = sec.first_element_child() {
                    if first_tr.tag_name().eq_ignore_ascii_case("tr") {
                        return Some(first_tr);
                    }
                }
            }
            next_section = sec.next_element_sibling();
        }
    }
    None
}

/// Symmetric to [`next_row`].
pub fn prev_row(cell: &Element) -> Option<Element> {
    let row = cell.parent_element()?;
    if let Some(sibling) = prev_tr_sibling(&row) {
        return Some(sibling);
    }
    let section = row.parent_element()?;
    let stag = section.tag_name().to_ascii_lowercase();
    if stag == "thead" || stag == "tbody" {
        let mut prev_section = section.previous_element_sibling();
        while let Some(sec) = prev_section {
            let t = sec.tag_name().to_ascii_lowercase();
            if t == "tbody" || t == "thead" {
                if let Some(last_tr) = sec.last_element_child() {
                    if last_tr.tag_name().eq_ignore_ascii_case("tr") {
                        return Some(last_tr);
                    }
                }
            }
            prev_section = sec.previous_element_sibling();
        }
    }
    None
}

fn next_tr_sibling(row: &Element) -> Option<Element> {
    let mut s = row.next_element_sibling();
    while let Some(n) = s {
        if n.tag_name().eq_ignore_ascii_case("tr") {
            return Some(n);
        }
        s = n.next_element_sibling();
    }
    None
}

fn prev_tr_sibling(row: &Element) -> Option<Element> {
    let mut s = row.previous_element_sibling();
    while let Some(n) = s {
        if n.tag_name().eq_ignore_ascii_case("tr") {
            return Some(n);
        }
        s = n.previous_element_sibling();
    }
    None
}

/// Cell in the row above `cell` at the same column index.  Returns
/// `None` when there is no row above.
pub fn cell_above(cell: &Element) -> Option<Element> {
    let col = cell_column_index(cell)?;
    let row = prev_row(cell)?;
    cell_at_column(&row, col)
}

/// Symmetric to [`cell_above`].
pub fn cell_below(cell: &Element) -> Option<Element> {
    let col = cell_column_index(cell)?;
    let row = next_row(cell)?;
    cell_at_column(&row, col)
}

fn cell_at_column(row: &Element, col: usize) -> Option<Element> {
    let mut i = 0usize;
    let mut sibling = row.first_element_child();
    while let Some(s) = sibling {
        let tag = s.tag_name().to_ascii_lowercase();
        if tag == "td" || tag == "th" {
            if i == col {
                return Some(s);
            }
            i += 1;
        }
        sibling = s.next_element_sibling();
    }
    // Column is past this row's last cell — clamp to the last cell so
    // the caret still lands somewhere visible.
    last_cell_of_row(row)
}

fn first_cell_of_row(tr: &Element) -> Option<Element> {
    let mut child = tr.first_element_child();
    while let Some(c) = child {
        let tag = c.tag_name().to_ascii_lowercase();
        if tag == "td" || tag == "th" {
            return Some(c);
        }
        child = c.next_element_sibling();
    }
    None
}

fn last_cell_of_row(tr: &Element) -> Option<Element> {
    let mut child = tr.last_element_child();
    while let Some(c) = child {
        let tag = c.tag_name().to_ascii_lowercase();
        if tag == "td" || tag == "th" {
            return Some(c);
        }
        child = c.previous_element_sibling();
    }
    None
}

/// Move the window selection's caret into `cell`, placed at the end
/// of its existing text content.  Used by Tab / Shift+Tab to land the
/// caret in the next / previous cell after navigation.
pub fn focus_cell(window: &Window, document: &Document, cell: &Element) {
    let Ok(range) = document.create_range() else {
        return;
    };
    // Place the range at the end of the cell's contents.  If the cell
    // has children, that's `range.set_start_after(last_child)`;
    // otherwise the cell itself becomes the anchor.
    let cell_node: &Node = JsCast::unchecked_ref(cell);
    if let Some(last) = cell.last_child() {
        let _ = range.set_start_after(&last);
    } else {
        let _ = range.set_start(cell_node, 0);
    }
    let _ = range.collapse_with_to_start(true);
    if let Ok(Some(selection)) = window.get_selection() {
        let _ = selection.remove_all_ranges();
        let _ = selection.add_range(&range);
    }
}
