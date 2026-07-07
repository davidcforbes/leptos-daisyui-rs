use super::style::{MenuDirection, MenuSize};
use super::types::{first_enabled_index, last_enabled_index, next_enabled_index};
use crate::components::icon::{Icon, IconSize};
use crate::components::kbd::{Kbd, KbdSize};
use crate::merge_classes;
use leptos::{
    ev,
    html::{H2, Li, Ul},
    prelude::*,
};
use leptos_router::components::A;
use std::sync::atomic::{AtomicU64, Ordering};

/// Per-instance sequence so each `Menu`/`SubMenu` gets unique item DOM ids for
/// `aria-activedescendant` wiring (WAI-ARIA menu pattern). ldui-jcs.21
static MENU_NAV_SEQ: AtomicU64 = AtomicU64::new(0);

/// Build the DOM id for item `index` of the `instance`-th menu/submenu scope.
fn menu_item_dom_id(instance: u64, index: usize) -> String {
    format!("ld-menu-{instance}-item-{index}")
}

/// # Menu Component
///
/// A reactive Leptos wrapper for daisyUI's menu component that provides
/// structured navigation with automatic selection tracking and hierarchical organization.
///
/// Supports `ArrowUp`/`ArrowDown` keyboard highlight navigation (wrapping,
/// skipping disabled items), `Home`/`End`, and `Enter`/`Space` activation of
/// the highlighted item — ported from d2d-ui's `controls::menu::Menu`
/// `highlight_next`/`highlight_prev`. The highlight itself is a virtual,
/// context-tracked index (mirroring d2d's `highlight_index`, which has no
/// concept of real OS focus) exposed to the DOM via `aria-activedescendant`
/// on the root `<ul>` rather than moving real DOM focus per item; each
/// [`MenuItem`]/[`MenuCheckItem`]/[`MenuRadioItem`] renders the daisyUI
/// `menu-focus` class when it is the highlighted item.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("menu menu-horizontal menu-vertical menu-xs menu-sm menu-md menu-lg menu-xl menu-title menu-active menu-focus menu-disabled");
/// ```
///
/// ## Node References
/// - `node_ref` - References the ul element ([HTMLUListElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLUListElement))
#[component]
pub fn Menu(
    /// If true, disables automatic selection tracking
    #[prop(optional)]
    manual: bool,

    /// Signal for tracking the currently selected menu item value
    #[prop(optional)]
    selected: RwSignal<Option<String>>,

    /// Layout direction of menu items
    #[prop(optional, into)]
    direction: Signal<MenuDirection>,

    /// Size variant for menu items
    #[prop(optional, into)]
    size: Signal<MenuSize>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the ul element
    #[prop(optional)]
    node_ref: NodeRef<Ul>,

    /// Menu content
    children: Children,
) -> impl IntoView {
    let manager = MenuManager { manual, selected };
    provide_context(manager);
    let nav = MenuNav::new();
    provide_context(nav);

    view! {
        <ul
            node_ref=node_ref
            role="menu"
            tabindex="0"
            aria-activedescendant=move || nav.active_dom_id()
            class=move || {
                merge_classes!(
                    "menu",
                    direction.get().as_str(),
                    size.get().as_str(),
                    class
                )
            }
            on:keydown=nav.keydown_handler()
        >
            {children()}
        </ul>
    }
}

/// # Menu Item Component
///
/// A reactive Leptos wrapper for individual menu items with selection tracking
/// and navigation support.
///
/// Registers with the enclosing [`Menu`]/[`SubMenu`]'s keyboard-highlight
/// tracker on creation (see the [`Menu`] docs); disabled items are skipped by
/// `ArrowUp`/`ArrowDown` and cannot be activated. Optional `icon` (a Lucide
/// icon name) and `shortcut` (a keyboard-hint string rendered in a `<kbd>`)
/// props add structured leading/trailing slots.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex items-center gap-2 w-full flex-1 min-w-0 shrink-0 cursor-pointer");
/// ```
///
/// ## Node References
/// - `node_ref` - References the li element ([HTMLLIElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLLIElement))
#[component]
pub fn MenuItem(
    /// Optional URL for navigation
    #[prop(optional, into)]
    href: Signal<String>,

    /// Unique identifier for selection tracking
    #[prop(optional, into)]
    value: Signal<String>,

    /// Manual active state (manual mode only)
    #[prop(optional, into)]
    active: Signal<bool>,

    /// Whether the item is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the li element
    #[prop(optional)]
    node_ref: NodeRef<Li>,

    /// If true, renders without an anchor wrapper — used for items nested
    /// inside a [`SubMenu`] whose parent [`MenuItem`] already renders its own
    /// `<a>` (anchors cannot nest inside anchors).
    #[prop(optional, into)]
    is_submenu: bool,

    /// Optional click callback (for action-based menus, not navigation)
    #[prop(optional, into)]
    on_click: Option<Callback<()>>,

    /// Lucide icon name rendered before the label (leading slot). Empty
    /// string (the default) renders no icon. No-op when `is_submenu=true`
    /// (that branch renders children as-is; see `is_submenu`'s docs).
    #[prop(optional, into)]
    icon: Signal<String>,

    /// Keyboard-shortcut hint rendered right-aligned in a `<kbd>` (e.g.
    /// `"Ctrl+S"`). Empty string (the default) renders nothing. No-op when
    /// `is_submenu=true`.
    #[prop(optional, into)]
    shortcut: Signal<String>,

    /// Item content
    children: Children,
) -> impl IntoView {
    let MenuManager { manual, selected } = MenuManager::expect_context();
    let nav = MenuNav::expect_context();

    let activate = move || {
        if disabled.get_untracked() {
            return;
        }
        if let Some(cb) = on_click {
            cb.run(());
        }
        if !value.get_untracked().is_empty() {
            *selected.write() = Some(value.get_untracked());
        }
    };

    let index = nav.register(disabled.get_untracked(), Callback::new(move |_| activate()));
    Effect::new(move |_| nav.set_disabled(index, disabled.get()));

    let on_anchor_click = move |e: ev::MouseEvent| {
        if disabled.get_untracked() {
            e.prevent_default();
            return;
        }

        // Fire action callback + selection tracking.
        activate();

        // Prevent default navigation if href is empty or not a real URL
        let href_value = href.get_untracked();
        if href_value.is_empty() || href_value == "#" || href_value.starts_with("javascript:") {
            e.prevent_default();
        }
        // Note: Don't prevent default for valid URLs - let the <A> component handle navigation
    };

    let is_active = move || {
        if manual {
            return active.get();
        }

        selected
            .get()
            .as_ref()
            .is_some_and(|s| s == &value.get_untracked())
    };

    // Whether to wrap children in the icon/shortcut flex row, decided once at
    // creation (like `is_submenu`, a plain bool) rather than reactively: most
    // existing `MenuItem`s (including ones nesting a `SubMenu` in their own
    // children, e.g. the demo's "Files"/"Tools" pattern) never set `icon`/
    // `shortcut` and rely on rendering their (possibly block-level, multi-line)
    // children as-is — imposing a single-line flex row on those would break
    // their layout. Only opting into `icon`/`shortcut` opts into the row.
    let has_decoration = !icon.get_untracked().is_empty() || !shortcut.get_untracked().is_empty();

    view! {
        <li node_ref=node_ref role="none" class=class>
            {if !is_submenu {
                view! {
                    <A
                        href=move || href.get()
                        attr:role="menuitem"
                        attr:id=menu_item_dom_id(nav.instance, index)
                        attr:aria-disabled=move || disabled.get().to_string()
                        class:menu-active=is_active
                        class:menu-focus=move || nav.is_highlighted(index)
                        class:menu-disabled=move || disabled.get()
                        on:click=on_anchor_click
                    >
                        {if has_decoration {
                            view! {
                                <span class="flex items-center gap-2 w-full">
                                    <Show when=move || !icon.get().is_empty()>
                                        <Icon name=icon size=IconSize::Small />
                                    </Show>
                                    <span class="flex-1 min-w-0">{children()}</span>
                                    <Show when=move || !shortcut.get().is_empty()>
                                        <Kbd size=KbdSize::Sm class="shrink-0">{move || shortcut.get()}</Kbd>
                                    </Show>
                                </span>
                            }
                                .into_any()
                        } else {
                            children().into_any()
                        }}
                    </A>
                }
                    .into_any()
            } else {
                // Note: `is_submenu` items render their children as-is (no
                // icon/shortcut row wrapper) — this branch is also used for
                // container items whose children are arbitrary block content
                // (e.g. a `MenuTitle` + nested `SubMenu`, not a single-line
                // label), so imposing a flex row here would break that
                // layout. `icon`/`shortcut` are no-ops on `is_submenu` items.
                view! {
                    <span
                        role="menuitem"
                        tabindex="-1"
                        id=menu_item_dom_id(nav.instance, index)
                        aria-disabled=move || disabled.get().to_string()
                        class="cursor-pointer"
                        class:menu-active=is_active
                        class:menu-focus=move || nav.is_highlighted(index)
                        class:menu-disabled=move || disabled.get()
                        on:click=move |_| activate()
                    >
                        {children()}
                    </span>
                }
                    .into_any()
            }}

        </li>
    }
}

/// # Menu Check Item Component
///
/// A checkable menu item (`role="menuitemcheckbox"`) rendering a checkmark
/// leading indicator instead of navigating. Fully controlled: the caller owns
/// `checked` and reacts to `on_toggle` (fired with the item's *next* checked
/// value). Registers with the enclosing [`Menu`]/[`SubMenu`]'s
/// keyboard-highlight tracker like [`MenuItem`].
///
/// Ported from d2d-ui's `controls::menu::MenuItem::Check` (`beads-kqo9`).
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex items-center gap-2 w-full flex-1 min-w-0 shrink-0 cursor-pointer w-4 text-center");
/// ```
///
/// ## Node References
/// - `node_ref` - References the li element ([HTMLLIElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLLIElement))
#[component]
pub fn MenuCheckItem(
    /// Whether the item is currently checked
    #[prop(optional, into)]
    checked: Signal<bool>,

    /// Fired with the toggled (opposite) checked value when the item is
    /// activated (click, or `Enter`/`Space` while keyboard-highlighted)
    #[prop(optional)]
    on_toggle: Option<Callback<bool>>,

    /// Whether the item is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Keyboard-shortcut hint rendered right-aligned in a `<kbd>`. Empty
    /// string (the default) renders nothing.
    #[prop(optional, into)]
    shortcut: Signal<String>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the li element
    #[prop(optional)]
    node_ref: NodeRef<Li>,

    /// Item content
    children: Children,
) -> impl IntoView {
    let nav = MenuNav::expect_context();

    let activate = move || {
        if disabled.get_untracked() {
            return;
        }
        if let Some(cb) = on_toggle {
            cb.run(!checked.get_untracked());
        }
    };

    let index = nav.register(disabled.get_untracked(), Callback::new(move |_| activate()));
    Effect::new(move |_| nav.set_disabled(index, disabled.get()));

    view! {
        <li node_ref=node_ref role="none" class=class>
            <span
                role="menuitemcheckbox"
                tabindex="-1"
                id=menu_item_dom_id(nav.instance, index)
                aria-checked=move || checked.get().to_string()
                aria-disabled=move || disabled.get().to_string()
                class="cursor-pointer"
                class:menu-focus=move || nav.is_highlighted(index)
                class:menu-disabled=move || disabled.get()
                on:click=move |_| activate()
            >
                <span class="flex items-center gap-2 w-full">
                    <span class="w-4 shrink-0 text-center" aria-hidden="true">
                        <Show when=move || checked.get()>"\u{2713}"</Show>
                    </span>
                    <span class="flex-1 min-w-0">{children()}</span>
                    <Show when=move || !shortcut.get().is_empty()>
                        <Kbd size=KbdSize::Sm class="shrink-0">{move || shortcut.get()}</Kbd>
                    </Show>
                </span>
            </span>
        </li>
    }
}

/// # Menu Radio Item Component
///
/// A radio-group menu item (`role="menuitemradio"`) rendering a filled-bullet
/// leading indicator when checked. Fully controlled: the caller owns
/// `checked` and reacts to `on_toggle` (always fired with `true` — clicking a
/// radio item selects it, it never unchecks itself). Radio grouping ("only
/// one checked at a time") is achieved the same way as any controlled radio
/// group: give every item in the group a `checked` signal derived from the
/// same shared selection state, and have `on_toggle` update that shared
/// state (see the demo for an example).
///
/// Ported from d2d-ui's `controls::menu::MenuItem::Radio` (`beads-kqo9`).
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex items-center gap-2 w-full flex-1 min-w-0 shrink-0 cursor-pointer w-4 text-center");
/// ```
///
/// ## Node References
/// - `node_ref` - References the li element ([HTMLLIElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLLIElement))
#[component]
pub fn MenuRadioItem(
    /// Whether the item is currently checked/selected
    #[prop(optional, into)]
    checked: Signal<bool>,

    /// Fired with `true` when the item is activated (click, or `Enter`/`Space`
    /// while keyboard-highlighted)
    #[prop(optional)]
    on_toggle: Option<Callback<bool>>,

    /// Whether the item is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the li element
    #[prop(optional)]
    node_ref: NodeRef<Li>,

    /// Item content
    children: Children,
) -> impl IntoView {
    let nav = MenuNav::expect_context();

    let activate = move || {
        if disabled.get_untracked() {
            return;
        }
        if let Some(cb) = on_toggle {
            cb.run(true);
        }
    };

    let index = nav.register(disabled.get_untracked(), Callback::new(move |_| activate()));
    Effect::new(move |_| nav.set_disabled(index, disabled.get()));

    view! {
        <li node_ref=node_ref role="none" class=class>
            <span
                role="menuitemradio"
                tabindex="-1"
                id=menu_item_dom_id(nav.instance, index)
                aria-checked=move || checked.get().to_string()
                aria-disabled=move || disabled.get().to_string()
                class="cursor-pointer"
                class:menu-focus=move || nav.is_highlighted(index)
                class:menu-disabled=move || disabled.get()
                on:click=move |_| activate()
            >
                <span class="flex items-center gap-2 w-full">
                    <span class="w-4 shrink-0 text-center" aria-hidden="true">
                        <Show when=move || checked.get()>"\u{25CF}"</Show>
                    </span>
                    <span class="flex-1 min-w-0">{children()}</span>
                </span>
            </span>
        </li>
    }
}

/// # Menu Title Component
///
/// A reactive Leptos wrapper for menu section titles that provide
/// visual organization of menu items into groups.
///
/// ## Node References
/// - `node_ref` - References the h2 element ([HTMLHeadingElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLHeadingElement))
#[component]
pub fn MenuTitle(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the h2 element
    #[prop(optional)]
    node_ref: NodeRef<H2>,

    /// Title text
    children: Children,
) -> impl IntoView {
    view! {
        <h2 node_ref=node_ref class=move || merge_classes!("menu-title", class)>
            {children()}
        </h2>
    }
}

/// # SubMenu Component
///
/// A reactive Leptos wrapper for nested submenu containers within menu items.
/// Used for hierarchical navigation structures.
///
/// Establishes its own keyboard-highlight scope (a fresh [`MenuNav`]) so
/// `ArrowUp`/`ArrowDown`/`Home`/`End`/`Enter`/`Space` navigate the submenu's
/// own items independently of the parent [`Menu`]'s highlight. Visibility
/// defaults to always-open (the pre-existing "always rendered, styled by CSS"
/// behavior); pass `open` to control it programmatically in addition to (or
/// instead of) CSS hover.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("hidden");
/// ```
///
/// ## Node References
/// - `node_ref` - References the ul element ([HTMLUListElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLUListElement))
#[component]
pub fn SubMenu(
    /// Whether the submenu is open. Defaults to always-open, preserving the
    /// original CSS-hover-only behavior; pass a signal to control it
    /// programmatically.
    #[prop(optional, into, default = Signal::derive(|| true))]
    open: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the ul element
    #[prop(optional)]
    node_ref: NodeRef<Ul>,

    /// Submenu content
    children: Children,
) -> impl IntoView {
    let nav = MenuNav::new();
    provide_context(nav);

    view! {
        <ul
            node_ref=node_ref
            role="menu"
            tabindex="0"
            aria-activedescendant=move || nav.active_dom_id()
            class=class
            class:hidden=move || !open.get()
            on:keydown=nav.keydown_handler()
        >
            {children()}
        </ul>
    }
}

/// # Menu Divider Component
///
/// A horizontal rule divider for separating groups of menu items.
/// Renders as `<li><hr /></li>` following daisyUI 5 conventions.
#[component]
pub fn MenuDivider() -> impl IntoView {
    view! {
        <li role="none">
            <hr role="separator" />
        </li>
    }
}

/// Internal context manager for menu selection state.
#[derive(Clone)]
pub(crate) struct MenuManager {
    /// Whether the menu operates in manual mode (disables automatic selection)
    manual: bool,
    /// Signal tracking the currently selected menu item value
    selected: RwSignal<Option<String>>,
}

impl MenuManager {
    /// Retrieves the MenuManager from context.
    ///
    /// # Panics
    /// Panics if MenuItem or MenuDropdown is used outside of a Menu component.
    pub fn expect_context() -> Self {
        use_context::<MenuManager>()
            .expect("MenuItem and MenuDropdown must be used within a Menu component")
    }
}

/// Internal context tracking a [`Menu`]/[`SubMenu`]'s keyboard highlight
/// state (ldui-jcs.21). Ported from d2d-ui's `controls::menu::Menu`'s
/// `highlight_index` + `step_highlight`: a *virtual*, index-based highlight
/// independent of real DOM focus, exposed to the DOM via
/// `aria-activedescendant` on the owning `<ul>`.
///
/// Items register themselves once, at creation, via [`MenuNav::register`];
/// registration order is document order (component setup functions for
/// direct/nested children of a `view!` run top-to-bottom during the initial
/// render pass), so no item needs to know its own position ahead of time.
#[derive(Clone, Copy)]
pub(crate) struct MenuNav {
    /// Unique id for this Menu/SubMenu scope, used to build collision-free
    /// per-item DOM ids for `aria-activedescendant`.
    instance: u64,
    /// The currently keyboard-highlighted item index, if any.
    highlighted: RwSignal<Option<usize>>,
    /// `disabled[i]` is `true` when registered item `i` should be skipped by
    /// keyboard navigation and cannot be activated.
    disabled: RwSignal<Vec<bool>>,
    /// Per-item activation callback, invoked by `Enter`/`Space` on the
    /// highlighted item.
    activate: RwSignal<Vec<Option<Callback<()>>>>,
}

impl MenuNav {
    /// Create a fresh highlight-tracking scope for a `Menu` or `SubMenu`.
    pub(crate) fn new() -> Self {
        Self {
            instance: MENU_NAV_SEQ.fetch_add(1, Ordering::Relaxed),
            highlighted: RwSignal::new(None),
            disabled: RwSignal::new(Vec::new()),
            activate: RwSignal::new(Vec::new()),
        }
    }

    /// Retrieves the nearest enclosing `MenuNav` from context.
    ///
    /// # Panics
    /// Panics if called outside of a `Menu` or `SubMenu` component.
    pub(crate) fn expect_context() -> Self {
        use_context::<MenuNav>().expect(
            "MenuItem, MenuCheckItem, and MenuRadioItem must be used within a Menu or SubMenu component",
        )
    }

    /// Register a selectable item (called once per item, at creation).
    /// Returns its stable index within this scope.
    pub(crate) fn register(&self, disabled: bool, on_activate: Callback<()>) -> usize {
        self.disabled.update(|d| d.push(disabled));
        self.activate.update(|a| a.push(Some(on_activate)));
        self.disabled.get_untracked().len() - 1
    }

    /// Update a registered item's disabled state (kept in sync with its
    /// reactive `disabled` prop via an `Effect`).
    pub(crate) fn set_disabled(&self, index: usize, value: bool) {
        self.disabled.update(|d| {
            if let Some(slot) = d.get_mut(index) {
                *slot = value;
            }
        });
    }

    /// Whether item `index` is the currently keyboard-highlighted item.
    pub(crate) fn is_highlighted(&self, index: usize) -> bool {
        self.highlighted.get() == Some(index)
    }

    /// The DOM id `aria-activedescendant` should point at, if any item is
    /// highlighted.
    pub(crate) fn active_dom_id(&self) -> Option<String> {
        self.highlighted
            .get()
            .map(|i| menu_item_dom_id(self.instance, i))
    }

    fn move_highlight(&self, forward: bool) {
        let disabled = self.disabled.get_untracked();
        let current = self.highlighted.get_untracked();
        self.highlighted
            .set(next_enabled_index(current, &disabled, forward));
    }

    fn move_home(&self) {
        let disabled = self.disabled.get_untracked();
        self.highlighted.set(first_enabled_index(&disabled));
    }

    fn move_end(&self) {
        let disabled = self.disabled.get_untracked();
        self.highlighted.set(last_enabled_index(&disabled));
    }

    fn activate_highlighted(&self) {
        if let Some(i) = self.highlighted.get_untracked()
            && let Some(Some(cb)) = self.activate.get_untracked().get(i)
        {
            cb.run(());
        }
    }

    /// Build the `on:keydown` handler for this scope's root `<ul>`. Handled
    /// keys are `prevent_default`-ed and `stop_propagation`-ed so an open
    /// `SubMenu`'s own handler doesn't also get acted on by an ancestor
    /// `Menu`'s handler for the same keypress.
    pub(crate) fn keydown_handler(&self) -> impl Fn(ev::KeyboardEvent) + use<> {
        let nav = *self;
        move |e: ev::KeyboardEvent| {
            let handled = match e.key().as_str() {
                "ArrowDown" => {
                    nav.move_highlight(true);
                    true
                }
                "ArrowUp" => {
                    nav.move_highlight(false);
                    true
                }
                "Home" => {
                    nav.move_home();
                    true
                }
                "End" => {
                    nav.move_end();
                    true
                }
                "Enter" | " " => {
                    nav.activate_highlighted();
                    true
                }
                _ => false,
            };
            if handled {
                e.prevent_default();
                e.stop_propagation();
            }
        }
    }
}
