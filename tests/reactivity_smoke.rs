//! Reactivity smoke suite (ldui-49w.1) — drives real CDP input against the
//! demo app and asserts internal Leptos state through the
//! `window.__APP_DEBUG__` oracle (ldui-49w.3), not through pixels.
//!
//! No screenshots, so this suite is deterministic across machines and is
//! **selectively gated** — its 32 checks run only when explicitly requested via
//! `cargo xtask test-reactivity` or the requested `cargo xtask verify-full`.
//! They are deliberately absent from the ordinary `cargo xtask verify`
//! rebuild. (Its sibling `visual_smoke.rs` compares pixels against baselines
//! and stays manual; see `doc/ci-cd.md`.)
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
use pixelproof_web::{Key, ViewportSize};
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
    let header = "#keyboard-sort-table thead th:first-child > button";

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

/// Phase 0B keyboard/a11y contract (ldui-w1e): shared client and server table
/// sorting is one native-button activation path for pointer, Enter, and Space.
/// Non-sortable headers expose no sort control, while their independent resize
/// separator remains keyboard reachable and cannot change sort state.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_sort_is_keyboard_operable_for_client_and_server_tables() {
    let h = harness_at("/components/data-table").await;
    begin_browser_error_capture(&h).await;

    let client_sort = "#keyboard-sort-table thead tr:first-child th:first-child > button";
    let server_sort = "#server-table thead tr:first-child th:first-child > button";
    let structure = eval_json(
        &h,
        r#"(() => {
            const client = document.querySelector('#keyboard-sort-table');
            const server = document.querySelector('#server-table');
            const mixedStatus = document.querySelector(
                '#mixed-sort-table thead tr:first-child th:nth-child(4)'
            );
            const clientButton = client?.querySelector(
                'thead tr:first-child th:first-child > button'
            );
            const serverButton = server?.querySelector(
                'thead tr:first-child th:first-child > button'
            );
            return {
                clientButtons: client?.querySelectorAll(
                    'thead tr:first-child th:first-child > button'
                ).length ?? 0,
                serverButtons: server?.querySelectorAll(
                    'thead tr:first-child th:first-child > button'
                ).length ?? 0,
                clientName: clientButton?.textContent.trim() ?? '',
                serverName: serverButton?.textContent.trim() ?? '',
                clientAccessibleName: clientButton?.getAttribute('aria-label') ?? null,
                serverAccessibleName: serverButton?.getAttribute('aria-label') ?? null,
                clientAriaSort: clientButton?.parentElement?.getAttribute('aria-sort') ?? null,
                serverAriaSort: serverButton?.parentElement?.getAttribute('aria-sort') ?? null,
                nonSortableButtons: mixedStatus?.querySelectorAll('button').length ?? 0,
                nonSortableSeparators: mixedStatus?.querySelectorAll(
                    '[role="separator"][tabindex="0"]'
                ).length ?? 0,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        structure["clientButtons"],
        json!(1),
        "client sort control: {structure}"
    );
    assert_eq!(
        structure["serverButtons"],
        json!(1),
        "server sort control: {structure}"
    );
    // The button's raw `textContent` includes the always-visible idle sort
    // glyph (ldui-875k: "a quiet bidirectional affordance", `header.rs`'s
    // `sort_indicator_symbol`) alongside the column label -- that glyph is
    // `aria-hidden="true"` and the button carries its own `aria-label`
    // (asserted below as `client`/`serverAccessibleName`), so it is never
    // announced; only the raw DOM text picks it up.
    assert_eq!(
        structure["clientName"],
        json!("Name\u{21c5}"),
        "client name: {structure}"
    );
    assert_eq!(
        structure["serverName"],
        json!("Name\u{21c5}"),
        "server name: {structure}"
    );
    assert_eq!(
        structure["clientAccessibleName"],
        json!("Name, not sorted. Activate to sort ascending."),
        "focused client control names current state and next action: {structure}"
    );
    assert_eq!(
        structure["serverAccessibleName"],
        json!("Name, not sorted. Activate to sort ascending."),
        "focused server control names current state and next action: {structure}"
    );
    assert_eq!(
        structure["clientAriaSort"],
        json!("none"),
        "client state: {structure}"
    );
    assert_eq!(
        structure["serverAriaSort"],
        json!("none"),
        "server state: {structure}"
    );
    assert_eq!(
        structure["nonSortableButtons"],
        json!(0),
        "non-sortable Status must not gain a sort tab stop: {structure}"
    );
    assert_eq!(
        structure["nonSortableSeparators"],
        json!(1),
        "resizing stays independently keyboard operable: {structure}"
    );

    // Pointer activation and both native keyboard activations each advance the
    // controlled oracle exactly one state. A handwritten key handler plus the
    // native click would advance twice and fail these alternating assertions.
    click(&h, client_sort).await;
    let state = oracle(&h).await;
    assert_eq!(
        state["state"]["datatable.sort"],
        json!({ "column": "name", "order": "ascending" }),
        "one pointer activation: {state}"
    );
    let semantics = eval_json(
        &h,
        r#"(() => {
            const button = document.querySelector('#keyboard-sort-table thead th:first-child > button');
            return { name: button.getAttribute('aria-label'), sort: button.parentElement.getAttribute('aria-sort') };
        })()"#,
    )
    .await;
    assert_eq!(
        semantics,
        json!({
            "name": "Name, sorted ascending. Activate to sort descending.",
            "sort": "ascending",
        }),
        "client ascending semantics: {semantics}"
    );
    h.page()
        .find_element(client_sort)
        .await
        .expect("find client sort control")
        .focus()
        .await
        .expect("focus client sort control");
    h.press_key_sequence(&[Key::Enter])
        .await
        .expect("client Enter");
    settle(&h).await;
    let state = oracle(&h).await;
    assert_eq!(
        state["state"]["datatable.sort"],
        json!({ "column": "name", "order": "descending" }),
        "one Enter activation: {state}"
    );
    let semantics = eval_json(
        &h,
        r#"(() => {
            const button = document.querySelector('#keyboard-sort-table thead th:first-child > button');
            return { name: button.getAttribute('aria-label'), sort: button.parentElement.getAttribute('aria-sort') };
        })()"#,
    )
    .await;
    assert_eq!(
        semantics,
        json!({
            "name": "Name, sorted descending. Activate to sort ascending.",
            "sort": "descending",
        }),
        "client descending semantics: {semantics}"
    );
    h.press_key_sequence(&[Key::Space])
        .await
        .expect("client Space");
    settle(&h).await;
    let state = oracle(&h).await;
    assert_eq!(
        state["state"]["datatable.sort"],
        json!({ "column": "name", "order": "ascending" }),
        "one Space activation: {state}"
    );

    click(
        &h,
        "#keyboard-sort-table thead tr:first-child th:first-child [role=\"separator\"]",
    )
    .await;
    let state = oracle(&h).await;
    assert_eq!(
        state["state"]["datatable.sort"],
        json!({ "column": "name", "order": "ascending" }),
        "resize separator click must not sort: {state}"
    );

    h.page()
        .find_element(server_sort)
        .await
        .expect("find server sort control")
        .focus()
        .await
        .expect("focus server sort control");
    h.press_key_sequence(&[Key::Enter])
        .await
        .expect("server Enter");
    settle(&h).await;
    let state = oracle(&h).await;
    assert_eq!(
        state["state"]["server_datatable.query"]["sort"],
        json!({ "column": "name", "order": "ascending" }),
        "one server Enter activation: {state}"
    );
    let semantics = eval_json(
        &h,
        r#"(() => {
            const button = document.querySelector('#server-table thead th:first-child > button');
            return { name: button.getAttribute('aria-label'), sort: button.parentElement.getAttribute('aria-sort') };
        })()"#,
    )
    .await;
    assert_eq!(
        semantics,
        json!({
            "name": "Name, sorted ascending. Activate to sort descending.",
            "sort": "ascending",
        }),
        "server ascending semantics: {semantics}"
    );
    h.press_key_sequence(&[Key::Space])
        .await
        .expect("server Space");
    settle(&h).await;
    let state = oracle(&h).await;
    assert_eq!(
        state["state"]["server_datatable.query"]["sort"],
        json!({ "column": "name", "order": "descending" }),
        "one server Space activation: {state}"
    );
    let semantics = eval_json(
        &h,
        r#"(() => {
            const button = document.querySelector('#server-table thead th:first-child > button');
            return { name: button.getAttribute('aria-label'), sort: button.parentElement.getAttribute('aria-sort') };
        })()"#,
    )
    .await;
    assert_eq!(
        semantics,
        json!({
            "name": "Name, sorted descending. Activate to sort ascending.",
            "sort": "descending",
        }),
        "server descending semantics: {semantics}"
    );

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(h.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("shared-data-table-sort")
        .unwrap_or_else(|error| {
            panic!(
                "{error}; {}; violations: {:#?}",
                report.summary(),
                report.violations
            )
        });
    assert_no_browser_errors(&h, "shared DataTable sort keyboard journey").await;
}

/// Opinionated table geometry (ldui-gbs): sorting is a body-data operation.
/// It must not replace header/filter nodes or move the table shell, column
/// tracks, grid lines, or horizontal scroll origin.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_sort_preserves_shell_geometry_and_semantic_bands() {
    let h = harness_at("/components/data-table").await;
    begin_browser_error_capture(&h).await;

    // Exercise the 10,000-row client table in a narrow, horizontally scrolled
    // viewport. Sorting Email replaces a page of mostly one/two-digit values
    // with lexicographically adjacent four-digit values, exposing any track
    // sizing derived from only the current page's cell contents.
    let prepared = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('#geometry-sort-table');
            root.style.width = '420px';
            root.scrollIntoView({ block: 'center' });
            root.querySelector('thead tr:first-child th:nth-child(3)')
                .scrollIntoView({ block: 'nearest', inline: 'center' });
            return !!root;
        })()"#,
    )
    .await;
    assert_eq!(prepared, json!(true));
    settle(&h).await;

    let client_baseline = mark_table_geometry(&h, "#geometry-sort-table").await;
    assert!(
        client_baseline["scrollWidth"].as_f64() > client_baseline["clientWidth"].as_f64(),
        "narrow fixture must genuinely overflow: {client_baseline}"
    );

    let client_sort = "#geometry-sort-table thead tr:first-child th:nth-child(3) > button";
    for activation in ["pointer", "Enter", "Space"] {
        if activation == "pointer" {
            click(&h, client_sort).await;
        } else {
            h.page()
                .find_element(client_sort)
                .await
                .expect("find client geometry sort control")
                .focus()
                .await
                .expect("focus client geometry sort control");
            h.press_key_sequence(&[if activation == "Enter" {
                Key::Enter
            } else {
                Key::Space
            }])
            .await
            .expect("activate client geometry sort control");
        }
        settle(&h).await;
        assert_table_geometry_unchanged(
            &compare_table_geometry(&h, "#geometry-sort-table").await,
            &format!("client {activation}"),
        );
        mark_table_geometry(&h, "#geometry-sort-table").await;
    }

    // The server-query path owns different state and replaces its page rows,
    // but it shares exactly the same stable shell contract.
    eval_json(
        &h,
        "document.querySelector('#server-table').scrollIntoView({ block: 'center' }); true",
    )
    .await;
    settle(&h).await;
    let server_baseline = mark_table_geometry(&h, "#server-table").await;
    assert_eq!(
        server_baseline["filterCells"], server_baseline["headerCells"],
        "server filter/header tracks must be one-to-one: {server_baseline}"
    );
    let server_sort = "#server-table thead tr:first-child th:first-child > button";
    for activation in ["pointer", "Enter", "Space"] {
        if activation == "pointer" {
            click(&h, server_sort).await;
        } else {
            h.page()
                .find_element(server_sort)
                .await
                .expect("find server geometry sort control")
                .focus()
                .await
                .expect("focus server geometry sort control");
            h.press_key_sequence(&[if activation == "Enter" {
                Key::Enter
            } else {
                Key::Space
            }])
            .await
            .expect("activate server geometry sort control");
        }
        settle(&h).await;
        assert_table_geometry_unchanged(
            &compare_table_geometry(&h, "#server-table").await,
            &format!("server {activation}"),
        );
        mark_table_geometry(&h, "#server-table").await;
    }

    let palette = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('#filter-row-table');
            const header = root.querySelector('thead tr:first-child th');
            const filter = root.querySelector('thead tr.data-table-filter-row th');
            const cell = root.querySelector('tbody td');
            const indicatorWidths = Array.from(
                root.querySelectorAll('thead tr:first-child th button')
            ).map(button => button.querySelector('[data-table-sort-indicator]')?.getBoundingClientRect().width ?? 0);
            const indicators = Array.from(
                root.querySelectorAll('thead tr:first-child th button [data-table-sort-indicator]')
            );
            return {
                headerBackground: getComputedStyle(header).backgroundColor,
                headerContent: getComputedStyle(header).color,
                filterBackground: getComputedStyle(filter).backgroundColor,
                filterContent: getComputedStyle(filter).color,
                grid: getComputedStyle(cell).borderRightColor,
                indicatorWidths,
                indicatorTexts: indicators.map(indicator => indicator.textContent.trim()),
                indicatorStates: indicators.map(indicator => indicator.dataset.tableSortState),
                indicatorOpacities: indicators.map(indicator => Number(getComputedStyle(indicator).opacity)),
            };
        })()"#,
    )
    .await;
    assert_eq!(palette["headerBackground"], json!("rgb(0, 69, 120)"));
    assert_eq!(palette["headerContent"], json!("rgb(255, 255, 255)"));
    assert_eq!(palette["filterBackground"], json!("rgb(229, 241, 251)"));
    assert_eq!(palette["filterContent"], json!("rgb(26, 26, 26)"));
    assert_eq!(palette["grid"], json!("rgb(224, 224, 224)"));
    assert!(
        palette["indicatorWidths"]
            .as_array()
            .is_some_and(|widths| widths
                .iter()
                .all(|width| width.as_f64().is_some_and(|width| width > 0.0))),
        "every sortable header reserves an indicator slot: {palette}"
    );
    assert!(
        palette["indicatorTexts"]
            .as_array()
            .is_some_and(|texts| texts.iter().all(|text| text == &json!("⇅"))),
        "every idle sortable header needs a visible bidirectional affordance: {palette}"
    );
    assert!(
        palette["indicatorStates"]
            .as_array()
            .is_some_and(|states| states.iter().all(|state| state == &json!("idle"))),
        "initial sortable headers must identify their indicator as idle: {palette}"
    );
    assert!(
        palette["indicatorOpacities"]
            .as_array()
            .is_some_and(|values| {
                values.iter().all(|value| {
                    value
                        .as_f64()
                        .is_some_and(|opacity| opacity > 0.0 && opacity < 1.0)
                })
            }),
        "idle affordances should be visible but quieter than the active arrow: {palette}"
    );

    assert_no_browser_errors(&h, "sort-stable shared DataTable geometry").await;
}

/// Store a browser-measured shell snapshot on the table root and tag every
/// header/filter node. The tags disappear if a reactive map replaces a node.
async fn mark_table_geometry(h: &pixelproof_web::Harness, selector: &str) -> serde_json::Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector({selector:?});
                const table = root.querySelector('table');
                const viewport = root.querySelector(':scope > .overflow-x-auto');
                const headers = Array.from(table.querySelectorAll('thead tr:first-child th'));
                const filters = Array.from(table.querySelectorAll('thead tr.data-table-filter-row th'));
                const box = element => {{
                    const rect = element.getBoundingClientRect();
                    return [rect.x, rect.y, rect.width, rect.height, rect.right, rect.bottom];
                }};
                [...headers, ...filters].forEach((cell, index) => {{
                    cell.dataset.geometryNodeId = `geometry-${{index}}-${{Math.random()}}`;
                }});
                if (viewport.scrollWidth > viewport.clientWidth) {{
                    viewport.scrollLeft = Math.min(73, viewport.scrollWidth - viewport.clientWidth);
                }}
                const snapshot = {{
                    table: box(table),
                    viewport: box(viewport),
                    headers: headers.map(box),
                    filters: filters.map(box),
                    headerNodes: headers.map(cell => cell.dataset.geometryNodeId),
                    filterNodes: filters.map(cell => cell.dataset.geometryNodeId),
                    scrollLeft: viewport.scrollLeft,
                }};
                root.__lduiGeometryBaseline = snapshot;
                return {{
                    headerCells: headers.length,
                    filterCells: filters.length,
                    clientWidth: viewport.clientWidth,
                    scrollWidth: viewport.scrollWidth,
                    scrollLeft: viewport.scrollLeft,
                }};
            }})()"#
        ),
    )
    .await
}

async fn compare_table_geometry(h: &pixelproof_web::Harness, selector: &str) -> serde_json::Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector({selector:?});
                const before = root.__lduiGeometryBaseline;
                const table = root.querySelector('table');
                const viewport = root.querySelector(':scope > .overflow-x-auto');
                const headers = Array.from(table.querySelectorAll('thead tr:first-child th'));
                const filters = Array.from(table.querySelectorAll('thead tr.data-table-filter-row th'));
                const box = element => {{
                    const rect = element.getBoundingClientRect();
                    return [rect.x, rect.y, rect.width, rect.height, rect.right, rect.bottom];
                }};
                const after = {{
                    table: box(table),
                    viewport: box(viewport),
                    headers: headers.map(box),
                    filters: filters.map(box),
                    headerNodes: headers.map(cell => cell.dataset.geometryNodeId ?? null),
                    filterNodes: filters.map(cell => cell.dataset.geometryNodeId ?? null),
                    scrollLeft: viewport.scrollLeft,
                }};
                const deltas = [];
                const visit = (left, right) => {{
                    if (Array.isArray(left)) {{
                        left.forEach((value, index) => visit(value, right[index]));
                    }} else {{
                        deltas.push(Math.abs(left - right));
                    }}
                }};
                visit(before.table, after.table);
                visit(before.viewport, after.viewport);
                visit(before.headers, after.headers);
                visit(before.filters, after.filters);
                return {{
                    maxDelta: Math.max(0, ...deltas),
                    sameHeaderNodes: JSON.stringify(before.headerNodes) === JSON.stringify(after.headerNodes),
                    sameFilterNodes: JSON.stringify(before.filterNodes) === JSON.stringify(after.filterNodes),
                    beforeScrollLeft: before.scrollLeft,
                    afterScrollLeft: after.scrollLeft,
                    before,
                    after,
                }};
            }})()"#
        ),
    )
    .await
}

fn assert_table_geometry_unchanged(result: &serde_json::Value, journey: &str) {
    assert_eq!(
        result["sameHeaderNodes"],
        json!(true),
        "{journey} replaced header nodes: {result}"
    );
    assert_eq!(
        result["sameFilterNodes"],
        json!(true),
        "{journey} replaced filter nodes: {result}"
    );
    assert_eq!(
        result["beforeScrollLeft"], result["afterScrollLeft"],
        "{journey} changed the horizontal scroll origin: {result}"
    );
    assert!(
        result["maxDelta"]
            .as_f64()
            .is_some_and(|delta| delta <= 0.5),
        "{journey} moved table shell geometry by more than 0.5px: {result}"
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

/// Regression guard for ldui-89rp: a short first `<tbody>` row must not
/// derive an `auto_page_size` count that overflows once a genuinely tall row
/// further down the page is accounted for. The fix measures the MAX
/// `offset_height` across every currently rendered row (not just the first),
/// plus a bounded next-frame belt-and-braces check.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn auto_page_size_does_not_overflow_with_a_tall_variable_height_row() {
    let h = harness_at("/components/data-table").await;

    let snapshot_expr = r#"(() => {
        const root = document.querySelector('#auto-page-variable-height-table');
        const viewport = root.querySelector(':scope > .overflow-x-auto');
        return {
            rows: root.querySelectorAll('tbody tr').length,
            viewportHeight: viewport.clientHeight,
            scrollHeight: viewport.scrollHeight,
        };
    })()"#;

    // Let the ResizeObserver's settle pass, the era-damped multi-pass
    // self-correction, and the belt-and-braces next-frame check all finish.
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let first = eval_json(&h, snapshot_expr).await;

    // Settling, not just a reading that happens to look fine once (ldui-89rp
    // CRITICAL fix): the demo's tall row sits past the default page size, so
    // it is absent from the very first render and only appears once a
    // measurement pass grows the page. Without the high-water-mark ratchet
    // the derived row count oscillates forever between two values as that
    // tall row is alternately revealed and excluded again. A second reading
    // after another full settle window must see the SAME row count -- proof
    // the derivation reached a fixed point instead of still cycling.
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let second = eval_json(&h, snapshot_expr).await;

    assert!(
        first["rows"].as_u64().unwrap_or(0) > 0,
        "expected at least one rendered row: {first}"
    );
    assert_eq!(
        first["rows"], second["rows"],
        "auto_page_size did not settle -- the derived row count is still changing \
         (oscillation): first={first} second={second}"
    );
    for (label, snapshot) in [("first", &first), ("second", &second)] {
        assert!(
            snapshot["scrollHeight"].as_u64() <= snapshot["viewportHeight"].as_u64(),
            "auto_page_size overflowed its own scroll wrapper with a tall variable-height \
             row in the rendered set (pagination + scrollbar together) at the {label} \
             reading: {snapshot}"
        );
    }
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

/// DataTable search/filter naming (ldui-86h): both variants expose a stable
/// localized accessible name and a real associated `<label>`, so neither the
/// browser nor the structural drift audit has to infer a name from placeholder
/// text or column position.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_search_and_filter_controls_have_associated_names() {
    let h = harness_at("/components/data-table").await;
    begin_browser_error_capture(&h).await;

    let names = eval_json(
        &h,
        r#"(() => {
            const describe = element => ({
                aria: element.getAttribute('aria-label'),
                labels: element.labels?.length ?? 0,
                labelText: Array.from(element.labels || []).map(label =>
                    (label.matches('.sr-only') ? label : label.querySelector('.sr-only'))
                        ?.textContent.trim() ?? ''
                ),
            });
            return {
                clientSearch: describe(document.querySelector('#filter-row-table input')),
                serverSearch: describe(document.querySelector('#server-table input')),
                clientFilters: Array.from(document.querySelectorAll('#filter-row-table tr.data-table-filter-row select')).map(describe),
                serverFilters: Array.from(document.querySelectorAll('#server-table tr.data-table-filter-row select')).map(describe),
            };
        })()"#,
    )
    .await;
    for search in [&names["clientSearch"], &names["serverSearch"]] {
        assert_eq!(search["aria"], json!("Search table"));
        assert_eq!(search["labels"], json!(1));
        assert_eq!(search["labelText"], json!(["Search table"]));
    }
    for filter in names["clientFilters"]
        .as_array()
        .unwrap()
        .iter()
        .chain(names["serverFilters"].as_array().unwrap())
    {
        assert_eq!(filter["labels"], json!(1));
        assert!(filter["aria"].as_str().unwrap().starts_with("Filter by "));
        assert_eq!(filter["labelText"][0], filter["aria"]);
    }

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(h.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("associated DataTable search/filter names")
        .unwrap_or_else(|error| panic!("{error}; {}", report.summary()));
    assert_no_browser_errors(&h, "DataTable associated search/filter names").await;
}

/// DataTable detail rows (ldui-5tv): optional row-specific content is a full
/// sibling row, stays paired with its source through sort, preserves stable
/// header geometry, and cannot activate the source row.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_detail_subrows_stay_paired_and_isolated() {
    let h = harness_at("/components/data-table").await;
    begin_browser_error_capture(&h).await;
    let initial = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('#detail-row-table');
            const table = root.querySelector('table');
            const headers = Array.from(table.querySelectorAll('thead tr:first-child th'));
            headers.forEach((header, index) => header.__detailHeader = index);
            const details = Array.from(table.querySelectorAll('tbody tr[data-table-detail-row]'));
            return {
                mainRows: table.querySelectorAll('tbody tr:not([data-table-detail-row])').length,
                detailRows: details.length,
                colspans: details.map(row => Number(row.cells[0].getAttribute('colspan'))),
                columns: headers.length,
                paired: details.every(row => row.previousElementSibling && !row.previousElementSibling.matches('[data-table-detail-row]')),
                tableBox: [table.getBoundingClientRect().x, table.getBoundingClientRect().width],
                headerBoxes: headers.map(header => [header.getBoundingClientRect().x, header.getBoundingClientRect().width]),
            };
        })()"#,
    )
    .await;
    assert_eq!(initial["mainRows"], json!(3));
    assert_eq!(initial["detailRows"], json!(2));
    assert_eq!(initial["paired"], json!(true));
    assert!(
        initial["colspans"]
            .as_array()
            .unwrap()
            .iter()
            .all(|span| span == &initial["columns"])
    );

    assert_eq!(
        eval_json(
            &h,
            "document.querySelector('#detail-row-table [data-testid=\"data-table-detail-action\"]').click(); true",
        )
        .await,
        json!(true),
    );
    settle(&h).await;
    assert_eq!(
        eval_json(
            &h,
            "document.querySelector('[data-testid=\"detail-row-activation-count\"]').textContent",
        )
        .await,
        json!("0"),
        "detail-row interaction bubbled into the source-row activation"
    );

    click(
        &h,
        "#detail-row-table thead tr:first-child th:first-child > button",
    )
    .await;
    settle(&h).await;
    click(
        &h,
        "#detail-row-table thead tr:first-child th:first-child > button",
    )
    .await;
    settle(&h).await;
    let sorted = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('#detail-row-table');
            const table = root.querySelector('table');
            const headers = Array.from(table.querySelectorAll('thead tr:first-child th'));
            const details = Array.from(table.querySelectorAll('tbody tr[data-table-detail-row]'));
            const tableBox = [table.getBoundingClientRect().x, table.getBoundingClientRect().width];
            const headerBoxes = headers.map(header => [header.getBoundingClientRect().x, header.getBoundingClientRect().width]);
            return {
                paired: details.every(row => {
                    const detailName = row.textContent.match(/User \d+/)?.[0];
                    return detailName && row.previousElementSibling.textContent.includes(detailName);
                }),
                sameHeaders: headers.every((header, index) => header.__detailHeader === index),
                tableBox,
                headerBoxes,
            };
        })()"#,
    )
    .await;
    assert_eq!(sorted["paired"], json!(true));
    assert_eq!(sorted["sameHeaders"], json!(true));
    assert_eq!(sorted["tableBox"], initial["tableBox"]);
    assert_eq!(sorted["headerBoxes"], initial["headerBoxes"]);
    assert_no_browser_errors(&h, "DataTable detail subrows").await;
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

/// Field's native monotonic-counter test is necessary but insufficient: this
/// fixture proves that one real WASM form receives six distinct control IDs,
/// six exact label targets, and resolvable help/error references.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn field_ids_and_associations_are_unique_in_real_wasm() {
    let h = harness_at("/components/fieldset").await;
    begin_browser_error_capture(&h).await;
    common::wait_for_selector(&h, "#field-unique-associations").await;

    let snapshot = eval_json(
        &h,
        r#"(() => {
            const form = document.querySelector('#field-unique-associations');
            const cases = [...form.querySelectorAll('[data-field-case]')];
            const controls = cases.map(root => root.querySelector('input, select'));
            const labels = cases.map(root => root.querySelector('label[for]'));
            const ids = controls.map(control => control?.id ?? '');
            const describedTokens = controls.flatMap(control =>
                (control?.getAttribute('aria-describedby') ?? '')
                    .split(/\s+/)
                    .filter(Boolean)
            );
            const errorTokens = controls.flatMap(control =>
                (control?.getAttribute('aria-errormessage') ?? '')
                    .split(/\s+/)
                    .filter(Boolean)
            );
            const allIds = [...document.querySelectorAll('[id]')].map(node => node.id);
            const duplicates = [...new Set(allIds.filter(
                (id, index) => id && allIds.indexOf(id) !== index
            ))];
            return {
                caseCount: cases.length,
                controlCount: controls.filter(Boolean).length,
                labelCount: labels.filter(Boolean).length,
                ids,
                uniqueControlIds: new Set(ids).size,
                emptyControlIds: ids.filter(id => !id).length,
                labelTargets: labels.map(label => label?.getAttribute('for') ?? ''),
                exactLabelTargets: labels.every((label, index) => {
                    const target = label?.getAttribute('for') ?? '';
                    return target === ids[index]
                        && form.querySelectorAll(`[id="${CSS.escape(target)}"]`).length === 1;
                }),
                describedTokens,
                describedTargetsResolveOnce: describedTokens.every(
                    id => document.querySelectorAll(`[id="${CSS.escape(id)}"]`).length === 1
                ),
                errorTokens,
                errorTargetsResolveOnce: errorTokens.every(
                    id => document.querySelectorAll(`[id="${CSS.escape(id)}"]`).length === 1
                ),
                duplicateIds: duplicates,
            };
        })()"#,
    )
    .await;

    assert_eq!(snapshot["caseCount"], json!(6), "fixture shape: {snapshot}");
    assert_eq!(snapshot["controlCount"], json!(6), "controls: {snapshot}");
    assert_eq!(snapshot["labelCount"], json!(6), "labels: {snapshot}");
    assert_eq!(snapshot["emptyControlIds"], json!(0), "ids: {snapshot}");
    assert_eq!(
        snapshot["uniqueControlIds"],
        json!(6),
        "every WASM control id must be unique: {snapshot}"
    );
    assert_eq!(
        snapshot["exactLabelTargets"],
        json!(true),
        "each visible label must target its own control exactly once: {snapshot}"
    );
    assert_eq!(
        snapshot["describedTargetsResolveOnce"],
        json!(true),
        "every aria-describedby token must resolve once: {snapshot}"
    );
    assert_eq!(
        snapshot["errorTargetsResolveOnce"],
        json!(true),
        "every aria-errormessage token must resolve once: {snapshot}"
    );
    assert_eq!(
        snapshot["duplicateIds"],
        json!([]),
        "Chrome must not see a duplicate form-field id: {snapshot}"
    );

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    // Inject axe through PixelProof, then scope the release assertion to this
    // fixture. The component-catalog page intentionally contains older raw
    // form examples outside this form; those cannot be allowed to obscure the
    // six-control Field contract being proven here.
    let _page_report = axe.run(h.page()).await.expect("inject and run axe-core");
    let scoped_axe = eval_json(
        &h,
        r#"(async () => {
            const report = await axe.run(document.querySelector('#field-unique-associations'), {
                runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21aa'] },
                resultTypes: ['violations'],
            });
            return report.violations
                .filter(violation => violation.impact === 'serious' || violation.impact === 'critical')
                .map(violation => ({
                    id: violation.id,
                    nodes: violation.nodes.map(node => ({ target: node.target, html: node.html })),
                }));
        })()"#,
    )
    .await;
    assert_eq!(
        scoped_axe,
        json!([]),
        "the six-control Field fixture has blocking axe findings: {scoped_axe}"
    );
    assert_no_browser_errors(&h, "Field unique-association fixture").await;
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

    assert_eq!(
        eval_json(
            &h,
            r#"(() => {
                const input = document.querySelector('#server-table input[type="text"]');
                input.value = '  USER 1  ';
                input.dispatchEvent(new Event('input', { bubbles: true }));
                return document.querySelector('#server-table').dataset.serverQueryOwnership;
            })()"#,
        )
        .await,
        json!("controlled")
    );
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    let normalized = eval_json(
        &h,
        r#"(() => ({
            value: document.querySelector('#server-table input[type="text"]').value,
            query: document.querySelector('[data-testid="server-last-query"]').textContent,
        }))()"#,
    )
    .await;
    assert_eq!(normalized["value"], json!("user 1"));
    assert!(
        normalized["query"]
            .as_str()
            .unwrap()
            .contains("search=\"user 1\"")
    );
    assert!(normalized["query"].as_str().unwrap().contains("page=1"));

    click(&h, "[data-testid='server-query-reset']").await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let reset = eval_json(
        &h,
        r#"(() => ({
            search: document.querySelector('#server-table input[type="text"]').value,
            sort: document.querySelector('#server-table thead th:first-child').getAttribute('aria-sort'),
            role: document.querySelector('#server-table [data-table-filter-column="role"] select').value,
            pageSize: document.querySelector('#server-table select[id$="-page-size"]').value,
        }))()"#,
    )
    .await;
    assert_eq!(reset["search"], json!(""));
    assert!(reset["sort"].is_null() || reset["sort"] == json!("none"));
    assert_eq!(reset["role"], json!(""));
    assert_eq!(reset["pageSize"], json!("10"));

    click(&h, "[data-testid='server-query-accept']").await;
    let before_rejections: u64 = testid_text(&h, "server-query-proposals")
        .await
        .parse()
        .expect("numeric proposal count");
    click(&h, "#server-table thead th:first-child").await;
    assert_eq!(
        eval_json(
            &h,
            r#"(() => {
                const select = document.querySelector('#server-table [data-table-filter-column="role"] select');
                select.value = 'Admin';
                select.dispatchEvent(new Event('change', { bubbles: true }));
                const size = document.querySelector('#server-table select[id$="-page-size"]');
                size.value = '25';
                size.dispatchEvent(new Event('change', { bubbles: true }));
                const search = document.querySelector('#server-table input[type="text"]');
                search.value = 'declined';
                search.dispatchEvent(new Event('input', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    let rejected = eval_json(
        &h,
        r#"(() => ({
            search: document.querySelector('#server-table input[type="text"]').value,
            sort: document.querySelector('#server-table thead th:first-child').getAttribute('aria-sort'),
            role: document.querySelector('#server-table [data-table-filter-column="role"] select').value,
            pageSize: document.querySelector('#server-table select[id$="-page-size"]').value,
            proposals: Number(document.querySelector('[data-testid="server-query-proposals"]').textContent),
        }))()"#,
    )
    .await;
    assert_eq!(rejected["search"], json!(""));
    assert!(rejected["sort"].is_null() || rejected["sort"] == json!("none"));
    assert_eq!(rejected["role"], json!(""));
    assert_eq!(rejected["pageSize"], json!("10"));
    assert_eq!(rejected["proposals"], json!(before_rejections + 4));

    let before_locale = rejected["proposals"].clone();
    click(&h, "#locale-toggle").await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let localized = eval_json(
        &h,
        r#"(() => ({
            searchLabel: document.querySelector('#server-table input[type="text"]').getAttribute('aria-label'),
            pageSizeLabel: document.querySelector('#server-table select[id$="-page-size"]').getAttribute('aria-label'),
            filterLabel: document.querySelector('#server-table [data-table-filter-column="role"] select').getAttribute('aria-label'),
            proposals: Number(document.querySelector('[data-testid="server-query-proposals"]').textContent),
        }))()"#,
    )
    .await;
    assert_eq!(localized["searchLabel"], json!("Buscar en la tabla"));
    assert_eq!(localized["pageSizeLabel"], json!("Filas por página"));
    assert!(localized["filterLabel"].as_str().unwrap().contains("Role"));
    assert_eq!(localized["proposals"], before_locale);

    click(&h, "[data-testid='server-query-accept']").await;
    click(&h, "#server-table thead th:first-child").await;
    let before_scope: u64 = testid_text(&h, "server-query-proposals")
        .await
        .parse()
        .expect("numeric proposal count before scope reset");
    click(&h, "[data-testid='server-query-scope']").await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let scoped = eval_json(
        &h,
        r#"(() => ({
            search: document.querySelector('#server-table input[type="text"]').value,
            sort: document.querySelector('#server-table thead th:first-child').getAttribute('aria-sort'),
            role: document.querySelector('#server-table [data-table-filter-column="role"] select').value,
            proposals: Number(document.querySelector('[data-testid="server-query-proposals"]').textContent),
        }))()"#,
    )
    .await;
    assert_eq!(scoped["search"], json!(""));
    assert!(scoped["sort"].is_null() || scoped["sort"] == json!("none"));
    assert_eq!(scoped["role"], json!(""));
    assert_eq!(scoped["proposals"], json!(before_scope + 1));

    assert_server_table_cursor_pagination_preserves_slice_truth(&h).await;
}

/// Cursor paging (ldui-9k1): opaque previous/next tokens produce exactly one
/// controlled query proposal, shape changes restart at `First`, retained rows
/// stay visible and truthfully labelled, and mixed offset/cursor props are
/// rejected instead of silently choosing a strategy.
async fn assert_server_table_cursor_pagination_preserves_slice_truth(h: &pixelproof_web::Harness) {
    let initial = eval_json(
        h,
        r#"(() => {
            const table = document.querySelector('#cursor-server-table');
            const controls = table.querySelector('[data-server-cursor-state]');
            return {
                strategy: table.dataset.serverPaginationStrategy,
                ownership: table.dataset.serverQueryOwnership,
                rows: table.querySelectorAll('tbody tr').length,
                buttons: controls.querySelectorAll('button').length,
                status: controls.querySelector('[role="status"]').textContent.trim(),
                proposals: Number(document.querySelector('[data-testid="cursor-query-proposals"]').textContent),
                vocabulary: table.dataset.serverFilterVocabulary,
                roles: Array.from(table.querySelectorAll('tbody tr[data-row-key]')).map(row => row.cells[2].textContent.trim()),
                roleOptions: Array.from(table.querySelector('[data-table-filter-column="role"] select').options).map(option => option.value),
            };
        })()"#,
    )
    .await;
    assert_eq!(initial["strategy"], json!("cursor"));
    assert_eq!(initial["ownership"], json!("controlled"));
    assert_eq!(initial["rows"], json!(4));
    assert_eq!(initial["buttons"], json!(2));
    assert_eq!(initial["status"], json!("Showing 4 rows"));
    assert_eq!(initial["proposals"], json!(0));
    assert_eq!(initial["vocabulary"], json!("authoritative"));
    assert!(
        !initial["roles"]
            .as_array()
            .is_some_and(|roles| roles.contains(&json!("Analyst"))),
        "Analyst is deliberately absent from the first four-row slice: {initial}"
    );
    assert!(
        initial["roleOptions"]
            .as_array()
            .is_some_and(|options| options.contains(&json!("role.analyst"))),
        "the authoritative population vocabulary must still offer Analyst: {initial}"
    );

    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const current = document.querySelector('#cursor-current-slice-vocabulary [data-table-data-mode="server-query"]');
                const missing = document.querySelector('#cursor-missing-vocabulary [data-table-data-mode="server-query"]');
                const currentSelect = current.querySelector('[data-table-filter-column="role"] select');
                const missingError = missing.querySelector('[data-server-filter-vocabulary-config-error]');
                return {
                    current: {
                        vocabulary: current.dataset.serverFilterVocabulary,
                        allLabel: currentSelect.options[0].textContent.trim(),
                        aria: currentSelect.getAttribute('aria-label'),
                        options: Array.from(currentSelect.options).map(option => option.value),
                    },
                    missing: {
                        vocabulary: missing.dataset.serverFilterVocabulary,
                        role: missingError.getAttribute('role'),
                        message: missingError.textContent.trim(),
                        filterRows: missing.querySelectorAll('tr.data-table-filter-row').length,
                    },
                };
            })()"#,
        )
        .await,
        json!({
            "current": {
                "vocabulary": "current-slice",
                "allLabel": "All on this page",
                "aria": "Filter current page by Role",
                "options": ["", "Admin", "Designer", "Developer", "Manager"],
            },
            "missing": {
                "vocabulary": "invalid",
                "role": "alert",
                "message": "ServerDataTable exact filter columns require authoritative filter_options or an explicit current-slice vocabulary",
                "filterRows": 0,
            },
        })
    );

    // Stable-key reconciliation: moving, inserting around, and removing
    // around a row must preserve that business entity's exact DOM node and
    // focus. The mutation buttons are invoked programmatically so clicking a
    // separate control does not itself steal focus from the row under test.
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const row = document.querySelector('#cursor-server-table tbody tr[data-row-key="001"]');
                row.__lduiIdentityProbe = 'row-001';
                row.focus();
                document.querySelector('[data-testid="cursor-reverse-rows"]').click();
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const row = document.querySelector('#cursor-server-table tbody tr[data-row-key="001"]');
                return {
                    keys: Array.from(document.querySelectorAll('#cursor-server-table tbody tr[data-row-key]')).map(row => row.dataset.rowKey),
                    probe: row.__lduiIdentityProbe ?? null,
                    focused: document.activeElement === row,
                    index: Number(row.dataset.rowIndex),
                };
            })()"#,
        )
        .await,
        json!({
            "keys": ["004", "003", "002", "001"],
            "probe": "row-001",
            "focused": true,
            "index": 3,
        })
    );

    // Two separate `evaluate()` round trips (rather than one script doing
    // both the insert-click and the remove-click back to back), each
    // followed by a short settle: Leptos flushes a reactive DOM patch on a
    // microtask, not synchronously inside the click handler's own call
    // stack, so reading `document.querySelector` in the SAME synchronous
    // script as the click that triggers the insert observes the DOM from
    // BEFORE the patch every time (confirmed 100% reproducible, not a
    // flake). Splitting into separate scripts -- and, belt-and-braces, a
    // short sleep -- lets the intervening microtask/reactive flush actually
    // run before the read.
    //
    // The click is still `element.click()` INSIDE the script (never the
    // `click()` test helper's real CDP mouse events): a genuine synthesized
    // pointer click legitimately moves keyboard focus to the button it
    // lands on (confirmed empirically), which is real browser behaviour,
    // not a bug -- and would make this section assert something false. The
    // module comment above ("clicking a separate control does not itself
    // steal focus") is specifically about the JS `.click()` method, which
    // does not.
    eval_json(
        h,
        r#"(() => { document.querySelector('[data-testid="cursor-insert-row"]').click(); return true; })()"#,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const inserted = document.querySelector('#cursor-server-table tbody tr[data-row-key="inserted"]');
                const stable = document.querySelector('#cursor-server-table tbody tr[data-row-key="001"]');
                return {
                    inserted: !!inserted,
                    probe: stable.__lduiIdentityProbe ?? null,
                    index: Number(stable.dataset.rowIndex),
                    focused: document.activeElement === stable,
                };
            })()"#,
        )
        .await,
        json!({
            "inserted": true,
            "probe": "row-001",
            "index": 4,
            "focused": true,
        }),
        "row_key insert must keep the stable row's identity and focus"
    );

    eval_json(
        h,
        r#"(() => { document.querySelector('[data-testid="cursor-remove-inserted-row"]').click(); return true; })()"#,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const stable = document.querySelector('#cursor-server-table tbody tr[data-row-key="001"]');
                return {
                    insertedAfterRemove: !!document.querySelector('#cursor-server-table tbody tr[data-row-key="inserted"]'),
                    indexAfterRemove: Number(stable.dataset.rowIndex),
                    probeAfterRemove: stable.__lduiIdentityProbe ?? null,
                    focusedAfterRemove: document.activeElement === stable,
                };
            })()"#,
        )
        .await,
        json!({
            "insertedAfterRemove": false,
            "indexAfterRemove": 3,
            "probeAfterRemove": "row-001",
            "focusedAfterRemove": true,
        }),
        "row_key remove must keep the stable row's identity and focus"
    );

    click(
        h,
        "#cursor-server-table tbody tr[data-row-key='001'] td:first-child",
    )
    .await;
    assert_eq!(
        testid_text(h, "cursor-keyed-activation").await,
        "001|3|User 1",
        "keyed activation must carry the displayed identity snapshot"
    );
    assert_eq!(
        eval_json(
            h,
            r#"(() => ({
                accepted: document.querySelector('[data-testid="cursor-selected-key"]').textContent.trim(),
                proposals: Number(document.querySelector('[data-testid="cursor-selection-proposals"]').textContent),
                selected: Array.from(document.querySelectorAll('#cursor-server-table tbody tr[aria-selected="true"]')).map(row => row.dataset.rowKey),
            }))()"#,
        )
        .await,
        json!({ "accepted": "001", "proposals": 1, "selected": ["001"] })
    );
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const row = document.querySelector('#cursor-server-table tbody tr[data-row-key="001"]');
                row.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', shiftKey: true, bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    assert_eq!(
        testid_text(h, "cursor-keyed-inspection").await,
        "001|3|User 1",
        "keyed inspection must carry the displayed identity snapshot"
    );

    // Rejected proposals never paint optimistic selection. The callback sees
    // exactly one proposed key, while the accepted key and aria-selected row
    // remain 001. Re-enable acceptance and prove keyboard Space uses the same
    // controlled path.
    //
    // Split into two scripts with a settle between (same rationale as the
    // insert/remove split above): the accept-toggle click and the very next
    // row click, back to back in one synchronous script with no yield,
    // observably raced the reactive flush -- the row click's own handler
    // never fired in that shape (proposals stayed at 1, not 2), 100%
    // reproducible.
    eval_json(
        h,
        r#"(() => { document.querySelector('[data-testid="cursor-selection-accept"]').click(); return true; })()"#,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    eval_json(
        h,
        r#"(() => { document.querySelector('#cursor-server-table tbody tr[data-row-key="002"]').click(); return true; })()"#,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        eval_json(
            h,
            r#"(() => ({
                accepted: document.querySelector('[data-testid="cursor-selected-key"]').textContent.trim(),
                proposed: document.querySelector('[data-testid="cursor-last-selection-proposal"]').textContent.trim(),
                proposals: Number(document.querySelector('[data-testid="cursor-selection-proposals"]').textContent),
                selected: Array.from(document.querySelectorAll('#cursor-server-table tbody tr[aria-selected="true"]')).map(row => row.dataset.rowKey),
            }))()"#,
        )
        .await,
        json!({
            "accepted": "001",
            "proposed": "002",
            "proposals": 2,
            "selected": ["001"],
        })
    );
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                document.querySelector('[data-testid="cursor-selection-accept"]').click();
                document.querySelector('#cursor-server-table tbody tr[data-row-key="003"]').focus();
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    h.press_key_sequence(&[Key::Space])
        .await
        .expect("Space selects the focused keyed server row");
    assert_eq!(
        eval_json(
            h,
            r#"(() => ({
                accepted: document.querySelector('[data-testid="cursor-selected-key"]').textContent.trim(),
                proposals: Number(document.querySelector('[data-testid="cursor-selection-proposals"]').textContent),
                selected: Array.from(document.querySelectorAll('#cursor-server-table tbody tr[aria-selected="true"]')).map(row => row.dataset.rowKey),
            }))()"#,
        )
        .await,
        json!({ "accepted": "003", "proposals": 3, "selected": ["003"] })
    );

    // Duplicate keys fail closed: only an explicit alert row renders. Restore
    // the accepted server slice before continuing the cursor journey.
    //
    // The real CDP `click()` helper (rather than a JS `.click()` immediately
    // followed by a same-script read) both triggers the click AND settles
    // before returning -- see the module comment above the insert/remove
    // split earlier in this test for why a click and an immediately
    // following read of its reactive effect, in one synchronous script,
    // observably races Leptos's DOM patch. Focus semantics do not matter for
    // these two buttons (nothing here asserts on `document.activeElement`),
    // so the real click's focus-stealing is harmless.
    click(h, "[data-testid='cursor-duplicate-row-key']").await;
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const table = document.querySelector('#cursor-server-table');
                return {
                    dataRows: table.querySelectorAll('tbody tr[data-row-key]').length,
                    errors: table.querySelectorAll('tbody tr[data-table-row-key-error] [role="alert"]').length,
                    message: table.querySelector('tbody tr[data-table-row-key-error]')?.textContent.trim() ?? '',
                };
            })()"#,
        )
        .await,
        json!({
            "dataRows": 0,
            "errors": 1,
            "message": "DataTable row_key returned duplicate key \"004\" for page rows 0 and 4",
        })
    );
    click(h, "[data-testid='cursor-restore-rows']").await;
    assert_eq!(
        eval_json(
            h,
            r#"Array.from(document.querySelectorAll('#cursor-server-table tbody tr[data-row-key]')).map(row => row.dataset.rowKey)"#,
        )
        .await,
        json!(["001", "002", "003", "004"])
    );

    // A cursor replacement must not recycle the focused row-001 node for the
    // different entity that takes page position zero.
    eval_json(
        h,
        r#"(() => {
            const old = document.querySelector('#cursor-server-table tbody tr[data-row-key="001"]');
            old.__lduiOldPageProbe = true;
            old.focus();
            return true;
        })()"#,
    )
    .await;
    // Real CDP click, not a same-script JS `.click()`+immediate-read (see
    // the module comment on the insert/remove split above): it moves the
    // browser's focus to the button itself, which is fine here since the
    // assertion only cares whether focus landed on the NEW row (it must
    // not), not on the button.
    click(h, "#cursor-server-table [data-server-cursor-action='next']").await;
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const next = document.querySelector('#cursor-server-table tbody tr[data-row-key="005"]');
                return {
                    nextExists: !!next,
                    inheritedProbe: next?.__lduiOldPageProbe ?? false,
                    focusTransferred: document.activeElement === next,
                };
            })()"#,
        )
        .await,
        json!({
            "nextExists": true,
            "inheritedProbe": false,
            "focusTransferred": false,
        })
    );
    assert_eq!(
        count_of(h, "#cursor-server-table tbody tr[aria-selected='true']",).await,
        0,
        "accepted key 003 is outside the next slice and must not transfer by index"
    );

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "1");
    assert!(
        testid_text(h, "cursor-last-query")
            .await
            .contains("request=Next(offset:4)"),
        "Next must forward the exact opaque server token"
    );
    assert!(
        eval_json(
            h,
            "document.querySelector('#cursor-server-table tbody tr').textContent",
        )
        .await
        .as_str()
        .is_some_and(|text| text.contains("User 5"))
    );

    click(
        h,
        "#cursor-server-table [data-server-cursor-action='previous']",
    )
    .await;
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "2");
    assert!(
        eval_json(
            h,
            "document.querySelector('#cursor-server-table tbody tr').textContent",
        )
        .await
        .as_str()
        .is_some_and(|text| text.contains("User 1"))
    );
    assert_eq!(
        eval_json(
            h,
            r#"document.querySelector('#cursor-server-table tbody tr[aria-selected="true"]')?.dataset.rowKey ?? null"#,
        )
        .await,
        json!("003"),
        "returning to the previous slice restores the accepted business key"
    );

    click(h, "#cursor-server-table thead th:first-child").await;
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "3");
    let sorted = testid_text(h, "cursor-last-query").await;
    assert!(sorted.contains("request=First") && sorted.contains("sort=Some"));

    // A high-cardinality server column emits the same stable filter map, with
    // its Contains interpretation carried by the supplied Column definition.
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const input = document.querySelector('#cursor-server-table [data-table-filter-column="name"] input[data-table-filter-kind="contains"]');
                input.value = 'USER 1';
                input.dispatchEvent(new Event('input', { bubbles: true }));
                return { aria: input.getAttribute('aria-label'), immediate: input.value };
            })()"#,
        )
        .await,
        json!({ "aria": "Filter Name by text", "immediate": "USER 1" })
    );
    tokio::time::sleep(std::time::Duration::from_millis(225)).await;
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "4");
    assert!(
        testid_text(h, "cursor-last-query").await.contains("USER 1"),
        "the debounced proposal must preserve the entered substring value"
    );
    assert!(
        eval_json(
            h,
            "document.querySelector('#cursor-server-table tbody').textContent",
        )
        .await
        .as_str()
        .is_some_and(|text| text.contains("User 1"))
    );

    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const input = document.querySelector('#cursor-server-table [data-table-filter-column="name"] input');
                input.value = '';
                input.dispatchEvent(new Event('input', { bubbles: true }));
                return input.value;
            })()"#,
        )
        .await,
        json!("")
    );
    tokio::time::sleep(std::time::Duration::from_millis(225)).await;
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "5");

    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const filter = document.querySelector('#cursor-server-table [data-table-filter-column="role"] select');
                filter.value = 'role.analyst';
                filter.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "6");
    assert!(
        testid_text(h, "cursor-last-query")
            .await
            .contains("role.analyst")
    );

    // Labels are reactive presentation while the stable value remains query
    // truth. If metadata temporarily omits the accepted value, the framework
    // retains a removable fallback option instead of blanking the select.
    assert_eq!(
        {
            click(h, "[data-testid='cursor-filter-locale']").await;
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            eval_json(
                h,
                r#"(() => {
                    const select = document.querySelector('#cursor-server-table [data-table-filter-column="role"] select');
                    return {
                        value: select.value,
                        label: select.selectedOptions[0].textContent.trim(),
                    };
                })()"#,
            )
            .await
        },
        json!({ "value": "role.analyst", "label": "Analista" })
    );
    click(h, "[data-testid='cursor-filter-active-option']").await;
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const select = document.querySelector('#cursor-server-table [data-table-filter-column="role"] select');
                return { value: select.value, label: select.selectedOptions[0].textContent.trim() };
            })()"#,
        )
        .await,
        json!({ "value": "role.analyst", "label": "role.analyst" })
    );
    click(h, "[data-testid='cursor-filter-active-option']").await;
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const select = document.querySelector('#cursor-server-table [data-table-filter-column="role"] select');
                return { value: select.value, label: select.selectedOptions[0].textContent.trim() };
            })()"#,
        )
        .await,
        json!({ "value": "role.analyst", "label": "Analista" })
    );

    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                document.querySelector('[data-testid="cursor-query-accept"]').click();
                const input = document.querySelector('#cursor-server-table [data-table-filter-column="name"] input');
                input.value = 'User 2';
                input.dispatchEvent(new Event('input', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(std::time::Duration::from_millis(225)).await;
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "7");
    assert!(
        testid_text(h, "cursor-last-query").await.contains("User 2"),
        "the rejected proposal must still carry the entered substring"
    );
    assert_eq!(
        eval_json(
            h,
            r#"document.querySelector('#cursor-server-table [data-table-filter-column="name"] input').value"#,
        )
        .await,
        json!(""),
        "controlled rejection must restore the accepted empty substring"
    );
    assert_eq!(
        eval_json(
            h,
            r#"document.querySelector('#cursor-server-table [data-table-filter-column="role"] select').value"#,
        )
        .await,
        json!("role.analyst"),
        "a rejected text proposal must not disturb an accepted exact filter"
    );
    click(h, "[data-testid='cursor-query-accept']").await;

    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const size = document.querySelector('#cursor-server-table select[id$="-page-size"]');
                size.value = '8';
                size.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "8");
    assert!(
        testid_text(h, "cursor-last-query")
            .await
            .contains("request=First")
    );

    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const input = document.querySelector('#cursor-server-table input[id^="ldui-data-table-search-"]');
                input.value = 'User 1';
                input.dispatchEvent(new Event('input', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    assert_eq!(testid_text(h, "cursor-query-proposals").await, "9");
    assert!(
        testid_text(h, "cursor-last-query")
            .await
            .contains("request=First")
    );

    let retained_rows = count_of(h, "#cursor-server-table tbody tr").await;
    click(h, "[data-testid='cursor-retain-loading']").await;
    let retained_loading = eval_json(
        h,
        r#"(() => {
            const table = document.querySelector('#cursor-server-table');
            const controls = table.querySelector('[data-server-cursor-state]');
            return {
                rows: table.querySelectorAll('tbody tr').length,
                state: controls.dataset.serverCursorState,
                status: controls.querySelector('[role="status"]').textContent.trim(),
                disabled: Array.from(controls.querySelectorAll('button')).every(button => button.disabled),
            };
        })()"#,
    )
    .await;
    assert_eq!(retained_loading["rows"], json!(retained_rows));
    assert_eq!(retained_loading["state"], json!("retained-loading"));
    assert!(
        retained_loading["status"]
            .as_str()
            .is_some_and(|text| text.contains("retained rows while loading"))
    );
    assert_eq!(retained_loading["disabled"], json!(true));

    click(h, "[data-testid='cursor-retain-failure']").await;
    let retained_failure = eval_json(
        h,
        r#"(() => {
            const table = document.querySelector('#cursor-server-table');
            const controls = table.querySelector('[data-server-cursor-state]');
            return {
                rows: table.querySelectorAll('tbody tr').length,
                state: controls.dataset.serverCursorState,
                status: controls.querySelector('[role="status"]').textContent.trim(),
            };
        })()"#,
    )
    .await;
    assert_eq!(retained_failure["rows"], json!(retained_rows));
    assert_eq!(retained_failure["state"], json!("retained-failure"));
    assert!(
        retained_failure["status"]
            .as_str()
            .is_some_and(|text| text.contains("latest request failed"))
    );

    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const error = document.querySelector('#cursor-mixed-config [data-server-pagination-config-error]');
                return {
                    role: error.getAttribute('role'),
                    message: error.dataset.serverPaginationConfigError,
                    tables: document.querySelectorAll('#cursor-mixed-config table').length,
                };
            })()"#,
        )
        .await,
        json!({
            "role": "alert",
            "message": "ServerDataTable pagination is mutually exclusive with legacy offset props",
            "tables": 0,
        })
    );

    assert_server_table_controls_match_declared_query_capabilities(h).await;
}

/// Server query capabilities (ldui-8v5): a fixed cursor slice exposes only
/// Previous/Next and cannot emit unsupported shape proposals, while a mixed
/// policy independently enables search/sort and the omitted policy preserves
/// the historical full-query controls.
async fn assert_server_table_controls_match_declared_query_capabilities(
    h: &pixelproof_web::Harness,
) {
    let navigation_only = eval_json(
        h,
        r#"(() => {
            const table = document.querySelector('#cursor-navigation-only-table');
            return {
                search: table.dataset.serverQuerySearch,
                pageSize: table.dataset.serverQueryPageSize,
                sorting: table.dataset.serverQuerySorting,
                filtering: table.dataset.serverQueryFiltering,
                textInputs: table.querySelectorAll('input[type="text"]').length,
                pageSizeSelects: table.querySelectorAll('select[id$="-page-size"]').length,
                filterRows: table.querySelectorAll('[data-table-filter-row]').length,
                sortButtons: table.querySelectorAll('[data-table-sort-state]').length,
                navigationButtons: table.querySelectorAll('[data-server-cursor-action]').length,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        navigation_only,
        json!({
            "search": "disabled",
            "pageSize": "disabled",
            "sorting": "disabled",
            "filtering": "disabled",
            "textInputs": 0,
            "pageSizeSelects": 0,
            "filterRows": 0,
            "sortButtons": 0,
            "navigationButtons": 2,
        })
    );

    click(h, "#cursor-navigation-only-table thead th:first-child").await;
    assert_eq!(
        testid_text(h, "cursor-navigation-only-proposals").await,
        "0",
        "an inert header cannot produce an unsupported sort proposal"
    );
    click(
        h,
        "#cursor-navigation-only-table [data-server-cursor-action='next']",
    )
    .await;
    assert_eq!(
        testid_text(h, "cursor-navigation-only-proposals").await,
        "1",
        "cursor navigation remains independently operable"
    );

    let mixed = eval_json(
        h,
        r#"(() => {
            const table = document.querySelector('#cursor-mixed-capability-table');
            return {
                search: table.dataset.serverQuerySearch,
                pageSize: table.dataset.serverQueryPageSize,
                sorting: table.dataset.serverQuerySorting,
                filtering: table.dataset.serverQueryFiltering,
                textInputs: table.querySelectorAll('input[type="text"]').length,
                pageSizeSelects: table.querySelectorAll('select[id$="-page-size"]').length,
                filterRows: table.querySelectorAll('[data-table-filter-row]').length,
                sortButtons: table.querySelectorAll('[data-table-sort-state]').length,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        mixed,
        json!({
            "search": "enabled",
            "pageSize": "disabled",
            "sorting": "enabled",
            "filtering": "disabled",
            "textInputs": 1,
            "pageSizeSelects": 0,
            "filterRows": 0,
            "sortButtons": 3,
        })
    );
    click(h, "#cursor-mixed-capability-table thead th:first-child").await;
    assert_eq!(
        testid_text(h, "cursor-mixed-capability-proposals").await,
        "1"
    );
    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const input = document.querySelector('#cursor-mixed-capability-table input[type="text"]');
                input.value = 'matter';
                input.dispatchEvent(new Event('input', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    assert_eq!(
        testid_text(h, "cursor-mixed-capability-proposals").await,
        "2"
    );

    let compatible_default = eval_json(
        h,
        r#"(() => {
            const table = document.querySelector('#cursor-server-table');
            return {
                markers: [
                    table.dataset.serverQuerySearch,
                    table.dataset.serverQueryPageSize,
                    table.dataset.serverQuerySorting,
                    table.dataset.serverQueryFiltering,
                ],
                search: table.querySelectorAll('input[type="text"]').length,
                pageSize: table.querySelectorAll('select[id$="-page-size"]').length,
                filters: table.querySelectorAll('[data-table-filter-row]').length,
                sorts: table.querySelectorAll('[data-table-sort-state]').length,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        compatible_default["markers"],
        json!(["enabled", "enabled", "enabled", "enabled"])
    );
    assert_eq!(compatible_default["search"], json!(1));
    assert_eq!(compatible_default["pageSize"], json!(1));
    assert_eq!(compatible_default["filters"], json!(1));
    assert_eq!(compatible_default["sorts"], json!(3));

    assert_eq!(
        eval_json(
            h,
            r#"(() => {
                const error = document.querySelector('#cursor-capability-conflict [data-server-query-capability-config-error]');
                return { role: error.getAttribute('role'), message: error.textContent.trim() };
            })()"#,
        )
        .await,
        json!({
            "role": "alert",
            "message": "ServerDataTable query enables search while the search capability is disabled",
        })
    );
}

/// Viewport-fit query sizing (ldui-2bt3): a controlled offset `ServerDataTable`
/// with `viewport_fit=true` measures the rendered height exactly like
/// `DataTable`'s `auto_page_size` and proposes a page-size query -- growing
/// past the configured size in a tall container, and retaining the last
/// accepted size (never proposing a shrink below the usability floor) once
/// the container drops below it, while the bounded wrapper scrolls instead.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn viewport_fit_offset_proposes_growth_and_retains_below_the_floor_while_scrolling() {
    let h = harness_at("/components/data-table").await;

    let snapshot_expr = r#"(() => {
        const root = document.querySelector('#viewport-fit-offset-server-table');
        const viewport = root.querySelector(':scope > .overflow-x-auto');
        return {
            rows: root.querySelectorAll('tbody tr').length,
            viewportHeight: viewport.clientHeight,
            scrollHeight: viewport.scrollHeight,
        };
    })()"#;

    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let settled = eval_json(&h, snapshot_expr).await;
    let settled_rows = settled["rows"].as_u64().unwrap_or(0);
    // Not ">= the configured page_size(5)": this fixture's default `h-96`
    // container leaves a genuinely shorter scroll wrapper than five 46px
    // rows under a 53px header actually needs, so a real (post-ldui-2bt3
    // epoch-fix) viewport-fit pass legitimately measures and accepts 4 on
    // first settle -- a correct responsive fit, not a bug, since 4 is still
    // >= this table's `viewport_fit_min_rows(3)` usability floor. The prior
    // ">= 5" expectation only ever passed because a `ViewportFitEpoch`
    // bug (fixed alongside this test) discarded every measurement pass, so
    // the accepted size silently stayed frozen at the initial query's
    // page_size regardless of what the container actually fit.
    assert!(
        settled_rows >= 3,
        "initial settle should show at least this table's usability floor: {settled}"
    );

    // Grow the container: the measured fit must widen the accepted page
    // size and the proposal must reset to page one.
    h.page()
        .evaluate(
            r#"(() => {
                document.querySelector('#viewport-fit-offset-server-table').parentElement.style.height = '900px';
                return true;
            })()"#,
        )
        .await
        .expect("grow the viewport-fit container");
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let grown = eval_json(&h, snapshot_expr).await;
    let grown_rows = grown["rows"].as_u64().unwrap_or(0);
    assert!(
        grown_rows > settled_rows,
        "a taller viewport must propose and accept a larger page size: settled={settled} grown={grown}"
    );
    assert!(
        testid_text(&h, "viewport-fit-last-query")
            .await
            .starts_with("page=1 "),
        "offset viewport-fit proposals must reset to page one"
    );

    // Shrink well below the usability floor: the accepted size must be
    // retained (no shrinking proposal below min_rows) and the bounded
    // wrapper must scroll instead.
    h.page()
        .evaluate(
            r#"(() => {
                document.querySelector('#viewport-fit-offset-server-table').parentElement.style.height = '110px';
                return true;
            })()"#,
        )
        .await
        .expect("shrink the viewport-fit container below the usability floor");
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let shrunk = eval_json(&h, snapshot_expr).await;
    assert_eq!(
        shrunk["rows"], grown["rows"],
        "a fit below min_rows must retain the last accepted page size rather than \
         shrinking it: grown={grown} shrunk={shrunk}"
    );
    assert!(
        shrunk["scrollHeight"].as_u64() > shrunk["viewportHeight"].as_u64(),
        "the bounded wrapper must scroll rather than collapsing pagination: {shrunk}"
    );
}

/// Rapid, narrow resize (ldui-2bt3): several resizes in quick succession,
/// ending narrow enough to force a horizontal scrollbar, must still settle
/// to a fixed page size (the `offset_height`-based measurement is immune to
/// the scrollbar the way `client_height` is not) rather than oscillating.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn viewport_fit_offset_rapid_narrow_resize_settles_without_oscillation() {
    let h = harness_at("/components/data-table").await;

    let snapshot_expr = r#"(() => {
        const root = document.querySelector('#viewport-fit-offset-server-table');
        return { rows: root.querySelectorAll('tbody tr').length };
    })()"#;

    for (height, width) in [("260px", Some("180px")), ("640px", None), ("420px", None)] {
        h.page()
            .evaluate(
                format!(
                    r#"(() => {{
                        const root = document.querySelector('#viewport-fit-offset-server-table').parentElement;
                        root.style.height = '{height}';
                        {width_assignment}
                        return true;
                    }})()"#,
                    width_assignment = width
                        .map(|w| format!("root.style.width = '{w}';"))
                        .unwrap_or_default(),
                )
                .as_str(),
            )
            .await
            .expect("resize the viewport-fit container");
    }

    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let first_rows = eval_json(&h, snapshot_expr).await;
    let first_proposals = testid_text(&h, "viewport-fit-proposals").await;
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let second_rows = eval_json(&h, snapshot_expr).await;
    let second_proposals = testid_text(&h, "viewport-fit-proposals").await;

    assert!(
        first_rows["rows"].as_u64().unwrap_or(0) > 0,
        "expected at least one rendered row: {first_rows}"
    );
    assert_eq!(
        first_rows["rows"], second_rows["rows"],
        "viewport_fit did not settle -- the derived page size is still changing \
         (oscillation): first={first_rows} second={second_rows}"
    );
    assert_eq!(
        first_proposals, second_proposals,
        "a settled table must stop emitting proposals once the fixed point is reached"
    );
}

/// Own-induced refetch across DIFFERING row heights (ldui-2bt3 CRITICAL
/// fix): the offset fixture's row "009" (0-based index 8) renders as a
/// deliberately tall wrapped cell, mirroring the client `auto_page_size`
/// ldui-89rp regression guard. Growing the container is guaranteed to
/// eventually accept a page size that includes it, which then proposes
/// shrinking back down -- and the shrunk (short-only) page must not
/// propose growing again forever. Two readings apart with a stable
/// proposal count is the settlement proof; without carrying the
/// `RowHeightEra` high-water mark across an own-induced refetch, this
/// exact shape oscillates.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn viewport_fit_offset_own_induced_refetch_settles_with_differing_row_heights() {
    let h = harness_at("/components/data-table").await;

    h.page()
        .evaluate(
            r#"(() => {
                document.querySelector('#viewport-fit-offset-server-table').parentElement.style.height = '900px';
                return true;
            })()"#,
        )
        .await
        .expect("grow the viewport-fit container so the tall row is eventually included");

    let rows_expr =
        r#"document.querySelectorAll('#viewport-fit-offset-server-table tbody tr').length"#;
    tokio::time::sleep(std::time::Duration::from_millis(900)).await;
    let first_rows = eval_json(&h, rows_expr).await;
    let first_proposals = testid_text(&h, "viewport-fit-proposals").await;

    tokio::time::sleep(std::time::Duration::from_millis(900)).await;
    let second_rows = eval_json(&h, rows_expr).await;
    let second_proposals = testid_text(&h, "viewport-fit-proposals").await;

    assert_eq!(
        first_rows, second_rows,
        "an own-induced refetch across differing row heights did not settle: \
         first={first_rows} second={second_rows}"
    );
    assert_eq!(
        first_proposals, second_proposals,
        "a settled table must stop emitting proposals once the fixed point is reached"
    );
}

/// Declined proposals (ldui-2bt3): when the caller rejects a viewport-fit
/// page-size proposal, the displayed accepted rows and size must be
/// retained exactly -- a rejected proposal is not applied speculatively.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn viewport_fit_declined_offset_proposal_retains_accepted_rows_and_size() {
    let h = harness_at("/components/data-table").await;

    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let snapshot_expr = r#"(() => {
        const root = document.querySelector('#viewport-fit-offset-server-table');
        return {
            rows: root.querySelectorAll('tbody tr').length,
            lastQuery: document.querySelector('[data-testid="viewport-fit-last-query"]').textContent,
        };
    })()"#;
    let before = eval_json(&h, snapshot_expr).await;

    click(&h, "[data-testid='viewport-fit-accept']").await;

    h.page()
        .evaluate(
            r#"(() => {
                document.querySelector('#viewport-fit-offset-server-table').parentElement.style.height = '900px';
                return true;
            })()"#,
        )
        .await
        .expect("resize while declining proposals");
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    let after = eval_json(&h, snapshot_expr).await;
    assert_eq!(
        after["rows"], before["rows"],
        "a declined proposal must retain the accepted rows: before={before} after={after}"
    );
    assert!(
        after["lastQuery"]
            .as_str()
            .unwrap_or_default()
            .starts_with("declined:"),
        "the decline must be visible in the last-query readout: {after}"
    );
}

/// Cursor viewport-fit (ldui-2bt3): a page-size proposal against a cursor
/// query must request the server-defined first slice, never replaying a
/// previous/next token that was minted for the OLD size -- `ServerCursorRequest`
/// carries no size at all, so requesting `First` is structurally incapable of
/// reusing one.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn viewport_fit_cursor_resets_to_first_never_reusing_a_stale_token() {
    let h = harness_at("/components/data-table").await;

    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let before_rows = eval_json(
        &h,
        r#"document.querySelectorAll('#viewport-fit-cursor-server-table tbody tr').length"#,
    )
    .await;

    // Navigate forward once at the settled size, minting an opaque
    // previous/next token pair for that size.
    click(
        &h,
        "#viewport-fit-cursor-server-table [data-server-cursor-action='next']",
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        testid_text(&h, "viewport-fit-cursor-last-query")
            .await
            .starts_with("request=Next(offset:"),
        "expected the manual Next navigation to carry an opaque token"
    );

    // Grow the container: a genuine page-size proposal is due, and it must
    // request First rather than replaying the token just minted above.
    h.page()
        .evaluate(
            r#"(() => {
                document.querySelector('#viewport-fit-cursor-server-table').parentElement.style.height = '900px';
                return true;
            })()"#,
        )
        .await
        .expect("grow the cursor viewport-fit container");
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    let after_resize = testid_text(&h, "viewport-fit-cursor-last-query").await;
    assert!(
        after_resize.starts_with("request=First "),
        "a cursor viewport-fit proposal must request First: {after_resize}"
    );

    let after_rows = eval_json(
        &h,
        r#"document.querySelectorAll('#viewport-fit-cursor-server-table tbody tr').length"#,
    )
    .await;
    assert!(
        after_rows.as_u64().unwrap_or(0) > before_rows.as_u64().unwrap_or(0),
        "the accepted slice must reflect the larger proposed size: before={before_rows} after={after_rows}"
    );
}

/// Fail-closed configuration (ldui-2bt3): `viewport_fit=true` against a
/// fixed-slice (`navigation_only`) endpoint -- one that cannot accept a
/// page-size change -- must reject the policy visibly rather than silently
/// doing nothing, mirroring every other `ServerDataTable` configuration
/// error's `role="alert"` + `data-server-*-config-error` shape.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn viewport_fit_rejects_a_fixed_slice_endpoint_visibly() {
    let h = harness_at("/components/data-table").await;

    let result = eval_json(
        &h,
        r#"(() => {
            const container = document.querySelector('#viewport-fit-rejected-table');
            const table = container.querySelector('[data-table-data-mode="server-query"]');
            const error = container.querySelector('[data-server-viewport-fit-config-error]');
            return {
                status: table ? table.dataset.serverViewportFit : null,
                errorRole: error ? error.getAttribute('role') : null,
                errorMessage: error ? error.textContent.trim() : null,
                pageSizeSelects: container.querySelectorAll('select[id$="-page-size"]').length,
            };
        })()"#,
    )
    .await;

    assert_eq!(result["status"], json!("rejected"));
    assert_eq!(result["errorRole"], json!("alert"));
    assert_eq!(
        result["errorMessage"],
        json!(
            "ServerDataTable viewport_fit requires an endpoint that accepts page-size \
             changes (a fixed-slice endpoint or a disabled page-size capability rejects \
             the policy)"
        )
    );
    assert_eq!(result["pageSizeSelects"], json!(0));
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
    let sort_name = eval_json(
        &h,
        r#"document.querySelector('#localized-table thead th:first-child > button')
            ?.getAttribute('aria-label') ?? null"#,
    )
    .await;
    assert_eq!(
        sort_name,
        json!("Nombre, sin ordenar. Activar para ordenar ascendente."),
        "the mounted sort control's state/action copy must relocalize"
    );
}

/// Runtime-localized DataTable controls (ldui-rmc): changing locale must
/// relabel the mounted search and column-filter controls without clearing the
/// user's search text, selected filter, or active sort. This complements the
/// header-only localization proof above and guards the full stateful a11y
/// contract inherited by consumers.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn data_table_control_names_relocalize_without_resetting_state() {
    let h = harness_at("/components/data-table").await;
    begin_browser_error_capture(&h).await;

    let root = "#filter-row-table";
    click(
        &h,
        "#filter-row-table thead tr:first-child th:first-child > button",
    )
    .await;

    let armed = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector('#filter-row-table');
            const search = root?.querySelector('input[id^="ldui-data-table-search-"]');
            const textFilter = root?.querySelector('[data-table-filter-column="name"] input[data-table-filter-kind="contains"]');
            const filter = root?.querySelector('tr.data-table-filter-row select');
            if (!search || !textFilter || !filter) return false;
            search.value = 'User';
            search.dispatchEvent(new Event('input', { bubbles: true }));
            textFilter.value = 'USER 2';
            textFilter.dispatchEvent(new Event('input', { bubbles: true }));
            filter.value = 'Admin';
            filter.dispatchEvent(new Event('change', { bubbles: true }));
            return true;
        })()"#,
    )
    .await;
    assert_eq!(
        armed,
        json!(true),
        "localized fixture must expose searchable and filterable controls"
    );
    tokio::time::sleep(std::time::Duration::from_millis(450)).await;

    let describe = |locale: &'static str| {
        format!(
            r#"(() => {{
                const root = document.querySelector('{root}');
                const search = root.querySelector('input[id^="ldui-data-table-search-"]');
                const textFilter = root.querySelector('[data-table-filter-column="name"] input[data-table-filter-kind="contains"]');
                const filter = root.querySelector('tr.data-table-filter-row select');
                const labelText = element => Array.from(element.labels || [])
                    .map(label =>
                        (label.matches('.sr-only') ? label : label.querySelector('.sr-only'))
                            ?.textContent.trim() ?? ''
                    );
                return {{
                    locale: '{locale}',
                    rows: root.querySelectorAll('tbody tr:not([data-table-detail-row])').length,
                    sort: root.querySelector('thead tr:first-child th:first-child')?.getAttribute('aria-sort'),
                    search: {{
                        value: search.value,
                        placeholder: search.placeholder,
                        aria: search.getAttribute('aria-label'),
                        labels: labelText(search),
                    }},
                    textFilter: {{
                        value: textFilter.value,
                        aria: textFilter.getAttribute('aria-label'),
                        labels: labelText(textFilter),
                    }},
                    filter: {{
                        value: filter.value,
                        aria: filter.getAttribute('aria-label'),
                        labels: labelText(filter),
                        all: filter.options[0]?.textContent.trim(),
                    }},
                }};
            }})()"#
        )
    };

    let english = eval_json(&h, &describe("en")).await;
    assert_eq!(english["rows"], json!(2));
    assert_eq!(english["sort"], json!("ascending"));
    assert_eq!(english["search"]["value"], json!("User"));
    assert_eq!(english["search"]["placeholder"], json!("Search..."));
    assert_eq!(english["search"]["aria"], json!("Search table"));
    assert_eq!(english["search"]["labels"], json!(["Search table"]));
    assert_eq!(english["textFilter"]["value"], json!("USER 2"));
    assert_eq!(english["textFilter"]["aria"], json!("Filter Name by text"));
    assert_eq!(
        english["textFilter"]["labels"],
        json!(["Filter Name by text"])
    );
    assert_eq!(english["filter"]["value"], json!("Admin"));
    assert_eq!(english["filter"]["aria"], json!("Filter by Role"));
    assert_eq!(english["filter"]["labels"], json!(["Filter by Role"]));
    assert_eq!(english["filter"]["all"], json!("All"));

    click(&h, "#locale-toggle").await;
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    let spanish = eval_json(&h, &describe("es")).await;
    assert_eq!(spanish["rows"], json!(2), "locale change reset filtering");
    assert_eq!(
        spanish["sort"],
        json!("ascending"),
        "locale change reset sort"
    );
    assert_eq!(spanish["search"]["value"], json!("User"));
    assert_eq!(spanish["search"]["placeholder"], json!("Buscar..."));
    assert_eq!(spanish["search"]["aria"], json!("Buscar en la tabla"));
    assert_eq!(spanish["search"]["labels"], json!(["Buscar en la tabla"]));
    assert_eq!(spanish["textFilter"]["value"], json!("USER 2"));
    assert_eq!(
        spanish["textFilter"]["aria"],
        json!("Filtrar Nombre por texto")
    );
    assert_eq!(
        spanish["textFilter"]["labels"],
        json!(["Filtrar Nombre por texto"])
    );
    assert_eq!(spanish["filter"]["value"], json!("Admin"));
    assert_eq!(spanish["filter"]["aria"], json!("Filtrar por Rol"));
    assert_eq!(spanish["filter"]["labels"], json!(["Filtrar por Rol"]));
    assert_eq!(spanish["filter"]["all"], json!("Todos"));
    assert_no_browser_errors(&h, "localized DataTable control state").await;
}

/// FilterSidebar's optional search input (ldui-g66e): a real accessible name
/// independent of placeholder and typed value, reactive to a locale switch
/// without resetting the search signal/caret/focus/typed text, correct
/// regardless of collapsed/expanded state, and independently named per panel
/// -- including the right-docked orientation. `search` omission (no hidden
/// label emitted) is a structural, compile-time property of the source and
/// is proven natively in `filter_sidebar::tests` instead, since this fixed
/// demo page cannot itself omit the prop and still exercise the other five
/// variants.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn filter_sidebar_search_accessible_name_covers_every_variant() {
    let h = harness_at("/components/filter-sidebar").await;
    begin_browser_error_capture(&h).await;

    let describe = |panel_id: &'static str| {
        format!(
            r#"(() => {{
                const root = document.getElementById('{panel_id}');
                const input = root ? root.querySelector('input[type="text"]') : null;
                if (!input) return null;
                const labelText = Array.from(input.labels || []).map(label =>
                    (label.matches('.sr-only') ? label : label.querySelector('.sr-only'))
                        ?.textContent.trim() ?? ''
                );
                return {{
                    aria: input.getAttribute('aria-label'),
                    labels: labelText,
                    placeholder: input.placeholder,
                    value: input.value,
                }};
            }})()"#
        )
    };

    // ── empty: a real, nonempty accessible name, independent of placeholder ──
    let left = eval_json(&h, &describe("fs-interactive-left")).await;
    assert_eq!(left["value"], json!(""));
    assert_eq!(left["aria"], json!("Search filters"));
    assert_eq!(left["labels"], json!(["Search filters"]));
    assert_ne!(
        left["aria"], left["placeholder"],
        "the accessible name must not merely echo the placeholder text"
    );

    // ── right-side: independently named from the left panel ──
    let right = eval_json(&h, &describe("fs-interactive-right")).await;
    assert_eq!(right["aria"], json!("Search the assistant"));
    assert_eq!(right["labels"], json!(["Search the assistant"]));
    assert_ne!(
        left["aria"], right["aria"],
        "multiple FilterSidebars on one page must stay independently named"
    );

    // ── typed: the accessible name does not change while the user types ──
    let typed_aria = eval_json(
        &h,
        r#"(() => {
            const input = document.getElementById('fs-interactive-left').querySelector('input[type="text"]');
            input.value = 'acme';
            input.dispatchEvent(new Event('input', { bubbles: true }));
            return input.getAttribute('aria-label');
        })()"#,
    )
    .await;
    assert_eq!(
        typed_aria,
        json!("Search filters"),
        "typing must not change the accessible name"
    );

    // ── localized: the label reacts to a locale switch WITHOUT resetting the
    //    search signal, caret, or focus ──
    let before = eval_json(
        &h,
        r#"(() => {
            const input = document.getElementById('fs-interactive-left').querySelector('input[type="text"]');
            input.focus();
            input.setSelectionRange(2, 2);
            return {
                active: document.activeElement === input,
                start: input.selectionStart,
                end: input.selectionEnd,
                value: input.value,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        before["active"],
        json!(true),
        "the search input must be focusable"
    );
    assert_eq!(before["value"], json!("acme"));

    click(&h, "#filter-sidebar-locale-toggle").await;
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    let localized = eval_json(&h, &describe("fs-interactive-left")).await;
    assert_eq!(localized["aria"], json!("Buscar filtros"));
    assert_eq!(localized["labels"], json!(["Buscar filtros"]));
    assert_eq!(
        localized["value"],
        json!("acme"),
        "a locale switch must not clear the typed search value"
    );

    let after = eval_json(
        &h,
        r#"(() => {
            const input = document.getElementById('fs-interactive-left').querySelector('input[type="text"]');
            return {
                active: document.activeElement === input,
                start: input.selectionStart,
                end: input.selectionEnd,
                value: input.value,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        after, before,
        "a locale switch must not disturb focus, caret position, or the typed value"
    );

    // ── collapsed / expanded: the label is present and correct regardless of
    //    collapsed state -- collapsing hides content in place, it is never
    //    unmounted (see the component's own doc comments) ──
    let collapsed = eval_json(&h, &describe("fs-collapsed-left")).await;
    assert_eq!(collapsed["aria"], json!("Search filters (collapsed)"));
    assert_eq!(collapsed["labels"], json!(["Search filters (collapsed)"]));

    let expanded = eval_json(&h, &describe("fs-expanded-left")).await;
    assert_eq!(expanded["aria"], json!("Search filters (expanded)"));
    assert_eq!(expanded["labels"], json!(["Search filters (expanded)"]));

    // ── the input-outside-field audit no longer reports this control ──
    let font = common::body_font_family(&h).await;
    let profile = ldui_audit::from_ui_tokens(font);
    let report = ldui_audit::audit_page(&h, &profile, &Default::default())
        .await
        .expect("audit_page");
    let outstanding: Vec<_> = report
        .families
        .iter()
        .filter(|f| f.family == ldui_audit::family::COMPONENT_DRIFT)
        .flat_map(|f| f.violations.iter())
        .filter(|v| v.detail.contains("input-outside-field"))
        .collect();
    assert!(
        outstanding.is_empty(),
        "input-outside-field still reports on the filter-sidebar page: {outstanding:?}"
    );

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let axe_report = axe.run(h.page()).await.expect("run axe-core");
    axe_report
        .assert_no_blocking("FilterSidebar search accessible name")
        .unwrap_or_else(|error| panic!("{error}; {}", axe_report.summary()));

    assert_no_browser_errors(&h, "FilterSidebar search accessible name").await;
}

/// Tabs: controlled selection, roving focus, relationships, localization,
/// overflow, orientation, disabled skipping, and removal recovery stay coherent.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn tab_click_selects_via_oracle() {
    let h = harness_at("/components/tab").await;

    let initial = eval_json(
        &h,
        r#"(() => {
            const set = document.querySelector('#basic-tabs');
            const list = set.querySelector('[role="tablist"]');
            const tabs = Array.from(list.querySelectorAll('[role="tab"]'));
            const panels = Array.from(set.querySelectorAll('[role="tabpanel"]'));
            return {
                mode: set.dataset.tabset,
                listRole: list.getAttribute('role'),
                label: list.getAttribute('aria-label'),
                orientation: list.getAttribute('aria-orientation'),
                selected: tabs.filter(tab => tab.getAttribute('aria-selected') === 'true').map(tab => tab.textContent.trim()),
                roving: tabs.filter(tab => tab.tabIndex === 0).map(tab => tab.textContent.trim()),
                relationships: tabs.map(tab => ({
                    tabId: tab.id,
                    controls: tab.getAttribute('aria-controls'),
                    panelExists: !!document.getElementById(tab.getAttribute('aria-controls')),
                    panelLabelledBy: document.getElementById(tab.getAttribute('aria-controls'))?.getAttribute('aria-labelledby'),
                })),
                visiblePanels: panels.filter(panel => !panel.hidden).map(panel => panel.textContent.trim()),
            };
        })()"#,
    )
    .await;
    assert_eq!(initial["mode"], json!("controlled"));
    assert_eq!(initial["listRole"], json!("tablist"));
    assert_eq!(initial["label"], json!("Basic tabs"));
    assert_eq!(initial["orientation"], json!("horizontal"));
    assert_eq!(initial["selected"], json!(["Tab 1"]));
    assert_eq!(initial["roving"], json!(["Tab 1"]));
    assert_eq!(initial["visiblePanels"], json!(["Content for Tab 1"]));
    assert!(
        initial["relationships"]
            .as_array()
            .is_some_and(|relationships| relationships.iter().all(|relationship| {
                relationship["panelExists"] == json!(true)
                    && relationship["tabId"] == relationship["panelLabelledBy"]
            })),
        "every tab must own one labelled panel: {initial}"
    );

    click(&h, "main .tabs .tab:nth-child(2)").await;
    let s = oracle(&h).await;
    assert_eq!(s["state"]["tab.active"], json!(1), "oracle: {s}");

    click(&h, "main .tabs .tab:nth-child(3)").await;
    let s = oracle(&h).await;
    assert_eq!(s["state"]["tab.active"], json!(2), "oracle: {s}");

    click(&h, "#basic-tabs [role='tab']:nth-child(1)").await;
    h.page()
        .find_element("#basic-tabs [role='tab']:nth-child(1)")
        .await
        .expect("find first controlled tab")
        .focus()
        .await
        .expect("focus first controlled tab");
    h.press_key_sequence(&[Key::ArrowRight])
        .await
        .expect("move controlled tab focus");
    assert_eq!(
        eval_json(
            &h,
            r#"(() => ({
                focused: document.activeElement.textContent.trim(),
                selected: document.querySelector('#basic-tabs [role="tab"][aria-selected="true"]').textContent.trim(),
                roving: Array.from(document.querySelectorAll('#basic-tabs [role="tab"]')).filter(tab => tab.tabIndex === 0).map(tab => tab.textContent.trim()),
            }))()"#,
        )
        .await,
        json!({ "focused": "Tab 2", "selected": "Tab 1", "roving": ["Tab 2"] }),
        "Arrow movement is manual activation and must not mutate controlled selection"
    );
    h.press_key_sequence(&[Key::Enter])
        .await
        .expect("select focused controlled tab");
    let s = oracle(&h).await;
    assert_eq!(s["state"]["tab.active"], json!(1), "oracle: {s}");
    assert_eq!(
        eval_json(
            &h,
            r#"(() => ({
                selected: document.querySelector('#basic-tabs [role="tab"][aria-selected="true"]').textContent.trim(),
                panel: document.querySelector('#basic-tabs [role="tabpanel"]:not([hidden])').textContent.trim(),
            }))()"#,
        )
        .await,
        json!({ "selected": "Tab 2", "panel": "Content for Tab 2" })
    );
    h.press_key_sequence(&[Key::Tab])
        .await
        .expect("leave the tablist through its single roving stop");
    assert_eq!(
        eval_json(&h, "document.activeElement.getAttribute('role')").await,
        json!("tabpanel"),
        "Tab must leave the composite for the selected panel"
    );

    h.set_viewport(ViewportSize::new(320, 844))
        .await
        .expect("set compact controlled-tabs viewport");
    let compact = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="controlled-tab-fixture"]');
            const list = fixture.querySelector('[role="tablist"]');
            const gamma = fixture.querySelector('#controlled-tabs-fixture-tab-67616d6d61');
            return {
                scrollable: list.scrollWidth > list.clientWidth,
                overflowX: getComputedStyle(list).overflowX,
                pageOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth,
                gammaDisabled: gamma.getAttribute('aria-disabled'),
                gammaTabIndex: gamma.tabIndex,
                rovingCount: Array.from(list.querySelectorAll('[role="tab"]')).filter(tab => tab.tabIndex === 0).length,
            };
        })()"#,
    )
    .await;
    assert_eq!(compact["scrollable"], json!(true));
    assert!(matches!(
        compact["overflowX"].as_str(),
        Some("auto" | "scroll")
    ));
    assert_eq!(compact["pageOverflow"], json!(false));
    assert_eq!(compact["gammaDisabled"], json!("true"));
    assert_eq!(compact["gammaTabIndex"], json!(-1));
    assert_eq!(compact["rovingCount"], json!(1));

    click(&h, "[data-testid='tab-select-beta']").await;
    let beta = "#controlled-tabs-fixture-tab-62657461";
    h.page()
        .find_element(beta)
        .await
        .expect("find externally selected Beta tab")
        .focus()
        .await
        .expect("focus Beta tab");
    h.press_key_sequence(&[Key::ArrowRight])
        .await
        .expect("skip disabled Gamma tab");
    assert_eq!(
        eval_json(
            &h,
            r#"(() => ({
                focused: document.activeElement.id,
                selected: document.querySelector('#controlled-tabs-fixture [role="tab"][aria-selected="true"]').id,
            }))()"#,
        )
        .await,
        json!({
            "focused": "controlled-tabs-fixture-tab-64656c7461",
            "selected": "controlled-tabs-fixture-tab-62657461",
        })
    );
    h.press_key_sequence(&[Key::Space])
        .await
        .expect("Space selects focused Delta tab");
    assert_eq!(
        oracle(&h).await["state"]["tab.fixture.selected"],
        json!("delta")
    );
    h.press_key_sequence(&[Key::Home])
        .await
        .expect("Home focuses first enabled tab");
    assert_eq!(
        eval_json(&h, "document.activeElement.id").await,
        json!("controlled-tabs-fixture-tab-616c706861")
    );
    h.press_key_sequence(&[Key::End])
        .await
        .expect("End focuses last enabled tab");
    assert_eq!(
        eval_json(&h, "document.activeElement.id").await,
        json!("controlled-tabs-fixture-tab-7a657461")
    );

    let before_locale_ids = eval_json(
        &h,
        "Array.from(document.querySelectorAll('#controlled-tabs-fixture [role=\"tab\"]')).map(tab => tab.id)",
    )
    .await;
    click(&h, "[data-testid='tab-toggle-locale']").await;
    assert_eq!(
        eval_json(
            &h,
            r#"(() => ({
                label: document.querySelector('#controlled-tabs-fixture [role="tablist"]').getAttribute('aria-label'),
                first: document.querySelector('#controlled-tabs-fixture [role="tab"]').textContent.trim(),
                ids: Array.from(document.querySelectorAll('#controlled-tabs-fixture [role="tab"]')).map(tab => tab.id),
            }))()"#,
        )
        .await,
        json!({ "label": "Flujo de trabajo", "first": "Alfa", "ids": before_locale_ids })
    );

    click(&h, "[data-testid='tab-toggle-orientation']").await;
    h.page()
        .find_element("#controlled-tabs-fixture-tab-616c706861")
        .await
        .expect("find Alpha after orientation replacement")
        .focus()
        .await
        .expect("focus Alpha in vertical tabset");
    h.press_key_sequence(&[Key::ArrowDown])
        .await
        .expect("move through vertical tabs");
    assert_eq!(
        eval_json(
            &h,
            r#"(() => ({
                orientation: document.querySelector('#controlled-tabs-fixture [role="tablist"]').getAttribute('aria-orientation'),
                focused: document.activeElement.id,
            }))()"#,
        )
        .await,
        json!({ "orientation": "vertical", "focused": "controlled-tabs-fixture-tab-62657461" })
    );

    eval_json(
        &h,
        "document.querySelector('[data-testid=\"tab-select-beta\"]').click(); true",
    )
    .await;
    h.page()
        .find_element(beta)
        .await
        .expect("find Beta before removal")
        .focus()
        .await
        .expect("focus selected Beta before removal");
    eval_json(
        &h,
        "document.querySelector('[data-testid=\"tab-remove-beta\"]').click(); true",
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    assert_eq!(
        eval_json(
            &h,
            r#"(() => ({
                betaTabs: document.querySelectorAll('#controlled-tabs-fixture-tab-62657461').length,
                betaPanels: document.querySelectorAll('#controlled-tabs-fixture-panel-62657461').length,
                selected: document.querySelector('#controlled-tabs-fixture [role="tab"][aria-selected="true"]').id,
                focused: document.activeElement.id,
                visiblePanel: document.querySelector('#controlled-tabs-fixture [role="tabpanel"]:not([hidden])').textContent.trim(),
                rovingCount: Array.from(document.querySelectorAll('#controlled-tabs-fixture [role="tab"]')).filter(tab => tab.tabIndex === 0).length,
            }))()"#,
        )
        .await,
        json!({
            "betaTabs": 0,
            "betaPanels": 0,
            "selected": "controlled-tabs-fixture-tab-616c706861",
            "focused": "controlled-tabs-fixture-tab-616c706861",
            "visiblePanel": "Alpha panel",
            "rovingCount": 1,
        })
    );
    assert_no_browser_errors(&h, "controlled accessible Tabs").await;
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

// ── Button native form semantics (ldui-9vs) ──────────────────────────────
//
// The "Native Form Semantics" section of /components/button wires
// `ButtonType::{Button,Submit,Reset}` into `#button-type-form`, whose
// `on:submit`/`on:reset` handlers `prevent_default()` (no real navigation in
// a headless test) and count activations via the debug oracle at
// `button.form_submit_count` / `button.form_reset_count`.

/// Each `button_type` variant emits the matching native `type` attribute,
/// and the default (`ButtonType::Button`, no prop passed) stays "button" —
/// existing callers render unchanged.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn button_type_variants_emit_the_matching_type_attribute() {
    let h = harness_at("/components/button").await;

    let types = eval_json(
        &h,
        r#"(() => ({
            default: document.querySelector('#button-type-default').getAttribute('type'),
            submit: document.querySelector('#button-type-submit').getAttribute('type'),
            reset: document.querySelector('#button-type-reset').getAttribute('type'),
        }))()"#,
    )
    .await;
    assert_eq!(
        types,
        json!({ "default": "button", "submit": "submit", "reset": "reset" }),
        "emitted type attribute per ButtonType variant: {types}"
    );
    assert_no_browser_errors(&h, "Button type attribute variants").await;
}

/// A `ButtonType::Submit` button activates its containing form exactly once
/// per mouse click and once per keyboard activation (Enter, then Space) —
/// native `<button type="submit">` behavior, no JS in the component.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn submit_button_activates_form_once_per_click_and_per_key() {
    let h = harness_at("/components/button").await;

    let before = oracle(&h).await;
    assert_eq!(
        before["state"]["button.form_submit_count"],
        serde_json::Value::Null,
        "precondition: no submit yet: {before}"
    );

    click(&h, "#button-type-submit").await;
    let after_click = oracle(&h).await;
    assert_eq!(
        after_click["state"]["button.form_submit_count"],
        json!(1),
        "one click => exactly one submit: {after_click}"
    );

    h.page()
        .find_element("#button-type-submit")
        .await
        .expect("find submit button")
        .focus()
        .await
        .expect("focus submit button");
    h.press_key_sequence(&[Key::Enter])
        .await
        .expect("Enter on submit button");
    settle(&h).await;
    let after_enter = oracle(&h).await;
    assert_eq!(
        after_enter["state"]["button.form_submit_count"],
        json!(2),
        "one Enter => exactly one more submit: {after_enter}"
    );

    h.press_key_sequence(&[Key::Space])
        .await
        .expect("Space on submit button");
    settle(&h).await;
    let after_space = oracle(&h).await;
    assert_eq!(
        after_space["state"]["button.form_submit_count"],
        json!(3),
        "one Space => exactly one more submit: {after_space}"
    );
    assert_no_browser_errors(&h, "submit button click/Enter/Space").await;
}

/// Both the explicitly `disabled` and the `loading` submit buttons cannot
/// submit the form: a native `disabled` button dispatches no `click` at all,
/// so neither mouse click nor keyboard activation moves the counter.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn disabled_and_loading_submit_buttons_cannot_submit() {
    let h = harness_at("/components/button").await;

    let disabled_native = eval_json(
        &h,
        r#"(() => ({
            disabled: document.querySelector('#button-type-submit-disabled').disabled,
            loading: document.querySelector('#button-type-submit-loading').disabled,
        }))()"#,
    )
    .await;
    assert_eq!(
        disabled_native,
        json!({ "disabled": true, "loading": true }),
        "both must carry the native disabled DOM property: {disabled_native}"
    );

    click(&h, "#button-type-submit-disabled").await;
    click(&h, "#button-type-submit-loading").await;
    let after = oracle(&h).await;
    assert_eq!(
        after["state"]["button.form_submit_count"],
        serde_json::Value::Null,
        "disabled/loading submit clicks must not reach the form: {after}"
    );
    assert_no_browser_errors(&h, "disabled/loading submit buttons").await;
}

/// A `ButtonType::Reset` button restores the form's fields to their native
/// defaults (`defaultChecked`), not to any reactive signal value — no
/// `on_click` wiring exists on the Reset button in the fixture, so this is
/// entirely native `<form>` behavior.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn reset_button_restores_native_defaults() {
    let h = harness_at("/components/button").await;

    click(&h, "#button-type-form-checkbox").await;
    let checked_after_toggle = eval_json(
        &h,
        "document.querySelector('#button-type-form-checkbox').checked",
    )
    .await;
    assert_eq!(
        checked_after_toggle,
        json!(true),
        "checkbox must be checked before reset"
    );

    click(&h, "#button-type-reset").await;
    let checked_after_reset = eval_json(
        &h,
        "document.querySelector('#button-type-form-checkbox').checked",
    )
    .await;
    assert_eq!(
        checked_after_reset,
        json!(false),
        "native reset must restore defaultChecked (false)"
    );
    let after = oracle(&h).await;
    assert_eq!(
        after["state"]["button.form_reset_count"],
        json!(1),
        "reset fires the form's on:reset exactly once: {after}"
    );
    assert_no_browser_errors(&h, "reset button native behavior").await;
}

/// A duplicate spread `attr:type` beats the component's own `button_type`.
/// `#button-type-precedence-probe` sets `button_type=ButtonType::Reset`
/// *and* spreads `attr:type="submit"`; on this crate's CSR-only rendering
/// path the spread attribute is applied to the root element after the
/// component's own view is built, so `set_attribute` runs last and the
/// spread wins. This is the concrete verification behind Button's doc
/// comment "Precedence vs a spread `attr:type`" (ldui-9vs) — never rely on
/// this in real code, use `button_type` alone.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn spread_attr_type_overrides_the_button_type_prop() {
    let h = harness_at("/components/button").await;

    let emitted_type = eval_json(
        &h,
        "document.querySelector('#button-type-precedence-probe').getAttribute('type')",
    )
    .await;
    assert_eq!(
        emitted_type,
        json!("submit"),
        "a later spread attr:type overrides button_type on the CSR path: {emitted_type}"
    );
    assert_no_browser_errors(&h, "attr:type precedence probe").await;
}

/// Each temporal `InputType` variant (ldui-z16) emits the exact valid HTML
/// `type` token the spec requires -- not some caller-facing format string.
/// Exercises all five: `date`, `time`, `month`, `week`, `datetime-local`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn temporal_input_types_emit_the_exact_html_type_tokens() {
    let h = harness_at("/components/input").await;

    let tokens = eval_json(
        &h,
        r#"(() => ({
            date: document.querySelector('#input-type-date').getAttribute('type'),
            time: document.querySelector('#input-type-time').getAttribute('type'),
            month: document.querySelector('#input-type-month').getAttribute('type'),
            week: document.querySelector('#input-type-week').getAttribute('type'),
            datetimeLocal: document.querySelector('#input-type-datetime-local').getAttribute('type'),
        }))()"#,
    )
    .await;
    assert_eq!(tokens["date"], json!("date"), "{tokens}");
    assert_eq!(tokens["time"], json!("time"), "{tokens}");
    assert_eq!(tokens["month"], json!("month"), "{tokens}");
    assert_eq!(tokens["week"], json!("week"), "{tokens}");
    assert_eq!(
        tokens["datetimeLocal"],
        json!("datetime-local"),
        "the DateTimeLocal variant token is hyphenated, not `datetimelocal`: {tokens}"
    );
    assert_no_browser_errors(&h, "temporal input type tokens").await;
}

/// A temporal value typed into a controlled `Input` (native `.value` set +
/// a bubbling `input` event, same as a real keystroke) flows through
/// `on_input` into the owning signal and back out through `prop:value`
/// unchanged -- LDUI applies no parsing, reformatting, or timezone
/// normalization anywhere in that loop. Exercises date, time, month, and
/// datetime-local (the brief's required minimum), plus week.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn temporal_input_values_round_trip_through_controlled_value_without_normalization() {
    let h = harness_at("/components/input").await;

    let cases = [
        ("#input-type-date", "2031-11-03"),
        ("#input-type-time", "09:15"),
        ("#input-type-month", "2031-11"),
        ("#input-type-week", "2031-W44"),
        ("#input-type-datetime-local", "2031-11-03T09:15"),
    ];
    for (selector, typed) in cases {
        let script = format!(
            r#"(() => {{
                const input = document.querySelector('{selector}');
                input.value = '{typed}';
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                return input.value;
            }})()"#
        );
        let immediate = eval_json(&h, &script).await;
        assert_eq!(
            immediate,
            json!(typed),
            "{selector}: browser accepted the native-format value before any Leptos round trip"
        );
        settle(&h).await;
        let after_settle =
            eval_json(&h, &format!("document.querySelector('{selector}').value")).await;
        assert_eq!(
            after_settle,
            json!(typed),
            "{selector}: value must survive the on_input -> signal -> prop:value round trip \
             unchanged (no LDUI normalization)"
        );
    }
    assert_no_browser_errors(&h, "temporal input value round trip").await;
}

/// A duplicate spread `attr:type` beats the component's own `input_type`,
/// mirroring Button's `attr:type` precedence (ldui-9vs).
/// `#input-type-precedence-probe` sets `input_type=InputType::Date` *and*
/// spreads `attr:r#type="time"`; on this crate's CSR-only rendering path the
/// spread attribute is applied to the root element after the component's own
/// view is built, so `set_attribute` runs last and the spread wins. This is
/// the concrete verification behind Input's doc comment "Precedence vs a
/// spread `attr:type`" (ldui-z16) -- never rely on this in real code, use
/// `input_type` alone.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn spread_attr_type_overrides_the_input_type_prop() {
    let h = harness_at("/components/input").await;

    let emitted_type = eval_json(
        &h,
        "document.querySelector('#input-type-precedence-probe').getAttribute('type')",
    )
    .await;
    assert_eq!(
        emitted_type,
        json!("time"),
        "a later spread attr:type overrides input_type on the CSR path: {emitted_type}"
    );
    assert_no_browser_errors(&h, "input attr:type precedence probe").await;
}

/// FilterBar's `search` slot is optional (ldui-3br). The `actions-only`
/// fixture (`demo/src/demos/client_snapshot_list.rs`,
/// `data-testid="filter-bar-actions-only"`) supplies no `search` and no
/// column-filter `children`: the bar must render no `[data-filter-search]`
/// wrapper at all -- not an empty one -- while the framework actions row
/// (Reset plus the caller's own compat action) still renders and works.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn filter_bar_actions_only_omits_the_search_wrapper() {
    let h = harness_at("/components/client-snapshot-list").await;

    let shape = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector(
                '[data-testid="filter-bar-actions-only"] [data-filter-bar="local"]'
            );
            return {
                hasSearch: !!root.querySelector('[data-filter-search]'),
                hasSummary: !!root.querySelector('[data-filter-summary]'),
                hasResultCount: !!root.querySelector('[data-filter-result-count]'),
                hasActions: !!root.querySelector('[data-filter-actions]'),
                hasExportButton: !!root.querySelector('[data-testid="filter-bar-actions-only-export"]'),
            };
        })()"#,
    )
    .await;
    assert_eq!(shape["hasSearch"], json!(false), "{shape}");
    assert_eq!(shape["hasSummary"], json!(false), "{shape}");
    assert_eq!(shape["hasResultCount"], json!(false), "{shape}");
    assert_eq!(shape["hasActions"], json!(true), "{shape}");
    assert_eq!(shape["hasExportButton"], json!(true), "{shape}");

    click(
        &h,
        "[data-testid=\"filter-bar-actions-only\"] [data-filter-reset]",
    )
    .await;
    let reset_count =
        eval_json(&h, "document.querySelector('[data-testid=\"filter-bar-actions-only-reset-count\"]').textContent")
            .await;
    assert_eq!(
        reset_count,
        json!("1"),
        "Reset must still fire with no search slot present: {reset_count}"
    );
    assert_no_browser_errors(&h, "filter bar actions-only fixture").await;
}

/// The `columns-only` fixture (`data-testid="filter-bar-columns-only"`)
/// supplies a column filter (`children`) plus the active-filter chip summary
/// and result count, and no framework actions at all -- no `search`, no
/// `on_reset`, no `default_save`. No `[data-filter-search]` or
/// `[data-filter-actions]` wrapper should render, and the summary/result
/// count must still react to the column filter -- omitting `search` changes
/// none of that state wiring.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn filter_bar_columns_only_summary_omits_the_search_wrapper() {
    let h = harness_at("/components/client-snapshot-list").await;

    let before = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector(
                '[data-testid="filter-bar-columns-only"] [data-filter-bar="local"]'
            );
            return {
                hasSearch: !!root.querySelector('[data-filter-search]'),
                hasActions: !!root.querySelector('[data-filter-actions]'),
                hasPrioritySelect: !!root.querySelector('[data-testid="filter-bar-columns-only-priority"]'),
                resultText: root.querySelector('[data-filter-result-count]')?.textContent,
                chipCount: root.querySelectorAll('[data-active-filters] .badge').length,
            };
        })()"#,
    )
    .await;
    assert_eq!(before["hasSearch"], json!(false), "{before}");
    assert_eq!(before["hasActions"], json!(false), "{before}");
    assert_eq!(before["hasPrioritySelect"], json!(true), "{before}");
    assert_eq!(before["resultText"], json!("9 of 9 results"), "{before}");
    assert_eq!(before["chipCount"], json!(0), "{before}");

    eval_json(
        &h,
        r#"(() => {
            const select = document.querySelector('[data-testid="filter-bar-columns-only-priority"]');
            select.value = 'Urgent';
            select.dispatchEvent(new Event('change', { bubbles: true }));
            return true;
        })()"#,
    )
    .await;
    settle(&h).await;

    let after = eval_json(
        &h,
        r#"(() => {
            const root = document.querySelector(
                '[data-testid="filter-bar-columns-only"] [data-filter-bar="local"]'
            );
            return {
                resultText: root.querySelector('[data-filter-result-count]')?.textContent,
                chipCount: root.querySelectorAll('[data-active-filters] .badge').length,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        after["resultText"],
        json!("3 of 9 results"),
        "the result summary must react to the column filter with no search present: {after}"
    );
    assert_eq!(after["chipCount"], json!(1), "{after}");
    assert_no_browser_errors(&h, "filter bar columns-only fixture").await;
}

/// The pre-existing search-backed FilterBar on the same page (no
/// `data-testid` wrapper -- it is the client-snapshot table's own filter
/// row) must keep rendering `[data-filter-search]` first, unaffected by the
/// slot becoming optional (ldui-3br: existing search callers render
/// compatibly).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo make test-visual)"]
async fn filter_bar_ordinary_search_configuration_still_renders_the_search_wrapper_first() {
    let h = harness_at("/components/client-snapshot-list").await;

    let shape = eval_json(
        &h,
        r#"(() => {
            const bars = document.querySelectorAll('[data-filter-bar="local"]');
            const root = bars[0];
            return {
                total: bars.length,
                firstChildIsSearch: root.children[0]?.hasAttribute('data-filter-search'),
                hasSearchInput: !!root.querySelector('[data-filter-search] input'),
                hasActions: !!root.querySelector('[data-filter-actions]'),
            };
        })()"#,
    )
    .await;
    assert_eq!(shape["total"], json!(3), "{shape}");
    assert_eq!(shape["firstChildIsSearch"], json!(true), "{shape}");
    assert_eq!(shape["hasSearchInput"], json!(true), "{shape}");
    assert_eq!(shape["hasActions"], json!(true), "{shape}");
    assert_no_browser_errors(&h, "filter bar ordinary search configuration").await;
}
