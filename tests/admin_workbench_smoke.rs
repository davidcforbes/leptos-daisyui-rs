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
//! The Help FAB assertions (ldui-0qro) were run live against the demo dev
//! server and verified passing, including the root-cause fix documented on
//! the `style:position="fixed"` override in `demo/src/demos/admin_workbench.rs`.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, body_font_family, click, harness_at,
};
use pixelproof_web::{Key, ViewportSize};
use serde_json::{Value, json};
use std::time::Duration;

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

/// Per-card `{ id, ariaLabel, cardHeight, valueTop, labelClipped, labelText }`
/// for every `[data-kpi-card]` in the admin-workbench `KpiStrip`. `labelClipped`
/// compares the label span's `scrollHeight` (the height its full text needs)
/// against its `clientHeight` (capped by `line-clamp-2`'s reserved two-line
/// box): a normal-length label fits inside that box and is therefore never
/// visually clamped, while the fixture's one deliberately over-long label
/// (`avg-first-response`) is allowed to clip.
async fn kpi_label_wrap_report(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const cards = Array.from(
                document.querySelectorAll('[data-testid="admin-workbench-kpis"] [data-kpi-card]')
            );
            return cards.map(card => {
                const label = card.querySelector('.line-clamp-2');
                const value = card.querySelector('[data-kpi-card-value]');
                const cardRect = card.getBoundingClientRect();
                const valueRect = value ? value.getBoundingClientRect() : null;
                return {
                    id: card.getAttribute('data-kpi-card'),
                    ariaLabel: card.getAttribute('aria-label'),
                    cardHeight: Math.round(cardRect.height),
                    valueTop: valueRect ? Math.round(valueRect.top) : null,
                    labelClipped: label ? label.scrollHeight > label.clientHeight + 1 : null,
                    labelText: label ? label.textContent.trim() : null,
                };
            });
        })()"#,
    )
    .await
}

/// ldui-tbaw: at the reported 1680px consumer width, ordinary and
/// two-line-length labels render with no ellipsis, cards in the row stay
/// equal height, every card's value starts at the identical vertical offset
/// regardless of whether its own label used one or two lines, and every
/// card's accessible name still carries its label in full -- including the
/// fixture's one deliberately over-long label, which is allowed to clip
/// visually after two lines but must not lose text from assistive tech.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn wide_viewport_kpi_labels_wrap_without_ellipsis_and_stay_aligned() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1680, 900))
        .await
        .expect("set 1680px viewport");

    let report = kpi_label_wrap_report(&h).await;
    let cards = report.as_array().expect("kpi label wrap report array");
    assert_eq!(cards.len(), 8, "eight KPI cards: {report}");

    let heights: Vec<i64> = cards
        .iter()
        .map(|c| c["cardHeight"].as_i64().expect("card height"))
        .collect();
    let min_height = *heights.iter().min().expect("at least one card");
    let max_height = *heights.iter().max().expect("at least one card");
    assert!(
        max_height - min_height <= 1,
        "cards must stay equal height across one- and two-line labels: {report}"
    );

    let value_tops: Vec<i64> = cards
        .iter()
        .map(|c| c["valueTop"].as_i64().expect("value top"))
        .collect();
    let min_top = *value_tops.iter().min().expect("at least one card");
    let max_top = *value_tops.iter().max().expect("at least one card");
    assert!(
        max_top - min_top <= 1,
        "values must stay aligned across one- and two-line labels: {report}"
    );

    for card in cards {
        let id = card["id"].as_str().unwrap_or_default();
        let aria = card["ariaLabel"].as_str().unwrap_or_default();
        let label_text = card["labelText"].as_str().unwrap_or_default();
        assert!(
            !label_text.is_empty(),
            "every card must expose its label span: {card}"
        );
        assert!(
            aria.contains(label_text),
            "the accessible name must contain the complete label for {id}: {card}"
        );
        // Only the fixture's one deliberately over-long label is allowed to
        // clip visually after two lines; every ordinary/two-line label must
        // render without an ellipsis.
        if id != "avg-first-response" {
            assert_eq!(
                card["labelClipped"],
                json!(false),
                "normal-length labels must render without ellipsis for {id}: {card}"
            );
        }
    }

    assert_no_browser_errors(&h, "admin-workbench kpi label wrap").await;
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

/// ldui-kmpa: every KpiCard carries a coloured LEFT accent edge, blue by
/// default, and a status only changes its colour -- never the geometry.
///
/// Asserts computed geometry and paint rather than class names: a class-level
/// check would pass while the edge rendered at the top, at zero width, or in a
/// colour the theme never resolved. The neutral card's blue is the point of
/// the bead -- an accent that appeared only on exceptional cards would make
/// the EDGE the signal, whereas a universal edge makes the COLOUR the signal.
#[tokio::test]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn kpi_cards_carry_a_left_accent_edge_blue_by_default_ldui_kmpa() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set viewport");

    let report = eval_json(
        &h,
        r#"(() => {
            const pick = id => document.querySelector(`[data-kpi-card="${id}"]`);
            const read = card => {
                const bar = card.firstElementChild;
                const c = card.getBoundingClientRect();
                const b = bar.getBoundingClientRect();
                return {
                    barWidth: +b.width.toFixed(1),
                    // Inset from the card's own edge: the 1px border, not 0.
                    leftInset: +(b.x - c.x).toFixed(1),
                    // Short by the two 1px borders, never by the card's padding.
                    heightShortfall: +(c.height - b.height).toFixed(1),
                    color: getComputedStyle(bar).backgroundColor,
                };
            };
            return { neutral: read(pick('open-matters')), semantic: read(pick('customer-success-pts')) };
        })()"#,
    )
    .await;

    for key in ["neutral", "semantic"] {
        let m = &report[key];
        let width = m["barWidth"].as_f64().expect("bar width");
        assert!(
            width >= 2.0,
            "{key} card must have a visible accent edge, got {width}px: {report}"
        );
        let inset = m["leftInset"].as_f64().expect("inset");
        assert!(
            inset <= 2.0,
            "the accent must sit on the LEFT edge, not inboard or on top;              inset was {inset}px: {report}"
        );
        let shortfall = m["heightShortfall"].as_f64().expect("shortfall");
        assert!(
            shortfall <= 4.0,
            "the accent must run the card's full height (a top stripe or a              padded bar fails here); shortfall {shortfall}px: {report}"
        );
    }

    // The default card is BLUE, not uncoloured: rgb(0, 69, 120) is
    // ui_tokens::color::STATUS_BLUE_FG (#004578).
    assert_eq!(
        report["neutral"]["color"],
        json!("rgb(0, 69, 120)"),
        "a Neutral card must paint the house blue accent: {report}"
    );
    assert_ne!(
        report["semantic"]["color"], report["neutral"]["color"],
        "a semantic status must override the default colour: {report}"
    );

    // Geometry is identical across statuses -- only the colour differs.
    assert_eq!(
        report["neutral"]["barWidth"], report["semantic"]["barWidth"],
        "a status must not change the accent's geometry: {report}"
    );

    assert_no_browser_errors(&h, "admin-workbench kpi accent edge").await;
}

/// A theme's approved card shadow, injected the way a product actually ships
/// one: a single `:root` custom-property declaration in its own stylesheet.
/// Deliberately not a value from `ui_tokens::elevation`, so a substitution
/// that silently did nothing would still read as the framework default.
const THEME_CARD_SHADOW: &str = "0px 3px 9px rgba(0, 0, 0, 0.33)";

/// ldui-k4fn: KPI cards rest at the framework's declared, NON-interactive
/// card elevation, and a product theme can substitute its own approved card
/// shadow by setting one custom property.
///
/// Reads computed style, never class names. A class-level assertion passes
/// while the class resolves to nothing -- which is exactly the ldui-h7tw
/// failure mode this bead had to avoid, because the class replaced a stock
/// `shadow-sm` and a rule that does not resolve leaves the card with NO
/// shadow at all.
///
/// Three properties, each of which fails independently:
///
/// 1. The resting shadow parses as ONE painted layer and matches
///    `ui_tokens::elevation::LEVEL_4` -- the declared "card resting
///    elevation". Tailwind's `shadow-sm` is two painted layers, so the
///    single-painted-layer parse alone rejects what this replaced. (The
///    transparent zero placeholders `ld-card-depth` composes around its
///    one layer, so rings can coexist with it -- ldui-xr7i -- paint
///    nothing and are not layers in this sense.)
/// 2. That shadow is on the active [`ldui_audit::StyleProfile`] -- the same
///    profile and the same `shadow_ok` epsilon the depth family sweeps with,
///    so this cannot pass while `cargo xtask test-style` would report the
///    card.
/// 3. A `:root { --ld-card-shadow: ... }` rule in a separate stylesheet
///    overrides it with no `!important`, no descendant selector into
///    KpiCard's markup, and no fork of the class -- and removing that
///    stylesheet restores the framework default, proving the override is a
///    substitution rather than a one-way overwrite.
#[tokio::test]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn kpi_cards_rest_at_the_declared_card_elevation_ldui_k4fn() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set viewport");

    // Parses a computed `box-shadow` into the ONE painted layer, in the
    // component form `ShadowSpec` compares on. `ld-card-depth` composes
    // with Tailwind's ring variables the way Tailwind's own shadow
    // utilities do (ldui-xr7i), so the computed value carries fully
    // transparent zero-geometry placeholder layers (`rgba(0, 0, 0, 0) 0px
    // 0px 0px 0px`) around the real one; those paint nothing and are
    // dropped, exactly as the depth audit drops them by opacity. Anything
    // other than exactly one remaining layer -- `none`, or the two opaque
    // layers every stock Tailwind `shadow-*` paints -- comes back null,
    // which is itself a finding rather than a skipped check.
    const PARSE: &str = r#"
        const parseLayer = v => {
            const m = /^rgba?\(([^)]+)\)\s+(-?[\d.]+)px\s+(-?[\d.]+)px\s+(-?[\d.]+)px(?:\s+(-?[\d.]+)px)?(\s+inset)?$/.exec(v.trim());
            if (!m) return null;
            const c = m[1].split(',').map(s => parseFloat(s.trim()));
            return {
                r: c[0], g: c[1], b: c[2], a: c.length > 3 ? c[3] : 1,
                offsetX: +m[2], offsetY: +m[3], blur: +m[4],
                spread: m[5] ? +m[5] : 0, inset: !!m[6],
            };
        };
        const parse = v => {
            if (!v || v === 'none') return null;
            const layers = v.split(/,(?![^(]*\))/).map(parseLayer);
            if (layers.some(l => l === null)) return null;
            const painted = layers.filter(l =>
                !(l.a === 0 && l.offsetX === 0 && l.offsetY === 0 && l.blur === 0 && l.spread === 0));
            return painted.length === 1 ? painted[0] : null;
        };
        const card = document.querySelector('[data-kpi-card]');
    "#;

    let resting = eval_json(
        &h,
        &format!(
            r#"(() => {{
            {PARSE}
            const cs = getComputedStyle(card);
            return {{
                raw: cs.boxShadow,
                parsed: parse(cs.boxShadow),
                transform: cs.transform,
                classes: card.className,
                cardCount: document.querySelectorAll('[data-kpi-card]').length,
            }};
        }})()"#
        ),
    )
    .await;

    assert!(
        resting["cardCount"].as_u64().unwrap_or(0) >= 2,
        "fixture must actually render KPI cards: {resting}"
    );

    // No stock Tailwind elevation utility survives on the card, and the
    // interactive `ld-elevated` was not reached for either.
    let classes = resting["classes"].as_str().unwrap_or_default().to_owned();
    assert!(
        !classes.split_whitespace().any(|c| c.contains("shadow-")),
        "KPI card still carries a stock Tailwind shadow utility: {classes}"
    );
    assert!(
        classes.split_whitespace().any(|c| c == "ld-card-depth"),
        "KPI card is missing the framework's static elevation class: {classes}"
    );
    assert_eq!(
        resting["transform"],
        json!("none"),
        "a read-only KPI card must not carry a lift transform (that is \
         ld-elevated's job, and it is not what a KPI tile is): {resting}"
    );

    let parsed = &resting["parsed"];
    assert!(
        !parsed.is_null(),
        "the KPI card's resting box-shadow is not a single painted layer -- \
         `none` means the ld-card-depth rule did not resolve (the ldui-h7tw \
         trap: a class defined only by the runtime preamble), and more than \
         one PAINTED layer (transparent zero placeholders excluded) means a \
         stock Tailwind shadow-* is still painting. Got {:?}",
        resting["raw"]
    );

    let spec = shadow_spec(parsed);

    // 1. It is the declared CARD resting level, not merely some shadow.
    let level_4 = ui_tokens::elevation::LEVEL_4;
    assert!(
        (spec.offset_x - level_4.offset_x as f64).abs() < 0.5
            && (spec.offset_y - level_4.offset_y as f64).abs() < 0.5
            && (spec.blur - level_4.blur as f64).abs() < 0.5
            && (spec.opacity - level_4.opacity as f64).abs() < 0.01
            && spec.spread.abs() < 0.5
            && !spec.inset,
        "KPI card must rest at ui_tokens::elevation::LEVEL_4 \
         ({level_4:?}); computed {:?} parsed to {spec:?}",
        resting["raw"]
    );

    // 2. …and the depth family's own profile agrees, with its own epsilon.
    let profile = ldui_audit::from_ui_tokens(body_font_family(&h).await);
    assert!(
        profile.shadow_ok(&spec),
        "KPI card's resting shadow is off the active StyleProfile's declared \
         set -- `cargo xtask test-style` would report it as ad-hoc depth \
         (doc/visual-quality/ad-hoc-shadow.md). Got {spec:?}"
    );

    // 3. Product-theme override, then restore.
    let overridden = eval_json(
        &h,
        &format!(
            r#"(() => {{
            {PARSE}
            const s = document.createElement('style');
            s.id = 'ldui-k4fn-theme-probe';
            s.textContent = ':root {{ --ld-card-shadow: {THEME_CARD_SHADOW}; }}';
            document.head.appendChild(s);
            const applied = getComputedStyle(card).boxShadow;
            s.remove();
            return {{ applied, parsed: parse(applied), restored: getComputedStyle(card).boxShadow }};
        }})()"#
        ),
    )
    .await;

    let themed = &overridden["parsed"];
    assert!(
        !themed.is_null(),
        "a theme's --ld-card-shadow did not resolve to a single shadow: {overridden}"
    );
    let themed = shadow_spec(themed);
    assert!(
        (themed.offset_y - 3.0).abs() < 0.5
            && (themed.blur - 9.0).abs() < 0.5
            && (themed.opacity - 0.33).abs() < 0.01,
        "a product theme setting only `--ld-card-shadow` on :root must \
         replace the framework default with no page-local selector; the card \
         still painted {:?} ({themed:?})",
        overridden["applied"]
    );
    assert_eq!(
        overridden["restored"], resting["raw"],
        "removing the theme stylesheet must restore the framework default, \
         proving the hook is a substitution and not a one-way overwrite: \
         {overridden}"
    );

    assert_no_browser_errors(&h, "admin-workbench kpi card elevation").await;
}

/// Rebuild a [`ldui_audit::ShadowSpec`] from the JS parser's output, so the
/// comparison runs against the same struct (and the same epsilon) the depth
/// sweep uses rather than a hand-rolled tolerance.
fn shadow_spec(parsed: &Value) -> ldui_audit::ShadowSpec {
    let n = |k: &str| {
        parsed[k]
            .as_f64()
            .unwrap_or_else(|| panic!("{k} in {parsed}"))
    };
    let mut spec = ldui_audit::ShadowSpec::new(
        n("offsetX"),
        n("offsetY"),
        n("blur"),
        parsed["a"].as_f64().unwrap_or(1.0),
    )
    .with_spread(n("spread"));
    spec.color = [n("r"), n("g"), n("b")];
    spec.inset = parsed["inset"].as_bool().unwrap_or(false);
    spec
}

// ----------------------------------------------------------------------
// ldui-ztgo: typed baseline comparison and card activation.
//
// The fixture deliberately mixes four cards carrying a comparison row with
// four that carry none, so `wide_viewport_kpi_labels_wrap_without_ellipsis_and_stay_aligned`
// above -- which already asserts equal card heights and identical value
// offsets across the whole strip -- doubles as the proof that a
// baseline-bearing card and a plain one still line up.
// ----------------------------------------------------------------------

/// Per-card comparison and activation markers, read straight off the DOM
/// rather than off any colour.
async fn kpi_comparison_report(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const cards = Array.from(
                document.querySelectorAll('[data-testid="admin-workbench-kpis"] [data-kpi-card]')
            );
            return cards.map(card => {
                const comparison = card.querySelector('[data-kpi-card-comparison]');
                const bar = card.querySelector('[data-kpi-baseline-bar]');
                const readout = card.querySelector('[data-kpi-baseline-readout]');
                const sentence = card.querySelector('[data-kpi-baseline-sentence]');
                const action = card.querySelector('[data-kpi-card-action]');
                const barRect = bar ? bar.getBoundingClientRect() : null;
                const fill = bar
                    ? Array.from(bar.querySelectorAll('span')).map(span => ({
                          left: span.getBoundingClientRect().left - barRect.left,
                          width: span.getBoundingClientRect().width,
                      }))
                    : null;
                return {
                    id: card.getAttribute('data-kpi-card'),
                    status: card.getAttribute('data-kpi-card-status'),
                    activatable: card.getAttribute('data-kpi-card-activatable'),
                    state: comparison
                        ? comparison.getAttribute('data-kpi-baseline-state')
                        : null,
                    percent: comparison
                        ? comparison.getAttribute('data-kpi-baseline-percent')
                        : null,
                    saturated: comparison
                        ? comparison.getAttribute('data-kpi-baseline-saturated')
                        : null,
                    degraded: comparison
                        ? comparison.getAttribute('data-kpi-baseline-degraded')
                        : null,
                    hasBar: bar !== null,
                    barWidth: barRect ? barRect.width : null,
                    barChildren: fill,
                    readout: readout ? readout.textContent.trim() : null,
                    sentence: sentence ? sentence.textContent.trim() : null,
                    hasAction: action !== null,
                    actionName: action ? action.getAttribute('aria-label') : null,
                    actionDisabled: action ? action.disabled : null,
                    // Every focusable descendant of the card. The activation
                    // contract is at most ONE.
                    focusables: card.querySelectorAll(
                        'a[href], button, input, select, textarea, [tabindex]'
                    ).length,
                };
            });
        })()"#,
    )
    .await
}

fn card<'a>(report: &'a Value, id: &str) -> &'a Value {
    report
        .as_array()
        .expect("comparison report array")
        .iter()
        .find(|card| card["id"] == json!(id))
        .unwrap_or_else(|| panic!("card {id} in {report}"))
}

/// The bounded bar / truthful number pair, on real geometry.
///
/// `conversations-open` is 17 against a baseline of 4 -- 425% -- so its bar
/// runs out of track. The bar must be pinned to its track while the readout
/// keeps saying 425%, and `data-kpi-baseline-saturated` must record that the
/// geometry, not the value, is what ran out.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn an_over_baseline_card_bounds_its_bar_while_the_percentage_stays_truthful() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1680, 900))
        .await
        .expect("set 1680px viewport");

    let report = kpi_comparison_report(&h).await;

    let over = card(&report, "conversations-open");
    assert_eq!(over["state"], json!("above"), "{over}");
    assert_eq!(
        over["percent"],
        json!("425"),
        "the readout must report the real ratio, not the clamped bar: {over}"
    );
    assert_eq!(
        over["saturated"],
        json!("true"),
        "a bar that ran out of track must say so: {over}"
    );
    let readout = over["readout"].as_str().unwrap_or_default();
    assert!(
        readout.contains("425"),
        "the visible readout must stay truthful past the cap: {over}"
    );

    // The painted fill spans the whole track, and the baseline tick is still
    // there at 80% of it -- so a full bar reads as "well past the marker",
    // never as "exactly at the cap".
    let bar_width = over["barWidth"].as_f64().expect("bar width");
    let children = over["barChildren"]
        .as_array()
        .expect("bar children")
        .to_vec();
    let painted = children
        .iter()
        .map(|c| c["left"].as_f64().unwrap_or(0.0) + c["width"].as_f64().unwrap_or(0.0))
        .fold(0.0_f64, f64::max);
    assert!(
        (painted - bar_width).abs() <= 1.5,
        "the fill must be bounded by the track: painted {painted} of {bar_width} in {over}"
    );
    let marker = children
        .iter()
        .map(|c| c["left"].as_f64().unwrap_or(0.0))
        .fold(0.0_f64, f64::max);
    assert!(
        (marker - bar_width * 0.8).abs() <= 2.0,
        "the baseline marker must stay at 80% of the track even when the bar \
         is saturated: marker {marker} of {bar_width} in {over}"
    );

    // A card under its baseline puts the marker in exactly the same place.
    let under = card(&report, "payments-collected");
    assert_eq!(under["state"], json!("below"), "{under}");
    let under_width = under["barWidth"].as_f64().expect("bar width");
    assert!(
        (under_width - bar_width).abs() <= 1.5,
        "every comparison track in a strip is the same width: {under} / {over}"
    );

    assert_no_browser_errors(&h, "admin-workbench kpi comparison").await;
}

/// A settling baseline draws no bar and fabricates no percentage, but it
/// still SPEAKS -- its own sentence, distinct from the no-baseline one.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn a_settling_baseline_draws_no_bar_and_still_says_why() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1680, 900))
        .await
        .expect("set 1680px viewport");

    let report = kpi_comparison_report(&h).await;
    let settling = card(&report, "customer-success-pts");
    assert_eq!(settling["state"], json!("settling"), "{settling}");
    assert_eq!(settling["hasBar"], json!(false), "{settling}");
    assert_eq!(
        settling["percent"],
        Value::Null,
        "a settling baseline must not fabricate a ratio: {settling}"
    );
    assert_eq!(
        settling["degraded"],
        Value::Null,
        "a DECLARED settling window is not a defect: {settling}"
    );
    let sentence = settling["sentence"].as_str().unwrap_or_default();
    assert!(
        !sentence.is_empty() && !sentence.contains('{'),
        "the settling card must carry its own substituted copy: {settling}"
    );

    // Cards with no baseline at all render no comparison row whatsoever --
    // proof that the row is opt-in and existing cards are untouched.
    let plain = card(&report, "open-matters");
    assert_eq!(plain["state"], Value::Null, "{plain}");
    assert_eq!(plain["hasBar"], json!(false), "{plain}");

    assert_no_browser_errors(&h, "admin-workbench settling baseline").await;
}

/// Activation is opt-in, is exactly one tab stop, and never nests one
/// interactive element inside another.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn only_activatable_cards_are_focusable_and_only_once() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1680, 900))
        .await
        .expect("set 1680px viewport");

    let report = kpi_comparison_report(&h).await;
    for card in report.as_array().expect("comparison report array") {
        let activatable = card["activatable"] == json!("true");
        assert_eq!(
            card["hasAction"],
            json!(activatable),
            "the action control and the activatable marker must agree: {card}"
        );
        let focusables = card["focusables"].as_i64().expect("focusable count");
        if activatable {
            assert_eq!(
                focusables, 1,
                "an activatable card is ONE tab stop -- the help affordance \
                 stays a non-interactive span: {card}"
            );
            let name = card["actionName"].as_str().unwrap_or_default();
            assert!(
                name.starts_with("View details"),
                "the visible label must prefix the accessible name (WCAG 2.5.3): {card}"
            );
        } else {
            assert_eq!(
                focusables, 0,
                "a read-only card must not become focusable or announce as a \
                 control: {card}"
            );
        }
        // Status is readable without sampling a colour.
        assert!(
            card["status"].is_string(),
            "every card exposes its status as a marker: {card}"
        );
    }

    assert_no_browser_errors(&h, "admin-workbench kpi activation shape").await;
}

/// Pointer and keyboard both activate, and both hand back the stable
/// `KpiItem::id` -- never an index, never a label.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn pointer_and_keyboard_both_emit_the_stable_item_id() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1680, 900))
        .await
        .expect("set 1680px viewport");

    async fn activated(h: &pixelproof_web::Harness) -> Value {
        eval_json(
            h,
            r#"document.querySelector('[data-testid="admin-workbench-kpi-activated"]').textContent.trim()"#,
        )
        .await
    }

    let no_hire = "[data-kpi-card=\"no-hire-conversions\"] [data-kpi-card-action]";
    click(&h, no_hire).await;
    assert_eq!(
        activated(&h).await,
        json!("no-hire-conversions"),
        "a pointer press emits the activated card's stable id"
    );

    // Enter on a different card's control, so a stale value cannot pass.
    let payments = "[data-kpi-card=\"payments-collected\"] [data-kpi-card-action]";
    h.page()
        .find_element(payments)
        .await
        .expect("find the payments action")
        .focus()
        .await
        .expect("focus the payments action");
    h.press_key_sequence(&[Key::Enter])
        .await
        .expect("Enter on the payments action");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        activated(&h).await,
        json!("payments-collected"),
        "Enter activates the focused card's control"
    );

    // And Space, which is the other native button activation key.
    click(&h, no_hire).await;
    h.page()
        .find_element(payments)
        .await
        .expect("find the payments action")
        .focus()
        .await
        .expect("focus the payments action");
    h.press_key_sequence(&[Key::Space])
        .await
        .expect("Space on the payments action");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        activated(&h).await,
        json!("payments-collected"),
        "Space activates the focused card's control"
    );

    assert_no_browser_errors(&h, "admin-workbench kpi activation").await;
}

// ======================================================================
// ldui-k3ip: the typed balanced-six layout profile.
//
// These drive `/components/kpi_strip`, not `PAGE`. The suite already runs
// against the general demo app (`html_target: None`), so any showcase route
// is reachable from it, and the alternative -- adding twelve more cards to
// the admin-workbench fixture -- would move this file's own pinned
// `kpiCardCount` and eight-card label sweep. A separate suite would need an
// xtask step, which is out of this bead's scope.
// ======================================================================

const KPI_STRIP_PAGE: &str = "/components/kpi_strip";

/// Geometry for one KpiStrip fixture: the container width the container
/// queries actually measure, the strip's declared profile, and every card's
/// rounded position and width.
///
/// Rows are derived from the cards' own vertical offsets, not from a class
/// name: two rows of six means twelve cards occupying exactly two distinct
/// offsets, six at each. That is the assertion the bead asks for, and it is
/// one a class string cannot fake.
async fn strip_geometry(h: &pixelproof_web::Harness, testid: &str) -> Value {
    let expr = format!(
        r#"(() => {{
            const root = document.querySelector('[data-testid="{testid}"]');
            const container = root.querySelector('[data-kpi-strip-container]');
            const grid = root.querySelector('[data-kpi-strip]');
            const cards = Array.from(grid.querySelectorAll('[data-kpi-card]'));
            const rows = new Map();
            for (const card of cards) {{
                const r = card.getBoundingClientRect();
                const top = Math.round(r.top);
                if (!rows.has(top)) rows.set(top, []);
                rows.get(top).push(Math.round(r.width * 100) / 100);
            }}
            const ordered = Array.from(rows.keys()).sort((a, b) => a - b);
            return {{
                layout: grid.getAttribute('data-kpi-strip-layout'),
                containerWidth: Math.round(container.getBoundingClientRect().width * 100) / 100,
                cardCount: cards.length,
                rowWidths: ordered.map((top) => rows.get(top)),
                overflowing: grid.scrollWidth > grid.clientWidth + 1,
            }};
        }})()"#
    );
    eval_json(h, &expr).await
}

/// Every card in a row must share one track width, to within a rounding
/// pixel.
fn assert_equal_tracks(widths: &[f64], context: &str) {
    let first = widths[0];
    for width in widths {
        assert!(
            (width - first).abs() <= 1.0,
            "{context}: unequal card tracks {widths:?}"
        );
    }
}

/// The widths of one row of a `strip_geometry` report.
fn row_widths(report: &Value, index: usize) -> Vec<f64> {
    report["rowWidths"]
        .as_array()
        .expect("rowWidths array")
        .get(index)
        .unwrap_or_else(|| panic!("row {index} exists: {report}"))
        .as_array()
        .expect("row array")
        .iter()
        .map(|width| width.as_f64().expect("card width is a number"))
        .collect()
}

/// How many rows a `strip_geometry` report has.
fn row_count(report: &Value) -> usize {
    report["rowWidths"]
        .as_array()
        .expect("rowWidths array")
        .len()
}

/// THE bead's reproduction, measured. At a desktop width, twelve peer KPIs
/// under the balanced-six profile occupy exactly two rows of six with equal
/// tracks -- and the SAME twelve items under the default profile do not,
/// which is the negative control proving the assertion measures something.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn balanced_six_lays_twelve_peer_kpis_out_as_two_rows_of_six_ldui_k3ip() {
    let h = harness_at(KPI_STRIP_PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1680, 1000))
        .await
        .expect("set 1680px viewport");

    let balanced = strip_geometry(&h, "kpi-strip-balanced-six").await;
    assert_eq!(
        balanced["layout"],
        json!("balanced-six"),
        "the strip must report the profile it was asked for: {balanced}"
    );
    assert_eq!(balanced["cardCount"], json!(12), "{balanced}");
    let container = balanced["containerWidth"]
        .as_f64()
        .expect("container width");
    assert!(
        container >= 896.0,
        "the six-column rung starts at 896px; this fixture measured {container}px, \
         so the two-rows-of-six assertion below would be vacuous"
    );

    assert_eq!(
        row_count(&balanced),
        2,
        "twelve peers in two rows: {balanced}"
    );
    let mut every_card: Vec<f64> = Vec::new();
    for index in 0..2 {
        let row = row_widths(&balanced, index);
        assert_eq!(row.len(), 6, "row {index} must hold six cards: {balanced}");
        assert_equal_tracks(&row, &format!("balanced-six row {index}"));
        every_card.extend(row);
    }
    // Both rows share one track width, so neither reads as a different group.
    assert_equal_tracks(&every_card, "balanced-six whole strip");
    assert_eq!(balanced["overflowing"], json!(false), "{balanced}");

    // NEGATIVE CONTROL. The identical twelve items with no `layout` prop --
    // the hard-coded eight-column ladder -- cannot produce two rows of six.
    let default_strip = strip_geometry(&h, "kpi-strip-dashboard").await;
    assert_eq!(
        default_strip["layout"],
        json!("auto-eight"),
        "the default profile must be unchanged: {default_strip}"
    );
    assert_eq!(default_strip["cardCount"], json!(12), "{default_strip}");
    assert_eq!(
        row_widths(&default_strip, 0).len(),
        8,
        "the default ladder still fills eight columns: {default_strip}"
    );
    assert_eq!(
        row_widths(&default_strip, 1).len(),
        4,
        "and still ends on the ragged four this bead was filed about: {default_strip}"
    );

    assert_no_browser_errors(&h, "kpi-strip balanced-six twelve").await;
}

/// The other item counts the acceptance criteria name: six is one full row,
/// five is a deliberately ragged row whose cards keep the SAME track width
/// as the six-card strip (they do not stretch), and an empty strip renders
/// no cards and no overflow.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn balanced_six_handles_six_five_and_empty_item_sets_ldui_k3ip() {
    let h = harness_at(KPI_STRIP_PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1680, 1000))
        .await
        .expect("set 1680px viewport");

    let six = strip_geometry(&h, "kpi-strip-balanced-six-six").await;
    assert_eq!(six["cardCount"], json!(6), "{six}");
    assert_eq!(row_count(&six), 1, "six cards are one row: {six}");
    let six_row = row_widths(&six, 0);
    assert_eq!(six_row.len(), 6, "{six}");
    assert_equal_tracks(&six_row, "balanced-six six items");

    let five = strip_geometry(&h, "kpi-strip-balanced-six-five").await;
    assert_eq!(five["cardCount"], json!(5), "{five}");
    assert_eq!(row_count(&five), 1, "{five}");
    let five_row = row_widths(&five, 0);
    assert_eq!(
        five_row.len(),
        5,
        "a short last row is the deliberate outcome: {five}"
    );
    assert_equal_tracks(&five_row, "balanced-six five items");

    // The ragged row does NOT stretch: five cards in six explicit tracks are
    // the same width as six cards in six tracks, so a five-card scorecard and
    // a six-card one are visually the same kind of thing.
    assert!(
        (six_row[0] - five_row[0]).abs() <= 1.0,
        "a ragged row must not stretch its cards: {} vs {}",
        five_row[0],
        six_row[0]
    );

    let empty = strip_geometry(&h, "kpi-strip-balanced-six-empty").await;
    assert_eq!(empty["cardCount"], json!(0), "{empty}");
    assert_eq!(row_count(&empty), 0, "an empty strip has no rows: {empty}");
    assert_eq!(empty["overflowing"], json!(false), "{empty}");

    assert_no_browser_errors(&h, "kpi-strip balanced-six counts").await;
}

/// A balanced-six strip in a constrained column steps DOWN rather than
/// asking how wide the window is (ldui-tnyq), and never overflows.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn balanced_six_steps_down_in_a_narrow_column_ldui_k3ip() {
    let h = harness_at(KPI_STRIP_PAGE).await;
    begin_browser_error_capture(&h).await;

    for width in [1680_u32, 1280, 1024, 768] {
        h.set_viewport(ViewportSize::new(width, 1000))
            .await
            .expect("set viewport");

        let narrow = strip_geometry(&h, "kpi-strip-balanced-six-narrow").await;
        let container = narrow["containerWidth"].as_f64().expect("width");
        let row = row_widths(&narrow, 0);
        assert!(
            container < 896.0,
            "the narrow fixture must actually be narrow at {width}px: {narrow}"
        );
        assert!(
            row.len() <= 4,
            "a {container}px column must not render six columns at a {width}px \
             window -- that is the viewport-breakpoint bug ldui-tnyq fixed: {narrow}"
        );
        assert!(row.len() >= 2, "{narrow}");
        assert_eq!(
            narrow["overflowing"],
            json!(false),
            "the strip must wrap, never scroll horizontally: {narrow}"
        );
        assert_equal_tracks(&row, "balanced-six narrow column");
    }

    assert_no_browser_errors(&h, "kpi-strip balanced-six narrow").await;
}

/// ldui-ztgo's baseline comparison row still reads at six-column width: the
/// bar is measurably WIDER than the one the default eight-column ladder
/// already ships at the same container width, and the fixed 80% marker is
/// still clear of the track's right edge.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-admin-workbench)"]
async fn the_baseline_comparison_bar_still_reads_at_six_columns_ldui_k3ip() {
    let h = harness_at(KPI_STRIP_PAGE).await;
    begin_browser_error_capture(&h).await;

    h.set_viewport(ViewportSize::new(1680, 1000))
        .await
        .expect("set 1680px viewport");

    async fn bar_width(h: &pixelproof_web::Harness, testid: &str) -> f64 {
        let expr = format!(
            r#"(() => {{
                const bar = document.querySelector(
                    '[data-testid="{testid}"] [data-kpi-card="intakes"] [data-kpi-baseline-bar]'
                );
                return Math.round(bar.getBoundingClientRect().width * 100) / 100;
            }})()"#
        );
        eval_json(h, &expr)
            .await
            .as_f64()
            .expect("baseline bar width")
    }

    let balanced = bar_width(&h, "kpi-strip-balanced-six").await;
    let default_strip = bar_width(&h, "kpi-strip-dashboard").await;
    assert!(
        balanced > default_strip,
        "six columns are wider than eight at one container width, so the \
         comparison bar must gain room, not lose it: {balanced} vs {default_strip}"
    );
    assert!(
        balanced >= 100.0,
        "the comparison bar must stay legible at six-column width: {balanced}px"
    );
    // The marker sits at a fixed 80% of the track on every card, so it is
    // still well clear of the right edge at this width.
    assert!(balanced - balanced * 0.8 >= 2.0, "{balanced}");

    assert_no_browser_errors(&h, "kpi-strip balanced-six baseline bar").await;
}
