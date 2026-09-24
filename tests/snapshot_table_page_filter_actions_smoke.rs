//! Real-browser proof for the opt-in framework utility row on
//! `SnapshotTablePage` (`ldui-nj3q`): the localized visible/total result
//! count, one Reset, and one explicit Save as Default, obtained from the
//! opinionated composite itself rather than from a consumer-composed
//! `FilterBar`.
//!
//! Every assertion about the opted-in page (`#snapshot-actions`) is paired
//! with the same query against a second `SnapshotTablePage` on the same
//! document that does NOT opt in (`#snapshot-plain`). That page is the
//! negative control: it must keep rendering exactly as it does today, with
//! no filter bar, no count, and neither action -- so a passing assertion
//! about the count or the buttons cannot be satisfied by something the
//! composite renders unconditionally. After that same-state comparison, the
//! test transitions the plain page to a no-local-projection, authoritative
//! empty state to prove the wrapper's reachable empty-range passthrough.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .expect("evaluate snapshot-table filter-actions fixture")
        .into_value()
        .expect("snapshot-table filter-actions expression returns JSON")
}

async fn actions_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const describe = (element) => element === null || element === undefined
                ? null
                : {
                    tag: element.tagName,
                    type: element.getAttribute('type'),
                    role: element.getAttribute('role'),
                    // The VISIBLE label. `Button::disabled_reason`
                    // (ldui-p82h) renders its reason as an aria-hidden
                    // `sr-only` span INSIDE the button, so raw textContent
                    // concatenates it ("ResetNo active filters") although
                    // neither the eye nor the accessible name sees it.
                    text: (() => {
                        const clone = element.cloneNode(true);
                        clone.querySelectorAll('[data-button-disabled-reason]')
                            .forEach(hint => hint.remove());
                        return clone.textContent?.trim() ?? null;
                    })(),
                    ariaLabel: element.getAttribute('aria-label'),
                    disabled: element.disabled === true,
                    // The disabled reason as assistive tech receives it:
                    // the text of the element aria-describedby names.
                    describedBy: (() => {
                        const id = element.getAttribute('aria-describedby');
                        const hint = id ? document.getElementById(id) : null;
                        return hint?.textContent?.trim() ?? null;
                    })(),
                };
            const opted = document.getElementById('snapshot-actions-filters');
            const bar = opted?.querySelector('[data-filter-bar="local"]') ?? null;
            const plain = document.getElementById('snapshot-plain-filters');
            const plainTable = document.getElementById('snapshot-plain-table');
            const plainRowRange = plainTable?.querySelector('[data-entity-row-range]');
            const feedback = bar?.querySelector('[data-filter-save-feedback]') ?? null;
            return {
                barPresent: bar !== null,
                barLabel: bar?.getAttribute('aria-label') ?? null,
                resultCount: bar
                    ?.querySelector('[data-filter-result-count]')
                    ?.textContent?.trim() ?? null,
                reset: describe(bar?.querySelector('[data-filter-reset]')),
                save: describe(bar?.querySelector('[data-filter-save-default]')),
                chips: bar?.querySelectorAll('[data-filter-summary] [data-active-filters] .badge')
                    .length ?? null,
                feedbackKind: feedback?.dataset.filterSaveFeedback ?? null,
                feedbackRole: feedback?.getAttribute('role') ?? null,
                feedbackText: feedback?.textContent?.trim() ?? null,
                // Consumer content still renders inside the opted-in row.
                consumerFilterPresent:
                    opted?.querySelector('[data-testid="actions-filter-urgent"]') !== null
                    && opted?.querySelector('[data-testid="actions-filter-urgent"]') !== undefined,
                rows: document
                    .querySelectorAll('#snapshot-actions-table [data-entity-table-grid] tbody tr')
                    .length,
                resetClicks: document
                    .querySelector('[data-testid="actions-reset-clicks"]')
                    ?.textContent?.trim() ?? null,
                savedFilter: document
                    .querySelector('[data-testid="actions-saved-filter"]')
                    ?.textContent?.trim() ?? null,
                // Negative control: the composite without `filter_actions`.
                plainBarPresent: plain?.querySelector('[data-filter-bar]') !== null
                    && plain?.querySelector('[data-filter-bar]') !== undefined,
                plainResultCount: plain
                    ?.querySelector('[data-filter-result-count]')
                    ?.textContent?.trim() ?? null,
                plainReset: describe(plain?.querySelector('[data-filter-reset]')),
                plainSave: describe(plain?.querySelector('[data-filter-save-default]')),
                plainConsumerFilterPresent:
                    plain?.querySelector('[data-testid="plain-filter-all"]') !== null
                    && plain?.querySelector('[data-testid="plain-filter-all"]') !== undefined,
                plainRows: plainTable
                    ?.querySelectorAll('[data-entity-table-grid] tbody tr[data-entity-row-key]')
                    .length ?? null,
                plainRowRange: plainRowRange?.textContent?.trim() ?? null,
                plainRowRangeVisible:
                    plainRowRange ? plainRowRange.getClientRects().length > 0 : false,
                plainRowRangeProbe: plainRowRange?.dataset.emptyRangeProbe ?? null,
            };
        })()"#,
    )
    .await
}

/// `ldui-nj3q`, binding acceptance: a snapshot-table consumer obtains the
/// result count, Reset, and Save as Default from `SnapshotTablePage` alone;
/// both actions are real buttons with stable accessible names in English and
/// Spanish; and a page that does not opt in renders none of it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-snapshot-table-page-filter-actions)"]
async fn filter_actions_supply_count_reset_and_save_without_a_consumer_filter_bar() {
    let harness = harness_at("/components/snapshot-table-page-filter-actions").await;
    wait_for_selector(
        &harness,
        "#snapshot-actions-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    // --- English, unfiltered, nothing saved -----------------------------
    let initial = actions_snapshot(&harness).await;
    assert_eq!(initial["barPresent"], json!(true));
    assert_eq!(initial["barLabel"], json!("Filters"));
    assert_eq!(
        initial["resultCount"],
        json!("3 of 3 results"),
        "the composite must render the framework result count: {initial}"
    );
    assert_eq!(initial["rows"], json!(3));
    // The consumer's own `filters` content is composed inside the row, not
    // displaced by it.
    assert_eq!(initial["consumerFilterPresent"], json!(true));

    assert_eq!(initial["reset"]["tag"], json!("BUTTON"));
    assert_eq!(initial["reset"]["type"], json!("button"));
    assert_eq!(initial["reset"]["text"], json!("Reset"));
    // No active filter yet, so the framework Reset reports nothing to reset.
    assert_eq!(initial["reset"]["disabled"], json!(true));
    // ...and says so (ldui-p82h): a disabled action carries its reason.
    assert_eq!(
        initial["reset"]["describedBy"],
        json!("No active filters"),
        "a disabled Reset must describe why: {initial}"
    );

    assert_eq!(initial["save"]["tag"], json!("BUTTON"));
    assert_eq!(initial["save"]["type"], json!("button"));
    assert_eq!(initial["save"]["text"], json!("Save as Default"));
    assert_eq!(
        initial["save"]["ariaLabel"],
        json!("Save as Default. Defaults are already saved"),
        "a clean view must say why Save is unavailable in its accessible name: {initial}"
    );
    assert_eq!(initial["save"]["disabled"], json!(true));
    assert_eq!(
        initial["save"]["describedBy"],
        json!("Defaults are already saved"),
        "a disabled Save must describe why: {initial}"
    );
    assert_eq!(initial["feedbackKind"], json!(null));

    // --- Negative control: the same composite, no opt-in ----------------
    assert_eq!(
        initial["plainBarPresent"],
        json!(false),
        "a page that does not opt in must render no filter bar: {initial}"
    );
    assert_eq!(initial["plainResultCount"], json!(null));
    assert_eq!(initial["plainReset"], json!(null));
    assert_eq!(initial["plainSave"], json!(null));
    assert_eq!(
        initial["plainConsumerFilterPresent"],
        json!(true),
        "the un-opted page must still render its own filters slot content"
    );
    assert_eq!(initial["plainRows"], json!(3), "{initial}");
    assert_eq!(
        initial["plainRowRange"],
        json!("Showing 1-3 of 3"),
        "positive ranges must ignore the configured empty copy: {initial}"
    );

    // Only after the established same-data negative control is complete do
    // we move this page to the supported no-local-projection empty path.
    click(&harness, "[data-testid='plain-empty-snapshot']").await;
    let plain_empty = actions_snapshot(&harness).await;
    assert_eq!(plain_empty["plainRows"], json!(0), "{plain_empty}");
    assert_eq!(
        plain_empty["plainRowRange"],
        json!("Activity rows 0 of 0"),
        "the no-local-projection wrapper must forward its empty-range copy: {plain_empty}"
    );
    assert_eq!(
        plain_empty["plainRowRangeVisible"],
        json!(true),
        "{plain_empty}"
    );
    assert_eq!(
        eval_json(
            &harness,
            r#"(() => {
                const range = document.querySelector(
                    '#snapshot-plain-table [data-entity-row-range]'
                );
                range.dataset.emptyRangeProbe = 'retained';
                return range.textContent.trim();
            })()"#,
        )
        .await,
        json!("Activity rows 0 of 0")
    );

    // --- The count tracks the identity-bound local projection -----------
    click(&harness, "[data-testid='actions-filter-urgent']").await;
    let filtered = actions_snapshot(&harness).await;
    assert_eq!(filtered["resultCount"], json!("1 of 3 results"));
    assert_eq!(filtered["rows"], json!(1));
    assert_eq!(filtered["chips"], json!(1));
    assert_eq!(
        filtered["reset"]["disabled"],
        json!(false),
        "an active filter must enable the framework Reset: {filtered}"
    );
    assert_eq!(
        filtered["plainResultCount"],
        json!(null),
        "the un-opted page must not grow a count when the opted-in one changes"
    );

    // --- Reset is a real activation, not decoration ---------------------
    click(&harness, "[data-filter-reset]").await;
    let after_reset = actions_snapshot(&harness).await;
    assert_eq!(after_reset["resetClicks"], json!("1"));
    assert_eq!(after_reset["resultCount"], json!("3 of 3 results"));
    assert_eq!(after_reset["rows"], json!(3));
    assert_eq!(after_reset["chips"], json!(0));
    assert_eq!(after_reset["reset"]["disabled"], json!(true));

    // --- Save as Default: dirty -> activation -> saved feedback ---------
    click(&harness, "[data-testid='actions-save-dirty']").await;
    let dirty = actions_snapshot(&harness).await;
    assert_eq!(dirty["save"]["disabled"], json!(false));
    assert_eq!(
        dirty["save"]["ariaLabel"],
        json!("Save as Default"),
        "a dirty view must drop the disabled reason from the accessible name: {dirty}"
    );
    assert_eq!(dirty["savedFilter"], json!("(none)"));

    click(&harness, "[data-filter-save-default]").await;
    let saved = actions_snapshot(&harness).await;
    assert!(
        saved["savedFilter"]
            .as_str()
            .expect("saved filter value is text")
            .contains("all"),
        "Save must deliver the schema-projected payload to the consumer: {saved}"
    );
    assert_eq!(saved["feedbackKind"], json!("status"));
    assert_eq!(saved["feedbackRole"], json!("status"));
    assert_eq!(saved["feedbackText"], json!("Default view saved"));
    assert_eq!(saved["save"]["disabled"], json!(true));

    // --- A rejected save is an assertive alert, still retryable ---------
    click(&harness, "[data-testid='actions-save-conflict']").await;
    let conflict = actions_snapshot(&harness).await;
    assert_eq!(conflict["feedbackKind"], json!("alert"));
    assert_eq!(conflict["feedbackRole"], json!("alert"));
    assert_eq!(
        conflict["feedbackText"],
        json!("Default view conflict: A newer default exists.")
    );
    assert_eq!(conflict["save"]["disabled"], json!(false));

    // --- Spanish: one `FilterBarTexts` localizes all of it --------------
    click(&harness, "[data-testid='actions-locale-es']").await;
    let spanish = actions_snapshot(&harness).await;
    assert_eq!(spanish["barLabel"], json!("Filtros"));
    assert_eq!(spanish["resultCount"], json!("3 de 3 resultados"));
    assert_eq!(spanish["reset"]["text"], json!("Restablecer"));
    assert_eq!(spanish["reset"]["tag"], json!("BUTTON"));
    assert_eq!(
        spanish["save"]["text"],
        json!("Guardar como predeterminado")
    );
    assert_eq!(spanish["save"]["tag"], json!("BUTTON"));
    assert_eq!(
        spanish["save"]["ariaLabel"],
        json!("Guardar como predeterminado"),
        "the Spanish conflict state keeps the bare localized name: {spanish}"
    );
    assert_eq!(
        spanish["feedbackText"],
        json!("Conflicto de vista predeterminada: A newer default exists.")
    );
    assert_eq!(
        spanish["plainRowRange"],
        json!("Filas de actividad: 0 de 0"),
        "the mounted wrapper footer did not react to the caller's locale signal: {spanish}"
    );
    assert_eq!(spanish["plainRowRangeProbe"], json!("retained"));

    // A clean Spanish view carries the localized disabled reason too.
    click(&harness, "[data-filter-save-default]").await;
    let spanish_saved = actions_snapshot(&harness).await;
    assert_eq!(
        spanish_saved["save"]["ariaLabel"],
        json!("Guardar como predeterminado. Los valores predeterminados ya están guardados")
    );
    assert_eq!(
        spanish_saved["feedbackText"],
        json!("Vista predeterminada guardada")
    );

    // The un-opted page never grew any of it, in either language.
    assert_eq!(spanish_saved["plainBarPresent"], json!(false));
    assert_eq!(spanish_saved["plainReset"], json!(null));
    assert_eq!(spanish_saved["plainSave"], json!(null));

    // The utility row is persistence-neutral: the framework never writes.
    assert_eq!(
        eval_json(&harness, "Object.keys(window.localStorage).length").await,
        json!(0),
        "the framework utility row must never perform storage I/O"
    );

    assert_no_browser_errors(&harness, "snapshot-table-page filter actions").await;
}

/// Browser proof for two layout changes that native tests cannot see
/// (op-trula side panel; FilterBar empty-frame collapse). Both are geometry,
/// so both are measured rather than inferred from markup.
///
/// SIDE PANEL. `#snapshot-side` passes one; `#snapshot-plain` and
/// `#snapshot-actions` do not, and are the negative control that an unset
/// panel changes nothing. Measured at 1440 wide, because the harness mounts
/// narrower and `lg:flex-row` only applies from 1024.
///
/// COLLAPSE. The fixture FilterBar holds only an empty status line, a
/// visible-but-`absolute` span, and a closed Modal. The frame must collapse
/// while that is all it holds, re-expand when real text appears, and collapse
/// again when it goes. The floating span is asserted VISIBLE, so a pass cannot
/// come from it simply being absent: it is present, rendered, and correctly
/// ignored because it is out of flow (the 4iiz-etl review's point -- an OPEN
/// dialog is visible and viewport-sized too).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-snapshot-table-page-filter-actions)"]
async fn a_side_panel_sits_beside_the_table_and_an_empty_filter_frame_collapses() {
    use pixelproof_web::ViewportSize;
    use std::time::Duration;

    let harness = harness_at("/components/snapshot-table-page-filter-actions").await;
    begin_browser_error_capture(&harness).await;
    harness
        .set_viewport(ViewportSize::new(1440, 900))
        .await
        .expect("set a viewport wide enough for lg:flex-row");
    wait_for_selector(
        &harness,
        "#snapshot-side [data-snapshot-page-slot=\"side-panel\"]",
    )
    .await;

    let side = eval_json(
        &harness,
        r#"(() => {
            const slot = sel => document.querySelector(`${sel} [data-snapshot-page-slot="table"]`);
            const side = slot('#snapshot-side');
            const table = side.querySelector('[data-entity-table]');
            const panel = side.querySelector('[data-snapshot-page-slot="side-panel"]');
            const t = table ? table.getBoundingClientRect() : null;
            const p = panel ? panel.getBoundingClientRect() : null;
            return {
                flagged: side.getAttribute('data-snapshot-side-panel'),
                hasContent: !!side.querySelector('[data-testid="side-panel-content"]'),
                tableWidth: t ? t.width : null,
                panelWidth: p ? p.width : null,
                panelRightOfTable: t && p ? p.left >= t.right - 1 : null,
                sameRow: t && p ? Math.abs(p.top - t.top) < 1 : null,
                plainFlag: slot('#snapshot-plain').getAttribute('data-snapshot-side-panel'),
                plainAside: !!slot('#snapshot-plain').querySelector('aside'),
                actionsFlag: slot('#snapshot-actions').getAttribute('data-snapshot-side-panel'),
            };
        })()"#,
    )
    .await;
    assert_eq!(side["flagged"], json!("true"), "{side}");
    assert_eq!(
        side["hasContent"],
        json!(true),
        "panel content rendered: {side}"
    );
    assert!(
        side["tableWidth"].as_f64().is_some_and(|w| w > 200.0),
        "the table must keep real width beside the panel, not be squeezed out: {side}"
    );
    assert!(
        side["panelWidth"]
            .as_f64()
            .is_some_and(|w| (355.0..=365.0).contains(&w)),
        "the column takes its CONTENT's width (the fixture panel is 360px); a slot          must not impose a width, or a 360px PersonPicker scrolls sideways in it: {side}"
    );
    assert_eq!(
        side["panelRightOfTable"],
        json!(true),
        "side by side at lg: {side}"
    );
    assert_eq!(
        side["sameRow"],
        json!(true),
        "top-aligned on one row: {side}"
    );
    // Negative controls: an unset panel changes nothing.
    assert_eq!(side["plainFlag"], json!(null), "{side}");
    assert_eq!(side["plainAside"], json!(false), "{side}");
    assert_eq!(side["actionsFlag"], json!(null), "{side}");

    async fn frame(harness: &pixelproof_web::Harness) -> Value {
        eval_json(
            harness,
            r#"(() => {
                const bar = document.querySelector(
                    '[data-testid="filter-bar-collapse-fixture"] [data-filter-bar]');
                const cs = getComputedStyle(bar);
                const floating = document.querySelector('[data-testid="collapse-floating"]');
                const f = floating.getBoundingClientRect();
                const actionsBar = document.querySelector('#snapshot-actions [data-filter-bar]');
                return {
                    empty: bar.getAttribute('data-filter-bar-empty'),
                    borderTop: cs.borderTopWidth,
                    paddingTop: cs.paddingTop,
                    height: bar.getBoundingClientRect().height,
                    floatingVisible: f.width > 0 && f.height > 0
                        && getComputedStyle(floating).visibility === 'visible',
                    actionsEmpty: actionsBar ? actionsBar.getAttribute('data-filter-bar-empty') : 'absent',
                };
            })()"#,
        )
        .await
    }

    async fn frame_until(harness: &pixelproof_web::Harness, want_empty: bool) -> Value {
        let mut last = frame(harness).await;
        for _ in 0..60 {
            if (last["empty"] == json!("true")) == want_empty {
                return last;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
            last = frame(harness).await;
        }
        panic!("filter frame never settled to empty={want_empty}: {last}");
    }

    // At rest: only invisible and out-of-flow content, so the frame collapses.
    let rest = frame_until(&harness, true).await;
    assert_eq!(
        rest["floatingVisible"],
        json!(true),
        "the out-of-flow span must be PRESENT and VISIBLE, or this proves nothing: {rest}"
    );
    assert_eq!(
        rest["borderTop"],
        json!("0px"),
        "no border when empty: {rest}"
    );
    assert_eq!(
        rest["paddingTop"],
        json!("0px"),
        "no padding when empty: {rest}"
    );
    assert_eq!(
        rest["actionsEmpty"],
        json!(null),
        "negative control: a filter row with a visible count/Reset/Save stays framed: {rest}"
    );

    // Real text appears: the frame comes back.
    click(&harness, "[data-testid=\"collapse-show\"]").await;
    let shown = frame_until(&harness, false).await;
    assert_ne!(shown["borderTop"], json!("0px"), "framed again: {shown}");

    // And goes again when the text does.
    click(&harness, "[data-testid=\"collapse-hide\"]").await;
    let hidden = frame_until(&harness, true).await;
    assert_eq!(
        hidden["borderTop"],
        json!("0px"),
        "collapses again: {hidden}"
    );

    assert_no_browser_errors(&harness, "side panel and filter-frame collapse").await;
}

/// ldui-8ia5 (4iiz-Office /no-hires/, measured on 4671d75): a `filters` slot
/// whose ONLY child is a FilterBar that collapsed itself still spent a row
/// gap -- two gaps between the dataset select and the table, not one.
/// `#snapshot-collapsed-filters` holds exactly that shape: a FilterBar around
/// an empty status line. At rest the slot is out of flow (like the idle
/// feedback slot) and the dataset-to-table distance is ONE row gap; when the
/// status line gets text the bar expands and the slot rejoins the flow; when
/// the text goes, the gap goes again. `#snapshot-actions`, whose bar shows a
/// count, is the negative control that a non-empty bar stays in flow.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-snapshot-table-page-filter-actions)"]
async fn a_filters_slot_holding_only_a_collapsed_filter_bar_costs_no_gap() {
    use std::time::Duration;

    let harness = harness_at("/components/snapshot-table-page-filter-actions").await;
    begin_browser_error_capture(&harness).await;
    wait_for_selector(
        &harness,
        "#snapshot-collapsed-filters [data-snapshot-page-slot=\"filters\"] [data-filter-bar]",
    )
    .await;

    async fn slot(harness: &pixelproof_web::Harness) -> Value {
        eval_json(
            harness,
            r#"(() => {
                const root = document.querySelector('#snapshot-collapsed-filters');
                const part = name => root.querySelector(
                    `:scope > [data-snapshot-page-slot="${name}"]`);
                const dataset = part('dataset').getBoundingClientRect();
                const table = part('table').getBoundingClientRect();
                const filters = part('filters');
                const bar = filters.querySelector(':scope > [data-filter-bar]');
                const status = filters.querySelector('[data-testid="slot-status"]');
                const control = document.querySelector(
                    '#snapshot-actions > [data-snapshot-page-slot="filters"]');
                return {
                    rowGap: parseFloat(getComputedStyle(root).rowGap),
                    datasetToTable: Math.round(table.top - dataset.bottom),
                    filtersPosition: getComputedStyle(filters).position,
                    filtersDisplay: getComputedStyle(filters).display,
                    filtersHeight: Math.round(filters.getBoundingClientRect().height),
                    barEmpty: bar ? bar.getAttribute('data-filter-bar-empty') : 'absent',
                    statusRendered: !!status && getComputedStyle(status).display !== 'none',
                    controlPosition: control ? getComputedStyle(control).position : null,
                };
            })()"#,
        )
        .await
    }

    async fn slot_until(harness: &pixelproof_web::Harness, want_empty: bool) -> Value {
        let mut last = slot(harness).await;
        for _ in 0..60 {
            let empty = last["barEmpty"] == json!("true");
            let out = last["filtersPosition"] == json!("absolute");
            if empty == want_empty && out == want_empty {
                return last;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
            last = slot(harness).await;
        }
        panic!("filters slot never settled to empty={want_empty}: {last}");
    }

    let rest = slot_until(&harness, true).await;
    let gap = rest["rowGap"].as_f64().unwrap_or(0.0);
    assert!(gap > 0.0, "the page column has a row gap to lose: {rest}");
    // Rects are fractional; one pixel of rounding either way.
    let near = |v: &Value, want: f64| v.as_f64().is_some_and(|got| (got - want).abs() <= 1.0);
    assert!(
        near(&rest["datasetToTable"], gap),
        "ONE row gap between the dataset select and the table, not two: {rest}"
    );
    assert_ne!(
        rest["filtersDisplay"],
        json!("none"),
        "out of flow, never display:none -- the bar must keep observing: {rest}"
    );
    assert_eq!(
        rest["statusRendered"],
        json!(true),
        "the status live region stays rendered while collapsed: {rest}"
    );
    assert_eq!(
        rest["controlPosition"],
        json!("static"),
        "negative control: a bar showing a count stays in flow: {rest}"
    );

    click(&harness, "[data-testid=\"slot-status-show\"]").await;
    let shown = slot_until(&harness, false).await;
    let height = shown["filtersHeight"].as_f64().unwrap_or(0.0);
    assert!(height > 0.0, "the expanded bar has height: {shown}");
    assert!(
        near(&shown["datasetToTable"], 2.0 * gap + height),
        "back in flow: two gaps around the bar: {shown}"
    );

    click(&harness, "[data-testid=\"slot-status-hide\"]").await;
    let hidden = slot_until(&harness, true).await;
    assert!(
        near(&hidden["datasetToTable"], gap),
        "the gap goes again with the text: {hidden}"
    );

    assert_no_browser_errors(&harness, "collapsed filter bar costs no gap").await;
}

/// ldui-xhrw: a slot whose only content is an EMPTY WRAPPER -- `#snapshot-side`
/// passes `<div data-testid="side-filters"></div>` as its filters -- renders
/// nothing but used to stay a 0px flex item spending a gap, because the
/// no-element rule cannot see a wrapper as empty. It is out of flow now, never
/// display:none (a wrapper may hold a live region). `#snapshot-plain`, whose
/// filters hold a real Button, is the negative control that stays in flow.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-snapshot-table-page-filter-actions)"]
async fn a_slot_holding_only_an_empty_wrapper_costs_no_gap() {
    let harness = harness_at("/components/snapshot-table-page-filter-actions").await;
    begin_browser_error_capture(&harness).await;
    wait_for_selector(&harness, "#snapshot-side [data-testid=\"side-filters\"]").await;

    let shape = eval_json(
        &harness,
        r#"(() => {
            const measure = id => {
                const root = document.querySelector(id);
                const part = name => root.querySelector(
                    `:scope > [data-snapshot-page-slot="${name}"]`);
                const filters = part('filters');
                const cs = getComputedStyle(filters);
                return {
                    rowGap: parseFloat(getComputedStyle(root).rowGap),
                    datasetToTable: part('table').getBoundingClientRect().top
                        - part('dataset').getBoundingClientRect().bottom,
                    filtersHeight: filters.getBoundingClientRect().height,
                    position: cs.position,
                    display: cs.display,
                };
            };
            return { side: measure('#snapshot-side'), plain: measure('#snapshot-plain') };
        })()"#,
    )
    .await;
    let near = |v: &Value, want: f64| v.as_f64().is_some_and(|got| (got - want).abs() <= 1.0);
    let side = &shape["side"];
    let gap = side["rowGap"].as_f64().unwrap_or(0.0);
    assert!(gap > 0.0, "the page column has a row gap to lose: {shape}");
    assert_eq!(
        side["position"],
        json!("absolute"),
        "an empty-wrapper slot is out of flow: {shape}"
    );
    assert_ne!(
        side["display"],
        json!("none"),
        "out of flow, never display:none: {shape}"
    );
    assert!(
        near(&side["datasetToTable"], gap),
        "ONE row gap between the dataset select and the table: {shape}"
    );
    let plain = &shape["plain"];
    let height = plain["filtersHeight"].as_f64().unwrap_or(0.0);
    assert_eq!(
        plain["position"],
        json!("static"),
        "negative control: real filter content stays in flow: {shape}"
    );
    assert!(
        height > 0.0 && near(&plain["datasetToTable"], 2.0 * gap + height),
        "negative control: two gaps around a real filter row: {shape}"
    );

    // An empty element can still PAINT: a daisyUI spinner has no children.
    // Styled empties must keep the slot in flow, or a KPI slot showing only
    // loading skeletons would vanish. Insert one, then take it out again.
    let spinner = eval_json(
        &harness,
        r#"(async () => {
            const wrap = document.querySelector('#snapshot-side [data-testid="side-filters"]');
            const slot = wrap.closest('[data-snapshot-page-slot="filters"]');
            const s = document.createElement('span');
            s.className = 'loading loading-spinner';
            wrap.appendChild(s);
            await new Promise(r => requestAnimationFrame(() => r()));
            const withSpinner = getComputedStyle(slot).position;
            const spinnerWidth = s.getBoundingClientRect().width;
            s.remove();
            await new Promise(r => requestAnimationFrame(() => r()));
            return { withSpinner, spinnerWidth, after: getComputedStyle(slot).position };
        })()"#,
    )
    .await;
    assert!(
        spinner["spinnerWidth"].as_f64().is_some_and(|w| w > 0.0),
        "the spinner really paints, or this proves nothing: {spinner}"
    );
    assert_eq!(
        spinner["withSpinner"],
        json!("static"),
        "a styled empty element keeps the slot in flow: {spinner}"
    );
    assert_eq!(
        spinner["after"],
        json!("absolute"),
        "and the slot leaves the flow again without it: {spinner}"
    );

    assert_no_browser_errors(&harness, "empty-wrapper slot costs no gap").await;
}
