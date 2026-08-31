//! Admin workbench composition reference (ldui-ynmd.3, ldui-0qro).
//!
//! One deliberately small demo/reference state proving the intended
//! opinionated workbench hierarchy: [`AppShellTopBar`] with brand/search/
//! presence/settings/language/account/sign-out slots, a BORDERLESS
//! [`PageHeader`] (no back slot) with a greeting/subtitle and seven
//! [`PageQuickActions`], [`KpiStrip`] with eight varied cards, [`EntityTable`]
//! with typed text/select filters in its own aligned filter row (no external
//! filter bar), a right-docked [`FilterSidebar`] with assistant content and
//! controlled collapse, and a blue [`Fab`] Help button anchored bottom-right
//! (ldui-0qro).
//!
//! Every part here is an EXISTING component, reused exactly as published --
//! this file composes them, it does not extend or wrap any of them into a
//! new abstraction. Data and callbacks are entirely synthetic: no network or
//! API calls, no domain calculations, no persistence (the table's preference
//! ownership is intentionally left `Uncontrolled { Disabled }`, so nothing
//! is written to `localStorage`), and no production route -- this lives only
//! at the demo's own `/components/admin_workbench` showcase path.
//!
//! The 4iiz-Office regression this reference exists to prevent came from
//! composing these same valid primitives into the WRONG hierarchy (an
//! external `FilterBar` duplicating the table's own filter row, a joined
//! `Stats` strip standing in for independent KPI cards, a bordered
//! `PageHeader` competing with the shell's own top bar). This fixture is the
//! answer key, not a starting point for a new page generator -- if a real
//! composition needs something this fixture cannot express, file a follow-up
//! bead rather than growing this file.

use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::patterns::{
    KpiItem, KpiStatus, KpiStrip, KpiTrend, PageHeader, PageHeaderDivider, PageQuickActionContent,
    PageQuickActions,
};
use std::rc::Rc;

/// One synthetic workbench row. Display text only -- no domain calculation.
#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkbenchRow {
    id: String,
    client: String,
    status: String,
    matter_type: String,
    received: String,
}

const WORKBENCH_STATUSES: [&str; 3] = ["Urgent", "Ready", "Pending"];
const WORKBENCH_MATTER_TYPES: [&str; 2] = ["Family", "Civil"];

/// Deterministic synthetic dataset -- no fetch, no async, no seed drift.
fn workbench_rows() -> Vec<WorkbenchRow> {
    (0..22)
        .map(|index| WorkbenchRow {
            id: format!("wb-{index:03}"),
            client: format!("Client {:03}", index + 1),
            status: WORKBENCH_STATUSES[index % WORKBENCH_STATUSES.len()].to_owned(),
            matter_type: WORKBENCH_MATTER_TYPES[index % WORKBENCH_MATTER_TYPES.len()].to_owned(),
            received: format!("2026-08-{:02}", (index % 27) + 1),
        })
        .collect()
}

/// Four plain text columns -- the table's own presentation, nothing rich.
fn workbench_columns() -> Vec<EntityColumn<WorkbenchRow>> {
    vec![
        EntityColumn::text("client", "Client", |row: &WorkbenchRow| row.client.clone())
            .required()
            .with_min_width(200),
        EntityColumn::text("status", "Status", |row: &WorkbenchRow| row.status.clone())
            .with_min_width(110),
        EntityColumn::text("matter_type", "Matter type", |row: &WorkbenchRow| {
            row.matter_type.clone()
        })
        .with_min_width(130),
        EntityColumn::text("received", "Received", |row: &WorkbenchRow| {
            row.received.clone()
        })
        .with_min_width(120),
    ]
}

/// The table's own typed text/select filters, aligned beneath their
/// columns -- deliberately built INLINE at the `EntityTable` call site
/// (never as a pre-declared `let` captured into an outer closure): every
/// `EntityColumnFilter` renderer is `Rc`-based and therefore `!Send`, and
/// `AppShellContent`'s `children` slot -- like every plain Leptos
/// `Children` slot -- requires its generated closure to be `Send`. A value
/// constructed fresh inside that closure's body (a function call at the
/// attribute position, exactly like [`workbench_columns`] and `row_key`
/// below) never joins its captured environment, so it never affects the
/// closure's `Send`-ness; a `let` bound above `view! { .. }` and then used
/// inside it would.
fn workbench_column_filters(
    search: RwSignal<String>,
    status: RwSignal<String>,
) -> Vec<EntityColumnFilter> {
    vec![
        EntityColumnFilter::text(
            "client",
            "admin-workbench-client-filter",
            "Client",
            search,
            "Filter by client",
            Callback::new(move |next| search.set(next)),
        ),
        EntityColumnFilter::select(
            "status",
            "admin-workbench-status-filter",
            "Status",
            status,
            "All statuses",
            WORKBENCH_STATUSES
                .into_iter()
                .map(|status_value| EntityColumnFilterOption::new(status_value, status_value))
                .collect::<Vec<_>>(),
            Callback::new(move |next| status.set(next)),
        ),
    ]
}

/// Eight varied KPI cards -- available/unavailable, every [`KpiStatus`], with
/// and without a trend, matching the bead's "eight varied cards" wording.
fn workbench_kpis() -> Vec<KpiItem> {
    vec![
        KpiItem::new("open-matters", "Open matters", "128")
            .trend(KpiTrend::new(4.0, StatDeltaTrend::Positive).label("this week")),
        KpiItem::new("overdue-tasks", "Overdue tasks", "6")
            .status(KpiStatus::Warning)
            .help("Tasks past their due date, across every assignee."),
        KpiItem::new("new-leads", "New leads", "23")
            .status(KpiStatus::Info)
            .description("Last 7 days"),
        KpiItem::new("revenue-booked", "Revenue booked", "$18,400")
            .status(KpiStatus::Success)
            .trend(KpiTrend::new(12.5, StatDeltaTrend::Positive).label("vs last month")),
        KpiItem::new("sla-breaches", "SLA breaches", "2")
            .status(KpiStatus::Error)
            .trend(KpiTrend::new(1.0, StatDeltaTrend::Negative)),
        KpiItem::new("avg-response", "Avg. response time", "3h 12m"),
        KpiItem::new("client-satisfaction", "Client satisfaction", "94%")
            .status(KpiStatus::Success),
        KpiItem::new("last-sync", "Last sync", "").unavailable(),
    ]
}

/// The seven icon quick actions inside the borderless [`PageHeader`]'s
/// `actions` slot. Every action is a plain synthetic callback -- no route,
/// no HTTP form, no domain logic -- except "Reports", an in-page anchor
/// jump, and "Assistant", which ties the header directly to the right
/// [`FilterSidebar`]'s controlled collapse to demonstrate real composition
/// rather than a decorative row.
#[component]
fn WorkbenchQuickActions(
    action_count: RwSignal<usize>,
    assistant_collapsed: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <PageQuickActions label="Workbench actions">
            <Button
                style=ButtonStyle::Outline
                size=ButtonSize::Sm
                color=ButtonColor::Primary
                on_click=Callback::new(move |_| action_count.update(|count| *count += 1))
            >
                <PageQuickActionContent icon="plus" label="New matter" />
            </Button>
            <Button
                style=ButtonStyle::Outline
                size=ButtonSize::Sm
                on_click=Callback::new(move |_| action_count.update(|count| *count += 1))
            >
                <PageQuickActionContent icon="search" label="Search" />
            </Button>
            <Button
                style=ButtonStyle::Outline
                size=ButtonSize::Sm
                on_click=Callback::new(move |_| action_count.update(|count| *count += 1))
            >
                <PageQuickActionContent icon="filter" label="Filter" />
            </Button>
            <Button
                style=ButtonStyle::Outline
                size=ButtonSize::Sm
                on_click=Callback::new(move |_| action_count.update(|count| *count += 1))
            >
                <PageQuickActionContent icon="refresh" label="Refresh" />
            </Button>
            <Button
                style=ButtonStyle::Outline
                size=ButtonSize::Sm
                on_click=Callback::new(move |_| action_count.update(|count| *count += 1))
            >
                <PageQuickActionContent icon="upload" label="Export" />
            </Button>
            <LinkButton href="#admin-workbench-table" style=ButtonStyle::Outline size=ButtonSize::Sm>
                <PageQuickActionContent icon="file-text" label="Reports" />
            </LinkButton>
            <Button
                style=ButtonStyle::Outline
                size=ButtonSize::Sm
                color=ButtonColor::Secondary
                attr:aria-expanded=move || (!assistant_collapsed.get()).to_string()
                on_click=Callback::new(move |_| {
                    assistant_collapsed.update(|collapsed| *collapsed = !*collapsed);
                })
            >
                <PageQuickActionContent icon="message-square" label="Assistant" />
            </Button>
        </PageQuickActions>
    }
}

/// `AppShellTopBar`'s three slots hosting all seven named parts from the
/// bead: brand (start), search (center), presence/settings/language/
/// account/sign-out (end).
#[component]
fn WorkbenchTopBar(search_query: RwSignal<String>) -> impl IntoView {
    view! {
        <AppShellTopBar
            label="Workbench application controls"
            attr:data-testid="admin-workbench-topbar"
            start=Box::new(|| {
                view! {
                    <span class="ld-text-title font-semibold whitespace-nowrap">"Workbench"</span>
                }
                    .into_any()
            })
            center=Box::new(move || {
                view! {
                    <label class="flex min-w-0 w-full items-center gap-2">
                        <span class="sr-only">"Search matters, clients, and tasks"</span>
                        <Input
                            input_type=InputType::Search
                            size=InputSize::Sm
                            class="w-full"
                            placeholder="Search matters, clients, and tasks"
                            value=Signal::derive(move || search_query.get())
                            on_input=Callback::new(move |value| search_query.set(value))
                        />
                    </label>
                }
                    .into_any()
            })
            end=Box::new(|| {
                view! {
                    <span
                        class="flex items-center gap-1.5 ld-text-caption text-base-content/75"
                        data-testid="admin-workbench-presence"
                    >
                        <span class="status status-success" aria-hidden="true"></span>
                        "3 online"
                    </span>
                    <Button class="btn-ghost btn-sm btn-square" attr:aria-label="Settings">
                        <Icon name="settings" size=IconSize::Small />
                    </Button>
                    <label class="flex items-center gap-1">
                        <span class="sr-only">"Language"</span>
                        <select class="select select-bordered select-sm" aria-label="Language">
                            <option value="en">"EN"</option>
                            <option value="es">"ES"</option>
                        </select>
                    </label>
                    <Button class="btn-ghost btn-sm">
                        <Icon name="user" size=IconSize::Small />
                        <span class="ld-text-caption">"A. Rivera"</span>
                    </Button>
                    <Button class="btn-ghost btn-sm" attr:aria-label="Sign out">
                        <Icon name="log-out" size=IconSize::Small />
                        <span class="sr-only sm:not-sr-only">"Sign out"</span>
                    </Button>
                }
                    .into_any()
            })
        />
    }
}

#[component]
pub fn AdminWorkbenchDemo() -> impl IntoView {
    let topbar_search = RwSignal::new(String::new());
    let action_count = RwSignal::new(0_usize);
    let table_search = RwSignal::new(String::new());
    let table_status = RwSignal::new(String::new());
    let assistant_collapsed = RwSignal::new(false);
    let assistant_unread = RwSignal::new(2_usize);
    let assistant_draft = RwSignal::new(String::new());
    let assistant_messages = RwSignal::new(vec![
        "2 SLA breaches need review before end of day.".to_owned(),
        "3 new leads are unassigned.".to_owned(),
    ]);
    // Synthetic only, exactly like `action_count` above -- no Office-specific
    // Help route, copy, or handler. Activation stays caller-owned via this
    // callback (ldui-0qro).
    let help_requests = RwSignal::new(0_usize);
    let open_help = Callback::new(move |_| {
        help_requests.update(|count| *count += 1);
    });

    let rows = workbench_rows();
    let filtered_rows = Signal::derive_local(move || {
        let query = table_search.get().trim().to_lowercase();
        let status = table_status.get();
        Rc::new(
            rows.iter()
                .filter(|row| {
                    (query.is_empty() || row.client.to_lowercase().contains(&query))
                        && (status.is_empty() || row.status == status)
                })
                .cloned()
                .collect::<Vec<_>>(),
        )
    });

    let ask_assistant = Callback::new(move |_| {
        let draft = assistant_draft.get_untracked();
        if draft.trim().is_empty() {
            return;
        }
        assistant_messages.update(|messages| messages.push(draft.clone()));
        assistant_draft.set(String::new());
        assistant_unread.update(|unread| *unread += 1);
    });

    view! {
        <ContentLayout
            title="Admin Workbench (Composition Reference)"
            description="A deliberately small reference proving the intended opinionated hierarchy: one standard AppShellTopBar, one borderless base-page PageHeader with icon quick actions, independent KpiStrip cards, an EntityTable with its own aligned filter row (no external filter bar), and a right-docked assistant whose collapse returns width to the table. Resize the browser to review wide-desktop and compact-mobile wrapping; the assistant toggle exercises expanded/collapsed."
        >
            <Section title="Full composition" col=true>
                <p class="text-sm text-base-content/70">
                    "Synthetic data and callbacks only -- no network calls, no domain calculations, "
                    "no persistence, no production route. Reused components exactly as published; "
                    "this page composes them, it does not extend them."
                </p>
                <div
                    class="h-[820px] w-full overflow-hidden rounded-lg border border-base-300"
                    data-testid="admin-workbench"
                >
                    <AppShell top_bar=Box::new(move || {
                        view! { <WorkbenchTopBar search_query=topbar_search /> }.into_any()
                    })>
                        <AppShellContent class="p-0">
                            <div class="flex h-full min-h-0 w-full">
                                // `contain: layout` scopes the Help FAB's `.fab`
                                // `position: fixed` to THIS column (CSS
                                // Containment L1 sec.2: layout containment makes
                                // an element the containing block for its fixed-
                                // and absolutely-positioned descendants). That is
                                // structural, not incidental: the column is a flex
                                // sibling of the right-docked assistant panel, so
                                // a FAB anchored to the column's own bottom-right
                                // corner can never render past the column's right
                                // edge into the assistant panel, collapsed or
                                // expanded -- no per-state offset math needed. The
                                // extra `pb-24` (96px, on the canonical spacing
                                // scale) reserves clearance below the table's own
                                // pagination footer so the FAB -- pinned 1rem from
                                // the column's visible bottom edge -- never sits
                                // on top of it even when the column's content is
                                // tall enough to scroll.
                                <div
                                    class="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto pt-6 px-6 pb-24"
                                    style="contain: layout"
                                >
                                    <PageHeader
                                        title="Good afternoon, Alex"
                                        subtitle="Everything on your desk today, across every active matter."
                                        divider=PageHeaderDivider::Hidden
                                        actions=Box::new(move || {
                                            view! {
                                                <WorkbenchQuickActions
                                                    action_count=action_count
                                                    assistant_collapsed=assistant_collapsed
                                                />
                                            }
                                                .into_any()
                                        })
                                        attr:data-testid="admin-workbench-header"
                                    />

                                    <KpiStrip
                                        items=Signal::derive(workbench_kpis)
                                        attr:data-testid="admin-workbench-kpis"
                                    />

                                    <div id="admin-workbench-table" data-testid="admin-workbench-table">
                                        <EntityTable
                                            data=filtered_rows
                                            columns=workbench_columns()
                                            column_filters=workbench_column_filters(table_search, table_status)
                                            row_key=Rc::new(|row: &WorkbenchRow| row.id.clone())
                                            dataset_identity="admin-workbench"
                                        />
                                    </div>

                                    // Reuses the existing `Fab` primitive exactly
                                    // as published (ldui-0qro) -- no second
                                    // floating-button component. A single child,
                                    // not a speed dial: daisyUI's `.fab` fans out
                                    // siblings after the first only when a
                                    // `FabClose`/`FabMainAction` and further
                                    // buttons are present, so one plain `Button`
                                    // renders as one plain floating button, with
                                    // no double-nested interactive-element
                                    // wrapper. `Button` already carries
                                    // `ld-focus-ring` for a visible focus
                                    // treatment and `aria-label` is its only
                                    // accessible name (icon-only, matching the
                                    // top bar's Settings/Sign out buttons above).
                                    <Fab attr:data-testid="admin-workbench-help-fab">
                                        <Button
                                            color=ButtonColor::Info
                                            shape=ButtonShape::Circle
                                            size=ButtonSize::Lg
                                            attr:aria-label="Help"
                                            attr:data-testid="admin-workbench-help-fab-trigger"
                                            on_click=open_help
                                        >
                                            <Icon name="help-circle" size=IconSize::Medium />
                                        </Button>
                                    </Fab>
                                </div>

                                <FilterSidebar
                                    side=SidebarSide::Right
                                    collapsed=assistant_collapsed
                                    on_toggle=Callback::new(move |()| {
                                        assistant_collapsed.update(|collapsed| *collapsed = !*collapsed);
                                    })
                                    active_count=Signal::derive(move || assistant_unread.get())
                                    title="Assistant"
                                    toggle_label="Toggle the assistant panel"
                                    expanded_width="w-[280px] min-w-[280px]"
                                    attr:data-testid="admin-workbench-assistant"
                                >
                                    <div class="flex flex-col gap-3" data-testid="admin-workbench-assistant-body">
                                        <p class="ld-text-caption text-base-content/75">
                                            "Suggestions based on what's on your desk today."
                                        </p>
                                        <ul class="flex flex-col gap-2">
                                            <For
                                                each=move || {
                                                    assistant_messages
                                                        .get()
                                                        .into_iter()
                                                        .enumerate()
                                                        .collect::<Vec<_>>()
                                                }
                                                key=|(index, _)| *index
                                                children=move |(_, message)| {
                                                    view! {
                                                        <li class="rounded-box border border-base-300 bg-base-200/60 p-2 ld-text-caption">
                                                            {message}
                                                        </li>
                                                    }
                                                }
                                            />
                                        </ul>
                                        <label class="flex flex-col gap-1">
                                            <span class="sr-only">"Ask the assistant"</span>
                                            <Input
                                                size=InputSize::Sm
                                                placeholder="Ask the assistant"
                                                value=assistant_draft
                                                on_input=Callback::new(move |value| assistant_draft.set(value))
                                            />
                                        </label>
                                        <Button
                                            size=ButtonSize::Sm
                                            color=ButtonColor::Primary
                                            attr:data-testid="admin-workbench-assistant-ask"
                                            on_click=ask_assistant
                                        >
                                            "Ask"
                                        </Button>
                                    </div>
                                </FilterSidebar>
                            </div>
                        </AppShellContent>
                    </AppShell>
                </div>

                <div class="flex flex-wrap gap-4 ld-text-caption text-base-content/60" aria-live="polite">
                    <span>
                        "Quick actions used: "
                        <strong data-testid="admin-workbench-action-count">{move || action_count.get()}</strong>
                    </span>
                    <span>
                        "Assistant messages: "
                        <strong data-testid="admin-workbench-assistant-count">
                            {move || assistant_messages.get().len()}
                        </strong>
                    </span>
                    <span>
                        "Help requests: "
                        <strong data-testid="admin-workbench-help-count">
                            {move || help_requests.get()}
                        </strong>
                    </span>
                </div>
            </Section>
        </ContentLayout>
    }
}
