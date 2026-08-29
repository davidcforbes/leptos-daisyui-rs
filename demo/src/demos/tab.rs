use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn TabDemo() -> impl IntoView {
    let active_tab = RwSignal::new("tab-1".to_owned());
    let (bordered_tab, set_bordered_tab) = signal(0);
    let (boxed_tab, set_boxed_tab) = signal(0);
    let fixture_selected = RwSignal::new("alpha".to_owned());
    let fixture_localized = RwSignal::new(false);
    let fixture_beta_visible = RwSignal::new(true);
    let fixture_orientation = RwSignal::new(TabOrientation::Horizontal);

    // PixelProof oracle (ldui-49w.1): expose the Basic Tabs selection at
    // window.__APP_DEBUG__.state().state["tab.active"]. No-op in normal mode.
    let select_tab = Callback::new(move |key: String| {
        let index = match key.as_str() {
            "tab-1" => 0,
            "tab-2" => 1,
            "tab-3" => 2,
            _ => return,
        };
        active_tab.set(key);
        crate::debug_state::set("tab.active", index);
    });
    let select_fixture = Callback::new(move |key: String| {
        fixture_selected.set(key.clone());
        crate::debug_state::set("tab.fixture.selected", key);
    });

    view! {
        <ContentLayout
            title="Tab"
            description="Tabs are used to organize content into different sections"
        >
            <Section title="Basic Tabs">
                <TabSet
                    id="basic-tabs"
                    label="Basic tabs"
                    selected_key=active_tab
                    on_select=select_tab
                >
                    <Tabs variant=TabVariant::Lift>
                        <Tab tab_key="tab-1">"Tab 1"</Tab>
                        <Tab tab_key="tab-2">"Tab 2"</Tab>
                        <Tab tab_key="tab-3">"Tab 3"</Tab>
                    </Tabs>
                    <TabPanel tab_key="tab-1" class="bg-base-200 p-4 rounded-box mt-4">
                        <p>"Content for Tab 1"</p>
                    </TabPanel>
                    <TabPanel tab_key="tab-2" class="bg-base-200 p-4 rounded-box mt-4">
                        <p>"Content for Tab 2"</p>
                    </TabPanel>
                    <TabPanel tab_key="tab-3" class="bg-base-200 p-4 rounded-box mt-4">
                        <p>"Content for Tab 3"</p>
                    </TabPanel>
                </TabSet>
            </Section>

            <Section title="Tab Variants">

                <div>
                    <h3 class="text-sm font-medium mb-2">"Bordered Tabs"</h3>
                    <Tabs variant=TabVariant::Border>
                        <Tab
                            active=Signal::derive(move || bordered_tab.get() == 0)
                            on:click=move |_| set_bordered_tab.set(0)
                        >
                            "First"
                        </Tab>
                        <Tab
                            active=Signal::derive(move || bordered_tab.get() == 1)
                            on:click=move |_| set_bordered_tab.set(1)
                        >
                            "Second"
                        </Tab>
                        <Tab
                            active=Signal::derive(move || bordered_tab.get() == 2)
                            on:click=move |_| set_bordered_tab.set(2)
                        >
                            "Third"
                        </Tab>
                    </Tabs>
                </div>

                <div>
                    <h3 class="text-sm font-medium mb-2">"Boxed Tabs"</h3>
                    <Tabs variant=TabVariant::Boxed>
                        <Tab
                            active=Signal::derive(move || boxed_tab.get() == 0)
                            on:click=move |_| set_boxed_tab.set(0)
                        >
                            "Home"
                        </Tab>
                        <Tab
                            active=Signal::derive(move || boxed_tab.get() == 1)
                            on:click=move |_| set_boxed_tab.set(1)
                        >
                            "About"
                        </Tab>
                        <Tab
                            active=Signal::derive(move || boxed_tab.get() == 2)
                            on:click=move |_| set_boxed_tab.set(2)
                        >
                            "Contact"
                        </Tab>
                    </Tabs>
                </div>
            </Section>

            <Section title="Tab Sizes">
                <div>
                    <h3 class="text-sm font-medium mb-2">"Extra Small"</h3>
                    <Tabs size=TabSize::Xs>
                        <Tab active=RwSignal::new(true)>"XS Tab 1"</Tab>
                        <Tab>"XS Tab 2"</Tab>
                    </Tabs>
                </div>
                <div>
                    <h3 class="text-sm font-medium mb-2">"Small"</h3>
                    <Tabs size=TabSize::Sm>
                        <Tab active=RwSignal::new(true)>"SM Tab 1"</Tab>
                        <Tab>"SM Tab 2"</Tab>
                    </Tabs>
                </div>
                <div>
                    <h3 class="text-sm font-medium mb-2">"Large"</h3>
                    <Tabs size=TabSize::Lg>
                        <Tab active=RwSignal::new(true)>"LG Tab 1"</Tab>
                        <Tab>"LG Tab 2"</Tab>
                    </Tabs>
                </div>
            </Section>

            <Section title="Tab with Disabled">
                <Tabs>
                    <Tab active=RwSignal::new(true)>"Active"</Tab>
                    <Tab>"Normal"</Tab>
                    <Tab disabled=RwSignal::new(true)>"Disabled"</Tab>
                </Tabs>
            </Section>

            <Section title="Controlled accessibility fixture">
                <div class="space-y-3" data-testid="controlled-tab-fixture">
                    <div class="flex flex-wrap gap-2">
                        <Button
                            attr:data-testid="tab-select-beta"
                            on_click=Callback::new(move |_| fixture_selected.set("beta".to_owned()))
                        >
                            "Select Beta externally"
                        </Button>
                        <Button
                            attr:data-testid="tab-remove-beta"
                            on_click=Callback::new(move |_| fixture_beta_visible.set(false))
                        >
                            "Remove Beta"
                        </Button>
                        <Button
                            attr:data-testid="tab-toggle-locale"
                            on_click=Callback::new(move |_| fixture_localized.update(|value| *value = !*value))
                        >
                            "Toggle labels"
                        </Button>
                        <Button
                            attr:data-testid="tab-toggle-orientation"
                            on_click=Callback::new(move |_| {
                                fixture_orientation.update(|orientation| {
                                    *orientation = match *orientation {
                                        TabOrientation::Horizontal => TabOrientation::Vertical,
                                        TabOrientation::Vertical => TabOrientation::Horizontal,
                                    };
                                })
                            })
                        >
                            "Toggle orientation"
                        </Button>
                    </div>
                    <TabSet
                        id="controlled-tabs-fixture"
                        label=Signal::derive(move || {
                            if fixture_localized.get() {
                                "Flujo de trabajo".to_owned()
                            } else {
                                "Workflow".to_owned()
                            }
                        })
                        selected_key=fixture_selected
                        on_select=select_fixture
                        orientation=fixture_orientation
                    >
                        <Tabs variant=TabVariant::Border>
                            <Tab tab_key="alpha">
                                {move || if fixture_localized.get() { "Alfa" } else { "Alpha" }}
                            </Tab>
                            <Show when=move || fixture_beta_visible.get()>
                                <Tab tab_key="beta">
                                    {move || if fixture_localized.get() { "Beta localizada" } else { "Beta" }}
                                </Tab>
                            </Show>
                            <Tab tab_key="gamma" disabled=true>"Gamma disabled"</Tab>
                            <Tab tab_key="delta">"Delta with a deliberately long label"</Tab>
                            <Tab tab_key="epsilon">"Epsilon workflow details"</Tab>
                            <Tab tab_key="zeta">"Zeta audit history"</Tab>
                        </Tabs>
                        <TabPanel tab_key="alpha"><p>"Alpha panel"</p></TabPanel>
                        <Show when=move || fixture_beta_visible.get()>
                            <TabPanel tab_key="beta"><p>"Beta panel"</p></TabPanel>
                        </Show>
                        <TabPanel tab_key="gamma"><p>"Disabled panel"</p></TabPanel>
                        <TabPanel tab_key="delta"><p>"Delta panel"</p></TabPanel>
                        <TabPanel tab_key="epsilon"><p>"Epsilon panel"</p></TabPanel>
                        <TabPanel tab_key="zeta"><p>"Zeta panel"</p></TabPanel>
                    </TabSet>
                </div>
            </Section>
        </ContentLayout>
    }
}
