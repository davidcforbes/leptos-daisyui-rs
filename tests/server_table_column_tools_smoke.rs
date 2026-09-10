//! Real-browser proof for `ServerDataTable`'s opt-in opinionated capabilities.
//!
//! Two independent contracts share this lane because they share a demo page
//! and a release server: the presentation tools (`ldui-9j16`) below, and the
//! controlled checkbox multi-selection (`ldui-px06`) at the end of the file.
//!
//! # Presentation tools (ldui-9j16)
//!
//! The compact gear column chooser stays inside the viewport
//! and closes on `Escape` with focus restored, a required column can never
//! be hidden or even offered in the chooser list, the caller's toolbar
//! Export action sits beside the chooser, and the atomic
//! `on_displayed_slice` projection reflects ONLY the current server page --
//! never the fixture's full population. That last assertion is the
//! feature's whole point: a "CSV export" wired to this projection cannot
//! silently ship the wrong row count and call it complete.
//!
//! Run live and GREEN. Native evidence for the pure ordering/visibility/
//! projection functions lives in
//! `src/components/data_table/server_column_tools.rs`'s own test module;
//! this file is the DOM-level companion proof over the demo's
//! "Server-Owned Table" fixture (`#server-table`, a 57-row simulated
//! backend paged 10 rows at a time -- see `demo/src/demos/data_table.rs`).

mod common;

use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .expect("evaluate server-table column-tools fixture")
        .into_value()
        .expect("server-table column-tools expression returns JSON")
}

const HISTORY_ROOT: &str = "#server-entity-history";

async fn history_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#server-entity-history');
            const parse = testid => JSON.parse(
                root.querySelector(`[data-testid="${testid}"]`).textContent
            );
            return {
                accepted: parse('server-entity-history-accepted-query'),
                proposed: parse('server-entity-history-proposed-query'),
                acceptedIds: parse('server-entity-history-accepted-ids'),
                domIds: Array.from(root.querySelectorAll('tbody tr[data-row-key]'))
                    .map(row => row.dataset.rowKey),
                total: Number(root.querySelector(
                    '[data-testid="server-entity-history-total"]'
                ).textContent),
                proposals: Number(root.querySelector(
                    '[data-testid="server-entity-history-proposals"]'
                ).textContent),
                preferenceProposals: root.querySelector(
                    '[data-testid="server-entity-history-preference-proposals"]'
                ) ? Number(root.querySelector(
                    '[data-testid="server-entity-history-preference-proposals"]'
                ).textContent) : null,
                preferences: parse('server-entity-history-preferences'),
                requestState: root.querySelector(
                    '[data-testid="server-entity-history-request-state"]'
                ).textContent.trim(),
                failureState: root.querySelector(
                    '[data-testid="server-entity-history-failure-state"]'
                ).textContent.trim(),
                loading: root.querySelector(
                    '[data-testid="server-entity-history-loading"]'
                ).textContent.trim() === 'true',
            };
        })()"#,
    )
    .await
}

async fn choose_history_exact_filter(harness: &pixelproof_web::Harness, column: &str, value: &str) {
    let selector = format!(
        "{HISTORY_ROOT} [data-table-filter-column='{column}'] select[data-table-filter-kind='exact']"
    );
    let option_index = eval_json(
        harness,
        &format!(
            r#"(() => {{
                const select = document.querySelector({selector:?});
                return Array.from(select.options).findIndex(option => option.value === {value:?});
            }})()"#
        ),
    )
    .await
    .as_i64()
    .expect("native exact-filter option index");
    assert!(
        option_index > 0,
        "fixture option must follow the All choice"
    );

    harness
        .page()
        .find_element(&selector)
        .await
        .expect("find native exact-filter control")
        .focus()
        .await
        .expect("focus native exact-filter control");
    let mut keys = vec![pixelproof_web::Key::Space, pixelproof_web::Key::Home];
    keys.extend(std::iter::repeat_n(
        pixelproof_web::Key::ArrowDown,
        option_index as usize,
    ));
    keys.push(pixelproof_web::Key::Enter);
    harness
        .press_key_sequence(&keys)
        .await
        .expect("choose native exact-filter option");
}

async fn type_history_text_filter(harness: &pixelproof_web::Harness, column: &str, value: &str) {
    let selector = format!(
        "{HISTORY_ROOT} [data-table-filter-column='{column}'] input[data-table-filter-kind='contains']"
    );
    let input = harness
        .page()
        .find_element(selector)
        .await
        .expect("find History text-filter control");
    input
        .focus()
        .await
        .expect("focus History text-filter control");
    input
        .type_str(value)
        .await
        .expect("type through the real History text-filter control");
}

async fn wait_for_history_pending(
    harness: &pixelproof_web::Harness,
    expected_token: u64,
    column: &str,
    value: &str,
) -> Value {
    for _ in 0..60 {
        let snapshot = history_snapshot(harness).await;
        if snapshot["proposals"] == json!(expected_token)
            && snapshot["requestState"] == json!(format!("pending:{expected_token}"))
            && snapshot["proposed"]["filters"][column] == json!(value)
        {
            return snapshot;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!(
        "History fixture never exposed pending token {expected_token}: {}",
        history_snapshot(harness).await
    );
}

async fn wait_for_history_pending_query(
    harness: &pixelproof_web::Harness,
    expected_token: u64,
    predicate: impl Fn(&Value) -> bool,
) -> Value {
    for _ in 0..60 {
        let snapshot = history_snapshot(harness).await;
        if snapshot["proposals"] == json!(expected_token)
            && snapshot["requestState"] == json!(format!("pending:{expected_token}"))
            && predicate(&snapshot["proposed"])
        {
            return snapshot;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!(
        "History fixture never exposed matching pending token {expected_token}: {}",
        history_snapshot(harness).await
    );
}

async fn wait_for_history_acceptance(
    harness: &pixelproof_web::Harness,
    expected_token: u64,
    expected_filter: Option<(&str, &str)>,
) -> Value {
    for _ in 0..60 {
        let snapshot = history_snapshot(harness).await;
        let query_matches = expected_filter.map_or_else(
            || snapshot["accepted"]["filters"] == json!({}),
            |(column, value)| snapshot["accepted"]["filters"][column] == json!(value),
        );
        if snapshot["proposals"] == json!(expected_token)
            && snapshot["requestState"] == json!(format!("accepted:{expected_token}"))
            && query_matches
        {
            return snapshot;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!(
        "History fixture never accepted token {expected_token}: {}",
        history_snapshot(harness).await
    );
}

async fn history_interaction_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#server-entity-history');
            const table = root.querySelector('[data-table-data-mode="server-query"]');
            const footer = table.querySelector('[data-server-table-footer]');
            const pageSize = footer.querySelector('[data-table-page-size-control]');
            const chooser = table.querySelector('[data-server-column-chooser="true"]');
            const chooserMenu = chooser.closest('.dropdown').querySelector('.dropdown-content');
            const moduleSeparator = Array.from(table.querySelectorAll('[role="separator"]'))
                .find(separator => separator.getAttribute('aria-label') === 'Resize Module column');
            const moduleTrack = table.querySelector('[data-table-column-track="module"]');
            const headers = Array.from(table.querySelectorAll(
                'colgroup[data-table-column-tracks] col[data-table-column-track]'
            )).map(track => track.dataset.tableColumnTrack);
            const auditedControls = Array.from(document.querySelectorAll(
                '[data-testid="server-entity-reactive-fixture"] input, ' +
                '[data-testid="server-entity-reactive-fixture"] select, ' +
                '#server-entity-history input, #server-entity-history select'
            ));
            const rect = element => {
                const box = element?.getBoundingClientRect();
                return box ? { left: box.left, top: box.top, right: box.right,
                    bottom: box.bottom, width: box.width, height: box.height } : null;
            };
            const splitShadowLayers = value => {
                const layers = [];
                let depth = 0;
                let start = 0;
                for (let index = 0; index < value.length; index += 1) {
                    if (value[index] === '(') depth += 1;
                    if (value[index] === ')') depth -= 1;
                    if (value[index] === ',' && depth === 0) {
                        layers.push(value.slice(start, index).trim());
                        start = index + 1;
                    }
                }
                layers.push(value.slice(start).trim());
                return layers;
            };
            const shadowLayerIsTransparentAndZero = layer => {
                const lengths = Array.from(layer.matchAll(/(-?\d*\.?\d+)px/g), match => Number(match[1]));
                const transparent = /\btransparent\b/i.test(layer) ||
                    /rgba?\([^)]*(?:,|\/)\s*0(?:\.0+)?\s*\)/i.test(layer) ||
                    /#[0-9a-f]{6}00\b/i.test(layer);
                return lengths.length >= 2 && lengths.every(length => length === 0) && transparent;
            };
            const shadowIsNormalized = value => value === 'none' ||
                splitShadowLayers(value).every(shadowLayerIsTransparentAndZero);
            const modeFilter = table.querySelector(
                '[data-table-filter-column="mode"] select[data-table-filter-kind="exact"]'
            );
            const startedSort = table.querySelector(
                '[data-table-sort-column="started_at"]'
            )?.closest('th')?.getAttribute('aria-sort') ?? null;
            return {
                headers,
                modeFilterValue: modeFilter?.value ?? null,
                startedSort,
                chooserExpanded: chooser.getAttribute('aria-expanded'),
                chooserVisible: rect(chooserMenu)?.width > 0 && rect(chooserMenu)?.height > 0,
                chooserClass: chooserMenu.className,
                chooserMaxHeight: getComputedStyle(chooserMenu).maxHeight,
                chooserOverflowY: getComputedStyle(chooserMenu).overflowY,
                chooserInsideViewport: !rect(chooserMenu) || (
                    rect(chooserMenu).left >= 0 &&
                    rect(chooserMenu).top >= 0 &&
                    rect(chooserMenu).right <= document.documentElement.clientWidth &&
                    rect(chooserMenu).bottom <= document.documentElement.clientHeight
                ),
                pageSize: {
                    value: pageSize.value,
                    label: pageSize.selectedOptions[0]?.textContent,
                    values: Array.from(pageSize.options).map(option => option.value),
                    id: pageSize.id,
                    name: pageSize.name,
                    ariaLabel: pageSize.getAttribute('aria-label'),
                },
                range: footer.querySelector('[data-server-row-range]').textContent.trim(),
                auditFixtureControls: {
                    count: auditedControls.length,
                    withUnexpectedShadow: auditedControls.filter(control => {
                        const shadow = getComputedStyle(control).boxShadow;
                        return !shadowIsNormalized(shadow);
                    }).length,
                    rejectsMixedShadow: !shadowIsNormalized(
                        'rgba(0, 0, 0, 0) 0px 0px 0px 0px, rgba(0, 0, 0, 0.5) 0px 4px 6px 0px'
                    ),
                },
                moduleResize: {
                    name: moduleSeparator?.getAttribute('aria-label'),
                    now: Number(moduleSeparator?.getAttribute('aria-valuenow')),
                    min: Number(moduleSeparator?.getAttribute('aria-valuemin')),
                    max: Number(moduleSeparator?.getAttribute('aria-valuemax')),
                    valueText: moduleSeparator?.getAttribute('aria-valuetext'),
                    trackWidth: rect(moduleTrack)?.width,
                },
                geometry: {
                    root: rect(root),
                    table: rect(table),
                    footer: rect(footer),
                    chooser: rect(chooserMenu),
                    horizontalOverflowAvailable: ['auto', 'scroll'].includes(
                        getComputedStyle(table.querySelector('.overflow-x-auto')).overflowX
                    ),
                    footerInside: rect(footer).left >= rect(table).left - 1 &&
                        rect(footer).top >= rect(table).top - 1 &&
                        rect(footer).right <= rect(table).right + 1 &&
                        rect(footer).bottom <= rect(table).bottom + 1,
                },
            };
        })()"#,
    )
    .await
}

fn expected_history_range(snapshot: &Value) -> String {
    let page = snapshot["accepted"]["page"]
        .as_u64()
        .expect("accepted page");
    let page_size = snapshot["accepted"]["page_size"]
        .as_u64()
        .expect("accepted page size");
    let row_count = snapshot["acceptedIds"]
        .as_array()
        .expect("accepted ids")
        .len() as u64;
    let total = snapshot["total"].as_u64().expect("accepted total");
    let start = if row_count == 0 {
        0
    } else {
        (page - 1) * page_size + 1
    };
    let end = if row_count == 0 {
        0
    } else {
        (start + row_count - 1).min(total)
    };
    format!("Showing {start}\u{2013}{end} of {total}")
}

async fn choose_history_page_size(harness: &pixelproof_web::Harness, value: &str) {
    let selector = format!("{HISTORY_ROOT} [data-table-page-size-control]");
    let option_index = eval_json(
        harness,
        &format!(
            r#"(() => {{
                const select = document.querySelector({selector:?});
                select.scrollIntoView({{ block: 'center' }});
                select.focus();
                return Array.from(select.options).findIndex(option => option.value === {value:?});
            }})()"#
        ),
    )
    .await
    .as_i64()
    .expect("History page-size option index");
    assert!(
        option_index >= 0,
        "History page-size option {value} is present"
    );
    harness
        .press_key_sequence(&[pixelproof_web::Key::Space])
        .await
        .expect("open the real History native page-size menu");
    let mut keys = vec![pixelproof_web::Key::Home];
    keys.extend(std::iter::repeat_n(
        pixelproof_web::Key::ArrowDown,
        option_index as usize,
    ));
    keys.push(pixelproof_web::Key::Enter);
    harness
        .press_key_sequence(&keys)
        .await
        .expect("confirm the real History native page-size choice");
}

async fn wait_for_history_query(
    harness: &pixelproof_web::Harness,
    expected_token: u64,
    predicate: impl Fn(&Value) -> bool,
) -> Value {
    for _ in 0..120 {
        let snapshot = history_snapshot(harness).await;
        if snapshot["proposals"] == json!(expected_token)
            && snapshot["requestState"] == json!(format!("accepted:{expected_token}"))
            && predicate(&snapshot["accepted"])
        {
            return snapshot;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!(
        "History fixture never accepted matching token {expected_token}: {}",
        history_snapshot(harness).await
    );
}

/// Dispatch the same real CDP element click as the shared helper without its
/// post-click visual settle delay, so the fixture's explicit pending token can
/// be observed before the 240 ms simulated response is acknowledged.
async fn click_history_immediate(harness: &pixelproof_web::Harness, selector: &str) {
    harness
        .page()
        .find_element(selector)
        .await
        .unwrap_or_else(|error| panic!("find immediate History click {selector}: {error}"))
        .click()
        .await
        .unwrap_or_else(|error| panic!("click immediate History control {selector}: {error}"));
}

/// `ldui-9ke9`: the canonical server facade exposes one complete standard
/// interaction surface. Every gesture is real browser input, and query
/// controls move only after the fixture acknowledges a full replacement.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn server_entity_history_standard_controls_preserve_accepted_truth() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, &format!("{HISTORY_ROOT} tbody tr[data-row-key]")).await;
    begin_browser_error_capture(&harness).await;

    let initial = history_snapshot(&harness).await;
    let initial_controls = history_interaction_snapshot(&harness).await;
    assert_eq!(initial_controls["pageSize"]["value"], json!("8"));
    assert!(
        initial_controls["pageSize"]["values"]
            .as_array()
            .is_some_and(|values| values.contains(&json!("auto"))),
        "the canonical fill-parent facade must expose Auto sizing: {initial_controls}"
    );
    assert_eq!(
        initial_controls["pageSize"]["id"],
        json!("server-entity-history-table-page-size")
    );
    assert_eq!(
        initial_controls["pageSize"]["name"],
        initial_controls["pageSize"]["id"]
    );
    assert_eq!(
        initial_controls["pageSize"]["ariaLabel"],
        json!("Rows per page")
    );
    assert_eq!(
        initial_controls["auditFixtureControls"]["count"],
        json!(14),
        "the two audit fixtures must expose the expected 14 native controls"
    );
    assert_eq!(
        initial_controls["auditFixtureControls"]["withUnexpectedShadow"],
        json!(0),
        "fixture normalization must add zero non-transparent control shadows"
    );
    assert_eq!(
        initial_controls["auditFixtureControls"]["rejectsMixedShadow"],
        json!(true),
        "the shadow oracle must reject a zero layer followed by a visible layer"
    );
    assert_eq!(initial["preferenceProposals"], json!(0));

    // Gear: hide, restore, then reorder through real pointer input. The DOM
    // projection and accepted preference payload must agree after each click.
    click(
        &harness,
        &format!("{HISTORY_ROOT} [data-server-column-chooser='true']"),
    )
    .await;
    let opened = history_interaction_snapshot(&harness).await;
    assert_eq!(opened["chooserExpanded"], json!("true"));
    assert_eq!(opened["chooserVisible"], json!(true));
    assert_eq!(opened["chooserMaxHeight"], json!("320px"));
    assert_eq!(opened["chooserOverflowY"], json!("auto"));
    assert_eq!(
        opened["chooserInsideViewport"],
        json!(true),
        "open chooser must fit the viewport on both axes: {opened}"
    );
    click(
        &harness,
        &format!("{HISTORY_ROOT} [data-server-column='build'] [role='menuitemcheckbox']"),
    )
    .await;
    let hidden = history_snapshot(&harness).await;
    let hidden_controls = history_interaction_snapshot(&harness).await;
    assert!(
        hidden["preferences"]["hidden_columns"]
            .as_array()
            .is_some_and(|columns| columns.contains(&json!("build")))
    );
    assert!(
        !hidden_controls["headers"]
            .as_array()
            .is_some_and(|columns| columns.contains(&json!("build")))
    );
    assert_eq!(hidden["preferenceProposals"], json!(1));
    click(
        &harness,
        &format!("{HISTORY_ROOT} [data-server-column='build'] [role='menuitemcheckbox']"),
    )
    .await;
    let restored = history_snapshot(&harness).await;
    let restored_controls = history_interaction_snapshot(&harness).await;
    assert_eq!(restored["preferenceProposals"], json!(2));
    assert!(
        !restored["preferences"]["hidden_columns"]
            .as_array()
            .is_some_and(|columns| columns.contains(&json!("build")))
    );
    let order_before_reorder = restored_controls["headers"]
        .as_array()
        .expect("restored header order")
        .clone();
    let build_before = order_before_reorder
        .iter()
        .position(|column| column == "build")
        .expect("restored Build header");
    assert!(
        build_before > 0,
        "Build must have an earlier slot available"
    );
    click(
        &harness,
        &format!(
            "{HISTORY_ROOT} [data-server-column-order='build'][data-server-column-move='earlier']"
        ),
    )
    .await;
    let reordered = history_snapshot(&harness).await;
    let reordered_controls = history_interaction_snapshot(&harness).await;
    let order_after_reorder = reordered_controls["headers"]
        .as_array()
        .expect("reordered header order");
    let build_after = order_after_reorder
        .iter()
        .position(|column| column == "build")
        .expect("reordered Build header");
    let mut expected_order = order_before_reorder.clone();
    expected_order.swap(build_before - 1, build_before);
    assert_eq!(
        order_after_reorder, &expected_order,
        "Build must move exactly one position earlier"
    );
    assert_eq!(build_after + 1, build_before);
    assert_ne!(order_after_reorder, &order_before_reorder);
    assert!(
        !reordered["preferences"]["hidden_columns"]
            .as_array()
            .is_some_and(|columns| columns.contains(&json!("build")))
    );
    assert_eq!(reordered["preferenceProposals"], json!(3));
    assert_eq!(
        reordered_controls["headers"], reordered["preferences"]["column_order"],
        "rendered header order must equal accepted preference order"
    );

    // Named separator: each real keyboard gesture commits through controlled
    // preference ownership and is reflected in both ARIA and track geometry.
    let separator = format!("{HISTORY_ROOT} [role='separator'][aria-label='Resize Module column']");
    harness
        .page()
        .find_element(&separator)
        .await
        .expect("find named History Module separator")
        .focus()
        .await
        .expect("focus named History Module separator");

    let focused_controls = history_interaction_snapshot(&harness).await;
    let mut previous_width = focused_controls["moduleResize"]["now"]
        .as_u64()
        .expect("focused Module width");
    let minimum_width = focused_controls["moduleResize"]["min"]
        .as_u64()
        .expect("Module minimum width");
    let maximum_width = focused_controls["moduleResize"]["max"]
        .as_u64()
        .expect("Module maximum width");
    let mut preference_token = reordered["preferenceProposals"]
        .as_u64()
        .expect("preference proposal token");
    for (key, expected_width) in [
        (
            pixelproof_web::Key::ArrowRight,
            (previous_width + 16).min(maximum_width),
        ),
        (pixelproof_web::Key::Home, minimum_width),
        (pixelproof_web::Key::End, maximum_width),
        (
            pixelproof_web::Key::ArrowLeft,
            maximum_width.saturating_sub(16).max(minimum_width),
        ),
    ] {
        harness
            .press_key_sequence(&[key])
            .await
            .expect("resize the History Module track by keyboard");
        let accepted = history_snapshot(&harness).await;
        let controls = history_interaction_snapshot(&harness).await;
        let width = accepted["preferences"]["column_widths"]["module"]
            .as_u64()
            .expect("accepted Module width");
        preference_token += 1;
        assert_eq!(width, expected_width, "keyboard resize outcome: {controls}");
        assert_ne!(width, previous_width, "resize key must not be a no-op");
        assert_eq!(accepted["preferenceProposals"], json!(preference_token));
        assert_eq!(controls["moduleResize"]["now"].as_u64(), Some(width));
        assert_eq!(
            controls["moduleResize"]["trackWidth"].as_f64(),
            Some(width as f64)
        );
        assert!(
            controls["moduleResize"]["min"].as_u64() <= Some(width)
                && Some(width) <= controls["moduleResize"]["max"].as_u64(),
            "named separator range must remain ordered: {controls}"
        );
        assert_eq!(
            controls["moduleResize"]["valueText"],
            json!(format!("{width} pixels"))
        );
        previous_width = width;
    }

    // The same controlled ownership path must preview and accept one real
    // pointer drag on the canonical History facade.
    let (end_x, y) = begin_real_pointer_drag(&harness, &separator, -96.0).await;
    let pointer_preview = history_interaction_snapshot(&harness).await;
    let pointer_pending_model = history_snapshot(&harness).await;
    assert_eq!(
        pointer_pending_model["preferenceProposals"],
        json!(preference_token)
    );
    assert_eq!(
        pointer_pending_model["preferences"]["column_widths"]["module"],
        json!(previous_width)
    );
    assert_eq!(
        pointer_preview["moduleResize"]["now"],
        json!(previous_width - 96)
    );
    finish_real_pointer_drag(&harness, end_x, y).await;
    let pointer_accepted = history_snapshot(&harness).await;
    let pointer_controls = history_interaction_snapshot(&harness).await;
    preference_token += 1;
    let pointer_width = previous_width - 96;
    assert_eq!(
        pointer_accepted["preferenceProposals"],
        json!(preference_token)
    );
    assert_eq!(
        pointer_accepted["preferences"]["column_widths"]["module"],
        json!(pointer_width)
    );
    assert_eq!(
        pointer_controls["moduleResize"]["now"],
        json!(pointer_width)
    );
    assert_eq!(
        pointer_controls["moduleResize"]["trackWidth"],
        json!(pointer_width)
    );

    // Fixed intent is accepted as a query replacement and survives an actual
    // desktop viewport resize without another proposal.
    let mut token = initial["proposals"].as_u64().expect("proposal token");
    choose_history_page_size(&harness, "10").await;
    token += 1;
    let fixed =
        wait_for_history_query(&harness, token, |query| query["page_size"] == json!(10)).await;
    let fixed_controls = history_interaction_snapshot(&harness).await;
    assert_eq!(fixed_controls["pageSize"]["value"], json!("10"));
    harness
        .set_viewport(pixelproof_web::ViewportSize::new(1280, 1_000))
        .await
        .expect("resize the History desktop viewport");
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    let fixed_after_resize = history_snapshot(&harness).await;
    let fixed_controls_after_resize = history_interaction_snapshot(&harness).await;
    assert_eq!(fixed_after_resize["proposals"], fixed["proposals"]);
    assert_eq!(fixed_after_resize["accepted"], fixed["accepted"]);
    assert_eq!(
        fixed_controls_after_resize["pageSize"]["value"],
        json!("10")
    );
    assert_eq!(
        fixed_controls_after_resize["pageSize"]["label"],
        json!("10")
    );

    // Auto is chosen through the native menu, then a second desktop resize
    // must propose and accept a different server capacity.
    choose_history_page_size(&harness, "auto").await;
    token += 1;
    let auto =
        wait_for_history_query(&harness, token, |query| query["page_size"] != json!(10)).await;
    let first_auto_size = auto["accepted"]["page_size"]
        .as_i64()
        .expect("first accepted Auto capacity");
    let auto_controls = history_interaction_snapshot(&harness).await;
    assert_eq!(auto_controls["pageSize"]["value"], json!("auto"));
    assert_eq!(
        auto_controls["pageSize"]["label"],
        json!(format!("Auto ({first_auto_size})"))
    );
    harness
        .set_viewport(pixelproof_web::ViewportSize::new(1280, 700))
        .await
        .expect("resize the Auto-sized History desktop viewport");
    token += 1;
    let resized_auto = wait_for_history_query(&harness, token, |query| {
        query["page_size"].as_i64() != Some(first_auto_size)
    })
    .await;
    assert_ne!(
        resized_auto["accepted"]["page_size"],
        auto["accepted"]["page_size"]
    );
    let resized_auto_controls = history_interaction_snapshot(&harness).await;
    assert_eq!(resized_auto_controls["pageSize"]["value"], json!("auto"));
    assert_eq!(
        resized_auto_controls["pageSize"]["label"],
        json!(format!(
            "Auto ({})",
            resized_auto["accepted"]["page_size"]
                .as_u64()
                .expect("resized Auto capacity")
        ))
    );

    // Sort then move to page two. Both visible changes await accepted server
    // replacements; no client-local row transform is allowed.
    let before_sort = history_snapshot(&harness).await;
    let controls_before_sort = history_interaction_snapshot(&harness).await;
    click_history_immediate(
        &harness,
        &format!("{HISTORY_ROOT} [data-table-sort-column='started_at']"),
    )
    .await;
    token += 1;
    let pending_sort = wait_for_history_pending_query(&harness, token, |query| {
        query["sort"]["column"] == json!("started_at")
            && query["sort"]["order"] == json!("ascending")
    })
    .await;
    let controls_pending_sort = history_interaction_snapshot(&harness).await;
    assert_eq!(pending_sort["loading"], json!(true));
    assert_eq!(controls_pending_sort["range"], json!("Loading..."));
    for field in ["accepted", "acceptedIds", "domIds", "total"] {
        assert_eq!(
            pending_sort[field], before_sort[field],
            "pending sort changed {field}"
        );
    }
    for field in ["pageSize", "headers", "modeFilterValue", "startedSort"] {
        assert_eq!(
            controls_pending_sort[field], controls_before_sort[field],
            "pending sort changed visible {field}"
        );
    }
    let sorted = wait_for_history_query(&harness, token, |query| {
        query["sort"]["column"] == json!("started_at")
            && query["sort"]["order"] == json!("ascending")
    })
    .await;
    assert_eq!(sorted["acceptedIds"], sorted["domIds"]);
    assert_eq!(
        history_interaction_snapshot(&harness).await["startedSort"],
        json!("ascending")
    );
    let before_page = history_snapshot(&harness).await;
    let controls_before_page = history_interaction_snapshot(&harness).await;
    click_history_immediate(
        &harness,
        &format!("{HISTORY_ROOT} [data-server-table-footer] button:nth-last-child(1)"),
    )
    .await;
    token += 1;
    let pending_page =
        wait_for_history_pending_query(&harness, token, |query| query["page"] == json!(2)).await;
    let controls_pending_page = history_interaction_snapshot(&harness).await;
    assert_eq!(pending_page["loading"], json!(true));
    assert_eq!(controls_pending_page["range"], json!("Loading..."));
    for field in ["accepted", "acceptedIds", "domIds", "total"] {
        assert_eq!(
            pending_page[field], before_page[field],
            "pending page changed {field}"
        );
    }
    for field in ["pageSize", "headers", "modeFilterValue", "startedSort"] {
        assert_eq!(
            controls_pending_page[field], controls_before_page[field],
            "pending page changed visible {field}"
        );
    }
    let page_two = wait_for_history_query(&harness, token, |query| query["page"] == json!(2)).await;
    assert_eq!(page_two["acceptedIds"], page_two["domIds"]);
    assert_eq!(
        history_interaction_snapshot(&harness).await["range"],
        json!(expected_history_range(&page_two))
    );

    // Fail the next filter request. Accepted query, rows, population count,
    // range, page-size label, order, widths, and preferences all remain the
    // exact pre-request truth while the proposal/failure state is observable.
    click(
        &harness,
        "[data-testid='server-entity-history-fail-next-request']",
    )
    .await;
    let before_failure = history_snapshot(&harness).await;
    let controls_before_failure = history_interaction_snapshot(&harness).await;

    choose_history_exact_filter(&harness, "mode", "Replay").await;
    token += 1;
    let pending_failure = wait_for_history_pending(&harness, token, "mode", "Replay").await;
    let controls_pending_failure = history_interaction_snapshot(&harness).await;
    assert_eq!(pending_failure["loading"], json!(true));
    assert_eq!(controls_pending_failure["range"], json!("Loading..."));
    for field in ["accepted", "acceptedIds", "domIds", "total", "preferences"] {
        assert_eq!(
            pending_failure[field], before_failure[field],
            "pending failed request changed accepted {field}"
        );
    }
    for field in [
        "pageSize",
        "headers",
        "moduleResize",
        "modeFilterValue",
        "startedSort",
        "chooserExpanded",
        "chooserVisible",
    ] {
        assert_eq!(
            controls_pending_failure[field], controls_before_failure[field],
            "pending failed request changed visible {field}"
        );
    }
    for _ in 0..120 {
        let failed = history_snapshot(&harness).await;
        if failed["requestState"] == json!(format!("failed:{token}")) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let failed = history_snapshot(&harness).await;
    let controls_after_failure = history_interaction_snapshot(&harness).await;
    assert_eq!(failed["requestState"], json!(format!("failed:{token}")));
    assert_eq!(failed["failureState"], json!("retained-failure"));
    assert_eq!(failed["loading"], json!(false));
    for field in ["accepted", "acceptedIds", "domIds", "total", "preferences"] {
        assert_eq!(
            failed[field], before_failure[field],
            "failed {field}: {failed}"
        );
    }
    for field in [
        "range",
        "pageSize",
        "headers",
        "moduleResize",
        "modeFilterValue",
        "startedSort",
        "chooserExpanded",
        "chooserVisible",
    ] {
        assert_eq!(
            controls_after_failure[field], controls_before_failure[field],
            "failed request changed accepted {field}"
        );
    }
    assert_eq!(
        controls_after_failure["geometry"]["footerInside"],
        json!(true)
    );
    assert_eq!(
        controls_after_failure["geometry"]["horizontalOverflowAvailable"],
        json!(true),
        "wide History columns must remain available through horizontal scrolling when needed"
    );

    // Layer C: inject the vendored axe runtime, then scope blocking WCAG
    // findings to this deterministic fixture rather than unrelated examples.
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let _page_report = axe
        .run(harness.page())
        .await
        .expect("inject and run axe-core");
    let blocking = eval_json(
        &harness,
        r#"(async () => {
            const report = await axe.run(document.querySelector('#server-entity-history'), {
                runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21aa'] }
            });
            return report.violations
                .filter(v => v.impact === 'serious' || v.impact === 'critical')
                .map(v => ({ id: v.id, impact: v.impact, nodes: v.nodes.length }));
        })()"#,
    )
    .await;
    assert_eq!(
        blocking,
        json!([]),
        "blocking History axe findings: {blocking}"
    );
    assert_no_browser_errors(&harness, "ServerEntityTable complete interaction contract").await;
}

/// `ldui-9ke9`: every History filter is evaluated against the complete
/// simulated population. The accepted page and DOM remain locked while a
/// proposal is pending, then move together to a page containing an id that
/// was not present in the original eight-row slice.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn server_entity_history_filters_use_accepted_population_truth() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, &format!("{HISTORY_ROOT} tbody tr[data-row-key]")).await;
    begin_browser_error_capture(&harness).await;

    let initial = history_snapshot(&harness).await;
    assert_eq!(
        initial["acceptedIds"], initial["domIds"],
        "initial: {initial}"
    );
    assert_eq!(initial["acceptedIds"].as_array().map(Vec::len), Some(8));
    assert_eq!(initial["total"], json!(48));
    assert_eq!(initial["proposals"], json!(0));
    assert_eq!(initial["requestState"], json!("accepted:0"));
    assert_eq!(initial["failureState"], json!("none"));
    assert_eq!(initial["loading"], json!(false));
    assert_eq!(initial["preferences"]["schema_version"], json!(1));
    let original_ids = initial["acceptedIds"]
        .as_array()
        .expect("initial accepted ids")
        .clone();

    let cases = [
        ("run", "run-037", true),
        ("module", "matter_timeline_history", true),
        ("mode", "Replay", false),
        ("started_at", "2026-09-07T03:37", true),
        ("duration", "901", true),
        ("verdict", "Retried", false),
        ("captured", "370037", true),
        ("rejected", "137", true),
        ("trigger", "Backfill", false),
        ("build", "history-proof-37", false),
    ];

    let mut previous_proposals = 0_u64;
    for (column, value, text_filter) in cases {
        click(
            &harness,
            "[data-testid='server-entity-history-reset-query']",
        )
        .await;
        previous_proposals += 1;
        let reset = wait_for_history_acceptance(&harness, previous_proposals, None).await;
        assert_eq!(reset["proposals"], json!(previous_proposals));
        assert_eq!(reset["accepted"]["filters"], json!({}));
        assert_eq!(reset["acceptedIds"], initial["acceptedIds"]);
        assert_eq!(reset["loading"], json!(false));

        if text_filter {
            type_history_text_filter(&harness, column, value).await;
        } else {
            choose_history_exact_filter(&harness, column, value).await;
        }

        previous_proposals += 1;
        let pending = wait_for_history_pending(&harness, previous_proposals, column, value).await;
        assert_eq!(
            pending["proposals"],
            json!(previous_proposals),
            "{column}: {pending}"
        );
        assert_eq!(pending["proposed"]["filters"][column], json!(value));
        assert_eq!(pending["accepted"]["filters"], json!({}));
        assert_eq!(pending["acceptedIds"], initial["acceptedIds"]);
        assert_eq!(pending["domIds"], initial["acceptedIds"]);
        assert_eq!(pending["total"], json!(48));
        assert_eq!(pending["loading"], json!(true));
        assert!(
            pending["requestState"]
                .as_str()
                .is_some_and(|state| state.starts_with("pending:")),
            "{column}: {pending}"
        );

        let accepted =
            wait_for_history_acceptance(&harness, previous_proposals, Some((column, value))).await;
        assert_eq!(accepted["accepted"]["filters"][column], json!(value));
        assert_eq!(accepted["acceptedIds"], accepted["domIds"]);
        assert_eq!(accepted["loading"], json!(false));
        assert!(
            accepted["acceptedIds"]
                .as_array()
                .expect("filtered accepted ids")
                .iter()
                .any(|id| !original_ids.contains(id)),
            "{column} must find a complete-population row outside page one: {accepted}"
        );
        assert_eq!(accepted["failureState"], json!("none"));
    }

    assert_no_browser_errors(&harness, "ServerEntityTable History population truth").await;
}

/// Reactive column declarations must switch the facade between its standard
/// table and its fail-visible configuration alert in both directions.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn server_entity_table_column_validation_tracks_reactive_columns() {
    let h = harness_at("/components/data-table").await;
    wait_for_selector(
        &h,
        "[data-testid='server-entity-reactive-fixture'] [data-server-entity-table='true']",
    )
    .await;
    begin_browser_error_capture(&h).await;

    click(
        &h,
        "[data-testid='server-entity-reactive-fixture'] [data-server-column-chooser='true']",
    )
    .await;
    click(
        &h,
        "[data-testid='server-entity-reactive-fixture'] [data-server-column='verdict'] [role='menuitemcheckbox']",
    )
    .await;
    let hidden_before_invalid = eval_json(
        &h,
        r#"!!document.querySelector('[data-testid="server-entity-reactive-fixture"] [data-table-sort-column="verdict"]')"#,
    )
    .await;
    assert_eq!(
        hidden_before_invalid,
        json!(false),
        "uncontrolled preference must hide Verdict before the configuration swap"
    );

    click(&h, "[data-testid='server-entity-columns-invalid']").await;
    let invalid = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="server-entity-reactive-fixture"]');
            const alert = fixture.querySelector('[role="alert"][data-server-entity-table-config-error]');
            return {
                alert: alert?.dataset.serverEntityTableConfigError,
                table: !!fixture.querySelector('[data-server-entity-table="true"]'),
            };
        })()"#,
    )
    .await;
    assert_eq!(invalid["alert"], json!("name"), "invalid swap: {invalid}");
    assert_eq!(invalid["table"], json!(false), "invalid swap: {invalid}");

    click(&h, "[data-testid='server-entity-columns-valid']").await;
    let recovered = eval_json(
        &h,
        r#"(() => {
            const fixture = document.querySelector('[data-testid="server-entity-reactive-fixture"]');
            return {
                alert: !!fixture.querySelector('[data-server-entity-table-config-error]'),
                facade: !!fixture.querySelector('[data-server-entity-table="true"]'),
                server: !!fixture.querySelector('[data-table-data-mode="server-query"]'),
                verdict: !!fixture.querySelector('[data-table-sort-column="verdict"]'),
            };
        })()"#,
    )
    .await;
    assert_eq!(recovered["alert"], json!(false), "recovery: {recovered}");
    assert_eq!(recovered["facade"], json!(true), "recovery: {recovered}");
    assert_eq!(recovered["server"], json!(true), "recovery: {recovered}");
    assert_eq!(
        recovered["verdict"],
        json!(false),
        "uncontrolled hidden-column preference must survive recovery: {recovered}"
    );
    assert_no_browser_errors(&h, "reactive ServerEntityTable validation").await;
}

/// The opinionated server footer keeps the named size control below the body.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn server_page_size_control_is_in_a_stable_footer() {
    let h = harness_at("/components/data-table").await;
    wait_for_selector(&h, "#viewport-fit-offset-server-table tbody tr").await;
    let state = eval_json(&h, r#"(() => {
        const root = document.querySelector('#viewport-fit-offset-server-table');
        const select = root.querySelector('[data-table-page-size-control]');
        const table = root.querySelector('table');
        return { below: select.getBoundingClientRect().top >= table.getBoundingClientRect().bottom,
            footer: !!select.closest('[data-server-table-footer]'),
            named: !!select.id && select.name === select.id && !!select.getAttribute('aria-label') };
    })()"#).await;
    assert_eq!(
        state["footer"],
        json!(true),
        "page-size selector belongs in the footer: {state}"
    );
    assert_eq!(
        state["below"],
        json!(true),
        "page-size selector follows the body: {state}"
    );
    assert_eq!(
        state["named"],
        json!(true),
        "stable accessible identity: {state}"
    );
}

const FIT_ROOT: &str = "#viewport-fit-offset-server-table";

async fn size_state(h: &pixelproof_web::Harness) -> Value {
    eval_json(h, r#"(() => {
        const root = document.querySelector('#viewport-fit-offset-server-table');
        const select = root.querySelector('[data-table-page-size-control]');
        const wrapper = root.querySelector(':scope > .overflow-x-auto');
        const footer = root.querySelector('[data-server-table-footer]');
        return { value: select.value, label: select.selectedOptions[0]?.textContent,
            id: select.id, name: select.name, mode: root.dataset.serverPageSizeIntent,
            size: Number(root.dataset.serverAcceptedPageSize),
            rows: root.querySelectorAll('tbody tr[data-table-row]').length || root.querySelectorAll('tbody tr').length,
            height: root.getBoundingClientRect().height, viewport: wrapper.clientHeight,
            range: footer?.textContent,
            query: document.querySelector('[data-testid="viewport-fit-last-query"]').textContent,
            proposals: Number(document.querySelector('[data-testid="viewport-fit-proposals"]').textContent),
            preference: document.querySelector('[data-testid="viewport-fit-preference"]')?.textContent,
            navigationDisabled: !!footer && [...footer.querySelectorAll('button')].every(b => b.disabled),
            footerInside: !!footer && footer.scrollWidth <= footer.clientWidth + 1 &&
                footer.getBoundingClientRect().bottom <= root.getBoundingClientRect().bottom + 1 &&
                [...footer.querySelectorAll('select,button,[data-server-row-range]')].every(el => {
                    const r = el.getBoundingClientRect(), f = footer.getBoundingClientRect();
                    return r.left >= f.left - 1 && r.right <= f.right + 1 &&
                        r.top >= f.top - 1 && r.bottom <= f.bottom + 1;
                }) };
    })()"#).await
}

async fn settle_size(h: &pixelproof_web::Harness) -> Value {
    // Bounded quiet observation catches refetch/measurement loops as well as
    // allowing the intentionally delayed host acknowledgment to finish.
    tokio::time::sleep(std::time::Duration::from_millis(750)).await;
    size_state(h).await
}

async fn choose_size(h: &pixelproof_web::Harness, value: &str, pointer_open: bool) {
    let selector = format!("{FIT_ROOT} [data-table-page-size-control]");
    let index = eval_json(
        h,
        &format!(
            r#"(() => {{
        const select = document.querySelector({selector:?});
        select.scrollIntoView({{block:'center'}}); select.focus();
        return [...select.options].findIndex(o => o.value === {value:?});
    }})()"#
        ),
    )
    .await
    .as_i64()
    .expect("option index");
    assert!(index >= 0, "missing page-size option {value}");
    if pointer_open {
        click(h, &selector).await;
    } else {
        h.press_key_sequence(&[pixelproof_web::Key::Space])
            .await
            .unwrap();
    }
    let mut keys = vec![pixelproof_web::Key::Home];
    keys.extend((0..index).map(|_| pixelproof_web::Key::ArrowDown));
    keys.push(pixelproof_web::Key::Enter);
    h.press_key_sequence(&keys)
        .await
        .expect("commit native size selection");
}

async fn resize_size_slot(h: &pixelproof_web::Harness, height: u32) {
    eval_json(
        h,
        &format!(
            r#"(() => {{
        document.querySelector('{FIT_ROOT}').parentElement.style.height = '{height}px';
        return true;
    }})()"#
        ),
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn server_manual_page_size_survives_resize_until_auto_is_selected() {
    let h = harness_at("/components/data-table").await;
    wait_for_selector(&h, "#viewport-fit-offset-server-table tbody tr").await;
    begin_browser_error_capture(&h).await;
    settle_size(&h).await;
    choose_size(&h, "10", true).await;
    let fixed = settle_size(&h).await;
    assert_eq!(
        fixed["value"],
        json!("10"),
        "a manual size must stay selected: {fixed}"
    );
    resize_size_slot(&h, 900).await;
    let tall = settle_size(&h).await;
    assert_eq!(
        tall["value"],
        json!("10"),
        "resize cannot overwrite fixed intent: {tall}"
    );
    assert_eq!(
        tall["rows"],
        json!(10),
        "server must retain ten supplied rows: {tall}"
    );
    assert_eq!(
        tall["proposals"], fixed["proposals"],
        "resize must not request another fixed page: {tall}"
    );
    click(&h, "[data-testid='viewport-fit-filter-one']").await;
    assert_eq!(settle_size(&h).await["rows"], json!(1));
    click(&h, "[data-testid='viewport-fit-filter-one']").await;
    let refetched = settle_size(&h).await;
    assert_eq!(refetched["value"], json!("10"));
    assert_eq!(
        refetched["rows"],
        json!(10),
        "fixed size survives refetch: {refetched}"
    );
    choose_size(&h, "auto", false).await;
    let auto = settle_size(&h).await;
    assert_eq!(auto["value"], json!("auto"), "Auto resumes fitting: {auto}");
    assert_ne!(
        auto["rows"],
        json!(10),
        "Auto must measure the actual slot: {auto}"
    );
    assert_eq!(
        auto["id"], fixed["id"],
        "control identity survives intent changes"
    );
    assert_eq!(auto["name"], auto["id"]);
    let quiet = settle_size(&h).await;
    assert_eq!(
        quiet["proposals"], auto["proposals"],
        "measurement/refetch converges: {quiet}"
    );
    assert_no_browser_errors(&h, "server page-size intent").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn server_size_delayed_and_rejected_proposals_keep_accepted_truth() {
    let h = harness_at("/components/data-table").await;
    wait_for_selector(&h, "#viewport-fit-offset-server-table tbody tr").await;
    begin_browser_error_capture(&h).await;
    let initial = settle_size(&h).await;
    click(&h, "[data-testid='viewport-fit-delay']").await;
    choose_size(&h, "25", false).await;
    let pending = settle_size(&h).await;
    assert_eq!(
        pending["size"], initial["size"],
        "a request is not accepted data: {pending}"
    );
    assert_eq!(pending["rows"], initial["rows"]);
    assert_eq!(
        pending["value"].as_str().unwrap(),
        initial["size"].to_string()
    );
    assert!(pending["preference"].as_str().unwrap().contains("Fixed:25"));
    let quiet = settle_size(&h).await;
    assert_eq!(
        quiet["proposals"], pending["proposals"],
        "pending request is not retried: {quiet}"
    );
    click(&h, "[data-testid='viewport-fit-apply']").await;
    let applied = settle_size(&h).await;
    assert_eq!(applied["size"], json!(25));
    assert_eq!(applied["value"], json!("25"));
    assert_eq!(applied["rows"], json!(25));
    click(&h, "[data-testid='viewport-fit-delay']").await;
    click(&h, "[data-testid='viewport-fit-accept']").await;
    choose_size(&h, "10", false).await;
    let declined = settle_size(&h).await;
    assert_eq!(
        declined["value"],
        json!("25"),
        "declined native selection is restored: {declined}"
    );
    assert_eq!(declined["size"], json!(25));
    assert_eq!(declined["rows"], json!(25));
    click(&h, "[data-testid='viewport-fit-callback-probe']").await;
    let host_changed = settle_size(&h).await;
    assert_eq!(
        host_changed["proposals"], declined["proposals"],
        "host callback reads must not subscribe the proposal effect: {host_changed}"
    );
    choose_size(&h, "auto", false).await;
    resize_size_slot(&h, 900).await;
    let auto_declined = settle_size(&h).await;
    assert_eq!(auto_declined["value"], json!("auto"));
    assert_eq!(
        auto_declined["label"],
        json!("Auto (25)"),
        "Auto labels accepted capacity: {auto_declined}"
    );
    for height in [899, 900, 899, 900] {
        resize_size_slot(&h, height).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let settled = settle_size(&h).await;
    assert_eq!(
        settled["proposals"], auto_declined["proposals"],
        "same derived capacity cannot refetch-loop: {settled}"
    );
    click(&h, "[data-testid='viewport-fit-apply']").await;
    let intermediate = settle_size(&h).await;
    assert_ne!(
        intermediate["size"],
        json!(25),
        "host accepted measured capacity: {intermediate}"
    );
    click(&h, "[data-testid='viewport-fit-supply-25']").await;
    let returned = settle_size(&h).await;
    assert!(
        returned["proposals"].as_u64() > intermediate["proposals"].as_u64(),
        "returning to an earlier accepted query must permit a fresh proposal: {returned}"
    );
    assert_no_browser_errors(&h, "server accepted page-size truth").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn server_footer_survives_empty_loading_filter_restore_and_compact_layout() {
    let h = harness_at("/components/data-table").await;
    wait_for_selector(&h, "#viewport-fit-offset-server-table tbody tr").await;
    begin_browser_error_capture(&h).await;
    let initial = settle_size(&h).await;
    click(&h, "[data-testid='viewport-fit-filter-one']").await;
    let one = settle_size(&h).await;
    assert_eq!(one["rows"], json!(1));
    assert_eq!(
        one["height"], initial["height"],
        "one row cannot collapse the slot: {one}"
    );
    click(&h, "[data-testid='viewport-fit-filter-one']").await;
    let restored = settle_size(&h).await;
    assert_eq!(restored["height"], initial["height"]);
    assert_eq!(
        restored["size"], initial["size"],
        "Auto capacity recovers: {restored}"
    );
    click(&h, "[data-testid='viewport-fit-empty']").await;
    let empty = settle_size(&h).await;
    assert_eq!(empty["id"], initial["id"]);
    assert_eq!(empty["height"], initial["height"]);
    assert!(
        empty["range"]
            .as_str()
            .unwrap()
            .contains("Showing 0–0 of 0"),
        "truthful empty range: {empty}"
    );
    click(&h, "[data-testid='viewport-fit-loading']").await;
    let loading = settle_size(&h).await;
    assert_eq!(loading["id"], initial["id"]);
    assert_eq!(loading["height"], initial["height"]);
    assert!(loading["range"].as_str().unwrap().contains("Loading"));
    assert_eq!(
        loading["navigationDisabled"],
        json!(true),
        "loading cannot navigate stale pages: {loading}"
    );
    click(&h, "[data-testid='viewport-fit-loading']").await;
    click(&h, "[data-testid='viewport-fit-empty']").await;
    click(&h, "[data-testid='viewport-fit-ready']").await;
    let paused = settle_size(&h).await;
    resize_size_slot(&h, 600).await;
    let resized = settle_size(&h).await;
    assert_eq!(resized["proposals"], paused["proposals"]);
    assert_eq!(resized["value"], json!("auto"));
    click(&h, "[data-testid='viewport-fit-ready']").await;
    settle_size(&h).await;
    click(&h, "[data-testid='viewport-fit-localize']").await;
    let localized = settle_size(&h).await;
    assert!(
        localized["label"]
            .as_str()
            .unwrap()
            .starts_with("Ajustar (")
    );
    for width in [1280, 375] {
        h.set_viewport(pixelproof_web::ViewportSize::new(width, 900))
            .await
            .unwrap();
        let state = settle_size(&h).await;
        assert_eq!(
            state["footerInside"],
            json!(true),
            "footer must wrap at {width}px: {state}"
        );
        assert_eq!(state["id"], initial["id"]);
        eval_json(
            &h,
            &format!(
                "document.querySelector('{FIT_ROOT}').scrollIntoView({{block:'center'}}); true"
            ),
        )
        .await;
        let png = h.screenshot_bytes().await.expect("capture server footer");
        std::fs::create_dir_all(".review/gfk7").unwrap();
        std::fs::write(format!(".review/gfk7/footer-{width}.png"), png).unwrap();
    }
    assert_no_browser_errors(&h, "server footer lifecycle and geometry").await;
}

async fn snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#server-table');
            const chooser = root.querySelector('[data-server-column-chooser="true"]');
            const dropdown = chooser ? chooser.closest('.dropdown') : null;
            const menu = dropdown ? dropdown.querySelector('.dropdown-content') : null;
            const menuBox = menu ? menu.getBoundingClientRect() : null;
            const headerOrder = Array.from(
                root.querySelectorAll('thead tr:first-child th [data-table-sort-column]')
            ).map(btn => btn.dataset.tableSortColumn);
            return {
                chooserPresentation: chooser ? chooser.dataset.serverColumnChooserPresentation : null,
                chooserExpanded: chooser ? chooser.getAttribute('aria-expanded') : null,
                chooserOpenAttr: dropdown ? (dropdown.dataset.serverColumnChooserOpen ?? null) : null,
                menuVisible: !!menuBox && menuBox.width > 0 && menuBox.height > 0,
                withinViewport: !menuBox || (
                    menuBox.left >= 0 && menuBox.right <= document.documentElement.clientWidth
                ),
                focusedIsChooser: document.activeElement === chooser,
                bodyRows: root.querySelectorAll('tbody tr').length,
                hasEmailHeader: !!root.querySelector('thead [data-table-sort-column="email"]'),
                hasNameHeader: !!root.querySelector('thead [data-table-sort-column="name"]'),
                nameToggleExists: !!root.querySelector('[data-server-column="name"]'),
                emailToggleExists: !!root.querySelector('[data-server-column="email"]'),
                headerOrder,
            };
        })()"#,
    )
    .await
}

async fn text_of(harness: &pixelproof_web::Harness, selector: &str) -> String {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const el = document.querySelector({selector:?});
                return el ? el.textContent.trim() : null;
            }})()"#
        ),
    )
    .await
    .as_str()
    .unwrap_or_default()
    .to_owned()
}

/// `ldui-9j16` binding acceptance, end to end: the chooser opens inside the
/// viewport, a required column is unhideable and absent from the chooser
/// list entirely, hide/reorder/reset all reach the rendered DOM, `Escape`
/// closes with focus restored, the toolbar Export action sits beside the
/// chooser, and the displayed-slice projection tracks only the current page.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn column_tools_chooser_projection_and_required_column_contract() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, "#server-table tbody tr").await;
    begin_browser_error_capture(&harness).await;

    // Closed by default; the icon presentation was requested explicitly.
    let initial = snapshot(&harness).await;
    assert_eq!(initial["chooserPresentation"], json!("icon"));
    assert_eq!(initial["chooserExpanded"], json!("false"));
    assert_eq!(initial["chooserOpenAttr"], Value::Null);
    assert_eq!(initial["menuVisible"], json!(false));
    assert_eq!(initial["hasNameHeader"], json!(true));
    assert_eq!(initial["hasEmailHeader"], json!(true));
    let page_rows = initial["bodyRows"].as_u64().expect("initial page has rows");
    assert!(
        page_rows > 0 && page_rows <= 10,
        "server table is paged at 10 rows: {initial}"
    );

    // Open: stays inside the viewport, and the required "name" column is
    // not merely undisturbed -- it is never offered as a toggle at all,
    // matching EntityTable's own `.filter(|column| !column.required)`.
    click(
        &harness,
        "#server-table [data-server-column-chooser=\"true\"]",
    )
    .await;
    let opened = snapshot(&harness).await;
    assert_eq!(opened["chooserExpanded"], json!("true"));
    assert_eq!(opened["chooserOpenAttr"], json!("true"));
    assert_eq!(opened["menuVisible"], json!(true));
    assert_eq!(
        opened["withinViewport"],
        json!(true),
        "chooser menu must stay inside the viewport: {opened}"
    );
    assert_eq!(
        opened["nameToggleExists"],
        json!(false),
        "a required column must not be offered in the chooser list: {opened}"
    );
    assert_eq!(opened["emailToggleExists"], json!(true));

    // Hide the optional "email" column: it leaves the rendered header, the
    // required "name" column does not.
    click(
        &harness,
        "#server-table [data-server-column=\"email\"] [role=\"menuitemcheckbox\"]",
    )
    .await;
    let hidden = snapshot(&harness).await;
    assert_eq!(hidden["hasEmailHeader"], json!(false));
    assert_eq!(hidden["hasNameHeader"], json!(true));
    assert_eq!(
        hidden["bodyRows"], initial["bodyRows"],
        "hiding a column must not change the row count: {hidden}"
    );

    // Reorder: move "status" one step earlier and confirm the rendered
    // header order actually changed (not just the preference value).
    let before_order = hidden["headerOrder"].clone();
    click(
        &harness,
        "#server-table [data-server-column-order=\"status\"][data-server-column-move=\"earlier\"]",
    )
    .await;
    let reordered = snapshot(&harness).await;
    assert_ne!(
        reordered["headerOrder"], before_order,
        "moving a column earlier must change the rendered header order"
    );

    // Escape closes the menu and returns focus to the trigger -- same
    // contract as EntityTable's own chooser (ldui-vn81).
    harness
        .press_key_sequence(&[pixelproof_web::Key::Escape])
        .await
        .expect("dismiss the server-table chooser with Escape");
    let after_escape = snapshot(&harness).await;
    assert_eq!(after_escape["chooserExpanded"], json!("false"));
    assert_eq!(after_escape["chooserOpenAttr"], Value::Null);
    assert_eq!(after_escape["menuVisible"], json!(false));
    assert_eq!(
        after_escape["focusedIsChooser"],
        json!(true),
        "Escape must return focus to the chooser trigger: {after_escape}"
    );
    // Hiding/reordering persisted through the close.
    assert_eq!(after_escape["hasEmailHeader"], json!(false));

    // Reopen and reset: visibility and order both return to their declared
    // defaults.
    click(
        &harness,
        "#server-table [data-server-column-chooser=\"true\"]",
    )
    .await;
    click(
        &harness,
        "#server-table [data-server-column-reset=\"true\"]",
    )
    .await;
    let after_reset = snapshot(&harness).await;
    assert_eq!(
        after_reset["hasEmailHeader"],
        json!(true),
        "reset must restore a previously hidden column: {after_reset}"
    );
    assert_eq!(
        after_reset["headerOrder"], initial["headerOrder"],
        "reset must restore declared column order: {after_reset}"
    );

    // The central contract: the toolbar Export action sits beside the
    // chooser, and the atomic displayed-slice projection it reads carries
    // ONLY the current server page -- never the fixture's full 57-row
    // population. This is the assertion that makes the one-page-CSV
    // mistake impossible to ship unnoticed.
    click(&harness, "[data-testid=\"server-export-slice\"]").await;
    let export_count = text_of(&harness, "[data-testid=\"server-export-count\"]").await;
    assert_eq!(export_count, "1");
    let slice_rows: u64 = text_of(&harness, "[data-testid=\"server-displayed-slice-rows\"]")
        .await
        .parse()
        .expect("displayed-slice row count is numeric");
    let rendered_rows = after_reset["bodyRows"].as_u64().expect("rendered rows");
    assert_eq!(
        slice_rows, rendered_rows,
        "the displayed-slice projection must carry exactly the rendered page, not more"
    );
    assert!(
        slice_rows < 57,
        "the displayed slice must never grow to the full simulated population (57 rows): got {slice_rows}"
    );

    assert_no_browser_errors(&harness, "server-table column-tools chooser/projection").await;
}

async fn server_name_width_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#server-table');
            const separator = Array.from(root.querySelectorAll('[role="separator"]'))
                .find(el => el.getAttribute('aria-label') === 'Resize Name column');
            const track = root.querySelector('[data-table-column-track="name"]');
            const accepted = document.querySelector('[data-testid="server-column-widths"]');
            const proposals = document.querySelector(
                '[data-testid="server-column-preference-proposals"]'
            );
            return {
                separatorName: separator ? separator.getAttribute('aria-label') : null,
                min: separator ? Number(separator.getAttribute('aria-valuemin')) : null,
                now: separator ? Number(separator.getAttribute('aria-valuenow')) : null,
                max: separator ? Number(separator.getAttribute('aria-valuemax')) : null,
                valueText: separator ? separator.getAttribute('aria-valuetext') : null,
                trackWidth: track ? track.getBoundingClientRect().width : null,
                acceptedWidths: accepted ? JSON.parse(accepted.textContent) : null,
                proposals: proposals ? Number(proposals.textContent) : null,
                queryProposals: Number(
                    document.querySelector('[data-testid="server-query-proposals"]').textContent
                ),
                scrollX: window.scrollX,
            };
        })()"#,
    )
    .await
}

async fn server_email_width_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#server-table');
            const separator = Array.from(root.querySelectorAll('[role="separator"]'))
                .find(el => el.getAttribute('aria-label') === 'Resize Email column');
            const track = root.querySelector('[data-table-column-track="email"]');
            const accepted = JSON.parse(
                document.querySelector('[data-testid="server-column-widths"]').textContent
            );
            return {
                now: separator ? Number(separator.getAttribute('aria-valuenow')) : null,
                trackWidth: track ? track.getBoundingClientRect().width : null,
                acceptedWidth: accepted.email ?? null,
                proposals: Number(
                    document.querySelector(
                        '[data-testid="server-column-preference-proposals"]'
                    ).textContent
                ),
            };
        })()"#,
    )
    .await
}

async fn dispatch_real_mouse(
    harness: &pixelproof_web::Harness,
    params: DispatchMouseEventParams,
    action: &str,
) {
    harness
        .page()
        .execute(params)
        .await
        .unwrap_or_else(|error| panic!("dispatch {action}: {error}"));
}

async fn begin_real_pointer_drag(
    harness: &pixelproof_web::Harness,
    selector: &str,
    delta_x: f64,
) -> (f64, f64) {
    harness
        .page()
        .find_element(selector)
        .await
        .expect("find pointer resize separator")
        .scroll_into_view()
        .await
        .expect("scroll pointer resize separator into view");
    let bounds = harness
        .element_box(selector)
        .await
        .unwrap_or_else(|error| panic!("element_box {selector}: {error}"));
    let start_x = bounds.x + bounds.width / 2.0;
    let y = bounds.y + bounds.height / 2.0;
    let end_x = start_x + delta_x;
    for (event_type, x, button, buttons, click_count) in [
        (DispatchMouseEventType::MouseMoved, start_x, None, 0, None),
        (
            DispatchMouseEventType::MousePressed,
            start_x,
            Some(MouseButton::Left),
            1,
            Some(1),
        ),
        (
            DispatchMouseEventType::MouseMoved,
            end_x,
            Some(MouseButton::Left),
            1,
            None,
        ),
    ] {
        let mut builder = DispatchMouseEventParams::builder()
            .r#type(event_type)
            .x(x)
            .y(y)
            .buttons(buttons);
        if let Some(button) = button {
            builder = builder.button(button);
        }
        if let Some(click_count) = click_count {
            builder = builder.click_count(click_count);
        }
        dispatch_real_mouse(
            harness,
            builder.build().expect("pointer drag event params"),
            "pointer drag preview",
        )
        .await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
    (end_x, y)
}

async fn finish_real_pointer_drag(harness: &pixelproof_web::Harness, x: f64, y: f64) {
    let released = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseReleased)
        .x(x)
        .y(y)
        .button(MouseButton::Left)
        .buttons(0)
        .click_count(1)
        .build()
        .expect("pointer release params");
    dispatch_real_mouse(harness, released, "pointer drag commit").await;
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

async fn cancel_real_pointer_drag(harness: &pixelproof_web::Harness, selector: &str) {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const separator = document.querySelector({selector:?});
                return separator.dispatchEvent(new PointerEvent('pointercancel', {{
                    bubbles: true,
                    pointerId: 1,
                    pointerType: 'mouse',
                    isPrimary: true,
                }}));
            }})()"#
        ),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn focusing_an_unstored_server_resize_separator_emits_no_proposal() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, "#server-table tbody tr").await;
    begin_browser_error_capture(&harness).await;

    let separator = "#server-table [role='separator'][aria-label='Resize Email column']";
    let before = server_email_width_snapshot(&harness).await;
    assert_eq!(before["acceptedWidth"], Value::Null);
    assert_eq!(before["proposals"], json!(0));
    harness
        .page()
        .find_element(separator)
        .await
        .expect("find Email resize separator")
        .focus()
        .await
        .expect("focus Email resize separator");
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;

    let focused = server_email_width_snapshot(&harness).await;
    assert_eq!(focused["acceptedWidth"], Value::Null);
    assert_eq!(
        focused["proposals"],
        json!(0),
        "focus may hydrate local ARIA/rendered width but must not persist it"
    );
    assert_no_browser_errors(&harness, "server resize focus-only boundary").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn focusing_one_column_cannot_leak_into_another_columns_commit() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, "#server-table tbody tr").await;
    begin_browser_error_capture(&harness).await;

    let email_separator = "#server-table [role='separator'][aria-label='Resize Email column']";
    let name_separator = "#server-table [role='separator'][aria-label='Resize Name column']";
    harness
        .page()
        .find_element(email_separator)
        .await
        .expect("find Email resize separator")
        .focus()
        .await
        .expect("focus unstored Email resize separator");
    harness
        .page()
        .find_element(name_separator)
        .await
        .expect("find Name resize separator")
        .focus()
        .await
        .expect("focus stored Name resize separator");
    harness
        .press_key_sequence(&[pixelproof_web::Key::ArrowRight])
        .await
        .expect("commit Name resize after focusing Email");

    let committed = server_name_width_snapshot(&harness).await;
    assert_eq!(committed["proposals"], json!(1));
    assert_eq!(
        committed["acceptedWidths"],
        json!({ "name": 200 }),
        "a Name commit must not persist Email's focus-only runtime width: {committed}"
    );
    assert_no_browser_errors(&harness, "cross-column focus/commit isolation").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn canceling_an_unstored_column_drag_restores_the_absent_override() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, "#server-table tbody tr").await;
    begin_browser_error_capture(&harness).await;

    let separator = "#server-table [role='separator'][aria-label='Resize Email column']";
    let before = server_email_width_snapshot(&harness).await;
    assert_eq!(before["acceptedWidth"], Value::Null);
    assert_eq!(before["proposals"], json!(0));

    begin_real_pointer_drag(&harness, separator, 96.0).await;
    let preview = server_email_width_snapshot(&harness).await;
    assert!(
        preview["trackWidth"].as_f64() > before["trackWidth"].as_f64(),
        "the canceled journey must first prove a visible pointer preview: before={before}, preview={preview}"
    );
    cancel_real_pointer_drag(&harness, separator).await;

    let canceled = server_email_width_snapshot(&harness).await;
    assert_eq!(canceled["acceptedWidth"], Value::Null);
    assert_eq!(canceled["proposals"], json!(0));
    assert_eq!(
        canceled["now"], before["now"],
        "cancel must remove an override that was absent before pointerdown: before={before}, canceled={canceled}"
    );
    let restored_delta =
        (canceled["trackWidth"].as_f64().unwrap() - before["trackWidth"].as_f64().unwrap()).abs();
    assert!(
        restored_delta <= 0.5,
        "cancel must restore the implicit track geometry: before={before}, canceled={canceled}"
    );
    assert_no_browser_errors(&harness, "server pointer-cancel rollback").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn real_pointer_drag_previews_locally_and_commits_exactly_once() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, "#server-table tbody tr").await;
    begin_browser_error_capture(&harness).await;

    let separator = "#server-table [role='separator'][aria-label='Resize Email column']";
    let before = server_email_width_snapshot(&harness).await;
    assert_eq!(before["acceptedWidth"], Value::Null);
    assert_eq!(before["proposals"], json!(0));

    let (end_x, y) = begin_real_pointer_drag(&harness, separator, 96.0).await;
    let preview = server_email_width_snapshot(&harness).await;
    assert!(
        preview["trackWidth"].as_f64() > before["trackWidth"].as_f64(),
        "pointer movement must remain visible before commit: before={before}, preview={preview}"
    );
    assert_eq!(
        preview["acceptedWidth"],
        Value::Null,
        "pointer preview must remain local until release"
    );
    assert_eq!(
        preview["proposals"],
        json!(0),
        "pointer movement must not emit intermediate replacements"
    );

    finish_real_pointer_drag(&harness, end_x, y).await;
    let committed = server_email_width_snapshot(&harness).await;
    assert_eq!(committed["proposals"], json!(1));
    assert_eq!(
        committed["acceptedWidth"].as_f64(),
        committed["trackWidth"].as_f64(),
        "pointer release must commit the exact rendered preview width"
    );
    assert_eq!(
        committed["now"].as_f64(),
        committed["acceptedWidth"].as_f64(),
        "ARIA state and accepted pointer width must agree"
    );
    assert_no_browser_errors(&harness, "server pointer resize commit boundary").await;
}

/// `ldui-9ke9`: server resize widths are controlled preference state, not
/// header-local presentation. Real keyboard commits update the rendered
/// track and accepted model exactly once; a declined commit restores truth.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn controlled_server_resize_round_trips_the_accepted_width_model() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, "#server-table tbody tr").await;
    begin_browser_error_capture(&harness).await;

    let separator = "#server-table [role='separator'][aria-label='Resize Name column']";
    let initial = server_name_width_snapshot(&harness).await;
    assert_eq!(initial["separatorName"], json!("Resize Name column"));
    assert_eq!(initial["acceptedWidths"], json!({ "name": 184 }));
    assert_eq!(initial["proposals"], json!(0));
    assert_eq!(initial["now"].as_f64(), Some(184.0));
    assert_eq!(initial["trackWidth"].as_f64(), Some(184.0));
    assert!(
        initial["min"].as_f64() <= initial["now"].as_f64()
            && initial["now"].as_f64() <= initial["max"].as_f64(),
        "separator range must be ordered: {initial}"
    );
    assert_eq!(initial["valueText"], json!("184 pixels"));

    harness
        .page()
        .find_element(separator)
        .await
        .expect("find named Name resize separator")
        .focus()
        .await
        .expect("focus named Name resize separator");

    let mut previous = initial;
    for (key, expected) in [
        (pixelproof_web::Key::ArrowRight, 200_u64),
        (pixelproof_web::Key::Home, 48_u64),
        (pixelproof_web::Key::End, 1_200_u64),
    ] {
        harness
            .press_key_sequence(&[key])
            .await
            .expect("commit a keyboard column resize");
        let current = server_name_width_snapshot(&harness).await;
        assert_eq!(current["acceptedWidths"]["name"], json!(expected));
        assert_eq!(current["now"].as_f64(), Some(expected as f64));
        assert_eq!(current["trackWidth"].as_f64(), Some(expected as f64));
        assert_ne!(
            current["trackWidth"], previous["trackWidth"],
            "each committed resize must change the rendered column track"
        );
        assert_eq!(
            current["proposals"].as_u64(),
            previous["proposals"].as_u64().map(|count| count + 1),
            "each committed keyboard resize must emit exactly one replacement"
        );
        assert_eq!(
            current["queryProposals"], previous["queryProposals"],
            "resize must not activate server sorting or mutate the query"
        );
        assert_eq!(
            current["scrollX"], previous["scrollX"],
            "resize keys must not scroll the page"
        );
        assert!(
            current["min"].as_f64() <= current["now"].as_f64()
                && current["now"].as_f64() <= current["max"].as_f64(),
            "separator range must remain ordered: {current}"
        );
        previous = current;
    }

    click(&harness, "[data-testid='server-column-preference-accept']").await;
    harness
        .page()
        .find_element(separator)
        .await
        .expect("find named Name resize separator after toggling acceptance")
        .focus()
        .await
        .expect("restore focus to named Name resize separator");
    harness
        .press_key_sequence(&[pixelproof_web::Key::Home])
        .await
        .expect("propose a declined keyboard resize");
    let declined = server_name_width_snapshot(&harness).await;
    assert_eq!(
        declined["proposals"].as_u64(),
        previous["proposals"].as_u64().map(|count| count + 1)
    );
    assert_eq!(declined["acceptedWidths"]["name"], json!(1_200));
    assert_eq!(declined["now"].as_f64(), Some(1_200.0));
    assert_eq!(
        declined["trackWidth"].as_f64(),
        Some(1_200.0),
        "declining the replacement must restore the accepted rendered width"
    );

    assert_no_browser_errors(&harness, "controlled server column resize").await;
}

// ---------------------------------------------------------------------------
// ldui-px06: controlled checkbox multi-selection over a server slice
// ---------------------------------------------------------------------------

const MULTI: &str = "#server-multi-select-table";

async fn selection_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.querySelector('#server-multi-select-table');
            const header = root.querySelector('[data-server-selection-toggle="slice"]');
            const status = root.querySelector('[data-server-selection]');
            const notice = root.querySelector('[data-server-selection-off-slice-notice]');
            const rowBoxes = Array.from(
                root.querySelectorAll('tbody [data-server-selection-row]')
            );
            const active = document.activeElement;
            return {
                sliceState: status ? status.dataset.serverSelectionSliceState : null,
                scope: status ? status.dataset.serverSelectionScope : null,
                offSlice: status ? status.dataset.serverSelectionOffSlice : null,
                noticeText: notice ? notice.textContent.trim() : null,
                headerChecked: header ? header.checked : null,
                headerIndeterminate: header ? header.indeterminate : null,
                headerDisabled: header ? header.disabled : null,
                headerLabel: header ? header.getAttribute('aria-label') : null,
                headerColumnName: (() => {
                    const cell = root.querySelector('[data-server-selection-header="true"]');
                    return cell ? cell.textContent.trim() : null;
                })(),
                rows: rowBoxes.map(box => ({
                    key: box.dataset.serverSelectionRow,
                    checked: box.checked,
                    blocked: box.dataset.serverSelectionBlocked ?? null,
                    ariaDisabled: box.getAttribute('aria-disabled'),
                    label: box.getAttribute('aria-label'),
                    title: box.getAttribute('title'),
                })),
                ariaSelectedRows: Array.from(root.querySelectorAll('tbody tr[data-row-key]'))
                    .filter(tr => tr.getAttribute('aria-selected') === 'true')
                    .map(tr => tr.dataset.rowKey),
                // One leading control cell per rendered row, and a matching
                // extra <col> track: alignment is part of the contract.
                leadingCells: root.querySelectorAll('tbody [data-table-leading-cell]').length,
                bodyRows: root.querySelectorAll('tbody tr[data-row-key]').length,
                colTracks: root.querySelectorAll('colgroup col').length,
                headerCells: root.querySelectorAll('thead tr:first-child th').length,
                focusedSelectionKey: active
                    ? (active.dataset ? (active.dataset.serverSelectionRow ?? null) : null)
                    : null,
            };
        })()"#,
    )
    .await
}

fn keys_of(snapshot: &Value) -> Vec<String> {
    snapshot["rows"]
        .as_array()
        .expect("row checkboxes")
        .iter()
        .map(|row| row["key"].as_str().unwrap_or_default().to_owned())
        .collect()
}

fn checked_keys(snapshot: &Value) -> Vec<String> {
    snapshot["rows"]
        .as_array()
        .expect("row checkboxes")
        .iter()
        .filter(|row| row["checked"] == json!(true))
        .map(|row| row["key"].as_str().unwrap_or_default().to_owned())
        .collect()
}

async fn focus_selector(harness: &pixelproof_web::Harness, selector: &str) {
    let _ = eval_json(
        harness,
        &format!(
            r#"(() => {{
                const el = document.querySelector({selector:?});
                if (el) {{ el.focus(); }}
                return document.activeElement === el;
            }})()"#
        ),
    )
    .await;
}

/// `ldui-px06` binding acceptance in a real browser: the header checkbox
/// means the current page and only the current page (including its
/// `indeterminate` DOM property), accepted keys for rows that are not
/// displayed survive a cursor transition without relabelling anything on the
/// new slice, a declined proposal leaves no optimistic divergence, `Space`
/// operates a row checkbox without losing its focus, a blocked row stays
/// focusable and says why, and an atomic dataset-scope change clears
/// selection.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn controlled_multi_selection_is_page_scoped_and_never_optimistic() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, &format!("{MULTI} tbody tr")).await;
    begin_browser_error_capture(&harness).await;

    // Page one: conv-1, conv-2 selectable; conv-3 archived and blocked.
    let initial = selection_snapshot(&harness).await;
    assert_eq!(initial["sliceState"], json!("none"));
    assert_eq!(initial["headerChecked"], json!(false));
    assert_eq!(initial["headerIndeterminate"], json!(false));
    assert_eq!(initial["headerDisabled"], json!(false));
    assert_eq!(initial["offSlice"], json!("0"));
    assert_eq!(initial["scope"], json!("conversations/v1"));
    assert_eq!(keys_of(&initial), vec!["conv-1", "conv-2", "conv-3"]);
    assert_eq!(
        initial["leadingCells"], initial["bodyRows"],
        "every rendered row needs exactly one leading control cell: {initial}"
    );
    assert_eq!(
        initial["colTracks"].as_u64(),
        initial["headerCells"].as_u64(),
        "the control column must have its own <col> track: {initial}"
    );
    // Copy names the page, not "all".
    let header_label = initial["headerLabel"].as_str().expect("header label");
    assert!(
        header_label.to_lowercase().contains("this page"),
        "header checkbox must name the current page: {header_label:?}"
    );
    // The blocked row is focusable (aria-disabled, not `disabled`) and its
    // reason is in the accessible name AND the tooltip.
    let blocked = initial["rows"][2].clone();
    assert_eq!(blocked["blocked"], json!("true"));
    assert_eq!(blocked["ariaDisabled"], json!("true"));
    assert!(
        blocked["label"]
            .as_str()
            .is_some_and(|label| label.contains("cannot be selected")),
        "a blocked row must say why in its accessible name: {blocked}"
    );
    assert!(blocked["title"].as_str().is_some_and(|t| !t.is_empty()));

    // One row: partial, and `indeterminate` is a DOM PROPERTY -- an attribute
    // would leave assistive tech with no partial state at all.
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-row=\"conv-1\"]"),
    )
    .await;
    let partial = selection_snapshot(&harness).await;
    assert_eq!(partial["sliceState"], json!("partial"));
    assert_eq!(partial["headerChecked"], json!(false));
    assert_eq!(partial["headerIndeterminate"], json!(true));
    assert_eq!(checked_keys(&partial), vec!["conv-1"]);
    assert_eq!(partial["ariaSelectedRows"], json!(["conv-1"]));

    // Header: covers exactly the SELECTABLE rows on this page. conv-3 is
    // blocked, so the page reads `all` with two of three rows checked --
    // a blocked row must not hold the header at `partial` forever.
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-toggle=\"slice\"]"),
    )
    .await;
    let all = selection_snapshot(&harness).await;
    assert_eq!(all["sliceState"], json!("all"));
    assert_eq!(all["headerChecked"], json!(true));
    assert_eq!(all["headerIndeterminate"], json!(false));
    assert_eq!(checked_keys(&all), vec!["conv-1", "conv-2"]);
    assert_eq!(all["offSlice"], json!("0"));

    // Cursor transition. The two accepted keys are now OFF-slice: they must
    // not relabel anything on the new page, the header must read `none`
    // rather than `partial`, and the count must be stated out loud.
    click(
        &harness,
        &format!("{MULTI} [data-server-cursor-action=\"next\"]"),
    )
    .await;
    let page_two = selection_snapshot(&harness).await;
    assert_eq!(keys_of(&page_two), vec!["conv-4", "conv-5", "conv-6"]);
    assert_eq!(
        page_two["sliceState"],
        json!("none"),
        "off-slice keys must never tint this page's header: {page_two}"
    );
    assert_eq!(page_two["headerIndeterminate"], json!(false));
    assert!(checked_keys(&page_two).is_empty());
    assert_eq!(page_two["ariaSelectedRows"], json!([]));
    assert_eq!(page_two["offSlice"], json!("2"));
    assert!(
        page_two["noticeText"]
            .as_str()
            .is_some_and(|text| text.contains('2') && text.contains("not on this page")),
        "the off-slice count must be stated, not implied: {page_two}"
    );

    // Selecting this page adds to -- never replaces -- the accepted set.
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-toggle=\"slice\"]"),
    )
    .await;
    let page_two_all = selection_snapshot(&harness).await;
    assert_eq!(page_two_all["sliceState"], json!("all"));
    assert_eq!(checked_keys(&page_two_all), vec!["conv-4", "conv-5"]);
    assert_eq!(
        page_two_all["offSlice"],
        json!("2"),
        "page one's accepted keys must still be accepted: {page_two_all}"
    );

    // Back: page one's keys survived the round trip untouched.
    click(
        &harness,
        &format!("{MULTI} [data-server-cursor-action=\"previous\"]"),
    )
    .await;
    let back = selection_snapshot(&harness).await;
    assert_eq!(keys_of(&back), vec!["conv-1", "conv-2", "conv-3"]);
    assert_eq!(back["sliceState"], json!("all"));
    assert_eq!(checked_keys(&back), vec!["conv-1", "conv-2"]);
    assert_eq!(back["offSlice"], json!("2"));

    // Keyboard: Space operates the checkbox, and the checkbox keeps focus
    // through the accepted-state change (it is keyed by business identity,
    // so a data change never re-mounts it out from under the user).
    focus_selector(
        &harness,
        &format!("{MULTI} [data-server-selection-row=\"conv-1\"]"),
    )
    .await;
    harness
        .press_key_sequence(&[pixelproof_web::Key::Space])
        .await
        .expect("Space toggles a row checkbox");
    let after_space = selection_snapshot(&harness).await;
    assert_eq!(checked_keys(&after_space), vec!["conv-2"]);
    assert_eq!(after_space["sliceState"], json!("partial"));
    assert_eq!(
        after_space["focusedSelectionKey"],
        json!("conv-1"),
        "the toggled checkbox must keep keyboard focus: {after_space}"
    );

    // Rejection: the proposal is emitted, and NOTHING moves. This is the
    // assertion that proves the checkbox is controlled rather than merely
    // reported -- a native checkbox flips itself on click, so a component
    // that does not re-assert would silently diverge here.
    let before_reject: u64 = text_of(&harness, "[data-testid=\"multi-proposal-count\"]")
        .await
        .parse()
        .expect("proposal count is numeric");
    click(&harness, "[data-testid=\"multi-accept-toggle\"]").await;
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-row=\"conv-1\"]"),
    )
    .await;
    let rejected = selection_snapshot(&harness).await;
    let after_reject: u64 = text_of(&harness, "[data-testid=\"multi-proposal-count\"]")
        .await
        .parse()
        .expect("proposal count is numeric");
    assert_eq!(
        after_reject,
        before_reject + 1,
        "a declined gesture must still emit exactly one proposal"
    );
    assert_eq!(
        checked_keys(&rejected),
        vec!["conv-2"],
        "a declined proposal must leave the DOM on accepted truth: {rejected}"
    );
    assert_eq!(rejected["sliceState"], json!("partial"));

    // A blocked row emits nothing at all, accepted or not.
    let before_blocked = after_reject;
    click(
        &harness,
        &format!("{MULTI} [data-server-selection-row=\"conv-3\"]"),
    )
    .await;
    let after_blocked: u64 = text_of(&harness, "[data-testid=\"multi-proposal-count\"]")
        .await
        .parse()
        .expect("proposal count is numeric");
    assert_eq!(
        after_blocked, before_blocked,
        "a blocked row must not propose anything"
    );

    // An atomic dataset change: the caller moves the scope and clears the
    // accepted set together, so no key can be carried into a dataset where
    // it means something else.
    click(&harness, "[data-testid=\"multi-accept-toggle\"]").await;
    click(&harness, "[data-testid=\"multi-change-scope\"]").await;
    let rescoped = selection_snapshot(&harness).await;
    assert_eq!(rescoped["scope"], json!("conversations/v2"));
    assert_eq!(rescoped["sliceState"], json!("none"));
    assert_eq!(rescoped["offSlice"], json!("0"));
    assert!(checked_keys(&rescoped).is_empty());

    assert_no_browser_errors(&harness, "server-table controlled multi-selection").await;
}

/// Every framework-owned form control the two demo tables render, with the
/// `id`/`name` each carries.
async fn identity_inventory(harness: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const root = document.querySelector('{root}');
                if (!root) return null;
                const selector = [
                    'input[data-table-search-control="true"]',
                    'select[data-table-page-size-control="true"]',
                    '[data-table-filter-kind]',
                    '[data-server-selection-toggle="slice"]',
                    '[data-server-selection-row]',
                ].join(',');
                return Array.from(root.querySelectorAll(selector)).map(el => ({{
                    role: el.dataset.tableSearchControl ? 'search'
                        : el.dataset.tablePageSizeControl ? 'page-size'
                        : el.dataset.tableFilterKind ? 'filter'
                        : el.dataset.serverSelectionToggle ? 'slice'
                        : 'row',
                    column: el.closest('[data-table-filter-column]')
                        ?.dataset.tableFilterColumn ?? null,
                    key: el.dataset.serverSelectionRow ?? null,
                    id: el.id,
                    name: el.getAttribute('name'),
                    // The accessible name must survive gaining an identity.
                    ariaLabel: el.getAttribute('aria-label'),
                    // An explicit `for` on the wrapping visually-hidden label
                    // is the point of having an `id` at all.
                    labelFor: Array.from(el.labels ?? []).map(l => l.getAttribute('for')),
                }}));
            }})()"#
        ),
    )
    .await
}

fn rows_by_key(inventory: &Value) -> Vec<(String, String)> {
    inventory
        .as_array()
        .expect("inventory is an array")
        .iter()
        .filter(|entry| entry["role"] == json!("row"))
        .map(|entry| {
            (
                entry["key"].as_str().unwrap_or_default().to_owned(),
                entry["id"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

/// `ldui-j6sh` in a real browser: the Office Conversations satellite found
/// visible LDUI-owned form controls with no `id` and no `name`, which a
/// consuming page cannot repair without reaching into markup this crate owns.
///
/// Two tables render on this page -- `#server-table` takes the minted
/// fallback prefix, `#server-multi-select-table` supplies `control_id`
/// -- so one pass proves both paths, proves they cannot collide, and proves
/// a row checkbox's identity follows its stable key across a cursor page
/// rather than its position in the slice.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-server-table-column-tools)"]
async fn every_framework_owned_table_control_has_a_stable_identity() {
    let harness = harness_at("/components/data-table").await;
    wait_for_selector(&harness, &format!("{MULTI} tbody tr")).await;
    begin_browser_error_capture(&harness).await;

    for root in ["#server-table", MULTI] {
        let inventory = identity_inventory(&harness, root).await;
        let entries = inventory
            .as_array()
            .unwrap_or_else(|| panic!("{root} rendered no framework-owned controls"));
        assert!(
            !entries.is_empty(),
            "{root} rendered no framework-owned controls to inspect"
        );
        for entry in entries {
            let id = entry["id"].as_str().unwrap_or_default();
            let name = entry["name"].as_str().unwrap_or_default();
            assert!(
                !id.is_empty() && !id.contains(' '),
                "{root} control has no usable id: {entry}"
            );
            assert_eq!(name, id, "{root} control's name must match its id: {entry}");
            assert!(
                entry["ariaLabel"]
                    .as_str()
                    .is_some_and(|label| !label.trim().is_empty()),
                "gaining an identity must not cost the accessible name: {entry}"
            );
            if entry["role"] == json!("filter") {
                assert_eq!(
                    entry["labelFor"],
                    json!([id]),
                    "a filter's visually-hidden label must point at it: {entry}"
                );
            }
        }
    }

    // The caller-supplied prefix is honoured verbatim, and the row control is
    // named by the ROW KEY (escape-encoded), not by its slice position.
    let multi = identity_inventory(&harness, MULTI).await;
    let page_one = rows_by_key(&multi);
    assert!(
        page_one
            .iter()
            .all(|(_, id)| id.starts_with("conversations-select-row-")),
        "caller-supplied control_id must win: {page_one:?}"
    );
    assert!(
        page_one
            .iter()
            .any(|(key, id)| key == "conv-1" && id == "conversations-select-row-conv_2d1"),
        "row identity must encode the stable key: {page_one:?}"
    );
    let header_id = multi
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["role"] == json!("slice"))
        .and_then(|entry| entry["id"].as_str())
        .expect("the current-slice checkbox renders")
        .to_owned();
    assert_eq!(header_id, "conversations-select-all");
    assert!(
        page_one.iter().all(|(_, id)| *id != header_id),
        "no row may share the current-slice checkbox's identity"
    );

    // Every id on the whole page -- both tables at once -- is unique.
    let duplicates = eval_json(
        &harness,
        r#"(() => {
            const seen = new Map();
            const duplicates = [];
            for (const el of document.querySelectorAll('[id]')) {
                if (seen.has(el.id)) duplicates.push(el.id);
                seen.set(el.id, true);
            }
            return duplicates;
        })()"#,
    )
    .await;
    assert_eq!(
        duplicates,
        json!([]),
        "two tables on one page minted a duplicate id"
    );

    // Page the slice. An index-derived id would hand page two's first row the
    // id page one's first row just had; a key-derived one must not.
    click(
        &harness,
        &format!("{MULTI} [data-server-cursor-action=\"next\"]"),
    )
    .await;
    let page_two = rows_by_key(&identity_inventory(&harness, MULTI).await);
    assert!(!page_two.is_empty());
    assert!(
        page_two
            .iter()
            .all(|(_, id)| page_one.iter().all(|(_, seen)| seen != id)),
        "a paged slice reused an id for a different row: {page_one:?} then {page_two:?}"
    );

    // Back: the same keys must come back with byte-identical ids.
    click(
        &harness,
        &format!("{MULTI} [data-server-cursor-action=\"previous\"]"),
    )
    .await;
    let returned = rows_by_key(&identity_inventory(&harness, MULTI).await);
    assert_eq!(
        returned, page_one,
        "a row's identity must survive a round trip through another page"
    );

    assert_no_browser_errors(&harness, "server-table control identity").await;
}
