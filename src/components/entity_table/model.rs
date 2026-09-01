//! Pure ordering, pagination, visibility, and resize behavior.

use super::grouping::ENTITY_GROUP_COLUMN_ID;
use super::types::{
    EntityColumn, EntityComparator, EntityPageSize, EntityPageSizeIntent,
    EntityPreparedSortComparator, EntitySort, EntitySortDirection, EntityTableActionColumnPolicy,
    EntityTableDisplayColumn, EntityTableDisplayProjection, EntityTableDisplayRow,
    EntityTablePreferences,
};
use crate::components::data_table::{
    ColumnVisibilityAction, MAX_COLUMN_WIDTH, clamp_page, column_visibility_action,
    effective_min_width, page_bounds, resized_width,
};
use std::collections::BTreeSet;
use std::ops::Range;
use std::rc::Rc;

/// Supported row counts for client-snapshot tables.
pub const ENTITY_PAGE_SIZE_CHOICES: [usize; 3] = [25, 50, 100];

/// Returns whether a row count is one of the opinionated choices.
pub fn valid_page_size(page_size: usize) -> bool {
    ENTITY_PAGE_SIZE_CHOICES.contains(&page_size)
}

/// Resolves the one [`EntityPageSize`] a render is allowed to use.
///
/// This is the single place a rows-per-page intent and a measured viewport
/// capacity are combined, which is what keeps the rendered body, the result
/// summary, the rows-per-page control, and the pager from describing different
/// page sizes (ldui-5p06). It is total over its four inputs:
///
/// - auto unavailable (no `viewport_fit` policy) resolves fixed, whatever the
///   stored intent says, so a preference restored from a table that did fit
///   the viewport cannot label a non-fitting table `Auto`;
/// - `Auto` with a measurement is that measurement;
/// - `Auto` before the first measurement lands is the configured size, still
///   labeled `Auto`, because that is genuinely what the body renders on the
///   first paint;
/// - `Fixed` ignores the measurement entirely — an explicit `25` renders up to
///   25 rows and the table region scrolls.
///
/// ```
/// use leptos_daisyui_rs::components::{
///     EntityPageSizeIntent, resolve_entity_page_size,
/// };
///
/// let fitted = resolve_entity_page_size(EntityPageSizeIntent::Auto, true, 25, Some(5));
/// assert_eq!(fitted.rows(), 5);
/// assert!(fitted.is_auto());
///
/// let explicit = resolve_entity_page_size(EntityPageSizeIntent::Fixed, true, 25, Some(5));
/// assert_eq!(explicit.rows(), 25);
/// assert!(!explicit.is_auto());
/// ```
pub fn resolve_entity_page_size(
    intent: EntityPageSizeIntent,
    auto_available: bool,
    configured_rows: usize,
    measured_rows: Option<usize>,
) -> EntityPageSize {
    match (auto_available, intent) {
        (true, EntityPageSizeIntent::Auto) => {
            EntityPageSize::auto(measured_rows.unwrap_or(configured_rows))
        }
        _ => EntityPageSize::fixed(configured_rows),
    }
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
                    compare: sort_key.prepare(rows, clause.direction),
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

/// Builds one atomic display/export projection from the table's canonical
/// ordering, visibility, paging, row-key, and cell-text rules.
pub fn entity_table_display_projection<T>(
    rows: &[T],
    columns: &[EntityColumn<T>],
    preferences: &EntityTablePreferences,
    current_page: usize,
    effective_page_size: usize,
    row_key: &dyn Fn(&T) -> String,
    action_columns: EntityTableActionColumnPolicy,
) -> EntityTableDisplayProjection {
    let indices = sorted_indices(rows, columns, &preferences.sort);
    let bounds = page_bounds(current_page, effective_page_size.max(1), indices.len());
    entity_table_display_projection_from_indices(
        rows,
        columns,
        preferences,
        &indices,
        bounds,
        row_key,
        action_columns,
        None,
    )
}

/// Group identity carried into a display/export projection (`ldui-iyfa`).
///
/// The visual table stops repeating the group label in every row, so the
/// projection has to put it back: a leading synthetic column carries the
/// label, and [`EntityTableDisplayRow::group_key`] carries the stable
/// identity. Both are absent on an ungrouped table.
pub(crate) struct EntityProjectionGrouping<'a> {
    /// Group key of each entry in `indices`, parallel to it.
    pub group_keys: &'a [String],
    /// Resolves a group key to its display label.
    pub label_of: &'a dyn Fn(&str) -> String,
    /// Localized header of the synthetic group column.
    pub column_header: &'a str,
}

// Keeping the already-derived index window explicit makes this pure helper
// directly testable without coupling it to component state.
#[allow(clippy::too_many_arguments)]
pub(crate) fn entity_table_display_projection_from_indices<T>(
    rows: &[T],
    columns: &[EntityColumn<T>],
    preferences: &EntityTablePreferences,
    indices: &[usize],
    // The window the body is ACTUALLY painting, handed in rather than
    // recomputed: a grouped table's pages deliberately stop short of capacity
    // (ldui-5in5), so `page * capacity` no longer names the same rows and an
    // export that recomputed it would describe a different page than the
    // screen.
    page_bounds: Range<usize>,
    row_key: &dyn Fn(&T) -> String,
    action_columns: EntityTableActionColumnPolicy,
    grouping: Option<EntityProjectionGrouping<'_>>,
) -> EntityTableDisplayProjection {
    let projected_columns = ordered_columns(preferences, columns)
        .into_iter()
        .filter(|column| !preferences.hidden_columns.contains(column.id))
        .filter(|column| {
            !column.is_action || action_columns == EntityTableActionColumnPolicy::Include
        })
        .collect::<Vec<_>>();
    // Prepended rather than appended so the group identity reads as the
    // outermost fact of the record, the same way the heading reads above the
    // rows it heads.
    let descriptors = grouping
        .as_ref()
        .map(|grouping| {
            EntityTableDisplayColumn::new(ENTITY_GROUP_COLUMN_ID, grouping.column_header, false)
        })
        .into_iter()
        .chain(projected_columns.iter().map(|column| {
            EntityTableDisplayColumn::new(column.id, &column.header, column.is_action)
        }))
        .collect::<Vec<_>>();
    let projected_rows = indices
        .iter()
        .enumerate()
        .map(|(position, index)| {
            let row = &rows[*index];
            let group_key = grouping
                .as_ref()
                .and_then(|grouping| grouping.group_keys.get(position))
                .cloned();
            let group_cell = grouping
                .as_ref()
                .zip(group_key.as_ref())
                .map(|(grouping, key)| (grouping.label_of)(key));
            let projected = EntityTableDisplayRow::new(
                row_key(row),
                group_cell
                    .into_iter()
                    .chain(projected_columns.iter().map(|column| (column.text)(row))),
            );
            match group_key {
                Some(key) => projected.with_group_key(key),
                None => projected,
            }
        })
        .collect::<Vec<_>>();
    let start = page_bounds.start.min(projected_rows.len());
    let end = page_bounds.end.clamp(start, projected_rows.len());
    EntityTableDisplayProjection::from_parts(descriptors, projected_rows, start, end)
}

enum PreparedSort<T> {
    Keys {
        compare: EntityPreparedSortComparator,
    },
    Comparator {
        direction: EntitySortDirection,
        compare: EntityComparator<T>,
    },
}

impl<T> PreparedSort<T> {
    fn compare(&self, left: usize, right: usize, rows: &[T]) -> std::cmp::Ordering {
        match self {
            Self::Keys { compare } => compare(left, right),
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
    semantic_generation: u64,
    indices: Rc<Vec<usize>>,
}

impl<T> SortedIndexCache<T> {
    pub(crate) fn new() -> Self {
        Self {
            rows: None,
            sort: EntitySort::System,
            semantic_generation: 0,
            indices: Rc::new(Vec::new()),
        }
    }

    pub(crate) fn indices(
        &mut self,
        rows: Rc<Vec<T>>,
        columns: &[EntityColumn<T>],
        sort: &EntitySort,
        semantic_generation: u64,
    ) -> Rc<Vec<usize>> {
        let unchanged = self
            .rows
            .as_ref()
            .is_some_and(|cached| Rc::ptr_eq(cached, &rows))
            && self.sort == *sort
            && self.semantic_generation == semantic_generation;
        if !unchanged {
            self.indices = Rc::new(sorted_indices(rows.as_slice(), columns, sort));
            self.rows = Some(rows);
            self.sort = sort.clone();
            self.semantic_generation = semantic_generation;
        }
        Rc::clone(&self.indices)
    }
}

/// Focus identity captured when a marked row action receives focus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityFocusRecord {
    /// Opaque dataset/access generation supplied by the page.
    pub scope: String,
    /// Stable source-row identity.
    pub row_key: String,
    /// Stable action identity within that row.
    pub action_id: String,
    /// Zero-based position in the actual rendered page at focus time.
    pub visible_position: usize,
}

/// Framework-owned focus decision after source or rendered rows change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityFocusTarget {
    /// The focused row/action still exists and native focus should remain.
    NoChange,
    /// Focus the same action on the selected neighboring rendered row.
    RowAction {
        /// Stable neighboring row identity.
        row_key: String,
        /// Stable action identity copied from the prior record.
        action_id: String,
    },
    /// Focus the named table region instead of an unrelated row.
    TableRegion,
    /// Drop recovery state because scope changed or the user moved focus.
    Clear,
}

/// Selects a deterministic post-change focus target from authoritative source
/// membership and the table's actual filtered/sorted/paged order.
pub fn focus_target(
    record: &EntityFocusRecord,
    current_source_keys: &[String],
    current_visible_keys: &[String],
    current_scope: &str,
    user_moved_focus: bool,
    candidate_action_eligible: bool,
) -> EntityFocusTarget {
    if user_moved_focus || record.scope != current_scope {
        return EntityFocusTarget::Clear;
    }

    if current_source_keys.contains(&record.row_key) {
        return if current_visible_keys.contains(&record.row_key) {
            EntityFocusTarget::NoChange
        } else {
            EntityFocusTarget::TableRegion
        };
    }

    let Some(row_key) = current_visible_keys.get(
        record
            .visible_position
            .min(current_visible_keys.len().saturating_sub(1)),
    ) else {
        return EntityFocusTarget::TableRegion;
    };
    if !candidate_action_eligible {
        return EntityFocusTarget::TableRegion;
    }
    EntityFocusTarget::RowAction {
        row_key: row_key.clone(),
        action_id: record.action_id.clone(),
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
