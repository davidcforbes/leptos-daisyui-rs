use super::types::{
    ResultListItem, current_result_item, keyed_option_dom_id, move_result_key,
    reconcile_result_key, validate_result_list_items,
};
use crate::merge_classes;
use leptos::{ev::KeyboardEvent, html::Div, prelude::*};
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::JsCast;
use web_sys::{ScrollIntoViewOptions, ScrollLogicalPosition};

/// Per-instance sequence so every result list gets unique option DOM ids for
/// `aria-activedescendant` wiring.
static RESULT_LIST_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResultReplacementPolicy {
    ResetFirst,
    // Constructed by the public keyed wrapper in Task 3 of the approved plan.
    #[allow(dead_code)]
    PreserveKey,
}

#[allow(clippy::too_many_arguments)]
fn result_option<T>(
    instance_id: u64,
    key: String,
    items: Signal<Vec<ResultListItem<T>>>,
    selected: RwSignal<Option<String>>,
    hover: RwSignal<Option<String>>,
    on_select: Option<Callback<ResultListItem<T>>>,
    on_selection_change: Option<Callback<Option<String>>>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let dom_id = keyed_option_dom_id(instance_id, &key);
    let selected_attr_key = key.clone();
    let selected_class_key = key.clone();
    let hover_key = key.clone();
    let enter_key = key.clone();
    let leave_key = key.clone();
    let click_key = key.clone();
    let title_key = key.clone();
    let has_secondary_key = key.clone();
    let secondary_key = key.clone();

    view! {
        <div
            id=dom_id
            data-result-key=key
            role="option"
            aria-selected=move || (selected.get().as_deref() == Some(selected_attr_key.as_str())).to_string()
            class=move || {
                let is_selected = selected.get().as_deref() == Some(selected_class_key.as_str());
                let is_hovered = hover.get().as_deref() == Some(hover_key.as_str());
                merge_classes!(
                    "flex flex-col gap-1 px-3 py-2 cursor-pointer rounded-box",
                    if is_selected {
                        "bg-primary/10 text-primary"
                    } else if is_hovered {
                        "bg-base-200"
                    } else {
                        ""
                    }
                )
            }
            on:mouseenter=move |_| hover.set(Some(enter_key.clone()))
            on:mouseleave=move |_| {
                if hover.get_untracked().as_deref() == Some(leave_key.as_str()) {
                    hover.set(None);
                }
            }
            on:click=move |_| {
                let latest = items.get_untracked();
                if validate_result_list_items(&latest).is_err() {
                    return;
                }
                let Some(item) = current_result_item(&latest, &click_key) else {
                    return;
                };

                selected.set(Some(click_key.clone()));
                if let Some(callback) = on_selection_change {
                    callback.run(Some(click_key.clone()));
                }
                if let Some(callback) = on_select {
                    callback.run(item);
                }
            }
        >
            <span class="font-semibold text-sm truncate">
                {move || {
                    current_result_item(&items.get(), &title_key)
                        .map(|item| item.row.title)
                        .unwrap_or_default()
                }}
            </span>
            <span
                class="text-xs opacity-60 whitespace-normal break-words"
                style:display=move || {
                    current_result_item(&items.get(), &has_secondary_key)
                        .is_some_and(|item| !item.row.secondary_line().is_empty())
                        .then_some("inline")
                        .unwrap_or("none")
                }
            >
                {move || {
                    current_result_item(&items.get(), &secondary_key)
                        .map(|item| item.row.secondary_line().to_owned())
                        .unwrap_or_default()
                }}
            </span>
        </div>
    }
}

#[component]
pub(super) fn ResultListCore<T>(
    items: Signal<Vec<ResultListItem<T>>>,
    empty_message: Signal<String>,
    replacement_policy: ResultReplacementPolicy,
    #[prop(optional_no_strip)] on_select: Option<Callback<ResultListItem<T>>>,
    #[prop(optional_no_strip)] on_selection_change: Option<Callback<Option<String>>>,
    class: &'static str,
    node_ref: NodeRef<Div>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let instance_id = RESULT_LIST_SEQ.fetch_add(1, Ordering::Relaxed);
    let selected = RwSignal::new(None::<String>);
    let hover = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        let latest = items.get();
        let next = if validate_result_list_items(&latest).is_err() {
            None
        } else {
            match replacement_policy {
                ResultReplacementPolicy::ResetFirst => latest.first().map(|item| item.key.clone()),
                ResultReplacementPolicy::PreserveKey => {
                    let current = selected.get_untracked();
                    reconcile_result_key(current.as_deref(), &latest)
                }
            }
        };
        let changed = selected.get_untracked() != next;

        selected.set(next.clone());
        hover.set(None);
        if (changed || replacement_policy == ResultReplacementPolicy::ResetFirst)
            && let Some(callback) = on_selection_change
        {
            callback.run(next);
        }
    });

    // Keep the selected option visible after either selection or current-list
    // order changes. Native scrolling replaces the original Direct2D offset
    // calculations.
    Effect::new(move |_| {
        let _ = items.get();
        let Some(key) = selected.get() else {
            return;
        };
        if let Some(element) = node_ref.get_untracked() {
            let container = element.unchecked_ref::<web_sys::Element>();
            let selector = format!("#{}", keyed_option_dom_id(instance_id, &key));
            if let Ok(Some(target)) = container.query_selector(&selector) {
                let options = ScrollIntoViewOptions::new();
                options.set_block(ScrollLogicalPosition::Nearest);
                target.scroll_into_view_with_scroll_into_view_options(&options);
            }
        }
    });

    let on_keydown = move |event: KeyboardEvent| {
        let latest = items.get_untracked();
        if latest.is_empty() || validate_result_list_items(&latest).is_err() {
            return;
        }

        let current = selected.get_untracked();
        let next = match event.key().as_str() {
            "ArrowDown" => {
                event.prevent_default();
                Some(move_result_key(current.as_deref(), 1, &latest))
            }
            "ArrowUp" => {
                event.prevent_default();
                Some(move_result_key(current.as_deref(), -1, &latest))
            }
            "Home" => {
                event.prevent_default();
                Some(latest.first().map(|item| item.key.clone()))
            }
            "End" => {
                event.prevent_default();
                Some(latest.last().map(|item| item.key.clone()))
            }
            "Enter" => {
                event.prevent_default();
                if let Some(key) = current
                    && let Some(item) = current_result_item(&latest, &key)
                    && let Some(callback) = on_select
                {
                    callback.run(item);
                }
                None
            }
            _ => None,
        };

        if let Some(next) = next {
            selected.set(next.clone());
            if let Some(callback) = on_selection_change {
                callback.run(next);
            }
        }
    };

    let current_keys = move || {
        items
            .get()
            .into_iter()
            .map(|item| item.key)
            .collect::<Vec<_>>()
    };

    view! {
        <div
            node_ref=node_ref
            role="listbox"
            tabindex="0"
            aria-activedescendant=move || {
                let latest = items.get();
                if validate_result_list_items(&latest).is_err() {
                    None
                } else {
                    selected
                        .get()
                        .map(|key| keyed_option_dom_id(instance_id, &key))
                }
            }
            class=move || {
                merge_classes!(
                    "flex flex-col gap-2 max-h-80 overflow-y-auto rounded-box border border-base-300 bg-base-100 outline-none focus:ring-2 focus:ring-primary/50",
                    class
                )
            }
            on:keydown=on_keydown
        >
            <Show
                when=move || validate_result_list_items(&items.get()).is_ok()
                fallback=move || {
                    view! {
                        <div
                            role="alert"
                            data-result-list-key-error="true"
                            class="border border-error bg-error/10 p-4 text-sm text-error"
                        >
                            {move || {
                                validate_result_list_items(&items.get())
                                    .err()
                                    .map(|error| error.to_string())
                                    .unwrap_or_default()
                            }}
                        </div>
                    }
                }
            >
                <Show
                    when=move || !items.get().is_empty()
                    fallback=move || {
                        view! {
                            <div role="presentation" class="p-4 text-sm text-center opacity-60">
                                {move || empty_message.get()}
                            </div>
                        }
                    }
                >
                    <For
                        each=current_keys
                        key=|key| key.clone()
                        children=move |key| {
                            result_option(
                                instance_id,
                                key,
                                items,
                                selected,
                                hover,
                                on_select,
                                on_selection_change,
                            )
                        }
                    />
                </Show>
            </Show>
        </div>
    }
}
