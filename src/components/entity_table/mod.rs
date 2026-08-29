//! Generic, typed client-side table for complete dataset snapshots.

mod component;
mod model;
mod storage;
mod types;

pub use model::{
    ENTITY_PAGE_SIZE_CHOICES, EntityColumnMove, EntityFocusRecord, EntityFocusTarget,
    entity_table_display_projection, focus_target, move_column, next_sort, next_sort_additive,
    normalize_preferences, ordered_columns, page_after_dataset_change, page_after_row_delta,
    reset_columns, reset_sort, set_preferred_width, sorted_indices, toggle_hidden_column,
    valid_page_size,
};
pub use storage::{decode_preferences, encode_preferences};
pub use types::{
    EntityBadgeCell, EntityBadgePresentation, EntityCellPresentation, EntityCellRenderer,
    EntityColumn, EntityColumnAlignment, EntityColumnChooserTrigger, EntityColumnFilter,
    EntityColumnFilterOption, EntityColumnFilterRenderer, EntityColumnFilters, EntityColumns,
    EntityCompactRow, EntityComparator, EntityIconCell, EntityIconColor, EntityIconPresentation,
    EntityNullOrder, EntityPreparedSortComparator, EntityRowKey, EntityRowRenderer, EntitySort,
    EntitySortColumn, EntitySortDirection, EntitySortKey, EntitySortKeyFactory,
    EntityTableActionColumnPolicy, EntityTableDisplayColumn, EntityTableDisplayProjection,
    EntityTableDisplayRow, EntityTablePreferenceOwnership, EntityTablePreferencePersistence,
    EntityTablePreferences, EntityTableProjectionScope, EntityTableTexts, EntityTableViewportFit,
    EntityTextOverflow,
};

#[cfg(test)]
pub(crate) use types::{
    entity_alignment_class, entity_header_justify_class, entity_text_overflow_style,
};

#[cfg(test)]
use model::SortedIndexCache;

#[cfg(test)]
mod tests;
pub use component::{EntityRowAction, EntityTable};
