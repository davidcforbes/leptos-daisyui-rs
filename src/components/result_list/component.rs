use super::{
    core::{ResultListCore, ResultReplacementPolicy},
    types::{ResultListItem, ResultRow, current_result_item, result_row_key},
};
use leptos::{html::Div, prelude::*};

/// # ResultList Component
///
/// A flat, ranked, keyboard-navigable search-results picker. Ported from
/// d2d-ui's `controls::result_list::ResultList` (a self-painting Direct2D
/// control) to a Leptos + daisyUI listbox: each row shows a bold **title**
/// plus a wrapped secondary line (the `snippet` when present, else the
/// `subtitle`). Rows are naturally variable-height because the browser does
/// the word-wrap layout — none of d2d's manual row-height measurement or
/// scroll-offset math is needed.
///
/// Supports `ArrowUp`/`ArrowDown` (move one row, clamped at the ends — no
/// wraparound, matching d2d), `Home`/`End` (jump to the first/last row), and
/// `Enter` (activate the selected row via `on_select`). Hovering a row
/// previews it; clicking both selects and activates it. The selected row is
/// scrolled into view (`Element::scroll_into_view`, `block: "nearest"`) on
/// every selection change — native `overflow-y-auto` handles the rest.
///
/// # Example
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     let items = vec![
///         ResultRow { title: "index.md".into(), subtitle: "/docs".into(), snippet: String::new() },
///         ResultRow { title: "readme.md".into(), subtitle: "/".into(), snippet: "...matched text...".into() },
///     ];
///     view! {
///         <ResultList
///             items=Signal::derive(move || items.clone())
///             on_select=Callback::new(|row: ResultRow| leptos::logging::log!("selected {}", row.title))
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col gap-2 max-h-80 overflow-y-auto rounded-box border border-base-300 bg-base-100");
/// @source inline("outline-none focus:ring-2 focus:ring-primary/50");
/// @source inline("px-3 py-2 cursor-pointer");
/// @source inline("bg-primary/10 text-primary bg-base-200");
/// @source inline("font-semibold text-sm truncate");
/// @source inline("text-xs opacity-60 whitespace-normal break-words");
/// @source inline("p-4 text-sm text-center opacity-60");
/// ```
///
/// ## Node References
/// - `node_ref` - References the listbox container div ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ResultList(
    /// Ranked result rows to display, top to bottom.
    #[prop(optional, into)]
    items: Signal<Vec<ResultRow>>,

    /// Message shown in place of the list when `items` is empty.
    #[prop(optional, into, default = "No results found.".to_string().into())]
    empty_message: Signal<String>,

    /// Fired when a row is activated (`Enter` key or click) with a clone of
    /// the activated row.
    #[prop(optional)]
    on_select: Option<Callback<ResultRow>>,

    /// Fired whenever the highlighted row changes (keyboard nav, click, or
    /// the automatic reset that runs when `items` is replaced).
    #[prop(optional)]
    on_selection_change: Option<Callback<Option<usize>>>,

    /// Additional CSS classes for the listbox container.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the listbox container div.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let core_items: Signal<Vec<ResultListItem<(usize, ResultRow)>>> = Signal::derive(move || {
        items
            .get()
            .into_iter()
            .enumerate()
            .map(|(index, row)| {
                let (_, hash) = result_row_key(index, &row);
                ResultListItem::new(
                    format!("legacy-{index}-{hash:016x}"),
                    row.clone(),
                    (index, row),
                )
            })
            .collect()
    });

    let core_on_select = on_select.map(|callback| {
        Callback::new(move |item: ResultListItem<(usize, ResultRow)>| {
            callback.run(item.payload.1);
        })
    });
    let core_on_selection_change = on_selection_change.map(|callback| {
        Callback::new(move |key: Option<String>| {
            let index = key.as_deref().and_then(|key| {
                current_result_item(&core_items.get_untracked(), key).map(|item| item.payload.0)
            });
            callback.run(index);
        })
    });

    view! {
        <ResultListCore
            items=core_items
            empty_message=empty_message
            replacement_policy=ResultReplacementPolicy::ResetFirst
            on_select=core_on_select
            on_selection_change=core_on_selection_change
            class=class
            node_ref=node_ref
        />
    }
}
