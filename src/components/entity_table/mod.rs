//! Generic, typed client-side table for complete dataset snapshots.

mod component;
mod model;
mod storage;
mod types;

pub use model::{
    ENTITY_PAGE_SIZE_CHOICES, next_sort, normalize_preferences, page_after_dataset_change,
    page_after_row_delta, reset_columns, reset_sort, set_preferred_width, sorted_indices,
    toggle_hidden_column, valid_page_size,
};
pub use storage::{decode_preferences, encode_preferences};
pub use types::{
    EntityCellRenderer, EntityColumn, EntityComparator, EntityRowKey, EntityRowRenderer,
    EntitySort, EntitySortKey, EntityTablePreferenceOwnership, EntityTablePreferencePersistence,
    EntityTablePreferences, EntityTableTexts,
};

#[cfg(test)]
use model::SortedIndexCache;

#[cfg(test)]
mod tests;
pub use component::EntityTable;
