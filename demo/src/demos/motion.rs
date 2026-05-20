use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn MotionDemo() -> impl IntoView {
    let (dialog_open, set_dialog_open) = signal(false);
    let (toast_counter, set_toast_counter) = signal(0_u32);
    let (dropdown_open, set_dropdown_open) = signal(false);
    let (drawer_start_open, set_drawer_start_open) = signal(false);
    let (drawer_end_open, set_drawer_end_open) = signal(false);

    view! {
        <ContentLayout
            title="Motion"
            description="Every ld-* motion primitive shipped by ldui-e1o, on one page. Hover, click, tab, and watch."
        >
            <Section title="1. Eased state transitions">
                <p class="text-sm opacity-70 mb-3">
                    "Hover and press each button \u{2014} colour and transform ease over var(--ld-duration-fast)."
                </p>
                <div class="flex flex-wrap gap-3">
                    <Button color=ButtonColor::Primary>"Primary"</Button>
                    <Button color=ButtonColor::Secondary>"Secondary"</Button>
                    <Button style=ButtonStyle::Ghost>"Ghost"</Button>
                    <Button style=ButtonStyle::Outline>"Outline"</Button>
                </div>
            </Section>

            <Section title="2. Press depression">
                <p class="text-sm opacity-70 mb-3">
                    "Press-and-hold to feel the subtle scale(0.97) on .ld-pressable."
                </p>
                <div class="flex flex-wrap gap-3">
                    <Button color=ButtonColor::Primary>"Press me"</Button>
                </div>
            </Section>

            <Section title="3. Card elevation lift">
                <p class="text-sm opacity-70 mb-3">
                    "Hover both cards \u{2014} the one with " <code>"elevate=true"</code>
                    " lifts from elevation-4 to elevation-8 with a 1px translate."
                </p>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <Card class="bg-base-100 max-w-sm">
                        <CardBody>
                            <h3 class="card-title text-base">"Static card"</h3>
                            <p class="text-sm opacity-70">"No elevate prop \u{2014} sits flat."</p>
                        </CardBody>
                    </Card>
                    <Card elevate=true class="bg-base-100 max-w-sm">
                        <CardBody>
                            <h3 class="card-title text-base">"Elevated card"</h3>
                            <p class="text-sm opacity-70">"Hover to lift."</p>
                        </CardBody>
                    </Card>
                </div>
            </Section>

            <Section title="4. Click ripple">
                <p class="text-sm opacity-70 mb-3">
                    "Click each button \u{2014} a "<code>"currentColor"</code>
                    " ripple radiates from the click point over var(--ld-duration-normal)."
                </p>
                <div class="flex flex-wrap gap-3">
                    <Button color=ButtonColor::Primary ripple=true>"Primary + ripple"</Button>
                    <Button color=ButtonColor::Accent ripple=true>"Accent + ripple"</Button>
                    <Button style=ButtonStyle::Outline ripple=true>"Outline + ripple"</Button>
                </div>
                <div class="mt-4">
                    <Card ripple=true elevate=true class="bg-base-100 max-w-sm cursor-pointer">
                        <CardBody>
                            <h3 class="card-title text-base">"Pressable Card"</h3>
                            <p class="text-sm opacity-70">"Click anywhere on this card."</p>
                        </CardBody>
                    </Card>
                </div>
            </Section>

            <Section title="5. Dialog scale-up + backdrop fade">
                <p class="text-sm opacity-70 mb-3">
                    "Opens an HTML dialog with " <code>"@keyframes ld-dialog-in"</code>
                    " on the modal-box and " <code>"ld-backdrop-in"</code> " on ::backdrop."
                </p>
                <Button color=ButtonColor::Primary on:click=move |_| set_dialog_open.set(true)>
                    "Open dialog"
                </Button>
                <Modal open=dialog_open on:close=move |_| set_dialog_open.set(false)>
                    <ModalBox>
                        <h3 class="text-lg font-bold">"Dialog entrance"</h3>
                        <p class="py-4">
                            "The box scaled from 0.92 \u{2192} 1.0 over var(--ld-duration-normal)
                             with the decelerate easing, while the backdrop faded from 0 \u{2192} 1."
                        </p>
                        <ModalAction>
                            <Button on:click=move |_| set_dialog_open.set(false)>"Close"</Button>
                        </ModalAction>
                    </ModalBox>
                </Modal>
            </Section>

            <Section title="6. Toast slide-in">
                <p class="text-sm opacity-70 mb-3">
                    "Each click adds a new toast child. CSS animation fires on insertion."
                </p>
                <Button
                    color=ButtonColor::Secondary
                    on:click=move |_| set_toast_counter.update(|n| *n += 1)
                >
                    "Drop a toast"
                </Button>
                <div class="relative h-48 mt-4 border border-base-300 rounded-lg overflow-hidden">
                    <Toast position=ToastPosition::BottomEnd class="absolute">
                        {move || {
                            (0..toast_counter.get())
                                .map(|i| view! {
                                    <div class="alert alert-info">
                                        <span>"Toast #" {i + 1}</span>
                                    </div>
                                })
                                .collect_view()
                        }}
                    </Toast>
                </div>
            </Section>

            <Section title="7. Dropdown slide-down">
                <p class="text-sm opacity-70 mb-3">
                    "Toggle to see " <code>"ld-dropdown-in"</code>
                    " fire each time .dropdown-open begins matching."
                </p>
                <Dropdown open=dropdown_open>
                    <Button on:click=move |_| set_dropdown_open.update(|o| *o = !*o)>
                        "Toggle dropdown"
                    </Button>
                    <DropdownContent class="menu p-2 shadow bg-base-100 rounded-box w-52">
                        <li><a>"Option one"</a></li>
                        <li><a>"Option two"</a></li>
                        <li><a>"Option three"</a></li>
                    </DropdownContent>
                </Dropdown>
            </Section>

            <Section title="8. Drawer edge-slide">
                <p class="text-sm opacity-70 mb-3">
                    "Two variants \u{2014} start (slides from the left) and end (slides from the right)."
                </p>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="h-64 bg-base-200 rounded-lg overflow-hidden">
                        <Drawer open=drawer_start_open>
                            <DrawerToggle
                                id="motion-drawer-start"
                                checked=drawer_start_open
                                on:click=move |_| {
                                    set_drawer_start_open.update(|o| *o = !*o)
                                }
                            />
                            <div class="drawer-content flex items-center justify-center">
                                <Button
                                    on:click=move |_| set_drawer_start_open.set(true)
                                >"Open start"</Button>
                            </div>
                            <div class="drawer-side">
                                <label
                                    for="motion-drawer-start"
                                    class="drawer-overlay"
                                    on:click=move |_| set_drawer_start_open.set(false)
                                />
                                <ul class="menu p-4 w-56 min-h-full bg-base-100">
                                    <li><a>"Start \u{2190}"</a></li>
                                </ul>
                            </div>
                        </Drawer>
                    </div>

                    <div class="h-64 bg-base-200 rounded-lg overflow-hidden">
                        <Drawer open=drawer_end_open placement=DrawerPlacement::End>
                            <DrawerToggle
                                id="motion-drawer-end"
                                checked=drawer_end_open
                                on:click=move |_| {
                                    set_drawer_end_open.update(|o| *o = !*o)
                                }
                            />
                            <div class="drawer-content flex items-center justify-center">
                                <Button
                                    on:click=move |_| set_drawer_end_open.set(true)
                                >"Open end"</Button>
                            </div>
                            <div class="drawer-side">
                                <label
                                    for="motion-drawer-end"
                                    class="drawer-overlay"
                                    on:click=move |_| set_drawer_end_open.set(false)
                                />
                                <ul class="menu p-4 w-56 min-h-full bg-base-100">
                                    <li><a>"End \u{2192}"</a></li>
                                </ul>
                            </div>
                        </Drawer>
                    </div>
                </div>
            </Section>

            <Section title="9. Animated focus ring">
                <p class="text-sm opacity-70 mb-3">
                    "Press " <kbd class="kbd kbd-xs">"Tab"</kbd>
                    " through these controls \u{2014} the outline fades in via "
                    <code>"ld-focus-ring-in"</code>
                    ". Clicking instead does NOT trigger it (matches :focus-visible)."
                </p>
                <div class="flex flex-wrap gap-3 items-center">
                    <Button>"Tab here"</Button>
                    <input type="text" class="input ld-focus-ring" placeholder="text input" />
                    <Checkbox />
                    <Toggle />
                    <Radio name="motion-radio" />
                </div>
            </Section>

            <Section title="10. Reduced motion">
                <p class="text-sm opacity-70 mb-3">
                    "Open DevTools \u{2192} Rendering \u{2192} \"Emulate CSS media feature
                     prefers-reduced-motion: reduce\". Every transition, transform, and animation
                     on this page should go static \u{2014} state changes happen instantly with no easing."
                </p>
            </Section>
        </ContentLayout>
    }
}
