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

// ── Interactive line chart component-region baselines (ldui-9tr.7) ──────────
//
// Component-scoped (`capture_and_compare_region` on the chart root) so a
// small chart regression cannot be diluted by the full showcase page. Every
// interactive state is produced by real CDP input through the Task 5 seams,
// and DOM-boxed floaters (the tooltip card) are containment-asserted before
// the pixel compare.

/// Pointer overlay of the interactive (first) categorical chart.
const CHART_ROOT: &str = "[data-testid=\"interactive-line-chart\"]";
const CHART_OVERLAY: &str =
    "[data-testid=\"interactive-line-chart\"] [data-line-chart-pointer-overlay]";
const CHART_STAGE: &str = "[data-testid=\"interactive-line-chart\"] [data-line-chart-stage]";
const CHART_TOOLTIP: &str =
    "[data-testid=\"interactive-line-chart\"] [data-testid=\"line-chart-tooltip\"]";
const CHART_TAB_STOP: &str =
    "[data-testid=\"interactive-line-chart\"] [data-line-chart-focus][tabindex=\"0\"]";

/// Default: no active point; three patterned lines, legend, labels, markers.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_default_matches_baseline() {
    let h = harness_at("/components/charts").await;
    let r = h
        .capture_and_compare_region(
            "charts",
            "interactive-line-chart",
            &state("default"),
            CHART_ROOT,
        )
        .await
        .expect("capture charts/interactive-line-chart/default");
    assert!(r.passed, "{}", r.summary());
}

/// Hovered: real pointer at category 8; card lists all values and stays
/// inside the stage.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_hovered_matches_baseline() {
    let h = harness_at("/components/charts").await;
    common::move_pointer_to_svg_fraction(&h, CHART_OVERLAY, 7.0 / 13.0, 0.5).await;
    h.assert_region_within(CHART_TOOLTIP, CHART_STAGE)
        .await
        .expect("hover card contained in stage");
    let r = h
        .capture_and_compare_region(
            "charts",
            "interactive-line-chart",
            &state("hovered"),
            CHART_ROOT,
        )
        .await
        .expect("capture charts/interactive-line-chart/hovered");
    assert!(r.passed, "{}", r.summary());
}

/// Focused: real focus + arrow input; visible focus cue, same card contract.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_focused_matches_baseline() {
    let h = harness_at("/components/charts").await;
    h.page()
        .find_element(CHART_TAB_STOP)
        .await
        .expect("find tab stop")
        .focus()
        .await
        .expect("focus tab stop");
    h.press_key_sequence(&[
        pixelproof_web::Key::ArrowRight,
        pixelproof_web::Key::ArrowRight,
    ])
    .await
    .expect("arrows");
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    h.assert_region_within(CHART_TOOLTIP, CHART_STAGE)
        .await
        .expect("focus card contained in stage");
    let r = h
        .capture_and_compare_region(
            "charts",
            "interactive-line-chart",
            &state("focused"),
            CHART_ROOT,
        )
        .await
        .expect("capture charts/interactive-line-chart/focused");
    assert!(r.passed, "{}", r.summary());
}

/// Missing data: the `Show gaps` control swaps in the deterministic multi-gap
/// fixture; paths must never bridge an interior gap.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_missing_data_matches_baseline() {
    let h = harness_at("/components/charts").await;
    common::click(&h, "[data-testid=\"line-chart-gaps\"]").await;
    let r = h
        .capture_and_compare_region(
            "charts",
            "interactive-line-chart",
            &state("missing-data"),
            CHART_ROOT,
        )
        .await
        .expect("capture charts/interactive-line-chart/missing-data");
    assert!(r.passed, "{}", r.summary());
}

/// Narrow (tablet width): ticks thin, legend wraps, edge label and card stay
/// inside the stage.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_narrow_matches_baseline() {
    let h = harness_at("/components/charts").await;
    h.set_viewport(pixelproof_web::ViewportSize::TABLET)
        .await
        .expect("tablet viewport");
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    common::move_pointer_to_svg_fraction(&h, CHART_OVERLAY, 0.999, 0.3).await;
    h.assert_region_within(CHART_TOOLTIP, CHART_STAGE)
        .await
        .expect("narrow edge card contained in stage");
    let r = h
        .capture_and_compare_region(
            "charts",
            "interactive-line-chart",
            &common::state_at("narrow", 768),
            CHART_ROOT,
        )
        .await
        .expect("capture charts/interactive-line-chart/narrow");
    assert!(r.passed, "{}", r.summary());
}
