use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// The filter blocks both orientations share, so the two panels differ in
/// nothing but `side` and a reviewer can read any asymmetry as a bug.
#[component]
fn ExampleFilters() -> impl IntoView {
    view! {
        <FilterSection title="Filter by fields">
            <FilterField label="Status">
                <select class="select select-sm w-full">
                    <option>"Any"</option>
                    <option>"Open"</option>
                    <option>"Closed"</option>
                </select>
            </FilterField>
            <FilterField label="Owner">
                <select class="select select-sm w-full">
                    <option>"Anyone"</option>
                    <option>"Ada Lovelace"</option>
                    <option>"Grace Hopper"</option>
                </select>
            </FilterField>
            <FilterField label="Reference">
                <input type="text" class="input input-sm w-full" placeholder="e.g. CC-1042" />
            </FilterField>
        </FilterSection>
    }
}

/// One panel plus the stub of page content it sits beside, so the border lands
/// on a real seam rather than on nothing.
#[component]
fn Workspace(
    #[prop(into)] side: Signal<SidebarSide>,
    #[prop(into)] collapsed: Signal<bool>,
    on_toggle: Callback<()>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] active_count: Signal<usize>,
    #[prop(into)] body: Signal<String>,
    search: RwSignal<String>,
    /// Accessible name for this panel's search input (ldui-g66e). Given
    /// explicitly and distinctly per panel below so the browser fixture can
    /// prove multiple `FilterSidebar`s stay independently named -- the
    /// library's own documented "Search filters" fallback (used when a
    /// caller omits the prop entirely) is covered by the native tests in
    /// `filter_sidebar::tests` instead, since that is a compile-time default
    /// this fixed demo layout cannot itself vary.
    #[prop(into)]
    search_label: Signal<String>,
    /// A stable id on the panel's root `<aside>`, via the spread-attribute
    /// forwarding every component here gets for free -- so the reactivity
    /// fixture can select one specific panel among the several this page
    /// renders.
    #[prop(into)]
    panel_id: &'static str,
) -> impl IntoView {
    // `side` is read twice below and `SidebarSide` is `Copy`, so this is a
    // read, not a clone of anything expensive.
    let panel_is_right = Signal::derive(move || side.get() == SidebarSide::Right);

    let panel = move || {
        view! {
            <FilterSidebar
                attr:id=panel_id
                side=side
                collapsed=collapsed
                on_toggle=on_toggle
                active_count=active_count
                title=title
                search=search
                // Deliberately NOT the same text as any `search_label` value
                // below (ldui-g66e): the placeholder and the accessible name
                // are two different things, and identical strings would let
                // a fixture asserting their independence pass by accident.
                search_placeholder="Type to filter…"
                search_label=search_label
                toggle_label="Toggle the panel"
            >
                <ExampleFilters />
            </FilterSidebar>
        }
    };

    view! {
        // h-96 (384px) is a SIZE, not spacing: the panel is `h-full` and needs
        // a bounded parent or the collapsed rail has no rail to sit in.
        <div class="flex h-96 w-full overflow-hidden rounded-lg border border-base-300">
            <Show when=move || !panel_is_right.get()>{panel}</Show>
            <div class="flex min-w-0 flex-1 flex-col gap-2 bg-base-200/40 p-4">
                <p class="text-sm font-semibold">"Page content"</p>
                <p class="text-sm opacity-60">{move || body.get()}</p>
            </div>
            <Show when=move || panel_is_right.get()>{panel}</Show>
        </div>
    }
}

#[component]
pub fn FilterSidebarDemo() -> impl IntoView {
    // Independent state per example: the point of the page is comparing two
    // orientations in the SAME state, and a shared signal would make the
    // side-by-side collapsed pair impossible.
    let left_collapsed = RwSignal::new(false);
    let right_collapsed = RwSignal::new(false);
    let left_search = RwSignal::new(String::new());
    let right_search = RwSignal::new(String::new());
    let mirror_left_search = RwSignal::new(String::new());
    let mirror_right_search = RwSignal::new(String::new());
    let expanded_left_search = RwSignal::new(String::new());
    let expanded_right_search = RwSignal::new(String::new());

    let always_expanded = Signal::derive(|| false);
    let always_collapsed = Signal::derive(|| true);
    let noop = Callback::new(|()| {});

    // ldui-g66e: the search box's accessible name is reactive, so a locale
    // switch relabels it live -- without touching `left_search`'s value,
    // caret, or focus. Scoped to the one interactive left panel; the other
    // panels below each get their own fixed, distinct label so the browser
    // fixture can also prove multiple sidebars stay independently named.
    let filter_sidebar_locale_is_es = RwSignal::new(false);
    let interactive_left_search_label = Signal::derive(move || {
        if filter_sidebar_locale_is_es.get() {
            "Buscar filtros".to_string()
        } else {
            "Search filters".to_string()
        }
    });

    view! {
        <ContentLayout
            title="Filter Sidebar"
            description="A collapsible side panel that participates in page layout and animates its own width - 220px expanded, 44px collapsed, over a measured 250ms. Nothing unmounts on collapse, so scroll position and half-typed values survive it, and the collapsed rail keeps showing the active filter count. Docks against either edge via `side`."
        >
            <Section title="Interactive: one panel per edge, mirrored" col=true>
                <p class="text-sm opacity-60">
                    "Collapse each panel and watch the four mirrored details: the hairline border sits on the inner edge, the chevron points the way the panel would move, the toggle button stays beside the content it reveals, and the collapsed rail's vertical title reads bottom-to-top on the left and top-to-bottom on the right."
                </p>
                <p class="text-sm opacity-60">
                    "The left panel's search box carries a reactive, localizable accessible name (ldui-g66e) - independent of its placeholder and of the value typed into it. Toggle the locale and watch the name relabel live without touching what is typed."
                </p>
                <Button
                    attr:id="filter-sidebar-locale-toggle"
                    size=ButtonSize::Sm
                    style=ButtonStyle::Outline
                    on:click=move |_| filter_sidebar_locale_is_es.update(|es| *es = !*es)
                >
                    {move || {
                        if filter_sidebar_locale_is_es.get() {
                            "Switch search label to English"
                        } else {
                            "Switch search label to Spanish"
                        }
                    }}
                </Button>
                <div class="flex flex-wrap gap-4">
                    <div class="flex min-w-0 flex-1 flex-col gap-2">
                        <Button
                            size=ButtonSize::Sm
                            on:click=move |_| left_collapsed.update(|c| *c = !*c)
                        >
                            {move || {
                                if left_collapsed.get() { "Expand left" } else { "Collapse left" }
                            }}
                        </Button>
                        <Workspace
                            side=SidebarSide::Left
                            collapsed=left_collapsed
                            on_toggle=Callback::new(move |()| left_collapsed.update(|c| *c = !*c))
                            title="Filters"
                            active_count=3usize
                            body="A left-docked filter panel, the default and the only thing this component could do before ldui-vh6."
                            search=left_search
                            search_label=interactive_left_search_label
                            panel_id="fs-interactive-left"
                        />
                    </div>
                    <div class="flex min-w-0 flex-1 flex-col gap-2">
                        <Button
                            size=ButtonSize::Sm
                            on:click=move |_| right_collapsed.update(|c| *c = !*c)
                        >
                            {move || {
                                if right_collapsed.get() { "Expand right" } else { "Collapse right" }
                            }}
                        </Button>
                        <Workspace
                            side=SidebarSide::Right
                            collapsed=right_collapsed
                            on_toggle=Callback::new(move |()| right_collapsed.update(|c| *c = !*c))
                            title="Assistant"
                            active_count=3usize
                            body="A right-docked panel - 4iiz-Office's Client Coordinator Assistant, the request behind the side prop."
                            search=right_search
                            search_label="Search the assistant"
                            panel_id="fs-interactive-right"
                        />
                    </div>
                </div>
            </Section>

            <Section title="Both expanded, side by side" col=true>
                <p class="text-sm opacity-60">
                    "Pinned open so the header rows can be compared directly: title on the outer edge, toggle on the inner one, hairline on the inner one."
                </p>
                <div class="flex flex-wrap gap-4">
                    <div class="min-w-0 flex-1">
                        <Workspace
                            side=SidebarSide::Left
                            collapsed=always_expanded
                            on_toggle=noop
                            title="Filters"
                            active_count=2usize
                            body="Left, pinned expanded."
                            search=expanded_left_search
                            search_label="Search filters (expanded)"
                            panel_id="fs-expanded-left"
                        />
                    </div>
                    <div class="min-w-0 flex-1">
                        <Workspace
                            side=SidebarSide::Right
                            collapsed=always_expanded
                            on_toggle=noop
                            title="Assistant"
                            active_count=2usize
                            body="Right, pinned expanded."
                            search=expanded_right_search
                            search_label="Search the assistant (expanded)"
                            panel_id="fs-expanded-right"
                        />
                    </div>
                </div>
            </Section>

            <Section title="Both collapsed, side by side" col=true>
                <p class="text-sm opacity-60">
                    "The 44px rails. Each carries the active-filter count - the single detail that stops a filtered list being read as the whole list - above a vertical title that reads outward-to-inward on its own edge."
                </p>
                <div class="flex flex-wrap gap-4">
                    <div class="min-w-0 flex-1">
                        <Workspace
                            side=SidebarSide::Left
                            collapsed=always_collapsed
                            on_toggle=noop
                            title="Filters"
                            active_count=4usize
                            body="Left, pinned collapsed."
                            search=mirror_left_search
                            search_label="Search filters (collapsed)"
                            panel_id="fs-collapsed-left"
                        />
                    </div>
                    <div class="min-w-0 flex-1">
                        <Workspace
                            side=SidebarSide::Right
                            collapsed=always_collapsed
                            on_toggle=noop
                            title="Assistant"
                            active_count=4usize
                            body="Right, pinned collapsed."
                            search=mirror_right_search
                            search_label="Search the assistant (collapsed)"
                            panel_id="fs-collapsed-right"
                        />
                    </div>
                </div>
            </Section>

            <Section title="No active filters" col=true>
                <p class="text-sm opacity-60">
                    "active_count=0 hides the badge entirely on both sides; the vertical title still says what the rail is."
                </p>
                <div class="flex flex-wrap gap-4">
                    <div class="min-w-0 flex-1">
                        <Workspace
                            side=SidebarSide::Left
                            collapsed=always_collapsed
                            on_toggle=noop
                            title="Filters"
                            active_count=0usize
                            body="Left, nothing applied."
                            search=RwSignal::new(String::new())
                            search_label="Search filters (no active filters)"
                            panel_id="fs-empty-left"
                        />
                    </div>
                    <div class="min-w-0 flex-1">
                        <Workspace
                            side=SidebarSide::Right
                            collapsed=always_collapsed
                            on_toggle=noop
                            title="Assistant"
                            active_count=0usize
                            body="Right, nothing applied."
                            search=RwSignal::new(String::new())
                            search_label="Search the assistant (no active filters)"
                            panel_id="fs-empty-right"
                        />
                    </div>
                </div>
            </Section>
        </ContentLayout>
    }
}
