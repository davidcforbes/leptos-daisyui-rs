//! 5-column kanban board used by the QA Requests pipeline (UT2-15) and
//! reusable for incident triage / sandbox lifecycle boards.
//!
//! Renders one column per `KanbanColumn` and one card per `KanbanCard`,
//! using the `Card` / `CardBody` / `Progress` library wrappers — no
//! hand-coded daisyUI classes per `CLAUDE.md` rules.
//!
//! Cards are read-only for the demo path. Drag-and-drop / click-to-advance
//! semantics are deferred to a follow-up bead; the public API stays the
//! same when those land.

use leptos::prelude::*;

use crate::components::{Card, CardBody, Progress, ProgressColor};

/// One card on the kanban board.
#[derive(Clone, Debug, PartialEq)]
pub struct KanbanCard {
    /// Project / request title shown in the card header.
    pub title: String,
    /// Optional priority label ("P1", "P2", ...).
    pub priority_label: Option<String>,
    /// Hex colour for the priority badge (no leading `#`).
    pub priority_color: Option<String>,
    /// Coverage percent shown under the title; `None` hides the row.
    pub coverage_pct: Option<u32>,
    /// Defect count shown under the title alongside coverage.
    pub defect_count: Option<u32>,
    /// Sub-line shown below the title when no coverage / defects apply
    /// (e.g. "Awaiting test" for the Queued column).
    pub sub_label: Option<String>,
}

/// One column of the kanban.
#[derive(Clone, Debug, PartialEq)]
pub struct KanbanColumn {
    /// Display name shown in the column header.
    pub title: String,
    /// Background tint applied to the column header (rgba string).
    pub header_bg: String,
    /// Progress bar colour used for any cards inside this column.
    pub progress_color: ProgressColor,
    /// Cards belonging to this column.
    pub cards: Vec<KanbanCard>,
}

#[component]
pub fn PipelineKanban(columns: Vec<KanbanColumn>) -> impl IntoView {
    view! {
        <div class="flex gap-3 mt-2 overflow-x-auto">
            {columns.into_iter().map(|col| {
                let count = col.cards.len();
                let header_style = format!("background-color: {};", col.header_bg);
                let progress_color = col.progress_color;
                view! {
                    <div class="flex-1 min-w-[170px]">
                        <div class="rounded-md px-3 py-1.5 mb-2 text-center" style=header_style>
                            <span class="text-[11px] font-semibold text-base-content/60">
                                {format!("{} ({})", col.title, count)}
                            </span>
                        </div>
                        <div class="space-y-2">
                            {col.cards.into_iter().map(|card| {
                                let title = card.title.clone();
                                let priority_label = card.priority_label.clone();
                                let priority_color = card.priority_color.clone();
                                let coverage_pct = card.coverage_pct;
                                let defect_count = card.defect_count;
                                let sub_label = card.sub_label.clone();
                                let progress_color_for_card = progress_color.clone();
                                view! {
                                    <Card class="shadow-sm border border-base-200">
                                        <CardBody class="p-3">
                                            <div class="flex justify-between items-start">
                                                <span class="text-[11px] font-medium text-base-content">{title}</span>
                                                {priority_label.map(|pl| {
                                                    let color = priority_color.unwrap_or_else(|| "#808080".into());
                                                    let style = format!(
                                                        "background-color: {}26; color: {};",
                                                        color, color
                                                    );
                                                    view! {
                                                        <span
                                                            class="text-[8px] font-bold px-1.5 py-0.5 rounded"
                                                            style=style
                                                        >{pl}</span>
                                                    }
                                                })}
                                            </div>
                                            {match (coverage_pct, defect_count, sub_label) {
                                                (Some(cov), Some(d), _) => view! {
                                                    <span class="text-[9px] text-base-content/50 mt-1">
                                                        {format!("Coverage: {}% | Defects: {}", cov, d)}
                                                    </span>
                                                    <Progress
                                                        color=progress_color_for_card
                                                        class="h-1.5 mt-1"
                                                        attr:value=cov
                                                        attr:max=100
                                                    />
                                                }.into_any(),
                                                (_, _, Some(sub)) => view! {
                                                    <span class="text-[9px] text-base-content/50 mt-1">{sub}</span>
                                                }.into_any(),
                                                _ => ().into_any(),
                                            }}
                                        </CardBody>
                                    </Card>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
