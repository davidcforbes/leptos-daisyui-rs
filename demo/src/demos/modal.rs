use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn ModalDemo() -> impl IntoView {
    let (modal_1_open, set_modal_1_open) = signal(false);
    let (restore_open, set_restore_open) = signal(false);
    let (restore_query, set_restore_query) = signal(String::new());

    view! {
        <ContentLayout
            title="Modal"
            description="Modals are used to show content in a layer above the page"
        >

            <Section title="Basic Modal">

                <Button color=ButtonColor::Primary on:click=move |_| set_modal_1_open.set(true)>
                    "Open Modal"
                </Button>

                <Modal open=modal_1_open on:close=move |_| set_modal_1_open.set(false)>
                    <ModalBox>
                        <h3 class="text-lg font-bold">"Hello!"</h3>
                        <p class="py-4">"Press ESC key or click the button below to close"</p>
                        <ModalAction>
                            <form method="dialog">
                                <Button on:click=move |_| {
                                    set_modal_1_open.set(false)
                                }>

                                    "Close"
                                </Button>
                            </form>
                        </ModalAction>
                    </ModalBox>
                </Modal>
            </Section>

            <Section title="Find & Restore recipe">
                <p class="text-sm opacity-70 mb-4">
                    <code>"ModalInfoRow"</code>", " <code>"ModalSearchRow"</code>
                    ", and " <code>"ModalStatusRow"</code>
                    " formalise the title \u{2192} info \u{2192} search \u{2192} status \u{2192} body \u{2192} actions pattern."
                </p>

                <Button
                    color=ButtonColor::Secondary
                    on:click=move |_| set_restore_open.set(true)
                >
                    "Open Find & Restore"
                </Button>

                <Modal
                    open=restore_open
                    on:close=move |_| set_restore_open.set(false)
                >
                    <ModalBox class="max-w-2xl">
                        <h3 class="text-lg font-bold">"Find & Restore"</h3>

                        <ModalInfoRow label="Source:">
                            "backup-2026-05-19"
                        </ModalInfoRow>
                        <ModalInfoRow label="Files indexed:">
                            "12,408"
                        </ModalInfoRow>

                        <ModalSearchRow>
                            <input
                                type="text"
                                class="input input-bordered input-sm w-full"
                                placeholder="filter files…"
                                prop:value=move || restore_query.get()
                                on:input=move |ev| {
                                    let target = event_target_value(&ev);
                                    set_restore_query.set(target);
                                }
                            />
                        </ModalSearchRow>

                        <ModalStatusRow>
                            {move || {
                                let q = restore_query.get();
                                if q.is_empty() {
                                    "Showing all files".to_string()
                                } else {
                                    format!("Filtering by \"{}\"", q)
                                }
                            }}
                        </ModalStatusRow>

                        <ul class="py-3 space-y-1 text-sm">
                            <li>"/home/user/docs/notes.md"</li>
                            <li>"/home/user/docs/quarterly-report.pdf"</li>
                            <li>"/home/user/code/leptos-daisyui-rs/README.md"</li>
                        </ul>

                        <ModalAction>
                            <Button
                                style=ButtonStyle::Ghost
                                on:click=move |_| set_restore_open.set(false)
                            >
                                "Cancel"
                            </Button>
                            <Button
                                color=ButtonColor::Primary
                                on:click=move |_| set_restore_open.set(false)
                            >
                                "Restore selected"
                            </Button>
                        </ModalAction>
                    </ModalBox>
                </Modal>
            </Section>
        </ContentLayout>
    }
}
