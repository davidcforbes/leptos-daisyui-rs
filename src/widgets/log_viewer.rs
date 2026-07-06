use crate::components::{Select, SelectSize};
use leptos::either::Either;
use leptos::prelude::*;

/// Severity level for a log entry.
#[derive(Clone, Debug, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// CSS class for color-coding the level badge.
    fn badge_class(&self) -> &'static str {
        match self {
            LogLevel::Debug => "badge badge-ghost badge-xs",
            LogLevel::Info => "badge badge-info badge-xs",
            LogLevel::Warn => "badge badge-warning badge-xs",
            LogLevel::Error => "badge badge-error badge-xs",
        }
    }

    /// Display label for the level.
    fn label(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// CSS class for the log message text color.
    fn text_class(&self) -> &'static str {
        match self {
            LogLevel::Debug => "text-base-content/50",
            LogLevel::Info => "text-base-content",
            LogLevel::Warn => "text-warning",
            LogLevel::Error => "text-error",
        }
    }
}

/// A single log entry.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    /// ISO-8601 timestamp string.
    pub timestamp: String,
    /// Severity level.
    pub level: LogLevel,
    /// Source system or module that generated the entry.
    pub source: String,
    /// The log message content.
    pub message: String,
}

/// Fixed row height for the virtual scroller — log rows are single-line
/// text inside a `py-0.5` container; 24px gives a comfortable line + padding.
const ROW_HEIGHT_PX: usize = 24;
/// Viewport height for the scrollable container (matches `max-h-96`).
const VIEWPORT_HEIGHT_PX: usize = 384;
/// Number of rows to render above and below the visible slice as a scroll buffer.
const VIEWPORT_BUFFER_ROWS: usize = 5;

/// Scrollable log viewer component for network logs and prompt history.
///
/// Displays log entries in a terminal-style scrollable container with level
/// color-coding. Includes filter controls for level and source.
///
/// EUC-0ip5: Implements real fixed-row-height virtual scrolling — only the
/// rows in the viewport (plus a small buffer above and below) are rendered,
/// regardless of total entry count. Scales to 15,000+ network log entries.
#[component]
pub fn LogViewer(
    /// All log entries to display.
    entries: Vec<LogEntry>,
    /// Reserved for backward compatibility — no longer enforces a cap on
    /// rendered rows now that the virtual scroller handles arbitrary sizes.
    /// Pass any value (defaults to 100); ignored at runtime.
    #[prop(default = 100)]
    _max_visible: usize,
    /// Whether to show the filter controls bar.
    #[prop(default = true)]
    show_filters: bool,
) -> impl IntoView {
    // Collect unique sources for the filter dropdown
    let sources: Vec<String> = {
        let mut s: Vec<String> = entries.iter().map(|e| e.source.clone()).collect();
        s.sort();
        s.dedup();
        s
    };

    let entries = StoredValue::new(entries);
    let sources = StoredValue::new(sources);

    // Filter state signals
    let level_filter = RwSignal::new(String::from("all"));
    let source_filter = RwSignal::new(String::from("all"));
    // Scroll position (top of viewport, in pixels) — driven by on:scroll.
    let scroll_top = RwSignal::new(0_usize);

    // Derived signal: filtered entries (no take() cap — virtual scroll handles size).
    let filtered_entries = Memo::new(move |_| {
        let all = entries.get_value();
        let level = level_filter.get();
        let source = source_filter.get();

        all.into_iter()
            .filter(|e| {
                if level != "all" {
                    let entry_level = e.level.label();
                    if entry_level != level {
                        return false;
                    }
                }
                if source != "all" && e.source != source {
                    return false;
                }
                true
            })
            .collect::<Vec<_>>()
    });

    // Visible window: derive start/end row indices from scroll position +
    // viewport size + buffer, then slice the filtered entries.
    let visible_entries = Memo::new(move |_| {
        let all = filtered_entries.get();
        let total = all.len();
        if total == 0 {
            return (0_usize, Vec::new(), 0_usize);
        }
        let st = scroll_top.get();
        let first_visible_row = st / ROW_HEIGHT_PX;
        let last_visible_row = (st + VIEWPORT_HEIGHT_PX).div_ceil(ROW_HEIGHT_PX);
        let start = first_visible_row.saturating_sub(VIEWPORT_BUFFER_ROWS);
        let end = (last_visible_row + VIEWPORT_BUFFER_ROWS).min(total);
        let slice: Vec<LogEntry> = all[start..end].to_vec();
        let trailing = total.saturating_sub(end);
        (start, slice, trailing)
    });

    view! {
        <div class="flex flex-col gap-2">
            // Filter controls
            <Show when=move || show_filters>
                <div class="flex flex-wrap gap-3 items-center p-2 bg-base-200 rounded-lg">
                    <label class="text-xs font-semibold text-base-content/60 uppercase tracking-wide">"Filters"</label>

                    // Level filter
                    <Select
                        size=SelectSize::Xs
                        class="select-bordered w-auto"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            level_filter.set(val);
                        }
                    >
                        <option value="all">"All Levels"</option>
                        <option value="DEBUG">"Debug"</option>
                        <option value="INFO">"Info"</option>
                        <option value="WARN">"Warn"</option>
                        <option value="ERROR">"Error"</option>
                    </Select>

                    // Source filter
                    <Select
                        size=SelectSize::Xs
                        class="select-bordered w-auto"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            source_filter.set(val);
                        }
                    >
                        <option value="all">"All Sources"</option>
                        {move || sources.get_value().into_iter().map(|s| {
                            let s2 = s.clone();
                            view! { <option value=s>{s2}</option> }
                        }).collect::<Vec<_>>()}
                    </Select>

                    // Entry count — total filtered, not just rendered slice.
                    <span class="text-xs text-base-content/50 ml-auto">
                        {move || format!("Showing {} entries (virtual-scrolled)", filtered_entries.get().len())}
                    </span>
                </div>
            </Show>

            // Log output area — virtual scroller. The on:scroll handler updates
            // scroll_top, which drives the visible_entries memo to recompute the
            // rendered slice. Top + bottom spacer divs preserve the total scroll
            // height so the scrollbar reflects the full dataset size.
            <div
                class="bg-base-300 rounded-lg p-2 overflow-y-auto font-mono text-xs"
                style:max-height=move || format!("{}px", VIEWPORT_HEIGHT_PX)
                style:height=move || format!("{}px", VIEWPORT_HEIGHT_PX)
                on:scroll=move |ev| {
                    use leptos::wasm_bindgen::JsCast;
                    if let Some(target) = ev.target()
                        && let Ok(elem) = target.dyn_into::<web_sys::HtmlElement>()
                    {
                        scroll_top.set(elem.scroll_top().max(0) as usize);
                    }
                }
            >
                {move || {
                    let (start, slice, trailing) = visible_entries.get();
                    if filtered_entries.get().is_empty() {
                        Either::Left(view! {
                            <div class="text-center text-base-content/40 py-8">
                                "No log entries match the current filters."
                            </div>
                        })
                    } else {
                        let top_spacer_px = start * ROW_HEIGHT_PX;
                        let bottom_spacer_px = trailing * ROW_HEIGHT_PX;
                        Either::Right(view! {
                            <div style:height=move || format!("{}px", top_spacer_px)></div>
                            {slice.into_iter().map(|entry| {
                                let text_class = entry.level.text_class();
                                let badge_class = entry.level.badge_class();
                                let level_label = entry.level.label().to_string();
                                let source_title = entry.source.clone();
                                let source_text = entry.source.clone();
                                view! {
                                    <div
                                        class="flex items-start gap-2 border-b border-base-content/5 last:border-0"
                                        style:height=format!("{}px", ROW_HEIGHT_PX)
                                    >
                                        <span class="text-base-content/40 shrink-0 w-40">{entry.timestamp}</span>
                                        <span class=badge_class>{level_label}</span>
                                        <span class="text-info/70 shrink-0 w-24 truncate" title=source_title>{source_text}</span>
                                        <span class=text_class>{entry.message}</span>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                            <div style:height=move || format!("{}px", bottom_spacer_px)></div>
                        })
                    }
                }}
            </div>
        </div>
    }
}
