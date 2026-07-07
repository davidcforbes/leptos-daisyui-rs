use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn EmptyStateDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Empty State"
            description="A centered icon, title, and muted subtitle for empty regions -- no results, nothing to do, connection lost -- with an optional action slot."
        >
            <Section title="Basic">
                <div class="w-full rounded-box border border-base-300">
                    <EmptyState
                        icon=Box::new(move || view! { <span>"📭"</span> }.into_any())
                        title="No messages"
                        subtitle="You're all caught up."
                    />
                </div>
            </Section>

            <Section title="With an action slot">
                <div class="w-full rounded-box border border-base-300">
                    <EmptyState
                        icon=Box::new(move || view! { <span>"🔍"</span> }.into_any())
                        title="No results"
                        subtitle="Try a different search term or clear your filters."
                    >
                        <Button color=ButtonColor::Primary>"Clear filters"</Button>
                    </EmptyState>
                </div>
            </Section>

            <Section title="Color overrides" row=true>
                <div class="w-72 rounded-box border border-base-300">
                    <EmptyState
                        icon=Box::new(move || view! { <span>"⚠️"</span> }.into_any())
                        title="Connection lost"
                        subtitle="Check your network and try again."
                        icon_color=EmptyStateColor::Warning
                        title_color=EmptyStateColor::Warning
                    >
                        <Button color=ButtonColor::Warning size=ButtonSize::Sm>"Retry"</Button>
                    </EmptyState>
                </div>
                <div class="w-72 rounded-box border border-base-300">
                    <EmptyState
                        icon=Box::new(move || view! { <span>"✅"</span> }.into_any())
                        title="All clear"
                        subtitle="Nothing needs your attention right now."
                        icon_color=EmptyStateColor::Success
                        title_color=EmptyStateColor::Success
                    />
                </div>
                <div class="w-72 rounded-box border border-base-300">
                    <EmptyState
                        icon=Box::new(move || view! { <span>"⛔"</span> }.into_any())
                        title="Not found"
                        subtitle="The item you're looking for doesn't exist."
                        icon_color=EmptyStateColor::Error
                        title_color=EmptyStateColor::Error
                    />
                </div>
            </Section>

            <Section title="Without an icon">
                <div class="w-full rounded-box border border-base-300">
                    <EmptyState title="Nothing here yet" subtitle="Create your first item to get started." />
                </div>
            </Section>
        </ContentLayout>
    }
}
