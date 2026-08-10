use leptos::prelude::*;

/// Content layout component for the demos
#[component]
pub fn ContentLayout(
    title: &'static str,
    description: &'static str,
    /// The content to display within the layout
    children: Children,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            // `ld-text-display` (28px/36px), not Tailwind's `text-3xl`
            // (30px): the page title is the demo's own chrome, not a daisyUI
            // component, so it belongs on the shared `ui-tokens` type ramp.
            // 30px is off that ramp and was a real typography violation on
            // every page (ldui-gne).
            <h1 class="ld-text-display font-bold">{title}</h1>
            <p class="text-base-content/70">{description}</p>

            <div class="space-y-4">{{ children() }}</div>
        </div>
    }
}
