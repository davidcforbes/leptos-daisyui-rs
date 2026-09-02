//! Real-browser proof for `Collapse`'s toggle identity and accessible name
//! (ldui-3k00): the internal checkbox carries an id, a name, and an
//! accessible name -- `aria-labelledby` pointing at the visible
//! `CollapseTitle` by default, or an explicit `aria-label` that suppresses
//! it -- and Chrome's own accessible-name computation resolves to the title
//! text. Also proves the stretched input still toggles on a title click
//! (the naming must not have disturbed daisyUI's overlay), and that the
//! fixture has zero blocking axe findings, which is how the consumer's
//! form-field audit surfaced the defect.
//!
//! Drives the general demo app (`html_target: None`, like
//! `reactivity_smoke.rs`/`section_heading_smoke.rs`) because the fixture
//! lives on the existing `/components/collapse` showcase route. Kept in its
//! own file/xtask step (`cargo xtask test-collapse-naming`) rather than
//! folded into `reactivity_smoke.rs`, whose check count is pinned.
mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use serde_json::{Value, json};

const PAGE: &str = "/components/collapse";
const FIXTURE: &str = "#collapse-naming-fixture";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate collapse fixture")
        .into_value()
        .expect("collapse expression returns JSON")
}

/// Shape of one collapse's toggle: its id/name, both naming attributes, the
/// id and text of the element `aria-labelledby` resolves to, whether the
/// title element carries the referenced id, and the checked state.
async fn toggle(h: &pixelproof_web::Harness, testid: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('[data-testid="{testid}"]');
                const input = root.querySelector('input[type="checkbox"]');
                const title = root.querySelector('.collapse-title');
                const labelledBy = input.getAttribute('aria-labelledby');
                const referenced = labelledBy ? document.getElementById(labelledBy) : null;
                return {{
                    id: input.getAttribute('id'),
                    name: input.getAttribute('name'),
                    ariaLabel: input.getAttribute('aria-label'),
                    labelledBy,
                    referencedText: referenced ? referenced.textContent.trim() : null,
                    titleId: title.getAttribute('id'),
                    titleText: title.textContent.trim(),
                    checked: input.checked,
                }};
            }})()"#
        ),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-collapse-naming)"]
async fn collapse_toggle_is_identified_and_named_by_its_title() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    // Explicit id + name: both forwarded verbatim, and the toggle is named
    // by the visible title through a resolvable aria-labelledby reference.
    let titled = toggle(&h, "collapse-naming-titled").await;
    assert_eq!(titled["id"], json!("collapse-naming-filters"), "{titled}");
    assert_eq!(titled["name"], json!("show_filters"), "{titled}");
    assert_eq!(titled["ariaLabel"], Value::Null, "{titled}");
    assert_eq!(
        titled["labelledBy"],
        json!("collapse-naming-filters-title"),
        "{titled}"
    );
    assert_eq!(titled["titleId"], titled["labelledBy"], "{titled}");
    assert_eq!(titled["referencedText"], json!("Filters"), "{titled}");
    assert_eq!(titled["titleText"], json!("Filters"), "{titled}");

    // No id supplied: a minted, prefixed id that doubles as the name, still
    // named by its own title (not the other collapse's).
    let minted = toggle(&h, "collapse-naming-minted").await;
    assert!(
        minted["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("ld-collapse-")),
        "{minted}"
    );
    assert_eq!(minted["name"], minted["id"], "{minted}");
    assert_eq!(minted["referencedText"], json!("Sort options"), "{minted}");
    assert_ne!(minted["id"], titled["id"], "{minted}");

    // Explicit aria_label: names the toggle directly and suppresses the
    // aria-labelledby reference, so the two can never disagree.
    let labelled = toggle(&h, "collapse-naming-labelled").await;
    assert_eq!(
        labelled["ariaLabel"],
        json!("Show advanced options"),
        "{labelled}"
    );
    assert_eq!(labelled["labelledBy"], Value::Null, "{labelled}");
    assert_eq!(labelled["referencedText"], Value::Null, "{labelled}");

    // The stretched input still toggles on a title click: naming must not
    // have disturbed daisyUI's overlay.
    assert_eq!(titled["checked"], json!(false), "{titled}");
    click(
        &h,
        r#"[data-testid="collapse-naming-titled"] .collapse-title"#,
    )
    .await;
    let toggled = toggle(&h, "collapse-naming-titled").await;
    assert_eq!(
        toggled["checked"],
        json!(true),
        "clicking the title must check the toggle: {toggled}"
    );

    // The fixture is what a consumer's form-field audit sees: zero blocking
    // axe findings, scoped to the fixture so unrelated catalog examples on
    // the page cannot obscure this contract.
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let _page_report = axe.run(h.page()).await.expect("inject and run axe-core");
    let scoped_axe = eval_json(
        &h,
        &format!(
            r#"(async () => {{
                const report = await axe.run(document.querySelector('{FIXTURE}'), {{
                    runOnly: {{ type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21aa'] }},
                    resultTypes: ['violations'],
                }});
                return report.violations
                    .filter(v => v.impact === 'serious' || v.impact === 'critical')
                    .map(v => ({{
                        id: v.id,
                        nodes: v.nodes.map(node => ({{ target: node.target, html: node.html }})),
                    }}));
            }})()"#
        ),
    )
    .await;
    assert_eq!(
        scoped_axe,
        json!([]),
        "the collapse naming fixture has blocking axe findings: {scoped_axe}"
    );

    assert_no_browser_errors(&h, "collapse toggle naming").await;
}
