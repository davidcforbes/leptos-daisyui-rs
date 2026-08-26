//! Removable summary chips for active local filters.

use crate::components::Button;
use leptos::prelude::*;

/// One active local filter rendered as a removable chip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveFilterChip {
    /// Stable filter key passed to the removal callback.
    pub id: String,
    /// Human-readable filter label.
    pub label: String,
    /// Current filter value.
    pub value: String,
}

impl ActiveFilterChip {
    /// Creates a removable local-filter chip.
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Produces the canonical active-filter count label.
pub fn active_filter_summary(count: usize) -> String {
    match count {
        0 => "No active filters".to_owned(),
        1 => "1 active filter".to_owned(),
        count => format!("{count} active filters"),
    }
}

/// Renders individually removable local filters and an optional clear-all action.
#[component]
pub fn ActiveFilterChips(
    /// Current active local filters.
    #[prop(into)]
    chips: Signal<Vec<ActiveFilterChip>>,
    /// Called with the stable filter key when a chip is removed.
    on_remove: Callback<String>,
    /// Optional clear-all callback. It never affects the dataset selector.
    #[prop(optional)]
    on_clear: Option<Callback<()>>,
    /// Clear-all action label.
    #[prop(into, default = Signal::stored("Clear filters".to_owned()))]
    clear_label: Signal<String>,
) -> impl IntoView {
    view! {
        <div
            class="flex min-h-7 flex-wrap items-center gap-2"
            data-active-filters="true"
            data-resets-dataset="false"
        >
            <span class="text-xs text-base-content/60">
                {move || active_filter_summary(chips.with(|chips| chips.len()))}
            </span>
            {move || chips.get().into_iter().map(|chip| {
                let id = chip.id.clone();
                let remove_label = format!("Remove {} filter", chip.label);
                view! {
                    <span class="badge badge-outline gap-1 py-3">
                        <span class="font-medium">{chip.label}</span>
                        <span>{chip.value}</span>
                        <button
                            type="button"
                            class="ld-focus-ring rounded-full px-1"
                            aria-label=remove_label
                            on:click=move |_| on_remove.run(id.clone())
                        >
                            "×"
                        </button>
                    </span>
                }
            }).collect_view()}
            {move || {
                let has_filters = chips.with(|chips| !chips.is_empty());
                (has_filters && on_clear.is_some()).then(|| view! {
                    <Button
                        class="btn-ghost btn-xs"
                        on_click=Callback::new(move |_| {
                            if let Some(callback) = on_clear {
                                callback.run(());
                            }
                        })
                    >
                        {move || clear_label.get()}
                    </Button>
                })
            }}
        </div>
    }
}
