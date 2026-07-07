use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn SlaChipDemo() -> impl IntoView {
    // Ticks once a second; drives every chip on this page. `base_ms` is
    // captured once at mount so the example deadlines stay put while the
    // "remaining" label keeps counting down live.
    let now_ms = use_sla_now(1_000);
    let base_ms = js_sys::Date::now() as i64;
    const MIN: i64 = 60_000;
    const HOUR: i64 = 60 * MIN;

    view! {
        <ContentLayout
            title="SLA Chip"
            description="A live SLA-countdown chip: a colored deadline indicator that turns from green to amber to red as a deadline approaches and passes, or reads a neutral 'No SLA' when no deadline is set."
        >
            <Section title="Tones" row=true>
                <div class="flex flex-wrap items-center gap-3">
                    <SlaChip now_ms=now_ms deadline_ms=Some(base_ms + 5 * HOUR) />
                    <SlaChip now_ms=now_ms deadline_ms=Some(base_ms + 30 * MIN) />
                    <SlaChip now_ms=now_ms deadline_ms=Some(base_ms - 45 * MIN) />
                    <SlaChip now_ms=now_ms />
                </div>
            </Section>

            <Section title="Stale (frozen feed)">
                <div class="flex flex-wrap items-center gap-3">
                    <SlaChip now_ms=now_ms deadline_ms=Some(base_ms - 45 * MIN) stale=true />
                </div>
            </Section>

            <Section title="Enriched: leading icon + border, large size">
                <div class="flex flex-wrap items-center gap-3">
                    <SlaChip
                        now_ms=now_ms
                        deadline_ms=Some(base_ms + 5 * HOUR)
                        show_icon=true
                        big=true
                    />
                    <SlaChip
                        now_ms=now_ms
                        deadline_ms=Some(base_ms + 30 * MIN)
                        show_icon=true
                        big=true
                    />
                    <SlaChip
                        now_ms=now_ms
                        deadline_ms=Some(base_ms - 45 * MIN)
                        show_icon=true
                        big=true
                    />
                </div>
            </Section>

            <Section title="Custom threshold">
                <div class="flex flex-wrap items-center gap-3">
                    // 5h remaining is "green" against the default 2h threshold,
                    // but "amber" once the approach window is widened to 6h.
                    <SlaChip now_ms=now_ms deadline_ms=Some(base_ms + 5 * HOUR) />
                    <SlaChip
                        now_ms=now_ms
                        deadline_ms=Some(base_ms + 5 * HOUR)
                        threshold_ms=6 * HOUR
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
