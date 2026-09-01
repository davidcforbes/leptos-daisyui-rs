use super::style::{FilterSidebarTogglePlacement, SidebarSide, join_side_class};
use crate::components::icon::{Icon, IconSize};
use leptos::{
    html::{Aside, Button},
    prelude::*,
};
use std::sync::atomic::{AtomicU64, Ordering};

/// Process-wide sequence for the search input's `id`/`label[for]` pairing, so
/// multiple `FilterSidebar` instances on one page stay independently named.
/// Mirrors `data_table`'s `next_data_table_search_id`.
static FILTER_SIDEBAR_SEARCH_ID: AtomicU64 = AtomicU64::new(0);

fn next_filter_sidebar_search_id() -> String {
    format!(
        "ldui-filter-sidebar-search-{}",
        FILTER_SIDEBAR_SEARCH_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// Builds the header row's optional panel-scoped controls slot, or emits
/// nothing at all when the consumer omits it (`ldui-bx6n`). Follows the same
/// `Option`-gated idiom as `FilterBar`'s own optional slots
/// (`filter_bar_children_wrapper`): an unconditionally-rendered empty wrapper
/// is a phantom flex item in the header row's `justify-between` layout.
///
/// Fades and stops receiving pointer events alongside the title when the
/// panel collapses -- the same "hidden in place, never unmounted" treatment
/// the header title and the main content area already get, so collapsing
/// cannot discard interaction state inside a header control.
///
/// **`ldui-8hba` regression, fixed here.** `opacity-0 pointer-events-none`
/// alone hides *paint*, not *layout*: the wrapper's `shrink-0` kept its full
/// intrinsic width even while invisible, so on a right-docked panel with a
/// wide slot (a Select plus a setup action) the collapsed header's
/// `flex-row-reverse` line demanded far more width than the 44px rail has.
/// With nothing left to absorb the overflow (the title has already shrunk to
/// 0), the toggle button was pushed out past the panel's own edge and
/// clipped by the panel's `overflow-hidden` -- `elementFromPoint` at the
/// toggle's visual center resolved whatever sat underneath instead. Collapsed
/// state now zeroes the wrapper's actual footprint (`w-0`, no margin, its own
/// `overflow-hidden` so children cannot bleed past it) rather than merely
/// fading it, so it contributes nothing to the header's flex layout and the
/// toggle keeps the full available width to itself. The slot stays mounted
/// (per the module doc comment) and additionally goes `inert` -- excluded
/// from both the tab order and the accessibility tree, belt-and-braces with
/// `aria-hidden`, matching the "visually hidden must not still be reachable"
/// rule -- rather than merely uninteractive under `pointer-events-none`,
/// which a keyboard or screen-reader user could still reach.
pub(crate) fn filter_sidebar_header_actions_wrapper(
    header_actions: Option<Children>,
    collapsed: Signal<bool>,
) -> Option<impl IntoView> {
    header_actions.map(|header_actions| {
        view! {
            <div
                class=move || {
                    format!(
                        "flex items-center gap-2 transition-opacity duration-[150ms] {}",
                        if collapsed.get() {
                            "mx-0 w-0 overflow-hidden opacity-0 pointer-events-none"
                        } else {
                            "mx-2 shrink-0 opacity-100"
                        },
                    )
                }
                inert=move || collapsed.get()
                aria-hidden=move || collapsed.get().to_string()
                data-filter-sidebar-header-actions="true"
            >
                {header_actions()}
            </div>
        }
    })
}

/// Renders the ONE toggle-button markup shared by [`FilterSidebar`]'s own
/// internal header toggle and the externally placed [`FilterSidebarToggle`]
/// (`ldui-vshu`) -- a single source of truth so the two can never drift
/// apart in label, `aria-expanded`, class treatment, or click wiring. The
/// only thing that differs between the two call sites is `controls`: the
/// internal toggle has historically carried no `aria-controls` (added here
/// would be a behaviour change for a mode the bead requires stay
/// unchanged), while the external one always does, since it is the only
/// thing telling assistive tech what a *separately placed* button expands.
fn filter_sidebar_toggle_button(
    collapsed: Signal<bool>,
    on_toggle: Callback<()>,
    toggle_label: Signal<String>,
    side: Signal<SidebarSide>,
    controls: Option<Signal<String>>,
    node_ref: NodeRef<Button>,
) -> impl IntoView {
    view! {
        <button
            node_ref=node_ref
            type="button"
            aria-label=move || toggle_label.get()
            aria-expanded=move || (!collapsed.get()).to_string()
            aria-controls=move || controls.map(|c| c.get())
            // `btn` first so this reads as a real daisyUI button to
            // style tooling; every utility after it re-asserts the
            // panel toggle's own geometry and quiet palette.
            class="btn btn-square btn-xs size-7 shrink-0 rounded-md \
                   border border-black/[.12] bg-base-100 text-base-content/75 \
                   shadow-none hover:bg-black/[.06] hover:text-base-content"
            on:click=move |_| on_toggle.run(())
            // Unambiguous selector for tests: `header_actions` can itself
            // contain a `<button aria-label>` (e.g. a setup action), so
            // `[aria-label]` alone does not uniquely identify the toggle
            // (`ldui-8hba`).
            data-filter-sidebar-toggle="true"
        >
            // The arrow points where the panel would GO, so it depends on
            // the side AND the collapsed state - see `SidebarSide::chevron_name`.
            <Icon
                name=Signal::derive(move || {
                    side.get().chevron_name(collapsed.get()).to_string()
                })
                size=IconSize::XSmall
            />
        </button>
    }
}

/// # FilterSidebarToggle
///
/// The panel's collapse/expand control, placed OUTSIDE the panel itself.
/// Pair with `<FilterSidebar toggle_placement=FilterSidebarTogglePlacement::External ...>`
/// (`ldui-vshu`) when a consumer already has an established page-level
/// Hide/Show action and must not also render the panel's own internal
/// chevron -- rendering both would put two controls for one signal in the
/// accessibility tree, which is exactly what this component exists to avoid.
///
/// Renders the identical markup [`FilterSidebar`]'s own internal toggle
/// would have -- same label wiring, same `aria-expanded`, same hover/focus
/// treatment, same click handling -- via [`filter_sidebar_toggle_button`],
/// so the two paths cannot drift apart. The one addition is `controls`,
/// which becomes this button's `aria-controls`: pass the SAME string you
/// gave `FilterSidebar` via `attr:id`.
///
/// ## Exact consumer call shape (`ldui-vshu` — Office's Conversation Detail)
///
/// ```ignore
/// let collapsed = RwSignal::new(false);
/// let on_toggle = Callback::new(move |()| collapsed.update(|c| *c = !*c));
///
/// view! {
///     // Wherever the page's own action row lives -- need not be anywhere
///     // near the panel in the DOM.
///     <FilterSidebarToggle
///         collapsed=collapsed
///         on_toggle=on_toggle
///         toggle_label="Toggle the assistant panel"
///         side=SidebarSide::Right
///         controls="conversation-assistant-panel"
///     />
///     // ... the rest of the page ...
///     <FilterSidebar
///         attr:id="conversation-assistant-panel"
///         toggle_placement=FilterSidebarTogglePlacement::External
///         collapsed=collapsed
///         on_toggle=on_toggle
///         toggle_label="Toggle the assistant panel"
///         side=SidebarSide::Right
///         active_count=active_count
///         title="Assistant"
///     >
///         { /* filter fields */ }
///     </FilterSidebar>
/// }
/// ```
///
/// `collapsed`, `on_toggle`, `toggle_label` and `side` are passed to BOTH
/// components -- there is nothing to keep in sync beyond reusing the same
/// variables, since `Signal`/`Callback` are cheap to pass twice. Only
/// `controls`/`attr:id` are a pair of matching strings the consumer must
/// keep aligned, exactly like every other `aria-controls` relationship in
/// this crate (see `Tab`/`TabPanel`).
///
/// ## Node References
/// - `node_ref` - References the button element
///   ([HTMLButtonElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement)).
#[component]
pub fn FilterSidebarToggle(
    /// Whether the paired panel is collapsed. Pass the SAME signal given to
    /// that `FilterSidebar`'s own `collapsed` prop.
    #[prop(into)]
    collapsed: Signal<bool>,
    /// Fired when this toggle is activated. Pass the SAME callback given to
    /// the paired `FilterSidebar`'s own `on_toggle` prop.
    #[prop(into)]
    on_toggle: Callback<()>,
    /// Accessible label for this icon-only button. Pass the SAME text given
    /// to the paired `FilterSidebar`'s own `toggle_label` prop.
    #[prop(into)]
    toggle_label: Signal<String>,
    /// Which edge the paired panel docks against, so the chevron points the
    /// way the panel would move. Pass the SAME value given to the paired
    /// `FilterSidebar`'s own `side` prop. Defaults to [`SidebarSide::Left`],
    /// matching that prop's own default.
    #[prop(optional, into)]
    side: Signal<SidebarSide>,
    /// The paired `FilterSidebar`'s DOM id (set on that panel via
    /// `attr:id`), so this button's `aria-controls` truthfully names the
    /// panel it expands and collapses.
    #[prop(into)]
    controls: Signal<String>,
    /// Reference to the button element
    /// ([HTMLButtonElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement)).
    #[prop(optional)]
    node_ref: NodeRef<Button>,
) -> impl IntoView {
    filter_sidebar_toggle_button(
        collapsed,
        on_toggle,
        toggle_label,
        side,
        Some(controls),
        node_ref,
    )
}

/// # FilterSidebar
///
/// A collapsible side panel: header with a toggle, a filter-search field,
/// stacked labelled field blocks, an optional saved-filter list, and — when
/// collapsed — a narrow rail carrying the ACTIVE FILTER COUNT and a vertical
/// label.
///
/// Docks against either edge via [`side`](SidebarSide); left by default, which
/// is what it always was.
///
/// Every dimension here is measured from a running reference implementation
/// (4Ease's `pilot-filter-sidebar`), not chosen: 220px expanded, 44px collapsed,
/// a 52px header, `width .25s ease`.
///
/// ## The detail that matters most
///
/// **The collapsed rail is not empty.** It shows how many filters are active. A
/// collapsed filter panel that gives no such indication is how people read a
/// filtered list as if it were the whole list — and on a work queue that means
/// believing there is less work than there is. If you simplify anything here,
/// do not simplify that.
///
/// ## Nothing is unmounted
///
/// Collapsing toggles classes; the content stays in the DOM at `opacity-0` and
/// `pointer-events-none`. That keeps the width transition smooth, preserves scroll
/// position, and means collapsing cannot discard a half-typed filter value.
/// Unmounting would be less code and worse behaviour.
///
/// **It also goes `inert` (`ldui-gae5`).** `opacity-0 pointer-events-none`
/// hides paint, not the accessibility tree or the tab order -- a collapsed
/// panel's search box and every control inside `children` stayed
/// keyboard-focusable and screen-reader-announced despite being invisible
/// and unclickable. The main content area is now `inert` (plus `aria-hidden`
/// belt-and-braces) while collapsed, matching the treatment `header_actions`
/// already gets (`ldui-8hba`). `inert` changes focusability and
/// announcement only -- it does not affect layout or paint, so the
/// `<aside>`'s own `overflow-hidden` keeps clipping the content during the
/// width transition exactly as before.
///
/// ## Composition
///
/// Checked before writing this: the library's `Filter` is a daisyUI radio-button
/// group and `Drawer` is an off-canvas overlay driven by a hidden checkbox.
/// Neither is an inline, width-animating panel that participates in page layout,
/// so there was nothing to compose from.
///
/// ## CSS Classes
///
/// Add to your `input.css` — these are arbitrary-value utilities that a source
/// scan cannot discover inside this crate:
/// ```css
/// @source inline("w-[220px] min-w-[220px] w-[44px] min-w-[44px] h-[52px]");
/// @source inline("transition-[width,min-width] duration-[250ms] ease-out");
/// @source inline("opacity-0 opacity-100 pointer-events-none");
/// @source inline("border-r border-l flex-row-reverse rotate-180");
/// @source inline("[writing-mode:vertical-rl] [text-orientation:mixed]");
/// ```
///
/// The last two lines are the [`SidebarSide::Right`] half of every mirrored
/// pair. A project that scans this crate's sources already picks them up —
/// each is a bare string literal in `style.rs` — so listing them is belt and
/// braces, and the braces are there because the RIGHT halves are the ones that
/// go missing and they go missing *silently*.
///
/// Measured while this was written: the demo's built CSS carried
/// `.lg\:flex-row-reverse` and no bare `.flex-row-reverse` rule at all,
/// because Tailwind generates the variant it saw and never the base. The day
/// one of these stops being a bare literal — concatenated, hidden behind a
/// `const`, prefixed at its only call site — the right-hand panel quietly
/// loses its reversed header and renders merely odd rather than broken.
#[component]
pub fn FilterSidebar(
    /// Whether the panel is collapsed. CONTROLLED: the consumer owns this, so it
    /// can be persisted per user and restored on the next visit.
    #[prop(into)]
    collapsed: Signal<bool>,
    /// Fired when the toggle is clicked. The consumer flips `collapsed`.
    #[prop(into)]
    on_toggle: Callback<()>,
    /// How many filters are currently applied. Drives the collapsed badge, and is
    /// the whole reason a collapsed panel is still informative. `0` hides it.
    #[prop(into)]
    active_count: Signal<usize>,
    /// Panel title, shown in the header when expanded and rotated vertically on
    /// the collapsed rail.
    #[prop(into)]
    title: Signal<String>,
    /// Optional panel-scoped controls rendered in the SAME header row as the
    /// title and toggle (e.g. a model selector plus a setup action for an
    /// Assistant panel). `None` omits the wrapper entirely -- an
    /// unconditionally-rendered empty wrapper is a phantom flex item, the
    /// same reasoning `FilterBar` documents for its own optional slots.
    /// Shown only in the EXPANDED header: it fades out and collapses to zero
    /// layout width and no pointer events when the panel collapses
    /// (`ldui-8hba` -- retaining its width while merely invisible pushed the
    /// toggle button outside the panel on the right-docked orientation, see
    /// [`filter_sidebar_header_actions_wrapper`] for the full story), and
    /// goes `inert` so it drops out of the tab order and accessibility tree
    /// too. Sits between the title and the toggle in DOM order, so left/right
    /// mirroring (`flex-row-reverse` on [`SidebarSide::Right`]) carries it
    /// automatically -- see the header's own comment for why the toggle
    /// stays at the panel's inner edge.
    #[prop(optional)]
    header_actions: Option<Children>,
    /// Optional filter-search box. `None` omits it entirely rather than rendering
    /// a disabled one — a search field that cannot search is worse than none.
    #[prop(optional, into)]
    search: Option<RwSignal<String>>,
    /// Placeholder for that search box.
    #[prop(optional, into)]
    search_placeholder: Signal<String>,
    /// Accessible name for the search box, wired via a visually-hidden
    /// `<label>` associated by `id`/`for` (and an `aria-label` for belt and
    /// braces, matching `data_table`'s search input). Independent of
    /// `search_placeholder` and of the typed value: placeholder text is not
    /// an accessible name -- it vanishes once the field has a value, and many
    /// screen readers never announce it at all. Reactive, so it can trail a
    /// locale switch without touching the search signal, caret, focus, or
    /// typed text. Defaults to `"Search filters"` when omitted, so every
    /// existing call site compiles unchanged and gains a real accessible name
    /// rather than none. Ignored when `search` is `None` -- no label is
    /// emitted for a search box that does not exist.
    #[prop(into, default = Signal::stored(String::from("Search filters")))]
    search_label: Signal<String>,
    /// Accessible label for the toggle button. Localise it; the button is
    /// icon-only, so this is the only thing a screen reader announces.
    /// Read even when [`FilterSidebarTogglePlacement::External`] omits the
    /// panel's own button, so the SAME string can be passed straight through
    /// to [`FilterSidebarToggle`]'s own `toggle_label`.
    #[prop(into)]
    toggle_label: Signal<String>,
    /// Where the collapse/expand control is rendered. Defaults to
    /// [`FilterSidebarTogglePlacement::Internal`] -- the panel's own header
    /// button, the only behaviour that existed before `ldui-vshu`, so every
    /// existing call site is unchanged. Pass
    /// [`FilterSidebarTogglePlacement::External`] when the consumer places a
    /// [`FilterSidebarToggle`] elsewhere in their own layout; the panel then
    /// renders NO toggle of its own -- not a hidden or `inert` one -- so
    /// there is no phantom control and no gap left in the header for one.
    #[prop(optional, into)]
    toggle_placement: Signal<FilterSidebarTogglePlacement>,
    /// Which page edge the panel docks against. See [`SidebarSide`] for the
    /// four things that mirror and the two that deliberately do not. Defaults
    /// to [`SidebarSide::Left`], the only behaviour that existed before, so
    /// every existing call site is unchanged.
    #[prop(optional, into)]
    side: Signal<SidebarSide>,
    /// Expanded width classes. Defaults to the measured `220px`, which every
    /// existing call site keeps. Override for a panel hosting content wider
    /// than a filter column (e.g. an assistant card) — pass the full class
    /// pair (`"w-[360px] min-w-[360px]"`), and remember the consumer's CSS
    /// must `@source inline(...)` any arbitrary-value classes it passes,
    /// exactly as the type docs require for this crate's own.
    #[prop(default = String::from("w-[220px] min-w-[220px]"), into)]
    expanded_width: String,
    /// Reference to the panel element
    /// ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement)).
    #[prop(optional)]
    node_ref: NodeRef<Aside>,
    /// The filter field blocks — typically [`FilterSection`] wrapping
    /// [`FilterField`]s. Supplied by the consumer, because WHICH filters a screen
    /// offers is application knowledge, not a property of the panel.
    children: Children,
) -> impl IntoView {
    let width_class = move || {
        if collapsed.get() {
            "w-[44px] min-w-[44px]".to_string()
        } else {
            expanded_width.clone()
        }
    };
    // `opacity-0 pointer-events-none` rather than unmounting — see the type docs.
    let hidden_when_collapsed = move || {
        if collapsed.get() {
            "opacity-0 pointer-events-none"
        } else {
            "opacity-100"
        }
    };
    // Computed unconditionally, exactly like `data_table`'s
    // `search_input_id` -- cheap, and it means the id assignment does not
    // itself depend on whether `search` happens to be `Some` this call.
    let search_input_id = next_filter_sidebar_search_id();

    view! {
        <aside
            node_ref=node_ref
            // `overflow-hidden` is load-bearing: the 220px content must be
            // CLIPPED during the collapse, not reflowed, or the transition
            // visibly reflows text on every frame.
            class=move || {
                format!(
                    "relative flex h-full flex-col overflow-hidden {} \
                     border-black/[.08] bg-base-100 \
                     transition-[width,min-width] duration-[250ms] ease-out {}",
                    side.get().as_border_class(),
                    width_class(),
                )
            }
        >
            // ── header: 52px, and the toggle NEVER hides ────────────────────
            // The title fades out; the button stays, because it is the only way
            // back and a control that disappears when used is a trap.
            //
            // `flex-row-reverse` on a right-hand panel: the toggle belongs at
            // the panel's INNER edge, beside the content it reveals, not
            // pinned to the window frame. `justify-between` then pushes the
            // title to the outer edge, which is the mirror of what a left
            // panel does.
            <div class=move || {
                join_side_class(
                    "flex h-[52px] shrink-0 items-center justify-between px-3 pb-2.5 pt-3.5",
                    side.get().as_header_class(),
                )
            }>
                <h3
                    class=move || {
                        format!(
                            "min-w-0 flex-1 truncate text-[15px] font-bold \
                             transition-opacity duration-[150ms] {}",
                            hidden_when_collapsed(),
                        )
                    }
                >
                    {move || title.get()}
                </h3>
                {filter_sidebar_header_actions_wrapper(header_actions, collapsed)}
                // `ldui-vshu`: rendered only for the default
                // `Internal` placement -- `External` omits this node
                // entirely, not merely hides it, so there is no
                // phantom/duplicate toggle and no header gap left behind
                // for one. `controls=None` preserves the internal toggle's
                // pre-`ldui-vshu` markup byte-for-byte (no `aria-controls`),
                // exactly as the "default/internal mode is unchanged"
                // acceptance criterion requires.
                {move || {
                    matches!(toggle_placement.get(), FilterSidebarTogglePlacement::Internal)
                        .then(|| {
                            filter_sidebar_toggle_button(
                                collapsed,
                                on_toggle,
                                toggle_label,
                                side,
                                None,
                                NodeRef::new(),
                            )
                        })
                }}
            </div>

            // ── expanded content ────────────────────────────────────────────
            // `ldui-gae5`: `opacity-0 pointer-events-none` alone hides
            // PAINT, not the accessibility tree or the tab order -- a
            // collapsed panel's search input and every field/control inside
            // `children` (a textarea, an action button, anything) stayed
            // keyboard-focusable and screen-reader-announced. `inert` closes
            // both gaps at once (a non-focusable, non-announced subtree,
            // same guarantee `ldui-8hba` already gives `header_actions`
            // above), with `aria-hidden` alongside it belt-and-braces for a
            // user agent that does not yet honour `inert`. `overflow-hidden`
            // on the `<aside>` above stays the thing that CLIPS this content
            // during the width transition -- `inert` changes focusability
            // and announcement, nothing about layout or paint, so the
            // collapse still clips rather than reflows.
            <div
                class=move || {
                    format!(
                        "min-h-0 flex-1 overflow-y-auto px-3 pb-4 \
                         transition-opacity duration-[150ms] {}",
                        hidden_when_collapsed(),
                    )
                }
                inert=move || collapsed.get()
                aria-hidden=move || collapsed.get().to_string()
                data-filter-sidebar-content="true"
            >
                {search
                    .map(|s| {
                        let label_target = search_input_id.clone();
                        let control_id = search_input_id.clone();
                        view! {
                            // NOT mirrored, on purpose: the magnifier's
                            // `left-2.5` and the input's `pl-[30px]` are
                            // READING direction, not panel orientation. A
                            // search box looks the same whichever edge of the
                            // screen it sits near, and flipping it would put
                            // the icon where the caret goes.
                            <div class="relative mb-3">
                                // Visually hidden, but the input's REAL
                                // accessible name -- independent of the
                                // placeholder (which vanishes once the field
                                // has a value) and of the typed value itself.
                                // `aria-label` below is belt and braces, same
                                // as `data_table`'s search input.
                                <label class="sr-only" r#for=label_target>
                                    {move || search_label.get()}
                                </label>
                                <span class="pointer-events-none absolute left-2.5 top-1/2 \
                                             -translate-y-1/2 text-[13px] text-base-content/45">
                                    <Icon name="search" size=IconSize::XSmall />
                                </span>
                                <input
                                    id=control_id
                                    type="text"
                                    class="w-full rounded-md border border-black/20 py-1.5 \
                                           pl-[30px] pr-2.5 text-[13px]"
                                    placeholder=move || search_placeholder.get()
                                    aria-label=move || search_label.get()
                                    prop:value=move || s.get()
                                    on:input=move |ev| s.set(event_target_value(&ev))
                                />
                            </div>
                        }
                    })}
                {children()}
            </div>

            // ── collapsed rail ──────────────────────────────────────────────
            // Absolutely positioned BELOW the header so it cannot cover the
            // toggle. Fades in on a delay so it does not cross-fade with the
            // content on its way out.
            <div
                class=move || {
                    format!(
                        "absolute inset-x-0 bottom-0 top-[52px] flex flex-col items-center \
                         gap-1.5 pt-5 transition-opacity delay-[100ms] duration-[180ms] {}",
                        if collapsed.get() {
                            "opacity-100"
                        } else {
                            "opacity-0 pointer-events-none"
                        },
                    )
                }
                aria-hidden=move || (!collapsed.get()).to_string()
            >
                // `!= 0` rather than `> 0`: inside `view!` a bare `>` is parsed as a
                // tag close and the macro fails with a type error pointing at the
                // signal, which reads like a signal problem and is not one.
                <Show when=move || active_count.get() != 0>
                    <span
                        class="flex size-[22px] items-center justify-center rounded-full \
                               bg-primary text-[11px] font-bold text-primary-content"
                        // Announced, not decorative: this is the only thing that
                        // tells a reader the list beneath is filtered.
                        role="status"
                    >
                        {move || active_count.get()}
                    </span>
                </Show>
                // Bottom-to-top on a left edge, top-to-bottom on a right one:
                // the convention every IDE tool-window already follows. See
                // `SidebarSide::as_rail_title_class`.
                <span class=move || {
                    format!(
                        "text-[13px] font-semibold tracking-[.04em] text-base-content/75 {}",
                        side.get().as_rail_title_class(),
                    )
                }>{move || title.get()}</span>
            </div>
        </aside>
    }
}

/// A titled group of filter fields. The title is 4Ease's "eyebrow": small, bold,
/// uppercase, widely tracked.
#[component]
pub fn FilterSection(
    /// The eyebrow text, e.g. "Filter by fields".
    #[prop(into)]
    title: Signal<String>,
    /// The fields in this group.
    children: Children,
) -> impl IntoView {
    view! {
        <div class="mb-4">
            <p class="mb-[5px] text-[14px] font-bold uppercase tracking-[.05em] text-base-content/75">
                {move || title.get()}
            </p>
            <div class="flex flex-col gap-2.5">{children()}</div>
        </div>
    }
}

/// One labelled filter control, label ABOVE the control.
///
/// Stacked rather than side-by-side because a 220px panel cannot fit a readable
/// label beside a usable control — the reference stacks them for the same reason.
#[component]
pub fn FilterField(
    /// Label shown above the control.
    #[prop(into)]
    label: Signal<String>,
    /// The control itself — an input, select, or anything else.
    children: Children,
) -> impl IntoView {
    view! {
        <label class="flex flex-col gap-1">
            <span class="text-[12px] font-medium text-base-content/75">{move || label.get()}</span>
            {children()}
        </label>
    }
}
