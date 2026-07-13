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

mod body;
mod clipboard;
mod component;
mod controls;
mod header;
mod pagination;
mod resize;
mod selection;
mod server_component;

/// Typed column sorting: `SortAs` plus the cell parsers/comparator behind it
pub mod sort;
/// Types for DataTable component including Column, SortOrder, and configuration structs
pub mod types;

pub use clipboard::{cell_text, row_text, row_with_headers_text};
pub use component::*;
pub use selection::handle_row_click;
pub use server_component::*;
pub use sort::{SortAs, column_sort_as, compare_cells, parse_date, parse_number};
pub use types::*;
