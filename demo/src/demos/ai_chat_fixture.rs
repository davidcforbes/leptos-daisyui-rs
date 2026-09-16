//! Browser-proof fixture for the `AiChatWorkspace` composite: one workspace
//! over one seeded in-memory backend, plus an optional fault, an optional
//! fast clock and an optional scripted failure ladder.
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
//! composite gets, for four reasons no composite can cover itself:
//! `calls()` and the two `applied_*` accessors feed the
//! `window.__APP_DEBUG__` oracle (they are inherent test-mode methods,
//! deliberately not on the trait); `now_ms()` is the fixture's own clock
//! — which advances per transport poll and never by wall time, so a
//! `js Date.now()` watchdog could never cross its threshold; and the
//! per-turn record the page polls for the oracle is a source INDEPENDENT of
//! the DOM the composite renders, so a proof can assert the two agree
//! instead of reading one number twice.
//!
//! Two instances may be mounted on one document (see `case`/`oracle`), which
//! is how a fault and its opposite become negative controls for each other
//! without a second navigation.

use leptos::prelude::*;
use leptos_daisyui_rs::components::ai_assistant_workspace::{AssistantAccess, RefusalNextAction};
use leptos_daisyui_rs::patterns::{
    AiChatWorkspace, AvailabilityReasonCode, ChatPosture, ChatWorkspaceBackend, ChatWorkspaceFault,
    FixtureClock, InMemoryChatWorkspaceBackend, KnowledgeSelection, PromptMatcher, ProviderTuning,
    ReasoningEffort, TurnScript, desktop_provider_catalogue, lifecycle_id, outcome_id,
};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

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

/// The long answer the `cancel` prompt streams.
///
/// Deliberately many words: the fixture releases ONE stream event per
/// transport poll, and `AiChat` polls every 100 ms, so this is about four
/// seconds of streaming — long enough for a proof to press Stop while the
/// turn is genuinely mid-answer rather than racing its completion.
const CANCEL_ANSWER: &str = "This answer is long on purpose so that a proof \
     has time to stop it while it is still arriving one word at a time, \
     which is the only way to observe what a cancel keeps and what a cancel \
     throws away, and a short answer would simply finish first and prove \
     nothing at all about either case.";

/// Register the failure ladder on `claude-code`, keyed by prompt.
///
/// Every shape this phase renders gets one prompt, so a single document can
/// walk from an escalation to a truncation to a decline to a refusal to a
/// crash without a navigation — and so each is the others' negative control.
fn with_failure_scripts(backend: InMemoryChatWorkspaceBackend) -> InMemoryChatWorkspaceBackend {
    let engine = "claude-code";
    backend
        .with_script(
            engine,
            PromptMatcher::Contains("escalate".to_owned()),
            TurnScript::new()
                .escalate(ReasoningEffort::Medium, ReasoningEffort::Low)
                .text_words("The engine quietly dropped to a lower effort before answering."),
        )
        .with_script(
            engine,
            PromptMatcher::Contains("truncate".to_owned()),
            TurnScript::new()
                .text_words("This answer stops before it reaches the last step")
                .truncated(),
        )
        .with_script(
            engine,
            PromptMatcher::Contains("decline".to_owned()),
            TurnScript::new().declined(
                vec![
                    "The court calendar has not been indexed.".to_owned(),
                    "No filing deadline was published for this matter.".to_owned(),
                ],
                "2026-01-01T00:00:00Z",
            ),
        )
        // `fail` ends the turn Unavailable with a TYPED reason. The reason
        // chosen here is the one P1 pinned as "not a credential problem", so
        // the browser can prove the rendering agrees with the model.
        .with_script(
            engine,
            PromptMatcher::Contains("untested".to_owned()),
            TurnScript::new().fail(AvailabilityReasonCode::CliVersionUntested),
        )
        // Its negative control: a reason that IS a credential problem.
        .with_script(
            engine,
            PromptMatcher::Contains("expired".to_owned()),
            TurnScript::new().fail(AvailabilityReasonCode::SignInExpired),
        )
        .with_script(
            engine,
            PromptMatcher::Contains("error".to_owned()),
            TurnScript::new().error("The engine stopped before finishing."),
        )
        .with_script(
            engine,
            PromptMatcher::Contains("stall".to_owned()),
            TurnScript::new().stall(),
        )
        .with_script(
            engine,
            PromptMatcher::Contains("cancel".to_owned()),
            TurnScript::new().text_words(CANCEL_ANSWER),
        )
}

/// One `AiChatWorkspace` over the seeded fixture backend.
#[component]
pub fn AiChatFixture(
    /// A fault the backend should present, or `None` for the healthy seed.
    #[prop(optional)]
    fault: Option<ChatWorkspaceFault>,
    /// A clock to drive the fixture with, or `None` for the default
    /// 100 ms-per-poll one. A fast clock is what turns the 120 000 ms
    /// watchdog from a ~1200-poll proof into a handful of polls.
    #[prop(optional)]
    clock: Option<FixtureClock>,
    /// Whether to register the prompt-keyed failure ladder.
    #[prop(optional)]
    scripted: bool,
    /// A label for this instance, published as `data-ai-chat-case` on its
    /// wrapper so a document carrying two workspaces can address each.
    #[prop(optional, into)]
    case: Option<String>,
    /// Whether this instance owns the document's debug oracle. Exactly one
    /// instance per document may: the oracle keys are global, so a second
    /// writer would overwrite the first and a proof would read whichever
    /// timer happened to fire last.
    #[prop(optional, default = true)]
    oracle: bool,
) -> impl IntoView {
    let mut backend = InMemoryChatWorkspaceBackend::seeded();
    if let Some(c) = clock {
        backend = backend.with_clock(c);
    }
    if let Some(f) = fault {
        backend = backend.with_fault(f);
    }
    if scripted {
        backend = with_failure_scripts(backend);
    }

    if oracle {
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

        // The tier verdict, read straight off `settings()`. Published because
        // it is genuinely INDEPENDENT of the composite: a proof comparing the
        // header's `data-ai-chat-tier-verdict` against this is comparing the
        // rendering against the backend, not against a mirror of itself.
        let tier_backend = backend.clone();
        spawn_local(async move {
            if let Ok(s) = tier_backend.settings().await {
                let denied = matches!(s.reasoning_tier, AssistantAccess::Denied { .. });
                crate::debug_state::set(
                    "ai_chat.tier_verdict",
                    if denied { "denied" } else { "allowed" },
                );
            }
        });
    }

    // The fixture clock, republished into a signal the composite polls. It
    // advances per transport poll, so a value read once at mount would sit at
    // the seed forever. The same beat publishes the live turn's lifecycle and
    // outcome ids into the oracle, from the RECORD rather than from the DOM.
    let clock_backend = backend.clone();
    let clock = RwSignal::new(clock_backend.now_ms());
    // The last turn the oracle read, so a finished turn stays readable.
    let watched_turn: RwSignal<Option<String>> = RwSignal::new(None);
    if let Ok(handle) = leptos::leptos_dom::helpers::set_interval_with_handle(
        move || {
            let _ = clock.try_set(clock_backend.now_ms());
            if !oracle {
                return;
            }
            // The same latch the composite's tick has: a turn stops being
            // "in flight" in the very call that records its terminal
            // lifecycle, so an oracle keyed on the live id alone freezes at
            // `validating` and never publishes an outcome.
            let Some(id) = leptos_daisyui_rs::patterns::tick_turn_id(
                clock_backend.current_turn_id(),
                watched_turn.get_untracked().as_deref(),
            ) else {
                return;
            };
            let _ = watched_turn.try_set(Some(id.clone()));
            let fut = clock_backend.turn(&id);
            spawn_local(async move {
                if let Ok(record) = fut.await {
                    crate::debug_state::set("ai_chat.turn_status", lifecycle_id(&record.lifecycle));
                    crate::debug_state::set("ai_chat.outcome", outcome_id(&record));
                }
            });
        },
        std::time::Duration::from_millis(100),
    ) {
        on_cleanup(move || handle.clear());
    }

    let on_refusal_action = Callback::new(move |action: RefusalNextAction| {
        if oracle {
            crate::debug_state::set("ai_chat.refusal_action", action.as_str());
        }
    });

    let backend: Rc<dyn ChatWorkspaceBackend> = Rc::new(backend);

    // A scripted document opens in ASSISTANT posture, and this is not a
    // convenience: `KnowledgeSelection::default()` is GROUNDED, and a
    // grounded turn whose prompt matches no seeded document has its script
    // REPLACED by the localized not-found sentence (`memory.rs`'s `send`),
    // whatever was registered for that prompt. Every failure shape this
    // phase renders would silently become "I could not find that in this
    // folder" — which is the fixture behaving correctly and the proof
    // measuring nothing. The base document keeps the grounded default, so
    // P4's grounded assertions are untouched.
    let knowledge = KnowledgeSelection {
        posture: if scripted {
            ChatPosture::Assistant
        } else {
            KnowledgeSelection::default().posture
        },
        ..KnowledgeSelection::default()
    };

    view! {
        <div class="flex w-full flex-col gap-8">
            <section
                id="ai-chat-workspace"
                data-testid="ai-chat-workspace"
                data-ai-chat-case=case
            >
                <AiChatWorkspace
                    backend=backend
                    now_ms=Signal::from(clock)
                    initial_knowledge=knowledge
                    on_refusal_action=on_refusal_action
                />
            </section>
        </div>
    }
}
