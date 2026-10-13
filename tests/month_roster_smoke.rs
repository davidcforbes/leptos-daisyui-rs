//! Real-browser proof for `MonthRoster` (ldui-5lmt): the staff month
//! calendar with Week and Day zoom that 4iiz-Office's Staff > Monthly
//! Schedule adopts (op-uclmv).
//!
//! The pure half (spans, steps, order, roving moves, names, axis geometry,
//! EN/ES) is proven natively in `src/patterns/month_roster/tests.rs`. What
//! only a browser can add is checked here, one test per child bead's CHECK:
//! the rendered grid and its overflow (.2), the single tab stop and the keys
//! (.3), the Week view (.4), the Day timeline's geometry (.5), the container
//! query (.6) and print (.7), each with axe. Then the editing hooks 4iiz-Office
//! asked for (ldui-k145) on a second, editable fixture, and the month grid's
//! contrast on its own fills at Office's width (ldui-on0i).
//!
//! Every test drives `/components/month-roster`, whose fixture office is
//! October 2026 with 14 Oct as today. Elements are addressed by the
//! component's data hooks (`data-day`, `data-entry`, `data-month-roster-action`,
//! ...) and the demo's testids, never by document position.

mod common;

use chromiumoxide::cdp::browser_protocol::emulation::SetEmulatedMediaParams;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType, DispatchMouseEventParams, DispatchMouseEventType,
    MouseButton,
};
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
    common::axe::assert_axe_clean(h, None, context).await;
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

// ---- ldui-k145: the editing hooks (4iiz-Office op-rrfr4) -------------------
//
// `#editable-roster` is the same office with `on_day_activate`,
// `on_day_add` and `on_entry_remove`, days after today addable and their
// working entries removable, and the adjacent month's days supplied. The
// `#fixture-roster` above has none of the hooks, so every test above proves
// that an existing host is unchanged.

const EDIT: &str = "#editable-roster";

fn edit_day(iso: &str) -> String {
    format!("{EDIT} [data-day=\"{iso}\"]")
}

/// The editable roster's zoom, from its own `data-zoom`.
async fn edit_zoom(h: &Harness) -> String {
    eval_json(h, &format!("document.querySelector('{EDIT}').dataset.zoom"))
        .await
        .as_str()
        .unwrap_or_default()
        .to_string()
}

/// The last day the host was asked to open, and how many times it was asked.
async fn opened(h: &Harness) -> (String, u64) {
    let count = testid(h, "editable-opened-count").await;
    (
        testid(h, "editable-opened").await,
        count.parse().unwrap_or_else(|_| panic!("count {count:?}")),
    )
}

/// A real double-click (CDP press and release twice, click counts 1 then 2)
/// at the fractions `(fx, fy)` of `selector`'s box, after scrolling it into
/// view.
async fn double_click(h: &Harness, selector: &str, fx: f64, fy: f64) {
    let point = eval_json(
        h,
        &format!(
            r#"(() => {{
                const el = document.querySelector('{selector}');
                el.scrollIntoView({{ block: 'center' }});
                const r = el.getBoundingClientRect();
                return [r.left + r.width * {fx}, r.top + r.height * {fy}];
            }})()"#
        ),
    )
    .await;
    let (x, y) = (point[0].as_f64().expect("x"), point[1].as_f64().expect("y"));
    for count in [1, 2] {
        for kind in [
            DispatchMouseEventType::MousePressed,
            DispatchMouseEventType::MouseReleased,
        ] {
            let params = DispatchMouseEventParams::builder()
                .r#type(kind)
                .x(x)
                .y(y)
                .button(MouseButton::Left)
                .click_count(count)
                .build()
                .expect("mouse params");
            h.page().execute(params).await.expect("dispatch mouse");
        }
    }
    settle().await;
}

/// Two `pointerup`s of `pointer` type, on `first` then `second`, `gap_ms`
/// apart. The roster reads only the pointer type and the event time, which
/// a dispatched event carries; CDP's touch input would need touch emulation
/// for the whole page.
async fn taps(h: &Harness, first: &str, second: &str, gap_ms: u64, pointer: &str) {
    for (i, selector) in [first, second].into_iter().enumerate() {
        if i == 1 && gap_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(gap_ms)).await;
        }
        eval_json(
            h,
            &format!(
                "document.querySelector('{selector}').dispatchEvent(new PointerEvent('pointerup', {{ pointerType: '{pointer}', bubbles: true, cancelable: true }})), true"
            ),
        )
        .await;
    }
    settle().await;
}

/// `on_day_activate` opens a day from a double-click away from the day's
/// controls, a touch double-tap, and Enter on the focused box or Week
/// column. The day number still zooms, and joins the Tab order so the drill
/// stays one key away. The fixture without the hook keeps Enter = zoom
/// (`the_keys_move_page_and_drill`).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn a_day_opens_by_double_click_double_tap_and_enter() {
    let h = open().await;
    wait_for_selector(&h, &edit_day("2026-10-18")).await;
    assert_eq!(opened(&h).await, (String::new(), 0));

    // A double-click on an empty day, and on a chip's text (not a control),
    // opens the day once and leaves no word selected.
    double_click(&h, &edit_day("2026-10-18"), 0.5, 0.9).await;
    assert_eq!(opened(&h).await, ("2026-10-18".to_string(), 1));
    assert_eq!(edit_zoom(&h).await, "month", "a double-click never zooms");
    double_click(
        &h,
        &format!("{} li[data-entry] span", edit_day("2026-10-21")),
        0.3,
        0.5,
    )
    .await;
    assert_eq!(opened(&h).await, ("2026-10-21".to_string(), 2));
    let selected = eval_json(&h, "window.getSelection().toString()").await;
    assert_eq!(selected, json!(""), "the double-click left a word selected");

    // A double-clicked control keeps its own meaning.
    double_click(
        &h,
        &format!("{} [data-day-add]", edit_day("2026-10-22")),
        0.5,
        0.5,
    )
    .await;
    assert_eq!(opened(&h).await.1, 2, "a double-clicked + opened the day");
    assert_eq!(testid(&h, "editable-added").await, "2026-10-22");

    // Touch: two quick taps on one day open it, and the compatibility
    // dblclick a touch browser fires next does not open it again.
    let box25 = edit_day("2026-10-25");
    taps(&h, &box25, &box25, 0, "touch").await;
    assert_eq!(opened(&h).await, ("2026-10-25".to_string(), 3));
    eval_json(
        &h,
        &format!(
            "document.querySelector('{box25}').dispatchEvent(new MouseEvent('dblclick', {{ bubbles: true, cancelable: true }})), true"
        ),
    )
    .await;
    settle().await;
    assert_eq!(opened(&h).await.1, 3, "a double-tap opened the day twice");
    // Slow taps, taps on two days and mouse pointers are not a double-tap.
    let box26 = edit_day("2026-10-26");
    taps(&h, &box26, &box26, 700, "touch").await;
    taps(&h, &edit_day("2026-10-27"), &box26, 0, "touch").await;
    taps(&h, &box26, &box26, 0, "mouse").await;
    assert_eq!(opened(&h).await, ("2026-10-25".to_string(), 3));

    // Enter on the focused box opens it: with the hook, Enter never zooms.
    let active_box = format!("{EDIT} td[data-day][tabindex=\"0\"]");
    let active = eval_json(
        &h,
        &format!("document.querySelector('{active_box}').dataset.day"),
    )
    .await
    .as_str()
    .expect("an active box")
    .to_string();
    focus(&h, &active_box).await;
    press(&h, Key::Enter).await;
    assert_eq!(opened(&h).await, (active.clone(), 4));
    assert_eq!(edit_zoom(&h).await, "month", "Enter zoomed instead");

    // The box's next Tab stop is its day number, which still zooms.
    press(&h, Key::Tab).await;
    let on_number = eval_json(
        &h,
        "document.activeElement.hasAttribute('data-day-number') && document.activeElement.closest('td[data-day]').dataset.day",
    )
    .await;
    assert_eq!(on_number, json!(active), "Tab from the box");
    press(&h, Key::Enter).await;
    assert_eq!(edit_zoom(&h).await, "day", "the day number zooms");

    // Week: each column is a tab stop that Enter opens; a double-click on
    // the column opens it; Enter on a control inside keeps its meaning.
    click(
        &h,
        &format!("{EDIT} [data-month-roster-action=\"zoom-week\"]"),
    )
    .await;
    wait_for_selector(&h, &format!("{EDIT} [data-week-column]")).await;
    let stops = eval_json(
        &h,
        &format!(
            "[...document.querySelectorAll('{EDIT} [data-week-column]')].map((c) => c.tabIndex)"
        ),
    )
    .await;
    assert_eq!(stops, json!([0, 0, 0, 0, 0, 0, 0]), "one stop per column");
    let column = format!("{EDIT} [data-week-column][data-day=\"{active}\"]");
    focus(&h, &column).await;
    press(&h, Key::Enter).await;
    assert_eq!(opened(&h).await, (active.clone(), 5));
    double_click(&h, &column, 0.3, 0.98).await;
    assert_eq!(opened(&h).await, (active.clone(), 6));
    focus(&h, &format!("{column} [data-day-number]")).await;
    press(&h, Key::Enter).await;
    assert_eq!(edit_zoom(&h).await, "day", "the column's day number zooms");
    assert_eq!(opened(&h).await.1, 6);
    assert_no_browser_errors(&h, "day activation").await;
}

/// A "+" on exactly the days the host marks addable and an "x" in exactly
/// the entries it marks removable, in all three zooms: named buttons that
/// propose, which the host answers by changing `days`. In the month grid
/// they follow the active box's roving Tab order. The fixture without the
/// callbacks draws neither.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn add_and_remove_propose_and_the_host_answers() {
    let h = open().await;
    wait_for_selector(&h, &edit_day("2026-10-21")).await;
    let facts = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const edit = document.querySelector('{EDIT}');
                const adds = [...edit.querySelectorAll('td[data-day] [data-day-add]')];
                const removes = (iso) => edit.querySelectorAll(`td[data-day="${{iso}}"] [data-entry-remove]`).length;
                return {{
                    adds: adds.map((b) => b.closest('td').dataset.day),
                    realButtons: adds.concat([...edit.querySelectorAll('[data-entry-remove]')])
                        .every((b) => b.tagName === 'BUTTON' && b.type === 'button' && b.getAttribute('aria-label')),
                    addName: edit.querySelector('td[data-day="2026-10-21"] [data-day-add]').getAttribute('aria-label'),
                    removePast: removes('2026-10-13'),
                    removeToday: removes('{TODAY}'),
                    removeFuture: removes('2026-10-21'),
                    removeName: edit.querySelector('[data-entry-remove="p2-2026-10-21"]').getAttribute('aria-label'),
                    others: document.querySelectorAll('{ROOT} [data-day-add], {ROOT} [data-entry-remove]').length,
                }};
            }})()"#
        ),
    )
    .await;
    let mut addable: Vec<String> = (15..=31)
        .filter(|dom| *dom != 28 && *dom != 29)
        .map(|dom| format!("2026-10-{dom}"))
        .collect();
    addable.push("2026-11-01".to_string());
    assert_eq!(
        facts["adds"],
        json!(addable),
        "only the published days after today: {facts}"
    );
    assert_eq!(facts["realButtons"], json!(true), "{facts}");
    assert_eq!(
        facts["addName"],
        json!("Add someone on Wednesday, Oct 21, 2026")
    );
    assert_eq!(facts["removePast"], json!(0), "{facts}");
    assert_eq!(facts["removeToday"], json!(0), "{facts}");
    assert_eq!(facts["removeFuture"], json!(4), "{facts}");
    assert_eq!(
        facts["removeName"],
        json!("Remove Mia Chen from Wednesday, Oct 21, 2026")
    );
    assert_eq!(facts["others"], json!(0), "a host without the callbacks");

    // Each proposes; the host answers with new days.
    click(&h, &format!("{EDIT} [data-entry-remove=\"p2-2026-10-21\"]")).await;
    assert_eq!(testid(&h, "editable-removed").await, "p2-2026-10-21");
    click(&h, &format!("{} [data-day-add]", edit_day("2026-10-21"))).await;
    assert_eq!(testid(&h, "editable-added").await, "2026-10-21");
    let answered = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const cell = document.querySelector('{EDIT} td[data-day="2026-10-21"]');
                return {{
                    removed: !cell.querySelector('li[data-entry="p2-2026-10-21"]'),
                    added: !!cell.querySelector('li[data-entry="new-0"] [data-entry-remove="new-0"]'),
                    removes: [...cell.querySelectorAll('[data-entry-remove]')].map((b) => 'remove:' + b.dataset.entryRemove),
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(answered["removed"], json!(true), "{answered}");
    assert_eq!(answered["added"], json!(true), "{answered}");

    // The clicks made 21 Oct the active box: its controls are in the Tab
    // order in DOM order (number, +, each x), and every other box's are not.
    let cell = edit_day("2026-10-21");
    focus(&h, &cell).await;
    let mut order = Vec::new();
    for _ in 0..12 {
        press(&h, Key::Tab).await;
        let here = eval_json(
            &h,
            r#"(() => {
                const a = document.activeElement;
                if (a.closest('td[data-day]')?.dataset.day !== '2026-10-21') return null;
                if (a.hasAttribute('data-day-number')) return 'number';
                if (a.hasAttribute('data-day-add')) return 'add';
                if (a.hasAttribute('data-entry-remove')) return 'remove:' + a.dataset.entryRemove;
                return a.tagName;
            })()"#,
        )
        .await;
        if here.is_null() {
            break;
        }
        order.push(here);
    }
    let mut expected = vec![json!("number"), json!("add")];
    expected.extend(answered["removes"].as_array().cloned().unwrap_or_default());
    assert_eq!(expected.len(), 6, "{answered}");
    assert_eq!(json!(order), json!(expected), "Tab order in the active box");
    let others = eval_json(
        &h,
        &format!(
            "[...document.querySelectorAll('{EDIT} td[data-day]:not([tabindex=\"0\"]) :is([data-day-add], [data-entry-remove])')].every((b) => b.tabIndex === -1)"
        ),
    )
    .await;
    assert_eq!(
        others,
        json!(true),
        "an inactive box's control is a Tab stop"
    );
    focus(&h, &format!("{EDIT} [data-entry-remove=\"new-0\"]")).await;
    press(&h, Key::Escape).await;
    assert_eq!(focused_day(&h).await, json!("2026-10-21"));
    let on_box = eval_json(&h, "document.activeElement.matches('td[data-day]')").await;
    assert_eq!(on_box, json!(true), "Escape from an x returns to its box");

    // Week: a + on each future column, an x in each removable chip.
    click(
        &h,
        &format!("{EDIT} [data-month-roster-action=\"zoom-week\"]"),
    )
    .await;
    wait_for_selector(&h, &format!("{EDIT} [data-week-column]")).await;
    let week = eval_json(
        &h,
        &format!(
            r#"(() => ({{
                adds: [...document.querySelectorAll('{EDIT} [data-week-column] [data-day-add]')].map((b) => b.closest('[data-week-column]').dataset.day),
                removes: document.querySelectorAll('{EDIT} [data-week-column][data-day="2026-10-21"] [data-entry-remove]').length,
            }}))()"#
        ),
    )
    .await;
    assert_eq!(
        week["adds"],
        json!([
            "2026-10-19",
            "2026-10-20",
            "2026-10-21",
            "2026-10-22",
            "2026-10-23",
            "2026-10-24",
            "2026-10-25"
        ]),
        "{week}"
    );
    assert_eq!(week["removes"], json!(4), "{week}");
    click(&h, &format!("{EDIT} [data-entry-remove=\"p7-2026-10-21\"]")).await;
    assert_eq!(testid(&h, "editable-removed").await, "p7-2026-10-21");

    // Day: the + in the head, an x beside each removable timeline name.
    click(
        &h,
        &format!("{EDIT} [data-week-column][data-day=\"2026-10-21\"] [data-day-number]"),
    )
    .await;
    wait_for_selector(&h, &format!("{EDIT} [data-day-view]")).await;
    let day_view = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const view = document.querySelector('{EDIT} [data-day-view]');
                return {{
                    adds: view.querySelectorAll('[data-day-add]').length,
                    rowRemoves: view.querySelectorAll('[data-day-row-name] [data-entry-remove]').length,
                    bars: view.querySelectorAll('[data-day-bar]').length,
                    name: view.querySelector('[data-entry-remove="p12-2026-10-21"]').getAttribute('aria-label'),
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(day_view["adds"], json!(1), "{day_view}");
    assert_eq!(
        day_view["rowRemoves"], day_view["bars"],
        "an x per working row: {day_view}"
    );
    assert_eq!(day_view["bars"], json!(3), "{day_view}");
    assert_eq!(
        day_view["name"],
        json!("Remove Julia Rossi from Wednesday, Oct 21, 2026")
    );
    click(
        &h,
        &format!("{EDIT} [data-entry-remove=\"p12-2026-10-21\"]"),
    )
    .await;
    assert_eq!(testid(&h, "editable-removed").await, "p12-2026-10-21");
    let bars = eval_json(
        &h,
        &format!("document.querySelectorAll('{EDIT} [data-day-view] [data-day-bar]').length"),
    )
    .await;
    assert_eq!(bars, json!(2), "the host's answer removed the row");
    assert_no_browser_errors(&h, "add and remove").await;
}

// ---- ldui-on0i: contrast on the roster's own fills ----------------------------

/// Every visible text element under `root`: its contrast against the fill it
/// actually sits on (each translucent background composited down to the
/// first opaque one), as axe's color-contrast measures it, but with no
/// "incomplete" escape. In this build axe cannot judge a daisyUI button's
/// text: the button's computed `background-image` is `none, none`, which
/// axe-core takes for an image, so it files the contrast as incomplete
/// (`bgImage`), never as a violation, and it did not report the week buttons
/// scrolled out of the viewport at all. That is how the week buttons at
/// 1.79:1 here (1.06:1 under 4iiz-Office's theme) passed this lane's axe
/// (measured 2026-10-12, ldui-on0i). Colours are normalised through a 1px
/// canvas, which parses every CSS colour form.
const CONTRAST_JS: &str = r#"(root, visible) => {
    const ctx = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
    const rgba = (css) => {
        ctx.clearRect(0, 0, 1, 1);
        ctx.fillStyle = 'rgba(0, 0, 0, 0)';
        ctx.fillStyle = css;
        ctx.fillRect(0, 0, 1, 1);
        const d = ctx.getImageData(0, 0, 1, 1).data;
        return [d[0], d[1], d[2], d[3] / 255];
    };
    const over = (top, under) => [0, 1, 2].map((i) => top[i] * top[3] + under[i] * (1 - top[3])).concat([1]);
    const fill = (el) => {
        const layers = [];
        for (let n = el; n; n = n.parentElement) {
            const c = rgba(getComputedStyle(n).backgroundColor);
            if (c[3] > 0) layers.push(c);
            if (c[3] >= 1) break;
        }
        return layers.reverse().reduce((under, top) => over(top, under), [255, 255, 255, 1]);
    };
    const lum = (c) => c.slice(0, 3).map((v) => {
        v /= 255;
        return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    }).reduce((sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i], 0);
    const ratio = (a, b) => {
        const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
        return (x + 0.05) / (y + 0.05);
    };
    const out = [];
    for (const el of root.querySelectorAll('*')) {
        if (![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue;
        if (!visible(el)) continue;
        const s = getComputedStyle(el);
        const back = fill(el);
        const size = parseFloat(s.fontSize);
        const large = size >= 24 || (size >= 18.66 && parseInt(s.fontWeight, 10) >= 700);
        out.push({
            text: el.textContent.trim().slice(0, 24),
            ratio: Math.round(ratio(over(rgba(s.color), back), back) * 100) / 100,
            need: large ? 3 : 4.5,
            week: !!el.closest('[data-month-roster-action="week"]'),
            outside: !!el.closest('td[data-outside] li[data-entry]'),
        });
    }
    return out;
}"#;

async fn contrast(h: &Harness, root: &str) -> Vec<Value> {
    eval_json(
        h,
        &format!("({CONTRAST_JS})(document.querySelector('{root}'), {VISIBLE_JS})"),
    )
    .await
    .as_array()
    .cloned()
    .unwrap_or_default()
}

fn below_aa(items: &[Value]) -> Vec<&Value> {
    items
        .iter()
        .filter(|i| i["ratio"].as_f64() < i["need"].as_f64())
        .collect()
}

/// The lowest ratio among the items flagged `key`; infinite when none is.
fn lowest(items: &[Value], key: &str) -> f64 {
    items
        .iter()
        .filter(|i| i[key] == json!(true))
        .filter_map(|i| i["ratio"].as_f64())
        .fold(f64::INFINITY, f64::min)
}

/// 4iiz-Office's staging axe (1440, its theme) found the month grid's week
/// buttons at 1.06:1 and an adjacent month's chips at 4.4:1. At Office's
/// width, in both themes and all three zooms of the editable roster (with
/// its "+" and "x"), every text clears AA on its own fill and axe reports
/// nothing serious. Then each old class is put back and the sweep must see
/// it: the week buttons fall below 3:1 without the header's content colour,
/// and the adjacent month's chips lose contrast at `/75`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_grid_text_is_readable_on_its_own_fills() {
    let h = open().await;
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("Office's width");
    settle().await;
    let action = |zoom: &str| format!("{EDIT} [data-month-roster-action=\"zoom-{zoom}\"]");
    for zoom in ["month", "week", "day"] {
        click(&h, &action(zoom)).await;
        if zoom == "day" {
            // Today is not editable; the next day has its "+" and "x".
            click(&h, &format!("{EDIT} [data-month-roster-action=\"next\"]")).await;
            wait_for_selector(&h, &format!("{EDIT} [data-day-view] [data-day-add]")).await;
        }
        for theme in ["light", "dark"] {
            set_theme(&h, theme).await;
            for root in [ROOT, EDIT] {
                let items = contrast(&h, root).await;
                assert!(
                    items.len() > 10,
                    "{zoom} {theme} {root}: the sweep measured only {} texts",
                    items.len()
                );
                let failing = below_aa(&items);
                assert!(
                    failing.is_empty(),
                    "{zoom} {theme} {root}: below AA on its own fill: {failing:#?}"
                );
            }
            if zoom == "month" {
                let items = contrast(&h, EDIT).await;
                assert!(
                    lowest(&items, "week").is_finite(),
                    "no week button measured"
                );
                assert!(
                    lowest(&items, "outside").is_finite(),
                    "no adjacent-month chip measured"
                );
            }
            axe_clean(&h, &format!("month roster, {zoom}, {theme}, 1440")).await;
        }
    }

    set_theme(&h, "light").await;
    click(&h, &action("month")).await;
    let base = contrast(&h, EDIT).await;
    eval_json(
        &h,
        &format!(
            "document.querySelectorAll('{EDIT} tbody th, {EDIT} [data-month-roster-action=\"week\"]').forEach((el) => el.classList.remove('text-table-header-content')), true"
        ),
    )
    .await;
    settle().await;
    let broken = contrast(&h, EDIT).await;
    assert!(
        lowest(&broken, "week") < 3.0,
        "the sweep missed the week buttons losing the header colour: {} -> {}",
        lowest(&base, "week"),
        lowest(&broken, "week")
    );
    eval_json(
        &h,
        &format!(
            "document.querySelectorAll('{EDIT} tbody th, {EDIT} [data-month-roster-action=\"week\"]').forEach((el) => el.classList.add('text-table-header-content')), document.querySelectorAll('{EDIT} td[data-outside]').forEach((td) => td.classList.replace('text-base-content/85', 'text-base-content/75')), true"
        ),
    )
    .await;
    settle().await;
    let dimmer = contrast(&h, EDIT).await;
    assert!(
        lowest(&dimmer, "outside") < lowest(&base, "outside"),
        "the sweep missed the adjacent month dimming to /75: {} -> {}",
        lowest(&base, "outside"),
        lowest(&dimmer, "outside")
    );
    eprintln!(
        "adjacent-month chips, light theme: /85 {:.2}:1, /75 {:.2}:1; week buttons {:.2}:1 (without the header colour {:.2}:1)",
        lowest(&base, "outside"),
        lowest(&dimmer, "outside"),
        lowest(&base, "week"),
        lowest(&broken, "week")
    );
    assert_no_browser_errors(&h, "contrast").await;
}

// ---- ldui-3kix: the shared axe gate judges what axe cannot -------------------

/// axe files a daisyUI button's text contrast as incomplete (`bgImage`,
/// from the `.btn`'s `none, none` background image) and never fails it, so
/// the shared gate re-measures those nodes. The week buttons are put back to
/// their pre-ldui-u0yr markup (a daisyUI ghost button that took its colour
/// from the cell), with the cell's colour lost, and scrolled into view: the
/// gate must name them. Restored, it must be clean. axe's own violations stay
/// empty throughout, which is the blind spot itself.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_shared_axe_gate_judges_the_button_text_axe_cannot() {
    let h = open().await;
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("a wide grid");
    eval_json(
        &h,
        &format!(
            "document.querySelector('{ROOT} [data-month-roster-action=\"week\"]').scrollIntoView({{ block: 'center' }}), true"
        ),
    )
    .await;
    settle().await;
    let in_view = |findings: &Value| {
        findings["unjudged"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            // Match on axe's selector: its `html` snippet truncates long
            // attributes ("data-month-roster-ac...").
            .filter(|n| {
                n["target"].as_array().is_some_and(|t| {
                    t.iter().any(|s| {
                        s.as_str()
                            .is_some_and(|s| s.contains("data-month-roster-action=\"week\""))
                    })
                })
            })
            .collect::<Vec<_>>()
    };

    let clean = common::axe::axe_findings(&h, Some(ROOT)).await;
    assert_eq!(clean["violations"], json!([]), "{clean}");
    assert_eq!(clean["unjudged"], json!([]), "{clean}");

    eval_json(
        &h,
        &format!(
            "document.querySelectorAll('{ROOT} tbody th').forEach((th) => th.classList.remove('text-table-header-content')), document.querySelectorAll('{ROOT} [data-month-roster-action=\"week\"]').forEach((b) => {{ b.dataset.newClass = b.className; b.className = 'btn ld-eased ld-pressable ld-focus-ring btn-ghost btn-xs px-1'; }}), true"
        ),
    )
    .await;
    settle().await;
    let broken = common::axe::axe_findings(&h, Some(ROOT)).await;
    let hits = in_view(&broken);
    assert!(
        !hits.is_empty(),
        "the gate missed the week buttons at the old colour: {broken}"
    );
    assert!(
        hits.iter().all(|n| n["ratio"].as_f64() < Some(3.0)),
        "{hits:?}"
    );
    assert_eq!(
        broken["violations"],
        json!([]),
        "axe now fails the buttons itself; the blind spot may be gone: {broken}"
    );

    eval_json(
        &h,
        &format!(
            "document.querySelectorAll('{ROOT} tbody th').forEach((th) => th.classList.add('text-table-header-content')), document.querySelectorAll('{ROOT} [data-month-roster-action=\"week\"]').forEach((b) => {{ b.className = b.dataset.newClass; }}), true"
        ),
    )
    .await;
    settle().await;
    let restored = common::axe::axe_findings(&h, Some(ROOT)).await;
    assert_eq!(restored["unjudged"], json!([]), "{restored}");
    assert_no_browser_errors(&h, "shared axe gate").await;
}

// ---- ldui-u0yr: the week button in ANY daisyUI build --------------------------

/// The rule the daisyUI build 4iiz-Office serves applies to a ghost button
/// at rest: its colour is `--color-base-content`, never the inherited
/// `currentColor` our 5.5.19 gives it. Injected unlayered, it outranks every
/// layered rule, exactly as Office's layered rule outranked an inherited
/// colour.
const OFFICE_GHOST_CSS: &str = ".btn-ghost { color: var(--color-base-content); }";

/// The focused element's outline against the fill around it (its header
/// cell's background, composited down to the first opaque one): whether
/// `:focus-visible` matched, the outline style, and the contrast ratio.
const RING_JS: &str = r#"(selector) => {
    const ctx = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
    const rgba = (css) => {
        ctx.clearRect(0, 0, 1, 1);
        ctx.fillStyle = 'rgba(0, 0, 0, 0)';
        ctx.fillStyle = css;
        ctx.fillRect(0, 0, 1, 1);
        const d = ctx.getImageData(0, 0, 1, 1).data;
        return [d[0], d[1], d[2], d[3] / 255];
    };
    const over = (top, under) => [0, 1, 2].map((i) => top[i] * top[3] + under[i] * (1 - top[3])).concat([1]);
    const lum = (c) => c.slice(0, 3).map((v) => {
        v /= 255;
        return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    }).reduce((sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i], 0);
    const b = document.querySelector(selector);
    const layers = [];
    for (let n = b.parentElement; n; n = n.parentElement) {
        const c = rgba(getComputedStyle(n).backgroundColor);
        if (c[3] > 0) layers.push(c);
        if (c[3] >= 1) break;
    }
    const fill = layers.reverse().reduce((under, top) => over(top, under), [255, 255, 255, 1]);
    const s = getComputedStyle(b);
    const ring = over(rgba(s.outlineColor), fill);
    const [x, y] = [lum(ring), lum(fill)].sort((p, q) => q - p);
    return {
        focusVisible: b.matches(':focus-visible'),
        style: s.outlineStyle,
        width: s.outlineWidth,
        color: s.outlineColor,
        ratio: Math.round((x + 0.05) / (y + 0.05) * 100) / 100,
    };
}"#;

/// ldui-on0i coloured only the week column's CELL, and the week button stayed
/// a `btn-ghost`. That read 9.89:1 here and 1.06:1 on Office staging, because
/// the ghost's rest colour is daisyUI's and the two builds disagree. Under
/// Office's ghost rule, the button now reads on the navy header at rest, on
/// hover (a translucent white, never a light fill) and with keyboard focus.
/// The old ghost markup, put back on one button, must fail, which proves the
/// emulation bites. The focused button's ring must also clear 3:1 against
/// the header (ldui-fq9x), and fall below it without its colour hook.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-month-roster)"]
async fn the_week_buttons_read_on_the_header_in_any_daisyui_build() {
    let h = open().await;
    h.set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("a wide grid");
    set_theme(&h, "light").await;
    let first =
        format!("{ROOT} tr[data-month-week=\"2026-09-28\"] [data-month-roster-action=\"week\"]");
    eval_json(
        &h,
        &format!(
            "(() => {{ const s = document.createElement('style'); s.id = 'office-ghost'; s.textContent = '{OFFICE_GHOST_CSS}'; document.head.append(s); document.querySelector('{first}').scrollIntoView({{ block: 'center' }}); return true; }})()"
        ),
    )
    .await;
    settle().await;
    let rest = contrast(&h, ROOT).await;
    assert!(
        lowest(&rest, "week") >= 4.5,
        "at rest under Office's ghost rule: {}",
        lowest(&rest, "week")
    );

    // Hover: a real pointer over the first week button.
    let point = eval_json(
        &h,
        &format!(
            "(() => {{ const r = document.querySelector('{first}').getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2]; }})()"
        ),
    )
    .await;
    let moved = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseMoved)
        .x(point[0].as_f64().expect("x"))
        .y(point[1].as_f64().expect("y"))
        .build()
        .expect("mouse move params");
    h.page().execute(moved).await.expect("hover");
    settle().await;
    let hovered = eval_json(
        &h,
        &format!(
            "(() => {{ const b = document.querySelector('{first}'); return {{ hover: b.matches(':hover'), fill: getComputedStyle(b).backgroundColor }}; }})()"
        ),
    )
    .await;
    assert_eq!(
        hovered["hover"],
        json!(true),
        "the pointer is not over it: {hovered}"
    );
    assert_ne!(
        hovered["fill"],
        json!("rgba(0, 0, 0, 0)"),
        "the hover fill never applied, so this measured rest again: {hovered}"
    );
    let on_hover = contrast(&h, ROOT).await;
    assert!(
        lowest(&on_hover, "week") >= 4.5,
        "on hover: {} ({hovered})",
        lowest(&on_hover, "week")
    );

    // Keyboard focus.
    eval_json(
        &h,
        &format!("document.querySelector('{first}').focus({{ focusVisible: true }}), true"),
    )
    .await;
    settle().await;
    let focused = contrast(&h, ROOT).await;
    assert!(
        lowest(&focused, "week") >= 4.5,
        "with focus: {}",
        lowest(&focused, "week")
    );
    // The focus ring itself: a focus indicator needs 3:1 against the fill
    // beside it (WCAG 1.4.11). Primary on the navy measured about 1.4:1 in
    // Office (ldui-fq9x); the week button takes the header's content colour.
    let ring = eval_json(&h, &format!("({RING_JS})('{first}')")).await;
    assert_eq!(ring["focusVisible"], json!(true), "{ring}");
    assert_ne!(ring["style"], json!("none"), "no ring drawn: {ring}");
    assert!(
        ring["ratio"].as_f64() >= Some(3.0),
        "the focus ring is too faint on the header: {ring}"
    );
    common::axe::assert_axe_clean(&h, Some(ROOT), "week buttons under Office's ghost rule").await;

    // Negative control for the ring: without the colour hook the ring is
    // primary again and must fall below 3:1 on the navy.
    let hook = "[--ld-focus-ring-color:var(--color-table-header-content)]";
    let primary_ring = eval_json(
        &h,
        &format!(
            "(() => {{ const b = document.querySelector('{first}'); b.blur(); b.classList.remove('{hook}'); b.focus({{ focusVisible: true }}); return ({RING_JS})('{first}'); }})()"
        ),
    )
    .await;
    assert!(
        primary_ring["ratio"].as_f64() < Some(3.0),
        "the primary ring was not faint here, so the ring check proves nothing: {primary_ring}"
    );
    eval_json(
        &h,
        &format!(
            "(() => {{ const b = document.querySelector('{first}'); b.classList.add('{hook}'); return true; }})()"
        ),
    )
    .await;

    // Negative control: the ldui-on0i markup on one button fails under the rule.
    eval_json(
        &h,
        &format!(
            "(() => {{ const b = document.querySelector('{first}'); b.dataset.newClass = b.className; b.className = 'btn ld-eased ld-pressable ld-focus-ring btn-ghost btn-xs px-1'; b.blur(); return true; }})()"
        ),
    )
    .await;
    let mouse_away = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseMoved)
        .x(1.0)
        .y(1.0)
        .build()
        .expect("mouse move params");
    h.page().execute(mouse_away).await.expect("move away");
    settle().await;
    let old = contrast(&h, ROOT).await;
    assert!(
        lowest(&old, "week") < 3.0,
        "the emulated Office rule did not reproduce the old 1.06:1 failure: {}",
        lowest(&old, "week")
    );
    eval_json(
        &h,
        &format!(
            "(() => {{ const b = document.querySelector('{first}'); b.className = b.dataset.newClass; document.getElementById('office-ghost').remove(); return true; }})()"
        ),
    )
    .await;
    assert_no_browser_errors(&h, "week buttons in any daisyUI build").await;
}
