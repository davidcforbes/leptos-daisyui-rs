//! Real-browser proof for `ServerDataTable`'s opt-in presentation tools
//! (ldui-9j16): the compact gear column chooser stays inside the viewport
//! and closes on `Escape` with focus restored, a required column can never
//! be hidden or even offered in the chooser list, the caller's toolbar
//! Export action sits beside the chooser, and the atomic
//! `on_displayed_slice` projection reflects ONLY the current server page --
//! never the fixture's full population. That last assertion is the
//! feature's whole point: a "CSV export" wired to this projection cannot
//! silently ship the wrong row count and call it complete.
//!
//! Run live and GREEN. Native evidence for the pure ordering/visibility/
//! projection functions lives in
//! `src/components/data_table/server_column_tools.rs`'s own test module;
//! this file is the DOM-level companion proof over the demo's
//! "Server-Owned Table" fixture (`#server-table`, a 57-row simulated
//! backend paged 10 rows at a time -- see `demo/src/demos/data_table.rs`).

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
        .expect("evaluate server-table column-tools fixture")
        .into_value()
        .expect("server-table column-tools expression returns JSON")
}

async fn snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#server-table');
            const chooser = root.querySelector('[data-server-column-chooser="true"]');
            const dropdown = chooser ? chooser.closest('.dropdown') : null;
            const menu = dropdown ? dropdown.querySelector('.dropdown-content') : null;
            const menuBox = menu ? menu.getBoundingClientRect() : null;
            const headerOrder = Array.from(
                root.querySelectorAll('thead tr:first-child th [data-table-sort-column]')
            ).map(btn => btn.dataset.tableSortColumn);
            return {
                chooserPresentation: chooser ? chooser.dataset.serverColumnChooserPresentation : null,
                chooserExpanded: chooser ? chooser.getAttribute('aria-expanded') : null,
                chooserOpenAttr: dropdown ? (dropdown.dataset.serverColumnChooserOpen ?? null) : null,
                menuVisible: !!menuBox && menuBox.width > 0 && menuBox.height > 0,
                withinViewport: !menuBox || (
                    menuBox.left >= 0 && menuBox.right <= document.documentElement.clientWidth
                ),
                focusedIsChooser: document.activeElement === chooser,
                bodyRows: root.querySelectorAll('tbody tr').length,
                hasEmailHeader: !!root.querySelector('thead [data-table-sort-column="email"]'),
                hasNameHeader: !!root.querySelector('thead [data-table-sort-column="name"]'),
                nameToggleExists: !!root.querySelector('[data-server-column="name"]'),
                emailToggleExists: !!root.querySelector('[data-server-column="email"]'),
                headerOrder,
            };
        })()"#,
    )
    .await
}

async fn text_of(harness: &pixelproof_web::Harness, selector: &str) -> String {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const el = document.querySelector({selector:?});
                return el ? el.textContent.trim() : null;
            }})()"#
        ),
    )
    .await
    .as_str()
    .unwrap_or_default()
    .to_owned()
}

/// `ldui-9j16` binding acceptance, end to end: the chooser opens inside the
/// viewport, a required column is unhideable and absent from the chooser
/// list entirely, hide/reorder/reset all reach the rendered DOM, `Escape`
/// closes with focus restored, the toolbar Export action sits beside the
/// chooser, and the displayed-slice projection tracks only the current page.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn column_tools_chooser_projection_and_required_column_contract() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, "#server-table tbody tr").await;
    begin_browser_error_capture(&harness).await;

    // Closed by default; the icon presentation was requested explicitly.
    let initial = snapshot(&harness).await;
    assert_eq!(initial["chooserPresentation"], json!("icon"));
    assert_eq!(initial["chooserExpanded"], json!("false"));
    assert_eq!(initial["chooserOpenAttr"], Value::Null);
    assert_eq!(initial["menuVisible"], json!(false));
    assert_eq!(initial["hasNameHeader"], json!(true));
    assert_eq!(initial["hasEmailHeader"], json!(true));
    let page_rows = initial["bodyRows"].as_u64().expect("initial page has rows");
    assert!(
        page_rows > 0 && page_rows <= 10,
        "server table is paged at 10 rows: {initial}"
    );

    // Open: stays inside the viewport, and the required "name" column is
    // not merely undisturbed -- it is never offered as a toggle at all,
    // matching EntityTable's own `.filter(|column| !column.required)`.
    click(
        &harness,
        "#server-table [data-server-column-chooser=\"true\"]",
    )
    .await;
    let opened = snapshot(&harness).await;
    assert_eq!(opened["chooserExpanded"], json!("true"));
    assert_eq!(opened["chooserOpenAttr"], json!("true"));
    assert_eq!(opened["menuVisible"], json!(true));
    assert_eq!(
        opened["withinViewport"],
        json!(true),
        "chooser menu must stay inside the viewport: {opened}"
    );
    assert_eq!(
        opened["nameToggleExists"],
        json!(false),
        "a required column must not be offered in the chooser list: {opened}"
    );
    assert_eq!(opened["emailToggleExists"], json!(true));

    // Hide the optional "email" column: it leaves the rendered header, the
    // required "name" column does not.
    click(
        &harness,
        "#server-table [data-server-column=\"email\"] [role=\"menuitemcheckbox\"]",
    )
    .await;
    let hidden = snapshot(&harness).await;
    assert_eq!(hidden["hasEmailHeader"], json!(false));
    assert_eq!(hidden["hasNameHeader"], json!(true));
    assert_eq!(
        hidden["bodyRows"], initial["bodyRows"],
        "hiding a column must not change the row count: {hidden}"
    );

    // Reorder: move "status" one step earlier and confirm the rendered
    // header order actually changed (not just the preference value).
    let before_order = hidden["headerOrder"].clone();
    click(
        &harness,
        "#server-table [data-server-column-order=\"status\"][data-server-column-move=\"earlier\"]",
    )
    .await;
    let reordered = snapshot(&harness).await;
    assert_ne!(
        reordered["headerOrder"], before_order,
        "moving a column earlier must change the rendered header order"
    );

    // Escape closes the menu and returns focus to the trigger -- same
    // contract as EntityTable's own chooser (ldui-vn81).
    harness
        .press_key_sequence(&[pixelproof_web::Key::Escape])
        .await
        .expect("dismiss the server-table chooser with Escape");
    let after_escape = snapshot(&harness).await;
    assert_eq!(after_escape["chooserExpanded"], json!("false"));
    assert_eq!(after_escape["chooserOpenAttr"], Value::Null);
    assert_eq!(after_escape["menuVisible"], json!(false));
    assert_eq!(
        after_escape["focusedIsChooser"],
        json!(true),
        "Escape must return focus to the chooser trigger: {after_escape}"
    );
    // Hiding/reordering persisted through the close.
    assert_eq!(after_escape["hasEmailHeader"], json!(false));

    // Reopen and reset: visibility and order both return to their declared
    // defaults.
    click(
        &harness,
        "#server-table [data-server-column-chooser=\"true\"]",
    )
    .await;
    click(
        &harness,
        "#server-table [data-server-column-reset=\"true\"]",
    )
    .await;
    let after_reset = snapshot(&harness).await;
    assert_eq!(
        after_reset["hasEmailHeader"],
        json!(true),
        "reset must restore a previously hidden column: {after_reset}"
    );
    assert_eq!(
        after_reset["headerOrder"], initial["headerOrder"],
        "reset must restore declared column order: {after_reset}"
    );

    // The central contract: the toolbar Export action sits beside the
    // chooser, and the atomic displayed-slice projection it reads carries
    // ONLY the current server page -- never the fixture's full 57-row
    // population. This is the assertion that makes the one-page-CSV
    // mistake impossible to ship unnoticed.
    click(&harness, "[data-testid=\"server-export-slice\"]").await;
    let export_count = text_of(&harness, "[data-testid=\"server-export-count\"]").await;
    assert_eq!(export_count, "1");
    let slice_rows: u64 = text_of(&harness, "[data-testid=\"server-displayed-slice-rows\"]")
        .await
        .parse()
        .expect("displayed-slice row count is numeric");
    let rendered_rows = after_reset["bodyRows"].as_u64().expect("rendered rows");
    assert_eq!(
        slice_rows, rendered_rows,
        "the displayed-slice projection must carry exactly the rendered page, not more"
    );
    assert!(
        slice_rows < 57,
        "the displayed slice must never grow to the full simulated population (57 rows): got {slice_rows}"
    );

    assert_no_browser_errors(&harness, "server-table column-tools chooser/projection").await;
}
