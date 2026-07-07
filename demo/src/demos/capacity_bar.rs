use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn CapacityBarDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Capacity Bar"
            description="A horizontal load bar with a capacity-threshold marker: a value fill, a cap-line tick at the threshold, a distinct-colored overflow band once the value exceeds the cap, and an optional translucent ghost/preview fill."
        >
            <Section title="Under capacity">
                <div class="w-96">
                    <CapacityBar value=6.0 cap=10.0 />
                </div>
            </Section>

            <Section title="Over capacity">
                <div class="w-96">
                    <CapacityBar value=12.0 cap=10.0 />
                </div>
            </Section>

            <Section title="Ghost / preview fill">
                <div class="w-96">
                    <CapacityBar value=6.0 cap=10.0 ghost=8.0 />
                </div>
            </Section>

            <Section title="Colors" row=true>
                <div class="flex w-96 flex-col gap-3">
                    <CapacityBar value=4.0 cap=10.0 color=CapacityBarColor::Success />
                    <CapacityBar value=7.0 cap=10.0 color=CapacityBarColor::Warning />
                    <CapacityBar value=9.0 cap=10.0 color=CapacityBarColor::Info />
                    <CapacityBar
                        value=13.0
                        cap=10.0
                        color=CapacityBarColor::Success
                        over_color=CapacityBarColor::Warning
                    />
                </div>
            </Section>

            <Section title="Explicit scale max">
                <div class="w-96">
                    <CapacityBar value=12.0 cap=10.0 max=16.0 />
                </div>
            </Section>
        </ContentLayout>
    }
}
