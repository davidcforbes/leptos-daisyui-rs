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
