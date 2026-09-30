use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn WeekViewDemo() -> impl IntoView {
    // Monday 2026-03-02 (epoch day 20_514) -- Wednesday (day 2) is "today".
    let week_start = week_start_for(20_514);

    let events = Signal::derive(move || {
        vec![
            CalEvent::new(
                "Standup",
                0,
                9 * 60,
                9 * 60 + 15,
                SchedulerEventColor::Primary,
            )
            .with_location("Room 1"),
            CalEvent::new(
                "Design review",
                0,
                10 * 60,
                11 * 60,
                SchedulerEventColor::Accent,
            )
            .with_location("Room 2"),
            CalEvent::new(
                "Client call",
                1,
                14 * 60,
                15 * 60,
                SchedulerEventColor::Success,
            )
            .with_location("Zoom"),
            CalEvent::new(
                "Sprint planning",
                2,
                9 * 60,
                10 * 60 + 30,
                SchedulerEventColor::Primary,
            ),
            CalEvent::new(
                "1:1 with manager",
                2,
                9 * 60 + 30,
                10 * 60,
                SchedulerEventColor::Warning,
            ),
            CalEvent::new(
                "Lunch with team",
                3,
                12 * 60,
                13 * 60,
                SchedulerEventColor::Neutral,
            ),
            CalEvent::new(
                "Retro",
                4,
                15 * 60 + 30,
                16 * 60 + 30,
                SchedulerEventColor::Info,
            )
            .with_location("Room 1"),
            CalEvent::new("Company holiday", 5, 0, 0, SchedulerEventColor::Error).all_day(),
            CalEvent::new("Board meeting", 2, 0, 0, SchedulerEventColor::Accent)
                .with_location("HQ")
                .all_day(),
        ]
    });

    // Interactive planner (ldui-9sip): the page owns the events and applies
    // the time/day move requests the keyboard contract reports.
    let planned = RwSignal::new(vec![
        CalEvent::new(
            "Intake review",
            0,
            9 * 60,
            10 * 60,
            SchedulerEventColor::Primary,
        ),
        CalEvent::new(
            "Filing block",
            2,
            11 * 60,
            12 * 60 + 30,
            SchedulerEventColor::Info,
        ),
        CalEvent::new("Office closed", 5, 0, 0, SchedulerEventColor::Neutral).all_day(),
    ]);
    let selected = RwSignal::new(Option::<usize>::None);
    let last_activated = RwSignal::new(Option::<usize>::None);
    let day_opened = RwSignal::new(Option::<usize>::None);
    let apply_move = move |(idx, delta): (usize, i32)| {
        planned.update(|evs| {
            if let Some(ev) = evs.get_mut(idx) {
                let dur = ev.end_min - ev.start_min;
                let start =
                    (ev.start_min as i32 + delta).clamp(8 * 60, 18 * 60 - dur as i32) as u32;
                ev.start_min = start;
                ev.end_min = start + dur;
            }
        });
    };
    let apply_move_day = move |(idx, delta): (usize, i32)| {
        planned.update(|evs| {
            if let Some(ev) = evs.get_mut(idx) {
                ev.day = (ev.day as i32 + delta).clamp(0, 6) as usize;
            }
        });
    };
    let apply_resize = move |(idx, delta): (usize, i32)| {
        planned.update(|evs| {
            if let Some(ev) = evs.get_mut(idx) {
                ev.end_min =
                    (ev.end_min as i32 + delta).clamp(ev.start_min as i32 + 15, 18 * 60) as u32;
            }
        });
    };
    let show = |value: Option<usize>| {
        value
            .map(|i| i.to_string())
            .unwrap_or_else(|| "(none)".to_string())
    };

    // Sunday-first, localised week: the headers name each column's own
    // weekday, and a locale signal relabels headers and accessible names in
    // place.
    let sunday_start = week_start_for_weekday(20_514, Weekday::Sunday); // Sun 2026-03-01
    let locale = RwSignal::new("en".to_string());
    let locale_selected = RwSignal::new(Option::<usize>::None);
    let sunday_events = Signal::derive(|| {
        vec![CalEvent::new(
            "Cita",
            0,
            9 * 60,
            10 * 60,
            SchedulerEventColor::Accent,
        )]
    });

    view! {
        <ContentLayout
            title="Week View"
            description="A seven-day week calendar: seven day-columns as vertical time grids, day headers (today highlighted), an hour gutter, an all-day strip, absolutely time-positioned event blocks, and an amber 'now' line. Ported from d2d-ui's owner-drawn WeekView control, reusing DayScheduler's overlap-lane algorithm per day-column. Optionally an interactive planner: DayScheduler's activation, selection and keyboard contract plus Left/Right day moves, a work-day mask, localised day names and day drill-down."
        >
            <Section title="Business hours, today highlighted with a now-line">
                <div class="w-full max-w-5xl" id="display-week">
                    <WeekView
                        start_hour=8
                        end_hour=18
                        week_start_epoch_day=week_start
                        events=events
                        today=Some(2)
                        now_min=Some(10 * 60 + 45)
                    />
                </div>
            </Section>

            <Section title="12-hour labels, no 'today' highlight">
                <div class="w-full max-w-5xl">
                    <WeekView
                        start_hour=7
                        end_hour=19
                        week_start_epoch_day=week_start
                        events=events
                        hour_format=HourFormat::Twelve
                    />
                </div>
            </Section>

            <Section title="Custom hour labels (localised gutter)">
                <div class="w-full max-w-5xl">
                    <WeekView
                        start_hour=8
                        end_hour=18
                        week_start_epoch_day=week_start
                        events=events
                        hour_label=Callback::new(|hour: u32| format!("{hour} h"))
                    />
                </div>
            </Section>

            <Section title="Compact height override">
                <div class="w-full max-w-5xl">
                    <WeekView
                        start_hour=8
                        end_hour=18
                        week_start_epoch_day=week_start
                        events=events
                        today=Some(2)
                        now_min=Some(10 * 60 + 45)
                        height_px=360.0
                    />
                </div>
            </Section>

            <Section title="Interactive planner (Mon-Fri work days, day + time keyboard moves)">
                <p class="text-sm opacity-75 mb-2">
                    "Supply " <code>"on_event_activate"</code> " / " <code>"selected_event"</code>
                    " / " <code>"on_event_move"</code> " / " <code>"on_event_move_day"</code>
                    " and every block becomes a focusable button whose name includes its day: "
                    <kbd class="kbd kbd-xs">"Enter"</kbd> " activates and selects, "
                    <kbd class="kbd kbd-xs">"↑"</kbd> "/" <kbd class="kbd kbd-xs">"↓"</kbd>
                    " requests a 15-minute move, " <kbd class="kbd kbd-xs">"←"</kbd> "/"
                    <kbd class="kbd kbd-xs">"→"</kbd> " a one-day move, "
                    <kbd class="kbd kbd-xs">"Shift"</kbd> "+↑/↓ a resize. "
                    <code>"work_days"</code> " is Monday to Friday here, so the weekend is shaded "
                    "but still schedulable; the default is all seven days. Day headers are "
                    "buttons (" <code>"on_day_activate"</code> ")."
                </p>
                <div class="mb-2 flex flex-wrap gap-4 text-sm">
                    <span>
                        "Selected: "
                        <code data-testid="week-selected">{move || show(selected.get())}</code>
                    </span>
                    <span>
                        "Last activated: "
                        <code data-testid="week-activated">{move || show(last_activated.get())}</code>
                    </span>
                    <span>
                        "First event: "
                        <code data-testid="week-first">
                            {move || {
                                planned.with(|evs| {
                                    evs.first()
                                        .map(|e| format!("{}:{}-{}", e.day, e.start_min, e.end_min))
                                        .unwrap_or_default()
                                })
                            }}
                        </code>
                    </span>
                    <span>
                        "All-day event day: "
                        <code data-testid="week-all-day">
                            {move || planned.with(|evs| evs.get(2).map(|e| e.day.to_string()).unwrap_or_default())}
                        </code>
                    </span>
                    <span>
                        "Day opened: "
                        <code data-testid="week-day-opened">{move || show(day_opened.get())}</code>
                    </span>
                </div>
                <div class="w-full max-w-5xl" id="interactive-week">
                    <WeekView
                        start_hour=8
                        end_hour=18
                        week_start_epoch_day=week_start
                        events=Signal::derive(move || planned.get())
                        today=Some(2)
                        now_min=Some(10 * 60 + 45)
                        now_label="Now"
                        work_days=WeekdaySet::MONDAY_TO_FRIDAY
                        selected_event=selected
                        on_event_activate=Callback::new(move |idx: usize| last_activated.set(Some(idx)))
                        on_event_move=Callback::new(apply_move)
                        on_event_move_day=Callback::new(apply_move_day)
                        on_event_resize=Callback::new(apply_resize)
                        on_day_activate=Callback::new(move |day: usize| day_opened.set(Some(day)))
                    />
                </div>
            </Section>

            <Section title="Sunday-first week with localised day names (EN / ES)">
                <p class="text-sm opacity-75 mb-2">
                    "The week starts on whatever day " <code>"week_start_epoch_day"</code>
                    " is (here " <code>"week_start_for_weekday(.., Weekday::Sunday)"</code>
                    "), and each header names its own date's weekday. "
                    <code>"weekday_label"</code> " and " <code>"event_accessible_label"</code>
                    " read a locale signal, so the toggle relabels the same nodes in place."
                </p>
                <div class="mb-2 flex items-center gap-4 text-sm">
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="week-locale-toggle"
                        on:click=move |_| {
                            locale.update(|l| *l = if l == "en" { "es".into() } else { "en".into() })
                        }
                    >
                        {move || if locale.get() == "en" { "Español" } else { "English" }}
                    </Button>
                    <span>
                        "Selected: "
                        <code data-testid="week-locale-selected">{move || show(locale_selected.get())}</code>
                    </span>
                </div>
                <div class="w-full max-w-5xl" id="localized-week">
                    <WeekView
                        start_hour=8
                        end_hour=12
                        week_start_epoch_day=sunday_start
                        events=sunday_events
                        selected_event=locale_selected
                        weekday_label=Callback::new(move |day: Weekday| {
                            if locale.get() == "es" { es_weekday(day).0.to_string() } else { day.abbrev().to_string() }
                        })
                        event_accessible_label=Callback::new(move |(ev, epoch_day): (CalEvent, i64)| {
                            if locale.get() != "es" {
                                return String::new();
                            }
                            format!(
                                "{}, {} {}, de {} a {}",
                                ev.title,
                                es_weekday(Weekday::from_epoch_day(epoch_day)).1,
                                civil_from_days(epoch_day).2,
                                minute_label(ev.start_min),
                                minute_label(ev.end_min),
                            )
                        })
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}

/// Spanish `(abbreviation, full name)` for a weekday -- the demo's stand-in
/// for a consumer's own i18n catalogue.
fn es_weekday(day: Weekday) -> (&'static str, &'static str) {
    match day {
        Weekday::Monday => ("lun", "lunes"),
        Weekday::Tuesday => ("mar", "martes"),
        Weekday::Wednesday => ("mié", "miércoles"),
        Weekday::Thursday => ("jue", "jueves"),
        Weekday::Friday => ("vie", "viernes"),
        Weekday::Saturday => ("sáb", "sábado"),
        Weekday::Sunday => ("dom", "domingo"),
    }
}
