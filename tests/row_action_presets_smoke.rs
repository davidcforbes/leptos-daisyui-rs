//! Real-browser proof for the canonical row-action presets (ldui-bmqj), the
//! standard Export action (ldui-e6x8) and `Button::disabled_reason`
//! (ldui-p82h) over the `/components/row-action-presets` showcase.
//!
//! What a native test structurally cannot see: that each preset's `<use>`
//! references the glyph the sprite map promises, that the per-row
//! `aria-label` and the tooltip's `data-tip` are the SAME text, that a
//! reasoned button's `aria-describedby` RESOLVES to its reason in the live
//! document, and that the audit marker appears only on the button disabled
//! without one.
//!
//! Drives the general demo app (`html_target: None`). Kept in its own
//! file/xtask step (`cargo xtask test-row-action-presets`) rather than folded
//! into `reactivity_smoke.rs`, whose check count is pinned.
mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

const PAGE: &str = "/components/row-action-presets";

/// `(kind marker, sprite symbol id, EN label template)` for every preset.
const PRESETS: [(&str, &str, &str); 6] = [
    ("open", "eye", "Open {name}"),
    ("delete", "trash", "Delete {name}"),
    ("call", "phone", "Call {name}"),
    ("text", "message", "Text {name}"),
    ("email", "envelope", "Email {name}"),
    ("complete", "circle-check", "Complete {name}"),
];

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate row-action-presets fixture")
        .into_value()
        .expect("fixture expression returns JSON")
}

/// Everything a preset button exposes, read by stable hooks only.
async fn probe(h: &pixelproof_web::Harness, root_testid: &str, hook: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('[data-testid="{root_testid}"]');
                const buttons = root ? [...root.querySelectorAll('{hook}')] : [];
                if (buttons.length !== 1) return {{ count: buttons.length }};
                const b = buttons[0];
                const use = b.querySelector('svg use');
                const tooltip = b.closest('.tooltip');
                const describedby = b.getAttribute('aria-describedby');
                const described = describedby
                    ? [...describedby.split(/\s+/)].map(id => {{
                        const t = document.getElementById(id);
                        return t ? t.textContent.trim() : null;
                    }})
                    : null;
                return {{
                    count: 1,
                    isButton: b.tagName === 'BUTTON',
                    hasBtn: b.classList.contains('btn'),
                    ghostXsSquare: ['btn-ghost', 'btn-xs', 'btn-square'].every(c => b.classList.contains(c)),
                    href: use ? use.getAttribute('href') : null,
                    ariaLabel: b.getAttribute('aria-label'),
                    tip: tooltip ? tooltip.dataset.tip : null,
                    disabled: b.disabled,
                    btnDisabled: b.classList.contains('btn-disabled'),
                    describedby,
                    described,
                    title: b.getAttribute('title'),
                    withoutReason: b.hasAttribute('data-disabled-without-reason'),
                }};
            }})()"#
        ),
    )
    .await
}

async fn click(h: &pixelproof_web::Harness, root_testid: &str, hook: &str) {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const b = document.querySelector('[data-testid="{root_testid}"] {hook}');
                b.click();
                return true;
            }})()"#
        ),
    )
    .await;
}

async fn text_of(h: &pixelproof_web::Harness, testid: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(document.querySelector('[data-testid="{testid}"]') || {{ textContent: null }}).textContent"#
        ),
    )
    .await
}

fn assert_enabled_preset(probe: &Value, kind: &str, glyph: &str, label: &str) {
    assert_eq!(
        probe["count"],
        json!(1),
        "{kind}: exactly one button: {probe}"
    );
    assert_eq!(probe["isButton"], json!(true), "{kind}: {probe}");
    assert_eq!(
        probe["hasBtn"],
        json!(true),
        "{kind}: carries .btn: {probe}"
    );
    assert_eq!(
        probe["ghostXsSquare"],
        json!(true),
        "{kind}: square ghost xs: {probe}"
    );
    assert_eq!(
        probe["href"],
        json!(format!("#{glyph}")),
        "{kind}: the <use> references the promised sprite symbol: {probe}"
    );
    assert_eq!(probe["ariaLabel"], json!(label), "{kind}: {probe}");
    assert_eq!(
        probe["tip"],
        json!(label),
        "{kind}: the tooltip carries the SAME text as the accessible name: {probe}"
    );
    assert_eq!(probe["disabled"], json!(false), "{kind}: {probe}");
    assert_eq!(probe["btnDisabled"], json!(false), "{kind}: {probe}");
    assert_eq!(
        probe["describedby"],
        Value::Null,
        "{kind}: an enabled preset has no reason to describe: {probe}"
    );
    assert_eq!(probe["title"], Value::Null, "{kind}: {probe}");
    assert_eq!(
        probe["withoutReason"],
        json!(false),
        "{kind}: an enabled button never carries the audit marker: {probe}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-row-action-presets)"]
async fn row_action_presets_render_glyph_label_tooltip_and_reason() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, r#"[data-testid="row-action-presets-table"]"#).await;

    // Ana's row: all six enabled, each named for the row.
    for (kind, glyph, template) in PRESETS {
        let p = probe(&h, "row-ana", &format!("[data-row-action=\"{kind}\"]")).await;
        assert_enabled_preset(&p, kind, glyph, &template.replace("{name}", "Ana Ruiz"));
    }

    // Luis's row: the same five, plus a Complete disabled WITH a reason.
    for (kind, glyph, template) in PRESETS.iter().take(5) {
        let p = probe(&h, "row-luis", &format!("[data-row-action=\"{kind}\"]")).await;
        assert_enabled_preset(&p, kind, glyph, &template.replace("{name}", "Luis Ortega"));
    }
    let complete = probe(&h, "row-luis", r#"[data-row-action="complete"]"#).await;
    assert_eq!(complete["count"], json!(1), "{complete}");
    assert_eq!(complete["href"], json!("#circle-check"), "{complete}");
    assert_eq!(
        complete["ariaLabel"],
        json!("Complete Luis Ortega"),
        "the name is unchanged by the reason: {complete}"
    );
    assert_eq!(complete["tip"], json!("Complete Luis Ortega"), "{complete}");
    assert_eq!(complete["disabled"], json!(true), "{complete}");
    assert_eq!(complete["btnDisabled"], json!(true), "{complete}");
    assert_eq!(
        complete["described"],
        json!(["Already complete"]),
        "aria-describedby must RESOLVE to the reason in the live document: {complete}"
    );
    assert_eq!(
        complete["title"],
        json!("Already complete"),
        "the native title carries the reason for hover: {complete}"
    );
    assert_eq!(
        complete["withoutReason"],
        json!(false),
        "a reasoned button never carries the audit marker: {complete}"
    );

    // Activation reaches the caller with nothing but the trigger; the
    // disabled one dispatches no click at all (native semantics).
    click(&h, "row-ana", r#"[data-row-action="open"]"#).await;
    assert_eq!(text_of(&h, "row-action-last").await, json!("open:Ana Ruiz"));
    click(&h, "row-luis", r#"[data-row-action="delete"]"#).await;
    assert_eq!(
        text_of(&h, "row-action-last").await,
        json!("delete:Luis Ortega")
    );
    click(&h, "row-luis", r#"[data-row-action="complete"]"#).await;
    assert_eq!(
        text_of(&h, "row-action-last").await,
        json!("delete:Luis Ortega"),
        "a disabled preset must not fire"
    );

    assert_no_browser_errors(&h, "row action presets").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-row-action-presets)"]
async fn export_action_is_named_glyphed_and_disabled_with_a_reason_at_zero_rows() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(
        &h,
        r#"[data-testid="export-with-rows"] [data-entity-export="true"]"#,
    )
    .await;

    let live = probe(&h, "export-with-rows", r#"[data-entity-export="true"]"#).await;
    assert_eq!(live["count"], json!(1), "{live}");
    assert_eq!(live["hasBtn"], json!(true), "{live}");
    assert_eq!(live["ghostXsSquare"], json!(true), "{live}");
    assert_eq!(
        live["href"],
        json!("#download"),
        "the Export glyph is the sprite's download symbol: {live}"
    );
    assert_eq!(live["ariaLabel"], json!("Export to CSV"), "{live}");
    assert_eq!(live["tip"], json!("Export to CSV"), "{live}");
    assert_eq!(live["disabled"], json!(false), "{live}");
    assert_eq!(live["describedby"], Value::Null, "{live}");
    assert_eq!(live["withoutReason"], json!(false), "{live}");

    let empty = probe(&h, "export-empty", r#"[data-entity-export="true"]"#).await;
    assert_eq!(empty["count"], json!(1), "{empty}");
    assert_eq!(empty["href"], json!("#download"), "{empty}");
    assert_eq!(empty["ariaLabel"], json!("Export to CSV"), "{empty}");
    assert_eq!(empty["disabled"], json!(true), "{empty}");
    assert_eq!(empty["btnDisabled"], json!(true), "{empty}");
    assert_eq!(
        empty["described"],
        json!(["No rows to export"]),
        "zero rows must be explained through a resolving aria-describedby: {empty}"
    );
    assert_eq!(empty["title"], json!("No rows to export"), "{empty}");
    assert_eq!(empty["withoutReason"], json!(false), "{empty}");

    // The trigger reaches the caller once per activation; the empty one never.
    assert_eq!(text_of(&h, "export-count").await, json!("0"));
    click(&h, "export-with-rows", r#"[data-entity-export="true"]"#).await;
    assert_eq!(text_of(&h, "export-count").await, json!("1"));
    click(&h, "export-empty", r#"[data-entity-export="true"]"#).await;
    assert_eq!(
        text_of(&h, "export-count").await,
        json!("1"),
        "a zero-row Export must not fire"
    );

    assert_no_browser_errors(&h, "entity export action").await;
}

/// The audit marker's negative control on the SAME document: a reasoned
/// `Button` is described and unmarked; a `disabled=true` with no reason is
/// marked and undescribed -- which is exactly what the drift rule
/// `disabled-without-reason` reports.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-row-action-presets)"]
async fn disabled_without_a_reason_is_stamped_and_a_reasoned_button_is_described() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, r#"[data-testid="disabled-without-reason-probe"]"#).await;

    let reasoned = eval_json(
        &h,
        r#"(() => {
            const b = document.querySelector('[data-testid="reasoned-button"]');
            const id = b.getAttribute('aria-describedby');
            const hint = id ? document.getElementById(id) : null;
            return {
                disabled: b.disabled,
                btnDisabled: b.classList.contains('btn-disabled'),
                idPrefixed: !!id && id.startsWith('ld-btn-reason-'),
                hintInside: !!hint && b.contains(hint),
                hintHiddenFromName: !!hint && hint.getAttribute('aria-hidden') === 'true'
                    && hint.classList.contains('sr-only'),
                hintText: hint ? hint.textContent.trim() : null,
                title: b.getAttribute('title'),
                withoutReason: b.hasAttribute('data-disabled-without-reason'),
            };
        })()"#,
    )
    .await;
    assert_eq!(reasoned["disabled"], json!(true), "{reasoned}");
    assert_eq!(reasoned["btnDisabled"], json!(true), "{reasoned}");
    assert_eq!(reasoned["idPrefixed"], json!(true), "{reasoned}");
    assert_eq!(
        reasoned["hintInside"],
        json!(true),
        "the hint lives inside the single-root button: {reasoned}"
    );
    assert_eq!(
        reasoned["hintHiddenFromName"],
        json!(true),
        "the hint is sr-only and excluded from the accessible name: {reasoned}"
    );
    assert_eq!(
        reasoned["hintText"],
        json!("Sign-off is pending"),
        "{reasoned}"
    );
    assert_eq!(
        reasoned["title"],
        json!("Sign-off is pending"),
        "{reasoned}"
    );
    assert_eq!(reasoned["withoutReason"], json!(false), "{reasoned}");

    let probe_button = eval_json(
        &h,
        r#"(() => {
            const b = document.querySelector('[data-testid="disabled-without-reason-probe"]');
            return {
                disabled: b.disabled,
                describedby: b.getAttribute('aria-describedby'),
                title: b.getAttribute('title'),
                marker: b.getAttribute('data-disabled-without-reason'),
            };
        })()"#,
    )
    .await;
    assert_eq!(probe_button["disabled"], json!(true), "{probe_button}");
    assert_eq!(probe_button["describedby"], Value::Null, "{probe_button}");
    assert_eq!(probe_button["title"], Value::Null, "{probe_button}");
    assert_eq!(
        probe_button["marker"],
        json!("true"),
        "disabled with no reason must be stamped for the audit: {probe_button}"
    );

    assert_no_browser_errors(&h, "disabled_reason marker").await;
}
