use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn CollapseDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Collapse"
            description="Collapse is used for showing and hiding content"
        >
            <Section title="Basic Variations" col=true>
                <Collapse focus_open=true class="border border-base-300">
                    <CollapseTitle class="text-xl font-medium">
                        "Focus me to see content"
                    </CollapseTitle>
                    <CollapseContent>
                        <p>"tabindex=\"0\" attribute is necessary to make the div focusable"</p>
                    </CollapseContent>
                </Collapse>

                <Collapse focus_open=true class="border border-base-300">
                    <CollapseTitle class="text-xl font-medium">
                        "Click me to open content"
                    </CollapseTitle>
                    <CollapseContent>
                        <p>"The plus icon changes to minus when expanded"</p>
                    </CollapseContent>
                </Collapse>
            </Section>

            <Section title="Force States" col=true>
                <Collapse
                    force=CollapseForceModifier::Open
                    class="border border-base-300 bg-base-100 rounded-box"
                >
                    <CollapseTitle class="text-xl font-medium">"Always Open"</CollapseTitle>
                    <CollapseContent>
                        <p>"This collapse is always open"</p>
                    </CollapseContent>
                </Collapse>

                <Collapse
                    force=CollapseForceModifier::Close
                    class="border border-base-300 bg-base-100 rounded-box"
                >
                    <CollapseTitle class="text-xl font-medium">"Always Closed"</CollapseTitle>
                    <CollapseContent>
                        <p>"This collapse is always closed"</p>
                    </CollapseContent>
                </Collapse>
            </Section>

            // ldui-3k00: the checkbox toggle always carries an id, a name and
            // an accessible name. Browser proof: tests/collapse_naming_smoke.rs.
            <Section title="Toggle identity and accessible name (ldui-3k00)" col=true>
                <div id="collapse-naming-fixture" class="flex min-w-0 flex-col gap-4">
                <div data-testid="collapse-naming-titled">
                    <Collapse
                        id="collapse-naming-filters"
                        name="show_filters"
                        modifier=CollapseModifier::Arrow
                        class="border border-base-300 bg-base-100 rounded-box"
                    >
                        <CollapseTitle class="text-xl font-medium">"Filters"</CollapseTitle>
                        <CollapseContent>
                            <p>"Explicit id and name; the toggle is named by this title."</p>
                        </CollapseContent>
                    </Collapse>
                </div>
                <div data-testid="collapse-naming-minted">
                    <Collapse
                        modifier=CollapseModifier::Arrow
                        class="border border-base-300 bg-base-100 rounded-box"
                    >
                        <CollapseTitle class="text-xl font-medium">"Sort options"</CollapseTitle>
                        <CollapseContent>
                            <p>"No id supplied: a minted id doubles as the name."</p>
                        </CollapseContent>
                    </Collapse>
                </div>
                <div data-testid="collapse-naming-labelled">
                    <Collapse
                        id="collapse-naming-advanced"
                        aria_label="Show advanced options"
                        modifier=CollapseModifier::Plus
                        class="border border-base-300 bg-base-100 rounded-box"
                    >
                        <CollapseTitle class="text-xl font-medium">"Advanced"</CollapseTitle>
                        <CollapseContent>
                            <p>"An explicit aria_label replaces the title as the accessible name."</p>
                        </CollapseContent>
                    </Collapse>
                </div>
                </div>
            </Section>
        </ContentLayout>
    }
}
