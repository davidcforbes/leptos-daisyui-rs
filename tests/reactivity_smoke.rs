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
            const root = document.querySelector('[data-testid="interactive-line-chart"]');
            if (!root) return null;
            const series = [...new Map([...root.querySelectorAll('path[data-series-id]:not([data-category-index])')].map(path => [
                path.dataset.seriesId,
                { id: path.dataset.seriesId, dash: path.getAttribute('stroke-dasharray') },
            ])).values()];
            const marker = (selector) => root.querySelector(selector)?.tagName.toLowerCase() ?? null;
            const legend = root.querySelector('[data-line-chart-legend]');
            const table = root.querySelector('[data-line-chart-table]');
            return {
                plotCount: root.querySelectorAll('[data-line-chart-plot]').length,
                series,
                actualMarker: marker('circle[data-series-id="actual"][data-category-index="0"]'),
                averageMarker: marker('rect[data-series-id="rolling-average"][data-category-index="0"]'),
                targetMarker: marker('path[data-series-id="target"][data-category-index="0"]'),
                missingActual: root.querySelector('[data-series-id="actual"][data-category-key="week-07"]') === null,
                legendEntries: legend ? [...legend.querySelectorAll('[data-series-id]')].map(entry => entry.textContent.trim()) : [],
                legendSwatches: legend ? legend.querySelectorAll('[data-line-chart-pattern-swatch]').length : 0,
                caption: table?.querySelector('caption')?.textContent.trim() ?? null,
                bodyRows: table?.querySelectorAll('tbody tr').length ?? 0,
                seriesColumns: table?.querySelectorAll('thead th').length ?? 0,
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
        snapshot["missingActual"],
        json!(true),
        "missing actual point remains a gap: {snapshot}"
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
        snapshot["emptyCount"],
        json!(0),
        "populated fixture must not render empty state: {snapshot}"
    );
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

    // The DOM agrees with the oracle: <html data-theme="cupcake">.
    let dom = h.dom_html().await.expect("dom");
    assert!(
        dom.contains(r#"<html data-theme="cupcake""#),
        "html[data-theme] should be cupcake"
    );
}
