//! Higher-level composite widgets built on the daisyUI [`crate::components`]
//! primitives — read-only data displays (property lists, JSON trees, timelines,
//! risk matrices, kanban boards), small inputs (tag input), filter rows, and a
//! sortable data table. Promoted from the euc frontend so any Leptos/daisyUI
//! app can reuse them. Unlike [`crate::components`] (thin daisyUI wrappers),
//! these compose several primitives into a ready-to-use unit.

mod avatar_badge;
mod compliance_checklist;
mod data_table;
mod defect_severity_bar;
mod department_chips;
mod filter_tabs;
mod horizontal_stacked_bar;
mod json_tree_viewer;
mod log_viewer;
mod page_error_boundary;
mod pipeline_kanban;
mod progress_bar_row;
mod property_list;
mod quick_filter_row;
mod risk_assessment_matrix;
mod tag_input;
mod timeline_list;
// `tree_view` carries a flattened-node helper field/fn that the read-only
// view doesn't currently read; allow it rather than prune migrated logic.
#[allow(dead_code)]
mod tree_view;
mod version_history_list;
mod workflow_visualizer;

pub use avatar_badge::{
    AvatarBadge, AvatarBadgeSize, NAME_PALETTE, initials_from_name, name_color_class,
};
pub use compliance_checklist::{ComplianceChecklist, ComplianceChecklistItem};
pub use data_table::{DataTable, TableColumn};
pub use defect_severity_bar::{DefectSegment, DefectSeverityBar};
pub use department_chips::DepartmentChips;
pub use filter_tabs::FilterTabs;
pub use horizontal_stacked_bar::{HorizontalSeries, HorizontalStackedBar};
pub use json_tree_viewer::JsonTreeViewer;
pub use log_viewer::{LogEntry, LogLevel, LogViewer};
pub use page_error_boundary::PageErrorBoundary;
pub use pipeline_kanban::{KanbanCard, KanbanColumn, PipelineKanban};
pub use progress_bar_row::ProgressBarRow;
pub use property_list::{PropertyEntry, PropertyList};
pub use quick_filter_row::{QuickFilterAction, QuickFilterDropdown, QuickFilterRow};
pub use risk_assessment_matrix::{RiskAssessmentMatrix, RiskMatrixDatum};
pub use tag_input::TagInput;
pub use timeline_list::{TimelineList, TimelineListEntry};
pub use tree_view::{TreeNode, TreeView};
pub use version_history_list::{VersionHistoryEntry, VersionHistoryList};
pub use workflow_visualizer::{StepStatus, WorkflowStep, WorkflowVisualizer};
