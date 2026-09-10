use super::{
    CellRenderer, Column, DataTableClasses, DataTableFilterOptions, DataTableSortTexts,
    DataTableTexts, RowDetailRenderer, ServerDataTable, ServerDataTableProps,
    ServerQueryCapabilities, ServerTableColumnTools, ServerTableColumnToolsTexts,
    ServerTableDisplayedSlice, ServerTableMultiSelection, ServerTablePageSizePreference,
    ServerTableQueryOwnership, ServerTableRowAction, ServerTableSelection, TableQuery, TableRow,
    TypedCellFn,
};
use crate::components::data_table::server_column_tools::ServerColumnToolsState;
use crate::components::entity_table::{EntityColumnChooserTrigger, EntityTablePreferenceOwnership};
use crate::components::table::TableSize;
use leptos::prelude::*;
use std::collections::HashMap;
use std::fmt;

/// A configuration error that prevents [`ServerEntityTable`] from rendering
/// an incomplete version of the standard server table experience.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServerEntityTableConfigurationError {
    /// One declared domain column has no exact or text filter control.
    MissingFilter {
        /// Stable id of the column that lacks a filter declaration.
        column_id: &'static str,
    },
    /// The endpoint disabled the canonical per-column filtering controls.
    FilteringDisabled,
    /// The endpoint disabled the canonical rows-per-page control.
    PageSizeDisabled,
}

impl ServerEntityTableConfigurationError {
    fn code(self) -> &'static str {
        match self {
            Self::MissingFilter { column_id } => column_id,
            Self::FilteringDisabled => "filtering-disabled",
            Self::PageSizeDisabled => "page-size-disabled",
        }
    }
}

impl fmt::Display for ServerEntityTableConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFilter { column_id } => write!(
                formatter,
                "ServerEntityTable column `{column_id}` must declare an exact or text filter"
            ),
            Self::FilteringDisabled => formatter.write_str(
                "ServerEntityTable requires endpoint filtering support; use ServerDataTable for a partial composition",
            ),
            Self::PageSizeDisabled => formatter.write_str(
                "ServerEntityTable requires endpoint page-size support; use ServerDataTable for a partial composition",
            ),
        }
    }
}

fn validate_server_entity_capabilities(
    capabilities: ServerQueryCapabilities,
) -> Result<(), ServerEntityTableConfigurationError> {
    if !capabilities.filtering_enabled() {
        return Err(ServerEntityTableConfigurationError::FilteringDisabled);
    }
    if !capabilities.page_size_enabled() {
        return Err(ServerEntityTableConfigurationError::PageSizeDisabled);
    }
    Ok(())
}

fn validate_server_entity_configuration(
    columns: &[Column],
    capabilities: ServerQueryCapabilities,
) -> Result<(), ServerEntityTableConfigurationError> {
    validate_server_entity_capabilities(capabilities)?;
    validate_server_entity_columns(columns)
}

impl std::error::Error for ServerEntityTableConfigurationError {}

/// Checks that every domain column participates in the canonical server
/// table's one-control-per-column filtering contract.
pub fn validate_server_entity_columns(
    columns: &[Column],
) -> Result<(), ServerEntityTableConfigurationError> {
    columns
        .iter()
        .find(|column| column.filter_kind().is_none())
        .map_or(Ok(()), |column| {
            Err(ServerEntityTableConfigurationError::MissingFilter {
                column_id: column.id,
            })
        })
}

/// The canonical opinionated table for an accepted, offset-paged server
/// population.
///
/// The caller owns the accepted [`TableQuery`], current page of rows, total,
/// request lifecycle, and preference persistence. Gestures emit complete
/// query or preference replacements; this facade never sorts, filters, pages,
/// or fetches rows locally. Cursor-paged and deliberately partial table
/// compositions should use [`ServerDataTable`] directly.
#[component]
pub fn ServerEntityTable(
    /// Accepted rows for the current server page.
    #[prop(into)]
    rows: Signal<Vec<TableRow>>,
    /// Complete domain column declarations. Every column must be filterable.
    #[prop(into)]
    columns: Signal<Vec<Column>>,
    /// Accepted offset query represented by the displayed rows.
    #[prop(into)]
    query: Signal<TableQuery>,
    /// Authoritative result count for the accepted query.
    #[prop(into)]
    total_count: Signal<i64>,
    /// Receives full query replacement proposals.
    on_query_change: Callback<TableQuery>,
    /// Whether the host is loading a replacement page.
    #[prop(into)]
    loading: Signal<bool>,
    /// Stable prefix for every framework-owned control.
    control_id: String,
    /// Explicit ownership for column visibility, order, and widths.
    preference_ownership: EntityTablePreferenceOwnership,
    /// Schema version used to normalize incompatible preference payloads.
    preference_version: u16,
    /// Host-retainable Auto/fixed page-size intent.
    page_size_preference: RwSignal<ServerTablePageSizePreference>,
    /// Query operations accepted by the endpoint.
    query_capabilities: ServerQueryCapabilities,
    /// Whether the host currently has definite geometry for Auto sizing.
    #[prop(into)]
    viewport_fit: Signal<bool>,
    /// Authoritative string option lists for exact-filter columns.
    #[prop(optional, into)]
    filter_options: Option<Signal<HashMap<&'static str, Vec<String>>>>,
    /// Authoritative typed option entries for exact-filter columns.
    #[prop(optional, into)]
    filter_option_entries: Option<Signal<DataTableFilterOptions>>,
    /// Localized table chrome.
    #[prop(into, default = Signal::stored(DataTableTexts::default()))]
    texts: Signal<DataTableTexts>,
    /// Localized sortable-header accessible names.
    #[prop(into, default = Signal::stored(DataTableSortTexts::default()))]
    sort_texts: Signal<DataTableSortTexts>,
    /// Localized accessible-name template for text filters.
    #[prop(into, default = Signal::stored("Filter {column} by text".to_owned()))]
    text_filter_label: Signal<String>,
    /// Localized Auto page-size caption.
    #[prop(into, default = Signal::stored("Auto ({rows})".to_owned()))]
    page_size_auto_label: Signal<String>,
    /// Choices offered by the page-size control.
    #[prop(
        into,
        default = Signal::stored(vec![10_i64, 25_i64, 50_i64, 100_i64])
    )]
    page_size_options: Signal<Vec<i64>>,
    /// Localized column chooser copy.
    #[prop(into, default = Signal::stored(ServerTableColumnToolsTexts::default()))]
    column_tools_texts: Signal<ServerTableColumnToolsTexts>,
    /// Minimum usable row count for Auto sizing. The canonical fill-parent
    /// facade defaults to one usable row.
    #[prop(into, default = Signal::stored(1_usize))]
    viewport_fit_min_rows: Signal<usize>,
    /// Per-cell custom renderers referenced by column renderer indices.
    #[prop(optional)]
    cell_renderers: Vec<CellRenderer>,
    /// Lightweight typed-cell renderers referenced by column indices.
    #[prop(optional)]
    typed_cells: Vec<TypedCellFn>,
    /// Optional full-width detail renderer.
    #[prop(optional)]
    detail_renderer: Option<RowDetailRenderer>,
    /// Optional per-row presentation classes.
    #[prop(optional)]
    row_class_fn: Option<Callback<(usize, TableRow), String>>,
    /// Stable business key for row identity and keyed callbacks.
    #[prop(optional, into)]
    row_key: Option<Callback<TableRow, String>>,
    /// Optional controlled single selection.
    #[prop(optional)]
    selection: Option<ServerTableSelection>,
    /// Optional controlled multi-selection.
    #[prop(optional)]
    multi_selection: Option<ServerTableMultiSelection>,
    /// Compatibility activation callback using the page-local row index.
    #[prop(optional, into)]
    on_row_activate: Option<Callback<usize>>,
    /// Stable-key activation callback.
    #[prop(optional, into)]
    on_row_activate_keyed: Option<Callback<ServerTableRowAction>>,
    /// Compatibility inspect callback using the page-local row index.
    #[prop(optional, into)]
    on_row_inspect: Option<Callback<usize>>,
    /// Stable-key inspection callback.
    #[prop(optional, into)]
    on_row_inspect_keyed: Option<Callback<ServerTableRowAction>>,
    /// Caller-owned actions placed beside the always-present gear control.
    #[prop(optional, into)]
    toolbar_actions: Option<ViewFn>,
    /// Observes exactly the accepted server slice currently displayed.
    #[prop(optional, into)]
    on_displayed_slice: Option<Callback<ServerTableDisplayedSlice>>,
) -> impl IntoView {
    let stable_column_tools_state =
        ServerColumnToolsState::new(preference_ownership.clone(), preference_version, columns);
    let configuration = Memo::new(move |_| {
        columns.with(|columns| validate_server_entity_configuration(columns, query_capabilities))
    });

    move || {
        match configuration.get() {
        Err(error) => view! {
            <div
                role="alert"
                data-server-entity-table-config-error=error.code()
                class="rounded-box border border-error bg-error/10 px-3 py-2 text-sm text-error forced-colors:border-[CanvasText] forced-colors:text-[CanvasText]"
            >
                {error.to_string()}
            </div>
        }
        .into_any(),
        Ok(()) => {
            let current_page = Signal::derive(move || query.get().page.max(1));
            let page_size = Signal::derive(move || query.get().page_size.max(1));
            let no_op_compatibility_page_change = Callback::new(|_: i64| {});
            let query_ownership =
                ServerTableQueryOwnership::controlled(query, on_query_change);
            let mut column_tools = ServerTableColumnTools::new(
                preference_ownership.clone(),
                preference_version,
            )
            .with_prebuilt_state(stable_column_tools_state)
            .with_chooser_trigger(Signal::stored(EntityColumnChooserTrigger::Icon))
            .with_texts(column_tools_texts);
            if let Some(render_actions) = toolbar_actions.clone() {
                column_tools = column_tools.with_toolbar_actions(move || render_actions.run());
            }

            let server_table = ServerDataTable(ServerDataTableProps {
                rows,
                columns,
                current_page: Some(current_page),
                total_count: Some(total_count),
                page_size: Some(page_size),
                on_page_change: Some(no_op_compatibility_page_change),
                pagination: None,
                query_capabilities,
                loading,
                classes: DataTableClasses::default(),
                texts,
                sort_texts,
                text_filter_label,
                control_id: MaybeProp::from(control_id.clone()),
                class: "h-full min-h-0",
                table_size: Signal::stored(TableSize::default()),
                zebra: Signal::stored(false),
                pin_rows: Signal::stored(true),
                pin_cols: Signal::stored(false),
                max_height: Some("100%".to_owned()),
                viewport_fit,
                viewport_fit_min_rows,
                page_size_preference: Some(page_size_preference),
                page_size_auto_label,
                on_search: None,
                on_query_change: None,
                query_ownership: Some(query_ownership),
                query_reset_key: None,
                page_size_options,
                filter_options,
                filter_option_entries,
                filter_vocabulary: None,
                node_ref: NodeRef::new(),
                cell_renderers: cell_renderers.clone(),
                typed_cells: typed_cells.clone(),
                detail_renderer,
                row_class_fn,
                row_key,
                selection,
                multi_selection: multi_selection.clone(),
                on_row_activate,
                on_row_activate_keyed,
                on_row_inspect,
                on_row_inspect_keyed,
                column_tools: Some(column_tools),
                on_displayed_slice,
            });

            view! {
                <div class="contents" data-server-entity-table="true">
                    {server_table}
                </div>
            }
            .into_any()
        }
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::Column;

    #[test]
    fn missing_filter_is_rejected_even_for_a_standard_column() {
        assert_eq!(
            validate_server_entity_columns(&[Column::new("run", "Run")]),
            Err(ServerEntityTableConfigurationError::MissingFilter { column_id: "run" })
        );
    }

    #[test]
    fn action_column_is_not_exempt_from_the_filter_contract() {
        assert_eq!(
            validate_server_entity_columns(&[Column::new("run", "Run").action()]),
            Err(ServerEntityTableConfigurationError::MissingFilter { column_id: "run" })
        );
    }

    #[test]
    fn missing_filter_error_names_the_misconfigured_column() {
        assert_eq!(
            ServerEntityTableConfigurationError::MissingFilter { column_id: "run" }.to_string(),
            "ServerEntityTable column `run` must declare an exact or text filter"
        );
    }

    #[test]
    fn exact_and_text_filters_satisfy_the_standard_including_action_columns() {
        assert!(
            validate_server_entity_columns(&[
                Column::new("run", "Run").filterable_text().action(),
                Column::new("verdict", "Verdict").filterable(),
            ])
            .is_ok()
        );
    }

    #[test]
    fn filtering_capability_is_required_by_the_canonical_facade() {
        assert_eq!(
            validate_server_entity_capabilities(
                ServerQueryCapabilities::all().with_filtering(false)
            ),
            Err(ServerEntityTableConfigurationError::FilteringDisabled)
        );
    }

    #[test]
    fn page_size_capability_is_required_by_the_canonical_facade() {
        assert_eq!(
            validate_server_entity_capabilities(
                ServerQueryCapabilities::all().with_page_size(false)
            ),
            Err(ServerEntityTableConfigurationError::PageSizeDisabled)
        );
    }

    #[test]
    fn capability_errors_have_stable_alert_codes_and_copy() {
        assert_eq!(
            ServerEntityTableConfigurationError::FilteringDisabled.code(),
            "filtering-disabled"
        );
        assert_eq!(
            ServerEntityTableConfigurationError::FilteringDisabled.to_string(),
            "ServerEntityTable requires endpoint filtering support; use ServerDataTable for a partial composition"
        );
        assert_eq!(
            ServerEntityTableConfigurationError::PageSizeDisabled.code(),
            "page-size-disabled"
        );
        assert_eq!(
            ServerEntityTableConfigurationError::PageSizeDisabled.to_string(),
            "ServerEntityTable requires endpoint page-size support; use ServerDataTable for a partial composition"
        );
    }

    #[test]
    fn search_capability_can_be_disabled_for_history_like_endpoints() {
        assert!(
            validate_server_entity_capabilities(ServerQueryCapabilities::all().with_search(false))
                .is_ok()
        );
    }

    #[test]
    fn facade_delegates_server_data_operations_to_server_data_table() {
        let source = include_str!("server_entity_table.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes its tests");

        assert!(production.contains("ServerDataTable(ServerDataTableProps"));
        for forbidden in [
            "row_matches_",
            "compare_cells",
            "page_bounds",
            ".chunks(",
            ".skip(",
            ".take(",
        ] {
            assert!(
                !production.contains(forbidden),
                "ServerEntityTable must not perform local data operations: {forbidden}"
            );
        }
    }
}
