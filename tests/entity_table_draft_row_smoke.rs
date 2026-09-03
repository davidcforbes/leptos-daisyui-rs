//! Real-browser proof for `EntityTable` inline draft-row editing
//! (`ldui-ff2f`).
//!
//! Every positive assertion about the opted-in table (`#draft-optin`) is
//! paired with the same query against `#draft-plain`, which uses identical
//! columns and no `draft_row`. That table is the negative control: it must
//! render no `+`, no draft row, and no `data-entity-edit-phase` at all — so a
//! passing assertion here cannot be satisfied by something `EntityTable`
//! renders unconditionally.
//!
//! The fixture deliberately does not auto-resolve a commit. Save leaves the
//! table in `Committing` until an Accept/Reject button answers, which is the
//! only way to observe the in-flight state a synchronous fixture would skip.

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
        .expect("evaluate draft-row fixture")
        .into_value()
        .expect("draft-row expression returns JSON")
}

async fn snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const optin = document.querySelector('#draft-optin [data-entity-table]');
            const plain = document.querySelector('#draft-plain [data-entity-table]');
            const draft = optin?.querySelector('[data-entity-draft-row]') ?? null;
            const inputs = draft
                ? Array.from(draft.querySelectorAll('[data-entity-draft-input]'))
                : [];
            const dataRows = Array.from(
                optin?.querySelectorAll('[data-entity-row-key]') ?? []
            );
            return {
                phase: optin?.dataset.entityEditPhase ?? null,
                addPresent: optin?.querySelector('[data-entity-draft-add]') !== null,
                addDisabled:
                    optin?.querySelector('[data-entity-draft-add]')?.disabled ?? null,
                draftPresent: draft !== null,
                // Only columns that opted in get an editor; the derived `id`
                // column must stay read-only even inside the live row.
                editorColumns: inputs.map(i => i.dataset.entityDraftInput),
                savePresent: draft?.querySelector('[data-entity-draft-save]') !== null,
                saveDisabled:
                    draft?.querySelector('[data-entity-draft-save]')?.disabled ?? null,
                // Inert treatment: every data row while a row is live.
                dataRowCount: dataRows.length,
                inertRows: dataRows.filter(
                    r => r.getAttribute('aria-disabled') === 'true'
                ).length,
                tabbableRows: dataRows.filter(r => r.getAttribute('tabindex') === '0').length,
                commitCount: document
                    .querySelector('[data-testid="draft-commit-count"]')
                    ?.textContent?.trim() ?? null,
                lastCommitted: document
                    .querySelector('[data-testid="draft-last-committed"]')
                    ?.textContent?.trim() ?? null,
                // Negative control.
                plainPhase: plain?.dataset.entityEditPhase ?? null,
                plainAddPresent: plain?.querySelector('[data-entity-draft-add]') !== null,
                plainDraftPresent: plain?.querySelector('[data-entity-draft-row]') !== null,
                plainInertRows: Array.from(
                    plain?.querySelectorAll('[data-entity-row-key]') ?? []
                ).filter(r => r.getAttribute('aria-disabled') === 'true').length,
            };
        })()"#,
    )
    .await
}

async fn type_into(harness: &pixelproof_web::Harness, column: &str, value: &str) {
    let script = format!(
        r#"(() => {{
            const input = document.querySelector(
                '#draft-optin [data-entity-draft-input="{column}"]'
            );
            if (!input) {{ return false; }}
            const setter = Object.getOwnPropertyDescriptor(
                window.HTMLInputElement.prototype, 'value'
            ).set;
            setter.call(input, {value:?});
            input.dispatchEvent(new Event('input', {{ bubbles: true }}));
            return true;
        }})()"#
    );
    assert_eq!(
        eval_json(harness, &script).await,
        json!(true),
        "could not find the {column} editor"
    );
}

/// `ldui-ff2f` acceptance: `+` inserts an editable row, only opted-in columns
/// get editors, every other row goes inert, Save hands the consumer the typed
/// row and waits, a rejection keeps the input, and Escape leaves nothing.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-entity-draft-row)"]
async fn draft_row_edits_commit_and_cancel_without_touching_a_plain_table() {
    let harness = harness_at("/components/entity-table-draft-row").await;
    wait_for_selector(&harness, "#draft-optin [data-entity-row-key]").await;
    begin_browser_error_capture(&harness).await;

    // --- Idle -----------------------------------------------------------
    let initial = snapshot(&harness).await;
    assert_eq!(initial["phase"], json!("idle"));
    assert_eq!(initial["addPresent"], json!(true));
    assert_eq!(initial["addDisabled"], json!(false));
    assert_eq!(initial["draftPresent"], json!(false));
    assert_eq!(
        initial["inertRows"],
        json!(0),
        "nothing is inert before an edit starts: {initial}"
    );

    // Negative control: the un-opted table has none of this.
    assert_eq!(
        initial["plainPhase"],
        json!(null),
        "a table that did not opt in must not even emit the phase attribute"
    );
    assert_eq!(initial["plainAddPresent"], json!(false));
    assert_eq!(initial["plainDraftPresent"], json!(false));

    // --- `+` inserts an editable row ------------------------------------
    click(&harness, "#draft-optin [data-entity-draft-add]").await;
    let drafting = snapshot(&harness).await;
    assert_eq!(drafting["phase"], json!("drafting"));
    assert_eq!(drafting["draftPresent"], json!(true));
    assert_eq!(
        drafting["addDisabled"],
        json!(true),
        "+ must refuse a second session rather than silently doing nothing"
    );
    // Per-column opt-in: `id` declared no editor and must stay read-only.
    assert_eq!(
        drafting["editorColumns"],
        json!(["client", "status"]),
        "only columns that called .editable() may accept input: {drafting}"
    );

    // Every other row is inert and out of the tab order.
    let data_rows = drafting["dataRowCount"].as_u64().expect("row count");
    assert!(data_rows > 0, "fixture must have data rows to make inert");
    assert_eq!(drafting["inertRows"], json!(data_rows));
    assert_eq!(
        drafting["tabbableRows"],
        json!(0),
        "inert rows must leave the tab order, or Tab-to-Save walks the table"
    );
    // The negative control is unaffected by the other table's edit mode.
    assert_eq!(drafting["plainInertRows"], json!(0));
    assert_eq!(drafting["plainDraftPresent"], json!(false));

    // --- Typing, then Save ----------------------------------------------
    type_into(&harness, "client", "New Client").await;
    type_into(&harness, "status", "Urgent").await;

    click(&harness, "#draft-optin [data-entity-draft-save]").await;
    let committing = snapshot(&harness).await;
    assert_eq!(
        committing["phase"],
        json!("committing"),
        "the table must stay in flight until the consumer resolves: {committing}"
    );
    assert_eq!(committing["commitCount"], json!("1"));
    assert_eq!(
        committing["lastCommitted"],
        json!("New Client|Urgent"),
        // Break-and-revert verified 2026-09-03: replacing this expectation
        // fails the suite with `left: "New Client|Urgent"`, which proves the
        // whole chain ran -- `+` opened a session, the draft rendered,
        // editors existed for the opted-in columns, typing dispatched, the
        // reducer applied it, Save fired, and the consumer received exactly
        // those values. The assertion cannot pass vacuously.
        "the consumer receives exactly what was typed"
    );
    assert_eq!(
        committing["saveDisabled"],
        json!(true),
        "Save must not fire twice while a write is in flight"
    );
    assert_eq!(committing["draftPresent"], json!(true));

    // --- Rejection keeps the user's input --------------------------------
    click(&harness, "[data-testid='draft-reject']").await;
    let rejected = snapshot(&harness).await;
    assert_eq!(rejected["phase"], json!("drafting"));
    assert_eq!(rejected["draftPresent"], json!(true));
    assert_eq!(
        eval_json(
            &harness,
            r#"document.querySelector('#draft-optin [data-entity-draft-input="client"]').value"#
        )
        .await,
        json!("New Client"),
        "a rejected save must not cost the user their typing"
    );

    // --- Accept returns the table to read-only ---------------------------
    click(&harness, "#draft-optin [data-entity-draft-save]").await;
    click(&harness, "[data-testid='draft-accept']").await;
    let accepted = snapshot(&harness).await;
    assert_eq!(accepted["phase"], json!("idle"));
    assert_eq!(accepted["draftPresent"], json!(false));
    assert_eq!(accepted["inertRows"], json!(0), "the table is usable again");
    assert_eq!(accepted["addDisabled"], json!(false));
    assert_eq!(accepted["commitCount"], json!("2"));

    // --- Cancel leaves no phantom row -----------------------------------
    click(&harness, "#draft-optin [data-entity-draft-add]").await;
    type_into(&harness, "client", "abandoned").await;
    click(&harness, "#draft-optin [data-entity-draft-cancel]").await;
    let cancelled = snapshot(&harness).await;
    assert_eq!(cancelled["phase"], json!("idle"));
    assert_eq!(
        cancelled["draftPresent"],
        json!(false),
        "cancelling must leave no phantom row behind"
    );
    assert_eq!(
        cancelled["commitCount"],
        json!("2"),
        "cancelling must not reach the consumer at all"
    );
    assert_eq!(cancelled["inertRows"], json!(0));

    // The whole sequence never touched the un-opted table.
    assert_eq!(cancelled["plainPhase"], json!(null));
    assert_eq!(cancelled["plainAddPresent"], json!(false));
    assert_eq!(cancelled["plainDraftPresent"], json!(false));

    assert_no_browser_errors(&harness, "entity-table draft row").await;
}
