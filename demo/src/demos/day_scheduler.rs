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
