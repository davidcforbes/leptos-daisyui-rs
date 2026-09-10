//! Compile-checked integration shape, not a standalone backend application.
//!
//! The consuming host fetches and correlates requests, maps its domain records
//! into these fields, and atomically replaces accepted rows/query/total. Exact
//! options cover the authorized population. Failed/stale responses replace none
//! of those accepted values; preference persistence is separately host-owned.
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use std::collections::HashMap;

/// Standard History presentation around the host's accepted server snapshot.
#[component]
pub fn HistoryTable(
    rows: Signal<Vec<TableRow>>,
    query: Signal<TableQuery>,
    total_count: Signal<i64>,
    loading: Signal<bool>,
    request_query: Callback<TableQuery>,
    exact_options: Signal<HashMap<&'static str, Vec<String>>>,
    preferences: Signal<EntityTablePreferences>,
    accept_preferences: Callback<EntityTablePreferences>,
    page_size_intent: RwSignal<ServerTablePageSizePreference>,
) -> impl IntoView {
    let columns = vec![
        Column::new("run", "Run").filterable_text(),
        Column::new("module", "Module").filterable_text(),
        Column::new("mode", "Mode").filterable(),
        Column::new("started_at", "Started").filterable_text(),
        Column::new("duration", "Duration").filterable_text(),
        Column::new("verdict", "Verdict").filterable(),
        Column::new("captured", "Captured").filterable_text(),
        Column::new("rejected", "Rejected").filterable_text(),
        Column::new("trigger", "Trigger").filterable(),
        Column::new("build", "Build").filterable(),
    ];
    view! {
        <div class="h-[60vh] min-h-0">
            <ServerEntityTable
                rows=rows
                columns=Signal::stored(columns)
                query=query
                total_count=total_count
                loading=loading
                on_query_change=request_query
                control_id="history-table".to_owned()
                preference_ownership=EntityTablePreferenceOwnership::controlled(
                    preferences, accept_preferences,
                )
                preference_version=1
                page_size_preference=page_size_intent
                query_capabilities=ServerQueryCapabilities::all().with_search(false)
                viewport_fit=true
                filter_options=exact_options
                row_key=Callback::new(|row: TableRow| {
                    row.get("id").cloned().unwrap_or_default()
                })
            />
        </div>
    }
}

fn main() {}
