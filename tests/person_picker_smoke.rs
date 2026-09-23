//! Browser proof for `PersonPicker` (doc/plans/2026-09-21-person-picker-design.md):
//! the listbox keyboard contract with toggle-off, one reachable card under
//! search, both x controls, the Unknown/Offline distinction, the activity
//! caption, wrapping at a fixed 360px without horizontal overflow, and WCAG
//! AA contrast at rest and with a selection.

mod common;

use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use pixelproof_web::ViewportSize;
use serde_json::{Value, json};

const PAGE: &str = "/components/person_picker";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate person picker fixture")
        .into_value()
        .expect("fixture JSON")
}

async fn open() -> pixelproof_web::Harness {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set a wide viewport");
    common::wait_for_selector(&h, "[data-person-listbox]").await;
    h
}

/// Copied from `selectable_summary_smoke.rs`, which is proven.
async fn press_key(
    h: &pixelproof_web::Harness,
    key: &str,
    code: &str,
    key_code: i64,
    text: Option<&str>,
) {
    let mut down = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::KeyDown)
        .key(key)
        .code(code)
        .windows_virtual_key_code(key_code)
        .native_virtual_key_code(key_code);
    if let Some(text) = text {
        down = down.text(text);
    }
    let down = down.build().expect("key-down params");
    h.page().execute(down).await.expect("dispatch key-down");

    let up = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::KeyUp)
        .key(key)
        .code(code)
        .windows_virtual_key_code(key_code)
        .native_virtual_key_code(key_code)
        .build()
        .expect("key-up params");
    h.page().execute(up).await.expect("dispatch key-up");

    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

async fn state(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const opts = [...document.querySelectorAll('[data-person-picker] [data-person-option]')];
            const id = el => el ? el.getAttribute('data-person-option') : null;
            const active = document.activeElement && document.activeElement.closest
                ? document.activeElement.closest('[data-person-option]') : null;
            const summary = document.querySelector('[data-person-selected-summary]');
            return {
                visible: opts.map(id),
                tabStops: opts.filter(o => o.getAttribute('tabindex') === '0').map(id),
                ariaSelected: opts.filter(o => o.getAttribute('aria-selected') === 'true').map(id),
                active: id(active),
                output: document.querySelector('[data-testid="person-picker-selected"]').textContent.trim(),
                summary: summary ? summary.getAttribute('data-person-selected-summary') : null,
                searchClear: !!document.querySelector('[data-person-search-clear]'),
            };
        })()"#,
    )
    .await
}

async fn focus_tab_stop(h: &pixelproof_web::Harness) {
    eval_json(
        h,
        r#"(() => { document.querySelector('[data-person-listbox] [tabindex="0"]').focus(); return true; })()"#,
    )
    .await;
}

async fn type_search(h: &pixelproof_web::Harness, text: &str) {
    let input = h
        .page()
        .find_element("[data-person-search]")
        .await
        .expect("search input");
    input.focus().await.expect("focus search");
    input.type_str(text).await.expect("type search");
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-person-picker)"]
async fn keyboard_moves_focus_without_selecting_and_space_or_click_toggles() {
    let h = open().await;

    let s = state(&h).await;
    assert_eq!(
        s["tabStops"],
        json!(["ana"]),
        "one tab stop, the first person: {s}"
    );
    assert_eq!(s["ariaSelected"], json!([]), "nothing selected yet: {s}");

    focus_tab_stop(&h).await;
    press_key(&h, "ArrowDown", "ArrowDown", 40, None).await;
    let s = state(&h).await;
    assert_eq!(s["active"], json!("ben"), "ArrowDown moves focus: {s}");
    assert_eq!(s["ariaSelected"], json!([]), "...WITHOUT selecting: {s}");
    assert_eq!(
        s["tabStops"],
        json!(["ben"]),
        "the tab stop follows focus: {s}"
    );

    press_key(&h, " ", "Space", 32, Some(" ")).await;
    let s = state(&h).await;
    assert_eq!(s["ariaSelected"], json!(["ben"]), "Space selects: {s}");
    assert_eq!(
        s["output"],
        json!("ben"),
        "the page received the proposal: {s}"
    );

    press_key(&h, " ", "Space", 32, Some(" ")).await;
    assert_eq!(
        state(&h).await["output"],
        json!("(none)"),
        "Space again deselects"
    );

    press_key(&h, "End", "End", 35, None).await;
    press_key(&h, "Enter", "Enter", 13, Some("\r")).await;
    let s = state(&h).await;
    assert_eq!(
        s["active"],
        json!("email"),
        "End jumps to the last person: {s}"
    );
    assert_eq!(s["output"], json!("email"), "Enter selects: {s}");

    click(&h, "[data-person-option=\"ana\"]").await;
    assert_eq!(
        state(&h).await["output"],
        json!("ana"),
        "click switches the selection"
    );
    click(&h, "[data-person-option=\"ana\"]").await;
    assert_eq!(
        state(&h).await["output"],
        json!("(none)"),
        "second click deselects"
    );

    assert_no_browser_errors(&h, "person picker keyboard and click").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-person-picker)"]
async fn search_keeps_one_tab_stop_and_each_x_clears_what_it_sits_on() {
    let h = open().await;

    click(&h, "[data-person-option=\"cara\"]").await;
    type_search(&h, "austin").await;
    let s = state(&h).await;
    assert_eq!(s["visible"], json!(["ben"]), "search filters: {s}");
    assert_eq!(
        s["tabStops"],
        json!(["ben"]),
        "exactly one reachable card after filtering: {s}"
    );
    assert_eq!(
        s["output"],
        json!("cara"),
        "search never clears the selection: {s}"
    );
    assert_eq!(
        s["summary"],
        json!("cara"),
        "the badge still names the hidden pick: {s}"
    );
    assert_eq!(
        s["searchClear"],
        json!(true),
        "a non-empty search shows its x: {s}"
    );

    click(&h, "[data-person-search-clear]").await;
    let s = state(&h).await;
    assert_eq!(
        s["visible"].as_array().map(Vec::len),
        Some(6),
        "search x clears the search: {s}"
    );
    assert_eq!(
        s["output"],
        json!("cara"),
        "...and leaves the selection alone: {s}"
    );
    assert_eq!(
        s["searchClear"],
        json!(false),
        "no x on an empty search: {s}"
    );

    click(&h, "[data-person-selection-clear]").await;
    let s = state(&h).await;
    assert_eq!(
        s["output"],
        json!("(none)"),
        "badge x clears the selection: {s}"
    );
    assert_eq!(s["summary"], json!(null), "and the badge goes: {s}");

    type_search(&h, "zzzz").await;
    let empty = eval_json(
        &h,
        r#"(() => ({
            noMatches: !!document.querySelector('[data-person-no-matches]'),
            listbox: !!document.querySelector('[data-person-listbox]'),
        }))()"#,
    )
    .await;
    assert_eq!(
        empty,
        json!({"noMatches": true, "listbox": false}),
        "no empty listbox: {empty}"
    );

    assert_no_browser_errors(&h, "person picker search").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-person-picker)"]
async fn cards_wrap_at_360px_distinguish_unknown_and_pass_contrast() {
    let h = open().await;

    let layout = eval_json(
        &h,
        r#"(() => {
            const picker = document.querySelector('[data-person-picker]');
            const roster = document.querySelector('[data-person-roster]');
            const opt = id => document.querySelector(`[data-person-option="${id}"]`);
            const overflowing = [...document.querySelectorAll('[data-person-option]')]
                .filter(o => o.scrollWidth > o.clientWidth + 1)
                .map(o => o.getAttribute('data-person-option'));
            return {
                width: picker.getBoundingClientRect().width,
                rosterOverflowX: roster.scrollWidth > roster.clientWidth + 1,
                overflowing,
                longWrapped: opt('long').getBoundingClientRect().height > 60,
                unknownDot: !!opt('dmitri').querySelector('[data-person-presence]'),
                unknownText: opt('dmitri').textContent,
                offlineDot: !!opt('long').querySelector('[data-person-presence="offline"]'),
                activity: (opt('ben').querySelector('[data-person-activity]') || {}).textContent || null,
            };
        })()"#,
    )
    .await;
    assert!(
        layout["width"]
            .as_f64()
            .is_some_and(|w| (355.0..=365.0).contains(&w)),
        "fixed 360px: {layout}"
    );
    assert_eq!(
        layout["rosterOverflowX"],
        json!(false),
        "no horizontal scroll: {layout}"
    );
    assert_eq!(
        layout["overflowing"],
        json!([]),
        "no card overflows sideways: {layout}"
    );
    assert_eq!(
        layout["longWrapped"],
        json!(true),
        "the long name wraps: {layout}"
    );
    // op-wkppl: 'not signed in' must never read as 'offline'.
    assert_eq!(
        layout["unknownDot"],
        json!(false),
        "Unknown claims no state, so no dot: {layout}"
    );
    assert!(
        layout["unknownText"]
            .as_str()
            .is_some_and(|t| t.contains("Not signed in") && !t.contains("Offline")),
        "Unknown reads 'Not signed in', never 'Offline': {layout}"
    );
    assert_eq!(
        layout["offlineDot"],
        json!(true),
        "Offline does claim a state: {layout}"
    );
    assert!(
        layout["activity"]
            .as_str()
            .is_some_and(|t| t.contains("Replying")),
        "the activity caption renders: {layout}"
    );

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let _page_report = axe.run(h.page()).await.expect("inject axe-core");
    async fn contrast(h: &pixelproof_web::Harness) -> Value {
        eval_json(
            h,
            r#"(async () => {
                const report = await axe.run(document.querySelector('[data-person-picker]'), {
                    runOnly: { type: 'rule', values: ['color-contrast'] },
                    resultTypes: ['violations'],
                });
                return report.violations.map(v => ({
                    id: v.id,
                    nodes: v.nodes.slice(0, 5).map(n => ({ target: n.target, summary: n.failureSummary })),
                }));
            })()"#,
        )
        .await
    }
    let rest = contrast(&h).await;
    assert_eq!(rest, json!([]), "AA contrast at rest: {rest}");
    click(&h, "[data-person-option=\"ben\"]").await;
    let picked = contrast(&h).await;
    assert_eq!(
        picked,
        json!([]),
        "AA contrast with a selection (incl. the activity caption): {picked}"
    );

    assert_no_browser_errors(&h, "person picker layout and contrast").await;
}

/// ldui-8hmy (4iiz-Office owner ruling 2026-09-23): a page whose presence
/// refresh failed says so QUIETLY. Live, the dots carry today's fills and no
/// option has a description. Stale, every KNOWN dot greys to the one stale
/// fill in exactly the same box, carries the caption as its tooltip, and the
/// caption is the accessible description of that dot's option; Unknown still
/// has no dot and gains nothing. Turning it off restores today's rendering.
/// BREAK: ignore the prop and the greyed-fill assertion fails.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-person-picker)"]
async fn stale_presence_greys_the_dots_in_place_and_describes_the_options() {
    let h = open().await;

    async fn dots(h: &pixelproof_web::Harness) -> Value {
        eval_json(
            h,
            r#"(() => {
                const opts = [...document.querySelectorAll('[data-person-picker] [data-person-option]')];
                return opts.map(opt => {
                    const dot = opt.querySelector('[data-person-presence]');
                    const r = dot ? dot.getBoundingClientRect() : null;
                    const describedBy = opt.getAttribute('aria-describedby');
                    const hint = describedBy ? document.getElementById(describedBy) : null;
                    return {
                        id: opt.getAttribute('data-person-option'),
                        presence: dot ? dot.getAttribute('data-person-presence') : null,
                        fill: dot
                            ? [...dot.classList].find(c => c.startsWith('bg-')) ?? null
                            : null,
                        stale: dot ? dot.getAttribute('data-person-presence-stale') : null,
                        title: dot ? dot.getAttribute('title') : null,
                        box: r ? [r.left, r.top, r.width, r.height].map(Math.round) : null,
                        description: hint ? hint.textContent.trim() : null,
                        // The caption must not leak into the option's NAME.
                        // A listbox option is named from its content minus
                        // aria-hidden subtrees (the accname rule, and the
                        // one ldui-audit's drift.js uses). Raw textContent
                        // would include the aria-hidden caption span by
                        // design, which is exactly the disabled_reason trap.
                        nameLeak: (() => {
                            const clone = opt.cloneNode(true);
                            clone.querySelectorAll('[aria-hidden="true"]').forEach(n => n.remove());
                            return /as of|a las/.test(clone.textContent);
                        })(),
                    };
                });
            })()"#,
        )
        .await
    }

    let live = dots(&h).await;
    let live_rows = live.as_array().expect("option rows");
    let fills: Vec<&str> = live_rows
        .iter()
        .filter_map(|row| row["fill"].as_str())
        .collect();
    assert!(
        fills.contains(&"bg-success")
            && fills.contains(&"bg-error")
            && fills.contains(&"bg-warning")
            && fills.contains(&"bg-base-300"),
        "live dots carry today's four fills: {live}"
    );
    for row in live_rows {
        assert_eq!(row["stale"], json!(null), "live: {row}");
        assert_eq!(
            row["title"],
            json!(null),
            "live dots have no tooltip: {row}"
        );
        assert_eq!(
            row["description"],
            json!(null),
            "live options are undescribed: {row}"
        );
    }

    click(&h, "[data-testid='person-picker-stale']").await;
    let stale = dots(&h).await;
    let stale_rows = stale.as_array().expect("option rows");
    assert_eq!(stale_rows.len(), live_rows.len(), "{stale}");
    for (before, after) in live_rows.iter().zip(stale_rows) {
        assert_eq!(before["id"], after["id"], "{stale}");
        if before["presence"].is_null() {
            // Unknown: no dot before, none after, and no description.
            assert_eq!(
                after["presence"],
                json!(null),
                "Unknown stays dotless: {after}"
            );
            assert_eq!(after["description"], json!(null), "{after}");
            continue;
        }
        assert_eq!(
            after["fill"],
            json!("bg-base-content/40"),
            "every known dot greys to the stale fill: {after}"
        );
        assert_eq!(after["stale"], json!("true"), "{after}");
        assert_eq!(after["title"], json!("Status as of 1:52 PM"), "{after}");
        assert_eq!(
            after["description"],
            json!("Status as of 1:52 PM"),
            "the caption is the option's accessible description: {after}"
        );
        assert_eq!(
            after["presence"], before["presence"],
            "the kept status is still recorded: {after}"
        );
        assert_eq!(
            after["box"], before["box"],
            "the dot must not move or resize when it greys: {before} -> {after}"
        );
        assert_eq!(after["nameLeak"], json!(false), "{after}");
    }
    assert!(
        stale_rows.iter().any(|row| row["id"] == json!("dmitri")),
        "the Unknown fixture is in the roster: {stale}"
    );

    // The caption is the caller's translated text and follows it live.
    click(&h, "[data-testid='person-picker-spanish']").await;
    let spanish = dots(&h).await;
    let ana = spanish
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["id"] == json!("ana")))
        .cloned()
        .expect("ana row");
    assert_eq!(
        ana["description"],
        json!("Estado a las 1:52 p. m."),
        "{spanish}"
    );
    assert_eq!(ana["title"], json!("Estado a las 1:52 p. m."), "{spanish}");
    click(&h, "[data-testid='person-picker-spanish']").await;

    // Back to live: exactly today's rendering again.
    click(&h, "[data-testid='person-picker-stale']").await;
    let restored = dots(&h).await;
    assert_eq!(
        restored, live,
        "turning staleness off restores the live render"
    );

    assert_no_browser_errors(&h, "person picker stale presence").await;
}
