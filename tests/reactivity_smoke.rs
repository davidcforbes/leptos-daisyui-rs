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

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, click_svg_fraction, harness_at,
    move_pointer_to_svg_fraction, oracle,
};
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

/// Responsive paging must not collapse a short viewport to one row. When the
/// measured fit falls below the usability floor, the configured page size is
/// retained and the already-bounded table viewport scrolls instead.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn auto_page_size_keeps_a_usable_page_and_scrolls_short_viewports() {
    let h = harness_at("/components/data-table").await;

    h.page()
        .evaluate(
            r#"(() => {
                const root = document.querySelector('#auto-page-table');
                root.parentElement.style.height = '128px';
                return true;
            })()"#,
        )
        .await
        .expect("shrink responsive table host");
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    let snapshot = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('#auto-page-table');
            const viewport = root.querySelector(':scope > .overflow-x-auto');
            return {
                rows: root.querySelectorAll('tbody tr').length,
                viewportHeight: viewport.clientHeight,
                scrollHeight: viewport.scrollHeight,
            };
        })()"#,
    )
    .await;

    assert_eq!(
        snapshot["rows"],
        json!(10),
        "the default configured page size is the short-viewport fallback: {snapshot}"
    );
    assert!(
        snapshot["scrollHeight"].as_u64() > snapshot["viewportHeight"].as_u64(),
        "the bounded wrapper must scroll rather than collapsing pagination: {snapshot}"
    );
}

/// Text content of the element with the given `data-testid`.
async fn testid_text(h: &pixelproof_web::Harness, testid: &str) -> String {
    let sel = format!("[data-testid=\"{testid}\"]");
    let expr = format!(
        "document.querySelector({}).textContent",
        serde_json::to_string(&sel).unwrap()
    );
    h.page()
        .evaluate(expr.as_str())
        .await
        .expect("evaluate")
        .into_value()
        .expect("string")
}

/// Action cells (beads-2knb / `Column::action`): clicking the "Open" button
/// inside the activation demo's action column runs the button's handler but
/// must NOT fire `on_row_activate` — no per-renderer `stop_propagation`
/// involved, the containment lives in the framework. A click on a plain cell
/// of the same row still activates.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn action_cell_click_does_not_activate_row() {
    let h = harness_at("/components/data-table").await;

    click(
        &h,
        "#activation-table tbody tr:first-child td:last-child button",
    )
    .await;
    assert_eq!(testid_text(&h, "open-count").await, "1", "Open must run");
    assert_eq!(
        testid_text(&h, "activate-count").await,
        "0",
        "an action-cell click must not activate the row"
    );

    click(&h, "#activation-table tbody tr:first-child td:first-child").await;
    assert_eq!(
        testid_text(&h, "activate-count").await,
        "1",
        "a plain-cell click on the same row must still activate"
    );
    assert_eq!(testid_text(&h, "open-count").await, "1");
}

/// Number of elements matching `sel`.
async fn count_of(h: &pixelproof_web::Harness, sel: &str) -> u32 {
    let expr = format!(
        "document.querySelectorAll({}).length",
        serde_json::to_string(sel).unwrap()
    );
    h.page()
        .evaluate(expr.as_str())
        .await
        .expect("evaluate")
        .into_value()
        .expect("number")
}

/// Categorical line chart (ldui-9tr.4): the first chart example is a
/// deterministic, non-empty fourteen-week fixture. This is intentionally a
/// DOM contract rather than a screenshot oracle: Task 7 owns reviewed visual
/// baselines, while this test pins the semantic SVG/table structure that a
/// later interaction slice will build upon.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn categorical_line_chart_exposes_static_render_contract() {
    let h = harness_at("/components/charts").await;
    let snapshot = eval_json(
        &h,
        r#"(() => {
            const roots = [...document.querySelectorAll('[data-testid="interactive-line-chart"]')];
            const root = roots[0];
            if (!root) return null;
            const svg = root.querySelector('[data-line-chart-plot]');
            const series = [...new Map([...root.querySelectorAll('path[data-series-id]:not([data-category-index])')].map(path => [
                path.dataset.seriesId,
                { id: path.dataset.seriesId, dash: path.getAttribute('stroke-dasharray') },
            ])).values()];
            const marker = (selector) => root.querySelector(selector)?.tagName.toLowerCase() ?? null;
            const legend = root.querySelector('[data-line-chart-legend]');
            const table = root.querySelector('[data-line-chart-table]');
            // Dedicated focus-target marker: a bare rect[data-category-index]
            // also matches the square series markers.
            const focusTargets = [...svg.querySelectorAll('rect[data-line-chart-focus]')];
            const ids = [...svg.querySelectorAll('[id]')].map(element => element.id);
            const labelledBy = (svg.getAttribute('aria-labelledby') ?? '').split(/\s+/).filter(Boolean);
            const rowCells = index => [...(table?.querySelectorAll('tbody tr')[index]?.querySelectorAll('th, td') ?? [])]
                .map(cell => cell.textContent.trim());
            return {
                rootCount: roots.length,
                plotCount: root.querySelectorAll('[data-line-chart-plot]').length,
                series,
                actualSegments: root.querySelectorAll('path[data-series-id="actual"]:not([data-category-index])').length,
                actualMarker: marker('circle[data-series-id="actual"][data-category-index="0"]'),
                averageMarker: marker('rect[data-series-id="rolling-average"][data-category-index="0"]'),
                targetMarker: marker('path[data-series-id="target"][data-category-index="0"]'),
                markerCounts: {
                    actual: root.querySelectorAll('[data-series-id="actual"][data-category-index]').length,
                    average: root.querySelectorAll('[data-series-id="rolling-average"][data-category-index]').length,
                    target: root.querySelectorAll('[data-series-id="target"][data-category-index]').length,
                },
                firstActualMarkerFill: root.querySelector('circle[data-series-id="actual"][data-category-index="0"]')?.style.getPropertyValue('fill') ?? null,
                missingActual: root.querySelector('[data-series-id="actual"][data-category-key="week-07"]') === null,
                focusCount: focusTargets.length,
                tabstopCount: focusTargets.filter(target => target.getAttribute('tabindex') === '0').length,
                focusRoles: [...new Set(focusTargets.map(target => target.getAttribute('role')))],
                legendEntries: legend ? [...legend.querySelectorAll('[data-series-id]')].map(entry => entry.textContent.trim()) : [],
                legendSwatches: legend ? legend.querySelectorAll('[data-line-chart-pattern-swatch]').length : 0,
                caption: table?.querySelector('caption')?.textContent.trim() ?? null,
                bodyRows: table?.querySelectorAll('tbody tr').length ?? 0,
                seriesColumns: table?.querySelectorAll('thead th').length ?? 0,
                weekOneCells: rowCells(0),
                weekSevenCells: rowCells(6),
                weekElevenCells: rowCells(10),
                ids,
                labelledBy,
                labelledByResolve: labelledBy.every(id => svg.querySelector(`#${CSS.escape(id)}`) !== null),
                finiteSvg: !/NaN|Infinity/.test(svg.outerHTML),
                emptyCount: root.querySelectorAll('[data-line-chart-empty]').length,
            };
        })()"#,
    )
    .await;

    assert!(
        !snapshot.is_null(),
        "categorical chart wrapper must exist: {snapshot}"
    );
    assert_eq!(
        snapshot["rootCount"],
        json!(2),
        "the showcase renders the interactive fixture plus the callback-less one: {snapshot}"
    );
    assert_eq!(
        snapshot["plotCount"],
        json!(1),
        "one categorical SVG plot: {snapshot}"
    );
    assert_eq!(
        snapshot["series"],
        json!([
            { "id": "actual", "dash": null },
            { "id": "rolling-average", "dash": "6 4" },
            { "id": "target", "dash": "2 3" },
        ]),
        "three distinct series retain their solid/dashed/dotted identities: {snapshot}"
    );
    assert_eq!(
        snapshot["actualSegments"],
        json!(2),
        "the actual series splits into two paths around its missing week: {snapshot}"
    );
    assert_eq!(
        snapshot["actualMarker"],
        json!("circle"),
        "actual uses circles: {snapshot}"
    );
    assert_eq!(
        snapshot["averageMarker"],
        json!("rect"),
        "rolling average uses squares: {snapshot}"
    );
    assert_eq!(
        snapshot["targetMarker"],
        json!("path"),
        "target uses diamonds: {snapshot}"
    );
    assert_eq!(
        snapshot["markerCounts"],
        json!({ "actual": 13, "average": 14, "target": 10 }),
        "marker counts retain the gap and short-series padding: {snapshot}"
    );
    assert_eq!(
        snapshot["firstActualMarkerFill"],
        json!("var(--color-success)"),
        "per-point marker color overrides the series marker fill: {snapshot}"
    );
    assert_eq!(
        snapshot["missingActual"],
        json!(true),
        "missing actual point remains a gap: {snapshot}"
    );
    assert_eq!(
        snapshot["focusCount"],
        json!(14),
        "every finite category has one static focus target: {snapshot}"
    );
    assert_eq!(
        snapshot["tabstopCount"],
        json!(1),
        "the finite initial category owns the only tab stop: {snapshot}"
    );
    assert_eq!(
        snapshot["focusRoles"],
        json!(["button"]),
        "with an activation callback wired, targets expose button semantics: {snapshot}"
    );
    assert_eq!(
        snapshot["legendEntries"],
        json!(["Actual", "Rolling average", "Target"])
    );
    assert_eq!(
        snapshot["legendSwatches"],
        json!(3),
        "legend retains patterned swatches: {snapshot}"
    );
    assert_eq!(snapshot["caption"], json!("Weekly resolution trend"));
    assert_eq!(
        snapshot["bodyRows"],
        json!(14),
        "one table row per category: {snapshot}"
    );
    assert_eq!(
        snapshot["seriesColumns"],
        json!(4),
        "category plus three series columns: {snapshot}"
    );
    assert_eq!(
        snapshot["weekOneCells"],
        json!(["W01", "42 resolved", "43.0 average", "Target 48"]),
        "formatted display values reach the hidden table: {snapshot}"
    );
    assert_eq!(
        snapshot["weekSevenCells"],
        json!(["W07", "No value", "51.0 average", "Target 54"]),
        "a categorical gap is announced as No value: {snapshot}"
    );
    assert_eq!(
        snapshot["weekElevenCells"],
        json!(["W11", "61 resolved", "59.0 average", "No value"]),
        "short-series padding is announced as No value: {snapshot}"
    );
    assert_eq!(
        snapshot["ids"].as_array().map(Vec::len),
        snapshot["ids"]
            .as_array()
            .map(|ids| ids.iter().collect::<std::collections::HashSet<_>>().len()),
        "SVG ARIA ids are unique: {snapshot}"
    );
    assert_eq!(
        snapshot["labelledByResolve"],
        json!(true),
        "aria-labelledby resolves to the chart title and description: {snapshot}"
    );
    assert_eq!(
        snapshot["finiteSvg"],
        json!(true),
        "serialized SVG must not contain NaN or Infinity: {snapshot}"
    );
    assert_eq!(
        snapshot["emptyCount"],
        json!(0),
        "populated fixture must not render empty state: {snapshot}"
    );
}

/// Pointer overlay of the interactive (first) categorical chart.
const CHART_OVERLAY: &str =
    "[data-testid=\"interactive-line-chart\"] [data-line-chart-pointer-overlay]";
/// The interactive chart's roving tab stop (whichever target holds it).
const CHART_TAB_STOP: &str =
    "[data-testid=\"interactive-line-chart\"] [data-line-chart-focus][tabindex=\"0\"]";
/// Horizontal overlay fraction of category index 7 (`week-08`) in the
/// 14-category fixture: index / (count - 1).
const WEEK_08_X: f64 = 7.0 / 13.0;

/// Let focus/key-driven reducer updates and the rAF-deferred tooltip
/// placement land before reading state back.
async fn settle(h: &pixelproof_web::Harness) {
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

/// Snapshot of the interactive chart's live interaction surface: root state
/// attributes, tooltip visibility/content, and the focused element.
async fn line_chart_state(h: &pixelproof_web::Harness) -> serde_json::Value {
    eval_json(
        h,
        r#"(() => {
            const root = document.querySelector('[data-testid="interactive-line-chart"]');
            const stage = root.querySelector('[data-line-chart-stage]');
            const tip = root.querySelector('[data-testid="line-chart-tooltip"]');
            const tipStyle = tip ? getComputedStyle(tip) : null;
            const tipVisible = !!tipStyle && tipStyle.display !== 'none' && tipStyle.visibility !== 'hidden';
            const tipRect = tip ? tip.getBoundingClientRect() : null;
            const stageRect = stage.getBoundingClientRect();
            const rows = tip ? [...tip.querySelectorAll('[data-series-id]')].map(row => ({
                id: row.dataset.seriesId,
                preferred: row.dataset.preferred,
                text: row.textContent.trim(),
            })) : [];
            const active = document.activeElement;
            return {
                activeCategory: root.dataset.activeCategory ?? null,
                preferredSeries: root.dataset.preferredSeries ?? null,
                tooltipVisible: tipVisible,
                tooltipCategory: tip?.querySelector('div')?.textContent.trim() ?? null,
                rows,
                tooltipWithinStage: !tipVisible || (tipRect.left >= stageRect.left - 1
                    && tipRect.right <= stageRect.right + 1
                    && tipRect.top >= stageRect.top - 1
                    && tipRect.bottom <= stageRect.bottom + 1),
                tabStops: root.querySelectorAll('[data-line-chart-focus][tabindex="0"]').length,
                focusedCategoryKey: active?.dataset?.categoryKey ?? null,
                focusedIsChartTarget: !!active && active.hasAttribute('data-line-chart-focus'),
            };
        })()"#,
    )
    .await
}

/// Journey 1 (ldui-9tr.5): hovering a category shows exactly one card with
/// the category label, every finite series row in input order, one preferred
/// row, and matching root state.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_hover_shows_category_card() {
    let h = harness_at("/components/charts").await;
    common::wait_for_selector(&h, CHART_OVERLAY).await;
    begin_browser_error_capture(&h).await;

    move_pointer_to_svg_fraction(&h, CHART_OVERLAY, WEEK_08_X, 0.5).await;
    let s = line_chart_state(&h).await;
    assert_eq!(s["tooltipVisible"], json!(true), "card visible: {s}");
    assert_eq!(s["tooltipCategory"], json!("W08"), "category label: {s}");
    assert_eq!(
        s["activeCategory"],
        json!("week-08"),
        "root active state: {s}"
    );
    let rows = s["rows"].as_array().expect("rows");
    assert_eq!(
        rows.iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["actual", "rolling-average", "target"],
        "all finite series in input order: {s}"
    );
    assert!(
        rows[0]["text"].as_str().unwrap().contains("55 resolved")
            && rows[1]["text"].as_str().unwrap().contains("53.0 average")
            && rows[2]["text"].as_str().unwrap().contains("Target 54"),
        "host display strings reach the card: {s}"
    );
    let preferred: Vec<_> = rows
        .iter()
        .filter(|r| r["preferred"] == json!("true"))
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(preferred.len(), 1, "exactly one preferred row: {s}");
    assert_eq!(
        s["preferredSeries"].as_str().unwrap(),
        preferred[0],
        "root preferred-series matches the highlighted row: {s}"
    );

    assert_no_browser_errors(&h, "hover journey").await;
}

/// Journey 2: at the first and last categories the placed card stays within
/// the chart stage (edge clamping).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_tooltip_stays_within_stage_at_edges() {
    let h = harness_at("/components/charts").await;
    common::wait_for_selector(&h, CHART_OVERLAY).await;
    begin_browser_error_capture(&h).await;

    for (x, expected) in [(0.001, "week-01"), (0.999, "week-14")] {
        move_pointer_to_svg_fraction(&h, CHART_OVERLAY, x, 0.3).await;
        let s = line_chart_state(&h).await;
        assert_eq!(
            s["activeCategory"],
            json!(expected),
            "edge hover at {x}: {s}"
        );
        assert_eq!(s["tooltipVisible"], json!(true), "edge card visible: {s}");
        assert_eq!(
            s["tooltipWithinStage"],
            json!(true),
            "card clamped inside the stage at {x}: {s}"
        );
    }

    assert_no_browser_errors(&h, "edge clamping journey").await;
}

/// Journey 3: one roving tab stop; ArrowRight/End/Home move focus within the
/// composite and Escape dismisses the card without losing focus.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_keyboard_roving_navigation() {
    let h = harness_at("/components/charts").await;
    common::wait_for_selector(&h, CHART_OVERLAY).await;
    begin_browser_error_capture(&h).await;

    let s = line_chart_state(&h).await;
    assert_eq!(s["tabStops"], json!(1), "exactly one roving tab stop: {s}");

    // Enter the composite on its single tab stop (real focus, then real keys).
    h.page()
        .find_element(CHART_TAB_STOP)
        .await
        .expect("find tab stop")
        .focus()
        .await
        .expect("focus tab stop");
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(
        s["focusedCategoryKey"],
        json!("week-01"),
        "entry focus: {s}"
    );
    assert_eq!(
        s["tooltipVisible"],
        json!(true),
        "focus shows the card: {s}"
    );

    h.press_key_sequence(&[Key::ArrowRight])
        .await
        .expect("arrow right");
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(s["focusedCategoryKey"], json!("week-02"), "ArrowRight: {s}");

    h.press_key_sequence(&[Key::End]).await.expect("end");
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(s["focusedCategoryKey"], json!("week-14"), "End: {s}");

    h.press_key_sequence(&[Key::Home]).await.expect("home");
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(s["focusedCategoryKey"], json!("week-01"), "Home: {s}");
    assert_eq!(s["tabStops"], json!(1), "roving keeps one tab stop: {s}");

    h.press_key_sequence(&[Key::Escape]).await.expect("escape");
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(s["tooltipVisible"], json!(false), "Escape dismisses: {s}");
    assert_eq!(
        s["focusedCategoryKey"],
        json!("week-01"),
        "Escape keeps focus in the composite: {s}"
    );

    assert_no_browser_errors(&h, "keyboard journey").await;
}

/// Journey 4: click, Enter, and Space each emit exactly one typed activation
/// with category key, preferred series, finite values, source, and false
/// modifiers — counted so a duplicated callback cannot hide.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_click_enter_space_activate_once_each() {
    let h = harness_at("/components/charts").await;
    common::wait_for_selector(&h, CHART_OVERLAY).await;
    begin_browser_error_capture(&h).await;

    click_svg_fraction(&h, CHART_OVERLAY, WEEK_08_X, 0.5).await;
    let s = oracle(&h).await;
    assert_eq!(
        s["state"]["chart.activation_count"],
        json!(1),
        "one click, one activation: {s}"
    );
    let activation = &s["state"]["chart.activation"];
    assert_eq!(
        activation["categoryKey"],
        json!("week-08"),
        "payload key: {activation}"
    );
    assert_eq!(
        activation["source"],
        json!("pointer"),
        "payload source: {activation}"
    );
    assert_eq!(
        activation["modifiers"],
        json!({"shift": false, "ctrl": false, "alt": false, "meta": false}),
        "unmodified activation: {activation}"
    );
    let values = activation["values"].as_array().expect("values");
    assert_eq!(
        values.len(),
        3,
        "all finite series in the payload: {activation}"
    );
    assert!(
        values
            .iter()
            .all(|value| value["value"].as_f64().is_some_and(f64::is_finite)),
        "finite values only: {activation}"
    );
    assert!(
        activation["preferredSeriesId"].is_string(),
        "preferred series present: {activation}"
    );

    // Keyboard: Enter then Space on the focused target each add exactly one.
    h.page()
        .find_element(CHART_TAB_STOP)
        .await
        .expect("find tab stop")
        .focus()
        .await
        .expect("focus tab stop");
    settle(&h).await;
    h.press_key_sequence(&[Key::Enter]).await.expect("enter");
    settle(&h).await;
    let s = oracle(&h).await;
    assert_eq!(
        s["state"]["chart.activation_count"],
        json!(2),
        "Enter adds one: {s}"
    );
    assert_eq!(
        s["state"]["chart.activation"]["source"],
        json!("keyboard"),
        "keyboard source: {s}"
    );
    h.press_key_sequence(&[Key::Space]).await.expect("space");
    settle(&h).await;
    let s = oracle(&h).await;
    assert_eq!(
        s["state"]["chart.activation_count"],
        json!(3),
        "Space adds one: {s}"
    );

    assert_no_browser_errors(&h, "activation journey").await;
}

/// Journey 5: reconciliation by key — a reorder keeps the active category by
/// key; removing it closes the card, moves focus to a neighbouring valid
/// category, and fires no activation.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_reconciles_active_state_across_data_changes() {
    let h = harness_at("/components/charts").await;
    common::wait_for_selector(&h, CHART_OVERLAY).await;
    begin_browser_error_capture(&h).await;

    // Focus week-08 by roving there (real keys), so the active state under
    // reconciliation is the focus flavor — a programmatic control click
    // below does not steal focus the way a real button click would.
    h.page()
        .find_element(CHART_TAB_STOP)
        .await
        .expect("find tab stop")
        .focus()
        .await
        .expect("focus tab stop");
    settle(&h).await;
    for _ in 0..7 {
        h.press_key_sequence(&[Key::ArrowRight])
            .await
            .expect("arrow right");
    }
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(
        s["focusedCategoryKey"],
        json!("week-08"),
        "arrived at week-08: {s}"
    );
    assert_eq!(
        s["activeCategory"],
        json!("week-08"),
        "card follows focus: {s}"
    );

    // Reorder: the active category keeps its key, not its index.
    eval_json(
        &h,
        "(document.querySelector('[data-testid=\"line-chart-reorder\"]').click(), true)",
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    let s = line_chart_state(&h).await;
    assert_eq!(
        s["activeCategory"],
        json!("week-08"),
        "reorder reconciles the active category by key: {s}"
    );

    // Remove the active key: card closes, focus lands on a neighbouring
    // valid category, and no activation fires.
    eval_json(
        &h,
        "(document.querySelector('[data-testid=\"line-chart-remove\"]').click(), true)",
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    let s = line_chart_state(&h).await;
    assert_eq!(
        s["tooltipVisible"],
        json!(false),
        "card closes with its key: {s}"
    );
    assert_eq!(
        s["focusedIsChartTarget"],
        json!(true),
        "focus stays in the chart: {s}"
    );
    let neighbour = s["focusedCategoryKey"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        neighbour == "week-07" || neighbour == "week-09",
        "focus moves to a neighbouring valid category, got {neighbour}: {s}"
    );
    let o = oracle(&h).await;
    assert!(
        o["state"]["chart.activation_count"].is_null(),
        "no activation fires from reconciliation: {o}"
    );

    // Restore for any later state readers.
    eval_json(
        &h,
        "(document.querySelector('[data-testid=\"line-chart-restore\"]').click(), true)",
    )
    .await;

    assert_no_browser_errors(&h, "reconciliation journey").await;
}

/// Semantic accessibility graph (ldui-9tr.6): named group root, labelled
/// SVG, roving tab stop, value-bearing target names, activation-conditional
/// button roles, tooltip description wiring, and a non-color focus cue.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_semantic_graph() {
    let h = harness_at("/components/charts").await;
    common::wait_for_selector(&h, CHART_OVERLAY).await;

    let s = eval_json(
        &h,
        r#"(() => {
            const roots = [...document.querySelectorAll('[data-testid="interactive-line-chart"]')];
            const graph = roots.map(root => {
                const svg = root.querySelector('[data-line-chart-plot]');
                const labelledBy = (svg.getAttribute('aria-labelledby') ?? '').split(/\s+/).filter(Boolean);
                const targets = [...svg.querySelectorAll('[data-line-chart-focus]')];
                return {
                    role: root.getAttribute('role'),
                    named: (root.getAttribute('aria-label') ?? '').length > 0,
                    svgRole: svg.getAttribute('role'),
                    labelledByCount: labelledBy.length,
                    labelledByUnique: new Set(labelledBy).size === labelledBy.length,
                    labelledByResolve: labelledBy.every(id => svg.querySelector('#' + CSS.escape(id)) !== null),
                    targetRoles: [...new Set(targets.map(t => t.getAttribute('role')))],
                    tabStops: targets.filter(t => t.getAttribute('tabindex') === '0').length,
                    firstTargetName: targets[0]?.getAttribute('aria-label') ?? '',
                };
            });
            return { graph };
        })()"#,
    )
    .await;
    let graph = s["graph"].as_array().expect("graph");
    assert_eq!(graph.len(), 2, "two categorical fixtures: {s}");
    for (index, chart) in graph.iter().enumerate() {
        assert_eq!(
            chart["role"],
            json!("group"),
            "chart {index} root role: {s}"
        );
        assert_eq!(chart["named"], json!(true), "chart {index} root named: {s}");
        // "group", not "img": role=img would make the focusable category
        // targets inside presentational (axe nested-interactive).
        assert_eq!(
            chart["svgRole"],
            json!("group"),
            "chart {index} svg role: {s}"
        );
        assert_eq!(
            chart["labelledByCount"],
            json!(2),
            "chart {index} title+desc: {s}"
        );
        assert_eq!(
            chart["labelledByUnique"],
            json!(true),
            "chart {index} unique refs: {s}"
        );
        assert_eq!(
            chart["labelledByResolve"],
            json!(true),
            "chart {index} refs resolve: {s}"
        );
        assert_eq!(
            chart["tabStops"],
            json!(1),
            "chart {index} one roving stop: {s}"
        );
    }
    assert_eq!(
        graph[0]["targetRoles"],
        json!(["button"]),
        "activation-configured chart exposes button targets: {s}"
    );
    assert_eq!(
        graph[1]["targetRoles"],
        json!(["group"]),
        "callback-less chart keeps descriptive groups, no false buttons: {s}"
    );
    // Target names carry the category and every available series value.
    let name = graph[0]["firstTargetName"].as_str().unwrap_or_default();
    assert!(
        name.contains("W01")
            && name.contains("42 resolved")
            && name.contains("43.0 average")
            && name.contains("Target 48"),
        "target name carries category + all series values, got {name:?}"
    );

    // The active target's aria-describedby points at the one visible tooltip,
    // and keyboard focus changes a non-color cue (the focus ring's width).
    h.page()
        .find_element(CHART_TAB_STOP)
        .await
        .expect("find tab stop")
        .focus()
        .await
        .expect("focus tab stop");
    settle(&h).await;
    let s = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('[data-testid="interactive-line-chart"]');
            const active = document.activeElement;
            const tipId = active?.getAttribute('aria-describedby');
            const tip = tipId ? document.getElementById(tipId) : null;
            const tipStyle = tip ? getComputedStyle(tip) : null;
            const describedTargets = [...root.querySelectorAll('[data-line-chart-focus][aria-describedby]')];
            return {
                hasDescription: !!tipId,
                describesVisibleTooltip: !!tipStyle && tip.getAttribute('role') === 'tooltip'
                    && tipStyle.display !== 'none' && tipStyle.visibility !== 'hidden',
                describedCount: describedTargets.length,
                focusedRingWidth: active?.getAttribute('stroke-width') ?? null,
                unfocusedRingWidth: [...root.querySelectorAll('[data-line-chart-focus]')]
                    .filter(t => t !== active)[0]?.getAttribute('stroke-width') ?? null,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        s["hasDescription"],
        json!(true),
        "active target described: {s}"
    );
    assert_eq!(
        s["describesVisibleTooltip"],
        json!(true),
        "description resolves to the visible tooltip: {s}"
    );
    assert_eq!(
        s["describedCount"],
        json!(1),
        "only the active target carries the description: {s}"
    );
    assert_eq!(s["focusedRingWidth"], json!("2"), "focus ring appears: {s}");
    assert_eq!(
        s["unfocusedRingWidth"],
        json!("0"),
        "others stay ringless: {s}"
    );
}

/// axe-core gate (ldui-9tr.6): zero Serious/Critical violations on the
/// charts page, with the vendored engine (no network at test time).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_axe_clean() {
    let h = harness_at("/components/charts").await;
    common::wait_for_selector(&h, CHART_OVERLAY).await;

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(h.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("interactive-line-chart")
        .unwrap_or_else(|e| {
            panic!(
                "{e}; {}
violations: {:#?}",
                report.summary(),
                report.violations
            )
        });
}

/// Tab entry, arrow traversal containment, and Tab exit (ldui-9tr.6): the
/// composite is one stop in the page tab order; arrows move inside it
/// without adding stops; the next Tab leaves the chart entirely. The
/// callback-less fixture's Enter stays inert.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn line_chart_tab_entry_traversal_and_exit() {
    let h = harness_at("/components/charts").await;
    common::wait_for_selector(&h, CHART_OVERLAY).await;
    begin_browser_error_capture(&h).await;

    h.page()
        .find_element(CHART_TAB_STOP)
        .await
        .expect("find tab stop")
        .focus()
        .await
        .expect("focus tab stop");
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(
        s["focusedIsChartTarget"],
        json!(true),
        "entered composite: {s}"
    );

    h.press_key_sequence(&[Key::ArrowRight, Key::ArrowRight])
        .await
        .expect("arrows");
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(
        s["focusedCategoryKey"],
        json!("week-03"),
        "arrows stay inside: {s}"
    );
    assert_eq!(s["tabStops"], json!(1), "still one stop: {s}");

    h.press_key_sequence(&[Key::Tab]).await.expect("tab out");
    settle(&h).await;
    let s = line_chart_state(&h).await;
    assert_eq!(
        s["focusedIsChartTarget"],
        json!(false),
        "Tab leaves the composite instead of visiting more categories: {s}"
    );

    // The callback-less fixture: Enter on its target activates nothing.
    eval_json(
        &h,
        r#"([...document.querySelectorAll('[data-testid="interactive-line-chart"]')][1]
            .querySelector('[data-line-chart-focus][tabindex="0"]').focus(), true)"#,
    )
    .await;
    settle(&h).await;
    h.press_key_sequence(&[Key::Enter]).await.expect("enter");
    settle(&h).await;
    let o = oracle(&h).await;
    assert!(
        o["state"]["chart.activation_count"].is_null(),
        "a callback-less chart's Enter is inert: {o}"
    );

    assert_no_browser_errors(&h, "tab entry/exit journey").await;
}

/// Controlled custom filter (beads-je5r / `extra_filter` + `toolbar`): the
/// demo's "Admins only" toggle lives in the toolbar slot and its predicate
/// ANDs with the built-in filters. Toggling narrows the 25-row table to its
/// 5 Admin rows and back, with the built-in toolbar (search box) intact.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn extra_filter_composes_with_builtin_toolbar() {
    let h = harness_at("/components/data-table").await;
    let rows = "#custom-filter-table tbody tr";

    assert_eq!(
        eval_json(
            &h,
            "document.querySelector('#custom-filter-table').dataset.tableDataMode",
        )
        .await,
        json!("compatibility-client"),
        "the dynamic HashMap table is an explicit compatibility client mode",
    );

    assert_eq!(count_of(&h, rows).await, 25, "all rows visible initially");
    // The built-in search box must be present alongside the custom control.
    assert_eq!(
        count_of(
            &h,
            "#custom-filter-table input[aria-label=\"Search table\"]"
        )
        .await,
        1,
        "toolbar slot must compose with, not replace, the built-in toolbar"
    );

    click(&h, "#admins-only-toggle").await;
    assert_eq!(count_of(&h, rows).await, 5, "only Admin rows remain");

    click(&h, "#admins-only-toggle").await;
    assert_eq!(count_of(&h, rows).await, 25, "toggle off restores all rows");
}

/// Evaluate a JS expression returning a JSON-serializable value.
async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> serde_json::Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate")
        .into_value()
        .expect("json value")
}

/// Field association (ldui-a8p): wrapping this crate's Input in a Field
/// yields real programmatic association — label[for] == input[id], the help
/// line's id is in aria-describedby, and flipping to the error state moves
/// the reference to aria-errormessage + aria-invalid="true".
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn field_wires_label_help_and_error_to_the_input() {
    let h = harness_at("/components/fieldset").await;

    let snapshot = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('#field-assoc');
            const input = root.querySelector('input');
            const label = root.querySelector('label.label');
            return {
                id: input.id,
                for_: label.getAttribute('for'),
                describedby: input.getAttribute('aria-describedby'),
                errormessage: input.getAttribute('aria-errormessage'),
                invalid: input.getAttribute('aria-invalid'),
                lineText: document.getElementById(input.getAttribute('aria-describedby'))?.textContent ?? null,
            };
        })()"#,
    )
    .await;
    assert!(
        snapshot["id"].as_str().is_some_and(|s| !s.is_empty()),
        "input must carry the Field-minted id: {snapshot}"
    );
    assert_eq!(
        snapshot["for_"], snapshot["id"],
        "label[for] must point at the input: {snapshot}"
    );
    assert!(
        snapshot["lineText"]
            .as_str()
            .is_some_and(|t| t.contains("YY-NNNNN")),
        "aria-describedby must resolve to the help line: {snapshot}"
    );
    assert!(
        snapshot["errormessage"].is_null() && snapshot["invalid"].is_null(),
        "no error attributes while the field is valid: {snapshot}"
    );

    click(&h, "#field-assoc-toggle").await;
    let snapshot = eval_json(
        &h,
        r#"(() => {
            const input = document.querySelector('#field-assoc input');
            return {
                errormessage: input.getAttribute('aria-errormessage'),
                invalid: input.getAttribute('aria-invalid'),
                errText: document.getElementById(input.getAttribute('aria-errormessage'))?.textContent ?? null,
            };
        })()"#,
    )
    .await;
    assert_eq!(snapshot["invalid"], serde_json::json!("true"));
    assert!(
        snapshot["errText"]
            .as_str()
            .is_some_and(|t| t.contains("required")),
        "aria-errormessage must resolve to the rendered error line: {snapshot}"
    );
}

/// Modal accessible naming (ldui-nui): the demo's basic modal names itself
/// via aria-labelledby pointing at its visible heading (and describes via
/// aria-describedby); the hardcoded aria-label="Modal" is gone from it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn modal_is_named_by_its_visible_heading() {
    let h = harness_at("/components/modal").await;

    let snapshot = eval_json(
        &h,
        r#"(() => {
            const dialog = document.querySelector('dialog[aria-labelledby="basic-modal-title"]');
            if (!dialog) return null;
            return {
                label: dialog.getAttribute('aria-label'),
                heading: document.getElementById('basic-modal-title')?.textContent ?? null,
                desc: document.getElementById(dialog.getAttribute('aria-describedby'))?.textContent ?? null,
            };
        })()"#,
    )
    .await;
    assert!(
        !snapshot.is_null(),
        "the basic modal must carry aria-labelledby"
    );
    assert!(
        snapshot["label"].is_null(),
        "aria-label must be suppressed when labelledby names the dialog: {snapshot}"
    );
    assert_eq!(snapshot["heading"], serde_json::json!("Hello!"));
    assert!(
        snapshot["desc"].as_str().is_some_and(|t| t.contains("ESC")),
        "aria-describedby must resolve to the summary text: {snapshot}"
    );
}

/// DayScheduler keyboard contract (ldui-j6s): clicking an event selects and
/// activates it; ArrowDown on the focused block requests a +15-minute move,
/// which the demo applies — and because event blocks are keyed by index (not
/// by their times), the SAME node keeps focus, so a second press moves again.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn day_scheduler_keyboard_moves_event() {
    let h = harness_at("/components/day-scheduler").await;
    let block = "#interactive-scheduler [role=\"button\"]";

    click(&h, block).await;
    assert_eq!(testid_text(&h, "sched-selected").await, "0");
    assert_eq!(testid_text(&h, "sched-activated").await, "0");
    assert_eq!(
        testid_text(&h, "sched-first-times").await,
        "540-600",
        "Intake review starts at 09:00"
    );

    h.press_key_sequence(&[Key::ArrowDown]).await.expect("key");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert_eq!(
        testid_text(&h, "sched-first-times").await,
        "555-615",
        "ArrowDown must move the focused event 15 minutes later"
    );

    // Focus survived the move (index-keyed node): the next press works too.
    h.press_key_sequence(&[Key::ArrowDown]).await.expect("key");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert_eq!(testid_text(&h, "sched-first-times").await, "570-630");
}

/// ServerDataTable typed query API (beads-uy2r / `on_query_change`): a header
/// click on the demo's server-owned table emits a TableQuery carrying the
/// sort (previously a no-op), and page navigation emits the new page — the
/// simulated backend re-fetches from it, so the rendered page follows.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn server_table_round_trips_typed_query() {
    let h = harness_at("/components/data-table").await;

    assert_eq!(
        eval_json(
            &h,
            "document.querySelector('#server-table').dataset.tableDataMode",
        )
        .await,
        json!("server-query"),
        "server-owned rows must never look like a complete client snapshot",
    );

    assert_eq!(
        count_of(&h, "#server-table tbody tr").await,
        10,
        "initial fetch renders one 10-row page"
    );

    click(&h, "#server-table thead th:first-child").await;
    let q = testid_text(&h, "server-last-query").await;
    assert!(
        q.contains("sort=Some((\"name\", Asc))") && q.contains("page=1"),
        "header sort must round-trip through the query: {q}"
    );

    // The Next button is the last button of the pagination join strip.
    click(&h, "#server-table .join > button:last-child").await;
    let q = testid_text(&h, "server-last-query").await;
    assert!(
        q.contains("page=2"),
        "page navigation must round-trip through the query: {q}"
    );
}

/// Server-variant activation forwarding (ldui-1gp): ServerDataTable passes
/// `on_row_activate`/`on_row_inspect` through to the shared body, so its
/// rows carry the keyboard contract and a plain click activates with the
/// page-local index — previously the server variant dropped both callbacks
/// entirely.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn server_table_forwards_activation() {
    let h = harness_at("/components/data-table").await;

    assert_eq!(
        count_of(&h, "#server-table tbody tr[tabindex=\"0\"]").await,
        10,
        "activation-wired server rows are keyboard-operable"
    );
    assert_eq!(
        testid_text(&h, "server-activated-row").await,
        "(none)",
        "no activation before any click"
    );

    click(&h, "#server-table tbody tr:nth-child(2) td:first-child").await;
    assert_eq!(
        testid_text(&h, "server-activated-row").await,
        "1",
        "a plain click activates with the page-local row index"
    );
    // The dblclick/Shift+Enter inspector rides the shared body's already
    // browser-proven discrimination (ldui-tmr); its wiring is exercised by
    // the single-click path above reaching the same forwarded pair.
}

/// Unmount safety (ldui-d54): DataTable's zero-delay auto-rows measure timer
/// and both search debounces (client + server variants) must not fire into a
/// disposed reactive owner. Arms all three timers and navigates away in the
/// same task — before the fix, a late timer panicked the entire wasm app
/// ("Tried to access a reactive value that has already been disposed"),
/// which 4iiz-etl's visual gate only dodged by pacing navigations 150 ms
/// apart.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_timers_survive_unmount() {
    let h = harness_at("/components/data-table").await;
    begin_browser_error_capture(&h).await;

    // One synchronous task: arm the client and server search debounces
    // (300 ms each), force a data change (the auto-rows Effect schedules its
    // ZERO-delay measure macrotask), then navigate — so every pending timer
    // fires only after the DataTable's owner is disposed.
    let armed = eval_json(
        &h,
        r#"(() => {
            const arm = (sel) => {
                const input = document.querySelector(sel);
                if (!input) return false;
                input.value = 'x';
                input.dispatchEvent(new Event('input', { bubbles: true }));
                return true;
            };
            const client = arm('#custom-filter-table input[aria-label="Search table"]');
            const server = arm('#server-table input[aria-label="Search table"]');
            document.querySelector('#keyed-reverse').click();
            document.querySelector('a[href="/components/button"]').click();
            return client && server;
        })()"#,
    )
    .await;
    assert_eq!(armed, json!(true), "both search boxes found and armed");

    // Let the 0 ms measure timer and both 300 ms debounces fire post-unmount.
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let s = oracle(&h).await;
    assert_eq!(
        s["route"],
        json!("/components/button"),
        "navigation away happened: {s}"
    );
    assert_no_browser_errors(&h, "DataTable timers firing after unmount").await;
}

/// Keyed row identity (beads-py7i / `row_key`): select a row, then replace
/// the data vec (the demo's Reverse button). The selection must follow the
/// row's stable id to its new position rather than clearing (the positional
/// behaviour) or sticking to the old index (a different row).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn keyed_selection_survives_data_replacement() {
    let h = harness_at("/components/data-table").await;

    click(&h, "#keyed-table tbody tr:first-child td:first-child").await;
    assert_eq!(
        testid_text(&h, "keyed-selected-ids").await,
        "001",
        "plain click must select the first row (id 001)"
    );

    click(&h, "#keyed-reverse").await;
    assert_eq!(
        testid_text(&h, "keyed-selected-ids").await,
        "001",
        "the selected id must survive the data replacement"
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

    // The DOM agrees with the oracle: html[data-theme="cupcake"]. Read the
    // attribute rather than substring-matching the serialized tag — the tag
    // also carries lang="en" now, and attribute order made the old literal
    // prefix match break the moment another attribute appeared.
    let dom_theme: String = h
        .page()
        .evaluate("document.documentElement.getAttribute('data-theme')")
        .await
        .expect("evaluate html data-theme")
        .into_value()
        .expect("data-theme is a string");
    assert_eq!(dom_theme, "cupcake", "html[data-theme] should be cupcake");
}
