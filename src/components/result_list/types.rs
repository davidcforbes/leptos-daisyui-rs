//! Data model and pure keyboard-navigation logic for
//! [`ResultList`](super::ResultList).
//!
//! Ported from d2d-ui's `controls::result_list::ResultList` — a self-painting
//! Direct2D control with variable-height rows and manual row-height
//! measurement/scroll-offset math (needed because Direct2D has no native
//! layout or scrolling). None of that ports: on the web, variable-height rows
//! are free (the browser word-wraps the secondary line and the row grows to
//! fit), and scrolling the selected row into view is a single
//! `Element::scroll_into_view` call driven from the component. What *does*
//! port is the index-movement math, kept pure here so it is unit-testable
//! without a DOM.

/// One entry in a [`ResultList`](super::ResultList) — a ranked search result.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResultRow {
    /// Primary, single-line title (rendered bold, truncated with an ellipsis
    /// if it overflows).
    pub title: String,
    /// Secondary line shown when [`snippet`](Self::snippet) is empty — e.g. a
    /// file path.
    pub subtitle: String,
    /// Secondary line shown in preference to [`subtitle`](Self::subtitle) —
    /// e.g. a matched search snippet. Word-wraps across multiple lines; the
    /// row grows to fit (mirrors d2d-ui's wrapped second line).
    pub snippet: String,
}

impl ResultRow {
    /// Convenience constructor for a title-only row (empty subtitle/snippet).
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: String::new(),
            snippet: String::new(),
        }
    }

    /// The line rendered below the title: the snippet if non-empty, else the
    /// subtitle. Mirrors d2d-ui's `ResultList::draw` row-2 selection.
    pub fn secondary_line(&self) -> &str {
        if self.snippet.is_empty() {
            &self.subtitle
        } else {
            &self.snippet
        }
    }
}

/// Move the selection by `delta` rows, clamped to `[0, len - 1]` (no
/// wraparound). Direct port of d2d-ui's `ResultList::move_selection`.
///
/// A `None` current selection is treated as row `0` before applying `delta`
/// — matching d2d's `self.selected.unwrap_or(0)` — so
/// `move_selection(None, 1, len)` selects row `1`, not row `0`. Callers that
/// want "select row 0" from an unselected list should use [`select_first`].
///
/// Returns `None` when `len == 0`.
pub fn move_selection(current: Option<usize>, delta: i32, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let cur = current.unwrap_or(0) as i32;
    let next = (cur + delta).clamp(0, len as i32 - 1);
    Some(next as usize)
}

/// Select the first row (`Home` key), or `None` if there are no rows.
pub fn select_first(len: usize) -> Option<usize> {
    if len == 0 { None } else { Some(0) }
}

/// Select the last row (`End` key), or `None` if there are no rows.
pub fn select_last(len: usize) -> Option<usize> {
    len.checked_sub(1)
}
