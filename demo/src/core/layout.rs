use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_icons::Icon;
use leptos_router::{components::Outlet, hooks::use_location};
use leptos_use::use_interval_fn;
use wasm_bindgen::prelude::*;

/// Read the non-standard, Chrome-only `performance.memory` **property**.
///
/// `performance.memory` is a plain object property, not a method — calling
/// it as a function (as a naive `#[wasm_bindgen(js_namespace = performance,
/// getter)] fn memory() -> JsValue` extern would, since `getter` only takes
/// effect on a `method`-style binding with a `this` receiver, not on a bare
/// namespace-qualified function) throws `TypeError: performance.memory is
/// not a function` in every browser, including Chrome. Firefox and Safari
/// don't implement the property at all. `js_sys::Reflect::get` reads it as
/// a property lookup and simply returns `undefined` (never throws) when the
/// property is absent, so this degrades silently everywhere.
fn read_performance_memory() -> Option<JsValue> {
    let performance = web_sys::window()?.performance()?;
    js_sys::Reflect::get(&performance, &JsValue::from_str("memory")).ok()
}

/// Layout component for the demos
#[component]
pub fn Layout() -> impl IntoView {
    let location = use_location();

    let selected = RwSignal::new(None);

    Effect::new(move || {
        let init_component_name = location
            .pathname
            .get_untracked()
            .strip_prefix("/components/")
            .map(|v| v.to_string());

        selected.set(init_component_name);
    });

    // Status bar state
    let (current_time, set_current_time) = signal(String::new());
    let (memory_usage, set_memory_usage) = signal(String::new());

    // PixelProof determinism (ldui-49w.1): under ?pp-freeze=1 the status bar
    // must not tick — a live clock changes every capture and the
    // performance.memory readout varies per run. Show fixed placeholders.
    let frozen = leptos_daisyui_rs::test_mode::is_test_mode();
    if frozen {
        set_current_time.set("00:00:00".to_string());
        set_memory_usage.set("frozen".to_string());
    }

    // Update time and memory every second
    let _ = use_interval_fn(
        move || {
            if frozen {
                return;
            }
            // Update time
            let date = js_sys::Date::new_0();
            let time_str = format!(
                "{:02}:{:02}:{:02}",
                date.get_hours(),
                date.get_minutes(),
                date.get_seconds()
            );
            set_current_time.set(time_str);

            // Update memory usage (Chrome only, graceful degradation)
            let memory_obj = read_performance_memory().unwrap_or(JsValue::UNDEFINED);
            if !memory_obj.is_undefined() && !memory_obj.is_null() {
                // Try to get usedJSHeapSize and jsHeapSizeLimit
                if let Ok(used) =
                    js_sys::Reflect::get(&memory_obj, &JsValue::from_str("usedJSHeapSize"))
                {
                    if let Some(used_bytes) = used.as_f64() {
                        let used_mb = used_bytes / 1_048_576.0;

                        // Also try to get heap limit
                        if let Ok(limit) =
                            js_sys::Reflect::get(&memory_obj, &JsValue::from_str("jsHeapSizeLimit"))
                        {
                            if let Some(limit_bytes) = limit.as_f64() {
                                let limit_mb = limit_bytes / 1_048_576.0;
                                set_memory_usage.set(format!("{:.1}/{:.1} MB", used_mb, limit_mb));
                            } else {
                                set_memory_usage.set(format!("{:.1} MB", used_mb));
                            }
                        } else {
                            set_memory_usage.set(format!("{:.1} MB", used_mb));
                        }
                    } else {
                        set_memory_usage.set("N/A".to_string());
                    }
                } else {
                    set_memory_usage.set("N/A".to_string());
                }
            } else {
                set_memory_usage.set("N/A".to_string());
            }
        },
        1000,
    );

    view! {
        <div class="h-screen w-screen bg-base-100 flex flex-col">
            <Navbar class="w-screen bg-base-200 border-b border-base-300 flex-none">
                <NavbarStart class="gap-4">
                    <Icon icon=icondata::CgComponents />
                    <h1 class="text-xl font-bold">"daisyUI + Leptos Showcase"</h1>
                </NavbarStart>
                <NavbarEnd class="gap-2">
                    <LinkButton
                        href="https://github.com/noshishiRust/leptos-daisyui-rs"
                        style=ButtonStyle::Ghost
                        shape=ButtonShape::Circle
                    >
                        <Icon icon=icondata::AiGithubFilled />
                    </LinkButton>
                </NavbarEnd>
            </Navbar>

            <div class="flex-1 flex overflow-hidden">
                // Sidebar
                <aside class="hidden lg:flex flex-col w-64 flex-none bg-base-200 border-r border-base-300 overflow-y-auto">
                    <div class="p-4">
                        <h2 class="text-lg font-semibold mb-4">"Components"</h2>
                        <Menu
                            selected=selected
                            direction=MenuDirection::Vertical
                            class="w-full"
                        >
                            {get_menu_categories()
                                .into_iter()
                                .map(|category| {
                                    view! {
                                        <MenuItem is_submenu=true>
                                            <MenuTitle>{category.title}</MenuTitle>
                                            <SubMenu>
                                                {category
                                                    .items
                                                    .into_iter()
                                                    .map(|item| {
                                                        view! {
                                                            <MenuItem href=item.href value=item.value>
                                                                {item.name}
                                                            </MenuItem>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </SubMenu>
                                        </MenuItem>
                                    }
                                })
                                .collect_view()}
                        </Menu>
                    </div>
                </aside>

                // Main content area
                <main class="flex-1 overflow-y-auto min-w-0">
                    <div class="p-6">
                        <Outlet />
                    </div>
                </main>
            </div>

            // Status footer
            <div class="w-screen bg-base-300 border-t border-base-content/10 px-4 py-1 flex-none">
                <div class="flex items-center text-xs text-base-content/70 font-mono">
                    // Left: Path
                    <span class="flex items-center gap-1">
                        <span class="w-3 h-3">
                            <Icon icon=icondata::AiFolderOutlined />
                        </span>
                        {move || location.pathname.get()}
                    </span>

                    // Center: Memory (with flex-1 to take remaining space and center content)
                    <div class="flex-1 flex justify-center">
                        <span class="flex items-center gap-1">
                            <span class="w-3 h-3">
                                <Icon icon=icondata::AiDatabaseOutlined />
                            </span>
                            "Memory: "
                            {move || memory_usage.get()}
                        </span>
                    </div>

                    // Right: Clock
                    <span class="flex items-center gap-1">
                        <span class="w-3 h-3">
                            <Icon icon=icondata::AiClockCircleOutlined />
                        </span>
                        {move || current_time.get()}
                    </span>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Debug)]
struct ComponentItem {
    name: &'static str,
    href: &'static str,
    value: &'static str,
}

#[derive(Clone, Debug)]
struct MenuCategory {
    title: &'static str,
    items: Vec<ComponentItem>,
}

fn get_menu_categories() -> Vec<MenuCategory> {
    vec![
        MenuCategory {
            title: "Action",
            items: vec![
                ComponentItem {
                    name: "Button",
                    href: "/components/button",
                    value: "button",
                },
                ComponentItem {
                    name: "Dropdown",
                    href: "/components/dropdown",
                    value: "dropdown",
                },
                ComponentItem {
                    name: "FAB",
                    href: "/components/fab",
                    value: "fab",
                },
                ComponentItem {
                    name: "Modal",
                    href: "/components/modal",
                    value: "modal",
                },
                ComponentItem {
                    name: "Swap",
                    href: "/components/swap",
                    value: "swap",
                },
                ComponentItem {
                    name: "Theme Controller",
                    href: "/components/theme_controller",
                    value: "theme_controller",
                },
                ComponentItem {
                    name: "SWR Cache",
                    href: "/components/swr",
                    value: "swr",
                },
                ComponentItem {
                    name: "Toolbar",
                    href: "/components/toolbar",
                    value: "toolbar",
                },
            ],
        },
        MenuCategory {
            title: "Data Display",
            items: vec![
                ComponentItem {
                    name: "Accordion",
                    href: "/components/accordion",
                    value: "accordion",
                },
                ComponentItem {
                    name: "Avatar",
                    href: "/components/avatar",
                    value: "avatar",
                },
                ComponentItem {
                    name: "Badge",
                    href: "/components/badge",
                    value: "badge",
                },
                ComponentItem {
                    name: "Card",
                    href: "/components/card",
                    value: "card",
                },
                ComponentItem {
                    name: "Carousel",
                    href: "/components/carousel",
                    value: "carousel",
                },
                ComponentItem {
                    name: "AiChat",
                    href: "/components/ai-chat",
                    value: "ai-chat",
                },
                ComponentItem {
                    name: "Chat",
                    href: "/components/chat",
                    value: "chat",
                },
                ComponentItem {
                    name: "Collapse",
                    href: "/components/collapse",
                    value: "collapse",
                },
                ComponentItem {
                    name: "ConfigProvider",
                    href: "/components/config-provider",
                    value: "config-provider",
                },
                ComponentItem {
                    name: "Countdown",
                    href: "/components/countdown",
                    value: "countdown",
                },
                ComponentItem {
                    name: "DataTable",
                    href: "/components/data-table",
                    value: "data-table",
                },
                ComponentItem {
                    name: "Day Scheduler (Calendar · Day)",
                    href: "/components/day-scheduler",
                    value: "day-scheduler",
                },
                ComponentItem {
                    name: "Week View (Calendar · Week)",
                    href: "/components/week-view",
                    value: "week-view",
                },
                ComponentItem {
                    name: "Diff",
                    href: "/components/diff",
                    value: "diff",
                },
                ComponentItem {
                    name: "Hover 3D",
                    href: "/components/hover_3d",
                    value: "hover_3d",
                },
                ComponentItem {
                    name: "Hover Gallery",
                    href: "/components/hover_gallery",
                    value: "hover_gallery",
                },
                ComponentItem {
                    name: "Icon",
                    href: "/components/icon",
                    value: "icon",
                },
                ComponentItem {
                    name: "Icon Tile",
                    href: "/components/icon_tile",
                    value: "icon_tile",
                },
                ComponentItem {
                    name: "Gantt",
                    href: "/components/gantt",
                    value: "gantt",
                },
                ComponentItem {
                    name: "Kanban",
                    href: "/components/kanban",
                    value: "kanban",
                },
                ComponentItem {
                    name: "Kbd",
                    href: "/components/kbd",
                    value: "kbd",
                },
                ComponentItem {
                    name: "List",
                    href: "/components/list",
                    value: "list",
                },
                ComponentItem {
                    name: "Metric Row",
                    href: "/components/metric_row",
                    value: "metric_row",
                },
                ComponentItem {
                    name: "Persona",
                    href: "/components/persona",
                    value: "persona",
                },
                ComponentItem {
                    name: "Sparkline",
                    href: "/components/sparkline",
                    value: "sparkline",
                },
                ComponentItem {
                    name: "Stats",
                    href: "/components/stats",
                    value: "stats",
                },
                ComponentItem {
                    name: "Status",
                    href: "/components/status",
                    value: "status",
                },
                ComponentItem {
                    name: "Table",
                    href: "/components/table",
                    value: "table",
                },
                ComponentItem {
                    name: "Tag",
                    href: "/components/tag",
                    value: "tag",
                },
                ComponentItem {
                    name: "Text Rotate",
                    href: "/components/text_rotate",
                    value: "text_rotate",
                },
                ComponentItem {
                    name: "Timeline",
                    href: "/components/timeline",
                    value: "timeline",
                },
                ComponentItem {
                    name: "Tree",
                    href: "/components/tree",
                    value: "tree",
                },
                ComponentItem {
                    name: "SVG Display",
                    href: "/components/svg_display",
                    value: "svg_display",
                },
                ComponentItem {
                    name: "Mermaid Display",
                    href: "/components/mermaid_display",
                    value: "mermaid_display",
                },
            ],
        },
        MenuCategory {
            title: "Navigation",
            items: vec![
                ComponentItem {
                    name: "Breadcrumbs",
                    href: "/components/breadcrumbs",
                    value: "breadcrumbs",
                },
                ComponentItem {
                    name: "Menu",
                    href: "/components/menu",
                    value: "menu",
                },
                ComponentItem {
                    name: "Login / Enrollment",
                    href: "/components/login_enrollment",
                    value: "login_enrollment",
                },
                ComponentItem {
                    name: "Navbar",
                    href: "/components/navbar",
                    value: "navbar",
                },
                ComponentItem {
                    name: "NavRail",
                    href: "/components/nav_rail",
                    value: "nav_rail",
                },
                ComponentItem {
                    name: "Pagination",
                    href: "/components/pagination",
                    value: "pagination",
                },
                ComponentItem {
                    name: "Steps",
                    href: "/components/steps",
                    value: "steps",
                },
                ComponentItem {
                    name: "Tab",
                    href: "/components/tab",
                    value: "tab",
                },
                ComponentItem {
                    name: "Vertical Steps",
                    href: "/components/vertical-steps",
                    value: "vertical-steps",
                },
            ],
        },
        MenuCategory {
            title: "Feedback",
            items: vec![
                ComponentItem {
                    name: "Alert",
                    href: "/components/alert",
                    value: "alert",
                },
                ComponentItem {
                    name: "Capacity Bar",
                    href: "/components/capacity_bar",
                    value: "capacity_bar",
                },
                ComponentItem {
                    name: "Empty State",
                    href: "/components/empty_state",
                    value: "empty_state",
                },
                ComponentItem {
                    name: "Loading",
                    href: "/components/loading",
                    value: "loading",
                },
                ComponentItem {
                    name: "Progress",
                    href: "/components/progress",
                    value: "progress",
                },
                ComponentItem {
                    name: "Radial Progress",
                    href: "/components/radial_progress",
                    value: "radial_progress",
                },
                ComponentItem {
                    name: "Skeleton",
                    href: "/components/skeleton",
                    value: "skeleton",
                },
                ComponentItem {
                    name: "SLA Chip",
                    href: "/components/sla_chip",
                    value: "sla_chip",
                },
                ComponentItem {
                    name: "Toast",
                    href: "/components/toast",
                    value: "toast",
                },
                ComponentItem {
                    name: "Tooltip",
                    href: "/components/tooltip",
                    value: "tooltip",
                },
            ],
        },
        MenuCategory {
            title: "Data Input",
            items: vec![
                ComponentItem {
                    name: "AutoComplete",
                    href: "/components/auto-complete",
                    value: "auto-complete",
                },
                ComponentItem {
                    name: "Calendar (Month · 3rd-party)",
                    href: "/components/calendar",
                    value: "calendar",
                },
                ComponentItem {
                    name: "Checkbox",
                    href: "/components/checkbox",
                    value: "checkbox",
                },
                ComponentItem {
                    name: "ColorPicker",
                    href: "/components/color-picker",
                    value: "color-picker",
                },
                ComponentItem {
                    name: "Fieldset",
                    href: "/components/fieldset",
                    value: "fieldset",
                },
                ComponentItem {
                    name: "File Input",
                    href: "/components/file_input",
                    value: "file_input",
                },
                ComponentItem {
                    name: "Filter",
                    href: "/components/filter",
                    value: "filter",
                },
                ComponentItem {
                    name: "Input",
                    href: "/components/input",
                    value: "input",
                },
                ComponentItem {
                    name: "Label",
                    href: "/components/label",
                    value: "label",
                },
                ComponentItem {
                    name: "Radio",
                    href: "/components/radio",
                    value: "radio",
                },
                ComponentItem {
                    name: "Range",
                    href: "/components/range",
                    value: "range",
                },
                ComponentItem {
                    name: "Rating",
                    href: "/components/rating",
                    value: "rating",
                },
                ComponentItem {
                    name: "Result List",
                    href: "/components/result-list",
                    value: "result-list",
                },
                ComponentItem {
                    name: "Select",
                    href: "/components/select",
                    value: "select",
                },
                ComponentItem {
                    name: "Textarea",
                    href: "/components/textarea",
                    value: "textarea",
                },
                ComponentItem {
                    name: "Toggle",
                    href: "/components/toggle",
                    value: "toggle",
                },
                ComponentItem {
                    name: "Validator",
                    href: "/components/validator",
                    value: "validator",
                },
            ],
        },
        MenuCategory {
            title: "Layout",
            items: vec![
                ComponentItem {
                    name: "AppShell",
                    href: "/components/app-shell",
                    value: "app-shell",
                },
                ComponentItem {
                    name: "Divider",
                    href: "/components/divider",
                    value: "divider",
                },
                ComponentItem {
                    name: "Dock",
                    href: "/components/dock",
                    value: "dock",
                },
                ComponentItem {
                    name: "Drawer",
                    href: "/components/drawer",
                    value: "drawer",
                },
                ComponentItem {
                    name: "Footer",
                    href: "/components/footer",
                    value: "footer",
                },
                ComponentItem {
                    name: "Hero",
                    href: "/components/hero",
                    value: "hero",
                },
                ComponentItem {
                    name: "Indicator",
                    href: "/components/indicator",
                    value: "indicator",
                },
                ComponentItem {
                    name: "Join",
                    href: "/components/join",
                    value: "join",
                },
                ComponentItem {
                    name: "Link",
                    href: "/components/link",
                    value: "link",
                },
                ComponentItem {
                    name: "Mask",
                    href: "/components/mask",
                    value: "mask",
                },
                ComponentItem {
                    name: "Stack",
                    href: "/components/stack",
                    value: "stack",
                },
            ],
        },
        MenuCategory {
            title: "Mockup",
            items: vec![
                ComponentItem {
                    name: "Mockup Browser",
                    href: "/components/mockup_browser",
                    value: "mockup_browser",
                },
                ComponentItem {
                    name: "Mockup Code",
                    href: "/components/mockup_code",
                    value: "mockup_code",
                },
                ComponentItem {
                    name: "Mockup Phone",
                    href: "/components/mockup_phone",
                    value: "mockup_phone",
                },
                ComponentItem {
                    name: "Mockup Window",
                    href: "/components/mockup_window",
                    value: "mockup_window",
                },
            ],
        },
        MenuCategory {
            title: "Foundation",
            items: vec![
                ComponentItem {
                    name: "Design Tokens",
                    href: "/components/tokens",
                    value: "tokens",
                },
                ComponentItem {
                    name: "Motion",
                    href: "/components/motion",
                    value: "motion",
                },
            ],
        },
    ]
}
