//! Horizontal, wrapping local-filter composition.

use leptos::prelude::*;

/// Canonical horizontal-filter classes.
pub const FILTER_BAR_BASE_CLASS: &str = "flex w-full min-w-0 flex-wrap items-end gap-3 rounded-box border border-base-300 bg-base-100 p-3";

/// Merges caller classes with the canonical filter-bar contract.
pub fn filter_bar_class(class: &str) -> String {
    [FILTER_BAR_BASE_CLASS, class]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Search-first, actions-last row for local filters only.
#[component]
pub fn FilterBar(
    /// Search control rendered first and allowed to grow.
    search: Children,
    /// Clear/reset and other filter-level actions rendered last.
    #[prop(optional)]
    actions: Option<Children>,
    /// Additional classes.
    #[prop(optional, into)]
    class: &'static str,
    /// Selects and other local filter controls.
    children: Children,
) -> impl IntoView {
    view! {
        <section class=filter_bar_class(class) data-filter-bar="local" aria-label="Filters">
            <div class="min-w-56 flex-[2_1_20rem]" data-filter-search="true">
                {search()}
            </div>
            <div class="flex min-w-0 flex-[3_1_28rem] flex-wrap items-end gap-3">
                {children()}
            </div>
            {actions.map(|actions| view! {
                <div class="ml-auto flex shrink-0 items-center gap-2" data-filter-actions="true">
                    {actions()}
                </div>
            })}
        </section>
    }
}
