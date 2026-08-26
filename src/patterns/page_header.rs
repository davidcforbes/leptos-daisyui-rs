//! Consistent hierarchy and slots for list-page headings.

use leptos::prelude::*;

/// Page heading with explicit navigation, freshness, dataset, and action slots.
#[component]
pub fn PageHeader(
    /// Primary page title.
    #[prop(into)]
    title: Signal<String>,
    /// Supporting page description.
    #[prop(optional, into)]
    subtitle: Signal<String>,
    /// Optional back-navigation action rendered before the title.
    #[prop(optional)]
    back: Option<Children>,
    /// Optional freshness/status content adjacent to the title.
    #[prop(optional)]
    freshness: Option<Children>,
    /// Optional dataset selector capsule, separate from page filters.
    #[prop(optional)]
    dataset: Option<Children>,
    /// Optional page-level actions.
    #[prop(optional)]
    actions: Option<Children>,
    /// Additional header classes.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    view! {
        <header
            class=format!(
                "flex flex-col gap-4 border-b border-base-300 pb-4 lg:flex-row lg:items-start lg:justify-between {class}"
            )
            data-page-header="true"
        >
            <div class="flex min-w-0 flex-col items-start gap-3 sm:flex-row">
                {back.map(|back| view! { <div class="shrink-0 pt-1">{back()}</div> })}
                <div class="min-w-0 space-y-1">
                    <div class="flex flex-wrap items-center gap-2">
                        <h1 class="ld-text-display font-semibold tracking-tight text-base-content">
                            {move || title.get()}
                        </h1>
                        {freshness.map(|freshness| freshness())}
                    </div>
                    <p class="max-w-3xl text-sm text-base-content/75 sm:text-base">
                        {move || subtitle.get()}
                    </p>
                </div>
            </div>
            <div class="flex flex-wrap items-end gap-2 lg:justify-end">
                {dataset.map(|dataset| view! {
                    <div class="min-w-56" data-page-dataset-slot="true">{dataset()}</div>
                })}
                {actions.map(|actions| view! {
                    <div class="flex items-center gap-2" data-page-actions-slot="true">
                        {actions()}
                    </div>
                })}
            </div>
        </header>
    }
}
