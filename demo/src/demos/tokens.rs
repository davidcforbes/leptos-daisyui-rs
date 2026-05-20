use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::tokens::{ui_animations_css, ui_tokens_css};

#[component]
pub fn TokensDemo() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-3xl font-bold">"Design tokens"</h1>
            <p class="text-base-content/70">
                "Shared with d2d-ui via the "<code>"ui-tokens"</code>" crate. The "
                <code>"UiTokensPreamble"</code>
                " component emits these as "<code>"--ld-*"</code>
                " CSS custom properties on "<code>":root"</code>"."
            </p>

            <h2 class="text-xl font-semibold">"Durations"</h2>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <DurationCard name="--ld-duration-fast" var_name="--ld-duration-fast" />
                <DurationCard name="--ld-duration-normal" var_name="--ld-duration-normal" />
                <DurationCard name="--ld-duration-slow" var_name="--ld-duration-slow" />
            </div>

            <h2 class="text-xl font-semibold">"Easings"</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <EasingCard name="linear" var_name="--ld-ease-linear" />
                <EasingCard name="standard" var_name="--ld-ease-standard" />
                <EasingCard name="decelerate" var_name="--ld-ease-decelerate" />
                <EasingCard name="accelerate" var_name="--ld-ease-accelerate" />
            </div>

            <h2 class="text-xl font-semibold">"Elevation tiers"</h2>
            <div class="grid grid-cols-1 md:grid-cols-5 gap-6 py-4">
                <ElevationSwatch tier="2" />
                <ElevationSwatch tier="4" />
                <ElevationSwatch tier="8" />
                <ElevationSwatch tier="16" />
                <ElevationSwatch tier="64" />
            </div>

            <h2 class="text-xl font-semibold">"Motion primitives"</h2>
            <p class="text-base-content/70 text-sm">
                "Utility classes from "<code>"UiAnimationsPreamble"</code>
                ". Hover and click the live samples below to feel the eased transitions."
            </p>
            <div class="flex flex-wrap gap-4 items-center py-4">
                <Button color=ButtonColor::Primary>"ld-eased ld-pressable"</Button>
                <Card elevate=true class="bg-base-100">
                    <CardBody class="p-4">
                        <span class="font-mono text-xs">"ld-elevated card"</span>
                    </CardBody>
                </Card>
            </div>

            <h2 class="text-xl font-semibold">"Generated CSS"</h2>
            <p class="text-base-content/70 text-sm">
                "Output of "<code>"ui_tokens_css()"</code>" and "
                <code>"ui_animations_css()"</code>"."
            </p>
            <pre class="bg-base-200 p-4 rounded-lg overflow-x-auto text-xs">
                <code>{ui_tokens_css()}{ui_animations_css()}</code>
            </pre>
        </div>
    }
}

#[component]
fn DurationCard(name: &'static str, var_name: &'static str) -> impl IntoView {
    view! {
        <Card class="bg-base-100 shadow">
            <CardBody>
                <h3 class="font-mono text-sm">{name}</h3>
                <p class="text-base-content/70 text-xs">"var("{var_name}")"</p>
            </CardBody>
        </Card>
    }
}

#[component]
fn EasingCard(name: &'static str, var_name: &'static str) -> impl IntoView {
    view! {
        <Card class="bg-base-100 shadow">
            <CardBody>
                <h3 class="font-mono text-sm">{name}</h3>
                <p class="text-base-content/70 text-xs">"var("{var_name}")"</p>
            </CardBody>
        </Card>
    }
}

#[component]
fn ElevationSwatch(tier: &'static str) -> impl IntoView {
    let style = format!("box-shadow: var(--ld-elevation-{}); ", tier);
    view! {
        <div class="flex flex-col items-center gap-2">
            <div
                class="w-24 h-24 bg-base-100 rounded-lg"
                style=style
            />
            <span class="font-mono text-xs text-base-content/70">"LEVEL_"{tier}</span>
        </div>
    }
}
