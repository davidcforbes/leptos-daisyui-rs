//! # DataTable Component
//!
//! Production-ready data table with sorting, pagination, loading states,
//! and efficient handling of large datasets.
//!
//! ## Features
//! - Column-based sorting (click headers to toggle Asc/Desc)
//! - Pagination with customizable page size
//! - Loading and empty states
//! - Fully themed with daisyUI
//! - Efficient index-based operations for 10,000+ rows
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
mod header;
mod pagination;
mod resize;
mod selection;
mod server_component;

/// Every DataTable variant keeps wide columns reachable instead of clipping
/// the right-hand side of the table at constrained viewport widths.
pub(super) const TABLE_SCROLL_WRAPPER_CLASS: &str = crate::components::table::TABLE_VIEWPORT_CLASS;

/// Typed column sorting: `SortAs` plus the cell parsers/comparator behind it
pub mod sort;
/// Types for DataTable component including Column, SortOrder, and configuration structs
pub mod types;

pub use auto_page::{FALLBACK_HEADER_HEIGHT, FALLBACK_ROW_HEIGHT, rows_per_page_for_height};
pub use chooser::DataTableColumnChooser;
pub use clipboard::{cell_text, row_text, row_with_headers_text};
pub use component::*;
pub use filter::{
    ColumnFilters, DataTableFilterRow, FILTER_ALL, distinct_values, has_filterable_columns,
    prune_stale_filters, row_matches_filters, row_matches_search,
};
pub use selection::{
    RowClickKind, click_swallowed_by_inspect, handle_row_click, key_inspects, row_click_kind,
    row_is_interactive,
};
pub use server_component::*;
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
