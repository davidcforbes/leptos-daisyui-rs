use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn ModalDemo() -> impl IntoView {
    let (modal_1_open, set_modal_1_open) = signal(false);
    let (restore_open, set_restore_open) = signal(false);
    let (restore_query, set_restore_query) = signal(String::new());

    // Controlled-close fixture (ldui-e0fw). `accept` lets the browser suite
    // drive both halves of the contract from one dialog: accepting a
    // proposal must close it exactly once, declining must leave it open with
    // the accepted state untouched.
    let (controlled_open, set_controlled_open) = signal(false);
    let (accept, set_accept) = signal(true);
    let (last_cause, set_last_cause) = signal(String::new());
    let (proposal_count, set_proposal_count) = signal(0usize);

    let on_close_request = Callback::new(move |proposal: ModalCloseProposal| {
        set_last_cause.set(proposal.cause.as_str().to_string());
        set_proposal_count.update(|count| *count += 1);
        crate::debug_state::set("modal.controlled.last_cause", proposal.cause.as_str());
        crate::debug_state::set(
            "modal.controlled.proposal_count",
            proposal_count.get_untracked(),
        );
        if accept.get_untracked() {
            set_controlled_open.set(false);
            crate::debug_state::set("modal.controlled.open", false);
        }
    });

    view! {
        <ContentLayout
            title="Modal"
            description="Modals are used to show content in a layer above the page"
        >

            <Section title="Basic Modal">

                <Button
                    color=ButtonColor::Primary
                    on:click=move |_| {
                        set_modal_1_open.set(true);
                        // PixelProof oracle (ldui-49w.1): expose open state at
                        // window.__APP_DEBUG__.state().state["modal.open"].
                        crate::debug_state::set("modal.open", true);
                    }
                >
                    "Open Modal"
                </Button>

                <Modal
                    open=modal_1_open
                    labelled_by="basic-modal-title"
                    described_by="basic-modal-desc"
                    on:close=move |_| {
                        set_modal_1_open.set(false);
                        crate::debug_state::set("modal.open", false);
                    }
                >
                    <ModalBox>
                        <h3 class="text-lg font-bold" id="basic-modal-title">"Hello!"</h3>
                        <p class="py-4" id="basic-modal-desc">
                            "Press ESC key or click the button below to close"
                        </p>
                        <ModalAction>
                            <form method="dialog">
                                <Button on:click=move |_| {
                                    set_modal_1_open.set(false);
                                    crate::debug_state::set("modal.open", false);
                                }>

                                    "Close"
                                </Button>
                            </form>
                        </ModalAction>
                    </ModalBox>
                </Modal>
            </Section>

            <Section title="Controlled close proposals">
                <p class="text-sm text-base-content/75 mb-4">
                    "Escape, the backdrop, and an in-content "
                    <code>"method=\"dialog\""</code>
                    " form each emit a typed proposal instead of closing the dialog behind the owner's back. The owner decides."
                </p>

                <div class="flex flex-wrap items-center gap-3 mb-4">
                    <Button
                        color=ButtonColor::Primary
                        attr:data-testid="controlled-modal-trigger"
                        on:click=move |_| {
                            set_controlled_open.set(true);
                            crate::debug_state::set("modal.controlled.open", true);
                        }
                    >
                        "Open controlled modal"
                    </Button>

                    <button
                        class="btn btn-sm"
                        data-testid="controlled-modal-accept-toggle"
                        on:click=move |_| set_accept.update(|value| *value = !*value)
                    >
                        {move || {
                            if accept.get() {
                                "Accepting proposals"
                            } else {
                                "Declining proposals"
                            }
                        }}
                    </button>
                </div>

                <dl class="flex flex-wrap gap-6 text-sm">
                    <div class="flex gap-2">
                        <dt class="text-base-content/75">"Accepted open state:"</dt>
                        <dd data-testid="controlled-modal-open">
                            {move || if controlled_open.get() { "true" } else { "false" }}
                        </dd>
                    </div>
                    <div class="flex gap-2">
                        <dt class="text-base-content/75">"Proposals received:"</dt>
                        <dd data-testid="controlled-modal-proposal-count">
                            {move || proposal_count.get()}
                        </dd>
                    </div>
                    <div class="flex gap-2">
                        <dt class="text-base-content/75">"Last cause:"</dt>
                        <dd data-testid="controlled-modal-last-cause">
                            {move || {
                                let cause = last_cause.get();
                                if cause.is_empty() { "none".to_string() } else { cause }
                            }}
                        </dd>
                    </div>
                    <div class="flex gap-2">
                        <dt class="text-base-content/75">"Policy:"</dt>
                        <dd data-testid="controlled-modal-policy">
                            {move || if accept.get() { "accept" } else { "decline" }}
                        </dd>
                    </div>
                </dl>

                <Modal
                    open=controlled_open
                    backdrop=true
                    labelled_by="controlled-modal-title"
                    described_by="controlled-modal-desc"
                    on_close_request=on_close_request
                >
                    <ModalBox attr:data-testid="controlled-modal-box">
                        <h3 class="text-lg font-bold" id="controlled-modal-title">
                            "Reassign matter"
                        </h3>
                        <p class="py-4" id="controlled-modal-desc">
                            "Press Escape, click the backdrop, or use either button below."
                        </p>
                        <ModalAction>
                            <form method="dialog">
                                <Button
                                    style=ButtonStyle::Ghost
                                    attr:data-testid="controlled-modal-dialog-form-close"
                                >
                                    "Dialog-form close"
                                </Button>
                            </form>
                            <Button
                                color=ButtonColor::Primary
                                attr:data-testid="controlled-modal-programmatic-close"
                                on:click=move |_| {
                                    set_controlled_open.set(false);
                                    crate::debug_state::set("modal.controlled.open", false);
                                }
                            >
                                "Programmatic close"
                            </Button>
                        </ModalAction>
                    </ModalBox>
                </Modal>
            </Section>

            <Section title="Find & Restore recipe">
                <p class="text-sm text-base-content/75 mb-4">
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
