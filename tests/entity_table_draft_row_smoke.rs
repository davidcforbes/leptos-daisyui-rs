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

use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
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
        .unwrap_or_else(|error| {
            panic!("draft-row expression returns JSON (`{expression}`): {error}")
        })
}

async fn press_key(
    harness: &pixelproof_web::Harness,
    key: &str,
    code: &str,
    key_code: i64,
    text: Option<&str>,
) {
    let mut down = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::KeyDown)
        .key(key)
        .code(code)
        .windows_virtual_key_code(key_code)
        .native_virtual_key_code(key_code);
    if let Some(text) = text {
        down = down.text(text);
    }
    harness
        .page()
        .execute(down.build().expect("key-down params"))
        .await
        .expect("dispatch key-down");
    harness
        .page()
        .execute(
            DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::KeyUp)
                .key(key)
                .code(code)
                .windows_virtual_key_code(key_code)
                .native_virtual_key_code(key_code)
                .build()
                .expect("key-up params"),
        )
        .await
        .expect("dispatch key-up");
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

async fn press_tab(harness: &pixelproof_web::Harness) {
    press_key(harness, "Tab", "Tab", 9, None).await;
}

async fn press_escape(harness: &pixelproof_web::Harness) {
    press_key(harness, "Escape", "Escape", 27, None).await;
}

async fn press_arrow_right(harness: &pixelproof_web::Harness) {
    press_key(harness, "ArrowRight", "ArrowRight", 39, None).await;
}

async fn type_text(harness: &pixelproof_web::Harness, value: &str) {
    for character in value.chars() {
        let key = character.to_string();
        let code = if character.is_ascii_alphabetic() {
            format!("Key{}", character.to_ascii_uppercase())
        } else {
            String::new()
        };
        press_key(
            harness,
            &key,
            &code,
            i64::from(character.to_ascii_uppercase() as u8),
            Some(&key),
        )
        .await;
    }
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
                renderedRows: dataRows.map(row =>
                    ['id', 'client', 'status']
                        .map(column => row.querySelector(
                            `td[data-entity-column="${column}"]`
                        )?.textContent?.trim() ?? '')
                        .join('|')
                ),
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
                lastTarget: document
                    .querySelector('[data-testid="draft-last-target"]')
                    ?.textContent?.trim() ?? null,
                retireCount: document
                    .querySelector('[data-testid="draft-retire-count"]')
                    ?.textContent?.trim() ?? null,
                filterProposals: document
                    .querySelector('[data-testid="draft-filter-proposals"]')
                    ?.textContent?.trim() ?? null,
                toolbarClicks: document
                    .querySelector('[data-testid="draft-toolbar-clicks"]')
                    ?.textContent?.trim() ?? null,
                rowActivations: document
                    .querySelector('[data-testid="draft-row-activations"]')
                    ?.textContent?.trim() ?? null,
                selectionProposals: document
                    .querySelector('[data-testid="draft-selection-proposals"]')
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

async fn active_control(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const active = document.activeElement;
            return {
                tag: active?.tagName ?? null,
                value: active?.value ?? null,
                editInput: active?.dataset.entityEditInput ?? null,
                draftInput: active?.dataset.entityDraftInput ?? null,
                editState: active?.dataset.entityRowEditState ?? null,
                draftAdd: active?.hasAttribute('data-entity-draft-add') ?? false,
                draftSave: active?.hasAttribute('data-entity-draft-save') ?? false,
                rowKey: active?.closest('[data-entity-row-key]')
                    ?.dataset.entityRowKey ?? null,
                action: active?.closest('[data-entity-row-action]')
                    ?.dataset.entityRowAction ?? null,
                tableRegion: active?.hasAttribute('data-entity-focus-region') ?? false,
            };
        })()"#,
    )
    .await
}

/// The single live row owns keyboard focus while every competing table
/// mutation is natively inert. Escape restores a stable, target-specific stop.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-entity-draft-row)"]
async fn edit_mode_owns_focus_and_locks_table_mutations() {
    let harness = harness_at("/components/entity-table-draft-row").await;
    wait_for_selector(&harness, "#draft-optin [data-entity-row-key='office-mx-1']").await;
    begin_browser_error_capture(&harness).await;

    // Draft entry focuses the first real editor. Sequential Tab then follows
    // declared field order and lands on Save without traversing frozen rows.
    click(&harness, "#draft-optin [data-entity-draft-add]").await;
    let first = active_control(&harness).await;
    assert_eq!(
        first["editInput"],
        json!("client"),
        "draft entry focus: {first}"
    );
    assert_eq!(
        first["draftInput"],
        json!("client"),
        "draft compatibility hook: {first}"
    );
    type_text(&harness, "X").await;
    let typed = active_control(&harness).await;
    assert_eq!(
        typed["value"],
        json!("X"),
        "typing must arrive through real Chromium key events and retain focus: {typed}"
    );
    press_tab(&harness).await;
    assert_eq!(active_control(&harness).await["editInput"], json!("status"));
    press_tab(&harness).await;
    let draft_save = active_control(&harness).await;
    assert_eq!(
        draft_save["draftSave"],
        json!(true),
        "Tab lands on Save: {draft_save}"
    );
    assert_eq!(draft_save["action"], json!("inline-edit"));

    press_escape(&harness).await;
    assert_eq!(snapshot(&harness).await["phase"], json!("idle"));
    assert_eq!(active_control(&harness).await["draftAdd"], json!(true));

    // Existing-row entry uses the same focus path and freezes every competing
    // table operation, including consumer-owned descendants.
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-row-edit-state='edit']",
    )
    .await;
    let existing_focus = active_control(&harness).await;
    assert_eq!(
        existing_focus["editInput"],
        json!("client"),
        "existing entry focus: {existing_focus}"
    );
    assert_eq!(existing_focus["rowKey"], json!("office-mx-1"));

    let before = lock_state(&harness).await;
    assert_eq!(
        before["toolbarInert"],
        json!(true),
        "toolbar lock: {before}"
    );
    assert_eq!(
        before["headInert"],
        json!(true),
        "header/filter lock: {before}"
    );
    assert_eq!(before["footerInert"], json!(true), "paging lock: {before}");
    assert_eq!(
        before["liveInert"],
        json!(false),
        "live row remains active: {before}"
    );
    assert_eq!(
        before["otherInert"],
        json!(true),
        "other rows are natively inert: {before}"
    );
    assert_eq!(before["otherAriaDisabled"], json!("true"));
    assert_eq!(
        before["everyControlInsideInert"],
        json!(true),
        "all competing controls are covered: {before}"
    );
    assert_eq!(
        before["nextDisabled"],
        json!(false),
        "fixture must have a meaningful next-page action: {before}"
    );

    assert_eq!(
        eval_json(
            &harness,
            r##"(() => {
                const button = document.querySelector(
                    "#draft-optin [data-entity-row-key='office-mx-2'] [data-entity-inline-edit-host] button"
                );
                button.focus();
                return document.activeElement === button;
            })()"##,
        )
        .await,
        json!(false),
        "an inert descendant cannot take focus"
    );

    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-2'] [data-entity-inline-edit-host] [data-fixture-retire]",
    )
    .await;
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-2'] td[data-entity-column='client']",
    )
    .await;
    click(
        &harness,
        "#draft-optin [data-testid='draft-toolbar-action']",
    )
    .await;
    click(&harness, "#draft-optin [data-entity-sort-column='client']").await;
    click(
        &harness,
        "#draft-optin [data-entity-filter-control='status'][data-entity-filter-placement='header']",
    )
    .await;
    type_text(&harness, "Z").await;
    click(&harness, "#draft-optin [data-entity-column-chooser]").await;
    click(&harness, "#draft-optin [data-entity-page='next']").await;
    click(
        &harness,
        "#draft-optin [data-entity-page-size-control] select",
    )
    .await;
    click(
        &harness,
        "#draft-optin th[data-entity-column='client'] [role='separator']",
    )
    .await;
    press_arrow_right(&harness).await;

    let after = lock_state(&harness).await;
    assert_eq!(
        after, before,
        "real pointer/key attempts must not sort, filter, page, resize, select, activate, retire, or open table controls"
    );

    press_escape(&harness).await;
    assert_eq!(snapshot(&harness).await["phase"], json!("idle"));
    let restored = active_control(&harness).await;
    assert_eq!(restored["rowKey"], json!("office-mx-1"));
    assert_eq!(restored["editState"], json!("edit"));
    assert_eq!(restored["action"], json!("inline-edit"));

    // Escape cannot cancel a write after Save has crossed the consumer
    // boundary. A rejection returns to Drafting, where Escape is truthful.
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-row-edit-state='edit']",
    )
    .await;
    press_tab(&harness).await;
    press_tab(&harness).await;
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-row-edit-state='save']",
    )
    .await;
    assert_eq!(snapshot(&harness).await["phase"], json!("committing"));
    press_escape(&harness).await;
    assert_eq!(
        snapshot(&harness).await["phase"],
        json!("committing"),
        "Escape must not claim an in-flight write was cancelled"
    );
    click(&harness, "[data-testid='draft-reject']").await;
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-edit-input='client']",
    )
    .await;
    press_escape(&harness).await;
    assert_eq!(snapshot(&harness).await["phase"], json!("idle"));

    // If releasing the pending snapshot removes the edited row, recovery uses
    // the named table region rather than guessing a neighboring row.
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-row-edit-state='edit']",
    )
    .await;
    click(&harness, "[data-testid='draft-refresh-2']").await;
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-edit-input='client']",
    )
    .await;
    press_escape(&harness).await;
    assert_eq!(snapshot(&harness).await["phase"], json!("idle"));
    assert_eq!(active_control(&harness).await["tableRegion"], json!(true));

    assert_no_browser_errors(&harness, "EntityTable edit locks and keyboard focus").await;
}

async fn lock_state(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#draft-optin [data-entity-table]');
            const table = root?.querySelector('[data-entity-table-grid]');
            const live = root?.querySelector('[data-entity-row-key="office-mx-1"]');
            const other = root?.querySelector('[data-entity-row-key="office-mx-2"]');
            const sort = root?.querySelector('[data-entity-sort-column="client"]');
            const filter = root?.querySelector(
                '[data-entity-filter-control="status"][data-entity-filter-placement="header"]'
            );
            const next = root?.querySelector('[data-entity-page="next"]');
            const chooser = root?.querySelector('[data-entity-column-chooser]');
            const separator = root?.querySelector('th[data-entity-column="client"] [role="separator"]');
            const toolbarAction = root?.querySelector('[data-testid="draft-toolbar-action"]');
            const pageSize = root?.querySelector('[data-entity-page-size-control] select');
            const lockedControls = [sort, filter, next, chooser, separator, toolbarAction, pageSize];
            const read = testid => document.querySelector(`[data-testid="${testid}"]`)
                ?.textContent?.trim() ?? null;
            return {
                phase: root?.dataset.entityEditPhase ?? null,
                toolbarInert: root?.querySelector('[data-entity-table-toolbar]')
                    ?.hasAttribute('inert') ?? false,
                headInert: table?.querySelector('thead')?.hasAttribute('inert') ?? false,
                footerInert: root?.querySelector('[data-entity-table-footer]')
                    ?.hasAttribute('inert') ?? false,
                liveInert: live?.hasAttribute('inert') ?? null,
                otherInert: other?.hasAttribute('inert') ?? null,
                otherAriaDisabled: other?.getAttribute('aria-disabled') ?? null,
                everyControlInsideInert: lockedControls.every(
                    control => control !== null && control.closest('[inert]') !== null
                ),
                rowKeys: Array.from(root?.querySelectorAll('[data-entity-row-key]') ?? [])
                    .map(row => row.dataset.entityRowKey),
                sortDirection: table?.querySelector('th[data-entity-column="client"]')
                    ?.dataset.entitySortDirection ?? null,
                filterValue: filter?.value ?? null,
                pageSize: pageSize?.value ?? null,
                nextDisabled: next?.disabled ?? null,
                chooserOpen: root?.querySelector('[data-entity-column-chooser-open]') !== null,
                resizeNow: separator?.getAttribute('aria-valuenow') ?? null,
                retireCount: read('draft-retire-count'),
                filterProposals: read('draft-filter-proposals'),
                toolbarClicks: read('draft-toolbar-clicks'),
                rowActivations: read('draft-row-activations'),
                selectionProposals: read('draft-selection-proposals'),
            };
        })()"#,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-entity-draft-row)"]
async fn refreshes_wait_for_edit_completion_and_only_the_latest_is_published() {
    let harness = harness_at("/components/entity-table-draft-row").await;
    wait_for_selector(&harness, "#draft-optin [data-entity-row-key]").await;
    let before = snapshot(&harness).await["renderedRows"].clone();

    click(&harness, "#draft-optin [data-entity-draft-add]").await;
    click(&harness, "[data-testid='draft-refresh-1']").await;
    assert_eq!(snapshot(&harness).await["renderedRows"], before);
    click(&harness, "[data-testid='draft-refresh-2']").await;
    assert_eq!(snapshot(&harness).await["renderedRows"], before);

    click(&harness, "#draft-optin [data-entity-draft-cancel]").await;
    assert_eq!(
        snapshot(&harness).await["renderedRows"],
        json!(["refresh-2|Refresh 2|Generation 2"])
    );
}

async fn type_into(harness: &pixelproof_web::Harness, column: &str, value: &str) {
    type_into_selector(
        harness,
        &format!("#draft-optin [data-entity-draft-input={column:?}]"),
        value,
    )
    .await;
}

async fn type_into_selector(harness: &pixelproof_web::Harness, selector: &str, value: &str) {
    let script = format!(
        r#"(() => {{
            const input = document.querySelector(
                {selector:?}
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
        "could not find editor {selector}"
    );
}

async fn wide_structure(harness: &pixelproof_web::Harness, row_selector: &str) -> Value {
    let script = format!(
        r#"(() => {{
            const table = document.querySelector('#draft-optin [data-entity-table-grid]');
            const row = table?.querySelector({row_selector:?}) ?? null;
            const host = row?.querySelector('td[data-entity-inline-edit-host]') ?? null;
            const visibleCells = row
                ? Array.from(row.children).filter(
                    cell => getComputedStyle(cell).display === 'table-cell'
                  ).length
                : 0;
            return {{
                colTracks: table?.querySelectorAll('colgroup col').length ?? 0,
                headers: table?.querySelectorAll(
                    'thead > tr:first-child > th[data-entity-column]'
                ).length ?? 0,
                filters: table?.querySelectorAll(
                    '[data-entity-column-filter-row] > th'
                ).length ?? 0,
                visibleCells,
                hostCount: row?.querySelectorAll(
                    ':scope > td[data-entity-inline-edit-host]'
                ).length ?? 0,
                hostActions: host
                    ? Array.from(host.querySelectorAll('[data-entity-row-action]'))
                        .map(action => action.dataset.entityRowAction)
                    : [],
                hasRetire: host?.querySelector('[data-fixture-retire]') !== null,
                hasEdit: host?.querySelector('[data-entity-row-edit-state="edit"]') !== null,
                hasSave: host?.querySelector('[data-entity-row-edit-state="save"], [data-entity-draft-save]') !== null,
                hasCancel: host?.querySelector('[data-entity-row-cancel], [data-entity-draft-cancel]') !== null,
            }};
        }})()"#
    );
    eval_json(harness, &script).await
}

/// Existing-row editing reuses the consumer-declared action column. The
/// framework must not append a fifth wide cell or replace the keyed row node.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-entity-draft-row)"]
async fn existing_rows_edit_inside_the_declared_action_host() {
    let harness = harness_at("/components/entity-table-draft-row").await;
    wait_for_selector(&harness, "#draft-optin [data-entity-row-key='office-mx-1']").await;
    begin_browser_error_capture(&harness).await;

    let idle = wide_structure(&harness, "[data-entity-row-key='office-mx-1']").await;
    assert_eq!(
        idle["colTracks"],
        json!(4),
        "fixture declares four columns: {idle}"
    );
    assert_eq!(
        idle["headers"],
        json!(4),
        "header tracks must match: {idle}"
    );
    assert_eq!(
        idle["filters"],
        json!(4),
        "filter tracks must match: {idle}"
    );
    assert_eq!(
        idle["visibleCells"],
        json!(4),
        "no synthetic cell is allowed: {idle}"
    );
    assert_eq!(
        idle["hostCount"],
        json!(1),
        "one declared host cell: {idle}"
    );
    assert_eq!(
        idle["hostActions"],
        json!(["retire", "inline-edit"]),
        "consumer and framework actions share the declared host: {idle}"
    );
    assert_eq!(idle["hasRetire"], json!(true));
    assert_eq!(idle["hasEdit"], json!(true));

    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-inline-edit-host] [data-fixture-retire='office-mx-1']",
    )
    .await;
    assert_eq!(snapshot(&harness).await["retireCount"], json!("1"));

    assert_eq!(
        eval_json(
            &harness,
            r##"(() => {
                window.__entityEditedRow = document.querySelector(
                    "#draft-optin [data-entity-row-key='office-mx-1']"
                );
                return window.__entityEditedRow !== null;
            })()"##,
        )
        .await,
        json!(true)
    );
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-row-edit-state='edit']",
    )
    .await;

    let editing = wide_structure(&harness, "[data-entity-row-key='office-mx-1']").await;
    assert_eq!(
        editing["visibleCells"],
        json!(4),
        "edit keeps four cells: {editing}"
    );
    assert_eq!(editing["hostActions"], json!(["inline-edit"]));
    assert_eq!(
        editing["hasRetire"],
        json!(false),
        "consumer actions are inert by absence"
    );
    assert_eq!(editing["hasSave"], json!(true));
    assert_eq!(editing["hasCancel"], json!(true));
    assert_eq!(
        eval_json(
            &harness,
            r##"window.__entityEditedRow === document.querySelector(
                "#draft-optin [data-entity-row-key='office-mx-1']"
            )"##,
        )
        .await,
        json!(true),
        "editing must preserve the keyed tr node"
    );

    type_into_selector(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-edit-input='client']",
        "Edited Client",
    )
    .await;
    type_into_selector(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-edit-input='status']",
        "Reviewed",
    )
    .await;
    click(
        &harness,
        "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-row-edit-state='save']",
    )
    .await;
    let committing = snapshot(&harness).await;
    assert_eq!(committing["phase"], json!("committing"));
    assert_eq!(committing["lastTarget"], json!("existing:office-mx-1"));
    assert_eq!(committing["lastCommitted"], json!("Edited Client|Reviewed"));

    click(&harness, "[data-testid='draft-reject']").await;
    assert_eq!(snapshot(&harness).await["phase"], json!("drafting"));
    assert_eq!(
        eval_json(
            &harness,
            r##"document.querySelector(
                "#draft-optin [data-entity-row-key='office-mx-1'] [data-entity-edit-input='client']"
            )?.value"##,
        )
        .await,
        json!("Edited Client"),
        "a rejected existing-row commit keeps its typed working clone"
    );

    assert_no_browser_errors(&harness, "EntityTable existing-row inline edit").await;
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
    let draft_layout = wide_structure(&harness, "[data-entity-draft-row]").await;
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
    assert_eq!(
        draft_layout["visibleCells"],
        json!(4),
        "the draft uses the same four declared tracks: {draft_layout}"
    );
    assert_eq!(draft_layout["hostCount"], json!(1));
    assert_eq!(draft_layout["hostActions"], json!(["inline-edit"]));
    assert_eq!(draft_layout["hasSave"], json!(true));
    assert_eq!(draft_layout["hasCancel"], json!(true));

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
    assert_eq!(committing["lastTarget"], json!("draft"));
    assert_eq!(
        committing["saveDisabled"],
        json!(true),
        "Save must not fire twice while a write is in flight"
    );
    assert_eq!(committing["draftPresent"], json!(true));

    // --- Rejection keeps the user's input --------------------------------
    click(&harness, "[data-testid='draft-reject']").await;
    assert_no_browser_errors(&harness, "entity-table draft rejection callback").await;
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
