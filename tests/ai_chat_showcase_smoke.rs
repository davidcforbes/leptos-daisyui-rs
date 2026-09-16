//! Real-browser proof for `patterns::AiChatWorkspace` (one workspace, the
//! seeded in-memory backend, on one document).
//!
//! Every assertion here carries its negative control on the SAME document:
//! the workspace switches engines in place, so "codex-cli shows a model
//! select" is only meaningful beside "claude-code shows a model input on the
//! very next read".
//!
//! Two structural notes for whoever extends this in P5-P7:
//!
//! * The backend's call log, the applied `ProviderTuning` per engine and the
//!   applied `ChatSettings` all arrive through `window.__APP_DEBUG__.state()`
//!   (`demo/src/demos/ai_chat_fixture.rs` registers them). They are inherent
//!   test-mode methods on the concrete fixture, deliberately NOT on
//!   `ChatWorkspaceBackend`, so the page is the only place that can publish
//!   them.
//! * `BackendCall::Turn(id)` carries a TURN ID and is logged on every record
//!   lookup, which the composite does on a timer. It is therefore useless as
//!   a "how many turns were sent" counter; the count of DISTINCT ids in the
//!   log is the number of turns that ever existed, and that is what the send
//!   assertions below use.

mod common;

use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

/// Press Shift+Enter so the BROWSER inserts the newline itself.
///
/// `common::shift_enter` dispatches `RawKeyDown`, which by definition
/// generates no character event, so a textarea receiving it sees the keydown
/// and inserts nothing — fine for a handler that intercepts the key, useless
/// for proving the composer's default behaviour survives. This sends a real
/// `KeyDown` carrying `text`, which is what actually types a line break.
async fn shift_enter_newline(h: &pixelproof_web::Harness) {
    let params = [
        DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::RawKeyDown)
            .key("Shift")
            .code("ShiftLeft")
            .windows_virtual_key_code(16)
            .native_virtual_key_code(16)
            .modifiers(8)
            .build()
            .expect("Shift key-down params"),
        DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::KeyDown)
            .key("Enter")
            .code("Enter")
            .text("\r")
            .windows_virtual_key_code(13)
            .native_virtual_key_code(13)
            .modifiers(8)
            .build()
            .expect("Shift+Enter key-down params"),
        DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::KeyUp)
            .key("Enter")
            .code("Enter")
            .windows_virtual_key_code(13)
            .native_virtual_key_code(13)
            .modifiers(8)
            .build()
            .expect("Shift+Enter key-up params"),
        DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::KeyUp)
            .key("Shift")
            .code("ShiftLeft")
            .windows_virtual_key_code(16)
            .native_virtual_key_code(16)
            .build()
            .expect("Shift key-up params"),
    ];
    for p in params {
        h.page().execute(p).await.expect("dispatch Shift+Enter");
    }
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

const PAGE: &str = "/ai-chat-fixture";
const ROOT: &str = "[data-ai-chat-workspace]";

const CLAUDE: &str = "claude-code";
const CODEX: &str = "codex-cli";
const SPARK: &str = "codex-spark";
const GROQ: &str = "groq-gpt-oss-120b";
const OLLAMA: &str = "ollama";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate ai-chat fixture")
        .into_value()
        .expect("ai-chat expression returns JSON")
}

/// The whole `window.__APP_DEBUG__.state()` map.
async fn oracle_state(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const raw = window.__APP_DEBUG__ ? window.__APP_DEBUG__.state() : '{}';
            return JSON.parse(raw);
        })()"#,
    )
    .await
}

/// The backend's ordered call log, as `Debug` strings.
async fn backend_calls(h: &pixelproof_web::Harness) -> Vec<String> {
    oracle_state(h).await["ai_chat_calls"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// The `ProviderTuning` the backend actually holds for one engine, or
/// `Value::Null` when that engine was never tuned or opened.
async fn applied_tuning(h: &pixelproof_web::Harness, engine: &str) -> Value {
    oracle_state(h).await["ai_chat_applied_tuning"][engine].clone()
}

/// The `ChatSettings` the transport's `configure` last received, or
/// `Value::Null` before the first Apply.
async fn applied_chat_settings(h: &pixelproof_web::Harness) -> Value {
    oracle_state(h).await["ai_chat_applied_chat_settings"].clone()
}

/// How many DISTINCT turn ids the log has ever mentioned — see the module
/// note on why a raw `Turn` count cannot be used.
fn distinct_turn_ids(calls: &[String]) -> Vec<String> {
    let mut ids: Vec<String> = calls
        .iter()
        .filter_map(|c| {
            let rest = c.strip_prefix("Turn(")?;
            let inner = rest.strip_suffix(')')?;
            Some(inner.trim_matches('"').to_owned())
        })
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// The settings-popover shape the workspace currently renders.
async fn settings_shape(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{ROOT}');
                const q = sel => root.querySelector(sel);
                const modelSelect = q('[data-ai-chat-model-select]');
                const permission = q('[data-ai-chat-permission-select]');
                const thinking = q('[data-ai-chat-show-thinking]');
                const tools = q('[data-ai-chat-show-tool-calls]');
                const keyish = Array.from(root.querySelectorAll('input')).filter(i =>
                    i.type === 'password'
                    || /key/i.test(i.name || '')
                    || /key/i.test(i.id || '')
                    || /key/i.test(i.placeholder || ''));
                return {{
                    engine: root.getAttribute('data-ai-chat-workspace-engine'),
                    locale: root.getAttribute('data-ai-chat-workspace-locale'),
                    turnStatus: q('[data-ai-chat-turn-status]')
                        ?.getAttribute('data-ai-chat-turn-status') ?? null,
                    backendOptions: Array.from(
                        q('[data-ai-chat-backend]')?.options ?? []
                    ).map(o => o.value),
                    backendValue: q('[data-ai-chat-backend]')?.value ?? null,
                    modelSelect: modelSelect !== null,
                    modelSelectOptions: modelSelect
                        ? Array.from(modelSelect.options).map(o => o.value)
                        : null,
                    modelInput: q('[data-ai-chat-model-input]') !== null,
                    modelFixed: q('[data-ai-chat-model-fixed]') !== null,
                    modelFixedValue: q('[data-ai-chat-model-fixed]')?.value ?? null,
                    permission: permission !== null,
                    permissionOptionCount: permission ? permission.options.length : 0,
                    permissionValue: permission ? permission.value : null,
                    credentialNote: q('[data-ai-chat-credential-note]') !== null,
                    keyLikeInputs: keyish.length,
                    effortSelect: q('[data-ai-chat-effort-select]') !== null,
                    effortOptions: q('[data-ai-chat-effort-select]')
                        ? Array.from(q('[data-ai-chat-effort-select]').options)
                            .map(o => o.value)
                        : null,
                    effortValue: q('[data-ai-chat-effort-select]')?.value ?? null,
                    effortSelectedText: q('[data-ai-chat-effort-select]')
                        ? (q('[data-ai-chat-effort-select]').selectedOptions[0]
                            ?.textContent?.trim() ?? null)
                        : null,
                    railLabels: Array.from(
                        root.querySelectorAll('[data-ai-chat-rail-label]')
                    ).map(el => ({{
                        control: el.getAttribute('data-ai-chat-rail-label'),
                        label: el.textContent.trim(),
                        options: Array.from(
                            el.closest('label')?.querySelector('select')?.options ?? []
                        ).map(o => o.textContent.trim()),
                    }})),
                    temperature: q('[data-ai-chat-temperature]') !== null,
                    temperatureValue: q('[data-ai-chat-temperature]')?.value ?? null,
                    codexLevers: Array.from(
                        root.querySelectorAll('[data-ai-chat-codex-lever]')
                    ).map(e => e.getAttribute('data-ai-chat-codex-lever')),
                    noMcpNote: q('[data-ai-chat-no-mcp]') !== null,
                    thinkingDisabled: thinking ? thinking.disabled : null,
                    thinkingTitle: thinking ? (thinking.getAttribute('title') ?? null) : null,
                    thinkingChecked: thinking ? thinking.checked : null,
                    toolsDisabled: tools ? tools.disabled : null,
                    toolsTitle: tools ? (tools.getAttribute('title') ?? null) : null,
                    toolsChecked: tools ? tools.checked : null,
                    roles: Array.from(root.querySelectorAll('[data-chat-role]'))
                        .map(e => e.getAttribute('data-chat-role')),
                    texts: Array.from(root.querySelectorAll('[data-chat-role]'))
                        .map(e => e.textContent.trim()),
                    chatState: q('[data-ai-chat-state]')
                        ?.getAttribute('data-ai-chat-state') ?? null,
                    composer: q('[data-ai-chat-composer]')?.value ?? null,
                }};
            }})()"#
        ),
    )
    .await
}

/// Switch the workspace's engine through its real `<select>`, the way P2's
/// own picker reports a change.
async fn select_engine(h: &pixelproof_web::Harness, engine: &str) {
    let changed: bool = eval_json(
        h,
        &format!(
            r#"(() => {{
                const sel = document.querySelector('{ROOT} [data-ai-chat-backend]');
                if (!sel) return false;
                sel.value = '{engine}';
                sel.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return sel.value === '{engine}';
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(changed, "the engine picker must accept {engine}");
    wait_for_selector(
        h,
        &format!("{ROOT}[data-ai-chat-workspace-engine=\"{engine}\"]"),
    )
    .await;
    // The panel REMOUNTS on a switch, so give the new tree one settle beat
    // before anything reads it.
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

/// Wait until the workspace has opened its first session and mounted the
/// panel. Everything else in this file depends on it.
async fn ready(h: &pixelproof_web::Harness) {
    wait_for_selector(h, &format!("{ROOT} [data-ai-chat-panel]")).await;
    wait_for_selector(h, &format!("{ROOT} [data-ai-chat-settings]")).await;
}

/// Put text in the composer through a real input event, then hand focus back
/// to it so a following key press lands there.
async fn type_prompt(h: &pixelproof_web::Harness, text: &str) {
    let ok: bool = eval_json(
        h,
        &format!(
            r#"(() => {{
                const c = document.querySelector('{ROOT} [data-ai-chat-composer]');
                if (!c) return false;
                c.focus();
                c.value = {text:?};
                c.dispatchEvent(new InputEvent('input', {{
                    bubbles: true, inputType: 'insertText'
                }}));
                return document.activeElement === c;
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(ok, "the composer must accept text and keep focus");
}

/// Poll until the panel is idle again (its `data-ai-chat-state` leaves
/// `waiting`/`streaming`), or give up after `budget_ms`.
async fn wait_for_idle(h: &pixelproof_web::Harness, budget_ms: u64) {
    let step = 200;
    let mut waited = 0;
    loop {
        let state = settings_shape(h).await["chatState"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if state == "idle" {
            return;
        }
        assert!(
            waited < budget_ms,
            "the panel never returned to idle (last state {state:?})"
        );
        tokio::time::sleep(std::time::Duration::from_millis(step)).await;
        waited += step;
    }
}

// ── 1 ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn capabilities_drive_the_settings_form_shape() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;

    select_engine(&h, CODEX).await;
    let codex = settings_shape(&h).await;
    assert_eq!(
        codex["modelSelectOptions"].as_array().map(Vec::len),
        Some(7),
        "codex-cli publishes seven models: {codex}"
    );
    assert_eq!(codex["modelInput"], json!(false), "{codex}");
    assert_eq!(codex["modelFixed"], json!(false), "{codex}");

    select_engine(&h, CLAUDE).await;
    let claude = settings_shape(&h).await;
    assert_eq!(
        claude["modelInput"],
        json!(true),
        "an empty model list is free text: {claude}"
    );
    assert_eq!(claude["modelSelect"], json!(false), "{claude}");
    assert_eq!(claude["modelFixed"], json!(false), "{claude}");

    select_engine(&h, SPARK).await;
    let spark = settings_shape(&h).await;
    assert_eq!(
        spark["modelFixed"],
        json!(true),
        "a pinned model is shown, never offered: {spark}"
    );
    assert_eq!(
        spark["modelFixedValue"],
        json!("gpt-5.3-codex-spark"),
        "{spark}"
    );
    assert_eq!(spark["modelSelect"], json!(false), "{spark}");
    assert_eq!(spark["modelInput"], json!(false), "{spark}");

    // I2: an unchosen reasoning effort must select the leading engine-default
    // option. With three levels and nothing selected the browser shows its
    // FIRST option — "low" — while `to_tuning` sends `None` and the engine
    // runs at its own default, which is the `permission_mode` defect one row
    // away.
    let spark = settings_shape(&h).await;
    assert_eq!(
        spark["effortSelect"],
        json!(true),
        "codex-spark declares reasoning effort: {spark}"
    );
    assert_eq!(
        spark["effortOptions"],
        json!(["", "low", "medium", "high"]),
        "the leading option carries an empty value: {spark}"
    );
    assert_eq!(
        spark["effortValue"],
        json!(""),
        "nothing is chosen, so the engine-default option is selected — NOT \
         `low`: {spark}"
    );
    assert_eq!(
        spark["effortSelectedText"],
        json!("Engine default"),
        "and what the actor reads says so: {spark}"
    );
    assert_eq!(
        applied_tuning(&h, SPARK).await["reasoning_effort"],
        json!(null),
        "the payload agrees with the control: {spark}"
    );

    // Ollama declares no reasoning effort at all — the row's negative control.
    select_engine(&h, OLLAMA).await;
    let ollama = settings_shape(&h).await;
    assert_eq!(ollama["effortSelect"], json!(false), "{ollama}");

    assert_no_browser_errors(&h, "settings form shape").await;
}

// ── 9 ───────────────────────────────────────────────────────────────────────

/// The knowledge rail's controls name what they DECIDE, and changing one does
/// not announce a provider switch that never happened.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn knowledge_rail_labels_its_controls_and_never_announces_a_switch() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;

    // I4: every rail control's visible label must be a FIELD label, not a
    // repeat of one of its own options. A select whose accessible name is
    // "Fused" tells a screen-reader user what is currently chosen and nothing
    // about what the control decides.
    let shape = settings_shape(&h).await;
    let labels = shape["railLabels"].as_array().cloned().unwrap_or_default();
    assert_eq!(
        labels.len(),
        3,
        "corpus scope, query mode and posture each carry a label: {shape}"
    );
    for row in &labels {
        let control = row["control"].as_str().unwrap_or_default();
        let label = row["label"].as_str().unwrap_or_default();
        let options: Vec<String> = row["options"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap_or_default().to_owned())
            .collect();
        assert!(!label.is_empty(), "{control} has no label: {row}");
        assert!(
            !options.is_empty(),
            "{control}'s options must be readable, or the check below is vacuous: {row}"
        );
        assert!(
            !options.iter().any(|o| o == label),
            "{control} is labelled {label:?}, which is one of its own options \
             {options:?}"
        );
    }

    // I1: a knowledge change rebuilds the session (a turn is opened against a
    // knowledge mix) but does NOT change the engine, so it must not announce
    // "Switched to {engine}". Pre-fix, `open_engine` read "a session already
    // exists" as "this is a switch" and announced on every rail change.
    let engine_before = shape["engine"].as_str().unwrap_or_default().to_owned();
    let changed: bool = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const sel = document.querySelector('{ROOT} [data-ai-chat-posture]');
                if (!sel) return false;
                sel.value = 'assistant';
                sel.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return sel.value === 'assistant';
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(changed, "the posture select must accept a value");
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    let after = settings_shape(&h).await;
    assert_eq!(
        after["engine"].as_str(),
        Some(engine_before.as_str()),
        "a posture change does not change the engine: {after}"
    );
    assert!(
        after["roles"].as_array().is_some_and(|r| r.is_empty()),
        "and it announces NOTHING — the transcript stays empty rather than \
         claiming a switch: {after}"
    );
    let calls = backend_calls(&h).await;
    assert!(
        calls
            .iter()
            .any(|c| c.starts_with("OpenSession") && c.contains("posture: Assistant")),
        "the new posture did reach the backend, so the silence above is not \
         simply a change that never happened: {calls:?}"
    );

    // The positive control on the same document: an ENGINE switch still
    // announces.
    select_engine(&h, SPARK).await;
    let switched = settings_shape(&h).await;
    let roles: Vec<String> = switched["roles"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(roles, vec!["notice".to_owned()], "{switched}");

    assert_no_browser_errors(&h, "knowledge rail").await;
}

// ── 2 ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn permission_modes_render_only_for_providers_that_declare_them() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;

    select_engine(&h, CLAUDE).await;
    let claude = settings_shape(&h).await;
    assert_eq!(claude["permission"], json!(true), "{claude}");
    assert_eq!(
        claude["permissionOptionCount"],
        json!(4),
        "claude-code publishes exactly four modes and no placeholder, because \
         the composite SEEDS the control: {claude}"
    );
    assert_eq!(
        claude["permissionValue"],
        json!("default"),
        "the seed prefers the engine's own `default` mode over its first: {claude}"
    );

    select_engine(&h, CODEX).await;
    let codex = settings_shape(&h).await;
    assert_eq!(
        codex["permission"],
        json!(false),
        "codex-cli publishes no permission modes, so the row must not render \
         at all: {codex}"
    );

    assert_no_browser_errors(&h, "permission modes").await;
}

// ── 3 ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn needs_api_key_renders_credential_note_not_an_input() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;

    select_engine(&h, GROQ).await;
    let groq = settings_shape(&h).await;
    assert_eq!(groq["credentialNote"], json!(true), "{groq}");
    assert_eq!(
        groq["keyLikeInputs"],
        json!(0),
        "a key must never be typeable here — no password field and nothing \
         whose name/id/placeholder reads as a key: {groq}"
    );

    select_engine(&h, CLAUDE).await;
    let claude = settings_shape(&h).await;
    assert_eq!(
        claude["credentialNote"],
        json!(false),
        "an engine that needs no key shows no note: {claude}"
    );
    assert_eq!(claude["keyLikeInputs"], json!(0), "{claude}");

    assert_no_browser_errors(&h, "credential note").await;
}

// ── 4 ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn thinking_and_tool_toggles_follow_supports_flags_and_reach_the_transport() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;

    // Ollama is the catalogue's only card that supports NEITHER, so it is the
    // negative control for both toggles at once.
    select_engine(&h, OLLAMA).await;
    let ollama = settings_shape(&h).await;
    assert_eq!(ollama["toolsDisabled"], json!(true), "{ollama}");
    assert!(
        ollama["toolsTitle"].as_str().is_some_and(|t| !t.is_empty()),
        "a disabled toggle's title is the only explanation the user gets: {ollama}"
    );
    assert_eq!(ollama["thinkingDisabled"], json!(true), "{ollama}");
    assert_eq!(
        ollama["thinkingChecked"],
        json!(false),
        "a session opened against an engine that cannot emit thinking must not \
         ask for it: {ollama}"
    );

    // Claude supports both, so its toggles are live and can reach the
    // transport. `applied_chat_settings` is `ChatTransport::configure`'s own
    // record — the payload, not just that a call happened.
    select_engine(&h, CLAUDE).await;
    let before = settings_shape(&h).await;
    assert_eq!(before["thinkingDisabled"], json!(false), "{before}");
    assert_eq!(before["toolsDisabled"], json!(false), "{before}");
    assert_eq!(before["thinkingChecked"], json!(true), "{before}");

    click(&h, &format!("{ROOT} [data-ai-chat-settings]")).await;
    click(&h, &format!("{ROOT} [data-ai-chat-show-thinking]")).await;
    let flipped = settings_shape(&h).await;
    assert_eq!(
        flipped["thinkingChecked"],
        json!(false),
        "the toggle itself flipped: {flipped}"
    );
    click(&h, &format!("{ROOT} [data-ai-chat-apply]")).await;

    let settings = applied_chat_settings(&h).await;
    assert_eq!(
        settings["show_thinking"],
        json!(false),
        "the flag the transport RECEIVED, not merely that a call was logged: {settings}"
    );
    assert_eq!(
        settings["show_tool_calls"],
        json!(true),
        "the untouched flag is the same payload's negative control: {settings}"
    );

    assert_no_browser_errors(&h, "toggles").await;
}

// ── 5 ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn per_provider_levers_configure_only_their_own_provider() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;

    select_engine(&h, GROQ).await;
    let groq = settings_shape(&h).await;
    assert_eq!(groq["temperature"], json!(true), "{groq}");
    assert_eq!(
        groq["codexLevers"].as_array().map(Vec::len),
        Some(0),
        "a Groq card must not grow a Codex lever: {groq}"
    );

    // Move the slider through a real input event and let the composite push
    // the tuning at the backend.
    let moved: bool = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const r = document.querySelector('{ROOT} [data-ai-chat-temperature]');
                if (!r) return false;
                r.value = '1.25';
                r.dispatchEvent(new Event('input', {{ bubbles: true }}));
                return r.value === '1.25';
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(moved, "the temperature slider must accept a value");
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let groq_tuning = applied_tuning(&h, GROQ).await;
    assert_eq!(
        groq_tuning["temperature"]
            .as_f64()
            .map(|v| (v * 100.0).round()),
        Some(125.0),
        "the payload the backend holds for groq: {groq_tuning}"
    );
    assert_eq!(groq_tuning["codex"], json!(null), "{groq_tuning}");

    select_engine(&h, CODEX).await;
    let codex = settings_shape(&h).await;
    assert_eq!(
        codex["codexLevers"],
        json!(["web_search", "suppress_plugins", "disable_code_mode"]),
        "{codex}"
    );
    assert_eq!(
        codex["temperature"],
        json!(false),
        "a Codex card must not grow a temperature: {codex}"
    );
    assert_eq!(
        codex["noMcpNote"],
        json!(true),
        "no MCP is an immutable fact about this engine, rendered as text, not \
         as a control: {codex}"
    );

    let codex_tuning = applied_tuning(&h, CODEX).await;
    assert_eq!(
        codex_tuning["temperature"],
        json!(null),
        "switching engines must not smuggle groq's temperature into codex's \
         payload: {codex_tuning}"
    );
    assert!(
        !codex_tuning["codex"].is_null(),
        "codex's own levers ARE in its payload: {codex_tuning}"
    );
    // And groq's own payload is untouched by the switch.
    assert_eq!(
        applied_tuning(&h, GROQ).await["temperature"]
            .as_f64()
            .map(|v| (v * 100.0).round()),
        Some(125.0)
    );

    assert_no_browser_errors(&h, "per-provider levers").await;
}

// ── 6 ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn provider_switch_is_announced_in_the_transcript_and_sent_to_the_transport() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;

    select_engine(&h, CLAUDE).await;
    type_prompt(&h, "the intake checklist conflict check").await;
    click(&h, &format!("{ROOT} [data-ai-chat-send]")).await;
    wait_for_idle(&h, 30_000).await;
    let answered = settings_shape(&h).await;
    let before_roles = answered["roles"].as_array().cloned().unwrap_or_default();
    assert!(
        before_roles.iter().any(|r| r == "user"),
        "the first engine really did hold a conversation: {answered}"
    );

    select_engine(&h, SPARK).await;
    let after = settings_shape(&h).await;
    let roles: Vec<String> = after["roles"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(
        roles,
        vec!["notice".to_owned()],
        "a switch resets the conversation, leaving exactly the announcement: {after}"
    );
    let texts: Vec<String> = after["texts"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_owned())
        .collect();
    assert!(
        texts[0].contains("Codex Spark"),
        "the notice names the engine that is now running: {texts:?}"
    );

    let calls = backend_calls(&h).await;
    assert!(
        calls
            .iter()
            .any(|c| c.starts_with("OpenSession") && c.contains(SPARK)),
        "the switch reached the backend as an OpenSession for the new engine: {calls:?}"
    );

    assert_no_browser_errors(&h, "provider switch").await;
}

// ── 7 ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn send_streams_thinking_tool_call_result_and_text_in_order() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;
    // NO engine switch here. `initial_engine` is `None`, so the composite
    // opens the first card `card_ready_for_ask` accepts — claude-code, the
    // one whose script streams thinking and a tool pair. Re-selecting it
    // would announce a switch and put a notice row at the top of the
    // transcript, which is exactly the ordering this test is about.
    let boot = settings_shape(&h).await;
    assert_eq!(
        boot["engine"],
        json!(CLAUDE),
        "the first ready card is the one that boots: {boot}"
    );
    assert!(
        boot["roles"].as_array().is_some_and(|r| r.is_empty()),
        "a FIRST open announces nothing — there was no conversation to reset: {boot}"
    );

    let before = distinct_turn_ids(&backend_calls(&h).await);
    assert!(
        before.is_empty(),
        "no turn before the first send: {before:?}"
    );

    // A prompt that shares two significant words with the seeded intake
    // checklist, so the grounded posture resolves the engine's own script
    // rather than the not-found sentence.
    type_prompt(&h, "the intake checklist conflict check").await;
    click(&h, &format!("{ROOT} [data-ai-chat-send]")).await;
    wait_for_idle(&h, 30_000).await;

    let shape = settings_shape(&h).await;
    let roles: Vec<String> = shape["roles"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_owned())
        .collect();
    // `ai_chat_core` lands a ToolCall and a ToolResult as two messages of the
    // SAME `ChatRole::Tool`, so the ordering claim is about the role sequence
    // plus each tool row's own text — not about two different hooks.
    assert_eq!(
        roles,
        vec![
            "user".to_owned(),
            "thinking".to_owned(),
            "tool".to_owned(),
            "tool".to_owned(),
            "assistant".to_owned(),
        ],
        "thinking, then the call, then the result, then the answer: {shape}"
    );
    let texts: Vec<String> = shape["texts"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_owned())
        .collect();
    assert!(texts[2].contains("Read"), "the tool CALL row: {texts:?}");
    assert!(
        texts[3].contains("conflict check"),
        "the tool RESULT row: {texts:?}"
    );
    assert!(
        texts[4].contains("conflict check"),
        "the assistant bubble: {texts:?}"
    );

    let ids = distinct_turn_ids(&backend_calls(&h).await);
    assert_eq!(ids.len(), 1, "exactly one turn ever existed: {ids:?}");

    assert_no_browser_errors(&h, "streamed turn").await;
}

// ── 8 ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn shift_enter_inserts_a_newline_and_does_not_send() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    ready(&h).await;
    // The boot engine, for the same reason as the streaming test: a switch
    // would leave a notice row and the transcript would not start empty.
    type_prompt(&h, "first line").await;
    shift_enter_newline(&h).await;
    let after_shift = settings_shape(&h).await;
    assert!(
        after_shift["composer"]
            .as_str()
            .is_some_and(|v| v.contains('\n')),
        "Shift+Enter inserts a newline: {after_shift}"
    );
    assert_eq!(
        distinct_turn_ids(&backend_calls(&h).await).len(),
        0,
        "and sends nothing"
    );
    let roles = after_shift["roles"].as_array().cloned().unwrap_or_default();
    assert!(
        roles.is_empty(),
        "the transcript is untouched: {after_shift}"
    );

    // The negative control on the same composer: plain Enter DOES send.
    common::press_enter(&h).await;
    wait_for_idle(&h, 30_000).await;
    let sent = settings_shape(&h).await;
    let roles: Vec<String> = sent["roles"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_owned())
        .collect();
    assert!(roles.contains(&"user".to_owned()), "{sent}");
    assert_eq!(
        distinct_turn_ids(&backend_calls(&h).await).len(),
        1,
        "exactly one turn, from the plain Enter"
    );
    assert_eq!(
        sent["composer"],
        json!(""),
        "a sent composer is cleared: {sent}"
    );

    assert_no_browser_errors(&h, "shift enter").await;
}

// ── P5: honesty, failure shapes, refusals, lifecycle, usage ─────────────────
//
// Six further fixture documents, each selected by PATHNAME SUFFIX (never a
// `?query` — the harness appends its own `?pp-freeze=1`). Three of them mount
// TWO workspaces, because one fixture backend carries one fault and the pairs
// below only mean anything beside each other: a runtime that is not running
// against a model that is not installed, a cancel that keeps against a cancel
// that discards, and a fixture clock that advances against one that does not.

const PAGE_FAILURES: &str = "/ai-chat-fixture-failures";
const PAGE_GROQ_NO_KEY: &str = "/ai-chat-fixture-groq-no-key";
const PAGE_TIER_DISABLED: &str = "/ai-chat-fixture-tier-disabled";
const PAGE_OLLAMA: &str = "/ai-chat-fixture-ollama-unreachable";
const PAGE_WATCHDOG: &str = "/ai-chat-fixture-watchdog";
const PAGE_BUDGET: &str = "/ai-chat-fixture-budget-exhausted";
const PAGE_CANCEL: &str = "/ai-chat-fixture-cancel";

/// The root selector for one workspace on a two-workspace document.
fn case_root(case: &str) -> String {
    format!("[data-ai-chat-case=\"{case}\"] [data-ai-chat-workspace]")
}

/// The generic per-page debug map (`debug_state::set` writes), which is where
/// the fixture PAGE publishes what it read off the backend — a source
/// independent of the DOM the composite rendered.
async fn debug_state(h: &pixelproof_web::Harness) -> Value {
    oracle_state(h).await["state"].clone()
}

/// Everything this phase renders, read off ONE workspace root.
///
/// Every value is a stable `data-*` hook; nothing here is selected by
/// document position, so a layout change cannot silently repoint an
/// assertion at a different element.
async fn workspace_shape(h: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{root}');
                if (!root) return null;
                const q = sel => root.querySelector(sel);
                const header = q('[data-ai-chat-workspace-header]');
                const attr = (el, name) => el ? el.getAttribute(name) : null;
                const refusal = q('[data-ai-chat-refusal]');
                const honestyText = q('[data-ai-chat-honesty-text]');
                const num = sel => {{
                    const el = q('[' + sel + ']');
                    return el ? Number(el.getAttribute(sel)) : null;
                }};
                return {{
                    engine: root.getAttribute('data-ai-chat-workspace-engine'),
                    honesty: attr(header, 'data-ai-chat-honesty'),
                    honestyReason: attr(header, 'data-ai-chat-honesty-reason'),
                    honestyClass: honestyText ? honestyText.className : null,
                    honestyText: honestyText ? honestyText.textContent.trim() : null,
                    tierVerdict: attr(header, 'data-ai-chat-tier-verdict'),
                    failure: attr(header, 'data-ai-chat-failure'),
                    outcome: attr(header, 'data-ai-chat-outcome'),
                    cancelDiscarded: attr(header, 'data-ai-chat-cancel-discarded'),
                    turnStatus: attr(header, 'data-ai-chat-turn-status'),
                    refusalKind: attr(refusal, 'data-ai-chat-refusal'),
                    refusalReason: attr(refusal, 'data-ai-chat-refusal-reason'),
                    refusalText: refusal ? refusal.textContent.trim() : null,
                    refusalActions: Array.from(
                        root.querySelectorAll('[data-ai-chat-refusal-action]')
                    ).map(e => e.getAttribute('data-ai-chat-refusal-action')),
                    usagePlan: num('data-ai-chat-usage-plan'),
                    usageMetered: num('data-ai-chat-usage-metered'),
                    usageReasoning: num('data-ai-chat-usage-reasoning'),
                    usageOutput: num('data-ai-chat-usage-output'),
                    usageTps: q('[data-ai-chat-usage-tps]')
                        ?.getAttribute('data-ai-chat-usage-tps') ?? null,
                    usageCost: q('[data-ai-chat-usage-cost]')?.textContent.trim() ?? null,
                    usageBudget: q('[data-ai-chat-usage-budget]')?.textContent.trim() ?? null,
                    limitations: Array.from(
                        root.querySelectorAll('[data-ai-chat-limitation]')
                    ).map(e => e.textContent.trim()),
                    availability: Array.from(
                        root.querySelectorAll('[data-ai-chat-engine-reason]')
                    ).map(e => ({{
                        engine: e.getAttribute('data-ai-chat-engine-reason-for'),
                        reason: e.getAttribute('data-ai-chat-engine-reason'),
                        text: e.textContent.trim(),
                    }})),
                    escalation: q('[data-ai-chat-escalation]') !== null,
                    truncated: q('[data-ai-chat-truncated]') !== null,
                    canceledPartial: q('[data-ai-chat-canceled-partial]')
                        ?.textContent.trim() ?? null,
                    noticeRows: Array.from(
                        root.querySelectorAll('[data-chat-role="notice"]')
                    ).map(e => e.textContent.trim()),
                    roles: Array.from(root.querySelectorAll('[data-chat-role]'))
                        .map(e => e.getAttribute('data-chat-role')),
                    texts: Array.from(root.querySelectorAll('[data-chat-role]'))
                        .map(e => e.textContent.trim()),
                    panel: q('[data-ai-chat-panel]') !== null,
                    errorStrip: q('[data-ai-chat-error]') !== null,
                    retryButton: q('[data-ai-chat-retry]') !== null,
                    chatState: q('[data-ai-chat-state]')
                        ?.getAttribute('data-ai-chat-state') ?? null,
                    composer: q('[data-ai-chat-composer]')?.value ?? null,
                }};
            }})()"#
        ),
    )
    .await
}

/// Wait until one workspace root has mounted its panel.
async fn ready_at(h: &pixelproof_web::Harness, root: &str) {
    wait_for_selector(h, &format!("{root} [data-ai-chat-panel]")).await;
    wait_for_selector(h, &format!("{root} [data-ai-chat-settings]")).await;
}

/// Put text in one workspace's composer through a real input event.
async fn type_prompt_at(h: &pixelproof_web::Harness, root: &str, text: &str) {
    let ok: bool = eval_json(
        h,
        &format!(
            r#"(() => {{
                const c = document.querySelector('{root} [data-ai-chat-composer]');
                if (!c) return false;
                c.focus();
                c.value = {text:?};
                c.dispatchEvent(new InputEvent('input', {{
                    bubbles: true, inputType: 'insertText'
                }}));
                return document.activeElement === c;
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(ok, "the composer must accept text and keep focus");
}

/// Send one prompt through the composer, wait for the panel to go idle, and
/// then wait for the turn's RECORD to catch up.
///
/// Two clocks, deliberately: `AiChat` drains the stream on its own 100 ms
/// poll, while the workspace reads the turn record on a separate 200 ms one.
/// The panel therefore reaches `idle` up to a beat BEFORE the header's
/// lifecycle and outcome hooks do, and reading them in that window sees
/// `validating` — a real race in the proof, not in the product. This waits
/// for the record; the assertions still check its exact value.
async fn ask(h: &pixelproof_web::Harness, root: &str, prompt: &str) {
    type_prompt_at(h, root, prompt).await;
    click(h, &format!("{root} [data-ai-chat-send]")).await;
    wait_for_idle_at(h, root, 30_000).await;
    let mut waited = 0;
    while waited < 5_000 {
        if !workspace_shape(h, root).await["outcome"].is_null() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        waited += 200;
    }
}

/// Poll until one workspace's panel leaves `waiting`/`streaming`.
async fn wait_for_idle_at(h: &pixelproof_web::Harness, root: &str, budget_ms: u64) {
    let step = 200;
    let mut waited = 0;
    loop {
        let state = workspace_shape(h, root).await["chatState"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if state == "idle" {
            return;
        }
        assert!(
            waited < budget_ms,
            "the panel never returned to idle (last state {state:?})"
        );
        tokio::time::sleep(std::time::Duration::from_millis(step)).await;
        waited += step;
    }
}

/// Ask the engine picker for `engine` WITHOUT waiting for the switch to take
/// effect — a refused switch never changes the workspace's engine, so the
/// waiting variant could not be used to observe one.
async fn request_engine(h: &pixelproof_web::Harness, root: &str, engine: &str) {
    let changed: bool = eval_json(
        h,
        &format!(
            r#"(() => {{
                const sel = document.querySelector('{root} [data-ai-chat-backend]');
                if (!sel) return false;
                sel.value = '{engine}';
                sel.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return sel.value === '{engine}';
            }})()"#
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false);
    assert!(changed, "the engine picker must accept {engine}");
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
}

/// Read `data-ai-chat-turn-status` repeatedly while a turn runs, returning
/// the ids in the order they were first seen.
async fn observed_statuses(h: &pixelproof_web::Harness, root: &str, samples: u32) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for _ in 0..samples {
        let status = workspace_shape(h, root).await["turnStatus"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if !status.is_empty() && seen.last() != Some(&status) && !seen.contains(&status) {
            seen.push(status);
        }
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    }
    seen
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_owned())
        .collect()
}

// ── 9 ───────────────────────────────────────────────────────────────────────

/// The lifecycle ladder is published as a typed id, and the id the actor can
/// see agrees with the record the backend holds.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn turn_lifecycle_states_are_typed_and_exposed() {
    let h = harness_at(PAGE_FAILURES).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    // Negative control, before anything is asked: no turn means `idle`, not
    // a state borrowed from a turn that never happened.
    let boot = workspace_shape(&h, ROOT).await;
    assert_eq!(boot["turnStatus"], json!("idle"), "{boot}");
    assert_eq!(boot["outcome"], json!(null), "{boot}");

    // The `cancel` script streams one word per 100 ms poll, which is the
    // only way to sample a ladder that would otherwise be over in one frame.
    type_prompt_at(&h, ROOT, "cancel").await;
    click(&h, &format!("{ROOT} [data-ai-chat-send]")).await;
    let seen = observed_statuses(&h, ROOT, 60).await;
    wait_for_idle_at(&h, ROOT, 30_000).await;
    // The record trails the panel by up to one 200 ms poll; see `ask`.
    let mut waited = 0;
    while waited < 5_000 && workspace_shape(&h, ROOT).await["outcome"].is_null() {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        waited += 200;
    }

    assert!(
        seen.contains(&"running".to_owned()),
        "the ladder must be observable while it runs: {seen:?}"
    );
    let ladder = ["admitted", "queued", "running", "validating", "completed"];
    let mut last = 0usize;
    for state in &seen {
        if state == "idle" {
            continue;
        }
        let at = ladder
            .iter()
            .position(|s| s == state)
            .unwrap_or_else(|| panic!("{state} is not a lifecycle id: {seen:?}"));
        assert!(at >= last, "the ladder ran backwards: {seen:?}");
        last = at;
    }

    let done = workspace_shape(&h, ROOT).await;
    assert_eq!(done["turnStatus"], json!("completed"), "{done}");
    assert_eq!(done["outcome"], json!("completed"), "{done}");

    // The DOM and the backend's own record agree. The oracle value is read
    // from `TurnRecord::lifecycle` by the fixture PAGE, so this compares the
    // rendering against the record rather than against a copy of itself.
    let state = debug_state(&h).await;
    assert_eq!(
        state["ai_chat.turn_status"], done["turnStatus"],
        "the header and the record disagree: {state}"
    );
    assert_eq!(state["ai_chat.outcome"], done["outcome"], "{state}");

    assert_no_browser_errors(&h, "turn lifecycle").await;
}

// ── 10 ──────────────────────────────────────────────────────────────────────

/// Plan-covered and metered usage are two numbers forever, and reasoning
/// never exceeds the output it is part of.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn usage_keeps_plan_and_metered_apart_and_reasoning_inside_output() {
    let h = harness_at(PAGE_FAILURES).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    // claude-code's seeded script reports a PRICED turn: every token lands
    // on the metered side.
    ask(&h, ROOT, "the intake checklist conflict check").await;
    let metered = workspace_shape(&h, ROOT).await;
    assert_eq!(metered["usageMetered"], json!(812 + 96), "{metered}");
    assert_eq!(
        metered["usagePlan"],
        json!(0),
        "nothing leaks across the split: {metered}"
    );
    let reasoning = metered["usageReasoning"]
        .as_u64()
        .expect("reasoning tokens");
    let output = metered["usageOutput"].as_u64().expect("output tokens");
    assert_eq!(reasoning, 41, "{metered}");
    assert_eq!(output, 96, "{metered}");
    assert!(
        reasoning <= output && reasoning > 0,
        "reasoning is part of output, and the check is not vacuous: {metered}"
    );
    assert!(
        metered["usageCost"]
            .as_str()
            .is_some_and(|c| c.starts_with('$')),
        "a priced turn shows its cost: {metered}"
    );
    assert!(
        metered["usageTps"].as_str().is_some(),
        "throughput is published once an elapsed time exists: {metered}"
    );
    // No hook anywhere carries the forbidden total.
    let total = metered["usagePlan"].as_u64().unwrap() + metered["usageMetered"].as_u64().unwrap();
    assert_ne!(
        json!(total),
        metered["usageOutput"],
        "plan+metered must never be published as one number: {metered}"
    );

    // The negative control on the SAME document: codex-cli's script reports
    // no cost, so the identical four hooks move to the plan side.
    select_engine(&h, CODEX).await;
    ask(&h, ROOT, "anything at all").await;
    let plan = workspace_shape(&h, ROOT).await;
    assert_eq!(plan["usagePlan"], json!(240 + 44), "{plan}");
    assert_eq!(plan["usageMetered"], json!(0), "{plan}");
    assert!(
        plan["usageReasoning"].as_u64().unwrap() <= plan["usageOutput"].as_u64().unwrap(),
        "{plan}"
    );
    assert_eq!(
        plan["usageCost"],
        json!(null),
        "an unpriced turn shows no cost rather than $0.0000: {plan}"
    );

    // The budget is ABSENT when the host publishes none — not an empty
    // element, which would read as a budget of nothing.
    assert_eq!(plan["usageBudget"], json!(null), "{plan}");
    let b = harness_at(PAGE_BUDGET).await;
    begin_browser_error_capture(&b).await;
    wait_for_selector(&b, &format!("{ROOT} [data-ai-chat-workspace-header]")).await;
    let exhausted = workspace_shape(&b, ROOT).await;
    assert!(
        exhausted["usageBudget"]
            .as_str()
            .is_some_and(|s| s.contains("$0.00")),
        "a host that publishes a budget gets the hook: {exhausted}"
    );
    assert_no_browser_errors(&b, "usage budget").await;

    assert_no_browser_errors(&h, "usage split").await;
}

// ── 11 ──────────────────────────────────────────────────────────────────────

/// "The account says no" and "we tried and it broke" are different states,
/// with different hooks, different tones and different words.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn honesty_not_enabled_is_distinct_from_failed() {
    let tier = harness_at(PAGE_TIER_DISABLED).await;
    begin_browser_error_capture(&tier).await;
    wait_for_selector(&tier, &format!("{ROOT} [data-ai-chat-workspace-header]")).await;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    let denied = workspace_shape(&tier, ROOT).await;
    assert_eq!(denied["honesty"], json!("not_enabled"), "{denied}");
    assert_eq!(
        denied["honestyReason"],
        json!("tier_effects_disabled"),
        "{denied}"
    );
    // Read back from the BACKEND's own settings, not from the header.
    assert_eq!(
        debug_state(&tier).await["ai_chat.tier_verdict"],
        json!("denied")
    );
    assert_no_browser_errors(&tier, "tier honesty").await;

    let h = harness_at(PAGE_FAILURES).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    // Negative control #1, same document: nothing is wrong, and the strip
    // says so positively rather than disappearing.
    let ok = workspace_shape(&h, ROOT).await;
    assert_eq!(ok["honesty"], json!("ready"), "{ok}");
    assert_eq!(ok["honestyReason"], json!(null), "{ok}");

    // Negative control #2, same document: a turn that broke.
    ask(&h, ROOT, "error").await;
    let failed = workspace_shape(&h, ROOT).await;
    assert_eq!(failed["honesty"], json!("failed"), "{failed}");
    assert_eq!(failed["honestyReason"], json!("engine_error"), "{failed}");

    assert_ne!(
        denied["honesty"], failed["honesty"],
        "the two states must not collapse into one hook value"
    );
    assert_ne!(
        denied["honestyClass"], failed["honestyClass"],
        "nor into one tone — a proof that only compared hooks could pass \
         while both looked identical on screen"
    );
    assert_ne!(
        denied["honestyText"], failed["honestyText"],
        "nor into one sentence"
    );

    assert_no_browser_errors(&h, "failed honesty").await;
}

// ── 12 ──────────────────────────────────────────────────────────────────────

/// A failure renders the kind its RECORD carries, and an untested CLI
/// version is not an expired credential.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn failure_shapes_render_their_own_kind_and_untested_cli_is_not_expired() {
    let h = harness_at(PAGE_FAILURES).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    ask(&h, ROOT, "untested").await;
    let untested = workspace_shape(&h, ROOT).await;
    assert_eq!(
        untested["failure"],
        json!("cli_version_untested"),
        "the kind comes off the record's own reason code: {untested}"
    );
    assert_eq!(untested["outcome"], json!("unavailable"), "{untested}");
    let untested_text = untested["honestyText"]
        .as_str()
        .unwrap_or_default()
        .to_lowercase();
    assert!(
        !untested_text.contains("sign") && !untested_text.contains("expire"),
        "an untested CLI version must never read as a stale credential: \
         {untested_text:?}"
    );

    // The negative control on the same document: a reason that really IS a
    // credential problem, rendered as its own kind with its own words.
    ask(&h, ROOT, "expired").await;
    let expired = workspace_shape(&h, ROOT).await;
    assert_eq!(expired["failure"], json!("sign_in_expired"), "{expired}");
    assert_ne!(untested["failure"], expired["failure"]);
    assert_ne!(untested["honestyText"], expired["honestyText"]);
    assert!(
        expired["honestyText"]
            .as_str()
            .unwrap_or_default()
            .to_lowercase()
            .contains("sign-in"),
        "and the expired one does say so: {expired}"
    );

    // A crash is a third shape again: `failed`, not `unavailable`.
    ask(&h, ROOT, "error").await;
    let errored = workspace_shape(&h, ROOT).await;
    assert_eq!(errored["failure"], json!("engine_error"), "{errored}");
    assert_eq!(errored["outcome"], json!("failed"), "{errored}");
    assert_ne!(errored["outcome"], untested["outcome"]);

    assert_no_browser_errors(&h, "failure shapes").await;
}

// ── 13 ──────────────────────────────────────────────────────────────────────

/// Declining is a healthy completion that says what it could not cover — it
/// is never dressed as an error.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn declined_is_a_healthy_completion_with_limitations() {
    let h = harness_at(PAGE_FAILURES).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    ask(&h, ROOT, "decline").await;
    let declined = workspace_shape(&h, ROOT).await;
    assert_eq!(declined["turnStatus"], json!("completed"), "{declined}");
    assert_eq!(declined["outcome"], json!("declined"), "{declined}");
    assert_eq!(
        declined["failure"],
        json!(null),
        "a decline carries no failure kind: {declined}"
    );
    assert_eq!(
        declined["honesty"],
        json!("ready"),
        "and puts the workspace in no honesty fault: {declined}"
    );
    let limitations = strings(&declined["limitations"]);
    assert_eq!(limitations.len(), 2, "{declined}");
    assert!(
        limitations.iter().any(|l| l.contains("court calendar")),
        "each limitation is listed, not summarized away: {limitations:?}"
    );

    // The negative control on the same document: a turn that really failed
    // lists nothing and carries a kind.
    ask(&h, ROOT, "error").await;
    let failed = workspace_shape(&h, ROOT).await;
    assert_eq!(failed["outcome"], json!("failed"), "{failed}");
    assert!(strings(&failed["limitations"]).is_empty(), "{failed}");
    assert_ne!(failed["failure"], json!(null), "{failed}");

    assert_no_browser_errors(&h, "declined completion").await;
}

// ── 14 ──────────────────────────────────────────────────────────────────────

/// A refusal before a turn is admitted keeps the actor's draft and offers
/// exactly one typed next step.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn pre_admission_refusals_keep_the_draft_and_offer_the_typed_next_action() {
    let h = harness_at(PAGE_GROQ_NO_KEY).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    // Negative control, same document: nothing is refused yet.
    let before = workspace_shape(&h, ROOT).await;
    assert_eq!(before["refusalKind"], json!(null), "{before}");
    assert!(
        before["refusalActions"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "{before}"
    );

    const DRAFT: &str = "a half-written question that must survive the refusal";
    type_prompt_at(&h, ROOT, DRAFT).await;
    request_engine(&h, ROOT, GROQ).await;

    let refused = workspace_shape(&h, ROOT).await;
    assert_eq!(
        refused["refusalKind"],
        json!("refused"),
        "the refusal's wire string comes from `ChatWorkspaceErrorKind::as_str`, \
         never from Debug formatting: {refused}"
    );
    assert_eq!(
        refused["refusalReason"],
        json!("credential_key_unavailable"),
        "{refused}"
    );
    assert_eq!(
        refused["refusalActions"],
        json!(["open_settings"]),
        "exactly one next step, and the one the reason code picks: {refused}"
    );
    assert_eq!(
        refused["engine"],
        json!(CLAUDE),
        "a refused switch leaves the running engine alone: {refused}"
    );
    assert_eq!(
        refused["composer"],
        json!(DRAFT),
        "and the draft the actor was writing survives: {refused}"
    );

    // The button reaches the host, which is the only place a settings pane
    // can actually be opened.
    click(&h, &format!("{ROOT} [data-ai-chat-refusal-action]")).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert_eq!(
        debug_state(&h).await["ai_chat.refusal_action"],
        json!("open_settings"),
        "the typed action, not a label, is what the host receives"
    );

    assert_no_browser_errors(&h, "pre-admission refusal").await;
}

// ── 15 ──────────────────────────────────────────────────────────────────────

/// An escalation and a truncation are both announced — once each per turn,
/// never folded into a silent completion.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn groq_escalation_retry_and_truncation_are_shown() {
    let h = harness_at(PAGE_FAILURES).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    ask(&h, ROOT, "escalate").await;
    let escalated = workspace_shape(&h, ROOT).await;
    assert_eq!(escalated["escalation"], json!(true), "{escalated}");
    assert_eq!(
        escalated["truncated"],
        json!(false),
        "an escalation is not a truncation: {escalated}"
    );
    let notices = strings(&escalated["noticeRows"]);
    let escalation_notices = notices
        .iter()
        .filter(|n| n.contains("medium") && n.contains("low"))
        .count();
    assert_eq!(
        escalation_notices, 1,
        "the transcript is told exactly once, naming both efforts: {notices:?}"
    );

    // Announced once PER TURN, not once per poll: the composite re-reads the
    // record every 200 ms, so a missing "already announced" guard would pile
    // up duplicates rather than add exactly one.
    ask(&h, ROOT, "escalate").await;
    let twice = workspace_shape(&h, ROOT).await;
    let notices = strings(&twice["noticeRows"]);
    let escalation_notices = notices
        .iter()
        .filter(|n| n.contains("medium") && n.contains("low"))
        .count();
    assert_eq!(
        escalation_notices, 2,
        "two turns, two notices — not one and not a dozen: {notices:?}"
    );

    // A truncation is its own hook and its own outcome on the same document.
    ask(&h, ROOT, "truncate").await;
    let truncated = workspace_shape(&h, ROOT).await;
    assert_eq!(truncated["truncated"], json!(true), "{truncated}");
    assert_eq!(
        truncated["escalation"],
        json!(false),
        "and the previous turn's escalation is gone: {truncated}"
    );
    assert_eq!(
        truncated["outcome"],
        json!("truncated"),
        "a cut-short answer never reports itself as completed: {truncated}"
    );

    assert_no_browser_errors(&h, "escalation and truncation").await;
}

// ── 16 ──────────────────────────────────────────────────────────────────────

/// A cancel that keeps its partial and a cancel that discards it are two
/// visibly different events, side by side on one document.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn cancel_discarded_true_and_false_render_differently() {
    let h = harness_at(PAGE_CANCEL).await;
    begin_browser_error_capture(&h).await;
    let kept_root = case_root("kept");
    let discarded_root = case_root("discarded");
    ready_at(&h, &kept_root).await;
    ready_at(&h, &discarded_root).await;

    for root in [&kept_root, &discarded_root] {
        type_prompt_at(&h, root, "cancel").await;
        click(&h, &format!("{root} [data-ai-chat-send]")).await;
    }
    // Let both answers genuinely start arriving before stopping them, or the
    // "kept" case would have nothing to keep and the test would be vacuous.
    tokio::time::sleep(std::time::Duration::from_millis(1_500)).await;
    for root in [&kept_root, &discarded_root] {
        let streaming = workspace_shape(&h, root).await;
        assert!(
            strings(&streaming["texts"])
                .iter()
                .any(|t| t.contains("long on purpose")),
            "a partial answer must already be on screen before the stop: \
             {streaming}"
        );
        click(&h, &format!("{root} [data-ai-chat-stop]")).await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(900)).await;

    let kept = workspace_shape(&h, &kept_root).await;
    let discarded = workspace_shape(&h, &discarded_root).await;

    assert_eq!(kept["cancelDiscarded"], json!("false"), "{kept}");
    assert_eq!(discarded["cancelDiscarded"], json!("true"), "{discarded}");
    assert_eq!(kept["outcome"], json!("canceled"), "{kept}");
    assert_eq!(discarded["outcome"], json!("canceled"), "{discarded}");

    assert!(
        kept["canceledPartial"].as_str().is_some(),
        "what survived is shown as a PARTIAL, never as an answer: {kept}"
    );
    assert_eq!(
        discarded["canceledPartial"],
        json!(null),
        "and a discarded partial is not rendered as an empty one: {discarded}"
    );

    let kept_notices = strings(&kept["noticeRows"]);
    let discarded_notices = strings(&discarded["noticeRows"]);
    assert!(
        kept_notices.iter().any(|n| n.contains("kept")),
        "{kept_notices:?}"
    );
    assert!(
        discarded_notices.iter().any(|n| n.contains("discarded")),
        "{discarded_notices:?}"
    );
    assert_ne!(
        kept_notices, discarded_notices,
        "the two cancels must not share one sentence"
    );

    assert_no_browser_errors(&h, "cancel discarded").await;
}

// ── 17 ──────────────────────────────────────────────────────────────────────

/// Starting a new conversation clears the transcript AND reaches the
/// transport — a local clear that never told the host would leave the two
/// sides disagreeing about what exists.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn restart_clears_the_transcript_and_reaches_the_transport() {
    let h = harness_at(PAGE_FAILURES).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    // Negative control: nothing has restarted yet.
    assert!(
        !backend_calls(&h).await.iter().any(|c| c == "Restart"),
        "no restart before the click"
    );

    ask(&h, ROOT, "the intake checklist conflict check").await;
    let answered = workspace_shape(&h, ROOT).await;
    assert!(
        strings(&answered["roles"]).contains(&"user".to_owned()),
        "there is a conversation to clear: {answered}"
    );

    click(&h, &format!("{ROOT} [data-ai-chat-new-session]")).await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let cleared = workspace_shape(&h, ROOT).await;
    assert!(
        strings(&cleared["roles"]).is_empty(),
        "the transcript is gone: {cleared}"
    );
    assert!(
        backend_calls(&h).await.iter().any(|c| c == "Restart"),
        "and the transport was told, not just the screen"
    );

    assert_no_browser_errors(&h, "restart").await;
}

// ── 18 ──────────────────────────────────────────────────────────────────────

/// Retrying a failed turn re-sends the same prompt exactly once.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn retry_failed_turn_resends_the_same_prompt_once() {
    let h = harness_at(PAGE_FAILURES).await;
    begin_browser_error_capture(&h).await;
    ready_at(&h, ROOT).await;

    ask(&h, ROOT, "error").await;
    let failed = workspace_shape(&h, ROOT).await;
    assert_eq!(failed["errorStrip"], json!(true), "{failed}");
    assert_eq!(failed["retryButton"], json!(true), "{failed}");
    let before = distinct_turn_ids(&backend_calls(&h).await);
    assert_eq!(before.len(), 1, "one turn so far: {before:?}");
    let user_bubbles = strings(&failed["roles"])
        .iter()
        .filter(|r| *r == "user")
        .count();
    assert_eq!(user_bubbles, 1, "{failed}");

    click(&h, &format!("{ROOT} [data-ai-chat-retry]")).await;
    // While the retry is in flight the strip — and its button — are gone, so
    // a second press cannot produce a third turn.
    let in_flight = workspace_shape(&h, ROOT).await;
    assert_eq!(
        in_flight["retryButton"],
        json!(false),
        "the retry button must not be pressable twice for one failure: \
         {in_flight}"
    );
    wait_for_idle_at(&h, ROOT, 30_000).await;
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let after = distinct_turn_ids(&backend_calls(&h).await);
    assert_eq!(
        after.len(),
        2,
        "exactly one more turn existed, never two: {after:?}"
    );
    let retried = workspace_shape(&h, ROOT).await;
    let user_bubbles = strings(&retried["roles"])
        .iter()
        .filter(|r| *r == "user")
        .count();
    assert_eq!(
        user_bubbles, 1,
        "the SAME prompt was replayed — a retry that echoed a new user \
         message would have typed it again: {retried}"
    );
    assert_eq!(
        retried["failure"],
        json!("engine_error"),
        "and the replay failed the same way, which is why the strip returns: \
         {retried}"
    );

    assert_no_browser_errors(&h, "retry").await;
}

// ── 19 ──────────────────────────────────────────────────────────────────────

/// A denied tier outranks every engine's own verdict.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn tier_verdict_outranks_the_engine_list() {
    let h = harness_at(PAGE_TIER_DISABLED).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{ROOT} [data-ai-chat-workspace-header]")).await;
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    let denied = workspace_shape(&h, ROOT).await;
    assert_eq!(denied["tierVerdict"], json!("denied"), "{denied}");
    let rows = denied["availability"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        rows.len(),
        5,
        "every card is listed, not only the broken ones: {denied}"
    );
    for row in &rows {
        assert_eq!(
            row["reason"],
            json!("tier_effects_disabled"),
            "each card reports the ACCOUNT's reason, not its own: {row}"
        );
    }
    // The same-document control that makes this claim mean something:
    // claude-code is individually ready in this catalogue, and it is still
    // listed as denied.
    assert!(
        rows.iter().any(|r| r["engine"] == json!(CLAUDE)),
        "a card that is perfectly healthy on its own is still denied: {denied}"
    );
    assert_eq!(
        denied["panel"],
        json!(false),
        "and no session is opened at all, so there is nothing to ask with: \
         {denied}"
    );
    assert_eq!(
        denied["refusalActions"],
        json!(["open_settings"]),
        "the one place an actor can check a plan: {denied}"
    );
    assert_no_browser_errors(&h, "tier denied").await;

    // The negative control: an allowed tier lists only the engine that is
    // actually broken, and the healthy ones are absent.
    let g = harness_at(PAGE_GROQ_NO_KEY).await;
    begin_browser_error_capture(&g).await;
    ready_at(&g, ROOT).await;
    let allowed = workspace_shape(&g, ROOT).await;
    assert_eq!(allowed["tierVerdict"], json!("allowed"), "{allowed}");
    let rows = allowed["availability"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(rows.len(), 1, "{allowed}");
    assert_eq!(rows[0]["engine"], json!(GROQ), "{allowed}");
    assert_eq!(
        allowed["panel"],
        json!(true),
        "and a healthy engine still opens a session: {allowed}"
    );
    assert_no_browser_errors(&g, "tier allowed").await;
}

// ── 20 ──────────────────────────────────────────────────────────────────────

/// A local runtime that is not running and a model that is not installed are
/// different problems with different fixes, and they say so.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn ollama_not_running_and_model_not_pulled_are_distinct() {
    let h = harness_at(PAGE_OLLAMA).await;
    begin_browser_error_capture(&h).await;
    let not_running_root = case_root("not_running");
    let model_missing_root = case_root("model_missing");
    ready_at(&h, &not_running_root).await;
    ready_at(&h, &model_missing_root).await;

    let not_running = workspace_shape(&h, &not_running_root).await;
    let model_missing = workspace_shape(&h, &model_missing_root).await;

    let row_for = |shape: &Value| -> Value {
        shape["availability"]
            .as_array()
            .and_then(|rows| rows.iter().find(|r| r["engine"] == json!(OLLAMA)).cloned())
            .unwrap_or(Value::Null)
    };
    let stopped = row_for(&not_running);
    let missing = row_for(&model_missing);

    assert_eq!(
        stopped["reason"],
        json!("engine_process_not_running"),
        "{not_running}"
    );
    assert_eq!(
        missing["reason"],
        json!("model_not_installed"),
        "{model_missing}"
    );
    assert_ne!(
        stopped["reason"], missing["reason"],
        "two problems, two codes"
    );

    let stopped_text = stopped["text"].as_str().unwrap_or_default();
    let missing_text = missing["text"].as_str().unwrap_or_default();
    assert!(
        stopped_text.contains("ollama serve"),
        "an actor is told how to start the runtime: {stopped_text:?}"
    );
    assert!(
        missing_text.contains("ollama pull"),
        "and, separately, how to fetch the model: {missing_text:?}"
    );
    assert!(
        !stopped_text.contains("ollama pull") && !missing_text.contains("ollama serve"),
        "neither is told to do the other one's job"
    );
    for text in [stopped_text, missing_text] {
        let lower = text.to_lowercase();
        assert!(
            !lower.contains("sign in") && !lower.contains("credential"),
            "a local runtime problem is never a credential problem: {text:?}"
        );
    }

    assert_no_browser_errors(&h, "ollama reasons").await;
}

// ── 21 ──────────────────────────────────────────────────────────────────────

/// The watchdog fires from the FIXTURE CLOCK, exactly once per turn, and
/// lands a real failure in the transcript.
///
/// Three properties, each with its own assertions below:
///
/// 1. **Once per turn, not once per tick.** The fast document's clock jumps
///    20 s per transport poll, so this waits several whole watchdog periods
///    past the 120 000 ms threshold. The original bug cleared the watched
///    entry instead of marking it fired, which re-announced every notice on
///    the next period — an "at least one" assertion would have passed
///    straight through it.
/// 2. **The elapsed time comes from the `now_ms` prop.** The FROZEN
///    workspace beside it runs the identical stalled script with a clock of
///    0 ms per poll; if the threshold were read from any other source it
///    would trip there too.
/// 3. **`fail_turn` and its annotation land.** The transcript carries the
///    timeout notice and the panel carries the session's own error.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ai-chat)"]
async fn watchdog_fires_once_from_the_fixture_clock_and_never_without_it() {
    let h = harness_at(PAGE_WATCHDOG).await;
    begin_browser_error_capture(&h).await;
    let fast_root = case_root("fast");
    let frozen_root = case_root("frozen");
    ready_at(&h, &fast_root).await;
    ready_at(&h, &frozen_root).await;

    // A script that never emits `Done`: only a watchdog can end either turn.
    for root in [&fast_root, &frozen_root] {
        type_prompt_at(&h, root, "stall").await;
        click(&h, &format!("{root} [data-ai-chat-send]")).await;
    }
    // At 20 s of fixture time per 100 ms poll this is ~600 s of fixture
    // time — five whole watchdog periods past the threshold.
    tokio::time::sleep(std::time::Duration::from_millis(3_000)).await;

    let fast = workspace_shape(&h, &fast_root).await;
    let frozen = workspace_shape(&h, &frozen_root).await;

    // Property 3: the failure is real and typed.
    assert_eq!(
        fast["failure"],
        json!("watchdog"),
        "a turn nobody finished is a failure, and its own kind: {fast}"
    );
    assert_eq!(fast["honesty"], json!("failed"), "{fast}");
    assert_eq!(
        fast["outcome"],
        json!("failed"),
        "`fail_turn` reaches the transport through `cancel()`, but the actor \
         never pressed Stop and must not be told they did: {fast}"
    );
    assert_eq!(
        fast["cancelDiscarded"],
        json!(null),
        "for the same reason, no cancel is published at all: {fast}"
    );
    // `ChatSession::fail_turn` applies a `StreamEvent::Error`, which the
    // transcript renders as a SYSTEM row — it does not go through `AiChat`'s
    // own retry strip, which only its poll path fills. The system row is the
    // evidence that `fail_turn` really ran.
    let system_rows = strings(&fast["roles"])
        .iter()
        .filter(|r| *r == "system")
        .count();
    assert_eq!(
        system_rows, 1,
        "`fail_turn` really ran, exactly once: {fast}"
    );
    assert!(
        strings(&fast["texts"])
            .iter()
            .any(|t| t.contains("System") && t.contains("timed out")),
        "and the transcript says what happened: {fast}"
    );

    // Property 1: exactly one notice, after several periods' worth of ticks.
    let notices = strings(&fast["noticeRows"]);
    let timeouts = notices.iter().filter(|n| n.contains("timed out")).count();
    assert_eq!(
        timeouts, 1,
        "the watchdog announces ONCE per turn, not once per tick and not \
         once per period: {notices:?}"
    );

    // Property 2: without a clock that advances, the identical script is
    // still running and nothing fired.
    assert_eq!(
        frozen["failure"],
        json!(null),
        "a frozen clock can never cross the threshold: {frozen}"
    );
    assert_eq!(frozen["honesty"], json!("ready"), "{frozen}");
    assert_eq!(
        frozen["turnStatus"],
        json!("running"),
        "and its turn really is still in flight — the control is not simply \
         a workspace that never started: {frozen}"
    );
    let frozen_notices = strings(&frozen["noticeRows"]);
    assert!(
        !frozen_notices.iter().any(|n| n.contains("timed out")),
        "{frozen_notices:?}"
    );

    assert_no_browser_errors(&h, "watchdog").await;
}
