use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use std::collections::VecDeque;

/// A self-contained scripted transport so the demo needs no backend: each
/// `send()` queues a streaming markdown reply that drains one fragment per
/// poll tick (the component polls on a timer), so you see it stream in.
struct DemoTransport {
    queue: VecDeque<StreamEvent>,
    turn: u32,
}

impl ChatTransport for DemoTransport {
    fn send(&mut self, req: ChatRequest) -> Result<(), ChatError> {
        self.turn += 1;
        let reply = format!(
            "You said: **{}**\n\nThis is a scripted demo reply (turn {}) rendered as \
             `markdown` via the in-crate `MarkdownView`:\n\n- streams one fragment per poll \
             tick\n- no backend required\n",
            req.prompt, self.turn
        );
        self.queue.push_back(StreamEvent::SessionStarted {
            session_id: "demo".into(),
            user_message_id: None,
        });
        for chunk in reply.split_inclusive(' ') {
            self.queue
                .push_back(StreamEvent::TextDelta(chunk.to_string()));
        }
        // Scripted usage so the `show_usage` caption has something to show.
        self.queue.push_back(StreamEvent::Usage(Usage {
            cost_usd: 0.0006 * self.turn as f64,
            input_tokens: 40 * self.turn as u64,
            output_tokens: 65 * self.turn as u64,
            // `Usage` lives in the `ai-chat-core` sibling path dep and gains
            // fields over time (it grew three at once on 2026-08-10). Spreading
            // the default keeps this scripted fixture compiling across those,
            // and leaves the new counters at zero, which is what a non-reasoning
            // uncached turn actually reports.
            ..Default::default()
        }));
        self.queue.push_back(StreamEvent::Done);
        Ok(())
    }
    fn try_recv(&mut self) -> Option<StreamEvent> {
        self.queue.pop_front()
    }
    fn restart(&mut self) -> Result<(), ChatError> {
        self.queue.clear();
        Ok(())
    }
    fn cancel(&mut self) -> Result<(), ChatError> {
        self.queue.clear();
        Ok(())
    }
    fn configure(&mut self, _settings: ChatSettings) {}
}

#[component]
pub fn AiChatDemo() -> impl IntoView {
    let session = StoredValue::new_local(ChatSession::new(Box::new(DemoTransport {
        queue: VecDeque::new(),
        turn: 0,
    })));

    view! {
        <ContentLayout
            title="AiChat"
            description="Reusable chat panel over ai-chat-core (the same shared model the editmark desktop chat uses). This demo drives it with a scripted in-memory transport — type a message and watch the markdown reply stream in."
        >
            <Section title="Live demo" col=true>
                <div class="h-96 w-full max-w-2xl border border-base-300 rounded-box overflow-hidden">
                    <AiChat
                        session=session
                        subtitle="Scripted demo transport"
                        welcome_prompts=vec![
                            "Summarize this document".to_string(),
                            "What changed recently?".to_string(),
                            "Draft a reply".to_string(),
                        ]
                        show_usage=true
                        show_settings=true
                    />
                </div>
                <p class="text-sm opacity-70 mt-2">
                    "Try the welcome chips on the empty transcript, press Enter to send \
                     (Shift+Enter for a newline), hit Stop mid-stream to cancel a turn, \
                     copy any message, or click New session to reset. The cost/token caption \
                     (bottom-right of the composer) updates after each scripted reply. The gear \
                     icon in the header opens a settings popover (model, system prompt, allowed \
                     tools, thinking/tool-call visibility) wired to ChatSession::configure — this \
                     scripted transport ignores the settings, but the form still round-trips \
                     through the session."
                </p>
            </Section>
        </ContentLayout>
    }
}
