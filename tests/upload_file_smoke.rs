//! Real-browser proof for `UploadFile`'s caller-supplied strings and the
//! `FileDropZone` (ldui-1ff4): the Spanish strings a caller passes render
//! where English used to be hard-coded (and the default is still that
//! English), a dropped `application/pdf` reaches `on_files` while a file
//! outside the accept list goes to `on_rejected`, the zone is one keyboard
//! tab stop named by its prompt that opens the picker on Enter and Space,
//! it highlights while a drag is over it, and the fixture is axe-clean.
//!
//! Drives the general demo app (`html_target: None`, like
//! `collapse_naming_smoke.rs`) on the `/components/upload_file` showcase
//! route. Kept in its own file/xtask step (`cargo xtask test-upload-file`)
//! rather than folded into `reactivity_smoke.rs`, whose check count is pinned.
mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, harness_at};
use serde_json::{Value, json};

const PAGE: &str = "/components/upload_file";
const FIXTURE: &str = "#upload-file-fixture";
const ZONE: &str = r#"document.querySelector('[data-testid="upload-file-drop"] [role="button"]')"#;

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate upload-file fixture")
        .into_value()
        .expect("upload-file expression returns JSON")
}

fn text(v: &Value, key: &str) -> String {
    v[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} is not a string: {v}"))
        .to_string()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-upload-file)"]
async fn strings_come_from_the_caller_and_a_dropped_pdf_reaches_the_callback() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    // Strings: the default is the old English; a caller's Spanish replaces it
    // on both components.
    let texts = eval_json(
        &h,
        r#"(() => {
            const t = (id) => document.querySelector(`[data-testid="${id}"]`).textContent;
            return {
                english: t('upload-file-english'),
                spanish: t('upload-file-spanish'),
                drop: t('upload-file-drop'),
            };
        })()"#,
    )
    .await;
    assert!(
        text(&texts, "english").contains("Click to upload"),
        "{texts}"
    );
    assert!(
        text(&texts, "spanish").contains("Haga clic para subir"),
        "{texts}"
    );
    assert!(
        !text(&texts, "spanish").contains("Click to upload"),
        "{texts}"
    );
    assert!(
        text(&texts, "drop").contains("Suelte el documento aquí o pulse Intro para elegirlo"),
        "{texts}"
    );
    assert!(!text(&texts, "drop").contains("Drop files here"), "{texts}");

    // The zone is one tab stop named by its own prompt; the hidden input is
    // not a second one.
    let shape = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const zone = {ZONE};
                const labelledBy = zone.getAttribute('aria-labelledby');
                const named = labelledBy ? document.getElementById(labelledBy) : null;
                const input = document.querySelector('[data-testid="upload-file-drop"] input[type="file"]');
                return {{
                    tabindex: zone.getAttribute('tabindex'),
                    labelledBy,
                    name: named ? named.textContent.trim() : null,
                    focusRing: zone.classList.contains('ld-focus-ring'),
                    inputTabindex: input.getAttribute('tabindex'),
                    inputAccept: input.getAttribute('accept'),
                    inputMultiple: input.hasAttribute('multiple'),
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(shape["tabindex"], json!("0"), "{shape}");
    assert!(
        shape["labelledBy"]
            .as_str()
            .is_some_and(|id| id.starts_with("ld-file-drop-")),
        "{shape}"
    );
    assert_eq!(
        shape["name"],
        json!("Suelte el documento aquí o pulse Intro para elegirlo"),
        "{shape}"
    );
    assert_eq!(shape["focusRing"], json!(true), "{shape}");
    assert_eq!(shape["inputTabindex"], json!("-1"), "{shape}");
    assert_eq!(
        shape["inputAccept"],
        json!("application/pdf,.png"),
        "{shape}"
    );
    assert_eq!(shape["inputMultiple"], json!(true), "{shape}");

    // Keyboard: the zone takes focus, and Enter or Space opens the picker
    // (the hidden input's `click`, stubbed here because a real picker would
    // block the headless page).
    let keyboard = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const zone = {ZONE};
                const input = document.querySelector('[data-testid="upload-file-drop"] input[type="file"]');
                let clicks = 0;
                input.click = () => {{ clicks += 1; }};
                zone.focus();
                const focused = document.activeElement === zone;
                const prevented = [];
                for (const key of ['Enter', ' ', 'a']) {{
                    const ev = new KeyboardEvent('keydown', {{ key, bubbles: true, cancelable: true }});
                    zone.dispatchEvent(ev);
                    prevented.push(ev.defaultPrevented);
                }}
                return {{ focused, clicks, prevented }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(keyboard["focused"], json!(true), "{keyboard}");
    assert_eq!(keyboard["clicks"], json!(2), "{keyboard}");
    assert_eq!(
        keyboard["prevented"],
        json!([true, true, false]),
        "{keyboard}"
    );

    // Drag over highlights and is accepted (default prevented, or the browser
    // would refuse the drop). The highlight is a signal the renderer applies
    // on its next tick, so it is read after a settle beat, not in the same
    // synchronous block as the dispatch.
    let over = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const zone = {ZONE};
                const dt = new DataTransfer();
                dt.items.add(new File(['%PDF-1.4'], 'scan.pdf', {{ type: 'application/pdf' }}));
                dt.items.add(new File(['hello'], 'notes.txt', {{ type: 'text/plain' }}));
                window.__lduiDropData = dt;
                const over = new DragEvent('dragover', {{ dataTransfer: dt, bubbles: true, cancelable: true }});
                zone.dispatchEvent(over);
                return {{ overPrevented: over.defaultPrevented }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(over["overPrevented"], json!(true), "{over}");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let highlighted = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const zone = {ZONE};
                return {{
                    dragging: zone.getAttribute('data-dragging'),
                    primary: zone.classList.contains('border-primary'),
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(highlighted["dragging"], json!("true"), "{highlighted}");
    assert_eq!(highlighted["primary"], json!(true), "{highlighted}");

    // The drop hands the PDF to `on_files` and the text file to
    // `on_rejected`, then the highlight clears.
    let dropped = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const zone = {ZONE};
                const drop = new DragEvent('drop', {{
                    dataTransfer: window.__lduiDropData, bubbles: true, cancelable: true,
                }});
                zone.dispatchEvent(drop);
                return {{ dropPrevented: drop.defaultPrevented }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(dropped["dropPrevented"], json!(true), "{dropped}");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let received = eval_json(
        &h,
        &format!(
            r#"(() => ({{
                received: [...document.querySelectorAll('[data-testid="upload-file-drop-received"] li')]
                    .map((li) => li.textContent),
                rejected: document.querySelector('[data-testid="upload-file-drop-rejected"]').textContent,
                zoneText: {ZONE}.textContent,
                stillHighlighted: {ZONE}.hasAttribute('data-dragging'),
            }}))()"#
        ),
    )
    .await;
    assert_eq!(received["received"], json!(["scan.pdf"]), "{received}");
    assert_eq!(received["rejected"], json!("1"), "{received}");
    assert_eq!(received["stillHighlighted"], json!(false), "{received}");
    assert!(
        text(&received, "zoneText").contains("Seleccionado: scan.pdf"),
        "{received}"
    );

    // What a consumer's form audit sees: zero blocking axe findings in the
    // fixture, Spanish strings and hidden input included.
    common::axe::assert_axe_clean(&h, Some(FIXTURE), "the upload-file fixture").await;

    assert_no_browser_errors(&h, "upload file strings and drop zone").await;
}
