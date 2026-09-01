//! Real-browser proof for `SearchPickerDialog` (ldui-i95p): opening focuses
//! the search field, Escape/Cancel/backdrop all close and restore focus to
//! the trigger (`ldui-rolc`), typed activation resolves the exact current
//! keyed payload (including duplicate-titled rows), a superseded async
//! response never lands, loading/empty/retained-error presentation and
//! retry work, and two independent dialog instances never collide.
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

/// The confirmable pattern's own dialog root (`ldui-iq0o`), scoped to one
/// fixture instance. A separate marker from `dialog_selector`'s because the
/// two are separate components, not two modes of one.
fn confirmable_selector(instance: &str) -> String {
    format!("[data-testid=\"{instance}-fixture\"] [data-confirmable-search-picker-dialog=\"true\"]")
}

async fn open_dialog(h: &pixelproof_web::Harness, instance: &str) {
    // `.focus()` before `.click()`: a real mouse click focuses a clickable
    // button as part of its default mousedown behavior *before* the click
    // event fires, which is what lets the dialog's native `previously
    // focused element` bookkeeping (and therefore its native close-time
    // focus restoration) land on the trigger. `Element.click()` alone is a
    // synthetic click that does not synthesize that mousedown-driven focus
    // step, so without this the trigger is never actually focused and the
    // dialog silently records `<body>` as the element to restore focus to.
    let script = format!(
        r#"(() => {{
            const trigger = document.querySelector('[data-testid="{instance}-trigger"]');
            trigger.focus();
            trigger.click();
        }})()"#
    );
    let _ = h.page().evaluate(script.as_str()).await;
    settle(150).await;
}

/// Activates the backdrop the way a pointer does -- by submitting the
/// `method="dialog"` form `Modal` renders for it under `backdrop=true`.
/// Located by the component's own `data-modal-backdrop` marker, scoped to
/// this instance's dialog, exactly like `modal_close_proposal_smoke.rs`'s
/// own `activate_backdrop`.
async fn activate_backdrop(h: &pixelproof_web::Harness, instance: &str) {
    let dialog = dialog_selector(instance);
    let script = format!(
        r#"(() => {{
            const dialogEl = document.querySelector('{dialog}').closest('dialog');
            const backdrop = dialogEl.querySelector('[data-modal-backdrop="true"]');
            backdrop.querySelector('button').click();
        }})()"#
    );
    let _ = h.page().evaluate(script.as_str()).await;
    settle(200).await;
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
                    hasBackdrop: !!(dialogEl && dialogEl.querySelector('[data-modal-backdrop="true"]')),
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

/// A backdrop click closes the dialog and returns focus to the trigger, the
/// same as `Escape` -- the regression `ldui-rolc` fixes. The dialog used to
/// hand-roll only `on:cancel`, which fires for Escape alone; the backdrop is
/// a `method="dialog"` form, so activating it submits rather than cancels,
/// and used to close the dialog with no event the pattern was listening for
/// at all, leaving the caller's `open` signal `true` behind a shut dialog.
/// Migrating onto `Modal`'s `on_close_request` contract (`ldui-e0fw`) routes
/// the backdrop's proposal through the same `request_close` path as
/// Escape/Cancel, so this must now behave identically to both.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn backdrop_click_closes_and_restores_focus() {
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
        opened["hasBackdrop"],
        json!(true),
        "SearchPickerDialog renders Modal's backdrop=true form: {opened}"
    );

    activate_backdrop(&h, "dialog-a").await;
    let closed = snapshot(&h, "dialog-a").await;
    assert_eq!(
        closed["dialogOpen"],
        json!(false),
        "backdrop click closes: {closed}"
    );
    assert_eq!(
        closed["activeIsTrigger"],
        json!(true),
        "focus returns to the element that opened the dialog: {closed}"
    );

    assert_no_browser_errors(&h, "search-picker-dialog backdrop journey").await;
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

// ---------------------------------------------------------------------------
// ConfirmableSearchPickerDialog (ldui-iq0o)
// ---------------------------------------------------------------------------

/// Types into the confirmable dialog's own search field, located by the
/// pattern's stable marker rather than by document position.
async fn type_confirmable_query(h: &pixelproof_web::Harness, instance: &str, text: &str) {
    let dialog = confirmable_selector(instance);
    let script = format!(
        r#"(() => {{
            const input = document.querySelector(
                '{dialog} [data-confirmable-search-picker-search="true"]'
            );
            input.value = {text:?};
            input.dispatchEvent(new Event('input', {{ bubbles: true }}));
        }})()"#
    );
    let _ = h.page().evaluate(script.as_str()).await;
}

/// Everything the acceptance criteria ask about at once: the side-effect
/// counter, the semantic (`aria-*`) and rendered (`data-*`) confirm state,
/// the selected-result summary, the visible rows, and where focus is.
async fn confirmable_snapshot(h: &pixelproof_web::Harness, instance: &str) -> Value {
    let dialog = confirmable_selector(instance);
    eval_json(
        h,
        &format!(
            r##"(() => {{
                const status = document.querySelector('[data-testid="{instance}-status"]');
                const root = document.querySelector('{dialog}');
                const dialogEl = root ? root.closest('dialog') : null;
                const confirm = document.querySelector(
                    '{dialog} [data-confirmable-search-picker-confirm="true"]'
                );
                const summary = document.querySelector(
                    '{dialog} [data-confirmable-search-picker-summary="true"]'
                );
                const hint = document.querySelector(
                    '{dialog} [data-confirmable-search-picker-confirm-hint="true"]'
                );
                const failure = document.querySelector(
                    '{dialog} [data-confirmable-search-picker-error="true"]'
                );
                const rows = Array.from(
                    document.querySelectorAll('{dialog} [data-result-key]')
                ).map(el => el.dataset.resultKey);
                const selectedRows = Array.from(
                    document.querySelectorAll('{dialog} [data-result-key][aria-selected="true"]')
                ).map(el => el.dataset.resultKey);
                const active = document.activeElement;
                return {{
                    statusText: status ? status.textContent.replace(/\s+/g, ' ').trim() : null,
                    dialogOpen: dialogEl ? dialogEl.open : null,
                    confirmState: confirm ? confirm.getAttribute('data-confirm-state') : null,
                    confirmAriaDisabled: confirm ? confirm.getAttribute('aria-disabled') : null,
                    confirmNativelyDisabled: confirm ? confirm.disabled : null,
                    confirmFocusable: confirm ? !confirm.matches(':disabled') : null,
                    confirmLabel: confirm ? confirm.textContent.trim() : null,
                    confirmDescribedBy: confirm ? confirm.getAttribute('aria-describedby') : null,
                    hintId: hint ? hint.id : null,
                    hintText: hint ? hint.textContent.trim() : null,
                    selectionState: summary ? summary.getAttribute('data-selection-state') : null,
                    selectedKey: summary ? summary.getAttribute('data-selected-key') : null,
                    summaryText: summary ? summary.textContent.replace(/\s+/g, ' ').trim() : null,
                    failureText: failure ? failure.textContent.trim() : null,
                    rowKeys: rows,
                    selectedRowKeys: selectedRows,
                    activeIsSearchInput: active
                        ? active.matches('{dialog} [data-confirmable-search-picker-search="true"]')
                        : false,
                    activeIsTrigger: active
                        ? active === document.querySelector('[data-testid="{instance}-trigger"]')
                        : false,
                    activeIsConfirm: active ? active === confirm : false,
                }};
            }})()"##
        ),
    )
    .await
}

/// Activates one of the dialog's own controls by its stable data marker.
async fn click_confirmable(h: &pixelproof_web::Harness, instance: &str, marker: &str) {
    let dialog = confirmable_selector(instance);
    let script = format!(r#"document.querySelector('{dialog} [{marker}]').click()"#);
    let _ = h.page().evaluate(script.as_str()).await;
    settle(120).await;
}

async fn click_result_row(h: &pixelproof_web::Harness, instance: &str, key: &str) {
    let dialog = confirmable_selector(instance);
    let script = format!(r#"document.querySelector('{dialog} [data-result-key="{key}"]').click()"#);
    let _ = h.page().evaluate(script.as_str()).await;
    settle(120).await;
}

const CONFIRM_MARKER: &str = r#"data-confirmable-search-picker-confirm="true""#;
const CANCEL_MARKER: &str = r#"data-confirmable-search-picker-cancel="true""#;

/// The bead's central invariant: pointer *and* keyboard activation of a
/// result update only the selected presentation. The fixture's confirmation
/// counter -- the only place a side effect is recorded -- must still read
/// zero afterwards.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn selecting_a_result_never_confirms() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "confirm-x").await;
    let opened = confirmable_snapshot(&h, "confirm-x").await;
    assert_eq!(opened["dialogOpen"], json!(true), "{opened}");
    assert_eq!(
        opened["activeIsSearchInput"],
        json!(true),
        "opening focuses the search field: {opened}"
    );
    assert_eq!(
        opened["selectionState"],
        json!("none"),
        "nothing is selected yet: {opened}"
    );

    click_result_row(&h, "confirm-x", "worker-b").await;
    let clicked = confirmable_snapshot(&h, "confirm-x").await;
    assert_eq!(
        clicked["selectionState"],
        json!("resolved"),
        "a click selects: {clicked}"
    );
    assert_eq!(clicked["selectedKey"], json!("worker-b"), "{clicked}");
    assert_eq!(
        clicked["selectedRowKeys"],
        json!(["worker-b"]),
        "exactly the clicked row reports aria-selected: {clicked}"
    );
    assert_eq!(
        clicked["dialogOpen"],
        json!(true),
        "selecting must not close the dialog: {clicked}"
    );
    assert!(
        clicked["statusText"]
            .as_str()
            .unwrap_or_default()
            .contains("Confirmations: 0"),
        "a click must not perform the write: {clicked}"
    );

    // Keyboard: Arrow moves the selection, Enter must not confirm.
    h.press_key_sequence(&[Key::ArrowDown])
        .await
        .expect("ArrowDown");
    h.press_key_sequence(&[Key::Enter]).await.expect("Enter");
    settle(250).await;
    let keyboard = confirmable_snapshot(&h, "confirm-x").await;
    assert_eq!(
        keyboard["selectedKey"],
        json!("worker-c"),
        "Arrow keys forwarded from the search field move the selection: {keyboard}"
    );
    assert_eq!(
        keyboard["activeIsSearchInput"],
        json!(true),
        "keyboard navigation never steals focus out of the search field: {keyboard}"
    );
    assert!(
        keyboard["statusText"]
            .as_str()
            .unwrap_or_default()
            .contains("Confirmations: 0"),
        "Enter must not confirm -- confirming is a separate control: {keyboard}"
    );

    assert_no_browser_errors(&h, "confirmable picker selection-is-not-a-write").await;
}

/// Confirm returns the typed item behind the selected key -- `worker-b`
/// (W-200), not the identically titled `worker-a` -- and fires exactly once
/// even when clicked repeatedly, because the in-flight state blocks the
/// second activation in the handler, not merely in the presentation.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn confirm_submits_the_selected_typed_item_exactly_once() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "confirm-x").await;
    click_result_row(&h, "confirm-x", "worker-b").await;

    let ready = confirmable_snapshot(&h, "confirm-x").await;
    assert_eq!(
        ready["confirmState"],
        json!("ready"),
        "a resolved selection unblocks confirmation: {ready}"
    );
    assert_eq!(
        ready["confirmAriaDisabled"],
        json!(null),
        "no aria-disabled once confirmation is available: {ready}"
    );
    assert_eq!(
        ready["hintText"],
        json!(""),
        "the description carries text only when it has something to explain: {ready}"
    );

    // Two activations inside the fixture's 250ms flight window.
    let dialog = confirmable_selector("confirm-x");
    let script = format!(
        r#"(() => {{
            const confirm = document.querySelector(
                '{dialog} [data-confirmable-search-picker-confirm="true"]'
            );
            confirm.click();
            confirm.click();
        }})()"#
    );
    let _ = h.page().evaluate(script.as_str()).await;
    settle(60).await;

    let inflight = confirmable_snapshot(&h, "confirm-x").await;
    assert_eq!(
        inflight["confirmState"],
        json!("pending"),
        "a confirmation in flight blocks another: {inflight}"
    );
    assert_eq!(
        inflight["confirmAriaDisabled"],
        json!("true"),
        "blocked confirmation reports aria-disabled: {inflight}"
    );
    assert_eq!(
        inflight["confirmNativelyDisabled"],
        json!(false),
        "never the native disabled attribute -- the control must stay in the a11y tree: {inflight}"
    );
    assert_eq!(
        inflight["dialogOpen"],
        json!(true),
        "the dialog does not close optimistically on confirm: {inflight}"
    );

    settle(600).await;
    let settled = confirmable_snapshot(&h, "confirm-x").await;
    let status = settled["statusText"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(
        status.contains("Confirmations: 1"),
        "confirm fires exactly once, never twice: {settled}"
    );
    assert!(
        status.contains("worker-b (W-200)"),
        "confirm submits the payload behind the selected key, not the duplicate title: {settled}"
    );

    assert_no_browser_errors(&h, "confirmable picker single confirmation").await;
}

/// With nothing selected, Confirm is genuinely inert *and* still reachable:
/// `aria-disabled`, never the native attribute, so its explanation stays in
/// the accessibility tree and the tab order. Activating it anyway performs
/// no write.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn confirm_is_blocked_and_explained_with_no_selection() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "confirm-y").await;
    let blocked = confirmable_snapshot(&h, "confirm-y").await;
    assert_eq!(
        blocked["confirmState"],
        json!("blocked-no-selection"),
        "{blocked}"
    );
    assert_eq!(blocked["confirmAriaDisabled"], json!("true"), "{blocked}");
    assert_eq!(
        blocked["confirmNativelyDisabled"],
        json!(false),
        "native disabled would remove the reason from the a11y tree: {blocked}"
    );
    assert_eq!(
        blocked["confirmFocusable"],
        json!(true),
        "the control stays focusable so its reason is reachable: {blocked}"
    );
    assert_eq!(
        blocked["confirmDescribedBy"], blocked["hintId"],
        "aria-describedby points at the reason element: {blocked}"
    );
    assert_eq!(
        blocked["hintText"],
        json!("Select a result to continue."),
        "the reason comes from the Texts struct: {blocked}"
    );

    click_confirmable(&h, "confirm-y", CONFIRM_MARKER).await;
    settle(500).await;
    let after = confirmable_snapshot(&h, "confirm-y").await;
    assert!(
        after["statusText"]
            .as_str()
            .unwrap_or_default()
            .contains("Confirmations: 0"),
        "activating a blocked confirm must fail closed: {after}"
    );

    assert_no_browser_errors(&h, "confirmable picker blocked confirmation").await;
}

/// Search narrows the list past the selected row. The selection must survive
/// that: still named in the summary, still confirmable, and confirming still
/// returns its own typed payload rather than one of the visible rows'.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn a_selection_narrowed_out_of_the_results_stays_named_and_confirmable() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "confirm-x").await;
    click_result_row(&h, "confirm-x", "worker-b").await;

    type_confirmable_query(&h, "confirm-x", "Priya").await;
    settle(250).await;
    let narrowed = confirmable_snapshot(&h, "confirm-x").await;
    assert_eq!(
        narrowed["rowKeys"],
        json!(["worker-c"]),
        "the selected row is no longer in the results: {narrowed}"
    );
    assert_eq!(
        narrowed["selectedRowKeys"],
        json!([]),
        "and no visible row is falsely highlighted: {narrowed}"
    );
    assert_eq!(
        narrowed["selectionState"],
        json!("resolved"),
        "yet the selection is still resolvable: {narrowed}"
    );
    assert_eq!(narrowed["selectedKey"], json!("worker-b"), "{narrowed}");
    assert!(
        narrowed["summaryText"]
            .as_str()
            .unwrap_or_default()
            .contains("Alex Morgan"),
        "and still visibly named: {narrowed}"
    );
    assert_eq!(
        narrowed["confirmState"],
        json!("ready"),
        "and still confirmable: {narrowed}"
    );

    click_confirmable(&h, "confirm-x", CONFIRM_MARKER).await;
    settle(600).await;
    let confirmed = confirmable_snapshot(&h, "confirm-x").await;
    assert!(
        confirmed["statusText"]
            .as_str()
            .unwrap_or_default()
            .contains("worker-b (W-200)"),
        "the retained selection confirms its own payload: {confirmed}"
    );

    assert_no_browser_errors(&h, "confirmable picker retained selection").await;
}

/// Escape and Cancel close without confirming and restore focus to the
/// trigger. Neither discards the selection the user made: the pattern never
/// proposes clearing the caller's key on dismissal, so reopening shows it
/// again.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn dismissal_never_confirms_and_never_discards_the_selection() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "confirm-y").await;
    click_result_row(&h, "confirm-y", "worker-d").await;

    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    settle(250).await;
    let escaped = confirmable_snapshot(&h, "confirm-y").await;
    assert_eq!(escaped["dialogOpen"], json!(false), "{escaped}");
    assert_eq!(
        escaped["activeIsTrigger"],
        json!(true),
        "focus returns to the trigger through the dialog's own close(): {escaped}"
    );
    assert!(
        escaped["statusText"]
            .as_str()
            .unwrap_or_default()
            .contains("Confirmations: 0"),
        "Escape must never confirm: {escaped}"
    );

    open_dialog(&h, "confirm-y").await;
    let reopened = confirmable_snapshot(&h, "confirm-y").await;
    assert_eq!(
        reopened["selectedKey"],
        json!("worker-d"),
        "the selection survived dismissal: {reopened}"
    );
    assert_eq!(reopened["selectionState"], json!("resolved"), "{reopened}");

    click_confirmable(&h, "confirm-y", CANCEL_MARKER).await;
    settle(250).await;
    let cancelled = confirmable_snapshot(&h, "confirm-y").await;
    assert_eq!(cancelled["dialogOpen"], json!(false), "{cancelled}");
    assert_eq!(cancelled["activeIsTrigger"], json!(true), "{cancelled}");
    assert!(
        cancelled["statusText"]
            .as_str()
            .unwrap_or_default()
            .contains("Confirmations: 0"),
        "Cancel must never confirm: {cancelled}"
    );

    assert_no_browser_errors(&h, "confirmable picker dismissal").await;
}

/// A failed confirmation must not cost the user their context: the dialog is
/// still open, the selection is still made, the failure is announced, and
/// Confirm is available again.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn a_failed_confirmation_keeps_the_dialog_open_with_its_selection() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let _ = h
        .page()
        .evaluate(r#"document.querySelector('[data-testid="confirm-y-toggle-failure"]').click()"#)
        .await;
    settle(120).await;

    open_dialog(&h, "confirm-y").await;
    click_result_row(&h, "confirm-y", "worker-e").await;
    click_confirmable(&h, "confirm-y", CONFIRM_MARKER).await;
    settle(700).await;

    let failed = confirmable_snapshot(&h, "confirm-y").await;
    assert_eq!(
        failed["dialogOpen"],
        json!(true),
        "a failed write must not have closed the dialog: {failed}"
    );
    assert_eq!(
        failed["failureText"],
        json!("The assignment could not be saved."),
        "the failure is rendered in the dialog: {failed}"
    );
    assert_eq!(
        failed["selectedKey"],
        json!("worker-e"),
        "the selection is intact: {failed}"
    );
    assert_eq!(
        failed["confirmState"],
        json!("ready"),
        "and Confirm is available to retry: {failed}"
    );
    assert!(
        failed["statusText"]
            .as_str()
            .unwrap_or_default()
            .contains("Confirmations: 0"),
        "nothing was written: {failed}"
    );

    assert_no_browser_errors(&h, "confirmable picker failed confirmation").await;
}

/// Every user-visible string comes from the reactive Texts struct: switching
/// the fixture's locale relabels the chrome without touching the stable
/// result keys or the selected identity.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn reactive_localized_copy_preserves_keys_and_selected_identity() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    open_dialog(&h, "confirm-x").await;
    click_result_row(&h, "confirm-x", "worker-c").await;
    let english = confirmable_snapshot(&h, "confirm-x").await;
    assert_eq!(english["confirmLabel"], json!("Confirm"), "{english}");

    let _ = h
        .page()
        .evaluate(r#"document.querySelector('[data-testid="confirm-x-toggle-locale"]').click()"#)
        .await;
    settle(250).await;

    let spanish = confirmable_snapshot(&h, "confirm-x").await;
    assert_eq!(spanish["confirmLabel"], json!("Asignar"), "{spanish}");
    assert!(
        spanish["summaryText"]
            .as_str()
            .unwrap_or_default()
            .contains("Seleccionado"),
        "the summary label is localized too: {spanish}"
    );
    assert_eq!(
        spanish["selectedKey"], english["selectedKey"],
        "localization must not disturb the selected identity: {spanish}"
    );
    assert_eq!(
        spanish["rowKeys"], english["rowKeys"],
        "nor the stable result keys: {spanish}"
    );

    assert_no_browser_errors(&h, "confirmable picker localized copy").await;
}

/// Two simultaneous instances derive every id and name from their own
/// contract id, so nothing collides -- the failure mode that makes
/// `aria-labelledby` and `aria-describedby` point at the wrong dialog.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-search-picker-dialog)"]
async fn two_confirmable_instances_derive_distinct_ids() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let ids = eval_json(
        &h,
        r#"(() => {
            const roots = Array.from(
                document.querySelectorAll('[data-confirmable-search-picker-dialog="true"]')
            );
            const collect = (root) => {
                const dialogEl = root.closest('dialog');
                return {
                    controlId: root.getAttribute('data-control-id'),
                    labelledBy: dialogEl.getAttribute('aria-labelledby'),
                    describedBy: dialogEl.getAttribute('aria-describedby'),
                    summaryId: root.querySelector(
                        '[data-confirmable-search-picker-summary="true"]'
                    ).id,
                    hintId: root.querySelector(
                        '[data-confirmable-search-picker-confirm-hint="true"]'
                    ).id,
                    searchName: root.querySelector(
                        '[data-confirmable-search-picker-search="true"]'
                    ).getAttribute('name'),
                };
            };
            const all = roots.map(collect);
            const flat = all.flatMap(entry => Object.values(entry));
            const summaryIdElements = all
                .map(entry => document.querySelectorAll(
                    '[id="' + entry.summaryId + '"]'
                ).length)
                .reduce((a, b) => a + b, 0);
            return {
                count: roots.length,
                entries: all,
                uniqueValues: new Set(flat).size,
                totalValues: flat.length,
                summaryIdElements: summaryIdElements,
            };
        })()"#,
    )
    .await;

    assert_eq!(
        ids["count"],
        json!(2),
        "the fixture renders two simultaneous instances: {ids}"
    );
    assert_eq!(
        ids["uniqueValues"], ids["totalValues"],
        "no id or name is shared across the two instances: {ids}"
    );
    assert_eq!(
        ids["summaryIdElements"],
        json!(2),
        "each summary id matches exactly one element: {ids}"
    );

    assert_no_browser_errors(&h, "confirmable picker id uniqueness").await;
}
