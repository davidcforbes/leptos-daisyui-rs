use super::style::{
    ComposerAction, annotation_bubble_class, chat_state_attr, clamp_composer_height,
    composer_hint_for, composer_key_action, composer_placeholder_for, effective_assistant_label,
    is_markdown, is_thinking, role_avatar_bg, role_avatar_initial_with, role_classes,
    role_data_attr, role_label_for, should_stick_to_bottom, show_welcome_chips,
    tool_phase_data_attr,
};
use super::texts::AiChatTexts;
use super::types::{
    AnnotationBody, ModelFieldMode, RetryOutcome, TranscriptAnnotation, annotation_slots,
    format_allowed_tools, format_usage, model_field_mode, permission_selection,
    reconcile_model_for, retry_outcome, settings_from_form_fields, settings_rows_for,
};
use crate::components::{Dropdown, DropdownAlignment, DropdownContent, Input, Textarea, Toggle};
use crate::markdown::MarkdownView;
use crate::merge_classes;
use ai_chat_core::{Capabilities, ChatMessage, ChatRequest, ChatSession, ChatSettings};
use leptos::html::{Div, Textarea as HtmlTextarea};
use leptos::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Per-instance sequence so each mounted panel mints unique settings-control
/// DOM ids — every control carries a real `label for=` association, which
/// requires the id to be unique across the whole document.
static AI_CHAT_SEQ: AtomicU64 = AtomicU64::new(0);

/// Distance (px) from the bottom within which the transcript keeps auto-scrolling.
const STICK_THRESHOLD_PX: f64 = 40.0;
/// Composer's collapsed height (px) — matches the old `rows="2"` sizing.
const COMPOSER_BASE_HEIGHT_PX: f64 = 48.0;
/// Composer's max auto-grow height (px) before it scrolls internally.
const COMPOSER_MAX_HEIGHT_PX: f64 = 320.0;

/// One option in the chat scope selector (e.g. "This file" / "This folder" /
/// "Whole workspace"). `id` is opaque to the component — the host interprets it
/// in `on_scope_change` (e.g. maps it to a grounding / `DocMode`). `label` is
/// what the user sees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatScopeOption {
    /// Opaque scope id passed back to the host via `on_scope_change`.
    pub id: String,
    /// User-visible label shown in the selector.
    pub label: String,
}

impl ChatScopeOption {
    /// Construct a scope option from an `id` and a display `label`.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// # AiChat
///
/// A reusable Leptos chat panel rendering the shared `ai_chat_core` presentation
/// model. The component **owns the drive loop**: it drains the session's
/// transport on a timer (`poll_ms`) and re-renders when the transcript changes —
/// the web analogue of editmark's desktop `WM_TIMER` poll.
///
/// The session is supplied by the parent as a `StoredValue` handle so the caller
/// owns transport construction (local subprocess, remote HTTP/SSE, a mock, …).
/// `ChatSession` owns a `Box<dyn ChatTransport>` (`!Sync`), hence `LocalStorage`.
///
/// Assistant/user messages render markdown via the in-crate
/// [`MarkdownView`](crate::markdown::MarkdownView); thinking/tool/system messages
/// render as plain text.
///
/// Feature parity with the d2d-ui desktop chat: a multiline composer
/// (Enter=send, Shift+Enter=newline), a Stop button that cancels the in-flight
/// turn (`ChatSession::cancel`), a "Thinking…" indicator before the first token,
/// auto-stick-to-bottom scrolling, per-message copy, welcome/prompt chips, and a
/// header with a New-session (restart) action and optional subtitle.
///
/// ## Staying generic
/// The panel adapts to the SELECTED backend's `ai_chat_core::Capabilities` —
/// the Model row becomes a free-text box, a picker or a pinned value, the
/// permission row appears only where permission modes exist, and an
/// unsupported toggle is disabled and explained rather than hidden — but it
/// knows no provider by name. Provider-specific levers arrive through the
/// `settings_extra` slot, provider narration through `annotations`, and every
/// visible string through `texts`. No API-key field belongs here: keys must
/// never travel through a panel whose settings become an `ai_chat_core`
/// wire type.
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::*;
///
/// let session = StoredValue::new_local(ChatSession::new(Box::new(my_transport)));
/// view! {
///     <AiChat
///         session=session
///         subtitle="Grounded on 12 docs"
///         welcome_prompts=vec!["Summarize this".into(), "What changed?".into()]
///     />
/// }
/// ```
///
/// ## CSS
/// Every literal class this component can render (add to your `input.css`):
/// ```css
/// @source inline("chat chat-start chat-end chat-header chat-bubble chat-bubble-primary chat-bubble-info chat-bubble-warning chat-bubble-neutral chat-bubble-ghost chat-image avatar avatar-placeholder");
/// @source inline("link text-error");
/// @source inline("flex flex-col flex-1 flex-wrap items-center justify-between justify-start gap-1 gap-2 h-full min-h-0 w-full w-72 w-52 w-6 h-6 overflow-y-auto p-1 p-2 p-3 p-4 space-y-3 space-y-2 contents");
/// @source inline("border-t border-b border-base-300 text-xs text-sm opacity-50 opacity-60 text-right whitespace-pre-wrap resize-none max-h-[320px] text-[10px]");
/// @source inline("btn btn-primary btn-error btn-ghost btn-sm btn-xs textarea textarea-bordered loading loading-dots loading-sm rounded-full");
/// @source inline("select select-sm select-bordered");
/// @source inline("dropdown-content bg-base-100 rounded-box z-10 shadow border");
/// @source inline("bg-primary bg-neutral bg-info bg-base-300 text-primary-content text-neutral-content text-info-content text-base-content");
/// ```
#[component]
pub fn AiChat(
    /// The chat session (parent builds it over a `ChatTransport`).
    session: StoredValue<ChatSession, LocalStorage>,
    /// Transport poll interval in milliseconds (default 100).
    #[prop(optional, into)]
    poll_ms: Option<u64>,
    /// Placeholder for the input box.
    #[prop(optional, into)]
    placeholder: Signal<String>,
    /// Optional header subtitle (token/context usage, corpus name, …). The
    /// header span always renders; an empty string just renders as blank space.
    #[prop(optional, into)]
    subtitle: Signal<String>,
    /// Welcome/prompt chips shown on an empty transcript; clicking one sends it.
    #[prop(optional, into)]
    welcome_prompts: Signal<Vec<String>>,
    /// Show the last turn's cost/token caption (from `ChatSession::last_usage`)
    /// next to the composer hint. Defaults to `false` (no usage line) so
    /// existing consumers are unchanged; renders nothing until the first
    /// `Usage` event lands, and nothing again after a `restart`.
    #[prop(optional, into)]
    show_usage: Signal<bool>,
    /// Show a gear button in the header that opens a settings popover
    /// (model, system prompt, allowed tools, show-thinking/show-tool-calls)
    /// wired to `ChatSession::configure`. Defaults to `false` so existing
    /// consumers are unchanged.
    #[prop(optional, into)]
    show_settings: Signal<bool>,
    /// Scope options for the header scope selector (e.g. This file / This
    /// folder / Whole workspace). Empty (default) hides the selector. The
    /// component tracks the active option internally and shows its label; the
    /// host interprets the selected `id` via `on_scope_change`.
    #[prop(optional, into)]
    scopes: Signal<Vec<ChatScopeOption>>,
    /// Fired with the selected scope `id` when the user changes the scope. The
    /// host maps it to a grounding (e.g. `DocMode`) and restarts the session.
    #[prop(optional, into)]
    on_scope_change: Option<Callback<String>>,
    /// Extra classes for the root element.
    #[prop(optional, into)]
    class: &'static str,
    /// Fired once per poll in which the engine reported a document edit
    /// (`StreamEvent::DocumentChanged`). The host uses it to refresh the editor.
    #[prop(optional, into)]
    on_document_changed: Option<Callback<()>>,
    /// Fired after the session is restarted via the New-session header button.
    #[prop(optional, into)]
    on_restart: Option<Callback<()>>,
    /// Assistant attribution: the configured backend's human label (e.g.
    /// `ai_chat_core::Capabilities::label`). Drives the assistant bubbles'
    /// header + avatar initial, the "Thinking…" row, and the default composer
    /// placeholder — a Codex session reads as Codex end to end, matching the
    /// desktop (em-c7w1). Empty (default) keeps the historical "Claude".
    #[prop(optional, into)]
    assistant_label: Signal<String>,
    /// Backend capability list for a Backend picker in the settings popover
    /// (`ai_chat_core::Capabilities`, e.g. fetched from ai-chat-engine's
    /// HTTP/SSE service). Empty (default) hides the picker.
    #[prop(optional, into)]
    backends: Signal<Vec<Capabilities>>,
    /// Fired with the selected backend `id` when the user picks one; the host
    /// swaps the transport / reconfigures the session.
    #[prop(optional, into)]
    on_backend_change: Option<Callback<String>>,
    /// Which published backend is actually active, when the HOST owns that
    /// choice rather than the picker.
    ///
    /// `None` (the default) keeps the historical behaviour: the picker seeds
    /// itself from the first published backend and tracks its own selection.
    /// That is wrong for a host that switches backends by some other route —
    /// its own picker, a route change, a remount after an engine switch —
    /// because the panel then shapes the Model row, the permission row and
    /// the two toggles from the FIRST backend's `Capabilities` while running
    /// a different one, with no event to tell it otherwise. Same defect class
    /// as `permission_mode` one row down: a host-owned value the panel can
    /// only display.
    ///
    /// A value naming a backend this panel does not publish is ignored, so a
    /// stale id degrades to the seeded behaviour rather than emptying the
    /// picker.
    #[prop(optional, into)]
    active_backend_id: Signal<Option<String>>,
    /// Every label, placeholder and caption this panel renders. Defaults to
    /// the English table; `AiChatTexts::es` is the Spanish one, and a host
    /// overrides individual fields by struct-update syntax.
    #[prop(into, default = Signal::stored(AiChatTexts::default()))]
    texts: Signal<AiChatTexts>,
    /// The selected CLI permission mode, when the active backend publishes
    /// `Capabilities::permission_modes`.
    ///
    /// Host-owned rather than part of the popover's own form state, because
    /// `ai_chat_core::ChatSettings` — a WIRE type — has no field to carry it.
    /// The panel only displays this value and reports changes through
    /// `on_permission_mode_change`; the host decides what a mode means and
    /// how it reaches the backend.
    ///
    /// **A host that renders this row should seed it.** `None`, and a value
    /// this backend does not publish, both render a disabled "not set"
    /// placeholder rather than silently displaying the first mode as though it
    /// were chosen — a permission control must never claim a setting the host
    /// never made.
    #[prop(optional, into)]
    permission_mode: Signal<Option<String>>,
    /// Fired with the chosen permission mode; see `permission_mode`.
    #[prop(optional, into)]
    on_permission_mode_change: Option<Callback<String>>,
    /// Extra settings rows, rendered between the tool toggles and Apply.
    ///
    /// The panel stays generic: provider-specific levers (a temperature
    /// slider, a reasoning-effort picker) belong to the pattern above it and
    /// arrive through this slot rather than growing props here. Never put an
    /// API-key field in it — keys must not travel through this panel.
    #[prop(optional, into)]
    settings_extra: Option<ViewFn>,
    /// Host-owned rows interleaved into the transcript — engine switches,
    /// stop notices, escalation, truncation, citations. `ChatSession` has no
    /// `announce()`, so this is the only way such a row reaches the
    /// transcript, and it keeps the vocabulary in the host's hands.
    #[prop(optional, into)]
    annotations: Signal<Vec<TranscriptAnnotation>>,
    /// Fired after the user presses Retry on the terminal-error strip. The
    /// panel has already re-sent the turn via `ChatSession::retry`; the host
    /// uses this to re-check availability, log, or clear its own banner.
    #[prop(optional, into)]
    on_retry: Option<Callback<()>>,
    /// The composer's draft text, when the host wants to own it.
    ///
    /// The panel clears and reads this signal exactly as it does its own, so
    /// a host control that seeds a prompt (a quick-action chip, a template
    /// picker) writes the same place the actor types — and a draft seeded
    /// that way is still only a draft: nothing here sends. `None` keeps the
    /// draft private to the panel, which is what every existing call site
    /// gets.
    #[prop(optional)]
    composer: Option<RwSignal<String>>,
) -> impl IntoView {
    // Bumped whenever a poll() (or a send) changes the transcript, so the
    // message-list closure re-runs. `messages()` borrows the session, so we
    // snapshot to an owned Vec for rendering.
    let version = RwSignal::new(0u32);
    let input = composer.unwrap_or_else(|| RwSignal::new(String::new()));
    // Whether the transcript should follow new content; toggled by the user's scroll.
    let stick = RwSignal::new(true);
    let list_ref: NodeRef<Div> = NodeRef::new();
    let textarea_ref: NodeRef<HtmlTextarea> = NodeRef::new();
    let interval = poll_ms.unwrap_or(100);
    // The last turn's terminal `StreamEvent::Error`, if one was consumed.
    // Cleared on send, retry and restart, so it only ever describes the most
    // recent turn.
    let last_turn_error: RwSignal<Option<String>> = RwSignal::new(None);
    // Unique per mounted panel, so two panels on one page do not mint
    // colliding `label for=` targets.
    let instance = AI_CHAT_SEQ.fetch_add(1, Ordering::Relaxed);
    // Minted as `Copy` closures rather than `String`s so the same id can be
    // handed to both the `label for=` and the control's `id=` without either
    // use moving it out of the enclosing `Fn` view closure.
    let id_backend = move || format!("aichat-{instance}-backend");
    let id_model = move || format!("aichat-{instance}-model");
    let id_permission = move || format!("aichat-{instance}-permission");
    let id_system_prompt = move || format!("aichat-{instance}-system-prompt");
    let id_tools = move || format!("aichat-{instance}-tools");
    let id_show_thinking = move || format!("aichat-{instance}-show-thinking");
    let id_show_tool_calls = move || format!("aichat-{instance}-show-tool-calls");
    let id_composer = move || format!("aichat-{instance}-composer");

    // Auto-grow the composer to fit its content, up to `COMPOSER_MAX_HEIGHT_PX`
    // (beyond which it scrolls internally via the `overflow-y-auto` class).
    // Reset `height:auto` first so `scroll_height` reflects a shrink (e.g. a
    // deleted line), not just growth.
    let resize_textarea = move || {
        if let Some(el) = textarea_ref.get() {
            // `.style()` is ambiguous between `web_sys::HtmlElement`'s inherent
            // getter and tachys' `ElementExt::style` (a setter); UFCS picks the
            // web_sys inherent method unambiguously.
            let style = web_sys::HtmlElement::style(&el);
            let _ = style.set_property("height", "auto");
            let clamped = clamp_composer_height(
                el.scroll_height() as f64,
                COMPOSER_BASE_HEIGHT_PX,
                COMPOSER_MAX_HEIGHT_PX,
            );
            let _ = style.set_property("height", &format!("{clamped}px"));
        }
    };
    // Collapse back to the base height after a turn is sent/the composer is
    // cleared, rather than leaving a tall empty textarea.
    let reset_textarea_height = move || {
        if let Some(el) = textarea_ref.get() {
            let _ = web_sys::HtmlElement::style(&el)
                .set_property("height", &format!("{COMPOSER_BASE_HEIGHT_PX}px"));
        }
    };

    // Settings popover form state, seeded once from the session's current
    // `ChatSettings` at creation time. This seed read runs unconditionally on
    // every mount regardless of `show_settings`; only the popover UI itself is
    // gated behind that prop.
    let initial_settings = session.with_value(|s| s.settings().clone());
    let settings_model = RwSignal::new(initial_settings.model.clone().unwrap_or_default());
    let settings_system_prompt =
        RwSignal::new(initial_settings.system_prompt.clone().unwrap_or_default());
    let settings_tools_text = RwSignal::new(format_allowed_tools(&initial_settings.allowed_tools));
    let settings_show_thinking = RwSignal::new(initial_settings.show_thinking);
    let settings_show_tool_calls = RwSignal::new(initial_settings.show_tool_calls);

    // Rebuild a `ChatSettings` from the form fields and forward it through
    // `ChatSession::configure` (which also forwards to the transport).
    let apply_settings = move || {
        let new_settings: ChatSettings = settings_from_form_fields(
            &settings_model.get(),
            &settings_system_prompt.get(),
            &settings_tools_text.get(),
            settings_show_thinking.get(),
            settings_show_tool_calls.get(),
        );
        session.update_value(|s| s.configure(new_settings));
        version.update(|n| *n += 1);
    };

    let interval_handle = leptos::leptos_dom::helpers::set_interval_with_handle(
        move || {
            // `poll()` has ALREADY applied every event to the transcript;
            // `drain_events` only takes the separate recording buffer
            // `last_events` (see `ai-chat-core`'s `session.rs`), so reading
            // the batch here cannot starve `messages()`. This panel is the
            // only drainer in a browser shell, which is why a terminal
            // `Error` can be observed here rather than re-derived from the
            // last message's text.
            let (changed, doc_changed, turn_error) = session
                .try_update_value(|s| {
                    let changed = s.poll();
                    let mut doc_changed = false;
                    let mut turn_error = None;
                    for ev in s.drain_events() {
                        match ev {
                            ai_chat_core::StreamEvent::DocumentChanged { .. } => doc_changed = true,
                            ai_chat_core::StreamEvent::Error(message) => turn_error = Some(message),
                            _ => {}
                        }
                    }
                    (changed, doc_changed, turn_error)
                })
                .unwrap_or((false, false, None));
            if changed {
                version.update(|n| *n += 1);
            }
            if turn_error.is_some() {
                last_turn_error.set(turn_error);
            }
            if doc_changed && let Some(cb) = on_document_changed {
                cb.run(());
            }
        },
        Duration::from_millis(interval),
    );
    if let Ok(handle) = interval_handle {
        on_cleanup(move || handle.clear());
    }

    let send_text = move |text: String| {
        if text.trim().is_empty() {
            return;
        }
        let _ = session.try_update_value(|s| {
            s.send(ChatRequest {
                prompt: text,
                attachments: Vec::new(),
                page_context: None,
            })
        });
        input.set(String::new());
        reset_textarea_height();
        stick.set(true); // a fresh turn always pins to the bottom
        last_turn_error.set(None);
        version.update(|n| *n += 1);
    };

    let submit = move || send_text(input.get());

    // Re-send the failed turn without echoing a new user message
    // (`ChatSession::retry` replays `last_sent`).
    //
    // The retry is attempted FIRST and its outcome decides the strip: clearing
    // the error up front would delete the user's only signal that anything is
    // wrong the moment the retry itself fails, flip the panel to `idle`, and
    // remove the very button they would press again.
    let do_retry = move || {
        let outcome = session
            .try_update_value(|s| retry_outcome(s.retry(), s.is_waiting(), &texts.get()))
            .unwrap_or(RetryOutcome::NothingToRetry);
        match outcome {
            RetryOutcome::Sent => {
                last_turn_error.set(None);
                stick.set(true);
                version.update(|n| *n += 1);
                if let Some(cb) = on_retry {
                    cb.run(());
                }
            }
            RetryOutcome::Failed(message) => {
                last_turn_error.set(Some(message));
                version.update(|n| *n += 1);
            }
            // Nothing was re-sent, so nothing changed: leave the strip, its
            // text and the host exactly as they were.
            RetryOutcome::NothingToRetry => {}
        }
    };

    let cancel = move || {
        let _ = session.try_update_value(|s| s.cancel());
        version.update(|n| *n += 1);
    };

    let restart = move || {
        let _ = session.try_update_value(|s| s.restart());
        input.set(String::new());
        reset_textarea_height();
        stick.set(true);
        last_turn_error.set(None);
        version.update(|n| *n += 1);
        if let Some(cb) = on_restart {
            cb.run(());
        }
    };

    // Effective assistant attribution: the configured backend's label, or the
    // localized `AiChatTexts::role_assistant` when the host supplied none.
    let assistant =
        Signal::derive(move || effective_assistant_label(&assistant_label.get(), &texts.get()));

    // The transcript, with the host's annotations interleaved at their
    // anchors. Bubbles and annotation rows share one list so an annotation
    // anchored after message `i` really renders there, not in a side channel.
    let messages = move || {
        version.track();
        let label = assistant.get();
        let t = texts.get();
        let anns = annotations.get();
        let snapshot = session.with_value(|s| s.messages().to_vec());
        let slots = annotation_slots(&anns, snapshot.len());
        let mut rows: Vec<AnyView> = Vec::new();
        for i in &slots[0] {
            rows.push(view! { <AnnotationRow ann=anns[*i].clone() /> }.into_any());
        }
        for (index, m) in snapshot.into_iter().enumerate() {
            rows.push(
                view! {
                    <MessageBubble
                        msg=m
                        index=index
                        assistant_label=label.clone()
                        texts=t.clone()
                    />
                }
                .into_any(),
            );
            for i in &slots[index + 1] {
                rows.push(view! { <AnnotationRow ann=anns[*i].clone() /> }.into_any());
            }
        }
        rows
    };

    let waiting = move || {
        version.track();
        session.with_value(|s| s.is_waiting())
    };
    let streaming = move || {
        version.track();
        session.with_value(|s| s.messages().iter().any(|m| m.streaming))
    };
    let panel_state = move || {
        chat_state_attr(
            waiting(),
            streaming(),
            last_turn_error.with(Option::is_some),
        )
    };
    let thinking = move || {
        version.track();
        session
            .with_value(|s| is_thinking(s.is_waiting(), s.messages().iter().any(|m| m.streaming)))
    };
    let message_count = move || {
        version.track();
        session.with_value(|s| s.messages().len())
    };
    let show_chips = move || show_welcome_chips(message_count(), welcome_prompts.get().len());
    // Last turn's cost/token caption; tracks `version` so it refreshes after
    // each turn (a `Usage` event lands mid-poll) and clears on `restart`
    // (which resets `last_usage` to `None`).
    let usage_caption = move || {
        version.track();
        if !show_usage.get() {
            return None;
        }
        format_usage(session.with_value(|s| s.last_usage()))
    };
    // Composer hint: keybinding reminder while idle, "Esc to stop" while busy.
    // Always on (no gating prop) — a single unobtrusive caption line, matching
    // the header subtitle's text-xs/opacity-60 styling.
    let hint_text = move || composer_hint_for(waiting(), &texts.get());

    // Auto-stick-to-bottom: after each transcript change, if the user is pinned
    // to the bottom, scroll the list all the way down.
    //
    // Leptos `Effect`s run after the reactive update but *before* the browser
    // has laid out newly-appended DOM (e.g. a fresh `MessageBubble`). Reading
    // `scroll_height` synchronously here would observe a stale, too-small
    // value and under-scroll — leaving the newest message partly below the
    // fold, and potentially causing the `on:scroll` handler to see a large
    // remaining distance and flip `stick` to `false`, disengaging future
    // auto-scroll. Deferring the read+write to a `request_animation_frame`
    // callback runs it after the browser has painted the new layout, so
    // `scroll_height` reflects the settled content size.
    Effect::new(move |_| {
        version.track();
        thinking(); // also follow the thinking indicator's appearance
        if stick.get_untracked() {
            request_animation_frame(move || {
                if let Some(el) = list_ref.get() {
                    el.set_scroll_top(el.scroll_height());
                }
            });
        }
    });

    // Active scope id for the selector; defaults to the first option once
    // `scopes` is populated. The host is notified via `on_scope_change`.
    let active_scope: RwSignal<String> = RwSignal::new(String::new());
    Effect::new(move |_| {
        let opts = scopes.get();
        if active_scope.with_untracked(|a| a.is_empty())
            && let Some(first) = opts.first()
        {
            active_scope.set(first.id.clone());
        }
    });
    let scope_label = move || {
        let id = active_scope.get();
        scopes
            .get()
            .into_iter()
            .find(|o| o.id == id)
            .map(|o| o.label)
            .unwrap_or_else(|| texts.get().scope_unknown)
    };

    // Backend picker selection, seeded from the first capability like scopes.
    let internal_backend = RwSignal::new(String::new());
    Effect::new(move |_| {
        let list = backends.get();
        if internal_backend.with_untracked(|a| a.is_empty())
            && let Some(first) = list.first()
        {
            internal_backend.set(first.id.clone());
        }
    });
    // A host-owned `active_backend_id` outranks the internal seed, but only
    // when it names something this panel actually publishes.
    let active_backend = Signal::derive(move || {
        let list = backends.get();
        match active_backend_id.get() {
            Some(id) if list.iter().any(|c| c.id == id) => id,
            _ => internal_backend.get(),
        }
    });

    // The `Capabilities` whose `id` matches the selected backend, if any. It
    // drives which settings rows exist and in what shape; `None` (no backend
    // list, or an id with no match) keeps the legacy popover exactly.
    let selected_caps = move || {
        let id = active_backend.get();
        backends.get().into_iter().find(|c| c.id == id)
    };
    let rows = move || settings_rows_for(selected_caps().as_ref());
    let permission_options = move || {
        selected_caps()
            .map(|c| c.permission_modes)
            .unwrap_or_default()
    };
    // Stored so the popover's `Fn` render closure can run the slot on every
    // pass; consuming the `Option<ViewFn>` directly would make that closure
    // `FnOnce`.
    let settings_extra = StoredValue::new_local(settings_extra);

    // An unsupported toggle is rendered DISABLED, never hidden — honesty over
    // tidiness — so its `title` is the only explanation the user gets. `None`
    // omits the attribute entirely on a supported toggle.
    let unsupported_thinking_title =
        move || (!rows().thinking).then(|| texts.get().unsupported_here);
    let unsupported_tool_calls_title =
        move || (!rows().tool_calls).then(|| texts.get().unsupported_here);

    // Keep the form state and the rendered Model control in agreement: a
    // backend that publishes a closed model list (`Pinned` or `Select`) snaps
    // `settings_model` onto that list, so Apply can never ship the previous
    // backend's model id while the popover displays one of the new backend's.
    Effect::new(move |_| {
        let mode = model_field_mode(selected_caps().as_ref());
        let reconciled = settings_model.with_untracked(|cur| reconcile_model_for(&mode, cur));
        if let Some(m) = reconciled {
            settings_model.set(m);
        }
    });

    // The Model row's control, in whichever of the three shapes the selected
    // backend's `Capabilities::models` calls for.
    let model_field = move || {
        let id = id_model();
        match model_field_mode(selected_caps().as_ref()) {
            ModelFieldMode::FreeText => {
                let hint = texts.get().model_free_text_hint;
                view! {
                    <Input
                        attr:id=id
                        attr:data-ai-chat-model-input=""
                        size=crate::components::InputSize::Sm
                        value=Signal::derive(move || settings_model.get())
                        on_input=move |v| settings_model.set(v)
                        placeholder=hint
                    />
                }
                .into_any()
            }
            ModelFieldMode::Select(models) => {
                let options = models
                    .into_iter()
                    .map(|m| {
                        let value = m.clone();
                        let is_selected = {
                            let m = m.clone();
                            move || settings_model.get() == m
                        };
                        view! { <option value=value selected=is_selected>{m}</option> }
                    })
                    .collect_view();
                view! {
                    <select
                        id=id
                        data-ai-chat-model-select=""
                        class="select select-sm select-bordered w-full"
                        on:change=move |e| settings_model.set(event_target_value(&e))
                    >
                        {options}
                    </select>
                }
                .into_any()
            }
            ModelFieldMode::Pinned(m) => {
                let value = m.clone();
                view! {
                    <select
                        id=id
                        data-ai-chat-model-fixed=""
                        class="select select-sm select-bordered w-full"
                        disabled=true
                    >
                        <option value=value selected=true>{m}</option>
                    </select>
                }
                .into_any()
            }
        }
    };

    view! {
        <div
            class=move || merge_classes!("lds-aichat flex flex-col h-full min-h-0", class)
            data-ai-chat-panel=""
            data-ai-chat-state=panel_state
        >
            <div class="lds-aichat-header border-b border-base-300 p-2 flex items-center justify-between gap-2">
                <span class="text-sm opacity-60 truncate">{move || subtitle.get()}</span>
                <div class="flex items-center gap-2">
                    <Show when=move || !scopes.get().is_empty()>
                        <Dropdown alignment=DropdownAlignment::End class="lds-aichat-scope">
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs"
                                title=move || texts.get().scope
                                aria-label=move || texts.get().scope
                            >
                                {scope_label}
                                " \u{25be}"
                            </button>
                            <DropdownContent class="dropdown-content bg-base-100 rounded-box z-10 w-52 p-1 shadow border border-base-300">
                                // `DropdownContent` renders a `<div>` off-menu
                                // (ldui-afan), so this wrapper is a `<div>` too:
                                // an `<li>` outside a list is an axe `listitem`
                                // violation, which is how the change was found
                                // (ldui-q9zf). `contents` makes the wrapper
                                // generate no box, so the children keep the
                                // popover's own flow and the layout is unchanged.
                                <div class="contents">
                                <For each=move || scopes.get() key=|o| o.id.clone() let:opt>
                                    {
                                        let id = opt.id.clone();
                                        let label = opt.label.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class="btn btn-ghost btn-xs w-full justify-start"
                                                on:click=move |_| {
                                                    active_scope.set(id.clone());
                                                    if let Some(cb) = on_scope_change {
                                                        cb.run(id.clone());
                                                    }
                                                }
                                            >
                                                {label}
                                            </button>
                                        }
                                    }
                                </For>
                                </div>
                            </DropdownContent>
                        </Dropdown>
                    </Show>
                    <Show when=move || show_settings.get()>
                        <Dropdown alignment=DropdownAlignment::End class="lds-aichat-settings">
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs"
                                data-ai-chat-settings=""
                                title=move || texts.get().settings
                                aria-label=move || texts.get().settings
                            >
                                "\u{2699}"
                            </button>
                            <DropdownContent class="dropdown-content bg-base-100 rounded-box z-10 w-72 p-3 shadow border border-base-300">
                                // See the scope dropdown above: the popover is
                                // a `<div>`, so its wrapper is too. `space-y-2`
                                // lives on the wrapper because it spaces a
                                // parent's OWN children — on the popover it
                                // would space the single wrapper against
                                // nothing.
                                <div class="space-y-2">
                                <Show when=move || !backends.get().is_empty()>
                                    <label class="flex flex-col gap-1 text-xs" for=id_backend>
                                        <span class="opacity-60">{move || texts.get().backend}</span>
                                        <select
                                            id=id_backend
                                            data-ai-chat-backend=""
                                            class="select select-sm select-bordered w-full"
                                            on:change=move |e| {
                                                let id = event_target_value(&e);
                                                internal_backend.set(id.clone());
                                                if let Some(cb) = on_backend_change {
                                                    cb.run(id);
                                                }
                                            }
                                        >
                                            <For each=move || backends.get() key=|b| b.id.clone() let:b>
                                                {
                                                    let id = b.id.clone();
                                                    let label = b.label.clone();
                                                    let sel = {
                                                        let id = id.clone();
                                                        move || active_backend.get() == id
                                                    };
                                                    view! {
                                                        <option value=id selected=sel>{label}</option>
                                                    }
                                                }
                                            </For>
                                        </select>
                                    </label>
                                </Show>
                                <label class="flex flex-col gap-1 text-xs" for=id_model>
                                    <span class="opacity-60">{move || texts.get().model}</span>
                                    {model_field}
                                </label>
                                <Show when=move || rows().permission>
                                    <label
                                        class="flex flex-col gap-1 text-xs"
                                        for=id_permission
                                    >
                                        <span class="opacity-60">
                                            {move || texts.get().permission_mode}
                                        </span>
                                        <select
                                            id=id_permission
                                            data-ai-chat-permission-select=""
                                            class="select select-sm select-bordered w-full"
                                            on:change=move |e| {
                                                let picked = event_target_value(&e);
                                                // The placeholder carries an
                                                // empty value and is disabled;
                                                // never report it as a choice.
                                                if !picked.is_empty()
                                                    && let Some(cb) = on_permission_mode_change
                                                {
                                                    cb.run(picked);
                                                }
                                            }
                                        >
                                            {move || {
                                                let options = permission_options();
                                                let active = permission_mode.get();
                                                let chosen = permission_selection(
                                                    active.as_deref(),
                                                    &options,
                                                );
                                                // Nothing chosen, or a value
                                                // this backend does not
                                                // publish: show "not set"
                                                // rather than letting the
                                                // browser display the first
                                                // mode as if it were chosen.
                                                let placeholder = chosen.is_none().then(|| {
                                                    let label = texts.get().permission_mode_unset;
                                                    view! {
                                                        <option value="" disabled selected=true>
                                                            {label}
                                                        </option>
                                                    }
                                                });
                                                let items = options
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(i, m)| {
                                                        let is_selected = chosen == Some(i);
                                                        let value = m.clone();
                                                        view! {
                                                            <option value=value selected=is_selected>
                                                                {m}
                                                            </option>
                                                        }
                                                    })
                                                    .collect_view();
                                                view! { {placeholder} {items} }
                                            }}
                                        </select>
                                    </label>
                                </Show>
                                <label
                                    class="flex flex-col gap-1 text-xs"
                                    for=id_system_prompt
                                >
                                    <span class="opacity-60">
                                        {move || texts.get().system_prompt}
                                    </span>
                                    <Textarea
                                        attr:id=id_system_prompt
                                        size=crate::components::TextareaSize::Sm
                                        rows=2
                                        value=Signal::derive(move || settings_system_prompt.get())
                                        on_input=move |v| settings_system_prompt.set(v)
                                    />
                                </label>
                                <label class="flex flex-col gap-1 text-xs" for=id_tools>
                                    <span class="opacity-60">
                                        {move || texts.get().allowed_tools}
                                    </span>
                                    <Input
                                        attr:id=id_tools
                                        size=crate::components::InputSize::Sm
                                        value=Signal::derive(move || settings_tools_text.get())
                                        on_input=move |v| settings_tools_text.set(v)
                                        placeholder=Signal::derive(move || {
                                            texts.get().allowed_tools_hint
                                        })
                                    />
                                </label>
                                <label
                                    class="flex items-center justify-between gap-2 text-xs"
                                    for=id_show_thinking
                                >
                                    <span class="opacity-60">
                                        {move || texts.get().show_thinking}
                                    </span>
                                    <Toggle
                                        attr:id=id_show_thinking
                                        attr:data-ai-chat-show-thinking=""
                                        attr:title=unsupported_thinking_title
                                        size=crate::components::ToggleSize::Sm
                                        disabled=Signal::derive(move || !rows().thinking)
                                        prop:checked=move || settings_show_thinking.get()
                                        on:change=move |e| {
                                            settings_show_thinking.set(event_target_checked(&e))
                                        }
                                    />
                                </label>
                                <label
                                    class="flex items-center justify-between gap-2 text-xs"
                                    for=id_show_tool_calls
                                >
                                    <span class="opacity-60">
                                        {move || texts.get().show_tool_calls}
                                    </span>
                                    <Toggle
                                        attr:id=id_show_tool_calls
                                        attr:data-ai-chat-show-tool-calls=""
                                        attr:title=unsupported_tool_calls_title
                                        size=crate::components::ToggleSize::Sm
                                        disabled=Signal::derive(move || !rows().tool_calls)
                                        prop:checked=move || settings_show_tool_calls.get()
                                        on:change=move |e| {
                                            settings_show_tool_calls.set(event_target_checked(&e))
                                        }
                                    />
                                </label>
                                {move || {
                                    settings_extra
                                        .with_value(|slot| slot.as_ref().map(|s| s.run()))
                                }}
                                <button
                                    type="button"
                                    class="btn btn-primary btn-xs w-full"
                                    data-ai-chat-apply=""
                                    on:click=move |_| apply_settings()
                                >
                                    {move || texts.get().apply}
                                </button>
                                </div>
                            </DropdownContent>
                        </Dropdown>
                    </Show>
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs"
                        data-ai-chat-new-session=""
                        on:click=move |_| restart()
                    >
                        {move || texts.get().new_session}
                    </button>
                </div>
            </div>

            <div
                node_ref=list_ref
                data-chat-history=""
                class="lds-aichat-list flex-1 min-h-0 overflow-y-auto p-4 space-y-3"
                on:scroll=move |_| {
                    if let Some(el) = list_ref.get() {
                        stick.set(should_stick_to_bottom(
                            el.scroll_top() as f64,
                            el.scroll_height() as f64,
                            el.client_height() as f64,
                            STICK_THRESHOLD_PX,
                        ));
                    }
                }
            >
                {messages}
                <Show when=thinking>
                    <div class="chat chat-start">
                        <div class="chat-header text-xs opacity-60">
                            {move || assistant.get()}
                        </div>
                        <div class="chat-bubble chat-bubble-ghost">
                            <span
                                class="loading loading-dots loading-sm"
                                role="status"
                                aria-label=move || texts.get().thinking
                            ></span>
                        </div>
                    </div>
                </Show>
                <Show when=show_chips>
                    <div class="lds-aichat-chips flex flex-wrap gap-2">
                        {move || {
                            welcome_prompts
                                .get()
                                .into_iter()
                                .map(|p| {
                                    let prompt = p.clone();
                                    view! {
                                        <button
                                            type="button"
                                            class="btn btn-sm btn-ghost"
                                            on:click=move |_| send_text(prompt.clone())
                                        >
                                            {p}
                                        </button>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </Show>
            </div>

            <Show when=move || last_turn_error.with(Option::is_some)>
                <div
                    class="lds-aichat-error border-t border-base-300 p-2 flex items-center justify-between gap-2 text-xs"
                    role="alert"
                    data-ai-chat-error=""
                >
                    <span class="text-error">
                        {move || last_turn_error.get().unwrap_or_default()}
                    </span>
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs"
                        data-ai-chat-retry=""
                        on:click=move |_| do_retry()
                    >
                        {move || texts.get().retry}
                    </button>
                </div>
            </Show>

            <div class="lds-aichat-input border-t border-base-300 p-3 flex flex-col gap-1">
                <div class="flex gap-2">
                    // The textarea's accessible name. sr-only: the placeholder
                    // carries the visible hint, but a placeholder stops being a
                    // name the moment text is typed (ldui-iilm.11, drift
                    // input-outside-field).
                    // ldui-iay0: the SAME text the placeholder shows -- the
                    // template substituted ({assistant} -> the engine's
                    // name), or the host's own placeholder -- never the raw
                    // `composer_placeholder` template.
                    <label class="sr-only" for=id_composer data-ai-chat-composer-label="">
                        {move || {
                            let p = placeholder.get();
                            if p.is_empty() {
                                composer_placeholder_for(&assistant.get(), &texts.get())
                            } else {
                                p
                            }
                        }}
                    </label>
                    <textarea
                        node_ref=textarea_ref
                        id=id_composer
                        data-ai-chat-composer=""
                        class="textarea textarea-bordered flex-1 resize-none max-h-[320px] overflow-y-auto"
                        rows="2"
                        placeholder=move || {
                            let p = placeholder.get();
                            if p.is_empty() {
                                composer_placeholder_for(&assistant.get(), &texts.get())
                            } else {
                                p
                            }
                        }
                        prop:value=move || input.get()
                        on:input=move |e| {
                            input.set(event_target_value(&e));
                            resize_textarea();
                        }
                        on:keydown=move |e| {
                            // Esc cancels an in-flight turn — matches the composer
                            // hint's "Esc to stop" while `waiting`.
                            if e.key() == "Escape" && waiting() {
                                e.prevent_default();
                                cancel();
                                return;
                            }
                            match composer_key_action(&e.key(), e.shift_key(), e.is_composing()) {
                                ComposerAction::Send => {
                                    e.prevent_default();
                                    submit();
                                }
                                // Newline / Ignore: let the textarea handle it natively.
                                ComposerAction::Newline | ComposerAction::Ignore => {}
                            }
                        }
                    />
                    <Show
                        when=waiting
                        fallback=move || view! {
                            <button
                                type="button"
                                class="btn btn-primary"
                                data-ai-chat-send=""
                                on:click=move |_| submit()
                            >
                                {move || texts.get().send}
                            </button>
                        }
                    >
                        <button
                            type="button"
                            class="btn btn-error"
                            data-ai-chat-stop=""
                            on:click=move |_| cancel()
                        >
                            {move || texts.get().stop}
                        </button>
                    </Show>
                </div>
                <div class="flex items-center justify-between gap-2 text-xs opacity-60">
                    <span>{hint_text}</span>
                    <Show when=move || usage_caption().is_some()>
                        <span class="lds-aichat-usage text-right">
                            {move || usage_caption().unwrap_or_default()}
                        </span>
                    </Show>
                </div>
            </div>
        </div>
    }
}

/// One transcript message as a daisyUI chat bubble, with a copy button.
#[component]
fn MessageBubble(
    msg: ChatMessage,
    /// The message's position in `ChatSession::messages`, exposed as
    /// `data-chat-index` so a browser assertion can name a bubble by
    /// position without relying on a positional CSS selector.
    index: usize,
    /// Assistant attribution label (empty = historical "Claude").
    #[prop(optional, into)]
    assistant_label: String,
    /// Localized copy for the bubble's role header.
    #[prop(optional, into)]
    texts: AiChatTexts,
) -> impl IntoView {
    let (side, bubble) = role_classes(&msg.role);
    let role_attr = role_data_attr(&msg.role);
    // `None` renders no attribute at all, so `[data-chat-tool-phase]`
    // selects a tool call or its result and nothing else.
    let tool_phase_attr = tool_phase_data_attr(&msg.meta);
    let label = role_label_for(&msg.role, &assistant_label, &texts);
    let copy_label = texts.copy.clone();
    let copy_title = texts.copy_message.clone();
    let avatar_bg = role_avatar_bg(&msg.role);
    let avatar_initial = role_avatar_initial_with(&msg.role, &assistant_label);
    let md = is_markdown(&msg.role);
    let streaming = msg.streaming;
    let content = msg.content.clone();
    let copy_src = msg.content.clone();

    let body = if md {
        let src = content.clone();
        view! { <MarkdownView source=Signal::derive(move || src.clone()) inline=false /> }
            .into_any()
    } else {
        view! { <span class="whitespace-pre-wrap">{content.clone()}</span> }.into_any()
    };

    let copy = move |_| {
        let _text = copy_src.clone();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.navigator().clipboard().write_text(&_text);
            }
        }
    };

    view! {
        <div
            class=move || merge_classes!("chat ld-aichat-msg-in", side)
            data-chat-role=role_attr
            data-chat-tool-phase=tool_phase_attr
            data-chat-index=index
        >
            <div class="chat-image avatar avatar-placeholder">
                <div class=move || {
                    merge_classes!("w-6 h-6 rounded-full", avatar_bg)
                }>
                    <span class="text-[10px]">{avatar_initial}</span>
                </div>
            </div>
            <div class="chat-header text-xs opacity-60 flex items-center gap-2">
                {label}
                <button
                    type="button"
                    class="btn btn-ghost btn-xs opacity-50"
                    title=copy_title
                    on:click=copy
                >
                    {copy_label}
                </button>
            </div>
            <div class=move || merge_classes!("chat-bubble", bubble)>
                {body}
                {streaming.then(|| view! { <span class="lds-aichat-cursor">"\u{258d}"</span> })}
            </div>
        </div>
    }
}

/// One host-supplied [`TranscriptAnnotation`] as a full-width transcript row.
///
/// It is deliberately NOT a `ChatMessage`: annotations are the host's own
/// narration ("Switched to Codex", "Response truncated", a citation list),
/// never part of the model's transcript, and `ChatSession` has no
/// `announce()` to put them in. The row carries `data-chat-role="notice"` so
/// a transcript sweep can tell narration from conversation.
#[component]
fn AnnotationRow(ann: TranscriptAnnotation) -> impl IntoView {
    let kind = ann.kind.as_str();
    let bubble = annotation_bubble_class(ann.kind);
    let body = match ann.body {
        AnnotationBody::Text(text) => {
            view! { <span class="whitespace-pre-wrap">{text}</span> }.into_any()
        }
        AnnotationBody::Citations(citations) => {
            let items = citations
                .into_iter()
                .map(|c| match c.href {
                    Some(href) => view! {
                        <a class="link" href=href target="_blank" rel="noreferrer">
                            {c.label}
                        </a>
                    }
                    .into_any(),
                    None => view! { <span class="opacity-60">{c.label}</span> }.into_any(),
                })
                .collect_view();
            view! { <div class="flex flex-wrap gap-2">{items}</div> }.into_any()
        }
    };

    view! {
        <div
            class="chat chat-start lds-aichat-annotation"
            data-chat-role="notice"
            data-chat-annotation-kind=kind
        >
            <div class=move || merge_classes!("chat-bubble w-full text-xs", bubble)>{body}</div>
        </div>
    }
}
