use crate::merge_classes;
use leptos::{
    ev,
    html::{Button, Div, Nav},
    prelude::*,
};

/// # AppShell Component
///
/// A 3-panel admin layout component providing an icon navigation strip,
/// a secondary side panel, and a main content area.
///
/// Tracks the active section via context, allowing child components to
/// react to section changes.
///
/// ## Example
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::*;
///
/// #[component]
/// fn MyApp() -> impl IntoView {
///     let section = RwSignal::new(Some("home".to_string()));
///
///     view! {
///         <AppShell active_section=section>
///             <AppShellIconNav class="w-16">
///                 <AppShellIconNavItem value="home">
///                     "Home"
///                 </AppShellIconNavItem>
///                 <AppShellIconNavItem value="settings">
///                     "Settings"
///                 </AppShellIconNavItem>
///             </AppShellIconNav>
///             <AppShellSidePanel class="w-48">
///                 <Show when=move || section.get() == Some("home".to_string())>
///                     "Home navigation"
///                 </Show>
///             </AppShellSidePanel>
///             <AppShellContent class="p-6">
///                 "Main content"
///             </AppShellContent>
///         </AppShell>
///     }
/// }
/// ```
///
/// ## Node References
/// - `node_ref` - References the root div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn AppShell(
    /// If true, disables automatic section tracking via context
    #[prop(optional)]
    manual: bool,

    /// Signal for tracking the currently active section value
    #[prop(optional)]
    active_section: RwSignal<Option<String>>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the root div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Shell content (AppShellIconNav, AppShellSidePanel, AppShellContent)
    children: Children,
) -> impl IntoView {
    let ctx = AppShellContext {
        active_section,
        manual,
    };
    provide_context(ctx);

    view! {
        <div node_ref=node_ref class=move || merge_classes!("flex h-full w-full", class)>
            {children()}
        </div>
    }
}

/// # AppShell Icon Nav Component
///
/// Narrow vertical navigation strip for section icons. Typically the leftmost
/// column in a 3-panel layout. Width is controlled via the `class` prop
/// (e.g., `class="w-16"` for 64px).
///
/// ## Node References
/// - `node_ref` - References the nav element ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement))
#[component]
pub fn AppShellIconNav(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the nav element
    #[prop(optional)]
    node_ref: NodeRef<Nav>,

    /// Navigation items (AppShellIconNavItem components)
    children: Children,
) -> impl IntoView {
    view! {
        <nav
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "flex flex-col items-center bg-base-300",
                    class
                )
            }
        >
            {children()}
        </nav>
    }
}

/// # AppShell Icon Nav Item Component
///
/// An individual icon button within the icon nav strip. Supports automatic
/// active state tracking via the AppShell context.
///
/// The `active` CSS class is toggled when this item is selected, enabling
/// styling with Tailwind's `[&.active]:bg-base-100` or custom CSS targeting
/// `button.active`.
///
/// ## Node References
/// - `node_ref` - References the button element ([HTMLButtonElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement))
#[component]
pub fn AppShellIconNavItem(
    /// Unique identifier for section tracking
    #[prop(optional, into)]
    value: Signal<String>,

    /// Manual active state (only used when AppShell has `manual=true`)
    #[prop(optional, into)]
    active: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the button element
    #[prop(optional)]
    node_ref: NodeRef<Button>,

    /// Item content (icon and/or label)
    children: Children,
) -> impl IntoView {
    let AppShellContext {
        active_section,
        manual,
    } = AppShellContext::expect_context();

    let on_click = move |_: ev::MouseEvent| {
        let v = value.get_untracked();
        if !v.is_empty() {
            active_section.set(Some(v));
        }
    };

    let is_active = move || {
        if manual {
            return active.get();
        }
        active_section
            .get()
            .as_ref()
            .is_some_and(|s| s == &value.get_untracked())
    };

    view! {
        <button
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "flex flex-col items-center justify-center gap-1 p-2 w-full cursor-pointer",
                    class
                )
            }
            class:active=is_active
            on:click=on_click
        >
            {children()}
        </button>
    }
}

/// # AppShell Side Panel Component
///
/// Secondary navigation panel displayed between the icon nav and the main
/// content area. Width is controlled via the `class` prop
/// (e.g., `class="w-48"` for 192px or `class="w-64"` for 256px).
///
/// Use conditional rendering (`Show`, `match`, etc.) to swap panel content
/// based on the active section from `AppShellContext`.
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn AppShellSidePanel(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Panel content
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "flex flex-col bg-base-200 overflow-y-auto",
                    class
                )
            }
        >
            {children()}
        </div>
    }
}

/// # AppShell Content Component
///
/// Main content area that fills the remaining horizontal space in the layout.
/// Scrolls independently of the icon nav and side panel.
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn AppShellContent(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Main content
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("flex-1 overflow-y-auto", class)
        >
            {children()}
        </div>
    }
}

/// Context manager for AppShell active section state.
///
/// Access this in any descendant of `AppShell` to read or write the
/// active section:
///
/// ```rust,ignore
/// let ctx = AppShellContext::expect_context();
/// let current = ctx.active_section.get(); // Option<String>
/// ctx.active_section.set(Some("settings".to_string()));
/// ```
#[derive(Clone)]
pub struct AppShellContext {
    /// Signal tracking the currently active section value
    pub active_section: RwSignal<Option<String>>,
    /// Whether the shell operates in manual mode (disables automatic tracking)
    manual: bool,
}

impl AppShellContext {
    /// Retrieves the AppShellContext from Leptos context.
    ///
    /// # Panics
    /// Panics if called outside of an `AppShell` component tree.
    pub fn expect_context() -> Self {
        use_context::<AppShellContext>()
            .expect("AppShell child components must be used within an AppShell component")
    }
}
