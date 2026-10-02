//! The compact layout's view (ldui-eq1e, Office op-flpq1). Every line it shows
//! comes from the `compact_*` functions in [`super::compact`]; this file only
//! places them.

use super::component::{
    Destination, Guidance, WrapUp, context_reason_id, control_disabled, control_name, emit,
    reason_id,
};
use super::*;
use crate::components::{
    Button, ButtonColor, ButtonShape, ButtonSize, ButtonStyle, IconSize, Softphone,
    SoftphoneCommand,
};
use leptos::prelude::*;

#[component]
pub(super) fn CompactWorkspace(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
    now: Signal<i64>,
    badges: Signal<Vec<String>>,
    base_id: String,
    keypad_id: String,
    console_id: String,
    session_command: Callback<SoftphoneCommand>,
    header_actions: Option<ViewFn>,
) -> impl IntoView {
    let context_reason = context_reason_id(&base_id);
    let destination_base = base_id.clone();
    let history_base = base_id.clone();
    let guidance_base = base_id.clone();
    let wrap_base = base_id;
    let name = move || state.with(|s| s.call.client.name.clone());
    let subtitle = move || state.with(|s| s.call.client.subtitle.clone());
    // Dismiss is disabled only while no context exists; its reason then says
    // what the shared context line says.
    let dismiss_reason = Signal::derive(move || {
        state
            .with(|s| texts.with(|t| t.disabled_reason(&ClientCallControl::Dismiss, s)))
            .unwrap_or_default()
    });
    let status = Signal::derive(move || state.with(|s| texts.with(|t| t.compact_status(s))));
    let uncertain =
        Signal::derive(move || state.with(|s| texts.with(|t| t.compact_uncertain_hint(s))));
    view! {
        <header class="flex min-w-0 flex-col gap-2 border-b border-base-300 px-4 py-3" data-call-header="compact">
            <div class="flex min-w-0 items-center gap-2">
                <div class="flex min-w-0 flex-1 items-baseline gap-2">
                    <h2 class="min-w-0 truncate text-base font-semibold" title=name>{name}</h2>
                    <Show when=move || !subtitle().trim().is_empty()>
                        <span aria-hidden="true" class="shrink-0 text-sm text-base-content/75">"·"</span>
                        <p class="min-w-0 flex-1 truncate text-sm text-base-content/75" title=subtitle data-call-subtitle="true">{subtitle}</p>
                    </Show>
                </div>
                // The host's actions never shrink: the name block (flex-1,
                // basis 0) truncates first to make room on the one row.
                {header_actions.map(|actions| view! {
                    <div class="flex shrink-0 items-center gap-2 whitespace-nowrap text-sm" data-call-header-actions="true">{actions.run()}</div>
                })}
                <Button style=ButtonStyle::Ghost shape=ButtonShape::Square size=ButtonSize::Sm class="shrink-0"
                    attr:data-call-action="dismiss"
                    attr:aria-label=move || texts.get().close
                    disabled_reason=dismiss_reason
                    on_click=Callback::new(move |_| emit(state, on_command, ClientCallAction::Dismiss))>
                    <svg class=IconSize::XSmall.as_str() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                        stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
                        <path d="M18 6 6 18" />
                        <path d="m6 6 12 12" />
                    </svg>
                </Button>
            </div>
            <Show when=move || badges.with(|badges| !badges.is_empty())>
                <div class="flex flex-wrap gap-2" data-call-client-badges="true">
                    {move || badges.get().into_iter().map(|badge| view! {
                        <span class="badge badge-outline badge-sm">{badge}</span>
                    }).collect_view()}
                </div>
            </Show>
        </header>
        // The shared line for the state that disables every control at once.
        <p id=context_reason role="note" class="border-b border-base-300 px-4 py-2 text-sm text-base-content/75 [overflow-wrap:anywhere]"
            data-call-disabled-reason="context"
            class:hidden=move || !state.with(ClientCallWorkspaceState::context_missing)>
            {move || state.with(ClientCallWorkspaceState::context_missing).then(|| texts.get().not_ready)}
        </p>
        <div class="flex min-w-0 flex-col gap-4 p-4 [overflow-wrap:anywhere]" data-call-body="compact">
            // One line, present from mount so the live region announces changes;
            // hidden before any attempt with nothing to explain.
            <div role="status" class="flex min-w-0 flex-col gap-1" data-call-status="true"
                class:hidden=move || status.with(Option::is_none)>
                <p class="flex min-w-0 items-center gap-2 text-sm font-medium" data-call-status-line="true">
                    // Decorative. daisyUI's `.status` mixes its tone into a drop
                    // shadow, a per-tone depth value off the elevation ramp; an
                    // 8 px dot needs no depth (doc/visual-quality/ad-hoc-shadow.md).
                    <span aria-hidden="true" class=move || format!("{} shadow-none", state.with(ClientCallWorkspaceState::compact_status_tone))></span>
                    <span class="min-w-0">{move || status.get()}</span>
                </p>
                <p class="text-sm" data-call-uncertain-hint="true" class:hidden=move || uncertain.with(Option::is_none)>{move || uncertain.get()}</p>
            </div>
            <Show when=move || state.with(|s| s.attempt == ClientCallAttempt::Managed && s.call.phase.is_live())
                fallback=move || view! { <CompactDestination state=state texts=texts on_command=on_command keypad_id=keypad_id.clone() base_id=destination_base.clone() /> }>
                <Softphone id=console_id.clone() state=Signal::derive(move || state.get().call)
                    texts=Signal::derive(move || texts.get().softphone) on_command=session_command
                    now_ms=now class="max-w-none" />
            </Show>
            <Show when=move || state.with(|s| s.history.is_some())>
                <CompactHistory state=state texts=texts base_id=history_base.clone() />
            </Show>
            <Show when=move || state.with(|s| s.guidance.is_some())>
                <CompactGuidance state=state texts=texts on_command=on_command base_id=guidance_base.clone() />
            </Show>
            <Show when=move || state.with(|s| s.wrap_up.is_some())>
                <WrapUp state=state texts=texts on_command=on_command base_id=wrap_base.clone() />
            </Show>
        </div>
    }
}

/// The launcher's numbers and call button; a typed panel keeps its full
/// destination block.
#[component]
fn CompactDestination(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
    keypad_id: String,
    base_id: String,
) -> impl IntoView {
    let numbers_base = base_id.clone();
    view! {
        <Show when=move || state.with(ClientCallWorkspaceState::saved_only)
            fallback=move || view! { <Destination state=state texts=texts on_command=on_command keypad_id=keypad_id.clone() base_id=base_id.clone() /> }>
            <CompactNumbers state=state texts=texts on_command=on_command base_id=numbers_base.clone() />
        </Show>
    }
}

#[component]
fn CompactNumbers(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
    base_id: String,
) -> impl IntoView {
    let phones = Memo::new(move |_| state.get().call.client.phones);
    let heading_id = format!("{base_id}-numbers-heading");
    // Bound before the view: the macro moves the heading's id first.
    let labelled_by = heading_id.clone();
    let group = format!("{base_id}-saved-number");
    let context_id = context_reason_id(&base_id);
    let lock_id = reason_id(&base_id, "numbers-locked");
    let lock_ref = lock_id.clone();
    let call_line_id = reason_id(&base_id, "dial");
    let lock_line = Signal::derive(move || state.with(|s| texts.with(|t| t.compact_lock_line(s))));
    let call_line = Signal::derive(move || state.with(|s| texts.with(|t| t.compact_call_line(s))));
    let call_reason = Signal::derive(move || {
        state
            .with(|s| texts.with(|t| t.compact_call_reason(s)))
            .unwrap_or_default()
    });
    let call_label = move || state.with(|s| texts.with(|t| t.compact_call_label(s)));
    let starts_another = move || {
        state.with(|s| ClientCallControl::compact_call(s) == ClientCallControl::StartAnotherCall)
    };
    let launch_hint =
        Signal::derive(move || texts.with(ClientCallWorkspaceTexts::compact_launch_hint));
    // Bumped after every choice so each radio re-reads the host's answer: a
    // choice the host does not adopt snaps back to the confirmed number.
    let revision = RwSignal::new(0_u32);
    view! {
        <div class="flex min-w-0 flex-col gap-2" data-call-destination="true" data-call-destination-entry="saved-only">
            <p id=heading_id class="text-sm font-medium">{move || texts.get().destination}</p>
            <Show when=move || phones.with(Vec::is_empty)>
                <p class="text-sm text-base-content/75">{move || texts.get().no_saved_numbers}</p>
            </Show>
            <div role="radiogroup" aria-labelledby=labelled_by class="flex min-w-0 flex-col gap-1" data-call-numbers="true"
                class:hidden=move || phones.with(Vec::is_empty)>
                {move || phones.get().into_iter().enumerate().map(|(index, phone)| {
                    let control = ClientCallControl::SavedNumber(phone.id.clone());
                    let action = ClientCallAction::ChooseSavedNumber(phone.id.clone());
                    let name = control_name(state, texts, control.clone());
                    let disabled = control_disabled(state, control);
                    // Index-keyed: a phone id is host text and may not be a valid id token.
                    let own_id = reason_id(&base_id, &format!("saved-number-{index}"));
                    let described = format!("{context_id} {lock_ref} {own_id}");
                    let line_phone = phone.id.clone();
                    let own = Signal::derive(move || state.with(|s| texts.with(|t| t.compact_number_line(s, &line_phone))));
                    let selected_phone = phone.id.clone();
                    let selected = Signal::derive(move || state.with(|s| s.is_selected_number(&selected_phone)));
                    view! {
                        <label class="grid min-w-0 cursor-pointer grid-cols-[auto_minmax(0,1fr)] items-center gap-x-3 py-1"
                            class:cursor-not-allowed=move || disabled.get()>
                            <input type="radio" class="radio radio-sm" name=group.clone() value=phone.id.clone()
                                data-call-saved-number=phone.id.clone()
                                data-call-selected=move || selected.get().then_some("true")
                                aria-label=name aria-describedby=described
                                disabled=move || disabled.get()
                                prop:checked=move || { revision.track(); selected.get() }
                                on:change=move |_| {
                                    emit(state, on_command, action.clone());
                                    revision.update(|n| *n = n.wrapping_add(1));
                                } />
                            <span class="flex min-w-0 flex-wrap items-baseline gap-x-3 text-sm">
                                <span class="font-medium">{phone.label}</span>
                                <span class="tabular-nums">{phone.number}</span>
                            </span>
                            <span id=own_id class="col-start-2 text-sm text-base-content/75" data-call-number-blocked="true"
                                class:hidden=move || own.with(Option::is_none)>{move || own.get()}</span>
                        </label>
                    }
                }).collect_view()}
            </div>
            // The ONE lock line; each radio's aria-describedby names it.
            <p id=lock_id class="text-sm text-base-content/75" data-call-numbers-locked="true"
                class:hidden=move || lock_line.with(Option::is_none)>{move || lock_line.get()}</p>
            <Button color=ButtonColor::Primary class="w-full"
                attr:data-call-action=move || if starts_another() { "start-another-call" } else { "dial" }
                attr:aria-label=call_label
                disabled_reason=call_reason
                on_click=Callback::new(move |_| {
                    let current = state.get_untracked();
                    if ClientCallControl::compact_call(&current) == ClientCallControl::StartAnotherCall {
                        emit(state, on_command, ClientCallAction::StartAnotherCall);
                    } else {
                        emit(state, on_command, ClientCallAction::Dial { number: current.destination });
                    }
                })>
                {call_label}
            </Button>
            // Any reason other than the context or the lock, once.
            <p id=call_line_id class="text-sm text-base-content/75" data-call-disabled-reason="dial"
                class:hidden=move || call_line.with(Option::is_none)>{move || call_line.get()}</p>
            <Show when=move || launch_hint.with(Option::is_some)>
                <p class="text-sm text-base-content/75" data-call-launch-hint="true">{move || launch_hint.get()}</p>
            </Show>
        </div>
    }
}

/// Recent calls as one tabular row each, the first three visible.
#[component]
fn CompactHistory(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    base_id: String,
) -> impl IntoView {
    let history = Signal::derive(move || state.get().history.unwrap_or_default());
    let entries = Memo::new(move |_| history.get().entries);
    let expanded = RwSignal::new(false);
    // A different client starts collapsed again.
    Effect::new(move |previous: Option<String>| {
        let context = state.with(|s| s.call.context_id.clone());
        if previous.is_some_and(|previous| previous != context) {
            expanded.set(false);
        }
        context
    });
    let heading_id = format!("{base_id}-history-heading");
    let labelled_by = heading_id.clone();
    let list_id = format!("{base_id}-history-list");
    let controls = list_id.clone();
    view! {
        <section class="flex min-w-0 flex-col gap-2" aria-labelledby=labelled_by data-call-history="true">
            <h3 id=heading_id class="text-sm font-semibold">{move || texts.get().history}</h3>
            <Show when=move || history.get().loading>
                <p role="status" class="text-sm text-base-content/75" data-call-history-loading="true">{move || texts.get().history_loading}</p>
            </Show>
            {move || history.get().note.map(|note| view! { <p class="text-sm text-base-content/75" data-call-history-note="true">{note}</p> })}
            <Show when=move || history.with(|h| !h.loading && h.entries.is_empty())>
                <p class="text-sm text-base-content/75" data-call-history-empty="true">{move || texts.get().history_empty}</p>
            </Show>
            <Show when=move || !entries.with(Vec::is_empty)>
                <ol id=list_id.clone() class="grid min-w-0 grid-cols-[auto_auto_minmax(0,1fr)_minmax(0,1fr)_auto] items-baseline gap-x-2 gap-y-1 text-sm"
                    data-call-history-list="true">
                    {move || {
                        let shown = if expanded.get() { usize::MAX } else { COMPACT_HISTORY_VISIBLE };
                        let copy = texts.get();
                        entries.get().into_iter().take(shown).map(|entry| {
                            // Absent talk time is unknown, never zero.
                            let talk = entry
                                .talk_seconds
                                .map(crate::components::format_softphone_duration)
                                .unwrap_or_else(|| copy.provider_talk_unknown.clone());
                            let inbound = entry.direction == ClientCallDirection::Inbound;
                            view! {
                                <li class="col-span-full grid grid-cols-subgrid items-baseline" data-call-history-entry=entry.id
                                    data-call-direction=entry.direction.as_str()>
                                    <span class="self-center text-base-content/75">
                                        <svg class=IconSize::XSmall.as_str() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                                            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
                                            {if inbound {
                                                view! { <path d="M17 7 7 17" /><path d="M17 17H7V7" /> }.into_any()
                                            } else {
                                                view! { <path d="M7 7h10v10" /><path d="M7 17 17 7" /> }.into_any()
                                            }}
                                        </svg>
                                        <span class="sr-only">{copy.direction(entry.direction)}</span>
                                    </span>
                                    <span class="whitespace-nowrap">{entry.when}</span>
                                    <span class="min-w-0">{entry.who}</span>
                                    <span class="min-w-0">{entry.outcome}</span>
                                    <span class="text-right tabular-nums">{talk}</span>
                                </li>
                            }
                        }).collect_view()
                    }}
                </ol>
            </Show>
            <Show when=move || entries.with(|entries| entries.len() > COMPACT_HISTORY_VISIBLE)>
                <Button style=ButtonStyle::Ghost size=ButtonSize::Sm class="self-end" attr:data-call-history-toggle="true"
                    attr:aria-expanded=move || expanded.get().to_string() attr:aria-controls=controls.clone()
                    on_click=Callback::new(move |_| expanded.update(|value| *value = !*value))>
                    {move || texts.with(|t| t.history_toggle(expanded.get(), entries.with(Vec::len)))}
                </Button>
            </Show>
        </section>
    }
}

/// The script as a disclosure, collapsed until the operator opens it.
#[component]
fn CompactGuidance(
    state: Signal<ClientCallWorkspaceState>,
    texts: Signal<ClientCallWorkspaceTexts>,
    on_command: Callback<ClientCallCommand>,
    base_id: String,
) -> impl IntoView {
    let summary = move || {
        state.with(|s| {
            texts.with(|t| {
                s.guidance
                    .as_ref()
                    .map(|guidance| t.compact_script_summary(guidance))
                    .unwrap_or_default()
            })
        })
    };
    view! {
        <details class="min-w-0 rounded-box border border-base-300" data-call-script-disclosure="true">
            <summary class="ld-focus-ring cursor-pointer rounded-box px-3 py-2 text-sm font-semibold [overflow-wrap:anywhere]">{summary}</summary>
            <div class="border-t border-base-300 p-3">
                <Guidance state=state texts=texts on_command=on_command base_id=base_id embedded=true />
            </div>
        </details>
    }
}
