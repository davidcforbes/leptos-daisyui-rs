//! # DataTable Component
//!
//! Production-ready data table with sorting, pagination, loading states,
//! and efficient handling of large datasets.
//!
//! ## Features
//! - Column-based sorting through native pointer/Enter/Space header controls
//! - Pagination with customizable page size
//! - Loading and empty states
//! - Fully themed with daisyUI
//! - Efficient index-based operations for 10,000+ rows
//! - One-to-one column filters in a second, aligned header row
//! - Stable declared column tracks: sorting never derives geometry from body rows
//! - Semantic dark-blue headers, light-blue filters, and faint full-cell grids
//! - Zebra striping as an explicit opt-in rather than the opinionated default
//!
//! Sort controls reserve a fixed indicator slot and header/filter nodes are
//! keyed independently of sort state. A non-resizable column, when present,
//! absorbs otherwise-unused full-width space so resizable tracks retain their
//! exact pixel and accessibility values. Narrow tables scroll horizontally
//! without moving their header/filter alignment or scroll origin during sort.
//!
//! ## Example
//! ```rust,no_run
//! use std::collections::HashMap;
//! use leptos::prelude::*;
//! use leptos_daisyui_rs::components::*;
//!
//! #[component]
//! fn MyTable() -> impl IntoView {
//!     let columns = vec![
//!         Column::new("name", "Name"),
//!         Column::new("email", "Email"),
//!         Column::new_non_sortable("status", "Status"),
//!     ];
//!
//!     let data = vec![
//!         HashMap::from([
//!             ("name", "Alice".to_string()),
//!             ("email", "alice@example.com".to_string()),
//!             ("status", "Active".to_string()),
//!         ]),
//!     ];
//!
//!     view! {
//!         <DataTable
//!             columns=Signal::derive(move || columns.clone())
//!             data=Signal::derive(move || data.clone())
//!             page_size=10
//!         />
//!     }
//! }
//! ```

mod auto_page;
mod body;
mod chooser;
mod clipboard;
mod component;
mod controls;
mod filter;
mod geometry;
mod header;
mod identity;
mod pagination;
mod resize;
mod selection;
mod server_column_tools;
mod server_component;
mod server_selection;

pub(super) use identity::{
    column_tools_control_id, filter_control_id, next_data_table_control_id, page_size_control_id,
    resolve_control_id, search_control_id, selection_header_control_id, selection_row_control_id,
};

/// Every DataTable variant keeps wide columns reachable instead of clipping
/// the right-hand side of the table at constrained viewport widths.
pub(super) const TABLE_SCROLL_WRAPPER_CLASS: &str = crate::components::table::TABLE_VIEWPORT_CLASS;

/// Typed column sorting: `SortAs` plus the cell parsers/comparator behind it
pub mod sort;
/// Types for DataTable component including Column, SortOrder, and configuration structs
pub mod types;

pub use auto_page::{
    DEFAULT_AUTO_MIN_ROWS, FALLBACK_HEADER_HEIGHT, FALLBACK_ROW_HEIGHT, RowHeightEra,
    RowHeightEraKey, auto_page_size_for_height, max_row_height, overflow_check_floor,
    rows_per_page_for_height,
};
pub use chooser::DataTableColumnChooser;
pub(crate) use chooser::{ColumnVisibilityAction, column_visibility_action};
pub use clipboard::{cell_text, row_text, row_with_headers_text};
pub use component::*;
pub use filter::{
    COLUMN_TEXT_FILTER_DEBOUNCE_MS, ColumnFilters, DataTableFilterOption,
    DataTableFilterOptionError, DataTableFilterOptions, DataTableFilterRow, FILTER_ALL,
    distinct_values, filter_options_from_strings, has_exact_filterable_columns,
    has_filterable_columns, prune_stale_column_filters, prune_stale_filters,
    row_matches_column_filters, row_matches_filters, row_matches_search, validate_filter_options,
};
pub(crate) use geometry::{StableColumnTrack, StableTableColGroup, stable_table_content_style};
pub(crate) use pagination::{PageSlot, page_window, row_range};
pub use pagination::{clamp_page, page_bounds, page_count};
pub(crate) use resize::{
    MAX_COLUMN_WIDTH, effective_min_width, keyboard_resized_width, resized_width,
};
pub use selection::{
    RowClickKind, click_swallowed_by_inspect, handle_row_click, key_inspects, row_click_kind,
    row_is_interactive,
};
pub use server_column_tools::{
    ServerTableColumnTools, ServerTableColumnToolsTexts, ServerTableDisplayColumn,
    ServerTableDisplayRow, ServerTableDisplayedSlice,
};
pub use server_component::*;
pub use server_selection::{
    ServerTableMultiSelection, ServerTableRowSelectability, ServerTableSelectionCause,
    ServerTableSelectionProposal, ServerTableSelectionTexts, ServerTableSliceSelectionState,
    off_slice_selected_count, propose_row_toggle, propose_slice_toggle, slice_selection_state,
};
pub use sort::{SortAs, column_sort_as, compare_cells, parse_date, parse_number};
pub use types::*;

#[cfg(test)]
mod responsive_contract {
    use super::TABLE_SCROLL_WRAPPER_CLASS;

    #[test]
    fn both_data_table_variants_use_the_horizontal_scroll_wrapper() {
        assert!(
            TABLE_SCROLL_WRAPPER_CLASS
                .split_ascii_whitespace()
                .any(|class| class == "overflow-x-auto")
        );

        // ONE contract, not two that happen to match today: DataTable's
        // wrapper is an alias of the public TableViewport class, so a bare
        // Table in a TableViewport and a DataTable scroll identically.
        assert_eq!(
            TABLE_SCROLL_WRAPPER_CLASS,
            crate::components::table::TABLE_VIEWPORT_CLASS,
        );

        for source in [
            include_str!("component.rs"),
            include_str!("server_component.rs"),
        ] {
            assert!(
                source.contains("class=TABLE_SCROLL_WRAPPER_CLASS"),
                "a DataTable variant stopped applying the shared horizontal-scroll contract"
            );
        }
    }
}
