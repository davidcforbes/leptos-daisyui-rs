//! Public types used to configure a typed entity table.

use leptos::prelude::{AnyView, Callback, Signal};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

/// Direction of one clause in an [`EntitySort`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitySortDirection {
    /// Sort values from low to high.
    Ascending,
    /// Sort values from high to low.
    Descending,
}

impl EntitySortDirection {
    pub(crate) const fn aria_value(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }
}

/// One ordered column-and-direction clause in an [`EntitySort`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySortColumn {
    /// Stable column identifier.
    pub column: String,
    /// Direction applied after all preceding clauses compare equal.
    pub direction: EntitySortDirection,
}

impl EntitySortColumn {
    /// Creates an ascending clause for a column.
    pub fn ascending(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            direction: EntitySortDirection::Ascending,
        }
    }

    /// Creates a descending clause for a column.
    pub fn descending(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            direction: EntitySortDirection::Descending,
        }
    }
}

/// The table's ordered client-side sort clauses.
///
/// An empty clause list preserves the dataset's server-supplied system order.
/// Serialization always uses the canonical clause array. Deserialization also
/// accepts the historical `System`/`Ascending`/`Descending` enum payload so
/// legacy local-storage values migrate without a separate browser pass.
/// Historical single-column source patterns also remain available:
///
/// ```
/// use leptos_daisyui_rs::components::EntitySort;
///
/// let sort = EntitySort::ascending("status");
/// assert!(matches!(
///     sort,
///     EntitySort::Ascending { ref column } if column == "status"
/// ));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum EntitySort {
    /// Preserve the dataset's server-supplied order.
    #[default]
    System,
    /// Sort one column from low to high.
    Ascending {
        /// Stable column identifier.
        column: String,
    },
    /// Sort one column from high to low.
    Descending {
        /// Stable column identifier.
        column: String,
    },
    /// Apply two or more ordered sort clauses.
    Multiple {
        /// Clauses in primary-to-last priority order.
        clauses: Vec<EntitySortColumn>,
    },
}

impl Serialize for EntitySort {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.clauses().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EntitySort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WireSort {
            Canonical(Vec<EntitySortColumn>),
            Legacy(LegacySort),
        }

        #[derive(Deserialize)]
        enum LegacySort {
            System,
            Ascending { column: String },
            Descending { column: String },
        }

        Ok(match WireSort::deserialize(deserializer)? {
            WireSort::Canonical(clauses) => Self::multiple(clauses),
            WireSort::Legacy(LegacySort::System) => Self::System,
            WireSort::Legacy(LegacySort::Ascending { column }) => Self::ascending(column),
            WireSort::Legacy(LegacySort::Descending { column }) => Self::descending(column),
        })
    }
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

    /// Creates an ordered multi-column sort.
    pub fn multiple(clauses: impl IntoIterator<Item = EntitySortColumn>) -> Self {
        let mut clauses = clauses.into_iter();
        let Some(first) = clauses.next() else {
            return Self::System;
        };
        let Some(second) = clauses.next() else {
            return match first.direction {
                EntitySortDirection::Ascending => Self::ascending(first.column),
                EntitySortDirection::Descending => Self::descending(first.column),
            };
        };
        let mut multiple = vec![first, second];
        multiple.extend(clauses);
        Self::Multiple { clauses: multiple }
    }

    /// Returns the ordered clauses. An empty value means system order.
    pub fn clauses(&self) -> Vec<EntitySortColumn> {
        match self {
            Self::System => Vec::new(),
            Self::Ascending { column } => vec![EntitySortColumn::ascending(column.clone())],
            Self::Descending { column } => vec![EntitySortColumn::descending(column.clone())],
            Self::Multiple { clauses } => clauses.clone(),
        }
    }

    /// Returns whether the dataset remains in system order.
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System)
            || matches!(self, Self::Multiple { clauses } if clauses.is_empty())
    }

    /// Returns the primary clause, if the table is sorted.
    pub fn primary(&self) -> Option<EntitySortColumn> {
        match self {
            Self::System => None,
            Self::Ascending { column } => Some(EntitySortColumn::ascending(column.clone())),
            Self::Descending { column } => Some(EntitySortColumn::descending(column.clone())),
            Self::Multiple { clauses } => clauses.first().cloned(),
        }
    }

    /// Returns the clause for a column, if active.
    pub fn clause_for(&self, column_id: &str) -> Option<EntitySortColumn> {
        match self {
            Self::System => None,
            Self::Ascending { column } if column == column_id => {
                Some(EntitySortColumn::ascending(column.clone()))
            }
            Self::Descending { column } if column == column_id => {
                Some(EntitySortColumn::descending(column.clone()))
            }
            Self::Multiple { clauses } => clauses
                .iter()
                .find(|clause| clause.column == column_id)
                .cloned(),
            Self::Ascending { .. } | Self::Descending { .. } => None,
        }
    }

    /// Returns a column's one-based sort priority, if active.
    pub fn priority_for(&self, column_id: &str) -> Option<usize> {
        match self {
            Self::System => None,
            Self::Ascending { column } | Self::Descending { column } if column == column_id => {
                Some(1)
            }
            Self::Multiple { clauses } => clauses
                .iter()
                .position(|clause| clause.column == column_id)
                .map(|index| index + 1),
            Self::Ascending { .. } | Self::Descending { .. } => None,
        }
    }

    /// Returns a column's active direction, if any.
    pub fn direction_for(&self, column_id: &str) -> Option<EntitySortDirection> {
        self.clause_for(column_id).map(|clause| clause.direction)
    }

    /// Returns the primary column, or `None` for system order.
    pub fn column(&self) -> Option<&str> {
        match self {
            Self::System => None,
            Self::Ascending { column } | Self::Descending { column } => Some(column),
            Self::Multiple { clauses } => clauses.first().map(|clause| clause.column.as_str()),
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
            Self::Multiple { clauses } => clauses
                .first()
                .filter(|clause| clause.column == column_id)
                .map(|clause| clause.direction.aria_value()),
            Self::System | Self::Ascending { .. } | Self::Descending { .. } => None,
        }
    }

    /// Returns an accessible label describing the next sort state.
    pub fn next_label(&self, column_id: &str) -> &'static str {
        match self.direction_for(column_id) {
            Some(EntitySortDirection::Ascending) => "Sort descending",
            Some(EntitySortDirection::Descending) => "Restore system order",
            None => "Sort ascending",
        }
    }

    /// Describes the column's current direction and multi-sort priority.
    pub fn current_label(&self, column_id: &str) -> String {
        match (self.direction_for(column_id), self.priority_for(column_id)) {
            (Some(direction), Some(priority)) => format!(
                "Currently sorted {} at priority {priority} of {}",
                direction.aria_value(),
                self.clauses().len()
            ),
            _ => "Not currently sorted".to_owned(),
        }
    }

    /// Describes the result of activating a sort button without Shift.
    pub fn plain_action_label(&self, column_id: &str) -> &'static str {
        match self.direction_for(column_id) {
            Some(EntitySortDirection::Ascending) => "Activate to sort descending as the only sort",
            Some(EntitySortDirection::Descending) => "Activate to restore system order",
            None => "Activate to sort ascending as the only sort",
        }
    }

    /// Describes the result of Shift-activating a sort button.
    pub fn additive_action_label(&self, column_id: &str) -> String {
        match (self.direction_for(column_id), self.priority_for(column_id)) {
            (Some(EntitySortDirection::Ascending), Some(priority)) => {
                format!("Shift+activate to change priority {priority} to descending")
            }
            (Some(EntitySortDirection::Descending), Some(priority)) => {
                format!("Shift+activate to remove priority {priority}")
            }
            _ => format!(
                "Shift+activate to add ascending at priority {}",
                self.clauses().len() + 1
            ),
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
    /// Explicit display order of stable column identifiers.
    #[serde(default)]
    pub column_order: Vec<String>,
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
            column_order: Vec::new(),
            hidden_columns: BTreeSet::new(),
            column_widths: BTreeMap::new(),
        }
    }
}

/// Component-owned persistence used only by uncontrolled tables.
///
/// Controlled tables never carry this policy: their consumer owns both the
/// current value and any persistence performed after a change callback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityTablePreferencePersistence {
    /// Keep preferences in memory for this component instance only.
    #[default]
    Disabled,
    /// Preserve the historical automatic `localStorage` behavior.
    LegacyLocalStorage {
        /// Namespace appended to the framework's EntityTable storage prefix.
        storage_key: &'static str,
    },
}

/// Declares who owns an [`EntityTablePreferences`] value.
///
/// Controlled ownership is persistence-neutral: each UI operation emits one
/// normalized full replacement and the consumer decides whether or where to
/// store it. Uncontrolled ownership retains the component's compatibility
/// behavior and may opt into the legacy browser-storage mechanism.
#[derive(Clone)]
pub enum EntityTablePreferenceOwnership {
    /// The consumer supplies the current value and receives replacements.
    Controlled {
        /// Reactive current preferences supplied by the consumer.
        current: Signal<EntityTablePreferences>,
        /// Receives one normalized full replacement per UI preference action.
        on_change: Callback<EntityTablePreferences>,
    },
    /// The component owns its in-memory preference signal.
    Uncontrolled {
        /// Optional component-managed persistence.
        persistence: EntityTablePreferencePersistence,
    },
}

impl EntityTablePreferenceOwnership {
    /// Creates consumer-controlled, persistence-neutral ownership.
    pub fn controlled(
        current: Signal<EntityTablePreferences>,
        on_change: Callback<EntityTablePreferences>,
    ) -> Self {
        Self::Controlled { current, on_change }
    }

    /// Creates component-owned preferences with the selected persistence.
    pub fn uncontrolled(persistence: EntityTablePreferencePersistence) -> Self {
        Self::Uncontrolled { persistence }
    }
}

impl fmt::Debug for EntityTablePreferenceOwnership {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Controlled { .. } => formatter.debug_struct("Controlled").finish_non_exhaustive(),
            Self::Uncontrolled { persistence } => formatter
                .debug_struct("Uncontrolled")
                .field("persistence", persistence)
                .finish(),
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
    /// Action label that restores server-supplied ordering.
    pub reset_sort: String,
    /// Action label that restores default column visibility, widths, and order.
    pub reset_columns: String,
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
            reset_sort: "Reset sort".to_owned(),
            reset_columns: "Reset columns".to_owned(),
            previous: "Previous".to_owned(),
            next: "Next".to_owned(),
            row_range: "Showing {start}-{end} of {total}".to_owned(),
            no_rows: "No rows".to_owned(),
        }
    }
}
