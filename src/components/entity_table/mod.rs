//! Generic, typed client-side table for complete dataset snapshots.

mod component;
mod date_filter;
mod emphasis;
mod focus_request;
mod grouping;
mod identity;
mod model;
mod multi_selection;
mod paging;
mod selection;
mod storage;
mod types;

pub use date_filter::{
    EntityDate, EntityDateBound, EntityDateFilter, EntityDateFilterCause, EntityDateFilterProposal,
    EntityDateFilterStatus, EntityDateParseError,
};
pub use emphasis::{EntityRowEmphasis, EntityRowEmphasisClassifier};
pub use focus_request::{
    EntityFocusRequest, EntityFocusRequestOutcome, EntityFocusRequestResolution,
    EntityFocusRequestTarget, entity_focus_request_outcome,
};
pub use grouping::{
    ENTITY_GROUP_COLUMN_ID, EntityGroupActions, EntityGroupCollapseCause,
    EntityGroupCollapseProposal, EntityGroupKey, EntityGroupOrder, EntityGroupTexts,
    EntityRowGroup, EntityRowGrouping, propose_entity_group_collapse,
};
pub use model::{
    ENTITY_PAGE_SIZE_CHOICES, EntityColumnMove, EntityFocusRecord, EntityFocusTarget,
    entity_table_display_projection, focus_target, move_column, next_sort, next_sort_additive,
    normalize_preferences, ordered_columns, page_after_dataset_change, page_after_row_delta,
    reset_columns, reset_sort, resolve_entity_page_size, set_preferred_width, sorted_indices,
    toggle_hidden_column, valid_page_size,
};
pub use multi_selection::{
    EntityTableDisplayedPage, EntityTableDisplayedPageSelection, EntityTableMultiSelection,
    EntityTableSelectionCause, EntityTableSelectionProposal, EntityTableSelectionTexts,
    displayed_page_selection_state, off_page_selected_count, propose_entity_displayed_page_toggle,
    propose_entity_row_toggle,
};
pub use paging::EntityPagePlan;
pub use selection::EntityTableSelection;
pub use storage::{decode_preferences, encode_preferences};
pub use types::{
    ENTITY_PAGE_SIZE_AUTO_VALUE, EntityBadgeCell, EntityBadgePresentation, EntityCellPresentation,
    EntityCellRenderer, EntityColumn, EntityColumnAlignment, EntityColumnChooserTrigger,
    EntityColumnFilter, EntityColumnFilterOption, EntityColumnFilterRenderer, EntityColumnFilters,
    EntityColumnKind, EntityColumns, EntityCompactRow, EntityComparator, EntityEmptyState,
    EntityIconCell, EntityIconColor, EntityIconPresentation, EntityNullOrder, EntityPageSize,
    EntityPageSizeIntent, EntityPreparedSortComparator, EntityPrimaryTextCell, EntityRowKey,
    EntityRowRenderer, EntitySecondaryTextCell, EntitySort, EntitySortColumn, EntitySortDirection,
    EntitySortKey, EntitySortKeyFactory, EntityTableActionColumnPolicy, EntityTableDisplayColumn,
    EntityTableDisplayProjection, EntityTableDisplayRow, EntityTablePreferenceOwnership,
    EntityTablePreferencePersistence, EntityTablePreferences, EntityTableProjectionScope,
    EntityTableTexts, EntityTableViewportFit, EntityTextOverflow,
};

#[cfg(test)]
pub(crate) use emphasis::{
    entity_row_emphasis_cell_class, entity_row_emphasis_for, entity_row_emphasis_row_class,
};
#[cfg(test)]
pub(crate) use types::{
    entity_alignment_class, entity_header_justify_class, entity_text_overflow_style,
    normalize_entity_secondary_text,
};

#[cfg(test)]
use model::SortedIndexCache;

#[cfg(test)]
pub(crate) use component::next_entity_page_size_id;

#[cfg(test)]
pub(crate) use grouping::{
    EntityGroupedOrder, entity_group_header_colspan, entity_group_label, entity_group_meta,
    entity_group_ranks, entity_grouped_order, entity_grouped_page_sections,
    entity_previous_group_key,
};

#[cfg(test)]
mod tests;
pub use component::{EntityRowAction, EntityTable};
