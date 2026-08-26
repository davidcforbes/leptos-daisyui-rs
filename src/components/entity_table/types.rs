//! Public types used to configure a typed entity table.

use leptos::prelude::AnyView;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

/// A callback that renders one typed cell from a borrowed row.
pub type EntityCellRenderer<T> = Rc<dyn Fn(&T) -> AnyView>;

/// A callback that renders the compact representation of a borrowed row.
pub type EntityRowRenderer<T> = Rc<dyn Fn(&T) -> AnyView>;

/// A callback that returns the stable identity of a borrowed row.
pub type EntityRowKey<T> = Rc<dyn Fn(&T) -> String>;

/// A callback that compares two borrowed rows for one column.
pub type EntityComparator<T> = Rc<dyn Fn(&T, &T) -> Ordering>;

/// A callback that extracts one normalized text key for local sorting.
///
/// The table evaluates this once per row when the dataset or sort changes,
/// avoiding string allocation inside the `O(n log n)` comparison loop.
pub type EntitySortKey<T> = Rc<dyn Fn(&T) -> String>;

/// The table's current client-side ordering.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntitySort {
    /// Preserve the order supplied by the dataset.
    #[default]
    System,
    /// Sort the named column from low to high.
    Ascending {
        /// Stable column identifier.
        column: String,
    },
    /// Sort the named column from high to low.
    Descending {
        /// Stable column identifier.
        column: String,
    },
}

impl EntitySort {
    /// Creates an ascending sort for a column.
    pub fn ascending(column: impl Into<String>) -> Self {
        Self::Ascending {
            column: column.into(),
        }
    }

    /// Creates a descending sort for a column.
    pub fn descending(column: impl Into<String>) -> Self {
        Self::Descending {
            column: column.into(),
        }
    }

    /// Returns the active column, or `None` for system order.
    pub fn column(&self) -> Option<&str> {
        match self {
            Self::System => None,
            Self::Ascending { column } | Self::Descending { column } => Some(column),
        }
    }

    /// Returns the WAI-ARIA sort value for the actively sorted header cell.
    ///
    /// Inactive and non-sortable headers omit `aria-sort`; assistive
    /// technology only needs the state on the one active sort column.
    pub fn aria_value_for(&self, column_id: &str) -> Option<&'static str> {
        match self {
            Self::Ascending { column } if column == column_id => Some("ascending"),
            Self::Descending { column } if column == column_id => Some("descending"),
            _ => None,
        }
    }

    /// Returns an accessible label describing the next sort state.
    pub fn next_label(&self, column_id: &str) -> &'static str {
        match self {
            Self::Ascending { column } if column == column_id => "Sort descending",
            Self::Descending { column } if column == column_id => "Restore system order",
            _ => "Sort ascending",
        }
    }
}

/// Column behavior and borrowed-row callbacks for [`EntityTable`](super::EntityTable).
pub struct EntityColumn<T> {
    /// Stable identifier used by sort and persisted preferences.
    pub id: &'static str,
    /// Visible column heading.
    pub header: String,
    /// Whether the header cycles table ordering.
    pub sortable: bool,
    /// Whether users are forbidden from hiding this column.
    pub required: bool,
    /// Whether this cell contains actions and therefore suppresses row activation.
    pub is_action: bool,
    /// Whether users may resize this column.
    pub resizable: bool,
    /// Optional column-specific minimum width in pixels.
    pub min_width: Option<u32>,
    /// Optional initial width in pixels.
    pub initial_width: Option<u32>,
    /// Plain text used for default rendering and accessible/exported content.
    pub text: Rc<dyn Fn(&T) -> String>,
    /// Optional rich renderer invoked with a borrowed typed row.
    pub renderer: Option<EntityCellRenderer<T>>,
    /// Typed comparator invoked with borrowed rows.
    pub comparator: Option<EntityComparator<T>>,
    /// Normalized text key extracted once per row by the default sorter.
    pub sort_key: Option<EntitySortKey<T>>,
}

impl<T> Clone for EntityColumn<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            header: self.header.clone(),
            sortable: self.sortable,
            required: self.required,
            is_action: self.is_action,
            resizable: self.resizable,
            min_width: self.min_width,
            initial_width: self.initial_width,
            text: Rc::clone(&self.text),
            renderer: self.renderer.as_ref().map(Rc::clone),
            comparator: self.comparator.as_ref().map(Rc::clone),
            sort_key: self.sort_key.as_ref().map(Rc::clone),
        }
    }
}

impl<T> fmt::Debug for EntityColumn<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EntityColumn")
            .field("id", &self.id)
            .field("header", &self.header)
            .field("sortable", &self.sortable)
            .field("required", &self.required)
            .field("is_action", &self.is_action)
            .field("resizable", &self.resizable)
            .field("min_width", &self.min_width)
            .field("initial_width", &self.initial_width)
            .finish_non_exhaustive()
    }
}

impl<T: 'static> EntityColumn<T> {
    /// Creates a sortable text column.
    pub fn new(
        id: &'static str,
        header: impl Into<String>,
        text: impl Fn(&T) -> String + 'static,
    ) -> Self {
        let text: Rc<dyn Fn(&T) -> String> = Rc::new(text);
        let comparator_text = Rc::clone(&text);
        Self {
            id,
            header: header.into(),
            sortable: true,
            required: false,
            is_action: false,
            resizable: true,
            min_width: None,
            initial_width: None,
            text,
            renderer: None,
            comparator: None,
            sort_key: Some(Rc::new(move |row| comparator_text(row).to_lowercase())),
        }
    }

    /// Creates a sortable text column; an explicit alias for [`Self::new`].
    pub fn text(
        id: &'static str,
        header: impl Into<String>,
        text: impl Fn(&T) -> String + 'static,
    ) -> Self {
        Self::new(id, header, text)
    }

    /// Creates a non-sortable action column.
    pub fn action(
        id: &'static str,
        header: impl Into<String>,
        text: impl Fn(&T) -> String + 'static,
    ) -> Self {
        let mut column = Self::new(id, header, text);
        column.sortable = false;
        column.is_action = true;
        column.comparator = None;
        column.sort_key = None;
        column
    }

    /// Makes this column mandatory in the visible-column set.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Uses a typed comparator for this column.
    pub fn sortable_by(mut self, compare: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        self.sortable = true;
        self.comparator = Some(Rc::new(compare));
        self.sort_key = None;
        self
    }

    /// Uses a rich cell renderer while retaining the text callback for accessibility.
    pub fn render_with(mut self, render: impl Fn(&T) -> AnyView + 'static) -> Self {
        self.renderer = Some(Rc::new(render));
        self
    }

    /// Sets this column's minimum width in pixels.
    pub fn with_min_width(mut self, width: u32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Sets this column's initial width in pixels.
    pub fn with_width(mut self, width: u32) -> Self {
        self.initial_width = Some(width);
        self
    }

    /// Prevents interactive width changes.
    pub fn non_resizable(mut self) -> Self {
        self.resizable = false;
        self
    }
}

/// Versioned user preferences persisted independently of a dataset snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTablePreferences {
    /// Consumer-defined schema version used to invalidate incompatible payloads.
    pub schema_version: u16,
    /// Number of rows rendered per page.
    pub page_size: usize,
    /// Current local ordering.
    pub sort: EntitySort,
    /// Optional columns hidden by the user.
    pub hidden_columns: BTreeSet<String>,
    /// User-adjusted widths keyed by stable column identifier.
    pub column_widths: BTreeMap<String, u32>,
}

impl EntityTablePreferences {
    /// Creates the opinionated defaults for a preference schema.
    pub fn new(schema_version: u16) -> Self {
        Self {
            schema_version,
            page_size: 25,
            sort: EntitySort::System,
            hidden_columns: BTreeSet::new(),
            column_widths: BTreeMap::new(),
        }
    }
}

/// Localizable labels used by [`EntityTable`](super::EntityTable).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityTableTexts {
    /// Label for the page-size control.
    pub rows_per_page: String,
    /// Accessible label for the column chooser.
    pub choose_columns: String,
    /// Previous-page action label.
    pub previous: String,
    /// Next-page action label.
    pub next: String,
    /// Row-range template with `{start}`, `{end}`, and `{total}` placeholders.
    pub row_range: String,
    /// Empty-table message used when the component is rendered directly.
    pub no_rows: String,
}

impl Default for EntityTableTexts {
    fn default() -> Self {
        Self {
            rows_per_page: "Rows per page".to_owned(),
            choose_columns: "Choose columns".to_owned(),
            previous: "Previous".to_owned(),
            next: "Next".to_owned(),
            row_range: "Showing {start}-{end} of {total}".to_owned(),
            no_rows: "No rows".to_owned(),
        }
    }
}
