//! Targeted browser proof for the typed client-snapshot list architecture.

mod common;

use common::{
    assert_no_browser_errors, assert_not_truncated, begin_browser_error_capture, body_font_family,
    click, force_desktop_hover_media, harness_at, move_pointer_to_svg_fraction, oracle,
    shift_click, shift_enter, wait_for_selector,
};
use ldui_audit::{Ceiling, ShadowSpec, family};
use pixelproof_web::{Key, ViewportSize};
use serde_json::{Value, json};
use std::time::Duration;

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .unwrap_or_else(|error| panic!("evaluate `{expression}`: {error}"))
        .into_value()
        .unwrap_or_else(|error| panic!("JSON value for `{expression}`: {error}"))
}

async fn assert_entity_projection_matches_wide_dom(
    harness: &pixelproof_web::Harness,
    context: &str,
) {
    let state = oracle(harness).await;
    let projection = &state["state"]["entity_table.display_projection"];
    let projected_columns = projection["columns"]
        .as_array()
        .unwrap_or_else(|| panic!("missing projected columns for {context}: {projection}"));
    let column_ids = projected_columns
        .iter()
        .map(|column| column["id"].clone())
        .collect::<Vec<_>>();
    let encoded_ids = serde_json::to_string(&column_ids).expect("encode projected column IDs");
    let dom = eval_json(
        harness,
        &format!(
            r#"(() => {{
                const ids = {encoded_ids};
                const table = document.querySelector('[data-entity-table-grid]');
                const columns = ids.map(id => {{
                    const th = table.querySelector(`thead tr:first-child th[data-entity-column="${{id}}"]`);
                    const sort = th.querySelector('[data-entity-sort-column]');
                    const label = sort
                        ? sort.querySelector(':scope > span > span:first-child').textContent.trim()
                        : th.querySelector(':scope > span').textContent.trim();
                    return {{ id, label, is_action: false }};
                }});
                const rows = Array.from(table.querySelectorAll('tbody tr[data-entity-row-key]')).map(row => ({{
                    key: row.dataset.entityRowKey,
                    cells: ids.map(id => row.querySelector(`td[data-entity-column="${{id}}"]`).textContent.trim()),
                }}));
                return {{ columns, rows }};
            }})()"#,
        ),
    )
    .await;
    assert_eq!(
        dom["columns"], projection["columns"],
        "projected columns diverged from the wide DOM after {context}"
    );
    let all_rows = projection["all_filtered_rows"]
        .as_array()
        .unwrap_or_else(|| panic!("missing projected rows for {context}: {projection}"));
    let start = projection["current_page_start"]
        .as_u64()
        .expect("projected current-page start") as usize;
    let end = projection["current_page_end"]
        .as_u64()
        .expect("projected current-page end") as usize;
    assert_eq!(
        dom["rows"],
        Value::Array(all_rows[start..end].to_vec()),
        "projected current-page rows diverged from the wide DOM after {context}"
    );
}

async fn mark_entity_table_geometry(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('[data-entity-table]');
            const table = root.querySelector('[data-entity-table-grid]');
            const viewport = table.parentElement.parentElement;
            const headers = Array.from(table.querySelectorAll('thead tr:first-child th'));
            const cells = Array.from(table.querySelectorAll('tbody tr:first-child td'))
                .filter(cell => getComputedStyle(cell).display !== 'none');
            const box = element => {
                const rect = element.getBoundingClientRect();
                return [rect.x, rect.y, rect.width, rect.height, rect.right, rect.bottom];
            };
            table.dataset.geometryNodeId = `entity-table-${Math.random()}`;
            headers.forEach((cell, index) => {
                cell.dataset.geometryNodeId = `entity-header-${index}-${Math.random()}`;
            });
            if (viewport.scrollWidth > viewport.clientWidth) {
                viewport.scrollLeft = Math.min(73, viewport.scrollWidth - viewport.clientWidth);
            }
            root.__lduiEntityGeometryBaseline = {
                table: box(table),
                viewport: box(viewport),
                headers: headers.map(box),
                cells: cells.map(box),
                tableNode: table.dataset.geometryNodeId,
                headerNodes: headers.map(cell => cell.dataset.geometryNodeId),
                scrollLeft: viewport.scrollLeft,
            };
            return {
                headers: headers.length,
                cells: cells.length,
                scrollLeft: viewport.scrollLeft,
                scrollWidth: viewport.scrollWidth,
                clientWidth: viewport.clientWidth,
            };
        })()"#,
    )
    .await
}

async fn compare_entity_table_geometry(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('[data-entity-table]');
            const before = root.__lduiEntityGeometryBaseline;
            const table = root.querySelector('[data-entity-table-grid]');
            const viewport = table.parentElement.parentElement;
            const headers = Array.from(table.querySelectorAll('thead tr:first-child th'));
            const cells = Array.from(table.querySelectorAll('tbody tr:first-child td'))
                .filter(cell => getComputedStyle(cell).display !== 'none');
            const box = element => {
                const rect = element.getBoundingClientRect();
                return [rect.x, rect.y, rect.width, rect.height, rect.right, rect.bottom];
            };
            const after = {
                table: box(table),
                viewport: box(viewport),
                headers: headers.map(box),
                cells: cells.map(box),
                tableNode: table.dataset.geometryNodeId ?? null,
                headerNodes: headers.map(cell => cell.dataset.geometryNodeId ?? null),
                scrollLeft: viewport.scrollLeft,
            };
            const deltas = [];
            const visit = (left, right) => {
                if (Array.isArray(left)) {
                    left.forEach((value, index) => visit(value, right[index]));
                } else {
                    deltas.push(Math.abs(left - right));
                }
            };
            visit(before.table, after.table);
            visit(before.viewport, after.viewport);
            visit(before.headers, after.headers);
            visit(before.cells, after.cells);
            return {
                maxDelta: Math.max(0, ...deltas),
                sameTableNode: before.tableNode === after.tableNode,
                sameHeaderNodes: JSON.stringify(before.headerNodes) === JSON.stringify(after.headerNodes),
                beforeScrollLeft: before.scrollLeft,
                afterScrollLeft: after.scrollLeft,
                before,
                after,
            };
        })()"#,
    )
    .await
}

fn assert_entity_table_geometry_unchanged(result: &Value, journey: &str) {
    assert_eq!(
        result["sameTableNode"],
        json!(true),
        "{journey} replaced the table node: {result}"
    );
    assert_eq!(
        result["sameHeaderNodes"],
        json!(true),
        "{journey} replaced header nodes: {result}"
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

async fn viewport_fit_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#entity-viewport-fit-fixture [data-entity-table]');
            const region = root.querySelector('[data-entity-focus-region]');
            const rows = Array.from(root.querySelectorAll('[data-entity-table-grid] tbody tr'));
            const effective = Number(root.dataset.entityEffectivePageSize);
            const configured = Number(root.dataset.entityConfiguredPageSize);
            const pageButtons = Array.from(root.querySelectorAll('[data-entity-page]'))
                .filter(button => /^\d+$/.test(button.dataset.entityPage));
            const currentPage = Number(pageButtons.find(button => button.disabled)?.dataset.entityPage ?? 1);
            const verticalScrollers = Array.from(root.querySelectorAll('*')).filter(element => {
                const overflow = getComputedStyle(element).overflowY;
                return (overflow === 'auto' || overflow === 'scroll')
                    && element.scrollHeight > element.clientHeight + 1;
            });
            const last = rows.at(-1)?.getBoundingClientRect();
            const regionRect = region.getBoundingClientRect();
            const viewportWidth = document.documentElement.clientWidth;
            const overflowing = Array.from(document.querySelectorAll('body *'))
                .map(element => ({ element, rect: element.getBoundingClientRect() }))
                .filter(({ rect }) => rect.right > viewportWidth + 1 || rect.left < -1)
                .slice(0, 8)
                .map(({ element, rect }) => ({
                    tag: element.tagName,
                    id: element.id,
                    className: String(element.className),
                    left: rect.left,
                    right: rect.right,
                    width: rect.width,
                }));
            return {
                effective,
                configured,
                rows: rows.length,
                first: rows[0]?.dataset.rowKey ?? null,
                currentPage,
                totalPages: Math.max(1, Math.ceil(60 / effective)),
                rootHeight: root.getBoundingClientRect().height,
                regionHeight: region.clientHeight,
                regionScrollHeight: region.scrollHeight,
                regionScrolls: region.scrollHeight > region.clientHeight + 1,
                verticalScrollers: verticalScrollers.length,
                clippedLastRow: !!last && last.bottom > regionRect.bottom + 1,
                pageOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
                documentWidth: document.documentElement.scrollWidth,
                viewportWidth,
                overflowing,
                rowsPerPageLabel: root.querySelector('label span')?.textContent.trim() ?? null,
                rowHeights: rows.map(row => row.getBoundingClientRect().height),
                rowTops: rows.map(row => row.getBoundingClientRect().top - regionRect.top),
                tableHeight: root.querySelector('[data-entity-table-grid]').getBoundingClientRect().height,
                headerHeight: root.querySelector('thead')?.getBoundingClientRect().height ?? null,
                regionOffsetHeight: region.offsetHeight,
                horizontalScrollbar: region.offsetHeight - region.clientHeight,
            };
        })()"#,
    )
    .await
}

/// Reads the four surfaces ldui-5p06 requires to agree -- the rendered body,
/// the `Showing x-y of z` summary, the rows-per-page control, and the pager --
/// plus the consumer-visible `on_page_size_resolved` readout, in ONE
/// evaluation so they are sampled from a single rendered frame.
async fn page_size_agreement_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#entity-viewport-fit-fixture [data-entity-table]');
            const select = root.querySelector('#viewport-fit-page-size');
            const selectedOption = select.options[select.selectedIndex];
            const rows = root.querySelectorAll('[data-entity-table-grid] tbody tr').length;
            const summary = root.querySelector('[data-entity-table-footer] span.text-sm')
                ?.textContent.trim() ?? null;
            const match = /Showing (\d+)-(\d+) of (\d+)/.exec(summary ?? '');
            const pageButtons = Array.from(root.querySelectorAll('[data-entity-page]'))
                .filter(button => /^\d+$/.test(button.dataset.entityPage));
            return {
                mode: root.dataset.entityPageSizeMode,
                effective: Number(root.dataset.entityEffectivePageSize),
                configured: Number(root.dataset.entityConfiguredPageSize),
                rows,
                controlValue: select.value,
                controlLabel: selectedOption ? selectedOption.textContent.trim() : null,
                optionValues: Array.from(select.options).map(option => option.value),
                summary,
                summaryStart: match ? Number(match[1]) : null,
                summaryEnd: match ? Number(match[2]) : null,
                summaryTotal: match ? Number(match[3]) : null,
                advertisedPages: pageButtons.length,
                lastAdvertisedPage: pageButtons.length
                    ? Number(pageButtons.at(-1).dataset.entityPage)
                    : null,
                nextDisabled: root.querySelector('[data-entity-page="next"]').disabled,
                resolved: document.querySelector('[data-testid="viewport-fit-resolved"]')
                    ?.textContent.trim() ?? null,
            };
        })()"#,
    )
    .await
}

fn assert_one_page_size_everywhere(snapshot: &Value, context: &str) {
    let effective = snapshot["effective"]
        .as_u64()
        .unwrap_or_else(|| panic!("{context}: no effective page size: {snapshot}"));
    let total = snapshot["summaryTotal"]
        .as_u64()
        .unwrap_or_else(|| panic!("{context}: no summary total: {snapshot}"));
    let start = snapshot["summaryStart"].as_u64().expect("summary start");
    let end = snapshot["summaryEnd"].as_u64().expect("summary end");

    assert_eq!(
        snapshot["rows"].as_u64(),
        Some(effective.min(total)),
        "{context}: the body renders a different count than the resolved page size: {snapshot}"
    );
    assert_eq!(
        end - start + 1,
        snapshot["rows"].as_u64().expect("rendered rows"),
        "{context}: the summary range disagrees with the rendered body: {snapshot}"
    );
    assert_eq!(
        snapshot["lastAdvertisedPage"].as_u64(),
        Some(total.div_ceil(effective)),
        "{context}: the pager advertises a page count from a different size: {snapshot}"
    );
    // The control names the MODE, and its label carries the resolved count --
    // it can never read a number the body is not rendering.
    let control_value = snapshot["controlValue"].as_str().expect("control value");
    let control_label = snapshot["controlLabel"].as_str().expect("control label");
    if snapshot["mode"] == json!("auto") {
        assert_eq!(control_value, "auto", "{context}: {snapshot}");
        assert!(
            control_label.contains(&effective.to_string()),
            "{context}: the auto option must name the rows it fitted: {snapshot}"
        );
        assert_eq!(
            snapshot["resolved"].as_str(),
            Some(format!("auto:{effective}").as_str()),
            "{context}: on_page_size_resolved disagrees with the DOM: {snapshot}"
        );
    } else {
        assert_eq!(
            control_value,
            effective.to_string(),
            "{context}: {snapshot}"
        );
        assert_eq!(
            control_label,
            effective.to_string(),
            "{context}: {snapshot}"
        );
        assert_eq!(
            snapshot["resolved"].as_str(),
            Some(format!("fixed:{effective}").as_str()),
            "{context}: on_page_size_resolved disagrees with the DOM: {snapshot}"
        );
    }
}

async fn choose_page_size(harness: &pixelproof_web::Harness, value: &str) {
    let expression = format!(
        r#"(() => {{
            const select = document.querySelector('#viewport-fit-page-size');
            select.value = {value:?};
            select.dispatchEvent(new Event('change', {{ bubbles: true }}));
            return select.value;
        }})()"#
    );
    assert_eq!(
        eval_json(harness, &expression).await,
        json!(value),
        "the rows-per-page control did not accept `{value}`"
    );
    tokio::time::sleep(Duration::from_millis(250)).await;
}

/// ldui-5p06: with 17 rows, a control reading `25` may never sit over a
/// five-row body advertising four pages. Auto is an explicit choice that names
/// its own fitted count; a numeric choice renders that many rows.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn seventeen_rows_cannot_show_twenty_five_while_rendering_five() {
    let harness = harness_at("/components/entity-table-viewport-fit").await;
    wait_for_selector(
        &harness,
        "#entity-viewport-fit-fixture [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;
    click(&harness, "[data-testid='viewport-fit-rows-17']").await;
    click(&harness, "[data-testid='viewport-fit-short']").await;
    tokio::time::sleep(Duration::from_millis(400)).await;

    // Auto: the control says so, and names the rows it fitted.
    let auto = page_size_agreement_snapshot(&harness).await;
    assert_eq!(auto["mode"], json!("auto"), "{auto}");
    assert_eq!(auto["summaryTotal"], json!(17), "{auto}");
    assert_eq!(
        auto["optionValues"],
        json!(["auto", "25", "50", "100"]),
        "viewport-fit tables offer Auto as a first-class choice: {auto}"
    );
    assert_one_page_size_everywhere(&auto, "auto fit over 17 rows");
    assert!(
        auto["controlLabel"]
            .as_str()
            .is_none_or(|label| label.trim() != "25"),
        "the control must never read a bare 25 while auto-fitting: {auto}"
    );

    // Explicit 25 over 17 rows: every row renders, on exactly one page.
    choose_page_size(&harness, "25").await;
    let fixed = page_size_agreement_snapshot(&harness).await;
    assert_eq!(fixed["mode"], json!("fixed"), "{fixed}");
    assert_eq!(fixed["effective"], json!(25), "{fixed}");
    assert_eq!(
        fixed["rows"],
        json!(17),
        "choosing 25 must render all 17 rows, not a fitted five: {fixed}"
    );
    assert_eq!(
        fixed["lastAdvertisedPage"],
        json!(1),
        "17 rows at 25 per page is one page, never four: {fixed}"
    );
    assert_eq!(fixed["nextDisabled"], json!(true), "{fixed}");
    assert_eq!(fixed["summary"], json!("Showing 1-17 of 17"), "{fixed}");
    assert_one_page_size_everywhere(&fixed, "explicit 25 over 17 rows");

    // A resize must not disturb an explicit choice.
    click(&harness, "[data-testid='viewport-fit-tall']").await;
    tokio::time::sleep(Duration::from_millis(350)).await;
    let after_resize = page_size_agreement_snapshot(&harness).await;
    assert_eq!(after_resize["mode"], json!("fixed"), "{after_resize}");
    assert_eq!(after_resize["effective"], json!(25), "{after_resize}");
    assert_one_page_size_everywhere(&after_resize, "explicit 25 after a resize");

    // Back to Auto, then across two desktop heights: the control's VALUE is
    // stable (so the selection never moves under the user) while the fitted
    // count and every surface reading it move together.
    choose_page_size(&harness, "auto").await;
    let tall = page_size_agreement_snapshot(&harness).await;
    assert_eq!(tall["mode"], json!("auto"), "{tall}");
    assert_one_page_size_everywhere(&tall, "auto fit at the tall height");

    click(&harness, "[data-testid='viewport-fit-default']").await;
    tokio::time::sleep(Duration::from_millis(350)).await;
    let default_height = page_size_agreement_snapshot(&harness).await;
    assert_eq!(default_height["mode"], json!("auto"), "{default_height}");
    assert_eq!(
        default_height["controlValue"], tall["controlValue"],
        "a resize must not move the control's selected value: {default_height}"
    );
    assert_one_page_size_everywhere(&default_height, "auto fit at the default height");

    assert_no_browser_errors(&harness, "EntityTable resolved page size").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn viewport_fit_paging_remeasures_without_persisting_or_nesting_scroll() {
    let harness = harness_at("/components/entity-table-viewport-fit").await;
    wait_for_selector(
        &harness,
        "#entity-viewport-fit-fixture [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;
    tokio::time::sleep(Duration::from_millis(250)).await;

    let initial = viewport_fit_snapshot(&harness).await;
    assert_eq!(initial["configured"], json!(25));
    assert!(
        initial["effective"]
            .as_u64()
            .is_some_and(|rows| (3..25).contains(&rows)),
        "initial fit: {initial}"
    );
    assert_eq!(initial["rows"], initial["effective"]);
    assert_eq!(initial["regionScrolls"], json!(false));
    assert_eq!(initial["verticalScrollers"], json!(0));
    assert_eq!(initial["clippedLastRow"], json!(false));

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const root = document.querySelector('#entity-viewport-fit-fixture [data-entity-table]');
                const original = root.dataset.entityEffectivePageSize;
                root.dataset.entityEffectivePageSize = '0';
                const caught = root.querySelectorAll('tbody tr').length !== Number(root.dataset.entityEffectivePageSize);
                root.dataset.entityEffectivePageSize = original;
                return caught && Number(root.dataset.entityEffectivePageSize) > 0;
            })()"#,
        )
        .await,
        json!(true),
        "the row-count/effective-capacity oracle must catch and revert a corrupted marker"
    );

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                for (let index = 0; index < 20; index += 1) {
                    const next = document.querySelector('[data-entity-page="next"]');
                    if (next.disabled) break;
                    next.click();
                }
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    let last_small_page = viewport_fit_snapshot(&harness).await;
    assert_eq!(
        last_small_page["currentPage"],
        last_small_page["totalPages"]
    );

    click(&harness, "[data-testid='viewport-fit-tall']").await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    let tall = viewport_fit_snapshot(&harness).await;
    assert!(
        tall["effective"].as_u64() > initial["effective"].as_u64(),
        "tall fit did not grow: initial={initial}, tall={tall}"
    );
    assert!(tall["rows"].as_u64().is_some_and(|rows| rows > 0));
    assert_eq!(tall["currentPage"], tall["totalPages"]);
    assert_eq!(tall["configured"], json!(25));
    assert_eq!(tall["regionScrolls"], json!(false));
    assert_eq!(tall["clippedLastRow"], json!(false));

    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport for fit recomputation");
    click(&harness, "[data-testid='viewport-fit-locale']").await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let localized = viewport_fit_snapshot(&harness).await;
    assert!(
        localized["rowsPerPageLabel"]
            .as_str()
            .is_some_and(|label| label.contains("viewport is too short"))
    );
    assert_eq!(localized["configured"], json!(25));
    assert_eq!(
        localized["pageOverflow"],
        json!(false),
        "localized fit overflowed the page: {localized}"
    );
    assert_eq!(
        localized["clippedLastRow"],
        json!(false),
        "localized fit clipped its last row: {localized}"
    );

    click(&harness, "[data-entity-column-chooser]").await;
    click(&harness, "[role='menu'] [data-entity-column='status']").await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    let hidden_column = viewport_fit_snapshot(&harness).await;
    assert_eq!(hidden_column["configured"], json!(25));
    assert_eq!(
        hidden_column["pageOverflow"],
        json!(false),
        "hidden-column fit overflowed the page: {hidden_column}"
    );
    assert_eq!(hidden_column["clippedLastRow"], json!(false));

    click(&harness, "[data-entity-page='1']").await;
    click(&harness, "[data-testid='viewport-fit-short']").await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let short = viewport_fit_snapshot(&harness).await;
    assert_eq!(short["effective"], json!(25));
    assert_eq!(short["configured"], json!(25));
    assert_eq!(short["rows"], json!(25));
    assert_eq!(short["regionScrolls"], json!(true));
    assert_eq!(short["verticalScrollers"], json!(1));
    assert_eq!(short["pageOverflow"], json!(false));

    tokio::time::sleep(Duration::from_millis(300)).await;
    let settled = viewport_fit_snapshot(&harness).await;
    assert_eq!(
        settled["effective"], short["effective"],
        "fit capacity oscillated: short={short}, settled={settled}"
    );
    assert_eq!(settled["currentPage"], short["currentPage"]);

    assert_no_browser_errors(&harness, "EntityTable viewport-fit paging").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn controlled_preferences_reorder_columns_and_compose_sort_clauses() {
    let harness = harness_at("/components/client-snapshot-list").await;
    begin_browser_error_capture(&harness).await;
    let storage_sentinel = eval_json(
        &harness,
        r#"(() => {
            const value = JSON.stringify({
                schema_version: 1,
                page_size: 100,
                sort: [{ column: 'client', direction: 'descending' }],
                column_order: ['actions', 'received', 'case_type', 'status', 'client'],
                hidden_columns: ['status'],
                column_widths: { client: 777 },
            });
            localStorage.setItem('ldui-entity-table:client-snapshot-demo', value);
            return value;
        })()"#,
    )
    .await;
    harness
        .navigate("/components/client-snapshot-list?pp-freeze=1")
        .await
        .expect("reload controlled EntityTable with a storage sentinel");
    wait_for_selector(&harness, "[data-entity-table-grid] tbody tr").await;
    let controlled_mount = eval_json(
        &harness,
        r#"(() => ({
            pageSize: document.querySelector('[data-entity-table] label select').value,
            headers: Array.from(document.querySelectorAll('[data-entity-table-grid] thead tr:first-child th')).map(th => th.dataset.entityColumn),
            stored: localStorage.getItem('ldui-entity-table:client-snapshot-demo'),
        }))()"#,
    )
    .await;
    assert_eq!(controlled_mount["pageSize"], json!("25"));
    assert_eq!(
        controlled_mount["headers"],
        json!(["client", "status", "case_type", "received", "actions"]),
        "controlled mount must not read the reversed hidden-column sentinel: {controlled_mount}"
    );
    assert_eq!(
        controlled_mount["stored"], storage_sentinel,
        "controlled mount must not overwrite the storage sentinel"
    );
    assert_eq!(
        eval_json(
            &harness,
            "localStorage.removeItem('ldui-entity-table:client-snapshot-demo'); true",
        )
        .await,
        json!(true)
    );

    click(&harness, "[data-entity-column-chooser]").await;
    let move_later = "[data-entity-column-order='status'][data-entity-column-move='later']";
    harness
        .page()
        .find_element(move_later)
        .await
        .expect("find Status move-later control")
        .focus()
        .await
        .expect("focus Status move-later control");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("keyboard-reorder Status column");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let reordered = eval_json(
        &harness,
        r#"(() => ({
            headers: Array.from(document.querySelectorAll('[data-entity-table-grid] thead tr:first-child th')).map(th => th.dataset.entityColumn),
            moveControls: document.querySelectorAll('[data-entity-column-move]').length,
            focusRetained: document.activeElement?.matches("[data-entity-column-order='status'][data-entity-column-move='later']") ?? false,
            firstEarlierDisabled: document.querySelector("[data-entity-column-order='client'][data-entity-column-move='earlier']").disabled,
            lastLaterDisabled: document.querySelector("[data-entity-column-order='actions'][data-entity-column-move='later']").disabled,
        }))()"#,
    )
    .await;
    assert_eq!(
        reordered,
        json!({
            "headers": ["client", "case_type", "status", "received", "actions"],
            "moveControls": 10,
            "focusRetained": true,
            "firstEarlierDisabled": true,
            "lastLaterDisabled": true,
        })
    );
    let state = oracle(&harness).await;
    assert_eq!(
        state["state"]["entity_table.preferences"]["column_order"],
        json!(["client", "case_type", "status", "received", "actions"]),
        "controlled preference oracle after reorder: {state}"
    );

    let received_later = "[data-entity-column-order='received'][data-entity-column-move='later']";
    harness
        .page()
        .find_element(received_later)
        .await
        .expect("find Received move-later control")
        .focus()
        .await
        .expect("focus Received move-later control");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("move Received to the final position");
    tokio::time::sleep(Duration::from_millis(100)).await;
    let boundary_focus = eval_json(
        &harness,
        r#"(() => ({
            order: Array.from(document.querySelectorAll('[data-entity-column-order]')).filter(el => el.tagName === 'LI').map(el => el.dataset.entityColumnOrder),
            focusMovedToEnabledOpposite: document.activeElement?.matches("[data-entity-column-order='received'][data-entity-column-move='earlier']") ?? false,
            boundaryLaterDisabled: document.querySelector("[data-entity-column-order='received'][data-entity-column-move='later']").disabled,
        }))()"#,
    )
    .await;
    assert_eq!(
        boundary_focus,
        json!({
            "order": ["client", "case_type", "status", "actions", "received"],
            "focusMovedToEnabledOpposite": true,
            "boundaryLaterDisabled": true,
        }),
        "boundary move must retain focus on an enabled control: {boundary_focus}"
    );
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("move Received back from the final position");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                order: Array.from(document.querySelectorAll('[data-entity-column-order]')).filter(el => el.tagName === 'LI').map(el => el.dataset.entityColumnOrder),
                focusRetained: document.activeElement?.matches("[data-entity-column-order='received'][data-entity-column-move='earlier']") ?? false,
                label: document.activeElement?.getAttribute('aria-label'),
            }))()"#,
        )
        .await,
        json!({
            "order": ["client", "case_type", "status", "received", "actions"],
            "focusRetained": true,
            "label": "Move Received earlier from position 4 of 5",
        })
    );

    let client_separator = "th[data-entity-column='client'] [role='separator']";
    harness
        .page()
        .find_element(client_separator)
        .await
        .expect("find Client column separator")
        .focus()
        .await
        .expect("focus Client column separator");
    tokio::time::sleep(Duration::from_millis(50)).await;
    let resize_before = eval_json(
        &harness,
        r#"(() => {
            const handle = document.querySelector("th[data-entity-column='client'] [role='separator']");
            return {
                active: document.activeElement === handle,
                tabindex: handle.getAttribute('tabindex'),
                min: handle.getAttribute('aria-valuemin'),
                max: handle.getAttribute('aria-valuemax'),
                now: Number(handle.getAttribute('aria-valuenow')),
                width: Math.round(handle.parentElement.getBoundingClientRect().width),
                opacity: getComputedStyle(handle).opacity,
            };
        })()"#,
    )
    .await;
    assert_eq!(resize_before["active"], json!(true));
    assert_eq!(resize_before["tabindex"], json!("0"));
    assert_eq!(resize_before["min"], json!("240"));
    assert_eq!(resize_before["max"], json!("1200"));
    assert_eq!(resize_before["now"], resize_before["width"]);
    assert_eq!(resize_before["opacity"], json!("1"));
    harness
        .press_key_sequence(&[Key::ArrowRight])
        .await
        .expect("keyboard-resize Client column");
    tokio::time::sleep(Duration::from_millis(100)).await;
    let resize_after = eval_json(
        &harness,
        r#"(() => {
            const handle = document.querySelector("th[data-entity-column='client'] [role='separator']");
            return {
                active: document.activeElement === handle,
                now: Number(handle.getAttribute('aria-valuenow')),
                width: Math.round(handle.parentElement.getBoundingClientRect().width),
                stored: localStorage.getItem('ldui-entity-table:client-snapshot-demo'),
            };
        })()"#,
    )
    .await;
    assert_eq!(resize_after["active"], json!(true));
    assert_eq!(resize_after["now"], resize_after["width"]);
    assert_eq!(resize_after["stored"], Value::Null);
    assert_eq!(
        resize_after["now"].as_u64(),
        resize_before["now"].as_u64().map(|width| width + 16),
        "ArrowRight must grow the controlled width by one keyboard step: before={resize_before}, after={resize_after}"
    );
    let state = oracle(&harness).await;
    assert_eq!(
        state["state"]["entity_table.preferences"]["column_widths"]["client"], resize_after["now"],
        "keyboard resize must emit the controlled replacement: {state}"
    );

    let geometry_baseline = mark_entity_table_geometry(&harness).await;
    assert_eq!(
        geometry_baseline["headers"], geometry_baseline["cells"],
        "EntityTable header/body tracks must be one-to-one: {geometry_baseline}"
    );

    harness
        .page()
        .find_element("[data-entity-sort-column='status']")
        .await
        .expect("find Status sort control")
        .focus()
        .await
        .expect("focus Status sort control");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("keyboard-sort Status ascending");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_entity_table_geometry_unchanged(
        &compare_entity_table_geometry(&harness).await,
        "EntityTable Enter sort",
    );
    mark_entity_table_geometry(&harness).await;
    shift_click(&harness, "[data-entity-sort-column='client']").await;
    assert_entity_table_geometry_unchanged(
        &compare_entity_table_geometry(&harness).await,
        "EntityTable shift+pointer sort",
    );
    mark_entity_table_geometry(&harness).await;
    shift_click(&harness, "[data-entity-sort-column='client']").await;
    assert_entity_table_geometry_unchanged(
        &compare_entity_table_geometry(&harness).await,
        "EntityTable shift+pointer direction change",
    );
    let sorted = eval_json(
        &harness,
        r#"(() => {
            const status = document.querySelector("th[data-entity-column='status']");
            const client = document.querySelector("th[data-entity-column='client']");
            return {
                statusAria: status.getAttribute('aria-sort'),
                clientAria: client.getAttribute('aria-sort'),
                statusPriority: status.dataset.entitySortPriority,
                clientPriority: client.dataset.entitySortPriority,
                statusLabel: status.querySelector('button').getAttribute('aria-label'),
                clientLabel: client.querySelector('button').getAttribute('aria-label'),
                first: document.querySelector('[data-entity-table-grid] tbody tr').dataset.rowKey,
                stored: localStorage.getItem('ldui-entity-table:client-snapshot-demo'),
            };
        })()"#,
    )
    .await;
    assert_eq!(sorted["statusAria"], json!("ascending"));
    assert_eq!(sorted["clientAria"], Value::Null);
    assert_eq!(sorted["statusPriority"], json!("1"));
    assert_eq!(sorted["clientPriority"], json!("2"));
    assert!(
        sorted["statusLabel"]
            .as_str()
            .unwrap()
            .contains("Currently sorted ascending at priority 1 of 2")
    );
    assert!(
        sorted["clientLabel"]
            .as_str()
            .unwrap()
            .contains("Currently sorted descending at priority 2 of 2")
    );
    assert!(
        sorted["statusLabel"]
            .as_str()
            .unwrap()
            .contains("Activate to sort descending as the only sort")
    );
    assert!(
        sorted["statusLabel"]
            .as_str()
            .unwrap()
            .contains("Shift+activate to change priority 1 to descending")
    );
    assert!(
        sorted["clientLabel"]
            .as_str()
            .unwrap()
            .contains("Activate to restore system order")
    );
    assert!(
        sorted["clientLabel"]
            .as_str()
            .unwrap()
            .contains("Shift+activate to remove priority 2")
    );
    assert_eq!(sorted["first"], json!("office-mx-071"));
    assert_eq!(
        sorted["stored"],
        Value::Null,
        "controlled preferences must not write browser storage"
    );

    let state = oracle(&harness).await;
    assert_eq!(
        state["state"]["entity_table.preferences"]["sort"],
        json!([
            { "column": "status", "direction": "ascending" },
            { "column": "client", "direction": "descending" },
        ]),
        "controlled preference oracle after multi-sort: {state}"
    );
    assert_entity_projection_matches_wide_dom(&harness, "controlled multi-sort").await;

    harness
        .page()
        .find_element("[data-entity-sort-column='case_type']")
        .await
        .expect("find Case type sort control")
        .focus()
        .await
        .expect("focus Case type sort control");
    shift_enter(&harness).await;
    let state = oracle(&harness).await;
    assert_eq!(
        state["state"]["entity_table.preferences"]["sort"],
        json!([
            { "column": "status", "direction": "ascending" },
            { "column": "client", "direction": "descending" },
            { "column": "case_type", "direction": "ascending" },
        ]),
        "real Shift+Enter must append a sort clause: {state}"
    );
    shift_enter(&harness).await;
    shift_enter(&harness).await;
    let state = oracle(&harness).await;
    assert_eq!(
        state["state"]["entity_table.preferences"]["sort"],
        json!([
            { "column": "status", "direction": "ascending" },
            { "column": "client", "direction": "descending" },
        ]),
        "Shift+Enter must cycle and remove only the focused clause: {state}"
    );

    click(&harness, "[data-entity-column-chooser]").await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-entity-column-order]')?.offsetParent !== null",
        )
        .await,
        json!(true),
        "column chooser must be open for the accessibility audit"
    );
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(harness.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("controlled-entity-table-multi-sort-chooser")
        .unwrap_or_else(|error| {
            panic!(
                "{error}; {}\nviolations: {:#?}",
                report.summary(),
                report.violations
            )
        });

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const office = document.querySelector('[data-dataset-selector] select');
                office.value = 'office-in';
                office.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-entity-table-grid] tbody tr').dataset.rowKey",
        )
        .await,
        json!("office-in-071"),
        "dataset replacement must preserve the controlled multi-sort"
    );
    let state = oracle(&harness).await;
    assert_eq!(
        state["state"]["entity_table.preferences"]["column_order"],
        json!(["client", "case_type", "status", "received", "actions"]),
        "dataset replacement must preserve controlled column order: {state}"
    );
    assert_no_browser_errors(&harness, "controlled EntityTable preferences").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn entity_table_toolbar_action_stays_adjacent_and_operable() {
    let harness = harness_at("/components/client-snapshot-list").await;
    begin_browser_error_capture(&harness).await;
    assert_entity_projection_matches_wide_dom(&harness, "initial toolbar mount").await;

    let inspect = |width: &'static str| {
        let harness = &harness;
        async move {
            eval_json(
                harness,
                &format!(
                r#"(() => {{
                    const root = document.querySelector('[data-entity-table]');
                    const action = root.querySelector('[data-testid="entity-toolbar-export"]');
                    const chooser = root.querySelector('[data-entity-column-chooser]');
                    const menu = root.querySelector('[role="menu"]');
                    const focusables = Array.from(root.querySelectorAll('select, button, [role="button"][tabindex="0"]'));
                    const actionBox = action.getBoundingClientRect();
                    const chooserBox = chooser.getBoundingClientRect();
                    return {{
                        width: {width},
                        actions: root.querySelectorAll('[data-testid="entity-toolbar-export"]').length,
                        label: action.getAttribute('aria-label'),
                        chooserTag: chooser.tagName.toLowerCase(),
                        chooserPresentation: chooser.dataset.entityColumnChooserPresentation,
                        chooserLabel: chooser.getAttribute('aria-label'),
                        chooserText: chooser.textContent.trim(),
                        chooserExpanded: chooser.getAttribute('aria-expanded'),
                        chooserForcedColors: chooser.className.includes('forced-colors:border'),
                        visible: actionBox.width > 0 && actionBox.height > 0,
                        withinViewport: actionBox.left >= 0 && actionBox.right <= document.documentElement.clientWidth,
                        chooserVisible: chooserBox.width > 0 && chooserBox.right <= document.documentElement.clientWidth,
                        menuVisible: menu.getBoundingClientRect().width > 0,
                        actionOrder: focusables.indexOf(action),
                        chooserOrder: focusables.indexOf(chooser),
                    }};
                }})()"#
                ),
            )
            .await
        }
    };

    harness
        .page()
        .find_element("[data-entity-column-chooser]")
        .await
        .expect("find column chooser")
        .focus()
        .await
        .expect("focus column chooser");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("keyboard-open column chooser");
    let wide = inspect("1280").await;
    assert_eq!(wide["actions"], json!(1));
    assert_eq!(wide["label"], json!("Export current rows"));
    assert_eq!(wide["chooserTag"], json!("button"));
    assert_eq!(wide["chooserPresentation"], json!("icon"));
    assert_eq!(wide["chooserLabel"], json!("Choose columns"));
    assert_eq!(wide["chooserText"], json!("⚙"));
    assert_eq!(wide["chooserExpanded"], json!("true"));
    assert_eq!(wide["chooserForcedColors"], json!(true));
    assert_eq!(wide["visible"], json!(true));
    assert_eq!(wide["withinViewport"], json!(true));
    assert_eq!(wide["chooserVisible"], json!(true));
    assert_eq!(wide["menuVisible"], json!(true));
    assert!(wide["actionOrder"].as_i64() < wide["chooserOrder"].as_i64());
    harness
        .press_key_sequence(&[Key::Escape])
        .await
        .expect("dismiss chooser with Escape");
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                expanded: document.querySelector('[data-entity-column-chooser]').getAttribute('aria-expanded'),
                focused: document.activeElement === document.querySelector('[data-entity-column-chooser]'),
                menuVisible: document.querySelector('[data-entity-column-chooser]').parentElement.querySelector('[role="menu"]').getBoundingClientRect().width > 0,
            }))()"#,
        )
        .await,
        json!({ "expanded": "false", "focused": true, "menuVisible": false })
    );
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("reopen chooser before action");
    click(&harness, "[data-testid='entity-toolbar-export']").await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-testid=\"entity-toolbar-action-count\"]').textContent",
        )
        .await,
        json!("1")
    );
    let first_export = eval_json(
        &harness,
        r#"(() => ({
            counts: document.querySelector('[data-testid="entity-export-counts"]').textContent,
            firstKey: document.querySelector('[data-testid="entity-export-first-key"]').textContent,
        }))()"#,
    )
    .await;
    assert_eq!(first_export["counts"], json!("25/72"));
    assert_eq!(first_export["firstKey"], json!("office-mx-000"));
    let wide_projection =
        oracle(&harness).await["state"]["entity_table.display_projection"].clone();

    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact toolbar viewport");
    harness
        .page()
        .find_element("[data-entity-column-chooser]")
        .await
        .expect("find compact column chooser")
        .focus()
        .await
        .expect("focus compact column chooser");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("keyboard-open compact column chooser");
    let compact = inspect("390").await;
    assert_eq!(compact["actions"], json!(1));
    assert_eq!(compact["visible"], json!(true));
    assert_eq!(compact["withinViewport"], json!(true));
    assert_eq!(compact["chooserVisible"], json!(true));
    assert_eq!(compact["menuVisible"], json!(true));
    assert_eq!(compact["chooserExpanded"], json!("true"));
    assert!(compact["actionOrder"].as_i64() < compact["chooserOrder"].as_i64());
    click(&harness, "[data-testid='entity-toolbar-export']").await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-testid=\"entity-toolbar-action-count\"]').textContent",
        )
        .await,
        json!("2")
    );
    assert_eq!(
        oracle(&harness).await["state"]["entity_table.display_projection"],
        wide_projection,
        "compact presentation changed the export projection"
    );

    harness
        .set_viewport(ViewportSize::new(1280, 800))
        .await
        .expect("restore wide viewport for EntityColumn presentation");
    harness
        .navigate("/components/entity-table-presentation?pp-freeze=1")
        .await
        .expect("navigate to EntityColumn presentation fixture");
    wait_for_selector(
        &harness,
        "#entity-table-presentation-fixture [data-entity-row-key='presentation-1']",
    )
    .await;

    let inspect_presentation = || {
        eval_json(
            &harness,
            r#"(() => {
                const root = document.querySelector('#entity-table-presentation-fixture');
                const row = root.querySelector('[data-entity-row-key="presentation-1"]');
                const reference = row.querySelector('td[data-entity-column="reference"] [data-entity-text-overflow]');
                const narrative = row.querySelector('td[data-entity-column="narrative"] [data-entity-text-overflow]');
                const richCell = row.querySelector('td[data-entity-column="rich"]');
                const numberCell = row.querySelector('td[data-entity-column="number"]');
                const optionalCell = row.querySelector('td[data-entity-column="optional"]');
                const currencyCell = row.querySelector('td[data-entity-column="currency"]');
                const percentageCell = row.querySelector('td[data-entity-column="percentage"]');
                const badgeCell = row.querySelector('td[data-entity-column="status_badge"]');
                const iconCell = row.querySelector('td[data-entity-column="state_icon"]');
                const unknownRow = root.querySelector('[data-entity-row-key="presentation-3"]');
                const emptyRow = root.querySelector('[data-entity-row-key="presentation-4"]');
                const numberHeader = root.querySelector('th[data-entity-column="number"]');
                const referenceStyle = getComputedStyle(reference);
                const narrativeStyle = getComputedStyle(narrative);
                const lineHeight = Number.parseFloat(narrativeStyle.lineHeight);
                return {
                    reference: {
                        policy: reference.dataset.entityTextOverflow,
                        title: reference.title,
                        text: reference.textContent,
                        overflow: referenceStyle.textOverflow,
                        whiteSpace: referenceStyle.whiteSpace,
                        clips: reference.scrollWidth > reference.clientWidth,
                    },
                    narrative: {
                        policy: narrative.dataset.entityTextOverflow,
                        lines: narrative.dataset.entityLineClamp,
                        title: narrative.title,
                        text: narrative.textContent,
                        clamp: narrativeStyle.webkitLineClamp,
                        overflow: narrativeStyle.overflow,
                        boundedToTwoLines: Number.isFinite(lineHeight)
                            && narrative.getBoundingClientRect().height <= lineHeight * 2 + 2,
                    },
                    rich: {
                        custom: !!richCell.querySelector('[data-entity-presentation-rich]'),
                        overflowMarkers: richCell.querySelectorAll('[data-entity-text-overflow]').length,
                        alignment: richCell.dataset.entityAlignment,
                        tabular: richCell.dataset.entityTabularNumbers,
                        textAlign: getComputedStyle(richCell).textAlign,
                        numericVariant: getComputedStyle(richCell).fontVariantNumeric,
                    },
                    numeric: {
                        headerAlignment: numberHeader.dataset.entityAlignment,
                        headerTextAlign: getComputedStyle(numberHeader).textAlign,
                        headerForcedColors: numberHeader.className.includes('forced-colors:border'),
                        cellAlignment: numberCell.dataset.entityAlignment,
                        cellTextAlign: getComputedStyle(numberCell).textAlign,
                        cellTabular: numberCell.dataset.entityTabularNumbers,
                        numericVariant: getComputedStyle(numberCell).fontVariantNumeric,
                        signedText: numberCell.textContent.trim(),
                        optionalText: optionalCell.textContent.trim(),
                        currencyText: currencyCell.textContent.trim(),
                        percentageText: percentageCell.textContent.trim(),
                        currencyTitle: currencyCell.querySelector('[data-entity-text-overflow]').title,
                    },
                    semantic: {
                        badgeText: badgeCell.querySelector('[data-entity-semantic-cell="badge"]').textContent.trim(),
                        badgeClass: badgeCell.querySelector('.badge').className,
                        badgeForcedColors: badgeCell.querySelector('.badge').className.includes('forced-colors:border'),
                        iconName: iconCell.querySelector('[data-entity-semantic-cell="icon"]').dataset.entityIconName,
                        iconAccessible: iconCell.querySelector('.sr-only').textContent,
                        iconSvgHidden: iconCell.querySelector('svg').getAttribute('aria-hidden'),
                        iconForcedColors: iconCell.querySelector('[data-entity-semantic-cell="icon"]').className.includes('forced-colors:text'),
                        unknownBadge: unknownRow.querySelector('td[data-entity-column="status_badge"] [data-entity-semantic-fallback]').textContent.trim(),
                        unknownIcon: unknownRow.querySelector('td[data-entity-column="state_icon"] [data-entity-semantic-fallback]').textContent.trim(),
                        emptyBadge: emptyRow.querySelector('td[data-entity-column="status_badge"] [data-entity-semantic-fallback]').dataset.entitySemanticFallback,
                        emptyIcon: emptyRow.querySelector('td[data-entity-column="state_icon"] [data-entity-semantic-fallback]').dataset.entitySemanticFallback,
                    },
                    pageOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth,
                };
            })()"#,
        )
    };

    let wide_presentation = inspect_presentation().await;
    assert_eq!(wide_presentation["reference"]["policy"], json!("ellipsis"));
    assert_eq!(
        wide_presentation["reference"]["overflow"],
        json!("ellipsis")
    );
    assert_eq!(
        wide_presentation["reference"]["whiteSpace"],
        json!("nowrap")
    );
    assert_eq!(wide_presentation["reference"]["clips"], json!(true));
    assert_eq!(
        wide_presentation["reference"]["title"],
        wide_presentation["reference"]["text"]
    );
    assert_eq!(
        wide_presentation["narrative"]["policy"],
        json!("line-clamp")
    );
    assert_eq!(wide_presentation["narrative"]["lines"], json!("2"));
    assert_eq!(wide_presentation["narrative"]["clamp"], json!("2"));
    assert_eq!(wide_presentation["narrative"]["overflow"], json!("hidden"));
    assert_eq!(
        wide_presentation["narrative"]["boundedToTwoLines"],
        json!(true)
    );
    assert_eq!(
        wide_presentation["narrative"]["title"],
        wide_presentation["narrative"]["text"]
    );
    assert_eq!(wide_presentation["rich"]["custom"], json!(true));
    assert_eq!(wide_presentation["rich"]["overflowMarkers"], json!(0));
    assert_eq!(wide_presentation["rich"]["alignment"], json!("end"));
    assert_eq!(wide_presentation["rich"]["tabular"], json!("true"));
    assert_eq!(wide_presentation["rich"]["textAlign"], json!("right"));
    assert!(
        wide_presentation["rich"]["numericVariant"]
            .as_str()
            .is_some_and(|value| value.contains("tabular-nums"))
    );
    assert_eq!(
        wide_presentation["numeric"]["headerAlignment"],
        json!("end")
    );
    assert_eq!(
        wide_presentation["numeric"]["headerTextAlign"],
        json!("right")
    );
    assert_eq!(
        wide_presentation["numeric"]["headerForcedColors"],
        json!(true)
    );
    assert_eq!(wide_presentation["numeric"]["cellAlignment"], json!("end"));
    assert_eq!(
        wide_presentation["numeric"]["cellTextAlign"],
        json!("right")
    );
    assert_eq!(wide_presentation["numeric"]["cellTabular"], json!("true"));
    assert!(
        wide_presentation["numeric"]["numericVariant"]
            .as_str()
            .is_some_and(|value| value.contains("tabular-nums"))
    );
    assert_eq!(wide_presentation["numeric"]["signedText"], json!("10"));
    assert_eq!(
        wide_presentation["numeric"]["optionalText"],
        json!("Not ranked")
    );
    assert_eq!(
        wide_presentation["numeric"]["currencyText"],
        json!("-$12,345,678,901.25")
    );
    assert_eq!(
        wide_presentation["numeric"]["currencyTitle"],
        wide_presentation["numeric"]["currencyText"]
    );
    assert_eq!(
        wide_presentation["numeric"]["percentageText"],
        json!("100.00%")
    );
    assert_eq!(
        wide_presentation["semantic"]["badgeText"],
        json!("Needs review")
    );
    assert!(
        wide_presentation["semantic"]["badgeClass"]
            .as_str()
            .is_some_and(
                |classes| classes.contains("badge-soft") && classes.contains("badge-warning")
            )
    );
    assert_eq!(
        wide_presentation["semantic"]["badgeForcedColors"],
        json!(true)
    );
    assert_eq!(
        wide_presentation["semantic"]["iconName"],
        json!("circle-check")
    );
    assert_eq!(
        wide_presentation["semantic"]["iconAccessible"],
        json!("Enabled")
    );
    assert_eq!(
        wide_presentation["semantic"]["iconSvgHidden"],
        json!("true")
    );
    assert_eq!(
        wide_presentation["semantic"]["iconForcedColors"],
        json!(true)
    );
    assert_eq!(
        wide_presentation["semantic"]["unknownBadge"],
        json!("Unknown status")
    );
    assert_eq!(
        wide_presentation["semantic"]["unknownIcon"],
        json!("Unknown state")
    );
    assert_eq!(wide_presentation["semantic"]["emptyBadge"], json!("empty"));
    assert_eq!(wide_presentation["semantic"]["emptyIcon"], json!("empty"));
    assert_eq!(wide_presentation["pageOverflow"], json!(false));

    click(&harness, "[data-testid='entity-presentation-locale']").await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const row = document.querySelector('[data-entity-row-key="presentation-1"]');
                return {
                    badge: row.querySelector('td[data-entity-column="status_badge"] [data-entity-semantic-cell]').textContent.trim(),
                    icon: row.querySelector('td[data-entity-column="state_icon"] .sr-only').textContent,
                };
            })()"#,
        )
        .await,
        json!({ "badge": "Revisión necesaria", "icon": "Habilitado" }),
        "reactive canonical localization must update visible badge and icon accessibility copy"
    );

    let narrative_separator = "th[data-entity-column='narrative'] [role='separator']";
    let before_resize = eval_json(
        &harness,
        &format!(
            "Number(document.querySelector(\"{narrative_separator}\").getAttribute('aria-valuenow'))"
        ),
    )
    .await;
    harness
        .page()
        .find_element(narrative_separator)
        .await
        .expect("find narrative resize separator")
        .focus()
        .await
        .expect("focus narrative resize separator");
    harness
        .press_key_sequence(&[Key::ArrowLeft, Key::ArrowLeft])
        .await
        .expect("resize narrative column from the keyboard");
    let after_resize = eval_json(
        &harness,
        &format!(
            "Number(document.querySelector(\"{narrative_separator}\").getAttribute('aria-valuenow'))"
        ),
    )
    .await;
    assert!(
        after_resize.as_f64() < before_resize.as_f64(),
        "keyboard resize must reduce the narrative column width: before={before_resize}, after={after_resize}"
    );
    let resized_presentation = inspect_presentation().await;
    assert_eq!(resized_presentation["narrative"]["lines"], json!("2"));
    assert_eq!(
        resized_presentation["narrative"]["text"],
        wide_presentation["narrative"]["text"]
    );

    click(
        &harness,
        "th[data-entity-column='number'] [data-entity-sort-column='number']",
    )
    .await;
    let ascending_typed = eval_json(
        &harness,
        r#"(() => {
            const header = document.querySelector('th[data-entity-column="number"]');
            return {
                rows: Array.from(document.querySelectorAll('#entity-table-presentation-fixture tbody [data-entity-row-key]')).map(row => row.dataset.entityRowKey),
                aria: header.getAttribute('aria-sort'),
                priority: header.dataset.entitySortPriority,
                direction: header.dataset.entitySortDirection,
                marker: header.querySelector('[data-entity-sort-indicator]').textContent.trim(),
            };
        })()"#,
    )
    .await;
    assert_eq!(
        ascending_typed,
        json!({
            "rows": ["presentation-3", "presentation-2", "presentation-4", "presentation-1"],
            "aria": "ascending",
            "priority": "1",
            "direction": "ascending",
            "marker": "▲1",
        }),
        "typed numeric sort must place -3, 2, 2, 10 rather than sorting display strings"
    );
    click(
        &harness,
        "th[data-entity-column='number'] [data-entity-sort-column='number']",
    )
    .await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const header = document.querySelector('th[data-entity-column="number"]');
                return {
                    rows: Array.from(document.querySelectorAll('#entity-table-presentation-fixture tbody [data-entity-row-key]')).map(row => row.dataset.entityRowKey),
                    aria: header.getAttribute('aria-sort'),
                    marker: header.querySelector('[data-entity-sort-indicator]').textContent.trim(),
                };
            })()"#,
        )
        .await,
        json!({
            "rows": ["presentation-1", "presentation-2", "presentation-4", "presentation-3"],
            "aria": "descending",
            "marker": "▼1",
        }),
        "descending typed order must retain source order for the equal 2 keys"
    );

    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact EntityColumn presentation viewport");
    tokio::time::sleep(Duration::from_millis(200)).await;
    let compact_presentation = eval_json(
        &harness,
        r#"(() => {
            const row = document.querySelector('#entity-table-presentation-fixture [data-entity-row-key="presentation-1"]');
            const reference = row.querySelector('td.lg\\:hidden [data-entity-column="reference"] [data-entity-text-overflow]');
            const narrative = row.querySelector('td.lg\\:hidden [data-entity-column="narrative"] [data-entity-text-overflow]');
            const rich = row.querySelector('td.lg\\:hidden [data-entity-column="rich"]');
            const number = row.querySelector('td.lg\\:hidden [data-entity-column="number"]');
            const optional = row.querySelector('td.lg\\:hidden [data-entity-column="optional"]');
            const currency = row.querySelector('td.lg\\:hidden [data-entity-column="currency"]');
            const badge = row.querySelector('td.lg\\:hidden [data-entity-column="status_badge"]');
            const icon = row.querySelector('td.lg\\:hidden [data-entity-column="state_icon"]');
            return {
                referencePolicy: reference.dataset.entityTextOverflow,
                referenceTitle: reference.title,
                referenceText: reference.textContent,
                narrativeLines: narrative.dataset.entityLineClamp,
                narrativeTitle: narrative.title,
                narrativeText: narrative.textContent,
                richCustom: !!rich.querySelector('[data-entity-presentation-rich]'),
                richOverflowMarkers: rich.querySelectorAll('[data-entity-text-overflow]').length,
                numberAlignment: number.dataset.entityAlignment,
                numberTextAlign: getComputedStyle(number.lastElementChild).textAlign,
                numberVariant: getComputedStyle(number.lastElementChild).fontVariantNumeric,
                optionalText: optional.lastElementChild.textContent.trim(),
                currencyText: currency.lastElementChild.textContent.trim(),
                badgeText: badge.querySelector('[data-entity-semantic-cell="badge"]').textContent.trim(),
                iconAccessible: icon.querySelector('.sr-only').textContent,
                pageOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth,
            };
        })()"#,
    )
    .await;
    assert_eq!(compact_presentation["referencePolicy"], json!("ellipsis"));
    assert_eq!(
        compact_presentation["referenceTitle"],
        compact_presentation["referenceText"]
    );
    assert_eq!(compact_presentation["narrativeLines"], json!("2"));
    assert_eq!(
        compact_presentation["narrativeTitle"],
        compact_presentation["narrativeText"]
    );
    assert_eq!(compact_presentation["richCustom"], json!(true));
    assert_eq!(compact_presentation["richOverflowMarkers"], json!(0));
    assert_eq!(compact_presentation["numberAlignment"], json!("end"));
    assert_eq!(compact_presentation["numberTextAlign"], json!("right"));
    assert!(
        compact_presentation["numberVariant"]
            .as_str()
            .is_some_and(|value| value.contains("tabular-nums"))
    );
    assert_eq!(compact_presentation["optionalText"], json!("Not ranked"));
    assert_eq!(
        compact_presentation["currencyText"],
        json!("-$12,345,678,901.25")
    );
    assert_eq!(
        compact_presentation["badgeText"],
        json!("Revisión necesaria")
    );
    assert_eq!(compact_presentation["iconAccessible"], json!("Habilitado"));
    assert_eq!(compact_presentation["pageOverflow"], json!(false));

    assert_no_browser_errors(
        &harness,
        "EntityTable toolbar actions and EntityColumn overflow presentation",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn client_snapshot_list_contract_works_end_to_end() {
    let harness = harness_at("/components/client-snapshot-list").await;
    begin_browser_error_capture(&harness).await;

    let initial = eval_json(
        &harness,
        r#"(() => {
            const root = document.querySelector('[data-list-page]');
            const table = root.querySelector('[data-entity-table-grid]');
            const firstHeader = table.querySelector('thead th:first-child');
            const chooser = root.querySelector('[data-entity-table] [role="menu"]');
            const datasetSelect = root.querySelector('[data-dataset-selector] select');
            const pageSizeSelect = root.querySelector('[data-entity-table] label select');
            return {
                contract: root.dataset.pageContract,
                dataMode: root.querySelector('[data-entity-table]').dataset.tableDataMode,
                tableTag: table?.tagName.toLowerCase(),
                rows: table.querySelectorAll('tbody tr').length,
                initialSort: firstHeader.getAttribute('aria-sort'),
                initialSortLabel: firstHeader.querySelector('button').getAttribute('aria-label'),
                chooserText: chooser.textContent.replace(/\s+/g, ' ').trim(),
                datasetInsideFilters: !!root.querySelector('[data-filter-bar] [data-dataset-selector]'),
                selectorResettable: root.querySelector('[data-dataset-selector]').dataset.resettableFilter,
                datasetSelectId: datasetSelect.id,
                datasetSelectLabel: datasetSelect.getAttribute('aria-label'),
                pageSizeSelectId: pageSizeSelect.id,
                pageSizeSelectLabel: pageSizeSelect.getAttribute('aria-label'),
                controlIdsDistinct: datasetSelect.id !== pageSizeSelect.id,
            };
        })()"#,
    )
    .await;
    assert_eq!(initial["contract"], json!("client-snapshot-demo"));
    assert_eq!(initial["dataMode"], json!("client-snapshot"));
    assert_eq!(initial["tableTag"], json!("table"));
    assert_eq!(
        initial["rows"],
        json!(25),
        "only one page belongs in the DOM"
    );
    assert_eq!(
        initial["initialSort"],
        Value::Null,
        "inactive headers must omit aria-sort"
    );
    assert!(
        initial["initialSortLabel"]
            .as_str()
            .is_some_and(|label| label.contains("Activate to sort ascending as the only sort")),
        "system-order label: {initial}"
    );
    let chooser = initial["chooserText"].as_str().unwrap_or_default();
    assert!(chooser.contains("Status") && chooser.contains("Case type"));
    assert!(!chooser.contains("Client") && !chooser.contains("Actions"));
    assert_eq!(initial["datasetInsideFilters"], json!(false));
    assert_eq!(initial["selectorResettable"], json!("false"));
    assert_eq!(
        initial["datasetSelectId"],
        json!("client-snapshot-dataset-selector")
    );
    assert_eq!(initial["datasetSelectLabel"], json!("Office"));
    assert_eq!(
        initial["pageSizeSelectId"],
        json!("client-snapshot-page-size")
    );
    assert_eq!(initial["pageSizeSelectLabel"], json!("Rows per page"));
    assert_eq!(initial["controlIdsDistinct"], json!(true));
    assert_entity_projection_matches_wide_dom(&harness, "initial client snapshot").await;

    let sort_button = "[data-entity-table-grid] thead th:first-child button";
    harness
        .page()
        .find_element(sort_button)
        .await
        .expect("find sortable header button")
        .focus()
        .await
        .expect("focus sortable header button");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("keyboard-sort header");
    tokio::time::sleep(Duration::from_millis(100)).await;
    let ascending = eval_json(
        &harness,
        r#"(() => {
            const th = document.querySelector('[data-entity-table-grid] thead th:first-child');
            return { sort: th.getAttribute('aria-sort'), label: th.querySelector('button').getAttribute('aria-label') };
        })()"#,
    )
    .await;
    assert_eq!(ascending["sort"], json!("ascending"));
    assert!(
        ascending["label"]
            .as_str()
            .unwrap()
            .contains("Activate to sort descending as the only sort")
    );

    click(&harness, sort_button).await;
    let descending = eval_json(
        &harness,
        r#"(() => {
            const table = document.querySelector('[data-entity-table-grid]');
            const th = table.querySelector('thead th:first-child');
            return {
                sort: th.getAttribute('aria-sort'),
                label: th.querySelector('button').getAttribute('aria-label'),
                first: table.querySelector('tbody tr').dataset.rowKey,
            };
        })()"#,
    )
    .await;
    assert_eq!(descending["sort"], json!("descending"));
    assert!(
        descending["label"]
            .as_str()
            .unwrap()
            .contains("Activate to restore system order")
    );
    assert_eq!(descending["first"], json!("office-mx-071"));
    assert_entity_projection_matches_wide_dom(&harness, "descending sort").await;

    let changed = eval_json(
        &harness,
        r#"(() => {
            const search = document.querySelector('[data-filter-search] input');
            search.value = 'Client 010';
            search.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText' }));
            const office = document.querySelector('[data-dataset-selector] select');
            office.value = 'office-in';
            office.dispatchEvent(new Event('change', { bubbles: true }));
            return true;
        })()"#,
    )
    .await;
    assert_eq!(changed, json!(true));
    tokio::time::sleep(Duration::from_millis(250)).await;
    let after_dataset_change = eval_json(
        &harness,
        r#"(() => ({
            search: document.querySelector('[data-filter-search] input').value,
            office: document.querySelector('[data-dataset-selector] select').value,
            sort: document.querySelector('[data-entity-table-grid] thead th:first-child').getAttribute('aria-sort'),
            row: document.querySelector('[data-entity-table-grid] tbody tr')?.dataset.rowKey,
            text: document.querySelector('[data-entity-table-grid] tbody tr')?.textContent,
        }))()"#,
    )
    .await;
    assert_eq!(after_dataset_change["search"], json!("Client 010"));
    assert_eq!(after_dataset_change["office"], json!("office-in"));
    assert_eq!(after_dataset_change["sort"], json!("descending"));
    assert_eq!(after_dataset_change["row"], json!("office-in-009"));
    assert!(
        after_dataset_change["text"]
            .as_str()
            .unwrap()
            .contains("Delhi Client 010")
    );
    assert_entity_projection_matches_wide_dom(&harness, "filtered dataset replacement").await;

    let active_filter_report = ldui_audit::audit_page(
        &harness,
        &ldui_audit::from_ui_tokens(body_font_family(&harness).await),
        &Default::default(),
    )
    .await
    .expect("audit client snapshot with an active filter chip");
    assert_eq!(
        active_filter_report.count(family::COMPONENT_DRIFT),
        0,
        "active filter controls must use design-system components:\n{}",
        active_filter_report.describe("client snapshot active filter")
    );

    click(&harness, "[data-filter-actions] button").await;
    let reset = eval_json(
        &harness,
        r#"(() => ({
            search: document.querySelector('[data-filter-search] input').value,
            office: document.querySelector('[data-dataset-selector] select').value,
            sort: document.querySelector('[data-entity-table-grid] thead th:first-child').getAttribute('aria-sort'),
            rows: document.querySelectorAll('[data-entity-table-grid] tbody tr').length,
        }))()"#,
    )
    .await;
    assert_eq!(
        reset,
        json!({ "search": "", "office": "office-in", "sort": "descending", "rows": 25 })
    );

    click(&harness, sort_button).await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                sort: document.querySelector('[data-entity-table-grid] thead th:first-child').getAttribute('aria-sort'),
                first: document.querySelector('[data-entity-table-grid] tbody tr').dataset.rowKey,
            }))()"#,
        )
        .await,
        json!({ "sort": null, "first": "office-in-000" })
    );

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const select = document.querySelector('[data-entity-table] label select');
                select.value = '50';
                select.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                rows: document.querySelectorAll('[data-entity-table-grid] tbody tr').length,
                range: document.querySelector('[data-entity-table] > div:last-child > span').textContent.trim(),
            }))()"#,
        )
        .await,
        json!({ "rows": 50, "range": "Showing 1-50 of 72" })
    );
    assert_entity_projection_matches_wide_dom(&harness, "page-size replacement").await;
    click(&harness, "[data-entity-page='next']").await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                rows: document.querySelectorAll('[data-entity-table-grid] tbody tr').length,
                first: document.querySelector('[data-entity-table-grid] tbody tr').dataset.rowKey,
            }))()"#,
        )
        .await,
        json!({ "rows": 22, "first": "office-in-050" })
    );
    assert_entity_projection_matches_wide_dom(&harness, "next page").await;
    click(&harness, "[data-entity-page='previous']").await;

    click(&harness, "[data-entity-column-chooser]").await;
    click(&harness, "[role='menu'] [data-entity-column='status']").await;
    let hidden_status = eval_json(
        &harness,
        r#"(() => ({
            statusHeader: Array.from(document.querySelectorAll('[data-entity-table-grid] thead th')).some(th => th.textContent.includes('Status')),
            stored: localStorage.getItem('ldui-entity-table:client-snapshot-demo'),
        }))()"#,
    )
    .await;
    assert_eq!(hidden_status["statusHeader"], json!(false));
    assert_entity_projection_matches_wide_dom(&harness, "hidden status column").await;
    assert_eq!(
        hidden_status["stored"],
        Value::Null,
        "controlled column changes must not use localStorage: {hidden_status}"
    );
    let state = oracle(&harness).await;
    assert!(
        state["state"]["entity_table.preferences"]["hidden_columns"]
            .as_array()
            .is_some_and(|columns| columns.contains(&json!("status"))),
        "controlled hidden-column oracle: {state}"
    );

    harness
        .page()
        .find_element(
            "[data-entity-table-grid] tbody tr:first-child td[data-entity-action='true'] button",
        )
        .await
        .expect("find row action")
        .focus()
        .await
        .expect("focus row action");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("keyboard-activate row action");
    tokio::time::sleep(Duration::from_millis(100)).await;
    let action_counts = eval_json(
        &harness,
        r#"(() => ({
            claim: document.querySelector('[data-testid="entity-claim-count"]').textContent,
            activate: document.querySelector('[data-testid="entity-activate-count"]').textContent,
        }))()"#,
    )
    .await;
    assert_eq!(action_counts, json!({ "claim": "1", "activate": "0" }));

    harness
        .page()
        .find_element("[data-entity-table-grid] tbody tr:first-child")
        .await
        .expect("find interactive row")
        .focus()
        .await
        .expect("focus interactive row");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("keyboard-activate row");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-testid=\"entity-activate-count\"]').textContent",
        )
        .await,
        json!("1")
    );

    assert_eq!(
        eval_json(
            &harness,
            "window.__retainedEntityRow = document.querySelector('[data-entity-table-grid] tbody tr'); true",
        )
        .await,
        json!(true)
    );
    click(&harness, "[data-testid='toggle-revalidating']").await;
    let retained = eval_json(
        &harness,
        r#"(() => ({
            sameNode: window.__retainedEntityRow === document.querySelector('[data-entity-table-grid] tbody tr'),
            busy: document.querySelector('[data-async-data-section]').getAttribute('aria-busy'),
            alert: document.querySelector('[data-retained-state]')?.textContent.trim(),
            hidden: document.querySelector('[data-retained-content]').classList.contains('hidden'),
        }))()"#,
    )
    .await;
    assert_eq!(retained["sameNode"], json!(true));
    assert_eq!(retained["busy"], json!("true"));
    assert_eq!(retained["hidden"], json!(false));
    assert!(retained["alert"].as_str().unwrap().contains("Refreshing"));
    click(&harness, "[data-testid='toggle-revalidating']").await;

    assert_eq!(
        eval_json(
        &harness,
        r#"(() => {
            const th = document.querySelector('[data-entity-table-grid] thead th:first-child');
            const handle = th.querySelector('[role="separator"]');
            const x = handle.getBoundingClientRect().left;
            const init = { bubbles: true, pointerId: 7, pointerType: 'mouse', isPrimary: true };
            handle.dispatchEvent(new PointerEvent('pointerdown', { ...init, clientX: x, buttons: 1 }));
            handle.dispatchEvent(new PointerEvent('pointermove', { ...init, clientX: x + 90, buttons: 1 }));
            handle.dispatchEvent(new PointerEvent('pointerup', { ...init, clientX: x + 90, buttons: 0 }));
            return true;
        })()"#,
        )
        .await,
        json!(true)
    );
    // Leptos DOM updates and effects flush after the current JavaScript turn.
    // The drag and all three synthetic pointer events above intentionally
    // happen in one turn, so give both the rendered width and preference
    // persistence one scheduling beat before observing them.
    tokio::time::sleep(Duration::from_millis(50)).await;
    let resized = eval_json(
        &harness,
        r#"(() => ({
            width: document.querySelector('[data-entity-table-grid] thead th:first-child').getBoundingClientRect().width,
            stored: localStorage.getItem('ldui-entity-table:client-snapshot-demo'),
        }))()"#,
    )
    .await;
    assert!(
        resized["width"].as_f64().unwrap() > 250.0,
        "resized: {resized}"
    );
    assert_eq!(
        resized["stored"],
        Value::Null,
        "controlled widths must not use localStorage: {resized}"
    );
    let state = oracle(&harness).await;
    assert!(
        state["state"]["entity_table.preferences"]["column_widths"]["client"]
            .as_u64()
            .is_some_and(|width| width > 250),
        "controlled width oracle: {state}"
    );
    let resized_width = resized["width"].as_f64().unwrap();

    harness
        .navigate("/components/client-snapshot-list?pp-freeze=1")
        .await
        .expect("remount demo page");
    wait_for_selector(&harness, "[data-entity-table-grid] tbody tr").await;
    let restored_width = eval_json(
        &harness,
        "document.querySelector('[data-entity-table-grid] thead th:first-child').getBoundingClientRect().width",
    )
    .await
    .as_f64()
    .unwrap();
    assert!(
        restored_width + 40.0 < resized_width,
        "controlled demo must reset rather than reading browser storage: before={resized_width}, after={restored_width}"
    );
    let restored_preferences = eval_json(
        &harness,
        r#"(() => ({
            pageSize: document.querySelector('[data-entity-table] label select').value,
            statusHeader: Array.from(document.querySelectorAll('[data-entity-table-grid] thead th')).some(th => th.textContent.includes('Status')),
        }))()"#,
    )
    .await;
    assert_eq!(
        restored_preferences,
        json!({ "pageSize": "25", "statusHeader": true })
    );
    let state = oracle(&harness).await;
    assert_eq!(
        state["state"]["entity_table.preferences"]["column_order"],
        json!(["client", "status", "case_type", "received", "actions"]),
        "controlled preferences reset to the consumer's mount value: {state}"
    );

    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport");
    tokio::time::sleep(Duration::from_millis(300)).await;
    let compact = eval_json(
        &harness,
        r#"(() => {
            const table = document.querySelector('[data-entity-table-grid]');
            const row = table.querySelector('tbody tr');
            return {
                rows: table.querySelectorAll('tbody tr').length,
                headerDisplay: getComputedStyle(table.querySelector('thead')).display,
                compactDisplay: getComputedStyle(row.cells[0]).display,
                wideDisplay: getComputedStyle(row.cells[1]).display,
                compactText: row.cells[0].textContent.replace(/\s+/g, ' ').trim(),
                compactAction: !!row.cells[0].querySelector('button'),
            };
        })()"#,
    )
    .await;
    assert_eq!(compact["rows"], json!(25));
    assert_eq!(compact["headerDisplay"], json!("none"));
    assert_ne!(compact["compactDisplay"], json!("none"));
    assert_eq!(compact["wideDisplay"], json!("none"));
    assert!(
        compact["compactText"]
            .as_str()
            .unwrap()
            .contains("Mexico City Client 001")
    );
    assert_eq!(compact["compactAction"], json!(true));

    begin_browser_error_capture(&harness).await;
    click(
        &harness,
        "[data-entity-table-grid] tbody tr:first-child td:first-child button",
    )
    .await;
    let compact_counts = eval_json(
        &harness,
        r#"(() => ({
            claim: document.querySelector('[data-testid="entity-claim-count"]').textContent,
            activate: document.querySelector('[data-testid="entity-activate-count"]').textContent,
        }))()"#,
    )
    .await;
    assert_eq!(compact_counts, json!({ "claim": "1", "activate": "0" }));

    let base_profile = ldui_audit::from_ui_tokens(body_font_family(&harness).await);
    // The showcase deliberately gives every daisyUI `.btn` a two-layer press
    // affordance in `demo/input.css`. Declare that authored vocabulary so a
    // different shadow still fails while known buttons do not consume the
    // page's debt ceiling. daisyUI's oklab/oklch form-control shadows remain
    // ratcheted below until the audit parser can understand those colours.
    let mut declared_shadows = base_profile.shadows.clone();
    declared_shadows.extend([
        ShadowSpec::new(0.0, 6.0, 12.0, 0.15).with_spread(-2.0),
        ShadowSpec::new(0.0, 3.0, 6.0, 0.10).with_spread(-2.0),
    ]);
    let profile = base_profile.shadows(declared_shadows);
    let compact_report = ldui_audit::audit_page(&harness, &profile, &Default::default())
        .await
        .expect("audit compact client snapshot page");
    assert_not_truncated(&compact_report, "client snapshot compact");
    assert_eq!(
        compact_report.count(family::OVERLAP),
        0,
        "compact client rows overlap:\n{}",
        compact_report.describe("client snapshot compact")
    );

    harness
        .set_viewport(ViewportSize::new(1280, 800))
        .await
        .expect("restore wide viewport for visual audit");
    tokio::time::sleep(Duration::from_millis(300)).await;
    let report = ldui_audit::audit_page(&harness, &profile, &Default::default())
        .await
        .expect("audit client snapshot page");
    assert_not_truncated(&report, "/components/client-snapshot-list");
    let ceilings = [
        Ceiling::new(family::TYPOGRAPHY, 0),
        Ceiling::new(family::SHAPE, 0),
        // daisyUI's input/select shadows use oklab/oklch colours, which the
        // current engine parser cannot decode -- the same documented gap
        // commit 2fb75d3 raised /components/data-table's ceiling for.
        // Was 5 (dataset selector, search input, page-size select, status
        // filter, case filter); the hybrid column-filters wave added the
        // `client` column filter (an `input.input`) and the entity table's
        // own internal rows-per-page `select.select`, two more instances of
        // the identical stock inset box-shadow, not two new defects. Exact
        // measured debt; no slack. Lower this when that parser gap is fixed.
        Ceiling::new(family::DEPTH, 7),
        Ceiling::new(family::GRID, 0),
        Ceiling::new(family::INTERNAL, 0),
        Ceiling::new(family::COMPONENT_DRIFT, 0),
    ];
    ldui_audit::verify("/components/client-snapshot-list", &report, &ceilings)
        .unwrap_or_else(|error| panic!("{error}"));

    // Negative control: inject one violation in every newly gated style and
    // layout family, prove the engine sees it, then remove it and prove the
    // page returns to its exact baseline counts.
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const host = document.createElement('div');
                host.id = 'client-snapshot-audit-probe';
                host.innerHTML =
                    '<p style="font-size:13.37px">off-ramp type</p>' +
                    '<div style="border-radius:17px;width:40px;height:40px">off-shape</div>' +
                    '<div><div style="height:40px"></div>' +
                    '<div style="height:40px;margin-top:-20px"></div>' +
                    '<div style="height:40px;margin-top:7px"></div></div>' +
                    '<button>raw control</button>';
                document.querySelector('main').appendChild(host);
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    let dirty = ldui_audit::audit_page(&harness, &profile, &Default::default())
        .await
        .expect("audit injected client snapshot violations");
    for visual_family in [
        family::TYPOGRAPHY,
        family::SHAPE,
        family::OVERLAP,
        family::GRID,
        family::COMPONENT_DRIFT,
    ] {
        assert!(
            dirty.count(visual_family) > report.count(visual_family),
            "negative control missed {visual_family}:\n{}",
            dirty.describe("client snapshot injected probe")
        );
    }
    assert_eq!(
        eval_json(
            &harness,
            "document.getElementById('client-snapshot-audit-probe').remove(); true",
        )
        .await,
        json!(true)
    );
    let restored = ldui_audit::audit_page(&harness, &profile, &Default::default())
        .await
        .expect("re-audit client snapshot after probe removal");
    for visual_family in [
        family::TYPOGRAPHY,
        family::SHAPE,
        family::DEPTH,
        family::OVERLAP,
        family::GRID,
        family::INTERNAL,
        family::COMPONENT_DRIFT,
    ] {
        assert_eq!(
            restored.count(visual_family),
            report.count(visual_family),
            "{visual_family} did not return to baseline after probe removal"
        );
    }

    assert_no_browser_errors(&harness, "client snapshot list journey").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn hybrid_filters_localization_defaults_and_focus_recovery_are_framework_owned() {
    let harness = harness_at("/components/client-snapshot-list").await;
    begin_browser_error_capture(&harness).await;
    wait_for_selector(&harness, "[data-entity-column-filter-row]").await;

    let initial = eval_json(
        &harness,
        r#"(() => {
            const table = document.querySelector('[data-entity-table-grid]');
            const header = Array.from(table.querySelectorAll('thead tr:first-child th'));
            const filters = Array.from(table.querySelectorAll('[data-entity-column-filter-row] th'));
            const controls = Array.from(table.querySelectorAll('[data-entity-filter-control]'));
            header.forEach((cell, index) => cell.__hybridNode = `header-${index}`);
            return {
                headerIds: header.map(cell => cell.dataset.entityColumn),
                filterIds: filters.map(cell => cell.dataset.entityColumn),
                controls: controls.length,
                controlIds: controls.map(control => control.id),
                uniqueControlIds: new Set(controls.map(control => control.id)).size,
                labelsTargetControls: controls.every(control => control.closest('label')?.htmlFor === control.id),
                statusCell: table.querySelector('#entity-status-filter')?.closest('th')?.dataset.entityColumn,
                caseCell: table.querySelector('#entity-case-filter')?.closest('th')?.dataset.entityColumn,
                clientCell: table.querySelector('#entity-client-filter')?.closest('th')?.dataset.entityColumn,
                detachedStatus: document.querySelectorAll('[data-filter-bar] #entity-status-filter').length,
                // ldui-3br added two more standalone `FilterBar` fixtures
                // further down this same page (`filter-bar-actions-only`,
                // `filter-bar-columns-only`) for the reactivity suite's own
                // coverage; the actions-only one supplies `on_reset` and so
                // renders its own Reset button. Scope to the primary
                // FilterBar this test is about, excluding those siblings,
                // rather than counting every Reset/Save button on the page.
                resets: document.querySelectorAll('[data-filter-reset]:not([data-testid="filter-bar-actions-only"] *, [data-testid="filter-bar-columns-only"] *)').length,
                saves: document.querySelectorAll('[data-filter-save-default]:not([data-testid="filter-bar-actions-only"] *, [data-testid="filter-bar-columns-only"] *)').length,
                generation: Number(document.querySelector('[data-entity-focus-region]').dataset.entityColumnGeneration),
            };
        })()"#,
    )
    .await;
    assert_eq!(initial["headerIds"], initial["filterIds"]);
    assert_eq!(initial["controls"], json!(3));
    assert_eq!(initial["uniqueControlIds"], initial["controls"]);
    assert_eq!(initial["labelsTargetControls"], json!(true));
    assert_eq!(
        initial["controlIds"],
        json!([
            "entity-client-filter",
            "entity-status-filter",
            "entity-case-filter"
        ])
    );
    assert_eq!(initial["clientCell"], json!("client"));
    assert_eq!(initial["statusCell"], json!("status"));
    assert_eq!(initial["caseCell"], json!("case_type"));
    assert_eq!(initial["detachedStatus"], json!(0));
    assert_eq!(initial["resets"], json!(1));
    assert_eq!(initial["saves"], json!(1));
    assert_entity_projection_matches_wide_dom(&harness, "initial controlled filters").await;

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const select = document.querySelector('#entity-status-filter');
                const sort = document.querySelector("th[data-entity-column='status']").getAttribute('aria-sort');
                select.value = 'Urgent';
                select.dispatchEvent(new Event('change', { bubbles: true }));
                return { beforeSort: sort, active: document.activeElement === select };
            })()"#,
        )
        .await["beforeSort"],
        Value::Null,
        "changing a column filter must not sort its column"
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    let filtered = eval_json(
        &harness,
        r#"(() => ({
            rows: document.querySelectorAll('[data-entity-table-grid] tbody tr').length,
            values: Array.from(document.querySelectorAll("[data-entity-table-grid] tbody td:nth-of-type(3)"))
                .filter(cell => getComputedStyle(cell).display !== 'none')
                .map(cell => cell.textContent.trim()),
            saveDisabled: document.querySelector('[data-filter-save-default]').disabled,
            resetDisabled: document.querySelector('[data-filter-reset]').disabled,
        }))()"#,
    )
    .await;
    assert_eq!(filtered["rows"], json!(24));
    assert!(
        filtered["values"]
            .as_array()
            .is_some_and(|values| values.iter().all(|value| value == "Urgent")),
        "status control did not filter the local snapshot: {filtered}"
    );
    assert_eq!(filtered["saveDisabled"], json!(false));
    assert_eq!(filtered["resetDisabled"], json!(false));
    assert_entity_projection_matches_wide_dom(&harness, "local status filter").await;

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const select = document.querySelector('#entity-status-filter');
                select.value = 'Rejected';
                select.dispatchEvent(new Event('change', { bubbles: true }));
                return {
                    value: select.value,
                    rows: document.querySelectorAll('[data-entity-table-grid] tbody tr').length,
                };
            })()"#,
        )
        .await,
        json!({ "value": "Urgent", "rows": 24 }),
        "a rejected controlled proposal must restore the accepted DOM value"
    );

    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-filter-save-default]').click(); true",
        )
        .await,
        json!(true)
    );
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                count: document.querySelector('[data-testid="entity-save-count"]').textContent,
                disabled: document.querySelector('[data-filter-save-default]').disabled,
                feedback: document.querySelector('[data-filter-save-feedback]')?.textContent.trim(),
            }))()"#,
        )
        .await,
        json!({ "count": "1", "disabled": true, "feedback": "Default view saved" })
    );
    click(&harness, "[data-filter-save-default]").await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-testid=\"entity-save-count\"]').textContent",
        )
        .await,
        json!("1"),
        "a disabled Save must not invoke persistence"
    );

    click(&harness, "[data-testid='save-state-dirty']").await;
    harness
        .page()
        .find_element("[data-filter-save-default]")
        .await
        .expect("find Save as Default")
        .focus()
        .await
        .expect("focus Save as Default");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("keyboard-save defaults with Enter");
    click(&harness, "[data-testid='save-state-dirty']").await;
    harness
        .page()
        .find_element("[data-filter-save-default]")
        .await
        .expect("refind Save as Default")
        .focus()
        .await
        .expect("refocus Save as Default");
    harness
        .press_key_sequence(&[Key::Space])
        .await
        .expect("keyboard-save defaults with Space");
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-testid=\"entity-save-count\"]').textContent",
        )
        .await,
        json!("3"),
        "pointer, Enter, and Space must each save exactly once"
    );

    click(&harness, "[data-testid='save-state-pending']").await;
    let pending = eval_json(
        &harness,
        r#"(() => ({
            disabled: document.querySelector('[data-filter-save-default]').disabled,
            label: document.querySelector('[data-filter-save-default]').getAttribute('aria-label'),
            status: document.querySelector('[data-filter-save-feedback="status"]')?.textContent.trim(),
        }))()"#,
    )
    .await;
    assert_eq!(pending["disabled"], json!(true));
    assert!(pending["label"].as_str().unwrap().contains("in progress"));
    assert_eq!(pending["status"], json!("Saving default view"));
    click(&harness, "[data-testid='save-state-conflict']").await;
    assert!(
        eval_json(
            &harness,
            "document.querySelector('[data-filter-save-feedback=\"alert\"]').textContent",
        )
        .await
        .as_str()
        .unwrap()
        .contains("revision changed")
    );
    click(&harness, "[data-testid='save-state-failure']").await;
    assert!(
        eval_json(
            &harness,
            "document.querySelector('[data-filter-save-feedback=\"alert\"]').textContent",
        )
        .await
        .as_str()
        .unwrap()
        .contains("network unavailable")
    );

    click(&harness, "[data-entity-column-chooser]").await;
    let active_hide = eval_json(
        &harness,
        r#"(() => {
            const item = document.querySelector("[role='menu'] [data-entity-column='status']");
            const control = item.matches('[role="menuitemcheckbox"]')
                ? item
                : item.querySelector('[role="menuitemcheckbox"]');
            const before = control.getAttribute('aria-checked');
            control.click();
            return {
                disabled: control.getAttribute('aria-disabled'),
                before,
                after: control.getAttribute('aria-checked'),
                statusHeader: !!document.querySelector("thead th[data-entity-column='status']"),
                activeCopy: item.textContent.replace(/\s+/g, ' ').trim(),
            };
        })()"#,
    )
    .await;
    assert_eq!(active_hide["disabled"], json!("true"));
    assert_eq!(active_hide["before"], json!("true"));
    assert_eq!(active_hide["after"], json!("true"));
    assert_eq!(active_hide["statusHeader"], json!(true));
    assert!(
        active_hide["activeCopy"]
            .as_str()
            .unwrap()
            .contains("Filter active")
    );
    click(
        &harness,
        "[data-entity-column-order='status'][data-entity-column-move='later']",
    )
    .await;
    click(&harness, "[role='menu'] [data-entity-column='case_type']").await;
    let aligned = eval_json(
        &harness,
        r#"(() => {
            const table = document.querySelector('[data-entity-table-grid]');
            const headerCells = Array.from(table.querySelectorAll('thead tr:first-child th'));
            headerCells.forEach(cell => cell.__beforeLocale = cell.dataset.entityColumn);
            return {
                headers: headerCells.map(cell => cell.dataset.entityColumn),
                filters: Array.from(table.querySelectorAll('[data-entity-column-filter-row] th')).map(cell => cell.dataset.entityColumn),
                statusCell: table.querySelector('#entity-status-filter')?.closest('th')?.dataset.entityColumn,
            };
        })()"#,
    )
    .await;
    assert_eq!(aligned["headers"], aligned["filters"]);
    assert_eq!(aligned["statusCell"], json!("status"));
    assert!(
        !aligned["headers"]
            .as_array()
            .unwrap()
            .contains(&json!("case_type"))
    );
    assert_entity_projection_matches_wide_dom(&harness, "reorder and hidden column").await;

    let before_locale_preferences =
        oracle(&harness).await["state"]["entity_table.preferences"].clone();
    click(&harness, "[data-testid='toggle-client-locale']").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let localized = eval_json(
        &harness,
        r#"(() => {
            const table = document.querySelector('[data-entity-table-grid]');
            const headers = Array.from(table.querySelectorAll('thead tr:first-child th'));
            return {
                labels: headers.map(cell => cell.textContent.replace(/\s+/g, ' ').trim()),
                sameNodes: headers.every(cell => cell.__beforeLocale === cell.dataset.entityColumn),
                statusLabel: table.querySelector('#entity-status-filter').getAttribute('aria-label'),
                statusOption: table.querySelector('#entity-status-filter option:checked').textContent,
                reset: document.querySelector('[data-filter-reset]').textContent.trim(),
                save: document.querySelector('[data-filter-save-default]').textContent.trim(),
                datasetStatus: document.querySelector('[data-dataset-selector-status]')?.textContent.trim(),
                statusValue: table.querySelector('#entity-status-filter').value,
                generation: Number(document.querySelector('[data-entity-focus-region]').dataset.entityColumnGeneration),
            };
        })()"#,
    )
    .await;
    assert!(
        localized["labels"].as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .contains("Cliente")
    );
    assert_eq!(localized["sameNodes"], json!(true));
    assert_eq!(localized["statusLabel"], json!("Estado"));
    assert_eq!(localized["statusOption"], json!("Urgente"));
    assert_eq!(localized["reset"], json!("Restablecer"));
    assert_eq!(localized["save"], json!("Guardar como predeterminado"));
    assert!(
        localized["datasetStatus"]
            .as_str()
            .unwrap()
            .starts_with("Mostrando ")
    );
    assert_eq!(localized["statusValue"], json!("Urgent"));
    assert!(localized["generation"].as_u64() > initial["generation"].as_u64());
    assert_eq!(
        oracle(&harness).await["state"]["entity_table.preferences"],
        before_locale_preferences,
        "locale-only column replacement reset surviving table preferences"
    );
    assert_entity_projection_matches_wide_dom(&harness, "localized column replacement").await;
    let localized_wide_projection =
        oracle(&harness).await["state"]["entity_table.display_projection"].clone();

    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set localized compact viewport");
    tokio::time::sleep(Duration::from_millis(150)).await;
    let compact = eval_json(
        &harness,
        r#"(() => ({
            rowCopy: document.querySelector('[data-entity-table-grid] tbody tr td:first-child').textContent.replace(/\s+/g, ' ').trim(),
            panel: document.querySelector('[data-entity-responsive-filter-panel]')?.textContent.replace(/\s+/g, ' ').trim(),
            panelControls: document.querySelectorAll('[data-entity-responsive-filter-panel] [data-entity-filter-control]').length,
            headerControls: document.querySelectorAll('[data-entity-column-filter-row] [data-entity-filter-control]').length,
            controlIds: Array.from(document.querySelectorAll('[data-entity-filter-control]')).map(control => control.id),
            uniqueControlIds: new Set(Array.from(document.querySelectorAll('[data-entity-filter-control]')).map(control => control.id)).size,
            statusId: document.querySelector('[data-entity-filter-control="status"]')?.id,
            caseId: document.querySelector('[data-entity-filter-control="case_type"]')?.id,
            statusValue: document.querySelector('[data-entity-filter-control="status"]')?.value,
            clearStatus: document.querySelectorAll('[data-entity-clear-filter="status"]').length,
            caseHidden: !Array.from(document.querySelectorAll('[data-entity-table-grid] tbody tr:first-child td[data-entity-column="case_type"]'))
                .some(cell => getComputedStyle(cell).display !== 'none'),
        }))()"#,
    )
    .await;
    assert!(
        compact["rowCopy"].as_str().unwrap().contains("Cliente")
            && compact["rowCopy"].as_str().unwrap().contains("Estado"),
        "default compact copy retained the old locale: {compact}"
    );
    assert!(
        compact["panel"]
            .as_str()
            .unwrap()
            .contains("Filtros de columnas")
    );
    assert!(compact["panel"].as_str().unwrap().contains("Estado"));
    assert!(compact["panel"].as_str().unwrap().contains("Tipo de caso"));
    assert_eq!(compact["panelControls"], json!(3));
    assert_eq!(compact["headerControls"], json!(0));
    assert_eq!(compact["uniqueControlIds"], compact["panelControls"]);
    assert_eq!(
        compact["statusId"],
        json!("entity-status-filter-responsive")
    );
    assert_eq!(compact["caseId"], json!("entity-case-filter-responsive"));
    assert_eq!(compact["statusValue"], json!("Urgent"));
    assert_eq!(compact["clearStatus"], json!(1));
    assert_eq!(compact["caseHidden"], json!(true));
    assert_eq!(
        oracle(&harness).await["state"]["entity_table.display_projection"],
        localized_wide_projection,
        "compact layout changed the atomic display projection"
    );

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const select = document.querySelector('[data-entity-filter-control="case_type"]');
                select.value = 'Family';
                select.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-entity-filter-control=\"case_type\"]').value",
        )
        .await,
        json!("Family")
    );
    harness
        .page()
        .find_element("[data-entity-clear-filter='status']")
        .await
        .expect("find compact status clear control")
        .focus()
        .await
        .expect("focus compact status clear control");
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("clear the controlled status filter with Enter");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                value: document.querySelector('[data-entity-filter-control="status"]').value,
                clearButtons: document.querySelectorAll('[data-entity-clear-filter="status"]').length,
                caseValue: document.querySelector('[data-entity-filter-control="case_type"]').value,
                uniqueIds: new Set(Array.from(document.querySelectorAll('[data-entity-filter-control]')).map(control => control.id)).size,
                controlCount: document.querySelectorAll('[data-entity-filter-control]').length,
            }))()"#,
        )
        .await,
        json!({ "value": "", "clearButtons": 0, "caseValue": "Family", "uniqueIds": 3, "controlCount": 3 })
    );
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const select = document.querySelector('[data-entity-filter-control="status"]');
                select.value = 'Urgent';
                select.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    harness
        .set_viewport(ViewportSize::new(1280, 800))
        .await
        .expect("restore wide viewport");
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Restoring: re-checking `case_type`'s chooser item unhides it even
    // though its filter is still active ("Family", set while it was hidden
    // via the responsive fallback panel above). `EntityColumnFilter`'s
    // `disabled` guard is one-directional -- it blocks HIDING a visible,
    // actively-filtered column (see the `status` column's `active_hide`
    // assertions earlier in this test), never SHOWING an already-hidden
    // one; otherwise a column filtered while hidden could never be restored
    // without first clearing its filter.
    click(&harness, "[data-entity-column-chooser]").await;
    click(&harness, "[role='menu'] [data-entity-column='case_type']").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let restored_filter = eval_json(
        &harness,
        r#"(() => ({
            total: document.querySelectorAll('[data-entity-filter-control="case_type"]').length,
            aligned: document.querySelector('[data-entity-filter-control="case_type"]')?.closest('th')?.dataset.entityColumn,
            value: document.querySelector('[data-entity-filter-control="case_type"]')?.value,
            fallbackPanel: document.querySelectorAll('[data-entity-responsive-filter-panel]').length,
        }))()"#,
    )
    .await;
    assert_eq!(restored_filter["total"], json!(1));
    assert_eq!(restored_filter["aligned"], json!("case_type"));
    assert_eq!(restored_filter["value"], json!("Family"));
    assert_eq!(restored_filter["fallbackPanel"], json!(0));
    assert_entity_projection_matches_wide_dom(&harness, "restored hidden filtered column").await;

    click(&harness, "[data-filter-reset]").await;
    click(&harness, "[data-testid='toggle-client-locale']").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const rows = Array.from(document.querySelectorAll('[data-entity-row-key]'));
                rows.forEach(row => row.__stableEntityRow = row.dataset.entityRowKey);
                return rows.length;
            })()"#,
        )
        .await,
        json!(25)
    );
    click(
        &harness,
        "[data-entity-table-grid] thead tr:first-child th[data-entity-column='status']",
    )
    .await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let stable_rows = eval_json(
        &harness,
        r#"(() => {
            const retained = Array.from(document.querySelectorAll('[data-entity-row-key]'))
                .filter(row => row.__stableEntityRow !== undefined);
            return {
                retained: retained.length,
                sameNodes: retained.every(row => row.__stableEntityRow === row.dataset.entityRowKey),
            };
        })()"#,
    )
    .await;
    assert!(stable_rows["retained"].as_u64().unwrap() > 0);
    assert_eq!(
        stable_rows["sameNodes"],
        json!(true),
        "sorting replaced keyed row nodes instead of moving them"
    );

    let focus_before = eval_json(
        &harness,
        r#"(() => {
            const buttons = Array.from(document.querySelectorAll('[data-entity-row-action="claim"] button'))
                .filter(button => button.getBoundingClientRect().width > 0);
            const first = buttons[0];
            const row = first.closest('[data-entity-row-key]');
            const next = buttons[1].closest('[data-entity-row-key]');
            first.focus();
            return { current: row.dataset.entityRowKey, expected: next.dataset.entityRowKey };
        })()"#,
    )
    .await;
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("claim and remove focused row");
    tokio::time::sleep(Duration::from_millis(150)).await;
    let recovered = eval_json(
        &harness,
        r#"(() => ({
            row: document.activeElement?.closest('[data-entity-row-key]')?.dataset.entityRowKey ?? null,
            action: document.activeElement?.closest('[data-entity-row-action]')?.dataset.entityRowAction ?? null,
            removedStillPresent: !!document.querySelector(`[data-entity-row-key="${window.__removedKey || ''}"]`),
        }))()"#,
    )
    .await;
    assert_eq!(recovered["row"], focus_before["expected"]);
    assert_eq!(recovered["action"], json!("claim"));

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const button = Array.from(document.querySelectorAll('[data-entity-row-action="claim"] button'))
                    .find(button => button.getBoundingClientRect().width > 0);
                const row = button.closest('[data-entity-row-key]');
                const status = row.querySelector('td:nth-of-type(3)')?.textContent.trim();
                button.focus();
                const select = document.querySelector('[data-entity-filter-control="status"]');
                select.value = status === 'Urgent' ? 'Ready' : 'Urgent';
                select.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(
        eval_json(
            &harness,
            "document.activeElement?.matches('[data-entity-focus-region]') ?? false",
        )
        .await,
        json!(true),
        "filtering a still-present source row out must fall back to the table region"
    );

    click(&harness, "[data-filter-reset]").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let retained_action = eval_json(
        &harness,
        r#"(() => {
            const button = Array.from(document.querySelectorAll('[data-retain-row]'))
                .find(button => button.getBoundingClientRect().width > 0);
            const key = button.closest('[data-entity-row-key]').dataset.entityRowKey;
            button.focus();
            button.click();
            return {
                key,
                focused: document.activeElement === button,
                present: !!document.querySelector(`[data-entity-row-key="${key}"]`),
            };
        })()"#,
    )
    .await;
    assert_eq!(retained_action["focused"], json!(true));
    assert_eq!(retained_action["present"], json!(true));
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-testid=\"entity-retain-count\"]').textContent",
        )
        .await,
        json!("1"),
        "a non-removing row action must fire once and retain native focus"
    );

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const claim = Array.from(document.querySelectorAll('[data-claim-row]'))
                    .find(button => button.getBoundingClientRect().width > 0);
                const search = document.querySelector('[data-filter-bar] input[type="search"]');
                claim.focus();
                search.focus();
                claim.click();
                return document.activeElement === search;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            "document.activeElement === document.querySelector('[data-filter-bar] input[type=\"search\"]')",
        )
        .await,
        json!(true),
        "row removal stole focus after the user had moved to search"
    );

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const claim = Array.from(document.querySelectorAll('[data-claim-row]'))
                    .find(button => button.getBoundingClientRect().width > 0);
                claim.focus();
                document.querySelector('[data-testid="change-access-generation"]').click();
                return document.activeElement === claim;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const focused = document.activeElement;
                focused.click();
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        eval_json(
            &harness,
            "({ action: document.activeElement?.closest('[data-entity-row-action]')?.dataset.entityRowAction ?? null })",
        )
        .await["action"],
        Value::Null,
        "an access-generation change allowed later recovery into another row"
    );

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const action = Array.from(document.querySelectorAll('[data-retain-row]'))
                    .find(button => button.getBoundingClientRect().width > 0);
                const selector = document.querySelector('[data-dataset-selector] select');
                action.focus();
                selector.value = 'office-in';
                selector.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            })()"#,
        )
        .await,
        json!(true)
    );
    tokio::time::sleep(Duration::from_millis(150)).await;
    let cross_dataset_focus = eval_json(
        &harness,
        r#"(() => ({
            action: document.activeElement?.closest('[data-entity-row-action]')?.dataset.entityRowAction ?? null,
            allNewDataset: Array.from(document.querySelectorAll('[data-entity-row-key]'))
                .every(row => row.dataset.entityRowKey.startsWith('office-in-')),
        }))()"#,
    )
    .await;
    assert_eq!(cross_dataset_focus["action"], Value::Null);
    assert_eq!(cross_dataset_focus["allNewDataset"], json!(true));

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(harness.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("hybrid filters, localization, defaults, and focus recovery")
        .unwrap_or_else(|error| panic!("{error}; {}", report.summary()));
    assert_no_browser_errors(&harness, "hybrid EntityTable foundation").await;
}

/// Controlled single-row selection (ldui-sh3): a caller-owned `selected_key`
/// drives both `aria-selected`/styling and a separate master-detail readout,
/// proposal-first (a rejected proposal never paints), coherent across the
/// wide and compact presentations (they are one shared `<tr>`), keyboard
/// Space works identically to a click, a row-action control neither selects
/// nor activates, and removing the accepted key's row leaves the table with
/// no selected row instead of aliasing a different one.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn entity_table_selection_drives_master_detail_and_survives_removal() {
    let harness = harness_at("/components/entity-table-selection").await;
    begin_browser_error_capture(&harness).await;
    wait_for_selector(
        &harness,
        "#entity-selection-table tbody tr[data-entity-row-key]",
    )
    .await;

    // Nothing selected yet: no row carries `aria-selected="true"`, and the
    // master-detail panel -- driven purely by the same accepted key -- shows
    // no row.
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                selected: Array.from(document.querySelectorAll('#entity-selection-table tbody tr[aria-selected="true"]')).map(row => row.dataset.entityRowKey),
                detail: document.querySelector('[data-testid="entity-selection-detail"]').textContent.trim(),
            }))()"#,
        )
        .await,
        json!({ "selected": [], "detail": "(no row selected)" })
    );

    // A plain click on a row-action control must neither select nor
    // activate the row it sits in (`event_origin_is_action` / the action
    // cell's own stop-propagation). Wide and compact share one `<tr>`, so
    // this `data-testid` exists twice at this viewport -- once in the
    // (visible) wide `<td>`, once in the compact `<td class="lg:hidden">`
    // that precedes it in document order. Scope to the wide `<td>`,
    // matching how the rest of this test scopes to
    // `td[data-entity-column='client']`, so the click lands on the visible
    // node rather than chromiumoxide's document-order-first (hidden) match.
    click(
        &harness,
        "#entity-selection-table tbody tr[data-entity-row-key='office-mx-1'] td[data-entity-column='view'] [data-testid='entity-selection-row-action']",
    )
    .await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                actionClicks: Number(document.querySelector('[data-testid="entity-selection-action-clicks"]').textContent),
                activations: Number(document.querySelector('[data-testid="entity-selection-activations"]').textContent),
                proposals: Number(document.querySelector('[data-testid="entity-selection-proposals"]').textContent),
                selected: Array.from(document.querySelectorAll('#entity-selection-table tbody tr[aria-selected="true"]')).map(row => row.dataset.entityRowKey),
            }))()"#,
        )
        .await,
        json!({ "actionClicks": 1, "activations": 0, "proposals": 0, "selected": [] }),
        "a row-action click must not select or activate the row"
    );

    // A plain click elsewhere on the row proposes and (proposals accepted by
    // default) becomes the accepted selection: one proposal, one activation
    // (they coexist), `aria-selected` + the selected class land on the row's
    // single shared `<tr>`, and the master-detail panel reflects the same
    // accepted key.
    click(
        &harness,
        "#entity-selection-table tbody tr[data-entity-row-key='office-mx-1'] td[data-entity-column='client']",
    )
    .await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const row = document.querySelector('#entity-selection-table tbody tr[data-entity-row-key="office-mx-1"]');
                return {
                    acceptedKey: document.querySelector('[data-testid="entity-selection-selected-key"]').textContent.trim(),
                    proposals: Number(document.querySelector('[data-testid="entity-selection-proposals"]').textContent),
                    activations: Number(document.querySelector('[data-testid="entity-selection-activations"]').textContent),
                    selected: Array.from(document.querySelectorAll('#entity-selection-table tbody tr[aria-selected="true"]')).map(row => row.dataset.entityRowKey),
                    selectedClass: row.classList.contains('bg-base-200'),
                    detail: document.querySelector('[data-testid="entity-selection-detail"]').textContent.trim(),
                };
            })()"#,
        )
        .await,
        json!({
            "acceptedKey": "office-mx-1",
            "proposals": 1,
            "activations": 1,
            "selected": ["office-mx-1"],
            "selectedClass": true,
            "detail": "Mexico City Client 1 — Urgent",
        })
    );

    // Coherent across wide and compact presentations: they are one shared
    // `<tr>`, so shrinking below the `lg:` breakpoint swaps which `<td>` is
    // visible but must not disturb the row's `aria-selected`/selected class.
    harness
        .set_viewport(ViewportSize::TABLET)
        .await
        .expect("shrink to a compact-layout viewport");
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const row = document.querySelector('#entity-selection-table tbody tr[data-entity-row-key="office-mx-1"]');
                const wideCell = row.querySelector('td[data-entity-column="client"]');
                const compactCell = row.querySelector('td.lg\\:hidden');
                return {
                    ariaSelected: row.getAttribute('aria-selected'),
                    selectedClass: row.classList.contains('bg-base-200'),
                    wideHidden: getComputedStyle(wideCell).display === 'none',
                    compactVisible: getComputedStyle(compactCell).display !== 'none',
                };
            })()"#,
        )
        .await,
        json!({
            "ariaSelected": "true",
            "selectedClass": true,
            "wideHidden": true,
            "compactVisible": true,
        }),
        "selected styling/aria-selected must hold on the same <tr> in the compact layout"
    );
    harness
        .set_viewport(ViewportSize::SMALL)
        .await
        .expect("restore the wide-layout viewport");

    // A rejected proposal never paints: the accepted key, aria-selected row,
    // and detail panel all stay on office-mx-1 even though a new key was
    // proposed.
    click(&harness, "[data-testid='entity-selection-accept']").await;
    click(
        &harness,
        "#entity-selection-table tbody tr[data-entity-row-key='office-mx-2'] td[data-entity-column='client']",
    )
    .await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                acceptedKey: document.querySelector('[data-testid="entity-selection-selected-key"]').textContent.trim(),
                lastProposal: document.querySelector('[data-testid="entity-selection-last-proposal"]').textContent.trim(),
                proposals: Number(document.querySelector('[data-testid="entity-selection-proposals"]').textContent),
                selected: Array.from(document.querySelectorAll('#entity-selection-table tbody tr[aria-selected="true"]')).map(row => row.dataset.entityRowKey),
                detail: document.querySelector('[data-testid="entity-selection-detail"]').textContent.trim(),
            }))()"#,
        )
        .await,
        json!({
            "acceptedKey": "office-mx-1",
            "lastProposal": "office-mx-2",
            "proposals": 2,
            "selected": ["office-mx-1"],
            "detail": "Mexico City Client 1 — Urgent",
        }),
        "a declined selection proposal must not paint or replace the accepted row"
    );

    // Re-enable acceptance; keyboard Space on a focused row proposes and
    // selects exactly like a click. Focus and selection stay distinct --
    // focusing the row alone (without Space/Enter/click) must not select it.
    click(&harness, "[data-testid='entity-selection-accept']").await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                document.querySelector('#entity-selection-table tbody tr[data-entity-row-key="office-mx-3"]').focus();
                return {
                    focused: document.activeElement.dataset.entityRowKey,
                    selected: Array.from(document.querySelectorAll('#entity-selection-table tbody tr[aria-selected="true"]')).map(row => row.dataset.entityRowKey),
                };
            })()"#,
        )
        .await,
        json!({ "focused": "office-mx-3", "selected": ["office-mx-1"] }),
        "focusing a row must not itself select it"
    );
    harness
        .press_key_sequence(&[Key::Space])
        .await
        .expect("Space selects the focused entity row");
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                acceptedKey: document.querySelector('[data-testid="entity-selection-selected-key"]').textContent.trim(),
                proposals: Number(document.querySelector('[data-testid="entity-selection-proposals"]').textContent),
                selected: Array.from(document.querySelectorAll('#entity-selection-table tbody tr[aria-selected="true"]')).map(row => row.dataset.entityRowKey),
                detail: document.querySelector('[data-testid="entity-selection-detail"]').textContent.trim(),
            }))()"#,
        )
        .await,
        json!({
            "acceptedKey": "office-mx-3",
            "proposals": 3,
            "selected": ["office-mx-3"],
            "detail": "Mexico City Client 3 — Ready",
        }),
        "keyboard Space must select the focused row exactly like a click"
    );

    // Removing the accepted key's row (e.g. it left a live pool) is a
    // fail-safe: the table renders the remaining rows without error, no row
    // renders selected (there is no positional fallback to alias a
    // different entity), and the master-detail panel -- which looks the
    // accepted key up in the same data -- reflects that too, entirely by
    // construction rather than special-cased removal handling.
    click(&harness, "[data-testid='entity-selection-remove-selected']").await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => ({
                acceptedKey: document.querySelector('[data-testid="entity-selection-selected-key"]').textContent.trim(),
                remainingKeys: Array.from(document.querySelectorAll('#entity-selection-table tbody tr[data-entity-row-key]')).map(row => row.dataset.entityRowKey).sort(),
                selected: Array.from(document.querySelectorAll('#entity-selection-table tbody tr[aria-selected="true"]')).map(row => row.dataset.entityRowKey),
                detail: document.querySelector('[data-testid="entity-selection-detail"]').textContent.trim(),
            }))()"#,
        )
        .await,
        json!({
            "acceptedKey": "office-mx-3",
            "remainingKeys": ["office-mx-1", "office-mx-2"],
            "selected": [],
            "detail": "(no row selected)",
        }),
        "removing the selected row's data must leave no row selected, without crashing"
    );

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(harness.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("entity table controlled selection")
        .unwrap_or_else(|error| panic!("{error}; {}", report.summary()));
    assert_no_browser_errors(&harness, "entity table controlled selection").await;
}

/// Reads the framework-owned primary/secondary presentation of the
/// `contact` column, for one row, at whichever width (wide `td` or the
/// compact `td.lg:hidden` row) is currently laid out.
async fn inspect_contact_presentation(
    harness: &pixelproof_web::Harness,
    row_key: &str,
    compact: bool,
) -> Value {
    let selector = if compact {
        // `td.lg\:hidden` needs the class-name colon CSS-escaped, but this
        // string is spliced into a double-quoted JS string literal by
        // `eval_json` below: a single backslash there is an unrecognized JS
        // string escape, so JS drops it and the browser never sees the
        // escape at all (`SyntaxError: ... is not a valid selector`).
        // `[class~='lg:hidden']` matches the same single class token without
        // any CSS escaping, so it survives the JS string round-trip intact.
        // (Single-quoted, not double: the whole selector is itself spliced
        // into a double-quoted JS string literal below.)
        format!(
            "#entity-table-presentation-fixture [data-entity-row-key='{row_key}'] td[class~='lg:hidden'] [data-entity-column='contact']"
        )
    } else {
        format!(
            "#entity-table-presentation-fixture [data-entity-row-key='{row_key}'] td[data-entity-column='contact']"
        )
    };
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const cell = document.querySelector("{selector}");
                const wrapper = cell.querySelector('[data-entity-semantic-cell="primary-secondary"]');
                const decorative = wrapper.querySelector(':scope > [aria-hidden="true"]');
                const primaryLine = wrapper.querySelector('[data-entity-primary-secondary-line="primary"]');
                const secondaryLine = wrapper.querySelector('[data-entity-primary-secondary-line="secondary"]');
                const srOnly = wrapper.querySelector(':scope > .sr-only');
                return {{
                    decorativeAriaHidden: decorative?.getAttribute('aria-hidden') ?? null,
                    decorativeContainsSrOnly: decorative ? decorative.contains(srOnly) : null,
                    primaryText: primaryLine?.textContent ?? null,
                    primaryForcedColors: primaryLine?.className.includes('forced-colors:text') ?? false,
                    primaryOverflow: primaryLine?.dataset.entityTextOverflow ?? null,
                    hasSecondary: !!secondaryLine,
                    secondaryText: secondaryLine?.textContent ?? null,
                    secondaryForcedColors: secondaryLine?.className.includes('forced-colors:text') ?? false,
                    accessibleText: srOnly?.textContent ?? null,
                    wrapperTitle: wrapper.getAttribute('title'),
                }};
            }})()"#
        ),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn entity_column_primary_secondary_composes_two_accessible_lines() {
    let harness = harness_at("/components/entity-table-presentation").await;
    wait_for_selector(
        &harness,
        "#entity-table-presentation-fixture [data-entity-row-key='presentation-1']",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    // A row with a non-empty secondary line: two visible lines, decorative
    // and hidden from assistive technology, plus exactly one accessible
    // name -- the canonical, complete text -- carried by the sole `sr-only`
    // node, so nothing is ever announced twice.
    let with_secondary = inspect_contact_presentation(&harness, "presentation-1", false).await;
    assert_eq!(with_secondary["decorativeAriaHidden"], json!("true"));
    assert_eq!(with_secondary["decorativeContainsSrOnly"], json!(false));
    assert_eq!(with_secondary["primaryText"], json!("Jordan Blake"));
    assert_eq!(with_secondary["primaryForcedColors"], json!(true));
    assert_eq!(with_secondary["primaryOverflow"], json!("wrap"));
    assert_eq!(with_secondary["hasSecondary"], json!(true));
    assert_eq!(with_secondary["secondaryText"], json!("Role: Team lead"));
    assert_eq!(with_secondary["secondaryForcedColors"], json!(true));
    assert_eq!(
        with_secondary["accessibleText"],
        json!("Jordan Blake (Role: Team lead)"),
        "the sr-only accessible name must be the complete canonical value"
    );
    assert_eq!(with_secondary["wrapperTitle"], Value::Null);

    // An empty-but-present secondary value normalizes away: no secondary
    // line, no extra spacing, no leftover punctuation, and the accessible
    // name collapses to just the name.
    let empty_secondary = inspect_contact_presentation(&harness, "presentation-2", false).await;
    assert_eq!(empty_secondary["primaryText"], json!("Sam Rivera"));
    assert_eq!(empty_secondary["hasSecondary"], json!(false));
    assert_eq!(empty_secondary["secondaryText"], Value::Null);
    assert_eq!(empty_secondary["accessibleText"], json!("Sam Rivera"));

    // An absent (`None`) secondary value behaves identically to an empty one.
    let absent_secondary = inspect_contact_presentation(&harness, "presentation-3", false).await;
    assert_eq!(absent_secondary["primaryText"], json!("Alex Chen"));
    assert_eq!(absent_secondary["hasSecondary"], json!(false));
    assert_eq!(absent_secondary["accessibleText"], json!("Alex Chen"));

    // Reactive ROW-DATA replacement is a different reactive primitive from a
    // columns-Signal swap: the fixture's `columns` Signal only tracks
    // `column_locale`, never the row-data toggle, so it must leave the
    // contact column's rendered lines untouched -- proving the two
    // mechanisms are isolated before the columns-Signal assertion below.
    click(&harness, "[data-testid='entity-presentation-locale']").await;
    let after_row_toggle = inspect_contact_presentation(&harness, "presentation-1", false).await;
    assert_eq!(
        after_row_toggle["secondaryText"], with_secondary["secondaryText"],
        "a row-data-only toggle must not touch the columns-Signal-driven presentation"
    );
    assert_eq!(
        after_row_toggle["accessibleText"],
        with_secondary["accessibleText"]
    );

    // Reactive COLUMN replacement (the established meaning in this codebase:
    // swapping the whole `columns` prop for a new `Signal<Vec<EntityColumn<T>>>`
    // value, see `ColumnStore::Reactive` in component.rs and the
    // "locale-only column replacement reset surviving table preferences"
    // case in `controlled_preferences_reorder_columns_and_compose_sort_clauses`
    // above). The new Vec's `contact` column carries fresh primary/secondary
    // closures -- a locale-style relabel of the secondary composition -- so
    // this proves EntityColumn::primary_secondary re-renders both lines when
    // the *columns list itself* is replaced, not just row data, and that the
    // framework's column-generation marker advances accordingly.
    let generation_before = eval_json(
        &harness,
        "Number(document.querySelector('[data-entity-focus-region]').dataset.entityColumnGeneration)",
    )
    .await;
    click(
        &harness,
        "[data-testid='entity-presentation-column-locale']",
    )
    .await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let generation_after = eval_json(
        &harness,
        "Number(document.querySelector('[data-entity-focus-region]').dataset.entityColumnGeneration)",
    )
    .await;
    assert!(
        generation_after.as_f64() > generation_before.as_f64(),
        "swapping the columns Signal must bump the column-generation marker: before={generation_before}, after={generation_after}"
    );
    let header_text = eval_json(
        &harness,
        "document.querySelector(\"#entity-table-presentation-fixture th[data-entity-column='contact']\").textContent.replace(/\\s+/g, ' ').trim()",
    )
    .await;
    assert!(
        header_text
            .as_str()
            .is_some_and(|text| text.contains("Principal y secundario")),
        "the columns Signal swap must relabel the contact header: {header_text}"
    );
    let column_localized = inspect_contact_presentation(&harness, "presentation-1", false).await;
    assert_eq!(
        column_localized["primaryText"],
        json!("Jordan Blake"),
        "the primary line's closure is unchanged by the relabel, only the secondary's is"
    );
    assert_eq!(column_localized["secondaryText"], json!("Rol: Team lead"));
    assert_eq!(
        column_localized["accessibleText"],
        json!("Jordan Blake (Rol: Team lead)"),
        "the columns Signal swap must update the accessible line alongside the visual ones"
    );
    let column_localized_empty =
        inspect_contact_presentation(&harness, "presentation-3", false).await;
    assert_eq!(
        column_localized_empty["hasSecondary"],
        json!(false),
        "a row with no role stays secondary-less after the columns Signal swap"
    );
    assert_eq!(column_localized_empty["accessibleText"], json!("Alex Chen"));

    // Resizing the column changes only geometry: both lines keep their text.
    let contact_separator = "th[data-entity-column='contact'] [role='separator']";
    harness
        .page()
        .find_element(contact_separator)
        .await
        .expect("find contact resize separator")
        .focus()
        .await
        .expect("focus contact resize separator");
    let before_resize = eval_json(
        &harness,
        &format!(
            "Number(document.querySelector(\"{contact_separator}\").getAttribute('aria-valuenow'))"
        ),
    )
    .await;
    harness
        .press_key_sequence(&[Key::ArrowLeft, Key::ArrowLeft])
        .await
        .expect("resize the contact column from the keyboard");
    let after_resize = eval_json(
        &harness,
        &format!(
            "Number(document.querySelector(\"{contact_separator}\").getAttribute('aria-valuenow'))"
        ),
    )
    .await;
    assert!(
        after_resize.as_f64() < before_resize.as_f64(),
        "keyboard resize must reduce the contact column width: before={before_resize}, after={after_resize}"
    );
    let resized = inspect_contact_presentation(&harness, "presentation-1", false).await;
    assert_eq!(resized["primaryText"], column_localized["primaryText"]);
    assert_eq!(resized["secondaryText"], column_localized["secondaryText"]);

    // Compact width: the same primary/secondary structure renders inside
    // the compact (label/value) row instead of the wide `<td>`.
    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport for the contact column");
    tokio::time::sleep(Duration::from_millis(150)).await;
    let compact = inspect_contact_presentation(&harness, "presentation-1", true).await;
    assert_eq!(compact["primaryText"], json!("Jordan Blake"));
    assert_eq!(compact["secondaryText"], json!("Rol: Team lead"));
    assert_eq!(
        compact["accessibleText"],
        json!("Jordan Blake (Rol: Team lead)")
    );
    let compact_empty = inspect_contact_presentation(&harness, "presentation-3", true).await;
    assert_eq!(compact_empty["hasSecondary"], json!(false));
    assert_eq!(
        eval_json(
            &harness,
            "document.documentElement.scrollWidth > document.documentElement.clientWidth"
        )
        .await,
        json!(false),
        "the compact contact cell must not overflow the page"
    );
    harness
        .set_viewport(ViewportSize::new(1280, 800))
        .await
        .expect("restore wide viewport before hiding the contact column");
    // Settle after the viewport change before the next CDP click, matching
    // the established pattern elsewhere in this file (see the identical
    // restore-then-click sequence above): without it the click's box model
    // can be computed against the pre-reflow (compact) layout and land on
    // the wrong coordinates, so the column chooser never opens and the
    // subsequent menu-item click is a no-op.
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Hiding: the column chooser removing `contact` drops both the header
    // and every row's presentation from the wide DOM.
    click(&harness, "[data-entity-column-chooser]").await;
    click(&harness, "[role='menu'] [data-entity-column='contact']").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    // Nested in an object rather than asserted as the eval's own bare
    // top-level result: chromiumoxide's `EvaluationResult::into_value`
    // deserializes `RemoteObject.value` through `Option<serde_json::Value>`,
    // and serde's blanket `Option<T>` impl collapses a JSON `null` into
    // `None` -- so a query that legitimately evaluates to `null` at the top
    // level surfaces as "No value found" instead of `Value::Null`. Every
    // other `Value::Null` assertion in this file already nests inside an
    // object for exactly this reason; this pair didn't, and had never
    // actually run (blocked by an unrelated selector bug) until now.
    // `r##"..."##`, not `r#"..."#`: the id selector's literal `"#entity...`
    // contains the `"#` sequence, which would otherwise close the raw
    // string early (same trap as a `"#e05654"` colour code).
    let removed = eval_json(
        &harness,
        r##"(() => ({
            header: document.querySelector("#entity-table-presentation-fixture th[data-entity-column='contact']"),
            cell: document.querySelector("#entity-table-presentation-fixture td[data-entity-column='contact']"),
        }))()"##,
    )
    .await;
    assert_eq!(
        removed["header"],
        Value::Null,
        "hiding the contact column must remove its header"
    );
    assert_eq!(
        removed["cell"],
        Value::Null,
        "hiding the contact column must remove its cells"
    );
    // The chooser dropdown is still open from the hide click above --
    // toggling a `MenuCheckItem` only updates `preferences`, it never
    // touches `column_chooser_open` -- so no second trigger click is needed
    // (and would in fact TOGGLE THE DROPDOWN CLOSED, since the trigger's own
    // click handler flips `column_chooser_open` unconditionally, making the
    // very next item click land on a hidden `[role='menu']`).
    click(&harness, "[role='menu'] [data-entity-column='contact']").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        eval_json(
            &harness,
            "!!document.querySelector(\"#entity-table-presentation-fixture th[data-entity-column='contact']\")",
        )
        .await
        .as_bool()
        .unwrap_or(false),
        "re-showing the contact column must restore its header"
    );

    assert_no_browser_errors(&harness, "EntityColumn primary/secondary presentation").await;
}

/// Typed summary-row emphasis (ldui-mqb): a caller classifies each row into
/// a narrow, framework-owned `EntityRowEmphasis` -- never a class-string
/// hook -- without touching any column renderer. Proves classification is
/// keyed to a row's identity rather than its rendered position (a sort that
/// moves the totals row must not move its classification), reads
/// identically in the compact single-cell presentation (they share one
/// `<tr>`), and composes with -- rather than fights -- the selected-row
/// background painted independently by `selection`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn entity_table_row_emphasis_survives_sort_and_composes_with_selection() {
    let harness = harness_at("/components/entity-table-emphasis").await;
    begin_browser_error_capture(&harness).await;
    wait_for_selector(
        &harness,
        "#entity-emphasis-table tbody tr[data-entity-row-key]",
    )
    .await;

    // Initial classification: one row per variant, resolved purely from row
    // content. `data-entity-row-emphasis` and the framework-owned classes
    // land on the shared `<tr>`; no variant sets a background, so nothing
    // here would race the `bg-base-200` selection composes with later.
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const rows = Array.from(document.querySelectorAll('#entity-emphasis-table tbody tr[data-entity-row-key]'));
                const byKey = {};
                for (const row of rows) {
                    byKey[row.dataset.entityRowKey] = {
                        emphasis: row.dataset.entityRowEmphasis,
                        bold: row.classList.contains('font-semibold'),
                        warning: row.classList.contains('text-warning'),
                        muted: row.classList.contains('text-base-content/75'),
                        background: row.classList.contains('bg-base-200'),
                    };
                }
                return byKey;
            })()"#,
        )
        .await,
        json!({
            "emphasis-1": { "emphasis": "standard", "bold": false, "warning": false, "muted": false, "background": false },
            "emphasis-2": { "emphasis": "muted", "bold": false, "warning": false, "muted": true, "background": false },
            "emphasis-3": { "emphasis": "attention", "bold": true, "warning": true, "muted": false, "background": false },
            "emphasis-4": { "emphasis": "standard", "bold": false, "warning": false, "muted": false, "background": false },
            "emphasis-total": { "emphasis": "summary", "bold": true, "warning": false, "muted": false, "background": false },
        }),
        "each row must carry exactly the classification its own content implies"
    );

    // The Summary row's wide `<td>` cells all carry the framework's top-rule
    // accent border; a Standard row's cells do not.
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const ruled = cellSelector => Array.from(
                    document.querySelectorAll(cellSelector)
                ).every(cell => cell.classList.contains('border-t-(--border-width-accent)')
                    && cell.classList.contains('border-t-base-content'));
                return {
                    totalRuled: ruled('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-total"] td[data-entity-column]'),
                    standardRuled: ruled('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-1"] td[data-entity-column]'),
                };
            })()"#,
        )
        .await,
        json!({ "totalRuled": true, "standardRuled": false }),
        "only the Summary row's wide cells carry the totals top rule"
    );

    // Zebra composition: the fixture's table has `zebra=true`, so
    // `table-zebra` is painting alternating `tbody tr:nth-child` row
    // backgrounds via CSS the whole time these assertions have been
    // running. The Summary row's own classes/attribute -- text and border
    // only, never `background-color` -- must survive that untouched.
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const table = document.querySelector('#entity-emphasis-table table[data-entity-table-grid]');
                const row = document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-total"]');
                return {
                    zebraActive: table.classList.contains('table-zebra'),
                    emphasisAttribute: row.dataset.entityRowEmphasis,
                    bold: row.classList.contains('font-semibold'),
                };
            })()"#,
        )
        .await,
        json!({ "zebraActive": true, "emphasisAttribute": "summary", "bold": true }),
        "the Summary row's emphasis must hold while table-zebra striping is active"
    );

    // A sort on `amount` (the totals row holds the largest value) actually
    // moves the row: ascending puts it last, descending puts it first. Its
    // classification -- keyed by `row_key`, not index -- must follow it.
    click(&harness, "[data-entity-sort-column='amount']").await;
    let ascending_order = eval_json(
        &harness,
        "Array.from(document.querySelectorAll('#entity-emphasis-table tbody tr[data-entity-row-key]')).map(row => row.dataset.entityRowKey)",
    )
    .await;
    assert_eq!(
        ascending_order,
        json!([
            "emphasis-3",
            "emphasis-1",
            "emphasis-2",
            "emphasis-4",
            "emphasis-total"
        ]),
        "ascending amount must put the totals row last"
    );
    assert_eq!(
        eval_json(
            &harness,
            r#"document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-total"]').dataset.entityRowEmphasis"#,
        )
        .await,
        json!("summary"),
        "the totals row must stay classified Summary after moving to the last position"
    );

    click(&harness, "[data-entity-sort-column='amount']").await;
    let descending_order = eval_json(
        &harness,
        "Array.from(document.querySelectorAll('#entity-emphasis-table tbody tr[data-entity-row-key]')).map(row => row.dataset.entityRowKey)",
    )
    .await;
    assert_eq!(
        descending_order,
        json!([
            "emphasis-total",
            "emphasis-4",
            "emphasis-2",
            "emphasis-1",
            "emphasis-3"
        ]),
        "descending amount must put the totals row first"
    );
    assert_eq!(
        eval_json(
            &harness,
            r#"document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-total"]').dataset.entityRowEmphasis"#,
        )
        .await,
        json!("summary"),
        "the totals row must stay classified Summary after moving to the first position"
    );

    // Compact presentation: they are one shared `<tr>`, so shrinking below
    // the `lg:` breakpoint swaps which `<td>` is visible but the totals
    // row's compact wrapper cell must carry the same top-rule accent.
    harness
        .set_viewport(ViewportSize::TABLET)
        .await
        .expect("shrink to a compact-layout viewport");
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const row = document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-total"]');
                const compactCell = row.querySelector('td.lg\\:hidden');
                const wideCell = row.querySelector('td[data-entity-column="client"]');
                return {
                    compactVisible: getComputedStyle(compactCell).display !== 'none',
                    wideHidden: getComputedStyle(wideCell).display === 'none',
                    compactRuled: compactCell.classList.contains('border-t-(--border-width-accent)')
                        && compactCell.classList.contains('border-t-base-content'),
                    rowStillSummary: row.dataset.entityRowEmphasis === 'summary',
                    rowStillBold: row.classList.contains('font-semibold'),
                };
            })()"#,
        )
        .await,
        json!({
            "compactVisible": true,
            "wideHidden": true,
            "compactRuled": true,
            "rowStillSummary": true,
            "rowStillBold": true,
        }),
        "the compact summary row must carry the same emphasis treatment as the wide row"
    );
    harness
        .set_viewport(ViewportSize::SMALL)
        .await
        .expect("restore the wide-layout viewport");

    // Selection composes with emphasis rather than fighting it: selecting
    // the totals row paints the selected background alongside the emphasis
    // classes already asserted above, and neither ejects the other.
    click(
        &harness,
        "#entity-emphasis-table tbody tr[data-entity-row-key='emphasis-total'] td[data-entity-column='client']",
    )
    .await;
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const row = document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-total"]');
                return {
                    ariaSelected: row.getAttribute('aria-selected'),
                    selectedBackground: row.classList.contains('bg-base-200'),
                    stillBold: row.classList.contains('font-semibold'),
                    stillSummaryAttribute: row.dataset.entityRowEmphasis === 'summary',
                };
            })()"#,
        )
        .await,
        json!({
            "ariaSelected": "true",
            "selectedBackground": true,
            "stillBold": true,
            "stillSummaryAttribute": true,
        }),
        "selecting the totals row must add the selected background without disturbing its emphasis"
    );

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(harness.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("entity table row emphasis")
        .unwrap_or_else(|error| panic!("{error}; {}", report.summary()));
    assert_no_browser_errors(&harness, "entity table row emphasis").await;
}

/// Interactive-row hover (ldui-jdzr): a row that would receive a click/
/// keyboard handler -- `on_row_activate` or `selection` -- gets the
/// framework's light-blue semantic hover background (`--color-table-filter`,
/// the same token the column-filter row already uses) across every visible
/// cell in both the wide and compact presentations, which share one `<tr>`.
/// Non-interactive rows get no hover class at all. Hover composes with row
/// emphasis (no `EntityRowEmphasis` variant sets a background -- ldui-mqb)
/// but is dropped outright, not merely out-specificity'd, once a row is
/// selected, so the selected background stays visually dominant under the
/// pointer -- see `entity_row_hover_class`'s doc comment in
/// `src/components/entity_table/selection.rs` for the full precedence
/// rationale.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn interactive_rows_get_the_framework_hover_and_selection_suppresses_it() {
    let harness = harness_at("/components/entity-table-emphasis").await;
    // Every harness page comes up with `chromiumoxide`'s touch emulation
    // force-enabled (a bug independent of `Harness::set_viewport`), which
    // makes Chromium report `(hover: hover)`/`(pointer: fine)` false and
    // drops every `hover:`-utility rule (Tailwind wraps them in `@media
    // (hover: hover)`) even though the CDP-dispatched pointer genuinely
    // puts the element in `:hover` -- see `force_desktop_hover_media`'s doc
    // comment for the full story (ldui-jdzr).
    force_desktop_hover_media(&harness).await;
    begin_browser_error_capture(&harness).await;
    wait_for_selector(
        &harness,
        "#entity-emphasis-table tbody tr[data-entity-row-key]",
    )
    .await;

    // Structural proof, no real mouse needed: this fixture wires
    // `selection`, so every row -- including the Muted/Attention/Summary
    // emphasis rows -- is interactive and carries the hover utility class
    // (plus its forced-colors compound) before anything is selected.
    let classes_by_key = eval_json(
        &harness,
        r#"(() => {
            const rows = Array.from(document.querySelectorAll('#entity-emphasis-table tbody tr[data-entity-row-key]'));
            const byKey = {};
            for (const row of rows) {
                byKey[row.dataset.entityRowKey] = {
                    hover: row.classList.contains('hover:bg-table-filter'),
                    forcedColorsHoverBg: row.classList.contains('forced-colors:hover:bg-[Highlight]'),
                    forcedColorsHoverText: row.classList.contains('forced-colors:hover:text-[HighlightText]'),
                };
            }
            return byKey;
        })()"#,
    )
    .await;
    assert_eq!(
        classes_by_key,
        json!({
            "emphasis-1": { "hover": true, "forcedColorsHoverBg": true, "forcedColorsHoverText": true },
            "emphasis-2": { "hover": true, "forcedColorsHoverBg": true, "forcedColorsHoverText": true },
            "emphasis-3": { "hover": true, "forcedColorsHoverBg": true, "forcedColorsHoverText": true },
            "emphasis-4": { "hover": true, "forcedColorsHoverBg": true, "forcedColorsHoverText": true },
            "emphasis-total": { "hover": true, "forcedColorsHoverBg": true, "forcedColorsHoverText": true },
        }),
        "every interactive, unselected row -- including each emphasis variant -- must carry the hover utility"
    );

    // Real-pointer proof: hovering an unselected row (odd position, off
    // this fixture's zebra stripe so nothing else could tint the paint)
    // actually resolves the `<tr>`'s own background-color to the
    // table-filter token's hex, not merely a class-list token.
    move_pointer_to_svg_fraction(
        &harness,
        "#entity-emphasis-table tbody tr[data-entity-row-key='emphasis-1']",
        0.5,
        0.5,
    )
    .await;
    let hovered_unselected_bg = eval_json(
        &harness,
        r#"getComputedStyle(document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-1"]')).backgroundColor"#,
    )
    .await;
    assert_eq!(
        hovered_unselected_bg,
        json!("rgb(229, 241, 251)"),
        "hovering an interactive, unselected row must paint the table-filter light-blue background"
    );

    // Selecting the same row must suppress the hover utility outright: the
    // class disappears from the row's class list, and re-hovering it no
    // longer resolves to the table-filter blue.
    click(
        &harness,
        "#entity-emphasis-table tbody tr[data-entity-row-key='emphasis-1'] td[data-entity-column='client']",
    )
    .await;
    let selected_state = eval_json(
        &harness,
        r#"(() => {
            const row = document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-1"]');
            return {
                ariaSelected: row.getAttribute('aria-selected'),
                selectedBackground: row.classList.contains('bg-base-200'),
                hoverClassPresent: row.classList.contains('hover:bg-table-filter'),
            };
        })()"#,
    )
    .await;
    assert_eq!(
        selected_state,
        json!({ "ariaSelected": "true", "selectedBackground": true, "hoverClassPresent": false }),
        "selecting a row must drop the hover utility class, not merely lose a specificity fight"
    );
    move_pointer_to_svg_fraction(
        &harness,
        "#entity-emphasis-table tbody tr[data-entity-row-key='emphasis-1']",
        0.5,
        0.5,
    )
    .await;
    let hovered_selected_bg = eval_json(
        &harness,
        r#"getComputedStyle(document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-1"]')).backgroundColor"#,
    )
    .await;
    assert_ne!(
        hovered_selected_bg,
        json!("rgb(229, 241, 251)"),
        "a selected row must stay legible under the pointer, never repainted table-filter blue by hover"
    );

    // Compact presentation: same shared `<tr>`, so shrinking below `lg:`
    // only swaps which `<td>` is visible -- the row-level hover still
    // resolves, and neither the wide nor the compact cell carries its own
    // background that could hide it, which is the reason this is a
    // row-level class rather than per-cell styling.
    harness
        .set_viewport(ViewportSize::TABLET)
        .await
        .expect("shrink to a compact-layout viewport");
    move_pointer_to_svg_fraction(
        &harness,
        "#entity-emphasis-table tbody tr[data-entity-row-key='emphasis-2']",
        0.5,
        0.5,
    )
    .await;
    let compact_hover = eval_json(
        &harness,
        r#"(() => {
            const row = document.querySelector('#entity-emphasis-table tbody tr[data-entity-row-key="emphasis-2"]');
            const compactCell = row.querySelector('td.lg\\:hidden');
            const wideCell = row.querySelector('td[data-entity-column="client"]');
            return {
                compactVisible: getComputedStyle(compactCell).display !== 'none',
                wideHidden: getComputedStyle(wideCell).display === 'none',
                compactCellOwnBackground: getComputedStyle(compactCell).backgroundColor,
                rowBackground: getComputedStyle(row).backgroundColor,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        compact_hover["compactVisible"],
        json!(true),
        "the compact cell must be the one visible below the lg breakpoint"
    );
    assert_eq!(
        compact_hover["wideHidden"],
        json!(true),
        "the wide cell must be hidden below the lg breakpoint"
    );
    assert_eq!(
        compact_hover["compactCellOwnBackground"],
        json!("rgba(0, 0, 0, 0)"),
        "the compact cell must carry no background of its own -- the row's hover paint is what shows through"
    );
    assert_eq!(
        compact_hover["rowBackground"],
        json!("rgb(229, 241, 251)"),
        "the shared row's hover background must resolve identically in the compact presentation"
    );
    harness
        .set_viewport(ViewportSize::SMALL)
        .await
        .expect("restore the wide-layout viewport");

    // Non-interactive control: EntityTablePresentationFixture wires neither
    // `on_row_activate` nor `selection`, so its rows carry no hover utility
    // at all and hovering one leaves its background untouched.
    let presentation_harness = harness_at("/components/entity-table-presentation").await;
    begin_browser_error_capture(&presentation_harness).await;
    wait_for_selector(
        &presentation_harness,
        "#entity-table-presentation-fixture [data-entity-row-key='presentation-1']",
    )
    .await;
    let non_interactive_classes = eval_json(
        &presentation_harness,
        r#"document.querySelector('#entity-table-presentation-fixture [data-entity-row-key="presentation-1"]').classList.contains('hover:bg-table-filter')"#,
    )
    .await;
    assert_eq!(
        non_interactive_classes,
        json!(false),
        "a non-interactive row must carry no hover utility class"
    );
    move_pointer_to_svg_fraction(
        &presentation_harness,
        "#entity-table-presentation-fixture [data-entity-row-key='presentation-1']",
        0.5,
        0.5,
    )
    .await;
    let non_interactive_hover_bg = eval_json(
        &presentation_harness,
        r#"getComputedStyle(document.querySelector('#entity-table-presentation-fixture [data-entity-row-key="presentation-1"]')).backgroundColor"#,
    )
    .await;
    assert_ne!(
        non_interactive_hover_bg,
        json!("rgb(229, 241, 251)"),
        "hovering a non-interactive row must never paint the table-filter light-blue background"
    );
    assert_no_browser_errors(
        &presentation_harness,
        "entity table non-interactive hover control",
    )
    .await;

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(harness.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("entity table interactive-row hover")
        .unwrap_or_else(|error| {
            panic!(
                "{error}; {}\nviolations: {:#?}",
                report.summary(),
                report.violations
            )
        });
    assert_no_browser_errors(&harness, "entity table interactive-row hover").await;
}

/// Focused browser proof for EntityTable's page-size select identity
/// (ldui-kl55): the framework-owned rows-per-page `<select>` had no `id`/
/// `name` at all when the caller omitted `page_size_control_id`, and Office
/// satellites mounting several `EntityTable`s on one Setup page had no way
/// to tell the controls apart. `demo/src/demos/snapshot_table_page.rs`'s
/// `EntityTablePageSizeIdentityFixture` mounts two tables without an
/// override plus one with an explicit override.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn page_size_select_gets_unique_identity_without_an_override_and_honors_one() {
    let harness = harness_at("/components/entity-table-page-size-identity").await;
    wait_for_selector(
        &harness,
        "#entity-table-page-size-identity-fixture [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    let identities = eval_json(
        &harness,
        r#"(() => {
            const describe = (testid) => {
                const root = document.querySelector(`[data-testid="${testid}"]`);
                const select = root.querySelector('label select');
                const label = root.querySelector('label');
                return {
                    id: select.id,
                    name: select.name,
                    labelWrapsSelect: label != null && label.contains(select),
                    associatedLabelCount: select.labels ? select.labels.length : 0,
                };
            };
            return {
                a: describe('page-size-identity-table-a'),
                b: describe('page-size-identity-table-b'),
                c: describe('page-size-identity-table-c'),
            };
        })()"#,
    )
    .await;

    let a = &identities["a"];
    let b = &identities["b"];
    let c = &identities["c"];

    // Acceptance: without `page_size_control_id`, the select renders a
    // non-empty id AND name.
    assert!(
        a["id"].as_str().is_some_and(|id| !id.is_empty()),
        "table A's page-size select must render a non-empty id: {identities}"
    );
    assert!(
        a["name"].as_str().is_some_and(|name| !name.is_empty()),
        "table A's page-size select must render a non-empty name: {identities}"
    );
    assert_eq!(
        a["id"], a["name"],
        "the generated default must drive both id and name: {identities}"
    );

    // Acceptance: two or more tables mounted together receive unique values.
    assert_ne!(
        a["id"], b["id"],
        "two EntityTables without an override must not share a page-size select id: {identities}"
    );
    assert_ne!(
        a["name"], b["name"],
        "two EntityTables without an override must not share a page-size select name: {identities}"
    );

    // Acceptance: caller-supplied identity remains stable and honored.
    assert_eq!(
        c["id"],
        json!("page-size-identity-explicit-override"),
        "a caller-supplied page_size_control_id must be honored verbatim: {identities}"
    );
    assert_eq!(
        c["name"],
        json!("page-size-identity-explicit-override"),
        "a caller-supplied page_size_control_id must also drive name: {identities}"
    );

    // Acceptance: labels remain correctly associated (the select is nested
    // inside the visible `<label>`, so the implicit association holds
    // regardless of which id path — generated or override — is in play).
    for (name, table) in [("a", a), ("b", b), ("c", c)] {
        assert_eq!(
            table["labelWrapsSelect"],
            json!(true),
            "table {name}'s page-size select must remain inside its visible label: {identities}"
        );
        assert!(
            table["associatedLabelCount"]
                .as_u64()
                .is_some_and(|count| count >= 1),
            "table {name}'s page-size select must have at least one associated label: {identities}"
        );
    }

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(harness.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("entity-table-page-size-identity")
        .unwrap_or_else(|error| {
            panic!(
                "{error}; {}\nviolations: {:#?}",
                report.summary(),
                report.violations
            )
        });

    assert_no_browser_errors(&harness, "EntityTable page-size select identity").await;
}

/// ldui-z0n1: the rows-per-page control must render in the footer row,
/// immediately before the row-range text, and never in the top toolbar --
/// toolbar actions and the column chooser stay above the table; pagination
/// metadata (rows-per-page, row range, Previous/page/Next) stays below it.
/// This proves placement and DOM order by position/containment, not by
/// class names. The select's id/name derivation, controlled preference
/// callback, and localized copy are unchanged and are covered separately by
/// `page_size_select_gets_unique_identity_without_an_override_and_honors_one`
/// and `controlled_preferences_reorder_columns_and_compose_sort_clauses`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn rows_per_page_renders_in_footer_before_row_range_and_never_in_toolbar() {
    let harness = harness_at("/components/client-snapshot-list").await;
    wait_for_selector(&harness, "[data-entity-table-grid] tbody tr").await;
    begin_browser_error_capture(&harness).await;

    let layout = eval_json(
        &harness,
        r#"(() => {
            const root = document.querySelector('[data-entity-table]');
            const toolbar = root.querySelector('[data-entity-table-toolbar]');
            const footer = root.querySelector('[data-entity-table-footer]');
            const group = footer.firstElementChild;
            const groupChildren = Array.from(group.children);
            const label = groupChildren[0];
            const rowRangeSpan = groupChildren[1];
            const pagination = footer.lastElementChild;
            const select = label.querySelector('select');
            return {
                toolbarHasSelect: toolbar.querySelector('select') !== null,
                toolbarHasLabel: toolbar.querySelector('label') !== null,
                groupChildCount: groupChildren.length,
                labelTag: label.tagName.toLowerCase(),
                rowRangeTag: rowRangeSpan.tagName.toLowerCase(),
                labelWrapsSelect: select !== null && label.contains(select),
                rowRangeText: rowRangeSpan.textContent.trim(),
                paginationHasJoinClass: pagination.classList.contains('join'),
                paginationIsAfterGroup: footer.children[1] === pagination,
            };
        })()"#,
    )
    .await;

    assert_eq!(
        layout["toolbarHasSelect"],
        json!(false),
        "the rows-per-page select must not render in the top toolbar: {layout}"
    );
    assert_eq!(
        layout["toolbarHasLabel"],
        json!(false),
        "no rows-per-page label may remain in the top toolbar: {layout}"
    );
    assert_eq!(
        layout["groupChildCount"],
        json!(2),
        "the footer's leading group must contain exactly the rows-per-page label and the row-range text: {layout}"
    );
    assert_eq!(
        layout["labelTag"],
        json!("label"),
        "the first footer-group child must be the rows-per-page label: {layout}"
    );
    assert_eq!(
        layout["rowRangeTag"],
        json!("span"),
        "the second footer-group child must be the row-range text: {layout}"
    );
    assert_eq!(
        layout["labelWrapsSelect"],
        json!(true),
        "the rows-per-page label must still wrap its select for label[for] association: {layout}"
    );
    assert!(
        layout["rowRangeText"]
            .as_str()
            .is_some_and(|text| text.contains(" of ")),
        "the second footer-group child must be the row-range text: {layout}"
    );
    assert_eq!(
        layout["paginationHasJoinClass"],
        json!(true),
        "the footer's second top-level child must be the Pagination join: {layout}"
    );
    assert_eq!(
        layout["paginationIsAfterGroup"],
        json!(true),
        "pagination must follow the rows-per-page/row-range group in the footer: {layout}"
    );

    assert_no_browser_errors(&harness, "EntityTable footer rows-per-page placement").await;
}

/// ldui-ibjk: at a 390px viewport `EntityTable` switches to its compact
/// single-column row renderer, but the desktop `<colgroup>` (and the
/// `min-width` it drives on the scroll-region content wrapper) used to keep
/// sizing the table regardless -- the compact `<td>` is `lg:hidden`, but
/// hiding a cell does not stop its `<col>` track from claiming width. The
/// visible region therefore had to grow past the viewport and scroll
/// horizontally even though nothing was hidden past the fold. Assert the
/// compact table/row/cell and the scroll region all fit within the 390px
/// viewport with no horizontal overflow, that the compact card content and
/// its action button stay usable, and -- restoring the desktop viewport --
/// that the colgroup-driven geometry is byte-identical to before the resize.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn compact_layout_fits_the_viewport_and_desktop_colgroup_survives_unchanged() {
    let harness = harness_at("/components/client-snapshot-list").await;
    wait_for_selector(
        &harness,
        "[data-entity-table-grid] tbody tr[data-entity-row-key]",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    let geometry = || {
        let harness = &harness;
        async move {
            eval_json(
                harness,
                r#"(() => {
                    const root = document.querySelector('[data-entity-table]');
                    const table = root.querySelector('[data-entity-table-grid]');
                    const content = table.parentElement;
                    const region = content.parentElement;
                    const headerCells = Array.from(table.querySelectorAll('thead tr:first-child th'));
                    const colgroup = table.querySelector('colgroup[data-table-column-tracks="stable"]');
                    const cols = colgroup ? Array.from(colgroup.querySelectorAll('col')) : [];
                    const firstRow = table.querySelector('tbody tr[data-entity-row-key]');
                    const compactCell = firstRow.querySelector(':scope > td:first-child');
                    const claimButton = firstRow.querySelector('[data-claim-row]');
                    const box = element => element.getBoundingClientRect();
                    const claimBox = claimButton ? box(claimButton) : null;
                    return {
                        viewportWidth: document.documentElement.clientWidth,
                        pageOverflows: document.documentElement.scrollWidth > document.documentElement.clientWidth,
                        colgroupPresent: colgroup !== null,
                        colCount: cols.length,
                        headerCount: headerCells.length,
                        contentMinWidthStyle: content.style.minWidth,
                        tableHasFixedClass: table.classList.contains('table-fixed'),
                        tableWidth: box(table).width,
                        contentWidth: box(content).width,
                        regionWidth: box(region).width,
                        regionScrollWidth: region.scrollWidth,
                        regionClientWidth: region.clientWidth,
                        rowWidth: box(firstRow).width,
                        compactCellWidth: box(compactCell).width,
                        compactCellTextLength: compactCell.textContent.trim().length,
                        claimVisible: claimBox !== null && claimBox.width > 0 && claimBox.height > 0,
                        claimRight: claimBox ? claimBox.right : null,
                    };
                })()"#,
            )
            .await
        }
    };

    // Desktop baseline: the colgroup pins one `<col>` per header column and
    // the content wrapper carries the matching forced `min-width`.
    let wide = geometry().await;
    assert_eq!(
        wide["colgroupPresent"],
        json!(true),
        "desktop must keep the stable colgroup: {wide}"
    );
    assert_eq!(
        wide["colCount"], wide["headerCount"],
        "desktop colgroup must have one track per header column: {wide}"
    );
    assert_ne!(
        wide["contentMinWidthStyle"],
        json!(""),
        "desktop content wrapper must keep its forced min-width: {wide}"
    );
    assert_eq!(
        wide["tableHasFixedClass"],
        json!(true),
        "desktop table must keep table-layout: fixed: {wide}"
    );

    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport for the colgroup regression");
    tokio::time::sleep(Duration::from_millis(150)).await;

    let compact = geometry().await;
    assert_eq!(
        compact["colgroupPresent"],
        json!(false),
        "compact mode must not emit the desktop colgroup tracks: {compact}"
    );
    assert_eq!(
        compact["contentMinWidthStyle"],
        json!(""),
        "compact mode must not force the desktop content min-width: {compact}"
    );
    assert_eq!(
        compact["pageOverflows"],
        json!(false),
        "compact mode must not require horizontal page scrolling: {compact}"
    );
    let viewport_width = compact["viewportWidth"].as_f64().expect("viewport width");
    for (field, label) in [
        ("tableWidth", "table"),
        ("contentWidth", "content wrapper"),
        ("regionWidth", "scroll region"),
        ("rowWidth", "row"),
        ("compactCellWidth", "compact cell"),
    ] {
        let measured = compact[field]
            .as_f64()
            .unwrap_or_else(|| panic!("missing {field} in {compact}"));
        assert!(
            measured <= viewport_width + 0.5,
            "compact {label} width {measured} exceeds the 390px viewport ({viewport_width}): {compact}"
        );
    }
    let region_scroll_width = compact["regionScrollWidth"]
        .as_f64()
        .expect("region scrollWidth");
    let region_client_width = compact["regionClientWidth"]
        .as_f64()
        .expect("region clientWidth");
    assert!(
        region_scroll_width <= region_client_width + 0.5,
        "compact scroll region must not require horizontal scrolling: {compact}"
    );
    assert!(
        compact["compactCellTextLength"].as_u64().unwrap_or(0) > 0,
        "compact card must still render readable row content: {compact}"
    );
    assert_eq!(
        compact["claimVisible"],
        json!(true),
        "compact card's claim action must remain visible: {compact}"
    );
    assert!(
        compact["claimRight"].as_f64().unwrap_or(f64::INFINITY) <= viewport_width + 0.5,
        "compact card's claim action must stay within the viewport: {compact}"
    );

    // Restoring the desktop viewport must reproduce the exact original
    // colgroup-driven geometry -- proving the compact-mode fix left the
    // desktop path untouched.
    harness
        .set_viewport(ViewportSize::new(1280, 800))
        .await
        .expect("restore wide viewport after the compact colgroup regression");
    tokio::time::sleep(Duration::from_millis(150)).await;
    let wide_after = geometry().await;
    assert_eq!(
        wide_after, wide,
        "restoring the desktop viewport must reproduce byte-identical colgroup geometry"
    );

    assert_no_browser_errors(&harness, "EntityTable compact colgroup viewport fit").await;
}
