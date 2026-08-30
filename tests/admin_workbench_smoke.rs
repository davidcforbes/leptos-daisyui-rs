//! Real-browser proof for the admin-workbench composition reference
//! (ldui-ynmd.3): one standard `AppShellTopBar`, one borderless base-page
//! `PageHeader` with seven icon quick actions, independent `KpiStrip` cards,
//! an `EntityTable` whose typed text/select filters live in its own aligned
//! filter row (no external filter bar duplicating it), and a right-docked
//! `FilterSidebar` assistant whose collapse returns width to the table.
//!
//! Drives the general demo app (`html_target: None`, like
//! `page_quick_actions_smoke.rs`/`section_heading_smoke.rs`) against the
//! existing `/components/admin_workbench` showcase route. Kept in its own
//! file/xtask step for the same reason as those: a focused fixture, not
//! folded into a pinned-count suite.
//!
//! COMPILE-ONLY as authored: this crate's actual browser lanes are being run
//! separately by the coordinator for this work session, so these `#[ignore]`
//! tests were verified only with `cargo test --test admin_workbench_smoke
//! --no-run`, never executed against a live demo server.

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use pixelproof_web::ViewportSize;
use serde_json::{Value, json};

const PAGE: &str = "/components/admin_workbench";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate admin-workbench fixture")
        .into_value()
        .expect("admin-workbench expression returns JSON")
}

/// Structural shape of the fixture: exactly one of each landmark/region, the
/// header's divider marker, how many quick actions render, how many KPI
/// cards render, and whether the table's own filter row (not an external
/// `FilterBar`) is present.
async fn snapshot(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="admin-workbench"]');
            const topBars = fixture.querySelectorAll('[data-app-shell-top-bar-region]');
            const headers = fixture.querySelectorAll('[data-page-header]');
            const mains = fixture.querySelectorAll('[role="main"]');
            const quickActions = fixture.querySelector('[data-page-quick-actions]');
            const kpiCards = fixture.querySelectorAll('[data-kpi-card]');
            const entityTables = fixture.querySelectorAll('[data-entity-table]');
            const headerFilterRow = fixture.querySelector('[data-entity-filter-control]');
            const externalFilterBar = fixture.querySelector('[data-filter-bar]');
            const assistant = fixture.querySelector('[data-testid="admin-workbench-assistant"]');
            return {
                topBarCount: topBars.length,
                headerCount: headers.length,
                headerDivider: headers[0] ? headers[0].getAttribute('data-page-header-divider') : null,
                mainCount: mains.length,
                quickActionCount: quickActions
                    ? quickActions.querySelectorAll('button, a').length
                    : 0,
                kpiCardCount: kpiCards.length,
                entityTableCount: entityTables.length,
                hasAlignedFilterRow: headerFilterRow !== null,
                hasExternalFilterBar: externalFilterBar !== null,
                assistantPresent: assistant !== null,
            };
        })()"#,
    )
    .await
}

/// The intended hierarchy: exactly one top bar, one borderless header, one
/// `role=main` landmark, seven quick actions, eight KPI cards, one
/// `EntityTable` with its own aligned filter row, and no external
/// `FilterBar` duplicating it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn wide_viewport_matches_the_intended_single_hierarchy() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set wide viewport");

    let s = snapshot(&h).await;
    assert_eq!(s["topBarCount"], json!(1), "exactly one top bar: {s}");
    assert_eq!(s["headerCount"], json!(1), "exactly one page header: {s}");
    assert_eq!(
        s["headerDivider"],
        json!("hidden"),
        "the base-page header is borderless: {s}"
    );
    assert_eq!(s["mainCount"], json!(1), "no duplicate main landmark: {s}");
    assert_eq!(
        s["quickActionCount"],
        json!(7),
        "seven icon quick actions: {s}"
    );
    assert_eq!(
        s["kpiCardCount"],
        json!(8),
        "eight independent KPI cards: {s}"
    );
    assert_eq!(
        s["entityTableCount"],
        json!(1),
        "exactly one EntityTable: {s}"
    );
    assert_eq!(
        s["hasAlignedFilterRow"],
        json!(true),
        "filters live in the table's own aligned row: {s}"
    );
    assert_eq!(
        s["hasExternalFilterBar"],
        json!(false),
        "no external FilterBar duplicating the table's filters: {s}"
    );
    assert_eq!(s["assistantPresent"], json!(true), "assistant renders: {s}");

    assert_no_browser_errors(&h, "admin-workbench wide viewport").await;
}

async fn table_width(h: &pixelproof_web::Harness) -> f64 {
    eval_json(
        h,
        r#"(() => {
            const table = document.querySelector('[data-testid="admin-workbench"] [data-entity-table]');
            return table.getBoundingClientRect().width;
        })()"#,
    )
    .await
    .as_f64()
    .expect("table width")
}

/// Collapsing the right assistant returns its freed width to the table --
/// the exact behavior the bead calls out by name.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn assistant_collapse_returns_width_to_the_table() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set wide viewport");

    let expanded_width = table_width(&h).await;

    click(
        &h,
        "[data-testid=\"admin-workbench-assistant\"] button[aria-expanded]",
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let collapsed_width = table_width(&h).await;

    assert!(
        collapsed_width > expanded_width,
        "collapsing the assistant must widen the table: expanded={expanded_width} collapsed={collapsed_width}"
    );

    assert_no_browser_errors(&h, "admin-workbench assistant collapse").await;
}

/// At a compact mobile viewport, nothing in the reference escapes the page
/// horizontally, and the seven quick actions wrap onto more than one row
/// instead of overflowing.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn compact_viewport_wraps_without_horizontal_overflow() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let no_horizontal_escape = eval_json(
        &h,
        r#"(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1)()"#,
    )
    .await;
    assert_eq!(
        no_horizontal_escape,
        json!(true),
        "the compact page must not scroll horizontally"
    );

    let rows = eval_json(
        &h,
        r#"(() => {
            const group = document.querySelector('[data-testid="admin-workbench"] [data-page-quick-actions]');
            const tops = Array.from(group.querySelectorAll('button, a')).map(
                el => Math.round(el.getBoundingClientRect().top)
            );
            return new Set(tops).size;
        })()"#,
    )
    .await;
    assert!(
        rows.as_u64().unwrap_or(0) > 1,
        "seven quick actions at a compact width must wrap onto more than one row: {rows}"
    );

    assert_no_browser_errors(&h, "admin-workbench compact viewport").await;
}
