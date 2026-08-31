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

/// Assert a captured page matches its committed baseline.
///
/// `Outcome::check` deliberately returns `Ok` in capture mode -- a baseline was
/// written, *not* verified -- which is the claim the removed `Outcome::passed`
/// field used to make silently (PixelProof 60dcd8b split capture from
/// comparison for exactly that reason). These tests are named
/// `*_matches_baseline`, so outside an explicit `VISUAL_TEST_MODE=capture` run
/// they must refuse to pass without a real comparison: a misconfigured run that
/// quietly captured instead of comparing would otherwise report green having
/// verified nothing.
fn assert_matches_baseline(outcome: &pixelproof_web::Outcome, what: &str) {
    let capturing = std::env::var("VISUAL_TEST_MODE")
        .map(|mode| mode.eq_ignore_ascii_case("capture"))
        .unwrap_or(false);
    if !capturing {
        assert!(
            outcome.compared().is_some(),
            "{what}: expected a real comparison against a committed baseline,              got a capture -- set VISUAL_TEST_MODE=capture only when refreshing"
        );
    }
    if let Err(summary) = outcome.check() {
        panic!("{what}: {summary}");
    }
}

#[test]
fn component_capture_region_requires_painted_area_inside_the_viewport() {
    let viewport = pixelproof_web::ViewportSize::new(1280, 800);

    assert!(common::region_fits_viewport(
        100.0, 100.0, 320.0, 180.0, viewport
    ));
    assert!(common::region_fits_viewport(
        0.0, 0.0, 1280.0, 800.0, viewport
    ));
    assert!(!common::region_fits_viewport(
        100.0, 100.0, 0.0, 180.0, viewport
    ));
    assert!(!common::region_fits_viewport(
        100.0, 100.0, 320.0, 0.0, viewport
    ));
    assert!(!common::region_fits_viewport(
        -1.0, 100.0, 320.0, 180.0, viewport
    ));
    assert!(!common::region_fits_viewport(
        100.0, 750.0, 320.0, 100.0, viewport
    ));
}

/// Button demo, default state.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn button_default_matches_baseline() {
    let h = harness_at("/components/button").await;
    let r = h
        .capture_and_compare("button", &state("default"))
        .await
        .expect("capture button/default");
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
}

/// The opinionated semantic header/filter bands and faint grid, captured as a
/// focused component region so the full table hierarchy remains reviewable.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_filter_row_matches_baseline() {
    let h = harness_at("/components/data-table").await;
    common::prepare_region_capture(&h, "#filter-row-table", common::VIEWPORT).await;
    let r = h
        .capture_and_compare_region(
            "data_table",
            "filter-row",
            &state("semantic-bands"),
            "#filter-row-table",
        )
        .await
        .expect("capture data_table/filter-row/semantic-bands");
    assert_matches_baseline(&r, "visual");
}

/// At a narrow viewport the aligned filter row stays attached to the same
/// stable tracks and the table exposes horizontal scrolling rather than
/// squeezing or clipping columns.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_filter_row_narrow_matches_baseline() {
    let h = harness_at("/components/data-table").await;
    let viewport = pixelproof_web::ViewportSize::new(420, 900);
    h.set_viewport(viewport)
        .await
        .expect("narrow table viewport");
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    common::prepare_region_capture(&h, "#filter-row-table", viewport).await;
    let r = h
        .capture_and_compare_region(
            "data_table",
            "filter-row",
            &common::state_at("semantic-bands-narrow", 420),
            "#filter-row-table",
        )
        .await
        .expect("capture data_table/filter-row/semantic-bands-narrow");
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
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
    assert_matches_baseline(&r, "visual");
}
