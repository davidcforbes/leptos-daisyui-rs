//! Targeted browser proof for the typed client-snapshot list architecture.

mod common;

use common::{
    assert_no_browser_errors, assert_not_truncated, begin_browser_error_capture, body_font_family,
    click, harness_at, oracle, shift_click, shift_enter, wait_for_selector,
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
        .expect("evaluate")
        .into_value()
        .expect("JSON value")
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
            headers: Array.from(document.querySelectorAll('[data-entity-table-grid] thead th')).map(th => th.dataset.entityColumn),
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
            headers: Array.from(document.querySelectorAll('[data-entity-table-grid] thead th')).map(th => th.dataset.entityColumn),
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
    shift_click(&harness, "[data-entity-sort-column='client']").await;
    shift_click(&harness, "[data-entity-sort-column='client']").await;
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
    click(&harness, "[data-entity-page='previous']").await;

    click(&harness, "[data-entity-column-chooser]").await;
    click(&harness, "[data-entity-column='status']").await;
    let hidden_status = eval_json(
        &harness,
        r#"(() => ({
            statusHeader: Array.from(document.querySelectorAll('[data-entity-table-grid] thead th')).some(th => th.textContent.includes('Status')),
            stored: localStorage.getItem('ldui-entity-table:client-snapshot-demo'),
        }))()"#,
    )
    .await;
    assert_eq!(hidden_status["statusHeader"], json!(false));
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
        // daisyUI's five input/select shadows use oklab/oklch colours, which
        // the current engine parser cannot decode. Exact measured debt; no
        // slack. Lower this when that parser gap is fixed.
        Ceiling::new(family::DEPTH, 5),
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
