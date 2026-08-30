use crate::components::data_table::auto_page::{
    DEFAULT_AUTO_MIN_ROWS, FALLBACK_HEADER_HEIGHT, FALLBACK_ROW_HEIGHT, RowHeightEra,
    auto_page_size_for_height, max_row_height,
};
use crate::components::data_table::body::{DataTableBody, DataTableBodyClick, DataTableBodyRow};
use crate::components::data_table::filter::{
    ColumnFilters, DataTableFilterOption, DataTableFilterOptions, DataTableFilterRow, FILTER_ALL,
    distinct_values, filter_options_from_strings, has_exact_filterable_columns,
    has_filterable_columns,
};
use crate::components::data_table::geometry::{
    StableColumnTrack, StableTableColGroup, stable_column_width, stable_table_content_style,
};
use crate::components::data_table::header::DataTableHeader;
use crate::components::data_table::selection::row_is_interactive;
use crate::components::data_table::types::{
    CellRenderer, Column, ColumnFilterKind, DataTableClasses, DataTableSortTexts, DataTableTexts,
    RowDetailRenderer, SortOrder, TableRow, TypedCellFn,
};
use crate::components::data_table::{TABLE_SCROLL_WRAPPER_CLASS, next_data_table_search_id};
use crate::components::table::{Table, TableSize};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use std::collections::{BTreeSet, HashMap};
use web_sys::wasm_bindgen::JsCast;

const KEYED_CALLBACK_WITHOUT_ROW_KEY_CONFIGURATION: &str =
    "ServerDataTable keyed row callbacks require row_key";
const SELECTION_WITHOUT_ROW_KEY_CONFIGURATION: &str =
    "ServerDataTable controlled selection requires row_key";
const MISSING_FILTER_VOCABULARY_CONFIGURATION: &str = "ServerDataTable exact filter columns require authoritative filter_options or an explicit current-slice vocabulary";
const CONFLICTING_FILTER_VOCABULARY_CONFIGURATION: &str =
    "ServerDataTable current-slice vocabulary cannot be combined with authoritative filter options";
const DUPLICATE_FILTER_OPTIONS_CONFIGURATION: &str =
    "ServerDataTable accepts either filter_options or filter_option_entries, not both";

/// Snapshot of the exact keyed server row that was activated or inspected.
///
/// Unlike the compatibility index callbacks, this value does not require the
/// consumer to look up a possibly-replaced page after an asynchronous event:
/// it carries the stable key, page-local index, and displayed row together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerTableRowAction {
    /// Stable business identity returned by `ServerDataTable::row_key`.
    pub key: String,
    /// Zero-based position within the displayed server page at event time.
    pub page_index: usize,
    /// Snapshot of the row displayed when the event fired.
    pub row: TableRow,
}

/// Controlled single-row selection for [`ServerDataTable`].
///
/// The supplied stable key is always displayed truth. Pointer or keyboard
/// selection emits a replacement proposal without optimistically changing the
/// row, so a rejected or delayed proposal leaves `aria-selected` and styling
/// aligned with the caller's accepted state.
#[derive(Clone, Copy)]
pub struct ServerTableSelection {
    selected_key: Signal<Option<String>>,
    on_change: Callback<Option<String>>,
}

impl ServerTableSelection {
    /// Creates controlled single-selection ownership.
    pub fn controlled(
        selected_key: Signal<Option<String>>,
        on_change: Callback<Option<String>>,
    ) -> Self {
        Self {
            selected_key,
            on_change,
        }
    }

    /// Returns the caller-owned accepted selection signal.
    pub fn selected_key(self) -> Signal<Option<String>> {
        self.selected_key
    }
}

fn selected_server_row_indices(
    rows: &[TableRow],
    selected_key: Option<&str>,
    key_of: impl Fn(&TableRow) -> String,
) -> BTreeSet<usize> {
    let Some(selected_key) = selected_key else {
        return BTreeSet::new();
    };
    rows.iter()
        .enumerate()
        .filter_map(|(index, row)| (key_of(row) == selected_key).then_some(index))
        .collect()
}

fn server_selection_proposal(key: &str, ctrl: bool, shift: bool) -> Option<String> {
    (!ctrl && !shift).then(|| key.to_owned())
}

/// Localized copy that explicitly tells users a server filter lists only the
/// values present in the displayed slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerCurrentSliceFilterTexts {
    /// Label for the empty/no-filter option, for example "All on this page".
    pub all_label: String,
    /// Accessible-name template containing `{column}`, for example
    /// "Filter current page by {column}".
    pub filter_label: String,
}

impl ServerCurrentSliceFilterTexts {
    /// Creates localized current-slice filter copy.
    pub fn new(all_label: impl Into<String>, filter_label: impl Into<String>) -> Self {
        Self {
            all_label: all_label.into(),
            filter_label: filter_label.into(),
        }
    }
}

impl Default for ServerCurrentSliceFilterTexts {
    fn default() -> Self {
        Self::new("All on this page", "Filter current page by {column}")
    }
}

/// Declares the truth represented by a server table's exact-value filters.
#[derive(Clone, Copy)]
pub enum ServerFilterVocabulary {
    /// `filter_options` is authoritative for the complete server population.
    Authoritative,
    /// Options are intentionally derived from only the displayed slice and
    /// labelled with caller-localizable copy that says so.
    CurrentSlice {
        /// Reactive localized labels for the current-slice contract.
        texts: Signal<ServerCurrentSliceFilterTexts>,
    },
}

impl ServerFilterVocabulary {
    /// Creates an explicitly labelled current-slice vocabulary.
    pub fn current_slice(texts: Signal<ServerCurrentSliceFilterTexts>) -> Self {
        Self::CurrentSlice { texts }
    }

    fn kind(self) -> ServerFilterVocabularyKind {
        match self {
            Self::Authoritative => ServerFilterVocabularyKind::Authoritative,
            Self::CurrentSlice { .. } => ServerFilterVocabularyKind::CurrentSlice,
        }
    }

    fn current_slice_texts(self) -> Option<Signal<ServerCurrentSliceFilterTexts>> {
        match self {
            Self::Authoritative => None,
            Self::CurrentSlice { texts } => Some(texts),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServerFilterVocabularyKind {
    Authoritative,
    CurrentSlice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolvedServerFilterVocabulary {
    Disabled,
    Authoritative,
    CurrentSlice,
}

fn resolve_server_filter_vocabulary(
    filtering_enabled: bool,
    has_filterable_columns: bool,
    has_filter_options: bool,
    declared: Option<ServerFilterVocabularyKind>,
) -> Result<ResolvedServerFilterVocabulary, &'static str> {
    if !filtering_enabled || !has_filterable_columns {
        return Ok(ResolvedServerFilterVocabulary::Disabled);
    }
    match (has_filter_options, declared) {
        (true, Some(ServerFilterVocabularyKind::CurrentSlice)) => {
            Err(CONFLICTING_FILTER_VOCABULARY_CONFIGURATION)
        }
        (true, _) => Ok(ResolvedServerFilterVocabulary::Authoritative),
        (false, Some(ServerFilterVocabularyKind::CurrentSlice)) => {
            Ok(ResolvedServerFilterVocabulary::CurrentSlice)
        }
        (false, _) => Err(MISSING_FILTER_VOCABULARY_CONFIGURATION),
    }
}

fn options_with_active_filter_values(
    mut options: DataTableFilterOptions,
    filters: &ColumnFilters,
    columns: &[Column],
) -> DataTableFilterOptions {
    for (column, active) in filters {
        if active == FILTER_ALL
            || columns
                .iter()
                .find(|candidate| candidate.id == *column)
                .and_then(Column::filter_kind)
                != Some(ColumnFilterKind::Exact)
        {
            continue;
        }
        let column_options = options.entry(*column).or_default();
        if !column_options.iter().any(|option| option.value == *active) {
            column_options.push(DataTableFilterOption::same(active.clone()));
        }
    }
    options
}

/// The full query a server-owned table is currently displaying: everything a
/// backend needs to produce the matching page. Emitted through
/// [`ServerDataTable`]'s `on_query_change` on every user change, so the query
/// round-trips instead of the server seeing only page numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct TableQuery {
    /// 1-based page number, matching `ServerDataTable`'s `current_page`.
    pub page: i64,
    /// Rows per page.
    pub page_size: i64,
    /// Debounced free-text search ("" when the box is empty or absent).
    pub search: String,
    /// Active sort as `(column id, order)`; `None` means the server's
    /// default order.
    pub sort: Option<(&'static str, SortOrder)>,
    /// Active per-column filter values. Interpret each value through the
    /// supplied column's [`Column::filter_kind`]: exact dropdowns use equality
    /// and [`Column::filterable_text`] inputs use substring matching.
    pub filters: ColumnFilters,
}

impl TableQuery {
    /// Creates an empty first-page offset query with a safe page size.
    pub fn first_page(page_size: i64) -> Self {
        Self {
            page: 1,
            page_size: page_size.max(1),
            search: String::new(),
            sort: None,
            filters: ColumnFilters::new(),
        }
    }

    /// Replaces free-text search and restarts paging.
    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.page = 1;
        self.search = search.into();
        self
    }

    /// Replaces server sorting and restarts paging.
    pub fn with_sort(mut self, sort: Option<(&'static str, SortOrder)>) -> Self {
        self.page = 1;
        self.sort = sort;
        self
    }

    /// Replaces controlled column filters and restarts paging.
    pub fn with_filters(mut self, filters: ColumnFilters) -> Self {
        self.page = 1;
        self.filters = filters;
        self
    }

    /// Replaces page size and restarts paging.
    pub fn with_page_size(mut self, page_size: i64) -> Self {
        self.page = 1;
        self.page_size = page_size.max(1);
        self
    }

    /// Replaces only the 1-based offset page.
    pub fn with_page(mut self, page: i64) -> Self {
        self.page = page.max(1);
        self
    }

    /// Clears every query-shape control while preserving the page-size choice.
    pub fn reset(mut self) -> Self {
        self.page = 1;
        self.search.clear();
        self.sort = None;
        self.filters.clear();
        self
    }
}

/// An opaque cursor supplied by a server API.
///
/// The table stores and returns this value but never parses, compares, exposes,
/// or otherwise assigns meaning to it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ServerCursorToken(String);

impl ServerCursorToken {
    /// Wraps an opaque transport cursor.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrows the exact opaque value supplied by the caller.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the exact opaque value supplied by the caller.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for ServerCursorToken {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ServerCursorToken {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Cursor-navigation intent carried by a [`ServerCursorQuery`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ServerCursorRequest {
    /// Restart from the server-defined first slice.
    #[default]
    First,
    /// Request the slice identified by the server's previous cursor.
    Previous(ServerCursorToken),
    /// Request the slice identified by the server's next cursor.
    Next(ServerCursorToken),
}

/// The complete controlled query represented by a cursor-paged server table.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerCursorQuery {
    /// Current navigation request. Query-shape builders reset this to `First`.
    pub request: ServerCursorRequest,
    /// Requested rows per slice.
    pub page_size: i64,
    /// Debounced free-text search.
    pub search: String,
    /// Active server sort.
    pub sort: Option<(&'static str, SortOrder)>,
    /// Active exact or contains column filters, distinguished by the matching
    /// supplied [`Column`] definition.
    pub filters: ColumnFilters,
}

impl ServerCursorQuery {
    /// Creates an empty query for the server-defined first slice.
    pub fn first_slice(page_size: i64) -> Self {
        Self {
            request: ServerCursorRequest::First,
            page_size: page_size.max(1),
            search: String::new(),
            sort: None,
            filters: ColumnFilters::new(),
        }
    }

    /// Replaces free-text search and restarts cursor navigation.
    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.request = ServerCursorRequest::First;
        self.search = search.into();
        self
    }

    /// Replaces server sorting and restarts cursor navigation.
    pub fn with_sort(mut self, sort: Option<(&'static str, SortOrder)>) -> Self {
        self.request = ServerCursorRequest::First;
        self.sort = sort;
        self
    }

    /// Replaces controlled column filters and restarts cursor navigation.
    pub fn with_filters(mut self, filters: ColumnFilters) -> Self {
        self.request = ServerCursorRequest::First;
        self.filters = filters;
        self
    }

    /// Replaces page size and restarts cursor navigation.
    pub fn with_page_size(mut self, page_size: i64) -> Self {
        self.request = ServerCursorRequest::First;
        self.page_size = page_size.max(1);
        self
    }

    /// Replaces only the opaque cursor-navigation intent.
    pub fn with_request(mut self, request: ServerCursorRequest) -> Self {
        self.request = request;
        self
    }

    /// Clears query-shape controls while preserving page size.
    pub fn reset(mut self) -> Self {
        self.request = ServerCursorRequest::First;
        self.search.clear();
        self.sort = None;
        self.filters.clear();
        self
    }
}

/// Declares which query-shape transitions a server endpoint accepts.
///
/// Pagination navigation is always available through the selected
/// [`ServerTablePagination`] strategy. These flags govern only the optional
/// search, page-size, sorting, and column-filter controls. The default keeps
/// the historical full-query behavior; [`Self::navigation_only`] is the
/// truthful choice for a fixed-size cursor endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerQueryCapabilities {
    search: bool,
    page_size: bool,
    sorting: bool,
    filtering: bool,
}

impl ServerQueryCapabilities {
    /// Enables every query-shape control, preserving the original API.
    pub const fn all() -> Self {
        Self {
            search: true,
            page_size: true,
            sorting: true,
            filtering: true,
        }
    }

    /// Disables every query-shape control while retaining pagination.
    pub const fn navigation_only() -> Self {
        Self {
            search: false,
            page_size: false,
            sorting: false,
            filtering: false,
        }
    }

    /// Enables or disables debounced free-text search.
    pub const fn with_search(mut self, enabled: bool) -> Self {
        self.search = enabled;
        self
    }

    /// Enables or disables the page-size selector.
    pub const fn with_page_size(mut self, enabled: bool) -> Self {
        self.page_size = enabled;
        self
    }

    /// Enables or disables sortable header controls.
    pub const fn with_sorting(mut self, enabled: bool) -> Self {
        self.sorting = enabled;
        self
    }

    /// Enables or disables column-filter controls.
    pub const fn with_filtering(mut self, enabled: bool) -> Self {
        self.filtering = enabled;
        self
    }

    /// Whether free-text search is supported.
    pub const fn search_enabled(self) -> bool {
        self.search
    }

    /// Whether changing the requested page size is supported.
    pub const fn page_size_enabled(self) -> bool {
        self.page_size
    }

    /// Whether server-side sorting is supported.
    pub const fn sorting_enabled(self) -> bool {
        self.sorting
    }

    /// Whether exact-value column filtering is supported.
    pub const fn filtering_enabled(self) -> bool {
        self.filtering
    }
}

impl Default for ServerQueryCapabilities {
    fn default() -> Self {
        Self::all()
    }
}

/// Truthful status of rows retained while a cursor request is unresolved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ServerCursorSliceState {
    /// The displayed rows are the accepted current slice.
    #[default]
    Current,
    /// The displayed rows are retained while a newer request is loading.
    RetainedWhileLoading,
    /// The displayed rows are retained because the latest request failed.
    RetainedAfterFailure,
}

impl ServerCursorSliceState {
    fn retains_rows(self) -> bool {
        matches!(
            self,
            Self::RetainedWhileLoading | Self::RetainedAfterFailure
        )
    }
}

/// Navigation metadata for the currently displayed cursor slice.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerCursorPage {
    /// Opaque cursor for the preceding slice, or `None` at the beginning.
    pub previous: Option<ServerCursorToken>,
    /// Opaque cursor for the following slice, or `None` at the end.
    pub next: Option<ServerCursorToken>,
    /// Whether these rows are current or deliberately retained.
    pub state: ServerCursorSliceState,
}

impl ServerCursorPage {
    /// Creates current-slice metadata from server-owned cursors.
    pub fn new(previous: Option<ServerCursorToken>, next: Option<ServerCursorToken>) -> Self {
        Self {
            previous,
            next,
            state: ServerCursorSliceState::Current,
        }
    }

    /// Labels the same slice as retained while a newer request is loading.
    pub fn retained_while_loading(mut self) -> Self {
        self.state = ServerCursorSliceState::RetainedWhileLoading;
        self
    }

    /// Labels the same slice as retained after the latest request failed.
    pub fn retained_after_failure(mut self) -> Self {
        self.state = ServerCursorSliceState::RetainedAfterFailure;
        self
    }
}

/// Localizable cursor-slice captions. Each template accepts `{count}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerCursorTexts {
    /// Caption for an accepted current slice.
    pub current: String,
    /// Caption while current rows remain visible during loading.
    pub retained_loading: String,
    /// Caption when current rows remain visible after a failed request.
    pub retained_failure: String,
}

impl Default for ServerCursorTexts {
    fn default() -> Self {
        Self {
            current: "Showing {count} rows".to_owned(),
            retained_loading: "Showing {count} retained rows while loading".to_owned(),
            retained_failure: "Showing {count} retained rows; latest request failed".to_owned(),
        }
    }
}

/// Explicit offset-paging signals for [`ServerTablePagination`].
#[derive(Clone, Copy)]
pub struct ServerOffsetPagination {
    current_page: Signal<i64>,
    total_count: Signal<i64>,
    page_size: Signal<i64>,
    on_page_change: Callback<i64>,
}

impl ServerOffsetPagination {
    /// Creates an offset strategy with a known total and numbered pages.
    pub fn new(
        current_page: Signal<i64>,
        total_count: Signal<i64>,
        page_size: Signal<i64>,
        on_page_change: Callback<i64>,
    ) -> Self {
        Self {
            current_page,
            total_count,
            page_size,
            on_page_change,
        }
    }
}

impl std::fmt::Debug for ServerOffsetPagination {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServerOffsetPagination")
            .finish_non_exhaustive()
    }
}

/// Caller-owned cursor query and accepted-slice metadata.
#[derive(Clone, Copy)]
pub struct ServerCursorPagination {
    current: Signal<ServerCursorQuery>,
    page: Signal<ServerCursorPage>,
    on_change: Callback<ServerCursorQuery>,
    texts: Signal<ServerCursorTexts>,
}

impl ServerCursorPagination {
    /// Creates a controlled cursor strategy.
    pub fn controlled(
        current: Signal<ServerCursorQuery>,
        page: Signal<ServerCursorPage>,
        on_change: Callback<ServerCursorQuery>,
    ) -> Self {
        Self {
            current,
            page,
            on_change,
            texts: Signal::stored(ServerCursorTexts::default()),
        }
    }

    /// Replaces cursor-only status captions with a reactive localization.
    pub fn with_texts(mut self, texts: Signal<ServerCursorTexts>) -> Self {
        self.texts = texts;
        self
    }
}

impl std::fmt::Debug for ServerCursorPagination {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServerCursorPagination")
            .finish_non_exhaustive()
    }
}

/// Mutually exclusive server pagination strategies.
#[derive(Clone, Copy, Debug)]
pub enum ServerTablePagination {
    /// Numbered offset paging with a known population total.
    Offset(ServerOffsetPagination),
    /// Opaque cursor paging with no invented total or page number.
    Cursor(ServerCursorPagination),
}

impl ServerTablePagination {
    /// Creates an explicit offset strategy.
    pub fn offset(
        current_page: Signal<i64>,
        total_count: Signal<i64>,
        page_size: Signal<i64>,
        on_page_change: Callback<i64>,
    ) -> Self {
        Self::Offset(ServerOffsetPagination::new(
            current_page,
            total_count,
            page_size,
            on_page_change,
        ))
    }

    /// Creates an explicit cursor strategy.
    pub const fn cursor(pagination: ServerCursorPagination) -> Self {
        Self::Cursor(pagination)
    }
}

const MIXED_PAGINATION_CONFIGURATION: &str =
    "ServerDataTable pagination is mutually exclusive with legacy offset props";
const INCOMPLETE_OFFSET_CONFIGURATION: &str =
    "ServerDataTable requires all four legacy offset props or one pagination strategy";
const DISABLED_SEARCH_CONFIGURATION: &str =
    "ServerDataTable query enables search while the search capability is disabled";
const DISABLED_SEARCH_CALLBACK_CONFIGURATION: &str =
    "ServerDataTable on_search requires the search capability";
const DISABLED_SORT_CONFIGURATION: &str =
    "ServerDataTable query enables sorting while the sorting capability is disabled";
const DISABLED_FILTER_CONFIGURATION: &str =
    "ServerDataTable query enables filtering while the filtering capability is disabled";

fn resolve_server_pagination(
    pagination: Option<ServerTablePagination>,
    current_page: Option<Signal<i64>>,
    total_count: Option<Signal<i64>>,
    page_size: Option<Signal<i64>>,
    on_page_change: Option<Callback<i64>>,
) -> Result<ServerTablePagination, &'static str> {
    let legacy_count = [
        current_page.is_some(),
        total_count.is_some(),
        page_size.is_some(),
        on_page_change.is_some(),
    ]
    .into_iter()
    .filter(|configured| *configured)
    .count();

    if let Some(pagination) = pagination {
        return if legacy_count == 0 {
            Ok(pagination)
        } else {
            Err(MIXED_PAGINATION_CONFIGURATION)
        };
    }

    if legacy_count != 4 {
        return Err(INCOMPLETE_OFFSET_CONFIGURATION);
    }

    Ok(ServerTablePagination::offset(
        current_page.expect("legacy count checked"),
        total_count.expect("legacy count checked"),
        page_size.expect("legacy count checked"),
        on_page_change.expect("legacy count checked"),
    ))
}

/// Declares who owns the offset query represented by a [`ServerDataTable`].
#[derive(Clone, Copy)]
pub enum ServerTableQueryOwnership {
    /// The caller supplies displayed-query truth and receives proposals.
    Controlled {
        /// Query represented by the currently displayed server slice.
        current: Signal<TableQuery>,
        /// One full replacement proposed for each user transition.
        on_change: Callback<TableQuery>,
    },
    /// The component owns query controls in memory. The compatibility
    /// `on_query_change` prop, when supplied, still receives replacements.
    Uncontrolled,
}

impl ServerTableQueryOwnership {
    /// Creates caller-controlled query ownership.
    pub fn controlled(current: Signal<TableQuery>, on_change: Callback<TableQuery>) -> Self {
        Self::Controlled { current, on_change }
    }

    /// Creates component-owned query state.
    pub const fn uncontrolled() -> Self {
        Self::Uncontrolled
    }
}

impl std::fmt::Debug for ServerTableQueryOwnership {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Controlled { .. } => formatter.debug_struct("Controlled").finish_non_exhaustive(),
            Self::Uncontrolled => formatter.write_str("Uncontrolled"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ServerQuerySnapshot {
    Offset(TableQuery),
    Cursor(ServerCursorQuery),
}

impl ServerQuerySnapshot {
    fn page_size(&self) -> i64 {
        match self {
            Self::Offset(query) => query.page_size,
            Self::Cursor(query) => query.page_size,
        }
    }

    fn search(&self) -> &str {
        match self {
            Self::Offset(query) => &query.search,
            Self::Cursor(query) => &query.search,
        }
    }

    fn sort(&self) -> Option<(&'static str, SortOrder)> {
        match self {
            Self::Offset(query) => query.sort,
            Self::Cursor(query) => query.sort,
        }
    }

    fn filters(&self) -> &ColumnFilters {
        match self {
            Self::Offset(query) => &query.filters,
            Self::Cursor(query) => &query.filters,
        }
    }

    fn offset_page(&self) -> Option<i64> {
        match self {
            Self::Offset(query) => Some(query.page),
            Self::Cursor(_) => None,
        }
    }

    #[cfg(test)]
    fn cursor_request(&self) -> Option<ServerCursorRequest> {
        match self {
            Self::Offset(_) => None,
            Self::Cursor(query) => Some(query.request.clone()),
        }
    }

    fn with_search(self, search: impl Into<String>) -> Self {
        match self {
            Self::Offset(query) => Self::Offset(query.with_search(search)),
            Self::Cursor(query) => Self::Cursor(query.with_search(search)),
        }
    }

    fn with_sort(self, sort: Option<(&'static str, SortOrder)>) -> Self {
        match self {
            Self::Offset(query) => Self::Offset(query.with_sort(sort)),
            Self::Cursor(query) => Self::Cursor(query.with_sort(sort)),
        }
    }

    fn with_filters(self, filters: ColumnFilters) -> Self {
        match self {
            Self::Offset(query) => Self::Offset(query.with_filters(filters)),
            Self::Cursor(query) => Self::Cursor(query.with_filters(filters)),
        }
    }

    fn with_page_size(self, page_size: i64) -> Self {
        match self {
            Self::Offset(query) => Self::Offset(query.with_page_size(page_size)),
            Self::Cursor(query) => Self::Cursor(query.with_page_size(page_size)),
        }
    }

    fn with_offset_page(self, page: i64) -> Self {
        match self {
            Self::Offset(query) => Self::Offset(query.with_page(page)),
            Self::Cursor(query) => Self::Cursor(query),
        }
    }

    fn with_cursor_request(self, request: ServerCursorRequest) -> Self {
        match self {
            Self::Offset(query) => Self::Offset(query),
            Self::Cursor(query) => Self::Cursor(query.with_request(request)),
        }
    }

    fn reset(self) -> Self {
        match self {
            Self::Offset(query) => Self::Offset(query.reset()),
            Self::Cursor(query) => Self::Cursor(query.reset()),
        }
    }
}

fn validate_server_query_capabilities(
    capabilities: ServerQueryCapabilities,
    query: &ServerQuerySnapshot,
    has_legacy_search_callback: bool,
) -> Result<(), &'static str> {
    if !capabilities.search_enabled() && !query.search().is_empty() {
        return Err(DISABLED_SEARCH_CONFIGURATION);
    }
    if !capabilities.search_enabled() && has_legacy_search_callback {
        return Err(DISABLED_SEARCH_CALLBACK_CONFIGURATION);
    }
    if !capabilities.sorting_enabled() && query.sort().is_some() {
        return Err(DISABLED_SORT_CONFIGURATION);
    }
    if !capabilities.filtering_enabled() && !query.filters().is_empty() {
        return Err(DISABLED_FILTER_CONFIGURATION);
    }
    Ok(())
}

const VIEWPORT_FIT_REQUIRES_PAGE_SIZE_CONFIGURATION: &str = "ServerDataTable viewport_fit requires an endpoint that accepts page-size changes (a fixed-slice endpoint or a disabled page-size capability rejects the policy)";

/// Resolves whether the opt-in `viewport_fit` policy (ldui-2bt3) is active
/// for the current configuration. Reusing `DataTable`'s own measurement math
/// (`auto_page.rs`) is pointless without somewhere to send the derived page
/// size: `page_size_controllable` mirrors `ServerDataTable`'s own
/// `has_page_size` gate (the query capability enabled AND a query callback
/// actually wired), so a fixed-slice cursor endpoint or a disabled page-size
/// capability fails closed and visibly rather than silently doing nothing.
fn resolve_viewport_fit(
    requested: bool,
    page_size_controllable: bool,
) -> Result<bool, &'static str> {
    if !requested {
        Ok(false)
    } else if !page_size_controllable {
        Err(VIEWPORT_FIT_REQUIRES_PAGE_SIZE_CONFIGURATION)
    } else {
        Ok(true)
    }
}

/// Derives one measurement pass's page-size proposal, or `None` when the
/// derived count already matches `accepted_page_size` (nothing to propose).
///
/// Comparing against the caller's currently *accepted* truth -- rather than
/// a locally cached "last proposed" value -- means a caller that declines a
/// proposal is asked again on the next genuine re-measurement (resize,
/// density, wrapping, column change), and a caller that accepts sees no
/// redundant repeat once the derived count matches what it already
/// supplied. `accepted_page_size` doubles as the "retain what's already
/// accepted" fallback `auto_page_size_for_height` falls back to below
/// `min_rows`, matching `DataTable::auto_page_size`'s own contract that a
/// too-short fit keeps the configured size and scrolls instead of shrinking
/// pagination toward unusability.
///
/// ```
/// use leptos_daisyui_rs::components::viewport_fit_page_size_proposal;
///
/// // 436px viewport, 36px header, 40px rows -> 10 rows fit; the caller is
/// // currently showing 5, so a growth proposal is due.
/// assert_eq!(
///     viewport_fit_page_size_proposal(436.0, 36.0, 40.0, 5, 5, 5),
///     Some(10),
/// );
/// // The derived count already matches what's accepted: nothing to propose.
/// assert_eq!(
///     viewport_fit_page_size_proposal(436.0, 36.0, 40.0, 10, 5, 10),
///     None,
/// );
/// ```
pub fn viewport_fit_page_size_proposal(
    viewport_height: f64,
    header_height: f64,
    row_height: f64,
    configured_page_size: i64,
    min_rows: usize,
    accepted_page_size: i64,
) -> Option<i64> {
    let accepted_page_size = accepted_page_size.max(1);
    let fallback = configured_page_size.max(accepted_page_size).max(1) as usize;
    let derived = auto_page_size_for_height(
        viewport_height,
        header_height,
        row_height,
        fallback,
        min_rows,
    ) as i64;
    (derived != accepted_page_size).then_some(derived)
}

/// Distinguishes a `rows` change induced by `ServerDataTable`'s OWN
/// just-sent `viewport_fit` proposal from every other cause (an external
/// query/sort/filter/dataset change, a declined or differed proposal, or a
/// container resize that happens to coincide with an unrelated refetch).
///
/// Reviewer trace (ldui-2bt3 CRITICAL fix): accepting a proposal makes the
/// caller refetch and replace `rows`, and treating every `rows` change as a
/// brand-new measurement era discards the tall-row high-water mark the
/// ratchet needs to converge -- accepted=5 fits 10 short rows, propose 10;
/// the refetched 10-row page reveals a tall row, but the fresh era forgets
/// it and derives 5; propose 5; the refetched 5-row page is short again (the
/// tall row falls outside a 5-row page), a fresh era forgets the tall
/// reading yet again and derives 10; propose 10 -- forever.
///
/// The caller's accepted query and its `rows` are not guaranteed to update
/// in the same reactive tick, so "own-induced" is detected retrospectively:
/// a `rows` change is own-induced exactly when a proposal is still pending
/// AND the accepted query's page size now equals the size that was
/// proposed. `ServerDataTable` keeps the SAME measurement era across an
/// own-induced change (never bumping the era's data-revision key) so the
/// row-height high-water mark carries forward; every other `rows` change
/// still starts a fresh era exactly as before.
fn viewport_fit_rows_change_is_own_induced(
    pending_proposal: Option<i64>,
    accepted_page_size: i64,
) -> bool {
    pending_proposal == Some(accepted_page_size)
}

/// Guards a scheduled (macrotask-delayed) `viewport_fit` measurement pass
/// against applying a stale result after a *newer* pass has already been
/// scheduled -- e.g. two `ResizeObserver` callbacks fire in quick
/// succession and their zero-delay timers could in principle run out of
/// schedule order. Every scheduled pass is stamped with the sequence number
/// returned by [`Self::next`] at *schedule* time (not run time); a pass may
/// only apply its proposal when [`Self::is_current`] says its stamp is still
/// the newest ever scheduled. This is deliberately independent of (and a
/// belt-and-braces pairing with) cancelling the previous pending timer
/// handle before scheduling a new one -- the same "only the most recent
/// scheduled pass may act" invariant, expressed as a pure, unit-testable
/// value instead of a side effect on a `TimeoutHandle`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewportFitEpoch(u64);

impl ViewportFitEpoch {
    /// A fresh epoch with nothing scheduled yet.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Stamps and returns `(next_epoch, stamp)` for a newly scheduled pass.
    /// Calling this immediately invalidates every earlier-stamped pass that
    /// has not yet applied, even if that pass has not run yet.
    #[must_use]
    pub fn next(self) -> (Self, u64) {
        let stamp = self.0.wrapping_add(1);
        (Self(stamp), stamp)
    }

    /// Whether a pass stamped `candidate` (from a prior [`Self::next`]) is
    /// still the newest scheduled pass -- i.e. safe to apply.
    pub fn is_current(self, candidate: u64) -> bool {
        candidate == self.0
    }
}

#[derive(Clone, Copy)]
enum ServerQuerySource {
    OffsetControlled {
        current: Signal<TableQuery>,
        on_change: Callback<TableQuery>,
    },
    OffsetUncontrolled {
        current: RwSignal<TableQuery>,
    },
    CursorControlled {
        current: Signal<ServerCursorQuery>,
        on_change: Callback<ServerCursorQuery>,
    },
}

#[derive(Clone, Copy)]
struct ServerQueryState {
    source: ServerQuerySource,
}

impl ServerQueryState {
    fn new_offset(ownership: Option<ServerTableQueryOwnership>, initial: TableQuery) -> Self {
        let source = match ownership.unwrap_or(ServerTableQueryOwnership::Uncontrolled) {
            ServerTableQueryOwnership::Controlled { current, on_change } => {
                ServerQuerySource::OffsetControlled { current, on_change }
            }
            ServerTableQueryOwnership::Uncontrolled => ServerQuerySource::OffsetUncontrolled {
                current: RwSignal::new(initial),
            },
        };
        Self { source }
    }

    fn new_cursor(
        current: Signal<ServerCursorQuery>,
        on_change: Callback<ServerCursorQuery>,
    ) -> Self {
        Self {
            source: ServerQuerySource::CursorControlled { current, on_change },
        }
    }

    fn get(self) -> ServerQuerySnapshot {
        match self.source {
            ServerQuerySource::OffsetControlled { current, .. } => {
                ServerQuerySnapshot::Offset(current.get())
            }
            ServerQuerySource::OffsetUncontrolled { current } => {
                ServerQuerySnapshot::Offset(current.get())
            }
            ServerQuerySource::CursorControlled { current, .. } => {
                ServerQuerySnapshot::Cursor(current.get())
            }
        }
    }

    fn get_untracked(self) -> ServerQuerySnapshot {
        match self.source {
            ServerQuerySource::OffsetControlled { current, .. } => {
                ServerQuerySnapshot::Offset(current.get_untracked())
            }
            ServerQuerySource::OffsetUncontrolled { current } => {
                ServerQuerySnapshot::Offset(current.get_untracked())
            }
            ServerQuerySource::CursorControlled { current, .. } => {
                ServerQuerySnapshot::Cursor(current.get_untracked())
            }
        }
    }

    fn propose(self, next: ServerQuerySnapshot, legacy: Option<Callback<TableQuery>>) {
        match (self.source, next) {
            (
                ServerQuerySource::OffsetControlled { on_change, .. },
                ServerQuerySnapshot::Offset(query),
            ) => on_change.run(query),
            (
                ServerQuerySource::OffsetUncontrolled { current },
                ServerQuerySnapshot::Offset(query),
            ) => {
                current.set(query.clone());
                if let Some(callback) = legacy {
                    callback.run(query);
                }
            }
            (
                ServerQuerySource::CursorControlled { on_change, .. },
                ServerQuerySnapshot::Cursor(query),
            ) => on_change.run(query),
            _ => unreachable!("query proposal did not match the pagination strategy"),
        }
    }

    fn sync_legacy_offset(self, page: i64, page_size: i64) {
        if let ServerQuerySource::OffsetUncontrolled { current } = self.source {
            current.update(|query| {
                query.page = page.max(1);
                query.page_size = page_size.max(1);
            });
        }
    }
}

/// # ServerDataTable Component
///
/// A server-side paginated data table where the parent component controls
/// data fetching. Unlike `DataTable`, this component does NOT perform
/// client-side sorting, filtering, or pagination. The `rows` prop contains
/// pre-fetched data for the current page, and `on_page_change` is called
/// when the user navigates to a different page.
///
/// ## Example
/// ```rust,no_run
/// use std::collections::HashMap;
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::*;
///
/// #[component]
/// fn MyServerTable() -> impl IntoView {
///     let columns = vec![
///         Column::new("name", "Name"),
///         Column::new("email", "Email"),
///     ];
///
///     let (rows, set_rows) = signal(vec![]);
///     let (current_page, set_current_page) = signal(1_i64);
///     let (total_count, _) = signal(100_i64);
///     let (page_size, _) = signal(10_i64);
///     let (loading, set_loading) = signal(false);
///
///     let on_page_change = Callback::new(move |page: i64| {
///         set_current_page.set(page);
///         // Fetch data from server for the new page...
///     });
///
///     view! {
///         <ServerDataTable
///             rows=Signal::derive(move || rows.get())
///             columns=Signal::derive(move || columns.clone())
///             current_page=Signal::derive(move || current_page.get())
///             total_count=Signal::derive(move || total_count.get())
///             page_size=Signal::derive(move || page_size.get())
///             on_page_change=on_page_change
///             loading=Signal::derive(move || loading.get())
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("table table-zebra table-pin-rows table-pin-cols table-xs table-sm table-md table-lg");
/// @source inline("btn btn-sm btn-ghost btn-active animate-pulse join join-item");
/// // Column-resize divider (header.rs)
/// @source inline("relative absolute top-0 right-0 z-10 h-full w-2 cursor-col-resize select-none");
/// @source inline("opacity-0 hover:opacity-100 hover:bg-primary/50 focus:opacity-100 focus:bg-primary/50 focus:outline focus:outline-2 focus:outline-primary active:opacity-100 active:bg-primary/70");
/// // Typed cells (Column::with_typed_cell -> TypedCell::Badge / TypedCell::Icon)
/// @source inline("badge badge-neutral badge-primary badge-secondary badge-accent badge-info badge-success badge-warning badge-error");
/// @source inline("inline-block w-4 h-4 w-5 h-5 w-6 h-6 w-8 h-8 w-12 h-12");
/// ```
///
/// ## Node References
/// - `node_ref` - References the container div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ServerDataTable(
    /// Pre-fetched rows for the current page (from server API)
    #[prop(into)]
    rows: Signal<Vec<TableRow>>,

    /// Column definitions
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Compatibility offset page number (1-based). Supply all four legacy
    /// offset props, or omit them and use `pagination`.
    #[prop(optional, into)]
    current_page: Option<Signal<i64>>,

    /// Compatibility population total for numbered offset paging.
    #[prop(optional, into)]
    total_count: Option<Signal<i64>>,

    /// Compatibility rows-per-page signal for numbered offset paging.
    #[prop(optional, into)]
    page_size: Option<Signal<i64>>,

    /// Compatibility callback for numbered offset navigation.
    #[prop(optional, into)]
    on_page_change: Option<Callback<i64>>,

    /// Explicit offset or cursor pagination. It is a configuration error to
    /// combine this with any legacy offset prop.
    #[prop(optional)]
    pagination: Option<ServerTablePagination>,

    /// Query-shape operations accepted by the backing endpoint. Pagination
    /// navigation remains available independently. Defaults to all controls
    /// for source and behavior compatibility.
    #[prop(optional)]
    query_capabilities: ServerQueryCapabilities,

    /// Loading state
    #[prop(optional, into)]
    loading: Signal<bool>,

    /// Custom CSS classes
    #[prop(optional)]
    classes: DataTableClasses,

    /// Custom text strings. A `Signal` so table chrome can be localized at
    /// runtime -- see [`DataTableTexts`].
    #[prop(into, default = Signal::stored(DataTableTexts::default()))]
    texts: Signal<DataTableTexts>,

    /// Localized accessible-name templates for sortable header controls.
    #[prop(into, default = Signal::stored(DataTableSortTexts::default()))]
    sort_texts: Signal<DataTableSortTexts>,

    /// Localized accessible-name template for substring column filters;
    /// `{column}` is replaced with the live header.
    #[prop(into, default = Signal::stored("Filter {column} by text".to_owned()))]
    text_filter_label: Signal<String>,

    /// Additional CSS classes for container
    #[prop(optional, into)]
    class: &'static str,

    /// Table size variant
    #[prop(optional, into)]
    table_size: Signal<TableSize>,

    /// Enable zebra striping
    #[prop(optional, into)]
    zebra: Signal<bool>,

    /// Pin header and footer rows
    #[prop(optional, into)]
    pin_rows: Signal<bool>,

    /// Pin first column
    #[prop(optional, into)]
    pin_cols: Signal<bool>,

    /// Maximum height for viewport-constrained scrolling (e.g. "calc(100vh - 260px)")
    #[prop(optional, into)]
    max_height: Option<String>,

    /// Opt-in viewport-fit page-size policy (ldui-2bt3). Reuses exactly the
    /// same measurement math as `DataTable`'s `auto_page_size` -- the
    /// rendered header/row heights, the era-scoped high-water-mark ratchet
    /// that prevents measure -> page-size -> rendered-set oscillation -- but
    /// PROPOSES the derived page size as a query change instead of slicing
    /// rows locally. An offset query resets to page one; a cursor query
    /// resets to `First` (an existing previous/next token was minted for the
    /// old size and is never replayed against a new one). A configuration
    /// that cannot accept page-size changes -- a fixed-slice cursor endpoint,
    /// `ServerQueryCapabilities::page_size_enabled() == false`, or no query
    /// callback wired at all -- rejects the policy visibly (a `role="alert"`
    /// panel with `data-server-viewport-fit-config-error`) rather than
    /// silently doing nothing. Requires the same definite height as
    /// `auto_page_size`: pass `max_height` (promoted to a real `height`) or
    /// give the table a parent with a definite height.
    ///
    /// ## Accepted tradeoff: the row-height memory can stay conservative
    ///
    /// Accepting a proposal makes the caller refetch and replace `rows`, and
    /// that refetch is itself a `rows` change. Treating every `rows` change
    /// as a brand-new measurement era would discard the tall-row high-water
    /// mark the ratchet needs to converge, and does not converge at all: a
    /// short page proposes growth, the grown page reveals a tall row and
    /// proposes shrinking back, the shrunk page is short again and proposes
    /// growth again -- forever. `ServerDataTable` instead tracks its own
    /// just-sent proposal and, when the accepted query's page size lands on
    /// exactly that value, treats the resulting `rows` change as the SAME
    /// era and carries the high-water mark forward; any other `rows` change
    /// (a query/sort/filter/dataset change, a declined or differed
    /// proposal) still starts a fresh era. The cost is the same one
    /// `DataTable::auto_page_size`'s own era already accepts: once a tall
    /// row has been seen, later pages that happen to be all-short are still
    /// measured as if the tall row were present until something external
    /// resets the era -- a possible under-fill rather than an oscillation.
    #[prop(optional, into)]
    viewport_fit: Signal<bool>,

    /// Usability floor for `viewport_fit` (default: 5). Mirrors
    /// `DataTable`'s `min_rows`: below this measured fit, the currently
    /// accepted page size is retained (no proposal is sent) and the table's
    /// existing scroll wrapper absorbs the overflow.
    #[prop(into, default = Signal::derive(|| DEFAULT_AUTO_MIN_ROWS))]
    viewport_fit_min_rows: Signal<usize>,

    /// Callback for server-side search (fires after 300ms debounce)
    #[prop(optional, into)]
    on_search: Option<Callback<String>>,

    /// Typed query/change API: fired with the full [`TableQuery`] (page, page
    /// size, search, sort, filters) on **every** user change -- page
    /// navigation, debounced search, a sort toggle on a sortable header, or a
    /// filter-row selection. Query-shape changes (search/sort/filters) report
    /// `page: 1`, since the old page number is meaningless against a new
    /// result set.
    ///
    /// This is the server-owned contract: the table renders whatever `rows`
    /// holds and never sorts or filters client-side; it only *reports* the
    /// query for the caller to re-fetch with. Supplying this callback is also
    /// what arms header sorting (without it a header click has nowhere to
    /// go, and sortable headers stay inert as before).
    #[prop(optional, into)]
    on_query_change: Option<Callback<TableQuery>>,

    /// Explicit query ownership. Controlled mode makes one supplied
    /// [`TableQuery`] the source of truth for every visible query control and
    /// emits full-replacement proposals without mutating it. Omit this prop to
    /// retain the component-owned compatibility behavior.
    #[prop(optional)]
    query_ownership: Option<ServerTableQueryOwnership>,

    /// Optional dataset/access identity. A change proposes a complete query
    /// reset (page one, empty search/sort/filters, preserved page size).
    #[prop(optional, into)]
    query_reset_key: Option<Signal<String>>,

    /// Choices shown by the server-query page-size selector.
    #[prop(
        optional,
        into,
        default = Signal::stored(vec![10_i64, 25_i64, 50_i64, 100_i64])
    )]
    page_size_options: Signal<Vec<i64>>,

    /// Authoritative option lists for exact [`Column::filterable`] columns,
    /// keyed by column id. Text-filter columns need no finite vocabulary.
    /// When exact columns are present and options are absent, callers must
    /// explicitly choose a labelled current-slice `filter_vocabulary`.
    #[prop(optional, into)]
    filter_options: Option<Signal<HashMap<&'static str, Vec<String>>>>,

    /// Authoritative typed option entries whose stable submitted `value` can
    /// differ from their reactive localized `label`. Mutually exclusive with
    /// the source-compatible string-only `filter_options` prop.
    #[prop(optional, into)]
    filter_option_entries: Option<Signal<DataTableFilterOptions>>,

    /// Truthfulness policy for exact-value filter options. Supplying
    /// `filter_options` is authoritative by default. Without options, callers
    /// must explicitly choose `CurrentSlice` and provide localized labels
    /// that say the dropdown covers only the displayed slice.
    #[prop(optional)]
    filter_vocabulary: Option<ServerFilterVocabulary>,

    /// Node reference to container element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Per-cell renderers indexed by `Column::renderer_index`. A column with
    /// `renderer_index = Some(i)` invokes `cell_renderers[i]` with
    /// `(abs_idx, row)` to produce its cell view; columns without an index
    /// render `row[col.id]` as text. Out-of-bounds indices fall back to text.
    #[prop(optional)]
    cell_renderers: Vec<CellRenderer>,

    /// Per-cell typed-cell resolvers indexed by `Column::typed_cell_index`,
    /// for lightweight `Badge`/`Icon` rendering without a full custom
    /// `CellRenderer`. Additive alongside `cell_renderers` -- a column's
    /// `renderer_index` (when set) always takes precedence.
    #[prop(optional)]
    typed_cells: Vec<TypedCellFn>,

    /// Optional full-width detail content rendered immediately after a row.
    #[prop(optional)]
    detail_renderer: Option<RowDetailRenderer>,

    /// Optional per-row extra CSS classes (e.g. a background tint) computed
    /// from the row's absolute index and data. Merged with `classes.row`.
    #[prop(optional)]
    row_class_fn: Option<Callback<(usize, TableRow), String>>,

    /// Stable business identity for each displayed row. When supplied, row
    /// DOM is keyed by this value across reorder, insertion, removal, cursor
    /// replacement, and reactive row updates. Empty or duplicate keys render
    /// a fail-closed table-body error instead of aliasing entities.
    #[prop(optional, into)]
    row_key: Option<Callback<TableRow, String>>,

    /// Optional controlled single-row selection, keyed by `row_key`. The
    /// accepted signal drives styling and `aria-selected`; plain click,
    /// Enter, or Space emits one replacement proposal. Ctrl/Meta/Shift
    /// gestures are ignored because this is deliberately not multi-select.
    #[prop(optional)]
    selection: Option<ServerTableSelection>,

    /// Optional callback fired on a **plain** row click (no Ctrl/Shift) or a
    /// keyboard Enter/Space, receiving the row's index **within the current
    /// page** (the server variant renders one page at a time; combine with
    /// `current_page`/`page_size` for a global position). Same contract as
    /// [`DataTable`](super::DataTable)'s `on_row_activate` (`ldui-1gp`) —
    /// e.g. navigate to the row's detail page. A modified click stays inert
    /// here (the server variant has no selection state machine).
    #[prop(optional, into)]
    on_row_activate: Option<Callback<usize>>,

    /// Keyed activation snapshot for a plain click or Enter/Space. Requires
    /// `row_key`; can be used alongside the compatibility index callback.
    #[prop(optional, into)]
    on_row_activate_keyed: Option<Callback<ServerTableRowAction>>,

    /// Optional secondary activation fired on a row **double-click** or
    /// Shift+Enter, receiving the page-local row index — same dblclick
    /// discrimination as the client-paged table (`ldui-tmr`/`ldui-1gp`): the
    /// first click still activates once, the repeat click is swallowed so
    /// activation never fires twice, and the inspector fires exactly once.
    #[prop(optional, into)]
    on_row_inspect: Option<Callback<usize>>,

    /// Keyed inspection snapshot for double-click or Shift+Enter. Requires
    /// `row_key`; can be used alongside the compatibility index callback.
    #[prop(optional, into)]
    on_row_inspect_keyed: Option<Callback<ServerTableRowAction>>,
) -> impl IntoView {
    // Column-width overrides from dragging a header divider, keyed by
    // column id. Shared between the header (writer) and body (reader) so
    // resized columns stay aligned.
    let column_widths = RwSignal::new(HashMap::<&'static str, f64>::new());

    // Row activation, forwarded to the shared body exactly like the
    // client-paged DataTable (ldui-1gp): a plain click/Enter/Space activates;
    // a modified click is inert here because the server variant has no
    // selection state machine to feed. `on_row_inspect` rides the body's own
    // dblclick discrimination (detail > 1 swallow), so the timing matches
    // the client table by construction.
    // A bare Callback (not Option) so the body's `optional, into` prop takes
    // it the same way DataTable's own forwarding does — passing an Option
    // here is the E0308 trap the ldui-tmr CI fix documented.
    let has_row_activation = on_row_activate.is_some() || on_row_activate_keyed.is_some();
    let selected_rows = Signal::derive(move || match (selection, row_key) {
        (Some(selection), Some(key_of)) => {
            let selected_key = selection.selected_key.get();
            rows.with(|rows| {
                selected_server_row_indices(rows, selected_key.as_deref(), |row| {
                    key_of.run(row.clone())
                })
            })
        }
        _ => BTreeSet::new(),
    });
    let on_row_click = Callback::new(move |event: DataTableBodyClick| {
        let Some(proposed_key) = event
            .row
            .stable_key
            .as_deref()
            .and_then(|key| server_selection_proposal(key, event.ctrl, event.shift))
        else {
            if !event.ctrl
                && !event.shift
                && let Some(callback) = on_row_activate
            {
                callback.run(event.row.index);
            }
            return;
        };
        if let Some(selection) = selection {
            selection.on_change.run(Some(proposed_key.clone()));
        }
        if let Some(callback) = on_row_activate {
            callback.run(event.row.index);
        }
        if let Some(callback) = on_row_activate_keyed {
            callback.run(ServerTableRowAction {
                key: proposed_key,
                page_index: event.row.index,
                row: event.row.row,
            });
        }
    });
    let body_on_row_inspect = if on_row_inspect.is_some() || on_row_inspect_keyed.is_some() {
        Some(Callback::new(move |row: DataTableBodyRow| {
            if let Some(callback) = on_row_inspect {
                callback.run(row.index);
            }
            if let (Some(callback), Some(key)) = (on_row_inspect_keyed, row.stable_key) {
                callback.run(ServerTableRowAction {
                    key,
                    page_index: row.index,
                    row: row.row,
                });
            }
        }))
    } else {
        None
    };
    // Inspect alone still needs focusable rows for its Shift+Enter path.
    let row_interactive = row_is_interactive(
        false,
        selection.is_some()
            || has_row_activation
            || on_row_inspect.is_some()
            || on_row_inspect_keyed.is_some(),
    );
    let container_class = merge_classes!(classes.container, class);

    let row_key_configuration_error = if row_key.is_none() && selection.is_some() {
        Some(SELECTION_WITHOUT_ROW_KEY_CONFIGURATION)
    } else if row_key.is_none()
        && (on_row_activate_keyed.is_some() || on_row_inspect_keyed.is_some())
    {
        Some(KEYED_CALLBACK_WITHOUT_ROW_KEY_CONFIGURATION)
    } else {
        None
    };
    if let Some(message) = row_key_configuration_error {
        return view! {
            <div
                class=container_class
                role="alert"
                data-server-row-key-config-error=message
            >
                {message}
            </div>
        }
        .into_any();
    }

    let pagination = match resolve_server_pagination(
        pagination,
        current_page,
        total_count,
        page_size,
        on_page_change,
    ) {
        Ok(pagination) => pagination,
        Err(message) => {
            return view! {
                <div
                    class=container_class
                    role="alert"
                    data-server-pagination-config-error=message
                >
                    {message}
                </div>
            }
            .into_any();
        }
    };

    let cursor_pagination = matches!(pagination, ServerTablePagination::Cursor(_));
    let controlled_offset_query = matches!(
        query_ownership,
        Some(ServerTableQueryOwnership::Controlled { .. })
    );
    let query_configuration_error = match pagination {
        ServerTablePagination::Offset(_)
            if controlled_offset_query && on_query_change.is_some() =>
        {
            Some(
                "ServerDataTable controlled query ownership is mutually exclusive with on_query_change",
            )
        }
        ServerTablePagination::Cursor(_)
            if query_ownership.is_some() || on_query_change.is_some() =>
        {
            Some(
                "ServerDataTable cursor pagination owns its query and cannot use offset query props",
            )
        }
        _ => None,
    };
    if let Some(message) = query_configuration_error {
        return view! {
            <div
                class=container_class
                role="alert"
                data-server-pagination-config-error=message
            >
                {message}
            </div>
        }
        .into_any();
    }

    let query_state = match pagination {
        ServerTablePagination::Offset(offset) => ServerQueryState::new_offset(
            query_ownership,
            TableQuery {
                page: offset.current_page.get_untracked().max(1),
                page_size: offset.page_size.get_untracked().max(1),
                search: String::new(),
                sort: None,
                filters: ColumnFilters::new(),
            },
        ),
        ServerTablePagination::Cursor(cursor) => {
            ServerQueryState::new_cursor(cursor.current, cursor.on_change)
        }
    };
    if let Err(message) = validate_server_query_capabilities(
        query_capabilities,
        &query_state.get_untracked(),
        on_search.is_some(),
    ) {
        return view! {
            <div
                class=container_class
                role="alert"
                data-server-query-capability-config-error=message
            >
                {message}
            </div>
        }
        .into_any();
    }
    if let ServerTablePagination::Offset(offset) = pagination
        && !controlled_offset_query
    {
        Effect::new(move |_| {
            query_state.sync_legacy_offset(offset.current_page.get(), offset.page_size.get());
        });
    }

    // Browser controls are projections of one full query. Draft search text
    // is kept only for the debounce interval; once a proposal fires it is
    // reasserted from supplied truth if the caller declines or delays it.
    let search_draft = RwSignal::new(query_state.get_untracked().search().to_owned());
    let search_input = NodeRef::<leptos::html::Input>::new();
    let page_size_select = NodeRef::<leptos::html::Select>::new();
    let (debounce_handle, set_debounce_handle) = signal(Option::<TimeoutHandle>::None);
    let has_query_callback =
        cursor_pagination || controlled_offset_query || on_query_change.is_some();
    let has_search =
        query_capabilities.search_enabled() && (on_search.is_some() || has_query_callback);
    let has_page_size = query_capabilities.page_size_enabled() && has_query_callback;
    let sorting_enabled = query_capabilities.sorting_enabled() && has_query_callback;
    let filtering_enabled = query_capabilities.filtering_enabled();
    let column_filters = RwSignal::new(query_state.get_untracked().filters().clone());
    let effective_columns = Signal::derive(move || {
        let mut effective = columns.get();
        if !sorting_enabled {
            for column in &mut effective {
                column.sortable = false;
            }
        }
        if !filtering_enabled {
            for column in &mut effective {
                column.filterable = false;
            }
        }
        effective
    });

    Effect::new(move |_| {
        let supplied = query_state.get();
        if search_draft.get_untracked() != supplied.search() {
            search_draft.set(supplied.search().to_owned());
        }
        if column_filters.get_untracked() != *supplied.filters() {
            column_filters.set(supplied.filters().clone());
        }
    });

    let on_search_input = move |ev: leptos::ev::Event| {
        // The event always has an `HtmlInputElement` target in practice, but
        // if the browser ever hands us something that doesn't cast cleanly,
        // skip this keystroke rather than panicking the whole WASM app.
        let Some(target) = ev.target() else {
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            return;
        };
        let value = input.value();
        search_draft.set(value.clone());

        // Clear previous timer, if any.
        if let Some(handle) = debounce_handle.get_untracked() {
            handle.clear();
        }

        // Set new 300ms debounce timer. If scheduling fails (no `window`,
        // e.g. outside a browser context), fall back to running the search
        // immediately instead of silently dropping the keystroke.
        let fire = move |v: String| {
            if let Some(cb) = on_search {
                cb.run(v.clone());
            }
            query_state.propose(query_state.get_untracked().with_search(v), on_query_change);
            let supplied = query_state.get_untracked().search().to_owned();
            search_draft.set(supplied.clone());
            if let Some(input) = search_input.get() {
                input.set_value(&supplied);
            }
        };
        let value_for_timeout = value.clone();
        match set_timeout_with_handle(
            move || {
                // Late-firing guard (ldui-d54): `fire` reaches emit_query,
                // which untracked-reads several of this owner's signals. A
                // debounce that outlives the table must be a no-op, not a
                // process-wide disposed-signal panic.
                if search_draft.try_get_untracked().is_none() {
                    return;
                }
                fire(value_for_timeout);
            },
            std::time::Duration::from_millis(300),
        ) {
            Ok(handle) => set_debounce_handle.set(Some(handle)),
            Err(_) => {
                fire(value);
                set_debounce_handle.set(None);
            }
        }
    };
    // Cancel a pending debounce on unmount — mirrors DataTable's own search
    // debounce cleanup; keep the two in sync (ldui-d54).
    on_cleanup(move || {
        if let Some(handle) = debounce_handle.try_get_untracked().flatten() {
            handle.clear();
        }
    });

    // Header sort proposes from supplied truth; a declined proposal therefore
    // leaves both `aria-sort` and the button name unchanged.
    let on_sort = Callback::new(move |col_id: &'static str| {
        if !sorting_enabled {
            return;
        }
        let current = query_state.get_untracked();
        let next_sort = match current.sort() {
            Some((column, order)) if column == col_id => Some((col_id, order.toggle())),
            _ => Some((col_id, SortOrder::Asc)),
        };
        query_state.propose(current.with_sort(next_sort), on_query_change);
    });

    // Page navigation: keep the plain `on_page_change` contract and mirror it
    // through the typed query.
    let page_change = Callback::new(move |page: i64| {
        if let ServerTablePagination::Offset(offset) = pagination {
            offset.on_page_change.run(page);
            query_state.propose(
                query_state.get_untracked().with_offset_page(page),
                on_query_change,
            );
        }
    });
    let cursor_change = Callback::new(move |request: ServerCursorRequest| {
        if cursor_pagination {
            query_state.propose(
                query_state.get_untracked().with_cursor_request(request),
                None,
            );
        }
    });

    let filter_change = Callback::new(move |filters: ColumnFilters| {
        if !filtering_enabled {
            return;
        }
        query_state.propose(
            query_state.get_untracked().with_filters(filters),
            on_query_change,
        );
        let supplied = query_state.get_untracked().filters().clone();
        if column_filters.get_untracked() != supplied {
            column_filters.set(supplied);
        }
    });

    if let Some(query_reset_key) = query_reset_key {
        let previous_reset_key = StoredValue::new(query_reset_key.get_untracked());
        Effect::new(move |_| {
            let next_key = query_reset_key.get();
            if previous_reset_key.get_value() != next_key {
                query_state.propose(query_state.get_untracked().reset(), on_query_change);
                previous_reset_key.set_value(next_key);
            }
        });
    }

    // Filter row plumbing. A server slice is never silently presented as a
    // population-wide vocabulary: authoritative options are explicit, and a
    // current-slice list requires an explicit, labelled policy.
    let filter_vocabulary_resolution = Memo::new(move |_| {
        if filter_options.is_some() && filter_option_entries.is_some() {
            return Err(DUPLICATE_FILTER_OPTIONS_CONFIGURATION);
        }
        resolve_server_filter_vocabulary(
            filtering_enabled,
            effective_columns.with(|columns| has_exact_filterable_columns(columns)),
            filter_options.is_some() || filter_option_entries.is_some(),
            filter_vocabulary.map(ServerFilterVocabulary::kind),
        )
    });
    let show_filter_row = Memo::new(move |_| {
        filtering_enabled
            && effective_columns.with(|columns| has_filterable_columns(columns))
            && filter_vocabulary_resolution.get().is_ok()
    });
    let effective_filter_options = Signal::derive(move || {
        let options = match filter_vocabulary_resolution.get() {
            Ok(ResolvedServerFilterVocabulary::Authoritative) => filter_options
                .map(|options| filter_options_from_strings(options.get()))
                .or_else(|| filter_option_entries.map(|options| options.get()))
                .unwrap_or_default(),
            Ok(ResolvedServerFilterVocabulary::CurrentSlice) => {
                filter_options_from_strings(rows.with(|rows| {
                    effective_columns.with(|columns| {
                        columns
                            .iter()
                            .filter(|column| column.filter_kind() == Some(ColumnFilterKind::Exact))
                            .map(|column| (column.id, distinct_values(rows, column.id)))
                            .collect::<HashMap<&'static str, Vec<String>>>()
                    })
                }))
            }
            Ok(ResolvedServerFilterVocabulary::Disabled) | Err(_) => HashMap::new(),
        };
        column_filters.with(|filters| {
            effective_columns
                .with(|columns| options_with_active_filter_values(options, filters, columns))
        })
    });
    let current_slice_filter_texts =
        filter_vocabulary.and_then(ServerFilterVocabulary::current_slice_texts);
    let effective_filter_all_label = Signal::derive(move || match current_slice_filter_texts {
        Some(current_slice) => current_slice.with(|texts| texts.all_label.clone()),
        None => texts.with(|texts| texts.filter_all.clone()),
    });
    let effective_filter_label = Signal::derive(move || match current_slice_filter_texts {
        Some(current_slice) => current_slice.with(|texts| texts.filter_label.clone()),
        None => texts.with(|texts| texts.filter_label.clone()),
    });

    // ── Viewport-fit query sizing (`viewport_fit`, ldui-2bt3) ──
    //
    // Reuses exactly the measurement math `DataTable`'s own `auto_page_size`
    // uses (`auto_page.rs`), but instead of slicing `rows` locally, PROPOSES
    // the derived page size through the same `query_state.propose` path
    // every other control (search, sort, filters, the page-size select)
    // already uses -- `TableQuery::with_page_size` resets to page one and
    // `ServerCursorQuery::with_page_size` resets to `First` by construction,
    // so cursor mode never replays a previous/next token minted for another
    // size.
    let viewport_fit_resolution: Memo<Result<bool, &'static str>> =
        Memo::new(move |_| resolve_viewport_fit(viewport_fit.get(), has_page_size));
    let viewport_fit_active =
        Signal::derive(move || matches!(viewport_fit_resolution.get(), Ok(true)));

    let table_wrapper_ref = NodeRef::<Div>::new();

    // Row-height "era" identity (ldui-89rp CRITICAL fix, reused verbatim --
    // see `auto_page.rs::RowHeightEra`'s own docs): a genuinely new server
    // page of `rows` is a fresh era (a different max row height is expected
    // and legitimate), while multiple measurement passes over the SAME
    // `rows` (density change, a resize that doesn't yet have new rows, a
    // settling layout) ratchet the row height fed to the derivation so it
    // can only grow within the era, guaranteeing the derived count reaches a
    // fixed point instead of oscillating.
    // Tracks the page size of the last `viewport_fit` proposal this table
    // sent, until the resulting `rows` change (or an unrelated one) is
    // observed and classified -- see `viewport_fit_rows_change_is_own_induced`.
    let viewport_fit_pending_proposal: StoredValue<Option<i64>> = StoredValue::new(None);
    let viewport_fit_data_revision: StoredValue<u64> = StoredValue::new(0);
    Effect::new(move |ran_before: Option<()>| {
        let _ = rows.get();
        if ran_before.is_some() {
            let accepted_page_size = query_state.get_untracked().page_size();
            let own_induced = viewport_fit_rows_change_is_own_induced(
                viewport_fit_pending_proposal.get_value(),
                accepted_page_size,
            );
            if !own_induced {
                // External change (a different query control, a declined or
                // differed proposal, a dataset swap): the row-height memory
                // from the previous era must not carry over.
                viewport_fit_data_revision
                    .update_value(|revision| *revision = revision.wrapping_add(1));
            }
            // Own-induced: deliberately do NOT bump the revision. The era
            // key (data_revision, container_width) stays the same, so the
            // next `RowHeightEra::observe` call in `measure_viewport_fit`
            // merges this pass's measured max into the SAME high-water
            // mark instead of starting over (ldui-2bt3 CRITICAL fix).
            viewport_fit_pending_proposal.set_value(None);
        }
    });
    let viewport_fit_row_era: StoredValue<Option<RowHeightEra>> = StoredValue::new(None);
    let viewport_fit_epoch: StoredValue<ViewportFitEpoch> =
        StoredValue::new(ViewportFitEpoch::new());

    let measure_viewport_fit = move |stamp: u64| {
        // Late-firing guard (ldui-d54), same rationale as `DataTable`'s own
        // measurement closure: this runs from a zero-delay macrotask, so a
        // navigation that disposes this table's reactive owner before the
        // timer fires must degrade to a no-op, not panic the whole wasm app.
        if !matches!(viewport_fit_resolution.try_get_untracked(), Some(Ok(true))) {
            return;
        }
        // Stale-measurement guard (ldui-2bt3): a newer pass may already have
        // been scheduled (and therefore have already invalidated this
        // stamp) between this timer being set and it actually firing.
        if !viewport_fit_epoch
            .try_get_value()
            .is_some_and(|epoch| epoch.is_current(stamp))
        {
            return;
        }
        let Some(wrapper) = table_wrapper_ref.try_get_untracked().flatten() else {
            return;
        };

        let measure = |selector: &str, fallback: f64| -> f64 {
            wrapper
                .query_selector(selector)
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                .map(|el| el.offset_height() as f64)
                .filter(|h| *h > 0.0)
                .unwrap_or(fallback)
        };

        // `offset_height`, deliberately, not `client_height` -- immune to a
        // horizontal scrollbar the widest rendered cell can introduce. See
        // `DataTable`'s own measurement closure for the full oscillation
        // rationale; the two must stay in sync.
        let viewport = wrapper.offset_height() as f64;
        let header_height = measure("thead", FALLBACK_HEADER_HEIGHT);
        let measured_max = wrapper
            .query_selector_all("tbody tr")
            .map(|found| {
                let heights: Vec<f64> = (0..found.length())
                    .filter_map(|i| found.item(i))
                    .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                    .map(|el| el.offset_height() as f64)
                    .collect();
                max_row_height(&heights, 0.0)
            })
            .unwrap_or(0.0);

        let era_key = (
            viewport_fit_data_revision.get_value(),
            wrapper.offset_width(),
        );
        let era = viewport_fit_row_era
            .get_value()
            .unwrap_or(RowHeightEra::empty(era_key))
            .observe(era_key, measured_max);
        viewport_fit_row_era.set_value(Some(era));
        let row_height = era.effective_row_height(FALLBACK_ROW_HEIGHT);

        let accepted_page_size = query_state.get_untracked().page_size().max(1);
        let min_rows = viewport_fit_min_rows
            .try_get_untracked()
            .unwrap_or(DEFAULT_AUTO_MIN_ROWS);
        if let Some(next_size) = viewport_fit_page_size_proposal(
            viewport,
            header_height,
            row_height,
            accepted_page_size,
            min_rows,
            accepted_page_size,
        ) {
            // Recorded BEFORE proposing: an uncontrolled offset table
            // applies the new size to its own internal query state
            // synchronously inside `propose`, so the pending value must
            // already be in place for the classification above to see it
            // whenever `rows` changes next.
            viewport_fit_pending_proposal.set_value(Some(next_size));
            query_state.propose(
                query_state.get_untracked().with_page_size(next_size),
                on_query_change,
            );
        }
    };

    // Measure on a fresh macrotask, same rationale as `DataTable`'s own
    // `schedule_measure`: a `ResizeObserver` callback can run before the
    // surrounding layout has settled. One pending measure at a time --
    // cancelling a not-yet-fired timer before scheduling a new one -- is a
    // belt-and-braces pairing with `ViewportFitEpoch`'s pure staleness
    // guard above, not a substitute for it.
    let viewport_fit_measure_handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let schedule_viewport_fit_measure = move || {
        if let Some(handle) = viewport_fit_measure_handle.try_get_value().flatten() {
            handle.clear();
        }
        let stamp = viewport_fit_epoch
            .try_update_value(|epoch| epoch.next())
            .map(|(_, stamp)| stamp)
            .unwrap_or(0);
        match set_timeout_with_handle(
            move || measure_viewport_fit(stamp),
            std::time::Duration::ZERO,
        ) {
            Ok(handle) => {
                viewport_fit_measure_handle.try_update_value(|slot| *slot = Some(handle));
            }
            // No `window` to schedule against: measuring now is better than
            // not at all.
            Err(_) => measure_viewport_fit(stamp),
        }
    };
    // A zero-delay macrotask must not outlive the reactive owner (ldui-d54).
    on_cleanup(move || {
        if let Some(handle) = viewport_fit_measure_handle.try_get_value().flatten() {
            handle.clear();
        }
    });

    // Re-measure whenever anything that moves the arithmetic changes: the
    // opt-in itself, the usability floor, table density, the rows available
    // to measure, the visible column set (widths drive wrapping), whether a
    // filter row is present, and the currently accepted page size (a
    // re-measure right after a size change corrects a height latched from
    // an unsettled layout).
    Effect::new(move |_| {
        let _ = viewport_fit_active.get();
        let _ = viewport_fit_min_rows.get();
        let _ = table_size.get();
        let _ = rows.get();
        let _ = effective_columns.get();
        let _ = show_filter_row.get();
        let _ = query_state.get().page_size();
        schedule_viewport_fit_measure();
    });

    // Attach the `ResizeObserver` once, when the wrapper first enters the
    // DOM -- identical rationale and pattern to `DataTable`'s own.
    Effect::new(move |_| {
        let Some(wrapper) = table_wrapper_ref.get() else {
            return;
        };

        schedule_viewport_fit_measure();

        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
            move |_entries: js_sys::Array, _observer: web_sys::ResizeObserver| {
                schedule_viewport_fit_measure();
            },
        )
            as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);

        match web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) {
            Ok(observer) => {
                observer.observe(wrapper.unchecked_ref::<web_sys::Element>());
                // Neither `Send` nor `Sync`; this component only ever runs
                // single-threaded (wasm32 in the browser) -- same rationale
                // as `DataTable`'s own ResizeObserver wiring.
                let guard = send_wrapper::SendWrapper::new((closure, observer));
                on_cleanup(move || {
                    let (closure, observer) = guard.take();
                    observer.disconnect();
                    drop(closure);
                });
            }
            Err(_) => drop(closure),
        }
    });

    // Container style for viewport-constrained scrolling. `viewport_fit`
    // needs a *definite* height here, not just a ceiling, for the same
    // reason `DataTable::auto_page_size` does: promoting `max_height` to a
    // real `height` breaks the circularity where the wrapper's measured
    // height would be a function of the row count derived from it.
    let has_max_height = max_height.is_some();
    let container_style = move || match (viewport_fit_active.get(), max_height.as_deref()) {
        (true, Some(h)) => Some(format!(
            "display: flex; flex-direction: column; height: {h}; max-height: {h}"
        )),
        (true, None) => Some("display: flex; flex-direction: column; height: 100%".to_string()),
        (false, Some(h)) => Some(format!(
            "display: flex; flex-direction: column; max-height: {h}"
        )),
        (false, None) => None,
    };
    let is_flex_column = move || has_max_height || viewport_fit_active.get();
    let table_wrapper_style =
        move || is_flex_column().then_some("flex: 1; overflow-y: auto; min-height: 0");
    let controls_style = move || is_flex_column().then_some("flex-shrink: 0; padding: 12px 0");
    let stable_tracks = Signal::derive(move || {
        let widths = column_widths.get();
        let columns = effective_columns.get();
        let flexible_column = columns
            .iter()
            .rev()
            .find(|column| !column.resizable)
            .map(|column| column.id);
        columns
            .into_iter()
            .map(|column| {
                let track = StableColumnTrack::new(
                    column.id,
                    stable_column_width(widths.get(column.id).copied(), column.min_width),
                );
                if flexible_column == Some(column.id) {
                    track.flexible()
                } else {
                    track
                }
            })
            .collect::<Vec<_>>()
    });
    let body_loading = Signal::derive(move || {
        let retain_rows = match pagination {
            ServerTablePagination::Cursor(cursor) => {
                cursor.page.get().state.retains_rows() && rows.with(|rows| !rows.is_empty())
            }
            ServerTablePagination::Offset(_) => false,
        };
        loading.get() && !retain_rows
    });
    let search_input_id = next_data_table_search_id();
    let page_size_input_id = format!("{search_input_id}-page-size");
    let query_ownership_marker = if cursor_pagination || controlled_offset_query {
        "controlled"
    } else {
        "uncontrolled"
    };
    let pagination_marker = if cursor_pagination {
        "cursor"
    } else {
        "offset"
    };

    view! {
        <div
            class=container_class
            node_ref=node_ref
            style=container_style
            data-table-data-mode="server-query"
            data-server-query-ownership=query_ownership_marker
            data-server-pagination-strategy=pagination_marker
            data-server-query-search=if query_capabilities.search_enabled() { "enabled" } else { "disabled" }
            data-server-query-page-size=if query_capabilities.page_size_enabled() { "enabled" } else { "disabled" }
            data-server-query-sorting=if query_capabilities.sorting_enabled() { "enabled" } else { "disabled" }
            data-server-query-filtering=if query_capabilities.filtering_enabled() { "enabled" } else { "disabled" }
            data-server-filter-vocabulary=move || match filter_vocabulary_resolution.get() {
                Ok(ResolvedServerFilterVocabulary::Disabled) => "disabled",
                Ok(ResolvedServerFilterVocabulary::Authoritative) => "authoritative",
                Ok(ResolvedServerFilterVocabulary::CurrentSlice) => "current-slice",
                Err(_) => "invalid",
            }
            data-server-viewport-fit=move || match viewport_fit_resolution.get() {
                Ok(true) => "active",
                Ok(false) => "disabled",
                Err(_) => "rejected",
            }
            aria-busy=move || loading.get().then_some("true")
        >
            <Show when=move || filter_vocabulary_resolution.get().is_err()>
                <div
                    role="alert"
                    data-server-filter-vocabulary-config-error=move || {
                        filter_vocabulary_resolution.get().err()
                    }
                    class="mb-3 rounded-box border border-error bg-error/10 px-3 py-2 text-sm text-error forced-colors:border-[CanvasText] forced-colors:text-[CanvasText]"
                >
                    {move || filter_vocabulary_resolution
                        .get()
                        .err()
                        .unwrap_or_default()}
                </div>
            </Show>
            <Show when=move || viewport_fit_resolution.get().is_err()>
                <div
                    role="alert"
                    data-server-viewport-fit-config-error=move || {
                        viewport_fit_resolution.get().err()
                    }
                    class="mb-3 rounded-box border border-error bg-error/10 px-3 py-2 text-sm text-error forced-colors:border-[CanvasText] forced-colors:text-[CanvasText]"
                >
                    {move || viewport_fit_resolution
                        .get()
                        .err()
                        .unwrap_or_default()}
                </div>
            </Show>
            {move || {
                if has_search || has_page_size {
                    let label_target = search_input_id.clone();
                    let control_id = search_input_id.clone();
                    let page_size_label_target = page_size_input_id.clone();
                    let page_size_control_id = page_size_input_id.clone();
                    Some(view! {
                        <div class="mb-3 flex min-w-0 flex-wrap items-end justify-between gap-3">
                            {has_search.then(|| view! {
                                <div class="min-w-0 flex-1">
                                    <label class="sr-only" r#for=label_target>
                                        {move || texts.with(|t| t.search_label.clone())}
                                    </label>
                                    <input
                                        node_ref=search_input
                                        id=control_id
                                        type="text"
                                        class="input input-bordered input-sm w-full max-w-xs"
                                        placeholder=move || texts.with(|t| t.search_placeholder.clone())
                                        aria-label=move || texts.with(|t| t.search_label.clone())
                                        prop:value=move || search_draft.get()
                                        on:input=on_search_input
                                    />
                                </div>
                            })}
                            {has_page_size.then(|| view! {
                                <label
                                    class="flex shrink-0 items-center gap-2 text-sm"
                                    r#for=page_size_label_target
                                >
                                    <span>{move || texts.with(|t| t.page_size_label.clone())}</span>
                                    <select
                                        node_ref=page_size_select
                                        id=page_size_control_id
                                        class="select select-bordered select-sm w-24"
                                        aria-label=move || texts.with(|t| t.page_size_label.clone())
                                        prop:value=move || query_state.get().page_size().to_string()
                                        on:change=move |event| {
                                            let Ok(next_size) = event_target_value(&event).parse::<i64>() else {
                                                return;
                                            };
                                            query_state.propose(
                                                query_state.get_untracked().with_page_size(next_size),
                                                on_query_change,
                                            );
                                            let supplied = query_state
                                                .get_untracked()
                                                .page_size()
                                                .to_string();
                                            if let Some(select) = page_size_select.get() {
                                                select.set_value(&supplied);
                                            }
                                        }
                                    >
                                        {move || page_size_options.get()
                                            .into_iter()
                                            .filter(|size| *size > 0)
                                            .map(|size| view! {
                                                <option value=size.to_string()>{size}</option>
                                            })
                                            .collect_view()}
                                    </select>
                                </label>
                            })}
                        </div>
                    })
                } else {
                    None
                }
            }}

            <div class=TABLE_SCROLL_WRAPPER_CLASS style=table_wrapper_style node_ref=table_wrapper_ref>
                <div style=move || stable_table_content_style(&stable_tracks.get())>
                    <Table
                        size=table_size
                        zebra=zebra
                        pin_rows=pin_rows
                        pin_cols=pin_cols
                        class="table-fixed w-full border-collapse border border-table-grid"
                        attr:data-table-layout="stable"
                    >
                        <StableTableColGroup tracks=stable_tracks />
                        <DataTableHeader
                            columns=effective_columns
                            sort_column=Signal::derive(move || {
                                query_state.get().sort().map(|(column, _)| column)
                            })
                            sort_order=Signal::derive(move || {
                                query_state
                                    .get()
                                    .sort()
                                    .map_or_else(SortOrder::default, |(_, order)| order)
                            })
                            sort_texts=sort_texts
                            on_sort=on_sort
                            header_cell_class=classes.header_cell
                            column_widths=column_widths
                        >
                            {move || {
                                show_filter_row.get().then(|| view! {
                                    <DataTableFilterRow
                                        columns=effective_columns
                                        options=effective_filter_options
                                        filters=column_filters
                                        on_filters_change=filter_change
                                        all_label=effective_filter_all_label
                                        filter_label=effective_filter_label
                                        text_filter_label=text_filter_label
                                    />
                                })
                            }}
                        </DataTableHeader>
                        <DataTableBody
                            columns=effective_columns
                            rows=Signal::derive(move || {
                                rows.get().into_iter().enumerate().collect::<Vec<_>>()
                            })
                            loading=body_loading
                            texts=texts
                            body_cell_class=classes.body_cell
                            row_class=classes.row
                            selected_row_class=classes.selected_row
                            selected_rows=selected_rows
                            loading_row_class=classes.loading_row
                            empty_row_class=classes.empty_row
                            cell_renderers=cell_renderers
                            column_widths=Signal::derive(move || column_widths.get())
                            typed_cells=typed_cells
                            detail_renderer=detail_renderer
                            row_class_fn=row_class_fn
                            on_row_click=Some(on_row_click)
                            on_row_inspect=body_on_row_inspect
                            row_key=row_key
                            interactive=row_interactive
                        />
                    </Table>
                </div>
            </div>

            {move || match pagination {
                ServerTablePagination::Offset(offset) => {
                    let total = offset.total_count.get();
                    let query = query_state.get();
                    let size = query.page_size().max(1);
                    let page = query.offset_page().unwrap_or(1).max(1);
                    let total_pages = if total == 0 {
                        1
                    } else {
                        ((total as f64) / (size as f64)).ceil() as i64
                    };

                    if total > 0 && !loading.get() {
                        let start = ((page - 1) * size) + 1;
                        let end = (page * size).min(total);
                        view! {
                            <div style=controls_style>
                                <ServerPaginationControls
                                    current_page=page
                                    total_pages=total_pages
                                    total_count=total
                                    start=start
                                    end=end
                                    on_page_change=page_change
                                    texts=texts.get()
                                    classes=classes.clone()
                                />
                            </div>
                        }
                        .into_any()
                    } else {
                        ().into_any()
                    }
                }
                ServerTablePagination::Cursor(cursor) => view! {
                    <div style=controls_style>
                        <ServerCursorPaginationControls
                            page=cursor.page.get()
                            row_count=rows.with(Vec::len)
                            loading=loading.get()
                            on_navigate=cursor_change
                            texts=texts.get()
                            cursor_texts=cursor.texts.get()
                            classes=classes.clone()
                        />
                    </div>
                }
                .into_any(),
            }}
        </div>
    }
    .into_any()
}

/// Cursor pagination controls with no fabricated page number or total.
#[component]
fn ServerCursorPaginationControls(
    page: ServerCursorPage,
    row_count: usize,
    loading: bool,
    on_navigate: Callback<ServerCursorRequest>,
    texts: DataTableTexts,
    cursor_texts: ServerCursorTexts,
    classes: DataTableClasses,
) -> impl IntoView {
    let state = page.state;
    let status_template = match state {
        ServerCursorSliceState::Current => cursor_texts.current,
        ServerCursorSliceState::RetainedWhileLoading => cursor_texts.retained_loading,
        ServerCursorSliceState::RetainedAfterFailure => cursor_texts.retained_failure,
    };
    let status = status_template.replace("{count}", &row_count.to_string());
    let previous = page.previous;
    let next = page.next;
    let previous_disabled = loading || previous.is_none();
    let next_disabled = loading || next.is_none();

    view! {
        <div
            class=classes.pagination
            data-server-cursor-state=match state {
                ServerCursorSliceState::Current => "current",
                ServerCursorSliceState::RetainedWhileLoading => "retained-loading",
                ServerCursorSliceState::RetainedAfterFailure => "retained-failure",
            }
        >
            <span class=classes.page_indicator role="status" aria-live="polite">
                {status}
            </span>
            <div class="join">
                <button
                    type="button"
                    class=merge_classes!(classes.pagination_button, "join-item")
                    data-server-cursor-action="previous"
                    disabled=previous_disabled
                    on:click=move |_| {
                        if !loading && let Some(cursor) = previous.clone() {
                            on_navigate.run(ServerCursorRequest::Previous(cursor));
                        }
                    }
                >
                    {texts.previous}
                </button>
                <button
                    type="button"
                    class=merge_classes!(classes.pagination_button, "join-item")
                    data-server-cursor-action="next"
                    disabled=next_disabled
                    on:click=move |_| {
                        if !loading && let Some(cursor) = next.clone() {
                            on_navigate.run(ServerCursorRequest::Next(cursor));
                        }
                    }
                >
                    {texts.next}
                </button>
            </div>
        </div>
    }
}

/// Server-side pagination controls with page numbers, ellipsis, and Showing X-Y of Z
#[component]
fn ServerPaginationControls(
    current_page: i64,
    total_pages: i64,
    total_count: i64,
    start: i64,
    end: i64,
    on_page_change: Callback<i64>,
    texts: DataTableTexts,
    classes: DataTableClasses,
) -> impl IntoView {
    // Build page number list with ellipsis
    let page_numbers = build_page_range(current_page, total_pages);

    // Extracted from the view because RSX parses an inline `>=` as a tag
    // close (same trap `DataTableControls` documents). Worse than a parse
    // error: `disabled=current_page >` truncated the attribute, demoted the
    // rest of the tag to text, and turned the Next button's click-guard block
    // into a *render-time child expression* — every render "clicked Next"
    // until the last page, silently walking a freshly mounted table to page
    // N. Unexercised until the typed-query demo made the walk visible.
    let prev_disabled = current_page <= 1;
    let next_disabled = current_page >= total_pages;

    view! {
        <div class=classes.pagination>
            <span class=classes.page_indicator>
                {texts
                    .row_range
                    .replace("{start}", &start.to_string())
                    .replace("{end}", &end.to_string())
                    .replace("{total}", &total_count.to_string())}
            </span>

            <div class="join">
                // Previous button
                <button
                    class=merge_classes!(classes.pagination_button, "join-item")
                    disabled=prev_disabled
                    on:click=move |_| {
                        if current_page > 1 {
                            on_page_change.run(current_page - 1);
                        }
                    }
                >
                    {texts.previous}
                </button>

                // Page number buttons
                {page_numbers.into_iter().map(|item| {
                    match item {
                        PageItem::Page(num) => {
                            let is_active = num == current_page;
                            let active_class = if is_active { "btn-active" } else { "" };
                            view! {
                                <button
                                    class=merge_classes!(classes.pagination_button, "join-item", active_class)
                                    disabled=is_active
                                    on:click=move |_| {
                                        on_page_change.run(num);
                                    }
                                >
                                    {num.to_string()}
                                </button>
                            }.into_any()
                        }
                        PageItem::Ellipsis => {
                            view! {
                                <button
                                    class=merge_classes!(classes.pagination_button, "join-item btn-ghost")
                                    disabled=true
                                >
                                    "..."
                                </button>
                            }.into_any()
                        }
                    }
                }).collect_view()}

                // Next button
                <button
                    class=merge_classes!(classes.pagination_button, "join-item")
                    disabled=next_disabled
                    on:click=move |_| {
                        if current_page < total_pages {
                            on_page_change.run(current_page + 1);
                        }
                    }
                >
                    {texts.next}
                </button>
            </div>
        </div>
    }
}

/// Represents a page number or ellipsis in pagination
#[derive(Clone, Copy, Debug, PartialEq)]
enum PageItem {
    Page(i64),
    Ellipsis,
}

/// Build a page range with ellipsis for large ranges.
///
/// Shows: first page, last page, current page, and 1 page on each side of current.
/// Ellipsis fills gaps larger than 1 page.
///
/// Examples:
/// - 7 pages, current=4: [1, 2, 3, 4, 5, 6, 7]
/// - 20 pages, current=1: [1, 2, ..., 20]
/// - 20 pages, current=10: [1, ..., 9, 10, 11, ..., 20]
/// - 20 pages, current=20: [1, ..., 19, 20]
fn build_page_range(current: i64, total: i64) -> Vec<PageItem> {
    if total <= 7 {
        return (1..=total).map(PageItem::Page).collect();
    }

    let mut pages: Vec<i64> = Vec::new();

    // Always include first and last
    pages.push(1);
    pages.push(total);

    // Include current and neighbors
    for p in (current - 1)..=(current + 1) {
        if p >= 1 && p <= total {
            pages.push(p);
        }
    }

    // Sort and deduplicate
    pages.sort();
    pages.dedup();

    // Build result with ellipsis
    let mut result = Vec::new();
    let mut prev = 0_i64;
    for &p in &pages {
        if prev > 0 && p - prev > 1 {
            result.push(PageItem::Ellipsis);
        }
        result.push(PageItem::Page(p));
        prev = p;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::reactive::owner::Owner;
    use std::sync::{Arc, Mutex};

    fn identified_row(id: &str) -> TableRow {
        HashMap::from([("id", id.to_owned())])
    }

    #[test]
    fn controlled_selection_maps_only_the_accepted_business_key() {
        let first = vec![identified_row("desk-1"), identified_row("desk-2")];
        let replacement = vec![identified_row("desk-3"), identified_row("desk-1")];

        assert_eq!(
            selected_server_row_indices(&first, Some("desk-1"), |row| row["id"].clone()),
            BTreeSet::from([0])
        );
        assert_eq!(
            selected_server_row_indices(&replacement, Some("desk-1"), |row| row["id"].clone()),
            BTreeSet::from([1]),
            "selection follows the accepted key rather than page position"
        );
        assert!(
            selected_server_row_indices(&replacement, Some("desk-missing"), |row| row["id"]
                .clone())
            .is_empty(),
            "an absent key must not select the replacement at its old index"
        );
    }

    #[test]
    fn controlled_single_selection_ignores_modified_gestures() {
        assert_eq!(
            server_selection_proposal("desk-1", false, false),
            Some("desk-1".to_owned())
        );
        assert_eq!(server_selection_proposal("desk-1", true, false), None);
        assert_eq!(server_selection_proposal("desk-1", false, true), None);
        assert_eq!(server_selection_proposal("desk-1", true, true), None);
    }

    #[test]
    fn server_filter_vocabulary_requires_an_explicit_truthful_scope() {
        assert_eq!(
            resolve_server_filter_vocabulary(true, true, false, None),
            Err(MISSING_FILTER_VOCABULARY_CONFIGURATION)
        );
        assert_eq!(
            resolve_server_filter_vocabulary(
                true,
                true,
                false,
                Some(ServerFilterVocabularyKind::CurrentSlice),
            ),
            Ok(ResolvedServerFilterVocabulary::CurrentSlice)
        );
        assert_eq!(
            resolve_server_filter_vocabulary(true, true, true, None),
            Ok(ResolvedServerFilterVocabulary::Authoritative)
        );
        assert_eq!(
            resolve_server_filter_vocabulary(
                true,
                true,
                true,
                Some(ServerFilterVocabularyKind::CurrentSlice),
            ),
            Err(CONFLICTING_FILTER_VOCABULARY_CONFIGURATION)
        );
        assert_eq!(
            resolve_server_filter_vocabulary(false, true, false, None),
            Ok(ResolvedServerFilterVocabulary::Disabled),
            "a disabled filtering capability needs no vocabulary"
        );
        assert_eq!(
            resolve_server_filter_vocabulary(true, false, false, None),
            Ok(ResolvedServerFilterVocabulary::Disabled),
            "a contains-only filter row has no finite exact-value vocabulary to declare"
        );
    }

    #[test]
    fn active_server_filter_value_survives_vocabulary_refresh() {
        let options =
            filter_options_from_strings(HashMap::from([("role", vec!["Admin".to_owned()])]));
        let filters = ColumnFilters::from([("role", "Analyst".to_owned())]);
        let columns = vec![Column::new("role", "Role").filterable()];

        let preserved = options_with_active_filter_values(options, &filters, &columns);

        assert_eq!(
            preserved["role"],
            vec![
                DataTableFilterOption::same("Admin"),
                DataTableFilterOption::same("Analyst"),
            ]
        );
    }

    #[test]
    fn active_contains_values_are_not_misrepresented_as_exact_options() {
        let filters = ColumnFilters::from([("job", "   ".to_owned())]);
        let columns = vec![Column::new("job", "Job").filterable_text()];

        assert!(
            options_with_active_filter_values(DataTableFilterOptions::new(), &filters, &columns)
                .is_empty()
        );
    }

    // ── viewport_fit (ldui-2bt3) ──

    #[test]
    fn viewport_fit_is_off_by_default_regardless_of_capability() {
        assert_eq!(resolve_viewport_fit(false, true), Ok(false));
        assert_eq!(resolve_viewport_fit(false, false), Ok(false));
    }

    #[test]
    fn viewport_fit_activates_only_when_page_size_is_controllable() {
        assert_eq!(resolve_viewport_fit(true, true), Ok(true));
    }

    #[test]
    fn viewport_fit_fails_closed_on_a_fixed_slice_or_disabled_capability() {
        // Covers both motivating cases the same way: a fixed-slice cursor
        // endpoint and `ServerQueryCapabilities::page_size_enabled() ==
        // false` both collapse to `page_size_controllable = false` before
        // this function ever sees them.
        assert_eq!(
            resolve_viewport_fit(true, false),
            Err(VIEWPORT_FIT_REQUIRES_PAGE_SIZE_CONFIGURATION),
        );
    }

    #[test]
    fn viewport_fit_proposal_grows_the_page_size_to_the_measured_fit() {
        // 436px viewport, 36px header, 40px rows -> 10 rows fit; accepted is
        // still 5, so a growth proposal is due.
        assert_eq!(
            viewport_fit_page_size_proposal(436.0, 36.0, 40.0, 5, 5, 5),
            Some(10),
        );
    }

    #[test]
    fn viewport_fit_proposal_is_none_once_it_matches_what_is_already_accepted() {
        assert_eq!(
            viewport_fit_page_size_proposal(436.0, 36.0, 40.0, 10, 5, 10),
            None,
            "the caller already accepted the derived size -- nothing to propose",
        );
    }

    #[test]
    fn viewport_fit_proposal_below_the_usability_floor_retains_the_accepted_size() {
        // A 224px wrapper, 77px header, 76px rows mathematically fit one
        // row (below `min_rows`), so the derivation retains whatever is
        // already accepted (12) rather than proposing 1.
        assert_eq!(
            viewport_fit_page_size_proposal(224.0, 77.0, 76.0, 12, 5, 12),
            None,
            "a fit below min_rows must retain the accepted size, not propose a smaller one",
        );
    }

    #[test]
    fn viewport_fit_proposal_is_stable_under_repeated_measurement_of_an_unchanged_viewport() {
        // Simulates a caller that declines a proposal: the accepted size
        // never moves, so every subsequent re-measurement of the SAME
        // viewport must derive the SAME proposal -- no drift, no escalation.
        let first = viewport_fit_page_size_proposal(436.0, 36.0, 40.0, 5, 5, 5);
        let second = viewport_fit_page_size_proposal(436.0, 36.0, 40.0, 5, 5, 5);
        assert_eq!(first, second);
        assert_eq!(first, Some(10));
    }

    #[test]
    fn viewport_fit_proposal_applied_to_an_offset_query_resets_to_page_one() {
        let accepted = TableQuery {
            page: 7,
            page_size: 5,
            search: String::new(),
            sort: None,
            filters: ColumnFilters::new(),
        };
        let Some(next_size) = viewport_fit_page_size_proposal(
            436.0,
            36.0,
            40.0,
            accepted.page_size,
            5,
            accepted.page_size,
        ) else {
            panic!("expected a growth proposal");
        };

        let proposed = accepted.with_page_size(next_size);

        assert_eq!(proposed.page_size, 10);
        assert_eq!(
            proposed.page, 1,
            "offset viewport-fit proposals reset to page one"
        );
    }

    #[test]
    fn viewport_fit_proposal_applied_to_a_cursor_query_resets_to_first_never_reusing_a_token() {
        // A token minted for `page_size: 5` must never be replayed against a
        // proposed `page_size: 10`.
        let accepted = ServerCursorQuery {
            request: ServerCursorRequest::Next(ServerCursorToken::new("minted-for-size-5")),
            page_size: 5,
            search: String::new(),
            sort: None,
            filters: ColumnFilters::new(),
        };
        let Some(next_size) = viewport_fit_page_size_proposal(
            436.0,
            36.0,
            40.0,
            accepted.page_size,
            5,
            accepted.page_size,
        ) else {
            panic!("expected a growth proposal");
        };

        let proposed = accepted.with_page_size(next_size);

        assert_eq!(proposed.page_size, 10);
        assert_eq!(
            proposed.request,
            ServerCursorRequest::First,
            "cursor viewport-fit proposals must request First, never replay an old token",
        );
    }

    #[test]
    fn viewport_fit_epoch_stamps_are_sequential_and_current() {
        let epoch = ViewportFitEpoch::new();
        let (epoch, first) = epoch.next();
        assert!(epoch.is_current(first));

        let (epoch, second) = epoch.next();
        assert_ne!(first, second);
        assert!(epoch.is_current(second));
    }

    #[test]
    fn viewport_fit_epoch_rejects_a_stale_stamp_once_a_newer_pass_is_scheduled() {
        let epoch = ViewportFitEpoch::new();
        let (epoch, stale) = epoch.next();
        // Scheduling a second pass immediately invalidates the first, even
        // though the first pass's timer has not fired yet.
        let (epoch, fresh) = epoch.next();

        assert!(
            !epoch.is_current(stale),
            "a stamp superseded by a newer schedule must never apply its measurement"
        );
        assert!(epoch.is_current(fresh));
    }

    #[test]
    fn viewport_fit_epoch_a_pass_that_fires_out_of_order_is_still_rejected_if_stale() {
        // Three passes scheduled back to back (e.g. rapid resize); only the
        // last-scheduled stamp may ever apply, regardless of firing order.
        let epoch = ViewportFitEpoch::new();
        let (epoch, first) = epoch.next();
        let (epoch, second) = epoch.next();
        let (epoch, third) = epoch.next();

        assert!(!epoch.is_current(first));
        assert!(!epoch.is_current(second));
        assert!(epoch.is_current(third));
    }

    // ── own-induced refetch era carry-forward (ldui-2bt3 CRITICAL fix) ──

    #[test]
    fn viewport_fit_own_induced_change_is_detected_by_matching_pending_to_accepted() {
        assert!(viewport_fit_rows_change_is_own_induced(Some(10), 10));
        assert!(
            !viewport_fit_rows_change_is_own_induced(None, 10),
            "no proposal was pending -- an unrelated rows change"
        );
        assert!(
            !viewport_fit_rows_change_is_own_induced(Some(10), 5),
            "the accepted size differs from what was proposed -- declined or superseded"
        );
    }

    #[test]
    fn viewport_fit_own_induced_refetch_carries_the_era_forward_and_terminates() {
        // Reviewer trace (ldui-2bt3 CRITICAL fix): a proposal's own-induced
        // refetch was previously treated as a brand-new `RowHeightEra`,
        // which discarded the tall-row high-water mark the ratchet needs to
        // converge -- accepted=5 fits 10 short rows, propose 10; the
        // refetched 10-row page reveals a tall row at index 8, but a fresh
        // era forgets it, deriving 5; propose 5; the refetched 5-row page
        // is short again (index 8 is outside a 5-row page), a fresh era
        // forgets the tall reading yet again, deriving 10; propose 10 --
        // forever. Carrying the era forward across an own-induced refetch
        // (never resetting the high-water mark just because `rows` changed
        // AS A RESULT of accepting the table's own proposal) terminates
        // the cycle.
        const SHORT: f64 = 40.0;
        const TALL: f64 = 80.0;
        const TALL_ROW_INDEX: usize = 8; // absolute index in the dataset
        const VIEWPORT: f64 = 400.0;
        const HEADER: f64 = 0.0;
        const MIN_ROWS: usize = 5;
        const CONTAINER_WIDTH: i32 = 400;

        // What the server would actually send for a page of `size` rows
        // starting at 0, given one tall row sitting at `TALL_ROW_INDEX`.
        let measured_max_for_page =
            |size: usize| -> f64 { if size > TALL_ROW_INDEX { TALL } else { SHORT } };

        let mut accepted: i64 = 5;
        let mut pending: Option<i64> = None;
        let key = (0_u64, CONTAINER_WIDTH);
        let mut era = RowHeightEra::empty(key);
        let mut proposals = Vec::new();

        for _round in 0..6 {
            if !proposals.is_empty() {
                // A `rows` change happened: classify it exactly the way the
                // component's Effect does.
                let own_induced = viewport_fit_rows_change_is_own_induced(pending, accepted);
                assert!(
                    own_induced,
                    "every refetch in this trace is caused by accepting the table's own \
                     proposal -- an external change is a different scenario"
                );
                pending = None;
                // Own-induced: `key` is deliberately NOT changed, so the
                // `observe` call below merges into the SAME high-water mark
                // instead of starting a fresh one -- the fix under test.
            }

            let measured_max = measured_max_for_page(accepted.max(0) as usize);
            era = era.observe(key, measured_max);
            let row_height = era.effective_row_height(FALLBACK_ROW_HEIGHT);

            match viewport_fit_page_size_proposal(
                VIEWPORT, HEADER, row_height, accepted, MIN_ROWS, accepted,
            ) {
                Some(next_size) => {
                    proposals.push(next_size);
                    pending = Some(next_size);
                    accepted = next_size; // the caller accepts and refetches
                }
                None => break,
            }
        }

        assert_eq!(
            proposals,
            vec![10, 5],
            "expected exactly the reviewer's two-step trace before settling: {proposals:?}"
        );
        assert_eq!(
            accepted, 5,
            "the trace must settle back at the smaller, tall-row-aware size"
        );
        assert!(
            pending.is_none(),
            "no further proposal is pending once the cycle terminates"
        );
    }

    #[test]
    fn viewport_fit_without_the_own_induced_carry_forward_the_same_trace_never_settles() {
        // Contrast for the fix above: if every `rows` change started a
        // fresh era (the pre-fix behavior -- no own-induced distinction),
        // the identical trace cycles between 10 and 5 forever instead of
        // terminating. This pins the bug the fix addresses.
        const SHORT: f64 = 40.0;
        const TALL: f64 = 80.0;
        const TALL_ROW_INDEX: usize = 8;
        const VIEWPORT: f64 = 400.0;
        const HEADER: f64 = 0.0;
        const MIN_ROWS: usize = 5;

        let measured_max_for_page =
            |size: usize| -> f64 { if size > TALL_ROW_INDEX { TALL } else { SHORT } };

        let mut accepted: i64 = 5;
        let mut proposals = Vec::new();

        for round in 0_u64..6 {
            let measured_max = measured_max_for_page(accepted.max(0) as usize);
            // Always a fresh era (the pre-fix behavior): a distinct key
            // every round means `observe` can never ratchet across rounds.
            let era = RowHeightEra::empty((round, 400)).observe((round, 400), measured_max);
            let row_height = era.effective_row_height(FALLBACK_ROW_HEIGHT);

            match viewport_fit_page_size_proposal(
                VIEWPORT, HEADER, row_height, accepted, MIN_ROWS, accepted,
            ) {
                Some(next) => {
                    proposals.push(next);
                    accepted = next;
                }
                None => break,
            }
        }

        assert_eq!(
            proposals,
            vec![10, 5, 10, 5, 10, 5],
            "without the own-induced carry-forward the trace oscillates instead of \
             settling: {proposals:?}"
        );
    }

    #[test]
    fn server_query_capabilities_default_to_the_compatible_full_contract() {
        let capabilities = ServerQueryCapabilities::default();

        assert!(capabilities.search_enabled());
        assert!(capabilities.page_size_enabled());
        assert!(capabilities.sorting_enabled());
        assert!(capabilities.filtering_enabled());
    }

    #[test]
    fn server_query_capabilities_support_navigation_only_and_mixed_contracts() {
        let navigation_only = ServerQueryCapabilities::navigation_only();
        assert!(!navigation_only.search_enabled());
        assert!(!navigation_only.page_size_enabled());
        assert!(!navigation_only.sorting_enabled());
        assert!(!navigation_only.filtering_enabled());

        let mixed = navigation_only
            .with_search(true)
            .with_sorting(true)
            .with_filtering(true);
        assert!(mixed.search_enabled());
        assert!(!mixed.page_size_enabled());
        assert!(mixed.sorting_enabled());
        assert!(mixed.filtering_enabled());
    }

    #[test]
    fn disabled_query_capabilities_reject_conflicting_supplied_truth() {
        let mut filters = ColumnFilters::new();
        filters.insert("status", "Open".to_owned());
        let all_disabled = ServerQueryCapabilities::navigation_only();

        assert_eq!(
            validate_server_query_capabilities(
                all_disabled,
                &ServerQuerySnapshot::Cursor(
                    ServerCursorQuery::first_slice(20).with_search("matter"),
                ),
                false,
            ),
            Err(DISABLED_SEARCH_CONFIGURATION),
        );
        assert_eq!(
            validate_server_query_capabilities(
                all_disabled,
                &ServerQuerySnapshot::Cursor(
                    ServerCursorQuery::first_slice(20).with_sort(Some(("name", SortOrder::Asc))),
                ),
                false,
            ),
            Err(DISABLED_SORT_CONFIGURATION),
        );
        assert_eq!(
            validate_server_query_capabilities(
                all_disabled,
                &ServerQuerySnapshot::Cursor(
                    ServerCursorQuery::first_slice(20).with_filters(filters),
                ),
                false,
            ),
            Err(DISABLED_FILTER_CONFIGURATION),
        );
        assert_eq!(
            validate_server_query_capabilities(
                all_disabled,
                &ServerQuerySnapshot::Cursor(ServerCursorQuery::first_slice(20)),
                true,
            ),
            Err(DISABLED_SEARCH_CALLBACK_CONFIGURATION),
        );
    }

    #[test]
    fn cursor_query_shape_transitions_restart_from_the_first_slice() {
        let cursor = ServerCursorToken::new("opaque-next");
        let mut filters = ColumnFilters::new();
        filters.insert("status", "Open".to_owned());
        let query = ServerCursorQuery {
            request: ServerCursorRequest::Next(cursor),
            page_size: 25,
            search: String::new(),
            sort: None,
            filters: ColumnFilters::new(),
        };

        assert_eq!(
            query.clone().with_search("matter").request,
            ServerCursorRequest::First
        );
        assert_eq!(
            query
                .clone()
                .with_sort(Some(("name", SortOrder::Desc)))
                .request,
            ServerCursorRequest::First
        );
        assert_eq!(
            query.clone().with_filters(filters).request,
            ServerCursorRequest::First
        );
        assert_eq!(query.with_page_size(50).request, ServerCursorRequest::First);
    }

    #[test]
    fn cursor_state_proposes_one_opaque_navigation_replacement() {
        let owner = Owner::new();
        owner.with(|| {
            let supplied = RwSignal::new(ServerCursorQuery::first_slice(10));
            let proposals = Arc::new(Mutex::new(Vec::<ServerCursorQuery>::new()));
            let observed = Arc::clone(&proposals);
            let state = ServerQueryState::new_cursor(
                supplied.into(),
                Callback::new(move |query| observed.lock().unwrap().push(query)),
            );
            let request = ServerCursorRequest::Next(ServerCursorToken::new("opaque-next"));

            state.propose(
                state.get_untracked().with_cursor_request(request.clone()),
                None,
            );

            assert_eq!(
                state.get_untracked().cursor_request(),
                Some(ServerCursorRequest::First)
            );
            assert_eq!(proposals.lock().unwrap().len(), 1);
            assert_eq!(proposals.lock().unwrap()[0].request, request);
        });
    }

    #[test]
    fn explicit_pagination_rejects_legacy_offset_props() {
        let owner = Owner::new();
        owner.with(|| {
            let query = RwSignal::new(ServerCursorQuery::first_slice(10));
            let page = RwSignal::new(ServerCursorPage::new(None, None));
            let pagination = ServerTablePagination::cursor(ServerCursorPagination::controlled(
                query.into(),
                page.into(),
                Callback::new(|_| {}),
            ));

            assert!(matches!(
                resolve_server_pagination(
                    Some(pagination),
                    Some(Signal::stored(1_i64)),
                    Some(Signal::stored(20_i64)),
                    Some(Signal::stored(10_i64)),
                    Some(Callback::new(|_| {})),
                ),
                Err(MIXED_PAGINATION_CONFIGURATION)
            ));
        });
    }

    #[test]
    fn controlled_query_state_proposes_once_without_overwriting_supplied_truth() {
        let owner = Owner::new();
        owner.with(|| {
            let supplied = RwSignal::new(TableQuery {
                page: 4,
                page_size: 25,
                search: "accepted".to_owned(),
                sort: None,
                filters: ColumnFilters::new(),
            });
            let proposals = Arc::new(Mutex::new(Vec::<TableQuery>::new()));
            let observed = Arc::clone(&proposals);
            let state = ServerQueryState::new_offset(
                Some(ServerTableQueryOwnership::controlled(
                    supplied.into(),
                    Callback::new(move |query| observed.lock().unwrap().push(query)),
                )),
                supplied.get_untracked(),
            );
            let proposed = supplied.get_untracked().with_search("declined");

            state.propose(ServerQuerySnapshot::Offset(proposed.clone()), None);

            assert_eq!(
                state.get_untracked(),
                ServerQuerySnapshot::Offset(supplied.get_untracked())
            );
            assert_eq!(proposals.lock().unwrap().as_slice(), [proposed]);
        });
    }

    #[test]
    fn uncontrolled_query_state_updates_and_mirrors_the_legacy_callback_once() {
        let owner = Owner::new();
        owner.with(|| {
            let initial = TableQuery::first_page(10);
            let proposals = Arc::new(Mutex::new(Vec::<TableQuery>::new()));
            let observed = Arc::clone(&proposals);
            let state = ServerQueryState::new_offset(
                Some(ServerTableQueryOwnership::uncontrolled()),
                initial.clone(),
            );
            let proposed = initial.with_page_size(50);

            state.propose(
                ServerQuerySnapshot::Offset(proposed.clone()),
                Some(Callback::new(move |query| {
                    observed.lock().unwrap().push(query)
                })),
            );

            assert_eq!(
                state.get_untracked(),
                ServerQuerySnapshot::Offset(proposed.clone())
            );
            assert_eq!(proposals.lock().unwrap().as_slice(), [proposed]);
        });
    }

    #[test]
    fn every_query_shape_transition_restarts_offset_paging() {
        let mut filters = ColumnFilters::new();
        filters.insert("status", "Open".to_owned());
        let query = TableQuery {
            page: 7,
            page_size: 25,
            search: String::new(),
            sort: None,
            filters: ColumnFilters::new(),
        };

        assert_eq!(query.clone().with_search("matter").page, 1);
        assert_eq!(
            query
                .clone()
                .with_sort(Some(("name", SortOrder::Desc)))
                .page,
            1
        );
        assert_eq!(query.clone().with_filters(filters).page, 1);
        assert_eq!(query.with_page_size(50).page, 1);
    }

    /// Helper: extract page numbers from PageItem vec, using -1 for Ellipsis
    fn to_nums(items: &[PageItem]) -> Vec<i64> {
        items
            .iter()
            .map(|item| match item {
                PageItem::Page(n) => *n,
                PageItem::Ellipsis => -1,
            })
            .collect()
    }

    // ── Single page ──

    #[test]
    fn one_total_page() {
        let result = build_page_range(1, 1);
        assert_eq!(to_nums(&result), vec![1]);
    }

    // ── Small ranges (total <= 7, no ellipsis) ──

    #[test]
    fn five_pages_current_1() {
        let result = build_page_range(1, 5);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn five_pages_current_3() {
        let result = build_page_range(3, 5);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn five_pages_current_5() {
        let result = build_page_range(5, 5);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn seven_pages_all_shown() {
        let result = build_page_range(4, 7);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, 5, 6, 7]);
    }

    // ── Large ranges with ellipsis (total > 7) ──

    #[test]
    fn ten_pages_current_1() {
        // current=1, neighbors=[1,2], always first=1 last=10
        // pages: [1, 2, 10] -> [1, 2, ..., 10]
        let result = build_page_range(1, 10);
        assert_eq!(to_nums(&result), vec![1, 2, -1, 10]);
    }

    #[test]
    fn ten_pages_current_5() {
        // current=5, neighbors=[4,5,6], always first=1 last=10
        // pages: [1, 4, 5, 6, 10] -> [1, ..., 4, 5, 6, ..., 10]
        let result = build_page_range(5, 10);
        assert_eq!(to_nums(&result), vec![1, -1, 4, 5, 6, -1, 10]);
    }

    #[test]
    fn ten_pages_current_10() {
        // current=10, neighbors=[9,10], always first=1 last=10
        // pages: [1, 9, 10] -> [1, ..., 9, 10]
        let result = build_page_range(10, 10);
        assert_eq!(to_nums(&result), vec![1, -1, 9, 10]);
    }

    #[test]
    fn ten_pages_current_2() {
        // current=2, neighbors=[1,2,3], always first=1 last=10
        // pages: [1, 2, 3, 10] -> [1, 2, 3, ..., 10]
        let result = build_page_range(2, 10);
        assert_eq!(to_nums(&result), vec![1, 2, 3, -1, 10]);
    }

    #[test]
    fn ten_pages_current_9() {
        // current=9, neighbors=[8,9,10], always first=1 last=10
        // pages: [1, 8, 9, 10] -> [1, ..., 8, 9, 10]
        let result = build_page_range(9, 10);
        assert_eq!(to_nums(&result), vec![1, -1, 8, 9, 10]);
    }

    #[test]
    fn twenty_pages_current_10() {
        // current=10, neighbors=[9,10,11], always first=1 last=20
        // pages: [1, 9, 10, 11, 20] -> [1, ..., 9, 10, 11, ..., 20]
        let result = build_page_range(10, 20);
        assert_eq!(to_nums(&result), vec![1, -1, 9, 10, 11, -1, 20]);
    }

    // ── Edge: current near first or last with gap of exactly 1 ──

    #[test]
    fn ten_pages_current_3_no_left_ellipsis() {
        // current=3, neighbors=[2,3,4], always first=1 last=10
        // pages: [1, 2, 3, 4, 10] -> [1, 2, 3, 4, ..., 10]
        let result = build_page_range(3, 10);
        assert_eq!(to_nums(&result), vec![1, 2, 3, 4, -1, 10]);
    }

    #[test]
    fn ten_pages_current_8_no_right_ellipsis() {
        // current=8, neighbors=[7,8,9], always first=1 last=10
        // pages: [1, 7, 8, 9, 10] -> [1, ..., 7, 8, 9, 10]
        let result = build_page_range(8, 10);
        assert_eq!(to_nums(&result), vec![1, -1, 7, 8, 9, 10]);
    }

    // ── Two pages ──

    #[test]
    fn two_pages_current_1() {
        let result = build_page_range(1, 2);
        assert_eq!(to_nums(&result), vec![1, 2]);
    }

    #[test]
    fn two_pages_current_2() {
        let result = build_page_range(2, 2);
        assert_eq!(to_nums(&result), vec![1, 2]);
    }

    // ── PageItem PartialEq ──

    #[test]
    fn page_item_page_equality() {
        assert_eq!(PageItem::Page(5), PageItem::Page(5));
        assert_ne!(PageItem::Page(5), PageItem::Page(6));
    }

    #[test]
    fn page_item_ellipsis_equality() {
        assert_eq!(PageItem::Ellipsis, PageItem::Ellipsis);
    }

    #[test]
    fn page_item_page_ne_ellipsis() {
        assert_ne!(PageItem::Page(1), PageItem::Ellipsis);
    }
}
