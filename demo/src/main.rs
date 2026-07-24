mod core;
mod debug;
mod debug_state;
mod demos;
mod test_mode;

use core::Layout;
use demos::*;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::theme::{ThemeProvider, use_theme_context};
use leptos_daisyui_rs::tokens::{UiAnimationsPreamble, UiTokensPreamble};
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path,
};

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    // PixelProof determinism seam (ldui-49w.2): `?pp-freeze=1` kills
    // animations/transitions before first paint so SSIM baselines don't
    // flap. See test_mode.rs module docs.
    if test_mode::is_test_mode() {
        test_mode::install_style_kill_switch();
    }

    mount_to_body(|| {
        view! { <App /> }
    });
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <ThemeProvider load_from_storage=true storage_key="leptos-daisyui-demo-theme">
            <AppInner />
        </ThemeProvider>
    }
}

#[component]
fn AppInner() -> impl IntoView {
    let theme_ctx = use_theme_context();

    // window.__APP_DEBUG__ state oracle (ldui-49w.3): registered once, root-
    // permanent (route + theme never unmount for the life of the session),
    // gated behind the same test-mode check as the style kill-switch above
    // so production windows never carry the debug surface. See debug.rs.
    //
    // NOTE (ldui-49w.1 fix): the route getter reads window.location.pathname
    // directly instead of leptos_router's use_location() — AppInner sits
    // OUTSIDE <Router>, so use_location() here panicked at mount (caught by
    // the suite's first real browser run). Getters are pulled on demand, so
    // the raw pathname is always current when the harness asks.
    if test_mode::is_test_mode() {
        debug::register_signal("route", || {
            serde_json::Value::String(
                web_sys::window()
                    .and_then(|w| w.location().pathname().ok())
                    .unwrap_or_default(),
            )
        });
        debug::register_signal("theme", move || {
            serde_json::Value::String(theme_ctx.config.get_untracked().base_theme)
        });
        debug::register_signal("state", debug_state::get_all);
        debug::install_debug_bridge();
    }

    // Above <Router> deliberately: an SwrCache provided inside a route dies with
    // that route, which is the re-mount the cache exists to survive.
    leptos_daisyui_rs::utils::provide_swr_cache();

    view! {
        <Html attr:data-theme=move || theme_ctx.base_theme() />
        <Title text="Leptos x daisyUI" />
        <UiTokensPreamble />
        <UiAnimationsPreamble />

        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=path!("/") view=Landing />
                <ParentRoute path=path!("/components") view=Layout>
                    <Route path=path!("/") view=AccordionDemo />
                    <Route path=path!("/accordion") view=AccordionDemo />
                    <Route path=path!("/app-shell") view=AppShellDemo />
                    <Route path=path!("/alert") view=AlertDemo />
                    <Route path=path!("/auto-complete") view=AutoCompleteDemo />
                    <Route path=path!("/avatar") view=AvatarDemo />
                    <Route path=path!("/badge") view=BadgeDemo />
                    <Route path=path!("/base_theme_selector") view=BaseThemeSelectorDemo />
                    <Route path=path!("/breadcrumbs") view=BreadcrumbsDemo />
                    <Route path=path!("/button") view=ButtonDemo />
                    <Route path=path!("/calendar") view=CalendarDemo />
                    <Route path=path!("/capacity_bar") view=CapacityBarDemo />
                    <Route path=path!("/card") view=CardDemo />
                    <Route path=path!("/carousel") view=CarouselDemo />
                    <Route path=path!("/component_customizer") view=ComponentCustomizerDemo />
                    <Route path=path!("/ai-chat") view=AiChatDemo />
                    <Route path=path!("/chat") view=ChatDemo />
                    <Route path=path!("/checkbox") view=CheckboxDemo />
                    <Route path=path!("/collapse") view=CollapseDemo />
                    <Route path=path!("/color-picker") view=ColorPickerDemo />
                    <Route path=path!("/config-provider") view=ConfigProviderDemo />
                    <Route path=path!("/countdown") view=CountdownDemo />
                    <Route path=path!("/data-table") view=DataTableDemo />
                    <Route path=path!("/day-scheduler") view=DaySchedulerDemo />
                    <Route path=path!("/diff") view=DiffDemo />
                    <Route path=path!("/divider") view=DividerDemo />
                    <Route path=path!("/dock") view=DockDemo />
                    <Route path=path!("/drawer") view=DrawerDemo />
                    <Route path=path!("/dropdown") view=DropdownDemo />
                    <Route path=path!("/empty_state") view=EmptyStateDemo />
                    <Route path=path!("/fab") view=FabDemo />
                    <Route path=path!("/fieldset") view=FieldsetDemo />
                    <Route path=path!("/file_input") view=FileInputDemo />
                    <Route path=path!("/filter") view=FilterDemo />
                    <Route path=path!("/footer") view=FooterDemo />
                    <Route path=path!("/hero") view=HeroDemo />
                    <Route path=path!("/hover_3d") view=Hover3dDemo />
                    <Route path=path!("/hover_gallery") view=HoverGalleryDemo />
                    <Route path=path!("/icon") view=IconDemo />
                    <Route path=path!("/icon_tile") view=IconTileDemo />
                    <Route path=path!("/indicator") view=IndicatorDemo />
                    <Route path=path!("/input") view=InputDemo />
                    <Route path=path!("/join") view=JoinDemo />
                    <Route path=path!("/gantt") view=GanttDemo />
                    <Route path=path!("/kanban") view=KanbanDemo />
                    <Route path=path!("/kbd") view=KbdDemo />
                    <Route path=path!("/label") view=LabelDemo />
                    <Route path=path!("/link") view=LinkDemo />
                    <Route path=path!("/list") view=ListDemo />
                    <Route path=path!("/login_enrollment") view=LoginEnrollmentDemo />
                    <Route path=path!("/loading") view=LoadingDemo />
                    <Route path=path!("/mask") view=MaskDemo />
                    <Route path=path!("/menu") view=MenuDemo />
                    <Route path=path!("/metric_row") view=MetricRowDemo />
                    <Route path=path!("/mockup_browser") view=MockupBrowserDemo />
                    <Route path=path!("/mockup_code") view=MockupCodeDemo />
                    <Route path=path!("/mockup_phone") view=MockupPhoneDemo />
                    <Route path=path!("/mockup_window") view=MockupWindowDemo />
                    <Route path=path!("/modal") view=ModalDemo />
                    <Route path=path!("/navbar") view=NavbarDemo />
                    <Route path=path!("/nav_rail") view=NavRailDemo />
                    <Route path=path!("/pagination") view=PaginationDemo />
                    <Route path=path!("/persona") view=PersonaDemo />
                    <Route path=path!("/progress") view=ProgressDemo />
                    <Route path=path!("/radial_progress") view=RadialProgressDemo />
                    <Route path=path!("/radio") view=RadioDemo />
                    <Route path=path!("/range") view=RangeDemo />
                    <Route path=path!("/rating") view=RatingDemo />
                    <Route path=path!("/result-list") view=ResultListDemo />
                    <Route path=path!("/select") view=SelectDemo />
                    <Route path=path!("/skeleton") view=SkeletonDemo />
                    <Route path=path!("/sla_chip") view=SlaChipDemo />
                    <Route path=path!("/sparkline") view=SparklineDemo />
                    <Route path=path!("/stack") view=StackDemo />
                    <Route path=path!("/stats") view=StatsDemo />
                    <Route path=path!("/status") view=StatusDemo />
                    <Route path=path!("/steps") view=StepsDemo />
                    <Route path=path!("/swap") view=SwapDemo />
                    <Route path=path!("/tab") view=TabDemo />
                    <Route path=path!("/table") view=TableDemo />
                    <Route path=path!("/tag") view=TagDemo />
                    <Route path=path!("/text_rotate") view=TextRotateDemo />
                    <Route path=path!("/textarea") view=TextareaDemo />
                    <Route path=path!("/theme_controller") view=ThemeControllerDemo />
                    <Route path=path!("/timeline") view=TimelineDemo />
                    <Route path=path!("/typography_customizer") view=TypographyCustomizerDemo />
                    <Route path=path!("/toast") view=ToastDemo />
                    <Route path=path!("/toggle") view=ToggleDemo />
                    <Route path=path!("/motion") view=MotionDemo />
                    <Route path=path!("/tokens") view=TokensDemo />
                    <Route path=path!("/swr") view=SwrDemo />
                    <Route path=path!("/toolbar") view=ToolbarDemo />
                    <Route path=path!("/tooltip") view=TooltipDemo />
                    <Route path=path!("/tree") view=TreeDemo />
                    <Route path=path!("/svg_display") view=SvgDisplayDemo />
                    <Route path=path!("/mermaid_display") view=MermaidDisplayDemo />
                    <Route path=path!("/validator") view=ValidatorDemo />
                    <Route path=path!("/vertical-steps") view=VerticalStepsDemo />
                    <Route path=path!("/week-view") view=WeekViewDemo />
                </ParentRoute>
            </Routes>
        </Router>
    }
}

#[component]
fn Landing() -> impl IntoView {
    view! {
        <Hero class="min-h-screen">
            <HeroContent>
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold mb-6">"leptos-daisyui-rs"</h1>
                    <div class="flex flex-wrap gap-4 justify-center mb-8">
                        <Badge color=BadgeColor::Accent>"Leptos"</Badge>
                        <Badge color=BadgeColor::Success>"Tailwind CSS"</Badge>
                        <Badge color=BadgeColor::Info>"daisyUI"</Badge>
                    </div>
                    <div class="flex flex-row gap-4 justify-center">
                        <LinkButton href="/components/button" color=ButtonColor::Primary>
                            "Browse Components"
                        </LinkButton>
                        <LinkButton
                            href="https://github.com/noshishiRust/leptos-daisyui-rs"
                            style=ButtonStyle::Ghost
                        >
                            <Icon icon=icondata::AiGithubFilled />
                            "View on GitHub"
                        </LinkButton>
                    </div>
                </div>
            </HeroContent>
        </Hero>
    }
}
