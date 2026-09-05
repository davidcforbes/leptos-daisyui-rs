use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

fn initial_state() -> SoftphoneState {
    SoftphoneState {
        context_id: "client-elena/interaction-1".into(),
        client: SoftphoneClient {
            name: "Elena Martinez".into(),
            subtitle: "Client · Account review".into(),
            phones: vec![
                SoftphoneNumber {
                    id: "mobile".into(),
                    label: "Mobile".into(),
                    number: "+1 (415) 555-0142".into(),
                },
                SoftphoneNumber {
                    id: "office".into(),
                    label: "Office".into(),
                    number: "+1 (415) 555-0186".into(),
                },
            ],
        },
        selected_phone_id: Some("mobile".into()),
        capabilities: SoftphoneCapabilities {
            mute: true,
            hold: true,
            voicemail: true,
            recording: true,
            transcription: true,
            keypad: true,
        },
        ..Default::default()
    }
}

#[component]
pub fn SoftphoneDemo() -> impl IntoView {
    let state = RwSignal::new(initial_state());
    let pending = RwSignal::new(None::<SoftphoneAction>);
    let last = RwSignal::new(String::new());
    let count = RwSignal::new(0_u32);
    let offset = RwSignal::new(0_i64);
    let base = if leptos_daisyui_rs::test_mode::is_test_mode() {
        Signal::stored(1_000_000_i64)
    } else {
        use_sla_now(1_000)
    };
    let now = Signal::derive(move || base.get() + offset.get());
    let french = RwSignal::new(false);
    let texts = Signal::derive(move || {
        if french.get() {
            SoftphoneTexts {
                label: "Appel client".into(),
                phone_number: "Numéro de téléphone".into(),
                call: "Appeler".into(),
                end_call: "Terminer l’appel".into(),
                hold: "Mettre en attente".into(),
                record: "Enregistrer".into(),
                ready: "Prêt à appeler".into(),
                active: "En conversation".into(),
                duration: "Durée de l’appel".into(),
                ..Default::default()
            }
        } else {
            SoftphoneTexts::default()
        }
    });
    let on_command = Callback::new(move |command: SoftphoneCommand| {
        // The host checks context before applying a request or a later response.
        if command.context_id != state.get_untracked().context_id {
            return;
        }
        count.update(|n| *n += 1);
        last.set(format!("{:?}", command.action));
        match command.action {
            SoftphoneAction::SelectNumber(id) => state.update(|s| s.selected_phone_id = Some(id)),
            SoftphoneAction::SendDigit(_) => {}
            action => {
                state.update(|s| {
                    s.pending = Some(action.kind());
                    s.error = None;
                    if matches!(action, SoftphoneAction::Call { .. }) {
                        s.phase = SoftphonePhase::Dialing;
                    }
                });
                pending.set(Some(action));
            }
        }
    });
    let accept = move |_| {
        if let Some(action) = pending.get_untracked() {
            state.update(|s| {
                match action {
                    SoftphoneAction::Call { .. } => {
                        s.phase = SoftphonePhase::Active;
                        s.timer = SoftphoneTimer::Running {
                            connected_at_ms: now.get_untracked() - 65_000,
                        };
                    }
                    SoftphoneAction::EndCall | SoftphoneAction::RouteToVoicemail => {
                        s.phase = SoftphonePhase::Ended;
                        s.timer = SoftphoneTimer::Stopped {
                            seconds: s.timer.elapsed_at(now.get_untracked()).unwrap_or(0),
                        };
                        s.recording = false;
                        s.transcribing = false;
                    }
                    SoftphoneAction::SetMuted(value) => s.muted = value,
                    SoftphoneAction::SetHeld(value) => {
                        s.phase = if value {
                            SoftphonePhase::Held
                        } else {
                            SoftphonePhase::Active
                        }
                    }
                    SoftphoneAction::SetRecording(value) => s.recording = value,
                    SoftphoneAction::SetTranscribing(value) => s.transcribing = value,
                    _ => {}
                }
                s.pending = None;
                s.error = None;
            });
            pending.set(None);
        }
    };
    let reject = move |_| {
        let action = pending.get_untracked();
        state.update(|s| {
            if matches!(action, Some(SoftphoneAction::Call { .. })) {
                s.phase = SoftphonePhase::Ready;
            }
            s.pending = None;
            s.error = Some("Request declined. Your call state has not changed.".into());
        });
        pending.set(None);
    };
    view! {
        <ContentLayout title="Softphone" description="An opinionated client calling surface. Your application owns the call; the component makes its state and controls clear.">
            <Section title="Client call console">
                <div class="grid w-full gap-6 lg:grid-cols-2">
                    <div class="flex min-w-0 flex-col gap-4">
                        <p class="text-base">"Choose a number, then call. The console keeps identity, elapsed time and the next action together."</p>
                        <p class="text-sm text-base-content/70">"Interactive simulation. No calls are placed and no audio is captured. Accept or reject each request to see how confirmed state differs from a pending action."</p>
                        <div class="flex flex-wrap gap-2">
                            <Button attr:id="softphone-accept" color=ButtonColor::Primary disabled=Signal::derive(move || pending.get().is_none()) on_click=Callback::new(accept)>"Accept request"</Button>
                            <Button attr:id="softphone-reject" disabled=Signal::derive(move || pending.get().is_none()) on_click=Callback::new(reject)>"Reject request"</Button>
                            <Button attr:id="softphone-reset" on_click=Callback::new(move |_| {
                                state.set(initial_state()); pending.set(None); count.set(0); last.set(String::new()); offset.set(0);
                            })>"Reset"</Button>
                            <Button attr:id="softphone-advance" on_click=Callback::new(move |_| offset.update(|n| *n += 65_000))>"Advance 65 seconds"</Button>
                        </div>
                        <div class="flex flex-wrap gap-2">
                            <Button attr:id="softphone-single" on_click=Callback::new(move |_| state.update(|s| {
                                s.client.phones.truncate(1); s.selected_phone_id = None;
                            }))>"Single number"</Button>
                            <Button attr:id="softphone-empty" on_click=Callback::new(move |_| state.update(|s| {
                                s.client.phones.clear(); s.selected_phone_id = None;
                            }))>"No numbers"</Button>
                            <Button attr:id="softphone-capabilities" on_click=Callback::new(move |_| state.update(|s| s.capabilities = SoftphoneCapabilities {
                                mute: false, hold: false, voicemail: false, recording: false, transcription: false, keypad: false,
                            }))>"Hide optional actions"</Button>
                            <Button attr:id="softphone-french" on_click=Callback::new(move |_| french.update(|v| *v = !*v))>"Change labels"</Button>
                            <Button attr:id="softphone-long" on_click=Callback::new(move |_| state.update(|s| {
                                s.client.name = "AlexandriaCatherineMontgomeryWorthington".into();
                                s.client.subtitle = "ClientReferenceWithoutWhitespaceForWrappingVerification".into();
                            }))>"Long client name"</Button>
                        </div>
                        <label class="flex flex-col gap-2 text-sm">"Conversation notes (host-owned)"
                            <textarea id="softphone-notes" class="textarea min-h-24 w-full" placeholder="Capture a follow-up without sending keypad tones." />
                        </label>
                        <div data-testid="softphone-host" class="rounded-box border border-base-300 p-3 text-sm"
                            data-command-count=move || count.get().to_string()
                            data-phase=move || state.get().phase.as_str()
                            data-muted=move || state.get().muted.to_string()
                            data-recording=move || state.get().recording.to_string()
                            data-transcribing=move || state.get().transcribing.to_string()
                            data-selected=move || state.get().selected_number().map(|p| p.id.clone()).unwrap_or_default()
                            data-last-command=move || last.get()
                            data-pending=move || state.get().pending.map(|p| p.as_str()).unwrap_or("")>
                            <p class="font-medium">"Host command receipt"</p>
                            <output class="break-words">{move || if last.get().is_empty() { "No command yet".into() } else { last.get() }}</output>
                        </div>
                    </div>
                    <div class="flex min-w-0 justify-center lg:justify-end">
                        <Softphone id="softphone-demo" state=state on_command=on_command now_ms=now texts=texts />
                    </div>
                </div>
            </Section>
        </ContentLayout>
    }
}
