//! Real-browser proof for the `SnapshotEntityTableConfig` behavior-only
//! passthroughs (`ldui-myhh` / `ldui-5ano`): `page_reset_key`, `viewport_fit`,
//! `toolbar_actions`, `on_display_projection`, and `column_chooser_trigger`,
//! all forwarded through `SnapshotTablePage`'s internally owned `EntityTable`
//! without granting rows, dataset identity, revision, or generation.
//!
//! Run live and GREEN. The typed builder/forwarding evidence in
//! `src/patterns/snapshot_table_page.rs`'s own test module
//! (`behavior_only_builders_forward_to_typed_fields`) is the primary native
//! evidence; this file is the DOM-level companion proof. The fixture gives the
//! composite a definite 520px containing block and uses
//! `fill_parent`; this catches a broken height chain at the framework-owned
//! table-slot boundary as both a geometry failure and an Auto-page-size latch.

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
        .expect("evaluate snapshot-table controls fixture")
        .into_value()
        .expect("snapshot-table controls expression returns JSON")
}

async fn controls_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.getElementById('snapshot-controls');
            const table = document.getElementById('snapshot-controls-table');
            const entity = table?.querySelector('[data-entity-table]');
            const previousPageButton = entity?.querySelector('[data-entity-page="previous"]');
            const chooserButton = entity?.querySelector('[data-entity-column-chooser]');
            return {
                generation: root?.dataset.snapshotGeneration,
                phase: root?.dataset.snapshotPhase,
                rows: table?.querySelectorAll('[data-entity-table-grid] tbody tr').length ?? 0,
                viewportFitEnabled: entity?.dataset.entityViewportFit ?? null,
                effectivePageSize: entity?.dataset.entityEffectivePageSize ?? null,
                pageHeight: root?.clientHeight ?? 0,
                remainingHeightError: root && table
                    ? Math.abs(root.getBoundingClientRect().bottom - table.getBoundingClientRect().bottom)
                    : null,
                tableSlotHeight: table?.clientHeight ?? 0,
                tableSlotFlexGrow: table ? getComputedStyle(table).flexGrow : null,
                onFirstPage: previousPageButton ? previousPageButton.disabled : null,
                chooserPresentation: chooserButton?.dataset.entityColumnChooserPresentation ?? null,
                chooserOpen: chooserButton
                    ?.closest('.dropdown')
                    ?.dataset.entityColumnChooserOpen === 'true',
                exportClicks: document.querySelector(
                    '[data-testid="controls-export-clicks"]'
                )?.textContent?.trim() ?? null,
                exportOutput: document.querySelector(
                    '[data-testid="controls-export-output"]'
                )?.textContent ?? null,
            };
        })()"#,
    )
    .await
}

/// `ldui-myhh` / `ldui-5ano`, binding acceptance: local filter reset from a
/// later page, adaptive height (`viewport_fit`), the icon chooser opening
/// visibly, export receiving the authoritative rendered projection, and no
/// dataset-identity drift across any of it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn behavior_only_passthroughs_forward_without_identity_drift() {
    let harness = harness_at("/components/snapshot-table-page-controls").await;
    wait_for_selector(
        &harness,
        "#snapshot-controls-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    let initial = controls_snapshot(&harness).await;
    // `fill_parent` is forwarded onto the internally owned `EntityTable`, and
    // the framework-owned slot carries the definite remaining-height budget.
    assert_eq!(initial["viewportFitEnabled"], json!("true"));
    assert_eq!(initial["pageHeight"], json!(520));
    assert_eq!(initial["tableSlotFlexGrow"], json!("1"));
    assert!(
        initial["remainingHeightError"].as_f64().unwrap() <= 1.0,
        "table slot must reach the bottom of its allocated page: {initial}"
    );
    assert!(
        initial["tableSlotHeight"].as_u64().unwrap_or_default() > 0,
        "table slot did not receive a remaining-height budget: {initial}"
    );
    // 8 rows and a short height budget: the table starts short of showing
    // every row on one page, so the filter-reset proof below starts from a
    // genuine later page rather than an already-page-one table.
    let effective_page_size = initial["effectivePageSize"]
        .as_str()
        .and_then(|value| value.parse::<usize>().ok())
        .expect("effective page size is numeric");
    assert!(
        effective_page_size < 8,
        "expected the short viewport-fit budget to page before all 8 rows: {initial}"
    );
    assert_eq!(initial["onFirstPage"], json!(true));
    // `with_column_chooser_trigger(EntityColumnChooserTrigger::Icon)`.
    assert_eq!(initial["chooserPresentation"], json!("icon"));
    assert_eq!(initial["chooserOpen"], json!(false));

    // Advance to a later page before changing the local filter.
    click(&harness, "#snapshot-controls-table [data-entity-page='2']").await;
    let on_page_two = controls_snapshot(&harness).await;
    assert_eq!(on_page_two["onFirstPage"], json!(false));
    assert_eq!(on_page_two["generation"], initial["generation"]);

    // `with_page_reset_key` bound to the local filter mode: changing the
    // filter from a later page must return to page one without disturbing
    // dataset/access generation.
    click(&harness, "[data-testid='controls-filter-urgent']").await;
    let after_filter = controls_snapshot(&harness).await;
    assert_eq!(
        after_filter["onFirstPage"],
        json!(true),
        "local filter change from a later page must reset paging: {after_filter}"
    );
    assert_eq!(after_filter["rows"], json!(1));
    assert_eq!(
        after_filter["tableSlotHeight"], initial["tableSlotHeight"],
        "filtering must not resize the table's available budget"
    );
    assert_eq!(
        after_filter["generation"], initial["generation"],
        "page reset must not disturb dataset/access generation"
    );

    click(&harness, "[data-testid='controls-filter-all']").await;
    let restored = controls_snapshot(&harness).await;
    assert_eq!(
        restored["effectivePageSize"], initial["effectivePageSize"],
        "Auto rows-per-page latched onto the one filtered row: {restored}"
    );
    assert_eq!(restored["rows"], json!(effective_page_size.min(8)));
    assert_eq!(restored["generation"], initial["generation"]);

    // Icon chooser opens visibly.
    click(
        &harness,
        "#snapshot-controls-table [data-entity-column-chooser]",
    )
    .await;
    let chooser_open = controls_snapshot(&harness).await;
    assert_eq!(chooser_open["chooserOpen"], json!(true));
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const menu = document
                    .querySelector('#snapshot-controls-table [data-entity-column-chooser]')
                    ?.closest('.dropdown')
                    ?.querySelector('.dropdown-content');
                if (!menu) { return { visible: false }; }
                const rect = menu.getBoundingClientRect();
                return { visible: rect.width > 0 && rect.height > 0 };
            })()"#,
        )
        .await["visible"],
        json!(true),
        "icon chooser menu did not open visibly"
    );
    click(
        &harness,
        "#snapshot-controls-table [data-entity-column-chooser]",
    )
    .await;

    // `on_display_projection` + `with_toolbar_actions`: Export reads the
    // authoritative rendered projection, not a page-local recomputation.
    click(&harness, "[data-testid='controls-export']").await;
    let exported = controls_snapshot(&harness).await;
    assert_eq!(exported["exportClicks"], json!("1"));
    let export_output = exported["exportOutput"]
        .as_str()
        .expect("export output is text");
    assert!(
        !export_output.trim().is_empty(),
        "export must receive a non-empty authoritative projection"
    );
    // The projection is AllFiltered-scoped, so with the "all rows" filter
    // active it must reflect every source row, not just the current page.
    assert_eq!(
        export_output.split(';').count(),
        8,
        "export projection did not carry all 8 filtered rows: {export_output}"
    );

    // No dataset-identity drift and no storage I/O across the entire
    // sequence above.
    assert_eq!(
        eval_json(&harness, "Object.keys(window.localStorage).length").await,
        json!(0),
        "SnapshotEntityTableConfig behavior-only passthroughs must never perform storage I/O"
    );
    assert_eq!(exported["generation"], initial["generation"]);
    assert_eq!(exported["phase"], json!("Displaying"));

    // Break-and-revert: remove exactly the production sizing contract while
    // one row is painted. The geometry oracle must detect the content-sized
    // slot; restoring the contract must restore the original height budget.
    click(&harness, "[data-testid='controls-filter-urgent']").await;
    eval_json(
        &harness,
        r#"(() => {
        document.getElementById('snapshot-controls-table').classList.remove('min-h-0', 'flex-1');
        return true;
    })()"#,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let broken = controls_snapshot(&harness).await;
    assert!(
        broken["remainingHeightError"].as_f64().unwrap() > 1.0,
        "negative control did not catch the removed height chain: {broken}"
    );
    eval_json(
        &harness,
        r#"(() => {
        document.getElementById('snapshot-controls-table').classList.add('min-h-0', 'flex-1');
        return true;
    })()"#,
    )
    .await;
    click(&harness, "[data-testid='controls-filter-all']").await;
    let repaired = controls_snapshot(&harness).await;
    assert_eq!(repaired["tableSlotHeight"], initial["tableSlotHeight"]);
    assert_eq!(repaired["effectivePageSize"], initial["effectivePageSize"]);
    assert!(repaired["remainingHeightError"].as_f64().unwrap() <= 1.0);

    assert_no_browser_errors(
        &harness,
        "snapshot-table-page-controls behavior-only passthroughs",
    )
    .await;
}
