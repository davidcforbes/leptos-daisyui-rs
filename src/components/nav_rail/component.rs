use super::style::{group_class, indicator_class, item_class, rail_class};
use crate::merge_classes;
use leptos::{
    ev,
    html::{Button, Div, Nav},
    prelude::*,
};

/// # NavRail Component
///
/// A vertical icon navigation rail: a narrow column of icon buttons with a
/// selected-item "pill" background, a left-edge accent indicator bar on the
/// active item, hover highlighting, and support for pinning a trailing
/// group (e.g. Settings) to the bottom of the rail via [`NavRailGroup`]'s
/// `pinned` prop (`mt-auto`).
///
/// Ported from d2d-ui's owner-drawn `NavRail` control, which tracked a
/// global index across a `top_items`/`bottom_items` split and did manual
/// hit-testing/hover math against Direct2D rects. None of that renderer
/// plumbing carries over: hover is expressed with the CSS `:hover`
/// pseudo-class, and selection is tracked by *value* (like `Menu` and
/// `AppShell`'s `AppShellIconNav`) via [`NavRailContext`] rather than a
/// numeric index, so items don't need to know their position.
///
/// ## Relationship to `AppShellIconNav`
/// `AppShellIconNav` is the icon strip built into the 3-panel `AppShell`
/// layout, and is only meant to be used inside an `AppShell`. `NavRail` is
/// the standalone equivalent: it can be dropped into any layout on its own,
/// adds the left-edge accent indicator and bottom-pinned [`NavRailGroup`]
/// that `AppShellIconNav` doesn't have, and uses its own context so it
/// doesn't require an `AppShell` ancestor. Pick `AppShellIconNav` when
/// you're already composing an `AppShell`; pick `NavRail` everywhere else.
///
/// ## Example
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::*;
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let active = RwSignal::new(Some("home".to_string()));
///
///     view! {
///         <NavRail active=active>
///             <NavRailItem value="home">"H"</NavRailItem>
///             <NavRailItem value="search">"S"</NavRailItem>
///             <NavRailGroup pinned=true>
///                 <NavRailItem value="settings">"G"</NavRailItem>
///             </NavRailGroup>
///         </NavRail>
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex h-full w-16 flex-col items-center gap-1 bg-base-300 py-2");
/// @source inline("flex flex-col items-center gap-1 mt-auto");
/// @source inline("relative flex h-12 w-12 cursor-pointer items-center justify-center rounded-box transition-colors");
/// @source inline("bg-base-200 bg-base-300 text-primary text-base-content/60 text-base-content hover:bg-base-200 hover:bg-base-300 hover:text-base-content");
/// @source inline("absolute left-0 top-1/2 h-6 w-1 -translate-y-1/2 rounded-r-full bg-primary bg-transparent");
/// ```
///
/// ## Bottom-Pinning with NavRailGroup
/// When a [`NavRailGroup`] has `pinned=true`, it is pushed to the bottom of
/// the rail via `mt-auto`. For the group to reach the rail's bottom edge, the
/// rail's parent container must have a definite height (e.g., `h-screen` or a
/// fixed-height flex parent). Without such a constraint, the pinned group will
/// not extend downward as expected.
///
/// ## Node References
/// - `node_ref` - References the root `<nav>` element ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement))
#[component]
pub fn NavRail(
    /// If true, disables automatic active-item tracking via context; each
    /// [`NavRailItem`]'s own `active` prop is used instead.
    #[prop(optional)]
    manual: bool,

    /// Signal for tracking the currently active item value.
    #[prop(optional)]
    active: RwSignal<Option<String>>,

    /// Additional CSS classes for the root `<nav>` element.
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the root `<nav>` element.
    #[prop(optional)]
    node_ref: NodeRef<Nav>,

    /// Rail content ([`NavRailItem`] / [`NavRailGroup`] components).
    children: Children,
) -> impl IntoView {
    let ctx = NavRailContext { active, manual };
    provide_context(ctx);

    view! {
        <nav node_ref=node_ref class=move || merge_classes!(rail_class(), class)>
            {children()}
        </nav>
    }
}

/// # NavRail Group Component
///
/// Wraps a set of [`NavRailItem`]s. Setting `pinned=true` pushes the group
/// (and anything after it) to the bottom of the rail via `mt-auto` --
/// equivalent to d2d-ui's `NavRail::with_bottom_items`, which stacked a
/// second group upward from the rail's bottom edge.
///
/// ## Node References
/// - `node_ref` - References the group `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn NavRailGroup(
    /// Pin this group to the bottom of the rail (`mt-auto`). Requires the
    /// rail's parent container to have a definite height (e.g., `h-screen` or
    /// a fixed-height flex parent) for the group to reach the container bottom.
    #[prop(optional, into)]
    pinned: Signal<bool>,

    /// Additional CSS classes.
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the group `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Group content ([`NavRailItem`] components).
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!(group_class(pinned.get()), class)>
            {children()}
        </div>
    }
}

/// # NavRail Item Component
///
/// An individual icon button within the rail. Supports automatic active
/// tracking via the [`NavRail`] context (matched by `value`), or manual
/// control (via the `active` prop) when the parent `NavRail` has
/// `manual=true`.
///
/// ## Node References
/// - `node_ref` - References the item `<button>` element ([HTMLButtonElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement))
#[component]
pub fn NavRailItem(
    /// Unique identifier for active-state tracking.
    #[prop(optional, into)]
    value: Signal<String>,

    /// Manual active state (only used when the parent `NavRail` has
    /// `manual=true`).
    #[prop(optional, into)]
    active: Signal<bool>,

    /// Accessible label, also used for the `aria-label` attribute. Icon-only
    /// items (with no label text) have no accessible name for screen-reader
    /// users and should always provide a label.
    #[prop(optional, into)]
    label: Signal<String>,

    /// Additional CSS classes.
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the item `<button>` element.
    #[prop(optional)]
    node_ref: NodeRef<Button>,

    /// Optional click callback, fired in addition to the built-in
    /// selection tracking.
    #[prop(optional)]
    on_click: Option<Callback<()>>,

    /// Item content (an icon, and/or a visually-hidden label).
    children: Children,
) -> impl IntoView {
    let NavRailContext {
        active: ctx_active,
        manual,
    } = NavRailContext::expect_context();

    let on_button_click = move |_: ev::MouseEvent| {
        let v = value.get_untracked();
        if !manual && !v.is_empty() {
            ctx_active.set(Some(v));
        }
        if let Some(cb) = on_click {
            cb.run(());
        }
    };

    let is_active = move || {
        if manual {
            return active.get();
        }
        ctx_active
            .get()
            .as_ref()
            .is_some_and(|s| s == &value.get_untracked())
    };

    view! {
        <button
            type="button"
            node_ref=node_ref
            class=move || merge_classes!(item_class(is_active()), class)
            aria-label=move || {
                let l = label.get();
                (!l.is_empty()).then_some(l)
            }
            aria-current=move || is_active().then_some("page")
            on:click=on_button_click
        >
            <span class=move || indicator_class(is_active())></span>
            {children()}
        </button>
    }
}

/// Internal context manager for `NavRail` active-item state.
///
/// Access this in any descendant of `NavRail` to read or write the active
/// item, mirroring `AppShellContext` / `MenuManager`.
#[derive(Clone)]
pub struct NavRailContext {
    /// Signal tracking the currently active item value.
    pub active: RwSignal<Option<String>>,
    /// Whether the rail operates in manual mode (disables automatic
    /// active-item tracking).
    manual: bool,
}

impl NavRailContext {
    /// Retrieves the `NavRailContext` from Leptos context.
    ///
    /// # Panics
    /// Panics if called outside of a `NavRail` component tree.
    pub fn expect_context() -> Self {
        use_context::<NavRailContext>()
            .expect("NavRailItem and NavRailGroup must be used within a NavRail component")
    }
}
