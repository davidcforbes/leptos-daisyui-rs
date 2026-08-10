use super::style::{SidebarSide, join_side_class};
use crate::components::icon::{Icon, IconSize};
use leptos::{html::Aside, prelude::*};

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
    /// Optional filter-search box. `None` omits it entirely rather than rendering
    /// a disabled one — a search field that cannot search is worse than none.
    #[prop(optional, into)]
    search: Option<RwSignal<String>>,
    /// Placeholder for that search box.
    #[prop(optional, into)]
    search_placeholder: Signal<String>,
    /// Accessible label for the toggle button. Localise it; the button is
    /// icon-only, so this is the only thing a screen reader announces.
    #[prop(into)]
    toggle_label: Signal<String>,
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
                <button
                    type="button"
                    aria-label=move || toggle_label.get()
                    aria-expanded=move || (!collapsed.get()).to_string()
                    class="flex size-7 shrink-0 items-center justify-center rounded-md \
                           border border-black/[.12] bg-base-100 text-base-content/75 \
                           hover:bg-black/[.06] hover:text-base-content"
                    on:click=move |_| on_toggle.run(())
                >
                    // The arrow points where the panel would GO, so it depends
                    // on the side AND the collapsed state - see
                    // `SidebarSide::chevron_name`.
                    <Icon
                        name=Signal::derive(move || {
                            side.get().chevron_name(collapsed.get()).to_string()
                        })
                        size=IconSize::XSmall
                    />
                </button>
            </div>

            // ── expanded content ────────────────────────────────────────────
            <div
                class=move || {
                    format!(
                        "min-h-0 flex-1 overflow-y-auto px-3 pb-4 \
                         transition-opacity duration-[150ms] {}",
                        hidden_when_collapsed(),
                    )
                }
            >
                {search
                    .map(|s| {
                        view! {
                            // NOT mirrored, on purpose: the magnifier's
                            // `left-2.5` and the input's `pl-[30px]` are
                            // READING direction, not panel orientation. A
                            // search box looks the same whichever edge of the
                            // screen it sits near, and flipping it would put
                            // the icon where the caret goes.
                            <div class="relative mb-3">
                                <span class="pointer-events-none absolute left-2.5 top-1/2 \
                                             -translate-y-1/2 text-[13px] text-base-content/45">
                                    <Icon name="search" size=IconSize::XSmall />
                                </span>
                                <input
                                    type="text"
                                    class="w-full rounded-md border border-black/20 py-1.5 \
                                           pl-[30px] pr-2.5 text-[13px]"
                                    placeholder=move || search_placeholder.get()
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
