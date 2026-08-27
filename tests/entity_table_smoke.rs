//! Targeted browser proof for the typed client-snapshot list architecture.

mod common;

use common::{
    assert_no_browser_errors, assert_not_truncated, begin_browser_error_capture, body_font_family,
    click, harness_at, wait_for_selector,
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
            .is_some_and(|label| label.contains("Sort ascending")),
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
            .contains("Sort descending")
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
            .contains("Restore system order")
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
    assert!(
        hidden_status["stored"]
            .as_str()
            .is_some_and(|stored| stored.contains("status")),
        "hidden column must persist: {hidden_status}"
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
    assert!(
        resized["stored"]
            .as_str()
            .is_some_and(|value| value.contains("client")),
        "stored preferences: {resized}"
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
        (restored_width - resized_width).abs() <= 2.0,
        "width must survive remount: before={resized_width}, after={restored_width}"
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
        json!({ "pageSize": "50", "statusHeader": false })
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
    assert_eq!(compact["rows"], json!(50));
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
