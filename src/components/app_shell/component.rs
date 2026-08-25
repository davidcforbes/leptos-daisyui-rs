use super::style::{app_shell_root_class, icon_nav_background_style, nav_group_class};
use crate::merge_classes;
use crate::utils::{badge_text, badge_visible};
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
///             <AppShellSidePanel class="w-70">
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
/// ### Status bar region
/// Pass [`AppShellStatusBar`] via the `status_bar` prop to pin a bottom
/// status/info bar below the main 3-panel row -- see [`AppShellStatusBar`]'s
/// docs for the full example. When `status_bar` is set, the root switches
/// to a column layout (`flex flex-col`) with the row area taking
/// `flex-1 min-h-0` so the status bar can sit at a fixed height below it;
/// when unset (the default), the root layout is unchanged from before this
/// prop existed.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col h-full w-full flex-1 min-h-0");
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

    /// Whether nav items show their text label alongside the icon.
    /// Defaults to `true` (labels shown when an item provides one via its
    /// `label` prop), preserving existing behavior. Set to `false` to
    /// collapse the icon nav to icon-only, mirroring d2d-ui's
    /// `AppShell::with_labels`.
    #[prop(optional, into, default = Signal::derive(|| true))]
    show_labels: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the root div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Optional status bar region, rendered pinned to the bottom of the
    /// shell below the main 3-panel row -- pass an [`AppShellStatusBar`]
    /// here. Structural -- like `icon` on
    /// [`EmptyState`](crate::components::empty_state::EmptyState) -- its
    /// presence is decided once, at creation.
    #[prop(optional)]
    status_bar: Option<Children>,

    /// Shell content (AppShellIconNav, AppShellSidePanel, AppShellContent)
    children: Children,
) -> impl IntoView {
    let ctx = AppShellContext {
        active_section,
        manual,
        show_labels,
    };
    provide_context(ctx);

    let has_status_bar = status_bar.is_some();

    view! {
        <div node_ref=node_ref class=move || merge_classes!(app_shell_root_class(has_status_bar), class)>
            {match status_bar {
                Some(status_bar_fn) => {
                    view! {
                        <div class="flex flex-1 min-h-0 w-full">{children()}</div>
                        {status_bar_fn()}
                    }
                        .into_any()
                }
                None => children().into_any(),
            }}
        </div>
    }
}

/// # AppShell Icon Nav Component
///
/// Narrow vertical navigation strip for section icons. Typically the leftmost
/// column in a 3-panel layout. Width is controlled via the `class` prop
/// (e.g., `class="w-16"` for 64px).
///
/// Children can freely mix [`AppShellHeader`] (a brand block at the top),
/// [`AppShellIconNavItem`]s, and [`AppShellIconNavGroup`] (e.g. a
/// bottom-pinned Settings item) -- this component itself is unchanged and
/// simply renders whatever it's given in a vertical flex column.
///
/// ## Branded background (`bg_color` / `bg_gradient` / `bg_image`)
///
/// The rail's background is composed of up to three layers, from back to
/// front: a **solid colour**, an optional **gradient**, and an optional
/// **texture image** on top. That is what a branded rail (dark navy base +
/// vertical gradient + a brand dot-wave PNG) needs:
///
/// ```rust,ignore
/// <AppShellIconNav
///     class="w-16"
///     bg_color="#0b1e3a"
///     bg_gradient="linear-gradient(180deg, rgba(255,255,255,0.06), rgba(0,0,0,0.25))"
///     bg_image="/assets/brand/sidebar-bg.png"
/// >
/// ```
///
/// Set the base colour with `bg_color`, **not** a `bg-[#0b1e3a]` class: the
/// rail already carries `bg-base-300`, and between two plain utility classes
/// the winner is whichever the generated sheet emits last (in practice
/// `bg-base-300`, leaving the rail grey). `bg_color` is an inline style, so it
/// outranks any class.
///
/// All three layers land on the `<nav>`'s own background, which always paints
/// *beneath* its content -- so item hover/active states and count badges stay
/// fully legible on top of the texture with no overlay element in the way.
/// All default to empty, and an unbranded rail renders exactly the markup it
/// did before, with no `style` attribute at all.
///
/// A repeating pattern (rather than one image stretched over the rail) wants
/// `bg_repeat="repeat"` together with `bg_size="auto"` -- the default
/// `cover` scales a single copy to fill the rail, which would defeat the tile.
///
/// `bg_gradient` is raw CSS (it has to be -- it is a `linear-gradient(...)`
/// expression), so treat it exactly like `class`: author it, never build it
/// from user input. `bg_image` is a URL and is escaped for you.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col items-center bg-base-300");
/// ```
///
/// ## Node References
/// - `node_ref` - References the nav element ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement))
#[component]
pub fn AppShellIconNav(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Optional base background colour for the rail (e.g. `"#0b1e3a"`,
    /// `"var(--brand-navy)"`), the bottom layer under `bg_gradient` and
    /// `bg_image`. Empty (default) = keep the `bg-base-300` class.
    ///
    /// This exists because a `bg-[#0b1e3a]` *class* does not reliably beat the
    /// rail's own `bg-base-300` class -- same cascade layer, and the sheet order
    /// decides. Set the colour here and it wins.
    #[prop(optional, into)]
    bg_color: Signal<String>,

    /// Optional background texture/pattern URL for the rail, painted on top of
    /// `bg_gradient` and `bg_color`, and beneath the nav items. Empty (default)
    /// = no image. Escaped before it reaches CSS.
    #[prop(optional, into)]
    bg_image: Signal<String>,

    /// Optional CSS gradient for the rail (e.g.
    /// `"linear-gradient(180deg, #0b1e3a, #071429)"`), painted beneath
    /// `bg_image`. Empty (default) = no gradient. Raw CSS -- trusted input,
    /// like `class`.
    #[prop(optional, into)]
    bg_gradient: Signal<String>,

    /// `background-size` for `bg_image` (default `"cover"`). Use `"auto"` for
    /// a repeating tile.
    #[prop(optional, into, default = Signal::derive(|| "cover".to_string()))]
    bg_size: Signal<String>,

    /// `background-repeat` for `bg_image` (default `"no-repeat"`). Use
    /// `"repeat"` for a tiling pattern.
    #[prop(optional, into, default = Signal::derive(|| "no-repeat".to_string()))]
    bg_repeat: Signal<String>,

    /// Reference to the nav element
    #[prop(optional)]
    node_ref: NodeRef<Nav>,

    /// Navigation items ([`AppShellHeader`], [`AppShellIconNavItem`],
    /// [`AppShellIconNavGroup`] components)
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
            // `None` when the rail is unbranded, so no empty `style` attribute
            // appears on markup that never asked for one.
            style=move || {
                let style = icon_nav_background_style(
                    &bg_color.get(),
                    &bg_image.get(),
                    &bg_gradient.get(),
                    &bg_size.get(),
                    &bg_repeat.get(),
                );
                (!style.is_empty()).then_some(style)
            }
        >
            {children()}
        </nav>
    }
}

/// # AppShell Header Component
///
/// An optional brand/logo block at the top of [`AppShellIconNav`], above the
/// nav items: an icon (`children`) plus an optional `title` shown next to it
/// only when the parent `AppShell`'s `show_labels` is `true` (icon-only when
/// collapsed) -- mirrors d2d-ui's `AppShell::with_header`.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col items-center gap-1 pb-2 text-primary");
/// @source inline("text-xs font-semibold text-center");
/// ```
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn AppShellHeader(
    /// Title shown under the icon, only while the parent `AppShell`'s
    /// `show_labels` is `true`. Empty (default) renders no title.
    #[prop(optional, into)]
    title: Signal<String>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Brand icon/logo content, always shown regardless of `show_labels`
    children: Children,
) -> impl IntoView {
    let AppShellContext { show_labels, .. } = AppShellContext::expect_context();

    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("flex flex-col items-center gap-1 pb-2 text-primary", class)
        >
            {children()}
            <Show when=move || show_labels.get() && !title.get().is_empty()>
                <span class="text-xs font-semibold text-center">{move || title.get()}</span>
            </Show>
        </div>
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
/// ### Text label (`label` prop)
/// Existing consumers put their icon *and* label markup directly in
/// `children` (as the doc example above does), which keeps working
/// unchanged. The optional `label` prop is an additive alternative: put only
/// the icon in `children` and pass the label text via `label`, and it
/// renders as a small caption under the icon plus an `aria-label` on the
/// button. The label is hidden (icon-only rail) when the parent `AppShell`'s
/// `show_labels` is `false`, mirroring d2d-ui's `AppShell::with_labels`.
///
/// ### Badge count (`badge` prop)
/// Pass `badge=Some(count)` to render a small count badge pinned to the
/// item's top-right corner, e.g. for an unread/exception count. `None` and
/// `Some(0)` both hide the badge; counts over 99 display as `"99+"`,
/// mirroring d2d-ui's `NavItem::with_badge`.
///
/// ### Manual mode, `on_click`, and `aria-current`
/// The rendered `<button>` carries `type="button"` so an `AppShell` embedded
/// in a `<form>` doesn't submit on nav clicks. When the parent `AppShell` has
/// `manual=true`, clicking an item no longer writes to `active_section` --
/// only the item's own `active` prop (driven externally, e.g. by a router)
/// determines its active state, mirroring [`NavRailItem`](crate::components::NavRailItem)'s
/// `manual` gating. The optional `on_click` callback fires on every click
/// regardless of `manual`, so callers can still react (e.g. to drive
/// external state in manual mode). The active item also gets
/// `aria-current="page"`.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("relative flex flex-col items-center justify-center gap-1 p-2 w-full cursor-pointer");
/// @source inline("text-xs");
/// @source inline("badge badge-error badge-sm absolute -top-1 -right-1");
/// ```
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

    /// Text label rendered under the icon (hidden when the parent
    /// `AppShell` has `show_labels=false`) and used as the button's
    /// `aria-label`. Leave unset to keep composing the label into
    /// `children` yourself (existing behavior).
    #[prop(optional, into)]
    label: Signal<String>,

    /// Optional count badge (e.g. unread/exception count) pinned to the
    /// item's top-right corner. `None` and `Some(0)` both hide it; counts
    /// over 99 display as `"99+"`.
    #[prop(optional, into)]
    badge: Signal<Option<u32>>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the button element
    #[prop(optional)]
    node_ref: NodeRef<Button>,

    /// Optional click callback, fired in addition to the built-in
    /// selection tracking (which itself is skipped when the parent
    /// `AppShell` has `manual=true`).
    #[prop(optional)]
    on_click: Option<Callback<()>>,

    /// Item content (icon and/or label)
    children: Children,
) -> impl IntoView {
    let AppShellContext {
        active_section,
        manual,
        show_labels,
    } = AppShellContext::expect_context();

    let on_button_click = move |_: ev::MouseEvent| {
        let v = value.get_untracked();
        if !manual && !v.is_empty() {
            active_section.set(Some(v));
        }
        if let Some(cb) = on_click {
            cb.run(());
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
            type="button"
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "relative flex flex-col items-center justify-center gap-1 p-2 w-full cursor-pointer",
                    class
                )
            }
            class:active=is_active
            aria-label=move || {
                let l = label.get();
                (!l.is_empty()).then_some(l)
            }
            aria-current=move || is_active().then_some("page")
            on:click=on_button_click
        >
            {children()}
            <Show when=move || show_labels.get() && !label.get().is_empty()>
                <span class="text-xs">{move || label.get()}</span>
            </Show>
            <Show when=move || badge_visible(badge.get())>
                <span class="badge badge-error badge-sm absolute -top-1 -right-1">
                    {move || badge_text(badge.get().unwrap_or(0))}
                </span>
            </Show>
        </button>
    }
}

/// # AppShell Icon Nav Group Component
///
/// Wraps a set of [`AppShellIconNavItem`]s within [`AppShellIconNav`].
/// Setting `pinned=true` pushes the group (and anything after it) to the
/// bottom of the icon nav strip via `mt-auto` -- the CSS equivalent of
/// d2d-ui's `AppShell::add_bottom_nav_item`, which stacked a second cluster
/// upward from the rail's foot. Mirrors [`NavRailGroup`](crate::components::NavRailGroup).
///
/// For the group to reach the icon nav's bottom edge, the icon nav's parent
/// (the `AppShell`) needs a definite height (e.g. `h-full` or `h-screen`),
/// which the default `AppShell` root already provides.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col items-center gap-1 mt-auto");
/// ```
///
/// ## Node References
/// - `node_ref` - References the group `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn AppShellIconNavGroup(
    /// Pin this group to the bottom of the icon nav strip (`mt-auto`).
    #[prop(optional, into)]
    pinned: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the group div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Group content ([`AppShellIconNavItem`] components)
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!(nav_group_class(pinned.get()), class)>
            {children()}
        </div>
    }
}

/// # AppShell Side Panel Component
///
/// Secondary navigation panel displayed between the icon nav and the main
/// content area. Width is controlled via the `width` prop or, as before, via
/// the `class` prop -- both are additive to the base classes.
///
/// **Use `width="w-70"` (280px).** That is `ui_tokens::spacing::RIGHT_PANEL_WIDTH`,
/// the same panel width the Direct2D desktop face draws, so the two faces
/// agree. The older `w-48` (192px) and `w-64` (256px) examples are on the 4px
/// grid but not on the canonical scale, and neither matches the desktop.
///
/// Use conditional rendering (`Show`, `match`, etc.) to swap panel content
/// based on the active section from `AppShellContext`.
///
/// ### Collapsing the panel
/// Set `visible=false` (e.g. from an `RwSignal<bool>` toggled by a
/// hamburger/chevron button) to collapse the panel. A collapsed panel is
/// removed from the flex layout (`hidden`), so `AppShellContent` reflows to
/// fill the freed width -- mirroring d2d-ui's `AppShell::toggle_sidebar` /
/// `set_sidebar_visible`.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col bg-base-200 overflow-y-auto");
/// @source inline("hidden");
/// ```
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn AppShellSidePanel(
    /// Whether the panel is visible. Defaults to `true`, preserving
    /// existing behavior; set to `false` (or bind an `RwSignal<bool>`) to
    /// collapse the panel and let `AppShellContent` reflow into the freed
    /// space.
    #[prop(optional, into, default = Signal::derive(|| true))]
    visible: Signal<bool>,

    /// Width class for the panel. Prefer `"w-70"` (280px) --- see the
    /// component docs for why. Leave unset
    /// (default) to size the panel entirely via the `class` prop, as
    /// before.
    #[prop(optional, into)]
    width: &'static str,

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
                    width,
                    class
                )
            }
            class:hidden=move || !visible.get()
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
/// ### Add to `input.css`
/// ```css
/// @source inline("flex-1 overflow-y-auto");
/// ```
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
            role="main"
            tabindex="0"
            class=move || merge_classes!("flex-1 overflow-y-auto", class)
        >
            {children()}
        </div>
    }
}

/// # AppShell Status Bar Component
///
/// An optional bottom status/info bar -- pass it to `AppShell`'s
/// `status_bar` prop to pin it below the main 3-panel row. `AppShell`
/// arranges the layout for you: the row area takes `flex-1 min-h-0` and the
/// status bar sits `shrink-0` beneath it.
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::*;
///
/// #[component]
/// fn MyApp() -> impl IntoView {
///     let section = RwSignal::new(Some("home".to_string()));
///     let show_status = RwSignal::new(true);
///
///     view! {
///         <AppShell
///             active_section=section
///             status_bar=Box::new(move || {
///                 view! {
///                     <AppShellStatusBar show=show_status>"Ready"</AppShellStatusBar>
///                 }
///                     .into_any()
///             })
///         >
///             <AppShellIconNav class="w-16">
///                 <AppShellIconNavItem value="home">"Home"</AppShellIconNavItem>
///             </AppShellIconNav>
///             <AppShellContent class="p-6">"Main content"</AppShellContent>
///         </AppShell>
///     }
/// }
/// ```
///
/// `show` accepts an `RwSignal<bool>` so callers can both show/hide it and
/// toggle it (`show.update(|v| *v = !*v)`) -- mirrors d2d-ui's
/// `AppShell::with_status_bar` / `toggle_status_bar`.
///
/// Standalone use (outside the `status_bar` slot, e.g. hand-wrapped in your
/// own `flex flex-col` container) remains possible -- the `shrink-0` in its
/// base classes is harmless there too -- but is no longer the documented
/// path.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex shrink-0 items-center gap-2 border-t border-base-300 bg-base-200 px-4 text-xs text-base-content/80");
/// @source inline("hidden");
/// ```
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn AppShellStatusBar(
    /// Whether the status bar is visible. Defaults to `true`; bind an
    /// `RwSignal<bool>` to show/hide or toggle it programmatically.
    #[prop(optional, into, default = Signal::derive(|| true))]
    show: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Status bar content (e.g. status text, small controls)
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "flex shrink-0 items-center gap-2 border-t border-base-300 bg-base-200 px-4 text-xs text-base-content/80",
                    class
                )
            }
            class:hidden=move || !show.get()
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
    /// Whether nav items and the header title show their text label. See
    /// `AppShell`'s `show_labels` prop.
    show_labels: Signal<bool>,
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
