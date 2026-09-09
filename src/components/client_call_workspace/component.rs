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

#[component]
fn Destination(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
    keypad_id: String,
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
                help_text=Signal::derive(move || Some(texts.get().destination_hint))
                error=Signal::derive(move || state.get().number_error)
                state=Signal::derive(move || if state.get().number_error.is_some() { FieldState::Error } else { FieldState::Default })
                label_class="whitespace-normal">
                <Input input_type=InputType::Tel node_ref=input_ref class="w-full min-w-0 font-mono"
                    attr:data-call-field="destination" attr:inputmode="tel" attr:autocomplete="off" maxlength=Some(64)
                    value=Signal::derive(move || state.get().destination)
                    disabled=Signal::derive(move || !state.with(ClientCallWorkspaceState::can_edit_destination))
                    on_input=Callback::new(move |value| {
                        emit(state, on_command, ClientCallAction::EditDestination(value));
                        if let Some(input) = input_ref.get_untracked() { input.set_value(&state.get_untracked().destination); }
                    }) />
            </Field>
            <div class="flex flex-col gap-2">
                <p class="text-sm font-medium">{move || texts.get().saved_numbers}</p>
                <Show when=move || state.get().call.client.phones.is_empty()>
                    <p class="text-sm text-base-content/70">{move || texts.get().no_numbers}</p>
                </Show>
                <div class="flex flex-col gap-2">
                    {move || phones.get().into_iter().map(|phone| {
                        let action = ClientCallAction::ChooseSavedNumber(phone.id.clone());
                        let for_disabled = action.clone();
                        view! {
                            <div class="flex min-w-0 flex-col gap-2">
                            <Button class="h-auto min-h-10 w-full justify-start whitespace-normal py-2 text-left" attr:data-call-saved-number=phone.id
                                disabled=Signal::derive(move || !state.with(|s| s.can_dispatch(&for_disabled)))
                                on_click=Callback::new(move |_| emit(state, on_command, action.clone()))>
                                <span class="min-w-0 [overflow-wrap:anywhere]">{format!("{} · {}", phone.label, phone.number)}</span>
                            </Button>
                            {phone.blocked_reason.map(|reason| view! { <p class="text-sm text-base-content/70 [overflow-wrap:anywhere]" data-call-number-blocked="true">{reason}</p> })}
                            </div>
                        }
                    }).collect_view()}
                </div>
                <Button style=ButtonStyle::Ghost attr:data-call-action="keypad" attr:aria-controls=controls
                    attr:aria-expanded=move || keypad.get().to_string()
                    disabled=Signal::derive(move || !state.with(ClientCallWorkspaceState::can_edit_destination))
                    on_click=Callback::new(move |_| if state.with_untracked(ClientCallWorkspaceState::can_edit_destination) { keypad.update(|value| *value = !*value); })>
                    {move || texts.get().keypad}
                </Button>
                <Show when=move || keypad.get()>
                    <div id=keypad_id.clone() class="grid grid-cols-3 gap-2" data-call-pad="true">
                        {['1','2','3','4','5','6','7','8','9','*','0','#'].into_iter().map(move |digit| view! {
                            <Button attr:data-call-digit=digit.to_string()
                                disabled=Signal::derive(move || !state.with(|s| s.can_edit_destination() && s.destination.chars().count() < 64))
                                on_click=Callback::new(move |_| {
                                    let mut value = state.get_untracked().destination;
                                    value.push(digit);
                                    emit(state, on_command, ClientCallAction::EditDestination(value));
                                })>{digit.to_string()}</Button>
                        }).collect_view()}
                        <Button class="col-span-3" attr:data-call-action="backspace"
                            disabled=Signal::derive(move || !state.with(|s| s.can_edit_destination() && !s.destination.is_empty()))
                            on_click=Callback::new(move |_| {
                                let mut value = state.get_untracked().destination;
                                value.pop();
                                emit(state, on_command, ClientCallAction::EditDestination(value));
                            })>{move || texts.get().backspace}</Button>
                    </div>
                </Show>
            </div>
            <Show when=move || state.get().number_update.is_some()>
                <div class="flex min-w-0 flex-col gap-2 rounded-box border border-base-300 p-3" data-call-number-update="true">
                    <Show when=move || !targets.get().is_empty()>
                        <Field label=Signal::derive(move || Some(texts.get().number_target)) label_class="whitespace-normal">
                            <Select node_ref=target_ref class="w-full min-w-0" attr:data-call-field="number-target"
                                value=Signal::derive(move || state.get().number_update.map(|u| u.field_id).unwrap_or_default())
                                options_revision=Signal::derive(move || format!("{:?}", targets.get()))
                                disabled=Signal::derive(move || !state.with(ClientCallWorkspaceState::can_edit_destination) || state.get().number_update.is_some_and(|u| u.blocked_reason.is_some()))
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
                    <p class="text-sm text-base-content/70 [overflow-wrap:anywhere]" data-call-confirmed-number="true">{move || state.get().number_update.and_then(|u| u.selected_target()).map(|t| t.saved_number).unwrap_or_default()}</p>
                    <Button attr:data-call-action="save-number"
                        disabled=Signal::derive(move || !state.with(|s| s.number_update.as_ref().is_some_and(|u| s.can_dispatch(&ClientCallAction::SaveNumber { field_id: u.field_id.clone(), number: s.destination.clone() }))))
                        on_click=Callback::new(move |_| {
                            let current = state.get_untracked();
                            if let Some(update) = current.number_update {
                                emit(state, on_command, ClientCallAction::SaveNumber { field_id: update.field_id, number: current.destination });
                            }
                        })>{move || if state.get().number_update.is_some_and(|u| u.pending) { texts.get().saving } else { texts.get().save_number }}</Button>
                    <p class="text-sm text-base-content/70 [overflow-wrap:anywhere]">{move || state.get().number_update.and_then(|u| u.blocked_reason)}</p>
                    <p class="text-sm text-base-content/70 [overflow-wrap:anywhere]">{move || state.get().number_update.and_then(|u| u.selected_target()).and_then(|t| t.blocked_reason)}</p>
                    <p class="text-sm text-error [overflow-wrap:anywhere]" role="status">{move || state.get().number_update.and_then(|u| u.error)}</p>
                </div>
            </Show>
            <Button color=ButtonColor::Primary class="w-full" attr:data-call-action="dial"
                disabled=Signal::derive(move || !state.with(|s| s.can_dispatch(&ClientCallAction::Dial { number: s.destination.clone() })))
                on_click=Callback::new(move |_| emit(state, on_command, ClientCallAction::Dial { number: state.get_untracked().destination }))>
                {move || texts.get().call}
            </Button>
            <p class="text-sm text-base-content/70 [overflow-wrap:anywhere]">{move || state.get().dial_blocked_reason}</p>
        </div>
    }
}

#[component]
fn Guidance(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
) -> impl IntoView {
    let guidance = Signal::derive(move || state.get().guidance.unwrap_or_default());
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
                <p class="text-xs text-base-content/70">{move || guidance.get().source}</p>
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
                            {beat.no_file_detail.then(|| view! { <p class="text-sm text-base-content/70" data-call-no-file-detail="true">{move || texts.get().script_no_file_detail}</p> })}
                        </li>
                    }).collect_view()}
                </ol>
            </Show>
            <Show when=move || guidance.get().regeneration.is_some()>
                <div class="flex min-w-0 flex-col gap-2" data-call-regeneration-state=move || guidance.get().regeneration.map(|r| r.state.as_str())>
                    <Button style=ButtonStyle::Outline class="h-auto min-h-10 whitespace-normal py-2" attr:data-call-action="regenerate"
                        disabled=Signal::derive(move || !state.with(|s| s.can_dispatch(&ClientCallAction::RegenerateGuidance)))
                        on_click=Callback::new(move |_| emit(state, on_command, ClientCallAction::RegenerateGuidance))>
                        {move || texts.get().regenerate}
                    </Button>
                    <p role="status" class="text-sm">{move || guidance.get().regeneration.map(|r| texts.get().regeneration(r.state))}</p>
                    <p class="text-sm text-base-content/70">{move || guidance.get().regeneration.and_then(|r| r.blocked_reason)}</p>
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
) -> impl IntoView {
    let wrap = Signal::derive(move || state.get().wrap_up.unwrap_or_default());
    let locked = Signal::derive(move || {
        !state.with(|s| s.can_dispatch(&ClientCallAction::SetOutcome(None)))
    });
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
                <p class="text-sm text-base-content/70">{move || texts.get().wrap_up_hint}</p>
            </div>
            <Field label=Signal::derive(move || Some(texts.get().outcome)) required=true label_class="whitespace-normal">
                <Select class="w-full min-w-0" node_ref=select_ref attr:data-call-field="outcome" attr:required=true disabled=locked
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
            <Field label=Signal::derive(move || Some(texts.get().notes)) label_class="whitespace-normal">
                <Textarea node_ref=notes_ref class="w-full min-w-0" rows=Some(5) maxlength=Some(8_000) attr:data-call-field="notes"
                    disabled=locked value=Signal::derive(move || wrap.get().notes)
                    on_input=Callback::new(move |value| {
                        emit(state, on_command, ClientCallAction::SetNotes(value));
                        if let Some(input) = notes_ref.get_untracked() { input.set_value(&wrap.get_untracked().notes); }
                    }) />
            </Field>
            <Field label=Signal::derive(move || Some(texts.get().duration)) error=duration_error
                state=Signal::derive(move || if duration_error.get().is_some() { FieldState::Error } else { FieldState::Default }) label_class="whitespace-normal">
                <Input node_ref=duration_ref class="w-full min-w-0" attr:inputmode="numeric" maxlength=Some(10) attr:data-call-field="duration"
                    disabled=locked value=Signal::derive(move || wrap.get().duration_minutes)
                    on_input=Callback::new(move |value| {
                        emit(state, on_command, ClientCallAction::SetDurationMinutes(value));
                        if let Some(input) = duration_ref.get_untracked() { input.set_value(&wrap.get_untracked().duration_minutes); }
                    }) />
            </Field>
            <Show when=move || wrap.get().outcome == Some(ClientCallOutcome::RequestedCallBack)>
                <Field label=Signal::derive(move || Some(texts.get().follow_up)) help_text=Signal::derive(move || Some(texts.get().follow_up_hint)) required=true label_class="whitespace-normal">
                    <Textarea node_ref=follow_ref class="w-full min-w-0" rows=Some(3) maxlength=Some(1_000) attr:data-call-field="follow-up" required=true
                        disabled=locked value=Signal::derive(move || wrap.get().follow_up)
                        on_input=Callback::new(move |value| {
                            emit(state, on_command, ClientCallAction::SetFollowUp(value));
                            if let Some(input) = follow_ref.get_untracked() { input.set_value(&wrap.get_untracked().follow_up); }
                        }) />
                </Field>
            </Show>
            <div class="flex flex-col gap-2">
                <p role="status" class="text-sm text-error [overflow-wrap:anywhere]">{move || wrap.get().error}</p>
                <Show when=move || !state.with(ClientCallWorkspaceState::finished)>
                    <p class="text-sm text-base-content/70">{move || texts.get().finish_hint}</p>
                </Show>
                <Button color=ButtonColor::Primary class="w-full" attr:data-call-action="save-wrap-up"
                    disabled=Signal::derive(move || !state.with(|s| s.wrap_up.as_ref().and_then(ClientCallWrapUp::payload).is_some_and(|payload| s.can_dispatch(&ClientCallAction::SaveWrapUp(payload)))))
                    on_click=Callback::new(move |_| {
                        if let Some(payload) = wrap.get_untracked().payload() { emit(state, on_command, ClientCallAction::SaveWrapUp(payload)); }
                    })>{move || { let draft = wrap.get(); let copy = texts.get(); if draft.saved { copy.saved } else if draft.pending { copy.saving } else { copy.save_wrap_up } }}</Button>
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
                    <p class="text-sm font-medium text-base-content/70">{move || texts.get().label}</p>
                    <h2 class="text-xl font-semibold">{move || state.get().call.client.name}</h2>
                    <p class="text-sm text-base-content/70">{move || state.get().call.client.subtitle}</p>
                </div>
                <Button style=ButtonStyle::Ghost class="shrink-0" attr:data-call-action="dismiss"
                    disabled=Signal::derive(move || !state.with(|s| s.can_dispatch(&ClientCallAction::Dismiss)))
                    on_click=Callback::new(move |_| emit(state, on_command, ClientCallAction::Dismiss))>{move || texts.get().close}</Button>
            </header>
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
                        fallback=move || view! { <Destination state=state texts=texts on_command=on_command keypad_id=keypad_id.clone() /> }>
                        <Softphone id=console_id.clone() state=Signal::derive(move || state.get().call)
                            texts=Signal::derive(move || texts.get().softphone) on_command=session_command
                            now_ms=now class="max-w-none" />
                    </Show>
                </div>
                <div class="flex min-w-0 flex-col gap-6">
                    <Show when=move || state.get().guidance.is_some()>
                        <Guidance state=state texts=texts on_command=on_command />
                    </Show>
                    <Show when=move || state.get().wrap_up.is_some()><WrapUp state=state texts=texts on_command=on_command /></Show>
                </div>
            </div>
        </section>
    }
}
