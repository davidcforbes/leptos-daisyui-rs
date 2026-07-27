//! Layout audit sweep (ldui-dg2) — the permanent version of the article's
//! manual "10-minute spacing audit".
//!
//! `#[ignore]`d because it needs the demo dev server running, exactly like the
//! visual and reactivity suites:
//!
//! ```text
//! cargo xtask test-layout            # orchestrated (starts the server)
//! # or manually:
//! trunk serve                        # in demo/ (npm install once first)
//! cargo test --test layout_audit_smoke -- --ignored
//! ```
//!
//! ## What is asserted, and what is only reported
//!
//! **Overlap is a hard failure.** Two visible in-flow siblings whose boxes
//! intersect is a real bug — it is the regression test for the entire class
//! where a component grows and its neighbour does not move. There is no
//! tolerance and no baseline.
//!
//! **Grid and internal-vs-external are ratcheted, not zeroed.** The audit
//! (`doc/plans/2026-07-26-spacing-audit.md`) found the source is already
//! ~97% on the 4px grid, but a *rendered* gap is the sum of margins, line
//! boxes, borders and daisyUI's own internal padding — much of which this
//! library does not control. Zeroing those on day one is not achievable;
//! letting them grow silently is what this stops. Each page carries a
//! committed ceiling, and the ceiling may only ever be lowered.
//!
//! Lower a ceiling whenever a fix drops the count. Raising one requires
//! saying why in the commit message — that is the whole point of the ratchet.

mod common;
use common::{harness_at, layout_report};

/// Pages swept, with their current violation ceilings.
///
/// `(path, max_grid, max_internal)`. Overlap is always 0.
///
/// The set mirrors the visual smoke suite's complexity tiers: simple,
/// stateful, and the layout-heavy fork additions where spacing bugs actually
/// live.
const PAGES: &[(&str, usize, usize)] = &[
    ("/components/button", 0, 0),
    ("/components/alert", 0, 0),
    ("/components/card", 0, 0),
    ("/components/tab", 0, 0),
    ("/components/data-table", 2, 0),
    ("/components/kanban", 34, 2),
];

async fn audit_page(path: &str, max_grid: usize, max_internal: usize) {
    let h = harness_at(path).await;
    let report = layout_report(&h).await;

    assert!(
        report.scanned > 0,
        "layout sweep scanned 0 elements on {path} — the page did not render"
    );

    assert!(report.overlaps.is_empty(), "{}", report.describe(path));

    assert!(
        report.grid.len() <= max_grid,
        "off-grid gaps on {path} rose to {} (ceiling {max_grid}).\n{}",
        report.grid.len(),
        report.describe(path)
    );

    assert!(
        report.internal.len() <= max_internal,
        "internal>external violations on {path} rose to {} (ceiling {max_internal}).\n{}",
        report.internal.len(),
        report.describe(path)
    );
}

macro_rules! audit_test {
    ($name:ident, $idx:expr) => {
        #[tokio::test]
        #[ignore = "needs the demo dev server (trunk serve in demo/)"]
        async fn $name() {
            let (path, g, i) = PAGES[$idx];
            audit_page(path, g, i).await;
        }
    };
}

audit_test!(button_layout_is_clean, 0);
audit_test!(alert_layout_is_clean, 1);
audit_test!(card_layout_is_clean, 2);
audit_test!(tab_layout_is_clean, 3);
audit_test!(data_table_layout_is_clean, 4);
audit_test!(kanban_layout_is_clean, 5);

/// Negative control: prove the sweep actually detects things.
///
/// A detector that reports zero because it is broken is worse than no
/// detector, because it reads as evidence. This injects a deliberate
/// overlap and a deliberate off-grid gap into a known-clean page and
/// asserts both are caught — then removes them and asserts the page goes
/// clean again.
#[tokio::test]
#[ignore = "needs the demo dev server (trunk serve in demo/)"]
async fn sweep_detects_injected_violations() {
    let h = harness_at("/components/button").await;

    let before = layout_report(&h).await;
    assert!(
        before.overlaps.is_empty(),
        "control page is not clean to begin with:\n{}",
        before.describe("/components/button")
    );

    // Two in-flow siblings deliberately pulled on top of each other, plus a
    // third sitting 7px below its neighbour (off every scale step).
    let injected: bool = h
        .page()
        .evaluate(
            r#"
            (() => {
              const host = document.createElement('div');
              host.id = 'ldui-audit-probe';
              host.innerHTML =
                '<div style="height:40px;background:#eee"></div>' +
                '<div style="height:40px;margin-top:-20px;background:#ddd"></div>' +
                '<div style="height:40px;margin-top:7px;background:#ccc"></div>';
              document.querySelector('main').appendChild(host);
              return true;
            })()
            "#,
        )
        .await
        .expect("probe injection failed")
        .into_value()
        .unwrap_or(false);
    assert!(injected, "probe was not injected");

    let dirty = layout_report(&h).await;
    assert!(
        !dirty.overlaps.is_empty(),
        "sweep missed a 20px sibling overlap — the overlap check is not working:\n{}",
        dirty.describe("probe")
    );
    assert!(
        !dirty.grid.is_empty(),
        "sweep missed a 7px off-grid gap — the grid check is not working:\n{}",
        dirty.describe("probe")
    );

    // And it goes quiet again once the probe is gone, so the checks are
    // responding to the probe rather than to something ambient.
    let removed: bool = h
        .page()
        .evaluate(
            "(() => { document.getElementById('ldui-audit-probe').remove(); return true; })()",
        )
        .await
        .expect("probe removal failed")
        .into_value()
        .unwrap_or(false);
    assert!(removed);

    let after = layout_report(&h).await;
    assert_eq!(
        after.overlaps.len(),
        before.overlaps.len(),
        "overlap count did not return to baseline after removing the probe"
    );
    assert_eq!(
        after.grid.len(),
        before.grid.len(),
        "off-grid count did not return to baseline after removing the probe"
    );
}

/// Print the full backlog across every swept page without asserting.
///
/// This is the enumeration `ldui-dg2` asks for — run it to get the list
/// `ldui-6qb` then works through, rather than hunting by eye.
#[tokio::test]
#[ignore = "reporting only; run explicitly to enumerate the spacing backlog"]
async fn report_layout_backlog() {
    let mut total = 0;
    for (path, _, _) in PAGES {
        let h = harness_at(path).await;
        let report = layout_report(&h).await;
        total += report.total();
        println!("{}", report.describe(path));
    }
    println!(
        "=== layout backlog across {} pages: {total} ===",
        PAGES.len()
    );
}
