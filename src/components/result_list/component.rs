use super::types::{ResultRow, move_selection, select_first, select_last};
use crate::merge_classes;
use leptos::{ev::KeyboardEvent, html::Div, prelude::*};
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::JsCast;
use web_sys::{ScrollIntoViewOptions, ScrollLogicalPosition};

/// Per-instance sequence so each `ResultList` gets unique option DOM ids for
/// `aria-activedescendant` wiring (WAI-ARIA listbox pattern).
static RESULT_LIST_SEQ: AtomicU64 = AtomicU64::new(0);

/// Build the DOM id for row `i` of the `instance`-th `ResultList` on the page.
fn option_dom_id(instance: u64, i: usize) -> String {
    format!("ld-result-list-{instance}-option-{i}")
}

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
/// @source inline("flex flex-col gap-0.5 max-h-80 overflow-y-auto rounded-box border border-base-300 bg-base-100");
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
    let instance_id = RESULT_LIST_SEQ.fetch_add(1, Ordering::Relaxed);

    let (selected, set_selected) = signal(None::<usize>);
    let (hover, set_hover) = signal(None::<usize>);

    // Whenever the item list is replaced, reset selection to the first row
    // (or none, if now empty) and clear hover — mirrors d2d-ui's
    // `ResultList::set_items`.
    Effect::new(move |_| {
        let len = items.get().len();
        let next = select_first(len);
        set_selected.set(next);
        set_hover.set(None);
        if let Some(cb) = on_selection_change {
            cb.run(next);
        }
    });

    // Keep the selected row visible as selection moves via the keyboard.
    // Native `overflow-y-auto` + `scroll_into_view` replaces d2d's manual
    // row-height measurement / scroll-offset math.
    Effect::new(move |_| {
        let Some(idx) = selected.get() else {
            return;
        };
        if let Some(el) = node_ref.get_untracked() {
            let container = el.unchecked_ref::<web_sys::Element>();
            let selector = format!("#{}", option_dom_id(instance_id, idx));
            if let Ok(Some(target)) = container.query_selector(&selector) {
                let opts = ScrollIntoViewOptions::new();
                opts.set_block(ScrollLogicalPosition::Nearest);
                target.scroll_into_view_with_scroll_into_view_options(&opts);
            }
        }
    });

    let on_keydown = move |ev: KeyboardEvent| {
        let len = items.get_untracked().len();
        if len == 0 {
            return;
        }
        let next = match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                Some(move_selection(selected.get_untracked(), 1, len))
            }
            "ArrowUp" => {
                ev.prevent_default();
                Some(move_selection(selected.get_untracked(), -1, len))
            }
            "Home" => {
                ev.prevent_default();
                Some(select_first(len))
            }
            "End" => {
                ev.prevent_default();
                Some(select_last(len))
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(idx) = selected.get_untracked()
                    && let Some(row) = items.get_untracked().get(idx).cloned()
                    && let Some(cb) = on_select
                {
                    cb.run(row);
                }
                None
            }
            _ => None,
        };
        if let Some(next) = next {
            set_selected.set(next);
            if let Some(cb) = on_selection_change {
                cb.run(next);
            }
        }
    };

    // Computed as a plain closure (rather than inline inside the `view!`
    // macro) so the `::<Vec<_>>` turbofish's `<`/`>` tokens don't confuse the
    // macro's RSX-style tag parser.
    let indexed_items = move || -> Vec<(usize, ResultRow)> { items.get().into_iter().enumerate().collect() };

    view! {
        <div
            node_ref=node_ref
            role="listbox"
            tabindex="0"
            aria-activedescendant=move || {
                selected.get().map(|i| option_dom_id(instance_id, i))
            }
            class=move || {
                merge_classes!(
                    "flex flex-col gap-0.5 max-h-80 overflow-y-auto rounded-box border border-base-300 bg-base-100 outline-none focus:ring-2 focus:ring-primary/50",
                    class
                )
            }
            on:keydown=on_keydown
        >
            <Show
                when=move || !items.get().is_empty()
                fallback=move || {
                    view! {
                        <div class="p-4 text-sm text-center opacity-60">
                            {move || empty_message.get()}
                        </div>
                    }
                }
            >
                <For
                    each=indexed_items
                    key=|(i, _row)| *i
                    children=move |(i, row)| {
                        let title = row.title.clone();
                        let secondary = row.secondary_line().to_string();
                        let has_secondary = !secondary.is_empty();
                        view! {
                            <div
                                id=option_dom_id(instance_id, i)
                                role="option"
                                aria-selected=move || (selected.get() == Some(i)).to_string()
                                class=move || {
                                    let sel = selected.get() == Some(i);
                                    let hov = hover.get() == Some(i);
                                    merge_classes!(
                                        "flex flex-col gap-0.5 px-3 py-2 cursor-pointer rounded-box",
                                        if sel {
                                            "bg-primary/10 text-primary"
                                        } else if hov {
                                            "bg-base-200"
                                        } else {
                                            ""
                                        }
                                    )
                                }
                                on:mouseenter=move |_| set_hover.set(Some(i))
                                on:mouseleave=move |_| {
                                    if hover.get_untracked() == Some(i) {
                                        set_hover.set(None);
                                    }
                                }
                                on:click=move |_| {
                                    set_selected.set(Some(i));
                                    if let Some(cb) = on_selection_change {
                                        cb.run(Some(i));
                                    }
                                    if let Some(cb) = on_select {
                                        cb.run(row.clone());
                                    }
                                }
                            >
                                <span class="font-semibold text-sm truncate">{title}</span>
                                <Show when=move || has_secondary>
                                    <span class="text-xs opacity-60 whitespace-normal break-words">
                                        {secondary.clone()}
                                    </span>
                                </Show>
                            </div>
                        }
                    }
                />
            </Show>
        </div>
    }
}
