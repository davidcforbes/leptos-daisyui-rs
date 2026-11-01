//! Real-browser proof for `MessageViewer`, `MessageViewerDialog` and
//! `ViewMessageRowAction` (ldui-mhur), on the `/components/message_viewer`
//! showcase:
//!
//! - the WhatsApp and SMS/MMS chrome (sender, number, Template / Session
//!   message / MMS badge, the segment indicator);
//! - the picture above the text, with the caller's alt or "{channel}
//!   picture" when the caller gave none, its loading skeleton cleared once it
//!   loads, and the broken-picture tile that says it could not be shown;
//! - the language / picture / template switches: `aria-pressed`, the shown
//!   text, and focus staying on the pressed button;
//! - the delivery-status chip and its reason, the reach line, the reply
//!   thread, the Spanish table;
//! - click-to-enlarge, closed by Escape without closing an outer dialog;
//! - the eye row action opening the dialog, focus inside it, Escape and the
//!   close button, and focus returning to the eye;
//! - axe in light and dark, and no component drift.
//!
//! Drives the general demo app (`html_target: None`); kept in its own
//! file/xtask step (`cargo xtask test-message-viewer`) rather than folded into
//! `reactivity_smoke.rs`, whose check count is pinned.
mod common;

use common::axe::assert_axe_clean;
use common::{
    assert_no_browser_errors, begin_browser_error_capture, body_font_family, click, harness_at,
};
use ldui_audit::family;
use pixelproof_web::{Harness, Key};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const PAGE: &str = "/components/message_viewer";

const FIXTURES: [&str; 7] = [
    "viewer-whatsapp-template",
    "viewer-whatsapp-session",
    "viewer-sms",
    "viewer-mms",
    "viewer-broken",
    "viewer-spanish",
    "viewer-replies",
];

const TEMPLATE_ALT: &str = "Reminder banner: free consultation, Tuesday 10:30 AM";
const EYE: &str = r#"[data-testid="viewer-table"] [data-row-action="open"][aria-label="View message: Consultation reminder"]"#;
const EYE_BROKEN: &str = r#"[data-testid="viewer-table"] [data-row-action="open"][aria-label="View message: Saturday clinic"]"#;
const DIALOG: &str = "dialog[data-message-viewer-dialog]";

async fn eval_json(h: &Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate message-viewer fixture")
        .into_value()
        .expect("message-viewer expression returns JSON")
}

async fn settle() {
    tokio::time::sleep(Duration::from_millis(200)).await;
}

/// The selector of a fixture's viewer root.
fn viewer(testid: &str) -> String {
    format!(r#"[data-testid="{testid}"] [data-message-viewer]"#)
}

/// Everything a test reads about one viewer, in one round trip.
async fn snapshot(h: &Harness, root: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{root}');
                if (!root) return {{ missing: true }};
                const q = (s) => root.querySelector(s);
                const text = (s) => {{ const e = q(s); return e ? e.textContent.trim() : null; }};
                const bubble = q('[data-rich-message][data-direction="out"]');
                const img = bubble ? bubble.querySelector('[data-message-header="image"] img') : null;
                const body = bubble ? bubble.querySelector('[data-message-body]') : null;
                const header = bubble ? bubble.querySelector('[data-message-header="image"]') : null;
                const open = bubble ? bubble.querySelector('[data-message-image-open]') : null;
                const status = q('[data-delivery-status]');
                const reasonId = status ? status.getAttribute('aria-describedby') : null;
                const reason = reasonId ? document.getElementById(reasonId) : null;
                const pressed = {{}};
                for (const b of root.querySelectorAll('[data-viewer-option]')) {{
                    pressed[b.getAttribute('data-viewer-option') + ':' + b.textContent.trim()] = b.getAttribute('aria-pressed');
                }}
                return {{
                    channel: root.getAttribute('data-viewer-channel'),
                    mode: root.getAttribute('data-viewer-mode'),
                    chrome: q('[data-viewer-chrome]')?.getAttribute('data-viewer-chrome') ?? null,
                    sender: text('[data-viewer-sender]'),
                    number: text('[data-viewer-number]'),
                    badge: text('[data-viewer-badge]'),
                    region: q('[data-viewer-screen]')?.getAttribute('aria-label') ?? null,
                    body: body ? body.textContent : null,
                    imgAlt: img ? img.getAttribute('alt') : null,
                    imgLoaded: img ? (img.complete && img.naturalWidth > 0) : null,
                    pictureAboveText: img && body ? img.getBoundingClientRect().bottom <= body.getBoundingClientRect().top + 1 : null,
                    loading: !!(bubble && bubble.querySelector('[data-media-loading]')),
                    busy: header ? header.getAttribute('aria-busy') : null,
                    openLabel: open ? open.getAttribute('aria-label') : null,
                    fallback: bubble ? (bubble.querySelector('[data-media-fallback]')?.textContent.trim() ?? null) : null,
                    placeholders: bubble ? [...bubble.querySelectorAll('[data-placeholder]')].map((e) => e.getAttribute('data-placeholder')) : [],
                    segments: bubble ? (bubble.querySelector('[data-sms-indicator]')?.getAttribute('data-sms-segments') ?? null) : null,
                    switches: [...root.querySelectorAll('[data-viewer-switch]')].map((e) => e.getAttribute('data-viewer-switch')),
                    switchLabels: [...root.querySelectorAll('[data-viewer-switch]')].map((e) => e.getAttribute('aria-label')),
                    pressed,
                    status: status ? status.getAttribute('data-delivery-status') : null,
                    statusText: status ? status.textContent.trim() : null,
                    statusTabindex: status ? status.getAttribute('tabindex') : null,
                    statusTip: status ? (status.closest('[data-tip]')?.getAttribute('data-tip') ?? null) : null,
                    reason: reason ? reason.textContent.trim() : null,
                    counts: text('[data-viewer-counts]'),
                    sentAt: text('[data-viewer-sent-at]'),
                    channelName: text('[data-viewer-channel-name]'),
                    replies: [...root.querySelectorAll('[data-viewer-replies] [data-rich-message]')].map((e) => [e.getAttribute('data-direction'), e.querySelector('[data-message-body]').textContent]),
                }};
            }})()"#,
            root = root.replace('\'', "\\'")
        ),
    )
    .await
}

/// Wait until every picture in `root` has finished loading or failing.
async fn wait_for_pictures(h: &Harness, root: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let done = eval_json(
            h,
            &format!(
                r#"[...document.querySelectorAll('{root} [data-message-header="image"] img')].every((i) => i.complete)"#,
                root = root.replace('\'', "\\'")
            ),
        )
        .await;
        if done == json!(true) {
            settle().await;
            return;
        }
        assert!(
            Instant::now() < deadline,
            "pictures in {root} never finished loading"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} is not a string: {v}"))
}

async fn whatsapp_template(h: &Harness) {
    let root = viewer("viewer-whatsapp-template");
    wait_for_pictures(h, &root).await;
    let v = snapshot(h, &root).await;
    assert_eq!(v["chrome"], json!("whatsapp"), "{v}");
    assert_eq!(v["channel"], json!("whatsapp"), "{v}");
    assert_eq!(v["sender"], json!("Diener Law"), "{v}");
    assert_eq!(v["number"], json!("+1 707-555-0100"), "{v}");
    assert_eq!(
        v["badge"],
        json!("Template"),
        "an approved template says so: {v}"
    );
    assert_eq!(v["region"], json!("Message as the recipient saw it"), "{v}");

    // The picture: the caller's alt, loaded, skeleton gone, above the text,
    // and an enlarge button named for it.
    assert_eq!(v["imgAlt"], json!(TEMPLATE_ALT), "{v}");
    assert_eq!(v["imgLoaded"], json!(true), "{v}");
    assert_eq!(
        v["loading"],
        json!(false),
        "the skeleton clears once the picture loads: {v}"
    );
    assert_eq!(v["busy"], Value::Null, "{v}");
    assert_eq!(v["pictureAboveText"], json!(true), "{v}");
    assert_eq!(
        v["openLabel"],
        json!(format!("Enlarge picture: {TEMPLATE_ALT}")),
        "{v}"
    );
    assert!(s(&v, "body").contains("Hi Ana"), "{v}");

    assert_eq!(
        v["switches"],
        json!(["language", "picture", "content"]),
        "{v}"
    );
    assert_eq!(v["pressed"]["language:EN"], json!("true"), "{v}");
    assert_eq!(v["pressed"]["language:ES"], json!("false"), "{v}");
    assert_eq!(
        v["pressed"]["with-picture:With picture"],
        json!("true"),
        "{v}"
    );
    assert_eq!(v["pressed"]["as-sent:As sent"], json!("true"), "{v}");

    assert_eq!(v["status"], json!("delivered"), "{v}");
    assert_eq!(v["statusText"], json!("Delivered"), "{v}");
    assert_eq!(
        v["counts"],
        json!("Sent to 132 · with picture 120 · text only 12"),
        "{v}"
    );
    assert_eq!(v["sentAt"], json!("Mon, Oct 27 2026, 10:02 AM"), "{v}");
    assert_eq!(v["channelName"], json!("WhatsApp"), "{v}");

    // Language: a real click shows the Spanish version, flips aria-pressed,
    // and leaves focus on the button that was pressed (the switch row is not
    // rebuilt under it).
    // Found by its label, never by position; tagged so the click is a real
    // CDP click on that exact button.
    let tagged = eval_json(
        h,
        &format!(
            r#"(() => {{
                const b = [...document.querySelectorAll('{root} [data-viewer-option="language"]')].find((b) => b.textContent.trim() === 'ES');
                if (!b) return false;
                b.setAttribute('data-test-target', 'es');
                return true;
            }})()"#,
            root = root.replace('\'', "\\'")
        ),
    )
    .await;
    assert_eq!(tagged, json!(true), "the ES language button exists");
    click(h, &format!(r#"{root} [data-test-target="es"]"#)).await;
    let after = snapshot(h, &root).await;
    assert!(s(&after, "body").contains("Hola Ana"), "{after}");
    assert_eq!(after["pressed"]["language:ES"], json!("true"), "{after}");
    assert_eq!(after["pressed"]["language:EN"], json!("false"), "{after}");
    let focus = eval_json(
        h,
        "document.activeElement && document.activeElement.getAttribute('data-test-target')",
    )
    .await;
    assert_eq!(
        focus,
        json!("es"),
        "focus stays on the pressed language button"
    );

    // Text only: the picture goes, the text stays.
    click(h, &format!(r#"{root} [data-viewer-option="text-only"]"#)).await;
    let text_only = snapshot(h, &root).await;
    assert_eq!(text_only["imgAlt"], Value::Null, "{text_only}");
    assert!(s(&text_only, "body").contains("Hola Ana"), "{text_only}");
    assert_eq!(
        text_only["pressed"]["text-only:Text only"],
        json!("true"),
        "{text_only}"
    );
    click(h, &format!(r#"{root} [data-viewer-option="with-picture"]"#)).await;

    // Template: the placeholders are highlighted.
    click(h, &format!(r#"{root} [data-viewer-option="template"]"#)).await;
    let template = snapshot(h, &root).await;
    assert_eq!(template["mode"], json!("template"), "{template}");
    assert_eq!(
        template["placeholders"],
        json!(["first_name", "appointment_date"]),
        "{template}"
    );
    click(h, &format!(r#"{root} [data-viewer-option="as-sent"]"#)).await;
    let back = snapshot(h, &root).await;
    assert_eq!(
        back["placeholders"],
        json!([]),
        "as sent shows no placeholder chips: {back}"
    );
}

async fn whatsapp_session(h: &Harness) {
    let v = snapshot(h, &viewer("viewer-whatsapp-session")).await;
    assert_eq!(v["badge"], json!("Session message"), "{v}");
    assert_eq!(
        v["switches"],
        json!([]),
        "a single text version offers no switch: {v}"
    );
    assert_eq!(v["imgAlt"], Value::Null, "{v}");
    assert_eq!(
        v["segments"],
        Value::Null,
        "WhatsApp has no segment indicator: {v}"
    );
    assert_eq!(v["status"], json!("read"), "{v}");
}

async fn sms(h: &Harness) {
    let v = snapshot(h, &viewer("viewer-sms")).await;
    assert_eq!(v["chrome"], json!("sms"), "{v}");
    assert_eq!(v["channel"], json!("sms"), "{v}");
    assert_eq!(v["badge"], Value::Null, "a plain SMS has no badge: {v}");
    assert_eq!(v["number"], json!("+1 707-415-3915"), "{v}");
    assert_eq!(
        v["segments"],
        json!("2"),
        "200 GSM-7 characters is two segments: {v}"
    );
    assert_eq!(v["status"], json!("sent"), "{v}");
}

async fn mms(h: &Harness) {
    let root = viewer("viewer-mms");
    wait_for_pictures(h, &root).await;
    let v = snapshot(h, &root).await;
    assert_eq!(v["channel"], json!("mms"), "{v}");
    assert_eq!(v["badge"], json!("MMS"), "{v}");
    assert_eq!(
        v["imgAlt"],
        json!("MMS picture"),
        "a picture without the caller's alt is named for its channel: {v}"
    );
    assert_eq!(v["imgLoaded"], json!(true), "{v}");
    assert_eq!(v["openLabel"], json!("Enlarge picture: MMS picture"), "{v}");
}

async fn broken(h: &Harness) {
    let root = viewer("viewer-broken");
    wait_for_pictures(h, &root).await;
    let v = snapshot(h, &root).await;
    let fallback = s(&v, "fallback");
    assert!(fallback.contains("The picture could not be shown"), "{v}");
    assert!(
        fallback.contains("Saturday clinic flyer"),
        "the tile names the picture: {v}"
    );
    assert_eq!(v["imgAlt"], Value::Null, "{v}");
    assert_eq!(
        v["openLabel"],
        Value::Null,
        "a broken picture is not an enlarge button: {v}"
    );
    assert_eq!(v["status"], json!("failed"), "{v}");
    assert_eq!(
        v["statusTabindex"],
        json!("0"),
        "a status with a reason is focusable: {v}"
    );
    assert_eq!(
        v["reason"],
        json!("Carrier rejected: the number is unreachable"),
        "{v}"
    );
    assert_eq!(
        v["statusTip"],
        json!("Carrier rejected: the number is unreachable"),
        "{v}"
    );
}

async fn spanish(h: &Harness) {
    let v = snapshot(h, &viewer("viewer-spanish")).await;
    assert_eq!(v["badge"], json!("Plantilla"), "{v}");
    assert_eq!(
        v["region"],
        json!("Mensaje tal como lo vio el destinatario"),
        "{v}"
    );
    assert_eq!(
        v["switchLabels"],
        json!(["Idioma", "Imagen", "Contenido"]),
        "{v}"
    );
    assert_eq!(
        v["pressed"]["with-picture:Con imagen"],
        json!("true"),
        "{v}"
    );
    assert_eq!(v["pressed"]["as-sent:Como se envió"], json!("true"), "{v}");
    assert_eq!(v["statusText"], json!("Entregado"), "{v}");
    assert_eq!(
        v["counts"],
        json!("Enviado a 132 · con imagen 120 · solo texto 12"),
        "{v}"
    );
    assert_eq!(v["channelName"], json!("WhatsApp"), "{v}");
    assert!(s(&v, "openLabel").starts_with("Ampliar imagen: "), "{v}");
}

async fn replies(h: &Harness) {
    let v = snapshot(h, &viewer("viewer-replies")).await;
    assert_eq!(
        v["replies"],
        json!([
            ["in", "Yes, Thursday works."],
            ["in", "Should I bring my wife?"]
        ]),
        "{v}"
    );
    assert_eq!(v["status"], json!("no-receipt"), "{v}");
}

/// Whether the lightbox inside `root` (or anywhere when `None`) is open.
async fn lightbox_state(h: &Harness, root: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const d = document.querySelector('{root} dialog[data-viewer-lightbox]');
                const img = d ? d.querySelector('[data-viewer-lightbox-image]') : null;
                return {{ open: d ? d.open : null, alt: img ? img.getAttribute('alt') : null }};
            }})()"#,
            root = root.replace('\'', "\\'")
        ),
    )
    .await
}

async fn enlarge(h: &Harness) {
    let root = viewer("viewer-whatsapp-template");
    click(h, &format!("{root} [data-message-image-open]")).await;
    let open = lightbox_state(h, &root).await;
    assert_eq!(open["open"], json!(true), "{open}");
    assert_eq!(
        open["alt"],
        json!(TEMPLATE_ALT),
        "the enlarged picture keeps its alt: {open}"
    );
    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    settle().await;
    let closed = lightbox_state(h, &root).await;
    assert_eq!(
        closed["open"],
        json!(false),
        "Escape closes the enlarged picture: {closed}"
    );
}

async fn dialog_state(h: &Harness) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const d = document.querySelector('{DIALOG}');
                const labelledBy = d.getAttribute('aria-labelledby');
                const title = labelledBy ? document.getElementById(labelledBy) : null;
                const active = document.activeElement;
                return {{
                    open: d.open,
                    title: title ? title.textContent.trim() : null,
                    focusInside: d.contains(active),
                    focusLabel: active ? active.getAttribute('aria-label') : null,
                    fallback: d.querySelector('[data-media-fallback]') ? true : false,
                    imgAlt: d.querySelector('[data-rich-message] [data-message-header="image"] img')?.getAttribute('alt') ?? null,
                    switches: [...d.querySelectorAll('[data-viewer-switch]')].map((e) => e.getAttribute('data-viewer-switch')),
                }};
            }})()"#
        ),
    )
    .await
}

/// Tag the one VISIBLE match of `selector` and return a selector for it.
/// EntityTable renders a row's cells twice (desktop cells and the compact
/// card, one of them hidden by a breakpoint), so the first match in document
/// order may be the hidden copy: pick by `checkVisibility`, never position.
async fn visible(h: &Harness, selector: &str, tag: &str) -> String {
    let found = eval_json(
        h,
        &format!(
            r#"(() => {{
                const shown = [...document.querySelectorAll('{selector}')]
                    .filter((e) => e.checkVisibility({{ visibilityProperty: true }}));
                if (shown.length !== 1) return shown.length;
                shown[0].setAttribute('data-test-target', '{tag}');
                return 1;
            }})()"#,
            selector = selector.replace('\'', "\\'")
        ),
    )
    .await;
    assert_eq!(found, json!(1), "exactly one visible {selector}");
    format!(r#"[data-test-target="{tag}"]"#)
}

async fn row_action_dialog(h: &Harness) {
    let eye_sel = visible(h, EYE, "eye").await;
    let eye = eval_json(
        h,
        &format!(
            r#"(() => {{ const b = document.querySelector('{sel}'); return [b.getAttribute('data-row-action'), b.closest('[data-tip]')?.getAttribute('data-tip') ?? null]; }})()"#,
            sel = eye_sel.replace('\'', "\\'")
        ),
    )
    .await;
    assert_eq!(
        eye,
        json!(["open", "View message: Consultation reminder"]),
        "the eye action and its tooltip"
    );

    click(h, &eye_sel).await;
    wait_for_pictures(h, DIALOG).await;
    let open = dialog_state(h).await;
    assert_eq!(open["open"], json!(true), "{open}");
    assert_eq!(
        open["title"],
        json!("Consultation reminder"),
        "the visible title names the dialog: {open}"
    );
    assert_eq!(
        open["focusInside"],
        json!(true),
        "focus moves into the dialog: {open}"
    );
    assert_eq!(open["imgAlt"], json!(TEMPLATE_ALT), "{open}");
    assert_eq!(
        open["switches"],
        json!(["language", "picture", "content"]),
        "{open}"
    );

    // Tab stays inside the modal dialog and reaches the variant switches.
    let mut reached_switch = false;
    for _ in 0..12 {
        h.press_key_sequence(&[Key::Tab]).await.expect("Tab");
        let where_ = eval_json(
            h,
            &format!(
                r#"(() => {{ const a = document.activeElement; return {{ inside: document.querySelector('{DIALOG}').contains(a), option: a ? a.getAttribute('data-viewer-option') : null }}; }})()"#
            ),
        )
        .await;
        assert_eq!(
            where_["inside"],
            json!(true),
            "Tab left the modal dialog: {where_}"
        );
        reached_switch |= !where_["option"].is_null();
    }
    assert!(reached_switch, "the keyboard reaches the variant switches");

    // The nested enlarged picture closes on Escape and leaves the dialog open.
    click(h, &format!("{DIALOG} [data-message-image-open]")).await;
    let lightbox = lightbox_state(h, DIALOG).await;
    assert_eq!(lightbox["open"], json!(true), "{lightbox}");
    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    settle().await;
    assert_eq!(lightbox_state(h, DIALOG).await["open"], json!(false));
    assert_eq!(
        dialog_state(h).await["open"],
        json!(true),
        "Escape on the picture leaves the dialog open"
    );

    // A second Escape closes the dialog, and focus returns to the eye.
    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    settle().await;
    let closed = dialog_state(h).await;
    assert_eq!(closed["open"], json!(false), "{closed}");
    assert_eq!(
        closed["focusLabel"],
        json!("View message: Consultation reminder"),
        "focus returns to the trigger: {closed}"
    );

    // Another row: the broken picture, closed by the close button.
    let broken_eye = visible(h, EYE_BROKEN, "eye-broken").await;
    click(h, &broken_eye).await;
    wait_for_pictures(h, DIALOG).await;
    let other = dialog_state(h).await;
    assert_eq!(other["open"], json!(true), "{other}");
    assert_eq!(other["title"], json!("Saturday clinic"), "{other}");
    assert_eq!(other["fallback"], json!(true), "{other}");
    assert_eq!(other["switches"], json!([]), "{other}");
    assert_axe_clean(h, Some(DIALOG), "the open message-viewer dialog").await;
    click(h, &format!("{DIALOG} [data-viewer-close]")).await;
    assert_eq!(
        dialog_state(h).await["open"],
        json!(false),
        "the close button closes the dialog"
    );
}

async fn axe_in_theme(h: &Harness, theme: &str) {
    eval_json(
        h,
        &format!(
            r#"(async () => {{
                document.documentElement.setAttribute('data-theme', '{theme}');
                await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
                return true;
            }})()"#
        ),
    )
    .await;
    for fixture in FIXTURES {
        let scope = format!("[data-testid=\"{fixture}\"]");
        eval_json(
            h,
            &format!(
                r#"(async () => {{
                    document.querySelector('{scope}').scrollIntoView({{ block: 'start' }});
                    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
                    return true;
                }})()"#,
                scope = scope.replace('\'', "\\'")
            ),
        )
        .await;
        assert_axe_clean(h, Some(&scope), &format!("{fixture} ({theme} theme)")).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-message-viewer)"]
async fn sent_messages_show_as_the_recipient_saw_them_and_stay_accessible() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    whatsapp_template(&h).await;
    whatsapp_session(&h).await;
    sms(&h).await;
    mms(&h).await;
    broken(&h).await;
    spanish(&h).await;
    replies(&h).await;
    enlarge(&h).await;
    row_action_dialog(&h).await;

    let original = eval_json(&h, "document.documentElement.getAttribute('data-theme')").await;
    axe_in_theme(&h, "light").await;
    axe_in_theme(&h, "dark").await;
    let restore = match original.as_str() {
        Some(theme) => {
            format!("document.documentElement.setAttribute('data-theme', '{theme}'), true")
        }
        None => "document.documentElement.removeAttribute('data-theme'), true".to_string(),
    };
    eval_json(&h, &restore).await;

    let report = ldui_audit::audit_page(
        &h,
        &ldui_audit::from_ui_tokens(body_font_family(&h).await),
        &Default::default(),
    )
    .await
    .expect("audit the message viewer page");
    assert_eq!(
        report.count(family::COMPONENT_DRIFT),
        0,
        "the message viewer must use design-system components:\n{}",
        report.describe("message viewer showcase")
    );

    assert_no_browser_errors(&h, "message viewer journeys").await;
}
