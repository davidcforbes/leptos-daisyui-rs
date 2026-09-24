//! Real-browser proof for `patterns::Helpdesk` (both roles on one document,
//! in-memory backend). See doc/plans/2026-09-14-helpdesk-composite-design.md §8.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use pixelproof_web::Key;
use serde_json::{Value, json};

const PAGE: &str = "/helpdesk-fixture";
const SUPPORT: &str = "#helpdesk-support";
const REQUESTER: &str = "#helpdesk-requester";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate helpdesk fixture")
        .into_value()
        .expect("helpdesk expression returns JSON")
}

/// Read the backend's call log via the shared `window.__APP_DEBUG__.state()`
/// oracle (`demo/src/debug.rs`), where `helpdesk_calls` is registered.
async fn backend_calls(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const raw = window.__APP_DEBUG__ ? window.__APP_DEBUG__.state() : '{}';
            const state = JSON.parse(raw);
            return state.helpdesk_calls ?? [];
        })()"#,
    )
    .await
}

async fn snapshot(h: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{root}');
                const bucketCount = id => root
                    .querySelector('[data-selectable-summary-card="' + id + '"] [data-selectable-summary-count]')
                    ?.textContent?.trim() ?? null;
                return {{
                    role: root.querySelector('[data-helpdesk]').getAttribute('data-helpdesk-role'),
                    open: bucketCount('open'),
                    inProgress: bucketCount('in-progress'),
                    done: bucketCount('done'),
                    rows: root.querySelectorAll('[data-helpdesk-table] tbody tr').length,
                    keys: Array.from(root.querySelectorAll('[data-helpdesk-table] tbody tr'))
                        .map(r => r.textContent.match(/OF-\d+/)?.[0] ?? null),
                    assigneeFilter: root.querySelector('[data-helpdesk-filter-assignee]') !== null,
                    mineOnly: root.querySelector('[data-helpdesk-mine-only]') !== null,
                    drawerOpen: root.querySelector('[data-helpdesk-drawer]')?.getAttribute('data-helpdesk-drawer-open') ?? null,
                    drawerKey: root.querySelector('[data-helpdesk-drawer-header]')?.textContent?.match(/OF-\d+/)?.[0] ?? null,
                    transition: root.querySelector('[data-helpdesk-transition]') !== null,
                    feedbackState: root.querySelector('[data-helpdesk-action-feedback]')?.getAttribute('data-helpdesk-action-feedback-state') ?? null,
                    comments: root.querySelectorAll('[data-helpdesk-comment]').length,
                    notConfigured: root.querySelector('[data-helpdesk-not-configured]') !== null,
                    submitPresent: root.querySelector('[data-helpdesk-submit]') !== null,
                    submitDisabled: root.querySelector('[data-helpdesk-submit]')?.disabled ?? null,
                    images: root.querySelectorAll('[data-image-attachment-item]').length,
                    imageStatus: root.querySelector('[data-image-attachment-status]')?.textContent?.trim() ?? null,
                    imagesOffered: root.querySelector('[data-helpdesk-images]') !== null,
                    summaryFocused: document.activeElement === root.querySelector('[data-helpdesk-summary]'),
                    launcherFocused: document.activeElement === root.querySelector('[data-helpdesk-new-request]'),
                }};
            }})()"#
        ),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn buckets_count_the_seed_and_filter_rows() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(
        (
            s["open"].as_str(),
            s["inProgress"].as_str(),
            s["done"].as_str()
        ),
        (Some("5"), Some("4"), Some("3")),
        "{s}"
    );
    assert_eq!(s["rows"], json!(12));
    click(
        &h,
        &format!("{SUPPORT} [data-selectable-summary-card=\"open\"]"),
    )
    .await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(
        s["rows"],
        json!(5),
        "open bucket filters to New-category rows: {s}"
    );
    click(
        &h,
        &format!("{SUPPORT} [data-selectable-summary-card=\"open\"]"),
    )
    .await;
    assert_eq!(
        snapshot(&h, SUPPORT).await["rows"],
        json!(12),
        "second click clears the bucket"
    );
    assert_no_browser_errors(&h, "buckets").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn requester_sees_only_own_tickets_and_no_support_controls() {
    let h = harness_at(PAGE).await;
    wait_for_selector(&h, &format!("{REQUESTER} [data-helpdesk-state=\"ready\"]")).await;
    let r = snapshot(&h, REQUESTER).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(r["role"], json!("requester"));
    assert_eq!(
        r["rows"],
        json!(6),
        "seed has six tickets requested by SEED_ME: {r}"
    );
    assert!(
        !r["assigneeFilter"].as_bool().unwrap() && !r["mineOnly"].as_bool().unwrap(),
        "{r}"
    );
    assert!(
        s["assigneeFilter"].as_bool().unwrap() && s["mineOnly"].as_bool().unwrap(),
        "negative control: {s}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn row_activation_opens_the_drawer_and_escape_closes_it() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(
        &h,
        &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child"),
    )
    .await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-drawer-header]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["drawerOpen"], json!("true"));
    assert!(s["drawerKey"].as_str().unwrap().starts_with("OF-"), "{s}");
    assert_eq!(
        snapshot(&h, REQUESTER).await["drawerOpen"],
        json!("false"),
        "negative control"
    );
    eval_json(
        &h,
        &format!(
            "(() => {{ const a = document.querySelector('{SUPPORT} [data-helpdesk-drawer] aside'); \
             a.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Escape', bubbles: true }})); return true; }})()"
        ),
    )
    .await;
    assert_eq!(snapshot(&h, SUPPORT).await["drawerOpen"], json!("false"));
    assert_no_browser_errors(&h, "drawer").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn support_transition_writes_and_requester_has_no_selects() {
    let h = harness_at(PAGE).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(
        &h,
        &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child"),
    )
    .await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-transition]")).await;
    eval_json(
        &h,
        &format!(
            "(() => {{ const s = document.querySelector('{SUPPORT} [data-helpdesk-transition]'); \
             s.value = '10003'; s.dispatchEvent(new Event('change', {{ bubbles: true }})); return true; }})()"
        ),
    )
    .await;
    wait_for_selector(
        &h,
        &format!("{SUPPORT} [data-helpdesk-action-feedback-state=\"success\"]"),
    )
    .await;
    let calls = backend_calls(&h).await;
    let calls_str = calls.to_string();
    assert!(
        calls_str.contains("Transition"),
        "backend saw the transition: {calls_str}"
    );
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["done"], json!("4"), "Done bucket grew by one: {s}");

    click(
        &h,
        &format!("{REQUESTER} [data-helpdesk-table] tbody tr:first-child"),
    )
    .await;
    wait_for_selector(&h, &format!("{REQUESTER} [data-helpdesk-drawer-header]")).await;
    assert_eq!(
        snapshot(&h, REQUESTER).await["transition"],
        json!(false),
        "requester drawer has no status select"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn failed_write_reverts_and_reports() {
    let h = harness_at(&format!("{PAGE}-fail-writes")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(
        &h,
        &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child"),
    )
    .await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-transition]")).await;
    let before = snapshot(&h, SUPPORT).await;
    eval_json(
        &h,
        &format!(
            "(() => {{ const s = document.querySelector('{SUPPORT} [data-helpdesk-transition]'); \
             s.value = '10003'; s.dispatchEvent(new Event('change', {{ bubbles: true }})); return true; }})()"
        ),
    )
    .await;
    wait_for_selector(
        &h,
        &format!("{SUPPORT} [data-helpdesk-action-feedback-state=\"error\"]"),
    )
    .await;
    let after = snapshot(&h, SUPPORT).await;
    assert_eq!(
        after["done"], before["done"],
        "bucket counts unchanged after a refused write"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn comments_append_in_both_roles() {
    let h = harness_at(PAGE).await;
    for root in [SUPPORT, REQUESTER] {
        wait_for_selector(&h, &format!("{root} [data-helpdesk-state=\"ready\"]")).await;
        click(
            &h,
            &format!("{root} [data-helpdesk-table] tbody tr:first-child"),
        )
        .await;
        wait_for_selector(&h, &format!("{root} [data-helpdesk-comment-input]")).await;
        let before = snapshot(&h, root).await["comments"].as_u64().unwrap();
        eval_json(
            &h,
            &format!(
                "(() => {{ const t = document.querySelector('{root} [data-helpdesk-comment-input]'); \
                 t.value = 'hello from {root}'; t.dispatchEvent(new Event('input', {{ bubbles: true }})); return true; }})()"
            ),
        )
        .await;
        click(&h, &format!("{root} [data-helpdesk-comment-submit]")).await;
        wait_for_selector(
            &h,
            &format!("{root} [data-helpdesk-action-feedback-state=\"success\"]"),
        )
        .await;
        assert_eq!(
            snapshot(&h, root).await["comments"].as_u64().unwrap(),
            before + 1,
            "{root}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn new_request_validates_pastes_and_files() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-summary]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(
        s["submitDisabled"],
        json!(true),
        "blank summary disables submit: {s}"
    );

    let paste = |root: &str, name: &str, bytes: &str, mime: &str| {
        format!(
            r#"(() => {{
                const bytes = new Uint8Array([{bytes}]);
                const file = new File([bytes], '{name}', {{ type: '{mime}' }});
                const dt = new DataTransfer(); dt.items.add(file);
                const zone = document.querySelector('{root} [data-image-attachment-dropzone]');
                zone.dispatchEvent(new ClipboardEvent('paste', {{ clipboardData: dt, bubbles: true }}));
                return true;
            }})()"#
        )
    };
    const PNG: &str = "0x89,0x50,0x4E,0x47,0x0D,0x0A,0x1A,0x0A,0,0,0,0";
    const JPEG: &str = "0xFF,0xD8,0xFF,0xE0,0,0,0,0,0,0,0,0";
    eval_json(&h, &paste(SUPPORT, "shot.png", PNG, "image/png")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-image-attachment-item]")).await;
    assert_eq!(snapshot(&h, SUPPORT).await["images"], json!(1));
    // Declared type is image/png, but the bytes are plain text: sniffing rejects it.
    // Polled rather than a fixed sleep: `read_file_bytes` round-trips through a
    // FileReader promise, whose latency isn't bounded under a loaded browser.
    eval_json(&h, &paste(SUPPORT, "notes.txt", "104,105", "image/png")).await;
    wait_for_selector(
        &h,
        &format!("{SUPPORT} [data-image-attachment-status]:not(:empty)"),
    )
    .await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(
        s["images"],
        json!(1),
        "sniffing rejects non-image bytes: {s}"
    );
    assert!(
        s["imageStatus"].as_str().unwrap().contains("notes.txt"),
        "{s}"
    );
    // A .png name over JPEG magic bytes is admitted as image/jpeg (sniffed, not declared).
    eval_json(&h, &paste(SUPPORT, "photo.png", JPEG, "image/png")).await;
    wait_for_selector(
        &h,
        &format!(
            "{SUPPORT} [data-image-attachment-list] [data-image-attachment-item]:nth-of-type(2)"
        ),
    )
    .await;
    assert_eq!(
        snapshot(&h, SUPPORT).await["images"],
        json!(2),
        "JPEG bytes under a .png name are admitted"
    );

    eval_json(
        &h,
        &format!(
            "(() => {{ const i = document.querySelector('{SUPPORT} [data-helpdesk-summary]'); \
             i.value = 'Board is blank'; i.dispatchEvent(new Event('input', {{ bubbles: true }})); return true; }})()"
        ),
    )
    .await;
    assert_eq!(snapshot(&h, SUPPORT).await["submitDisabled"], json!(false));
    click(&h, &format!("{SUPPORT} [data-helpdesk-submit]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-toast]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["keys"][0], json!("OF-13"), "new ticket is first: {s}");
    let calls = backend_calls(&h).await.to_string();
    assert!(
        calls.contains("images: 2"),
        "create carried both images: {calls}"
    );
    assert_no_browser_errors(&h, "new request").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn not_configured_shows_panel_and_no_submit() {
    let h = harness_at(&format!("{PAGE}-not-configured")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-not-configured]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert!(
        s["notConfigured"].as_bool().unwrap() && !s["submitPresent"].as_bool().unwrap(),
        "{s}"
    );
}

/// ldui-efuf: a dialog that opens without taking focus is unusable by
/// keyboard even when every label is right. Open moves focus to the summary
/// field; Escape closes and hands it back to the launcher. Both are waited
/// for by `:focus`, never inferred from a timer.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn opening_the_dialog_focuses_the_summary_and_escape_returns_focus() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-summary]:focus")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(
        s["summaryFocused"],
        json!(true),
        "open focuses the summary: {s}"
    );
    assert_eq!(
        s["imagesOffered"],
        json!(true),
        "the default offers attachments: {s}"
    );

    // A real key press: Modal answers the native `cancel` event, which a
    // synthetic keydown never fires.
    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-new-request]:focus")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(
        s["launcherFocused"],
        json!(true),
        "close returns focus to the launcher: {s}"
    );
    assert_no_browser_errors(&h, "dialog focus").await;
}

/// ldui-8tlg: a host whose backend takes no images withholds the control
/// instead of refusing at submit. The variant is a pathname suffix, like
/// every other fixture here.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn a_host_can_withhold_attachments() {
    let h = harness_at(&format!("{PAGE}-no-attachments")).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-summary]:focus")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(
        s["imagesOffered"],
        json!(false),
        "attachments withheld: {s}"
    );
    assert_eq!(
        s["submitPresent"],
        json!(true),
        "the rest of the form is intact: {s}"
    );
    assert_no_browser_errors(&h, "no attachments").await;
}

/// ldui-ftdy: entitlements belong to the host. Given a table, the composite
/// renders it and imposes nothing of its own: a Requester handed
/// `{ scope: All, triage: false, comment: true }` sees every ticket the
/// Support section sees, gets the comment box, and gets no triage select.
/// The default route is the negative control (the requester there sees six).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn a_host_owned_capability_table_overrides_the_role() {
    let h = harness_at(&format!("{PAGE}-read-all")).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{REQUESTER} [data-helpdesk-state=\"ready\"]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    let r = snapshot(&h, REQUESTER).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(
        r["role"],
        json!("requester"),
        "the mode is still the requester's: {r}"
    );
    assert_eq!(
        r["rows"], s["rows"],
        "scope All: the requester lists every ticket support does: {r} vs {s}"
    );
    assert!(
        r["rows"].as_u64().unwrap_or(0) > 6,
        "and that is more than the six SEED_ME filed: {r}"
    );
    assert!(
        !r["assigneeFilter"].as_bool().unwrap() && !r["mineOnly"].as_bool().unwrap(),
        "the rest of the table is the requester default: {r}"
    );

    click(
        &h,
        &format!("{REQUESTER} [data-helpdesk-table] tbody tr:first-child"),
    )
    .await;
    wait_for_selector(&h, &format!("{REQUESTER} [data-helpdesk-drawer-header]")).await;
    wait_for_selector(&h, &format!("{REQUESTER} [data-helpdesk-comment-input]")).await;
    let opened = snapshot(&h, REQUESTER).await;
    assert_eq!(
        opened["transition"],
        json!(false),
        "triage false: no status select even with every ticket visible: {opened}"
    );
    assert_no_browser_errors(&h, "host-owned capabilities").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn axe_clean_with_drawer_and_dialog_open() {
    let h = harness_at(PAGE).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(
        &h,
        &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child"),
    )
    .await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-drawer-header]")).await;
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(h.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("helpdesk drawer open")
        .unwrap_or_else(|e| {
            panic!(
                "{e}; {}\nviolations: {:#?}",
                report.summary(),
                report.violations
            )
        });
    click(&h, &format!("{SUPPORT} [data-helpdesk-drawer-close]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-summary]")).await;
    let report = axe.run(h.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("helpdesk dialog open")
        .unwrap_or_else(|e| {
            panic!(
                "{e}; {}\nviolations: {:#?}",
                report.summary(),
                report.violations
            )
        });
}

/// Poll until the rendered row count of `root`'s table satisfies `pred`
/// (the same 60 s budget as `wait_for_selector`; the harness exposes no
/// `wait_for_function`).
async fn wait_for_row_count(
    h: &pixelproof_web::Harness,
    root: &str,
    mut pred: impl FnMut(u64) -> bool,
) -> u64 {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        let n = snapshot(h, root).await["rows"]
            .as_u64()
            .expect("rows is a count");
        if pred(n) {
            return n;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{root} row count never satisfied the predicate (last {n})"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
/// The ticket table carries the standard record-list affordances on BOTH
/// roles (ldui-ei8v; Office op-ulgfu): the framework filter row beneath the
/// header row, the gear-glyph column-chooser trigger, and the `+` toolbar
/// action. Control ids are namespaced per role so the two composites on one
/// document never collide.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn table_has_filter_row_gear_chooser_and_add_row_action() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    for (root, prefix) in [
        (SUPPORT, "helpdesk-support"),
        (REQUESTER, "helpdesk-requester"),
    ] {
        wait_for_selector(&h, &format!("{root} [data-helpdesk-state=\"ready\"]")).await;
        // The filter row renders one option filter (status, namespaced id)
        // and one text filter (summary) beneath the header row.
        wait_for_selector(
            &h,
            &format!(
                "{root} [data-entity-column-filter-row=\"true\"] select#{prefix}-status-filter"
            ),
        )
        .await;
        wait_for_selector(&h, &format!("{root} input#{prefix}-summary-filter")).await;
        // The chooser trigger renders as the compact gear glyph, not text.
        assert_eq!(
            eval_json(
                &h,
                &format!(
                    "document.querySelector('{root} [data-entity-column-chooser-presentation]')\
                     ?.getAttribute('data-entity-column-chooser-presentation')"
                )
            )
            .await,
            json!("icon"),
            "{root} chooser renders the gear glyph"
        );
        // The `+` action lives in the table's toolbar slot.
        wait_for_selector(&h, &format!("{root} [data-helpdesk-add-row]")).await;
    }
    assert_no_browser_errors(&h, "standard table features").await;
}

/// The filter row actually narrows rows, and the `+` action opens the New
/// Request dialog (ldui-ei8v).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn filter_row_narrows_rows_and_add_row_opens_the_dialog() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    let before = snapshot(&h, SUPPORT).await["rows"]
        .as_u64()
        .expect("rows is a count");
    assert!(before > 1, "seed renders several rows");

    // Pick the first concrete status option, apply it, and require every
    // visible row to carry that status's label with fewer rows than before.
    let label = eval_json(
        &h,
        &format!(
            "(() => {{ const s = document.querySelector('{SUPPORT} select#helpdesk-support-status-filter'); \
             const opt = Array.from(s.options).find(o => o.value !== ''); \
             s.value = opt.value; s.dispatchEvent(new Event('change', {{ bubbles: true }})); return opt.textContent; }})()"
        ),
    )
    .await;
    let label = label.as_str().expect("a concrete status option").to_owned();
    let shown = wait_for_row_count(&h, SUPPORT, |n| n < before).await;
    let s = snapshot(&h, SUPPORT).await;
    assert!(shown > 0, "status filter kept some rows: {s}");
    let all_match = eval_json(
        &h,
        &format!(
            "(() => Array.from(document.querySelectorAll('{SUPPORT} [data-helpdesk-table] tbody tr'))\
             .every(r => r.textContent.includes({:?})))()",
            label
        ),
    )
    .await;
    assert_eq!(
        all_match,
        json!(true),
        "every visible row is status {label:?}"
    );

    // Clearing the filter restores the full list.
    eval_json(
        &h,
        &format!(
            "(() => {{ const s = document.querySelector('{SUPPORT} select#helpdesk-support-status-filter'); \
             s.value = ''; s.dispatchEvent(new Event('change', {{ bubbles: true }})); return true; }})()"
        ),
    )
    .await;
    let restored = wait_for_row_count(&h, SUPPORT, |n| n == before).await;
    assert_eq!(restored, before, "clearing the filter restores every row");

    // The `+` toolbar action opens the New Request dialog in BOTH roles.
    for root in [SUPPORT, REQUESTER] {
        click(&h, &format!("{root} [data-helpdesk-add-row]")).await;
        wait_for_selector(&h, &format!("{root} [data-helpdesk-summary]")).await;
        click(&h, &format!("{root} [data-helpdesk-cancel]")).await;
    }
    assert_no_browser_errors(&h, "filter row + add row").await;
}

/// The assignee picker's observable state inside `root` (ldui-purt).
async fn assign_snapshot(h: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{root}');
                const box = root.querySelector('[data-confirmable-search-picker-dialog]');
                const dialog = box ? box.closest('dialog') : null;
                const rows = Array.from(root.querySelectorAll(
                    '[data-confirmable-search-picker-results] [data-result-key]'));
                const count = root.querySelector('[data-confirmable-search-picker-count]');
                return {{
                    dialogOpen: dialog ? dialog.open : null,
                    rows: rows.map(r => r.textContent.trim()),
                    keys: rows.map(r => r.getAttribute('data-result-key')),
                    count: count?.textContent?.trim() ?? null,
                    countLive: count?.getAttribute('aria-live') ?? null,
                    selectedKey: root.querySelector('[data-confirmable-search-picker-summary]')
                        ?.getAttribute('data-selected-key') ?? null,
                    searchFocused: document.activeElement ===
                        root.querySelector('[data-confirmable-search-picker-search]'),
                    changeFocused: document.activeElement ===
                        root.querySelector('[data-helpdesk-assign-open]'),
                    assignee: root.querySelector('[data-helpdesk-assignee-name]')
                        ?.textContent?.trim() ?? null,
                    changePresent: root.querySelector('[data-helpdesk-assign-open]') !== null,
                    unavailable: root.querySelector('[data-helpdesk-assign-unavailable]')
                        ?.textContent?.trim() ?? null,
                    nativeSelect: root.querySelector('select[data-helpdesk-assign]') !== null,
                    drawerOpen: root.querySelector('[data-helpdesk-drawer]')
                        ?.getAttribute('data-helpdesk-drawer-open') ?? null,
                }};
            }})()"#
        ),
    )
    .await
}

/// Type into the picker's search field the way a user does: set the value,
/// then fire `input`, which the controlled query listens to.
async fn set_assign_query(h: &pixelproof_web::Harness, root: &str, text: &str) {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const input = document.querySelector('{root} [data-confirmable-search-picker-search]');
                input.value = '{text}';
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                return true;
            }})()"#
        ),
    )
    .await;
}

/// Poll until `root`'s picker renders exactly `n` result rows (the harness
/// exposes no `wait_for_function`; same 60 s budget as `wait_for_selector`).
async fn wait_for_result_rows(h: &pixelproof_web::Harness, root: &str, n: usize) {
    for _ in 0..600 {
        let rows = assign_snapshot(h, root).await["rows"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(usize::MAX);
        if rows == n {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!(
        "picker never rendered {n} rows: {}",
        assign_snapshot(h, root).await
    );
}

async fn open_support_drawer(h: &pixelproof_web::Harness) {
    wait_for_selector(h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(
        h,
        &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child"),
    )
    .await;
    wait_for_selector(h, &format!("{SUPPORT} [data-helpdesk-assign]")).await;
}

/// ldui-purt, Office op-ne1h9: the ~300-person native select became a
/// searchable, sorted, keyboard-operable confirmable picker. The seed
/// directory is deliberately unsorted with a lower-case name, so the order
/// asserted here exists only if the component sorted it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn assignee_picker_sorts_narrows_announces_and_assigns() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    open_support_drawer(&h).await;
    let closed = assign_snapshot(&h, SUPPORT).await;
    assert_eq!(
        closed["nativeSelect"],
        json!(false),
        "no native select: {closed}"
    );

    click(&h, &format!("{SUPPORT} [data-helpdesk-assign-open]")).await;
    wait_for_selector(
        &h,
        &format!("{SUPPORT} [data-confirmable-search-picker-search]:focus"),
    )
    .await;
    let opened = assign_snapshot(&h, SUPPORT).await;
    assert_eq!(opened["dialogOpen"], json!(true), "{opened}");
    assert_eq!(
        opened["rows"],
        json!([
            "Unassigned",
            "ana Ruiz",
            "Ben Adler",
            "Chris Forbes",
            "Dana Torres",
            "Zoe Park"
        ]),
        "unassigned first, then people sorted case-insensitively: {opened}"
    );
    assert_eq!(opened["count"], json!("5 people match"), "{opened}");
    assert_eq!(opened["countLive"], json!("polite"), "{opened}");

    set_assign_query(&h, SUPPORT, "ar").await;
    wait_for_result_rows(&h, SUPPORT, 1).await;
    let narrowed = assign_snapshot(&h, SUPPORT).await;
    assert_eq!(
        narrowed["rows"],
        json!(["Zoe Park"]),
        "typing narrows: {narrowed}"
    );
    assert_eq!(
        narrowed["count"],
        json!("1 person matches"),
        "the narrowed count is announced: {narrowed}"
    );

    // Arrow keys work from the search field and move the selection only.
    h.press_key_sequence(&[Key::ArrowDown])
        .await
        .expect("ArrowDown");
    wait_for_selector(
        &h,
        &format!(
            "{SUPPORT} [data-confirmable-search-picker-summary][data-selected-key=\"person:w-zoe\"]"
        ),
    )
    .await;
    let selected = assign_snapshot(&h, SUPPORT).await;
    assert_eq!(selected["searchFocused"], json!(true), "{selected}");
    let before = backend_calls(&h).await.to_string();
    assert!(
        !before.contains("w-zoe"),
        "selecting is not a write: {before}"
    );

    click(
        &h,
        &format!("{SUPPORT} [data-confirmable-search-picker-confirm]"),
    )
    .await;
    wait_for_selector(
        &h,
        &format!("{SUPPORT} [data-helpdesk-action-feedback-state=\"success\"]"),
    )
    .await;
    let calls = backend_calls(&h).await.to_string();
    assert!(
        calls.contains("Assign") && calls.contains("w-zoe"),
        "confirm wrote the keyed person: {calls}"
    );
    let assigned = assign_snapshot(&h, SUPPORT).await;
    assert_eq!(assigned["dialogOpen"], json!(false), "{assigned}");
    assert_eq!(assigned["assignee"], json!("Zoe Park"), "{assigned}");

    // Escape closes the picker, not the drawer under it, and focus returns
    // to the control that opened it. A real key press: Modal answers the
    // native `cancel` event, which a synthetic keydown never fires.
    click(&h, &format!("{SUPPORT} [data-helpdesk-assign-open]")).await;
    wait_for_selector(
        &h,
        &format!("{SUPPORT} [data-confirmable-search-picker-search]:focus"),
    )
    .await;
    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-assign-open]:focus")).await;
    let escaped = assign_snapshot(&h, SUPPORT).await;
    assert_eq!(escaped["dialogOpen"], json!(false), "{escaped}");
    assert_eq!(
        escaped["drawerOpen"],
        json!("true"),
        "Escape in the picker must not close the drawer: {escaped}"
    );
    assert_eq!(escaped["changeFocused"], json!(true), "{escaped}");
    assert_no_browser_errors(&h, "assignee picker").await;
}

/// ldui-purt: a host whose directory read failed renders the drawer's
/// Unavailable state, never an empty picker. `/helpdesk-fixture` itself is
/// the negative control (the test above opens a populated picker).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn assignee_picker_reports_an_unavailable_directory() {
    let h = harness_at(&format!("{PAGE}-no-directory")).await;
    begin_browser_error_capture(&h).await;
    open_support_drawer(&h).await;
    let s = assign_snapshot(&h, SUPPORT).await;
    assert_eq!(
        s["unavailable"],
        json!("Directory unavailable"),
        "an empty directory says so: {s}"
    );
    assert_eq!(s["changePresent"], json!(false), "nothing to open: {s}");
    assert_no_browser_errors(&h, "assignee directory unavailable").await;
}

/// 4iiz-Office op-ggymr: a column filter narrows the table AND both of its
/// counts -- the FilterBar's "N of M results" and the footer's row range. The
/// summary used to count bucket + search only, so it stayed at "12 of 12"
/// while the Key filter showed one row.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn a_column_filter_narrows_the_result_count_and_the_footer() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;

    async fn counts(h: &pixelproof_web::Harness) -> Value {
        eval_json(
            h,
            &format!(
                r#"(() => {{
                    const root = document.querySelector('{SUPPORT}');
                    return {{
                        rows: root.querySelectorAll('[data-helpdesk-table] tbody tr[data-entity-row-key]').length,
                        result: root.querySelector('[data-filter-result-count]')?.textContent?.trim() ?? null,
                        range: root.querySelector('[data-entity-row-range]')?.textContent?.trim() ?? null,
                    }};
                }})()"#
            ),
        )
        .await
    }

    let before = counts(&h).await;
    assert_eq!(before["rows"], json!(12), "{before}");
    assert!(
        before["result"]
            .as_str()
            .is_some_and(|t| t.starts_with("12 ")),
        "unfiltered summary counts all 12: {before}"
    );

    // Filter by one key that no other key contains, so exactly one row fits.
    let key = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{SUPPORT}');
                const keys = Array.from(root.querySelectorAll('[data-helpdesk-table] tbody tr'))
                    .map(r => r.textContent.match(/OF-\d+/)?.[0]).filter(Boolean);
                const key = keys.find(k => keys.filter(o => o.includes(k)).length === 1);
                const input = root.querySelector('[data-entity-filter-control="key"]');
                input.value = key;
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return key;
            }})()"#
        ),
    )
    .await;
    assert!(
        key.as_str().is_some(),
        "a unique key exists in the seed: {key}"
    );

    let mut after = Value::Null;
    for _ in 0..30 {
        after = counts(&h).await;
        if after["rows"] == json!(1) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert_eq!(
        after["rows"],
        json!(1),
        "the Key filter narrows the table: {after}"
    );
    assert!(
        after["result"]
            .as_str()
            .is_some_and(|t| t.starts_with("1 ")),
        "the result summary counts the column filter too: {after}"
    );
    assert!(
        after["range"].as_str().is_some_and(|t| t.contains("of 1")),
        "the footer range counts the column filter too: {after}"
    );
    assert_no_browser_errors(&h, "helpdesk column-filter counts").await;
}
