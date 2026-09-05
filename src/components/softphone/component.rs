use super::*;
use crate::components::{Button, ButtonColor, ButtonStyle, Persona, PersonaSize, Select};
use leptos::prelude::*;

fn action_for(kind: SoftphoneActionKind, state: &SoftphoneState) -> SoftphoneAction {
    match kind {
        SoftphoneActionKind::Mute => SoftphoneAction::SetMuted(!state.muted),
        SoftphoneActionKind::Hold => SoftphoneAction::SetHeld(state.phase != SoftphonePhase::Held),
        SoftphoneActionKind::Voicemail => SoftphoneAction::RouteToVoicemail,
        SoftphoneActionKind::Record => SoftphoneAction::SetRecording(!state.recording),
        SoftphoneActionKind::Transcribe => SoftphoneAction::SetTranscribing(!state.transcribing),
        SoftphoneActionKind::EndCall => SoftphoneAction::EndCall,
        SoftphoneActionKind::Call => SoftphoneAction::Call {
            phone_id: state
                .selected_number()
                .map(|phone| phone.id.clone())
                .unwrap_or_default(),
        },
        // Keypad visibility uses the same permission as sending a valid digit.
        SoftphoneActionKind::Keypad => SoftphoneAction::SendDigit('0'),
        SoftphoneActionKind::SelectNumber => SoftphoneAction::SelectNumber(String::new()),
    }
}

fn supports(state: &SoftphoneState, kind: SoftphoneActionKind) -> bool {
    match kind {
        SoftphoneActionKind::Mute => state.capabilities.mute,
        SoftphoneActionKind::Hold => state.capabilities.hold,
        SoftphoneActionKind::Voicemail => state.capabilities.voicemail,
        SoftphoneActionKind::Record => state.capabilities.recording,
        SoftphoneActionKind::Transcribe => state.capabilities.transcription,
        SoftphoneActionKind::Keypad => state.capabilities.keypad,
        _ => true,
    }
}

fn pressed(state: &SoftphoneState, kind: SoftphoneActionKind) -> Option<bool> {
    match kind {
        SoftphoneActionKind::Mute => Some(state.muted),
        SoftphoneActionKind::Hold => Some(state.phase == SoftphonePhase::Held),
        SoftphoneActionKind::Record => Some(state.recording),
        SoftphoneActionKind::Transcribe => Some(state.transcribing),
        _ => None,
    }
}

fn emit(
    state: Signal<SoftphoneState>,
    on_command: Callback<SoftphoneCommand>,
    action: SoftphoneAction,
) {
    let current = state.get_untracked();
    if current.can_dispatch(&action) {
        on_command.run(SoftphoneCommand {
            context_id: current.context_id,
            action,
        });
    }
}

#[component]
fn CallGlyph(kind: SoftphoneActionKind) -> impl IntoView {
    let path = match kind {
        SoftphoneActionKind::Mute => {
            "M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3ZM5 10v2a7 7 0 0 0 14 0v-2M12 19v3M8 22h8"
        }
        SoftphoneActionKind::Hold => "M8 4v16M16 4v16",
        SoftphoneActionKind::Voicemail => {
            "M9 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0Zm14 0a4 4 0 1 1-8 0 4 4 0 0 1 8 0ZM5 16h14"
        }
        SoftphoneActionKind::Record => {
            "M20 12a8 8 0 1 1-16 0 8 8 0 0 1 16 0ZM15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
        }
        SoftphoneActionKind::Transcribe => "M4 4h16v16H4ZM7 8h10M7 12h10M7 16h6",
        SoftphoneActionKind::Keypad => {
            "M5 5h1M11 5h1M17 5h1M5 11h1M11 11h1M17 11h1M5 17h1M11 17h1M17 17h1"
        }
        SoftphoneActionKind::EndCall => "M4 16v-4c4-5 12-5 16 0v4l-5-1v-3M4 16l5-1v-3",
        _ => "M5 3h4l2 5-3 2c2 3 3 4 6 6l2-3 5 2v4c0 2-2 3-4 2C8 18 4 14 2 7 1 5 3 3 5 3Z",
    };
    view! {
        <svg aria-hidden="true" viewBox="0 0 24 24" class="h-5 w-5 shrink-0" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d=path /></svg>
    }
}

fn browser_clock() -> Signal<i64> {
    #[cfg(target_arch = "wasm32")]
    {
        let now = RwSignal::new(js_sys::Date::now() as i64);
        if let Ok(handle) = leptos::leptos_dom::helpers::set_interval_with_handle(
            move || {
                now.try_set(js_sys::Date::now() as i64);
            },
            std::time::Duration::from_secs(1),
        ) {
            on_cleanup(move || handle.clear());
        }
        now.into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Signal::stored(0)
    }
}

/// An opinionated, controlled softphone UI. It requests actions; the host confirms state.
///
/// Mount once in the application's persistent UI. No telephony SDK, microphone,
/// storage or routing behavior is invoked. See `doc/components/softphone.md`.
#[component]
pub fn Softphone(
    /// Unique DOM identity for this instance and its number/keypad controls.
    #[prop(into)]
    id: String,
    /// One atomic caller-owned projection. Update it when the host confirms an operation.
    #[prop(into)]
    state: Signal<SoftphoneState>,
    /// Receives guarded commands with the current context identity.
    on_command: Callback<SoftphoneCommand>,
    /// Reactive localized copy, including status and accessible labels.
    #[prop(default = Signal::stored(SoftphoneTexts::default()), into)]
    texts: Signal<SoftphoneTexts>,
    /// Epoch milliseconds for deterministic tests or a shared application clock.
    /// Omit for a once-per-second browser clock. Native rendering uses zero.
    #[prop(optional)]
    now_ms: Option<Signal<i64>>,
    /// Additional classes on the bounded outer surface.
    #[prop(optional)]
    class: &'static str,
) -> impl IntoView {
    let now = now_ms.unwrap_or_else(browser_clock);
    let number_id = format!("{id}-number");
    let keypad_id = format!("{id}-keypad");
    let keypad_controls = keypad_id.clone();
    let show_keypad = RwSignal::new(false);
    let select_ref = NodeRef::<leptos::html::Select>::new();
    let selected = Signal::derive(move || {
        state.with(|s| {
            s.selected_number()
                .map(|p| p.id.clone())
                .unwrap_or_default()
        })
    });
    let number_locked =
        Signal::derive(move || state.with(|s| s.phase.is_live() || s.pending.is_some()));
    let revision = Signal::derive(move || state.with(|s| format!("{:?}", s.client.phones)));
    let keyed_context = Memo::new(move |_| state.with(|s| s.context_id.clone()));
    Effect::new(move |previous: Option<String>| {
        let context = keyed_context.get();
        let unavailable =
            state.with(|s| s.phase != SoftphonePhase::Active || !s.capabilities.keypad);
        if previous.as_ref() != Some(&context) || unavailable {
            show_keypad.set(false);
        }
        context
    });
    view! {
        <section id=id data-softphone="true" data-softphone-phase=move || state.get().phase.as_str()
            aria-label=move || texts.get().label
            class=crate::merge_classes!("w-full max-w-md overflow-hidden rounded-box border border-base-300 bg-base-100 text-base-content", class)>
            <div class="flex flex-col gap-4 p-5">
                <Persona name=Signal::derive(move || state.get().client.name)
                    secondary_text=Signal::derive(move || state.get().client.subtitle)
                    size=PersonaSize::Large palette=true class="min-w-0 [&>div:last-child]:min-w-0 [&>div:last-child]:[overflow-wrap:anywhere] [&>.avatar]:shrink-0" />
                <div class="flex min-w-0 flex-col gap-2">
                    <span class="text-sm font-medium">{move || texts.get().phone_number}</span>
                    <Show when=move || { state.get().client.phones.len() > 1 } fallback=move || view! {
                        <p data-softphone-number="true" class="break-words text-base font-medium">{move || state.with(|s| s.selected_number().map(|p| format!("{}: {}", p.label, p.number)).unwrap_or_else(|| texts.get().no_number))}</p>
                    }>
                            <Select id=number_id.clone() node_ref=select_ref class="w-full" value=selected
                                options_revision=revision disabled=number_locked label=Signal::derive(move || texts.get().phone_number)
                                on_change=Callback::new(move |value: String| {
                                    emit(state, on_command, SoftphoneAction::SelectNumber(value));
                                    if let Some(select) = select_ref.get() { select.set_value(&selected.get_untracked()); }
                                })>
                                <option value="">{move || texts.get().choose_number}</option>
                                {move || state.get().client.phones.into_iter().map(|p| view! {
                                    <option value=p.id>{format!("{}: {}", p.label, p.number)}</option>
                                }).collect_view()}
                            </Select>
                    </Show>
                </div>
            </div>
            <div class="flex flex-col gap-2 border-y border-base-300 bg-base-200 p-5">
                <div role="status" aria-live="polite" aria-atomic="true" data-softphone-status="true" class="flex items-center gap-2 text-sm font-medium">
                    <span aria-hidden="true" class="h-2 w-2 rounded-full bg-current" />
                    {move || texts.get().phase(state.get().phase)}
                </div>
                <span role="timer" aria-live="off" aria-label=move || texts.get().duration data-softphone-timer="true" class="text-2xl font-semibold tabular-nums tracking-tight">
                    {move || state.get().timer.elapsed_at(now.get()).map(format_softphone_duration).unwrap_or_else(|| texts.get().not_started)}
                </span>
                <div class="flex flex-wrap gap-2 text-sm">
                    <Show when=move || state.get().recording><span data-softphone-recording="true" class="badge badge-error badge-outline">{move || texts.get().recording}</span></Show>
                    <Show when=move || state.get().transcribing><span data-softphone-transcribing="true" class="badge badge-primary badge-outline">{move || texts.get().transcribing}</span></Show>
                </div>
            </div>
            <div class="flex flex-col gap-4 p-5">
                <div class="grid grid-cols-2 gap-2" data-softphone-actions="true">
                    {[SoftphoneActionKind::Mute, SoftphoneActionKind::Hold, SoftphoneActionKind::Voicemail,
                      SoftphoneActionKind::Record, SoftphoneActionKind::Transcribe, SoftphoneActionKind::Keypad]
                        .into_iter().map(|kind| { let keypad_controls = keypad_controls.clone(); view! {
                        <Show when=move || state.with(|s| supports(s, kind))>
                            <Button style=ButtonStyle::Outline class="h-auto min-h-16 min-w-0 flex-col gap-2 whitespace-normal border-base-300 px-2 py-3 text-center shadow-none"
                                disabled=Signal::derive(move || state.with(|s| !s.can_dispatch(&action_for(kind, s))))
                                active=Signal::derive(move || state.with(|s| pressed(s, kind).unwrap_or(false)))
                                attr:data-softphone-action=kind.as_str()
                                attr:aria-pressed=move || state.with(|s| pressed(s, kind).map(|v| v.to_string()))
                                attr:aria-expanded=move || (kind == SoftphoneActionKind::Keypad).then(|| show_keypad.get().to_string())
                                attr:aria-controls=(kind == SoftphoneActionKind::Keypad).then(|| keypad_controls.clone())
                                on_click=Callback::new(move |_| {
                                    let current = state.get_untracked();
                                    let action = action_for(kind, &current);
                                    if current.can_dispatch(&action) {
                                        if kind == SoftphoneActionKind::Keypad { show_keypad.update(|v| *v = !*v); }
                                        else { emit(state, on_command, action); }
                                    }
                                })>
                                <CallGlyph kind=kind />
                                <span class="break-words text-sm">{move || texts.get().action(kind, &state.get())}</span>
                            </Button>
                        </Show>
                    }}).collect_view()}
                </div>
                <Show when=move || show_keypad.get()>
                    <div id=keypad_id.clone() role="group" aria-label=move || texts.get().keypad class="grid grid-cols-3 gap-2" data-softphone-keypad="true">
                        {"123456789*0#".chars().map(|digit| view! {
                            <Button class="min-h-12 text-lg" attr:data-softphone-digit=digit.to_string()
                                attr:aria-label=move || texts.get().digit.replace("{digit}", &digit.to_string())
                                disabled=Signal::derive(move || !state.get().can_dispatch(&SoftphoneAction::SendDigit(digit)))
                                on_click=Callback::new(move |_| emit(state, on_command, SoftphoneAction::SendDigit(digit)))>{digit.to_string()}</Button>
                        }).collect_view()}
                    </div>
                </Show>
                <Show when=move || state.get().pending.is_some()>
                    <p role="status" data-softphone-pending="true" class="text-sm">{move || {
                        let s = state.get(); let t = texts.get();
                        s.pending.map(|kind| format!("{}: {}", t.pending, t.action(kind, &s))).unwrap_or_default()
                    }}</p>
                </Show>
                <Show when=move || state.get().error.is_some()>
                    <p role="alert" data-softphone-error="true" class="rounded-box border border-error p-3 text-sm text-error">{move || state.get().error.unwrap_or_default()}</p>
                </Show>
                {move || {
                    let kind = if state.get().phase.is_live() { SoftphoneActionKind::EndCall } else { SoftphoneActionKind::Call };
                    view! {
                        <Button color=if kind == SoftphoneActionKind::EndCall { ButtonColor::Error } else { ButtonColor::Primary }
                            class="min-h-12 w-full shadow-none" attr:data-softphone-action=kind.as_str()
                            disabled=Signal::derive(move || state.with(|s| !s.can_dispatch(&action_for(kind, s))))
                            on_click=Callback::new(move |_| emit(state, on_command, action_for(kind, &state.get_untracked())))>
                            <CallGlyph kind=kind />{move || texts.get().action(kind, &state.get())}
                        </Button>
                    }
                }}
            </div>
        </section>
    }
}
