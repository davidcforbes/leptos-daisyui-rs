//! `<MarkdownInline>` — compact inline renderer for hover-cards and chips.
//!
//! Thin wrapper over [`MarkdownView`] with `inline=true`.  Kept as its own
//! component so the consumer doesn't have to know about the inline flag,
//! and so the API surface can grow independently if hover-card-specific
//! behavior is needed (truncation, max-line, etc.).

use leptos::prelude::*;

use super::view::MarkdownView;

#[component]
pub fn MarkdownInline(
    /// Reactive markdown source.
    #[prop(into)]
    source: Signal<String>,
) -> impl IntoView {
    view! { <MarkdownView source=source inline=true /> }
}
