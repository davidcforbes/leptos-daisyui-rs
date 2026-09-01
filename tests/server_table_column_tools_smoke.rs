//! Real-browser proof for `ServerDataTable`'s opt-in opinionated capabilities.
//!
//! Two independent contracts share this lane because they share a demo page
//! and a release server: the presentation tools (`ldui-9j16`) below, and the
//! controlled checkbox multi-selection (`ldui-px06`) at the end of the file.
//!
//! # Presentation tools (ldui-9j16)
//!
//! The compact gear column chooser stays inside the viewport
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

// ---------------------------------------------------------------------------
// ldui-px06: controlled checkbox multi-selection over a server slice
// ---------------------------------------------------------------------------

const MULTI: &str = "#server-multi-select-table";

async fn selection_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#server-multi-select-table');
            const header = root.querySelector('[data-server-selection-toggle="slice"]');
            const status = root.querySelector('[data-server-selection]');
            const notice = root.querySelector('[data-server-selection-off-slice-notice]');
            const rowBoxes = Array.from(
                root.querySelectorAll('tbody [data-server-selection-row]')
            );
            const active = document.activeElement;
            return {
                sliceState: status ? status.dataset.serverSelectionSliceState : null,
                scope: status ? status.dataset.serverSelectionScope : null,
                offSlice: status ? status.dataset.serverSelectionOffSlice : null,
                noticeText: notice ? notice.textContent.trim() : null,
                headerChecked: header ? header.checked : null,
                headerIndeterminate: header ? header.indeterminate : null,
                headerDisabled: header ? header.disabled : null,
                headerLabel: header ? header.getAttribute('aria-label') : null,
                headerColumnName: (() => {
                    const cell = root.querySelector('[data-server-selection-header="true"]');
                    return cell ? cell.textContent.trim() : null;
                })(),
                rows: rowBoxes.map(box => ({
                    key: box.dataset.serverSelectionRow,
                    checked: box.checked,
                    blocked: box.dataset.serverSelectionBlocked ?? null,
                    ariaDisabled: box.getAttribute('aria-disabled'),
                    label: box.getAttribute('aria-label'),
                    title: box.getAttribute('title'),
                })),
                ariaSelectedRows: Array.from(root.querySelectorAll('tbody tr[data-row-key]'))
                    .filter(tr => tr.getAttribute('aria-selected') === 'true')
                    .map(tr => tr.dataset.rowKey),
                // One leading control cell per rendered row, and a matching
                // extra <col> track: alignment is part of the contract.
                leadingCells: root.querySelectorAll('tbody [data-table-leading-cell]').length,
                bodyRows: root.querySelectorAll('tbody tr[data-row-key]').length,
                colTracks: root.querySelectorAll('colgroup col').length,
                headerCells: root.querySelectorAll('thead tr:first-child th').length,
                focusedSelectionKey: active
                    ? (active.dataset ? (active.dataset.serverSelectionRow ?? null) : null)
                    : null,
            };
        })()"#,
    )
    .await
}

fn keys_of(snapshot: &Value) -> Vec<String> {
    snapshot["rows"]
        .as_array()
        .expect("row checkboxes")
        .iter()
        .map(|row| row["key"].as_str().unwrap_or_default().to_owned())
        .collect()
}

fn checked_keys(snapshot: &Value) -> Vec<String> {
    snapshot["rows"]
        .as_array()
        .expect("row checkboxes")
        .iter()
        .filter(|row| row["checked"] == json!(true))
        .map(|row| row["key"].as_str().unwrap_or_default().to_owned())
        .collect()
}

async fn focus_selector(harness: &pixelproof_web::Harness, selector: &str) {
    let _ = eval_json(
        harness,
        &format!(
            r#"(() => {{
                const el = document.querySelector({selector:?});
                if (el) {{ el.focus(); }}
                return document.activeElement === el;
            }})()"#
        ),
    )
    .await;
}

/// `ldui-px06` binding acceptance in a real browser: the header checkbox
/// means the current page and only the current page (including its
/// `indeterminate` DOM property), accepted keys for rows that are not
/// displayed survive a cursor transition without relabelling anything on the
/// new slice, a declined proposal leaves no optimistic divergence, `Space`
/// operates a row checkbox without losing its focus, a blocked row stays
/// focusable and says why, and an atomic dataset-scope change clears
/// selection.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn controlled_multi_selection_is_page_scoped_and_never_optimistic() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, &format!("{MULTI} tbody tr")).await;
    begin_browser_error_capture(&harness).await;

    // Page one: conv-1, conv-2 selectable; conv-3 archived and blocked.
    let initial = selection_snapshot(&harness).await;
    assert_eq!(initial["sliceState"], json!("none"));
    assert_eq!(initial["headerChecked"], json!(false));
    assert_eq!(initial["headerIndeterminate"], json!(false));
    assert_eq!(initial["headerDisabled"], json!(false));
    assert_eq!(initial["offSlice"], json!("0"));
    assert_eq!(initial["scope"], json!("conversations/v1"));
    assert_eq!(keys_of(&initial), vec!["conv-1", "conv-2", "conv-3"]);
    assert_eq!(
        initial["leadingCells"], initial["bodyRows"],
        "every rendered row needs exactly one leading control cell: {initial}"
    );
    assert_eq!(
        initial["colTracks"].as_u64(),
        initial["headerCells"].as_u64(),
        "the control column must have its own <col> track: {initial}"
    );
    // Copy names the page, not "all".
    let header_label = initial["headerLabel"].as_str().expect("header label");
    assert!(
        header_label.to_lowercase().contains("this page"),
        "header checkbox must name the current page: {header_label:?}"
    );
    // The blocked row is focusable (aria-disabled, not `disabled`) and its
    // reason is in the accessible name AND the tooltip.
    let blocked = initial["rows"][2].clone();
    assert_eq!(blocked["blocked"], json!("true"));
    assert_eq!(blocked["ariaDisabled"], json!("true"));
    assert!(
        blocked["label"]
            .as_str()
            .is_some_and(|label| label.contains("cannot be selected")),
        "a blocked row must say why in its accessible name: {blocked}"
    );
    assert!(blocked["title"].as_str().is_some_and(|t| !t.is_empty()));

    // One row: partial, and `indeterminate` is a DOM PROPERTY -- an attribute
    // would leave assistive tech with no partial state at all.
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-row=\"conv-1\"]"),
    )
    .await;
    let partial = selection_snapshot(&harness).await;
    assert_eq!(partial["sliceState"], json!("partial"));
    assert_eq!(partial["headerChecked"], json!(false));
    assert_eq!(partial["headerIndeterminate"], json!(true));
    assert_eq!(checked_keys(&partial), vec!["conv-1"]);
    assert_eq!(partial["ariaSelectedRows"], json!(["conv-1"]));

    // Header: covers exactly the SELECTABLE rows on this page. conv-3 is
    // blocked, so the page reads `all` with two of three rows checked --
    // a blocked row must not hold the header at `partial` forever.
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-toggle=\"slice\"]"),
    )
    .await;
    let all = selection_snapshot(&harness).await;
    assert_eq!(all["sliceState"], json!("all"));
    assert_eq!(all["headerChecked"], json!(true));
    assert_eq!(all["headerIndeterminate"], json!(false));
    assert_eq!(checked_keys(&all), vec!["conv-1", "conv-2"]);
    assert_eq!(all["offSlice"], json!("0"));

    // Cursor transition. The two accepted keys are now OFF-slice: they must
    // not relabel anything on the new page, the header must read `none`
    // rather than `partial`, and the count must be stated out loud.
    click(
        &harness,
        &format!("{MULTI} [data-server-cursor-action=\"next\"]"),
    )
    .await;
    let page_two = selection_snapshot(&harness).await;
    assert_eq!(keys_of(&page_two), vec!["conv-4", "conv-5", "conv-6"]);
    assert_eq!(
        page_two["sliceState"],
        json!("none"),
        "off-slice keys must never tint this page's header: {page_two}"
    );
    assert_eq!(page_two["headerIndeterminate"], json!(false));
    assert!(checked_keys(&page_two).is_empty());
    assert_eq!(page_two["ariaSelectedRows"], json!([]));
    assert_eq!(page_two["offSlice"], json!("2"));
    assert!(
        page_two["noticeText"]
            .as_str()
            .is_some_and(|text| text.contains('2') && text.contains("not on this page")),
        "the off-slice count must be stated, not implied: {page_two}"
    );

    // Selecting this page adds to -- never replaces -- the accepted set.
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-toggle=\"slice\"]"),
    )
    .await;
    let page_two_all = selection_snapshot(&harness).await;
    assert_eq!(page_two_all["sliceState"], json!("all"));
    assert_eq!(checked_keys(&page_two_all), vec!["conv-4", "conv-5"]);
    assert_eq!(
        page_two_all["offSlice"],
        json!("2"),
        "page one's accepted keys must still be accepted: {page_two_all}"
    );

    // Back: page one's keys survived the round trip untouched.
    click(
        &harness,
        &format!("{MULTI} [data-server-cursor-action=\"previous\"]"),
    )
    .await;
    let back = selection_snapshot(&harness).await;
    assert_eq!(keys_of(&back), vec!["conv-1", "conv-2", "conv-3"]);
    assert_eq!(back["sliceState"], json!("all"));
    assert_eq!(checked_keys(&back), vec!["conv-1", "conv-2"]);
    assert_eq!(back["offSlice"], json!("2"));

    // Keyboard: Space operates the checkbox, and the checkbox keeps focus
    // through the accepted-state change (it is keyed by business identity,
    // so a data change never re-mounts it out from under the user).
    focus_selector(
        &harness,
        &format!("{MULTI} [data-server-selection-row=\"conv-1\"]"),
    )
    .await;
    harness
        .press_key_sequence(&[pixelproof_web::Key::Space])
        .await
        .expect("Space toggles a row checkbox");
    let after_space = selection_snapshot(&harness).await;
    assert_eq!(checked_keys(&after_space), vec!["conv-2"]);
    assert_eq!(after_space["sliceState"], json!("partial"));
    assert_eq!(
        after_space["focusedSelectionKey"],
        json!("conv-1"),
        "the toggled checkbox must keep keyboard focus: {after_space}"
    );

    // Rejection: the proposal is emitted, and NOTHING moves. This is the
    // assertion that proves the checkbox is controlled rather than merely
    // reported -- a native checkbox flips itself on click, so a component
    // that does not re-assert would silently diverge here.
    let before_reject: u64 = text_of(&harness, "[data-testid=\"multi-proposal-count\"]")
        .await
        .parse()
        .expect("proposal count is numeric");
    click(&harness, "[data-testid=\"multi-accept-toggle\"]").await;
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-row=\"conv-1\"]"),
    )
    .await;
    let rejected = selection_snapshot(&harness).await;
    let after_reject: u64 = text_of(&harness, "[data-testid=\"multi-proposal-count\"]")
        .await
        .parse()
        .expect("proposal count is numeric");
    assert_eq!(
        after_reject,
        before_reject + 1,
        "a declined gesture must still emit exactly one proposal"
    );
    assert_eq!(
        checked_keys(&rejected),
        vec!["conv-2"],
        "a declined proposal must leave the DOM on accepted truth: {rejected}"
    );
    assert_eq!(rejected["sliceState"], json!("partial"));

    // A blocked row emits nothing at all, accepted or not.
    let before_blocked = after_reject;
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-row=\"conv-3\"]"),
    )
    .await;
    let after_blocked: u64 = text_of(&harness, "[data-testid=\"multi-proposal-count\"]")
        .await
        .parse()
        .expect("proposal count is numeric");
    assert_eq!(
        after_blocked, before_blocked,
        "a blocked row must not propose anything"
    );

    // An atomic dataset change: the caller moves the scope and clears the
    // accepted set together, so no key can be carried into a dataset where
    // it means something else.
    click(&harness, "[data-testid=\"multi-accept-toggle\"]").await;
    click(&harness, "[data-testid=\"multi-change-scope\"]").await;
    let rescoped = selection_snapshot(&harness).await;
    assert_eq!(rescoped["scope"], json!("conversations/v2"));
    assert_eq!(rescoped["sliceState"], json!("none"));
    assert_eq!(rescoped["offSlice"], json!("0"));
    assert!(checked_keys(&rescoped).is_empty());

    assert_no_browser_errors(&harness, "server-table controlled multi-selection").await;
}
