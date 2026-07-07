use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use std::sync::atomic::{AtomicU64, Ordering};

/// Per-instance sequence so each combobox gets a unique listbox id for
/// `aria-controls` / `aria-owns` wiring.
static COMBOBOX_SEQ: AtomicU64 = AtomicU64::new(0);

/// # Combobox Component
///
/// A combined dropdown and searchable input field for selecting from a list of options.
/// Built on daisyUI's dropdown, input, and menu components.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("dropdown input dropdown-content menu bg-base-200 rounded-box shadow");
/// ```
///
/// ## Node References
/// - `node_ref` - References the container div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Combobox(
    /// Current selected value
    #[prop(optional, into)]
    value: Signal<String>,

    /// List of options to display
    #[prop(optional, into)]
    options: Signal<Vec<String>>,

    /// Placeholder text
    #[prop(optional, into)]
    placeholder: Signal<String>,

    /// Callback when an option is selected
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,

    /// Whether the combobox is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the container div
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let (search_text, set_search_text) = signal(String::new());
    let (is_open, set_is_open) = signal(false);

    // Unique id for the popup listbox so the input can reference it via
    // `aria-controls` (WAI-ARIA 1.2 combobox pattern).
    let listbox_id = format!(
        "ld-combobox-listbox-{}",
        COMBOBOX_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    let controls_id = listbox_id.clone();

    let filtered_options = Signal::derive(move || {
        let search = search_text.get().to_lowercase();
        if search.is_empty() {
            options.get()
        } else {
            options
                .get()
                .into_iter()
                .filter(|opt| opt.to_lowercase().contains(&search))
                .collect()
        }
    });

    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("dropdown w-full", class)
        >
            <input
                type="text"
                role="combobox"
                aria-haspopup="listbox"
                aria-autocomplete="list"
                aria-expanded=move || {
                    if is_open.get() && !filtered_options.get().is_empty() {
                        "true"
                    } else {
                        "false"
                    }
                }
                aria-controls=move || {
                    (is_open.get() && !filtered_options.get().is_empty())
                        .then(|| controls_id.clone())
                }
                placeholder=move || {
                    let v = value.get();
                    if v.is_empty() { placeholder.get() } else { v }
                }

                value=move || search_text.get()
                disabled=disabled
                class="input input-bordered w-full"
                on:input=move |ev| {
                    set_search_text.set(event_target_value(&ev));
                    set_is_open.set(true);
                }

                on:focus=move |_| {
                    set_is_open.set(true);
                }

                on:blur=move |_| {
                    set_timeout(
                        move || {
                            set_is_open.set(false);
                        },
                        std::time::Duration::from_millis(200),
                    );
                }
            />

            {move || {
                if is_open.get() && !filtered_options.get().is_empty() {
                    view! {
                        <ul
                            id=listbox_id.clone()
                            role="listbox"
                            tabindex="0"
                            class="dropdown-content menu bg-base-200 rounded-box z-[1] w-full p-2 shadow max-h-60 overflow-auto"
                        >
                            {filtered_options
                                .get()
                                .into_iter()
                                .map(|option| {
                                    let option_clone = option.clone();
                                    view! {
                                        <li role="option">
                                            <a
                                                on:click=move |_| {
                                                    if let Some(ref callback) = on_select {
                                                        callback.run(option.clone());
                                                    }
                                                    set_search_text.set(String::new());
                                                    set_is_open.set(false);
                                                }
                                            >

                                                {option_clone}
                                            </a>
                                        </li>
                                    }
                                })
                                .collect_view()}

                        </ul>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}

        </div>
    }
}
