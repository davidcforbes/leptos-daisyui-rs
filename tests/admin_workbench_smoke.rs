//! Real-browser proof for the admin-workbench composition reference
//! (ldui-ynmd.3): one standard `AppShellTopBar`, one borderless base-page
//! `PageHeader` with seven icon quick actions, independent `KpiStrip` cards,
//! an `EntityTable` whose typed text/select filters live in its own aligned
//! filter row (no external filter bar duplicating it), a right-docked
//! `FilterSidebar` assistant whose collapse returns width to the table, and
//! a blue `Fab` Help button anchored bottom-right (ldui-0qro).
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

// ── Help FAB (ldui-0qro) ─────────────────────────────────────────────────
//
// The reference must render the blue Help floating action button anchored
// bottom-right, built from the existing `Fab`, at both viewports; it must
// have an accessible name and a visible focus treatment; it must not
// overlap the table's pagination footer or the right assistant panel in
// either assistant state; and collapsing/expanding the assistant must not
// strand or hide it.

/// A DOM element's viewport-relative bounding rect, or `null` if the
/// selector matches nothing.
async fn rect_of(h: &pixelproof_web::Harness, selector: &str) -> Value {
    let selector_json = serde_json::to_string(selector).expect("serialize rect selector");
    let script = format!(
        r#"(() => {{
            const el = document.querySelector({selector_json});
            if (!el) return null;
            const r = el.getBoundingClientRect();
            return {{ x: r.x, y: r.y, width: r.width, height: r.height }};
        }})()"#
    );
    eval_json(h, &script).await
}

/// Whether two rects (as returned by [`rect_of`]) overlap. Either input
/// being `null`/non-finite is treated as "does not overlap" so a caller
/// that already asserted presence gets a real intersection test.
fn rects_overlap(a: &Value, b: &Value) -> bool {
    let dims = |v: &Value| -> Option<(f64, f64, f64, f64)> {
        Some((
            v.get("x")?.as_f64()?,
            v.get("y")?.as_f64()?,
            v.get("width")?.as_f64()?,
            v.get("height")?.as_f64()?,
        ))
    };
    match (dims(a), dims(b)) {
        (Some((ax, ay, aw, ah)), Some((bx, by, bw, bh))) => {
            ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
        }
        _ => false,
    }
}

const HELP_FAB_TRIGGER: &str = "[data-testid=\"admin-workbench-help-fab-trigger\"]";
const ASSISTANT_PANEL: &str = "[data-testid=\"admin-workbench-assistant\"]";
const TABLE_PAGINATION: &str = "[data-testid=\"admin-workbench-table\"] .join";

/// `{ ariaLabel, hasFocusRing }` for the Help FAB's trigger button.
async fn help_fab_a11y(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const el = document.querySelector('[data-testid="admin-workbench-help-fab-trigger"]');
            if (!el) return null;
            return {
                ariaLabel: el.getAttribute('aria-label'),
                hasFocusRing: el.classList.contains('ld-focus-ring'),
            };
        })()"#,
    )
    .await
}

/// The Help FAB renders, is anchored to the bottom-right quadrant of the
/// reference, carries an accessible name and the shared visible-focus
/// treatment, does not overlap the table's pagination footer or the
/// (expanded) assistant panel, and its click callback is wired -- caller-
/// owned activation, exactly like the reference's other actions.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn wide_viewport_help_fab_present_placed_and_accessible() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set wide viewport");

    let a11y = help_fab_a11y(&h).await;
    assert_eq!(
        a11y["ariaLabel"],
        json!("Help"),
        "Help FAB trigger must have an accessible name: {a11y}"
    );
    assert_eq!(
        a11y["hasFocusRing"],
        json!(true),
        "Help FAB trigger must carry the shared visible focus treatment: {a11y}"
    );

    let fab = rect_of(&h, HELP_FAB_TRIGGER).await;
    let workbench = rect_of(&h, "[data-testid=\"admin-workbench\"]").await;
    let fab_x = fab["x"].as_f64().expect("fab rect x");
    let fab_y = fab["y"].as_f64().expect("fab rect y");
    let wb_x = workbench["x"].as_f64().expect("workbench rect x");
    let wb_y = workbench["y"].as_f64().expect("workbench rect y");
    let wb_w = workbench["width"].as_f64().expect("workbench rect width");
    let wb_h = workbench["height"].as_f64().expect("workbench rect height");
    assert!(
        fab_x > wb_x + wb_w / 2.0 && fab_y > wb_y + wb_h / 2.0,
        "Help FAB must be anchored in the bottom-right quadrant: fab={fab} workbench={workbench}"
    );

    let assistant = rect_of(&h, ASSISTANT_PANEL).await;
    let pagination = rect_of(&h, TABLE_PAGINATION).await;
    assert!(
        !rects_overlap(&fab, &assistant),
        "Help FAB must not overlap the (expanded) assistant panel: fab={fab} assistant={assistant}"
    );
    assert!(
        !rects_overlap(&fab, &pagination),
        "Help FAB must not overlap the table's pagination footer: fab={fab} pagination={pagination}"
    );

    let before = eval_json(
        &h,
        r#"document.querySelector('[data-testid="admin-workbench-help-count"]').textContent.trim()"#,
    )
    .await;
    assert_eq!(before, json!("0"), "help count starts at zero: {before}");

    click(&h, HELP_FAB_TRIGGER).await;

    let after = eval_json(
        &h,
        r#"document.querySelector('[data-testid="admin-workbench-help-count"]').textContent.trim()"#,
    )
    .await;
    assert_eq!(
        after,
        json!("1"),
        "clicking the Help FAB must run the caller-owned callback exactly once: {after}"
    );

    assert_no_browser_errors(&h, "admin-workbench help FAB wide viewport").await;
}

/// Collapsing and re-expanding the assistant must not strand or hide the
/// Help FAB: it stays rendered with a real size and never overlaps the
/// assistant panel or the table's pagination footer in either state.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn assistant_collapse_does_not_strand_the_help_fab() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set wide viewport");

    let assert_fab_visible_and_clear = |label: &'static str,
                                        fab: Value,
                                        assistant: Value,
                                        pagination: Value| {
        let width = fab["width"].as_f64().unwrap_or(0.0);
        let height = fab["height"].as_f64().unwrap_or(0.0);
        assert!(
            width > 0.0 && height > 0.0,
            "Help FAB must stay visible with the assistant {label}: fab={fab}"
        );
        assert!(
            !rects_overlap(&fab, &assistant),
            "Help FAB must not overlap the assistant panel while {label}: fab={fab} assistant={assistant}"
        );
        assert!(
            !rects_overlap(&fab, &pagination),
            "Help FAB must not overlap the table's pagination footer while the assistant is {label}: \
             fab={fab} pagination={pagination}"
        );
    };

    let fab_expanded = rect_of(&h, HELP_FAB_TRIGGER).await;
    let assistant_expanded = rect_of(&h, ASSISTANT_PANEL).await;
    let pagination_expanded = rect_of(&h, TABLE_PAGINATION).await;
    assert_fab_visible_and_clear(
        "expanded",
        fab_expanded,
        assistant_expanded,
        pagination_expanded,
    );

    click(
        &h,
        "[data-testid=\"admin-workbench-assistant\"] button[aria-expanded]",
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let fab_collapsed = rect_of(&h, HELP_FAB_TRIGGER).await;
    let assistant_collapsed = rect_of(&h, ASSISTANT_PANEL).await;
    let pagination_collapsed = rect_of(&h, TABLE_PAGINATION).await;
    assert_fab_visible_and_clear(
        "collapsed",
        fab_collapsed,
        assistant_collapsed,
        pagination_collapsed,
    );

    click(
        &h,
        "[data-testid=\"admin-workbench-assistant\"] button[aria-expanded]",
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let fab_re_expanded = rect_of(&h, HELP_FAB_TRIGGER).await;
    let assistant_re_expanded = rect_of(&h, ASSISTANT_PANEL).await;
    let pagination_re_expanded = rect_of(&h, TABLE_PAGINATION).await;
    assert_fab_visible_and_clear(
        "re-expanded",
        fab_re_expanded,
        assistant_re_expanded,
        pagination_re_expanded,
    );

    assert_no_browser_errors(&h, "admin-workbench help FAB assistant collapse").await;
}

/// At a compact mobile viewport the Help FAB still renders with an
/// accessible name, stays inside the reference (no horizontal escape), and
/// still clears the assistant panel and the table's pagination footer.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn compact_viewport_help_fab_present_placed_and_accessible() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let a11y = help_fab_a11y(&h).await;
    assert_eq!(
        a11y["ariaLabel"],
        json!("Help"),
        "Help FAB trigger must have an accessible name at compact width: {a11y}"
    );

    let no_horizontal_escape = eval_json(
        &h,
        r#"(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1)()"#,
    )
    .await;
    assert_eq!(
        no_horizontal_escape,
        json!(true),
        "the Help FAB must not introduce horizontal overflow at compact width"
    );

    let fab = rect_of(&h, HELP_FAB_TRIGGER).await;
    let assistant = rect_of(&h, ASSISTANT_PANEL).await;
    let pagination = rect_of(&h, TABLE_PAGINATION).await;
    let width = fab["width"].as_f64().unwrap_or(0.0);
    let height = fab["height"].as_f64().unwrap_or(0.0);
    assert!(
        width > 0.0 && height > 0.0,
        "Help FAB must render at compact width: fab={fab}"
    );
    assert!(
        !rects_overlap(&fab, &assistant),
        "Help FAB must not overlap the assistant panel at compact width: fab={fab} assistant={assistant}"
    );
    assert!(
        !rects_overlap(&fab, &pagination),
        "Help FAB must not overlap the table's pagination footer at compact width: \
         fab={fab} pagination={pagination}"
    );

    assert_no_browser_errors(&h, "admin-workbench help FAB compact viewport").await;
}
