use super::types::{CalEvent, compute_week_event_layout, day_of_month, weekday_abbrev};
use crate::components::day_scheduler::{
    EventLayout, HourFormat, effective_height_px, minute_to_percent,
};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # WeekView Component
///
/// A Mon-Sun week calendar: seven day-columns side by side, each a vertical
/// time grid sharing an hour gutter, with day headers (today highlighted),
/// an all-day strip, absolutely time-positioned event blocks (accent bar +
/// title + location), and an optional amber "now" line. The seven-day
/// analogue of [`DayScheduler`](crate::components::DayScheduler) -- it
/// reuses that component's overlap-lane algorithm
/// ([`compute_event_layout`](crate::components::day_scheduler::compute_event_layout),
/// wrapped per day-column as
/// [`compute_week_event_layout`](super::types::compute_week_event_layout))
/// and its `minute_to_percent` / `effective_height_px` helpers, applying
/// each day-column's events within that column independently. Ported from
/// d2d-ui's owner-drawn `WeekView` control -- the Direct2D `rect`/brush/
/// `draw()` painting is replaced by CSS absolute positioning, and the
/// dependency-free UTC date math (`civil_from_days`, `week_start_for`,
/// `week_range_label`) is carried over as pure functions in
/// [`super::types`]. This component has no internal clock -- pass a
/// caller-supplied `today` column index and `now_min` (e.g. derived from
/// [`use_sla_now`](crate::components::use_sla_now)) for a live "now" line.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{
///     CalEvent, HourFormat, SchedulerEventColor, WeekView, week_start_for,
/// };
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let week_start = week_start_for(20_514); // Monday 2026-03-02
///     let events = Signal::derive(move || {
///         vec![
///             CalEvent::new("Standup", 0, 9 * 60, 9 * 60 + 15, SchedulerEventColor::Primary)
///                 .with_location("Room 1"),
///             CalEvent::new("Board meeting", 2, 0, 0, SchedulerEventColor::Accent).all_day(),
///         ]
///     });
///
///     view! {
///         <WeekView
///             start_hour=8
///             end_hour=18
///             week_start_epoch_day=week_start
///             events=events
///             today=Some(2)
///             now_min=Some(10 * 60 + 30)
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col w-full overflow-hidden rounded-box border border-base-300");
/// @source inline("flex border-b border-base-300 bg-base-100");
/// @source inline("w-12 shrink-0 flex-1 border-l border-base-300 first:border-l-0 bg-base-200/60");
/// @source inline("py-1 text-center text-xs opacity-60 text-sm font-medium");
/// @source inline("min-h-6 flex items-start justify-end pr-1 pt-1 text-xs opacity-60");
/// @source inline("flex-col gap-px p-px truncate rounded-sm border-l-4 px-1 text-xs");
/// @source inline("relative overflow-hidden border-r");
/// @source inline("absolute inset-x-0 border-t border-base-300 border-base-300/40");
/// @source inline("absolute right-2 -translate-y-1/2 whitespace-nowrap text-xs opacity-60");
/// @source inline("absolute m-px overflow-hidden rounded-sm border-l-4 p-1 text-xs font-medium opacity-70");
/// @source inline("bg-neutral/15 bg-primary/15 bg-secondary/15 bg-accent/15 bg-info/15 bg-success/15 bg-warning/15 bg-error/15");
/// @source inline("border-neutral border-primary border-secondary border-accent border-info border-success border-warning border-error");
/// @source inline("pointer-events-none absolute inset-x-0 z-10 flex items-center");
/// @source inline("-ml-1 h-2 w-2 shrink-0 rounded-full bg-warning h-px flex-1");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn WeekView(
    /// First hour shown (0-23). Defaults to `0` (midnight).
    #[prop(optional, into, default = Signal::derive(|| 0))]
    start_hour: Signal<u32>,

    /// Last hour shown (1-24; coerced to at least `start_hour + 1`).
    /// Defaults to `24` (midnight).
    #[prop(optional, into, default = Signal::derive(|| 24))]
    end_hour: Signal<u32>,

    /// Epoch day (days since 1970-01-01 UTC) of the Monday that starts the
    /// displayed week -- drives the header's date numbers. Normalise with
    /// [`week_start_for`](super::types::week_start_for) if you have an
    /// arbitrary date rather than a known Monday.
    #[prop(optional, into, default = Signal::derive(|| 0))]
    week_start_epoch_day: Signal<i64>,

    /// Events for the week. Timed events are positioned within their
    /// [`CalEvent::day`] column; `all_day` events render in the all-day
    /// strip instead.
    #[prop(optional, into)]
    events: Signal<Vec<CalEvent>>,

    /// Hour-label clock format (24h default).
    #[prop(optional, into)]
    hour_format: Signal<HourFormat>,

    /// Which day column (`0` = Monday .. `6` = Sunday) is "today", if any.
    /// Highlights that column's header/body and gates the now-line.
    #[prop(optional, into)]
    today: Signal<Option<usize>>,

    /// Minutes-from-midnight for the amber now-line within `today`'s
    /// column. This component keeps no internal timer -- pair it with a
    /// ticking `Signal` for a live line. No line is drawn when `today` is
    /// `None` or this is `None`.
    #[prop(optional, into)]
    now_min: Signal<Option<u32>>,

    /// Height of the time grid, in pixels. `0.0` (the default) auto-computes
    /// 60px per displayed hour.
    #[prop(optional, into, default = Signal::derive(|| 0.0))]
    height_px: Signal<f64>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    // The seven day-column indices (0=Mon..6=Sun). Always the same seven
    // values -- a `Signal` (rather than a plain `Vec`) so it's `Copy` and
    // can be captured by each `<For>`'s `move` closure independently.
    let day_indices = Signal::derive(|| (0..7usize).collect::<Vec<usize>>());

    let hours = Signal::derive(move || {
        let s = start_hour.get();
        let e = end_hour.get().max(s + 1);
        (s..=e).collect::<Vec<u32>>()
    });

    // Hours that get an additional (fainter) half-hour line -- every
    // displayed hour except the last.
    let half_hours = Signal::derive(move || {
        let s = start_hour.get();
        let e = end_hour.get().max(s + 1);
        (s..e).collect::<Vec<u32>>()
    });

    let grid_height =
        move || effective_height_px(height_px.get(), start_hour.get(), end_hour.get());

    // Timed events grouped by day column, each column laid into overlap
    // lanes independently via `compute_week_event_layout`. `Memo` (not
    // `Signal::derive`) because each of the seven day-columns reads this
    // signal once per render -- a plain derived signal would recompute the
    // full grouping + lane layout on every one of those reads.
    let timed_by_day = Memo::new(move |_| {
        let evs = events.get();
        let s = start_hour.get();
        let e = end_hour.get();
        (0..7)
            .map(|day| {
                let day_events: Vec<CalEvent> = evs
                    .iter()
                    .filter(|ev| !ev.all_day && ev.day == day)
                    .cloned()
                    .collect();
                let layouts = compute_week_event_layout(&day_events, s, e);
                day_events
                    .into_iter()
                    .zip(layouts)
                    .enumerate()
                    .collect::<Vec<(usize, (CalEvent, EventLayout))>>()
            })
            .collect::<Vec<_>>()
    });

    // All-day events grouped by day column; simply stacked (no lane math --
    // the all-day strip has no time axis to overlap on). `Memo` for the same
    // reason as `timed_by_day` above -- read once per day-column.
    let all_day_by_day = Memo::new(move |_| {
        let evs = events.get();
        (0..7)
            .map(|day| {
                evs.iter()
                    .filter(|ev| ev.all_day && ev.day == day)
                    .cloned()
                    .enumerate()
                    .collect::<Vec<(usize, CalEvent)>>()
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("flex flex-col w-full overflow-hidden rounded-box border border-base-300", class)
        >
            // Day headers: weekday abbreviation + date number, today highlighted.
            <div class="flex border-b border-base-300 bg-base-100">
                <div class="w-12 shrink-0"></div>
                <For
                    each=move || day_indices.get()
                    key=|day| *day
                    children=move |day| {
                        view! {
                            <div class=move || {
                                merge_classes!(
                                    "flex-1 border-l border-base-300 first:border-l-0 py-1 text-center",
                                    if today.get() == Some(day) { "bg-base-200/60" } else { "" }
                                )
                            }>
                                <div class="text-xs opacity-60">{weekday_abbrev(day)}</div>
                                <div class="text-sm font-medium">
                                    {move || day_of_month(week_start_epoch_day.get(), day)}
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            // All-day strip.
            <div class="flex border-b border-base-300">
                <div class="min-h-6 w-12 shrink-0 flex items-start justify-end pr-1 pt-1 text-xs opacity-60">
                    "All day"
                </div>
                <For
                    each=move || day_indices.get()
                    key=|day| *day
                    children=move |day| {
                        view! {
                            <div class=move || {
                                merge_classes!(
                                    "flex flex-1 flex-col gap-px border-l border-base-300 first:border-l-0 p-px",
                                    if today.get() == Some(day) { "bg-base-200/60" } else { "" }
                                )
                            }>
                                <For
                                    each=move || all_day_by_day.get()[day].clone()
                                    key=|(idx, ev)| {
                                        (*idx, ev.day, ev.title.clone(), ev.location.clone(), format!("{:?}", ev.color))
                                    }
                                    children=move |(_, ev)| {
                                        view! {
                                            <div
                                                class=merge_classes!(
                                                    "truncate rounded-sm border-l-4 px-1 text-xs",
                                                    ev.color.bg_class(),
                                                    ev.color.border_class()
                                                )
                                                title=ev.title.clone()
                                            >
                                                {ev.title.clone()}
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                />
            </div>

            // Time grid: hour gutter + seven day-columns.
            <div class="flex" style:height=move || format!("{}px", grid_height())>
                // Hour gutter.
                <div class="relative w-12 shrink-0 border-r border-base-300">
                    <For
                        each=move || hours.get()
                        key=|h| *h
                        children=move |hour| {
                            view! {
                                <div
                                    class="absolute right-2 -translate-y-1/2 whitespace-nowrap text-xs opacity-60"
                                    style:top=move || {
                                        format!(
                                            "{}%",
                                            minute_to_percent(hour as f64 * 60.0, start_hour.get(), end_hour.get()),
                                        )
                                    }
                                >
                                    {move || hour_format.get().label(hour)}
                                </div>
                            }
                        }
                    />
                </div>

                // Seven day-columns.
                <div class="relative flex flex-1">
                    <For
                        each=move || day_indices.get()
                        key=|day| *day
                        children=move |day| {
                            view! {
                                <div class=move || {
                                    merge_classes!(
                                        "relative flex-1 overflow-hidden border-l border-base-300 first:border-l-0",
                                        if today.get() == Some(day) { "bg-base-200/60" } else { "" }
                                    )
                                }>
                                    // Hour gridlines.
                                    <For
                                        each=move || hours.get()
                                        key=|h| *h
                                        children=move |hour| {
                                            view! {
                                                <div
                                                    class="absolute inset-x-0 border-t border-base-300"
                                                    style:top=move || {
                                                        format!(
                                                            "{}%",
                                                            minute_to_percent(hour as f64 * 60.0, start_hour.get(), end_hour.get()),
                                                        )
                                                    }
                                                ></div>
                                            }
                                        }
                                    />

                                    // Half-hour gridlines (fainter).
                                    <For
                                        each=move || half_hours.get()
                                        key=|h| *h
                                        children=move |hour| {
                                            view! {
                                                <div
                                                    class="absolute inset-x-0 border-t border-base-300/40"
                                                    style:top=move || {
                                                        format!(
                                                            "{}%",
                                                            minute_to_percent(
                                                                hour as f64 * 60.0 + 30.0,
                                                                start_hour.get(),
                                                                end_hour.get(),
                                                            ),
                                                        )
                                                    }
                                                ></div>
                                            }
                                        }
                                    />

                                    // Timed event blocks for this day column.
                                    <For
                                        each=move || timed_by_day.get()[day].clone()
                                        key=|(idx, (ev, layout))| {
                                            (
                                                *idx,
                                                ev.day,
                                                ev.start_min,
                                                ev.end_min,
                                                ev.title.clone(),
                                                ev.location.clone(),
                                                format!("{:?}", ev.color),
                                                layout.top_pct.to_bits(),
                                                layout.height_pct.to_bits(),
                                                layout.left_pct.to_bits(),
                                                layout.width_pct.to_bits(),
                                            )
                                        }
                                        children=move |(_, (ev, layout))| {
                                            let has_location = !ev.location.is_empty();
                                            let location = ev.location.clone();
                                            view! {
                                                <div
                                                    class=merge_classes!(
                                                        "absolute m-px overflow-hidden rounded-sm border-l-4 p-1 text-xs",
                                                        ev.color.bg_class(),
                                                        ev.color.border_class()
                                                    )
                                                    style:top=format!("{}%", layout.top_pct)
                                                    style:height=format!("{}%", layout.height_pct)
                                                    style:left=format!("{}%", layout.left_pct)
                                                    style:width=format!("{}%", layout.width_pct)
                                                    title=ev.title.clone()
                                                >
                                                    <div class="truncate font-medium">{ev.title.clone()}</div>
                                                    <Show when=move || has_location>
                                                        <div class="truncate opacity-70">{location.clone()}</div>
                                                    </Show>
                                                </div>
                                            }
                                        }
                                    />

                                    // Amber "now" line -- only in today's column.
                                    <Show when=move || today.get() == Some(day) && now_min.get().is_some()>
                                        <div
                                            class="pointer-events-none absolute inset-x-0 z-10 flex items-center"
                                            style:top=move || {
                                                format!(
                                                    "{}%",
                                                    minute_to_percent(
                                                        now_min.get().unwrap_or(0) as f64,
                                                        start_hour.get(),
                                                        end_hour.get(),
                                                    ),
                                                )
                                            }
                                        >
                                            <span class="-ml-1 h-2 w-2 shrink-0 rounded-full bg-warning"></span>
                                            <span class="h-px flex-1 bg-warning"></span>
                                        </div>
                                    </Show>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
