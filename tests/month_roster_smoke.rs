//! Real-browser proof for `MonthRoster` (ldui-5lmt): the staff month
//! calendar with Week and Day zoom that 4iiz-Office's Staff > Monthly
//! Schedule adopts (op-uclmv).
//!
//! The pure half (spans, steps, order, roving moves, names, axis geometry,
//! EN/ES) is proven natively in `src/patterns/month_roster/tests.rs`. What
//! only a browser can add is checked here, one test per child bead's CHECK:
//! the rendered grid and its overflow (.2), the single tab stop and the keys
//! (.3), the Week view (.4), the Day timeline's geometry (.5), the container
//! query (.6) and print (.7), each with axe.
//!
//! Every test drives `/components/month-roster`, whose fixture office is
//! October 2026 with 14 Oct as today. Elements are addressed by the
//! component's data hooks (`data-day`, `data-entry`, `data-month-roster-action`,
//! ...) and the demo's testids, never by document position.

mod common;

use chromiumoxide::cdp::browser_protocol::emulation::SetEmulatedMediaParams;
use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use pixelproof_web::{Harness, Key, ViewportSize};
use serde_json::{Value, json};

const ROUTE: &str = "/components/month-roster";
const ROOT: &str = "#fixture-roster";
const TODAY: &str = "2026-10-14";

async fn eval_json(h: &Harness, expression: &str) -> Value {
    h.page()
        .evaluate(expression)
        .await
        .expect("evaluate month roster fixture")
        .into_value()
        .unwrap_or(Value::Null)
}

async fn testid(h: &Harness, id: &str) -> String {
    eval_json(
        h,
        &format!("document.querySelector('[data-testid=\"{id}\"]').textContent"),
    )
    .await
    .as_str()
    .unwrap_or_default()
    .to_string()
}

async fn settle() {
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
}

async fn press(h: &Harness, key: Key) {
    h.press_key_sequence(&[key]).await.expect("key press");
    settle().await;
}

/// PageUp / PageDown, which `pixelproof_web::Key` does not carry.
async fn press_page(h: &Harness, down: bool) {
    let (key, code) = if down {
        ("PageDown", 34)
    } else {
        ("PageUp", 33)
    };
    for kind in [
        DispatchKeyEventType::RawKeyDown,
        DispatchKeyEventType::KeyUp,
    ] {
        let params = DispatchKeyEventParams::builder()
            .r#type(kind)
            .key(key)
            .code(key)
            .windows_virtual_key_code(code)
            .native_virtual_key_code(code)
            .build()
            .expect("page key params");
        h.page().execute(params).await.expect("dispatch page key");
    }
    settle().await;
}

async fn open() -> Harness {
    let h = harness_at(ROUTE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{ROOT} td[data-day=\"{TODAY}\"]")).await;
    h
}

fn day(iso: &str) -> String {
    format!("{ROOT} [data-day=\"{iso}\"]")
}

/// The `data-day` of the focused element's date box, or null.
async fn focused_day(h: &Harness) -> Value {
    eval_json(
        h,
        "document.activeElement?.closest('[data-day]')?.getAttribute('data-day') ?? null",
    )
    .await
}

async fn title(h: &Harness) -> String {
    eval_json(
        h,
        &format!("document.querySelector('{ROOT} [data-month-roster-title]').textContent"),
    )
    .await
    .as_str()
    .unwrap_or_default()
    .to_string()
}

async fn focus(h: &Harness, selector: &str) {
    eval_json(
        h,
        &format!("document.querySelector('{selector}').focus(), true"),
    )
    .await;
    settle().await;
}

async fn axe_clean(h: &Harness, context: &str) {
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(h.page()).await.expect("run axe-core");
    report.assert_no_blocking(context).unwrap_or_else(|e| {
        panic!(
            "{e}; {}\nviolations: {:#?}",
            report.summary(),
            report.violations
        )
    });
}

async fn set_theme(h: &Harness, theme: &str) {
    eval_json(
        h,
        &format!("document.documentElement.setAttribute('data-theme', '{theme}'), true"),
    )
    .await;
    settle().await;
}

/// Visible to a sighted user: rendered, not `visibility: hidden`, and
/// neither the element NOR any ancestor clipped to the 1px box of `sr-only`.
/// `getBoundingClientRect` ignores an ancestor's clip, so a chip inside the
/// hidden overflow list reports its full size; only walking the ancestors
/// sees the 1px list around it. `display: contents` boxes have no rect and
/// are skipped.
const VISIBLE_JS: &str = "(el) => { if (!el.checkVisibility({ visibilityProperty: true })) return false; for (let n = el; n && n !== document.body; n = n.parentElement) { if (getComputedStyle(n).display === 'contents') continue; const r = n.getBoundingClientRect(); if (r.width <= 2 || r.height <= 2) return false; } return true; }";

// ---- .2: the month grid -------------------------------------------------

/// The fixture October renders as the kit's month grid: whole Monday weeks,
/// the 9-entry day shows 4 chips and "+5", unknown differs from empty.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_fixture_month_renders_as_the_kit_grid() {
    let h = open().await;
    let facts = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{ROOT}');
                const cell = (iso) => root.querySelector(`td[data-day="${{iso}}"]`);
                const busy = cell('{TODAY}');
                const visible = {VISIBLE_JS};
                return {{
                    cells: root.querySelectorAll('td[data-day]').length,
                    rows: root.querySelectorAll('tr[data-month-week]').length,
                    busyShown: busy.querySelectorAll('ul[data-chips] > li[data-entry]').length,
                    busyExtra: busy.querySelectorAll('[data-overflow-list] > li[data-entry]').length,
                    busyExtraVisible: [...busy.querySelectorAll('[data-overflow-list] > li[data-entry]')].filter(visible).length,
                    more: busy.querySelector('[data-overflow] [aria-hidden]')?.textContent ?? null,
                    moreSpoken: busy.querySelector('[data-overflow] .sr-only')?.textContent ?? null,
                    pending: busy.querySelector('[data-entry-pending] [data-entry-words]')?.textContent ?? null,
                    unknown: [cell('2026-10-28').hasAttribute('data-unknown'), cell('2026-10-28').textContent.includes('No schedule published')],
                    empty: [cell('2026-10-04').hasAttribute('data-empty'), cell('2026-10-04').textContent.includes('No one scheduled')],
                    unknownIsNotEmpty: cell('2026-10-28').textContent.includes('No one scheduled'),
                    holiday: cell('2026-10-12').querySelector('[data-day-mark="holiday"]').textContent,
                    halfDay: cell('2026-10-23').querySelector('[data-day-mark="holiday"]').textContent,
                    closed: cell('2026-10-30').querySelector('[data-day-mark="closed"]').textContent,
                    today: [busy.hasAttribute('data-today'), busy.querySelector('[data-today-mark]').textContent],
                    outside: cell('2026-09-28').hasAttribute('data-outside'),
                    staffingShort: busy.querySelector('[data-staffing]').hasAttribute('data-staffing-short'),
                    staffing: busy.querySelector('[data-staffing]').textContent,
                    title: root.querySelector('[data-month-roster-title]').textContent,
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(facts["cells"], json!(35), "five Monday weeks: {facts}");
    assert_eq!(facts["rows"], json!(5), "{facts}");
    assert_eq!(facts["busyShown"], json!(4), "4 chips in view: {facts}");
    assert_eq!(
        facts["busyExtra"],
        json!(5),
        "the other 5 stay in the DOM: {facts}"
    );
    assert_eq!(
        facts["busyExtraVisible"],
        json!(0),
        "and stay hidden until asked: {facts}"
    );
    assert_eq!(facts["more"], json!("+5"), "{facts}");
    assert_eq!(facts["moreSpoken"], json!("5 more people"), "{facts}");
    assert_eq!(facts["pending"], json!("Vacation, Requested"), "{facts}");
    assert_eq!(facts["unknown"], json!([true, true]), "{facts}");
    assert_eq!(facts["empty"], json!([true, true]), "{facts}");
    assert_eq!(facts["unknownIsNotEmpty"], json!(false), "{facts}");
    assert_eq!(facts["holiday"], json!("Holiday: Columbus Day"), "{facts}");
    assert_eq!(
        facts["halfDay"],
        json!("Holiday (half day): Staff appreciation afternoon"),
        "{facts}"
    );
    assert_eq!(
        facts["closed"],
        json!("Closed: Building maintenance"),
        "{facts}"
    );
    assert_eq!(facts["today"], json!([true, "Today"]), "{facts}");
    assert_eq!(facts["outside"], json!(true), "{facts}");
    assert_eq!(facts["staffing"], json!("5 of 6 needed"), "{facts}");
    assert_eq!(facts["staffingShort"], json!(true), "{facts}");
    assert_eq!(facts["title"], json!("October 2026"), "{facts}");

    // The "+5" toggle reveals the rest in place.
    click(&h, &format!("{} [data-overflow]", day(TODAY))).await;
    let revealed = eval_json(
        &h,
        &format!(
            "[...document.querySelectorAll('{} [data-overflow-list] > li[data-entry]')].filter({VISIBLE_JS}).length",
            day(TODAY)
        ),
    )
    .await;
    assert_eq!(revealed, json!(5), "the toggle reveals the 5 hidden chips");
    assert_no_browser_errors(&h, "month grid").await;
}

/// EN -> ES -> EN renames the SAME nodes: the date boxes are keyed by date
/// and the chips by content, never by the texts.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn a_locale_switch_renames_the_same_nodes() {
    let h = open().await;
    eval_json(
        &h,
        &format!(
            r#"(() => {{
                window.__cell = document.querySelector('{holiday}');
                window.__chip = document.querySelector('{busy} li[data-entry]');
                window.__title = document.querySelector('{ROOT} [data-month-roster-title]');
                return true;
            }})()"#,
            holiday = day("2026-10-12"),
            busy = day(TODAY),
        ),
    )
    .await;
    let read = r#"(() => ({
        same: window.__cell === document.querySelector('[data-day="2026-10-12"]')
            && window.__chip === document.querySelector('[data-day="2026-10-14"] li[data-entry]')
            && window.__title.isConnected && window.__cell.isConnected && window.__chip.isConnected,
        title: window.__title.textContent,
        cell: window.__cell.getAttribute('aria-label'),
        chip: window.__chip.querySelector('[data-entry-words]').textContent,
    }))()"#;
    click(&h, "[data-testid=\"roster-locale-toggle\"]").await;
    let es = eval_json(&h, read).await;
    assert_eq!(es["same"], json!(true), "Spanish rebuilt a node: {es}");
    assert_eq!(es["title"], json!("octubre de 2026"), "{es}");
    assert!(
        es["cell"]
            .as_str()
            .unwrap()
            .starts_with("lunes, 12 oct 2026"),
        "{es}"
    );
    assert!(
        es["cell"]
            .as_str()
            .unwrap()
            .contains("Festivo: Día de la Raza"),
        "{es}"
    );
    assert_eq!(es["chip"], json!("Jornada completa"), "{es}");

    click(&h, "[data-testid=\"roster-locale-toggle\"]").await;
    let en = eval_json(&h, read).await;
    assert_eq!(en["same"], json!(true), "English rebuilt a node: {en}");
    assert_eq!(en["title"], json!("October 2026"), "{en}");
    assert!(
        en["cell"]
            .as_str()
            .unwrap()
            .starts_with("Monday, Oct 12, 2026"),
        "{en}"
    );
    assert_eq!(en["chip"], json!("Full shift"), "{en}");
    assert_no_browser_errors(&h, "locale switch").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_month_is_axe_clean_in_light_and_dark() {
    let h = open().await;
    set_theme(&h, "light").await;
    axe_clean(&h, "month roster, light").await;
    set_theme(&h, "dark").await;
    axe_clean(&h, "month roster, dark").await;
}

// ---- .3: keyboard and accessibility --------------------------------------

/// Tab enters the grid once, at today, visits only that day's own controls
/// and then leaves: one tab stop for 35 date boxes.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_grid_is_one_tab_stop() {
    let h = open().await;
    let stops = eval_json(
        &h,
        &format!("document.querySelectorAll('{ROOT} td[data-day][tabindex=\"0\"]').length"),
    )
    .await;
    assert_eq!(stops, json!(1), "exactly one date box is in the tab order");

    // The control just before the grid in tab order is the last header
    // control: the zoom switch's Day button.
    focus(
        &h,
        &format!("{ROOT} [data-month-roster-action=\"zoom-day\"]"),
    )
    .await;
    press(&h, Key::Tab).await;
    assert_eq!(focused_day(&h).await, json!(TODAY), "Tab enters at today");
    let entered_on_box = eval_json(&h, "document.activeElement.matches('td[data-day]')").await;
    assert_eq!(entered_on_box, json!(true));

    let mut visited = Vec::new();
    for _ in 0..16 {
        press(&h, Key::Tab).await;
        let here = focused_day(&h).await;
        let in_grid = eval_json(
            &h,
            &format!("!!document.activeElement.closest('{ROOT} [data-month-grid]')"),
        )
        .await;
        if in_grid != json!(true) {
            break;
        }
        visited.push(here);
    }
    assert!(
        visited.iter().all(|d| d == &json!(TODAY)),
        "Tab visited another date box: {visited:?}"
    );
    let left = eval_json(
        &h,
        &format!("!document.activeElement.closest('{ROOT} [data-month-grid]')"),
    )
    .await;
    assert_eq!(left, json!(true), "Tab never left the grid: {visited:?}");
    assert_no_browser_errors(&h, "one tab stop").await;
}

/// Arrows move a day and a week, PageDown proposes the next month keeping
/// the day, Enter proposes the Day zoom of the focused day, and every box is
/// named in words.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_keys_move_page_and_drill() {
    let h = open().await;
    let holiday_name = eval_json(
        &h,
        &format!(
            "document.querySelector('{}').getAttribute('aria-label')",
            day("2026-10-12")
        ),
    )
    .await;
    assert!(
        holiday_name
            .as_str()
            .unwrap()
            .contains("Holiday: Columbus Day"),
        "{holiday_name}"
    );
    let today_name = eval_json(
        &h,
        &format!(
            "document.querySelector('{}').getAttribute('aria-label')",
            day(TODAY)
        ),
    )
    .await;
    assert_eq!(
        today_name,
        json!("Wednesday, Oct 14, 2026, Today, 5 working, 4 out, 5 of 6 needed")
    );

    focus(&h, &format!("{ROOT} td[data-day=\"{TODAY}\"]")).await;
    press(&h, Key::ArrowRight).await;
    assert_eq!(
        focused_day(&h).await,
        json!("2026-10-15"),
        "ArrowRight is one day"
    );
    press(&h, Key::ArrowDown).await;
    assert_eq!(
        focused_day(&h).await,
        json!("2026-10-22"),
        "ArrowDown is one week"
    );
    press(&h, Key::Home).await;
    assert_eq!(
        focused_day(&h).await,
        json!("2026-10-19"),
        "Home is the row start"
    );
    press(&h, Key::End).await;
    assert_eq!(
        focused_day(&h).await,
        json!("2026-10-25"),
        "End is the row end"
    );
    let stops = eval_json(
        &h,
        &format!("document.querySelectorAll('{ROOT} td[data-day][tabindex=\"0\"]').length"),
    )
    .await;
    assert_eq!(stops, json!(1), "the tab stop moved with the focus");
    axe_clean(&h, "month roster, focus inside the grid").await;

    press(&h, Key::ArrowUp).await;
    press(&h, Key::ArrowLeft).await;
    press(&h, Key::ArrowLeft).await;
    press(&h, Key::ArrowLeft).await;
    press(&h, Key::ArrowLeft).await;
    assert_eq!(focused_day(&h).await, json!(TODAY));
    press(&h, Key::Enter).await;
    assert_eq!(testid(&h, "roster-last-nav").await, "day:2026-10-14");
    assert_eq!(
        testid(&h, "roster-anchor-day").await,
        "20740",
        "on_navigate(Day, 20740)"
    );
    assert_eq!(testid(&h, "roster-zoom").await, "day");

    click(
        &h,
        &format!("{ROOT} [data-month-roster-action=\"zoom-month\"]"),
    )
    .await;
    assert_eq!(testid(&h, "roster-zoom").await, "month");
    focus(&h, &format!("{ROOT} td[data-day=\"{TODAY}\"]")).await;
    press_page(&h, true).await;
    assert_eq!(testid(&h, "roster-last-nav").await, "month:2026-11-14");
    assert_eq!(title(&h).await, "November 2026");
    assert_eq!(
        focused_day(&h).await,
        json!("2026-11-14"),
        "the focus lands on the same day of the new month"
    );
    press_page(&h, false).await;
    assert_eq!(testid(&h, "roster-last-nav").await, "month:2026-10-14");
    assert_eq!(focused_day(&h).await, json!(TODAY));
    assert_no_browser_errors(&h, "keys").await;
}

// ---- .4: zoom and the Week view -------------------------------------------

/// A row's week button proposes that week; the Week view lists every entry
/// of every day, working before "Out (N)"; next steps 7 days; Escape returns
/// to the month; the switch and headings rename in Spanish.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_week_view_lists_every_entry() {
    let h = open().await;
    click(
        &h,
        &format!("{ROOT} tr[data-month-week=\"2026-10-05\"] [data-month-roster-action=\"week\"]"),
    )
    .await;
    assert_eq!(testid(&h, "roster-last-nav").await, "week:2026-10-05");
    wait_for_selector(&h, &format!("{ROOT} [data-week-column]")).await;
    assert_eq!(title(&h).await, "Oct 5 - 11, 2026");
    click(&h, &format!("{ROOT} [data-month-roster-action=\"next\"]")).await;
    assert_eq!(
        testid(&h, "roster-last-nav").await,
        "week:2026-10-12",
        "next is 7 days"
    );
    assert_eq!(title(&h).await, "Oct 12 - 18, 2026");

    let week = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const col = document.querySelector('{ROOT} [data-week-column][data-day="{TODAY}"]');
                const chips = [...col.querySelectorAll('li[data-entry]')];
                const outHeading = col.querySelector('[data-week-out]');
                const working = chips.filter((c) => ['full', 'half'].includes(c.dataset.entryState));
                return {{
                    columns: document.querySelectorAll('{ROOT} [data-week-column]').length,
                    chips: chips.length,
                    visible: chips.filter({VISIBLE_JS}).length,
                    out: outHeading.textContent,
                    workingBeforeOut: working.every((c) => c.compareDocumentPosition(outHeading) & Node.DOCUMENT_POSITION_FOLLOWING),
                    absentAfterOut: chips.filter((c) => !working.includes(c)).every((c) => outHeading.compareDocumentPosition(c) & Node.DOCUMENT_POSITION_FOLLOWING),
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(week["columns"], json!(7), "{week}");
    assert_eq!(week["chips"], json!(9), "every entry, no +N: {week}");
    assert_eq!(week["visible"], json!(9), "{week}");
    assert_eq!(week["out"], json!("Out (4)"), "{week}");
    assert_eq!(week["workingBeforeOut"], json!(true), "{week}");
    assert_eq!(week["absentAfterOut"], json!(true), "{week}");
    axe_clean(&h, "month roster, week").await;

    click(&h, "[data-testid=\"roster-locale-toggle\"]").await;
    let spanish = eval_json(
        &h,
        &format!(
            r#"[...document.querySelectorAll('{ROOT} [data-month-roster-action^="zoom-"]')].map((b) => b.textContent)
                .concat([document.querySelector('{ROOT} [data-week-column][data-day="{TODAY}"] [data-week-out]').textContent])"#
        ),
    )
    .await;
    assert_eq!(spanish, json!(["Mes", "Semana", "Día", "Ausentes (4)"]));
    click(&h, "[data-testid=\"roster-locale-toggle\"]").await;

    focus(
        &h,
        &format!("{ROOT} [data-week-column][data-day=\"{TODAY}\"] [data-entry-open]"),
    )
    .await;
    press(&h, Key::Escape).await;
    assert_eq!(
        testid(&h, "roster-zoom").await,
        "month",
        "Escape returns to the month"
    );
    assert_no_browser_errors(&h, "week view").await;
}

// ---- .5: the Day view -------------------------------------------------------

/// The busiest day: one bar per working person, positioned on the axis, out
/// today grouped by kind, the now line only on today.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_day_view_shows_coverage_and_who_is_out() {
    let h = open().await;
    click(&h, &format!("{} [data-day-number]", day(TODAY))).await;
    assert_eq!(testid(&h, "roster-last-nav").await, "day:2026-10-14");
    wait_for_selector(&h, &format!("{ROOT} [data-day-view]")).await;
    let facts = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const view = document.querySelector('{ROOT} [data-day-view]');
                const tick = (hour) => view.querySelector(`[data-hour="${{hour}}"]`).getBoundingClientRect().left;
                const bar = view.querySelector('[data-day-bar="p22-2026-10-14"]').getBoundingClientRect();
                const group = (kind) => view.querySelectorAll(`[data-out-group="${{kind}}"] li[data-entry]`).length;
                return {{
                    bars: view.querySelectorAll('[data-day-bar]').length,
                    barLeft: bar.left,
                    tick8: tick(8),
                    barRight: bar.right,
                    tick1630: tick(16) + (tick(17) - tick(16)) / 2,
                    groups: [group('vacation'), group('out-of-office'), group('sick'), group('other')],
                    outHeading: view.querySelector('[data-day-out] h3').textContent,
                    pending: view.querySelector('[data-out-group="vacation"] [data-entry-pending] [data-entry-words]').textContent,
                    nowLines: view.querySelectorAll('[data-now-line]').length,
                    open: view.querySelector('[data-day-bar="p22-2026-10-14"] [data-entry-open]').getAttribute('aria-label'),
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(
        facts["bars"],
        json!(5),
        "one bar per working person: {facts}"
    );
    let near = |a: &Value, b: &Value| (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= 1.0;
    assert!(
        near(&facts["barLeft"], &facts["tick8"]),
        "08:00 edge off the tick: {facts}"
    );
    assert!(
        near(&facts["barRight"], &facts["tick1630"]),
        "16:30 edge off the axis: {facts}"
    );
    assert_eq!(facts["groups"], json!([2, 1, 1, 0]), "{facts}");
    assert_eq!(facts["outHeading"], json!("Out today (4)"), "{facts}");
    assert_eq!(facts["pending"], json!("Vacation, Requested"), "{facts}");
    assert!(
        facts["nowLines"].as_u64().unwrap() > 0,
        "today has a now line: {facts}"
    );
    assert_eq!(
        facts["open"],
        json!("Tara Singh, working 08:00 to 16:30"),
        "{facts}"
    );
    assert_eq!(title(&h).await, "Wednesday, October 14, 2026");
    axe_clean(&h, "month roster, day").await;

    click(&h, &format!("{ROOT} [data-month-roster-action=\"next\"]")).await;
    assert_eq!(testid(&h, "roster-last-nav").await, "day:2026-10-15");
    let tomorrow = eval_json(
        &h,
        &format!("document.querySelectorAll('{ROOT} [data-day-view] [data-now-line]').length"),
    )
    .await;
    assert_eq!(tomorrow, json!(0), "no now line on another day");

    focus(&h, &format!("{ROOT} [data-day-view] [data-entry-open]")).await;
    press(&h, Key::Escape).await;
    assert_eq!(
        testid(&h, "roster-zoom").await,
        "week",
        "Escape returns to the week"
    );
    assert_no_browser_errors(&h, "day view").await;
}

// ---- .6: the narrow container -----------------------------------------------

/// A 390px HOST inside a 1440px window gets the agenda, and a 1024px host
/// the grid: the layout follows the container, not the viewport.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn a_narrow_host_gets_the_agenda() {
    let h = open().await;
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("wide viewport");
    let measure = format!(
        r#"(() => {{
            const root = document.querySelector('{ROOT}');
            const visible = {VISIBLE_JS};
            const inMonth = [...root.querySelectorAll('td[data-day]:not([data-outside])')];
            const outside = [...root.querySelectorAll('td[data-day][data-outside]')];
            return {{
                hostWidth: document.querySelector('#month-roster-host').getBoundingClientRect().width,
                pageScrolls: document.documentElement.scrollWidth > document.documentElement.clientWidth,
                rosterOverflows: root.scrollWidth > root.clientWidth + 1,
                headerShown: getComputedStyle(root.querySelector('thead')).display !== 'none',
                inMonth: inMonth.length,
                headingsShown: inMonth.filter((td) => visible(td.querySelector('h3'))).length,
                outsideShown: outside.filter(visible).length,
                cellDisplay: getComputedStyle(root.querySelector('td[data-day="{TODAY}"]')).display,
            }};
        }})()"#
    );
    eval_json(
        &h,
        "document.querySelector('#month-roster-host').style.width = '390px', true",
    )
    .await;
    settle().await;
    let narrow = eval_json(&h, &measure).await;
    assert_eq!(narrow["hostWidth"].as_f64(), Some(390.0), "{narrow}");
    assert_eq!(
        narrow["pageScrolls"],
        json!(false),
        "no horizontal page scroll: {narrow}"
    );
    assert_eq!(narrow["rosterOverflows"], json!(false), "{narrow}");
    assert_eq!(
        narrow["headerShown"],
        json!(false),
        "weekday header hidden: {narrow}"
    );
    assert_eq!(narrow["inMonth"], json!(31), "{narrow}");
    assert_eq!(
        narrow["headingsShown"],
        json!(31),
        "every day has a visible date: {narrow}"
    );
    assert_eq!(
        narrow["outsideShown"],
        json!(0),
        "outside days hidden: {narrow}"
    );
    assert_eq!(narrow["cellDisplay"], json!("block"), "{narrow}");
    axe_clean(&h, "month roster, agenda").await;

    eval_json(
        &h,
        "document.querySelector('#month-roster-host').style.width = '1024px', true",
    )
    .await;
    settle().await;
    let wide = eval_json(&h, &measure).await;
    assert_eq!(wide["headerShown"], json!(true), "{wide}");
    assert_eq!(wide["headingsShown"], json!(0), "{wide}");
    assert_eq!(wide["outsideShown"], json!(4), "{wide}");
    assert_eq!(wide["cellDisplay"], json!("table-cell"), "{wide}");
    assert_no_browser_errors(&h, "narrow host").await;
}

// ---- .7: print ----------------------------------------------------------------

/// Printed, the month shows every entry and no control.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn print_shows_every_entry_without_controls() {
    let h = open().await;
    h.page()
        .execute(SetEmulatedMediaParams::builder().media("print").build())
        .await
        .expect("emulate print");
    settle().await;
    let printed = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const visible = {VISIBLE_JS};
                const root = document.querySelector('{ROOT}');
                return {{
                    print: matchMedia('print').matches,
                    controls: [...root.querySelectorAll('[data-month-roster-action]')].filter(visible).length,
                    chips: [...root.querySelectorAll('td[data-day="{TODAY}"] li[data-entry]')].filter(visible).length,
                    more: [...root.querySelectorAll('[data-overflow]')].filter(visible).length,
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(printed["print"], json!(true), "{printed}");
    assert_eq!(
        printed["controls"],
        json!(0),
        "no control prints: {printed}"
    );
    assert_eq!(printed["chips"], json!(9), "every entry prints: {printed}");
    assert_eq!(printed["more"], json!(0), "no +N prints: {printed}");
    h.page()
        .execute(SetEmulatedMediaParams::builder().media("").build())
        .await
        .expect("reset media");
    assert_no_browser_errors(&h, "print").await;
}
