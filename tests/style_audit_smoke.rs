//! Full-profile style/shape/depth + component-drift audit (Task 7): the
//! demo's real computed styles swept against the declared `ui-tokens` visual
//! system, plus the daisyUI component-drift heuristics, on a representative
//! page set. `layout_audit_smoke.rs`'s sibling — that suite owns
//! overlap/grid/internal, this one owns typography/shape/depth/component-drift
//! (the *other* families the same engine sweep reports).
//!
//! `#[ignore]`d, same convention as the sibling browser suites:
//!
//! ```text
//! cargo xtask test-style             # orchestrated (starts the server)
//! # or manually:
//! trunk serve                        # in demo/ (npm install once first)
//! cargo test --test style_audit_smoke -- --ignored
//! ```
//!
//! **Overlap is still a hard failure here too** (asserted separately, before
//! the ratchet, exactly like `layout_audit_smoke`) — `ldui_audit::verify`
//! itself refuses a ceiling for it.
//!
//! **Every other family is ratcheted, not zeroed**, including grid/internal:
//! the engine sweep reports every family under one profile in one pass, so
//! even though `layout_audit_smoke` owns the *intent* behind grid/internal,
//! this suite still has to declare a ceiling for them on pages where they
//! report — `check_ceilings` treats a reporting family with no ceiling entry
//! as an ungoverned (implicit ceiling 0) failure, by design (a new rule
//! family cannot slip through ungoverned). Those two numbers are kept in sync
//! with `layout_audit_smoke.rs`'s ceilings for the same page.
//!
//! Ceilings below are filled from the first run's actual counts — daisyUI's
//! own defaults report non-zero typography/shape/depth out of the box, which
//! is the ratchet baseline, not a blocker. Lower a ceiling whenever a fix
//! drops the count; raising one needs a reason in the commit message.

mod common;
use common::harness_at;
use ldui_audit::{Ceiling, check_ceilings, family};

/// Determine the demo's real computed `<body>` font family (first family in
/// the stack, quotes stripped) and pin it into the profile: a silent font
/// fallback (declared family never loads) is itself a regression the sweep
/// should catch, so the profile must assert the family that is *actually*
/// serving today rather than a guess.
async fn body_font_family(h: &ldui_audit::Harness) -> String {
    let raw: String = h
        .page()
        .evaluate("getComputedStyle(document.body).fontFamily")
        .await
        .expect("evaluate body font-family")
        .into_value()
        .expect("font-family computed style is a string");
    raw.split(',')
        .next()
        .unwrap_or(&raw)
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string()
}

/// Pages swept, with their current per-family violation ceilings.
///
/// `(family_name, max)`. Overlap is always 0 and asserted separately, never
/// listed here (a ceiling entry for it is itself a misconfiguration per
/// `ldui_audit::verify`'s contract, mirrored here for `check_ceilings`).
///
/// Filled from the first run's actual counts (`report_style_backlog` /
/// the `check_ceilings` failure output, which names the exact count per
/// family). SHAPE is 0 on every page today — daisyUI's border-radius
/// utilities already land on the declared radius set — so it stays listed
/// at 0 as a regression tripwire rather than an ungoverned family. DEPTH is
/// non-zero everywhere: daisyUI's `shadow-*` utilities don't match the
/// declared `ui_tokens::elevation` set, which is real debt this ratchet now
/// tracks rather than hides. GRID/INTERNAL mirror `layout_audit_smoke.rs`'s
/// committed ceilings for the same page (see the module doc above).
const PAGES: &[(&str, &[(&str, usize)])] = &[
    (
        "/components/button",
        &[
            (family::TYPOGRAPHY, 3),
            (family::SHAPE, 0),
            (family::DEPTH, 36),
        ],
    ),
    (
        "/components/card",
        &[
            (family::TYPOGRAPHY, 10),
            (family::SHAPE, 0),
            (family::DEPTH, 15),
        ],
    ),
    (
        "/components/data-table",
        &[
            (family::TYPOGRAPHY, 89),
            (family::SHAPE, 0),
            (family::DEPTH, 109),
            (family::GRID, 2),
            (family::COMPONENT_DRIFT, 10),
        ],
    ),
    (
        "/components/kanban",
        &[
            (family::TYPOGRAPHY, 11),
            (family::SHAPE, 0),
            (family::DEPTH, 12),
            (family::GRID, 34),
            (family::INTERNAL, 2),
            (family::COMPONENT_DRIFT, 1),
        ],
    ),
];

async fn audit_page(path: &str, ceilings: &[(&str, usize)]) {
    let h = harness_at(path).await;
    let font = body_font_family(&h).await;
    let profile = ldui_audit::from_ui_tokens(font);
    let report = ldui_audit::audit_page(&h, &profile, &Default::default())
        .await
        .expect("audit_page");

    report.sanity().unwrap();
    assert_eq!(
        report.count(family::OVERLAP),
        0,
        "{}",
        report.describe(path)
    );

    let ceiling_list: Vec<Ceiling> = ceilings.iter().map(|(f, m)| Ceiling::new(f, *m)).collect();
    let out = check_ceilings(&report, &ceiling_list);
    assert!(
        out.is_pass(),
        "{}\n{}",
        out.over.join("\n"),
        report.describe(path)
    );
}

macro_rules! style_audit_test {
    ($name:ident, $idx:expr) => {
        #[tokio::test]
        #[ignore = "needs the demo dev server (trunk serve in demo/)"]
        async fn $name() {
            let (path, ceilings) = PAGES[$idx];
            audit_page(path, ceilings).await;
        }
    };
}

style_audit_test!(button_style_is_within_ceiling, 0);
style_audit_test!(card_style_is_within_ceiling, 1);
style_audit_test!(data_table_style_is_within_ceiling, 2);
style_audit_test!(kanban_style_is_within_ceiling, 3);

/// Negative control: prove the sweep + merge actually detects things, across
/// both the engine's style families and the daisyUI drift heuristics.
///
/// A detector that reports zero because it is broken (or because merging the
/// drift sweep into the engine report silently dropped something) is worse
/// than no detector — it reads as evidence. This injects one deliberate
/// violation per family and asserts each is caught, then removes them and
/// asserts every count returns to its pre-injection value. Depth is skipped:
/// the engine's own negative controls already prove that family; this control
/// stays focused on the ldui-audit integration (merge) plus drift.
#[tokio::test]
#[ignore = "needs the demo dev server (trunk serve in demo/)"]
async fn sweep_detects_injected_style_and_drift_violations() {
    let h = harness_at("/components/button").await;
    let font = body_font_family(&h).await;
    let profile = ldui_audit::from_ui_tokens(font);

    let before = ldui_audit::audit_page(&h, &profile, &Default::default())
        .await
        .expect("audit_page");
    let before_typography = before.count(family::TYPOGRAPHY);
    let before_shape = before.count(family::SHAPE);
    let before_drift = before.count(family::COMPONENT_DRIFT);

    // (1) an off-ramp font-size -> typography; (2) an off-set border-radius
    // on a 40x40 box (never a "pill", since it's far below the min-side/2
    // threshold) -> shape; (3) a raw, class-less <button> -> component-drift.
    let injected: bool = h
        .page()
        .evaluate(
            r#"
            (() => {
              const host = document.createElement('div');
              host.id = 'ldui-audit-style-probe';
              host.innerHTML =
                '<p style="font-size:13.37px">x</p>' +
                '<div style="border-radius:17px;width:40px;height:40px">x</div>' +
                '<button id="ldui-audit-inject">Save</button>';
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
        dirty.count(family::TYPOGRAPHY) > before_typography,
        "sweep missed the injected off-ramp font-size:\n{}",
        dirty.describe("probe")
    );
    assert!(
        dirty.count(family::SHAPE) > before_shape,
        "sweep missed the injected off-set border-radius:\n{}",
        dirty.describe("probe")
    );
    assert!(
        dirty.count(family::COMPONENT_DRIFT) > before_drift,
        "merge missed the injected raw <button> — is run_drift's report actually\
         reaching audit_page?\n{}",
        dirty.describe("probe")
    );
    let drift_names_the_button_rule = dirty
        .families
        .iter()
        .find(|f| f.family == family::COMPONENT_DRIFT)
        .is_some_and(|f| {
            f.violations
                .iter()
                .any(|v| v.detail.contains("button-without-btn"))
        });
    assert!(
        drift_names_the_button_rule,
        "component-drift violation is missing its button-without-btn detail:\n{}",
        dirty.describe("probe")
    );

    // And it goes quiet again once the probe is gone, so the checks are
    // responding to the probe rather than to something ambient.
    let removed: bool = h
        .page()
        .evaluate(
            "(() => { document.getElementById('ldui-audit-style-probe').remove(); return true; })()",
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
        after.count(family::TYPOGRAPHY),
        before_typography,
        "typography count did not return to baseline after removing the probe"
    );
    assert_eq!(
        after.count(family::SHAPE),
        before_shape,
        "shape count did not return to baseline after removing the probe"
    );
    assert_eq!(
        after.count(family::COMPONENT_DRIFT),
        before_drift,
        "component-drift count did not return to baseline after removing the probe"
    );
}

/// Print the full multi-family backlog across every swept page without
/// asserting — the discovery pass this suite's committed ceilings were
/// filled from, and the tool for re-filling them after a deliberate change.
#[tokio::test]
#[ignore = "reporting only; run explicitly to enumerate the style backlog"]
async fn report_style_backlog() {
    for (path, _) in PAGES {
        let h = harness_at(path).await;
        let font = body_font_family(&h).await;
        let profile = ldui_audit::from_ui_tokens(font);
        let report = ldui_audit::audit_page(&h, &profile, &Default::default())
            .await
            .expect("audit_page");
        println!("{}", report.describe(path));
    }
}
