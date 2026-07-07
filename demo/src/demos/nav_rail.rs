use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;

#[component]
pub fn NavRailDemo() -> impl IntoView {
    let active = RwSignal::new(Some("home".to_string()));
    let (manual_active, set_manual_active) = signal(0usize);

    view! {
        <ContentLayout
            title="NavRail"
            description="A vertical icon navigation rail with a selected pill, left-edge accent indicator, hover highlighting, and a bottom-pinned group"
        >
            <Section title="Basic NavRail" col=true>
                <p class="text-sm text-base-content/70 mb-4">
                    "Click an icon to select it. The active item shows a filled pill and a left-edge accent bar; Settings is pinned to the bottom via NavRailGroup's `pinned` prop."
                </p>
                <div class="h-96 flex bg-base-200 rounded-lg overflow-hidden border border-base-300">
                    <NavRail active=active>
                        <NavRailItem value="home" label="Home">
                            <Icon icon=icondata::AiHomeFilled />
                        </NavRailItem>
                        <NavRailItem value="search" label="Search">
                            <Icon icon=icondata::AiSearchOutlined />
                        </NavRailItem>
                        <NavRailItem value="messages" label="Messages">
                            <Icon icon=icondata::AiMessageFilled />
                        </NavRailItem>
                        <NavRailGroup pinned=true>
                            <NavRailItem value="settings" label="Settings">
                                <Icon icon=icondata::AiSettingOutlined />
                            </NavRailItem>
                        </NavRailGroup>
                    </NavRail>
                    <div class="flex-1 p-6">
                        <h3 class="text-sm font-semibold mb-2">"Active Item"</h3>
                        <p class="text-2xl font-bold text-primary">
                            {move || active.get().unwrap_or_else(|| "none".to_string())}
                        </p>
                    </div>
                </div>
            </Section>

            <Section title="Manual Mode" col=true>
                <p class="text-sm text-base-content/70 mb-4">
                    "With `manual=true` on NavRail, each NavRailItem's own `active` prop controls its selected state instead of the shared context -- useful when integrating with an external router or index-based state."
                </p>
                <div class="h-72 flex bg-base-200 rounded-lg overflow-hidden border border-base-300">
                    <NavRail manual=true>
                        <NavRailItem
                            label="Dashboard"
                            active=Signal::derive(move || manual_active.get() == 0)
                            on:click=move |_| set_manual_active.set(0)
                        >
                            <Icon icon=icondata::AiHomeFilled />
                        </NavRailItem>
                        <NavRailItem
                            label="Reports"
                            active=Signal::derive(move || manual_active.get() == 1)
                            on:click=move |_| set_manual_active.set(1)
                        >
                            <Icon icon=icondata::BsBarChartFill />
                        </NavRailItem>
                        <NavRailGroup pinned=true>
                            <NavRailItem
                                label="Settings"
                                active=Signal::derive(move || manual_active.get() == 2)
                                on:click=move |_| set_manual_active.set(2)
                            >
                                <Icon icon=icondata::AiSettingOutlined />
                            </NavRailItem>
                        </NavRailGroup>
                    </NavRail>
                    <div class="flex-1 p-6">
                        <h3 class="text-sm font-semibold mb-2">"Selected Index"</h3>
                        <p class="text-2xl font-bold text-secondary">{move || manual_active.get()}</p>
                    </div>
                </div>
            </Section>

            <Section title="Relationship to AppShellIconNav" col=true>
                <Alert color=AlertColor::Info>
                    <Icon icon=icondata::AiInfoCircleOutlined />
                    "AppShellIconNav is the icon strip built into the 3-panel AppShell layout and requires an AppShell ancestor. NavRail is the standalone equivalent -- use it anywhere you need a rail on its own, with a left-edge accent indicator and a bottom-pinned NavRailGroup that AppShellIconNav doesn't provide."
                </Alert>
            </Section>

            <Section title="Code Example" col=true>
                <div class="mockup-code">
                    <pre data-prefix="1"><code>"<NavRail active=active>"</code></pre>
                    <pre data-prefix="2"><code>"  <NavRailItem value=\"home\" label=\"Home\">"</code></pre>
                    <pre data-prefix="3"><code>"    <Icon icon=icondata::AiHomeFilled />"</code></pre>
                    <pre data-prefix="4"><code>"  </NavRailItem>"</code></pre>
                    <pre data-prefix="5"><code>"  <NavRailGroup pinned=true>"</code></pre>
                    <pre data-prefix="6"><code>"    <NavRailItem value=\"settings\" label=\"Settings\">"</code></pre>
                    <pre data-prefix="7"><code>"      <Icon icon=icondata::AiSettingOutlined />"</code></pre>
                    <pre data-prefix="8"><code>"    </NavRailItem>"</code></pre>
                    <pre data-prefix="9"><code>"  </NavRailGroup>"</code></pre>
                    <pre data-prefix="10"><code>"</NavRail>"</code></pre>
                </div>
            </Section>
        </ContentLayout>
    }
}
