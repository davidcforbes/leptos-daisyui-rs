//! Opinionated page patterns and their machine-checkable contracts.

mod action_feedback;
mod active_filter_chips;
mod async_data_section;
mod contracts;
mod dataset_selector;
mod filter_bar;
mod list_page;
mod macros;
mod page_header;
mod page_state_panel;
mod snapshot_table;
mod snapshot_table_page;

pub use crate::{entity_columns, filter_schema, page_contract};
pub use action_feedback::{
    ActionAnnouncement, ActionFeedback, ActionFeedbackEntry, ActionFeedbackModel,
    ActionFeedbackState, ActionFeedbackTexts, ActionTransitionError,
};
pub use active_filter_chips::{ActiveFilterChip, ActiveFilterChips, active_filter_summary};
pub use async_data_section::{AsyncDataSection, AsyncDataTexts, state_shows_content};
pub use contracts::{
    AccessibilityContract, AccessibilityObligation, AccessibleLabel, CapabilityAction,
    CapabilityRule, ClientSnapshotContract, CompatibilityContract, CompatibilityDependencyKind,
    ContractError, ContractNameKind, DataContract, DataMode, DatasetBehavior, DatasetContract,
    DatasetDefault, DatasetLoad, DatasetSelector, FilterSchema, MutationContract, MutationOutcome,
    NamedBaseline, PAGE_CONTRACT_V2_EXPORT_SCHEMA, PAGE_CONTRACT_V2_VERSION, PageArchetype,
    PageBreakpoint, PageBudget, PageContract, PageContractExport, PageContractExportError,
    PageContractV2, PageContractV2Error, PageControl, PageDelivery, PageDependency, PagePattern,
    PageState, PresentationState, RealtimeContract, RealtimeEvent, RealtimeState,
    RealtimeTransport, ResponsiveBehavior, ResponsiveContract, ResponsiveLayout, RowIdentity,
    SortExecution, SourceOwnership, StateField, StateOwnership, TestLane,
};
pub use dataset_selector::{DatasetOption, DatasetSelector, selected_dataset_label};
pub use filter_bar::{FILTER_BAR_BASE_CLASS, FilterBar, filter_bar_class};
pub use list_page::{LIST_PAGE_BASE_CLASS, ListPage, list_page_class};
pub use page_header::PageHeader;
pub use page_state_panel::{PageStatePanel, PageStatePanelTexts};
pub use snapshot_table::{
    LocalResultSummary, PageStatePanelKind, SnapshotAccess, SnapshotActionDisposition,
    SnapshotActionHandle, SnapshotActionStartError, SnapshotData, SnapshotDataError,
    SnapshotGeneration, SnapshotRenderDecision, SnapshotRequestError, SnapshotRequestHandle,
    SnapshotTablePhase, SnapshotTableState, SnapshotTableView, SnapshotTransitionDisposition,
};
pub use snapshot_table_page::{
    SnapshotDatasetOption, SnapshotDatasetSelectorConfig, SnapshotEntityTableConfig,
    SnapshotTablePage,
};

#[cfg(test)]
mod tests;
