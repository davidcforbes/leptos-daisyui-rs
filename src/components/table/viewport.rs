use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// The one horizontal-overflow class every table viewport in this library
/// shares. `DataTable`'s private scroll wrapper aliases this constant, so a
/// bare [`Table`](super::Table) wrapped in a [`TableViewport`] and a `DataTable` clip and
/// scroll identically at constrained widths.
pub const TABLE_VIEWPORT_CLASS: &str = "overflow-x-auto";

/// # Table Viewport Component
///
/// A horizontal scroll viewport for a bare [`Table`](super::Table). [`Table`](super::Table) intentionally
/// returns a bare `<table>` element (its API is the table root, nothing
/// around it), which means a `table-fixed` percentage layout at a narrow
/// viewport squashes its columns into each other instead of scrolling —
/// headers, cells and action buttons visibly overlap. `DataTable` already
/// solves this with a private `overflow-x-auto` wrapper; this component is
/// that same behavior extracted into a public, composable frame so screens
/// that build their tables by hand get the identical contract.
///
/// Structure: an outer scroll viewport (`overflow-x-auto`) around an inner
/// content div that carries the optional `min_content_width` as an inline
/// `min-width` style. The minimum belongs on the *content*, not the viewport
/// — a `min-width` on the scrolling element itself would widen the viewport
/// rather than make its content scrollable. Because the width is an inline
/// style, callers never need a per-width arbitrary Tailwind class (and so no
/// per-caller `@source inline` entry).
///
/// The minimum width itself stays a caller decision: it is domain knowledge
/// (how many columns, which of them carry buttons) that the library cannot
/// guess. Choose it from the widest fixed-content column — typically the
/// action column: two `btn-xs` buttons side by side need roughly 140px, so a
/// `table-fixed` table giving its action column 16% needs about 880px total.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("overflow-x-auto");
/// ```
///
/// ## Node References
/// - `node_ref` - References the outer scroll viewport ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn TableViewport(
    /// Minimum width the table content must occupy, as any CSS length
    /// (e.g. `"880px"`). Below this the viewport scrolls horizontally
    /// instead of letting percentage columns squash into each other. The
    /// default (empty) applies no minimum: content narrower than the
    /// viewport lays out exactly as it would unwrapped.
    #[prop(optional, into)]
    min_content_width: Signal<String>,

    /// Additional CSS classes on the scroll viewport
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the outer scroll viewport element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Table content — typically a [`Table`](super::Table)
    children: Children,
) -> impl IntoView {
    let content_style = move || {
        let width = min_content_width.get();
        (!width.is_empty()).then(|| format!("min-width: {width}"))
    };
    view! {
        <div node_ref=node_ref class=move || merge_classes!(TABLE_VIEWPORT_CLASS, class)>
            <div style=content_style>{children()}</div>
        </div>
    }
}
