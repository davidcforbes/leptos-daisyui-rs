use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;

/// A cyan dot-wave brand texture, standing in for the PNG a real app would
/// ship. Inlined as a data URI so the demo needs no binary asset; `#` is
/// percent-encoded (`%23`) because a raw `#` would start a URL fragment.
const DOT_WAVE_TEXTURE: &str = "data:image/svg+xml,\
%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='40'%20height='40'%3E\
%3Ccircle%20cx='8'%20cy='10'%20r='1.5'%20fill='%2322d3ee'%20fill-opacity='0.45'/%3E\
%3Ccircle%20cx='28'%20cy='26'%20r='1.5'%20fill='%2322d3ee'%20fill-opacity='0.30'/%3E\
%3C/svg%3E";

#[component]
pub fn AppShellDemo() -> impl IntoView {
    let section = RwSignal::new(Some("home".to_string()));

    let sidebar_visible = RwSignal::new(true);
    let status_bar_visible = RwSignal::new(true);
    let show_labels = RwSignal::new(true);

    view! {
        <ContentLayout
            title="AppShell"
            description="A 3-panel admin layout with icon navigation, side panel, and content area"
        >
            <Section title="Basic AppShell" col=true>
                <p class="text-sm text-base-content/70 mb-4">
                    "A full 3-panel layout with icon nav strip, secondary side panel, and main content area. Click icons to switch sections. The rail also shows a brand header block ("
                    <code>"AppShellHeader"</code>
                    "), a badge count on Users ("<code>"badge"</code>" prop), and Settings pinned to the bottom via "
                    <code>"AppShellIconNavGroup"</code>
                    "'s "<code>"pinned"</code>" prop."
                </p>
                <div class="h-96 bg-base-200 rounded-lg overflow-hidden border border-base-300">
                    <AppShell active_section=section>
                        <AppShellIconNav class="w-16 border-r border-base-content/10">
                            <AppShellHeader title="Acme">
                                <Icon icon=icondata::AiThunderboltFilled />
                            </AppShellHeader>
                            <AppShellIconNavItem value="home" class="[&.active]:bg-base-100 hover:bg-base-100/50 py-3">
                                <Icon icon=icondata::AiHomeFilled />
                                <span class="text-[10px]">"Home"</span>
                            </AppShellIconNavItem>
                            <AppShellIconNavItem
                                value="users"
                                class="[&.active]:bg-base-100 hover:bg-base-100/50 py-3"
                                badge=Some(3u32)
                            >
                                <Icon icon=icondata::AiUserOutlined />
                                <span class="text-[10px]">"Users"</span>
                            </AppShellIconNavItem>
                            <AppShellIconNavItem value="reports" class="[&.active]:bg-base-100 hover:bg-base-100/50 py-3">
                                <Icon icon=icondata::BsBarChartFill />
                                <span class="text-[10px]">"Reports"</span>
                            </AppShellIconNavItem>
                            <AppShellIconNavGroup pinned=true>
                                <AppShellIconNavItem value="settings" class="[&.active]:bg-base-100 hover:bg-base-100/50 py-3">
                                    <Icon icon=icondata::AiSettingOutlined />
                                    <span class="text-[10px]">"Settings"</span>
                                </AppShellIconNavItem>
                            </AppShellIconNavGroup>
                        </AppShellIconNav>

                        <AppShellSidePanel class="w-70 border-r border-base-content/10 p-3">
                            <Show when=move || section.get() == Some("home".to_string())>
                                <h3 class="text-sm font-semibold mb-2">"Dashboard"</h3>
                                <Menu direction=MenuDirection::Vertical class="w-full">
                                    <MenuItem>"Overview"</MenuItem>
                                    <MenuItem>"Activity"</MenuItem>
                                    <MenuItem>"Notifications"</MenuItem>
                                </Menu>
                            </Show>
                            <Show when=move || section.get() == Some("users".to_string())>
                                <h3 class="text-sm font-semibold mb-2">"Users"</h3>
                                <Menu direction=MenuDirection::Vertical class="w-full">
                                    <MenuItem>"All Users"</MenuItem>
                                    <MenuItem>"Teams"</MenuItem>
                                    <MenuItem>"Roles"</MenuItem>
                                    <MenuItem>"Invitations"</MenuItem>
                                </Menu>
                            </Show>
                            <Show when=move || section.get() == Some("settings".to_string())>
                                <h3 class="text-sm font-semibold mb-2">"Settings"</h3>
                                <Menu direction=MenuDirection::Vertical class="w-full">
                                    <MenuItem>"General"</MenuItem>
                                    <MenuItem>"Security"</MenuItem>
                                    <MenuItem>"Billing"</MenuItem>
                                    <MenuItem>"API Keys"</MenuItem>
                                    <MenuItem>"Webhooks"</MenuItem>
                                </Menu>
                            </Show>
                            <Show when=move || section.get() == Some("reports".to_string())>
                                <h3 class="text-sm font-semibold mb-2">"Reports"</h3>
                                <Menu direction=MenuDirection::Vertical class="w-full">
                                    <MenuItem>"Analytics"</MenuItem>
                                    <MenuItem>"Usage"</MenuItem>
                                    <MenuItem>"Export"</MenuItem>
                                </Menu>
                            </Show>
                        </AppShellSidePanel>

                        <AppShellContent class="p-6">
                            <div class="space-y-4">
                                <h2 class="text-xl font-bold">
                                    {move || {
                                        let s = section.get().unwrap_or_else(|| "home".to_string());
                                        let mut chars = s.chars();
                                        match chars.next() {
                                            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                                            None => String::new(),
                                        }
                                    }}
                                </h2>
                                <p class="text-base-content/70">"Select a section from the icon nav to see the side panel update."</p>
                                <div class="grid grid-cols-2 gap-4">
                                    <div class="card bg-base-100 shadow-sm">
                                        <div class="card-body p-4">
                                            <h3 class="card-title text-sm">"Active Section"</h3>
                                            <p class="text-2xl font-bold text-primary">
                                                {move || section.get().unwrap_or_else(|| "none".to_string())}
                                            </p>
                                        </div>
                                    </div>
                                    <div class="card bg-base-100 shadow-sm">
                                        <div class="card-body p-4">
                                            <h3 class="card-title text-sm">"Panel Width"</h3>
                                            <p class="text-2xl font-bold text-secondary">"w-70"</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </AppShellContent>
                    </AppShell>
                </div>
            </Section>

            <Section title="Sidebar Collapse & Status Bar" col=true>
                <p class="text-sm text-base-content/70 mb-4">
                    "Toggle the side panel via "<code>"AppShellSidePanel"</code>"'s "<code>"visible"</code>
                    " signal -- the content area reflows to fill the freed width. Pass "
                    <code>"AppShellStatusBar"</code>" via "<code>"AppShell"</code>"'s "
                    <code>"status_bar"</code>" prop to pin it below the main row; "
                    <code>"AppShell"</code>" handles the layout, with the status bar's own "
                    <code>"show"</code>" signal for the show/toggle."
                </p>
                <div class="flex gap-2 mb-3">
                    <Button on:click=move |_| sidebar_visible.update(|v| *v = !*v)>
                        {move || if sidebar_visible.get() { "Collapse Sidebar" } else { "Expand Sidebar" }}
                    </Button>
                    <Button on:click=move |_| status_bar_visible.update(|v| *v = !*v)>
                        {move || if status_bar_visible.get() { "Hide Status Bar" } else { "Show Status Bar" }}
                    </Button>
                </div>
                <div class="h-72 bg-base-200 rounded-lg overflow-hidden border border-base-300">
                    <AppShell
                        status_bar=Box::new(move || {
                            view! {
                                <AppShellStatusBar show=Signal::derive(move || status_bar_visible.get())>
                                    <span class="status status-success"></span>
                                    "Ready"
                                </AppShellStatusBar>
                            }
                                .into_any()
                        })
                    >
                        <AppShellIconNav class="w-16 border-r border-base-content/10">
                            <AppShellIconNavItem value="home" class="py-3">
                                <Icon icon=icondata::AiHomeFilled />
                            </AppShellIconNavItem>
                        </AppShellIconNav>
                        <AppShellSidePanel
                            width="w-70"
                            visible=Signal::derive(move || sidebar_visible.get())
                            class="border-r border-base-content/10 p-3"
                        >
                            <h3 class="text-sm font-semibold">"Side Panel"</h3>
                            <p class="text-xs text-base-content/70">"Collapses via visible=false"</p>
                        </AppShellSidePanel>
                        <AppShellContent class="p-6">
                            <p class="text-base-content/70">
                                "Main content reflows to fill the sidebar's space when it's collapsed."
                            </p>
                        </AppShellContent>
                    </AppShell>
                </div>
            </Section>

            <Section title="Labels Toggle" col=true>
                <p class="text-sm text-base-content/70 mb-4">
                    "Set "<code>"show_labels=false"</code>" on "<code>"AppShell"</code>
                    " to collapse the icon nav from icon+label to icon-only. Each item's text label is passed via the "
                    <code>"label"</code>" prop instead of being composed into "<code>"children"</code>"."
                </p>
                <div class="mb-3">
                    <Button on:click=move |_| show_labels.update(|v| *v = !*v)>
                        {move || if show_labels.get() { "Hide Labels" } else { "Show Labels" }}
                    </Button>
                </div>
                <div class="h-56 bg-base-200 rounded-lg overflow-hidden border border-base-300">
                    <AppShell show_labels=Signal::derive(move || show_labels.get())>
                        <AppShellIconNav class="w-20 border-r border-base-content/10">
                            <AppShellIconNavItem value="home" label="Home" class="py-3">
                                <Icon icon=icondata::AiHomeFilled />
                            </AppShellIconNavItem>
                            <AppShellIconNavItem value="settings" label="Settings" class="py-3">
                                <Icon icon=icondata::AiSettingOutlined />
                            </AppShellIconNavItem>
                        </AppShellIconNav>
                        <AppShellContent class="p-6">
                            <p class="text-base-content/70">
                                "The rail above narrows to icon-only when labels are hidden -- the item buttons still carry an aria-label from the label prop."
                            </p>
                        </AppShellContent>
                    </AppShell>
                </div>
            </Section>

            <Section title="Branded Rail (background image + gradient)" col=true>
                <p class="text-sm text-base-content/70 mb-4">
                    "The rail's background stacks three layers, back to front: "<code>"bg_color"</code>
                    ", an optional "<code>"bg_gradient"</code>", and an optional "<code>"bg_image"</code>
                    " texture. They paint beneath the nav items, so hover and active states (and count badges) stay legible on top of the texture. "
                    "Set the base colour via "<code>"bg_color"</code>" rather than a "<code>"bg-[#0b1e3a]"</code>
                    " class -- the rail already carries "<code>"bg-base-300"</code>
                    ", and between two utility classes the sheet order decides the winner; an inline colour always wins. "
                    "The dot-wave below is a tiling pattern, so it pairs "<code>"bg_repeat=\"repeat\""</code>
                    " with "<code>"bg_size=\"auto\""</code>"."
                </p>
                <div class="h-56 bg-base-200 rounded-lg overflow-hidden border border-base-300">
                    <AppShell>
                        <AppShellIconNav
                            class="w-20 text-white"
                            bg_color="#0b1e3a"
                            bg_gradient="linear-gradient(180deg, rgba(255,255,255,0.08), rgba(0,0,0,0.35))"
                            bg_image=DOT_WAVE_TEXTURE
                            bg_repeat="repeat"
                            bg_size="auto"
                        >
                            <AppShellIconNavItem
                                value="home"
                                label="Home"
                                class="py-3 [&.active]:bg-white/15 hover:bg-white/10"
                            >
                                <Icon icon=icondata::AiHomeFilled />
                            </AppShellIconNavItem>
                            <AppShellIconNavItem
                                value="alerts"
                                label="Alerts"
                                badge=Some(7)
                                class="py-3 [&.active]:bg-white/15 hover:bg-white/10"
                            >
                                <Icon icon=icondata::AiBellFilled />
                            </AppShellIconNavItem>
                            <AppShellIconNavGroup pinned=true>
                                <AppShellIconNavItem
                                    value="settings"
                                    label="Settings"
                                    class="py-3 [&.active]:bg-white/15 hover:bg-white/10"
                                >
                                    <Icon icon=icondata::AiSettingOutlined />
                                </AppShellIconNavItem>
                            </AppShellIconNavGroup>
                        </AppShellIconNav>
                        <AppShellContent class="p-6">
                            <p class="text-base-content/70">
                                "Hover the rail's items: the texture never sits over an item's hover/active background or its count badge."
                            </p>
                        </AppShellContent>
                    </AppShell>
                </div>
            </Section>

            <Section title="Manual Mode" col=true>
                <p class="text-sm text-base-content/70 mb-4">
                    "With `manual=true`, the active state is controlled externally rather than by AppShell context."
                </p>
                <Alert color=AlertColor::Info>
                    <Icon icon=icondata::AiInfoCircleOutlined />
                    "Use manual mode when you need full control over which items appear active, for example when integrating with a router."
                </Alert>
            </Section>

            <Section title="Code Example" col=true>
                <div class="mockup-code">
                    <pre data-prefix="1"><code>"<AppShell active_section=section>"</code></pre>
                    <pre data-prefix="2"><code>"  <AppShellIconNav class=\"w-16\">"</code></pre>
                    <pre data-prefix="3"><code>"    <AppShellHeader title=\"Acme\"><Icon .../></AppShellHeader>"</code></pre>
                    <pre data-prefix="4"><code>"    <AppShellIconNavItem value=\"home\">"</code></pre>
                    <pre data-prefix="5"><code>"      <Icon icon=icondata::AiHomeFilled />"</code></pre>
                    <pre data-prefix="6"><code>"    </AppShellIconNavItem>"</code></pre>
                    <pre data-prefix="7"><code>"    <AppShellIconNavItem value=\"users\" badge=Some(3)>"</code></pre>
                    <pre data-prefix="8"><code>"      <Icon icon=icondata::AiUserOutlined />"</code></pre>
                    <pre data-prefix="9"><code>"    </AppShellIconNavItem>"</code></pre>
                    <pre data-prefix="10"><code>"    <AppShellIconNavGroup pinned=true>"</code></pre>
                    <pre data-prefix="11"><code>"      <AppShellIconNavItem value=\"settings\">...</AppShellIconNavItem>"</code></pre>
                    <pre data-prefix="12"><code>"    </AppShellIconNavGroup>"</code></pre>
                    <pre data-prefix="13"><code>"  </AppShellIconNav>"</code></pre>
                    <pre data-prefix="14"><code>"  <AppShellSidePanel width=\"w-70\" visible=sidebar_visible>"</code></pre>
                    <pre data-prefix="15"><code>"    // Conditional content per section"</code></pre>
                    <pre data-prefix="16"><code>"  </AppShellSidePanel>"</code></pre>
                    <pre data-prefix="17"><code>"  <AppShellContent class=\"p-6\">"</code></pre>
                    <pre data-prefix="18"><code>"    <Outlet />"</code></pre>
                    <pre data-prefix="19"><code>"  </AppShellContent>"</code></pre>
                    <pre data-prefix="20"><code>"</AppShell>"</code></pre>
                </div>
            </Section>

            <Section title="Context Access" col=true>
                <p class="text-sm text-base-content/70 mb-4">
                    "Access the active section from any descendant component:"
                </p>
                <div class="mockup-code">
                    <pre data-prefix="1"><code>"let ctx = AppShellContext::expect_context();"</code></pre>
                    <pre data-prefix="2"><code>"let current = ctx.active_section.get();"</code></pre>
                    <pre data-prefix="3"><code>"ctx.active_section.set(Some(\"home\".into()));"</code></pre>
                </div>
            </Section>
        </ContentLayout>
    }
}
