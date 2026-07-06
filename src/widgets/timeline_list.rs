//! Reusable chronological event list (UT2-30 / EUC-rqw6).
//!
//! Renders a vertical timeline using the leptos-daisyui-rs `Timeline` library
//! component, with one row per event: a colored dot, a title, and an optional
//! subtitle / description. Used by:
//!   - Alerts page right rail (UT2-30)
//!   - Investigations page right rail (UT2-32)
//!   - Network Logs page right rail (UT2-28)
//!
//! Composes library components only — no hand-coded daisyUI HTML.

use leptos::prelude::*;

use crate::components::{
    Timeline, TimelineItem, TimelineItemEnd, TimelineItemMiddle, TimelineItemPosition,
};

/// One row in a `TimelineList`.
#[derive(Clone, Debug)]
pub struct TimelineListEntry {
    /// Primary event text — required.
    pub title: String,
    /// Optional secondary line (e.g. timestamp, actor).
    pub subtitle: Option<String>,
    /// Optional third line (e.g. event detail / context).
    pub description: Option<String>,
    /// Free-text severity hint mapped to dot color: `error` | `warn`/`warning`
    /// | `info` (default) | `success` | `muted`.
    pub severity_code: Option<String>,
}

impl TimelineListEntry {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            description: None,
            severity_code: None,
        }
    }

    pub fn with_subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }

    pub fn with_description(mut self, s: impl Into<String>) -> Self {
        self.description = Some(s.into());
        self
    }

    pub fn with_severity(mut self, s: impl Into<String>) -> Self {
        self.severity_code = Some(s.into());
        self
    }
}

fn dot_class(severity_code: Option<&str>) -> &'static str {
    match severity_code {
        Some("error") | Some("critical") => "bg-red-500",
        Some("warn") | Some("warning") => "bg-amber-500",
        Some("success") | Some("ok") => "bg-emerald-500",
        Some("muted") => "bg-base-300",
        _ => "bg-blue-500",
    }
}

/// Vertical chronological timeline. Items are rendered in the order supplied
/// (caller controls newest-first vs. oldest-first). Compact = true to match
/// the right-rail aesthetic.
#[component]
pub fn TimelineList(
    /// Events to render, top-to-bottom.
    entries: Vec<TimelineListEntry>,
    /// If true (default), use compact spacing suitable for a right rail.
    #[prop(default = true)]
    compact: bool,
) -> impl IntoView {
    let len = entries.len();
    view! {
        <Timeline compact=compact>
            {entries
                .into_iter()
                .enumerate()
                .map(|(i, e)| {
                    let position = if i == 0 {
                        TimelineItemPosition::Start
                    } else if i + 1 == len {
                        TimelineItemPosition::End
                    } else {
                        TimelineItemPosition::Between
                    };
                    let dot = dot_class(e.severity_code.as_deref());
                    view! {
                        <TimelineItem position=position>
                            <TimelineItemMiddle>
                                <div class=format!("w-2 h-2 rounded-full {dot}")></div>
                            </TimelineItemMiddle>
                            <TimelineItemEnd>
                                <p class="text-sm leading-tight">{e.title}</p>
                                {e.subtitle.map(|s| view! {
                                    <p class="text-xs text-base-content/60">{s}</p>
                                })}
                                {e.description.map(|d| view! {
                                    <p class="text-xs text-base-content/50">{d}</p>
                                })}
                            </TimelineItemEnd>
                        </TimelineItem>
                    }
                })
                .collect::<Vec<_>>()}
        </Timeline>
    }
}
