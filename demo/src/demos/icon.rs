use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
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
                    "The recommended approach uses the " <code class="bg-base-300 text-base-content px-1 rounded">"leptos_icons"</code>
                    " crate which provides inline SVG icons from multiple icon sets."
                </p>
                <div class="flex gap-4 items-center">
                    <span class="w-6 h-6"><LeptosIcon icon=icondata::AiHomeFilled /></span>
                    <span class="w-6 h-6 text-error"><LeptosIcon icon=icondata::AiHeartFilled /></span>
                    <span class="w-6 h-6 text-warning"><LeptosIcon icon=icondata::AiStarFilled /></span>
                    <span class="w-6 h-6 text-success"><LeptosIcon icon=icondata::AiCheckCircleFilled /></span>
                    <span class="w-6 h-6 text-info"><LeptosIcon icon=icondata::AiInfoCircleFilled /></span>
                    <span class="w-6 h-6"><LeptosIcon icon=icondata::AiSettingFilled /></span>
                    <span class="w-6 h-6"><LeptosIcon icon=icondata::AiSearchOutlined /></span>
                    <span class="w-6 h-6 text-primary"><LeptosIcon icon=icondata::AiBellFilled /></span>
                </div>
            </Section>

            <Section title="Icon Sizes" col=true>
                <div class="flex gap-6 items-end">
                    <div class="flex flex-col items-center gap-2">
                        <span class="w-4 h-4"><LeptosIcon icon=icondata::AiStarFilled /></span>
                        <span class="text-xs">"16px"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <span class="w-5 h-5"><LeptosIcon icon=icondata::AiStarFilled /></span>
                        <span class="text-xs">"20px"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <span class="w-6 h-6"><LeptosIcon icon=icondata::AiStarFilled /></span>
                        <span class="text-xs">"24px"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <span class="w-8 h-8"><LeptosIcon icon=icondata::AiStarFilled /></span>
                        <span class="text-xs">"32px"</span>
                    </div>
                    <div class="flex flex-col items-center gap-2">
                        <span class="w-12 h-12"><LeptosIcon icon=icondata::AiStarFilled /></span>
                        <span class="text-xs">"48px"</span>
                    </div>
                </div>
            </Section>

            <Section title="Colored Icons" col=true>
                <div class="flex gap-4 items-center">
                    <span class="w-8 h-8 text-error"><LeptosIcon icon=icondata::AiHeartFilled /></span>
                    <span class="w-8 h-8 text-warning"><LeptosIcon icon=icondata::AiStarFilled /></span>
                    <span class="w-8 h-8 text-success"><LeptosIcon icon=icondata::AiCheckCircleFilled /></span>
                    <span class="w-8 h-8 text-info"><LeptosIcon icon=icondata::AiInfoCircleFilled /></span>
                    <span class="w-8 h-8 text-primary"><LeptosIcon icon=icondata::AiThunderboltFilled /></span>
                    <span class="w-8 h-8 text-secondary"><LeptosIcon icon=icondata::AiFireFilled /></span>
                    <span class="w-8 h-8 text-accent"><LeptosIcon icon=icondata::AiWarningFilled /></span>
                </div>
            </Section>

            <Section title="Icons in Buttons" col=true>
                <div class="flex gap-2 flex-wrap">
                    <button class="btn btn-primary">
                        <span class="w-5 h-5"><LeptosIcon icon=icondata::AiHeartFilled /></span>
                        "Like"
                    </button>
                    <button class="btn btn-secondary">
                        <span class="w-5 h-5"><LeptosIcon icon=icondata::AiShareAltOutlined /></span>
                        "Share"
                    </button>
                    <button class="btn btn-accent">
                        <span class="w-5 h-5"><LeptosIcon icon=icondata::AiDownloadOutlined /></span>
                        "Download"
                    </button>
                    <button class="btn btn-ghost btn-circle">
                        <span class="w-5 h-5"><LeptosIcon icon=icondata::AiSearchOutlined /></span>
                    </button>
                    <button class="btn btn-ghost btn-circle">
                        <span class="w-5 h-5"><LeptosIcon icon=icondata::AiBellFilled /></span>
                    </button>
                </div>
            </Section>

            <Section title="Library Icon Component" col=true>
                <div class="alert alert-info">
                    <div>
                        <h4 class="font-bold">"Lucide Icon Component"</h4>
                        <p class="text-sm">
                            "This library also includes an " <code class="bg-base-300 text-base-content px-1 rounded">"Icon"</code>
                            " component that renders Lucide icons via " <code class="bg-base-300 text-base-content px-1 rounded">"data-lucide"</code>
                            " attributes. This requires adding the Lucide JS library to your project."
                        </p>
                    </div>
                </div>
                <div class="mockup-code text-sm">
                    <pre data-prefix="1"><code>{"// Using the library's Icon component (requires Lucide JS)"}</code></pre>
                    <pre data-prefix="2"><code>{"<Icon name=\"heart\" size=IconSize::Large color=\"text-error\" />"}</code></pre>
                    <pre data-prefix="3"><code>{""}</code></pre>
                    <pre data-prefix="4"><code>{"// Using leptos_icons (recommended, no JS required)"}</code></pre>
                    <pre data-prefix="5"><code>{"use leptos_icons::Icon;"}</code></pre>
                    <pre data-prefix="6"><code>{"<span class=\"w-8 h-8 text-error\"><Icon icon=icondata::AiHeartFilled /></span>"}</code></pre>
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
                                <td><span class="w-4 h-4 inline-block"><LeptosIcon icon=icondata::AiStarFilled /></span></td>
                            </tr>
                            <tr>
                                <td><code>"Small"</code></td>
                                <td><code>"w-5 h-5"</code></td>
                                <td>"20px"</td>
                                <td><span class="w-5 h-5 inline-block"><LeptosIcon icon=icondata::AiStarFilled /></span></td>
                            </tr>
                            <tr>
                                <td><code>"Medium"</code></td>
                                <td><code>"w-6 h-6"</code></td>
                                <td>"24px"</td>
                                <td><span class="w-6 h-6 inline-block"><LeptosIcon icon=icondata::AiStarFilled /></span></td>
                            </tr>
                            <tr>
                                <td><code>"Large"</code></td>
                                <td><code>"w-8 h-8"</code></td>
                                <td>"32px"</td>
                                <td><span class="w-8 h-8 inline-block"><LeptosIcon icon=icondata::AiStarFilled /></span></td>
                            </tr>
                            <tr>
                                <td><code>"XLarge"</code></td>
                                <td><code>"w-12 h-12"</code></td>
                                <td>"48px"</td>
                                <td><span class="w-12 h-12 inline-block"><LeptosIcon icon=icondata::AiStarFilled /></span></td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </Section>
        </ContentLayout>
    }
}
