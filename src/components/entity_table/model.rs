//! Pure ordering, pagination, visibility, and resize behavior.

use super::types::{EntityColumn, EntitySort, EntityTablePreferences};
use crate::components::data_table::{MAX_COLUMN_WIDTH, effective_min_width, resized_width};
use std::ops::Range;

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
    let Some(compare) = column.comparator.as_ref() else {
        return indices;
    };

    indices.sort_by(|left, right| {
        let ordering = compare(&rows[*left], &rows[*right]);
        match sort {
            EntitySort::Descending { .. } => ordering.reverse(),
            EntitySort::Ascending { .. } | EntitySort::System => ordering,
        }
    });
    indices
}

/// Returns the total number of pages, treating a zero page size as one.
pub fn page_count(total_rows: usize, page_size: usize) -> usize {
    if total_rows == 0 {
        0
    } else {
        total_rows.div_ceil(page_size.max(1))
    }
}

/// Clamps a zero-based page index to the last available page.
pub fn clamp_page(current_page: usize, page_size: usize, total_rows: usize) -> usize {
    page_count(total_rows, page_size)
        .saturating_sub(1)
        .min(current_page)
}

/// Returns the source-index range for a page after clamping it to available rows.
pub fn page_bounds(current_page: usize, page_size: usize, total_rows: usize) -> Range<usize> {
    if total_rows == 0 {
        return 0..0;
    }
    let page_size = page_size.max(1);
    let page = clamp_page(current_page, page_size, total_rows);
    let start = page.saturating_mul(page_size);
    start..(start + page_size).min(total_rows)
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
    if preferences.hidden_columns.remove(column_id) {
        return true;
    }
    if column.required {
        return false;
    }
    let visible_count = columns
        .iter()
        .filter(|candidate| !preferences.hidden_columns.contains(candidate.id))
        .count();
    if visible_count <= 1 {
        return false;
    }
    preferences.hidden_columns.insert(column_id.to_owned());
    true
}

pub(crate) fn normalize_preferences<T>(
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
        && !columns
            .iter()
            .any(|column| column.id == column_id && column.sortable && column.comparator.is_some())
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
