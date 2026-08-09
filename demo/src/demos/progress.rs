use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn ProgressDemo() -> impl IntoView {
    // Drives the reactive `value` prop section below.
    let pct = RwSignal::new(35.0);

    view! {
        <ContentLayout
            title="Progress"
            description="Progress bars show the progress of a task or show the loading state"
        >

            <Section title="Colors" col=true>
                <Progress value=70.0 max=100.0 class="w-56" />
                <Progress color=ProgressColor::Primary value=70.0 max=100.0 class="w-56" />
                <Progress color=ProgressColor::Secondary value=70.0 max=100.0 class="w-56" />
                <Progress color=ProgressColor::Accent value=70.0 max=100.0 class="w-56" />
                <Progress color=ProgressColor::Info value=70.0 max=100.0 class="w-56" />
                <Progress color=ProgressColor::Success value=70.0 max=100.0 class="w-56" />
                <Progress color=ProgressColor::Warning value=70.0 max=100.0 class="w-56" />
                <Progress color=ProgressColor::Error value=70.0 max=100.0 class="w-56" />
            </Section>

            <Section title="Indeterminate Progress">
                <p class="text-sm opacity-70 mb-2">
                    "Leaving `value` unset omits the attribute entirely, which is what makes daisyUI animate the sliding stripe. It is NOT the same as value=0.0, shown below it as a determinate but empty bar."
                </p>
                <div class="flex flex-col gap-2">
                    <Progress class="w-56" />
                    <Progress value=0.0 max=100.0 class="w-56" />
                </div>
            </Section>

            <Section title="Reactive value / max" col=true>
                <p class="text-sm opacity-70">
                    "value and max are Signal-backed props (ldui-c1s), so a determinate bar can be driven straight from application state. Values outside 0..=max are clamped rather than overflowing."
                </p>
                <Progress
                    color=ProgressColor::Primary
                    value=pct
                    max=100.0
                    class="w-56"
                />
                <div class="flex items-center gap-2">
                    <Button
                        size=ButtonSize::Sm
                        on:click=move |_| pct.update(|p| *p = (*p - 10.0).max(-20.0))
                    >
                        "-10"
                    </Button>
                    <Button
                        size=ButtonSize::Sm
                        on:click=move |_| pct.update(|p| *p = (*p + 10.0).min(120.0))
                    >
                        "+10"
                    </Button>
                    <span class="text-sm opacity-70">{move || format!("value = {}", pct.get())}</span>
                </div>

                <p class="text-sm opacity-70">
                    "max defaults to HTML's own default of 1.0, so a fractional value needs no max at all:"
                </p>
                <Progress color=ProgressColor::Success value=0.7 class="w-56" />
            </Section>

        </ContentLayout>
    }
}
