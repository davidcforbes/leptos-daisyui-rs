//! Visual regression smoke suite (ldui-49w.1) — SSIM against committed
//! baselines, mirroring the desktop suite shape
//! (Rust-DeskApp `crates/rust-deskui/tests/visual_smoke.rs`).
//!
//! `#[ignore]`d because it needs the demo dev server running:
//!
//! ```text
//! cargo make test-visual                    # orchestrated (server + tests)
//! # or manually:
//! trunk serve                               # in demo/ (npm install once first)
//! cargo test --test visual_smoke -- --ignored
//! ```
//!
//! Capture/refresh baselines (same env-var convention as the desktop suite):
//!
//! ```text
//! $env:VISUAL_TEST_MODE="capture"; cargo test --test visual_smoke -- --ignored
//! ```
//!
//! Baselines live in `tests/visual/baselines/<page>/<state>.w<width>.png`
//! (viewport-suffixed; single smoke viewport 1280x800). Ephemeral renders and
//! diffs go to `.review/visual-renders` / `.review/visual-diffs` (gitignored).
//!
//! Coverage: ~9 representative pages spanning complexity tiers — simple
//! (button, alert, toast), stateful (tabs, data-table incl. a sorted state),
//! overlay interaction states (modal open, dropdown expanded), and a flagship
//! fork addition (kanban). Every navigation carries `?pp-freeze=1` so CSS
//! animations/transitions are dead before first paint.

mod common;

use common::{harness_at, state};

/// Button demo, default state.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn button_default_matches_baseline() {
    let h = harness_at("/components/button").await;
    let r = h
        .capture_and_compare("button", &state("default"))
        .await
        .expect("capture button/default");
    assert!(r.passed, "{}", r.summary());
}

/// Alert demo, default state (covers the toast/alert tier).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn alert_default_matches_baseline() {
    let h = harness_at("/components/alert").await;
    let r = h
        .capture_and_compare("alert", &state("default"))
        .await
        .expect("capture alert/default");
    assert!(r.passed, "{}", r.summary());
}

/// Toast demo, default state.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn toast_default_matches_baseline() {
    let h = harness_at("/components/toast").await;
    let r = h
        .capture_and_compare("toast", &state("default"))
        .await
        .expect("capture toast/default");
    assert!(r.passed, "{}", r.summary());
}

/// Tab demo, default state (first tab active).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn tab_default_matches_baseline() {
    let h = harness_at("/components/tab").await;
    let r = h
        .capture_and_compare("tab", &state("default"))
        .await
        .expect("capture tab/default");
    assert!(r.passed, "{}", r.summary());
}

/// DataTable demo, default (unsorted) state.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_default_matches_baseline() {
    let h = harness_at("/components/data-table").await;
    let r = h
        .capture_and_compare("data_table", &state("default"))
        .await
        .expect("capture data_table/default");
    assert!(r.passed, "{}", r.summary());
}

/// DataTable demo, sorted state: real CDP click on the "Name" header of the
/// Basic DataTable (first table on the page) sorts ascending, then capture.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_sorted_matches_baseline() {
    let h = harness_at("/components/data-table").await;
    let r = h
        .click_and_capture(
            "main table thead th:first-child",
            "data_table",
            &state("sorted"),
        )
        .await
        .expect("click header + capture data_table/sorted");
    assert!(r.passed, "{}", r.summary());
}

/// Modal demo, open state: click "Open Modal" (the page's first primary
/// button), assert the native <dialog> is actually visible, then capture.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn modal_open_matches_baseline() {
    let h = harness_at("/components/modal").await;
    let r = h
        .click_and_capture("main .btn.btn-primary", "modal", &state("open"))
        .await
        .expect("click open + capture modal/open");
    h.assert_modal_open().await.expect("modal should be open");
    assert!(r.passed, "{}", r.summary());
}

/// Dropdown demo, expanded state: click the first "Menu" trigger (daisyUI
/// dropdowns open on focus, which a real CDP click provides), then capture.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn dropdown_expanded_matches_baseline() {
    let h = harness_at("/components/dropdown").await;
    let r = h
        .click_and_capture("main .dropdown .btn", "dropdown", &state("expanded"))
        .await
        .expect("click trigger + capture dropdown/expanded");
    assert!(r.passed, "{}", r.summary());
}

/// Kanban board demo (flagship fork addition), default state.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn kanban_default_matches_baseline() {
    let h = harness_at("/components/kanban").await;
    let r = h
        .capture_and_compare("kanban", &state("default"))
        .await
        .expect("capture kanban/default");
    assert!(r.passed, "{}", r.summary());
}
