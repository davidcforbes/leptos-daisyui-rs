use super::day::DayZoom;
use super::style::{
    BLOCK, CHIP, DATE_HEADING, DAY_BOX, DAY_BOX_LAYOUT, DAY_CLOSED, DAY_HOLIDAY, DAY_OUTSIDE,
    DAY_TODAY, DAY_WEEKEND, GRID_ONLY, OUTSIDE_LAYOUT, ROOT, RosterLayout, RosterZoom, TABLE,
    TABLE_BASE, WEEK_HEAD, WEEKDAY_HEAD,
};
use super::texts::{MonthRosterTexts, fill};
use super::types::{
    RosterDay, RosterEntry, VISIBLE_PER_DAY, active_day, add_months, first_of_month, iso_date,
    iso_week, roving_target, sorted_entries, step, visible_range,
};
use super::week::WeekZoom;
use crate::components::gantt::utils::focus_element_by_id;
use crate::components::{Weekday, WeekdaySet};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Per-instance sequence so each roster on a page mints collision-free DOM
/// ids for its date boxes, toggles and headings.
static MONTH_ROSTER_SEQ: AtomicU64 = AtomicU64::new(0);

fn default_weekend() -> WeekdaySet {
    WeekdaySet::from_days([Weekday::Saturday, Weekday::Sunday])
}

/// Everything the roster's parts share. Every field is `Copy` (signals,
/// memos and callbacks are arena handles), so each part takes the whole
/// context by value.
#[derive(Clone, Copy)]
pub(super) struct Ctx {
    pub instance: u64,
    pub days: Memo<HashMap<i64, RosterDay>>,
    pub anchor: Signal<i64>,
    pub today: Signal<i64>,
    pub zoom: Signal<RosterZoom>,
    pub now_min: Signal<Option<u16>>,
    pub week_start: Signal<Weekday>,
    pub weekend: Signal<WeekdaySet>,
    pub texts: Signal<MonthRosterTexts>,
    pub layout: Signal<Option<RosterLayout>>,
    pub on_navigate: Option<Callback<(RosterZoom, i64)>>,
    pub on_entry_activate: Option<Callback<String>>,
    /// The month grid's remembered tab stop: `(day, month_first)`, where
    /// `month_first` is the first day of the month shown when it was set.
    pub focus: RwSignal<Option<(i64, i64)>>,
    /// The month grid's active (tab-stop) day.
    pub active: Memo<i64>,
    /// The grid is a keyboard widget (roving focus, `role="grid"`).
    pub interactive: bool,
}

impl Ctx {
    pub fn navigate(&self, zoom: RosterZoom, day: i64) {
        if let Some(cb) = self.on_navigate {
            cb.run((zoom, day));
        }
    }

    pub fn navigable(&self) -> bool {
        self.on_navigate.is_some()
    }

    pub fn cell_id(&self, day: i64) -> String {
        format!("month-roster-{}-day-{day}", self.instance)
    }

    pub fn id(&self, part: &str) -> String {
        format!("month-roster-{}-{part}", self.instance)
    }

    /// The host's data for `day`, `None` when the day is unknown.
    pub fn day_data(&self, day: i64) -> Option<RosterDay> {
        self.days.with(|days| days.get(&day).cloned())
    }

    pub fn layout_classes(&self, classes: super::style::LayoutClasses) -> &'static str {
        classes.pick(self.layout.get())
    }
}

/// # Month Roster
///
/// An opinionated staff calendar (ldui-5lmt, a port of the 4iiz-kit
/// `MonthRoster` so 4iiz-Office and 4iiz-People share one look): who is
/// scheduled to work each day, and who is out on vacation, out of the
/// office or sick, at three zoom levels of one data set.
///
/// - **Month**: the kit's grid of whole weeks. Each date box shows its marks
///   ("Holiday: X", "Closed: reason"), its staffing ("3 of 4 needed") and its
///   people as chips, 4 visible and "+N" for the rest.
/// - **Week**: seven day columns listing EVERY entry, working first, then
///   "Out (N)".
/// - **Day**: a shift coverage timeline (one bar per working person on an
///   hour axis) and "Out today", grouped by kind.
///
/// The roster is **controlled and data-agnostic**, like
/// [`WeekView`](crate::components::WeekView): it never fetches, never reads
/// a clock and never mutates `days`. Every prev / next / today / zoom /
/// drill request is a proposal through `on_navigate(zoom, epoch_day)`; the
/// host answers by setting `zoom` and `anchor_day`.
/// [`roster_visible_range`](crate::patterns::roster_visible_range) names
/// the days to load. Without
/// `on_navigate` the roster is display-only: no controls, no roving focus.
///
/// A day ABSENT from `days` reads "No schedule published"; a present day
/// with no entries reads "No one scheduled". Every chip says its state in
/// words, so colour is never the only channel.
///
/// The layout follows the roster's OWN width (a container query), not the
/// window: below 42rem the month is an agenda, the week stacks and the day
/// axis compresses. The root is an `@container`, so its parent must give it
/// a width (a block, or a flex item with a width). `layout` forces one.
///
/// ### Keyboard (month grid)
///
/// | Key | Effect |
/// |---|---|
/// | `Tab` | Enter the grid at the active day; the next Tab visits that day's own controls, then leaves |
/// | Arrows | One day / one week, within the shown weeks |
/// | `Home` / `End` | The ends of the week row |
/// | `PageUp` / `PageDown` | The previous / next month, same day of the month |
/// | `Enter` | Zoom to that day |
/// | `Escape` | From a chip back to its day; in Week back to Month; in Day back to Week |
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::ShiftState;
/// use leptos_daisyui_rs::patterns::{AbsenceKind, MonthRoster, RosterDay, RosterEntry, RosterZoom};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let zoom = RwSignal::new(RosterZoom::Month);
///     let anchor = RwSignal::new(20_740_i64); // 2026-10-14
///     let days = Signal::derive(|| {
///         vec![RosterDay::new(20_740).staffing(1, 2).entries([
///             RosterEntry::working("a1", "Ana Lopez", ShiftState::Full).times(8 * 60, 16 * 60 + 30),
///             RosterEntry::absent("l1", "Luis Garcia", AbsenceKind::Vacation).pending(),
///         ])]
///     });
///     view! {
///         <MonthRoster
///             days=days
///             anchor_day=anchor
///             today=Signal::stored(20_740_i64)
///             zoom=zoom
///             on_navigate=move |(z, day): (RosterZoom, i64)| {
///                 zoom.set(z);
///                 anchor.set(day);
///             }
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("@container flex w-full flex-col gap-3 items-center justify-between flex-wrap gap-1 gap-2 gap-x-2 gap-y-1 text-lg text-sm text-xs font-semibold");
/// @source inline("table table-fixed border-collapse border border-table-grid bg-table-header text-table-header-content text-center px-1 py-1 p-1 p-2 align-top w-10 h-28 mt-1");
/// @source inline("bg-base-200/40 text-base-content/75 bg-base-200/60 bg-accent/10 bg-base-200/30 outline-2 -outline-offset-2 outline-primary sr-only not-sr-only select-none");
/// @source inline("rounded-sm border-l-4 leading-tight truncate break-words translate-x-0 -translate-x-1/2 -translate-x-full");
/// @source inline("bg-success/15 bg-info/15 bg-warning/15 bg-secondary/15 bg-error/15 bg-accent/15 border-success border-info border-warning border-secondary border-error border-accent border-base-300 border-solid border-dashed");
/// @source inline("peer peer-checked:not-sr-only peer-checked:flex focus-within:not-sr-only focus-within:flex peer-focus-visible:ring-2 peer-focus-visible:ring-primary");
/// @source inline("@max-2xl:block @max-2xl:hidden @max-2xl:h-auto @max-2xl:border-x-0 @max-2xl:not-sr-only @max-2xl:grid-cols-1 @max-2xl:grid-cols-[6rem_1fr] grid-cols-7 grid-cols-1 grid-cols-[10rem_1fr] grid-cols-[6rem_1fr]");
/// @source inline("print:hidden print:h-auto print:not-sr-only print:flex btn btn-ghost btn-sm btn-xs btn-square btn-active join join-item");
/// @source inline("sticky top-0 z-10 bg-base-100 rounded-box border-base-300 relative absolute inset-y-0 h-6 border-l-2 border-error pointer-events-none");
/// @source inline("block hidden h-auto border-x-0 grid gap-4 gap-x-2 items-baseline tabular-nums hover:underline cursor-pointer min-w-0 text-left no-underline text-base-content inline-block size-4");
/// @source inline("overflow-hidden whitespace-nowrap bottom-0 bottom-2 left-0 inset-0 inset-y-1 h-2 h-7 w-0 w-fit border-l leading-7 px-1 sr-only");
/// ```
///
/// ## Node References
/// - `node_ref` - References the root `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn MonthRoster(
    /// The host's days. A day not in this list is unknown ("No schedule
    /// published"), not empty.
    #[prop(into)]
    days: Signal<Vec<RosterDay>>,

    /// The day the view is anchored on (epoch day): its month, its week, or
    /// the day itself, by `zoom`.
    #[prop(into)]
    anchor_day: Signal<i64>,

    /// Today (epoch day), from the host's clock and time zone. Ringed and
    /// marked "Today".
    #[prop(into)]
    today: Signal<i64>,

    /// The zoom level. The host owns it; see `on_navigate`.
    #[prop(optional, into)]
    zoom: Signal<RosterZoom>,

    /// The current time of day (minutes from midnight), for the Day view's
    /// now line. Shown only when the Day view shows today.
    #[prop(optional, into)]
    now_min: Signal<Option<u16>>,

    /// The first day of the week. Defaults to Monday; any weekday works.
    #[prop(optional, into, default = Signal::stored(Weekday::Monday))]
    week_start: Signal<Weekday>,

    /// The weekdays shaded as the weekend. Defaults to Saturday and Sunday.
    #[prop(optional, into, default = Signal::stored(default_weekend()))]
    weekend: Signal<WeekdaySet>,

    /// Every word the roster writes. Defaults to English;
    /// `MonthRosterTexts::es()` is Spanish.
    #[prop(optional, into)]
    texts: Signal<MonthRosterTexts>,

    /// Force the grid or the agenda. `None` (the default) follows the
    /// roster's own width.
    #[prop(optional, into)]
    layout: Signal<Option<RosterLayout>>,

    /// The month grid's accessible name. Defaults to the title.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Every navigation proposal: prev / next / Today / the zoom switch /
    /// a week button / a day number / Enter / PageUp / PageDown / Escape.
    /// The host answers by setting `zoom` and `anchor_day`.
    #[prop(optional, into)]
    on_navigate: Option<Callback<(RosterZoom, i64)>>,

    /// A chip was activated; receives the entry's `id`. With it every chip
    /// is a button.
    #[prop(optional, into)]
    on_entry_activate: Option<Callback<String>>,

    /// The host's own controls, at the end of the header (an office
    /// select).
    #[prop(optional)]
    actions: Option<Children>,

    /// Additional CSS classes for the root.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the root `div`.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let instance = MONTH_ROSTER_SEQ.fetch_add(1, Ordering::Relaxed);
    let day_map = Memo::new(move |_| {
        days.with(|days| {
            days.iter()
                .map(|day| (day.day, day.clone()))
                .collect::<HashMap<i64, RosterDay>>()
        })
    });
    let focus = RwSignal::new(None::<(i64, i64)>);
    let active = Memo::new(move |_| {
        let anchor = anchor_day.get();
        let (first, last) = visible_range(RosterZoom::Month, anchor, week_start.get());
        active_day(focus.get(), anchor, today.get()).clamp(first, last)
    });
    let ctx = Ctx {
        instance,
        days: day_map,
        anchor: anchor_day,
        today,
        zoom,
        now_min,
        week_start,
        weekend,
        texts,
        layout,
        on_navigate,
        on_entry_activate,
        focus,
        active,
        interactive: on_navigate.is_some() || on_entry_activate.is_some(),
    };

    let title_id = ctx.id("title");
    let title = move || texts.with(|t| t.title(zoom.get(), anchor_day.get(), week_start.get()));
    let grid_label = Signal::derive(move || label.get().unwrap_or_else(title));
    let layout_slug = move || layout.get().map_or("auto", RosterLayout::as_str);

    view! {
        <div
            node_ref=node_ref
            data-month-roster=""
            data-zoom=move || zoom.get().as_str()
            data-layout=layout_slug
            class=move || merge_classes!(ROOT, class)
        >
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h2 id=title_id data-month-roster-title="" class="text-lg font-semibold">
                    {title}
                </h2>
                <div class="flex flex-wrap items-center gap-2">
                    {ctx.navigable().then(|| view! { <Navigation ctx=ctx /> })}
                    {actions.map(|actions| actions())}
                </div>
            </div>
            {move || match zoom.get() {
                RosterZoom::Month => view! { <MonthGrid ctx=ctx label=grid_label /> }.into_any(),
                RosterZoom::Week => view! { <WeekZoom ctx=ctx /> }.into_any(),
                RosterZoom::Day => view! { <DayZoom ctx=ctx /> }.into_any(),
            }}
        </div>
    }
}

/// Prev / Today / next and the Month | Week | Day switch.
#[component]
fn Navigation(ctx: Ctx) -> impl IntoView {
    let texts = ctx.texts;
    let zoom = ctx.zoom;
    let anchor = ctx.anchor;
    let go = move |delta: i64| {
        let zoom = zoom.get_untracked();
        ctx.navigate(zoom, step(zoom, anchor.get_untracked(), delta));
    };
    let previous_name = move || texts.with(|t| t.arrows(zoom.get()).0.to_owned());
    let next_name = move || texts.with(|t| t.arrows(zoom.get()).1.to_owned());
    // From the month grid, Week and Day zoom to the ACTIVE day, so a keyboard
    // user can arrow to a day and then choose its week.
    let zoom_target = move || {
        if zoom.get_untracked() == RosterZoom::Month {
            ctx.active.get_untracked()
        } else {
            anchor.get_untracked()
        }
    };
    let switch = RosterZoom::ALL
        .into_iter()
        .map(|level| {
            let pressed = move || zoom.get() == level;
            let action = match level {
                RosterZoom::Month => "zoom-month",
                RosterZoom::Week => "zoom-week",
                RosterZoom::Day => "zoom-day",
            };
            view! {
                <button
                    type="button"
                    class="btn ld-eased ld-pressable ld-focus-ring btn-sm join-item"
                    class:btn-active=pressed
                    data-month-roster-action=action
                    aria-pressed=move || if pressed() { "true" } else { "false" }
                    on:click=move |_| ctx.navigate(level, zoom_target())
                >
                    {move || texts.with(|t| t.zoom_word(level).to_owned())}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="flex items-center gap-1">
            <button
                type="button"
                class="btn ld-eased ld-pressable ld-focus-ring btn-ghost btn-sm btn-square print:hidden"
                data-month-roster-action="previous"
                aria-label=previous_name
                on:click=move |_| go(-1)
            >
                "\u{2039}"
            </button>
            <button
                type="button"
                class="btn ld-eased ld-pressable ld-focus-ring btn-ghost btn-sm print:hidden"
                data-month-roster-action="today"
                on:click=move |_| ctx.navigate(zoom.get_untracked(), ctx.today.get_untracked())
            >
                {move || texts.with(|t| t.today.clone())}
            </button>
            <button
                type="button"
                class="btn ld-eased ld-pressable ld-focus-ring btn-ghost btn-sm btn-square print:hidden"
                data-month-roster-action="next"
                aria-label=next_name
                on:click=move |_| go(1)
            >
                "\u{203a}"
            </button>
        </div>
        <div
            role="group"
            class="join print:hidden"
            aria-label=move || texts.with(|t| t.zoom_label.clone())
        >
            {switch}
        </div>
    }
}

/// The month grid: whole weeks of date boxes.
#[component]
fn MonthGrid(ctx: Ctx, label: Signal<String>) -> impl IntoView {
    let span = Memo::new(move |_| {
        visible_range(RosterZoom::Month, ctx.anchor.get(), ctx.week_start.get())
    });
    let row_starts = move || {
        let (first, last) = span.get();
        (0..=(last - first) / 7)
            .map(|row| first + row * 7)
            .collect::<Vec<i64>>()
    };
    // The heads follow the week start: column `i` is weekday `start + i`.
    let heads = move || {
        let start = Weekday::from_epoch_day(span.get().0).index_from_monday();
        (0..7).map(|i| (start + i) % 7).collect::<Vec<usize>>()
    };
    // "No one is scheduled in October 2026", only when the month is KNOWN
    // (some in-month day was supplied) and nobody is on any of its days.
    let month_empty = move || {
        let anchor = ctx.anchor.get();
        let month = first_of_month(anchor);
        ctx.days.with(|days| {
            let in_month: Vec<&RosterDay> = days
                .values()
                .filter(|d| first_of_month(d.day) == month)
                .collect();
            !in_month.is_empty() && in_month.iter().all(|d| d.entries.is_empty())
        })
    };
    let empty_text = move || {
        ctx.texts.with(|t| {
            let month = t.month_title_of(ctx.anchor.get());
            fill(&t.month_empty, &[("month", &month)])
        })
    };

    view! {
        <Show when=month_empty>
            <p data-month-empty="" class="text-sm text-base-content/75">
                {empty_text}
            </p>
        </Show>
        <table
            data-month-grid=""
            role=ctx.interactive.then_some("grid")
            aria-label=move || label.get()
            class=move || merge_classes!(TABLE_BASE, ctx.layout_classes(TABLE))
        >
            <thead class=move || ctx.layout_classes(GRID_ONLY)>
                <tr>
                    {ctx
                        .navigable()
                        .then(|| {
                            view! {
                                <th scope="col" class=merge_classes!(WEEK_HEAD, "print:hidden")>
                                    <span class="sr-only">
                                        {move || ctx.texts.with(|t| t.week_column.clone())}
                                    </span>
                                </th>
                            }
                        })}
                    <For
                        each=heads
                        key=|weekday| *weekday
                        children=move |weekday| {
                            view! {
                                <th scope="col" class=WEEKDAY_HEAD data-weekday=weekday>
                                    <span aria-hidden="true">
                                        {move || ctx.texts.with(|t| t.weekday_short[weekday].clone())}
                                    </span>
                                    <span class="sr-only select-none">
                                        {move || ctx.texts.with(|t| t.weekday_names[weekday].clone())}
                                    </span>
                                </th>
                            }
                        }
                    />
                </tr>
            </thead>
            <tbody class=move || ctx.layout_classes(BLOCK)>
                <For
                    each=row_starts
                    key=|start| *start
                    children=move |row_start| view! { <MonthRow ctx=ctx row_start=row_start span=span /> }
                />
            </tbody>
        </table>
    }
}

/// One week row: the week button, then seven date boxes.
#[component]
fn MonthRow(ctx: Ctx, row_start: i64, span: Memo<(i64, i64)>) -> impl IntoView {
    let week_label = move || {
        ctx.texts.with(|t| {
            if ctx.week_start.get() == Weekday::Monday {
                Some(fill(
                    &t.week_number,
                    &[("week", &iso_week(row_start).to_string())],
                ))
            } else {
                None
            }
        })
    };
    let week_name = move || {
        ctx.texts
            .with(|t| fill(&t.show_week, &[("date", &t.date_long_of(row_start))]))
    };
    view! {
        <tr data-month-week=iso_date(row_start) class=move || ctx.layout_classes(BLOCK)>
            {ctx
                .navigable()
                .then(|| {
                    view! {
                        <th
                            scope="row"
                            class=move || merge_classes!(WEEK_HEAD, ctx.layout_classes(GRID_ONLY), "print:hidden")
                        >
                            <button
                                type="button"
                                tabindex="-1"
                                data-month-roster-action="week"
                                data-week-start=iso_date(row_start)
                                class="btn ld-eased ld-pressable ld-focus-ring btn-ghost btn-xs px-1"
                                aria-label=move || match week_label() {
                                    Some(text) => format!("{text}, {}", week_name()),
                                    None => week_name(),
                                }
                                on:click=move |_| ctx.navigate(RosterZoom::Week, row_start)
                            >
                                {move || match week_label() {
                                    Some(text) => text.into_any(),
                                    None => view! { <ChevronIcon /> }.into_any(),
                                }}
                            </button>
                        </th>
                    }
                })}
            {(0..7)
                .map(|offset| {
                    view! { <MonthCell ctx=ctx day=row_start + offset span=span /> }
                })
                .collect_view()}
        </tr>
    }
}

/// A small chevron for the week button when the week is not ISO (no week
/// number to show).
#[component]
fn ChevronIcon() -> impl IntoView {
    view! {
        <svg
            aria-hidden="true"
            viewBox="0 0 16 16"
            class="size-4"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
        >
            <path d="M6 3l5 5-5 5" />
        </svg>
    }
}

/// One month date box.
#[component]
fn MonthCell(ctx: Ctx, day: i64, span: Memo<(i64, i64)>) -> impl IntoView {
    let data = Memo::new(move |_| ctx.day_data(day));
    let outside = Memo::new(move |_| first_of_month(ctx.anchor.get()) != first_of_month(day));
    let is_today = move || ctx.today.get() == day;
    let weekday = Weekday::from_epoch_day(day);
    let is_active = Memo::new(move |_| ctx.interactive && ctx.active.get() == day);
    let in_tab_order = Signal::derive(move || is_active.get());
    let sorted = Memo::new(move |_| {
        data.with(|d| {
            d.as_ref()
                .map(|d| sorted_entries(&d.entries))
                .unwrap_or_default()
        })
    });
    let visible =
        move || sorted.with(|s| s.iter().take(VISIBLE_PER_DAY).cloned().collect::<Vec<_>>());
    let extra =
        move || sorted.with(|s| s.iter().skip(VISIBLE_PER_DAY).cloned().collect::<Vec<_>>());
    let extra_count = move || sorted.with(|s| s.len().saturating_sub(VISIBLE_PER_DAY));

    let class = move || {
        let data = data.get();
        let mut classes = vec![DAY_BOX, ctx.layout_classes(DAY_BOX_LAYOUT)];
        if ctx.interactive {
            classes.push("ld-focus-ring");
        }
        if outside.get() {
            classes.push(DAY_OUTSIDE);
            classes.push(ctx.layout_classes(OUTSIDE_LAYOUT));
        } else if data.as_ref().is_some_and(|d| d.closed.is_some()) {
            classes.push(DAY_CLOSED);
        } else if data.as_ref().is_some_and(|d| d.holiday.is_some()) {
            classes.push(DAY_HOLIDAY);
        } else if ctx.weekend.get().contains(weekday) {
            classes.push(DAY_WEEKEND);
        }
        if is_today() {
            classes.push(DAY_TODAY);
        }
        classes.join(" ")
    };
    let name = move || {
        ctx.texts
            .with(|t| data.with(|d| t.day_name(day, is_today(), d.as_ref())))
    };
    let on_keydown = move |ev: web_sys::KeyboardEvent| cell_keydown(ctx, day, span, ev);
    let on_focusin = move |_: web_sys::FocusEvent| {
        if ctx.interactive {
            ctx.focus
                .set(Some((day, first_of_month(ctx.anchor.get_untracked()))));
        }
    };
    let toggle_id = ctx.id(&format!("more-{day}"));
    let toggle_for = toggle_id.clone();

    view! {
        <td
            id=ctx.cell_id(day)
            data-day=iso_date(day)
            role=ctx.interactive.then_some("gridcell")
            tabindex=move || ctx.interactive.then(|| if is_active.get() { 0 } else { -1 })
            aria-label=name
            data-outside=move || outside.get().then_some("")
            data-today=move || is_today().then_some("")
            data-weekend=move || ctx.weekend.get().contains(weekday).then_some("")
            data-holiday=move || data.with(|d| d.as_ref().is_some_and(|d| d.holiday.is_some())).then_some("")
            data-closed=move || data.with(|d| d.as_ref().is_some_and(|d| d.closed.is_some())).then_some("")
            data-unknown=move || data.with(Option::is_none).then_some("")
            data-empty=move || data.with(|d| d.as_ref().is_some_and(|d| d.entries.is_empty())).then_some("")
            class=class
            on:keydown=on_keydown
            on:focusin=on_focusin
        >
            <h3 class=move || merge_classes!("text-sm font-semibold", ctx.layout_classes(DATE_HEADING))>
                {move || ctx.texts.with(|t| t.date_long_of(day))}
            </h3>
            <DayHead ctx=ctx day=day data=data tabbable=false />
            <DayStatus ctx=ctx data=data />
            <ul data-chips="" class="mt-1 flex flex-col gap-1">
                <For
                    each=visible
                    key=|entry| entry.content_key()
                    children=move |entry| {
                        view! { <Chip ctx=ctx entry=entry in_tab_order=in_tab_order prefer_times=false /> }
                    }
                />
            </ul>
            <Show when=move || { extra_count() > 0 }>
                <input
                    type="checkbox"
                    id=toggle_id.clone()
                    data-overflow-toggle=""
                    class="peer sr-only"
                    tabindex=move || (!in_tab_order.get()).then_some(-1)
                />
                <label
                    for=toggle_for.clone()
                    data-overflow=""
                    class="mt-1 block w-fit cursor-pointer rounded-sm px-1 text-xs font-semibold hover:underline peer-focus-visible:ring-2 peer-focus-visible:ring-primary print:hidden"
                >
                    <span aria-hidden="true">
                        {move || ctx.texts.with(|t| t.more_visible(extra_count()))}
                    </span>
                    <span class="sr-only">
                        {move || ctx.texts.with(|t| t.more_spoken(extra_count()))}
                    </span>
                </label>
                <ul
                    data-overflow-list=""
                    class="sr-only mt-1 flex-col gap-1 peer-checked:not-sr-only peer-checked:flex focus-within:not-sr-only focus-within:flex print:not-sr-only print:flex"
                >
                    <For
                        each=extra
                        key=|entry| entry.content_key()
                        children=move |entry| {
                            view! { <Chip ctx=ctx entry=entry in_tab_order=in_tab_order prefer_times=false /> }
                        }
                    />
                </ul>
            </Show>
        </td>
    }
}

/// A date box's keys: the roving moves on the box itself, PageUp/PageDown
/// anywhere inside it, Escape from a chip back to the box.
fn cell_keydown(ctx: Ctx, day: i64, span: Memo<(i64, i64)>, ev: web_sys::KeyboardEvent) {
    if !ctx.interactive || ev.alt_key() || ev.ctrl_key() || ev.meta_key() {
        return;
    }
    let key = ev.key();
    let on_box = ev.target() == ev.current_target();
    match key.as_str() {
        "PageUp" | "PageDown" if ctx.navigable() => {
            ev.prevent_default();
            let target = add_months(day, if key == "PageUp" { -1 } else { 1 });
            ctx.focus.set(Some((target, first_of_month(target))));
            ctx.navigate(RosterZoom::Month, target);
            focus_after_render(ctx.cell_id(target));
        }
        "Escape" if !on_box => {
            ev.prevent_default();
            focus_element_by_id(&ctx.cell_id(day));
        }
        "Enter" if on_box && ctx.navigable() => {
            ev.prevent_default();
            ctx.navigate(RosterZoom::Day, day);
        }
        "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End" if on_box => {
            // Bare arrows are stopped even at an edge, or they scroll the
            // page out from under the grid.
            ev.prevent_default();
            let (first, last) = span.get_untracked();
            if let Some(target) = roving_target(&key, day, first, last) {
                ctx.focus
                    .set(Some((target, first_of_month(ctx.anchor.get_untracked()))));
                focus_element_by_id(&ctx.cell_id(target));
            }
        }
        _ => {}
    }
}

/// Focus `id` once the host's answer has rendered: Leptos applies DOM
/// updates in a microtask, so a 0 ms macrotask runs after them. The closure
/// touches no signal, so a late fire after unmount is a harmless lookup.
pub(super) fn focus_after_render(id: String) {
    set_timeout(move || focus_element_by_id(&id), Duration::from_millis(0));
}

/// A day's head: the day number (a button that zooms to the day), "Today",
/// the holiday / closed marks and the staffing line.
#[component]
pub(super) fn DayHead(
    ctx: Ctx,
    day: i64,
    data: Memo<Option<RosterDay>>,
    /// The day-number button takes a tab stop (Week view); in the month
    /// grid it never does, because Enter on the box does the same.
    tabbable: bool,
    /// Show the weekday before the number (Week view).
    #[prop(optional)]
    with_weekday: bool,
    /// Leave the number out (the Day view, whose title is the date).
    #[prop(optional)]
    no_number: bool,
) -> impl IntoView {
    let is_today = move || ctx.today.get() == day;
    let number = move || {
        let (_, _, dom) = super::types::civil(day);
        if with_weekday {
            let weekday = Weekday::from_epoch_day(day).index_from_monday();
            ctx.texts
                .with(|t| format!("{} {dom}", t.weekday_short[weekday]))
        } else {
            dom.to_string()
        }
    };
    let holiday = move || {
        data.with(|d| {
            d.as_ref()
                .and_then(|d| d.holiday.as_ref())
                .map(|mark| ctx.texts.with(|t| t.holiday_mark_of(mark)))
        })
    };
    let closed = move || {
        data.with(|d| {
            d.as_ref()
                .and_then(|d| d.closed.as_ref())
                .map(|reason| ctx.texts.with(|t| t.closed_mark_of(reason)))
        })
    };
    let staffing = move || data.with(|d| d.as_ref().and_then(|d| d.staffing));
    let number_view = if ctx.navigable() {
        view! {
            <button
                type="button"
                tabindex=(!tabbable).then_some(-1)
                data-day-number=""
                data-pressable="true"
                aria-label=move || ctx.texts.with(|t| fill(&t.show_day, &[("date", &t.date_long_of(day))]))
                class="ld-focus-ring cursor-pointer rounded-sm text-sm font-semibold tabular-nums hover:underline"
                on:click=move |_| ctx.navigate(RosterZoom::Day, day)
            >
                {number}
            </button>
        }
        .into_any()
    } else {
        view! {
            <span aria-hidden="true" data-day-number="" class="text-sm font-semibold tabular-nums">
                {number}
            </span>
        }
        .into_any()
    };

    view! {
        <div data-day-head="" class="flex flex-wrap items-baseline gap-x-2 gap-y-1 text-xs">
            {(!no_number).then_some(number_view)}
            <Show when=is_today>
                <span data-today-mark="" class="font-semibold">
                    {move || ctx.texts.with(|t| t.today.clone())}
                </span>
            </Show>
            {move || holiday().map(|text| view! { <span data-day-mark="holiday" class="font-semibold">{text}</span> })}
            {move || closed().map(|text| view! { <span data-day-mark="closed" class="font-semibold">{text}</span> })}
            {move || {
                staffing()
                    .map(|staffing| {
                        let text = ctx.texts.with(|t| t.staffing_of(staffing));
                        let short = staffing.is_short();
                        view! {
                            <span
                                data-staffing=""
                                data-staffing-short=short.then_some("")
                                class=if short { "font-semibold" } else { "text-base-content/75" }
                            >
                                {text}
                            </span>
                        }
                    })
            }}
        </div>
    }
}

/// "No schedule published" for an unknown day, "No one scheduled" for a
/// known empty one; nothing when the day has entries.
#[component]
pub(super) fn DayStatus(ctx: Ctx, data: Memo<Option<RosterDay>>) -> impl IntoView {
    move || {
        data.with(|d| match d {
            None => Some(
                view! {
                    <p data-day-status="unknown" class="mt-1 text-xs text-base-content/75">
                        {move || ctx.texts.with(|t| t.no_schedule.clone())}
                    </p>
                }
                .into_any(),
            ),
            Some(d) if d.entries.is_empty() => Some(
                view! {
                    <p data-day-status="empty" class="mt-1 text-xs">
                        {move || ctx.texts.with(|t| t.no_one.clone())}
                    </p>
                }
                .into_any(),
            ),
            Some(_) => None,
        })
    }
}

/// One person on one day: the name, the detail, the state in words, the
/// state's tint and accent bar.
#[component]
pub(super) fn Chip(
    ctx: Ctx,
    entry: RosterEntry,
    /// Whether the chip's control is in the Tab order (the month grid takes
    /// every chip outside the active box out of it).
    in_tab_order: Signal<bool>,
    /// Show "HH:MM-HH:MM" before the host's detail (the Week view).
    prefer_times: bool,
) -> impl IntoView {
    let kind = entry.kind;
    let id = entry.id.clone();
    let person = entry.person.clone();
    let host_detail = entry.detail.clone();
    let times = entry.time_span();
    let detail = move || {
        let range = times.map(|(start, end)| ctx.texts.with(|t| t.time_range_of(start, end)));
        if prefer_times {
            range.or_else(|| host_detail.clone())
        } else {
            host_detail.clone().or(range)
        }
    };
    let words = move || ctx.texts.with(|t| t.state_words(kind));
    let lines = move || {
        view! {
            <span translate="no" class="block truncate font-semibold">{person.clone()}</span>
            {move || detail().map(|text| view! { <span translate="no" class="block break-words">{text}</span> })}
            <span data-entry-words="" class="block">{words}</span>
        }
    };
    let body = match (ctx.on_entry_activate, entry.link.clone()) {
        (Some(activate), _) => {
            let id = id.clone();
            view! {
                <button
                    type="button"
                    data-entry-open=""
                    data-pressable="true"
                    tabindex=move || (!in_tab_order.get()).then_some(-1)
                    class="ld-focus-ring block w-full min-w-0 cursor-pointer rounded-sm text-left"
                    on:click=move |_| activate.run(id.clone())
                >
                    {lines()}
                </button>
            }
            .into_any()
        }
        (None, Some(href)) => view! {
            <a
                href=href
                data-entry-open=""
                tabindex=move || (!in_tab_order.get()).then_some(-1)
                class="ld-focus-ring block min-w-0 rounded-sm text-base-content no-underline hover:underline"
            >
                {lines()}
            </a>
        }
        .into_any(),
        (None, None) => view! { <div class="min-w-0">{lines()}</div> }.into_any(),
    };

    view! {
        <li
            data-entry=id
            data-entry-state=kind.slug()
            data-entry-pending=kind.is_pending().then_some("")
            class=format!("{CHIP} {}", kind.colour_classes())
        >
            {body}
        </li>
    }
}
