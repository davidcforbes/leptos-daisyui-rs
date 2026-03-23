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
        <div class="flex gap-2 flex-wrap min-w-0" class:flex-col=col class:flex-row=row>
            {{ children() }}
        </div>
    }
}
