use super::style::{
    ComposerAction, clamp_composer_height, composer_hint, composer_key_action, is_markdown,
    is_thinking, role_avatar_bg, role_avatar_initial, role_classes, role_label,
    should_stick_to_bottom, show_welcome_chips,
};
use super::types::{format_allowed_tools, format_usage, settings_from_form_fields};
use crate::components::{Dropdown, DropdownAlignment, DropdownContent, Input, Textarea, Toggle};
use crate::markdown::MarkdownView;
use crate::merge_classes;
use ai_chat_core::{ChatMessage, ChatRequest, ChatSession, ChatSettings};
use leptos::html::{Div, Textarea as HtmlTextarea};
use leptos::prelude::*;
use std::time::Duration;

/// Distance (px) from the bottom within which the transcript keeps auto-scrolling.
const STICK_THRESHOLD_PX: f64 = 40.0;
/// Composer's collapsed height (px) — matches the old `rows="2"` sizing.
const COMPOSER_BASE_HEIGHT_PX: f64 = 48.0;
/// Composer's max auto-grow height (px) before it scrolls internally.
const COMPOSER_MAX_HEIGHT_PX: f64 = 320.0;

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
/// @source inline("chat chat-start chat-end chat-header chat-bubble chat-bubble-primary chat-bubble-info chat-bubble-neutral chat-bubble-ghost chat-image avatar placeholder");
/// @source inline("flex flex-col flex-1 flex-wrap items-center justify-between gap-1 gap-2 h-full min-h-0 w-full w-72 w-6 h-6 overflow-y-auto p-2 p-3 p-4 space-y-3 space-y-2");
/// @source inline("border-t border-b border-base-300 text-xs text-sm opacity-50 opacity-60 text-right whitespace-pre-wrap resize-none max-h-[320px] text-[10px]");
/// @source inline("btn btn-primary btn-error btn-ghost btn-sm btn-xs textarea textarea-bordered loading loading-dots loading-sm rounded-full");
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
) -> impl IntoView {
    // Bumped whenever a poll() (or a send) changes the transcript, so the
    // message-list closure re-runs. `messages()` borrows the session, so we
    // snapshot to an owned Vec for rendering.
    let version = RwSignal::new(0u32);
    let input = RwSignal::new(String::new());
    // Whether the transcript should follow new content; toggled by the user's scroll.
    let stick = RwSignal::new(true);
    let list_ref: NodeRef<Div> = NodeRef::new();
    let textarea_ref: NodeRef<HtmlTextarea> = NodeRef::new();
    let interval = poll_ms.unwrap_or(100);

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
            let (changed, doc_changed) = session
                .try_update_value(|s| {
                    let changed = s.poll();
                    let doc_changed = s
                        .drain_events()
                        .iter()
                        .any(|e| matches!(e, ai_chat_core::StreamEvent::DocumentChanged { .. }));
                    (changed, doc_changed)
                })
                .unwrap_or((false, false));
            if changed {
                version.update(|n| *n += 1);
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
        version.update(|n| *n += 1);
    };

    let submit = move || send_text(input.get());

    let cancel = move || {
        let _ = session.try_update_value(|s| s.cancel());
        version.update(|n| *n += 1);
    };

    let restart = move || {
        let _ = session.try_update_value(|s| s.restart());
        input.set(String::new());
        reset_textarea_height();
        stick.set(true);
        version.update(|n| *n += 1);
        if let Some(cb) = on_restart {
            cb.run(());
        }
    };

    let messages = move || {
        version.track();
        let snapshot = session.with_value(|s| s.messages().to_vec());
        snapshot
            .into_iter()
            .map(|m| view! { <MessageBubble msg=m /> })
            .collect_view()
    };

    let waiting = move || {
        version.track();
        session.with_value(|s| s.is_waiting())
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
    let hint_text = move || composer_hint(waiting());

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

    view! {
        <div class=move || merge_classes!("lds-aichat flex flex-col h-full min-h-0", class)>
            <div class="lds-aichat-header border-b border-base-300 p-2 flex items-center justify-between gap-2">
                <span class="text-sm opacity-60 truncate">{move || subtitle.get()}</span>
                <div class="flex items-center gap-2">
                    <Show when=move || show_settings.get()>
                        <Dropdown alignment=DropdownAlignment::End class="lds-aichat-settings">
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs"
                                title="Chat settings"
                                aria-label="Chat settings"
                            >
                                "\u{2699}"
                            </button>
                            <DropdownContent class="dropdown-content bg-base-100 rounded-box z-10 w-72 p-3 shadow border border-base-300 space-y-2">
                                <label class="flex flex-col gap-1 text-xs">
                                    <span class="opacity-60">"Model"</span>
                                    <Input
                                        size=crate::components::InputSize::Sm
                                        value=Signal::derive(move || settings_model.get())
                                        on_input=move |v| settings_model.set(v)
                                        placeholder="(transport default)"
                                    />
                                </label>
                                <label class="flex flex-col gap-1 text-xs">
                                    <span class="opacity-60">"System prompt"</span>
                                    <Textarea
                                        size=crate::components::TextareaSize::Sm
                                        rows=2
                                        value=Signal::derive(move || settings_system_prompt.get())
                                        on_input=move |v| settings_system_prompt.set(v)
                                    />
                                </label>
                                <label class="flex flex-col gap-1 text-xs">
                                    <span class="opacity-60">"Allowed tools (comma-separated)"</span>
                                    <Input
                                        size=crate::components::InputSize::Sm
                                        value=Signal::derive(move || settings_tools_text.get())
                                        on_input=move |v| settings_tools_text.set(v)
                                        placeholder="read, write"
                                    />
                                </label>
                                <label class="flex items-center justify-between gap-2 text-xs">
                                    <span class="opacity-60">"Show thinking"</span>
                                    <Toggle
                                        size=crate::components::ToggleSize::Sm
                                        prop:checked=move || settings_show_thinking.get()
                                        on:change=move |e| {
                                            settings_show_thinking.set(event_target_checked(&e))
                                        }
                                    />
                                </label>
                                <label class="flex items-center justify-between gap-2 text-xs">
                                    <span class="opacity-60">"Show tool calls"</span>
                                    <Toggle
                                        size=crate::components::ToggleSize::Sm
                                        prop:checked=move || settings_show_tool_calls.get()
                                        on:change=move |e| {
                                            settings_show_tool_calls.set(event_target_checked(&e))
                                        }
                                    />
                                </label>
                                <button
                                    type="button"
                                    class="btn btn-primary btn-xs w-full"
                                    on:click=move |_| apply_settings()
                                >
                                    "Apply"
                                </button>
                            </DropdownContent>
                        </Dropdown>
                    </Show>
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs"
                        on:click=move |_| restart()
                    >
                        "New session"
                    </button>
                </div>
            </div>

            <div
                node_ref=list_ref
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
                        <div class="chat-header text-xs opacity-60">"Claude"</div>
                        <div class="chat-bubble chat-bubble-ghost">
                            <span class="loading loading-dots loading-sm"></span>
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

            <div class="lds-aichat-input border-t border-base-300 p-3 flex flex-col gap-1">
                <div class="flex gap-2">
                    <textarea
                        node_ref=textarea_ref
                        class="textarea textarea-bordered flex-1 resize-none max-h-[320px] overflow-y-auto"
                        rows="2"
                        placeholder=move || {
                            let p = placeholder.get();
                            if p.is_empty() { "Ask Claude about this document\u{2026}".to_string() } else { p }
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
                                on:click=move |_| submit()
                            >
                                "Send"
                            </button>
                        }
                    >
                        <button
                            type="button"
                            class="btn btn-error"
                            on:click=move |_| cancel()
                        >
                            "Stop"
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
fn MessageBubble(msg: ChatMessage) -> impl IntoView {
    let (side, bubble) = role_classes(&msg.role);
    let label = role_label(&msg.role);
    let avatar_bg = role_avatar_bg(&msg.role);
    let avatar_initial = role_avatar_initial(&msg.role);
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
        <div class=move || merge_classes!("chat ld-aichat-msg-in", side)>
            <div class="chat-image avatar placeholder">
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
                    title="Copy message"
                    on:click=copy
                >
                    "Copy"
                </button>
            </div>
            <div class=move || merge_classes!("chat-bubble", bubble)>
                {body}
                {streaming.then(|| view! { <span class="lds-aichat-cursor">"\u{258d}"</span> })}
            </div>
        </div>
    }
}
