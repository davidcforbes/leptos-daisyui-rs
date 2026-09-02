//! Real-browser proof for the opt-in framework utility row on
//! `SnapshotTablePage` (`ldui-nj3q`): the localized visible/total result
//! count, one Reset, and one explicit Save as Default, obtained from the
//! opinionated composite itself rather than from a consumer-composed
//! `FilterBar`.
//!
//! Every assertion about the opted-in page (`#snapshot-actions`) is paired
//! with the same query against a second `SnapshotTablePage` on the same
//! document that does NOT opt in (`#snapshot-plain`). That page is the
//! negative control: it must keep rendering exactly as it does today, with
//! no filter bar, no count, and neither action -- so a passing assertion
//! about the count or the buttons cannot be satisfied by something the
//! composite renders unconditionally.

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
        .expect("evaluate snapshot-table filter-actions fixture")
        .into_value()
        .expect("snapshot-table filter-actions expression returns JSON")
}

async fn actions_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const describe = (element) => element === null || element === undefined
                ? null
                : {
                    tag: element.tagName,
                    type: element.getAttribute('type'),
                    role: element.getAttribute('role'),
                    text: element.textContent?.trim() ?? null,
                    ariaLabel: element.getAttribute('aria-label'),
                    disabled: element.disabled === true,
                };
            const opted = document.getElementById('snapshot-actions-filters');
            const bar = opted?.querySelector('[data-filter-bar="local"]') ?? null;
            const plain = document.getElementById('snapshot-plain-filters');
            const feedback = bar?.querySelector('[data-filter-save-feedback]') ?? null;
            return {
                barPresent: bar !== null,
                barLabel: bar?.getAttribute('aria-label') ?? null,
                resultCount: bar
                    ?.querySelector('[data-filter-result-count]')
                    ?.textContent?.trim() ?? null,
                reset: describe(bar?.querySelector('[data-filter-reset]')),
                save: describe(bar?.querySelector('[data-filter-save-default]')),
                chips: bar?.querySelectorAll('[data-filter-summary] [data-active-filters] .badge')
                    .length ?? null,
                feedbackKind: feedback?.dataset.filterSaveFeedback ?? null,
                feedbackRole: feedback?.getAttribute('role') ?? null,
                feedbackText: feedback?.textContent?.trim() ?? null,
                // Consumer content still renders inside the opted-in row.
                consumerFilterPresent:
                    opted?.querySelector('[data-testid="actions-filter-urgent"]') !== null
                    && opted?.querySelector('[data-testid="actions-filter-urgent"]') !== undefined,
                rows: document
                    .querySelectorAll('#snapshot-actions-table [data-entity-table-grid] tbody tr')
                    .length,
                resetClicks: document
                    .querySelector('[data-testid="actions-reset-clicks"]')
                    ?.textContent?.trim() ?? null,
                savedFilter: document
                    .querySelector('[data-testid="actions-saved-filter"]')
                    ?.textContent?.trim() ?? null,
                // Negative control: the composite without `filter_actions`.
                plainBarPresent: plain?.querySelector('[data-filter-bar]') !== null
                    && plain?.querySelector('[data-filter-bar]') !== undefined,
                plainResultCount: plain
                    ?.querySelector('[data-filter-result-count]')
                    ?.textContent?.trim() ?? null,
                plainReset: describe(plain?.querySelector('[data-filter-reset]')),
                plainSave: describe(plain?.querySelector('[data-filter-save-default]')),
                plainConsumerFilterPresent:
                    plain?.querySelector('[data-testid="plain-filter-all"]') !== null
                    && plain?.querySelector('[data-testid="plain-filter-all"]') !== undefined,
            };
        })()"#,
    )
    .await
}

/// `ldui-nj3q`, binding acceptance: a snapshot-table consumer obtains the
/// result count, Reset, and Save as Default from `SnapshotTablePage` alone;
/// both actions are real buttons with stable accessible names in English and
/// Spanish; and a page that does not opt in renders none of it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-snapshot-table-page-filter-actions)"]
async fn filter_actions_supply_count_reset_and_save_without_a_consumer_filter_bar() {
    let harness = harness_at("/components/snapshot-table-page-filter-actions").await;
    wait_for_selector(
        &harness,
        "#snapshot-actions-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    // --- English, unfiltered, nothing saved -----------------------------
    let initial = actions_snapshot(&harness).await;
    assert_eq!(initial["barPresent"], json!(true));
    assert_eq!(initial["barLabel"], json!("Filters"));
    assert_eq!(
        initial["resultCount"],
        json!("3 of 3 results"),
        "the composite must render the framework result count: {initial}"
    );
    assert_eq!(initial["rows"], json!(3));
    // The consumer's own `filters` content is composed inside the row, not
    // displaced by it.
    assert_eq!(initial["consumerFilterPresent"], json!(true));

    assert_eq!(initial["reset"]["tag"], json!("BUTTON"));
    assert_eq!(initial["reset"]["type"], json!("button"));
    assert_eq!(initial["reset"]["text"], json!("Reset"));
    // No active filter yet, so the framework Reset reports nothing to reset.
    assert_eq!(initial["reset"]["disabled"], json!(true));

    assert_eq!(initial["save"]["tag"], json!("BUTTON"));
    assert_eq!(initial["save"]["type"], json!("button"));
    assert_eq!(initial["save"]["text"], json!("Save as Default"));
    assert_eq!(
        initial["save"]["ariaLabel"],
        json!("Save as Default. Defaults are already saved"),
        "a clean view must say why Save is unavailable in its accessible name: {initial}"
    );
    assert_eq!(initial["save"]["disabled"], json!(true));
    assert_eq!(initial["feedbackKind"], json!(null));

    // --- Negative control: the same composite, no opt-in ----------------
    assert_eq!(
        initial["plainBarPresent"],
        json!(false),
        "a page that does not opt in must render no filter bar: {initial}"
    );
    assert_eq!(initial["plainResultCount"], json!(null));
    assert_eq!(initial["plainReset"], json!(null));
    assert_eq!(initial["plainSave"], json!(null));
    assert_eq!(
        initial["plainConsumerFilterPresent"],
        json!(true),
        "the un-opted page must still render its own filters slot content"
    );

    // --- The count tracks the identity-bound local projection -----------
    click(&harness, "[data-testid='actions-filter-urgent']").await;
    let filtered = actions_snapshot(&harness).await;
    assert_eq!(filtered["resultCount"], json!("1 of 3 results"));
    assert_eq!(filtered["rows"], json!(1));
    assert_eq!(filtered["chips"], json!(1));
    assert_eq!(
        filtered["reset"]["disabled"],
        json!(false),
        "an active filter must enable the framework Reset: {filtered}"
    );
    assert_eq!(
        filtered["plainResultCount"],
        json!(null),
        "the un-opted page must not grow a count when the opted-in one changes"
    );

    // --- Reset is a real activation, not decoration ---------------------
    click(&harness, "[data-filter-reset]").await;
    let after_reset = actions_snapshot(&harness).await;
    assert_eq!(after_reset["resetClicks"], json!("1"));
    assert_eq!(after_reset["resultCount"], json!("3 of 3 results"));
    assert_eq!(after_reset["rows"], json!(3));
    assert_eq!(after_reset["chips"], json!(0));
    assert_eq!(after_reset["reset"]["disabled"], json!(true));

    // --- Save as Default: dirty -> activation -> saved feedback ---------
    click(&harness, "[data-testid='actions-save-dirty']").await;
    let dirty = actions_snapshot(&harness).await;
    assert_eq!(dirty["save"]["disabled"], json!(false));
    assert_eq!(
        dirty["save"]["ariaLabel"],
        json!("Save as Default"),
        "a dirty view must drop the disabled reason from the accessible name: {dirty}"
    );
    assert_eq!(dirty["savedFilter"], json!("(none)"));

    click(&harness, "[data-filter-save-default]").await;
    let saved = actions_snapshot(&harness).await;
    assert!(
        saved["savedFilter"]
            .as_str()
            .expect("saved filter value is text")
            .contains("all"),
        "Save must deliver the schema-projected payload to the consumer: {saved}"
    );
    assert_eq!(saved["feedbackKind"], json!("status"));
    assert_eq!(saved["feedbackRole"], json!("status"));
    assert_eq!(saved["feedbackText"], json!("Default view saved"));
    assert_eq!(saved["save"]["disabled"], json!(true));

    // --- A rejected save is an assertive alert, still retryable ---------
    click(&harness, "[data-testid='actions-save-conflict']").await;
    let conflict = actions_snapshot(&harness).await;
    assert_eq!(conflict["feedbackKind"], json!("alert"));
    assert_eq!(conflict["feedbackRole"], json!("alert"));
    assert_eq!(
        conflict["feedbackText"],
        json!("Default view conflict: A newer default exists.")
    );
    assert_eq!(conflict["save"]["disabled"], json!(false));

    // --- Spanish: one `FilterBarTexts` localizes all of it --------------
    click(&harness, "[data-testid='actions-locale-es']").await;
    let spanish = actions_snapshot(&harness).await;
    assert_eq!(spanish["barLabel"], json!("Filtros"));
    assert_eq!(spanish["resultCount"], json!("3 de 3 resultados"));
    assert_eq!(spanish["reset"]["text"], json!("Restablecer"));
    assert_eq!(spanish["reset"]["tag"], json!("BUTTON"));
    assert_eq!(
        spanish["save"]["text"],
        json!("Guardar como predeterminado")
    );
    assert_eq!(spanish["save"]["tag"], json!("BUTTON"));
    assert_eq!(
        spanish["save"]["ariaLabel"],
        json!("Guardar como predeterminado"),
        "the Spanish conflict state keeps the bare localized name: {spanish}"
    );
    assert_eq!(
        spanish["feedbackText"],
        json!("Conflicto de vista predeterminada: A newer default exists.")
    );

    // A clean Spanish view carries the localized disabled reason too.
    click(&harness, "[data-filter-save-default]").await;
    let spanish_saved = actions_snapshot(&harness).await;
    assert_eq!(
        spanish_saved["save"]["ariaLabel"],
        json!("Guardar como predeterminado. Los valores predeterminados ya están guardados")
    );
    assert_eq!(
        spanish_saved["feedbackText"],
        json!("Vista predeterminada guardada")
    );

    // The un-opted page never grew any of it, in either language.
    assert_eq!(spanish_saved["plainBarPresent"], json!(false));
    assert_eq!(spanish_saved["plainReset"], json!(null));
    assert_eq!(spanish_saved["plainSave"], json!(null));

    // The utility row is persistence-neutral: the framework never writes.
    assert_eq!(
        eval_json(&harness, "Object.keys(window.localStorage).length").await,
        json!(0),
        "the framework utility row must never perform storage I/O"
    );

    assert_no_browser_errors(&harness, "snapshot-table-page filter actions").await;
}
