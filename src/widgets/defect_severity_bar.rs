//! Single horizontal stacked bar for defect-severity distribution.
//!
//! Each segment renders proportional to its share of the total. Empty bar
//! collapses to a "no defects" message. Wraps `Card`/`CardBody` from the
//! library — no hand-coded daisyUI per `CLAUDE.md`.

use leptos::prelude::*;

use crate::components::{Card, CardBody};

/// One segment of the severity bar.
#[derive(Clone, Debug, PartialEq)]
pub struct DefectSegment {
    pub label: String,
    pub count: u64,
    /// Hex colour for the segment fill (with leading `#`).
    pub color: String,
}

#[component]
pub fn DefectSeverityBar(segments: Vec<DefectSegment>) -> impl IntoView {
    let total: u64 = segments.iter().map(|s| s.count).sum();

    view! {
        <Card class="shadow-sm border border-base-200 mt-2">
            <CardBody class="p-4">
                {if total == 0 {
                    view! {
                        <div class="text-center text-xs text-base-content/50 py-2">
                            "No open defects"
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex items-center h-5 rounded-lg overflow-hidden">
                            {segments.into_iter().map(|seg| {
                                let pct = (seg.count as f64 / total as f64) * 100.0;
                                let style = format!(
                                    "width: {:.2}%; background-color: {};",
                                    pct, seg.color
                                );
                                let label = format!("{} ({})", seg.label, seg.count);
                                view! {
                                    <div
                                        class="h-full flex items-center justify-center"
                                        style=style
                                    >
                                        <span class="text-[9px] font-medium text-white whitespace-nowrap">
                                            {label}
                                        </span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </CardBody>
        </Card>
    }
}
