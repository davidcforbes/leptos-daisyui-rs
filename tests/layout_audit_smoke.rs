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
//!
//! ## Migration to the engine sweep (ldui-audit Task 7)
//!
//! This suite ran a local, hand-rolled sweep (`tests/common/layout_audit.rs`)
//! until Task 7, which moved the overlap/grid/internal rule model into the
//! shared `pixelproof-style-audit` engine (`ldui_audit::audit_page`) so the
//! desktop and web faces of the visual-quality audit share one
//! implementation. Only overlap/grid/internal are asserted here —
//! typography/shape/depth/component-drift are `style_audit_smoke`'s business
//! (that suite runs the same engine sweep with a fuller profile).

mod common;
use common::{assert_not_truncated, body_font_family, harness_at};
use ldui_audit::family;

/// The profile this suite sweeps with: token defaults pinned to the demo's
/// *real* computed body font.
///
/// This used to pass a `"__any__"` sentinel on the grounds that typography is
/// not asserted here — but the engine compares every text-bearing element's
/// computed `font-family` against the profile regardless of which families
/// the caller reads. A sentinel therefore made every element on a text-heavy
/// page a typography violation, overran the engine's 200-per-family cap, and
/// latched `truncated` on every run forever. (An empty `type_ramp` would not
/// have helped: the family comparison is independent of the ramp.)
async fn profile(h: &ldui_audit::Harness) -> ldui_audit::StyleProfile {
    ldui_audit::from_ui_tokens(body_font_family(h).await)
}

/// Pages swept, with their current violation ceilings.
///
/// `(path, max_grid, max_internal)`. Overlap is always 0.
///
/// The set mirrors the visual smoke suite's complexity tiers: simple,
/// stateful, and the layout-heavy fork additions where spacing bugs actually
/// live.
///
/// `/components/charts` (ldui-40g) is the SVG tier, and its ceilings behave
/// differently from the rest: all 23 GRID findings are distances between
/// `<text>` nodes laid out in a chart's own viewBox coordinate space and then
/// scaled by `w-full h-auto`, so they are not spacing debt anyone can work off
/// on the 4px grid — the page's own DOM markup reports nothing. The count
/// tracks the number of charts on the page rather than any defect: it was 14
/// for two fixtures, and is 23 since ldui-j0mt added a third with a secondary
/// axis, whose tick column and axis title are more `<text>` nodes. This
/// ceiling mirrors the one in `style_audit_smoke.rs` for the same page, and
/// the two must be kept in step. OVERLAP is the
/// number that earns the page its place here: sweeping it for the first time
/// caught `AreaChart` drawing its bottom y-scale label over its first x-axis
/// tick, which was fixed in `AreaChart` rather than absorbed.
const PAGES: &[(&str, usize, usize)] = &[
    ("/components/button", 0, 0),
    ("/components/alert", 0, 0),
    ("/components/card", 0, 0),
    ("/components/tab", 0, 0),
    ("/components/data-table", 2, 0),
    ("/components/kanban", 34, 2),
    // 16, was 14: the two interactive categorical LineChart fixtures
    // (ldui-9tr) add scaled-viewBox text distances — the same class of
    // non-workable finding the doc comment above describes. Overlap
    // stays 0 (an edge-tick label collision was found and fixed at the
    // source in visible_tick_indices rather than absorbed here).
    ("/components/charts", 23, 0),
];

async fn audit_page(path: &str, max_grid: usize, max_internal: usize) {
    let h = harness_at(path).await;
    let profile = profile(&h).await;
    let report = ldui_audit::audit_page(&h, &profile, &Default::default())
        .await
        .expect("audit_page");

    report.sanity().unwrap();
    // This suite governs only overlap/grid/internal, so it cannot use
    // `ldui_audit::verify` (whose ratchet demands a ceiling for *every*
    // reporting family — typography/shape/depth are `style_audit_smoke`'s
    // business). Truncation still has to be surfaced, and `verify` only notes
    // it anyway, so both suites share the same explicit assertion.
    assert_not_truncated(&report, path);

    assert_eq!(
        report.count(family::OVERLAP),
        0,
        "{}",
        report.describe(path)
    );

    assert!(
        report.count(family::GRID) <= max_grid,
        "off-grid gaps on {path} rose to {} (ceiling {max_grid}).\n{}",
        report.count(family::GRID),
        report.describe(path)
    );

    assert!(
        report.count(family::INTERNAL) <= max_internal,
        "internal>external violations on {path} rose to {} (ceiling {max_internal}).\n{}",
        report.count(family::INTERNAL),
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
audit_test!(charts_layout_is_clean, 6);

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
    let profile = profile(&h).await;

    let before = ldui_audit::audit_page(&h, &profile, &Default::default())
        .await
        .expect("audit_page");
    assert_eq!(
        before.count(family::OVERLAP),
        0,
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

    let dirty = ldui_audit::audit_page(&h, &profile, &Default::default())
        .await
        .expect("audit_page");
    assert!(
        dirty.count(family::OVERLAP) > 0,
        "sweep missed a 20px sibling overlap — the overlap check is not working:\n{}",
        dirty.describe("probe")
    );
    assert!(
        dirty.count(family::GRID) > 0,
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

    let after = ldui_audit::audit_page(&h, &profile, &Default::default())
        .await
        .expect("audit_page");
    assert_eq!(
        after.count(family::OVERLAP),
        before.count(family::OVERLAP),
        "overlap count did not return to baseline after removing the probe"
    );
    assert_eq!(
        after.count(family::GRID),
        before.count(family::GRID),
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
        let profile = profile(&h).await;
        let report = ldui_audit::audit_page(&h, &profile, &Default::default())
            .await
            .expect("audit_page");
        total += report.count(family::OVERLAP)
            + report.count(family::GRID)
            + report.count(family::INTERNAL);
        println!("{}", report.describe(path));
    }
    println!(
        "=== layout backlog across {} pages: {total} ===",
        PAGES.len()
    );
}
