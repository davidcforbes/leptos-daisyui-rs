use std::collections::HashMap;

/// Column definition for DataTable
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    /// Unique identifier for the column (HashMap key)
    pub id: &'static str,
    /// Display text for column header
    pub header: &'static str,
    /// Whether this column is sortable
    pub sortable: bool,
    /// Minimum width in pixels
    pub min_width: Option<u32>,
    /// Additional CSS classes for this column
    pub class: Option<&'static str>,
    /// Whether to truncate text with ellipsis
    pub truncate: bool,
    /// Maximum width in pixels (used with truncate)
    pub max_width: Option<u32>,
    /// Index into the cell_renderers vec (None = plain text)
    pub renderer_index: Option<usize>,
}

impl Column {
    /// Create a new sortable column
    pub fn new(id: &'static str, header: &'static str) -> Self {
        Self {
            id,
            header,
            sortable: true,
            min_width: None,
            class: None,
            truncate: false,
            max_width: None,
            renderer_index: None,
        }
    }

    /// Create a new non-sortable column
    pub fn new_non_sortable(id: &'static str, header: &'static str) -> Self {
        Self {
            id,
            header,
            sortable: false,
            min_width: None,
            class: None,
            truncate: false,
            max_width: None,
            renderer_index: None,
        }
    }

    /// Set minimum width
    pub fn with_min_width(mut self, width: u32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set CSS class
    pub fn with_class(mut self, class: &'static str) -> Self {
        self.class = Some(class);
        self
    }

    /// Enable text truncation with ellipsis
    pub fn with_truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    /// Set maximum width in pixels (used with truncate)
    pub fn with_max_width(mut self, width: u32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set the renderer index (indexes into a separate cell_renderers vec)
    pub fn with_renderer(mut self, index: usize) -> Self {
        self.renderer_index = Some(index);
        self
    }
}

/// Sort order enumeration
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SortOrder {
    /// Ascending order (A-Z, 0-9)
    #[default]
    Asc,
    /// Descending order (Z-A, 9-0)
    Desc,
}

impl SortOrder {
    /// Toggle between ascending and descending
    pub fn toggle(self) -> Self {
        match self {
            SortOrder::Asc => SortOrder::Desc,
            SortOrder::Desc => SortOrder::Asc,
        }
    }

    /// Get ARIA sort attribute value
    pub fn as_aria_str(&self) -> &'static str {
        match self {
            SortOrder::Asc => "ascending",
            SortOrder::Desc => "descending",
        }
    }

    /// Get sort indicator symbol
    pub fn as_symbol(&self) -> &'static str {
        match self {
            SortOrder::Asc => "↑",
            SortOrder::Desc => "↓",
        }
    }
}

/// CSS classes for DataTable styling
#[derive(Clone, Debug, PartialEq)]
pub struct DataTableClasses {
    /// Container wrapper class
    pub container: &'static str,
    /// Header cell class
    pub header_cell: &'static str,
    /// Body cell class
    pub body_cell: &'static str,
    /// Row class
    pub row: &'static str,
    /// Loading row class
    pub loading_row: &'static str,
    /// Empty row class
    pub empty_row: &'static str,
    /// Class applied to selected rows when multi-select is in use
    pub selected_row: &'static str,
    /// Pagination container class
    pub pagination: &'static str,
    /// Pagination button class
    pub pagination_button: &'static str,
    /// Page indicator class
    pub page_indicator: &'static str,
}

impl Default for DataTableClasses {
    fn default() -> Self {
        Self {
            container: "w-full",
            header_cell: "cursor-pointer select-none",
            body_cell: "",
            row: "",
            loading_row: "animate-pulse",
            empty_row: "text-center text-base-content/50",
            selected_row: "bg-base-200",
            pagination: "flex justify-between items-center mt-4",
            pagination_button: "btn btn-sm",
            page_indicator: "text-sm",
        }
    }
}

/// Customizable text strings for DataTable
#[derive(Clone, Debug, PartialEq)]
pub struct DataTableTexts {
    /// Loading state text
    pub loading: &'static str,
    /// Empty state text
    pub empty: &'static str,
    /// Previous button text
    pub previous: &'static str,
    /// Next button text
    pub next: &'static str,
    /// Page indicator format (use {current} and {total} placeholders)
    pub page_indicator: &'static str,
    /// Search input placeholder text
    pub search_placeholder: &'static str,
}

impl Default for DataTableTexts {
    fn default() -> Self {
        Self {
            loading: "Loading...",
            empty: "No data available",
            previous: "Previous",
            next: "Next",
            page_indicator: "Page {current} of {total}",
            search_placeholder: "Search...",
        }
    }
}

/// Type alias for table row data
pub type TableRow = HashMap<&'static str, String>;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Column::new ──

    #[test]
    fn column_new_sets_id_and_header() {
        let col = Column::new("email", "Email Address");
        assert_eq!(col.id, "email");
        assert_eq!(col.header, "Email Address");
    }

    #[test]
    fn column_new_is_sortable_by_default() {
        let col = Column::new("name", "Name");
        assert!(col.sortable);
    }

    #[test]
    fn column_new_has_no_optional_fields() {
        let col = Column::new("x", "X");
        assert_eq!(col.min_width, None);
        assert_eq!(col.max_width, None);
        assert_eq!(col.class, None);
        assert_eq!(col.renderer_index, None);
        assert!(!col.truncate);
    }

    // ── Column::new_non_sortable ──

    #[test]
    fn column_new_non_sortable_is_not_sortable() {
        let col = Column::new_non_sortable("actions", "Actions");
        assert!(!col.sortable);
    }

    #[test]
    fn column_new_non_sortable_sets_id_and_header() {
        let col = Column::new_non_sortable("actions", "Actions");
        assert_eq!(col.id, "actions");
        assert_eq!(col.header, "Actions");
    }

    #[test]
    fn column_new_non_sortable_has_no_optional_fields() {
        let col = Column::new_non_sortable("x", "X");
        assert_eq!(col.min_width, None);
        assert_eq!(col.max_width, None);
        assert_eq!(col.class, None);
        assert_eq!(col.renderer_index, None);
        assert!(!col.truncate);
    }

    // ── Builder methods ──

    #[test]
    fn with_truncate_sets_truncate_true() {
        let col = Column::new("title", "Title").with_truncate();
        assert!(col.truncate);
    }

    #[test]
    fn with_max_width_sets_max_width() {
        let col = Column::new("url", "URL").with_max_width(300);
        assert_eq!(col.max_width, Some(300));
    }

    #[test]
    fn with_min_width_sets_min_width() {
        let col = Column::new("date", "Date").with_min_width(120);
        assert_eq!(col.min_width, Some(120));
    }

    #[test]
    fn with_class_sets_class() {
        let col = Column::new("status", "Status").with_class("text-center font-bold");
        assert_eq!(col.class, Some("text-center font-bold"));
    }

    #[test]
    fn with_renderer_sets_renderer_index() {
        let col = Column::new("action", "Action").with_renderer(2);
        assert_eq!(col.renderer_index, Some(2));
    }

    #[test]
    fn builder_methods_chain() {
        let col = Column::new("url", "URL")
            .with_truncate()
            .with_max_width(400)
            .with_min_width(100)
            .with_class("monospace")
            .with_renderer(0);

        assert!(col.truncate);
        assert_eq!(col.max_width, Some(400));
        assert_eq!(col.min_width, Some(100));
        assert_eq!(col.class, Some("monospace"));
        assert_eq!(col.renderer_index, Some(0));
        // Original fields preserved
        assert_eq!(col.id, "url");
        assert_eq!(col.header, "URL");
        assert!(col.sortable);
    }

    #[test]
    fn builder_methods_chain_on_non_sortable() {
        let col = Column::new_non_sortable("actions", "")
            .with_min_width(80)
            .with_renderer(1);

        assert!(!col.sortable);
        assert_eq!(col.min_width, Some(80));
        assert_eq!(col.renderer_index, Some(1));
    }

    // ── SortOrder::toggle ──

    #[test]
    fn sort_order_toggle_asc_to_desc() {
        assert_eq!(SortOrder::Asc.toggle(), SortOrder::Desc);
    }

    #[test]
    fn sort_order_toggle_desc_to_asc() {
        assert_eq!(SortOrder::Desc.toggle(), SortOrder::Asc);
    }

    #[test]
    fn sort_order_toggle_roundtrip() {
        let order = SortOrder::Asc;
        assert_eq!(order.toggle().toggle(), SortOrder::Asc);
    }

    // ── SortOrder::as_aria_str ──

    #[test]
    fn sort_order_aria_str_asc() {
        assert_eq!(SortOrder::Asc.as_aria_str(), "ascending");
    }

    #[test]
    fn sort_order_aria_str_desc() {
        assert_eq!(SortOrder::Desc.as_aria_str(), "descending");
    }

    // ── SortOrder::as_symbol ──

    #[test]
    fn sort_order_symbol_asc() {
        assert_eq!(SortOrder::Asc.as_symbol(), "\u{2191}"); // ↑
    }

    #[test]
    fn sort_order_symbol_desc() {
        assert_eq!(SortOrder::Desc.as_symbol(), "\u{2193}"); // ↓
    }

    // ── SortOrder default ──

    #[test]
    fn sort_order_default_is_asc() {
        assert_eq!(SortOrder::default(), SortOrder::Asc);
    }

    // ── DataTableClasses::default ──

    #[test]
    fn data_table_classes_default_values() {
        let classes = DataTableClasses::default();
        assert_eq!(classes.container, "w-full");
        assert_eq!(classes.header_cell, "cursor-pointer select-none");
        assert_eq!(classes.body_cell, "");
        assert_eq!(classes.row, "");
        assert_eq!(classes.loading_row, "animate-pulse");
        assert_eq!(classes.empty_row, "text-center text-base-content/50");
        assert_eq!(classes.selected_row, "bg-base-200");
        assert_eq!(classes.pagination, "flex justify-between items-center mt-4");
        assert_eq!(classes.pagination_button, "btn btn-sm");
        assert_eq!(classes.page_indicator, "text-sm");
    }

    // ── DataTableTexts::default ──

    #[test]
    fn data_table_texts_default_values() {
        let texts = DataTableTexts::default();
        assert_eq!(texts.loading, "Loading...");
        assert_eq!(texts.empty, "No data available");
        assert_eq!(texts.previous, "Previous");
        assert_eq!(texts.next, "Next");
        assert_eq!(texts.page_indicator, "Page {current} of {total}");
        assert_eq!(texts.search_placeholder, "Search...");
    }

    // ── Column PartialEq ──

    #[test]
    fn columns_with_same_fields_are_equal() {
        let a = Column::new("id", "ID").with_min_width(50).with_truncate();
        let b = Column::new("id", "ID").with_min_width(50).with_truncate();
        assert_eq!(a, b);
    }

    #[test]
    fn columns_with_different_id_are_not_equal() {
        let a = Column::new("name", "Name");
        let b = Column::new("email", "Name");
        assert_ne!(a, b);
    }

    #[test]
    fn columns_with_different_sortable_are_not_equal() {
        let a = Column::new("x", "X");
        let b = Column::new_non_sortable("x", "X");
        assert_ne!(a, b);
    }

    #[test]
    fn columns_with_different_optional_fields_are_not_equal() {
        let a = Column::new("x", "X").with_max_width(100);
        let b = Column::new("x", "X").with_max_width(200);
        assert_ne!(a, b);
    }

    #[test]
    fn columns_with_different_renderer_are_not_equal() {
        let a = Column::new("x", "X").with_renderer(0);
        let b = Column::new("x", "X").with_renderer(1);
        assert_ne!(a, b);
    }

    #[test]
    fn columns_clone_equals_original() {
        let original = Column::new("col", "Col")
            .with_truncate()
            .with_max_width(250)
            .with_class("custom");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
