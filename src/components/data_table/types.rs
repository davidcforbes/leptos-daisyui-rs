use crate::components::badge::BadgeColor;
use crate::components::data_table::sort::SortAs;
use leptos::prelude::{AnyView, Callback};
use std::collections::HashMap;

/// Per-cell renderer.
///
/// Invoked with `(absolute_row_index, row_data)` and returns a type-erased
/// view. Stored in `DataTable`'s `cell_renderers` prop; columns opt in by
/// setting `renderer_index = Some(i)` (typically via `Column::with_renderer(i)`).
/// Falls back to plain text rendering when the index is `None` or out of bounds.
pub type CellRenderer = Callback<(usize, TableRow), AnyView>;

/// Optional full-width detail content rendered immediately after one row.
///
/// Returning `None` keeps the row single-height. Returning a view adds a
/// sibling detail `<tr>` spanning the currently rendered columns; the pair
/// moves together through sorting, filtering, and paging because both are
/// derived from the same absolute row identity.
pub type RowDetailRenderer = Callback<(usize, TableRow), Option<AnyView>>;

/// Lightweight built-in cell content, rendered via the crate's own `Badge`/
/// `Icon` components without requiring a full custom [`CellRenderer`].
///
/// Additive alongside `cell_renderers` -- a column's `renderer_index` (when
/// set) always takes precedence over its `typed_cell_index`, and columns
/// using neither are unaffected (they keep rendering `row[col.id]` as plain
/// text exactly as before).
#[derive(Clone, Debug, PartialEq)]
pub enum TypedCell {
    /// Plain text, rendered identically to the default (no typed cell) path.
    Text(String),
    /// A `Badge` pill with the given text and daisyUI semantic color.
    Badge {
        /// Badge label text.
        text: String,
        /// daisyUI semantic color for the badge.
        color: BadgeColor,
    },
    /// A Lucide `Icon` by name, with an optional color utility class (e.g. `"text-error"`).
    Icon {
        /// Lucide icon name (e.g. `"check"`, `"x"`, `"alert-triangle"`).
        name: String,
        /// Color utility class, or `""` for the default icon color.
        color: String,
    },
}

impl TypedCell {
    /// Text representation for clipboard export / accessibility purposes.
    /// `Icon` cells have no text and return `""`.
    pub fn as_text(&self) -> &str {
        match self {
            TypedCell::Text(s) => s,
            TypedCell::Badge { text, .. } => text,
            TypedCell::Icon { .. } => "",
        }
    }
}

/// Per-column typed-cell resolver.
///
/// Invoked with `(absolute_row_index, row_data)` and returns a [`TypedCell`]
/// describing what to render. Stored in `DataTable`'s `typed_cells` prop;
/// columns opt in by setting `typed_cell_index = Some(i)` (typically via
/// `Column::with_typed_cell(i)`).
pub type TypedCellFn = Callback<(usize, TableRow), TypedCell>;

/// Matching behavior for one enabled column filter.
///
/// Filter values continue to travel in the source-compatible
/// [`ColumnFilters`](super::ColumnFilters) map. Consumers of
/// [`TableQuery`](super::TableQuery) use the matching [`Column`] definition to
/// distinguish exact dropdown values from substring text values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColumnFilterKind {
    /// Case-sensitive equality, rendered as a finite option dropdown.
    #[default]
    Exact,
    /// Case-insensitive substring matching, rendered as a debounced text box.
    Contains,
}

/// Column definition for DataTable
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    /// Unique identifier for the column (HashMap key)
    pub id: &'static str,
    /// Display text for column header. Owned so it can come from a runtime
    /// localization lookup (`t()`); rebuild the `columns` vec (typically in a
    /// `Memo` reading the active locale) and every header re-renders.
    pub header: String,
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
    /// Whether this column's width can be adjusted by dragging its header
    /// divider (default: `true`). Set to `false` via
    /// [`Column::non_resizable`] to keep a column's width fixed.
    pub resizable: bool,
    /// Index into the `typed_cells` vec for lightweight built-in Badge/Icon
    /// rendering (`None` = no typed cell). Checked only when `renderer_index`
    /// is `None` or out of bounds -- `renderer_index` always wins.
    pub typed_cell_index: Option<usize>,
    /// How this column's cells are compared when sorting (default:
    /// [`SortAs::Text`], the plain lexicographic comparison). Set
    /// [`SortAs::Number`] on money/duration/percentage columns, whose display
    /// strings otherwise sort by first digit (`"$1,000" < "$900"`).
    pub sort_as: SortAs,
    /// Whether this column gets a dropdown in the filter row (default:
    /// `false`). Opt in with [`Column::filterable`]. When no column opts in,
    /// no filter row is rendered at all.
    pub filterable: bool,
    /// Matching and control kind used when [`Column::filterable`] is `true`.
    /// Existing exact dropdown columns retain [`ColumnFilterKind::Exact`];
    /// [`Column::filterable_text`] selects [`ColumnFilterKind::Contains`].
    pub filter_kind: ColumnFilterKind,
    /// Whether this column holds row actions (buttons/links rendered via a
    /// cell renderer). Opt in with [`Column::action`]. Events inside an
    /// action cell stay in the cell: a click or Enter/Space there never
    /// reaches the row's activate/select handling, so cell renderers don't
    /// need per-app `stop_propagation` wrappers.
    pub is_action: bool,
    /// Whether the free-text `searchable` box matches this column's cell
    /// values (default: `true`). Opt out with [`Column::searched`]. Note the
    /// search contract is column-scoped either way: a `TableRow` entry with
    /// no declared column (renderer-only metadata such as state codes, route
    /// ids or epoch instants) is never searched at all.
    pub searched: bool,
    /// Whether an opt-in column-chooser (e.g. `ServerDataTable`'s
    /// `column_tools`) is forbidden from hiding this column (default:
    /// `false`). Mirrors `EntityColumn::required`. A column left `false`
    /// stays visible by default but may be hidden by the user; a column with
    /// `true` can never be hidden through the chooser regardless of
    /// preference payload contents.
    pub required: bool,
}

impl Column {
    /// Create a new sortable column.
    ///
    /// Sorts as text. If the column holds formatted numbers (money, percentages,
    /// day counts), add [`with_sort_as(SortAs::Number)`](Column::with_sort_as) --
    /// text order puts `"$1,000"` before `"$900"`.
    pub fn new(id: &'static str, header: impl Into<String>) -> Self {
        Self {
            id,
            header: header.into(),
            sortable: true,
            min_width: None,
            class: None,
            truncate: false,
            max_width: None,
            renderer_index: None,
            resizable: true,
            typed_cell_index: None,
            sort_as: SortAs::Text,
            filterable: false,
            filter_kind: ColumnFilterKind::Exact,
            is_action: false,
            searched: true,
            required: false,
        }
    }

    /// Create a new non-sortable column
    pub fn new_non_sortable(id: &'static str, header: impl Into<String>) -> Self {
        Self {
            id,
            header: header.into(),
            sortable: false,
            min_width: None,
            class: None,
            truncate: false,
            max_width: None,
            renderer_index: None,
            resizable: true,
            typed_cell_index: None,
            sort_as: SortAs::Text,
            filterable: false,
            filter_kind: ColumnFilterKind::Exact,
            is_action: false,
            searched: true,
            required: false,
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

    /// Disable interactive column-width resizing for this column (columns
    /// are resizable by default).
    pub fn non_resizable(mut self) -> Self {
        self.resizable = false;
        self
    }

    /// Set the typed-cell index (indexes into a separate `typed_cells` vec)
    /// for lightweight Badge/Icon rendering without a full custom
    /// [`CellRenderer`].
    pub fn with_typed_cell(mut self, index: usize) -> Self {
        self.typed_cell_index = Some(index);
        self
    }

    /// Declare how this column's cells are compared when sorting.
    ///
    /// ```
    /// use leptos_daisyui_rs::components::{Column, SortAs};
    ///
    /// let balance = Column::new("balance", "Balance").with_sort_as(SortAs::Number);
    /// let opened = Column::new("opened", "Opened").with_sort_as(SortAs::Date);
    /// ```
    pub fn with_sort_as(mut self, sort_as: SortAs) -> Self {
        self.sort_as = sort_as;
        self
    }

    /// Give this column a dropdown in `DataTable`'s filter row, offering the
    /// column's distinct values. Selecting one narrows the table to rows whose
    /// cell equals it exactly.
    ///
    /// Filtering is opt-in: a table with no `filterable` column renders no
    /// filter row. Active filters combine with each other (AND) and with the
    /// `searchable` free-text box.
    ///
    /// Best on low-cardinality columns (status, owner, type) -- a dropdown of a
    /// thousand distinct ids is not a usable filter.
    ///
    /// ```
    /// use leptos_daisyui_rs::components::Column;
    ///
    /// let status = Column::new("status", "Status").filterable();
    /// ```
    pub fn filterable(mut self) -> Self {
        self.filterable = true;
        self.filter_kind = ColumnFilterKind::Exact;
        self
    }

    /// Give this high-cardinality column a debounced text box in the aligned
    /// filter row. The active value matches a case-insensitive substring of
    /// the cell, so `mat` matches both `zoho-matters` and
    /// `Matter_Timeline`.
    ///
    /// The same string map is used by local and server tables. A server
    /// consumer reads [`Column::filter_kind`] from the columns it supplied to
    /// interpret this entry as `Contains`; no finite option vocabulary is
    /// required for text-filter columns.
    pub fn filterable_text(mut self) -> Self {
        self.filterable = true;
        self.filter_kind = ColumnFilterKind::Contains;
        self
    }

    /// Returns the enabled filter kind, or `None` when this column has no
    /// filter-row control.
    pub fn filter_kind(&self) -> Option<ColumnFilterKind> {
        self.filterable.then_some(self.filter_kind)
    }

    /// Declare whether the free-text search box matches this column
    /// (`true` by default for every declared column).
    ///
    /// ```
    /// use leptos_daisyui_rs::components::Column;
    ///
    /// // A raw-epoch column the renderer formats: visible, but its digits
    /// // should not match what a user types.
    /// let deadline = Column::new("deadline_epoch", "Deadline").searched(false);
    /// ```
    pub fn searched(mut self, searched: bool) -> Self {
        self.searched = searched;
        self
    }

    /// Mark this column as holding row actions (buttons/links in its cells).
    ///
    /// On an interactive table (`selected_rows` / `on_row_activate`), a click
    /// on a row normally activates or selects it. Inside an action cell that
    /// is exactly wrong: pressing "Open" must not also fire the row's
    /// activation. Marking the column scopes row interaction away from the
    /// cell once, in the framework, instead of every cell renderer wrapping
    /// its buttons in `stop_propagation` by hand.
    ///
    /// ```
    /// use leptos_daisyui_rs::components::Column;
    ///
    /// let actions = Column::new_non_sortable("actions", "Actions").action().with_renderer(0);
    /// ```
    pub fn action(mut self) -> Self {
        self.is_action = true;
        self
    }

    /// Forbids an opt-in column chooser (e.g. `ServerDataTable`'s
    /// `column_tools`) from hiding this column.
    ///
    /// ```
    /// use leptos_daisyui_rs::components::Column;
    ///
    /// let name = Column::new("name", "Name").required();
    /// ```
    pub fn required(mut self) -> Self {
        self.required = true;
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
    /// Numbered pagination page-button class (non-active pages)
    pub pagination_page_button: &'static str,
    /// Numbered pagination page-button class for the current/active page
    pub pagination_active_page_button: &'static str,
    /// Row-range caption class (e.g. "Showing 1-10 of 42")
    pub row_range: &'static str,
}

impl Default for DataTableClasses {
    fn default() -> Self {
        Self {
            container: "w-full",
            header_cell: "cursor-pointer select-none",
            body_cell: "",
            row: "",
            // Muted + italic, NOT animate-pulse: the pulse animates opacity to 0.5,
            // so a contrast scanner (and a user, half the time) sees the loading
            // text at half its contrast — a real WCAG AA fail measured by axe.
            loading_row: "text-base-content/75 italic",
            // /75, not /50: base-content at 50% alpha fails WCAG AA color-contrast
            // on base-100 (axe BLOCKING, found by office-perf tier1_a11y
            // 2026-08-16 on the queue empty state); 75% is the muted-text
            // level the op-srip AA pass cleared across the consuming app.
            empty_row: "text-center text-base-content/75",
            selected_row: "bg-base-200",
            pagination: "flex justify-between items-center mt-4",
            pagination_button: "btn btn-sm",
            page_indicator: "text-sm",
            pagination_page_button: "btn btn-sm join-item",
            pagination_active_page_button: "btn btn-sm join-item btn-active",
            row_range: "text-sm text-base-content/75",
        }
    }
}

/// Customizable text strings for DataTable.
///
/// Fields are owned `String`s and the components take `texts` as a
/// `Signal<DataTableTexts>`, so table chrome can be localized at runtime:
/// derive the struct from your translation function inside a
/// `Signal::derive`/`Memo` that reads the active locale, and every string
/// re-renders on a language switch.
#[derive(Clone, Debug, PartialEq)]
pub struct DataTableTexts {
    /// Loading state text
    pub loading: String,
    /// Empty state text
    pub empty: String,
    /// Previous button text
    pub previous: String,
    /// Next button text
    pub next: String,
    /// Page indicator format (use {current} and {total} placeholders)
    pub page_indicator: String,
    /// Search input placeholder text
    pub search_placeholder: String,
    /// Accessible and associated label for the search input.
    pub search_label: String,
    /// Accessible and visible label for the server-query page-size selector.
    pub page_size_label: String,
    /// Row-range caption format (use {start}, {end}, and {total} placeholders)
    pub row_range: String,
    /// Label for the "no filter" option in every filter-row dropdown
    pub filter_all: String,
    /// Associated label template for a column filter; `{column}` is replaced.
    pub filter_label: String,
}

impl Default for DataTableTexts {
    fn default() -> Self {
        Self {
            loading: "Loading...".to_string(),
            empty: "No data available".to_string(),
            previous: "Previous".to_string(),
            next: "Next".to_string(),
            page_indicator: "Page {current} of {total}".to_string(),
            search_placeholder: "Search...".to_string(),
            search_label: "Search table".to_string(),
            page_size_label: "Rows per page".to_string(),
            row_range: "Showing {start}\u{2013}{end} of {total}".to_string(),
            filter_all: "All".to_string(),
            filter_label: "Filter by {column}".to_string(),
        }
    }
}

/// Localizable accessible-name templates for sortable table-header controls.
///
/// Each template must contain `{column}`. The focused control uses the
/// matching template to expose both its current state and the result of its
/// next plain activation; the parent header retains canonical `aria-sort`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataTableSortTexts {
    /// Name for a sortable column that is not currently active.
    pub unsorted: String,
    /// Name for the active ascending column.
    pub ascending: String,
    /// Name for the active descending column.
    pub descending: String,
}

impl DataTableSortTexts {
    /// Formats the focused sort control's complete accessible name.
    pub fn control_label(&self, column: &str, current: Option<SortOrder>) -> String {
        let template = match current {
            None => &self.unsorted,
            Some(SortOrder::Asc) => &self.ascending,
            Some(SortOrder::Desc) => &self.descending,
        };
        template.replace("{column}", column)
    }
}

impl Default for DataTableSortTexts {
    fn default() -> Self {
        Self {
            unsorted: "{column}, not sorted. Activate to sort ascending.".to_string(),
            ascending: "{column}, sorted ascending. Activate to sort descending.".to_string(),
            descending: "{column}, sorted descending. Activate to sort ascending.".to_string(),
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
        assert_eq!(col.typed_cell_index, None);
        assert!(!col.truncate);
    }

    #[test]
    fn column_new_is_resizable_by_default() {
        let col = Column::new("x", "X");
        assert!(col.resizable);
    }

    #[test]
    fn column_new_sorts_as_text_by_default() {
        assert_eq!(Column::new("x", "X").sort_as, SortAs::Text);
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
        assert_eq!(col.typed_cell_index, None);
        assert!(!col.truncate);
    }

    #[test]
    fn column_new_non_sortable_is_resizable_by_default() {
        let col = Column::new_non_sortable("x", "X");
        assert!(col.resizable);
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
    fn non_resizable_clears_resizable_flag() {
        let col = Column::new("id", "ID").non_resizable();
        assert!(!col.resizable);
    }

    #[test]
    fn with_typed_cell_sets_typed_cell_index() {
        let col = Column::new("status", "Status").with_typed_cell(3);
        assert_eq!(col.typed_cell_index, Some(3));
    }

    #[test]
    fn columns_are_not_action_by_default() {
        assert!(!Column::new("x", "X").is_action);
        assert!(!Column::new_non_sortable("x", "X").is_action);
    }

    #[test]
    fn action_sets_is_action() {
        let col = Column::new_non_sortable("actions", "Actions").action();
        assert!(col.is_action);
    }

    #[test]
    fn column_new_accepts_owned_header() {
        // Runtime localization hands out owned Strings (t() output).
        let translated = String::from("Nombre");
        let col = Column::new("name", translated);
        assert_eq!(col.header, "Nombre");
    }

    #[test]
    fn with_sort_as_sets_sort_as() {
        let col = Column::new("balance", "Balance").with_sort_as(SortAs::Number);
        assert_eq!(col.sort_as, SortAs::Number);
    }

    #[test]
    fn columns_with_different_sort_as_are_not_equal() {
        let a = Column::new("x", "X");
        let b = Column::new("x", "X").with_sort_as(SortAs::Number);
        assert_ne!(a, b);
    }

    #[test]
    fn builder_methods_chain() {
        let col = Column::new("url", "URL")
            .with_truncate()
            .with_max_width(400)
            .with_min_width(100)
            .with_class("monospace")
            .with_renderer(0)
            .non_resizable()
            .with_typed_cell(5);

        assert!(col.truncate);
        assert_eq!(col.max_width, Some(400));
        assert_eq!(col.min_width, Some(100));
        assert_eq!(col.class, Some("monospace"));
        assert_eq!(col.renderer_index, Some(0));
        assert!(!col.resizable);
        assert_eq!(col.typed_cell_index, Some(5));
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
        assert_eq!(classes.loading_row, "text-base-content/75 italic");
        assert_eq!(classes.empty_row, "text-center text-base-content/75");
        assert_eq!(classes.selected_row, "bg-base-200");
        assert_eq!(classes.pagination, "flex justify-between items-center mt-4");
        assert_eq!(classes.pagination_button, "btn btn-sm");
        assert_eq!(classes.page_indicator, "text-sm");
        assert_eq!(classes.pagination_page_button, "btn btn-sm join-item");
        assert_eq!(
            classes.pagination_active_page_button,
            "btn btn-sm join-item btn-active"
        );
        assert_eq!(classes.row_range, "text-sm text-base-content/75");
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
        assert_eq!(texts.search_label, "Search table");
        assert_eq!(texts.row_range, "Showing {start}\u{2013}{end} of {total}");
        assert_eq!(texts.filter_label, "Filter by {column}");
    }

    #[test]
    fn sort_control_texts_name_current_state_and_next_action() {
        let texts = DataTableSortTexts::default();
        assert_eq!(
            texts.control_label("Name", None),
            "Name, not sorted. Activate to sort ascending."
        );
        assert_eq!(
            texts.control_label("Name", Some(SortOrder::Asc)),
            "Name, sorted ascending. Activate to sort descending."
        );
        assert_eq!(
            texts.control_label("Name", Some(SortOrder::Desc)),
            "Name, sorted descending. Activate to sort ascending."
        );

        let localized = DataTableSortTexts {
            unsorted: "{column}, sin ordenar. Activar para ordenar ascendente.".to_string(),
            ascending: "{column}, orden ascendente. Activar para ordenar descendente.".to_string(),
            descending: "{column}, orden descendente. Activar para ordenar ascendente.".to_string(),
        };
        assert_eq!(
            localized.control_label("Nombre", Some(SortOrder::Asc)),
            "Nombre, orden ascendente. Activar para ordenar descendente."
        );
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
    fn columns_with_different_resizable_are_not_equal() {
        let a = Column::new("x", "X");
        let b = Column::new("x", "X").non_resizable();
        assert_ne!(a, b);
    }

    #[test]
    fn columns_with_different_typed_cell_index_are_not_equal() {
        let a = Column::new("x", "X").with_typed_cell(0);
        let b = Column::new("x", "X").with_typed_cell(1);
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

    // ── TypedCell::as_text ──

    #[test]
    fn typed_cell_text_as_text_returns_inner_string() {
        let cell = TypedCell::Text("hello".to_string());
        assert_eq!(cell.as_text(), "hello");
    }

    #[test]
    fn typed_cell_badge_as_text_returns_badge_text() {
        let cell = TypedCell::Badge {
            text: "Active".to_string(),
            color: BadgeColor::Success,
        };
        assert_eq!(cell.as_text(), "Active");
    }

    #[test]
    fn typed_cell_icon_as_text_is_empty() {
        let cell = TypedCell::Icon {
            name: "check".to_string(),
            color: "text-success".to_string(),
        };
        assert_eq!(cell.as_text(), "");
    }

    #[test]
    fn typed_cell_equality() {
        let a = TypedCell::Badge {
            text: "Active".to_string(),
            color: BadgeColor::Success,
        };
        let b = TypedCell::Badge {
            text: "Active".to_string(),
            color: BadgeColor::Success,
        };
        let c = TypedCell::Badge {
            text: "Active".to_string(),
            color: BadgeColor::Error,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
