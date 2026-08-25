use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn GaugeDemo() -> impl IntoView {
    let (cpu, set_cpu) = signal(62.0_f64);

    view! {
        <ContentLayout
            title="Gauge"
            description="An open-arc dial gauge with budget bands: a ~240-degree track, warn/error zones painted at threshold fractions, a value arc that escalates into the zone tone it has entered, and a tabular-nums readout with unit and sub-caption."
        >
            <Section title="Server cluster" row=true>
                <div class="flex flex-wrap gap-6">
                    <Gauge
                        value=62.0
                        max=100.0
                        unit="%"
                        caption="CPU"
                        warn_from=0.7
                        error_from=0.9
                        class="w-40"
                    />
                    <Gauge
                        value=83.0
                        max=100.0
                        unit="%"
                        caption="Memory"
                        warn_from=0.7
                        error_from=0.9
                        class="w-40"
                    />
                    <Gauge
                        value=96.0
                        max=100.0
                        unit="%"
                        caption="Disk"
                        warn_from=0.7
                        error_from=0.9
                        class="w-40"
                    />
                    <Gauge
                        value=1.8
                        max=10.0
                        unit="Gbps"
                        caption="Network"
                        warn_from=0.8
                        class="w-40"
                    />
                </div>
            </Section>

            <Section title="Without bands">
                <Gauge value=42.0 max=100.0 unit="%" caption="Utilization" class="w-40" />
            </Section>

            <Section title="Host display string">
                <Gauge
                    value=7.42
                    max=16.0
                    unit="GB"
                    caption="Heap"
                    display="7.4"
                    warn_from=0.75
                    error_from=0.9
                    class="w-40"
                />
            </Section>

            <Section title="Reactive value">
                <div class="flex w-96 flex-col gap-4">
                    <Gauge
                        value=cpu
                        max=100.0
                        unit="%"
                        caption="CPU"
                        warn_from=0.7
                        error_from=0.9
                        class="w-40"
                    />
                    <input
                        type="range"
                        class="range range-sm"
                        min="0"
                        max="100"
                        prop:value=move || format!("{:.0}", cpu.get())
                        on:input=move |ev| {
                            set_cpu.set(event_target_value(&ev).parse().unwrap_or(0.0));
                        }
                        aria-label="CPU percent"
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
