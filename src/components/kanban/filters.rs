use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;

use super::types::*;

/// Filter bar component for kanban board with dropdowns
#[component]
pub fn KanbanFilterBar(
    /// Current filter state
    #[prop(into)]
    filters: Signal<KanbanFilters>,

    /// Setter for filter state
    set_filters: WriteSignal<KanbanFilters>,

    /// Available assignees for filtering
    #[prop(optional, into)]
    available_assignees: Signal<Vec<Assignee>>,

    /// Available labels for filtering
    #[prop(optional, into)]
    available_labels: Signal<Vec<Label>>,

    /// Enable search functionality
    #[prop(optional, default = true)]
    enable_search: bool,

    /// Search debounce in milliseconds. The filter signal updates only after
    /// the user stops typing for this long; clamped to a minimum of 1 ms to
    /// avoid a tight setTimeout loop. Set to a low value (e.g. 1) to behave
    /// effectively un-debounced.
    #[prop(optional, default = 300)]
    search_debounce_ms: u64,
) -> impl IntoView {
    // Live input value (reflects every keystroke for the controlled <input>).
    let (search_input, set_search_input) = signal(String::new());
    // Debounce handle so a new keystroke can cancel the pending timer.
    let (debounce_handle, set_debounce_handle) = signal(Option::<i32>::None);
    let debounce_ms = search_debounce_ms.max(1) as i32;

    let schedule_filter_update = move |query: String| {
        // Cancel any pending update.
        if let Some(handle) = debounce_handle.get_untracked()
            && let Some(window) = web_sys::window()
        {
            window.clear_timeout_with_handle(handle);
        }

        // Schedule a new update after the debounce window.
        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
            set_filters.update(|f| {
                f.search_query = if query.is_empty() { None } else { Some(query) };
            });
        });
        if let Some(window) = web_sys::window()
            && let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                debounce_ms,
            )
        {
            set_debounce_handle.set(Some(handle));
        }
    };

    view! {
        <div class="kanban-filters flex flex-wrap gap-4 items-center mb-4 p-4 bg-base-200 rounded-lg">
            // Search input
            <Show when=move || enable_search>
                <div class="flex flex-col gap-2 flex-1 min-w-64">
                    <div class="input-group">
                        <input
                            type="text"
                            placeholder="Search cards..."
                            class="input input-bordered input-sm w-full"
                            prop:value=move || search_input.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_search_input.set(value.clone());
                                schedule_filter_update(value);
                            }
                        />
                        <Show when=move || !search_input.get().is_empty()>
                            <button
                                class="btn btn-square btn-sm"
                                on:click=move |_| {
                                    set_search_input.set(String::new());
                                    schedule_filter_update(String::new());
                                }
                            >
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke-width="1.5"
                                    stroke="currentColor"
                                    class="w-4 h-4"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        d="M6 18L18 6M6 6l12 12"
                                    />
                                </svg>
                            </button>
                        </Show>
                    </div>
                </div>
            </Show>

            // Priority filter
            <div class="dropdown">
                <label tabindex="0" class="btn btn-sm btn-outline">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-4 h-4"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M3 4.5h14.25M3 9h9.75M3 13.5h5.25m5.25-.75L17.25 9m0 0L21 12.75M17.25 9v12"
                        />
                    </svg>
                    "Priority"
                    <Show when=move || !filters.get().priorities.is_empty()>
                        <span class="badge badge-xs ml-1">{move || filters.get().priorities.len()}</span>
                    </Show>
                </label>
                <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-52 mt-1">
                    {[Priority::Low, Priority::Medium, Priority::High, Priority::Critical]
                        .iter()
                        .map(|priority| {
                            let p = *priority;
                            view! {
                                <li>
                                    <label class="label cursor-pointer">
                                        <input
                                            type="checkbox"
                                            class="checkbox checkbox-sm"
                                            checked=move || filters.get().priorities.contains(&p)
                                            on:change=move |_| {
                                                set_filters.update(|f| {
                                                    if f.priorities.contains(&p) {
                                                        f.priorities.retain(|x| x != &p);
                                                    } else {
                                                        f.priorities.push(p);
                                                    }
                                                });
                                            }
                                        />
                                        <span class="text-sm flex-1">{p.as_str()}</span>
                                    </label>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>

            // Assignee filter
            <Show when=move || !available_assignees.get().is_empty()>
                <div class="dropdown">
                    <label tabindex="0" class="btn btn-sm btn-outline">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                            class="w-4 h-4"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z"
                            />
                        </svg>
                        "Assignee"
                        <Show when=move || !filters.get().assignee_ids.is_empty()>
                            <span class="badge badge-xs ml-1">{move || filters.get().assignee_ids.len()}</span>
                        </Show>
                    </label>
                    <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-52 mt-1">
                        <For
                            each=move || available_assignees.get()
                            key=|a| a.id.clone()
                            children=move |assignee| {
                                let assignee_id = assignee.id.clone();
                                let assignee_id_for_check = assignee_id.clone();
                                let assignee_id_for_change = assignee_id.clone();
                                let assignee_name = assignee.name.clone();
                                view! {
                                    <li>
                                        <label class="label cursor-pointer">
                                            <input
                                                type="checkbox"
                                                class="checkbox checkbox-sm"
                                                checked=move || filters.get().assignee_ids.contains(&assignee_id_for_check)
                                                on:change=move |_| {
                                                    let id = assignee_id_for_change.clone();
                                                    set_filters.update(|f| {
                                                        if f.assignee_ids.contains(&id) {
                                                            f.assignee_ids.retain(|x| x != &id);
                                                        } else {
                                                            f.assignee_ids.push(id);
                                                        }
                                                    });
                                                }
                                            />
                                            <span class="text-sm flex-1">{assignee_name.clone()}</span>
                                        </label>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </div>
            </Show>

            // Label filter
            <Show when=move || !available_labels.get().is_empty()>
                <div class="dropdown">
                    <label tabindex="0" class="btn btn-sm btn-outline">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                            class="w-4 h-4"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M9.568 3H5.25A2.25 2.25 0 003 5.25v4.318c0 .597.237 1.17.659 1.591l9.581 9.581c.699.699 1.78.872 2.607.33a18.095 18.095 0 005.223-5.223c.542-.827.369-1.908-.33-2.607L11.16 3.66A2.25 2.25 0 009.568 3z"
                            />
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M6 6h.008v.008H6V6z"
                            />
                        </svg>
                        "Label"
                        <Show when=move || !filters.get().label_ids.is_empty()>
                            <span class="badge badge-xs ml-1">{move || filters.get().label_ids.len()}</span>
                        </Show>
                    </label>
                    <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-52 mt-1">
                        <For
                            each=move || available_labels.get()
                            key=|l| l.id.clone()
                            children=move |label| {
                                let label_id = label.id.clone();
                                let label_id_for_check = label_id.clone();
                                let label_id_for_change = label_id.clone();
                                let label_name = label.name.clone();
                                let label_color = label.color.clone();
                                view! {
                                    <li>
                                        <label class="label cursor-pointer">
                                            <input
                                                type="checkbox"
                                                class="checkbox checkbox-sm"
                                                checked=move || filters.get().label_ids.contains(&label_id_for_check)
                                                on:change=move |_| {
                                                    let id = label_id_for_change.clone();
                                                    set_filters.update(|f| {
                                                        if f.label_ids.contains(&id) {
                                                            f.label_ids.retain(|x| x != &id);
                                                        } else {
                                                            f.label_ids.push(id);
                                                        }
                                                    });
                                                }
                                            />
                                            <span
                                                class="badge badge-sm"
                                                style=move || {
                                                    label_color.clone().map(|c| format!("background-color: {}", c))
                                                }
                                            >
                                                {label_name.clone()}
                                            </span>
                                        </label>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </div>
            </Show>

            // Filter logic toggle
            <div class="join">
                <button
                    class="btn btn-xs join-item"
                    class:btn-active=move || filters.get().filter_logic == FilterLogic::And
                    on:click=move |_| {
                        set_filters.update(|f| f.filter_logic = FilterLogic::And);
                    }
                >
                    "AND"
                </button>
                <button
                    class="btn btn-xs join-item"
                    class:btn-active=move || filters.get().filter_logic == FilterLogic::Or
                    on:click=move |_| {
                        set_filters.update(|f| f.filter_logic = FilterLogic::Or);
                    }
                >
                    "OR"
                </button>
            </div>

            // Clear all filters button
            <button
                class="btn btn-xs btn-ghost"
                on:click=move |_| {
                    set_search_input.set(String::new());
                    // Cancel a pending debounced search so it doesn't
                    // overwrite the cleared filter state.
                    if let Some(handle) = debounce_handle.get_untracked()
                        && let Some(window) = web_sys::window()
                    {
                        window.clear_timeout_with_handle(handle);
                        set_debounce_handle.set(None);
                    }
                    set_filters.set(KanbanFilters::new());
                }
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke-width="1.5"
                    stroke="currentColor"
                    class="w-4 h-4"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M6 18L18 6M6 6l12 12"
                    />
                </svg>
                "Clear"
            </button>
        </div>
    }
}
