//! Pure ordering, pagination, visibility, and resize behavior.

use super::types::{
    EntityColumn, EntityComparator, EntitySort, EntitySortDirection, EntityTablePreferences,
};
use crate::components::data_table::{
    ColumnVisibilityAction, MAX_COLUMN_WIDTH, clamp_page, column_visibility_action,
    effective_min_width, resized_width,
};
use std::collections::BTreeSet;
use std::rc::Rc;

/// Supported row counts for client-snapshot tables.
pub const ENTITY_PAGE_SIZE_CHOICES: [usize; 3] = [25, 50, 100];

/// Returns whether a row count is one of the opinionated choices.
pub fn valid_page_size(page_size: usize) -> bool {
    ENTITY_PAGE_SIZE_CHOICES.contains(&page_size)
}

/// Cycles one sortable column through system, ascending, descending, and system order.
pub fn next_sort(current: &EntitySort, column_id: &str, sortable: bool) -> EntitySort {
    if !sortable {
        return current.clone();
    }
    match current.direction_for(column_id) {
        Some(EntitySortDirection::Ascending) => EntitySort::descending(column_id),
        Some(EntitySortDirection::Descending) => EntitySort::System,
        None => EntitySort::ascending(column_id),
    }
}

/// Cycles one sortable column without discarding the other active clauses.
///
/// An absent clause is appended ascending, ascending becomes descending, and
/// descending is removed. Existing clauses retain their relative priority.
pub fn next_sort_additive(current: &EntitySort, column_id: &str, sortable: bool) -> EntitySort {
    if !sortable {
        return current.clone();
    }
    let mut clauses = current.clauses();
    match clauses.iter().position(|clause| clause.column == column_id) {
        Some(index) if clauses[index].direction == EntitySortDirection::Ascending => {
            clauses[index].direction = EntitySortDirection::Descending;
        }
        Some(index) => {
            clauses.remove(index);
        }
        None => clauses.push(super::types::EntitySortColumn::ascending(column_id)),
    }
    EntitySort::multiple(clauses)
}

/// Adjacent direction for an ordered-column preference change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityColumnMove {
    /// Move the column one position toward the start of the table.
    Earlier,
    /// Move the column one position toward the end of the table.
    Later,
}

/// Returns cloned column definitions in canonical preference order.
pub fn ordered_columns<T>(
    preferences: &EntityTablePreferences,
    columns: &[EntityColumn<T>],
) -> Vec<EntityColumn<T>> {
    canonical_column_ids(preferences, columns)
        .into_iter()
        .filter_map(|id| columns.iter().find(|column| column.id == id).cloned())
        .collect()
}

/// Moves one column by one position in canonical preference order.
pub fn move_column<T>(
    preferences: &mut EntityTablePreferences,
    columns: &[EntityColumn<T>],
    column_id: &str,
    direction: EntityColumnMove,
) -> bool {
    let mut order = canonical_column_ids(preferences, columns);
    let Some(index) = order.iter().position(|id| id == column_id) else {
        return false;
    };
    let target = match direction {
        EntityColumnMove::Earlier if index > 0 => index - 1,
        EntityColumnMove::Later if index + 1 < order.len() => index + 1,
        EntityColumnMove::Earlier | EntityColumnMove::Later => return false,
    };
    order.swap(index, target);
    preferences.column_order = order;
    true
}

fn canonical_column_ids<T>(
    preferences: &EntityTablePreferences,
    columns: &[EntityColumn<T>],
) -> Vec<String> {
    let valid = columns
        .iter()
        .map(|column| column.id)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut order = preferences
        .column_order
        .iter()
        .filter(|id| valid.contains(id.as_str()) && seen.insert((*id).clone()))
        .cloned()
        .collect::<Vec<_>>();
    order.extend(
        columns
            .iter()
            .filter(|column| seen.insert(column.id.to_owned()))
            .map(|column| column.id.to_owned()),
    );
    order
}

/// Restores server-supplied system order without changing other preferences.
pub fn reset_sort(preferences: &mut EntityTablePreferences) -> bool {
    if preferences.sort.is_system() {
        return false;
    }
    preferences.sort = EntitySort::System;
    true
}

/// Restores default visibility, widths, and order without changing sort or page size.
pub fn reset_columns(preferences: &mut EntityTablePreferences) -> bool {
    if preferences.hidden_columns.is_empty()
        && preferences.column_widths.is_empty()
        && preferences.column_order.is_empty()
    {
        return false;
    }
    preferences.hidden_columns.clear();
    preferences.column_widths.clear();
    preferences.column_order.clear();
    true
}

/// Builds a stable index permutation without cloning or reordering source rows.
pub fn sorted_indices<T>(rows: &[T], columns: &[EntityColumn<T>], sort: &EntitySort) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..rows.len()).collect();
    let prepared = sort
        .clauses()
        .iter()
        .filter_map(|clause| {
            let column = columns
                .iter()
                .find(|column| column.id == clause.column && column.sortable)?;
            if let Some(sort_key) = column.sort_key.as_ref() {
                return Some(PreparedSort::Keys {
                    direction: clause.direction,
                    keys: rows.iter().map(|row| sort_key(row)).collect(),
                });
            }
            column
                .comparator
                .as_ref()
                .map(|compare| PreparedSort::Comparator {
                    direction: clause.direction,
                    compare: Rc::clone(compare),
                })
        })
        .collect::<Vec<_>>();
    if prepared.is_empty() {
        return indices;
    }
    indices.sort_by(|left, right| {
        prepared
            .iter()
            .map(|prepared| prepared.compare(*left, *right, rows))
            .find(|ordering| !ordering.is_eq())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    indices
}

enum PreparedSort<T> {
    Keys {
        direction: EntitySortDirection,
        keys: Vec<String>,
    },
    Comparator {
        direction: EntitySortDirection,
        compare: EntityComparator<T>,
    },
}

impl<T> PreparedSort<T> {
    fn compare(&self, left: usize, right: usize, rows: &[T]) -> std::cmp::Ordering {
        match self {
            Self::Keys { direction, keys } => {
                ordered_for_direction(keys[left].cmp(&keys[right]), *direction)
            }
            Self::Comparator { direction, compare } => {
                ordered_for_direction(compare(&rows[left], &rows[right]), *direction)
            }
        }
    }
}

fn ordered_for_direction(
    ordering: std::cmp::Ordering,
    direction: EntitySortDirection,
) -> std::cmp::Ordering {
    match direction {
        EntitySortDirection::Descending => ordering.reverse(),
        EntitySortDirection::Ascending => ordering,
    }
}

/// Memoizes the complete sorted permutation by immutable dataset identity and sort.
///
/// Pagination, visibility, and column-width changes may rerender the table, but
/// they reuse this permutation instead of re-running an `O(n log n)` sort.
pub(crate) struct SortedIndexCache<T> {
    rows: Option<Rc<Vec<T>>>,
    sort: EntitySort,
    indices: Rc<Vec<usize>>,
}

impl<T> SortedIndexCache<T> {
    pub(crate) fn new() -> Self {
        Self {
            rows: None,
            sort: EntitySort::System,
            indices: Rc::new(Vec::new()),
        }
    }

    pub(crate) fn indices(
        &mut self,
        rows: Rc<Vec<T>>,
        columns: &[EntityColumn<T>],
        sort: &EntitySort,
    ) -> Rc<Vec<usize>> {
        let unchanged = self
            .rows
            .as_ref()
            .is_some_and(|cached| Rc::ptr_eq(cached, &rows))
            && self.sort == *sort;
        if !unchanged {
            self.indices = Rc::new(sorted_indices(rows.as_slice(), columns, sort));
            self.rows = Some(rows);
            self.sort = sort.clone();
        }
        Rc::clone(&self.indices)
    }
}

/// Resets pagination only when a selector loads a different dataset identity.
pub fn page_after_dataset_change<T: PartialEq>(
    current_page: usize,
    previous_dataset: T,
    next_dataset: T,
) -> usize {
    if previous_dataset == next_dataset {
        current_page
    } else {
        0
    }
}

/// Preserves the current page across row deltas when it is still valid.
pub fn page_after_row_delta(current_page: usize, page_size: usize, total_rows: usize) -> usize {
    clamp_page(current_page, page_size, total_rows)
}

/// Stores a requested width after applying the same bounds as `DataTable`.
pub fn set_preferred_width(
    preferences: &mut EntityTablePreferences,
    column_id: impl Into<String>,
    requested_width: f64,
    minimum_width: Option<u32>,
) {
    let bounded = resized_width(
        requested_width,
        0.0,
        0.0,
        effective_min_width(minimum_width),
    )
    .round()
    .clamp(0.0, MAX_COLUMN_WIDTH) as u32;
    preferences.column_widths.insert(column_id.into(), bounded);
}

/// Toggles an optional column and refuses to hide required or last-visible columns.
pub fn toggle_hidden_column<T>(
    preferences: &mut EntityTablePreferences,
    columns: &[EntityColumn<T>],
    column_id: &str,
) -> bool {
    let Some(column) = columns.iter().find(|column| column.id == column_id) else {
        return false;
    };
    let visible_count = columns
        .iter()
        .filter(|candidate| !preferences.hidden_columns.contains(candidate.id))
        .count();
    let is_hidden = preferences.hidden_columns.contains(column_id);
    match column_visibility_action(is_hidden, column.required, visible_count) {
        ColumnVisibilityAction::Show => preferences.hidden_columns.remove(column_id),
        ColumnVisibilityAction::Hide => preferences.hidden_columns.insert(column_id.to_owned()),
        ColumnVisibilityAction::Unchanged => false,
    }
}

/// Returns a normalized clone for the declared schema and column set.
///
/// The supplied value is never mutated. Invalid versions reset to defaults;
/// unknown, required, or unsupported column preferences are removed
/// deterministically.
pub fn normalize_preferences<T>(
    preferences: &EntityTablePreferences,
    schema_version: u16,
    columns: &[EntityColumn<T>],
) -> EntityTablePreferences {
    let mut normalized = preferences.clone();
    normalize_preferences_in_place(&mut normalized, schema_version, columns);
    normalized
}

pub(crate) fn emit_normalized_preference_change<T>(
    current: &EntityTablePreferences,
    schema_version: u16,
    columns: &[EntityColumn<T>],
    update: impl FnOnce(&mut EntityTablePreferences),
    emit: impl FnOnce(EntityTablePreferences),
) -> EntityTablePreferences {
    let mut replacement = normalize_preferences(current, schema_version, columns);
    update(&mut replacement);
    replacement = normalize_preferences(&replacement, schema_version, columns);
    emit(replacement.clone());
    replacement
}

fn normalize_preferences_in_place<T>(
    preferences: &mut EntityTablePreferences,
    schema_version: u16,
    columns: &[EntityColumn<T>],
) {
    if preferences.schema_version != schema_version {
        *preferences = EntityTablePreferences::new(schema_version);
    }
    if !valid_page_size(preferences.page_size) {
        preferences.page_size = ENTITY_PAGE_SIZE_CHOICES[0];
    }

    let mut seen_sort_ids = BTreeSet::new();
    preferences.sort = EntitySort::multiple(
        preferences
            .sort
            .clauses()
            .iter()
            .filter(|clause| {
                seen_sort_ids.insert(clause.column.clone())
                    && columns.iter().any(|column| {
                        column.id == clause.column
                            && column.sortable
                            && (column.comparator.is_some() || column.sort_key.is_some())
                    })
            })
            .cloned(),
    );

    let valid_column_ids = columns
        .iter()
        .map(|column| column.id)
        .collect::<BTreeSet<_>>();
    let mut seen_column_ids = BTreeSet::new();
    preferences
        .column_order
        .retain(|id| valid_column_ids.contains(id.as_str()) && seen_column_ids.insert(id.clone()));
    for column in columns {
        if seen_column_ids.insert(column.id.to_owned()) {
            preferences.column_order.push(column.id.to_owned());
        }
    }

    preferences.hidden_columns.retain(|id| {
        columns
            .iter()
            .any(|column| column.id == id && !column.required)
    });
    if columns
        .iter()
        .all(|column| preferences.hidden_columns.contains(column.id))
        && let Some(first) = columns.first()
    {
        preferences.hidden_columns.remove(first.id);
    }

    preferences.column_widths.retain(|id, width| {
        let Some(column) = columns
            .iter()
            .find(|column| column.id == id && column.resizable)
        else {
            return false;
        };
        *width = resized_width(
            f64::from(*width),
            0.0,
            0.0,
            effective_min_width(column.min_width),
        )
        .round() as u32;
        true
    });
}
