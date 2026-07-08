use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn VerticalStepsDemo() -> impl IntoView {
    let (last_action, set_last_action) = signal(String::from("None"));

    let preflight = vec![
        VerticalStep::new(StepStatus::Ready, "PC", "Your computer is online"),
        VerticalStep::new(StepStatus::Ready, "Network", "Connected to the office VPN")
            .with_tech("10.20.4.117 via vpn-gw-03"),
        VerticalStep::new(StepStatus::NeedsYou, "Gateway", "Needs your sign-in")
            .with_tech("vpn-gw-03.internal:443")
            .with_action("Fix"),
        VerticalStep::new(
            StepStatus::Pending,
            "Database",
            "Checked once gateway clears",
        ),
    ];

    let statuses = vec![
        VerticalStep::new(
            StepStatus::Ready,
            "Ready",
            "Done / healthy — solid success dot",
        ),
        VerticalStep::new(
            StepStatus::Checking,
            "Checking",
            "In flight — pulsing accent dot; the rail below a Ready step animates a flow",
        ),
        VerticalStep::new(
            StepStatus::Pending,
            "Pending",
            "Not yet reached — hollow ring",
        ),
        VerticalStep::new(
            StepStatus::NeedsYou,
            "Needs You",
            "Blocked on the user — solid warning dot",
        )
        .with_action("Retry"),
        VerticalStep::new(StepStatus::Down, "Down", "Failed — solid error dot")
            .with_tech("exit code 1: connection refused")
            .with_action("Details"),
    ];

    let flowing = vec![
        VerticalStep::new(StepStatus::Ready, "Build", "Compiled in 42s"),
        VerticalStep::new(StepStatus::Ready, "Unit tests", "1505 passed"),
        VerticalStep::new(StepStatus::Checking, "Deploy", "Rolling out to staging..."),
        VerticalStep::new(StepStatus::Pending, "Smoke tests", "Runs after deploy"),
    ];

    view! {
        <ContentLayout
            title="Vertical Steps"
            description="Status-driven vertical connection path with rich per-step content — ported from d2d-ui's preflight view. For a simple presentational sequence, use the Steps component instead."
        >
            <Section title="Preflight Check (with action button)">
                <div class="alert alert-info mb-4 max-w-md">
                    <span>"Last action clicked: " <strong>{move || last_action.get()}</strong></span>
                </div>
                <div class="max-w-md">
                    <VerticalSteps
                        items=Signal::derive(move || preflight.clone())
                        on_action=Callback::new(move |i: usize| {
                            set_last_action.set(format!("step {i}"));
                        })
                    />
                </div>
            </Section>

            <Section title="All Statuses">
                <div class="max-w-md">
                    <VerticalSteps items=Signal::derive(move || statuses.clone()) />
                </div>
            </Section>

            <Section title="Animated Flow">
                <p class="text-sm opacity-70 mb-2">
                    "Rail segments below a Ready step are lit (accent) with a dash flowing downward. "
                    "The animation is defined in UiAnimationsPreamble and disabled under prefers-reduced-motion."
                </p>
                <div class="max-w-md">
                    <VerticalSteps items=Signal::derive(move || flowing.clone()) />
                </div>
            </Section>
        </ContentLayout>
    }
}
