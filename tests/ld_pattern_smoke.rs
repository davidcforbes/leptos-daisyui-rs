//! Real-DOM proof for ldui-wra5 on the general demo app: every
//! pixelproof-parity catalogue pattern the demo renders carries
//! `data-ld-pattern="<Pattern>"` on its ROOT element, nothing else carries
//! it, and every root an exclusive hook identifies declares it.
//!
//! `src/patterns/ld_pattern_tests.rs` reads the component source because
//! the library has no native renderer; this reads what Leptos rendered. The
//! pages are in `ld_pattern/mod.rs` (`PAGES`); `SnapshotTablePage` lives on
//! the client-snapshot test host, so `ld_pattern_snapshot_smoke.rs` proves
//! it and the dedicated-row `PageHeader`. `cargo xtask test-ld-pattern` runs
//! both files.
//!
//! `PageStatePanel` only renders mid-interaction, so it gets a journey: the
//! search-picker dialog's empty search (the panel's status root, a
//! `<section>`) and a failed refresh with retained rows (its error root, a
//! `<div>`) -- one proof per root the component can render.
mod common;
mod ld_pattern;

use common::{assert_no_browser_errors, begin_browser_error_capture, harness_at};
use ld_pattern::{Host, assert_roots_consistent, check_host, probe};
use serde_json::{Value, json};
use std::time::Duration;

const PICKER: &str = "/components/search_picker_dialog";
const DIALOG: &str = r#"[data-testid="dialog-a-fixture"] [data-search-picker-dialog="true"]"#;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ld-pattern)"]
async fn demo_pattern_roots_declare_data_ld_pattern() {
    check_host(Host::Demo).await;
}

async fn run(h: &pixelproof_web::Harness, script: &str) {
    let _ = h.page().evaluate(script).await;
}

async fn settle(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await;
}

async fn search(h: &pixelproof_web::Harness, text: &str) {
    run(
        h,
        &format!(
            r#"(() => {{
                const input = document.querySelector('{DIALOG} input[type="search"]');
                input.value = {text:?};
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
            }})()"#
        ),
    )
    .await;
    // The fixture's slowest simulated response is 500ms.
    settle(600).await;
}

/// The declared `PageStatePanel`s, after proving every declaration on the
/// page sits on its root.
async fn panels(h: &pixelproof_web::Harness, context: &str) -> Value {
    let report = probe(h).await;
    assert_roots_consistent(&report, context);
    report["panels"].clone()
}

/// Whether a declared panel of `tag` shows the `slug` state. The page has
/// several dialog instances, so the probe's list is page-wide.
fn has_panel(panels: &Value, tag: &str, slug: &str) -> bool {
    panels
        .as_array()
        .is_some_and(|all| all.contains(&json!({ "tag": tag, "slug": slug })))
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-ld-pattern)"]
async fn page_state_panel_declares_on_both_of_its_roots() {
    let h = harness_at(PICKER).await;
    begin_browser_error_capture(&h).await;

    run(
        &h,
        r#"(() => {
            const trigger = document.querySelector('[data-testid="dialog-a-trigger"]');
            trigger.focus();
            trigger.click();
        })()"#,
    )
    .await;
    settle(200).await;

    search(&h, "zzz-no-match").await;
    let status = panels(&h, "empty search").await;
    assert!(
        has_panel(&status, "section", "empty-dataset"),
        "an empty search renders PageStatePanel's status root, declared: {status}"
    );

    search(&h, "Alex").await;
    run(
        &h,
        r#"document.querySelector('[data-testid="dialog-a-force-error"]').click()"#,
    )
    .await;
    settle(200).await;
    let error = panels(&h, "failed refresh").await;
    assert!(
        has_panel(&error, "div", "retained-error"),
        "a failed refresh renders PageStatePanel's error root, declared: {error}"
    );

    assert_no_browser_errors(&h, "ld-pattern PageStatePanel journey").await;
}
