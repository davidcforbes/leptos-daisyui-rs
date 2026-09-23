//! Real-browser proof for `ServerDataTable`'s cursor-mode standard footer
//! (ldui-q14c).
//!
//! A keyset endpoint that also reports its population total and the slice's
//! offset (`ServerCursorPage::with_total_rows` / `with_position`) gets the
//! same three-region footer as offset paging (ldui-5oce): rows-per-page LEFT,
//! the pager CENTERED under the table with the one page number a cursor can
//! truthfully name, the `Showing x-y of z` range RIGHT. Without a total the
//! opaque Previous/Next fall back INSIDE the same grid, so the geometry does
//! not move when a backend stops reporting a total.
//!
//! The suite measures rendered geometry, not class names: pager and range
//! centres are compared (the row is `items-center`, so tops legitimately
//! differ), and the BREAK control is the demo's own "Forget total" toggle,
//! which drops the two fields at runtime and must collapse the range to the
//! opaque caption while leaving the pager where it was.
//!
//! Native evidence for the range derivation lives in
//! `src/components/data_table/server_component.rs`'s test module
//! (`cursor_known_range_*`); this file is the DOM-level companion over the
//! demo's "Cursor Paging With a Known Total" fixture
//! (`#cursor-known-total-table`, 120 simulated rows paged 25 at a time).

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use serde_json::{Value, json};
use std::time::Duration;

const KNOWN_TOTAL: &str = "#cursor-known-total-table";
const OPAQUE_CURSOR: &str = "#cursor-server-table";
const OFFSET: &str = "#server-table";

/// Alignment tolerance in CSS px: sub-pixel track rounding, never a region.
const ALIGNMENT_TOLERANCE: f64 = 4.0;

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .expect("evaluate server-cursor-footer fixture")
        .into_value()
        .expect("server-cursor-footer expression returns JSON")
}

/// One server table's footer: the three regions in document order, their
/// text, and where they sit relative to the table root.
async fn footer_layout(harness: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{root}');
                const footer = root.querySelector('[data-server-table-footer]');
                const controls = footer.querySelector('[data-server-table-footer-controls]');
                const label = controls.querySelector(':scope > label');
                const pager = controls.querySelector('[data-server-table-pagination]');
                const range = controls.querySelector('[data-server-row-range]');
                const join = pager.querySelector('[data-pagination]');
                const current = pager.querySelector('[data-server-cursor-page]');
                const r = el => el.getBoundingClientRect();
                const mid = rect => (rect.top + rect.bottom) / 2;
                const rootRect = r(root);
                const pagerRect = r(join);
                const rangeRect = r(range);
                return {{
                    children: Array.from(controls.children).map(el => el.tagName.toLowerCase()),
                    labelWrapsSelect: label !== null && label.querySelector('select') !== null,
                    rangeText: range.textContent.trim(),
                    rangeRole: range.getAttribute('role'),
                    pagerCenterOffset: Math.abs(
                        (pagerRect.left + pagerRect.right) / 2 - (rootRect.left + rootRect.right) / 2
                    ),
                    rangeRightOffset: Math.abs(rangeRect.right - rootRect.right),
                    labelLeftOffset: label === null ? null : Math.abs(r(label).left - rootRect.left),
                    sameRow: Math.abs(mid(rangeRect) - mid(pagerRect)) <= 1 &&
                        (label === null || Math.abs(mid(r(label)) - mid(pagerRect)) <= 1),
                    currentPage: current ? current.textContent.trim() : null,
                    currentPageAriaCurrent: current ? current.getAttribute('aria-current') : null,
                    pages: current ? current.dataset.serverCursorPages : null,
                    previousDisabled: pager.querySelector('[data-server-cursor-action="previous"]')?.disabled ?? null,
                    nextDisabled: pager.querySelector('[data-server-cursor-action="next"]')?.disabled ?? null,
                    buttons: join.querySelectorAll('button').length,
                    rows: root.querySelectorAll('tbody tr').length,
                }};
            }})()"#
        ),
    )
    .await
}

/// Polls the footer until its range text reads `expected`; a cursor
/// navigation re-runs the simulated backend on the next tick.
async fn wait_for_range_text(
    harness: &pixelproof_web::Harness,
    root: &str,
    expected: &str,
) -> Value {
    let mut last = Value::Null;
    for _ in 0..50 {
        last = footer_layout(harness, root).await;
        if last["rangeText"] == json!(expected) {
            return last;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("{root} range never read {expected:?}; last layout: {last}");
}

fn assert_standard_geometry(layout: &Value, what: &str) {
    assert_eq!(layout["sameRow"], json!(true), "{what}: {layout}");
    assert!(
        layout["pagerCenterOffset"].as_f64().unwrap_or(f64::MAX) <= ALIGNMENT_TOLERANCE,
        "{what}: the pager must be centered under the table (ldui-5oce): {layout}"
    );
    assert!(
        layout["rangeRightOffset"].as_f64().unwrap_or(f64::MAX) <= ALIGNMENT_TOLERANCE,
        "{what}: the range's right edge must align with the table's right edge (ldui-5oce): {layout}"
    );
    assert_eq!(
        layout["rangeRole"],
        json!("status"),
        "{what}: the range stays the live status region: {layout}"
    );
}

/// The bead's CHECK: with total 120 and page size 25 the known-total table
/// renders rows-per-page left, the current page number centred, and
/// `Showing 1-25 of 120` right; Next walks it to `26-50` on page 2; and the
/// BREAK control ("Forget total") collapses the range to the opaque caption
/// and drops the page slot while the pager stays exactly where it was.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo server (cargo xtask test-server-cursor-footer)"]
async fn known_total_cursor_table_renders_the_standard_footer_and_walks_the_range() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, &format!("{KNOWN_TOTAL} tbody tr")).await;
    begin_browser_error_capture(&harness).await;

    let first = wait_for_range_text(&harness, KNOWN_TOTAL, "Showing 1\u{2013}25 of 120").await;
    assert_eq!(
        first["children"],
        json!(["label", "div", "span"]),
        "rows-per-page, pager, range in that order: {first}"
    );
    assert_eq!(first["labelWrapsSelect"], json!(true), "{first}");
    assert!(
        first["labelLeftOffset"].as_f64().unwrap_or(f64::MAX) <= ALIGNMENT_TOLERANCE,
        "rows-per-page sits at the table's left edge: {first}"
    );
    assert_eq!(first["rows"], json!(25), "{first}");
    assert_eq!(first["currentPage"], json!("1"), "{first}");
    assert_eq!(first["currentPageAriaCurrent"], json!("page"), "{first}");
    assert_eq!(first["pages"], json!("5"), "{first}");
    assert_eq!(first["previousDisabled"], json!(true), "{first}");
    assert_eq!(first["nextDisabled"], json!(false), "{first}");
    assert_eq!(
        first["buttons"],
        json!(3),
        "Previous, the page slot, Next: {first}"
    );
    assert_standard_geometry(&first, "known total, first slice");
    let pager_center_at_rest = first["pagerCenterOffset"].as_f64().unwrap();

    click(
        &harness,
        &format!("{KNOWN_TOTAL} [data-server-cursor-action='next']"),
    )
    .await;
    let second = wait_for_range_text(&harness, KNOWN_TOTAL, "Showing 26\u{2013}50 of 120").await;
    assert_eq!(second["currentPage"], json!("2"), "{second}");
    assert_eq!(second["previousDisabled"], json!(false), "{second}");
    assert_eq!(second["rows"], json!(25), "{second}");
    assert_standard_geometry(&second, "known total, second slice");

    // BREAK: the backend stops reporting a total. Same query, same slice --
    // only the two optional fields are gone.
    click(&harness, "[data-testid='cursor-known-total-forget']").await;
    let forgotten = wait_for_range_text(&harness, KNOWN_TOTAL, "Showing 25 rows").await;
    assert_eq!(
        forgotten["currentPage"],
        json!(null),
        "no page slot without a total: {forgotten}"
    );
    assert_eq!(forgotten["buttons"], json!(2), "{forgotten}");
    assert_eq!(forgotten["rows"], json!(25), "{forgotten}");
    assert_eq!(
        forgotten["children"],
        json!(["label", "div", "span"]),
        "the fallback renders inside the same grid: {forgotten}"
    );
    assert_standard_geometry(&forgotten, "total forgotten");
    assert!(
        (forgotten["pagerCenterOffset"].as_f64().unwrap() - pager_center_at_rest).abs()
            <= ALIGNMENT_TOLERANCE,
        "the pager must not move when the total disappears: {forgotten}"
    );

    // Restore: the same slice regains its range and page slot.
    click(&harness, "[data-testid='cursor-known-total-forget']").await;
    let restored = wait_for_range_text(&harness, KNOWN_TOTAL, "Showing 26\u{2013}50 of 120").await;
    assert_eq!(restored["currentPage"], json!("2"), "{restored}");
    assert_eq!(restored["buttons"], json!(3), "{restored}");

    assert_no_browser_errors(&harness, "ServerDataTable cursor footer with a known total").await;
}

/// The opaque cursor table (no total) and the offset table share the footer
/// grid: pager centred, range right, on one row -- the ldui-5oce geometry is
/// a property of `ServerDataTable`, not of the paging strategy.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires release demo server (cargo xtask test-server-cursor-footer)"]
async fn opaque_cursor_and_offset_tables_share_the_footer_grid() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, &format!("{OPAQUE_CURSOR} tbody tr")).await;
    wait_for_selector(&harness, &format!("{OFFSET} tbody tr")).await;
    begin_browser_error_capture(&harness).await;

    let opaque = footer_layout(&harness, OPAQUE_CURSOR).await;
    assert_eq!(opaque["rangeText"], json!("Showing 4 rows"), "{opaque}");
    assert_eq!(opaque["currentPage"], json!(null), "{opaque}");
    assert_eq!(opaque["buttons"], json!(2), "{opaque}");
    assert_standard_geometry(&opaque, "opaque cursor table");

    let offset = footer_layout(&harness, OFFSET).await;
    assert!(
        offset["rangeText"]
            .as_str()
            .is_some_and(|text| text.contains(" of ")),
        "the offset table keeps its truthful range: {offset}"
    );
    assert_eq!(offset["currentPage"], json!(null), "{offset}");
    assert_standard_geometry(&offset, "offset table");

    assert_no_browser_errors(&harness, "ServerDataTable footer grid across strategies").await;
}
