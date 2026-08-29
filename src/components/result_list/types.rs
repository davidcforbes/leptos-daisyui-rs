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

use std::{collections::HashMap, fmt};

/// One entry in a [`ResultList`](super::ResultList) — a ranked search result.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
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

/// A result whose stable business identity and activation payload are kept
/// separate from its display-only [`ResultRow`].
///
/// The `key` must be non-empty after trimming and unique within the current
/// list supplied to [`KeyedResultList`](super::KeyedResultList).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResultListItem<T> {
    /// Stable business identity used for selection and DOM reconciliation.
    pub key: String,
    /// Display-only title and secondary text.
    pub row: ResultRow,
    /// Typed value returned to the consumer when this result is activated.
    pub payload: T,
}

impl<T> ResultListItem<T> {
    /// Creates a keyed result from its stable key, display row, and payload.
    pub fn new(key: impl Into<String>, row: ResultRow, payload: T) -> Self {
        Self {
            key: key.into(),
            row,
            payload,
        }
    }
}

/// Why a keyed result set cannot be rendered safely.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultListKeyError {
    /// The key at `index` is empty or contains only whitespace.
    EmptyKey {
        /// Position of the invalid item in the current list.
        index: usize,
    },
    /// The same key occurs at two positions in the current list.
    DuplicateKey {
        /// Repeated stable key.
        key: String,
        /// Position where the key first occurred.
        first_index: usize,
        /// Position where the key occurred again.
        duplicate_index: usize,
    },
}

impl fmt::Display for ResultListKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey { index } => {
                write!(formatter, "result at index {index} has an empty stable key")
            }
            Self::DuplicateKey {
                key,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "result key `{key}` is duplicated at indices {first_index} and {duplicate_index}"
            ),
        }
    }
}

impl std::error::Error for ResultListKeyError {}

/// Validates that every keyed result has a non-blank key and that all keys
/// are unique within `items`.
pub fn validate_result_list_items<T>(
    items: &[ResultListItem<T>],
) -> Result<(), ResultListKeyError> {
    let mut first_indices = HashMap::with_capacity(items.len());

    for (index, item) in items.iter().enumerate() {
        if item.key.trim().is_empty() {
            return Err(ResultListKeyError::EmptyKey { index });
        }

        if let Some(first_index) = first_indices.insert(item.key.as_str(), index) {
            return Err(ResultListKeyError::DuplicateKey {
                key: item.key.clone(),
                first_index,
                duplicate_index: index,
            });
        }
    }

    Ok(())
}

/// Preserves `current` when that key still exists; otherwise selects the
/// first current result. Returns `None` for an empty list.
pub fn reconcile_result_key<T>(
    current: Option<&str>,
    items: &[ResultListItem<T>],
) -> Option<String> {
    current
        .filter(|key| items.iter().any(|item| item.key == *key))
        .map(str::to_owned)
        .or_else(|| items.first().map(|item| item.key.clone()))
}

/// Moves a keyed selection through the current item order and clamps at the
/// first or last result.
pub fn move_result_key<T>(
    current: Option<&str>,
    delta: i32,
    items: &[ResultListItem<T>],
) -> Option<String> {
    let current_index = current.and_then(|key| items.iter().position(|item| item.key == key));
    move_selection(current_index, delta, items.len()).map(|index| items[index].key.clone())
}

/// Clones the latest item carrying `key`, if the current list still has it.
pub fn current_result_item<T: Clone>(
    items: &[ResultListItem<T>],
    key: &str,
) -> Option<ResultListItem<T>> {
    items.iter().find(|item| item.key == key).cloned()
}

/// Builds a collision-free option id by encoding every UTF-8 key byte as two
/// lowercase hexadecimal digits.
pub(crate) fn keyed_option_dom_id(instance: u64, key: &str) -> String {
    use fmt::Write as _;

    let mut id = format!("ld-result-list-{instance}-option-");
    for byte in key.as_bytes() {
        write!(&mut id, "{byte:02x}").expect("writing to a String cannot fail");
    }
    id
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

/// Content fingerprint of a result row, combined with its index, used as the
/// row key inside [`ResultList`](super::ResultList)'s `<For>` list so a row
/// re-renders whenever *any* of its render-relevant fields change — not just
/// when its index changes. Keying purely by index let `<For>` reuse stale
/// views when `items` was replaced wholesale (the component's primary flow:
/// search results changing per keystroke): rows at unchanged indices kept
/// their old titles/snippets on screen, and — because the `on:click` handler
/// closes over the row value captured when that view was created — clicking
/// a row fired `on_select` with the *previous* result set's row instead of
/// the current one. Hashing the row's content into the key forces `<For>` to
/// tear down and rebuild the view (with a fresh closure over the new row)
/// whenever the content at that index changes.
pub fn result_row_key(i: usize, row: &ResultRow) -> (usize, u64) {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    row.hash(&mut hasher);
    (i, hasher.finish())
}
