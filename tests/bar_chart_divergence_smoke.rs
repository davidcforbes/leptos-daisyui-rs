//! Real-browser proof for the typed diverging `BarChart` (ldui-y2ed).
//!
//! **Compile-only pending a gate run on this machine** — this lane is not yet
//! registered in `xtask`, and the primary evidence for the feature is the
//! native coverage in `src/charts/bar_chart/{normalize,geometry,interaction,
//! format,types}.rs` plus `src/charts/bar_chart/tests.rs`, which prove the
//! domain arithmetic, the reducer, the formatter and the role decisions
//! without a browser. What only a browser can add is the rendered geometry and
//! the live focus/activation journeys, which is what these tests are.
//!
//! Every test drives `/components/charts`, whose diverging fixture carries a
//! negative, an exact zero, a positive, a missing measurement, and a pair of
//! equal magnitudes with opposite signs.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, oracle,
    wait_for_selector,
};
use serde_json::{Value, json};

const CHART: &str = "[data-testid='diverging-bar-chart'] [data-testid='bar-chart']";
const NEUTRAL: &str = "[data-testid='neutral-bar-chart'] [data-testid='bar-chart']";

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .expect("evaluate bar-chart fixture")
        .into_value()
        .expect("bar-chart expression returns JSON")
}

/// Reads every drawn bar's rectangle and key straight out of the DOM.
async fn bars(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const root = document.querySelector("{CHART}");
                return Array.from(root.querySelectorAll('g[data-bar-chart-bar]')).map((g) => {{
                    const rect = g.querySelector('rect');
                    return {{
                        key: g.dataset.barKey,
                        status: g.dataset.status ?? null,
                        missing: g.hasAttribute('data-bar-missing'),
                        x: rect ? Number(rect.getAttribute('x')) : null,
                        y: rect ? Number(rect.getAttribute('y')) : null,
                        width: rect ? Number(rect.getAttribute('width')) : null,
                        height: rect ? Number(rect.getAttribute('height')) : null,
                        capped: !!g.querySelector('[data-bar-chart-cap]'),
                    }};
                }});
            }})()"#
        ),
    )
    .await
}

async fn table_rows(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const table = document.querySelector("{CHART} [data-bar-chart-table]");
                return {{
                    headers: Array.from(table.querySelectorAll('thead th')).map(
                        (th) => th.textContent.trim()
                    ),
                    rows: Array.from(table.querySelectorAll('tbody tr')).map((tr) => ({{
                        key: tr.dataset.barKey,
                        cells: Array.from(tr.children).map((c) => c.textContent.trim()),
                    }})),
                }};
            }})()"#
        ),
    )
    .await
}

/// Focuses the target for `key` and sends one key event to it.
async fn press_on(harness: &pixelproof_web::Harness, key: &str, dom_key: &str) {
    let expression = format!(
        r#"(() => {{
            const target = document.querySelector(
                "{CHART} [data-bar-chart-focus][data-bar-key='{key}']"
            );
            target.focus();
            target.dispatchEvent(new KeyboardEvent('keydown', {{
                key: '{dom_key}', bubbles: true, cancelable: true
            }}));
            return true;
        }})()"#
    );
    let _ = eval_json(harness, &expression).await;
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

async fn focused_key(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        "(() => document.activeElement?.dataset?.barKey ?? null)()",
    )
    .await
}

// ── signed geometry ─────────────────────────────────────────────────────────

/// The core defect: a negative value must extend the other way from one zero
/// line, not become a negative width.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn negative_and_positive_bars_diverge_from_one_zero_rule() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, CHART).await;
    begin_browser_error_capture(&harness).await;

    let rules: Value = eval_json(
        &harness,
        &format!(
            r#"(() => {{
                const lines = document.querySelectorAll(
                    "{CHART} [data-bar-chart-zero-rule]"
                );
                return lines.length === 1 ? Number(lines[0].getAttribute('x1')) : null;
            }})()"#,
        ),
    )
    .await;
    let zero = rules.as_f64().expect("exactly one zero rule is drawn");

    let bars = bars(&harness).await;
    let bars = bars.as_array().expect("bars are an array");
    for bar in bars {
        if bar["missing"] == json!(true) {
            continue;
        }
        let width = bar["width"].as_f64().expect("a drawn bar has a width");
        let height = bar["height"].as_f64().expect("a drawn bar has a height");
        assert!(width >= 0.0, "{bar:?} has a negative width");
        assert!(height >= 0.0, "{bar:?} has a negative height");
        let x = bar["x"].as_f64().expect("a drawn bar has an x");
        assert!(
            (x - zero).abs() < 0.51 || (x + width - zero).abs() < 0.51,
            "{bar:?} does not meet the zero rule at {zero}"
        );
    }

    let find = |key: &str| {
        bars.iter()
            .find(|bar| bar["key"] == json!(key))
            .cloned()
            .unwrap_or_else(|| panic!("no bar for {key}"))
    };
    let negative = find("harbour");
    let positive = find("east");
    assert!(
        negative["x"].as_f64().unwrap() < zero,
        "a negative value must sit left of zero"
    );
    assert!(
        positive["x"].as_f64().unwrap() >= zero - 0.51,
        "a non-negative value must sit right of zero"
    );
    assert!(
        (negative["width"].as_f64().unwrap() - positive["width"].as_f64().unwrap()).abs() < 0.51,
        "equal magnitudes must have equal geometry: {negative:?} vs {positive:?}"
    );

    assert_no_browser_errors(&harness, "diverging bar geometry").await;
}

/// A gap in the data draws nothing and reads as missing; an exact zero draws a
/// zero-length bar and is still a real, reachable measurement.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_missing_value_draws_no_bar_and_a_zero_draws_a_reachable_one() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, CHART).await;
    begin_browser_error_capture(&harness).await;

    let bars = bars(&harness).await;
    let bars = bars.as_array().expect("bars are an array");
    let missing = bars
        .iter()
        .find(|bar| bar["key"] == json!("riverside"))
        .expect("the missing office is still listed");
    assert_eq!(missing["missing"], json!(true));
    assert_eq!(
        missing["width"],
        json!(null),
        "a missing measurement must not draw a rect a reader would take for zero"
    );

    let zero = bars
        .iter()
        .find(|bar| bar["key"] == json!("central"))
        .expect("the zero office is drawn");
    assert_eq!(zero["width"], json!(0.0));

    let reachable: Value = eval_json(
        &harness,
        &format!(
            r#"(() => ({{
                zero: !!document.querySelector(
                    "{CHART} [data-bar-chart-focus][data-bar-key='central']"
                ),
                missing: !!document.querySelector(
                    "{CHART} [data-bar-chart-focus][data-bar-key='riverside']"
                ),
            }}))()"#,
        ),
    )
    .await;
    assert_eq!(reachable["zero"], json!(true), "zero is a measurement");
    assert_eq!(reachable["missing"], json!(false), "a gap is not");

    assert_no_browser_errors(&harness, "missing and zero bars").await;
}

// ── the accessibility contract ──────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn the_chart_is_named_described_and_tabulated_with_one_tab_stop() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, CHART).await;
    begin_browser_error_capture(&harness).await;

    let semantics: Value = eval_json(
        &harness,
        &format!(
            r#"(() => {{
                const root = document.querySelector("{CHART}");
                const svg = root.querySelector('[data-bar-chart-plot]');
                const labelled = svg.getAttribute('aria-labelledby').split(' ');
                const targets = Array.from(root.querySelectorAll('[data-bar-chart-focus]'));
                return {{
                    rootRole: root.getAttribute('role'),
                    rootLabel: root.getAttribute('aria-label'),
                    layout: root.dataset.barChartLayout,
                    svgRole: svg.getAttribute('role'),
                    title: document.getElementById(labelled[0])?.textContent?.trim(),
                    desc: document.getElementById(labelled[1])?.textContent?.trim(),
                    targetRoles: Array.from(new Set(targets.map((t) => t.getAttribute('role')))),
                    tabStops: targets.filter((t) => t.getAttribute('tabindex') === '0').length,
                    tableRows: root.querySelectorAll('[data-bar-chart-table] tbody tr').length,
                }};
            }})()"#,
        ),
    )
    .await;

    assert_eq!(semantics["rootRole"], json!("group"));
    assert_eq!(
        semantics["rootLabel"],
        json!("Current minus trailing baseline by office")
    );
    assert_eq!(semantics["layout"], json!("diverging-horizontal"));
    assert_eq!(
        semantics["svgRole"],
        json!("group"),
        "role=img would make the focusable targets presentational"
    );
    assert_eq!(
        semantics["title"],
        json!("Current minus trailing baseline by office")
    );
    assert_eq!(
        semantics["desc"],
        json!("Signed delta to the trailing 12-week baseline, most dragging first.")
    );
    assert_eq!(semantics["targetRoles"], json!(["button"]));
    assert_eq!(
        semantics["tabStops"],
        json!(1),
        "a composite widget is one tab stop, not one per bar"
    );
    assert_eq!(
        semantics["tableRows"],
        json!(6),
        "every item, including the missing one, is in the semantic table"
    );

    assert_no_browser_errors(&harness, "bar chart semantics").await;
}

/// The chart without a callback still navigates and describes itself, but no
/// bar claims to be a button; and the legacy positional charts on the same
/// page gain no tab stops at all.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_chart_with_no_callback_claims_no_button_and_legacy_charts_claim_nothing() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, NEUTRAL).await;
    begin_browser_error_capture(&harness).await;

    let roles: Value = eval_json(
        &harness,
        &format!(
            r#"(() => {{
                const neutral = document.querySelector("{NEUTRAL}");
                const legacy = Array.from(document.querySelectorAll('svg')).filter(
                    (svg) => !svg.hasAttribute('data-bar-chart-plot')
                        && !svg.closest('[data-testid="bar-chart"]')
                );
                return {{
                    targetRoles: Array.from(new Set(
                        Array.from(neutral.querySelectorAll('[data-bar-chart-focus]'))
                            .map((t) => t.getAttribute('role'))
                    )),
                    caps: neutral.querySelectorAll('[data-bar-chart-cap]').length,
                    legacyTabStops: legacy.reduce(
                        (n, svg) => n + svg.querySelectorAll('[tabindex]').length, 0
                    ),
                }};
            }})()"#,
        ),
    )
    .await;

    assert_eq!(
        roles["targetRoles"],
        json!(["group"]),
        "nothing may announce itself as a button when pressing it does nothing"
    );
    assert_eq!(
        roles["caps"],
        json!(0),
        "an activity measure carries no judgement, so no bar is capped"
    );
    assert_eq!(
        roles["legacyTabStops"],
        json!(0),
        "the preserved positional charts must gain no tab stops"
    );

    assert_no_browser_errors(&harness, "non-activating and legacy charts").await;
}

// ── activation and identity ─────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn enter_activates_the_focused_bar_and_emits_its_stable_key() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, CHART).await;
    begin_browser_error_capture(&harness).await;

    press_on(&harness, "west", "Enter").await;

    let state = oracle(&harness).await;
    let activation = &state["state"]["state"]["bar_chart.activation"];
    assert_eq!(activation["categoryKey"], json!("west"));
    assert_eq!(activation["categoryLabel"], json!("West"));
    assert_eq!(activation["value"], json!(9.5));
    assert_eq!(activation["displayValue"], json!("9.5 pts"));
    assert_eq!(activation["status"], json!("favorable"));
    assert_eq!(activation["source"], json!("keyboard"));
    assert_eq!(
        state["state"]["state"]["bar_chart.activation_count"],
        json!(1),
        "one key press is one activation"
    );
    assert!(
        activation.get("categoryIndex").is_none(),
        "an index would re-point at a different office after a sort"
    );

    assert_no_browser_errors(&harness, "keyboard activation").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_pointer_click_activates_the_same_bar_with_a_pointer_source() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, CHART).await;
    begin_browser_error_capture(&harness).await;

    click(
        &harness,
        &format!("{CHART} [data-bar-chart-focus][data-bar-key='north']"),
    )
    .await;

    let state = oracle(&harness).await;
    let activation = &state["state"]["state"]["bar_chart.activation"];
    assert_eq!(activation["categoryKey"], json!("north"));
    assert_eq!(activation["value"], json!(-12.5));
    assert_eq!(
        activation["displayValue"],
        json!("-12.5 pts"),
        "the payload states the value exactly as the bar draws it"
    );
    assert_eq!(activation["source"], json!("pointer"));

    assert_no_browser_errors(&harness, "pointer activation").await;
}

/// Arrow navigation moves one activatable bar at a time, skipping the gap, and
/// Escape gives up the highlight without moving the tab stop.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn arrow_keys_rove_over_activatable_bars_and_escape_dismisses() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, CHART).await;
    begin_browser_error_capture(&harness).await;

    press_on(&harness, "harbour", "ArrowDown").await;
    assert_eq!(
        focused_key(&harness).await,
        json!("central"),
        "the missing office has no target, so one step skips it"
    );

    press_on(&harness, "north", "Home").await;
    assert_eq!(focused_key(&harness).await, json!("north"));

    press_on(&harness, "north", "End").await;
    assert_eq!(focused_key(&harness).await, json!("west"));

    press_on(&harness, "west", "Escape").await;
    let active: Value = eval_json(
        &harness,
        &format!("(() => document.querySelector(\"{CHART}\").dataset.activeCategory || null)()"),
    )
    .await;
    assert_eq!(active, json!(null), "Escape drops the highlight");

    assert_no_browser_errors(&harness, "roving navigation").await;
}

// ── reconciliation by key ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn focus_follows_a_bar_through_a_sort_and_moves_predictably_after_removal() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, CHART).await;
    begin_browser_error_capture(&harness).await;

    press_on(&harness, "north", "Home").await;
    assert_eq!(focused_key(&harness).await, json!("north"));

    click(&harness, "[data-testid='bar-chart-sort']").await;
    assert_eq!(
        focused_key(&harness).await,
        json!("north"),
        "a sort must not hand focus to whatever now sits at the old index"
    );

    click(&harness, "[data-testid='bar-chart-remove']").await;
    let after = focused_key(&harness).await;
    assert_ne!(
        after,
        json!("north"),
        "the removed office cannot hold focus"
    );
    assert_ne!(after, json!(null), "focus must not simply vanish");

    click(&harness, "[data-testid='bar-chart-restore']").await;
    assert_no_browser_errors(&harness, "keyed reconciliation").await;
}

// ── reactive copy ───────────────────────────────────────────────────────────

/// EN -> ES -> EN changes the words and nothing else: keys, values and order
/// are byte-identical either side of the round trip.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_locale_change_moves_the_copy_and_leaves_the_data_alone() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, CHART).await;
    begin_browser_error_capture(&harness).await;

    let english = table_rows(&harness).await;
    let english_bars = bars(&harness).await;
    assert_eq!(
        english["headers"],
        json!(["Category", "Value", "Status"]),
        "the defaults reproduce the strings the chart already emitted"
    );

    click(&harness, "[data-testid='bar-chart-locale']").await;
    let spanish = table_rows(&harness).await;
    assert_eq!(spanish["headers"], json!(["Categoria", "Valor", "Estado"]));
    let spanish_keys: Vec<&Value> = spanish["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|row| &row["key"])
        .collect();
    let english_keys: Vec<&Value> = english["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|row| &row["key"])
        .collect();
    assert_eq!(spanish_keys, english_keys, "no key or order may move");
    let missing_row = spanish["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|row| row["key"] == json!("riverside"))
        .expect("the missing office is still a row");
    assert_eq!(
        missing_row["cells"][1],
        json!("Sin dato"),
        "the missing-value copy is supplied, not hardcoded"
    );

    click(&harness, "[data-testid='bar-chart-locale']").await;
    assert_eq!(
        table_rows(&harness).await,
        english,
        "the round trip must be exact"
    );
    assert_eq!(
        bars(&harness).await,
        english_bars,
        "geometry must not move when only words change"
    );

    assert_no_browser_errors(&harness, "locale round trip").await;
}
