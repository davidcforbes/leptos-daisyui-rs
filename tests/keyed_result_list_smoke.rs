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
/// (both its raw id and the target's own `data-result-key`), the listbox's
/// live `role="option"` key order, and whether the empty-state fallback or
/// key-validation error banner is currently showing.
///
/// The single source of truth for reading `aria-activedescendant`: `?? null`
/// (never a sentinel string) so an absent attribute round-trips as JSON
/// `null`, matching `Option::None` on the Rust side. Every test below reads
/// listbox state through this helper rather than a bespoke `eval_json` block,
/// so this null-handling only has to be right once.
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
                activeDescendantId: activeId,
                activeDescendantKey: activeEl?.dataset.resultKey ?? null,
                optionKeys: options.map(o => o.dataset.resultKey),
                selectedKeys: options
                    .filter(o => o.getAttribute('aria-selected') === 'true')
                    .map(o => o.dataset.resultKey),
                emptyStateShown: !!root?.querySelector('[role="presentation"]'),
                errorBanner: !!root?.querySelector('[data-result-list-key-error]'),
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

// ── Controlled selection (ldui-bf8c) ──
//
// A separate root/fixture (`#keyed-result-list-controlled`) so this section
// never shares state with the uncontrolled fixture above. Every gesture here
// is either an external button (never touches the list) or a click/keyboard
// action on the list itself, which this fixture's `on_change` callback
// applies to its own `RwSignal` — proving the proposal shape works without
// asserting anything about the demo's specific choice to always apply it.

const CONTROLLED_ROOT: &str = "#keyed-result-list-controlled [role=\"listbox\"]";

/// Same shape as [`state`], scoped to the controlled fixture, plus the
/// controlled status banner's own text (accepted key / last proposal /
/// activation).
async fn controlled_state(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const root = document.querySelector('#keyed-result-list-controlled [role="listbox"]');
            const status = document.querySelector('[data-testid="keyed-result-list-controlled-status"]');
            const activeId = root?.getAttribute('aria-activedescendant') ?? null;
            const activeEl = activeId ? document.getElementById(activeId) : null;
            const options = Array.from(root?.querySelectorAll('[role="option"]') ?? []);
            return {
                statusText: status?.textContent.trim() ?? null,
                activeDescendantId: activeId,
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

fn controlled_row_selector(key: &str) -> String {
    format!("#keyed-result-list-controlled [data-result-key=\"{key}\"]")
}

/// Restores only the controlled fixture's item set — deliberately leaves the
/// accepted key untouched, matching the button's own documented behavior.
async fn restore_controlled_items(h: &pixelproof_web::Harness) {
    click(h, "[data-testid=\"keyed-result-list-controlled-restore\"]").await;
}

async fn select_controlled_case_b(h: &pixelproof_web::Harness) {
    click(
        h,
        "[data-testid=\"keyed-result-list-controlled-select-case-b\"]",
    )
    .await;
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
    let s = state(&h).await;
    assert_eq!(s["optionKeys"], json!([]), "no option rows remain: {s}");
    assert_eq!(
        s["emptyStateShown"],
        json!(true),
        "empty-state fallback shown: {s}"
    );
    assert_eq!(
        s["errorBanner"],
        json!(false),
        "no key-validation error: {s}"
    );
    assert_eq!(
        s["activeDescendantId"],
        Value::Null,
        "aria-activedescendant is cleared, not pointing at a removed row: {s}"
    );

    restore_fixture(&h).await;
    assert_no_browser_errors(&h, "empty-state journey").await;
}

/// External changes to the caller's accepted-key signal are authoritative:
/// clicking a page button that never touches the list still moves the
/// rendered highlight, `aria-selected`, and `aria-activedescendant`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn controlled_external_selection_is_authoritative() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_controlled_items(&h).await;
    select_controlled_case_b(&h).await;

    let before = controlled_state(&h).await;
    assert_eq!(
        before["selectedKeys"],
        json!(["case-b"]),
        "the fixture's initial accepted key is case-b: {before}"
    );

    click(
        &h,
        "[data-testid=\"keyed-result-list-controlled-select-case-c\"]",
    )
    .await;
    let after = controlled_state(&h).await;
    assert_eq!(
        after["selectedKeys"],
        json!(["case-c"]),
        "an external button (never a row click) moved the highlight: {after}"
    );
    assert_eq!(
        after["activeDescendantKey"],
        json!("case-c"),
        "aria-activedescendant follows the externally accepted key: {after}"
    );
    assert!(
        after["statusText"]
            .as_str()
            .expect("status text")
            .contains("Accepted key: case-c"),
        "the caller's own signal is what changed: {after}"
    );

    assert_no_browser_errors(&h, "controlled external selection journey").await;
}

/// A controlled key that names no current row renders no false highlight and
/// never mutates the caller's accepted key; when a matching row reappears
/// the highlight (and its scroll target) is restored automatically.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn controlled_key_absent_from_items_renders_no_highlight_and_restores() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_controlled_items(&h).await;
    select_controlled_case_b(&h).await;

    click(
        &h,
        "[data-testid=\"keyed-result-list-controlled-filter-out-b\"]",
    )
    .await;
    let filtered = controlled_state(&h).await;
    assert_eq!(
        filtered["selectedKeys"],
        json!([]),
        "no row falsely renders as selected once case-b is filtered out: {filtered}"
    );
    assert_eq!(
        filtered["activeDescendantId"],
        Value::Null,
        "aria-activedescendant is absent, not pointing at a removed row: {filtered}"
    );
    assert!(
        filtered["statusText"]
            .as_str()
            .expect("status text")
            .contains("Accepted key: case-b"),
        "the accepted key itself is never silently overwritten just because \
         its row disappeared: {filtered}"
    );

    restore_controlled_items(&h).await;
    let restored = controlled_state(&h).await;
    assert_eq!(
        restored["selectedKeys"],
        json!(["case-b"]),
        "the highlight resumes automatically once a matching row reappears, \
         with no button re-asserting the key: {restored}"
    );
    assert_eq!(
        restored["activeDescendantKey"],
        json!("case-b"),
        "{restored}"
    );

    assert_no_browser_errors(&h, "controlled absent-then-restored key journey").await;
}

/// Clicking a row in the controlled list proposes a change (this fixture
/// applies every proposal) rather than the list deciding locally; activation
/// (`on_select`) still fires exactly as in the uncontrolled list.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn controlled_click_proposes_a_change_and_still_activates() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_controlled_items(&h).await;
    select_controlled_case_b(&h).await;

    click(&h, &controlled_row_selector("case-a")).await;
    let s = controlled_state(&h).await;
    assert_eq!(
        s["selectedKeys"],
        json!(["case-a"]),
        "the applied proposal is what moved the highlight: {s}"
    );
    let text = s["statusText"].as_str().expect("status text").to_owned();
    assert!(
        text.contains("Accepted key: case-a") && text.contains("case-a (click)"),
        "the proposal names its cause: {text}"
    );
    assert!(
        text.contains("Activated: case-a (A-100)"),
        "activation still fires on click in the controlled configuration: {text}"
    );

    assert_no_browser_errors(&h, "controlled click proposal journey").await;
}

/// Keyboard navigation in the controlled list also proposes rather than
/// diverging locally, and `aria-activedescendant` follows the applied
/// proposal exactly as it does in the uncontrolled list.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-keyed-result-list)"]
async fn controlled_keyboard_navigation_proposes_and_stays_coherent() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    restore_controlled_items(&h).await;
    select_controlled_case_b(&h).await;

    h.page()
        .find_element(CONTROLLED_ROOT)
        .await
        .expect("find controlled listbox")
        .focus()
        .await
        .expect("focus controlled listbox");

    h.press_key_sequence(&[Key::ArrowDown])
        .await
        .expect("ArrowDown");
    let s = controlled_state(&h).await;
    assert_eq!(
        s["activeDescendantKey"],
        json!("case-c"),
        "ArrowDown from the accepted case-b proposes and applies case-c: {s}"
    );
    assert!(
        s["statusText"]
            .as_str()
            .expect("status text")
            .contains("case-c (keyboard)"),
        "the proposal names Keyboard as its cause: {s}"
    );

    assert_no_browser_errors(&h, "controlled keyboard navigation journey").await;
}
