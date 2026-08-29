//! Consistent hierarchy and slots for list-page headings.

use leptos::prelude::*;

/// Placement policy for [`PageHeader`]'s optional back-navigation slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageHeaderNavigationLayout {
    /// Preserve the historical title-cluster placement: beside the title at
    /// wider widths and stacked by the existing responsive flex rules.
    #[default]
    InlineResponsive,
    /// Render one dedicated navigation landmark above all heading content at
    /// every viewport width.
    DedicatedRow,
}

impl PageHeaderNavigationLayout {
    /// Stable runtime marker emitted on the header root.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InlineResponsive => "inline-responsive",
            Self::DedicatedRow => "dedicated-row",
        }
    }
}

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
    /// Placement for `back`. The historical inline-responsive layout remains
    /// the default; select `DedicatedRow` for a separate row above the title.
    #[prop(optional)]
    navigation_layout: PageHeaderNavigationLayout,
    /// Localizable accessible name for the dedicated navigation landmark.
    #[prop(into, default = Signal::stored("Page navigation".to_owned()))]
    navigation_label: Signal<String>,
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
    let back = back.map(|slot| slot());
    let freshness = freshness.map(|slot| slot());
    let dataset = dataset.map(|slot| slot());
    let actions = actions.map(|slot| slot());

    match navigation_layout {
        PageHeaderNavigationLayout::InlineResponsive => view! {
            <header
                class=format!(
                    "flex flex-col gap-4 border-b border-base-300 pb-4 lg:flex-row lg:items-start lg:justify-between {class}"
                )
                data-page-header="true"
                data-page-header-navigation-layout=navigation_layout.as_str()
            >
                <div class="flex min-w-0 flex-col items-start gap-3 sm:flex-row">
                    {back.map(|back| view! { <div class="shrink-0 pt-1">{back}</div> })}
                    <div class="min-w-0 space-y-1">
                        <div class="flex flex-wrap items-center gap-2">
                            <h1 class="ld-text-display font-semibold tracking-tight text-base-content">
                                {move || title.get()}
                            </h1>
                            {freshness}
                        </div>
                        <p class="max-w-3xl text-sm text-base-content/75 sm:text-base">
                            {move || subtitle.get()}
                        </p>
                    </div>
                </div>
                <div class="flex flex-wrap items-end gap-2 lg:justify-end">
                    {dataset.map(|dataset| view! {
                        <div class="min-w-56" data-page-dataset-slot="true">{dataset}</div>
                    })}
                    {actions.map(|actions| view! {
                        <div class="flex items-center gap-2" data-page-actions-slot="true">
                            {actions}
                        </div>
                    })}
                </div>
            </header>
        }
        .into_any(),
        PageHeaderNavigationLayout::DedicatedRow => view! {
            <header
                class=format!(
                    "flex min-w-0 flex-col gap-3 border-b border-base-300 pb-4 {class}"
                )
                data-page-header="true"
                data-page-header-navigation-layout=navigation_layout.as_str()
            >
                {back.map(|back| view! {
                    <nav
                        class="flex min-w-0 flex-wrap items-center gap-2"
                        aria-label=move || navigation_label.get()
                        data-page-navigation-row="true"
                    >
                        {back}
                    </nav>
                })}
                <div class="flex min-w-0 flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                    <div class="min-w-0 flex-1 space-y-1">
                        <div class="flex flex-wrap items-center gap-2">
                            <h1 class="ld-text-display font-semibold tracking-tight text-base-content">
                                {move || title.get()}
                            </h1>
                            {freshness}
                        </div>
                        <p class="max-w-3xl text-sm text-base-content/75 sm:text-base">
                            {move || subtitle.get()}
                        </p>
                    </div>
                    <div class="flex min-w-0 flex-wrap items-end gap-2 lg:justify-end">
                        {dataset.map(|dataset| view! {
                            <div class="min-w-56" data-page-dataset-slot="true">{dataset}</div>
                        })}
                        {actions.map(|actions| view! {
                            <div class="flex items-center gap-2" data-page-actions-slot="true">
                                {actions}
                            </div>
                        })}
                    </div>
                </div>
            </header>
        }
        .into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_header_navigation_layout_keeps_inline_as_the_default() {
        assert_eq!(
            PageHeaderNavigationLayout::default(),
            PageHeaderNavigationLayout::InlineResponsive
        );
        assert_eq!(
            PageHeaderNavigationLayout::default().as_str(),
            "inline-responsive"
        );
    }

    #[test]
    fn dedicated_navigation_layout_has_a_stable_runtime_marker() {
        assert_eq!(
            PageHeaderNavigationLayout::DedicatedRow.as_str(),
            "dedicated-row"
        );
    }
}
