use crate::core::{ContentLayout, Section};
use ldui_design::fixtures::FixtureSection;
use ldui_design::fixtures::alert::{self as fixtures, AlertExample};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;

/// This page's status glyph for an alert colour.
fn status_icon(color: &AlertColor) -> icondata::Icon {
    match color {
        AlertColor::Success => icondata::AiCheckCircleFilled,
        AlertColor::Warning => icondata::AiWarningFilled,
        AlertColor::Error => icondata::AiCloseCircleFilled,
        AlertColor::Info | AlertColor::Default => icondata::AiInfoCircleFilled,
    }
}

/// One shared-fixture section (ldui-rz43): the title and every alert come
/// from `ldui_design::fixtures::alert`, the list 4iiz-kit's gallery renders
/// too. Name no variant and no message here, or the two renderers drift
/// (`tests/demo_shared_fixtures.rs`).
#[component]
fn AlertFixtureSection(section: FixtureSection<AlertExample>) -> impl IntoView {
    view! {
        <Section col=true title=section.title>
            {section
                .examples
                .iter()
                .map(|example| {
                    view! {
                        <Alert color=example.color.clone() style=example.style.clone()>
                            {example.icon.then(|| view! { <Icon icon=status_icon(&example.color) /> })}
                            <span>{example.message}</span>
                        </Alert>
                    }
                })
                .collect_view()}
        </Section>
    }
}

#[component]
pub fn AlertDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Alert"
            description="Alerts are used to display important messages to users"
        >
            <AlertFixtureSection section=fixtures::COLORS />
            <AlertFixtureSection section=fixtures::STYLES />

            <Section title="With Actions">
                <Alert color=AlertColor::Warning>
                    <Icon icon=icondata::AiWarningFilled />
                    <span>"New software update available"</span>
                    <div>
                        <button class="btn btn-sm">"Deny"</button>
                        <button class="btn btn-sm btn-primary">"Apply"</button>
                    </div>
                </Alert>
            </Section>
        </ContentLayout>
    }
}
