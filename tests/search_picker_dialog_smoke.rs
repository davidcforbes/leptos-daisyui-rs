//! Real-browser proof for `SearchPickerDialog` (ldui-i95p): opening focuses
//! the search field, Escape/Cancel closes and restores focus to the
//! trigger, typed activation resolves the exact current keyed payload
//! (including duplicate-titled rows), a superseded async response never
//! lands, loading/empty/retained-error presentation and retry work, and two
//! independent dialog instances never collide.
//!
//! Drives the general demo app (`html_target: None`, like
//! `reactivity_smoke.rs`/`keyed_result_list_smoke.rs`/
//! `section_heading_smoke.rs`) rather than a dedicated test-host page,
//! because the fixture lives on the existing
//! `/components/search_picker_dialog` showcase route. Kept in its own
//! file/xtask step (`cargo xtask test-search-picker-dialog`) rather than
//! folded into `reactivity_smoke.rs`, whose check count is pinned.

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, harness_at};
use pixelproof_web::Key;
use serde_json::{Value, json};
use std::time::Duration;

const PAGE: &str = "/components/search_picker_dialog";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate search-picker-dialog fixture")
        .into_value()
        .expect("search-picker-dialog expression returns JSON")
}

/// Let a debounced/simulated-async update land before reading state back.
/// The fixture's slowest scheduled response is 500ms (the race scenario).
async fn settle(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await;
}

fn dialog_selector(instance: &str) -> String {
    format!("[data-testid=\"{instance}-fixture\"] [data-search-picker-dialog=\"true\"]")
}

async fn open_dialog(h: &pixelproof_web::Harness, instance: &str) {
    let script = format!(r#"document.querySelector('[data-testid="{instance}-trigger"]').click()"#);
    let _ = h.page().evaluate(script.as_str()).await;
    settle(150).await;
}

async fn type_query(h: &pixelproof_web::Harness, instance: &str, text: &str) {
    let dialog = dialog_selector(instance);
    let script = format!(
        r#"(() => {{
            const input = document.querySelector('{dialog} input[type="search"]');
            input.value = {text:?};
            input.dispatchEvent(new Event('input', {{ bubbles: true }}));
        }})()"#
    );
    let _ = h.page().evaluate(script.as_str()).await;
}

/// Status readout text, active element description, dialog open state, the
/// currently rendered `[data-page-state-panel]` slug (if any), and the
/// visible result-row keys, all scoped to one dialog instance.
async fn snapshot(h: &pixelproof_web::Harness, instance: &str) -> Value {
    let dialog = dialog_selector(instance);
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const status = document.querySelector('[data-testid="{instance}-status"]');
                const dialogEl = document.querySelector('{dialog}')?.closest('dialog');
                const panel = document.querySelector('{dialog} [data-page-state-panel]');
                const rows = Array.from(
                    document.querySelectorAll('{dialog} [data-result-key]')
                ).map(el => el.dataset.resultKey);
                const active = document.activeElement;
                return {{
                    statusText: status ? status.textContent.trim() : null,
                    dialogOpen: dialogEl ? dialogEl.open : null,
                    panel: panel ? panel.getAttribute('data-page-state-panel') : null,
                    rowKeys: rows,
                    activeIsSearchInput: active
                        ? active.matches('{dialog} input[type="search"]')
                        : false,
                    activeIsTrigger: active
                        ? active === document.querySelector('[data-testid="{instance}-trigger"]')
                        : false,
                }};
            }})()"#
        ),
    )
    .await
}

/// Opening the dialog moves focus to the search field; `Escape` closes it
/// and returns focus to the trigger button that opened it (native
/// `<dialog>` focus-restoration, driven through the caller's controlled
/// `open` signal rather than the browser's default `cancel` action).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn opening_focuses_search_and_escape_closes_and_restores_focus() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "dialog-a").await;
    let opened = snapshot(&h, "dialog-a").await;
    assert_eq!(
        opened["dialogOpen"],
        json!(true),
        "dialog is open: {opened}"
    );
    assert_eq!(
        opened["activeIsSearchInput"],
        json!(true),
        "opening must focus the search field: {opened}"
    );

    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    settle(200).await;
    let closed = snapshot(&h, "dialog-a").await;
    assert_eq!(
        closed["dialogOpen"],
        json!(false),
        "Escape closes: {closed}"
    );
    assert_eq!(
        closed["activeIsTrigger"],
        json!(true),
        "focus returns to the element that opened the dialog: {closed}"
    );

    assert_no_browser_errors(&h, "search-picker-dialog focus/escape journey").await;
}

/// The Cancel button closes the dialog and returns focus to the trigger,
/// exactly like `Escape` -- proving the acceptance clause's "Escape/Cancel"
/// pairing rather than only its keyboard half.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn cancel_button_closes_and_restores_focus() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "dialog-a").await;
    let opened = snapshot(&h, "dialog-a").await;
    assert_eq!(
        opened["dialogOpen"],
        json!(true),
        "dialog is open: {opened}"
    );

    let cancel_script = format!(
        r#"document.querySelector('{} [data-search-picker-dialog-cancel="true"]').click()"#,
        dialog_selector("dialog-a")
    );
    let _ = h.page().evaluate(cancel_script.as_str()).await;
    settle(200).await;
    let closed = snapshot(&h, "dialog-a").await;
    assert_eq!(
        closed["dialogOpen"],
        json!(false),
        "Cancel closes: {closed}"
    );
    assert_eq!(
        closed["activeIsTrigger"],
        json!(true),
        "focus returns to the element that opened the dialog: {closed}"
    );

    assert_no_browser_errors(&h, "search-picker-dialog Cancel-button journey").await;
}

/// `case-a` and `case-b` both render the display title "Alex Morgan".
/// Arrowing to the second row and activating it must return `case-b`'s own
/// payload, resolved fresh against the current items -- never the first
/// row's, and never by display text.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn typed_activation_resolves_the_exact_duplicate_titled_row() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "dialog-a").await;
    type_query(&h, "dialog-a", "Alex").await;
    settle(400).await;

    let ready = snapshot(&h, "dialog-a").await;
    assert_eq!(
        ready["rowKeys"],
        json!(["case-a", "case-b"]),
        "both duplicate-titled rows render: {ready}"
    );

    h.press_key_sequence(&[Key::ArrowDown])
        .await
        .expect("ArrowDown onto case-b");
    h.press_key_sequence(&[Key::Enter])
        .await
        .expect("Enter activates case-b");
    settle(150).await;

    let activated = snapshot(&h, "dialog-a").await;
    assert_eq!(
        activated["statusText"],
        json!("Status: Ready | Activated: case-b (B-200)"),
        "activation must resolve case-b's own payload, not case-a's: {activated}"
    );
    assert_eq!(
        activated["dialogOpen"],
        json!(false),
        "the fixture closes the dialog on activation: {activated}"
    );

    assert_no_browser_errors(&h, "search-picker-dialog duplicate-title activation").await;
}

/// A slow "Alex" search is superseded by a fast "Priya" search before the
/// slow one resolves. The stale response must never land -- `items` (and
/// therefore the rendered rows) must reflect only the fast query's results.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn a_superseded_async_response_never_overwrites_the_current_query() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "dialog-a").await;
    let script = r#"document.querySelector('[data-testid="dialog-a-race"]').click()"#;
    let _ = h.page().evaluate(script).await;

    // The slow ("Alex") response is scheduled 500ms out; wait well past it.
    settle(800).await;

    let s = snapshot(&h, "dialog-a").await;
    assert_eq!(
        s["rowKeys"],
        json!(["case-c"]),
        "only the fast query's results (Priya Natarajan) may ever be visible: {s}"
    );

    assert_no_browser_errors(&h, "search-picker-dialog stale response race").await;
}

/// A query with no matches shows the empty-dataset panel with no rows.
/// Forcing an error while rows are retained shows the retained-error panel
/// *above* the still-visible rows; its retry action re-runs the search and
/// clears the error.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn empty_and_retained_error_states_present_correctly_and_retry_recovers() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "dialog-a").await;
    type_query(&h, "dialog-a", "zzz-no-match").await;
    settle(400).await;
    let empty = snapshot(&h, "dialog-a").await;
    assert_eq!(
        empty["panel"],
        json!("empty-dataset"),
        "no matches shows the empty-dataset panel: {empty}"
    );
    assert_eq!(empty["rowKeys"], json!([]), "no rows render: {empty}");

    type_query(&h, "dialog-a", "Alex").await;
    settle(400).await;
    let ready = snapshot(&h, "dialog-a").await;
    assert_eq!(
        ready["panel"],
        json!(null),
        "results with no panel: {ready}"
    );
    assert_eq!(ready["rowKeys"], json!(["case-a", "case-b"]), "{ready}");

    let script = r#"document.querySelector('[data-testid="dialog-a-force-error"]').click()"#;
    let _ = h.page().evaluate(script).await;
    settle(100).await;
    let errored = snapshot(&h, "dialog-a").await;
    assert_eq!(
        errored["panel"],
        json!("retained-error"),
        "a failed refresh with retained rows shows retained-error: {errored}"
    );
    assert_eq!(
        errored["rowKeys"],
        json!(["case-a", "case-b"]),
        "retained rows must stay visible under the error notice: {errored}"
    );

    let retry_script = format!(
        r#"document.querySelector('{} [data-page-state-panel="retained-error"] button')?.click()"#,
        dialog_selector("dialog-a")
    );
    let _ = h.page().evaluate(retry_script.as_str()).await;
    settle(400).await;
    let recovered = snapshot(&h, "dialog-a").await;
    assert_eq!(
        recovered["panel"],
        json!(null),
        "retry re-runs the search and clears the retained error: {recovered}"
    );

    assert_no_browser_errors(
        &h,
        "search-picker-dialog empty/retained-error/retry journey",
    )
    .await;
}

/// Two `SearchPickerDialog` instances on the same page never collide:
/// typing in one does not leak into the other's state, and each has its own
/// independent DOM (distinct search fields, distinct dialog elements).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn two_dialog_instances_stay_independent() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "dialog-a").await;
    type_query(&h, "dialog-a", "Priya").await;
    settle(400).await;
    let a = snapshot(&h, "dialog-a").await;
    assert_eq!(a["rowKeys"], json!(["case-c"]), "dialog-a shows Priya: {a}");

    h.press_key_sequence(&[Key::Escape])
        .await
        .expect("close dialog-a");
    settle(200).await;

    open_dialog(&h, "dialog-b").await;
    let b = snapshot(&h, "dialog-b").await;
    assert_eq!(
        b["statusText"],
        json!("Status: Idle | Activated: None"),
        "dialog-b starts fresh, unaffected by dialog-a's query: {b}"
    );
    assert_eq!(
        b["activeIsSearchInput"],
        json!(true),
        "dialog-b's own opening still focuses its own search field: {b}"
    );

    let distinct = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const a = document.querySelector('{} input[type="search"]');
                const b = document.querySelector('{} input[type="search"]');
                return !!a && !!b && a !== b;
            }})()"#,
            dialog_selector("dialog-a"),
            dialog_selector("dialog-b")
        ),
    )
    .await;
    assert_eq!(
        distinct,
        json!(true),
        "each dialog instance owns its own distinct search field element"
    );

    assert_no_browser_errors(&h, "search-picker-dialog two-instance independence").await;
}
