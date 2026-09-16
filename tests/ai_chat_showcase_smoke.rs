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
