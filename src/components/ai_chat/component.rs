use super::style::{is_markdown, role_classes, role_label};
use crate::markdown::MarkdownView;
use crate::merge_classes;
use ai_chat_core::{ChatMessage, ChatRequest, ChatSession};
use leptos::prelude::*;
use std::time::Duration;

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
/// ### Add to `input.css`
/// ```css
/// @source inline("chat chat-start chat-end chat-header chat-bubble chat-bubble-primary chat-bubble-info chat-bubble-neutral chat-bubble-ghost");
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
    /// Extra classes for the root element.
    #[prop(optional, into)]
    class: &'static str,
    /// Fired once per poll in which the engine reported a document edit
    /// (`StreamEvent::DocumentChanged`). The host uses it to refresh the editor.
    #[prop(optional, into)]
    on_document_changed: Option<Callback<()>>,
) -> impl IntoView {
    // Bumped whenever a poll() (or a send) changes the transcript, so the
    // message-list closure re-runs. `messages()` borrows the session, so we
    // snapshot to an owned Vec for rendering.
    let version = RwSignal::new(0u32);
    let input = RwSignal::new(String::new());
    let interval = poll_ms.unwrap_or(100);

    set_interval(
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

    let submit = move || {
        let text = input.get();
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
        version.update(|n| *n += 1);
    };

    let messages = move || {
        version.track();
        let snapshot = session.with_value(|s| s.messages().to_vec());
        snapshot
            .into_iter()
            .map(|m| view! { <MessageBubble msg=m /> })
            .collect_view()
    };

    let waiting = move || session.with_value(|s| s.is_waiting());

    view! {
        <div class=move || merge_classes!("lds-aichat flex flex-col h-full", class)>
            <div class="lds-aichat-list flex-1 overflow-y-auto p-4 space-y-3">
                {messages}
            </div>
            <div class="lds-aichat-input border-t border-base-300 p-3 flex gap-2">
                <input
                    class="input input-bordered flex-1"
                    placeholder=move || {
                        let p = placeholder.get();
                        if p.is_empty() { "Ask Claude about this document\u{2026}".to_string() } else { p }
                    }
                    prop:value=move || input.get()
                    on:input=move |e| input.set(event_target_value(&e))
                    on:keydown=move |e| {
                        if e.key() == "Enter" && !e.shift_key() {
                            e.prevent_default();
                            submit();
                        }
                    }
                />
                <button
                    class="btn btn-primary"
                    prop:disabled=waiting
                    on:click=move |_| submit()
                >
                    "Send"
                </button>
            </div>
        </div>
    }
}

/// One transcript message as a daisyUI chat bubble.
#[component]
fn MessageBubble(msg: ChatMessage) -> impl IntoView {
    let (side, bubble) = role_classes(&msg.role);
    let label = role_label(&msg.role);
    let md = is_markdown(&msg.role);
    let streaming = msg.streaming;
    let content = msg.content.clone();

    let body = if md {
        let src = content.clone();
        view! { <MarkdownView source=Signal::derive(move || src.clone()) inline=false /> }
            .into_any()
    } else {
        view! { <span class="whitespace-pre-wrap">{content.clone()}</span> }.into_any()
    };

    view! {
        <div class=move || merge_classes!("chat", side)>
            <div class="chat-header text-xs opacity-60">{label}</div>
            <div class=move || merge_classes!("chat-bubble", bubble)>
                {body}
                {streaming.then(|| view! { <span class="lds-aichat-cursor">"\u{258d}"</span> })}
            </div>
        </div>
    }
}
