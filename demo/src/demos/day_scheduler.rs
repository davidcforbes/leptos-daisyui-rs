use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn DaySchedulerDemo() -> impl IntoView {
    let basic_events = Signal::derive(|| {
        vec![
            SchedulerEvent::new("Standup", 9 * 60, 9 * 60 + 15, SchedulerEventColor::Primary),
            SchedulerEvent::new(
                "Design review",
                10 * 60,
                11 * 60,
                SchedulerEventColor::Accent,
            ),
            SchedulerEvent::new("Lunch", 12 * 60, 13 * 60, SchedulerEventColor::Neutral),
            SchedulerEvent::new(
                "Client call",
                15 * 60 + 30,
                16 * 60 + 30,
                SchedulerEventColor::Success,
            ),
        ]
    });

    let overlapping_events = Signal::derive(|| {
        vec![
            SchedulerEvent::new(
                "Sprint planning",
                9 * 60,
                10 * 60 + 30,
                SchedulerEventColor::Primary,
            ),
            SchedulerEvent::new(
                "1:1 with manager",
                9 * 60 + 30,
                10 * 60,
                SchedulerEventColor::Warning,
            ),
            SchedulerEvent::new(
                "Interview panel",
                9 * 60 + 45,
                11 * 60,
                SchedulerEventColor::Error,
            ),
        ]
    });

    // Interactive scheduler: the consumer owns the events and applies the
    // move/resize requests the keyboard contract reports.
    let planned = RwSignal::new(vec![
        SchedulerEvent::new(
            "Intake review",
            9 * 60,
            10 * 60,
            SchedulerEventColor::Primary,
        ),
        SchedulerEvent::new(
            "Filing block",
            11 * 60,
            12 * 60 + 30,
            SchedulerEventColor::Info,
        ),
    ]);
    let selected = RwSignal::new(Option::<usize>::None);
    let last_activated = RwSignal::new(Option::<usize>::None);
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
    let apply_resize = move |(idx, delta): (usize, i32)| {
        planned.update(|evs| {
            if let Some(ev) = evs.get_mut(idx) {
                ev.end_min =
                    (ev.end_min as i32 + delta).clamp(ev.start_min as i32 + 15, 18 * 60) as u32;
            }
        });
    };

    view! {
        <ContentLayout
            title="Day Scheduler"
            description="A single-day vertical time grid: an hour gutter, hour/half-hour gridlines, absolutely-timed event blocks laid into side-by-side lanes when they overlap, and an optional 'now' line. Ported from d2d-ui's owner-drawn DayScheduler control."
        >
            <Section title="Business hours, non-overlapping events">
                <div class="w-full max-w-2xl rounded-box border border-base-300 bg-base-100 p-2">
                    <DayScheduler start_hour=8 end_hour=18 events=basic_events />
                </div>
            </Section>

            <Section title="Overlapping events (lane packing)">
                <div class="w-full max-w-2xl rounded-box border border-base-300 bg-base-100 p-2">
                    <DayScheduler start_hour=9 end_hour=12 events=overlapping_events />
                </div>
            </Section>

            <Section title="12-hour labels with a 'now' line">
                <div class="w-full max-w-2xl rounded-box border border-base-300 bg-base-100 p-2">
                    <DayScheduler
                        start_hour=7
                        end_hour=19
                        events=basic_events
                        hour_format=HourFormat::Twelve
                        now_min=Some(11 * 60 + 20)
                        now_label="Now"
                    />
                </div>
            </Section>

            <Section title="Custom hour labels (localised gutter)">
                <div class="w-full max-w-2xl rounded-box border border-base-300 bg-base-100 p-2">
                    <DayScheduler
                        start_hour=8
                        end_hour=18
                        events=basic_events
                        hour_label=Callback::new(|hour: u32| format!("{hour} h"))
                    />
                </div>
            </Section>

            <Section title="Interactive events (activation, selection, keyboard move/resize)">
                <p class="text-sm opacity-70 mb-2">
                    "Supply " <code>"on_event_activate"</code> " / " <code>"selected_event"</code>
                    " / " <code>"on_event_move"</code> " / " <code>"on_event_resize"</code>
                    " and event blocks become focusable buttons: click or "
                    <kbd class="kbd kbd-xs">"Enter"</kbd> " activates and selects, "
                    <kbd class="kbd kbd-xs">"↑"</kbd> "/" <kbd class="kbd kbd-xs">"↓"</kbd>
                    " requests a 15-minute move, " <kbd class="kbd kbd-xs">"Shift"</kbd>
                    "+arrows resize. The scheduler never mutates events — this page owns them "
                    "and applies (and clamps) each request."
                </p>
                <div class="mb-2 flex gap-4 text-sm">
                    <span>
                        "Selected: "
                        <code data-testid="sched-selected">
                            {move || {
                                selected
                                    .get()
                                    .map(|i| i.to_string())
                                    .unwrap_or_else(|| "(none)".to_string())
                            }}
                        </code>
                    </span>
                    <span>
                        "Last activated: "
                        <code data-testid="sched-activated">
                            {move || {
                                last_activated
                                    .get()
                                    .map(|i| i.to_string())
                                    .unwrap_or_else(|| "(none)".to_string())
                            }}
                        </code>
                    </span>
                    <span>
                        "First event: "
                        <code data-testid="sched-first-times">
                            {move || {
                                planned.with(|evs| {
                                    evs.first()
                                        .map(|e| format!("{}-{}", e.start_min, e.end_min))
                                        .unwrap_or_default()
                                })
                            }}
                        </code>
                    </span>
                </div>
                <div
                    class="w-full max-w-2xl rounded-box border border-base-300 bg-base-100 p-2"
                    id="interactive-scheduler"
                >
                    <DayScheduler
                        start_hour=8
                        end_hour=18
                        events=Signal::derive(move || planned.get())
                        selected_event=selected
                        on_event_activate=Callback::new(move |idx: usize| {
                            last_activated.set(Some(idx));
                        })
                        on_event_move=Callback::new(apply_move)
                        on_event_resize=Callback::new(apply_resize)
                    />
                </div>
            </Section>

            <Section title="Compact height override">
                <div class="w-full max-w-2xl rounded-box border border-base-300 bg-base-100 p-2">
                    <DayScheduler
                        start_hour=8
                        end_hour=18
                        events=basic_events
                        height_px=360.0
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
