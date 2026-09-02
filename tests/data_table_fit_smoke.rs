//! Real-browser proof for `DataTable` column-track fit (ldui-qsqz): ten
//! columns that declare no width must fit a 1280px container exactly as the
//! table did before stable geometry landed -- no pixel `<col>` widths, no
//! horizontal overflow, the column-chooser toolbar inside the container --
//! while a table whose DECLARED widths cannot fit keeps its declared tracks
//! and scrolls inside its own `overflow-x: auto` wrapper (the affordance),
//! never spilling past the container.
//!
//! Drives the general demo app (`html_target: None`) because the fixtures
//! live on the existing `/components/data_table` route. Kept in its own
//! file/xtask step (`cargo xtask test-data-table-fit`) rather than folded
//! into `reactivity_smoke.rs`, whose check count is pinned.
mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, harness_at, wait_for_selector,
};
use pixelproof_web::ViewportSize;
use serde_json::{Value, json};

const PAGE: &str = "/components/data-table";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate data-table fit fixture")
        .into_value()
        .expect("fit expression returns JSON")
}

/// Geometry of one fixture: the container, the scroll wrapper, the table,
/// its `<col>` tracks, and the toolbar.
async fn fit(h: &pixelproof_web::Harness, testid: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('[data-testid="{testid}"]');
                const wrapper = root.querySelector('.overflow-x-auto');
                const table = root.querySelector('table[data-table-layout="stable"]');
                const cols = [...table.querySelectorAll('colgroup col')];
                // The column chooser is a daisyUI dropdown whose trigger is the
                // toolbar's only `.btn` above the table.
                const toolbar = root.querySelector('.dropdown .btn');
                const r = el => el ? el.getBoundingClientRect() : null;
                return {{
                    containerWidth: root.clientWidth,
                    wrapperClientWidth: wrapper.clientWidth,
                    wrapperScrollWidth: wrapper.scrollWidth,
                    wrapperOverflowX: getComputedStyle(wrapper).overflowX,
                    wrapperRight: r(wrapper).right,
                    containerRight: r(root).right,
                    tableWidth: r(table).width,
                    colCount: cols.length,
                    autoCols: cols.filter(c => c.dataset.tableColumnTrackAuto === 'true').length,
                    widthedCols: cols.filter(c => (c.getAttribute('style') || '').includes('width')).length,
                    toolbarInside: toolbar ? r(toolbar).right <= r(root).right + 0.5 : null,
                    lastHeaderRight: r(table.querySelector('thead th:last-child')).right,
                }};
            }})()"#
        ),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-data-table-fit)"]
async fn ten_undeclared_columns_fit_their_container_and_declared_ones_scroll() {
    let h = harness_at(PAGE).await;
    // The consumer's shape: a 1440px viewport with the table in a 1280px
    // container. The defect was a 1600px table (10 x 160px default tracks)
    // whose last two columns the host clipped.
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set the consumer-reported viewport");
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, r#"[data-testid="data-table-fit-undeclared"] table"#).await;

    let auto = fit(&h, "data-table-fit-undeclared").await;
    assert_eq!(auto["colCount"], json!(10), "{auto}");
    assert_eq!(
        auto["autoCols"],
        json!(10),
        "every undeclared column is an auto track: {auto}"
    );
    assert_eq!(
        auto["widthedCols"],
        json!(0),
        "no undeclared column paints a col width: {auto}"
    );
    assert!(
        auto["containerWidth"].as_f64().unwrap_or(0.0) <= 1280.0,
        "fixture container must be at most 1280px: {auto}"
    );
    assert!(
        auto["wrapperScrollWidth"].as_f64().unwrap_or(9e9)
            <= auto["wrapperClientWidth"].as_f64().unwrap_or(0.0) + 0.5,
        "ten undeclared columns must not overflow their wrapper: {auto}"
    );
    assert!(
        auto["tableWidth"].as_f64().unwrap_or(9e9)
            <= auto["containerWidth"].as_f64().unwrap_or(0.0) + 0.5,
        "the table must fit w-full inside the container: {auto}"
    );
    assert!(
        auto["lastHeaderRight"].as_f64().unwrap_or(9e9)
            <= auto["containerRight"].as_f64().unwrap_or(0.0) + 0.5,
        "the tenth column's header must be inside the container: {auto}"
    );
    assert_ne!(
        auto["toolbarInside"],
        json!(false),
        "the column-chooser toolbar must stay inside the container: {auto}"
    );

    // Declared widths that cannot fit keep their tracks and scroll inside the
    // wrapper -- the affordance -- instead of spilling past the container.
    let declared = fit(&h, "data-table-fit-declared").await;
    assert_eq!(declared["colCount"], json!(10), "{declared}");
    assert!(
        declared["widthedCols"].as_u64().unwrap_or(0) >= 2,
        "declared columns must keep pixel tracks: {declared}"
    );
    assert_eq!(declared["wrapperOverflowX"], json!("auto"), "{declared}");
    assert!(
        declared["wrapperScrollWidth"].as_f64().unwrap_or(0.0)
            > declared["wrapperClientWidth"].as_f64().unwrap_or(9e9),
        "declared tracks wider than the container must scroll the wrapper: {declared}"
    );
    assert!(
        declared["wrapperRight"].as_f64().unwrap_or(9e9)
            <= declared["containerRight"].as_f64().unwrap_or(0.0) + 0.5,
        "the wrapper itself must never spill past the container: {declared}"
    );

    assert_no_browser_errors(&h, "data table column-track fit").await;
}
