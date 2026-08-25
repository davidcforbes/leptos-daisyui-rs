use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

fn etl_phases() -> Vec<String> {
    vec![
        "capture".to_string(),
        "reconcile".to_string(),
        "apply".to_string(),
    ]
}

#[component]
pub fn PhaseProgressDemo() -> impl IntoView {
    let (pct, set_pct) = signal(40_u8);

    view! {
        <ContentLayout
            title="Phase Progress"
            description="A phase-run instrument: one equal segment per phase, where completed phases are solid, the current phase is partially filled to a percent, and future phases are muted. The current segment can carry a failed tone when the run stopped inside it."
        >
            <Section title="Mid-run">
                <div class="w-96">
                    <PhaseProgress phases=etl_phases() current=1_usize pct=40_u8 />
                </div>
            </Section>

            <Section title="First phase starting">
                <div class="w-96">
                    <PhaseProgress phases=etl_phases() current=0_usize pct=5_u8 />
                </div>
            </Section>

            <Section title="Run complete">
                <div class="w-96">
                    <PhaseProgress phases=etl_phases() current=3_usize pct=0_u8 />
                </div>
            </Section>

            <Section title="Failed in the current phase">
                <div class="w-96">
                    <PhaseProgress phases=etl_phases() current=2_usize pct=15_u8 failed=true />
                </div>
            </Section>

            <Section title="Reactive percent">
                <div class="flex w-96 flex-col gap-4">
                    <PhaseProgress phases=etl_phases() current=1_usize pct=pct />
                    <input
                        type="range"
                        class="range range-sm"
                        min="0"
                        max="100"
                        prop:value=move || pct.get().to_string()
                        on:input=move |ev| {
                            set_pct.set(event_target_value(&ev).parse().unwrap_or(0));
                        }
                        aria-label="Current phase percent"
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
