use leptos::prelude::*;

/// Section layout component for the demos
#[component]
pub fn Section(
    title: &'static str,
    #[prop(optional)] row: bool,
    #[prop(optional)] col: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <h2 class="text-xl font-semibold">{title}</h2>
        // gap-4 (16px), not gap-2 (8px): examples in a Section are separate
        // surfaces, and several carry their own padding directly on the
        // element that gets a background — daisyUI's `.alert` is 12px, the
        // demos' own `bg-base-200 p-4` panels are 16px. Internal <= external
        // (ldui-6qb) therefore needs the gap to be at least the largest of
        // those, or the examples merge into one another. 16px is that
        // maximum and is the canonical `M` step.
        <div class="flex gap-4 flex-wrap min-w-0" class:flex-col=col class:flex-row=row>
            {{ children() }}
        </div>
    }
}
