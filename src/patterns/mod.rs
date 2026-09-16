//! Opinionated page patterns and their machine-checkable contracts.

mod action_feedback;
mod active_filter_chips;
mod ai_chat_workspace;
mod async_data_section;
mod contracts;
mod dataset_selector;
mod filter_bar;
mod helpdesk;
mod kpi_strip;
mod list_page;
mod macros;
mod page_header;
mod page_quick_actions;
mod page_state_panel;
mod record_header;
mod search_picker_dialog;
mod section_grid;
mod section_heading;
mod selectable_summary;
mod server_cursor_history;
mod snapshot_table;
mod snapshot_table_page;

pub use crate::{entity_columns, filter_schema, page_contract};
pub use action_feedback::{
    ActionAnnouncement, ActionFeedback, ActionFeedbackContent, ActionFeedbackEntry,
    ActionFeedbackModel, ActionFeedbackState, ActionFeedbackTexts, ActionTransitionError,
};
pub use active_filter_chips::{
    ActiveFilterChip, ActiveFilterChips, ActiveFilterTexts, active_filter_summary,
    active_filter_summary_with,
};
pub use ai_chat_workspace::{
    AiChatWorkspace, AiChatWorkspaceTexts, AvailabilityReasonCode, ChatPosture,
    ChatWorkspaceBackend, ChatWorkspaceError, ChatWorkspaceErrorKind, CodexLever, CodexLevers,
    CorpusQueryMode, CorpusScope, EngineHeader, EvidenceRail, GroqTuning, GroundingVerdict,
    IngestPhase, IngestStatus, KnowledgeSelection, KnowledgeSource, KnowledgeSourceRail,
    MemoryDraft, MemoryRefusal, ModelSource, OfficeKnowledgeScope, ProviderCard,
    ProviderSettingsRows, ProviderTuning, QuickAction, ReasoningEffort, RecallCorpus, RecallHit,
    RecallReceipt, ReopenReason, TuningDraft, TuningSchema, TurnEvidence, TurnNotice, TurnRecord,
    UsageFigures, UsageTotals, WorkspaceFuture, WorkspaceRefusal, canceled_partial,
    card_ready_for_ask, card_unready_reason, citations_of, completed_answer, corpus_choices,
    cost_line, declined_limitations, desktop_provider_catalogue, effort_selection,
    engine_is_metered, guardrail_refusal, label_is_distinct_from_options, lifecycle_id,
    notice_text, outcome_id, published_capabilities, reopen_announces_switch, usage_figures,
    usage_line,
};
// Aliased: these are generic enough that a bare re-export would read as the
// whole `patterns` module's vocabulary rather than the chat workspace's.
pub use ai_chat_workspace::{
    DEFAULT_TEMPERATURE as AI_CHAT_DEFAULT_TEMPERATURE, EFFORT_CHOICES as AI_CHAT_EFFORT_CHOICES,
    WATCHDOG_FAILURE_KIND as AI_CHAT_WATCHDOG_FAILURE_KIND, WATCHDOG_MS as AI_CHAT_WATCHDOG_MS,
    cancel_notice as ai_chat_cancel_notice, failure_kind as ai_chat_failure_kind,
    header_cancel_discarded as ai_chat_header_cancel_discarded,
    header_failure_kind as ai_chat_header_failure_kind,
    header_outcome_id as ai_chat_header_outcome_id, honesty_for as ai_chat_honesty_for,
    honesty_state_for_code as ai_chat_honesty_state_for_code, honesty_tone as ai_chat_honesty_tone,
    is_terminal as ai_chat_turn_is_terminal, scope_label as ai_chat_scope_label,
    scope_value as ai_chat_scope_value, settings_for as ai_chat_settings_for_card,
    watchdog_should_fire as ai_chat_watchdog_should_fire,
};
// `BackendCall` and `SEED_NOW_MS` are also the names `helpdesk` uses for its
// own fixture's call log and seeded clock, and `helpdesk`'s are re-exported by
// glob below. The two are unrelated types, so the chat-workspace fixture's are
// aliased here rather than shadowing the helpdesk names.
#[cfg(feature = "test-mode")]
pub use ai_chat_workspace::{
    BackendCall as ChatWorkspaceCall, ChatWorkspaceFault, FixtureClock,
    InMemoryChatWorkspaceBackend, PromptMatcher, SEED_ACTOR as SEED_CHAT_ACTOR, SEED_FOLDER_COURT,
    SEED_FOLDER_INTAKE, SEED_NOW_MS as SEED_CHAT_NOW_MS, SEED_RECALL_QUERY, SEED_RECALL_SEARCH,
    ScriptedChatTransport, TurnScript,
};
pub use async_data_section::{AsyncDataSection, AsyncDataTexts, state_shows_content};
pub use contracts::{
    AccessibilityContract, AccessibilityObligation, AccessibleLabel, CapabilityAction,
    CapabilityRule, ClientSnapshotContract, CompatibilityContract, CompatibilityDependencyKind,
    ContractError, ContractNameKind, DataContract, DataMode, DatasetBehavior, DatasetContract,
    DatasetDefault, DatasetLoad, DatasetSelector, FilterProjectionError, FilterSchema,
    LocalFilterDefaults, MutationContract, MutationOutcome, NamedBaseline,
    PAGE_CONTRACT_V2_EXPORT_SCHEMA, PAGE_CONTRACT_V2_VERSION, PageArchetype, PageBreakpoint,
    PageBudget, PageContract, PageContractExport, PageContractExportError, PageContractV2,
    PageContractV2Error, PageControl, PageDelivery, PageDependency, PagePattern, PageState,
    PresentationState, RealtimeContract, RealtimeEvent, RealtimeState, RealtimeTransport,
    ResponsiveBehavior, ResponsiveContract, ResponsiveLayout, RowIdentity, SnapshotViewDefaults,
    SortExecution, SourceOwnership, StateField, StateOwnership, TestLane,
};
pub use dataset_selector::{
    DatasetOption, DatasetSelector, DatasetSelectorTexts, dataset_selector_shows_caption,
    selected_dataset_label,
};
pub use filter_bar::{
    FILTER_BAR_BASE_CLASS, FilterBar, FilterBarTexts, FilterResultSummary, SnapshotDefaultSave,
    SnapshotDefaultSaveState, filter_active_summary, filter_bar_class, filter_result_summary,
};
pub use helpdesk::*;
pub use kpi_strip::{
    KPI_BASELINE_TRACK_HEADROOM, KPI_CARD_HELP_FLOOR_PX, KPI_CARD_TWO_LINE_FLOOR_PX, KpiAction,
    KpiBaseline, KpiBaselineAvailability, KpiBaselineState, KpiCard, KpiComparison, KpiItem,
    KpiStatus, KpiStatusChip, KpiStrip, KpiStripLayout, KpiStripRowFit, KpiStripRung,
    KpiStripTexts, KpiTrend, kpi_card_is_activatable, kpi_strip_card_width_px, kpi_strip_gap_px,
    kpi_strip_row_fit,
};
pub use list_page::{LIST_PAGE_BASE_CLASS, ListPage, list_page_class};
pub use page_header::{PageHeader, PageHeaderDivider, PageHeaderNavigationLayout};
pub use page_quick_actions::{
    PageQuickActionContent, PageQuickActionLabelVisibility, PageQuickActions,
};
pub use page_state_panel::{PageStatePanel, PageStatePanelTexts};
pub use record_header::{
    RecordActionFeedback, RecordAvatar, RecordBadge, RecordHeader, RecordHeaderState,
    RecordHeaderTexts, RecordMetaItem, RecordQuickAction, RecordQuickActionState, RecordStatus,
    RecordStatusTone,
};
pub use search_picker_dialog::{
    ConfirmableSearchPickerDialog, ConfirmableSearchPickerDialogTexts, SearchPickerConfirmBlock,
    SearchPickerDialog, SearchPickerDialogTexts, SearchPickerDismissCause,
    SearchPickerRenderDecision, SearchPickerStatus, resolve_search_picker_confirmation,
    resolve_search_picker_selection, search_picker_confirm_block, search_picker_render_decision,
};
pub use section_grid::{
    SECTION_GRID_BASE_CLASS, SectionGrid, SectionGridColumns, section_grid_class,
};
pub use section_heading::{HeadingLevel, SectionHeading, SectionHeadingStatusPlacement};
pub use selectable_summary::{
    SelectableSummaryCard, SelectableSummaryGroup, SelectableSummaryItem, SelectableSummaryStatus,
    SelectableSummaryTexts,
};
pub use server_cursor_history::{
    ServerCursorHistory, ServerCursorHistoryDisposition, ServerCursorHistoryError,
    ServerCursorHistoryGeneration, ServerCursorHistoryHandle,
};
pub use snapshot_table::{
    LocalResultSummary, PageStatePanelKind, SnapshotAccess, SnapshotActionDisposition,
    SnapshotActionHandle, SnapshotActionStartError, SnapshotData, SnapshotDataError,
    SnapshotDeltaDisposition, SnapshotDeltaHandle, SnapshotDeltaStartError, SnapshotGeneration,
    SnapshotLocalRowProjection, SnapshotRenderDecision, SnapshotRequestError,
    SnapshotRequestHandle, SnapshotTablePhase, SnapshotTableState, SnapshotTableView,
    SnapshotTransitionDisposition,
};
pub use snapshot_table_page::{
    SnapshotDatasetOption, SnapshotDatasetSelectorConfig, SnapshotEntityTableConfig,
    SnapshotFilterActionsConfig, SnapshotTablePage,
};

#[cfg(test)]
mod tests;
