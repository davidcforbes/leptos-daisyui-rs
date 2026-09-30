//! Real-browser proof for `WeekView`'s interactive planner contract
//! (ldui-9sip): the seven-day grid that 4iiz-Office's Day Planner puts
//! beside its single-day `DayScheduler`.
//!
//! The pure halves -- the keyboard intents, the day-bearing accessible name,
//! the one-layer layout, the work-day mask and the Sunday-first week math --
//! are proven natively in `src/components/week_view/tests.rs`. What only a
//! browser can add is what these tests check: that a Left/Right day-move
//! repositions the SAME focused node (the reason timed blocks live in one
//! index-keyed layer), that the mask and today's tint really paint, that the
//! headers name their own dates' weekdays, and that a locale signal renames
//! headers and blocks in place.
//!
//! Every test drives `/components/week-view`. Its fixtures are addressed by
//! id (`#display-week`, `#interactive-week`, `#localized-week`) and the
//! component's own data hooks, never by document position.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use pixelproof_web::{Harness, Key};
use serde_json::{Value, json};

const ROUTE: &str = "/components/week-view";
const INTERACTIVE: &str = "#interactive-week";
const LOCALIZED: &str = "#localized-week";
const DISPLAY: &str = "#display-week";

async fn eval_json(h: &Harness, expression: &str) -> Value {
    h.page()
        .evaluate(expression)
        .await
        .expect("evaluate week fixture")
        .into_value()
        .unwrap_or(Value::Null)
}

async fn testid_text(h: &Harness, testid: &str) -> String {
    eval_json(
        h,
        &format!("document.querySelector('[data-testid=\"{testid}\"]').textContent"),
    )
    .await
    .as_str()
    .unwrap_or_default()
    .to_string()
}

async fn attr(h: &Harness, selector: &str, name: &str) -> Value {
    eval_json(
        h,
        &format!("document.querySelector('{selector}')?.getAttribute('{name}') ?? null"),
    )
    .await
}

async fn press(h: &Harness, key: Key) {
    h.press_key_sequence(&[key]).await.expect("key press");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

async fn open(route: &str) -> Harness {
    let h = harness_at(route).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{INTERACTIVE} [data-week-event]")).await;
    h
}

/// The localized fixture's `[weekday, date]` header texts, read through the
/// header's own data hooks.
async fn localized_headers(h: &Harness) -> Value {
    eval_json(
        h,
        &format!(
            r#"Array.from(document.querySelectorAll('{LOCALIZED} [data-week-day-header]')).map((c) => [
                c.querySelector('[data-week-weekday]').textContent,
                c.querySelector('[data-week-date]').textContent,
            ])"#
        ),
    )
    .await
}

/// Whether the tagged probe node is still attached, still focused, and
/// still the node the given selector resolves to.
async fn probe_is_same_focused_node(h: &Harness, selector: &str) -> bool {
    eval_json(
        h,
        &format!(
            "window.__weekProbe === document.querySelector('{selector}') \
             && document.activeElement === window.__weekProbe \
             && window.__weekProbe.isConnected"
        ),
    )
    .await
    .as_bool()
    .unwrap_or(false)
}

/// Which day column the probe's horizontal centre sits in, measured from the
/// rendered column rects of its own fixture.
async fn probe_column(h: &Harness, fixture: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const r = window.__weekProbe.getBoundingClientRect();
                const cx = (r.left + r.right) / 2;
                const cols = Array.from(document.querySelectorAll('{fixture} [data-week-day]'));
                const hit = cols.find((c) => {{
                    const cr = c.getBoundingClientRect();
                    return cx >= cr.left && cx < cr.right;
                }});
                return hit ? Number(hit.dataset.weekDay) : null;
            }})()"#
        ),
    )
    .await
}

/// A click selects and activates by the index into `events`, and the block's
/// accessible name carries its day -- the column header is never read when a
/// screen-reader user tabs between blocks.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-week-planner)"]
async fn click_selects_and_the_name_carries_the_day() {
    let h = open(ROUTE).await;
    let block = format!("{INTERACTIVE} [data-week-event=\"1\"]");

    click(&h, &block).await;
    assert_eq!(testid_text(&h, "week-selected").await, "1");
    assert_eq!(testid_text(&h, "week-activated").await, "1");
    assert_eq!(attr(&h, &block, "role").await, json!("button"));
    assert_eq!(attr(&h, &block, "aria-pressed").await, json!("true"));
    assert_eq!(
        attr(&h, &block, "aria-label").await,
        json!("Filing block, Wednesday Mar 4, 11:00 to 12:30")
    );
    // The other timed block is interactive but unselected.
    assert_eq!(
        attr(
            &h,
            &format!("{INTERACTIVE} [data-week-event=\"0\"]"),
            "aria-pressed"
        )
        .await,
        json!("false")
    );
    assert_no_browser_errors(&h, "week planner click").await;
}

/// ArrowDown moves the event in time and ArrowRight/ArrowLeft across days,
/// and every one of those presses lands on the SAME node, which keeps focus:
/// the node moves between columns instead of being rebuilt in another one.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-week-planner)"]
async fn arrow_keys_move_time_and_day_on_the_same_focused_node() {
    let h = open(ROUTE).await;
    let block = format!("{INTERACTIVE} [data-week-event=\"0\"]");
    eval_json(
        &h,
        &format!("window.__weekProbe = document.querySelector('{block}'); true"),
    )
    .await;

    click(&h, &block).await;
    assert_eq!(testid_text(&h, "week-first").await, "0:540-600");
    assert_eq!(probe_column(&h, INTERACTIVE).await, json!(0));

    press(&h, Key::ArrowDown).await;
    assert_eq!(
        testid_text(&h, "week-first").await,
        "0:555-615",
        "ArrowDown must request a 15-minute move"
    );
    assert!(probe_is_same_focused_node(&h, &block).await);

    press(&h, Key::ArrowRight).await;
    assert_eq!(
        testid_text(&h, "week-first").await,
        "1:555-615",
        "ArrowRight must request a one-day move and keep the time"
    );
    assert!(
        probe_is_same_focused_node(&h, &block).await,
        "a day-move must keep the same focused node, or the next press lands nowhere"
    );
    assert_eq!(attr(&h, &block, "data-week-event-day").await, json!("1"));
    assert_eq!(
        probe_column(&h, INTERACTIVE).await,
        json!(1),
        "the block must be painted inside Tuesday's column"
    );
    assert_eq!(
        attr(&h, &block, "aria-label").await,
        json!("Intake review, Tuesday Mar 3, 09:15 to 10:15")
    );

    // Back past the first column: the page clamps, the node survives.
    press(&h, Key::ArrowLeft).await;
    press(&h, Key::ArrowLeft).await;
    assert_eq!(testid_text(&h, "week-first").await, "0:555-615");
    assert_eq!(probe_column(&h, INTERACTIVE).await, json!(0));
    assert!(probe_is_same_focused_node(&h, &block).await);
    assert_no_browser_errors(&h, "week planner keyboard moves").await;
}

/// An all-day chip is a button too: Left/Right moves it between days on the
/// same node (placed by `grid-column`), and Up/Down -- which has no time axis
/// to act on -- does nothing.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-week-planner)"]
async fn all_day_chip_moves_across_days_and_ignores_time_keys() {
    let h = open(ROUTE).await;
    let chip = format!("{INTERACTIVE} [data-week-all-day-event=\"2\"]");
    eval_json(
        &h,
        &format!("window.__weekProbe = document.querySelector('{chip}'); true"),
    )
    .await;

    click(&h, &chip).await;
    assert_eq!(testid_text(&h, "week-selected").await, "2");
    assert_eq!(
        attr(&h, &chip, "aria-label").await,
        json!("Office closed, Saturday Mar 7, all day")
    );

    press(&h, Key::ArrowLeft).await;
    assert_eq!(testid_text(&h, "week-all-day").await, "4");
    assert!(probe_is_same_focused_node(&h, &chip).await);
    assert_eq!(
        eval_json(&h, "getComputedStyle(window.__weekProbe).gridColumnStart").await,
        json!("5"),
        "the chip must now sit in Friday's grid column"
    );

    press(&h, Key::ArrowDown).await;
    assert_eq!(testid_text(&h, "week-all-day").await, "4");
    assert_eq!(testid_text(&h, "week-first").await, "0:540-600");
    assert_no_browser_errors(&h, "week planner all-day chip").await;
}

/// `work_days` = Monday to Friday shades Saturday and Sunday (and only
/// those), today keeps its own tint and `aria-current="date"`, and a view
/// that never set `work_days` shades nothing: the default is a seven-day
/// workweek.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-week-planner)"]
async fn work_day_mask_shades_only_non_working_days() {
    let h = open(ROUTE).await;
    let columns = eval_json(
        &h,
        &format!(
            r#"Array.from(document.querySelectorAll('{INTERACTIVE} [data-week-day]')).map((c) => ({{
                day: Number(c.dataset.weekDay),
                work: c.dataset.workDay,
                bg: getComputedStyle(c).backgroundColor,
            }}))"#
        ),
    )
    .await;
    let columns = columns.as_array().expect("seven columns");
    assert_eq!(columns.len(), 7);
    let work: Vec<&str> = columns
        .iter()
        .map(|c| c["work"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(
        work,
        ["true", "true", "true", "true", "true", "false", "false"]
    );

    let bg = |day: usize| columns[day]["bg"].as_str().unwrap_or_default().to_string();
    assert_eq!(bg(5), bg(6), "both weekend days share the non-work shade");
    assert_ne!(
        bg(5),
        bg(0),
        "a non-work day must not paint like a work day"
    );
    assert_ne!(bg(2), bg(0), "today must keep its own tint");
    assert_ne!(bg(2), bg(5), "today's tint is not the non-work shade");
    assert_eq!(bg(0), bg(4), "work days share one (untinted) background");

    assert_eq!(
        attr(
            &h,
            &format!("{INTERACTIVE} [data-week-day-header=\"2\"]"),
            "aria-current"
        )
        .await,
        json!("date")
    );
    assert_eq!(
        attr(
            &h,
            &format!("{INTERACTIVE} [data-week-day-header=\"0\"]"),
            "aria-current"
        )
        .await,
        Value::Null
    );
    let now_label = eval_json(
        &h,
        &format!("document.querySelector('{INTERACTIVE} [data-week-day=\"2\"]').textContent"),
    )
    .await;
    assert!(
        now_label.as_str().is_some_and(|t| t.contains("Now")),
        "the now-line label must render in today's column: {now_label}"
    );

    let default_mask = eval_json(
        &h,
        &format!(
            "Array.from(document.querySelectorAll('{DISPLAY} [data-week-day]')).map((c) => c.dataset.workDay)"
        ),
    )
    .await;
    assert_eq!(default_mask, Value::from(vec!["true"; 7]));
    assert_no_browser_errors(&h, "week planner work-day mask").await;
}

/// A display-only week gains no tab stops and no header buttons -- the
/// interaction contract is strictly opt-in, as on `DayScheduler`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-week-planner)"]
async fn display_only_week_has_no_tab_stops() {
    let h = open(ROUTE).await;
    let counts = eval_json(
        &h,
        &format!(
            r#"({{
                blocks: document.querySelectorAll('{DISPLAY} [data-week-event]').length,
                buttons: document.querySelectorAll('{DISPLAY} [role="button"], {DISPLAY} button').length,
                tabbable: document.querySelectorAll('{DISPLAY} [tabindex]').length,
            }})"#
        ),
    )
    .await;
    assert!(
        counts["blocks"].as_u64().unwrap_or(0) >= 7,
        "the display fixture must render its timed events: {counts}"
    );
    assert_eq!(counts["buttons"], json!(0), "{counts}");
    assert_eq!(counts["tabbable"], json!(0), "{counts}");
    assert_no_browser_errors(&h, "display-only week").await;
}

/// `on_day_activate` turns each header into a button reporting its column.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-week-planner)"]
async fn day_header_button_drills_down() {
    let h = open(ROUTE).await;
    let header = format!("{INTERACTIVE} [data-week-day-button=\"4\"]");
    assert_eq!(attr(&h, &header, "type").await, json!("button"));
    click(&h, &header).await;
    assert_eq!(testid_text(&h, "week-day-opened").await, "4");
    assert_no_browser_errors(&h, "week planner drill-down").await;
}

/// A Sunday-first week labels each column with its own date's weekday, and a
/// locale signal renames the headers and the SAME event node in place --
/// selection survives, and EN -> ES -> EN round-trips exactly.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-week-planner)"]
async fn sunday_first_headers_and_locale_toggle_rename_in_place() {
    let h = open(ROUTE).await;
    assert_eq!(
        localized_headers(&h).await,
        json!([
            ["Sun", "1"],
            ["Mon", "2"],
            ["Tue", "3"],
            ["Wed", "4"],
            ["Thu", "5"],
            ["Fri", "6"],
            ["Sat", "7"]
        ])
    );

    let block = format!("{LOCALIZED} [data-week-event=\"0\"]");
    eval_json(
        &h,
        &format!("window.__weekProbe = document.querySelector('{block}'); true"),
    )
    .await;
    click(&h, &block).await;
    assert_eq!(testid_text(&h, "week-locale-selected").await, "0");
    let en_label = attr(&h, &block, "aria-label").await;
    assert_eq!(en_label, json!("Cita, Sunday Mar 1, 09:00 to 10:00"));

    click(&h, "[data-testid='week-locale-toggle']").await;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let weekdays: Vec<Value> = localized_headers(&h)
        .await
        .as_array()
        .expect("headers")
        .iter()
        .map(|pair| pair[0].clone())
        .collect();
    assert_eq!(
        weekdays,
        ["dom", "lun", "mar", "mié", "jue", "vie", "sáb"]
            .map(|d| json!(d))
            .to_vec()
    );
    let same_node = eval_json(
        &h,
        &format!("window.__weekProbe === document.querySelector('{block}')"),
    )
    .await;
    assert_eq!(
        same_node,
        json!(true),
        "the locale toggle must not rebuild the block"
    );
    assert_eq!(
        attr(&h, &block, "aria-label").await,
        json!("Cita, domingo 1, de 09:00 a 10:00")
    );
    assert_eq!(
        testid_text(&h, "week-locale-selected").await,
        "0",
        "selection must survive the locale toggle"
    );

    click(&h, "[data-testid='week-locale-toggle']").await;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    assert_eq!(attr(&h, &block, "aria-label").await, en_label);
    assert_no_browser_errors(&h, "week planner locale toggle").await;
}
