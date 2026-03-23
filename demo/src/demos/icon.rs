use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn IconDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Icon"
            description="Icon component with Lucide icon support for displaying SVG icons"
        >
            <Section title="Icon Sizes">
                <div class="flex gap-6 items-end">
                    <div class="flex flex-col items-center gap-2">
                        <Icon name="circle".to_string() size=IconSize::XSmall />
                        <span class="text-xs">"XSmall"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <Icon name="circle".to_string() size=IconSize::Small />
                        <span class="text-xs">"Small"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <Icon name="circle".to_string() size=IconSize::Medium />
                        <span class="text-xs">"Medium"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <Icon name="circle".to_string() size=IconSize::Large />
                        <span class="text-xs">"Large"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <Icon name="circle".to_string() size=IconSize::XLarge />
                        <span class="text-xs">"XLarge"</span>
                    </div>
                </div>
            </Section>

            <Section title="Setup Instructions" col=true>
                <div class="alert alert-info">
                    <div>
                        <h4 class="font-bold">"Lucide Icons Setup Required"</h4>
                        <p class="text-sm">
                            "The Icon component renders " <code class="bg-base-300 px-1 rounded">"<i data-lucide=\"name\">"</code>
                            " elements for use with the Lucide icon library. "
                            "Add the following to your " <code class="bg-base-300 px-1 rounded">"index.html"</code> ":"
                        </p>
                        <pre class="bg-base-300 p-2 rounded mt-2 text-xs overflow-x-auto">
                            {"<script src=\"https://unpkg.com/lucide@latest\"></script>\n<script>lucide.createIcons();</script>"}
                        </pre>
                        <p class="text-sm mt-2">
                            "For SPA frameworks like Leptos, call "
                            <code class="bg-base-300 px-1 rounded">"lucide.createIcons()"</code>
                            " after each route change to initialize new icon elements."
                        </p>
                    </div>
                </div>
            </Section>

            <Section title="Component API" col=true>
                <div class="mockup-code text-sm">
                    <pre data-prefix=">"><code>{"<Icon name=\"heart\" size=IconSize::Large color=\"text-error\" />"}</code></pre>
                    <pre data-prefix=">"><code>{"<Icon name=\"star\" size=IconSize::Medium />"}</code></pre>
                    <pre data-prefix=">"><code>{"<Icon name=\"user\" size=IconSize::Small color=\"text-primary\" />"}</code></pre>
                </div>
            </Section>

            <Section title="Available Sizes" col=true>
                <div class="overflow-x-auto">
                    <table class="table table-zebra">
                        <thead>
                            <tr>
                                <th>"Size"</th>
                                <th>"CSS Class"</th>
                                <th>"Dimensions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr><td><code>"IconSize::XSmall"</code></td><td><code>"w-4 h-4"</code></td><td>"16px"</td></tr>
                            <tr><td><code>"IconSize::Small"</code></td><td><code>"w-5 h-5"</code></td><td>"20px"</td></tr>
                            <tr><td><code>"IconSize::Medium"</code></td><td><code>"w-6 h-6"</code></td><td>"24px"</td></tr>
                            <tr><td><code>"IconSize::Large"</code></td><td><code>"w-8 h-8"</code></td><td>"32px"</td></tr>
                            <tr><td><code>"IconSize::XLarge"</code></td><td><code>"w-12 h-12"</code></td><td>"48px"</td></tr>
                        </tbody>
                    </table>
                </div>
            </Section>
        </ContentLayout>
    }
}
