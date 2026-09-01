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
//! **Overlap is still a hard failure here too.** Gating goes through
//! `ldui_audit::verify`, which does sanity + hard-fail overlap + the ratchet
//! in one call and refuses a ceiling for overlap as a misconfiguration.
//! Truncation is asserted separately (`common::assert_not_truncated`) because
//! `verify` only notes it.
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
//! Ceilings below are the actual measured counts — no slack. Lower one
//! whenever a fix drops the count; raising one needs a reason in the commit
//! message, and "the count went up" is not one.
//!
//! **A ceiling is the last resort, not the first.** When a page reports a
//! value the app deliberately ships, DECLARE it in [`demo_profile`] with the
//! CSS it comes from; only genuinely-undecided debt is left to the ratchet.
//! The distinction matters because the sweep pushes at most one violation per
//! element: a wide ceiling full of known-benign findings is headroom a real
//! regression on those same elements hides in. That is exactly what happened
//! here (ldui-gne) — every `.btn` reported its resting shadow, so a *wrong*
//! shadow on a button could not have been seen.

mod common;
use common::{assert_not_truncated, body_font_family, harness_at};
use ldui_audit::{Ceiling, ShadowSpec, StyleProfile, family};

/// The demo's audit profile: `ui-tokens` defaults plus this app's **declared
/// deviations**, each one a value the demo deliberately ships that the shared
/// token crate does not name (ldui-gne).
///
/// Deviations are *additive* — read back from `from_ui_tokens`' output and
/// extended, never retyped — so the token crate stays the source of truth and
/// a token change still flows through. Every entry below must cite the CSS it
/// comes from: an undocumented entry here is indistinguishable from raising a
/// ceiling, and blinds the family it widens.
///
/// **Why declare rather than ratchet.** The engine's sweep pushes at most one
/// violation per element and then moves on, so while every `.btn` on a page
/// reports its resting shadow, a genuinely *wrong* shadow on those same
/// buttons is invisible — the ceiling protects nothing. Naming the expected
/// value restores the check: any other shadow on a button now fails.
fn demo_profile(font_family: String) -> StyleProfile {
    let base = ldui_audit::from_ui_tokens(font_family);

    // --- depth ------------------------------------------------------------
    // The demo's "Enhanced Button Push Effect" (`demo/input.css`, the
    // `@layer components { .btn { … } }` block) overrides *every* button's
    // shadow with `!important`:
    //
    //   box-shadow: 0 6px 12px -2px rgba(0, 0, 0, 0.15),
    //               0 3px 6px  -2px rgba(0, 0, 0, 0.1) !important;
    //
    // Two layers, both black, neither inset, both with a -2px spread. It is a
    // deliberate press-affordance vocabulary (paired with a `translateY`),
    // authored in one place, and not expressible as a `ui_tokens::elevation`
    // level — those are single-layer, spread-less. Declaring it is what lets
    // the depth family still say something about buttons.
    //
    // The pressed variant (`0 1px 3px rgba(0, 0, 0, 0.12)`, same block) is
    // deliberately NOT declared: it only paints under `:active`, which a
    // static sweep never captures, and declaring it would additionally accept
    // Tailwind's `shadow-sm` first layer (`0 1px 3px rgba(0,0,0,0.1)`) —
    // inside the opacity epsilon — weakening a real check for no gain today.
    //
    // daisyUI's own control shadows (`.select`, `.input`, `.checkbox`,
    // `.alert`) are NOT declared here: they are authored in `oklch()`/
    // `oklab()`, which the engine's shadow parser does not understand, so any
    // spec written for them would encode a mis-parse (it reads the geometry
    // out of the colour's coordinates — negative blur and all). They stay
    // ratcheted until the engine can parse them: PixelProof-0il.
    //
    // **Nothing is declared here for KPI cards (ldui-k4fn), on purpose.**
    // That bead replaced `KpiCard`'s stock Tailwind `shadow-sm` with
    // `.ld-card-depth`, which paints
    // `var(--ld-card-shadow, var(--ld-elevation-4))` — and LEVEL_4 is
    // already in `base.shadows`, because `from_ui_tokens` seeds the profile
    // from `ui_tokens::elevation::LEVELS`. Landing on the declared
    // vocabulary is precisely what makes an entry here unnecessary: a
    // deviation is for a value the token crate does not name, and this one
    // is named. Adding a KPI entry would be the "undocumented entry"
    // failure this doc-comment warns about, widening the depth family for
    // nothing. No page ceiling moved either — no page in `PAGES` renders a
    // `KpiStrip` today (the fixture that does is
    // `tests/admin_workbench_smoke.rs`, which asserts the card's computed
    // shadow against this same `from_ui_tokens` profile via `shadow_ok`,
    // i.e. the identical declared set and epsilon this sweep uses).
    let mut shadows = base.shadows.clone();
    shadows.extend([
        ShadowSpec::new(0.0, 6.0, 12.0, 0.15).with_spread(-2.0),
        ShadowSpec::new(0.0, 3.0, 6.0, 0.10).with_spread(-2.0),
    ]);

    // --- typography -------------------------------------------------------
    // 18px is daisyUI's / Tailwind's "large" text step, and the demo exists to
    // show daisyUI at its declared sizes: `.table-lg` cells, `.card-title`,
    // `.btn-lg` and `h3.text-lg` all compute to it. The `ui-tokens` ramp
    // (28/20/16/14/12/11) has no 18 because the desktop face has no such step,
    // so this is a web-only, framework-supplied size — declared, not hidden.
    let mut ramp = base.type_ramp.clone();
    ramp.push(18.0);

    base.shadows(shadows).type_ramp(ramp)
}

/// Pages swept, with their current per-family violation ceilings.
///
/// `(family_name, max)`. Overlap is always 0 and asserted separately, never
/// listed here (a ceiling entry for it is itself a misconfiguration per
/// `ldui_audit::verify`'s contract, mirrored here for `check_ceilings`).
///
/// Filled from the actual counts (`report_style_backlog` / the
/// `check_ceilings` failure output, which names the exact count per family),
/// measured *after* [`demo_profile`]'s declared deviations. SHAPE is 0 on
/// every page — daisyUI's border-radius utilities already land on the
/// declared radius set — so it stays listed at 0 as a regression tripwire
/// rather than an ungoverned family. GRID/INTERNAL mirror
/// `layout_audit_smoke.rs`'s committed ceilings for the same page (see the
/// module doc above).
///
/// What is left under the non-zero ceilings, and why each is still there:
///
/// - **TYPOGRAPHY.** Inline `<code>` resolving `ui-monospace` against a
///   profile pinned to the demo's `ui-sans-serif` body face (70 of
///   data-table's 83). A `<code>` element *should* be monospace; the engine's
///   per-element family rule has no exemption for `code`/`pre`/`kbd`/`samp`,
///   which is an engine gap filed as **PixelProof-0oc**, not a page defect.
///   The remainder is daisyUI's `-xs` size step (10px on `.kbd-xs`,
///   `.badge-xs`), its `-xl` step (22px on `.btn-xl`), and Tailwind
///   `text-2xl` (24px) in the kanban demo's own headings — genuinely
///   undecided, so ratcheted rather than declared.
/// - **DEPTH.** Two distinct things. daisyUI's `oklch()`/`oklab()` control
///   shadows (`.select`, `.input`, `.checkbox`, `.alert`) cannot be declared
///   at all until the engine's shadow parser understands those colour
///   functions (**PixelProof-0il**) — 18 of data-table's 18, 2 of kanban's 7.
///   The rest is real ad-hoc-shadow debt: Tailwind `shadow-xl` on the card
///   demo's cards (11) and `shadow-sm` on kanban cards (5), neither from
///   `ui_tokens::elevation`. See `doc/visual-quality/ad-hoc-shadow.md`.
/// - **COMPONENT-DRIFT / GRID / INTERNAL.** Unchanged by this pass.
/// - **CHARTS (ldui-40g).** This page's numbers are a first measurement, not a
///   ratchet raise, and every one of them comes from inside the chart
///   components' SVG — the page's own markup contributes zero violations in
///   every family (verified in the browser: no non-SVG text on the page is off
///   the ramp, and no GRID finding names a non-`svg` selector). TYPOGRAPHY 78
///   is 60 axis/legend labels hardcoded at `font-size="10"` plus 18 heatmap
///   cell labels at `font-size="9"`, both below the `ui-tokens` ramp's smallest
///   step (11) — real, pre-existing library drift that had no page to report
///   on until now, and the reason this bead asked for the page. GRID 14 is
///   vertical distance between `<text>` nodes positioned in the SVG's own
///   viewBox coordinate space, then scaled by `w-full h-auto`; the 4px grid
///   cannot mean anything there, so unlike the other pages this number is not
///   spacing debt to work off. Both are worth an engine/library follow-up
///   rather than a ceiling raise. OVERLAP is 0 and stays 0 — the first run of
///   this page found a real one (`AreaChart`'s bottom y-tick over its first
///   x-tick) and it was fixed at the source, not absorbed.
const PAGES: &[(&str, &[(&str, usize)])] = &[
    (
        "/components/button",
        &[
            (family::TYPOGRAPHY, 1),
            (family::SHAPE, 0),
            (family::DEPTH, 0),
        ],
    ),
    (
        "/components/card",
        &[
            (family::TYPOGRAPHY, 1),
            (family::SHAPE, 0),
            (family::DEPTH, 11),
        ],
    ),
    (
        "/components/data-table",
        &[
            // 31, was 105 (and 92 before that). This drop is NOT a page fix:
            // it is ldui-kq9w landing upstream. PixelProof 8a74b06 exempts the
            // FAMILY check on the mono tags (code/pre/kbd/samp/tt), so this
            // page's inline <code> spans stopped being reported. Those should
            // be monospace, and were unfixable by construction short of
            // forcing code into the body sans face. The font-SIZE check still
            // runs on them, so a <code> at 13px is as off-ramp as anything.
            //
            // 31 is the real remaining count, measured by zeroing the ceiling
            // and reading the failure rather than estimated: daisyUI's own
            // -xs/-xl size steps and Tailwind text-2xl, all genuinely
            // undecided debt. The 105 it replaces was raised for
            // Column::identifier() (ldui-lrig), whose mono face is deliberate;
            // that reasoning held at the time but is moot now the upstream
            // exemption covers the whole class of finding. Ceilings here carry
            // no slack on purpose: the sweep pushes at most one violation per
            // element, so headroom is exactly where a real regression hides.
            (family::TYPOGRAPHY, 31),
            (family::SHAPE, 0),
            // 36, was 32. The +4 is the selection checkboxes added by
            // ldui-px06/ldui-nz6d, and it is the SAME already-declared
            // class, not new debt: every one of the 36 is a daisyUI control
            // shadow authored in oklch()/oklab(), which the engine's shadow
            // parser cannot read (it decodes geometry out of the colour's
            // coordinates), so a spec written for them would encode a
            // mis-parse. Verified rather than assumed - the audit report
            // caps at 20 findings per family, so 16 were elided; enumerating
            // every non-none box-shadow on the page gives 23 select + 10
            // checkbox + 3 alert = 36, with the 159 .btn shadows being the
            // demo's DECLARED press affordance rather than violations.
            // Stays ratcheted until PixelProof-0il teaches the parser oklch.
            (family::DEPTH, 36),
            (family::GRID, 2),
            (family::COMPONENT_DRIFT, 10),
        ],
    ),
    (
        "/components/kanban",
        &[
            (family::TYPOGRAPHY, 7),
            (family::SHAPE, 0),
            (family::DEPTH, 7),
            (family::GRID, 34),
            (family::INTERNAL, 2),
            (family::COMPONENT_DRIFT, 1),
        ],
    ),
    (
        "/components/charts",
        &[
            // 122, was 78. ldui-y2ed's diverging BarChart and ldui-8d94's two
            // typed Heatmaps add tick, axis and cell labels at the charts' own
            // 10px size. VERIFIED exhaustively rather than sampled, because the
            // report caps at 20 per family: sweeping every leaf text node on the
            // page found 122 off-ramp sizes, ALL of them inside an <svg> and none
            // outside it -- the same not-on-the-ramp class already declared here,
            // scaled by two more charts, not new page debt.
            (family::TYPOGRAPHY, 122),
            (family::SHAPE, 0),
            (family::DEPTH, 0),
            // 23, was 16: ldui-j0mt's dual-axis chart is the third
            // categorical LineChart fixture, and a secondary axis adds a
            // second column of tick text plus its own axis title. Every one
            // of these findings is an `svg > text` distance — scaled-viewBox
            // tick/label geometry, which is not workable on the 4px grid and
            // is the same class as the rest of this page (see the module doc
            // note above), not new page debt.
            // 33, was 23: the same two new charts, same svg > text distance class.
            (family::GRID, 33),
            (family::INTERNAL, 0),
            // 0, was 2. The categorical charts' screen-reader data tables
            // are raw <table class="sr-only"> by design, and the daisyUI
            // .table drift heuristic used to flag every one of them. Rather
            // than ratchet that to 3 for the new dual-axis chart, the rule
            // itself now exempts .sr-only (audit/src/drift.js): the check
            // exists to catch a table that LOOKS undesigned, and an
            // invisible table has no appearance to get wrong. Fixed at
            // source, so a consumer's audit no longer carries one finding
            // per chart for markup they cannot see — the same reasoning as
            // ldui-zl58's nav rail. Kept listed at 0 as a tripwire.
            (family::COMPONENT_DRIFT, 0),
        ],
    ),
];

async fn audit_page(path: &str, ceilings: &[(&str, usize)]) {
    let h = harness_at(path).await;
    let font = body_font_family(&h).await;
    let profile = demo_profile(font);
    let report = ldui_audit::audit_page(&h, &profile, &Default::default())
        .await
        .expect("audit_page");

    // `verify` is sanity + hard-fail overlap + the ratchet in one call, and it
    // additionally refuses a ceiling for OVERLAP as a misconfiguration. It
    // only *notes* truncation in its Ok path, though, so that gets its own
    // assertion — see `common::assert_not_truncated` for why a truncated
    // sweep must never read as a pass.
    let ceiling_list: Vec<Ceiling> = ceilings.iter().map(|(f, m)| Ceiling::new(f, *m)).collect();
    let notes = ldui_audit::verify(path, &report, &ceiling_list).unwrap_or_else(|e| panic!("{e}"));
    for n in &notes {
        println!("{n}");
    }
    assert_not_truncated(&report, path);
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
style_audit_test!(charts_style_is_within_ceiling, 4);

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
    let profile = demo_profile(font);

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
        let profile = demo_profile(font);
        let report = ldui_audit::audit_page(&h, &profile, &Default::default())
            .await
            .expect("audit_page");
        println!("{}", report.describe(path));
    }
}
