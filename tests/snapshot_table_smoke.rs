//! Real-browser proof for the typed `SnapshotTablePage` composition contract.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use pixelproof_web::ViewportSize;
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
            const header = root?.querySelector('[data-page-header]');
            const navigation = header?.querySelector('[data-page-navigation-row]');
            const heading = header?.querySelector('h1');
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
                headerLayout: header?.dataset.pageHeaderNavigationLayout ?? null,
                headingCount: header?.querySelectorAll('h1').length ?? 0,
                navigationRows: header?.querySelectorAll('[data-page-navigation-row]').length ?? 0,
                navigationLabel: navigation?.getAttribute('aria-label') ?? null,
                navigationBeforeHeading: navigation && heading
                    ? navigation.getBoundingClientRect().bottom <= heading.getBoundingClientRect().top + 1
                    : false,
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
    assert_eq!(initial["headerLayout"], json!("dedicated-row"));
    assert_eq!(initial["headingCount"], json!(1));
    assert_eq!(initial["navigationRows"], json!(1));
    assert_eq!(initial["navigationLabel"], json!("Snapshot navigation"));
    assert_eq!(initial["navigationBeforeHeading"], json!(true));
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

    harness
        .set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact page-header viewport");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let dedicated_compact = contract_snapshot(&harness).await;
    assert_eq!(dedicated_compact["headerLayout"], json!("dedicated-row"));
    assert_eq!(dedicated_compact["headingCount"], json!(1));
    assert_eq!(dedicated_compact["navigationBeforeHeading"], json!(true));
    assert_eq!(
        eval_json(
            &harness,
            r#"document.documentElement.scrollWidth <= document.documentElement.clientWidth"#,
        )
        .await,
        json!(true),
        "dedicated compact PageHeader caused horizontal page overflow"
    );

    harness
        .navigate("/components/client-snapshot-list?pp-freeze=1")
        .await
        .expect("navigate to inline PageHeader fixture");
    wait_for_selector(&harness, "[data-page-header] h1").await;
    let inline_compact = eval_json(
        &harness,
        r#"(() => {
            const header = document.querySelector('[data-page-header]');
            const back = header.querySelector('[data-testid="client-snapshot-back"]');
            const heading = header.querySelector('h1');
            return {
                layout: header.dataset.pageHeaderNavigationLayout,
                headingCount: header.querySelectorAll('h1').length,
                navigationRows: header.querySelectorAll('[data-page-navigation-row]').length,
                backBeforeHeading: back.getBoundingClientRect().bottom <= heading.getBoundingClientRect().top + 1,
                pageOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        inline_compact,
        json!({
            "layout": "inline-responsive",
            "headingCount": 1,
            "navigationRows": 0,
            "backBeforeHeading": true,
            "pageOverflow": false,
        })
    );

    harness
        .set_viewport(ViewportSize::new(1280, 800))
        .await
        .expect("restore wide page-header viewport");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let inline_wide = eval_json(
        &harness,
        r#"(() => {
            const header = document.querySelector('[data-page-header]');
            const back = header.querySelector('[data-testid="client-snapshot-back"]');
            const heading = header.querySelector('h1');
            const backRect = back.getBoundingClientRect();
            const headingRect = heading.getBoundingClientRect();
            return {
                layout: header.dataset.pageHeaderNavigationLayout,
                sameRow: Math.abs(backRect.top - headingRect.top) < Math.max(backRect.height, headingRect.height),
                backBeforeHeading: backRect.right <= headingRect.left + 1,
                headingCount: header.querySelectorAll('h1').length,
            };
        })()"#,
    )
    .await;
    assert_eq!(inline_wide["layout"], json!("inline-responsive"));
    assert_eq!(inline_wide["sameRow"], json!(true));
    assert_eq!(inline_wide["backBeforeHeading"], json!(true));
    assert_eq!(inline_wide["headingCount"], json!(1));

    assert_no_browser_errors(&harness, "typed snapshot-table retained transitions").await;
}

async fn action_feedback_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const feedback = document.getElementById('snapshot-page-feedback');
            const entry = (key) => {
                const el = feedback?.querySelector(`[data-action-feedback-key="${key}"]`);
                if (!el) { return null; }
                return {
                    state: el.dataset.actionFeedbackState,
                    text: el.querySelector('p')?.textContent ?? null,
                };
            };
            return {
                rowOne: entry('row-1'),
                rowTwo: entry('row-2'),
                rowThree: entry('row-3'),
                announcement: feedback?.querySelector('[data-action-announcement]')?.textContent ?? null,
            };
        })()"#,
    )
    .await
}

/// ldui-baz4: caller-supplied attempt-specific text (conflict reason,
/// partial-success count, retryable transport detail) renders alongside the
/// framework default, concurrent keys keep independent content, a stale
/// completion's content can never attach over a newer attempt's, and the
/// single live-region announcement always mirrors only the latest transition.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn action_feedback_attaches_attempt_specific_content_and_rejects_stale_completions() {
    let harness = harness_at("/components/snapshot-table-page").await;
    wait_for_selector(
        &harness,
        "#snapshot-page-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    // Conflict reason: caller detail appended after the localized default.
    click(&harness, "[data-testid='action-conflict']").await;
    let after_conflict = action_feedback_snapshot(&harness).await;
    assert_eq!(
        after_conflict["rowOne"]["state"],
        json!("recoverable-conflict")
    );
    assert_eq!(
        after_conflict["rowOne"]["text"],
        json!(
            "row-1: The record changed; review and retry. Another editor changed this record 2 minutes ago."
        )
    );

    // Partial-success count.
    click(&harness, "[data-testid='action-partial']").await;
    let after_partial = action_feedback_snapshot(&harness).await;
    assert_eq!(after_partial["rowOne"]["state"], json!("partial-success"));
    assert_eq!(
        after_partial["rowOne"]["text"],
        json!("row-1: The action completed only partially. 3 of 5 items updated.")
    );

    // Retryable transport detail.
    click(&harness, "[data-testid='action-retryable']").await;
    let after_retryable = action_feedback_snapshot(&harness).await;
    assert_eq!(
        after_retryable["rowOne"]["state"],
        json!("retryable-failure")
    );
    assert_eq!(
        after_retryable["rowOne"]["text"],
        json!(
            "row-1: The action failed and may be retried. Timed out contacting the service; retry."
        )
    );

    // Concurrent actions: row-1 completes while row-2 is independently Pending.
    click(&harness, "[data-testid='action-concurrent']").await;
    let concurrent = action_feedback_snapshot(&harness).await;
    assert_eq!(concurrent["rowOne"]["state"], json!("success"));
    assert_eq!(
        concurrent["rowOne"]["text"],
        json!("row-1: Action completed. Row 1 saved.")
    );
    assert_eq!(concurrent["rowTwo"]["state"], json!("pending"));
    assert_eq!(
        concurrent["rowTwo"]["text"],
        json!("row-2: Action in progress. Saving row 2\u{2026}")
    );

    // Stale completion: the superseded attempt's content must never render,
    // and the one live-region announcement must mirror only the latest
    // (row-3) transition.
    click(&harness, "[data-testid='action-stale']").await;
    let stale = action_feedback_snapshot(&harness).await;
    assert_eq!(stale["rowThree"]["state"], json!("retryable-failure"));
    let row_three_text = stale["rowThree"]["text"].as_str().expect("row-3 text");
    assert!(
        row_three_text.contains("Timed out contacting the service; retry."),
        "row-3 must show the fresh attempt's content: {row_three_text}"
    );
    assert!(
        !row_three_text.contains("STALE COMPLETION") && !row_three_text.contains("First attempt"),
        "row-3 must never show the superseded attempt's content: {row_three_text}"
    );
    let announcement = stale["announcement"].as_str().expect("announcement text");
    assert!(
        announcement.starts_with("row-3:") && announcement.contains("Timed out contacting"),
        "the live-region announcement must mirror only the latest (row-3) transition: {announcement}"
    );
    assert!(
        !announcement.contains("STALE COMPLETION") && !announcement.contains("First attempt"),
        "the announcement must never carry a superseded attempt's content: {announcement}"
    );

    assert_no_browser_errors(&harness, "ActionFeedback attempt-specific content").await;
}
