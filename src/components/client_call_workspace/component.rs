use super::*;
use crate::components::{
    Button, ButtonColor, ButtonStyle, Field, FieldState, Input, InputType, Select, Softphone,
    SoftphoneCommand, Textarea,
};
use leptos::prelude::*;

fn emit(
    state: Signal<ClientCallWorkspaceState>,
    callback: Callback<ClientCallCommand>,
    action: ClientCallAction,
) {
    let current = state.get_untracked();
    if current.can_dispatch(&action) {
        callback.run(ClientCallCommand {
            context_id: current.call.context_id,
            action,
        });
    }
}

/// The reactive accessible name, as a closure so it serves both an
/// `attr:aria-label` spread and (through `Signal::derive`) a `label` prop.
fn control_name(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    control: ClientCallControl,
) -> impl Fn() -> String + Clone + Send + Sync + 'static {
    move || state.with(|s| texts.with(|t| t.control_name(&control, s)))
}

fn control_disabled(
    state: Signal<ClientCallWorkspaceState>,
    control: ClientCallControl,
) -> Signal<bool> {
    Signal::derive(move || !state.with(|s| s.control_enabled(&control)))
}

/// The full disabled reason, for a Field-wrapped control whose help line
/// carries it (the Field owns that control's `aria-describedby`).
fn control_reason(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    control: ClientCallControl,
) -> Signal<Option<String>> {
    Signal::derive(move || state.with(|s| texts.with(|t| t.disabled_reason(&control, s))))
}

/// The reason a button's own line shows. While no context exists the shared
/// context line already says why, so the button line stays empty rather than
/// repeating it nine times; the button's `aria-describedby` names both lines.
///
/// The number pad is the exception (ldui-eray, Office op-1yxvd): its line sits
/// under the destination field, apart from the shared line, so it states the
/// reason itself -- and its `aria-describedby` names only its own line, so the
/// reason is announced once.
pub(crate) fn own_line_reason(
    control: &ClientCallControl,
    state: &ClientCallWorkspaceState,
    texts: &ClientCallWorkspaceTexts,
) -> Option<String> {
    if state.context_missing()
        && !matches!(
            control,
            ClientCallControl::Keypad | ClientCallControl::Destination
        )
    {
        None
    } else {
        texts.disabled_reason(control, state)
    }
}

fn local_reason(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    control: ClientCallControl,
) -> Signal<Option<String>> {
    Signal::derive(move || state.with(|s| texts.with(|t| own_line_reason(&control, s, t))))
}

fn context_reason_id(base_id: &str) -> String {
    format!("{base_id}-reason-context")
}

fn reason_id(base_id: &str, key: &str) -> String {
    format!("{base_id}-reason-{key}")
}

/// `aria-describedby` for a button: the shared context line plus its own line.
fn described_by(base_id: &str, key: &str) -> String {
    format!("{} {}", context_reason_id(base_id), reason_id(base_id, key))
}

/// A visible disabled-reason line; hidden (and empty) while there is no reason.
fn reason_line(id: String, key: &'static str, text: Signal<Option<String>>) -> impl IntoView {
    view! {
        <p id=id class="text-sm text-base-content/75 [overflow-wrap:anywhere]" data-call-disabled-reason=key
            class:hidden=move || text.with(Option::is_none)>{move || text.get()}</p>
    }
}

#[component]
fn Destination(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
    keypad_id: String,
    base_id: String,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let target_ref = NodeRef::<leptos::html::Select>::new();
    let targets = Memo::new(move |_| {
        state
            .get()
            .number_update
            .map(|u| u.targets)
            .unwrap_or_default()
    });
    let keypad = RwSignal::new(false);
    let controls = keypad_id.clone();
    let phones = Memo::new(move |_| state.get().call.client.phones);
    // Reason-line ids, minted once from the workspace id (Office op-stjm9).
    let saved_base = base_id.clone();
    let destination_reason = reason_id(&base_id, "destination");
    // Own line only: the pad's line carries the context reason too (ldui-eray).
    let destination_described = destination_reason.clone();
    let pad_described = described_by(&base_id, "pad");
    let pad_reason = reason_id(&base_id, "pad");
    let backspace_described = described_by(&base_id, "backspace");
    let backspace_reason = reason_id(&base_id, "backspace");
    let save_number_described = described_by(&base_id, "save-number");
    let save_number_reason = reason_id(&base_id, "save-number");
    let dial_described = described_by(&base_id, "dial");
    let dial_reason = reason_id(&base_id, "dial");
    Effect::new(move |previous: Option<String>| {
        // Both dependencies must be read even when the context changes.
        let context = state.with(|s| s.call.context_id.clone());
        let editable = state.with(ClientCallWorkspaceState::can_edit_destination);
        if previous.as_ref() != Some(&context) || !editable {
            keypad.set(false);
        }
        context
    });
    view! {
        <div class="flex min-w-0 flex-col gap-4" data-call-destination="true">
            <Field label=Signal::derive(move || Some(texts.get().destination))
                help_text={
                    let reason = control_reason(state, texts, ClientCallControl::Destination);
                    Signal::derive(move || reason.get().or_else(|| Some(texts.get().destination_hint)))
                }
                error=Signal::derive(move || state.get().number_error)
                state=Signal::derive(move || if state.get().number_error.is_some() { FieldState::Error } else { FieldState::Default })
                label_class="whitespace-normal">
                <Input input_type=InputType::Tel node_ref=input_ref class="w-full min-w-0 font-mono"
                    attr:data-call-field="destination" attr:inputmode="tel" attr:autocomplete="off" maxlength=Some(64)
                    attr:aria-label=control_name(state, texts, ClientCallControl::Destination)
                    value=Signal::derive(move || state.get().destination)
                    disabled=control_disabled(state, ClientCallControl::Destination)
                    on_input=Callback::new(move |value| {
                        emit(state, on_command, ClientCallAction::EditDestination(value));
                        if let Some(input) = input_ref.get_untracked() { input.set_value(&state.get_untracked().destination); }
                    }) />
            </Field>
            <div class="flex flex-col gap-2">
                <p class="text-sm font-medium">{move || texts.get().saved_numbers}</p>
                <Show when=move || state.get().call.client.phones.is_empty()>
                    <p class="text-sm text-base-content/75">{move || texts.get().no_numbers}</p>
                </Show>
                <div class="flex flex-col gap-2">
                    {move || phones.get().into_iter().enumerate().map(|(index, phone)| {
                        let action = ClientCallAction::ChooseSavedNumber(phone.id.clone());
                        let control = ClientCallControl::SavedNumber(phone.id.clone());
                        // Index-keyed: a phone id is host text and may not be a valid id token.
                        let key = format!("saved-number-{index}");
                        let own = local_reason(state, texts, control.clone());
                        // Bound before the view: the macro evaluates props ahead of spreads.
                        let name = control_name(state, texts, control.clone());
                        let disabled = control_disabled(state, control);
                        let described = described_by(&saved_base, &key);
                        let own_id = reason_id(&saved_base, &key);
                        view! {
                            <div class="flex min-w-0 flex-col gap-2">
                            <Button class="h-auto min-h-10 w-full justify-start whitespace-normal py-2 text-left" attr:data-call-saved-number=phone.id
                                attr:aria-label=name
                                attr:aria-describedby=described
                                disabled=disabled
                                on_click=Callback::new(move |_| emit(state, on_command, action.clone()))>
                                <span class="min-w-0 [overflow-wrap:anywhere]">{format!("{} · {}", phone.label, phone.number)}</span>
                            </Button>
                            <p id=own_id class="text-sm text-base-content/75 [overflow-wrap:anywhere]" data-call-number-blocked=phone.blocked_reason.is_some().then_some("true")
                                class:hidden=move || own.with(Option::is_none)>{move || own.get()}</p>
                            </div>
                        }
                    }).collect_view()}
                </div>
                <Button style=ButtonStyle::Ghost attr:data-call-action="keypad" attr:aria-controls=controls
                    attr:aria-expanded=move || keypad.get().to_string()
                    attr:aria-label=control_name(state, texts, ClientCallControl::Keypad)
                    attr:aria-describedby=destination_described
                    disabled=control_disabled(state, ClientCallControl::Keypad)
                    on_click=Callback::new(move |_| if state.with_untracked(ClientCallWorkspaceState::can_edit_destination) { keypad.update(|value| *value = !*value); })>
                    {move || texts.get().keypad}
                </Button>
                {reason_line(destination_reason, "destination", local_reason(state, texts, ClientCallControl::Keypad))}
                <Show when=move || keypad.get()>
                    <div id=keypad_id.clone() class="grid grid-cols-3 gap-2" data-call-pad="true">
                        {ClientCallControl::DIGITS.into_iter().map(|digit| view! {
                            <Button attr:data-call-digit=digit.to_string()
                                attr:aria-label=control_name(state, texts, ClientCallControl::Digit(digit))
                                attr:aria-describedby=pad_described.clone()
                                disabled=control_disabled(state, ClientCallControl::Digit(digit))
                                on_click=Callback::new(move |_| {
                                    let mut value = state.get_untracked().destination;
                                    value.push(digit);
                                    emit(state, on_command, ClientCallAction::EditDestination(value));
                                })>{digit.to_string()}</Button>
                        }).collect_view()}
                        <Button class="col-span-3" attr:data-call-action="backspace"
                            attr:aria-label=control_name(state, texts, ClientCallControl::Backspace)
                            attr:aria-describedby=backspace_described.clone()
                            disabled=control_disabled(state, ClientCallControl::Backspace)
                            on_click=Callback::new(move |_| {
                                let mut value = state.get_untracked().destination;
                                value.pop();
                                emit(state, on_command, ClientCallAction::EditDestination(value));
                            })>{move || texts.get().backspace}</Button>
                        {reason_line(pad_reason.clone(), "pad", local_reason(state, texts, ClientCallControl::Digit('0')))}
                        {reason_line(backspace_reason.clone(), "backspace", local_reason(state, texts, ClientCallControl::Backspace))}
                    </div>
                </Show>
            </div>
            <Show when=move || state.get().number_update.is_some()>
                <div class="flex min-w-0 flex-col gap-2 rounded-box border border-base-300 p-3" data-call-number-update="true">
                    <Show when=move || !targets.get().is_empty()>
                        <Field label=Signal::derive(move || Some(texts.get().number_target)) label_class="whitespace-normal"
                            help_text=control_reason(state, texts, ClientCallControl::NumberTarget)>
                            <Select node_ref=target_ref class="w-full min-w-0" attr:data-call-field="number-target"
                                label=Signal::derive(control_name(state, texts, ClientCallControl::NumberTarget))
                                value=Signal::derive(move || state.get().number_update.map(|u| u.field_id).unwrap_or_default())
                                options_revision=Signal::derive(move || format!("{:?}", targets.get()))
                                disabled=control_disabled(state, ClientCallControl::NumberTarget)
                                on_change=Callback::new(move |value| {
                                    emit(state, on_command, ClientCallAction::ChooseNumberTarget(value));
                                    if let Some(select) = target_ref.get_untracked() { select.set_value(&state.get_untracked().number_update.map(|u| u.field_id).unwrap_or_default()); }
                                })>
                                <option value="" disabled=true>{move || texts.get().number_target}</option>
                                {move || targets.get().into_iter().map(move |target| {
                                    let action = ClientCallAction::ChooseNumberTarget(target.field_id.clone());
                                    view! {
                                        <option value=target.field_id disabled=move || !state.with(|s| s.can_dispatch(&action))>{match target.blocked_reason { Some(ref reason) => format!("{}: {}", target.label, reason), None => target.label }}</option>
                                    }
                                }).collect_view()}
                            </Select>
                        </Field>
                    </Show>
                    <p class="text-sm font-medium [overflow-wrap:anywhere]">{move || state.get().number_update.and_then(|u| u.selected_target()).map(|t| t.label).unwrap_or_default()}</p>
                    <p class="text-sm text-base-content/75 [overflow-wrap:anywhere]" data-call-confirmed-number="true">{move || state.get().number_update.and_then(|u| u.selected_target()).map(|t| t.saved_number).unwrap_or_default()}</p>
                    <Button attr:data-call-action="save-number"
                        attr:aria-label=control_name(state, texts, ClientCallControl::SaveNumber)
                        attr:aria-describedby=save_number_described.clone()
                        disabled=control_disabled(state, ClientCallControl::SaveNumber)
                        on_click=Callback::new(move |_| {
                            let current = state.get_untracked();
                            if let Some(update) = current.number_update {
                                emit(state, on_command, ClientCallAction::SaveNumber { field_id: update.field_id, number: current.destination });
                            }
                        })>{move || if state.get().number_update.is_some_and(|u| u.pending) { texts.get().saving } else { texts.get().save_number }}</Button>
                    // The write's reason line leads with the host's blocked reasons
                    // (capability, then selected field) that used to be two bare lines.
                    {reason_line(save_number_reason.clone(), "save-number", local_reason(state, texts, ClientCallControl::SaveNumber))}
                    <p class="text-sm text-error [overflow-wrap:anywhere]" role="status">{move || state.get().number_update.and_then(|u| u.error)}</p>
                </div>
            </Show>
            <Button color=ButtonColor::Primary class="w-full" attr:data-call-action="dial"
                attr:aria-label=control_name(state, texts, ClientCallControl::Dial)
                attr:aria-describedby=dial_described
                disabled=control_disabled(state, ClientCallControl::Dial)
                on_click=Callback::new(move |_| emit(state, on_command, ClientCallAction::Dial { number: state.get_untracked().destination }))>
                {move || texts.get().call}
            </Button>
            // Leads with the host's dial_blocked_reason, the line's former content.
            {reason_line(dial_reason, "dial", local_reason(state, texts, ClientCallControl::Dial))}
        </div>
    }
}

#[component]
fn Guidance(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
    base_id: String,
) -> impl IntoView {
    let guidance = Signal::derive(move || state.get().guidance.unwrap_or_default());
    let regeneration_status = format!("{base_id}-regeneration-status");
    let regenerate_reason = reason_id(&base_id, "regenerate");
    // Context line, the independent status line, then the host's blocked reason.
    let regenerate_described = format!(
        "{} {} {}",
        context_reason_id(&base_id),
        regeneration_status,
        regenerate_reason
    );
    // Content, including a replaced beat with the same ID, is host-controlled.
    let beats = Memo::new(move |_| guidance.get().beats);
    view! {
        <aside class="flex min-w-0 flex-col gap-4 border-l-4 border-primary pl-4 [overflow-wrap:anywhere]"
            data-call-guidance="true" data-call-guidance-state=move || match guidance.get().state {
                ClientCallGuidanceState::Ready => "ready",
                ClientCallGuidanceState::Preparing { .. } => "preparing",
                ClientCallGuidanceState::Failed { .. } => "failed",
            }>
            <div class="flex min-w-0 flex-col gap-2">
                <h3 class="text-lg font-semibold">{move || guidance.get().title}</h3>
                <p class="whitespace-pre-wrap text-sm">{move || guidance.get().body}</p>
                <p class="text-xs text-base-content/75">{move || guidance.get().source}</p>
            </div>
            <div role="status" class="flex flex-col gap-2 text-sm" data-call-script-status="true">
                {move || match guidance.get().state {
                    ClientCallGuidanceState::Ready => String::new(),
                    ClientCallGuidanceState::Preparing { elapsed_secs, typical_secs } => {
                        let copy = texts.get();
                        format!("{} · {}: {} {} · {}: {} {}", copy.script_preparing, copy.script_elapsed, elapsed_secs, copy.script_seconds, copy.script_typical, typical_secs, copy.script_seconds)
                    }
                    ClientCallGuidanceState::Failed { reason } => format!("{}: {}", texts.get().script_failed, reason),
                }}
            </div>
            <Show when=move || matches!(guidance.get().state, ClientCallGuidanceState::Ready) && !beats.get().is_empty()>
                <ol class="flex min-w-0 flex-col gap-4" data-call-script-beats="true">
                    {move || beats.get().into_iter().enumerate().map(|(index, beat)| view! {
                        <li class="flex min-w-0 flex-col gap-2" data-call-beat=beat.id>
                            <h4 class="text-sm font-semibold">{format!("{}. {}", index + 1, beat.title)}</h4>
                            <p class="whitespace-pre-wrap text-sm">{beat.say}</p>
                            {beat.no_file_detail.then(|| view! { <p class="text-sm text-base-content/75" data-call-no-file-detail="true">{move || texts.get().script_no_file_detail}</p> })}
                        </li>
                    }).collect_view()}
                </ol>
            </Show>
            <Show when=move || guidance.get().regeneration.is_some()>
                <div class="flex min-w-0 flex-col gap-2" data-call-regeneration-state=move || guidance.get().regeneration.map(|r| r.state.as_str())>
                    <Button style=ButtonStyle::Outline class="h-auto min-h-10 whitespace-normal py-2" attr:data-call-action="regenerate"
                        attr:aria-label=control_name(state, texts, ClientCallControl::Regenerate)
                        attr:aria-describedby=regenerate_described.clone()
                        disabled=control_disabled(state, ClientCallControl::Regenerate)
                        on_click=Callback::new(move |_| emit(state, on_command, ClientCallAction::RegenerateGuidance))>
                        {move || texts.get().regenerate}
                    </Button>
                    <p id=regeneration_status.clone() role="status" class="text-sm">{move || guidance.get().regeneration.map(|r| texts.get().regeneration(r.state))}</p>
                    <p id=regenerate_reason.clone() class="text-sm text-base-content/75" data-call-disabled-reason="regenerate">{move || guidance.get().regeneration.and_then(|r| r.blocked_reason)}</p>
                    <p role="status" class="text-sm text-error">{move || guidance.get().regeneration.and_then(|r| r.error)}</p>
                </div>
            </Show>
        </aside>
    }
}

#[component]
fn WrapUp(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
    base_id: String,
) -> impl IntoView {
    let wrap = Signal::derive(move || state.get().wrap_up.unwrap_or_default());
    let locked = control_disabled(state, ClientCallControl::Outcome);
    // One reason serves all four draft fields (they share one lock); each
    // Field shows it as its help line, which the Field binds to the control.
    let field_reason = control_reason(state, texts, ClientCallControl::Outcome);
    let save_described = described_by(&base_id, "save-wrap-up");
    let save_reason = reason_id(&base_id, "save-wrap-up");
    let select_ref = NodeRef::<leptos::html::Select>::new();
    let notes_ref = NodeRef::<leptos::html::Textarea>::new();
    let duration_ref = NodeRef::<leptos::html::Input>::new();
    let follow_ref = NodeRef::<leptos::html::Textarea>::new();
    let duration_error = Signal::derive(move || {
        let value = wrap.get().duration_minutes;
        let value = value.trim();
        if !value.is_empty()
            && (!value.bytes().all(|c| c.is_ascii_digit())
                || value.parse::<u32>().ok().filter(|v| *v > 0).is_none())
        {
            Some(texts.get().duration_error)
        } else {
            None
        }
    });
    view! {
        <section class="flex min-w-0 flex-col gap-4" aria-label=move || texts.get().wrap_up data-call-wrap-up="true">
            <div class="flex flex-col gap-2">
                <h3 class="text-lg font-semibold">{move || texts.get().wrap_up}</h3>
                <p class="text-sm text-base-content/75">{move || texts.get().wrap_up_hint}</p>
            </div>
            <Field label=Signal::derive(move || Some(texts.get().outcome)) required=true label_class="whitespace-normal" help_text=field_reason>
                <Select class="w-full min-w-0" node_ref=select_ref attr:data-call-field="outcome" attr:required=true disabled=locked
                    label=Signal::derive(control_name(state, texts, ClientCallControl::Outcome))
                    value=Signal::derive(move || wrap.get().outcome.map(|o| o.as_str().to_owned()).unwrap_or_default())
                    on_change=Callback::new(move |value: String| {
                        if value.is_empty() || ClientCallOutcome::from_value(&value).is_some() {
                            emit(state, on_command, ClientCallAction::SetOutcome(ClientCallOutcome::from_value(&value)));
                        }
                        if let Some(select) = select_ref.get_untracked() { select.set_value(wrap.get_untracked().outcome.map(|o| o.as_str()).unwrap_or_default()); }
                    })>
                    <option value="">{move || texts.get().choose_outcome}</option>
                    {ClientCallOutcome::ALL.into_iter().map(move |outcome| view! { <option value=outcome.as_str()>{move || texts.get().outcome(outcome)}</option> }).collect_view()}
                </Select>
            </Field>
            <Field label=Signal::derive(move || Some(texts.get().notes)) label_class="whitespace-normal" help_text=field_reason>
                <Textarea node_ref=notes_ref class="w-full min-w-0" rows=Some(5) maxlength=Some(8_000) attr:data-call-field="notes"
                    label=Signal::derive(control_name(state, texts, ClientCallControl::Notes))
                    disabled=locked value=Signal::derive(move || wrap.get().notes)
                    on_input=Callback::new(move |value| {
                        emit(state, on_command, ClientCallAction::SetNotes(value));
                        if let Some(input) = notes_ref.get_untracked() { input.set_value(&wrap.get_untracked().notes); }
                    }) />
            </Field>
            <Field label=Signal::derive(move || Some(texts.get().duration)) error=duration_error help_text=field_reason
                state=Signal::derive(move || if duration_error.get().is_some() { FieldState::Error } else { FieldState::Default }) label_class="whitespace-normal">
                <Input node_ref=duration_ref class="w-full min-w-0" attr:inputmode="numeric" maxlength=Some(10) attr:data-call-field="duration"
                    attr:aria-label=control_name(state, texts, ClientCallControl::Duration)
                    disabled=locked value=Signal::derive(move || wrap.get().duration_minutes)
                    on_input=Callback::new(move |value| {
                        emit(state, on_command, ClientCallAction::SetDurationMinutes(value));
                        if let Some(input) = duration_ref.get_untracked() { input.set_value(&wrap.get_untracked().duration_minutes); }
                    }) />
            </Field>
            <Show when=move || wrap.get().outcome == Some(ClientCallOutcome::RequestedCallBack)>
                <Field label=Signal::derive(move || Some(texts.get().follow_up)) help_text=Signal::derive(move || field_reason.get().or_else(|| Some(texts.get().follow_up_hint))) required=true label_class="whitespace-normal">
                    <Textarea node_ref=follow_ref class="w-full min-w-0" rows=Some(3) maxlength=Some(1_000) attr:data-call-field="follow-up" required=true
                        label=Signal::derive(control_name(state, texts, ClientCallControl::FollowUp))
                        disabled=locked value=Signal::derive(move || wrap.get().follow_up)
                        on_input=Callback::new(move |value| {
                            emit(state, on_command, ClientCallAction::SetFollowUp(value));
                            if let Some(input) = follow_ref.get_untracked() { input.set_value(&wrap.get_untracked().follow_up); }
                        }) />
                </Field>
            </Show>
            <div class="flex flex-col gap-2">
                <p role="status" class="text-sm text-error [overflow-wrap:anywhere]">{move || wrap.get().error}</p>
                <Button color=ButtonColor::Primary class="w-full" attr:data-call-action="save-wrap-up"
                    attr:aria-label=control_name(state, texts, ClientCallControl::SaveWrapUp)
                    attr:aria-describedby=save_described
                    disabled=control_disabled(state, ClientCallControl::SaveWrapUp)
                    on_click=Callback::new(move |_| {
                        if let Some(payload) = wrap.get_untracked().payload() { emit(state, on_command, ClientCallAction::SaveWrapUp(payload)); }
                    })>{move || { let draft = wrap.get(); let copy = texts.get(); if draft.saved { copy.saved } else if draft.pending { copy.saving } else { copy.save_wrap_up } }}</Button>
                // Carries the finish hint (formerly a bare line above the button)
                // whenever an unfinished call is the reason Save is disabled.
                {reason_line(save_reason, "save-wrap-up", local_reason(state, texts, ClientCallControl::SaveWrapUp))}
            </div>
        </section>
    }
}

/// A complete controlled client-calling workspace with explicit dispatch and save evidence.
///
/// Use inline or inside the host's Modal. The host owns authorization, operation
/// correlation, telephony, dismissal and draft persistence. See the component guide.
#[component]
pub fn ClientCallWorkspace(
    /// Unique region ID; also prefixes the nested call console and number pad IDs.
    #[prop(into)]
    id: String,
    /// Atomic caller-owned projection for one client and attempt.
    #[prop(into)]
    state: Signal<ClientCallWorkspaceState>,
    /// Receives requests; synchronously adopt draft edits or reserve pending state.
    on_command: Callback<ClientCallCommand>,
    /// Reactive translated labels, including the nested live console.
    #[prop(default = Signal::stored(ClientCallWorkspaceTexts::default()), into)]
    texts: Signal<ClientCallWorkspaceTexts>,
    /// Optional deterministic or shared epoch clock passed to the live console.
    #[prop(optional)]
    now_ms: Option<Signal<i64>>,
    /// Additional classes on the bounded region.
    #[prop(optional)]
    class: &'static str,
) -> impl IntoView {
    let keypad_id = format!("{id}-destination-pad");
    let console_id = format!("{id}-session");
    // Reason-line ids share the region id so two workspaces never collide.
    let base_id = id.clone();
    let destination_base = base_id.clone();
    let guidance_base = base_id.clone();
    let wrap_base = base_id.clone();
    let context_reason = context_reason_id(&base_id);
    let dismiss_described = described_by(&base_id, "dismiss");
    let dismiss_reason = reason_id(&base_id, "dismiss");
    let now = now_ms.unwrap_or_else(|| {
        #[cfg(target_arch = "wasm32")]
        {
            crate::components::use_sla_now(1_000)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Signal::stored(0)
        }
    });
    let session_command = Callback::new(move |command: SoftphoneCommand| {
        if command.context_id == state.get_untracked().call.context_id {
            emit(state, on_command, ClientCallAction::Session(command.action));
        }
    });
    view! {
        <section id=id aria-label=move || texts.get().label data-client-call-workspace="true"
            data-call-attempt=move || state.get().attempt.as_str()
            class=crate::merge_classes!("@container w-full min-w-0 max-w-5xl rounded-box border border-base-300 bg-base-100 text-base-content [&_.label]:gap-2 [&_.label]:whitespace-normal [&_.label]:[overflow-wrap:anywhere] [&_.btn]:gap-2 [&_.input]:shadow-none [&_.select]:shadow-none [&_.textarea]:shadow-none", class)>
            <header class="flex items-start justify-between gap-4 border-b border-base-300 p-5">
                <div class="flex min-w-0 flex-col gap-2 [overflow-wrap:anywhere]">
                    <p class="text-sm font-medium text-base-content/75">{move || texts.get().label}</p>
                    <h2 class="text-xl font-semibold">{move || state.get().call.client.name}</h2>
                    <p class="text-sm text-base-content/75">{move || state.get().call.client.subtitle}</p>
                </div>
                <div class="flex shrink-0 flex-col items-end gap-2">
                    <Button style=ButtonStyle::Ghost class="shrink-0" attr:data-call-action="dismiss"
                        attr:aria-label=control_name(state, texts, ClientCallControl::Dismiss)
                        attr:aria-describedby=dismiss_described
                        disabled=control_disabled(state, ClientCallControl::Dismiss)
                        on_click=Callback::new(move |_| emit(state, on_command, ClientCallAction::Dismiss))>{move || texts.get().close}</Button>
                    {reason_line(dismiss_reason, "dismiss", local_reason(state, texts, ClientCallControl::Dismiss))}
                </div>
            </header>
            // One shared line for the state that disables every control at once:
            // no call context from the host (Office op-stjm9). Each button's
            // aria-describedby names it first.
            <p id=context_reason role="note" class="border-b border-base-300 px-5 py-3 text-sm text-base-content/75 [overflow-wrap:anywhere]"
                data-call-disabled-reason="context"
                class:hidden=move || !state.with(ClientCallWorkspaceState::context_missing)>
                {move || state.with(ClientCallWorkspaceState::context_missing).then(|| texts.get().not_ready)}
            </p>
            <div class="grid min-w-0 gap-6 p-5 @3xl:grid-cols-2">
                <div class="flex min-w-0 flex-col gap-4">
                    <div role="status" class="flex flex-col gap-2 rounded-box bg-base-200 p-4 [overflow-wrap:anywhere]" data-call-status="true">
                        <p class="text-base font-semibold">{move || { let current = state.get(); if current.attempt == ClientCallAttempt::Managed { texts.get().softphone.phase(current.call.phase) } else { texts.get().attempt(current.attempt) } }}</p>
                        <Show when=move || state.get().attempt == ClientCallAttempt::AgentRinging><p class="text-sm">{move || texts.get().bridge_hint}</p></Show>
                        <Show when=move || state.get().attempt == ClientCallAttempt::Uncertain><p class="text-sm">{move || texts.get().uncertain_hint}</p></Show>
                        <p class="text-sm">{move || state.get().status_detail}</p>
                        <p class="text-sm" data-call-talk-time="true">{move || {
                            let copy = texts.get();
                            let time = state.get().provider_talk_seconds.map(crate::components::format_softphone_duration).unwrap_or(copy.provider_talk_unknown);
                            format!("{}: {}", copy.provider_talk_time, time)
                        }}</p>
                        <Show when=move || state.with(|s| s.finished() && matches!(s.call.timer, crate::components::SoftphoneTimer::Stopped { .. }))>
                            <p class="text-sm" data-call-final-duration="true">{move || state.with(|s| {
                                let seconds = s.call.timer.elapsed_at(now.get()).unwrap_or(0);
                                format!("{}: {}", texts.get().softphone.duration, crate::components::format_softphone_duration(seconds))
                            })}</p>
                        </Show>
                    </div>
                    <Show when=move || state.with(|s| s.attempt == ClientCallAttempt::Managed && s.call.phase.is_live())
                        fallback=move || view! { <Destination state=state texts=texts on_command=on_command keypad_id=keypad_id.clone() base_id=destination_base.clone() /> }>
                        <Softphone id=console_id.clone() state=Signal::derive(move || state.get().call)
                            texts=Signal::derive(move || texts.get().softphone) on_command=session_command
                            now_ms=now class="max-w-none" />
                    </Show>
                </div>
                <div class="flex min-w-0 flex-col gap-6">
                    <Show when=move || state.get().guidance.is_some()>
                        <Guidance state=state texts=texts on_command=on_command base_id=guidance_base.clone() />
                    </Show>
                    <Show when=move || state.get().wrap_up.is_some()><WrapUp state=state texts=texts on_command=on_command base_id=wrap_base.clone() /></Show>
                </div>
            </div>
        </section>
    }
}
