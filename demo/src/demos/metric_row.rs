use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn MetricRowDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Metric Row"
            description="A compact label ... value key/value row for facts grids and detail panels, with an optional stacked layout, bold emphasis, color-tinted values, and a hairline bottom divider."
        >
            <Section title="Basic">
                <div class="w-72 rounded-box border border-base-300 p-4">
                    <MetricRow label="Case value" value="$1,200" />
                    <MetricRow label="Opened" value="2026-01-14" />
                    <MetricRow label="Owner" value="J. Smith" />
                </div>
            </Section>

            <Section title="Bold value">
                <div class="w-72 rounded-box border border-base-300 p-4">
                    <MetricRow label="Total due" value="$4,820.00" bold=true />
                </div>
            </Section>

            <Section title="With dividers">
                <div class="w-72 rounded-box border border-base-300 p-4">
                    <MetricRow label="Case value" value="$1,200" divider=true />
                    <MetricRow label="Fees" value="$150" divider=true />
                    <MetricRow label="Balance" value="$1,050" bold=true />
                </div>
            </Section>

            <Section title="Status-tinted values" row=true>
                <div class="w-72 rounded-box border border-base-300 p-4">
                    <MetricRow
                        label="Status"
                        value="Overdue"
                        value_color=MetricRowColor::Error
                        bold=true
                    />
                    <MetricRow
                        label="Status"
                        value="Paid"
                        value_color=MetricRowColor::Success
                        bold=true
                    />
                    <MetricRow
                        label="Status"
                        value="Pending"
                        value_color=MetricRowColor::Warning
                        bold=true
                    />
                </div>
            </Section>

            <Section title="Stacked layout">
                <div class="w-56 rounded-box border border-base-300 p-4">
                    <MetricRow label="Case value" value="$1,200" stacked=true />
                </div>
            </Section>
        </ContentLayout>
    }
}
