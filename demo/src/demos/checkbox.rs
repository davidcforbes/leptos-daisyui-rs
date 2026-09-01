use crate::core::{ContentLayout, Section};
use crate::debug_state;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// The two locales the controlled fixture swaps between, proving that the
/// accessible name is replaced in place rather than baked in at mount
/// (`ldui-fqan`).
fn past_due_label(locale: &str) -> String {
    match locale {
        "es" => "Solo vencidos".to_owned(),
        _ => "Past due only".to_owned(),
    }
}

fn partial_label(locale: &str) -> String {
    match locale {
        "es" => "Seleccionar todas las oficinas".to_owned(),
        _ => "Select all offices".to_owned(),
    }
}

#[component]
pub fn CheckboxDemo() -> impl IntoView {
    let checked_state = RwSignal::new(false);
    let selected_color = RwSignal::new(CheckboxColor::Primary);
    let selected_size = RwSignal::new(CheckboxSize::Md);

    // ── Controlled fixture (ldui-fqan) ───────────────────────────────────
    // Accepted truth lives here, in the caller. The checkbox only proposes.
    let accepted = RwSignal::new(false);
    let accept_proposals = RwSignal::new(true);
    let proposal_count = RwSignal::new(0_u32);
    let locale = RwSignal::new("en");
    // Partial-selection fixture: `mixed` is accepted truth just like
    // `all_selected`, so the tri-state is never inferred from the DOM.
    let all_selected = RwSignal::new(false);
    let mixed = RwSignal::new(true);

    Effect::new(move |_| {
        debug_state::set("checkbox.accepted", accepted.get());
        debug_state::set("checkbox.accept_proposals", accept_proposals.get());
        debug_state::set("checkbox.proposal_count", proposal_count.get());
        debug_state::set("checkbox.all_selected", all_selected.get());
        debug_state::set("checkbox.mixed", mixed.get());
        debug_state::set("checkbox.locale", locale.get());
    });
    on_cleanup(move || {
        for key in [
            "checkbox.accepted",
            "checkbox.accept_proposals",
            "checkbox.proposal_count",
            "checkbox.all_selected",
            "checkbox.mixed",
            "checkbox.locale",
            "checkbox.last_checked",
            "checkbox.last_from",
            "checkbox.disabled_proposals",
        ] {
            debug_state::remove(key);
        }
    });

    let past_due_binding = CheckboxBinding::controlled(
        accepted.into(),
        Callback::new(move |proposal: CheckboxChangeProposal| {
            proposal_count.update(|count| *count += 1);
            debug_state::set("checkbox.last_checked", proposal.checked);
            debug_state::set("checkbox.last_from", proposal.from.as_str());
            // The whole point of a proposal: the owner may decline it, and
            // the rendered checkbox stays on accepted truth either way.
            if accept_proposals.get_untracked() {
                accepted.set(proposal.checked);
            }
        }),
    );

    let partial_binding = CheckboxBinding::controlled(
        all_selected.into(),
        Callback::new(move |proposal: CheckboxChangeProposal| {
            proposal_count.update(|count| *count += 1);
            debug_state::set("checkbox.last_from", proposal.from.as_str());
            // Shares the owner's accept/decline policy, so a declined gesture
            // is the case that proves `indeterminate` is re-asserted after the
            // browser cleared it (ldui-nz6d).
            if accept_proposals.get_untracked() {
                all_selected.set(proposal.checked);
                mixed.set(false);
            }
        }),
    )
    .with_indeterminate(mixed.into());

    let disabled_proposals = RwSignal::new(0_u32);
    let disabled_binding = CheckboxBinding::controlled(
        Signal::derive(|| true),
        Callback::new(move |_: CheckboxChangeProposal| {
            disabled_proposals.update(|count| *count += 1);
            debug_state::set("checkbox.disabled_proposals", count_of(disabled_proposals));
        }),
    );

    view! {
        <ContentLayout
            title="Checkbox"
            description="Checkboxes are used to select one or multiple options from a list"
        >
            <Section row=true title="Colors">
                <Checkbox attr:checked=true />
                <Checkbox color=CheckboxColor::Primary attr:checked=true />
                <Checkbox color=CheckboxColor::Secondary attr:checked=true />
                <Checkbox color=CheckboxColor::Accent attr:checked=true />
                <Checkbox color=CheckboxColor::Success attr:checked=true />
                <Checkbox color=CheckboxColor::Warning attr:checked=true />
                <Checkbox color=CheckboxColor::Info attr:checked=true />
                <Checkbox color=CheckboxColor::Error attr:checked=true />
            </Section>

            <Section row=true title="Sizes">
                <Checkbox size=CheckboxSize::Xs attr:checked=true />
                <Checkbox size=CheckboxSize::Sm attr:checked=true />
                <Checkbox size=CheckboxSize::Md attr:checked=true />
                <Checkbox size=CheckboxSize::Lg attr:checked=true />
                <Checkbox size=CheckboxSize::Xl attr:checked=true />
            </Section>

            <Section row=true title="States">
                <Checkbox attr:data-testid="checkbox-bare" />
                <Checkbox attr:checked=true />
                <Checkbox attr:disabled=true />
                <Checkbox attr:disabled=true attr:checked=true />
                <Checkbox default_checked=true attr:data-testid="checkbox-default-checked" />
            </Section>

            <Section title="Interactive Example" col=true>
                <label class="cursor-pointer label">
                    <span class="text-sm">
                        {move || { if checked_state.get() { "Checked" } else { "Unchecked" } }}
                    </span>
                    <Checkbox
                        color=selected_color
                        size=selected_size
                        prop:checked=move || checked_state.get()
                        on:change=move |ev| checked_state.set(event_target_checked(&ev))
                    />
                </label>
            </Section>

            <Section title="Controlled (change proposals)" col=true>
                <Checkbox
                    id="past-due-only"
                    label=Signal::derive(move || past_due_label(locale.get()))
                    binding=past_due_binding
                    attr:data-testid="checkbox-controlled"
                />
                <div class="flex flex-wrap items-center gap-2">
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="checkbox-accept-toggle"
                        on:click=move |_| accept_proposals.update(|accept| *accept = !*accept)
                    >
                        {move || {
                            if accept_proposals.get() {
                                "Owner accepts proposals"
                            } else {
                                "Owner declines proposals"
                            }
                        }}
                    </Button>
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="checkbox-external-set"
                        on:click=move |_| accepted.set(true)
                    >
                        "Set accepted = true"
                    </Button>
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="checkbox-external-reset"
                        on:click=move |_| accepted.set(false)
                    >
                        "Reset accepted"
                    </Button>
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="checkbox-locale-toggle"
                        on:click=move |_| {
                            locale.update(|current| *current = if *current == "en" { "es" } else { "en" })
                        }
                    >
                        "Switch locale"
                    </Button>
                </div>
            </Section>

            <Section title="Controlled (indeterminate and disabled)" col=true>
                <Checkbox
                    id="all-offices"
                    label=Signal::derive(move || partial_label(locale.get()))
                    binding=partial_binding
                    attr:data-testid="checkbox-partial"
                />
                <Checkbox
                    id="locked-option"
                    label="Locked (disabled, emits nothing)"
                    disabled=true
                    binding=disabled_binding
                    attr:data-testid="checkbox-disabled"
                />
                <Button
                    size=ButtonSize::Sm
                    attr:data-testid="checkbox-mixed-reset"
                    on:click=move |_| {
                        mixed.set(true);
                        all_selected.set(false);
                    }
                >
                    "Back to partial"
                </Button>
            </Section>

            <Section title="Refused configuration" col=true>
                <Checkbox
                    label="Never rendered"
                    aria_label="Never rendered either"
                    attr:data-testid="checkbox-config-error"
                />
            </Section>
        </ContentLayout>
    }
}

/// Reads a counter without subscribing, for the debug-state write inside a
/// callback (a tracked read there would re-run the effect that owns it).
fn count_of(signal: RwSignal<u32>) -> u32 {
    signal.get_untracked()
}
