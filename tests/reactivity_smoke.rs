//! Reactivity smoke suite (ldui-49w.1) — drives real CDP input against the
//! demo app and asserts internal Leptos state through the
//! `window.__APP_DEBUG__` oracle (ldui-49w.3), not through pixels.
//!
//! No screenshots, so this suite is deterministic across machines and is
//! **auto-gated** — it runs as the `test-reactivity` step of
//! `cargo xtask verify-full`. (Its sibling `visual_smoke.rs` compares pixels
//! against baselines and stays manual; see `doc/ci-cd.md`.)
//!
//! The `#[ignore]` attributes mean "needs the demo dev server", not "manual":
//! the gate spawns a server on a free port and passes `--ignored` explicitly,
//! so a bare `cargo test` with no server still passes.
//!
//! ```text
//! cargo xtask test-reactivity               # spawns its own server, then tears it down
//! cargo make test-reactivity                # same, via cargo-make
//! # or against a server you already have running:
//! trunk serve                               # in demo/ (npm install once first)
//! cargo test --test reactivity_smoke -- --ignored --test-threads=1
//! ```
//!
//! Oracle shape (see `demo/src/debug.rs` and the ldui-49w.3 contract notes):
//! `state()` -> `{ "route": <pathname>, "theme": <daisyui theme>,
//! "state": { <per-page keys written via demo/src/debug_state.rs> } }`.
//! The per-page keys asserted here (`modal.open`, `tab.active`,
//! `datatable.sort`) are wired in the corresponding demo pages.

mod common;

use common::{click, harness_at, oracle};
use pixelproof_web::Key;
use serde_json::json;

/// The bridge reports the route it's on.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn oracle_reports_route_and_theme() {
    let h = harness_at("/components/button").await;
    let s = oracle(&h).await;
    assert_eq!(s["route"], json!("/components/button"), "oracle: {s}");
    // Fresh headless profile => no persisted theme => provider default.
    assert!(s["theme"].is_string(), "theme should be a string: {s}");
}

/// Without `?pp-freeze=1` neither seam exists: no bridge, no freeze style.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn bridge_absent_without_freeze_param() {
    let cfg = common::config();
    let h = pixelproof_web::Harness::launch_with_config(cfg)
        .await
        .expect("launch");
    h.navigate("/components/button").await.expect("navigate");
    // Wait for the app to actually mount, so the absence of the bridge is a
    // real assertion about a running app rather than a not-yet-loaded page.
    common::wait_for_selector(&h, "main").await;
    // Probe with a boolean expression rather than harness.app_debug_state():
    // chromiumoxide's into_value() rejects the JS `null` the guarded call
    // yields on bridge-less pages ("No value found"), so the harness helper
    // errors instead of returning None there.
    let bridge_present: bool = h
        .page()
        .evaluate("typeof window.__APP_DEBUG__ !== 'undefined'")
        .await
        .expect("evaluate")
        .into_value()
        .expect("bool");
    assert!(
        !bridge_present,
        "window.__APP_DEBUG__ must not exist without ?pp-freeze=1"
    );
    // The freeze style must be absent too (the other half of the seam).
    let freeze_present: bool = h
        .page()
        .evaluate(r#"document.querySelector('style[data-pixelproof="freeze"]') !== null"#)
        .await
        .expect("evaluate")
        .into_value()
        .expect("bool");
    assert!(
        !freeze_present,
        "freeze style must not be installed without ?pp-freeze=1"
    );
}

/// Modal: click-to-open flips `modal.open` true; Escape (native <dialog>
/// close event) flips it back false.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn modal_opens_and_escape_closes() {
    let h = harness_at("/components/modal").await;

    click(&h, "main .btn.btn-primary").await;
    h.assert_modal_open().await.expect("dialog visible");
    let s = oracle(&h).await;
    assert_eq!(s["state"]["modal.open"], json!(true), "oracle: {s}");

    h.press_key_sequence(&[Key::Escape]).await.expect("escape");
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    h.assert_modal_closed().await.expect("dialog hidden");
    let s = oracle(&h).await;
    assert_eq!(s["state"]["modal.open"], json!(false), "oracle: {s}");
}

/// DataTable: clicking the "Name" header sorts ascending; clicking it again
/// toggles to descending. Asserted via the sort oracle wired through the new
/// `DataTable on_sort_change` callback.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_sort_toggles_via_oracle() {
    let h = harness_at("/components/data-table").await;
    let header = "main table thead th:first-child";

    click(&h, header).await;
    let s = oracle(&h).await;
    assert_eq!(
        s["state"]["datatable.sort"],
        json!({ "column": "name", "order": "ascending" }),
        "oracle after first click: {s}"
    );

    click(&h, header).await;
    let s = oracle(&h).await;
    assert_eq!(
        s["state"]["datatable.sort"],
        json!({ "column": "name", "order": "descending" }),
        "oracle after second click: {s}"
    );
}

/// DataTable runtime localization (beads-gh7a): the demo's "Runtime
/// Localization" section derives `columns` and `texts` from a locale signal.
/// Toggling the locale must re-render the table chrome in place — the header
/// cells swap to the Spanish strings without a remount.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_headers_relocalize_via_dom() {
    let h = harness_at("/components/data-table").await;

    // Precondition: the Spanish headers are nowhere on the page while the
    // locale is English. ("Nombre"/"Correo" appear only in this section.)
    let dom = h.dom_html().await.expect("dom");
    assert!(
        !dom.contains("Nombre") && !dom.contains("Correo"),
        "Spanish headers must not render before the locale switch"
    );

    click(&h, "#locale-toggle").await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let dom = h.dom_html().await.expect("dom");
    assert!(
        dom.contains("Nombre") && dom.contains("Correo"),
        "headers must re-render to Spanish after the locale switch"
    );
}

/// Tabs: clicking the second tab of the Basic Tabs strip selects index 1.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn tab_click_selects_via_oracle() {
    let h = harness_at("/components/tab").await;

    click(&h, "main .tabs .tab:nth-child(2)").await;
    let s = oracle(&h).await;
    assert_eq!(s["state"]["tab.active"], json!(1), "oracle: {s}");

    click(&h, "main .tabs .tab:nth-child(3)").await;
    let s = oracle(&h).await;
    assert_eq!(s["state"]["tab.active"], json!(2), "oracle: {s}");
}

/// Theme switch: clicking the "cupcake" card in the BaseThemeSelector flips
/// the active theme, asserted via the oracle's top-level `theme` key. Each
/// harness gets its own throwaway Chrome profile (see `common::config`), so
/// the persisted `leptos-daisyui-demo-theme` localStorage entry cannot leak
/// between tests. "cupcake" is used (not "dark") so the precondition can
/// never collide with an OS-level prefers-color-scheme default.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn theme_switch_via_oracle() {
    let h = harness_at("/components/base_theme_selector").await;

    let before = oracle(&h).await;
    assert_ne!(before["theme"], json!("cupcake"), "precondition: {before}");

    click(&h, r#"main .card[data-theme="cupcake"]"#).await;
    let s = oracle(&h).await;
    assert_eq!(s["theme"], json!("cupcake"), "oracle: {s}");

    // The DOM agrees with the oracle: <html data-theme="cupcake">.
    let dom = h.dom_html().await.expect("dom");
    assert!(
        dom.contains(r#"<html data-theme="cupcake""#),
        "html[data-theme] should be cupcake"
    );
}
