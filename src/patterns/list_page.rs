//! Full-width composition root for opinionated list pages.

use leptos::prelude::*;

/// Canonical page-width and vertical-rhythm classes.
pub const LIST_PAGE_BASE_CLASS: &str = "w-full min-w-0 space-y-4";

/// Merges caller classes with the canonical list-page contract.
pub fn list_page_class(class: &str) -> String {
    [LIST_PAGE_BASE_CLASS, class]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Establishes one full-width page surface and consistent vertical rhythm.
#[component]
pub fn ListPage(
    /// Stable page-contract identifier exposed to tests and audits.
    contract_id: &'static str,
    /// Additional page classes.
    #[prop(optional, into)]
    class: &'static str,
    /// Page header, filters, status, and data content.
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=list_page_class(class)
            data-list-page="true"
            data-page-contract=contract_id
        >
            {children()}
        </div>
    }
}
