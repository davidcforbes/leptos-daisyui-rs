//! Real-browser proof for `SelectableSummaryGroup` (`ldui-l5cw`): the
//! radiogroup encoding and its full keyboard contract, exactly one tab stop,
//! an unmeasured check that is never announced as zero, the selected
//! treatment, and a container-query grid that narrows with the GROUP rather
//! than the window.
//!
//! Drives the general demo app (`html_target: None`, like
//! `section_heading_smoke.rs`), because the fixtures live on the existing
//! `/components/selectable_summary` showcase route. Kept in its own file
//! rather than folded into `reactivity_smoke.rs`, whose check count is
//! pinned.
//!
//! ⚠ This suite needs its own `xtask` step to run in any lane; it is
//! reported to the owner of `xtask/**` rather than registered here.
//!
//! Every element is located by a stable `data-` attribute, never by
//! position: a positional query does not fail when layout changes, it
//! silently starts describing a different element.

mod common;

use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use serde_json::{Value, json};

const PAGE: &str = "/components/selectable_summary";
const CHECKS: &str = "[data-testid=\"selectable-summary-checks\"]";
const NARROW: &str = "[data-testid=\"selectable-summary-narrow\"]";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate selectable-summary fixture")
        .into_value()
        .expect("selectable-summary expression returns JSON")
}

/// Press one key with real CDP key events, then wait the settle delay.
/// `text` is supplied for the activation keys so Chromium performs the
/// native `<button>` default action.
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

async fn arrow_right(h: &pixelproof_web::Harness) {
    press_key(h, "ArrowRight", "ArrowRight", 39, None).await;
}

async fn arrow_left(h: &pixelproof_web::Harness) {
    press_key(h, "ArrowLeft", "ArrowLeft", 37, None).await;
}

async fn arrow_down(h: &pixelproof_web::Harness) {
    press_key(h, "ArrowDown", "ArrowDown", 40, None).await;
}

/// The whole group's observable state, keyed by card id -- never by index.
async fn snapshot(h: &pixelproof_web::Harness, fixture: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const fixture = document.querySelector('{fixture}');
                const group = fixture.querySelector('[data-selectable-summary-group]');
                const cards = Array.from(
                    group.querySelectorAll('[data-selectable-summary-card]')
                );
                const active = document.activeElement;
                return {{
                    groupRole: group.getAttribute('role'),
                    groupLabel: group.getAttribute('aria-label'),
                    groupLabelledBy: group.getAttribute('aria-labelledby'),
                    count: cards.length,
                    roles: Array.from(new Set(cards.map((c) => c.getAttribute('role')))),
                    ids: cards.map((c) => c.getAttribute('data-selectable-summary-card')),
                    checked: cards
                        .filter((c) => c.getAttribute('aria-checked') === 'true')
                        .map((c) => c.getAttribute('data-selectable-summary-card')),
                    tabStops: cards
                        .filter((c) => c.getAttribute('tabindex') === '0')
                        .map((c) => c.getAttribute('data-selectable-summary-card')),
                    activeId: active
                        ? active.getAttribute('data-selectable-summary-card')
                        : null,
                }};
            }})()"#
        ),
    )
    .await
}

/// One card's own announcement, addressed by its stable id.
async fn card(h: &pixelproof_web::Harness, fixture: &str, id: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const card = document.querySelector(
                    '{fixture} [data-selectable-summary-card="{id}"]'
                );
                const count = card.querySelector('[data-selectable-summary-count]');
                const style = getComputedStyle(card);
                return {{
                    name: card.getAttribute('aria-label'),
                    checked: card.getAttribute('aria-checked'),
                    status: card.getAttribute('data-selectable-summary-status'),
                    measured: card.getAttribute('data-selectable-summary-measured'),
                    countText: count ? count.textContent.trim() : null,
                    countItalic: count ? getComputedStyle(count).fontStyle : null,
                    disabled: card.disabled,
                    boxShadow: style.boxShadow,
                    outlineStyle: style.outlineStyle,
                    outlineWidth: style.outlineWidth,
                    outlineOffset: style.outlineOffset,
                    outlineColor: style.outlineColor,
                    borderWidth: style.borderTopWidth,
                    glyphs: card.querySelectorAll('svg').length,
                }};
            }})()"#
        ),
    )
    .await
}

/// The group is a NAMED radiogroup of radios -- the encoding chosen over
/// `aria-pressed` toggles, and the reason fourteen cards cost one tab stop.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn fourteen_cards_form_one_named_radiogroup() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, CHECKS).await;
    assert_eq!(s["groupRole"], json!("radiogroup"), "{s}");
    assert_eq!(s["groupLabel"], json!("Data quality checks"), "{s}");
    assert_eq!(
        s["count"],
        json!(14),
        "the acceptance shape is 14 cards: {s}"
    );
    assert_eq!(s["roles"], json!(["radio"]), "every card is a radio: {s}");

    assert_no_browser_errors(&h, "fourteen_cards_form_one_named_radiogroup").await;
}

/// Exactly one tab stop, and it is the selected card -- the APG roving
/// rule. Fourteen tab stops is the cost this encoding exists to avoid.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn exactly_one_tab_stop_sits_on_the_selected_card() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, CHECKS).await;
    assert_eq!(
        s["tabStops"],
        json!(["duplicate-records"]),
        "one tab stop, on the selected card: {s}"
    );
    assert_eq!(s["checked"], json!(["duplicate-records"]), "{s}");

    // With nothing selected the tab stop falls to the first selectable card
    // rather than disappearing.
    let matrix = snapshot(&h, "[data-testid=\"selectable-summary-matrix\"]").await;
    assert_eq!(matrix["checked"], json!([]), "{matrix}");
    assert_eq!(matrix["tabStops"], json!(["neutral"]), "{matrix}");

    assert_no_browser_errors(&h, "exactly_one_tab_stop_sits_on_the_selected_card").await;
}

/// A card that could not be measured must not read as a measured zero.
/// This is the defect the two constructors exist to make unspellable.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn an_unmeasured_check_is_never_announced_as_zero() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let zero = card(&h, CHECKS, "orphaned-rows").await;
    assert_eq!(zero["measured"], json!("true"), "{zero}");
    assert_eq!(zero["countText"], json!("0"), "{zero}");
    assert_eq!(zero["name"], json!("Orphaned rows: 0, clean"), "{zero}");
    assert_eq!(zero["countItalic"], json!("normal"), "{zero}");

    let unmeasured = card(&h, CHECKS, "feed-freshness").await;
    assert_eq!(unmeasured["measured"], json!("false"), "{unmeasured}");
    assert_eq!(unmeasured["status"], json!("unavailable"), "{unmeasured}");
    assert_eq!(
        unmeasured["countText"],
        json!("Not measured"),
        "{unmeasured}"
    );
    assert_eq!(
        unmeasured["name"],
        json!("Feed freshness: Not measured"),
        "{unmeasured}"
    );
    assert_eq!(
        unmeasured["countItalic"],
        json!("italic"),
        "the placeholder must not look like a number: {unmeasured}"
    );

    assert_no_browser_errors(&h, "an_unmeasured_check_is_never_announced_as_zero").await;
}

/// Every non-neutral status renders a glyph, so status is legible as a
/// SHAPE and not only as a hue.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn status_carries_a_glyph_shape_not_only_a_colour() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let matrix = "[data-testid=\"selectable-summary-matrix\"]";
    for id in ["clean", "warning", "error", "unavailable"] {
        let c = card(&h, matrix, id).await;
        assert!(
            c["glyphs"].as_u64().unwrap_or(0) >= 1,
            "status {id} must render a glyph: {c}"
        );
    }
    let neutral = card(&h, matrix, "neutral").await;
    assert_eq!(
        neutral["glyphs"],
        json!(0),
        "neutral has no honest glyph, only its label and count: {neutral}"
    );

    assert_no_browser_errors(&h, "status_carries_a_glyph_shape_not_only_a_colour").await;
}

/// Arrow keys move focus AND selection, skip the disabled card, and wrap --
/// the APG radio-group contract, on both axes.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn arrow_keys_move_and_select_skipping_the_disabled_card() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    // Start from a known card by clicking it (pointer selection), then
    // navigate with the keyboard.
    click(
        &h,
        &format!("{CHECKS} [data-selectable-summary-card=\"timezone-drift\"]"),
    )
    .await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(s["checked"], json!(["timezone-drift"]), "{s}");

    arrow_right(&h).await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(s["activeId"], json!("closed-with-tasks"), "{s}");
    assert_eq!(s["checked"], json!(["closed-with-tasks"]), "{s}");

    // ArrowDown is the same step as ArrowRight: a wrapped grid has no single
    // reading direction.
    arrow_down(&h).await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(s["activeId"], json!("feed-freshness"), "{s}");

    arrow_right(&h).await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(s["activeId"], json!("archive-integrity"), "{s}");

    // "retired-codes" is disabled and is the LAST card, so the next step
    // skips it and wraps to the first.
    arrow_right(&h).await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(
        s["activeId"],
        json!("duplicate-records"),
        "disabled cards are skipped and Next wraps: {s}"
    );
    assert_eq!(s["checked"], json!(["duplicate-records"]), "{s}");

    // Previous wraps the other way, also skipping the disabled card.
    arrow_left(&h).await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(s["activeId"], json!("archive-integrity"), "{s}");

    assert_no_browser_errors(&h, "arrow_keys_move_and_select_skipping_the_disabled_card").await;
}

/// Home and End reach the first and last SELECTABLE cards, and Space
/// activates the focused card through the native `<button>` path.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn home_end_and_space_complete_the_keyboard_contract() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    click(
        &h,
        &format!("{CHECKS} [data-selectable-summary-card=\"stale-status\"]"),
    )
    .await;

    press_key(&h, "End", "End", 35, None).await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(
        s["activeId"],
        json!("archive-integrity"),
        "End skips the disabled last card: {s}"
    );
    assert_eq!(s["checked"], json!(["archive-integrity"]), "{s}");

    press_key(&h, "Home", "Home", 36, None).await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(s["activeId"], json!("duplicate-records"), "{s}");

    // Space activates the focused card natively; move focus without changing
    // selection is not part of the radio contract, so assert Space keeps the
    // focused card selected rather than moving anywhere.
    press_key(&h, " ", "Space", 32, Some(" ")).await;
    let s = snapshot(&h, CHECKS).await;
    assert_eq!(s["activeId"], json!("duplicate-records"), "{s}");
    assert_eq!(s["checked"], json!(["duplicate-records"]), "{s}");

    assert_no_browser_errors(&h, "home_end_and_space_complete_the_keyboard_contract").await;
}

/// The disabled card refuses pointer selection too, and never becomes the
/// tab stop.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn a_disabled_card_cannot_be_selected_by_pointer_either() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let before = snapshot(&h, CHECKS).await;
    let disabled = card(&h, CHECKS, "retired-codes").await;
    assert_eq!(disabled["disabled"], json!(true), "{disabled}");

    click(
        &h,
        &format!("{CHECKS} [data-selectable-summary-card=\"retired-codes\"]"),
    )
    .await;
    let after = snapshot(&h, CHECKS).await;
    assert_eq!(after["checked"], before["checked"], "{after}");
    assert_ne!(after["tabStops"], json!(["retired-codes"]), "{after}");

    assert_no_browser_errors(&h, "a_disabled_card_cannot_be_selected_by_pointer_either").await;
}

/// Selection is carried by a ring -- present or absent, not a hue swap --
/// and never changes the border WIDTH, so selecting a card cannot reflow
/// the grid. The ring is an outline (ldui-xr7i), and the card's resting
/// elevation must survive selection: the regression this pins was a
/// box-shadow ring silently discarded by the `ld-card-depth` elevation
/// rule, leaving selected and unselected cards with an identical shadow.
/// Keyboard focus is the framework's `ld-focus-ring` (primary, offset 2),
/// so a focused selected card shows its ring 2px outside the selection
/// outline and is never indistinguishable from a merely selected one.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn selection_adds_a_ring_without_reflowing_the_grid() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let selected = card(&h, CHECKS, "duplicate-records").await;
    let unselected = card(&h, CHECKS, "missing-email").await;

    assert_eq!(selected["checked"], json!("true"), "{selected}");
    assert_eq!(unselected["checked"], json!("false"), "{unselected}");
    assert_eq!(selected["outlineStyle"], json!("solid"), "{selected}");
    assert_eq!(selected["outlineWidth"], json!("2px"), "{selected}");
    assert_eq!(selected["outlineOffset"], json!("0px"), "{selected}");
    assert_eq!(
        unselected["outlineStyle"],
        json!("none"),
        "the unselected card must carry no ring at all: {unselected}"
    );
    assert_eq!(
        selected["borderWidth"], unselected["borderWidth"],
        "selection must not change the border width: {selected} vs {unselected}"
    );
    // Elevation is a fixed property of the card and must not vanish under
    // selection (nor be replaced by the ring): both states carry the same
    // declared card shadow.
    assert!(
        selected["boxShadow"]
            .as_str()
            .is_some_and(|s| s.contains("0px 2px 4px")),
        "the selected card must keep its ld-card-depth elevation: {selected}"
    );
    assert_eq!(
        selected["boxShadow"], unselected["boxShadow"],
        "elevation is independent of selection: {selected} vs {unselected}"
    );

    // Keyboard focus on the selected card is visibly distinct from selection:
    // ArrowLeft from the card after it moves focus back onto it by keyboard,
    // so the browser applies :focus-visible and the offset outline paints.
    click(
        &h,
        &format!("{CHECKS} [data-selectable-summary-card=\"missing-email\"]"),
    )
    .await;
    press_key(&h, "ArrowLeft", "ArrowLeft", 37, None).await;
    let focused = card(&h, CHECKS, "duplicate-records").await;
    assert_eq!(focused["checked"], json!("true"), "{focused}");
    assert_eq!(
        focused["outlineOffset"],
        json!("2px"),
        "a keyboard-focused selected card must show the framework focus ring at offset 2: {focused}"
    );
    assert_eq!(focused["outlineWidth"], json!("2px"), "{focused}");
    assert_eq!(focused["outlineStyle"], json!("solid"), "{focused}");
    assert_ne!(
        focused["outlineOffset"], selected["outlineOffset"],
        "focus must sit visibly outside the selection outline: {focused} vs {selected}"
    );

    assert_no_browser_errors(&h, "selection_adds_a_ring_without_reflowing_the_grid").await;
}

/// The grid follows the GROUP's width, not the window's: the same items in
/// a constrained column resolve to FEWER columns at the identical viewport.
/// A viewport breakpoint could not tell the two apart (`ldui-tnyq`).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn the_grid_narrows_with_the_group_not_the_window() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let columns = eval_json(
        &h,
        &format!(
            r#"(() => {{
                const track = (sel) => getComputedStyle(
                    document.querySelector(sel + ' [data-selectable-summary-group]')
                ).gridTemplateColumns.split(' ').length;
                return {{ wide: track('{CHECKS}'), narrow: track('{NARROW}') }};
            }})()"#
        ),
    )
    .await;

    let wide = columns["wide"].as_u64().expect("wide column count");
    let narrow = columns["narrow"].as_u64().expect("narrow column count");
    assert!(
        narrow < wide,
        "a constrained column must resolve to fewer columns at the same viewport: {columns}"
    );
    assert!(narrow >= 2, "never a single full-bleed column: {columns}");

    assert_no_browser_errors(&h, "the_grid_narrows_with_the_group_not_the_window").await;
}

/// A visible heading names the group through `aria-labelledby`, and the
/// `aria-label` is suppressed so it cannot override that name.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn a_visible_heading_can_name_the_group_instead() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let s = snapshot(&h, "[data-testid=\"selectable-summary-labelled\"]").await;
    assert_eq!(
        s["groupLabelledBy"],
        json!("selectable-summary-labelled-heading"),
        "{s}"
    );
    assert_eq!(
        s["groupLabel"],
        Value::Null,
        "aria-label must be suppressed so it cannot override the heading: {s}"
    );

    assert_no_browser_errors(&h, "a_visible_heading_can_name_the_group_instead").await;
}

/// Framework-owned copy is reactive: switching locale rewrites the spoken
/// status words and the unavailable placeholder, and the unmeasured card
/// still never reads as a number.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (needs an xtask step: test-selectable-summary)"]
async fn localized_copy_rewrites_the_generated_text() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let localized = "[data-testid=\"selectable-summary-localized\"]";
    let before = card(&h, localized, "fraicheur").await;
    assert_eq!(before["countText"], json!("Not measured"), "{before}");

    click(&h, "[data-testid=\"selectable-summary-locale-toggle\"]").await;

    let after = card(&h, localized, "fraicheur").await;
    assert_eq!(after["countText"], json!("Non mesure"), "{after}");
    assert_eq!(
        after["name"],
        json!("Fraicheur du flux: Non mesure"),
        "{after}"
    );

    let warning = card(&h, localized, "doublons").await;
    assert_eq!(
        warning["name"],
        json!("Doublons: 12, a verifier"),
        "{warning}"
    );

    assert_no_browser_errors(&h, "localized_copy_rewrites_the_generated_text").await;
}
