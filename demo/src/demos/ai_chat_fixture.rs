//! Browser-proof fixture for the `AiChatWorkspace` composite: one workspace
//! over one seeded in-memory backend, plus an optional fault.
//! Self-contained (no `crate::core` dependency) so the test-host binary can
//! include it without the full showcase chrome.
//!
//! The fault is a plain prop, not a `?fault=` query param: the test host's
//! harness always appends its own `?pp-freeze=1`, and `test_mode::is_enabled`
//! splits on `&` only, so `?fault=x?pp-freeze=1` parses as ONE param named
//! `fault` and test/freeze mode never activates. The test host selects the
//! fault by pathname suffix, matching every other fixture here.
//!
//! The page holds the CONCRETE backend, not just the `Rc<dyn …>` the
//! composite gets, for three reasons no composite can cover itself:
//! `calls()` and the two `applied_*` accessors feed the
//! `window.__APP_DEBUG__` oracle (they are inherent test-mode methods,
//! deliberately not on the trait), and `now_ms()` is the fixture's own clock
//! — which advances per transport poll and never by wall time, so a
//! `js Date.now()` watchdog could never cross its threshold.

use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{
    AiChatWorkspace, ChatWorkspaceBackend, ChatWorkspaceFault, InMemoryChatWorkspaceBackend,
    ProviderTuning, desktop_provider_catalogue,
};
use std::rc::Rc;

/// One applied [`ProviderTuning`] as JSON the oracle can assert on field by
/// field. `Debug` output would round-trip too, but a proof reading
/// `"groq: None"` out of a string asserts on a formatting choice rather than
/// on the payload.
fn tuning_json(t: &ProviderTuning) -> serde_json::Value {
    serde_json::json!({
        "permission_mode": t.permission_mode,
        "reasoning_effort": t.reasoning_effort.as_ref().map(|e| e.as_str().to_owned()),
        "temperature": t.groq.map(|g| g.temperature),
        "codex": t.codex.map(|c| serde_json::json!({
            "web_search": c.web_search,
            "suppress_plugins": c.suppress_plugins,
            "disable_code_mode": c.disable_code_mode,
        })),
    })
}

/// One `AiChatWorkspace` over the seeded fixture backend.
#[component]
pub fn AiChatFixture(
    /// A fault the backend should present, or `None` for the healthy seed.
    #[prop(optional)]
    fault: Option<ChatWorkspaceFault>,
) -> impl IntoView {
    let mut backend = InMemoryChatWorkspaceBackend::seeded();
    if let Some(f) = fault {
        backend = backend.with_fault(f);
    }

    let calls_backend = backend.clone();
    crate::debug::register_signal("ai_chat_calls", move || {
        serde_json::to_value(
            calls_backend
                .calls()
                .iter()
                .map(|c| format!("{c:?}"))
                .collect::<Vec<_>>(),
        )
        .unwrap_or(serde_json::Value::Null)
    });

    let tuning_backend = backend.clone();
    crate::debug::register_signal("ai_chat_applied_tuning", move || {
        let map: serde_json::Map<String, serde_json::Value> = desktop_provider_catalogue()
            .into_iter()
            .map(|card| {
                let value = tuning_backend
                    .applied_tuning(&card.engine.id)
                    .map(|t| tuning_json(&t))
                    .unwrap_or(serde_json::Value::Null);
                (card.engine.id, value)
            })
            .collect();
        serde_json::Value::Object(map)
    });

    let settings_backend = backend.clone();
    crate::debug::register_signal("ai_chat_applied_chat_settings", move || {
        settings_backend
            .applied_chat_settings()
            .map(|s| {
                serde_json::json!({
                    "model": s.model,
                    "system_prompt": s.system_prompt,
                    "allowed_tools": s.allowed_tools,
                    "show_thinking": s.show_thinking,
                    "show_tool_calls": s.show_tool_calls,
                })
            })
            .unwrap_or(serde_json::Value::Null)
    });

    // The fixture clock, republished into a signal the composite polls. It
    // advances per transport poll, so a value read once at mount would sit at
    // the seed forever.
    let clock_backend = backend.clone();
    let clock = RwSignal::new(clock_backend.now_ms());
    if let Ok(handle) = leptos::leptos_dom::helpers::set_interval_with_handle(
        move || {
            let _ = clock.try_set(clock_backend.now_ms());
        },
        std::time::Duration::from_millis(100),
    ) {
        on_cleanup(move || handle.clear());
    }

    let backend: Rc<dyn ChatWorkspaceBackend> = Rc::new(backend);

    view! {
        <div class="flex w-full flex-col gap-8">
            <section id="ai-chat-workspace" data-testid="ai-chat-workspace">
                <AiChatWorkspace backend=backend now_ms=Signal::from(clock) />
            </section>
        </div>
    }
}
