use super::style::{TabOrientation, TabPlacement, TabSize, TabVariant};
use crate::merge_classes;
use leptos::{
    ev,
    html::{A, Div, Input},
    prelude::*,
};
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::JsCast;

static TAB_REGISTRATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
struct RegisteredTab {
    registration_id: u64,
    key: String,
    disabled: Signal<bool>,
}

/// Direction used by the controlled tabset's pure roving-focus model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TabMove {
    Next,
    Previous,
    Home,
    End,
}

/// Returns the next enabled index, wrapping for directional movement.
pub(super) fn next_enabled_tab(
    current: Option<usize>,
    disabled: &[bool],
    movement: TabMove,
) -> Option<usize> {
    if disabled.is_empty() || disabled.iter().all(|disabled| *disabled) {
        return None;
    }
    match movement {
        TabMove::Home => disabled.iter().position(|disabled| !disabled),
        TabMove::End => disabled.iter().rposition(|disabled| !disabled),
        TabMove::Next | TabMove::Previous => {
            let length = disabled.len();
            let start = current.unwrap_or_else(|| {
                if matches!(movement, TabMove::Next) {
                    length - 1
                } else {
                    0
                }
            });
            (1..=length)
                .map(|offset| {
                    if matches!(movement, TabMove::Next) {
                        (start + offset) % length
                    } else {
                        (start + length - (offset % length)) % length
                    }
                })
                .find(|index| !disabled[*index])
        }
    }
}

fn encode_tab_key(key: &str) -> String {
    key.as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Stable DOM id for one controlled tab.
pub(super) fn tab_dom_id(tabset_id: &str, key: &str) -> String {
    format!("{tabset_id}-tab-{}", encode_tab_key(key))
}

/// Stable DOM id for the panel controlled by one tab.
pub(super) fn tab_panel_dom_id(tabset_id: &str, key: &str) -> String {
    format!("{tabset_id}-panel-{}", encode_tab_key(key))
}

#[derive(Clone, Copy)]
struct TabSetContext {
    id: StoredValue<String>,
    label: Signal<String>,
    selected_key: Signal<String>,
    on_select: Callback<String>,
    orientation: Signal<TabOrientation>,
    registrations: RwSignal<Vec<RegisteredTab>, LocalStorage>,
    focus_key: RwSignal<Option<String>, LocalStorage>,
    restore_focus: RwSignal<bool>,
}

impl TabSetContext {
    fn id(self) -> String {
        self.id.get_value()
    }

    fn register(self, key: String, disabled: Signal<bool>) -> u64 {
        let registration_id = TAB_REGISTRATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        self.registrations.update(|registrations| {
            assert!(
                registrations
                    .iter()
                    .all(|registration| registration.key != key),
                "TabSet keys must be unique; duplicate key: {key}"
            );
            registrations.push(RegisteredTab {
                registration_id,
                key,
                disabled,
            });
        });
        registration_id
    }

    fn unregister(self, registration_id: u64, tab_id: &str) {
        let was_focused = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .is_some_and(|element| element.id() == tab_id);
        if was_focused {
            self.restore_focus.set(true);
        }
        self.registrations.update(|registrations| {
            registrations.retain(|registration| registration.registration_id != registration_id)
        });
    }

    fn enabled_registration(self, key: &str) -> bool {
        self.registrations.with(|registrations| {
            registrations
                .iter()
                .find(|registration| registration.key == key)
                .is_some_and(|registration| !registration.disabled.get())
        })
    }

    fn first_enabled_key(self) -> Option<String> {
        self.registrations.with(|registrations| {
            registrations
                .iter()
                .find(|registration| !registration.disabled.get())
                .map(|registration| registration.key.clone())
        })
    }

    fn effective_selected_key(self) -> Option<String> {
        let selected = self.selected_key.get();
        self.enabled_registration(&selected)
            .then_some(selected)
            .or_else(|| self.first_enabled_key())
    }

    fn effective_focus_key(self) -> Option<String> {
        self.focus_key
            .get()
            .filter(|key| self.enabled_registration(key))
            .or_else(|| self.effective_selected_key())
    }

    fn is_selected(self, key: &str) -> bool {
        self.effective_selected_key().as_deref() == Some(key)
    }

    fn is_roving_stop(self, key: &str) -> bool {
        self.effective_focus_key().as_deref() == Some(key)
    }

    fn set_focused(self, key: &str) {
        if self.enabled_registration(key) {
            self.focus_key.set(Some(key.to_owned()));
        }
    }

    fn select(self, key: &str) {
        if self.enabled_registration(key) {
            self.focus_key.set(Some(key.to_owned()));
            self.on_select.run(key.to_owned());
        }
    }

    fn move_focus(self, key: &str, movement: TabMove) {
        let registrations = self.registrations.get_untracked();
        let disabled = registrations
            .iter()
            .map(|registration| registration.disabled.get_untracked())
            .collect::<Vec<_>>();
        let current = registrations
            .iter()
            .position(|registration| registration.key == key);
        let Some(next) = next_enabled_tab(current, &disabled, movement) else {
            return;
        };
        let next_key = registrations[next].key.clone();
        self.focus_key.set(Some(next_key.clone()));
        focus_tab(self.id(), next_key);
    }
}

fn focus_tab(tabset_id: String, key: String) {
    let id = tab_dom_id(&tabset_id, &key);
    request_animation_frame(move || {
        let Some(element) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id(&id))
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return;
        };
        let _ = element.focus();
    });
}

/// Controlled, accessible owner for one tab list and its panels.
///
/// The caller owns `selected_key`; pointer and keyboard activation emit one
/// proposed replacement through `on_select`. Arrow keys move focus only, while
/// Enter or Space selects the focused tab. Put one [`Tabs`] and its sibling
/// [`TabPanel`] components inside this owner.
#[component]
pub fn TabSet(
    /// Stable DOM id prefix used for tab/panel relationships.
    #[prop(into)]
    id: String,
    /// Localized accessible name for the tab list.
    #[prop(into)]
    label: Signal<String>,
    /// Caller-controlled stable selected key.
    #[prop(into)]
    selected_key: Signal<String>,
    /// Proposes one stable replacement key after activation or fallback.
    on_select: Callback<String>,
    /// Keyboard and ARIA orientation.
    #[prop(into, default = Signal::stored(TabOrientation::Horizontal))]
    orientation: Signal<TabOrientation>,
    /// Additional classes for the tabset owner.
    #[prop(optional, into)]
    class: &'static str,
    /// Reference to the tabset owner.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
    /// One [`Tabs`] list followed by its [`TabPanel`] siblings.
    children: Children,
) -> impl IntoView {
    assert!(!id.trim().is_empty(), "TabSet id must not be empty");
    let context = TabSetContext {
        id: StoredValue::new(id),
        label,
        selected_key,
        on_select,
        orientation,
        registrations: RwSignal::new_local(Vec::new()),
        focus_key: RwSignal::new_local(None),
        restore_focus: RwSignal::new(false),
    };
    provide_context(context);

    let previous_selected = StoredValue::new(selected_key.get_untracked());
    Effect::new(move |_| {
        let selected = selected_key.get();
        let selected_changed = previous_selected.get_value() != selected;
        previous_selected.set_value(selected.clone());
        let effective_selected = context.effective_selected_key();

        if selected_changed && context.enabled_registration(&selected) {
            context.focus_key.set(Some(selected.clone()));
        }

        let focus_is_valid = context
            .focus_key
            .get_untracked()
            .is_some_and(|key| context.enabled_registration(&key));
        if !focus_is_valid {
            context.focus_key.set(effective_selected.clone());
        }

        if let Some(fallback) = effective_selected
            && !context.enabled_registration(&selected)
        {
            context.on_select.run(fallback.clone());
            if context.restore_focus.get_untracked() {
                context.restore_focus.set(false);
                focus_tab(context.id(), fallback);
            }
        } else {
            context.restore_focus.set(false);
        }
    });

    view! {
        <div
            node_ref=node_ref
            id=move || context.id()
            class=move || merge_classes!("min-w-0", class)
            data-tabset="controlled"
            data-tabset-id=move || context.id()
            data-tabset-selected=move || context.effective_selected_key()
        >
            {children()}
        </div>
    }
}

/// # Tabs Component
///
/// A reactive Leptos wrapper for daisyUI's tabs component that provides
/// navigation controls for organizing content into switchable panels.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("tabs tab tab-active tab-disabled tabs-box tabs-border tabs-lift tabs-top tabs-bottom");
/// ```
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Tabs(
    /// Size variant for tab dimensions
    #[prop(optional, into)]
    size: Signal<TabSize>,

    /// Visual style variant
    #[prop(optional, into)]
    variant: Signal<TabVariant>,

    /// Tabs placement
    #[prop(optional, into)]
    placement: Signal<TabPlacement>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Tab content
    children: Children,
) -> impl IntoView {
    let context = use_context::<TabSetContext>();
    view! {
        <div
            node_ref=node_ref
            role=context.map(|_| "tablist")
            aria-label=move || context.map(|context| context.label.get())
            aria-orientation=move || {
                context.map(|context| context.orientation.get().as_str())
            }
            data-tab-mode=context.map_or("layout", |_| "controlled")
            data-tab-orientation=move || {
                context.map(|context| context.orientation.get().as_str())
            }
            class=move || {
                merge_classes!(
                    "tabs",
                    context.map_or("", |context| match context.orientation.get() {
                        TabOrientation::Horizontal => "max-w-full flex-nowrap overflow-x-auto overscroll-x-contain",
                        TabOrientation::Vertical => "max-h-full flex-col overflow-y-auto",
                    }),
                    size.get().as_str(),
                    variant.get().as_str(),
                    placement.get().as_str(),
                    class
                )
            }
        >
            {children()}
        </div>
    }
}

/// # Tab Component
///
/// A reactive Leptos wrapper for individual tab items with click handling
/// and active state.
///
/// ## Node References
/// - `node_ref` - References the anchor element ([HTMLAnchorElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement))
#[component]
pub fn Tab(
    /// Stable key. Required inside a controlled [`TabSet`]; a `Tab` that
    /// sees a `TabSet` context but has no `tab_key` (including one seen
    /// via context leaking to a later sibling on the same page, a known
    /// Leptos non-island-component quirk) renders as an uncontrolled tab
    /// instead of panicking.
    #[prop(optional, into)]
    tab_key: Option<String>,

    /// Whether this tab is currently active
    #[prop(optional, into)]
    active: Signal<bool>,

    /// Whether this tab is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the anchor element
    #[prop(optional)]
    node_ref: NodeRef<A>,

    /// Tab label content
    children: Children,
) -> impl IntoView {
    // `use_context` walks the reactive owner chain, and a plain
    // (non-island) `#[component]` never opens an owner boundary of its
    // own (leptos_macro only inserts one for islands) - so a `TabSet`
    // higher up the page can leave its `TabSetContext` visible to every
    // *sibling* rendered later in the same synchronous pass, not just to
    // its own children. A `Tab` with no `tab_key` sitting anywhere after
    // an unrelated `TabSet` on the same page therefore sees `Some`
    // context despite never being nested inside one (ldui-d2hg). Treat
    // that combination as "not actually controlled" rather than a
    // programmer error: crashing the whole wasm app over an ambient
    // context leak is worse than silently rendering an uncontrolled tab.
    // A `tab_key` with no enclosing `TabSet` gets the same graceful
    // treatment for the same reason - defense in depth against an assert
    // ever taking down the app again.
    let context = use_context::<TabSetContext>();
    let (context, tab_key) = match (context, tab_key) {
        (Some(context), Some(key)) => (Some(context), Some(key)),
        (Some(_), None) => {
            leptos::logging::warn!(
                "Tab: an ambient TabSet context is visible but no tab_key was supplied; rendering as an uncontrolled tab instead of panicking (ldui-d2hg)."
            );
            (None, None)
        }
        (None, Some(_)) => {
            leptos::logging::warn!(
                "Tab: tab_key was supplied but no enclosing TabSet was found; the key will be ignored (ldui-d2hg)."
            );
            (None, None)
        }
        (None, None) => (None, None),
    };
    let tab_key = tab_key.map(StoredValue::new);
    let tab_id = context
        .zip(tab_key)
        .map(|(context, key)| tab_dom_id(&context.id(), &key.get_value()));
    let panel_id = context
        .zip(tab_key)
        .map(|(context, key)| tab_panel_dom_id(&context.id(), &key.get_value()));
    if let (Some(context), Some(key)) = (context, tab_key) {
        let registration_id = context.register(key.get_value(), disabled);
        let cleanup_tab_id = tab_id.clone().expect("controlled tab has an id");
        on_cleanup(move || context.unregister(registration_id, &cleanup_tab_id));
    }

    view! {
        <a
            node_ref=node_ref
            id=tab_id
            role=context.map(|_| "tab")
            aria-selected=move || {
                context.zip(tab_key).map(|(context, key)| {
                    context.is_selected(&key.get_value()).to_string()
                })
            }
            aria-controls=panel_id
            aria-disabled=move || context.map(|_| disabled.get().to_string())
            tabindex=move || {
                context.zip(tab_key).map(|(context, key)| {
                    if context.is_roving_stop(&key.get_value()) { 0 } else { -1 }
                })
            }
            class=move || {
                merge_classes!(
                    "tab",
                    context.map_or("", |_| "shrink-0 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary forced-colors:focus-visible:outline-[Highlight]"),
                    class
                )
            }
            class:tab-active=move || {
                context.zip(tab_key).map_or_else(
                    || active.get(),
                    |(context, key)| context.is_selected(&key.get_value()),
                )
            }
            class:tab-disabled=disabled
            on:click=move |event: ev::MouseEvent| {
                let Some((context, key)) = context.zip(tab_key) else {
                    return;
                };
                event.prevent_default();
                if disabled.get_untracked() {
                    event.stop_propagation();
                    return;
                }
                context.select(&key.get_value());
            }
            on:focus=move |_| {
                if let Some((context, key)) = context.zip(tab_key) {
                    context.set_focused(&key.get_value());
                }
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                let Some((context, key)) = context.zip(tab_key) else {
                    return;
                };
                if disabled.get_untracked() {
                    return;
                }
                let movement = match (context.orientation.get_untracked(), event.key().as_str()) {
                    (TabOrientation::Horizontal, "ArrowRight")
                    | (TabOrientation::Vertical, "ArrowDown") => Some(TabMove::Next),
                    (TabOrientation::Horizontal, "ArrowLeft")
                    | (TabOrientation::Vertical, "ArrowUp") => Some(TabMove::Previous),
                    (_, "Home") => Some(TabMove::Home),
                    (_, "End") => Some(TabMove::End),
                    _ => None,
                };
                if let Some(movement) = movement {
                    event.prevent_default();
                    event.stop_propagation();
                    context.move_focus(&key.get_value(), movement);
                } else if matches!(event.key().as_str(), "Enter" | " " | "Spacebar") {
                    event.prevent_default();
                    event.stop_propagation();
                    context.select(&key.get_value());
                }
            }
        >
            {children()}
        </a>
    }
}

/// Panel associated with one stable tab key in the nearest [`TabSet`].
#[component]
pub fn TabPanel(
    /// Stable key of the controlling [`Tab`].
    #[prop(into)]
    tab_key: String,
    /// Additional panel classes.
    #[prop(optional, into)]
    class: &'static str,
    /// Reference to the panel element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
    /// Panel content.
    children: Children,
) -> impl IntoView {
    let context =
        use_context::<TabSetContext>().expect("TabPanel must be used within a controlled TabSet");
    let key = StoredValue::new(tab_key);
    let id = tab_panel_dom_id(&context.id(), &key.get_value());
    let labelled_by = tab_dom_id(&context.id(), &key.get_value());

    view! {
        <div
            node_ref=node_ref
            id=id
            role="tabpanel"
            aria-labelledby=labelled_by
            tabindex="0"
            hidden=move || !context.is_selected(&key.get_value())
            class=move || merge_classes!("min-w-0 focus-visible:outline focus-visible:outline-2 focus-visible:outline-primary", class)
            data-tab-panel=move || key.get_value()
        >
            {children()}
        </div>
    }
}

/// # Tab Radio Component
///
/// A reactive Leptos wrapper for radio input-based tabs providing form
/// integration and accessibility.
///
/// ## Node References
/// - `node_ref` - References the input element ([HTMLInputElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement))
#[component]
pub fn TabRadio(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the input element
    #[prop(optional)]
    node_ref: NodeRef<Input>,

    /// Tab label content
    children: Children,
) -> impl IntoView {
    view! {
        <input node_ref=node_ref class=move || merge_classes!("tab", class) />
        {children()}
    }
}
