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

    view! {
        <ContentLayout
            title="Week View"
            description="A Mon-Sun week calendar: seven day-columns as vertical time grids, day headers (today highlighted), an hour gutter, an all-day strip, absolutely time-positioned event blocks, and an amber 'now' line. Ported from d2d-ui's owner-drawn WeekView control, reusing DayScheduler's overlap-lane algorithm per day-column."
        >
            <Section title="Business hours, today highlighted with a now-line">
                <div class="w-full max-w-5xl">
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
        </ContentLayout>
    }
}
