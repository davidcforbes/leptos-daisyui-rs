//! Real-browser proof for the typed `SnapshotTablePage` composition contract.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .expect("evaluate snapshot-table fixture")
        .into_value()
        .expect("snapshot-table expression returns JSON")
}

async fn contract_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.getElementById('snapshot-page');
            const dataset = document.getElementById('snapshot-page-dataset');
            const filters = document.getElementById('snapshot-page-filters');
            const feedback = document.getElementById('snapshot-page-feedback');
            const table = document.getElementById('snapshot-page-table');
            const entity = table?.querySelector('[data-entity-table]');
            const datasetSelect = dataset?.querySelector('select');
            const pageSizeSelect = entity?.querySelector('select');
            const slots = Array.from(root?.children || [])
                .map(child => child.dataset.snapshotPageSlot)
                .filter(Boolean);
            return {
                ids: [root?.id, dataset?.id, filters?.id, feedback?.id, table?.id],
                slots,
                distinctSelector: !!dataset?.querySelector('[data-dataset-selector]')
                    && !root.querySelector('[data-page-header] [data-dataset-selector]'),
                generations: [
                    root?.dataset.snapshotGeneration,
                    dataset?.dataset.snapshotGeneration,
                    table?.dataset.snapshotGeneration,
                ],
                phase: root?.dataset.snapshotPhase,
                panel: feedback?.querySelector('[data-page-state-panel]')?.dataset.pageStatePanel ?? null,
                tablePanel: table?.querySelector('[data-page-state-panel]')?.dataset.pageStatePanel ?? null,
                rows: table?.querySelectorAll('[data-entity-table-grid] tbody tr').length ?? 0,
                sameTableNode: window.__snapshotFixtureTable === entity,
                selectedDataset: datasetSelect?.value ?? null,
                datasetSelectId: datasetSelect?.id ?? null,
                datasetSelectLabel: datasetSelect?.getAttribute('aria-label') ?? null,
                pageSizeSelectId: pageSizeSelect?.id ?? null,
                pageSizeSelectLabel: pageSizeSelect?.getAttribute('aria-label') ?? null,
                firstClient: table?.querySelector(
                    '[data-entity-table-grid] tbody tr td:nth-of-type(2)'
                )?.textContent?.trim() ?? null,
            };
        })()"#,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn typed_snapshot_page_preserves_order_identity_and_retained_table_node() {
    let harness = harness_at("/components/snapshot-table-page").await;
    wait_for_selector(
        &harness,
        "#snapshot-page-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                window.__snapshotFixtureTable = document.querySelector(
                    '#snapshot-page-table [data-entity-table]'
                );
                return !!window.__snapshotFixtureTable;
            })()"#,
        )
        .await,
        json!(true)
    );

    let initial = contract_snapshot(&harness).await;
    assert_eq!(
        initial["ids"],
        json!([
            "snapshot-page",
            "snapshot-page-dataset",
            "snapshot-page-filters",
            "snapshot-page-feedback",
            "snapshot-page-table"
        ])
    );
    assert_eq!(
        initial["slots"],
        json!(["header", "dataset", "kpis", "filters", "feedback", "table"])
    );
    assert_eq!(initial["distinctSelector"], json!(true));
    assert_eq!(initial["phase"], json!("Displaying"));
    assert_eq!(initial["rows"], json!(3));
    assert_eq!(initial["sameTableNode"], json!(true));
    assert_eq!(
        initial["datasetSelectId"],
        json!("snapshot-page-dataset-select")
    );
    assert_eq!(initial["datasetSelectLabel"], json!("Office"));
    assert_eq!(
        initial["pageSizeSelectId"],
        json!("snapshot-page-rows-per-page")
    );
    assert_eq!(initial["pageSizeSelectLabel"], json!("Rows per page"));
    assert!(
        initial["generations"]
            .as_array()
            .is_some_and(|values| values.windows(2).all(|pair| pair[0] == pair[1])),
        "page, selector, and table generations diverged: {initial}"
    );

    let swapped = eval_json(
        &harness,
        r#"(() => {
            const dataset = document.getElementById('snapshot-page-dataset');
            const filters = document.getElementById('snapshot-page-filters');
            dataset.dataset.snapshotPageSlot = 'filters';
            filters.dataset.snapshotPageSlot = 'dataset';
            return Array.from(document.getElementById('snapshot-page').children)
                .map(child => child.dataset.snapshotPageSlot)
                .filter(Boolean);
        })()"#,
    )
    .await;
    assert_ne!(
        swapped,
        json!(["header", "dataset", "kpis", "filters", "feedback", "table"]),
        "slot-order oracle did not detect the injected selector/filter swap"
    );
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const dataset = document.getElementById('snapshot-page-dataset');
                const filters = document.getElementById('snapshot-page-filters');
                dataset.dataset.snapshotPageSlot = 'dataset';
                filters.dataset.snapshotPageSlot = 'filters';
                return Array.from(document.getElementById('snapshot-page').children)
                    .map(child => child.dataset.snapshotPageSlot)
                    .filter(Boolean);
            })()"#,
        )
        .await,
        json!(["header", "dataset", "kpis", "filters", "feedback", "table"]),
        "slot markers did not return to the canonical order after the negative control"
    );

    let collided_controls = eval_json(
        &harness,
        r#"(() => {
            const datasetSelect = document.querySelector('#snapshot-page-dataset select');
            const pageSizeSelect = document.querySelector('#snapshot-page-table [data-entity-table] select');
            const original = pageSizeSelect.id;
            pageSizeSelect.id = datasetSelect.id;
            const collisionDetected = datasetSelect.id === pageSizeSelect.id;
            pageSizeSelect.id = original;
            return {
                collisionDetected,
                restored: datasetSelect.id !== pageSizeSelect.id,
            };
        })()"#,
    )
    .await;
    assert_eq!(collided_controls["collisionDetected"], json!(true));
    assert_eq!(collided_controls["restored"], json!(true));

    click(&harness, "[data-testid='snapshot-filter-urgent']").await;
    let filtered = contract_snapshot(&harness).await;
    assert_eq!(filtered["rows"], json!(1));
    assert_eq!(filtered["firstClient"], json!("Mexico City Client 1"));
    assert_eq!(filtered["selectedDataset"], json!("office-mx"));
    assert_eq!(filtered["generations"], initial["generations"]);

    click(&harness, "[data-testid='snapshot-filter-none']").await;
    let no_results = contract_snapshot(&harness).await;
    assert_eq!(no_results["rows"], json!(0));
    assert_eq!(no_results["tablePanel"], json!("no-local-results"));
    assert_eq!(no_results["selectedDataset"], json!("office-mx"));
    assert_eq!(no_results["generations"], initial["generations"]);

    click(&harness, "[data-testid='snapshot-filter-all']").await;
    let restored_rows = contract_snapshot(&harness).await;
    assert_eq!(restored_rows["rows"], json!(3));
    assert_eq!(restored_rows["tablePanel"], Value::Null);
    assert_eq!(restored_rows["selectedDataset"], json!("office-mx"));
    assert_eq!(
        restored_rows["sameTableNode"],
        json!(false),
        "the no-results replacement deliberately unmounts the prior table subtree"
    );
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                window.__snapshotFixtureTable = document.querySelector(
                    '#snapshot-page-table [data-entity-table]'
                );
                return !!window.__snapshotFixtureTable;
            })()"#,
        )
        .await,
        json!(true)
    );

    click(&harness, "[data-testid='snapshot-start-replacement']").await;
    let replacing = contract_snapshot(&harness).await;
    assert_eq!(replacing["phase"], json!("Replacing"));
    assert_eq!(replacing["panel"], json!("replacing"));
    assert_eq!(replacing["selectedDataset"], json!("office-in"));
    assert_eq!(replacing["rows"], json!(3));
    assert_eq!(replacing["sameTableNode"], json!(true));
    assert_eq!(replacing["generations"], initial["generations"]);

    click(&harness, "[data-testid='snapshot-fail-replacement']").await;
    let retained_error = contract_snapshot(&harness).await;
    assert_eq!(retained_error["phase"], json!("RetainedError"));
    assert_eq!(retained_error["panel"], json!("retained-error"));
    assert_eq!(retained_error["rows"], json!(3));
    assert_eq!(retained_error["sameTableNode"], json!(true));

    click(
        &harness,
        "#snapshot-page-feedback [data-page-state-panel='retained-error'] button",
    )
    .await;
    click(&harness, "[data-testid='snapshot-complete-replacement']").await;
    let replaced = contract_snapshot(&harness).await;
    assert_eq!(replaced["phase"], json!("Displaying"));
    assert_eq!(replaced["panel"], Value::Null);
    assert_eq!(replaced["sameTableNode"], json!(true));
    assert_eq!(replaced["firstClient"], json!("New Delhi Client 1"));
    assert_ne!(replaced["generations"], initial["generations"]);
    assert!(
        replaced["generations"]
            .as_array()
            .is_some_and(|values| values.windows(2).all(|pair| pair[0] == pair[1])),
        "generation markers diverged after atomic replacement: {replaced}"
    );

    assert_no_browser_errors(&harness, "typed snapshot-table retained transitions").await;
}
