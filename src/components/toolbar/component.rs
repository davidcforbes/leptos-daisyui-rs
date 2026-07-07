use super::style::ToolbarSize;
use super::types::{ToolbarItem, visible_count_for_width};
use crate::components::button::{Button, ButtonColor, ButtonStyle};
use crate::components::dropdown::{Dropdown, DropdownContent};
use crate::components::join::Join;
use crate::components::tooltip::{Tooltip, TooltipPosition};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use web_sys::wasm_bindgen::JsCast;

/// Fallback overflow-button width (px) used only if the hidden measurement
/// clone can't be found in the DOM (should not happen in normal operation).
const OVERFLOW_FALLBACK_WIDTH: f64 = 32.0;

/// # Toolbar Component
///
/// A horizontal strip of icon/label command buttons and toggle buttons with
/// tooltips, a checked-underline accent, disabled state, and automatic
/// overflow collapse. Ported from d2d-ui's `controls::toolbar::Toolbar` (a
/// self-painting Direct2D control using fixed 32×32 DIP buttons) to a Leptos +
/// daisyUI composition: visible items render inside a [`Join`](crate::components::Join)
/// group of [`Button`](crate::components::Button)s, each optionally wrapped in
/// a [`Tooltip`](crate::components::Tooltip); items that don't fit the
/// available width spill into a [`Dropdown`](crate::components::Dropdown)
/// overflow menu ("⋯") instead of d2d-ui's `Menu`.
///
/// Overflow detection measures a hidden, always-fully-rendered clone of every
/// item (so per-item widths stay known even once some are hidden) with
/// `Element::get_bounding_client_rect`, then feeds the container width and
/// item widths into the pure [`visible_count_for_width`] function — a
/// `ResizeObserver` on the container re-runs the measurement whenever the
/// toolbar's available width changes.
///
/// # Example
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     let items = vec![
///         ToolbarItem::new("save", "💾").tooltip("Save"),
///         ToolbarItem::new("open", "📂").tooltip("Open"),
///         ToolbarItem::new("undo", "↶").tooltip("Undo").disabled(),
///         ToolbarItem::new("bold", "B").tooltip("Bold").toggle(true),
///     ];
///
///     view! {
///         <Toolbar
///             items=Signal::derive(move || items.clone())
///             on_item_click=Callback::new(|id: String| leptos::logging::log!("clicked {id}"))
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("relative flex items-center");
/// @source inline("invisible pointer-events-none absolute -left-full top-0 -z-10 flex items-center");
/// @source inline("inline-flex relative");
/// @source inline("pointer-events-none absolute inset-x-1 -bottom-0.5 h-0.5 rounded-full");
/// @source inline("bg-primary opacity-60");
/// @source inline("join join-item");
/// @source inline("btn btn-ghost btn-soft btn-xs btn-sm btn-md btn-lg");
/// @source inline("tooltip tooltip-bottom");
/// @source inline("dropdown dropdown-content dropdown-end menu menu-sm z-10 w-40 p-2 shadow rounded-box bg-base-100");
/// ```
///
/// ## Node References
/// - `node_ref` - References the outer toolbar `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Toolbar(
    /// Items to render, left-to-right. Items that don't fit the available
    /// width are automatically moved into an overflow ("⋯") dropdown.
    #[prop(into)]
    items: Signal<Vec<ToolbarItem>>,

    /// Size of each toolbar button (also scales the join-row density).
    #[prop(optional, into)]
    size: Signal<ToolbarSize>,

    /// Fired with the clicked item's `id` for any *enabled* item — whether
    /// currently visible or parked in the overflow menu.
    #[prop(optional)]
    on_item_click: Option<Callback<String>>,

    /// Additional CSS classes for the outer toolbar container.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer toolbar `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let shadow_ref: NodeRef<Div> = NodeRef::new();
    let visible_count = RwSignal::new(items.get_untracked().len());

    // Pure DOM read + pure-fn call: measures the always-rendered hidden
    // shadow row and the live container, then updates `visible_count`. No
    // logic here beyond marshalling numbers into `visible_count_for_width`.
    let do_measure = move || {
        let Some(container) = node_ref.get_untracked() else {
            return;
        };
        let container_el = container.unchecked_ref::<web_sys::Element>();
        let container_width = container_el.get_bounding_client_rect().width();

        let Some(shadow) = shadow_ref.get_untracked() else {
            return;
        };
        let shadow_el = shadow.unchecked_ref::<web_sys::Element>();

        let mut widths: Vec<f64> = Vec::new();
        if let Ok(nodes) = shadow_el.query_selector_all(".toolbar-item") {
            for i in 0..nodes.length() {
                if let Some(node) = nodes.item(i)
                    && let Ok(el) = node.dyn_into::<web_sys::Element>()
                {
                    widths.push(el.get_bounding_client_rect().width());
                }
            }
        }

        let overflow_width = shadow_el
            .query_selector(".toolbar-overflow-shadow")
            .ok()
            .flatten()
            .map(|el| el.get_bounding_client_rect().width())
            .unwrap_or(OVERFLOW_FALLBACK_WIDTH);

        // daisyUI's `join` layout has no gap between items (borders touch).
        let count = visible_count_for_width(container_width, &widths, 0.0, overflow_width);
        visible_count.set(count);
    };

    // Re-measure whenever the item list is replaced (length or label changes
    // can change per-item widths).
    Effect::new(move |_| {
        let _ = items.get();
        do_measure();
    });

    // Set up a `ResizeObserver` exactly once, when the outer container first
    // attaches to the DOM (CSR-only — a browser is always present). Reading
    // `node_ref.get()` here is what makes this effect re-run the one time it
    // flips from `None` to `Some`; it never flips back, so this never runs
    // its setup branch again for the life of the component.
    Effect::new(move |_| {
        let Some(container) = node_ref.get() else {
            return;
        };

        do_measure();

        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
            move |_entries: js_sys::Array, _observer: web_sys::ResizeObserver| {
                do_measure();
            },
        )
            as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);

        match web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) {
            Ok(observer) => {
                observer.observe(container.unchecked_ref::<web_sys::Element>());
                // `Closure`/`ResizeObserver` wrap JS values and are not
                // `Send`/`Sync`, but `on_cleanup` requires both (the
                // reactive graph is generic over native multithreaded use).
                // In practice this component only ever runs single-threaded
                // (wasm32 in the browser); `SendWrapper` documents and
                // encodes that assumption rather than working around it
                // silently.
                let guard = send_wrapper::SendWrapper::new((closure, observer));
                on_cleanup(move || {
                    let (closure, observer) = guard.take();
                    observer.disconnect();
                    drop(closure);
                });
            }
            Err(_) => drop(closure),
        }
    });

    let visible_items = move || -> Vec<(usize, ToolbarItem)> {
        let count = visible_count.get();
        items.get().into_iter().enumerate().take(count).collect()
    };
    let overflow_items = move || -> Vec<(usize, ToolbarItem)> {
        let count = visible_count.get();
        items.get().into_iter().enumerate().skip(count).collect()
    };
    let has_overflow = move || visible_count.get() < items.get().len();

    let fire_click = move |item: &ToolbarItem| {
        if item.enabled
            && let Some(cb) = on_item_click
        {
            cb.run(item.id.clone());
        }
    };

    view! {
        <div
            node_ref=node_ref
            role="toolbar"
            aria-orientation="horizontal"
            class=move || merge_classes!("relative flex items-center", class)
        >
            // Hidden measurement row: every item (and a clone of the overflow
            // button) rendered unconstrained so `do_measure` can read each
            // one's natural width even while some are hidden from view.
            <div
                node_ref=shadow_ref
                aria-hidden="true"
                class="invisible pointer-events-none absolute -left-full top-0 -z-10 flex items-center"
            >
                <For
                    each=move || items.get()
                    key=|item| item.id.clone()
                    children=move |item| {
                        view! {
                            <Button
                                attr:r#type="button"
                                attr:tabindex="-1"
                                size=Signal::derive(move || size.get().button_size())
                                class="toolbar-item join-item"
                            >
                                {item.label.clone()}
                            </Button>
                        }
                    }
                />
                <Button
                    attr:r#type="button"
                    attr:tabindex="-1"
                    size=Signal::derive(move || size.get().button_size())
                    class="toolbar-overflow-shadow join-item"
                >
                    "⋯"
                </Button>
            </div>

            <Join>
                <For
                    each=visible_items
                    key=|(idx, item)| (*idx, item.id.clone())
                    children=move |(_idx, item)| {
                        let checked = item.checked;
                        let enabled = item.enabled;
                        let tooltip_text = item.tooltip.clone();
                        let item_for_click = item.clone();
                        let button = view! {
                            <div class="inline-flex relative">
                                <Button
                                    attr:r#type="button"
                                    size=Signal::derive(move || size.get().button_size())
                                    color=Signal::derive(move || {
                                        if checked == Some(true) {
                                            ButtonColor::Primary
                                        } else {
                                            ButtonColor::Default
                                        }
                                    })
                                    style=Signal::derive(move || {
                                        if checked == Some(true) {
                                            ButtonStyle::Soft
                                        } else {
                                            ButtonStyle::Ghost
                                        }
                                    })
                                    disabled=Signal::derive(move || !enabled)
                                    class="toolbar-item join-item"
                                    on:click=move |_| fire_click(&item_for_click)
                                >
                                    {item.label.clone()}
                                </Button>
                                <Show when=move || checked == Some(true)>
                                    <span
                                        aria-hidden="true"
                                        class="pointer-events-none absolute inset-x-1 -bottom-0.5 h-0.5 rounded-full bg-primary"
                                    ></span>
                                </Show>
                            </div>
                        };
                        match tooltip_text {
                            Some(text) => {
                                view! {
                                    <Tooltip
                                        tip=text
                                        position=TooltipPosition::Bottom
                                    >
                                        {button}
                                    </Tooltip>
                                }
                                    .into_any()
                            }
                            None => button.into_any(),
                        }
                    }
                />

                <Show when=has_overflow>
                    <Dropdown class="join-item">
                        <button
                            type="button"
                            tabindex="0"
                            aria-label="More toolbar items"
                            class="btn btn-ghost join-item"
                        >
                            "⋯"
                        </button>
                        <DropdownContent
                            is_menu=true
                            class="dropdown-content dropdown-end menu menu-sm z-10 w-40 p-2 shadow rounded-box bg-base-100"
                        >
                            <For
                                each=overflow_items
                                key=|(idx, item)| (*idx, item.id.clone())
                                children=move |(_idx, item)| {
                                    let enabled = item.enabled;
                                    let checked = item.checked == Some(true);
                                    let label = item.label.clone();
                                    let item_for_click = item.clone();
                                    view! {
                                        <li>
                                            <button
                                                type="button"
                                                disabled=!enabled
                                                class:menu-active=checked
                                                on:click=move |_| fire_click(&item_for_click)
                                            >
                                                {if checked { format!("✓ {label}") } else { label }}
                                            </button>
                                        </li>
                                    }
                                }
                            />
                        </DropdownContent>
                    </Dropdown>
                </Show>
            </Join>
        </div>
    }
}
