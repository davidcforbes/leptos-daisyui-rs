use super::style::{
    ComposerAction, composer_key_action, is_markdown, is_thinking, role_classes, role_label,
    should_stick_to_bottom, show_welcome_chips,
};
use crate::markdown::MarkdownView;
use crate::merge_classes;
use ai_chat_core::{ChatMessage, ChatRequest, ChatSession};
use leptos::html::Div;
use leptos::prelude::*;
use std::time::Duration;

/// Distance (px) from the bottom within which the transcript keeps auto-scrolling.
const STICK_THRESHOLD_PX: f64 = 40.0;

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
/// @source inline("chat chat-start chat-end chat-header chat-bubble chat-bubble-primary chat-bubble-info chat-bubble-neutral chat-bubble-ghost");
/// @source inline("flex flex-col flex-1 flex-wrap items-center justify-between gap-2 h-full min-h-0 w-full overflow-y-auto p-2 p-3 p-4 space-y-3");
/// @source inline("border-t border-b border-base-300 text-xs text-sm opacity-50 opacity-60 whitespace-pre-wrap resize-none");
/// @source inline("btn btn-primary btn-error btn-ghost btn-sm btn-xs textarea textarea-bordered loading loading-dots loading-sm");
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
    let interval = poll_ms.unwrap_or(100);

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
            if doc_changed
                && let Some(cb) = on_document_changed
            {
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
        session.with_value(|s| {
            is_thinking(s.is_waiting(), s.messages().iter().any(|m| m.streaming))
        })
    };
    let message_count = move || {
        version.track();
        session.with_value(|s| s.messages().len())
    };
    let show_chips = move || show_welcome_chips(message_count(), welcome_prompts.get().len());

    // Auto-stick-to-bottom: after each transcript change, if the user is pinned
    // to the bottom, scroll the list all the way down. Runs after DOM paint.
    Effect::new(move |_| {
        version.track();
        thinking(); // also follow the thinking indicator's appearance
        if stick.get_untracked()
            && let Some(el) = list_ref.get()
        {
            el.set_scroll_top(el.scroll_height());
        }
    });

    view! {
        <div class=move || merge_classes!("lds-aichat flex flex-col h-full min-h-0", class)>
            <div class="lds-aichat-header border-b border-base-300 p-2 flex items-center justify-between gap-2">
                <span class="text-sm opacity-60 truncate">{move || subtitle.get()}</span>
                <button
                    type="button"
                    class="btn btn-ghost btn-xs"
                    on:click=move |_| restart()
                >
                    "New session"
                </button>
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

            <div class="lds-aichat-input border-t border-base-300 p-3 flex gap-2">
                <textarea
                    class="textarea textarea-bordered flex-1 resize-none"
                    rows="2"
                    placeholder=move || {
                        let p = placeholder.get();
                        if p.is_empty() { "Ask Claude about this document\u{2026}".to_string() } else { p }
                    }
                    prop:value=move || input.get()
                    on:input=move |e| input.set(event_target_value(&e))
                    on:keydown=move |e| {
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
        </div>
    }
}

/// One transcript message as a daisyUI chat bubble, with a copy button.
#[component]
fn MessageBubble(msg: ChatMessage) -> impl IntoView {
    let (side, bubble) = role_classes(&msg.role);
    let label = role_label(&msg.role);
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
        <div class=move || merge_classes!("chat", side)>
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
