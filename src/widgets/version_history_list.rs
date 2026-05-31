//! Reusable version-history list (UT2-21 EUC-mz0z; reused by UT2-22 Standards).
//!
//! Renders a stack of versioned entries — version label, timestamp, change
//! summary — using `Timeline` from leptos-daisyui-rs. The first entry is
//! flagged "Current" via a green dot; older entries get a blue dot.
//!
//! Designed to be agnostic of the source domain (policies, standards,
//! reports, …): pass a `Vec<VersionHistoryEntry>` with whatever values
//! the resolver returns.

use leptos::prelude::*;

use crate::components::{
    Timeline, TimelineItem, TimelineItemEnd, TimelineItemMiddle, TimelineItemPosition,
};

/// One row in the version history list.
#[derive(Clone, Debug)]
pub struct VersionHistoryEntry {
    /// Display label, e.g. "v3.2".
    pub version_label: String,
    /// Timestamp / date string already formatted for display, e.g. "Jan 15, 2026".
    pub timestamp_label: String,
    /// Optional change summary line, e.g. "Updated AI model guidelines".
    pub change_summary: Option<String>,
    /// Marks this entry as the current/latest version (renders a green dot
    /// and " - Current" suffix on the version label).
    pub is_current: bool,
}

/// Renders a vertical timeline of `VersionHistoryEntry` rows.
///
/// Pass a non-empty `Vec`. An empty list renders a small muted "No version
/// history" placeholder so the right-rail layout remains stable.
#[component]
pub fn VersionHistoryList(
    /// Section heading shown above the timeline (defaults to "Version History").
    #[prop(into, default = "Version History".to_string())]
    title: String,
    /// Entries to render — newest-first. The first row is auto-flagged
    /// `is_current` if no entry has the flag set explicitly.
    entries: Vec<VersionHistoryEntry>,
) -> impl IntoView {
    let len = entries.len();

    let body_view = if entries.is_empty() {
        view! {
            <p class="text-xs text-base-content/50 italic">"No version history yet."</p>
        }
        .into_any()
    } else {
        view! {
            <Timeline compact=true>
                {entries
                    .into_iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let position = if i == 0 {
                            TimelineItemPosition::Start
                        } else if i == len - 1 {
                            TimelineItemPosition::End
                        } else {
                            TimelineItemPosition::Between
                        };
                        let dot_class = if e.is_current {
                            "bg-emerald-500"
                        } else {
                            "bg-blue-500"
                        };
                        let label_suffix = if e.is_current {
                            " \u{2014} Current"
                        } else {
                            ""
                        };
                        let title = format!("{}{}", e.version_label, label_suffix);
                        let summary = e.change_summary;
                        view! {
                            <TimelineItem position=position>
                                <TimelineItemMiddle>
                                    <div class=format!("w-2 h-2 rounded-full {dot_class}")></div>
                                </TimelineItemMiddle>
                                <TimelineItemEnd>
                                    <p class="text-sm leading-tight font-medium">{title}</p>
                                    <p class="text-xs text-base-content/60">{e.timestamp_label}</p>
                                    {summary.map(|s| view! {
                                        <p class="text-xs text-base-content/50">{s}</p>
                                    })}
                                </TimelineItemEnd>
                            </TimelineItem>
                        }
                    })
                    .collect::<Vec<_>>()}
            </Timeline>
        }
        .into_any()
    };

    view! {
        <div class="space-y-3">
            <h4 class="font-semibold text-sm">{title}</h4>
            {body_view}
        </div>
    }
}
