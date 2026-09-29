use crate::core::{ContentLayout, Section};
use ldui_design::fixtures::FixtureSection;
use ldui_design::fixtures::button::{self as fixtures, ButtonExample, ButtonExampleState};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// `active` for a fixture state.
fn state_is_active(state: ButtonExampleState) -> bool {
    matches!(state, ButtonExampleState::Active)
}

/// `disabled_reason` for a fixture state; blank leaves the button enabled.
fn state_disabled_reason(state: ButtonExampleState) -> &'static str {
    match state {
        ButtonExampleState::Disabled { reason } => reason,
        _ => "",
    }
}

/// The extra class for a fixture state. Loading is daisyUI's `loading` class
/// rather than the `loading` prop, as this page has always shown it.
fn state_class(state: ButtonExampleState) -> &'static str {
    match state {
        ButtonExampleState::Loading => "loading",
        _ => "",
    }
}

/// One shared-fixture section (ldui-rz43): the title and every button come
/// from `ldui_design::fixtures::button`, the list 4iiz-kit's gallery renders
/// too. Name no variant and no label here, or the two renderers drift
/// (`tests/demo_shared_fixtures.rs`).
#[component]
fn ButtonFixtureSection(section: FixtureSection<ButtonExample>) -> impl IntoView {
    view! {
        <Section row=true title=section.title>
            {section
                .examples
                .iter()
                .map(|example| {
                    view! {
                        <Button
                            color=example.color.clone()
                            style=example.style.clone()
                            size=example.size.clone()
                            shape=example.shape.clone()
                            active=state_is_active(example.state)
                            disabled_reason=state_disabled_reason(example.state)
                            class=state_class(example.state)
                        >
                            {example.label}
                        </Button>
                    }
                })
                .collect_view()}
        </Section>
    }
}

#[component]
pub fn ButtonDemo() -> impl IntoView {
    let (counter, set_counter) = signal(0);
    let (loading, set_loading) = signal(false);
    let (active_color, set_active_color) = signal(ButtonColor::Primary);

    // Native form semantics fixture (ldui-9vs). `submit_count` proves a
    // ButtonType::Submit button activates the containing form exactly once
    // per click/keyboard activation; the disabled/loading submit buttons
    // wire to the *same* form + counter so a browser fixture can prove they
    // never move it. PixelProof oracle: window.__APP_DEBUG__.state()
    // .state["button.form_submit_count"] / ["button.form_reset_count"].
    let (form_submit_count, set_form_submit_count) = signal(0);
    let (form_reset_count, set_form_reset_count) = signal(0);

    view! {
        <ContentLayout
            title="Button"
            description="Buttons allow users to take actions and make choices"
        >
            <ButtonFixtureSection section=fixtures::COLORS />
            <ButtonFixtureSection section=fixtures::SIZES />
            <ButtonFixtureSection section=fixtures::STYLES />
            <ButtonFixtureSection section=fixtures::STATES />
            <ButtonFixtureSection section=fixtures::SHAPES />

            <Section title="Push Effect Demo">
                <div class="flex flex-col gap-4">
                    <p class="text-sm opacity-70">
                        "Click any button to see the enhanced push-depth effect"
                    </p>
                    <div class="flex items-center gap-2">
                        <Button color=ButtonColor::Primary>"Click Me!"</Button>
                        <Button color=ButtonColor::Secondary>"Press Me!"</Button>
                        <Button color=ButtonColor::Accent>"Push Me!"</Button>
                        <Button style=ButtonStyle::Outline>"Outlined"</Button>
                        <Button style=ButtonStyle::Ghost>"Ghost"</Button>
                    </div>
                </div>
            </Section>

            <Section title="Reactive Examples">
                <div class="flex flex-col gap-4">
                    <div class="flex items-center gap-2">
                        <Button
                            color=ButtonColor::Primary
                            on:click=move |_| set_counter.update(|c| *c += 1)
                        >
                            "Clicked "
                            {counter}
                            " times"
                        </Button>
                        <Button
                            color=ButtonColor::Error
                            style=ButtonStyle::Outline
                            on:click=move |_| set_counter.set(0)
                        >
                            "Reset"
                        </Button>
                    </div>

                    <div class="flex items-center gap-2">
                        <Button
                            color=ButtonColor::Info
                            class:loading=loading
                            on:click=move |_| {
                                set_loading.set(true);
                                set_timeout(
                                    move || set_loading.set(false),
                                    std::time::Duration::from_millis(2000),
                                );
                            }
                        >
                            "Simulate Loading"
                        </Button>
                    </div>

                    <div class="flex items-center gap-2">
                        <Button
                            color=active_color
                            on:click=move |_| {
                                set_active_color
                                    .update(|color| {
                                        *color = match color {
                                            ButtonColor::Primary => ButtonColor::Secondary,
                                            ButtonColor::Secondary => ButtonColor::Accent,
                                            ButtonColor::Accent => ButtonColor::Success,
                                            ButtonColor::Success => ButtonColor::Warning,
                                            ButtonColor::Warning => ButtonColor::Error,
                                            ButtonColor::Error => ButtonColor::Primary,
                                            _ => ButtonColor::Primary,
                                        }
                                    });
                            }
                        >
                            "Cycle Color: "
                            {move || format!("{:?}", active_color.get())}
                        </Button>
                    </div>
                </div>
            </Section>

            <Section title="Native Form Semantics">
                <p class="text-sm opacity-70">
                    "button_type drives the native type attribute; form action/method stay on the <form> element (ldui-9vs)."
                </p>
                <form
                    id="button-type-form"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        let next = form_submit_count.get_untracked() + 1;
                        set_form_submit_count.set(next);
                        crate::debug_state::set("button.form_submit_count", next);
                    }
                    on:reset=move |_| {
                        let next = form_reset_count.get_untracked() + 1;
                        set_form_reset_count.set(next);
                        crate::debug_state::set("button.form_reset_count", next);
                    }
                    class="flex flex-col gap-4"
                >
                    <label class="flex items-center gap-2">
                        // Plain, unstyled native checkbox (ldui-d2hg): the
                        // fixture only proves native `reset` restores its
                        // `checked` state (tests/reactivity_smoke.rs), which
                        // is true of any `<input type="checkbox">` — no
                        // daisyUI styling needed. daisyUI's `.checkbox`
                        // class paints an inset box-shadow via an oklch
                        // color outside the page's declared shadow set,
                        // which pushed this page over its DEPTH ceiling.
                        <input id="button-type-form-checkbox" type="checkbox" />
                        "Toggled by hand, restored by native reset"
                    </label>
                    <div class="flex items-center gap-2">
                        <Button attr:id="button-type-default" button_type=ButtonType::Button>
                            "type=button (default, no form action)"
                        </Button>
                        <Button
                            attr:id="button-type-submit"
                            button_type=ButtonType::Submit
                            color=ButtonColor::Primary
                        >
                            "type=submit"
                        </Button>
                        <Button attr:id="button-type-reset" button_type=ButtonType::Reset>
                            "type=reset"
                        </Button>
                    </div>
                    <div class="flex items-center gap-2">
                        <Button
                            attr:id="button-type-submit-disabled"
                            button_type=ButtonType::Submit
                            disabled=true
                            disabled_reason="Shown disabled for the type showcase"
                        >
                            "type=submit, disabled"
                        </Button>
                        <Button
                            attr:id="button-type-submit-loading"
                            button_type=ButtonType::Submit
                            loading=true
                        >
                            "type=submit, loading"
                        </Button>
                    </div>
                    // Precedence probe (ldui-9vs): button_type=Reset plus a
                    // duplicate spread attr:type="submit". Never clicked —
                    // this only exists so a browser fixture can read which
                    // one the DOM actually carries. See Button's doc comment
                    // "Precedence vs a spread attr:type".
                    <Button
                        attr:id="button-type-precedence-probe"
                        button_type=ButtonType::Reset
                        attr:r#type="submit"
                    >
                        "precedence probe (not for clicking)"
                    </Button>
                </form>
                <p class="text-sm opacity-70">
                    "Submits: " {form_submit_count} " · Resets: " {form_reset_count}
                </p>
            </Section>

            <Section title="Link Button">
                <div class="flex items-center gap-2">
                    <LinkButton
                        href="#"
                        color=ButtonColor::Primary
                        on:click=move |e| {
                            e.prevent_default();
                        }
                    >
                        "Primary Link"
                    </LinkButton>
                    <LinkButton
                        href="#"
                        style=ButtonStyle::Outline
                        on:click=move |e| {
                            e.prevent_default();
                        }
                    >
                        "Outline Link"
                    </LinkButton>
                    <LinkButton
                        href="#"
                        style=ButtonStyle::Ghost
                        on:click=move |e| {
                            e.prevent_default();
                        }
                    >
                        "Ghost Link"
                    </LinkButton>
                </div>
            </Section>
        </ContentLayout>
    }
}
