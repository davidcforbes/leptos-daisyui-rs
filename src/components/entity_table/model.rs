//! Pure ordering, pagination, visibility, and resize behavior.

use super::types::{EntityColumn, EntitySort, EntityTablePreferences};
use crate::components::data_table::{
    ColumnVisibilityAction, MAX_COLUMN_WIDTH, clamp_page, column_visibility_action,
    effective_min_width, resized_width,
};
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
    match current {
        EntitySort::Ascending { column } if column == column_id => {
            EntitySort::descending(column_id)
        }
        EntitySort::Descending { column } if column == column_id => EntitySort::System,
        _ => EntitySort::ascending(column_id),
    }
}

/// Restores server-supplied system order without changing other preferences.
pub fn reset_sort(preferences: &mut EntityTablePreferences) -> bool {
    if preferences.sort == EntitySort::System {
        return false;
    }
    preferences.sort = EntitySort::System;
    true
}

/// Restores default visibility and widths without changing sort or page size.
pub fn reset_columns(preferences: &mut EntityTablePreferences) -> bool {
    if preferences.hidden_columns.is_empty() && preferences.column_widths.is_empty() {
        return false;
    }
    preferences.hidden_columns.clear();
    preferences.column_widths.clear();
    true
}

/// Builds a stable index permutation without cloning or reordering source rows.
pub fn sorted_indices<T>(rows: &[T], columns: &[EntityColumn<T>], sort: &EntitySort) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..rows.len()).collect();
    let Some(column_id) = sort.column() else {
        return indices;
    };
    let Some(column) = columns
        .iter()
        .find(|column| column.id == column_id && column.sortable)
    else {
        return indices;
    };
    if let Some(sort_key) = column.sort_key.as_ref() {
        let keys = rows.iter().map(|row| sort_key(row)).collect::<Vec<_>>();
        indices.sort_by(|left, right| ordered_for_direction(keys[*left].cmp(&keys[*right]), sort));
    } else if let Some(compare) = column.comparator.as_ref() {
        indices.sort_by(|left, right| {
            ordered_for_direction(compare(&rows[*left], &rows[*right]), sort)
        });
    }
    indices
}

fn ordered_for_direction(ordering: std::cmp::Ordering, sort: &EntitySort) -> std::cmp::Ordering {
    match sort {
        EntitySort::Descending { .. } => ordering.reverse(),
        EntitySort::Ascending { .. } | EntitySort::System => ordering,
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
        return;
    }
    if !valid_page_size(preferences.page_size) {
        preferences.page_size = ENTITY_PAGE_SIZE_CHOICES[0];
    }

    if let Some(column_id) = preferences.sort.column()
        && !columns.iter().any(|column| {
            column.id == column_id
                && column.sortable
                && (column.comparator.is_some() || column.sort_key.is_some())
        })
    {
        preferences.sort = EntitySort::System;
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
