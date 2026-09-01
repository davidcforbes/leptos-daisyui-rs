//! Real-browser proof for `Modal`'s controlled close contract (`ldui-e0fw`):
//! Escape, backdrop activation, and an in-content `method="dialog"` form each
//! emit exactly one typed proposal; accepting one closes the dialog exactly
//! once and returns focus to the trigger; declining one leaves the dialog
//! open with zero drift between the accepted `open` signal and
//! `HTMLDialogElement.open`; and a programmatic `open = false` closes without
//! emitting any proposal at all.
//!
//! Drives the general demo app (`html_target: None`, like
//! `search_picker_dialog_smoke.rs`) against the existing
//! `/components/modal` showcase route.
//!
//! ## Selector discipline
//!
//! Every element is located by an explicit `data-testid` minted by the
//! fixture, or by the `data-modal-close-mode` / `data-modal-backdrop`
//! markers the component itself emits. Nothing here is addressed by document
//! position (`:last-child`, "the first button in X"), because a positional
//! query keeps passing while silently describing a different element after a
//! layout change.

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, harness_at};
use pixelproof_web::Key;
use serde_json::Value;
use std::time::Duration;

const PAGE: &str = "/components/modal";

const TRIGGER: &str = "[data-testid=\"controlled-modal-trigger\"]";
const ACCEPT_TOGGLE: &str = "[data-testid=\"controlled-modal-accept-toggle\"]";
const BOX: &str = "[data-testid=\"controlled-modal-box\"]";
const DIALOG_FORM_CLOSE: &str = "[data-testid=\"controlled-modal-dialog-form-close\"]";
const PROGRAMMATIC_CLOSE: &str = "[data-testid=\"controlled-modal-programmatic-close\"]";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate controlled-modal fixture")
        .into_value()
        .expect("controlled-modal expression returns JSON")
}

/// Let the Leptos effect that calls `show_modal()` / `close()` run.
async fn settle() {
    tokio::time::sleep(Duration::from_millis(150)).await;
}

/// The accepted state the fixture publishes, the DOM truth
/// (`HTMLDialogElement.open`), the proposal tally, and where focus actually
/// is — everything needed to catch drift between the caller's signal and the
/// dialog.
async fn snapshot(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const box = document.querySelector('{BOX}');
                const dialog = box ? box.closest('dialog') : null;
                const trigger = document.querySelector('{TRIGGER}');
                const read = id => {{
                    const el = document.querySelector(`[data-testid="${{id}}"]`);
                    return el ? el.textContent.trim() : null;
                }};
                const active = document.activeElement;
                return {{
                    acceptedOpen: read('controlled-modal-open'),
                    proposalCount: read('controlled-modal-proposal-count'),
                    lastCause: read('controlled-modal-last-cause'),
                    policy: read('controlled-modal-policy'),
                    dialogOpen: dialog ? dialog.open : null,
                    closeMode: dialog ? dialog.getAttribute('data-modal-close-mode') : null,
                    hasBackdrop: !!(dialog && dialog.querySelector('[data-modal-backdrop="true"]')),
                    activeIsTrigger: !!(trigger && active === trigger),
                    activeInsideDialog: !!(dialog && active && dialog.contains(active)),
                }};
            }})()"#
        ),
    )
    .await
}

fn text(snapshot: &Value, key: &str) -> String {
    snapshot[key]
        .as_str()
        .unwrap_or_else(|| panic!("controlled-modal snapshot is missing a string `{key}`"))
        .to_string()
}

fn dialog_open(snapshot: &Value) -> bool {
    snapshot["dialogOpen"]
        .as_bool()
        .expect("controlled-modal snapshot reports HTMLDialogElement.open")
}

/// Focus the trigger before clicking it. A real mouse click focuses the
/// button during mousedown, which is what lets the dialog record it as the
/// element to restore focus to on close; `Element.click()` alone skips that
/// step and the dialog silently records `<body>` instead.
async fn open_modal(h: &pixelproof_web::Harness) {
    let script = format!(
        r#"(() => {{
            const trigger = document.querySelector('{TRIGGER}');
            trigger.focus();
            trigger.click();
        }})()"#
    );
    let _ = h.page().evaluate(script.as_str()).await;
    settle().await;
}

async fn click_testid(h: &pixelproof_web::Harness, selector: &str) {
    let script = format!(r#"document.querySelector('{selector}').click()"#);
    let _ = h.page().evaluate(script.as_str()).await;
    settle().await;
}

/// Activates the backdrop the way a pointer does — by submitting the
/// `method="dialog"` form the component renders for it. Located by the
/// component's own `data-modal-backdrop` marker, scoped to this fixture's
/// dialog.
async fn activate_backdrop(h: &pixelproof_web::Harness) {
    let script = format!(
        r#"(() => {{
            const dialog = document.querySelector('{BOX}').closest('dialog');
            const backdrop = dialog.querySelector('[data-modal-backdrop="true"]');
            backdrop.querySelector('button').click();
        }})()"#
    );
    let _ = h.page().evaluate(script.as_str()).await;
    settle().await;
}

/// Puts the fixture into accept or decline mode, reading the published
/// policy back rather than assuming the toggle landed.
async fn set_policy(h: &pixelproof_web::Harness, want_accept: bool) {
    let wanted = if want_accept { "accept" } else { "decline" };
    if text(&snapshot(h).await, "policy") != wanted {
        click_testid(h, ACCEPT_TOGGLE).await;
    }
    assert_eq!(
        text(&snapshot(h).await, "policy"),
        wanted,
        "fixture policy toggle did not land"
    );
}

/// Accepting an Escape proposal closes the dialog exactly once, keeps the
/// accepted signal and the DOM in agreement, and lets the platform return
/// focus to the trigger.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-modal-close-proposal)"]
async fn escape_proposes_and_accepting_closes_once_restoring_focus() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    set_policy(&h, true).await;
    open_modal(&h).await;

    let opened = snapshot(&h).await;
    assert!(dialog_open(&opened), "trigger opened the dialog");
    assert_eq!(text(&opened, "closeMode"), "controlled");
    assert!(
        opened["hasBackdrop"].as_bool().unwrap_or(false),
        "backdrop=true renders the component's own backdrop form"
    );
    let before: u64 = text(&opened, "proposalCount").parse().expect("count");

    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    settle().await;

    let after = snapshot(&h).await;
    assert_eq!(text(&after, "lastCause"), "escape");
    assert_eq!(
        text(&after, "proposalCount"),
        (before + 1).to_string(),
        "Escape emits exactly one proposal"
    );
    assert_eq!(text(&after, "acceptedOpen"), "false");
    assert!(!dialog_open(&after), "accepting the proposal closed it");
    assert!(
        after["activeIsTrigger"].as_bool().unwrap_or(false),
        "native close() returned focus to the trigger"
    );

    assert_no_browser_errors(&h, "escape accepted").await;
}

/// Declining leaves the dialog open and the accepted state untouched: no
/// optimistic close, so nothing to reconcile.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-modal-close-proposal)"]
async fn declining_an_escape_proposal_leaves_the_dialog_open() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    set_policy(&h, false).await;
    open_modal(&h).await;

    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    settle().await;

    let after = snapshot(&h).await;
    assert_eq!(text(&after, "lastCause"), "escape");
    assert_eq!(
        text(&after, "acceptedOpen"),
        "true",
        "a declined proposal never writes the accepted state"
    );
    assert!(
        dialog_open(&after),
        "a declined proposal leaves the dialog open"
    );
    assert!(
        after["activeInsideDialog"].as_bool().unwrap_or(false),
        "focus stays inside the still-open dialog"
    );

    assert_no_browser_errors(&h, "escape declined").await;
}

/// The backdrop is a `method="dialog"` form, so it fires no `cancel` at all —
/// the exact path that used to close the dialog behind the owner's back.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-modal-close-proposal)"]
async fn backdrop_activation_proposes_and_is_vetoable() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    set_policy(&h, false).await;
    open_modal(&h).await;
    activate_backdrop(&h).await;

    let declined = snapshot(&h).await;
    assert_eq!(text(&declined, "lastCause"), "backdrop");
    assert_eq!(text(&declined, "acceptedOpen"), "true");
    assert!(
        dialog_open(&declined),
        "a declined backdrop proposal leaves the dialog open"
    );

    set_policy(&h, true).await;
    activate_backdrop(&h).await;

    let accepted = snapshot(&h).await;
    assert_eq!(text(&accepted, "lastCause"), "backdrop");
    assert_eq!(text(&accepted, "acceptedOpen"), "false");
    assert!(!dialog_open(&accepted), "accepting the backdrop closed it");

    assert_no_browser_errors(&h, "backdrop proposal").await;
}

/// An in-content `<form method="dialog">` close button is the same silent
/// close path as the backdrop, and gets its own cause so a caller can tell a
/// deliberate confirm from a dismissal.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-modal-close-proposal)"]
async fn in_content_dialog_form_proposes_its_own_cause() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    set_policy(&h, true).await;
    open_modal(&h).await;
    click_testid(&h, DIALOG_FORM_CLOSE).await;

    let after = snapshot(&h).await;
    assert_eq!(text(&after, "lastCause"), "dialog-form");
    assert_eq!(text(&after, "acceptedOpen"), "false");
    assert!(!dialog_open(&after));

    assert_no_browser_errors(&h, "dialog-form proposal").await;
}

/// A programmatic `open = false` is not a user gesture and must not be
/// reported as one — otherwise every caller-driven close would look like a
/// dismissal and clear feedback the caller meant to keep.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-modal-close-proposal)"]
async fn programmatic_close_emits_no_proposal() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    set_policy(&h, true).await;
    open_modal(&h).await;

    let before = text(&snapshot(&h).await, "proposalCount");
    click_testid(&h, PROGRAMMATIC_CLOSE).await;

    let after = snapshot(&h).await;
    assert_eq!(text(&after, "acceptedOpen"), "false");
    assert!(!dialog_open(&after), "the owner's signal closed the dialog");
    assert_eq!(
        text(&after, "proposalCount"),
        before,
        "a programmatic close emits no user-close proposal"
    );

    assert_no_browser_errors(&h, "programmatic close").await;
}

/// The regression the bead names: after a user close, reopening must work.
/// It only does if the accepted state and the DOM went to `false` together,
/// so a later `false`-to-`true` change is a real change.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-modal-close-proposal)"]
async fn reopening_after_a_user_close_works() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    set_policy(&h, true).await;
    open_modal(&h).await;
    h.press_key_sequence(&[Key::Escape]).await.expect("Escape");
    settle().await;

    let closed = snapshot(&h).await;
    assert_eq!(text(&closed, "acceptedOpen"), "false");
    assert!(!dialog_open(&closed));

    open_modal(&h).await;

    let reopened = snapshot(&h).await;
    assert_eq!(text(&reopened, "acceptedOpen"), "true");
    assert!(
        dialog_open(&reopened),
        "the dialog reopens because nothing drifted while it was closed"
    );

    assert_no_browser_errors(&h, "reopen after user close").await;
}
