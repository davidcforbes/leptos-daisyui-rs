//! Real-browser proof for the rich-message components (ldui-g8hh), on the
//! `/components/rich_message` showcase route:
//! - the picture-message preview loads its image header, keeps emoji and
//!   curly quotes byte-for-byte, auto-links the email and website into new
//!   tabs, and reads "Picture message · N chars";
//! - the WhatsApp template renders bold/italic/strike/monospace as elements,
//!   fills the placeholders the host has values for, shows the rest as
//!   unfilled chips (all three syntaxes), and marks the template not
//!   sendable;
//! - preview buttons are focusable, `aria-disabled`, described by the
//!   preview hint, carry no `href`, and pressing one navigates nowhere;
//! - the thread is a labelled list whose failed picture shows the fallback
//!   tile, whose sent messages sit at the end, and whose cut message says so;
//! - host-wired buttons are live and report the press;
//! - the composer indicator switches GSM-7 to UCS-2 when an emoji is typed;
//! - the Spanish table reaches the accessible names and the hint;
//! - every section is axe-clean (WCAG 2.2 AA, through `common::axe`) in the
//!   light AND dark themes, and the page has no component drift.
//!
//! Kept in its own file/xtask step (`cargo xtask test-rich-message`) rather
//! than folded into `reactivity_smoke.rs`, whose check count is pinned.
mod common;

use common::axe::assert_axe_clean;
use common::{
    assert_no_browser_errors, begin_browser_error_capture, body_font_family, click, harness_at,
};
use ldui_audit::family;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const PAGE: &str = "/components/rich_message";
const SECTIONS: [&str; 6] = [
    "rich-sms-preview",
    "rich-whatsapp-preview",
    "rich-thread",
    "rich-live",
    "rich-composer",
    "rich-spanish",
];

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate rich message fixture")
        .into_value()
        .expect("rich message expression returns JSON")
}

/// Poll `expr` until it returns `true` (10 s budget).
async fn wait_until(h: &pixelproof_web::Harness, expr: &str, what: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if eval_json(h, expr).await == json!(true) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn picture_preview(h: &pixelproof_web::Harness) {
    wait_until(
        h,
        r#"(() => { const i = document.querySelector('[data-testid="rich-sms-preview"] img'); return !!i && i.complete && i.naturalWidth > 0; })()"#,
        "the picture header to load",
    )
    .await;
    let snap = eval_json(
        h,
        r#"(() => {
            const root = document.querySelector('[data-testid="rich-sms-preview"] [data-rich-message]');
            const body = root.querySelector('[data-message-body]');
            const links = [...body.querySelectorAll('a')].map(a => ({
                href: a.getAttribute('href'), target: a.target, rel: a.rel,
            }));
            const img = root.querySelector('[data-message-header="image"] img');
            const bubble = root.querySelector('.chat-bubble');
            return {
                label: root.getAttribute('aria-label'),
                channel: root.dataset.channel,
                placement: root.classList.contains('chat-start'),
                body: body.textContent,
                whiteSpace: getComputedStyle(body).whiteSpace,
                links,
                alt: img.getAttribute('alt'),
                imgFits: img.getBoundingClientRect().width <= bubble.getBoundingClientRect().width,
                indicator: root.querySelector('[data-sms-indicator]').textContent,
                fallback: !!root.querySelector('[data-media-fallback]'),
            };
        })()"#,
    )
    .await;
    assert_eq!(
        snap["label"],
        json!("Preview, +1 707-415-3915, 4:41 PM"),
        "{snap}"
    );
    assert_eq!(snap["channel"], json!("mms"), "{snap}");
    assert_eq!(
        snap["placement"],
        json!(true),
        "a preview reads as received: {snap}"
    );
    assert_eq!(snap["fallback"], json!(false), "{snap}");
    assert_eq!(
        snap["whiteSpace"],
        json!("pre-wrap"),
        "line breaks kept: {snap}"
    );
    let body = snap["body"].as_str().unwrap_or_default();
    for exact in [
        "if you haven’t already",
        "\n\n✅ I’m a lifelong Democrat\n\n",
        "please don’t hesitate",
    ] {
        assert!(
            body.contains(exact),
            "body must keep {exact:?} verbatim: {snap}"
        );
    }
    assert_eq!(
        snap["links"],
        json!([
            {"href": "mailto:sylvia@votelemus.com", "target": "_blank", "rel": "noopener noreferrer"},
            {"href": "https://www.VoteLemus.com", "target": "_blank", "rel": "noopener noreferrer"},
        ]),
        "{snap}"
    );
    assert_eq!(
        snap["alt"],
        json!("Campaign flyer: Real Leadership, Proven Experience"),
        "{snap}"
    );
    assert_eq!(
        snap["imgFits"],
        json!(true),
        "image stays inside the bubble: {snap}"
    );
    let indicator = snap["indicator"].as_str().unwrap_or_default();
    assert!(
        indicator.starts_with("Picture message · ") && indicator.ends_with(" chars"),
        "{snap}"
    );
}

async fn whatsapp_preview(h: &pixelproof_web::Harness) {
    let snap = eval_json(
        h,
        r#"(() => {
            const root = document.querySelector('[data-testid="rich-whatsapp-preview"] [data-rich-message]');
            const body = root.querySelector('[data-message-body]');
            const ph = [...root.querySelectorAll('[data-placeholder]')].map(p => ({
                key: p.dataset.placeholder,
                state: p.dataset.placeholderState,
                tag: p.tagName,
                text: p.firstChild ? p.firstChild.textContent : '',
            }));
            return {
                strong: [...body.querySelectorAll('strong')].map(e => e.textContent),
                em: [...body.querySelectorAll('em')].map(e => e.textContent),
                s: [...body.querySelectorAll('s')].map(e => e.textContent),
                mono: [...body.querySelectorAll('.font-mono')].map(e => getComputedStyle(e).fontFamily.length > 0),
                markersLeft: /[*~]|```/.test(body.textContent),
                ph,
                footer: root.querySelector('[data-message-footer]').textContent,
                notSendable: root.querySelector('[data-message-not-sendable]')?.textContent ?? null,
                indicator: !!root.querySelector('[data-sms-indicator]'),
            };
        })()"#,
    )
    .await;
    assert_eq!(snap["strong"], json!(["confirmed", "new"]), "{snap}");
    assert_eq!(snap["em"], json!(["Please bring"]), "{snap}");
    assert_eq!(snap["s"], json!(["old"]), "{snap}");
    assert_eq!(
        snap["markersLeft"],
        json!(false),
        "no raw markers survive: {snap}"
    );
    assert!(
        snap["mono"].as_array().is_some_and(|m| !m.is_empty()),
        "{snap}"
    );
    assert_eq!(
        snap["ph"],
        json!([
            {"key": "first_name", "state": "filled", "tag": "MARK", "text": "María"},
            {"key": "appointment_date", "state": "filled", "tag": "MARK", "text": "Oct 28"},
            {"key": "appointment_time", "state": "unfilled", "tag": "SPAN", "text": "{{appointment_time}}"},
            {"key": "1", "state": "filled", "tag": "MARK", "text": "A-1042"},
            {"key": "Contacts.First Name", "state": "unfilled", "tag": "SPAN", "text": "${Contacts.First Name}"},
        ]),
        "{snap}"
    );
    assert_eq!(
        snap["footer"],
        json!("Diener Law · Reply STOP to opt out"),
        "{snap}"
    );
    assert_eq!(snap["notSendable"], json!("Not sendable"), "{snap}");
    assert_eq!(
        snap["indicator"],
        json!(false),
        "WhatsApp has no SMS indicator: {snap}"
    );
}

async fn preview_buttons_are_inert(h: &pixelproof_web::Harness) {
    let snap = eval_json(
        h,
        r#"(() => {
            const group = document.querySelector('[data-testid="rich-whatsapp-preview"] [data-message-actions]');
            return {
                role: group.getAttribute('role'),
                label: group.getAttribute('aria-label'),
                buttons: [...group.querySelectorAll('button')].map(b => {
                    const hint = document.getElementById(b.getAttribute('aria-describedby') || '');
                    return {
                        kind: b.dataset.messageAction,
                        text: b.textContent.trim(),
                        disabled: b.getAttribute('aria-disabled'),
                        native: b.disabled,
                        hint: hint ? hint.textContent : null,
                        title: b.title,
                        href: b.hasAttribute('href') || !!b.closest('a'),
                        tabIndex: b.tabIndex,
                    };
                }),
            };
        })()"#,
    )
    .await;
    assert_eq!(snap["role"], json!("group"), "{snap}");
    assert_eq!(snap["label"], json!("Message buttons"), "{snap}");
    let hint = "Preview only: this is the recipient's button";
    assert_eq!(
        snap["buttons"],
        json!([
            {"kind": "url", "text": "Visit website", "disabled": "true", "native": false, "hint": hint, "title": "https://example.org/appointments", "href": false, "tabIndex": 0},
            {"kind": "call", "text": "Call now", "disabled": "true", "native": false, "hint": hint, "title": "+17075550100", "href": false, "tabIndex": 0},
            {"kind": "copy-code", "text": "Copy code", "disabled": "true", "native": false, "hint": hint, "title": "CONF-2026", "href": false, "tabIndex": 0},
            {"kind": "quick-reply", "text": "Push to respond", "disabled": "true", "native": false, "hint": hint, "title": "", "href": false, "tabIndex": 0},
        ]),
        "{snap}"
    );

    let before = eval_json(h, "location.href").await;
    for kind in ["url", "call"] {
        click(
            h,
            &format!("[data-testid=\"rich-whatsapp-preview\"] [data-message-action=\"{kind}\"]"),
        )
        .await;
    }
    let after = eval_json(h, "location.href").await;
    assert_eq!(before, after, "an inert preview button must not navigate");
}

async fn thread(h: &pixelproof_web::Harness) {
    wait_until(
        h,
        r#"!!document.querySelector('[data-testid="rich-thread"] [data-media-fallback]')"#,
        "the failed picture's fallback tile",
    )
    .await;
    let snap = eval_json(
        h,
        r#"(() => {
            const list = document.querySelector('[data-testid="rich-thread"] ol[data-rich-message-thread]');
            const items = [...list.children];
            return {
                label: list.getAttribute('aria-label'),
                items: items.map(li => li.tagName),
                messages: items.map(li => {
                    const m = li.querySelector('[data-rich-message]');
                    return {
                        label: m.getAttribute('aria-label'),
                        end: m.classList.contains('chat-end'),
                        indicator: !!m.querySelector('[data-sms-indicator]'),
                    };
                }),
                fallback: list.querySelector('[data-media-fallback]').textContent,
                brokenImg: !!list.querySelector('[data-media-fallback]')?.closest('[data-message-header]')?.querySelector('img'),
                doc: list.querySelector('[data-message-header="document"]').textContent,
                truncated: list.querySelector('[data-message-truncated]').textContent,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        snap["label"],
        json!("Conversation with María López"),
        "{snap}"
    );
    assert_eq!(
        snap["items"],
        json!(["LI", "LI", "LI", "LI", "LI"]),
        "{snap}"
    );
    let ends: Vec<_> = snap["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["end"].clone())
        .collect();
    assert_eq!(
        ends,
        vec![
            json!(true),
            json!(false),
            json!(false),
            json!(true),
            json!(false)
        ],
        "{snap}"
    );
    assert!(
        snap["messages"]
            .as_array()
            .unwrap()
            .iter()
            .all(|m| m["indicator"] == json!(false)),
        "a thread never shows the composer indicator: {snap}"
    );
    assert_eq!(
        snap["messages"][1]["label"],
        json!("Received, +1 707-555-0142, Mon 9:15 AM"),
        "{snap}"
    );
    assert_eq!(
        snap["fallback"],
        json!("Image not availablecampaign-flyer.png"),
        "{snap}"
    );
    assert_eq!(
        snap["brokenImg"],
        json!(false),
        "the broken <img> is replaced: {snap}"
    );
    assert!(
        snap["doc"]
            .as_str()
            .unwrap_or_default()
            .contains("intake-checklist.pdf"),
        "{snap}"
    );
    assert_eq!(
        snap["truncated"],
        json!("Showing 240 of 4310 characters"),
        "{snap}"
    );
}

async fn live_buttons(h: &pixelproof_web::Harness) {
    let snap = eval_json(
        h,
        r#"(() => [...document.querySelectorAll('[data-testid="rich-live"] [data-message-action]')].map(b => ({
            disabled: b.getAttribute('aria-disabled'),
            described: b.hasAttribute('aria-describedby'),
        })))()"#,
    )
    .await;
    assert_eq!(
        snap,
        json!([
            {"disabled": null, "described": false},
            {"disabled": null, "described": false},
        ]),
        "host-wired buttons are live: {snap}"
    );
    click(
        h,
        "[data-testid=\"rich-live\"] [data-message-action=\"quick-reply\"]",
    )
    .await;
    let pressed = eval_json(
        h,
        r#"document.querySelector('[data-testid="rich-live-pressed"]').textContent"#,
    )
    .await;
    assert_eq!(pressed, json!("Pressed: Push to respond"));
}

async fn composer(h: &pixelproof_web::Harness) {
    let read = r#"(() => { const p = document.querySelector('[data-testid="rich-composer"] [data-sms-indicator]'); return { enc: p.dataset.smsEncoding, segs: p.dataset.smsSegments, text: p.textContent }; })()"#;
    let before = eval_json(h, read).await;
    assert_eq!(before["enc"], json!("gsm7"), "{before}");
    assert_eq!(before["segs"], json!("1"), "{before}");
    eval_json(
        h,
        r#"(() => {
            const t = document.querySelector('[data-testid="rich-composer"] textarea');
            t.value = t.value + ' ✅';
            t.dispatchEvent(new Event('input', { bubbles: true }));
            return true;
        })()"#,
    )
    .await;
    wait_until(
        h,
        r#"document.querySelector('[data-testid="rich-composer"] [data-sms-indicator]').dataset.smsEncoding === 'ucs2'"#,
        "the indicator to switch to UCS-2",
    )
    .await;
    let after = eval_json(h, read).await;
    assert!(
        after["text"]
            .as_str()
            .unwrap_or_default()
            .starts_with("UCS-2 · 1 segment · "),
        "{after}"
    );
}

async fn spanish(h: &pixelproof_web::Harness) {
    let snap = eval_json(
        h,
        r#"(() => {
            const root = document.querySelector('[data-testid="rich-spanish"] [data-rich-message]');
            const b = root.querySelector('[data-message-action]');
            return {
                label: root.getAttribute('aria-label'),
                hint: document.getElementById(b.getAttribute('aria-describedby')).textContent,
                group: root.querySelector('[data-message-actions]').getAttribute('aria-label'),
                indicator: root.querySelector('[data-sms-indicator]').textContent,
                chip: root.querySelector('[data-placeholder-state="unfilled"]').textContent,
            };
        })()"#,
    )
    .await;
    assert_eq!(snap["label"], json!("Vista previa"), "{snap}");
    assert_eq!(snap["group"], json!("Botones del mensaje"), "{snap}");
    assert_eq!(
        snap["hint"],
        json!("Solo vista previa: es el botón del destinatario"),
        "{snap}"
    );
    assert!(
        snap["indicator"]
            .as_str()
            .unwrap_or_default()
            .ends_with(" caracteres"),
        "{snap}"
    );
    assert_eq!(
        snap["chip"],
        json!("{{first_name}} (sin completar)"),
        "{snap}"
    );
}

async fn axe_in_theme(h: &pixelproof_web::Harness, theme: &str) {
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
    for section in SECTIONS {
        let scope = format!("[data-testid=\"{section}\"]");
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
        assert_axe_clean(h, Some(&scope), &format!("{section} ({theme} theme)")).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-rich-message)"]
async fn rich_messages_render_what_the_recipient_sees_and_stay_accessible() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    picture_preview(&h).await;
    whatsapp_preview(&h).await;
    preview_buttons_are_inert(&h).await;
    thread(&h).await;
    live_buttons(&h).await;
    composer(&h).await;
    spanish(&h).await;

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
    .expect("audit the rich message page");
    assert_eq!(
        report.count(family::COMPONENT_DRIFT),
        0,
        "rich messages must use design-system components:\n{}",
        report.describe("rich message showcase")
    );

    assert_no_browser_errors(&h, "rich message journeys").await;
}
