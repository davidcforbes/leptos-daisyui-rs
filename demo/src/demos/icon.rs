use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon as LeptosIcon;

#[component]
pub fn IconDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Icon"
            description="Icon component for displaying icons with configurable size and color"
        >
            <Section title="Icons with leptos_icons" col=true>
                <p class="text-sm text-base-content/70">
                    "The recommended approach uses the " <code class="bg-base-300 px-1 rounded">"leptos_icons"</code>
                    " crate which provides inline SVG icons from multiple icon sets."
                </p>
                <div class="flex gap-4 items-center">
                    <LeptosIcon icon=icondata::AiHomeFilled class="w-6 h-6" />
                    <LeptosIcon icon=icondata::AiHeartFilled class="w-6 h-6 text-error" />
                    <LeptosIcon icon=icondata::AiStarFilled class="w-6 h-6 text-warning" />
                    <LeptosIcon icon=icondata::AiCheckCircleFilled class="w-6 h-6 text-success" />
                    <LeptosIcon icon=icondata::AiInfoCircleFilled class="w-6 h-6 text-info" />
                    <LeptosIcon icon=icondata::AiSettingFilled class="w-6 h-6" />
                    <LeptosIcon icon=icondata::AiSearchOutlined class="w-6 h-6" />
                    <LeptosIcon icon=icondata::AiBellFilled class="w-6 h-6 text-primary" />
                </div>
            </Section>

            <Section title="Icon Sizes" col=true>
                <div class="flex gap-6 items-end">
                    <div class="flex flex-col items-center gap-2">
                        <LeptosIcon icon=icondata::AiStarFilled class="w-4 h-4" />
                        <span class="text-xs">"16px"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <LeptosIcon icon=icondata::AiStarFilled class="w-5 h-5" />
                        <span class="text-xs">"20px"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <LeptosIcon icon=icondata::AiStarFilled class="w-6 h-6" />
                        <span class="text-xs">"24px"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <LeptosIcon icon=icondata::AiStarFilled class="w-8 h-8" />
                        <span class="text-xs">"32px"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <LeptosIcon icon=icondata::AiStarFilled class="w-12 h-12" />
                        <span class="text-xs">"48px"</span>
                    </div>
                </div>
            </Section>

            <Section title="Colored Icons" col=true>
                <div class="flex gap-4 items-center">
                    <LeptosIcon icon=icondata::AiHeartFilled class="w-8 h-8 text-error" />
                    <LeptosIcon icon=icondata::AiStarFilled class="w-8 h-8 text-warning" />
                    <LeptosIcon icon=icondata::AiCheckCircleFilled class="w-8 h-8 text-success" />
                    <LeptosIcon icon=icondata::AiInfoCircleFilled class="w-8 h-8 text-info" />
                    <LeptosIcon icon=icondata::AiThunderboltFilled class="w-8 h-8 text-primary" />
                    <LeptosIcon icon=icondata::AiFireFilled class="w-8 h-8 text-secondary" />
                    <LeptosIcon icon=icondata::AiWarningFilled class="w-8 h-8 text-accent" />
                </div>
            </Section>

            <Section title="Icons in Buttons" col=true>
                <div class="flex gap-2 flex-wrap">
                    <button class="btn btn-primary">
                        <LeptosIcon icon=icondata::AiHeartFilled class="w-5 h-5" />
                        "Like"
                    </button>
                    <button class="btn btn-secondary">
                        <LeptosIcon icon=icondata::AiShareAltOutlined class="w-5 h-5" />
                        "Share"
                    </button>
                    <button class="btn btn-accent">
                        <LeptosIcon icon=icondata::AiDownloadOutlined class="w-5 h-5" />
                        "Download"
                    </button>
                    <button class="btn btn-ghost btn-circle">
                        <LeptosIcon icon=icondata::AiSearchOutlined class="w-5 h-5" />
                    </button>
                    <button class="btn btn-ghost btn-circle">
                        <LeptosIcon icon=icondata::AiBellFilled class="w-5 h-5" />
                    </button>
                </div>
            </Section>

            <Section title="Library Icon Component" col=true>
                <div class="alert alert-info">
                    <div>
                        <h4 class="font-bold">"Lucide Icon Component"</h4>
                        <p class="text-sm">
                            "This library also includes an " <code class="bg-base-300 px-1 rounded">"Icon"</code>
                            " component that renders Lucide icons via " <code class="bg-base-300 px-1 rounded">"data-lucide"</code>
                            " attributes. This requires adding the Lucide JS library to your project."
                        </p>
                    </div>
                </div>
                <div class="mockup-code text-sm">
                    <pre data-prefix="1"><code>{"// Using the library's Icon component (requires Lucide JS)"}</code></pre>
                    <pre data-prefix="2"><code>{"<Icon name=\"heart\" size=IconSize::Large color=\"text-error\" />"}</code></pre>
                    <pre data-prefix="3"><code>{""}</code></pre>
                    <pre data-prefix="4"><code>{"// Using leptos_icons (recommended, no JS required)"}</code></pre>
                    <pre data-prefix="5"><code>{"<Icon icon=icondata::AiHeartFilled class=\"w-8 h-8 text-error\" />"}</code></pre>
                </div>
            </Section>

            <Section title="Icon Size Reference" col=true>
                <div class="overflow-x-auto">
                    <table class="table table-zebra">
                        <thead>
                            <tr>
                                <th>"IconSize"</th>
                                <th>"CSS Class"</th>
                                <th>"Pixels"</th>
                                <th>"Preview"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td><code>"XSmall"</code></td>
                                <td><code>"w-4 h-4"</code></td>
                                <td>"16px"</td>
                                <td><LeptosIcon icon=icondata::AiStarFilled class="w-4 h-4" /></td>
                            </tr>
                            <tr>
                                <td><code>"Small"</code></td>
                                <td><code>"w-5 h-5"</code></td>
                                <td>"20px"</td>
                                <td><LeptosIcon icon=icondata::AiStarFilled class="w-5 h-5" /></td>
                            </tr>
                            <tr>
                                <td><code>"Medium"</code></td>
                                <td><code>"w-6 h-6"</code></td>
                                <td>"24px"</td>
                                <td><LeptosIcon icon=icondata::AiStarFilled class="w-6 h-6" /></td>
                            </tr>
                            <tr>
                                <td><code>"Large"</code></td>
                                <td><code>"w-8 h-8"</code></td>
                                <td>"32px"</td>
                                <td><LeptosIcon icon=icondata::AiStarFilled class="w-8 h-8" /></td>
                            </tr>
                            <tr>
                                <td><code>"XLarge"</code></td>
                                <td><code>"w-12 h-12"</code></td>
                                <td>"48px"</td>
                                <td><LeptosIcon icon=icondata::AiStarFilled class="w-12 h-12" /></td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </Section>
        </ContentLayout>
    }
}
