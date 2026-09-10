use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

fn initial_state(context: &str) -> ClientCallWorkspaceState {
    ClientCallWorkspaceState {
        call: SoftphoneState {
            context_id: context.into(),
            client: SoftphoneClient {
                name: "Elena Martinez".into(),
                subtitle: "Client · Case 10428 · Account review".into(),
                phones: vec![
                    SoftphoneNumber { id: "phone".into(), label: "Phone".into(), number: "+1 (415) 555-0142".into(), ..Default::default() },
                    SoftphoneNumber { id: "alternate".into(), label: "Alternate".into(), number: "+1 (415) 555-0186".into(), ..Default::default() },
                ],
            },
            selected_phone_id: Some("phone".into()),
            capabilities: SoftphoneCapabilities { mute: true, end_call: true, hold: true, voicemail: true, recording: true, transcription: true, keypad: true },
            ..Default::default()
        },
        destination: "+1 (415) 555-0142".into(),
        dial_blocked_reason: None,
        number_update: Some(ClientCallNumberUpdate { field_id: "contact-phone".into(), label: "Contact phone".into(), saved_number: "+1 (415) 555-0142".into(), ..Default::default() }),
        guidance: Some(ClientCallGuidance {
            title: "Before you call".into(),
            body: "Confirm Elena has a moment to talk. Ask whether she received the account summary, then agree on the next step together.".into(),
            source: "Account review playbook · Reviewed by the service team".into(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn office_state(context: &str) -> ClientCallWorkspaceState {
    let mut state = initial_state(context);
    state.call.client.phones.push(SoftphoneNumber {
        id: "mobile".into(),
        label: "Mobile".into(),
        number: "+1 (415) 555-0199".into(),
        blocked_reason: Some("Mobile calling is restricted for this contact.".into()),
    });
    state.number_update.as_mut().unwrap().targets = vec![
        ClientCallNumberTarget {
            field_id: "contact-phone".into(),
            label: "Contact phone".into(),
            saved_number: "+1 (415) 555-0142".into(),
            blocked_reason: None,
        },
        ClientCallNumberTarget {
            field_id: "contact-mobile".into(),
            label: "Contact mobile".into(),
            saved_number: "+1 (415) 555-0199".into(),
            blocked_reason: None,
        },
    ];
    let guidance = state.guidance.as_mut().unwrap();
    guidance.title = "Account review conversation".into();
    guidance.beats = vec![
        ClientCallScriptBeat {
            id: "opening".into(),
            title: "Confirm a good time".into(),
            say: "Hello Elena, this is your service team. Is now a good time to review your account summary together?".into(),
            no_file_detail: false,
        },
        ClientCallScriptBeat {
            id: "next-step".into(),
            title: "Agree on the next step".into(),
            say: "What would you like us to help with next? I can arrange a follow-up with your consultant.".into(),
            no_file_detail: true,
        },
    ];
    guidance.regeneration = Some(ClientCallRegeneration::default());
    state
}

#[component]
pub fn ClientCallWorkspaceDemo() -> impl IntoView {
    let state = RwSignal::new(initial_state("elena/attempt-1"));
    let request = RwSignal::new(None::<ClientCallCommand>);
    // Regeneration may overlap a call or write, and retains its own context
    // through acceptance until the separate host completion arrives.
    let regeneration_request = RwSignal::new(None::<ClientCallCommand>);
    let office = RwSignal::new(false);
    let last = RwSignal::new(String::new());
    let count = RwSignal::new(0_u32);
    let writes = RwSignal::new(0_u32);
    let closed = RwSignal::new(false);
    let reject_edits = RwSignal::new(false);
    let generation = RwSignal::new(1_u32);
    let pending_write = Signal::derive(move || {
        request.get().is_some_and(|command| {
            matches!(
                command.action,
                ClientCallAction::SaveNumber { .. } | ClientCallAction::SaveWrapUp(_)
            )
        })
    });
    let on_command = Callback::new(move |command: ClientCallCommand| {
        // Count every callback receipt, even rejected ones, so browser tests
        // cannot confuse this host guard with the component's own guard.
        count.update(|value| *value += 1);
        if !state.with_untracked(|s| s.accepts(&command)) {
            return;
        }
        last.set(format!("{:?}", command.action));
        if reject_edits.get_untracked()
            && matches!(
                command.action,
                ClientCallAction::EditDestination(_)
                    | ClientCallAction::ChooseSavedNumber(_)
                    | ClientCallAction::ChooseNumberTarget(_)
                    | ClientCallAction::SetOutcome(_)
                    | ClientCallAction::SetNotes(_)
                    | ClientCallAction::SetDurationMinutes(_)
                    | ClientCallAction::SetFollowUp(_)
            )
        {
            return;
        }
        match command.action.clone() {
            ClientCallAction::EditDestination(value) => state.update(|s| {
                s.destination = value;
                s.number_error = None;
            }),
            ClientCallAction::ChooseSavedNumber(id) => state.update(|s| {
                if let Some(number) = s.call.client.phones.iter().find(|p| p.id == id) {
                    s.destination = number.number.clone();
                    s.number_error = None;
                }
            }),
            ClientCallAction::ChooseNumberTarget(id) => state.update(|s| {
                let update = s.number_update.as_mut().unwrap();
                if let Some(target) = update.targets.iter().find(|target| target.field_id == id) {
                    update.field_id = target.field_id.clone();
                    update.label = target.label.clone();
                    update.saved_number = target.saved_number.clone();
                    update.error = None;
                }
            }),
            ClientCallAction::RegenerateGuidance => {
                state.update(|s| {
                    let regeneration = s.guidance.as_mut().unwrap().regeneration.as_mut().unwrap();
                    regeneration.state = ClientCallRegenerationState::Busy;
                    regeneration.error = None;
                });
                regeneration_request.set(Some(command));
            }
            ClientCallAction::SetOutcome(value) => {
                state.update(|s| s.wrap_up.as_mut().unwrap().outcome = value)
            }
            ClientCallAction::SetNotes(value) => {
                state.update(|s| s.wrap_up.as_mut().unwrap().notes = value)
            }
            ClientCallAction::SetDurationMinutes(value) => {
                state.update(|s| s.wrap_up.as_mut().unwrap().duration_minutes = value)
            }
            ClientCallAction::SetFollowUp(value) => {
                state.update(|s| s.wrap_up.as_mut().unwrap().follow_up = value)
            }
            ClientCallAction::Dismiss => closed.set(true),
            action @ (ClientCallAction::Dial { .. }
            | ClientCallAction::SaveNumber { .. }
            | ClientCallAction::SaveWrapUp(_)
            | ClientCallAction::Session(
                SoftphoneAction::EndCall
                | SoftphoneAction::SetMuted(_)
                | SoftphoneAction::SetHeld(_)
                | SoftphoneAction::RouteToVoicemail
                | SoftphoneAction::SetRecording(_)
                | SoftphoneAction::SetTranscribing(_)
                | SoftphoneAction::SendDigit(_),
            )) => {
                state.update(|s| match action {
                    ClientCallAction::Dial { .. } => {
                        s.attempt = ClientCallAttempt::Submitting;
                        s.status_detail.clear();
                    }
                    ClientCallAction::SaveNumber { .. } => {
                        let update = s.number_update.as_mut().unwrap();
                        update.pending = true;
                        update.error = None;
                    }
                    ClientCallAction::SaveWrapUp(_) => {
                        let wrap = s.wrap_up.as_mut().unwrap();
                        wrap.pending = true;
                        wrap.error = None;
                    }
                    ClientCallAction::Session(ref action) => s.call.pending = Some(action.kind()),
                    _ => {}
                });
                request.set(Some(command));
            }
            _ => {}
        }
    });
    let accept = Callback::new(move |_| {
        let Some(command) = request.get_untracked() else {
            let Some(command) = regeneration_request.get_untracked() else {
                return;
            };
            if command.context_id != state.get_untracked().call.context_id {
                regeneration_request.set(None);
                return;
            }
            state.update(|s| {
                let guidance = s.guidance.as_mut().unwrap();
                guidance.regeneration.as_mut().unwrap().state =
                    ClientCallRegenerationState::Accepted;
                guidance.state = ClientCallGuidanceState::Preparing {
                    elapsed_secs: 12,
                    typical_secs: 75,
                };
            });
            return;
        };
        // This fixture has one outstanding host operation. Production adapters
        // additionally correlate a unique operation token before applying results.
        if command.context_id != state.get_untracked().call.context_id {
            request.set(None);
            return;
        }
        state.update(|s| match command.action {
            ClientCallAction::Dial { .. } => s.attempt = ClientCallAttempt::AgentRinging,
            ClientCallAction::SaveNumber { field_id, number } => {
                let update = s.number_update.as_mut().unwrap();
                update.saved_number = number.clone();
                update.pending = false;
                if let Some(target) = update
                    .targets
                    .iter_mut()
                    .find(|target| target.field_id == field_id)
                {
                    target.saved_number = number.clone();
                }
                let phone_id = if field_id == "contact-mobile" {
                    "mobile"
                } else {
                    "phone"
                };
                if let Some(phone) = s
                    .call
                    .client
                    .phones
                    .iter_mut()
                    .find(|phone| phone.id == phone_id)
                {
                    phone.number = number;
                }
                writes.update(|n| *n += 1);
            }
            ClientCallAction::SaveWrapUp(_) => {
                let wrap = s.wrap_up.as_mut().unwrap();
                wrap.pending = false;
                wrap.saved = true;
                writes.update(|n| *n += 1);
            }
            ClientCallAction::Session(action) => {
                match action {
                    SoftphoneAction::EndCall | SoftphoneAction::RouteToVoicemail => {
                        s.call.phase = SoftphonePhase::Ended;
                        s.call.timer = SoftphoneTimer::Stopped { seconds: 125 };
                    }
                    SoftphoneAction::SetMuted(value) => s.call.muted = value,
                    SoftphoneAction::SetHeld(value) => {
                        s.call.phase = if value {
                            SoftphonePhase::Held
                        } else {
                            SoftphonePhase::Active
                        }
                    }
                    SoftphoneAction::SetRecording(value) => s.call.recording = value,
                    SoftphoneAction::SetTranscribing(value) => s.call.transcribing = value,
                    _ => {}
                }
                s.call.pending = None;
            }
            _ => {}
        });
        request.set(None);
    });
    let reject = Callback::new(move |_| {
        let Some(command) = request.get_untracked() else {
            let Some(command) = regeneration_request.get_untracked() else {
                return;
            };
            if command.context_id == state.get_untracked().call.context_id {
                state.update(|s| {
                    let guidance = s.guidance.as_mut().unwrap();
                    guidance.state = ClientCallGuidanceState::Ready;
                    let regeneration = guidance.regeneration.as_mut().unwrap();
                    regeneration.state = ClientCallRegenerationState::Failed;
                    regeneration.error =
                        Some("Script refresh failed. Your previous guidance is retained.".into());
                });
            }
            regeneration_request.set(None);
            return;
        };
        if command.context_id != state.get_untracked().call.context_id {
            request.set(None);
            return;
        }
        state.update(|s| match command.action {
            ClientCallAction::Dial { .. } => {
                s.attempt = ClientCallAttempt::Refused;
                s.status_detail = "Provider confirmed that no call was submitted.".into();
            }
            ClientCallAction::SaveNumber { .. } => {
                let u = s.number_update.as_mut().unwrap();
                u.pending = false;
                u.error = Some("Update declined. The saved contact number is unchanged.".into());
            }
            ClientCallAction::SaveWrapUp(_) => {
                let w = s.wrap_up.as_mut().unwrap();
                w.pending = false;
                w.error = Some("Save declined. Your draft is retained.".into());
            }
            ClientCallAction::Session(_) => {
                s.call.pending = None;
                s.call.error = Some("Control request declined.".into());
            }
            _ => {}
        });
        request.set(None);
    });
    view! {
        <ContentLayout title="Client Call Workspace" description="One operational surface for client context, calling and a deliberate work record.">
            <Section title="Calling workspace">
                <div class="flex flex-col gap-6">
                    <p class="text-sm text-base-content/70">"Interactive simulation. No calls, contact writes or recordings leave this page. Host controls below let you confirm or decline each request."</p>
                    <div class="flex flex-wrap gap-2">
                        <Button attr:id="call-workspace-accept" disabled=Signal::derive(move || request.get().is_none() && regeneration_request.get().is_none()) on_click=accept>"Accept request"</Button>
                        <Button attr:id="call-workspace-reject" disabled=Signal::derive(move || request.get().is_none() && regeneration_request.get().is_none()) on_click=reject>"Reject request"</Button>
                        <Button attr:id="call-workspace-uncertain" disabled=pending_write on_click=Callback::new(move |_| { if pending_write.get_untracked() { return; } state.update(|s| { s.attempt = ClientCallAttempt::Uncertain; s.status_detail = "The provider response was lost.".into(); }); request.set(None); })>"Unknown dispatch"</Button>
                        <Button attr:id="call-workspace-connected" disabled=pending_write on_click=Callback::new(move |_| {
                            if pending_write.get_untracked() { return; }
                            state.update(|s| {
                                s.attempt = ClientCallAttempt::Managed; s.call.phase = SoftphonePhase::Active;
                                s.call.pending = None; s.call.timer = SoftphoneTimer::Running { connected_at_ms: 875_000 };
                                s.call.client.phones = vec![SoftphoneNumber { id: "dialed".into(), label: "Dialed".into(), number: s.destination.clone(), ..Default::default() }];
                                s.call.selected_phone_id = Some("dialed".into());
                            }); request.set(None);
                        })>"Confirm connection"</Button>
                        <Button attr:id="call-workspace-finished" disabled=pending_write on_click=Callback::new(move |_| { if pending_write.get_untracked() { return; } state.update(|s| { s.attempt = ClientCallAttempt::Finished; s.call.phase = SoftphonePhase::Ended; s.call.pending = None; }); request.set(None); })>"Confirm finished"</Button>
                        <Button attr:id="call-workspace-reset" on_click=Callback::new(move |_| {
                            generation.update(|n| *n += 1); state.set(initial_state(&format!("elena/attempt-{}", generation.get_untracked())));
                            request.set(None); count.set(0); writes.set(0); last.set(String::new()); closed.set(false); reject_edits.set(false);
                            regeneration_request.set(None); office.set(false);
                        })>"New attempt"</Button>
                        <Button attr:id="call-workspace-switch" on_click=Callback::new(move |_| {
                            generation.update(|n| *n += 1); let mut next = initial_state(&format!("alex/attempt-{}", generation.get_untracked()));
                            next.call.client.name = "AlexandriaCatherineMontgomeryWorthington".into(); next.call.client.subtitle = "ClientReferenceWithoutWhitespaceForWrappingVerification".into();
                            state.set(next); closed.set(false);
                        })>"Switch client"</Button>
                        <Button attr:id="call-workspace-reject-edits" on_click=Callback::new(move |_| reject_edits.update(|v| *v = !*v))>"Toggle draft rejection"</Button>
                        <Button attr:id="call-workspace-reopen" on_click=Callback::new(move |_| closed.set(false))>"Reopen"</Button>
                        <Button attr:id="call-workspace-block" on_click=Callback::new(move |_| state.update(|s| s.dial_blocked_reason = Some("Calling permission was revoked.".into())))>"Revoke calling"</Button>
                        <Button attr:id="call-workspace-office" on_click=Callback::new(move |_| {
                            generation.update(|n| *n += 1);
                            state.set(office_state(&format!("elena/office-{}", generation.get_untracked())));
                            request.set(None); regeneration_request.set(None); count.set(0); writes.set(0);
                            last.set(String::new()); closed.set(false); reject_edits.set(false); office.set(true);
                        })>"Office adoption scenario"</Button>
                    </div>
                    <Show when=move || office.get()>
                        <div class="flex flex-wrap gap-2">
                            <Button attr:id="call-workspace-preparing" on_click=Callback::new(move |_| state.update(|s| {
                                s.guidance.as_mut().unwrap().state = ClientCallGuidanceState::Preparing { elapsed_secs: 12, typical_secs: 75 };
                            }))>"Preparing script"</Button>
                            <Button attr:id="call-workspace-script-failed" on_click=Callback::new(move |_| state.update(|s| {
                                s.guidance.as_mut().unwrap().state = ClientCallGuidanceState::Failed { reason: "The script service could not prepare guidance.".into() };
                            }))>"Script unavailable"</Button>
                            <Button attr:id="call-workspace-regeneration-conflict" on_click=Callback::new(move |_| state.update(|s| {
                                if let Some(regeneration) = s.guidance.as_mut().and_then(|guidance| guidance.regeneration.as_mut()) {
                                    regeneration.blocked_reason = Some("A colleague is already refreshing this script.".into());
                                }
                            }))>"Refresh conflict"</Button>
                            <Button attr:id="call-workspace-regeneration-retry" on_click=Callback::new(move |_| state.update(|s| {
                                if let Some(regeneration) = s.guidance.as_mut().and_then(|guidance| guidance.regeneration.as_mut()) {
                                    regeneration.blocked_reason = None;
                                }
                            }))>"Clear refresh conflict"</Button>
                            <Button attr:id="call-workspace-script-complete" on_click=Callback::new(move |_| {
                                let Some(command) = regeneration_request.get_untracked() else { return; };
                                if command.context_id != state.get_untracked().call.context_id {
                                    regeneration_request.set(None);
                                    return;
                                }
                                state.update(|s| {
                                    let guidance = s.guidance.as_mut().unwrap();
                                    let regeneration = guidance.regeneration.as_mut().unwrap();
                                    if regeneration.state != ClientCallRegenerationState::Accepted { return; }
                                    regeneration.state = ClientCallRegenerationState::Succeeded;
                                    regeneration.error = None;
                                    guidance.state = ClientCallGuidanceState::Ready;
                                    guidance.beats[0].say = "Hello Elena, your updated account summary is ready. Shall we review the next step together?".into();
                                });
                                if state.with_untracked(|s| s.guidance.as_ref().unwrap().regeneration.as_ref().unwrap().state == ClientCallRegenerationState::Succeeded) {
                                    regeneration_request.set(None);
                                }
                            })>"Complete script refresh"</Button>
                            <Button attr:id="call-workspace-agent-not-ready" disabled=pending_write on_click=Callback::new(move |_| {
                                if pending_write.get_untracked() { return; }
                                state.update(|s| { s.attempt = ClientCallAttempt::RefusedWith(ClientCallRefusal::AgentNotReady); s.status_detail.clear(); }); request.set(None);
                            })>"Agent not ready"</Button>
                            <Button attr:id="call-workspace-agent-not-available" disabled=pending_write on_click=Callback::new(move |_| {
                                if pending_write.get_untracked() { return; }
                                state.update(|s| { s.attempt = ClientCallAttempt::RefusedWith(ClientCallRefusal::AgentNotAvailable); s.status_detail.clear(); }); request.set(None);
                            })>"Agent not available"</Button>
                            <Button attr:id="call-workspace-not-submitted" disabled=pending_write on_click=Callback::new(move |_| {
                                if pending_write.get_untracked() { return; }
                                state.update(|s| { s.attempt = ClientCallAttempt::RefusedWith(ClientCallRefusal::NotSubmitted); s.status_detail.clear(); }); request.set(None);
                            })>"Confirmed not submitted"</Button>
                            <Button attr:id="call-workspace-talk-zero" on_click=Callback::new(move |_| state.update(|s| s.provider_talk_seconds = Some(0)))>"Provider talk: zero"</Button>
                            <Button attr:id="call-workspace-talk-125" on_click=Callback::new(move |_| state.update(|s| s.provider_talk_seconds = Some(125)))>"Provider talk: 125 seconds"</Button>
                        </div>
                    </Show>
                    <div data-testid="call-workspace-host" class="rounded-box bg-base-200 p-4 text-sm"
                        data-context=move || state.get().call.context_id data-count=move || count.get().to_string()
                        data-attempt=move || state.get().attempt.as_str() data-destination=move || state.get().destination
                        data-outcome=move || state.get().wrap_up.and_then(|w| w.outcome).map(|o| o.as_str())
                        data-notes=move || state.get().wrap_up.map(|w| w.notes).unwrap_or_default()
                        data-saved=move || state.get().wrap_up.is_some_and(|w| w.saved).to_string()
                        data-writes=move || writes.get().to_string() data-last=move || last.get()
                        data-number-target=move || state.get().number_update.map(|u| u.field_id).unwrap_or_default()
                        data-guidance-state=move || state.get().guidance.map(|g| format!("{:?}", g.state)).unwrap_or_default()
                        data-regeneration-state=move || state.get().guidance.and_then(|g| g.regeneration).map(|r| r.state.as_str()).unwrap_or("")
                        data-beats=move || state.get().guidance.map(|g| g.beats.iter().map(|b| b.say.as_str()).collect::<Vec<_>>().join(" | ")).unwrap_or_default()
                        data-talk-seconds=move || state.get().provider_talk_seconds.map(|seconds| seconds.to_string()).unwrap_or_default()
                        data-duration-minutes=move || state.get().wrap_up.map(|w| w.duration_minutes).unwrap_or_default()
                        data-regeneration-pending=move || regeneration_request.get().is_some().to_string()
                        data-pending=move || request.get().is_some().to_string() data-closed=move || closed.get().to_string()>
                        <p>"Host receipts: "{move || count.get()}" · Confirmed writes: "{move || writes.get()}</p>
                        <output class="whitespace-pre-wrap [overflow-wrap:anywhere]">{move || if last.get().is_empty() { "No command yet".into() } else { last.get() }}</output>
                    </div>
                    <Show when=move || !closed.get() fallback=move || view! { <p>"Workspace dismissed. The host retains the attempt and draft. Use Reopen to return."</p> }>
                        <ClientCallWorkspace id="client-call-workspace-demo" state=state on_command=on_command now_ms=Signal::stored(1_000_000) />
                    </Show>
                </div>
            </Section>
        </ContentLayout>
    }
}
