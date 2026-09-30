use super::types::{
    CalEvent, WEEK_DAYS, WeekEventKeyIntent, Weekday, WeekdaySet, column_weekday,
    compute_week_layer_layout, day_of_month, resolve_week_event_accessible_label,
    week_event_key_intent,
};
use crate::components::day_scheduler::{
    EventLayout, HourFormat, effective_height_px, minute_to_percent,
};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # WeekView Component
///
/// A seven-day week calendar: seven day-columns side by side, each a
/// vertical time grid sharing an hour gutter, with day headers (today
/// highlighted), an all-day strip, absolutely time-positioned event blocks
/// (accent bar + title + location), and an optional amber "now" line. The
/// seven-day analogue of [`DayScheduler`](crate::components::DayScheduler)
/// -- it reuses that component's overlap-lane algorithm
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
/// The week starts on whatever day `week_start_epoch_day` is, and each
/// header names its own date's weekday: pass a Monday (see
/// [`week_start_for`](super::types::week_start_for)) for Mon .. Sun, or a
/// Sunday (see [`week_start_for_weekday`](super::types::week_start_for_weekday))
/// for Sun .. Sat. [`CalEvent::day`] and `today` are column indices counted
/// from that first day.
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
/// ### Work days
///
/// `work_days` is the set of working weekdays. Columns whose weekday is not
/// in it are shaded (`data-work-day="false"`) but stay fully schedulable --
/// shading a day is not hiding it. The default is all seven days, a
/// seven-day workweek, which shades nothing.
///
/// ### Interactive planner contract
///
/// The same contract as [`DayScheduler`](crate::components::DayScheduler),
/// plus a day axis. Supplying any of `on_event_activate`, `selected_event`,
/// `on_event_move`, `on_event_move_day` or `on_event_resize` makes every
/// event block (timed and all-day) a focusable `role="button"`; a
/// display-only week gains no tab stops. Every index is an index into
/// `events`.
///
/// | Key | Request |
/// |---|---|
/// | Enter / Space, click | select + `on_event_activate(index)` |
/// | ArrowUp / ArrowDown | `on_event_move((index, -/+step))` (timed events) |
/// | Shift+ArrowUp / Shift+ArrowDown | `on_event_resize((index, -/+step))` (timed events) |
/// | ArrowLeft / ArrowRight | `on_event_move_day((index, -1/+1))` |
///
/// The view never mutates `events`: the consumer applies, clamps or refuses
/// each request. Timed blocks live in ONE layer over all seven columns,
/// keyed by index, so a day-move changes the block's `left` and keeps the
/// same DOM node -- focus survives and the next arrow press still lands.
///
/// A block's accessible name carries its day, because a screen-reader user
/// tabbing between blocks never hears the column header:
/// `"Standup, Monday Mar 2, 09:00 to 09:15"`. A localised page supplies
/// `event_accessible_label` (given the event and its epoch day) and
/// `weekday_label` (given a [`Weekday`], returning the header text). Empty
/// callback output falls back to the English default, never to no name.
///
/// ### Consumer drop targets
///
/// The day columns' area carries `data-week-columns` and each column
/// `data-week-day="<index>"`. A drag-and-drop consumer measures the
/// `data-week-columns` element and converts the pointer with
/// [`week_slot_at`](super::types::week_slot_at). Timed blocks carry
/// `data-week-event` and a reactive `data-week-event-day`.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col w-full overflow-hidden rounded-box border border-base-300");
/// @source inline("flex border-b border-base-300 bg-base-100");
/// @source inline("w-12 shrink-0 flex-1 border-l border-base-300 first:border-l-0 bg-base-200/60 bg-base-300/30");
/// @source inline("py-1 text-center text-xs opacity-60 text-sm font-medium text-primary");
/// @source inline("min-h-6 flex items-start justify-end pr-1 pt-1 text-xs opacity-60");
/// @source inline("flex-col gap-px p-px truncate rounded-sm border-l-4 px-1 text-xs");
/// @source inline("grid grid-cols-7 grid-flow-row-dense min-w-0");
/// @source inline("relative overflow-hidden border-r");
/// @source inline("absolute inset-0 inset-x-0 border-t border-base-300 border-base-300/40");
/// @source inline("absolute right-2 -translate-y-1/2 whitespace-nowrap text-xs opacity-60");
/// @source inline("absolute m-px overflow-hidden rounded-sm border-l-4 p-1 text-xs font-medium opacity-70");
/// @source inline("bg-neutral/15 bg-primary/15 bg-secondary/15 bg-accent/15 bg-info/15 bg-success/15 bg-warning/15 bg-error/15");
/// @source inline("border-neutral border-primary border-secondary border-accent border-info border-success border-warning border-error");
/// @source inline("pointer-events-none pointer-events-auto absolute inset-x-0 z-10 flex items-center");
/// @source inline("-ml-1 h-2 w-2 shrink-0 rounded-full bg-warning h-px flex-1 ml-1 text-warning");
/// @source inline("hidden ring-2 ring-primary z-20 w-full rounded-box cursor-pointer hover:bg-base-200");
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

    /// Epoch day (days since 1970-01-01 UTC) of the week's first day --
    /// drives the headers' weekday names and date numbers. Normalise with
    /// [`week_start_for`](super::types::week_start_for) (Monday) or
    /// [`week_start_for_weekday`](super::types::week_start_for_weekday)
    /// (any first day) if you have an arbitrary date.
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

    /// Optional formatter for the gutter's hour labels, taking the hour
    /// (`start_hour..=end_hour`) and returning the text to draw. Overrides
    /// `hour_format` when supplied. This is the escape hatch for locales
    /// neither built-in format covers -- a Spanish page wanting `8 h`, or a
    /// label pulled from the consumer's own i18n catalogue.
    #[prop(optional, into)]
    hour_label: Option<Callback<u32, String>>,

    /// Optional formatter for the day headers' weekday text, given the
    /// column's [`Weekday`]. Defaults to the English abbreviation (`"Mon"`);
    /// a Spanish page returns `"lun"`. Read reactively, so a callback that
    /// closes over a locale `Signal` relabels the headers in place.
    #[prop(optional, into)]
    weekday_label: Option<Callback<Weekday, String>>,

    /// Which day column (`0` .. `6`, counted from the week's first day) is
    /// "today", if any. Highlights that column's header/body and gates the
    /// now-line.
    #[prop(optional, into)]
    today: Signal<Option<usize>>,

    /// Minutes-from-midnight for the amber now-line within `today`'s
    /// column. This component keeps no internal timer -- pair it with a
    /// ticking `Signal` for a live line. No line is drawn when `today` is
    /// `None` or this is `None`.
    #[prop(optional, into)]
    now_min: Signal<Option<u32>>,

    /// Optional inline label drawn beside the now-line (e.g. `"Now"`).
    /// Shown only when the line is drawn and this is non-empty.
    #[prop(optional, into)]
    now_label: Signal<String>,

    /// The working weekdays. Columns whose weekday is not in the set are
    /// shaded and marked `data-work-day="false"`; they stay schedulable.
    /// Defaults to [`WeekdaySet::ALL`] (a seven-day workweek: no shading).
    #[prop(optional, into)]
    work_days: Signal<WeekdaySet>,

    /// Height of the time grid, in pixels. `0.0` (the default) auto-computes
    /// 60px per displayed hour.
    #[prop(optional, into, default = Signal::derive(|| 0.0))]
    height_px: Signal<f64>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Optional event activation: a click or Enter/Space on an event block
    /// calls this with the event's index into `events`. Supplying any of the
    /// interaction props makes event blocks focusable buttons with an
    /// accessible name that includes the day; see the component docs.
    #[prop(optional, into)]
    on_event_activate: Option<Callback<usize>>,

    /// Optional accessible-name formatter for interactive event blocks,
    /// given `(event, epoch_day)` -- the block's own date, so a localised
    /// name can include the day. Empty or whitespace-only output falls back
    /// to the English [`week_event_aria_label`](super::types::week_event_aria_label).
    /// Read inside the reactive `aria-label` closure, so a callback over a
    /// locale `Signal` renames the SAME node in place.
    #[prop(optional, into)]
    event_accessible_label: Option<Callback<(CalEvent, i64), String>>,

    /// Optional controlled selection: the index of the selected event, if
    /// any. Clicking or activating an event selects it; the selected block
    /// carries `aria-pressed="true"` and a selection ring.
    #[prop(optional)]
    selected_event: Option<RwSignal<Option<usize>>>,

    /// Optional keyboard time-move contract: ArrowUp/ArrowDown on a focused
    /// timed event requests `(index, signed minute delta)` -- negative is
    /// earlier.
    #[prop(optional, into)]
    on_event_move: Option<Callback<(usize, i32)>>,

    /// Optional keyboard day-move contract: ArrowLeft/ArrowRight on a
    /// focused event requests `(index, signed day delta)` -- `-1` is the
    /// previous column. The consumer decides what a move past the first or
    /// last column means (refuse it, or page the week).
    #[prop(optional, into)]
    on_event_move_day: Option<Callback<(usize, i32)>>,

    /// Optional keyboard resize contract: Shift+ArrowUp/ArrowDown on a
    /// focused timed event requests `(index, signed minute delta)` applied
    /// to the event's end time.
    #[prop(optional, into)]
    on_event_resize: Option<Callback<(usize, i32)>>,

    /// Minute step for keyboard move/resize (default 15; `0` coerces to 15).
    #[prop(optional, into, default = Signal::derive(|| 15))]
    move_step_min: Signal<u32>,

    /// Optional custom event-block content, invoked with `(index, event)`.
    /// When absent a timed block renders its title and location, and an
    /// all-day chip its title.
    #[prop(optional, into)]
    event_content: Option<Callback<(usize, CalEvent), AnyView>>,

    /// Optional day drill-down: makes each day header a button that calls
    /// this with its column index (e.g. to open that day in a single-day
    /// scheduler).
    #[prop(optional, into)]
    on_day_activate: Option<Callback<usize>>,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    // Focus/keyboard semantics exist only when the consumer opted into some
    // interaction -- DayScheduler's rule.
    let interactive = on_event_activate.is_some()
        || selected_event.is_some()
        || on_event_move.is_some()
        || on_event_move_day.is_some()
        || on_event_resize.is_some();

    // The seven day-column indices. Always the same seven values -- a
    // `Signal` (rather than a plain `Vec`) so it's `Copy` and can be captured
    // by each `<For>`'s `move` closure independently.
    let day_indices = Signal::derive(|| (0..WEEK_DAYS).collect::<Vec<usize>>());

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

    // Every timed event's position over the whole seven-column area,
    // index-aligned with `events` (`None` for all-day events). `Memo`
    // because every block reads it.
    let layer_layouts = Memo::new(move |_| {
        events.with(|evs| compute_week_layer_layout(evs, start_hour.get(), end_hour.get()))
    });

    // Indices of the timed and all-day events. An event keeps its index --
    // and so its DOM node -- when it moves between days; only a switch
    // between timed and all-day moves it between the two lists.
    let timed_indices = Memo::new(move |_| {
        events.with(|evs| {
            (0..evs.len())
                .filter(|&i| !evs[i].all_day)
                .collect::<Vec<usize>>()
        })
    });
    let all_day_indices = Memo::new(move |_| {
        events.with(|evs| {
            (0..evs.len())
                .filter(|&i| evs[i].all_day)
                .collect::<Vec<usize>>()
        })
    });

    let is_work_day = move |day: usize| {
        work_days
            .get()
            .contains(column_weekday(week_start_epoch_day.get(), day))
    };
    // Today's tint wins over the non-work shading.
    let column_tint = move |day: usize| {
        if today.get() == Some(day) {
            "bg-base-200/60"
        } else if !is_work_day(day) {
            "bg-base-300/30"
        } else {
            ""
        }
    };

    // One block's shared behaviour -- the reactive event read, the
    // selection state, the name and the keyboard contract -- used by both
    // the timed layer and the all-day strip.
    let event_at =
        move |idx: usize| Signal::derive(move || events.with(|evs| evs.get(idx).cloned()));
    let select_and_activate = move |idx: usize| {
        if let Some(s) = selected_event {
            s.set(Some(idx));
        }
        if let Some(cb) = on_event_activate {
            cb.run(idx);
        }
    };
    let accessible_name = move |ev: Option<CalEvent>| {
        (interactive && ev.is_some()).then(|| {
            let ev = ev.unwrap_or_default();
            let epoch_day = week_start_epoch_day.get() + ev.day.min(WEEK_DAYS - 1) as i64;
            let formatted = event_accessible_label.map(|fmt| fmt.run((ev.clone(), epoch_day)));
            resolve_week_event_accessible_label(&ev, week_start_epoch_day.get(), formatted)
        })
    };
    let on_block_keydown = move |idx: usize, all_day: bool, ev_k: web_sys::KeyboardEvent| {
        if !interactive {
            return;
        }
        let step = match move_step_min.get_untracked() {
            0 => 15,
            s => s,
        };
        match week_event_key_intent(&ev_k.key(), ev_k.shift_key(), step) {
            Some(WeekEventKeyIntent::Activate) => {
                ev_k.prevent_default();
                select_and_activate(idx);
            }
            Some(WeekEventKeyIntent::MoveDay(delta)) => {
                ev_k.prevent_default();
                if let Some(cb) = on_event_move_day {
                    cb.run((idx, delta));
                }
            }
            // An all-day event has no time axis to move or resize along.
            Some(WeekEventKeyIntent::Move(delta)) if !all_day => {
                ev_k.prevent_default();
                if let Some(cb) = on_event_move {
                    cb.run((idx, delta));
                }
            }
            Some(WeekEventKeyIntent::Resize(delta)) if !all_day => {
                ev_k.prevent_default();
                if let Some(cb) = on_event_resize {
                    cb.run((idx, delta));
                }
            }
            _ => {}
        }
    };

    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("flex flex-col w-full overflow-hidden rounded-box border border-base-300", class)
        >
            // Day headers: weekday + date number, today highlighted.
            <div class="flex border-b border-base-300 bg-base-100">
                <div class="w-12 shrink-0"></div>
                <For
                    each=move || day_indices.get()
                    key=|day| *day
                    children=move |day| {
                        let weekday_text = move || {
                            let weekday = column_weekday(week_start_epoch_day.get(), day);
                            match weekday_label {
                                Some(fmt) => fmt.run(weekday),
                                None => weekday.abbrev().to_string(),
                            }
                        };
                        let date_number = move || day_of_month(week_start_epoch_day.get(), day);
                        let is_today = move || today.get() == Some(day);
                        let date_class = move || {
                            if is_today() { "text-sm font-medium text-primary" } else { "text-sm font-medium" }
                        };
                        view! {
                            <div
                                class=move || {
                                    merge_classes!(
                                        "flex-1 border-l border-base-300 first:border-l-0 py-1 text-center",
                                        column_tint(day)
                                    )
                                }
                                data-week-day-header=day
                                data-work-day=move || if is_work_day(day) { "true" } else { "false" }
                                aria-current=move || is_today().then_some("date")
                            >
                                {match on_day_activate {
                                    Some(cb) => view! {
                                        <button
                                            type="button"
                                            data-pressable="true"
                                            class="w-full cursor-pointer rounded-box hover:bg-base-200"
                                            data-week-day-button=day
                                            on:click=move |_| cb.run(day)
                                        >
                                            <div class="text-xs opacity-60" data-week-weekday="true">{weekday_text}</div>
                                            <div class=date_class data-week-date="true">{date_number}</div>
                                        </button>
                                    }
                                    .into_any(),
                                    None => view! {
                                        <div class="text-xs opacity-60" data-week-weekday="true">{weekday_text}</div>
                                        <div class=date_class data-week-date="true">{date_number}</div>
                                    }
                                    .into_any(),
                                }}
                            </div>
                        }
                    }
                />
            </div>

            // All-day strip: a background row of seven column cells, and over
            // it one grid whose chips are placed by `grid-column` and keyed by
            // index -- so a day-move keeps the chip's node, like a timed block.
            <div class="flex border-b border-base-300">
                <div class="min-h-6 w-12 shrink-0 flex items-start justify-end pr-1 pt-1 text-xs opacity-60">
                    "All day"
                </div>
                <div class="relative flex-1 min-w-0">
                    <div class="absolute inset-0 flex">
                        <For
                            each=move || day_indices.get()
                            key=|day| *day
                            children=move |day| {
                                view! {
                                    <div class=move || {
                                        merge_classes!(
                                            "flex-1 border-l border-base-300 first:border-l-0",
                                            column_tint(day)
                                        )
                                    }></div>
                                }
                            }
                        />
                    </div>
                    <div class="relative grid min-h-6 grid-cols-7 grid-flow-row-dense gap-px p-px">
                        <For
                            each=move || all_day_indices.get()
                            key=|idx| *idx
                            children=move |idx| {
                                let ev = event_at(idx);
                                let is_selected = move || {
                                    selected_event.is_some_and(|s| s.get() == Some(idx))
                                };
                                view! {
                                    <div
                                        class=move || {
                                            let ev = ev.get().unwrap_or_default();
                                            merge_classes!(
                                                "min-w-0 truncate rounded-sm border-l-4 px-1 text-xs",
                                                ev.color.bg_class(),
                                                ev.color.border_class()
                                            )
                                            .to_class()
                                        }
                                        class:ring-2=is_selected
                                        class:ring-primary=is_selected
                                        class:hidden=move || ev.with(|e| e.as_ref().is_none_or(|e| !e.all_day))
                                        style:grid-column=move || {
                                            format!("{}", ev.get().unwrap_or_default().day.min(WEEK_DAYS - 1) + 1)
                                        }
                                        data-week-all-day-event=idx
                                        data-week-event-day=move || ev.get().unwrap_or_default().day.min(WEEK_DAYS - 1)
                                        title=move || ev.get().unwrap_or_default().title
                                        role=interactive.then_some("button")
                                        tabindex=interactive.then_some(0)
                                        aria-label=move || accessible_name(ev.get())
                                        aria-pressed=move || {
                                            (interactive && selected_event.is_some()).then(|| {
                                                if is_selected() { "true" } else { "false" }
                                            })
                                        }
                                        on:click=move |_| {
                                            if interactive {
                                                select_and_activate(idx);
                                            }
                                        }
                                        on:keydown=move |ev_k: web_sys::KeyboardEvent| on_block_keydown(idx, true, ev_k)
                                    >
                                        {move || {
                                            ev.get().map(|e| match event_content {
                                                Some(renderer) => renderer.run((idx, e)).into_any(),
                                                None => e.title.into_any(),
                                            })
                                        }}
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
            </div>

            // Time grid: hour gutter + seven day-columns + the event layer.
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
                                    {move || match hour_label {
                                        Some(fmt) => fmt.run(hour),
                                        None => hour_format.get().label(hour),
                                    }}
                                </div>
                            }
                        }
                    />
                </div>

                // Seven day-columns.
                <div class="relative flex flex-1 min-w-0 overflow-hidden" data-week-columns="true">
                    <For
                        each=move || day_indices.get()
                        key=|day| *day
                        children=move |day| {
                            view! {
                                <div
                                    class=move || {
                                        merge_classes!(
                                            "relative flex-1 overflow-hidden border-l border-base-300 first:border-l-0",
                                            column_tint(day)
                                        )
                                    }
                                    data-week-day=day
                                    data-work-day=move || if is_work_day(day) { "true" } else { "false" }
                                >
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
                                            <Show when=move || !now_label.get().is_empty()>
                                                <span class="ml-1 shrink-0 text-xs font-medium text-warning">
                                                    {move || now_label.get()}
                                                </span>
                                            </Show>
                                        </div>
                                    </Show>
                                </div>
                            }
                        }
                    />

                    // Timed event blocks: ONE layer over all seven columns,
                    // keyed by index into `events`. Each block reads its own
                    // event and layout reactively, so a time- or day-move
                    // repositions the same node instead of replacing it (and
                    // dropping focus mid-interaction). The layer passes
                    // pointer events through to the columns underneath; only
                    // the blocks take them.
                    <div class="pointer-events-none absolute inset-0">
                        <For
                            each=move || timed_indices.get()
                            key=|idx| *idx
                            children=move |idx| {
                                let ev = event_at(idx);
                                let layout = Signal::derive(move || {
                                    layer_layouts.with(|l| l.get(idx).copied().flatten())
                                });
                                let pos = move || layout.get().unwrap_or(EventLayout::default());
                                let is_selected = move || {
                                    selected_event.is_some_and(|s| s.get() == Some(idx))
                                };
                                view! {
                                    <div
                                        class=move || {
                                            let ev = ev.get().unwrap_or_default();
                                            merge_classes!(
                                                "pointer-events-auto absolute m-px overflow-hidden rounded-sm border-l-4 p-1 text-xs",
                                                ev.color.bg_class(),
                                                ev.color.border_class()
                                            )
                                            .to_class()
                                        }
                                        class:ring-2=is_selected
                                        class:ring-primary=is_selected
                                        class:z-20=is_selected
                                        class:hidden=move || layout.get().is_none()
                                        style:top=move || format!("{}%", pos().top_pct)
                                        style:height=move || format!("{}%", pos().height_pct)
                                        style:left=move || format!("{}%", pos().left_pct)
                                        style:width=move || format!("{}%", pos().width_pct)
                                        data-week-event=idx
                                        data-week-event-day=move || ev.get().unwrap_or_default().day.min(WEEK_DAYS - 1)
                                        title=move || ev.get().unwrap_or_default().title
                                        role=interactive.then_some("button")
                                        tabindex=interactive.then_some(0)
                                        aria-label=move || accessible_name(ev.get())
                                        aria-pressed=move || {
                                            (interactive && selected_event.is_some()).then(|| {
                                                if is_selected() { "true" } else { "false" }
                                            })
                                        }
                                        on:click=move |_| {
                                            if interactive {
                                                select_and_activate(idx);
                                            }
                                        }
                                        on:keydown=move |ev_k: web_sys::KeyboardEvent| on_block_keydown(idx, false, ev_k)
                                    >
                                        {move || {
                                            ev.get().map(|e| match event_content {
                                                Some(renderer) => renderer.run((idx, e)).into_any(),
                                                None => {
                                                    let has_location = !e.location.is_empty();
                                                    let location = e.location;
                                                    view! {
                                                        <div class="truncate font-medium">{e.title}</div>
                                                        <Show when=move || has_location>
                                                            <div class="truncate opacity-70">{location.clone()}</div>
                                                        </Show>
                                                    }
                                                    .into_any()
                                                }
                                            })
                                        }}
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}
