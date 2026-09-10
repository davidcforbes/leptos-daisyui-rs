//! Release-browser proof for `HelpHint` (ldui-q25n): real pointer, keyboard,
//! and trusted touch input; computed disclosure visibility; stable description
//! semantics; independent instances; capture-phase Escape isolation; cleanup;
//! and scoped axe/browser-error checks.

mod common;

use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, DispatchTouchEventReturns, TouchPoint,
};
use chromiumoxide::{Command, Method};
use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, force_desktop_hover_media,
    harness_at, move_pointer_to_svg_fraction, wait_for_selector,
};
use pixelproof_web::{Harness, Key};
use serde_json::{Value, json};

#[derive(Debug, Clone, serde::Serialize)]
struct RawDispatchTouchEvent {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "touchPoints")]
    touch_points: Vec<TouchPoint>,
}

impl Method for RawDispatchTouchEvent {
    fn identifier(&self) -> chromiumoxide::types::MethodId {
        "Input.dispatchTouchEvent".into()
    }
}

impl Command for RawDispatchTouchEvent {
    type Response = DispatchTouchEventReturns;
}

const PAGE: &str = "/components/tooltip";
const FIXTURE: &str = "#help-hint-fixture";

async fn eval_json(harness: &Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .expect("evaluate HelpHint fixture")
        .into_value()
        .expect("HelpHint expression returns JSON")
}

async fn snapshot(harness: &Harness, id: &str) -> Value {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const root = document.querySelector(`[data-help-hint="${{CSS.escape('{id}')}}"]`);
                const trigger = root?.querySelector('[data-help-hint-trigger]');
                const tooltip = root?.querySelector('[role="tooltip"]');
                const tooltipRoot = root?.querySelector('.tooltip');
                const style = tooltip ? getComputedStyle(tooltip) : null;
                const before = tooltipRoot ? getComputedStyle(tooltipRoot, '::before') : null;
                const after = tooltipRoot ? getComputedStyle(tooltipRoot, '::after') : null;
                return {{
                    rootOpen: root?.dataset.helpHintOpen ?? null,
                    triggerId: trigger?.id ?? null,
                    describedBy: trigger?.getAttribute('aria-describedby') ?? null,
                    expanded: trigger?.getAttribute('aria-expanded') ?? null,
                    descriptionId: tooltip?.id ?? null,
                    descriptionRole: tooltip?.getAttribute('role') ?? null,
                    descriptionText: tooltip?.textContent.trim() ?? null,
                    tooltipOpenClass: root?.querySelector('.tooltip')?.classList.contains('tooltip-open') ?? false,
                    computedVisible: style
                        ? style.visibility !== 'hidden' && Number.parseFloat(style.opacity) > 0.9
                        : false,
                    pointerEvents: style?.pointerEvents ?? null,
                    beforeDisplay: before?.display ?? null,
                    afterDisplay: after?.display ?? null,
                    focused: root?.contains(document.activeElement) ?? false,
                }};
            }})()"#
        ),
    )
    .await
}

async fn focus(harness: &Harness, selector: &str) {
    harness
        .page()
        .find_element(selector)
        .await
        .unwrap_or_else(|error| panic!("find {selector}: {error}"))
        .focus()
        .await
        .unwrap_or_else(|error| panic!("focus {selector}: {error}"));
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

async fn scroll_into_view(harness: &Harness, selector: &str) {
    harness
        .page()
        .find_element(selector)
        .await
        .unwrap_or_else(|error| panic!("find {selector}: {error}"))
        .scroll_into_view()
        .await
        .unwrap_or_else(|error| panic!("scroll {selector} into view: {error}"));
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

async fn trusted_touch_tap(harness: &Harness, selector: &str) {
    let bounds = harness
        .element_box(selector)
        .await
        .unwrap_or_else(|error| panic!("measure {selector}: {error}"));
    let point = TouchPoint::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );
    harness
        .page()
        .execute(RawDispatchTouchEvent {
            kind: "touchStart",
            touch_points: vec![point],
        })
        .await
        .expect("dispatch trusted touch start");
    harness
        .page()
        .execute(RawDispatchTouchEvent {
            kind: "touchEnd",
            touch_points: Vec::new(),
        })
        .await
        .expect("dispatch trusted touch end");
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

async fn move_pointer_through_gap(harness: &Harness, trigger: &str, description: &str) {
    let point = eval_json(
        harness,
        &format!(
            r#"(() => {{
                const trigger = document.querySelector('{trigger}').getBoundingClientRect();
                const description = document.querySelector('{description}').getBoundingClientRect();
                return {{x: trigger.left + trigger.width / 2, y: (trigger.bottom + description.top) / 2}};
            }})()"#
        ),
    )
    .await;
    harness
        .page()
        .execute(
            DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseMoved)
                .x(point["x"].as_f64().expect("gap x"))
                .y(point["y"].as_f64().expect("gap y"))
                .build()
                .expect("gap mouse move params"),
        )
        .await
        .expect("move pointer through trigger/description gap");
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-help-hint)"]
async fn variants_and_subject_keep_stable_accessible_semantics() {
    let harness = harness_at(PAGE).await;
    wait_for_selector(&harness, FIXTURE).await;
    begin_browser_error_capture(&harness).await;

    let visible = snapshot(&harness, "help-hint-visible").await;
    assert_eq!(
        visible["triggerId"],
        json!("help-hint-visible"),
        "{visible}"
    );
    assert_eq!(
        visible["describedBy"],
        json!("help-hint-visible-description"),
        "{visible}"
    );
    assert_eq!(
        visible["descriptionId"], visible["describedBy"],
        "{visible}"
    );
    assert_eq!(visible["descriptionRole"], json!("tooltip"), "{visible}");
    assert_eq!(
        visible["descriptionText"],
        json!("This definition is supplementary, not an instruction."),
        "{visible}"
    );
    assert_eq!(visible["expanded"], json!("false"), "{visible}");
    assert_eq!(visible["computedVisible"], json!(false), "{visible}");
    assert_eq!(visible["beforeDisplay"], json!("none"), "{visible}");
    assert_eq!(visible["afterDisplay"], json!("none"), "{visible}");

    let ids = eval_json(
        &harness,
        r#"(() => {
            const values = Array.from(document.querySelectorAll('#help-hint-fixture [id]'), element => element.id);
            return {values, unique: new Set(values).size};
        })()"#,
    )
    .await;
    assert_eq!(
        ids["unique"].as_u64(),
        ids["values"].as_array().map(|values| values.len() as u64),
        "every caller-provided trigger and derived description id is unique: {ids}"
    );

    let compact_shape = eval_json(
        &harness,
        r#"(() => {
            const root = document.querySelector('[data-help-hint="help-hint-compact"]');
            const trigger = root.querySelector('[data-help-hint-trigger]');
            const hiddenLabel = trigger.querySelector('.sr-only');
            const icon = trigger.querySelector('[aria-hidden="true"]');
            const subject = root.querySelector('[data-help-subject]');
            return {
                square: trigger.classList.contains('btn-square'),
                hiddenLabel: hiddenLabel?.textContent,
                icon: icon?.textContent,
                subjectDescription: subject?.getAttribute('aria-describedby'),
            };
        })()"#,
    )
    .await;
    assert_eq!(
        compact_shape,
        json!({
            "square": true,
            "hiddenLabel": "About archive eligibility",
            "icon": "?",
            "subjectDescription": "help-hint-compact-description",
        })
    );

    focus(&harness, "#help-hint-visible").await;
    harness
        .press_key_sequence(&[Key::Tab, Key::Tab])
        .await
        .expect("Tab to compact HelpHint trigger");
    let focus_indicator = eval_json(
        &harness,
        r#"(() => {
            const trigger = document.querySelector('#help-hint-compact');
            const style = getComputedStyle(trigger);
            return {
                activeId: document.activeElement?.id,
                focusVisible: trigger.matches(':focus-visible'),
                outlineStyle: style.outlineStyle,
                outlineWidth: style.outlineWidth,
                boxShadow: style.boxShadow,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        focus_indicator["activeId"],
        json!("help-hint-compact"),
        "{focus_indicator}"
    );
    assert_eq!(
        focus_indicator["focusVisible"],
        json!(true),
        "{focus_indicator}"
    );
    assert!(
        focus_indicator["outlineStyle"] != json!("none")
            || focus_indicator["boxShadow"] != json!("none"),
        "keyboard focus must have a computed visible indicator: {focus_indicator}"
    );
    focus(&harness, "#help-hint-outside").await;

    focus(&harness, "[data-help-subject]").await;
    assert_eq!(
        snapshot(&harness, "help-hint-compact").await["computedVisible"],
        json!(true)
    );
    harness
        .press_key_sequence(&[Key::Escape])
        .await
        .expect("Escape");
    assert_eq!(
        snapshot(&harness, "help-hint-compact").await["computedVisible"],
        json!(false)
    );
    focus(&harness, "#help-hint-compact").await;
    let internal = snapshot(&harness, "help-hint-compact").await;
    assert_eq!(
        internal["computedVisible"],
        json!(false),
        "subject-to-trigger focus is internal, so it cannot undo Escape: {internal}"
    );
    focus(&harness, "#help-hint-outside").await;
    focus(&harness, "#help-hint-compact").await;
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("first keyboard activation");
    harness
        .press_key_sequence(&[Key::Space])
        .await
        .expect("second keyboard activation");
    assert_eq!(
        snapshot(&harness, "help-hint-compact").await["computedVisible"],
        json!(false),
        "the second real keyboard activation dismisses while focus remains"
    );
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("pin again with keyboard");
    focus(&harness, "#help-hint-outside").await;
    assert_eq!(
        snapshot(&harness, "help-hint-compact").await["computedVisible"],
        json!(false),
        "a keyboard pin clears on composition blur"
    );
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-subject-activations]').dataset.subjectActivations"
        )
        .await,
        json!("0"),
        "help interaction must not activate its subject"
    );
    click(&harness, "[data-help-subject]").await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-subject-activations]').dataset.subjectActivations"
        )
        .await,
        json!("1"),
        "the optional subject keeps its ordinary activation behavior"
    );
    click(&harness, "#help-hint-visible").await;
    focus(&harness, "[data-help-subject]").await;
    let placement = eval_json(
        &harness,
        r#"(() => {
            const fixture = document.querySelector('#help-hint-fixture').getBoundingClientRect();
            return Array.from(document.querySelectorAll('#help-hint-visible-description, #help-hint-compact-description'))
                .map(element => {
                    const rect = element.getBoundingClientRect();
                    return {id: element.id, left: rect.left, right: rect.right, fixtureLeft: fixture.left, fixtureRight: fixture.right};
                });
        })()"#,
    )
    .await;
    assert!(
        placement
            .as_array()
            .is_some_and(|items| items.iter().all(|item| {
                item["left"].as_f64().unwrap_or_default()
                    >= item["fixtureLeft"].as_f64().unwrap_or_default()
                    && item["right"].as_f64().unwrap_or_default()
                        <= item["fixtureRight"].as_f64().unwrap_or_default()
            })),
        "visible HelpHint descriptions must stay inside the content pane: {placement}"
    );
    std::fs::create_dir_all("target").expect("create screenshot directory");
    std::fs::write(
        "target/help-hint-q25n.png",
        harness
            .screenshot_bytes()
            .await
            .expect("capture HelpHint evidence"),
    )
    .expect("save HelpHint evidence screenshot");

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let _ = axe.run(harness.page()).await.expect("inject axe-core");
    let blocking = eval_json(
        &harness,
        r#"(async () => {
            const report = await axe.run(document.querySelector('#help-hint-fixture'), {
                runOnly: {type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21aa']},
                resultTypes: ['violations'],
            });
            return report.violations.filter(v => v.impact === 'serious' || v.impact === 'critical');
        })()"#,
    )
    .await;
    assert_eq!(
        blocking,
        json!([]),
        "blocking HelpHint axe findings: {blocking}"
    );
    assert_no_browser_errors(&harness, "HelpHint variants and subject").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-help-hint)"]
async fn pointer_keyboard_escape_and_multiple_instances_are_truthful() {
    let harness = harness_at(PAGE).await;
    wait_for_selector(&harness, FIXTURE).await;
    begin_browser_error_capture(&harness).await;
    force_desktop_hover_media(&harness).await;
    scroll_into_view(&harness, FIXTURE).await;

    move_pointer_to_svg_fraction(&harness, "#help-hint-visible", 0.5, 0.5).await;
    let hovered = snapshot(&harness, "help-hint-visible").await;
    assert_eq!(hovered["computedVisible"], json!(true), "{hovered}");
    assert_eq!(hovered["pointerEvents"], json!("auto"), "{hovered}");
    let gap = eval_json(
        &harness,
        r#"(() => {
            const trigger = document.querySelector('#help-hint-visible').getBoundingClientRect();
            const description = document.querySelector('#help-hint-visible-description').getBoundingClientRect();
            return description.top - trigger.bottom;
        })()"#,
    )
    .await;
    assert!(
        gap.as_f64().is_some_and(|gap| gap <= 0.5),
        "trigger-to-description path must not contain a dead hover gap: {gap}"
    );
    move_pointer_through_gap(
        &harness,
        "#help-hint-visible",
        "#help-hint-visible-description",
    )
    .await;
    assert_eq!(
        snapshot(&harness, "help-hint-visible").await["computedVisible"],
        json!(true),
        "the real pointer remains inside the disclosure at the trigger/description boundary"
    );
    move_pointer_to_svg_fraction(&harness, "#help-hint-visible-description", 0.5, 0.5).await;
    assert_eq!(
        snapshot(&harness, "help-hint-visible").await["computedVisible"],
        json!(true),
        "moving from the trigger into the help content must preserve hover disclosure"
    );
    move_pointer_to_svg_fraction(&harness, "#help-hint-outside", 0.5, 0.5).await;
    assert_eq!(
        snapshot(&harness, "help-hint-visible").await["computedVisible"],
        json!(false)
    );

    focus(&harness, "#help-hint-outside").await;
    move_pointer_to_svg_fraction(&harness, "#help-hint-visible", 0.5, 0.5).await;
    harness
        .press_key_sequence(&[Key::Escape])
        .await
        .expect("hover-only Escape");
    let hover_dismissed = snapshot(&harness, "help-hint-visible").await;
    assert_eq!(
        hover_dismissed["computedVisible"],
        json!(false),
        "{hover_dismissed}"
    );
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-unrelated-escapes]').dataset.unrelatedEscapes"
        )
        .await,
        json!("0"),
        "hover-only Escape must be captured even while focus is elsewhere"
    );
    move_pointer_to_svg_fraction(&harness, "#help-hint-outside", 0.5, 0.5).await;

    click(&harness, "#help-hint-visible").await;
    move_pointer_to_svg_fraction(&harness, "#help-hint-outside", 0.5, 0.5).await;
    assert_eq!(
        snapshot(&harness, "help-hint-visible").await["computedVisible"],
        json!(true)
    );
    click(&harness, "#help-hint-visible").await;
    let toggled_off = snapshot(&harness, "help-hint-visible").await;
    assert_eq!(toggled_off["expanded"], json!("false"), "{toggled_off}");
    assert_eq!(
        toggled_off["computedVisible"],
        json!(false),
        "CSS hover must not reopen it: {toggled_off}"
    );
    assert_eq!(toggled_off["beforeDisplay"], json!("none"), "{toggled_off}");
    assert_eq!(toggled_off["afterDisplay"], json!("none"), "{toggled_off}");

    move_pointer_to_svg_fraction(&harness, "#help-hint-outside", 0.5, 0.5).await;
    focus(&harness, "#help-hint-visible").await;
    harness
        .press_key_sequence(&[Key::Enter])
        .await
        .expect("Enter");
    focus(&harness, "#help-hint-outside").await;
    assert_eq!(
        snapshot(&harness, "help-hint-visible").await["computedVisible"],
        json!(false),
        "keyboard pin clears when focus leaves the composition"
    );

    click(&harness, "#help-hint-multi-b").await;
    focus(&harness, "#help-hint-outside").await;
    click(&harness, "#help-hint-multi-a").await;
    focus(&harness, "#help-hint-outside").await;
    assert_eq!(
        snapshot(&harness, "help-hint-multi-a").await["computedVisible"],
        json!(true)
    );
    assert_eq!(
        snapshot(&harness, "help-hint-multi-b").await["computedVisible"],
        json!(true)
    );
    harness
        .press_key_sequence(&[Key::Escape])
        .await
        .expect("Escape pinned A");
    assert_eq!(
        snapshot(&harness, "help-hint-multi-a").await["computedVisible"],
        json!(false)
    );
    assert_eq!(
        snapshot(&harness, "help-hint-multi-b").await["computedVisible"],
        json!(true),
        "the same Escape must not fall through to a second window listener"
    );
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-unrelated-escapes]').dataset.unrelatedEscapes"
        )
        .await,
        json!("0"),
        "a handled HelpHint Escape must not reach an unrelated overlay"
    );
    harness
        .press_key_sequence(&[Key::Escape])
        .await
        .expect("Escape pinned B");
    assert_eq!(
        snapshot(&harness, "help-hint-multi-b").await["computedVisible"],
        json!(false)
    );
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-unrelated-escapes]').dataset.unrelatedEscapes"
        )
        .await,
        json!("0"),
        "the second handled Escape is also isolated"
    );
    harness
        .press_key_sequence(&[Key::Escape])
        .await
        .expect("unhandled Escape");
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-unrelated-escapes]').dataset.unrelatedEscapes"
        )
        .await,
        json!("1"),
        "hidden hints must not hijack Escape"
    );
    assert_no_browser_errors(&harness, "HelpHint pointer, keyboard, and Escape").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-help-hint)"]
async fn trusted_touch_pins_and_unmount_removes_the_global_listener() {
    let harness = harness_at(PAGE).await;
    wait_for_selector(&harness, FIXTURE).await;
    begin_browser_error_capture(&harness).await;
    scroll_into_view(&harness, FIXTURE).await;
    eval_json(
        &harness,
        r#"(() => {
            window.__helpHintTrustedClicks = [];
            window.__helpHintListeners = {added: [], removed: []};
            window.__helpHintOriginalAdd = window.addEventListener;
            window.__helpHintOriginalRemove = window.removeEventListener;
            window.addEventListener = function(type, listener, options) {
                if (type === 'keydown' && options === true) window.__helpHintListeners.added.push(listener);
                return window.__helpHintOriginalAdd.call(this, type, listener, options);
            };
            window.removeEventListener = function(type, listener, options) {
                if (type === 'keydown' && options === true) window.__helpHintListeners.removed.push(listener);
                return window.__helpHintOriginalRemove.call(this, type, listener, options);
            };
            return true;
        })()"#,
    )
    .await;
    click(&harness, "#remove-help-hint").await;
    wait_for_selector(&harness, "#help-hint-removable").await;
    eval_json(
        &harness,
        r#"(() => {
            document.querySelector('#help-hint-removable').addEventListener('click', event => {
                window.__helpHintTrustedClicks.push(event.isTrusted);
            });
            return true;
        })()"#,
    )
    .await;

    trusted_touch_tap(&harness, "#help-hint-removable").await;
    focus(&harness, "#help-hint-outside").await;
    let pinned = snapshot(&harness, "help-hint-removable").await;
    assert_eq!(
        pinned["computedVisible"],
        json!(true),
        "first trusted tap pins: {pinned}"
    );
    assert_eq!(
        eval_json(&harness, "window.__helpHintTrustedClicks").await,
        json!([true]),
        "the component must receive a browser-trusted click synthesized from touch"
    );

    trusted_touch_tap(&harness, "#help-hint-removable").await;
    let toggled_off = snapshot(&harness, "help-hint-removable").await;
    assert_eq!(
        toggled_off["computedVisible"],
        json!(false),
        "the second trusted tap dismisses despite CSS focus/hover: {toggled_off}"
    );
    trusted_touch_tap(&harness, "#help-hint-removable").await;
    assert_eq!(
        eval_json(&harness, "window.__helpHintTrustedClicks").await,
        json!([true, true, true]),
        "all three activations must be browser-trusted touch-derived clicks"
    );

    click(&harness, "#remove-help-hint").await;
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('#help-hint-removable') === null"
        )
        .await,
        json!(true)
    );
    let listener_balance = eval_json(
        &harness,
        r#"(() => {
            const {added, removed} = window.__helpHintListeners;
            const result = {
                added: added.length,
                removed: removed.length,
                sameListener: added.length === 1 && removed.length === 1 && added[0] === removed[0],
            };
            window.addEventListener = window.__helpHintOriginalAdd;
            window.removeEventListener = window.__helpHintOriginalRemove;
            return result;
        })()"#,
    )
    .await;
    assert_eq!(
        listener_balance,
        json!({"added": 1, "removed": 1, "sameListener": true}),
        "mount/unmount must remove the exact capture listener it added"
    );
    focus(&harness, "#help-hint-outside").await;
    harness
        .press_key_sequence(&[Key::Escape])
        .await
        .expect("Escape after unmount");
    assert_eq!(
        eval_json(
            &harness,
            "document.querySelector('[data-unrelated-escapes]').dataset.unrelatedEscapes"
        )
        .await,
        json!("1"),
        "an unmounted hint must leave no capture listener behind"
    );
    assert_no_browser_errors(&harness, "HelpHint trusted touch and cleanup").await;
}
