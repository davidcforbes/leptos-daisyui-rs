use leptos::prelude::*;

use crate::components::{Button, ButtonSize};

/// Horizontal filter tab bar with an optional call-to-action button on the right.
///
/// Used on nearly every page to filter table data by category or status.
///
/// # Example
/// ```ignore
/// let active = RwSignal::new(0usize);
/// view! {
///     <FilterTabs
///         tabs=vec!["All", "Active", "Pending", "Completed"]
///         active_index=active
///         cta_label="+ Create New".to_string()
///         on_cta=move |_| { /* navigate or open modal */ }
///     />
/// }
/// ```
#[component]
pub fn FilterTabs(
    /// Tab labels to display.
    tabs: Vec<&'static str>,
    /// Signal tracking which tab is currently active (0-indexed).
    active_index: RwSignal<usize>,
    /// Optional label for the CTA button on the right.
    #[prop(optional, into)]
    cta_label: Option<String>,
    /// Optional click handler for the CTA button.
    #[prop(optional)]
    on_cta: Option<Callback<()>>,
    /// Optional additional CSS class for the CTA button (e.g. "btn-error" for Export).
    #[prop(optional, into)]
    cta_class: Option<String>,
) -> impl IntoView {
    let cta_view = cta_label.map(|label| {
        let cls: &'static str = match cta_class.as_deref() {
            Some(c) if !c.is_empty() => Box::leak(c.to_string().into_boxed_str()),
            _ => "bg-emerald-500 hover:bg-emerald-600 text-white border-none",
        };
        view! {
            <Button
                size=ButtonSize::Sm
                class=cls
                on:click=move |_| {
                    if let Some(cb) = &on_cta {
                        cb.run(());
                    }
                }
            >
                {label}
            </Button>
        }
    });

    view! {
        <div class="flex items-center gap-2 flex-wrap">
            <div class="flex gap-1 flex-wrap flex-1">
                {tabs
                    .into_iter()
                    .enumerate()
                    .map(|(i, label)| {
                        let is_active = Memo::new(move |_| active_index.get() == i);
                        view! {
                            <button
                                class=move || {
                                    if is_active.get() {
                                        "btn btn-sm bg-emerald-500 hover:bg-emerald-600 text-white border-none rounded-full"
                                    } else {
                                        "btn btn-sm btn-ghost rounded-full"
                                    }
                                }
                                on:click=move |_| active_index.set(i)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
            {cta_view}
        </div>
    }
}
