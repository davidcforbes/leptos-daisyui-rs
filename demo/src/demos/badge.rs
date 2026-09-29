use crate::core::{ContentLayout, Section};
use ldui_design::fixtures::FixtureSection;
use ldui_design::fixtures::badge::{self as fixtures, BadgeExample};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// One shared-fixture section (ldui-rz43): the title and every badge come
/// from `ldui_design::fixtures::badge`, the list 4iiz-kit's gallery renders
/// too. Name no variant and no label here, or the two renderers drift
/// (`tests/demo_shared_fixtures.rs`).
#[component]
fn BadgeFixtureSection(section: FixtureSection<BadgeExample>) -> impl IntoView {
    view! {
        <Section row=true title=section.title>
            {section
                .examples
                .iter()
                .map(|example| {
                    view! {
                        <Badge
                            color=example.color.clone()
                            style=example.style.clone()
                            size=example.size.clone()
                        >
                            {example.label}
                        </Badge>
                    }
                })
                .collect_view()}
        </Section>
    }
}

#[component]
pub fn BadgeDemo() -> impl IntoView {
    let (count, set_count) = signal(3);
    let (online, set_online) = signal(true);

    view! {
        <ContentLayout
            title="Badge"
            description="Badges are used to inform the user of the status of specific data"
        >
            <BadgeFixtureSection section=fixtures::COLORS />
            <BadgeFixtureSection section=fixtures::SIZES />
            <BadgeFixtureSection section=fixtures::STYLES />

            <Section title="Reactive Counter">
                <div class="flex items-center gap-4">
                    <div class="flex items-center gap-2">
                        "Messages " <Badge color=BadgeColor::Error>{move || count.get()}</Badge>
                    </div>
                    <div class="flex gap-2">
                        <Button size=ButtonSize::Sm on:click=move |_| set_count.update(|c| *c += 1)>
                            "Add Message"
                        </Button>
                        <Button
                            size=ButtonSize::Sm
                            color=ButtonColor::Neutral
                            on:click=move |_| set_count.set(0)
                        >
                            "Clear"
                        </Button>
                    </div>
                </div>
            </Section>

            <Section title="Status Indicator">
                <div class="flex items-center gap-4">
                    <div class="flex items-center gap-2">
                        "Server Status"
                        <Badge color=Signal::derive(move || {
                            if online.get() { BadgeColor::Success } else { BadgeColor::Error }
                        })>{move || if online.get() { "Online" } else { "Offline" }}</Badge>
                    </div>
                    <Button size=ButtonSize::Sm on:click=move |_| set_online.update(|s| *s = !*s)>
                        "Toggle Status"
                    </Button>
                </div>
            </Section>

            <Section title="Usage Examples">
                <div class="space-y-3">
                    <div class="text-lg">
                        "Inbox " <Badge color=BadgeColor::Secondary>"3"</Badge>
                    </div>
                    <div class="flex items-center gap-2">
                        "Notifications " <Badge color=BadgeColor::Error class="w-3 h-3 p-0">
                            " "
                        </Badge>
                    </div>
                    <div class="flex gap-2">
                        <Button class="relative">
                            "Profile"
                            <Badge
                                color=BadgeColor::Warning
                                size=BadgeSize::Sm
                                class="absolute -top-2 -right-2"
                            >
                                "NEW"
                            </Badge>
                        </Button>
                    </div>
                </div>
            </Section>
        </ContentLayout>
    }
}
