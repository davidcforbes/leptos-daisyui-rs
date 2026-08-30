//! Real-browser proof for `SectionHeading` (ldui-lwu): valid heading
//! hierarchy + stable id, empty optional regions leaving no spacing,
//! status/actions wrapping instead of squeezing the title, and reactive
//! localized copy. Drives the general demo app (`html_target: None`, like
//! `reactivity_smoke.rs`/`keyed_result_list_smoke.rs`) rather than a
//! dedicated test-host page, because the fixture lives on the existing
//! `/components/section_heading` showcase route. Kept in its own
//! file/xtask step rather than folded into `reactivity_smoke.rs`, whose
//! check count is pinned.

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use pixelproof_web::ViewportSize;
use serde_json::{Value, json};

const PAGE: &str = "/components/section_heading";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate section-heading fixture")
        .into_value()
        .expect("section-heading expression returns JSON")
}

/// Shape of one `[data-section-heading]` root: its tag name, id, whether an
/// eyebrow/description paragraph is present at all (rather than present but
/// empty), and whether a status/actions wrapper is present.
async fn snapshot(h: &pixelproof_web::Harness, fixture_testid: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const fixture = document.querySelector('[data-testid="{fixture_testid}"]');
                const root = fixture.querySelector('[data-section-heading]');
                const heading = root.querySelector('h2, h3, h4');
                const paragraphs = root.querySelectorAll('p').length;
                const actions = root.querySelector('[data-section-heading-actions]');
                return {{
                    tag: heading ? heading.tagName.toLowerCase() : null,
                    headingId: heading ? heading.id || null : null,
                    level: root.getAttribute('data-section-heading-level'),
                    paragraphCount: paragraphs,
                    hasActions: !!actions,
                }};
            }})()"#
        ),
    )
    .await
}

/// The plain variant renders exactly one `h2`, carries its stable id, and
/// both the eyebrow and description paragraphs are present (both props were
/// supplied non-empty) with no actions wrapper at all.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-section-heading)"]
async fn plain_variant_has_a_valid_h2_with_its_stable_id_and_no_actions_wrapper() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, "section-heading-plain").await;
    assert_eq!(s["tag"], json!("h2"), "default level is h2: {s}");
    assert_eq!(
        s["headingId"],
        json!("section-heading-plain-heading"),
        "the heading element itself must carry the caller's stable id: {s}"
    );
    assert_eq!(s["level"], json!("h2"), "level marker: {s}");
    assert_eq!(
        s["paragraphCount"],
        json!(2),
        "eyebrow + description both supplied non-empty: {s}"
    );
    assert_eq!(
        s["hasActions"],
        json!(false),
        "no actions slot was supplied: {s}"
    );

    assert_no_browser_errors(&h, "section-heading plain variant").await;
}

/// An explicit `H3` renders an `h3`, not an `h2` -- the level prop actually
/// changes the tag, not just a CSS class.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-section-heading)"]
async fn status_variant_renders_the_requested_h3_level_with_status_inline() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, "section-heading-status").await;
    assert_eq!(s["tag"], json!("h3"), "explicit H3 level: {s}");
    assert_eq!(s["level"], json!("h3"), "level marker: {s}");
    assert_eq!(
        s["paragraphCount"],
        json!(1),
        "eyebrow supplied, description omitted and therefore absent, not empty: {s}"
    );

    let badge_present = eval_json(
        &h,
        r#"!!document.querySelector('[data-testid="section-heading-status-badge"]')"#,
    )
    .await;
    assert_eq!(
        badge_present,
        json!(true),
        "status slot content actually reached the DOM"
    );

    assert_no_browser_errors(&h, "section-heading status variant").await;
}

/// The action variant renders its actions wrapper, and at a compact
/// viewport the actions wrap onto their own row rather than shrinking the
/// title's rendered width down toward the actions' width.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-section-heading)"]
async fn actions_wrap_at_compact_widths_instead_of_squeezing_the_title() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, "section-heading-action").await;
    assert_eq!(s["hasActions"], json!(true), "actions slot present: {s}");

    h.set_viewport(ViewportSize::new(390, 844))
        .await
        .expect("set compact viewport");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let boxes = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="section-heading-action"]');
            const root = fixture.querySelector('[data-section-heading]');
            const heading = root.querySelector('h2, h3, h4');
            const actions = root.querySelector('[data-section-heading-actions]');
            const box = el => {
                const r = el.getBoundingClientRect();
                return { top: r.top, bottom: r.bottom, width: r.width };
            };
            return { heading: box(heading), actions: box(actions) };
        })()"#,
    )
    .await;

    let heading_bottom = boxes["heading"]["bottom"].as_f64().expect("heading bottom");
    let actions_top = boxes["actions"]["top"].as_f64().expect("actions top");
    assert!(
        actions_top >= heading_bottom - 1.0,
        "actions must wrap onto their own row at a compact width, not overlap the title: {boxes}"
    );
    let heading_width = boxes["heading"]["width"].as_f64().expect("heading width");
    assert!(
        heading_width > 40.0,
        "the title must not be squeezed down to a sliver by the actions column: {boxes}"
    );

    assert_no_browser_errors(&h, "section-heading action variant compact layout").await;
}

/// The long-copy variant combines a long title, long description, status
/// and actions -- everything must still wrap without horizontal overflow
/// past the fixture's own width, and the description must be readable
/// (never clipped to one line).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-section-heading)"]
async fn long_copy_variant_wraps_without_overflowing_its_container() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, "section-heading-long-copy").await;
    assert_eq!(
        s["paragraphCount"],
        json!(2),
        "eyebrow + long description both present: {s}"
    );
    assert_eq!(s["hasActions"], json!(true), "actions present: {s}");

    let overflow = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="section-heading-long-copy"]');
            const root = fixture.querySelector('[data-section-heading]');
            const fixtureRight = fixture.getBoundingClientRect().right;
            const rootRight = root.getBoundingClientRect().right;
            return rootRight <= fixtureRight + 1;
        })()"#,
    )
    .await;
    assert_eq!(
        overflow,
        json!(true),
        "the heading must not overflow its own fixture width"
    );

    assert_no_browser_errors(&h, "section-heading long-copy variant").await;
}

/// The localized variant's eyebrow/title/description are reactive: clicking
/// the fixture's toggle swaps the rendered text without a route change or
/// remount, proving the props are live signals rather than one-shot strings.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-section-heading)"]
async fn localized_copy_updates_reactively_without_a_remount() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    const HEADING_TEXT_EXPR: &str =
        r#"document.querySelector('#section-heading-localized-heading')?.textContent ?? null"#;

    let before = eval_json(&h, HEADING_TEXT_EXPR).await;
    assert_eq!(
        before,
        json!("Case summary"),
        "English by default: {before}"
    );

    click(&h, "[data-testid=\"section-heading-localized-toggle\"]").await;
    let after = eval_json(&h, HEADING_TEXT_EXPR).await;
    assert_eq!(
        after,
        json!("Resume du dossier"),
        "toggling swaps the reactive title signal in place: {after}"
    );

    click(&h, "[data-testid=\"section-heading-localized-toggle\"]").await;
    let restored = eval_json(&h, HEADING_TEXT_EXPR).await;
    assert_eq!(restored, before, "toggling back restores the original copy");

    assert_no_browser_errors(&h, "section-heading localized variant").await;
}
