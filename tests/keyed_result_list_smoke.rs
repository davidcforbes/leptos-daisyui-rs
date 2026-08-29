//! Real-browser proof for `KeyedResultList` (ldui-r1z, resumed Task 3):
//! selection and activation must resolve by stable key against the *current*
//! `items`, never by index or display text, even when the result set is
//! replaced with duplicate-looking labels, a reorder, an insertion, a
//! removal, or a relabel.
//!
//! Drives the general demo app (`html_target: None`, like
//! `reactivity_smoke.rs`) rather than a dedicated test-host page, because the
//! fixture lives on the existing `/components/result-list` showcase route.
//! Kept in its own file/xtask step (`cargo xtask test-keyed-result-list`)
//! rather than folded into `reactivity_smoke.rs`, whose 32-check count is
//! pinned (`xtask::tests::reactivity_lane_has_exactly_32_browser_checks`).

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use pixelproof_web::Key;
use serde_json::{Value, json};

const PAGE: &str = "/components/result-list";
const ROOT: &str = "#keyed-result-list [role=\"listbox\"]";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate keyed-result-list fixture")
        .into_value()
        .expect("keyed-result-list expression returns JSON")
}

/// Text shown in the status banner, the root's `aria-activedescendant`
/// target's own `data-result-key`, and the listbox's live `role="option"`
/// key order.
async fn state(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const root = document.querySelector('#keyed-result-list [role="listbox"]');
            const status = document.querySelector('[data-testid="keyed-result-list-status"]');
            const activeId = root?.getAttribute('aria-activedescendant') ?? null;
            const activeEl = activeId ? document.getElementById(activeId) : null;
            const options = Array.from(root?.querySelectorAll('[role="option"]') ?? []);
            return {
                statusText: status?.textContent.trim() ?? null,
                activeDescendantKey: activeEl?.dataset.resultKey ?? null,
                optionKeys: options.map(o => o.dataset.resultKey),
                selectedKeys: options
                    .filter(o => o.getAttribute('aria-selected') === 'true')
                    .map(o => o.dataset.resultKey),
            };
        })()"#,
    )
    .await
}

fn row_selector(key: &str) -> String {
    format!("#keyed-result-list [data-result-key=\"{key}\"]")
}

async fn restore_fixture(h: &pixelproof_web::Harness) {
    click(h, "[data-testid=\"keyed-result-list-restore\"]").await;
}

/// Two rows (`case-a`, `case-b`) intentionally render the identical display
/// title "Alex Morgan". Clicking each must activate its own distinct
/// payload, never the other's, and never fall back to matching by title.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn duplicate_labels_activate_their_own_distinct_payload() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_fixture(&h).await;

    click(&h, &row_selector("case-b")).await;
    let s = state(&h).await;
    assert_eq!(s["selectedKeys"], json!(["case-b"]), "case-b selected: {s}");
    assert_eq!(
        s["statusText"],
        json!("Highlighted key: case-b | Activated: case-b (B-200)"),
        "clicking case-b both selects and activates it: {s}"
    );

    click(&h, &row_selector("case-a")).await;
    let s = state(&h).await;
    assert_eq!(s["selectedKeys"], json!(["case-a"]), "case-a selected: {s}");
    assert_eq!(
        s["statusText"],
        json!("Highlighted key: case-a | Activated: case-a (A-100)"),
        "an identical title on case-b must never leak into case-a's activation: {s}"
    );

    assert_no_browser_errors(&h, "duplicate-label activation").await;
}

/// A reorder changes every row's index but not its key: the selection
/// (arrived at by real keyboard focus, mirroring a person's journey) must
/// still resolve to the same identity after the replacement.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn reorder_preserves_the_selected_identity() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_fixture(&h).await;

    click(&h, &row_selector("case-b")).await;
    let before = state(&h).await;
    assert_eq!(
        before["selectedKeys"],
        json!(["case-b"]),
        "before: {before}"
    );

    click(&h, "[data-testid=\"keyed-result-list-reorder\"]").await;
    let after = state(&h).await;
    assert_eq!(
        after["selectedKeys"],
        json!(["case-b"]),
        "reorder must not move the selection off its key: {after}"
    );
    assert_ne!(
        after["optionKeys"], before["optionKeys"],
        "the fixture reversal must actually change row order: {after}"
    );

    assert_no_browser_errors(&h, "reorder journey").await;
}

/// Removing the selected result falls back to the new first result (never to
/// a stale payload) and fires no accidental activation.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn removing_the_selected_result_falls_back_to_first_remaining() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_fixture(&h).await;

    click(&h, &row_selector("case-a")).await;
    let before = state(&h).await;
    assert_eq!(
        before["selectedKeys"],
        json!(["case-a"]),
        "before: {before}"
    );

    click(&h, "[data-testid=\"keyed-result-list-remove\"]").await;
    let after = state(&h).await;
    assert_eq!(
        after["selectedKeys"],
        json!(["case-b"]),
        "case-a's removal falls back to the new first row (case-b), not a stale case-a: {after}"
    );
    assert!(
        !after["optionKeys"]
            .as_array()
            .expect("option keys")
            .iter()
            .any(|key| key == "case-a"),
        "case-a must actually be gone: {after}"
    );

    assert_no_browser_errors(&h, "removal journey").await;
}

/// An asynchronous replacement that only *inserts* a new top result must not
/// disturb an existing selection by index.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn inserting_a_new_result_preserves_the_selected_identity() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_fixture(&h).await;

    click(&h, &row_selector("case-b")).await;
    click(&h, "[data-testid=\"keyed-result-list-insert\"]").await;
    let s = state(&h).await;
    assert_eq!(
        s["selectedKeys"],
        json!(["case-b"]),
        "an inserted row ahead of the selection must not shift it by index: {s}"
    );
    assert_eq!(
        s["optionKeys"].as_array().map(std::vec::Vec::len),
        Some(4),
        "the new row is present: {s}"
    );

    assert_no_browser_errors(&h, "insertion journey").await;
}

/// Relabeling the selected row's display text (same key, new title) must
/// keep the selection on that key.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn relabeling_the_selected_result_preserves_its_key() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_fixture(&h).await;

    click(&h, &row_selector("case-a")).await;
    click(&h, "[data-testid=\"keyed-result-list-relabel\"]").await;
    let s = state(&h).await;
    assert_eq!(
        s["selectedKeys"],
        json!(["case-a"]),
        "a relabel must not desync the selection from its key: {s}"
    );
    let title = eval_json(
        &h,
        &format!(
            "document.querySelector({:?})?.textContent.trim() ?? null",
            format!("{} span", row_selector("case-a"))
        ),
    )
    .await;
    assert_eq!(
        title,
        json!("Alexandra Morgan"),
        "the new label reached the DOM under the same key: {s}"
    );

    assert_no_browser_errors(&h, "relabel journey").await;
}

/// Arrow/Home/End/Enter keyboard navigation and `aria-activedescendant`
/// stay coherent on the keyed listbox, matching the legacy `ResultList`
/// contract.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn keyboard_navigation_and_activedescendant_stay_coherent() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_fixture(&h).await;

    h.page()
        .find_element(ROOT)
        .await
        .expect("find keyed listbox")
        .focus()
        .await
        .expect("focus keyed listbox");

    // The mount-time reconciliation effect already selects the first row
    // (case-a), matching ResultList's own reset-to-first behavior.
    let initial = state(&h).await;
    assert_eq!(
        initial["activeDescendantKey"],
        json!("case-a"),
        "mount selects the first row: {initial}"
    );

    h.press_key_sequence(&[Key::ArrowDown])
        .await
        .expect("ArrowDown");
    let s = state(&h).await;
    assert_eq!(
        s["activeDescendantKey"],
        json!("case-b"),
        "ArrowDown moves off the already-selected first row to the second: {s}"
    );

    h.press_key_sequence(&[Key::End]).await.expect("End");
    let s = state(&h).await;
    assert_eq!(s["activeDescendantKey"], json!("case-c"), "End: {s}");

    h.press_key_sequence(&[Key::Home]).await.expect("Home");
    let s = state(&h).await;
    assert_eq!(s["activeDescendantKey"], json!("case-a"), "Home: {s}");

    h.press_key_sequence(&[Key::Enter]).await.expect("Enter");
    let s = state(&h).await;
    assert_eq!(
        s["statusText"],
        json!("Highlighted key: case-a | Activated: case-a (A-100)"),
        "Enter activates the highlighted row: {s}"
    );

    assert_no_browser_errors(&h, "keyboard navigation journey").await;
}

/// An empty `KeyedResultList` (no rows to select) renders the presentation
/// empty-state fallback, not an error banner or a stray listbox option.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn empty_result_set_renders_the_empty_state() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_fixture(&h).await;

    click(&h, &row_selector("case-a")).await;
    click(&h, "[data-testid=\"keyed-result-list-clear\"]").await;
    let s = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('#keyed-result-list [role="listbox"]');
            return {
                options: root?.querySelectorAll('[role="option"]').length ?? -1,
                empty: !!root?.querySelector('[role="presentation"]'),
                errorBanner: !!root?.querySelector('[data-result-list-key-error]'),
                activeDescendant: root?.getAttribute('aria-activedescendant') ?? 'present',
            };
        })()"#,
    )
    .await;
    assert_eq!(s["options"], json!(0), "no option rows remain: {s}");
    assert_eq!(s["empty"], json!(true), "empty-state fallback shown: {s}");
    assert_eq!(
        s["errorBanner"],
        json!(false),
        "no key-validation error: {s}"
    );
    assert_eq!(
        s["activeDescendant"],
        Value::Null,
        "aria-activedescendant is cleared, not pointing at a removed row: {s}"
    );

    restore_fixture(&h).await;
    assert_no_browser_errors(&h, "empty-state journey").await;
}
