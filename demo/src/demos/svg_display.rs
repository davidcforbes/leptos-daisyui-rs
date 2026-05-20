use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

const SAMPLE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100"><rect x="10" y="10" width="80" height="80" fill="#0078D4" rx="8"/><circle cx="150" cy="50" r="40" fill="#50E6FF"/><text x="100" y="55" text-anchor="middle" fill="white" font-size="14">SVG</text></svg>"##;

#[component]
pub fn SvgDisplayDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="SvgDisplay"
            description="Renders inline SVG content with configurable sizing and object-fit modes"
        >
            <Section title="Basic Inline SVG">
                <SvgDisplay
                    content=SAMPLE_SVG.to_string()
                    alt="Sample SVG with a blue rectangle and cyan circle"
                />
            </Section>

            <Section title="Fit: Contain" col=true>
                <p class="text-sm text-base-content/60">
                    "Scales the SVG to fit within its container while preserving aspect ratio."
                </p>
                <div class="w-64 h-32 border border-base-300 rounded-lg overflow-hidden">
                    <SvgDisplay
                        content=SAMPLE_SVG.to_string()
                        fit=SvgFit::Contain
                        alt="Contain fit example"
                    />
                </div>
            </Section>

            <Section title="Fit: Cover" col=true>
                <p class="text-sm text-base-content/60">
                    "Scales the SVG to cover the container, cropping as needed."
                </p>
                <div class="w-64 h-32 border border-base-300 rounded-lg overflow-hidden">
                    <SvgDisplay
                        content=SAMPLE_SVG.to_string()
                        fit=SvgFit::Cover
                        alt="Cover fit example"
                    />
                </div>
            </Section>

            <Section title="Fit: Fill" col=true>
                <p class="text-sm text-base-content/60">
                    "Stretches the SVG to fill the container exactly."
                </p>
                <div class="w-64 h-32 border border-base-300 rounded-lg overflow-hidden">
                    <SvgDisplay
                        content=SAMPLE_SVG.to_string()
                        fit=SvgFit::Fill
                        alt="Fill fit example"
                    />
                </div>
            </Section>

            <Section title="Constrained Max Width" col=true>
                <p class="text-sm text-base-content/60">
                    "The SVG is constrained to a maximum width of 150 pixels."
                </p>
                <SvgDisplay
                    content=SAMPLE_SVG.to_string()
                    max_width=Some(150.0f32)
                    alt="Max-width constrained example"
                />
            </Section>
        </ContentLayout>
    }
}
