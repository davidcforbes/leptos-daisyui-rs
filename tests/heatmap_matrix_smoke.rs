//! Real-browser proof for the typed accessible `Heatmap` (ldui-8d94).
//!
//! **Compile-only pending a gate run on this machine** — this lane is not yet
//! registered in `xtask`, and the primary evidence for the feature is the
//! native coverage in `src/charts/heatmap/{scale,geometry,normalize,
//! interaction,types}.rs` plus `src/charts/heatmap/tests.rs`, which prove the
//! colour ramp, the grid arithmetic, the dense normalization, the two-axis
//! reducer, the role decisions and the activation payload without a browser.
//! What only a browser can add is the rendered matrix, the live focus journeys
//! across both axes and the reactive locale change, which is what these tests
//! are.
//!
//! Every test drives `/components/charts`, whose typed fixture is three offices
//! by six KPIs with one reported gap, an exact zero, and both judgement
//! hues in play.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, oracle,
    wait_for_selector,
};
use serde_json::{Value, json};

const GRID: &str = "[data-testid='typed-heatmap'] [data-testid='heatmap']";
const SINGLE: &str = "[data-testid='single-office-heatmap'] [data-testid='heatmap']";

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .expect("evaluate heatmap fixture")
        .into_value()
        // A JS `null` is a legitimate answer here -- `focused_cell` returns it
        // when nothing is focused, which is exactly what the reconciliation
        // tests need to distinguish from a cell. chromiumoxide surfaces that
        // as "No value found", so mapping it to `Value::Null` is what lets the
        // helper express its own documented empty case instead of panicking.
        .unwrap_or(Value::Null)
}

/// Reads every drawn cell group straight out of the DOM, keyed by its two
/// stable identities rather than by position.
async fn cells(harness: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const grid = document.querySelector("{root}");
                return Array.from(grid.querySelectorAll('g[data-heatmap-cell]')).map((g) => {{
                    const rect = g.querySelector('rect');
                    const rule = g.querySelector('[data-heatmap-sense-rule]');
                    return {{
                        rowKey: g.dataset.rowKey,
                        columnKey: g.dataset.columnKey,
                        sense: g.dataset.heatmapSense,
                        missing: g.hasAttribute('data-heatmap-missing'),
                        tile: !!rect,
                        fill: rect ? (rect.getAttribute('style') ?? rect.getAttribute('fill')) : null,
                        text: g.querySelector('text')?.textContent?.trim() ?? null,
                        rule: rule ? rule.getAttribute('stroke-dasharray') : null,
                    }};
                }});
            }})()"#
        ),
    )
    .await
}

/// The accessible table as a matrix: the header row, then each body row's
/// header cell followed by its value cells.
async fn table(harness: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const table = document.querySelector("{root} [data-heatmap-table]");
                return {{
                    caption: table.querySelector('caption').textContent.trim(),
                    columnHeaders: Array.from(table.querySelectorAll('thead th')).map((th) => ({{
                        text: th.textContent.trim(),
                        scope: th.getAttribute('scope'),
                        columnKey: th.dataset.columnKey ?? null,
                    }})),
                    rows: Array.from(table.querySelectorAll('tbody tr')).map((tr) => ({{
                        rowKey: tr.dataset.rowKey,
                        header: {{
                            text: tr.querySelector('th').textContent.trim(),
                            scope: tr.querySelector('th').getAttribute('scope'),
                        }},
                        cells: Array.from(tr.querySelectorAll('td')).map((td) => ({{
                            columnKey: td.dataset.columnKey,
                            text: td.textContent.trim(),
                            missing: td.hasAttribute('data-heatmap-missing'),
                        }})),
                    }})),
                }};
            }})()"#
        ),
    )
    .await
}

/// Focuses the target for `(row, column)` and sends one key event to it.
async fn press_on(
    harness: &pixelproof_web::Harness,
    row: &str,
    column: &str,
    dom_key: &str,
    ctrl: bool,
) {
    let expression = format!(
        r#"(() => {{
            const target = document.querySelector(
                "{GRID} [data-heatmap-focus][data-row-key='{row}'][data-column-key='{column}']"
            );
            target.focus();
            target.dispatchEvent(new KeyboardEvent('keydown', {{
                key: '{dom_key}', ctrlKey: {ctrl}, bubbles: true, cancelable: true
            }}));
            return true;
        }})()"#
    );
    let _ = eval_json(harness, &expression).await;
    tokio::time::sleep(std::time::Duration::from_millis(harness.config().settle_ms)).await;
}

/// The two identities of whatever currently has focus.
/// The cell the ROVING TAB STOP points at.
///
/// Deliberately not `document.activeElement`: clicking a control outside the
/// grid -- a sort button, say -- necessarily moves DOM focus to that control,
/// and the component neither can nor should hold focus on a cell the user has
/// clicked away from. What must survive a data change is where a reader LANDS
/// when they tab back in, which is the roving tab stop (ldui-8d94).
async fn roving_cell(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        &format!(
            r#"(() => {{
                const el = document.querySelector(
                    "{GRID} [data-heatmap-focus][tabindex='0']"
                );
                if (!el) return null;
                return {{ rowKey: el.dataset.rowKey, columnKey: el.dataset.columnKey }};
            }})()"#
        ),
    )
    .await
}

async fn focused_cell(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        "(() => {
            const el = document.activeElement;
            if (!el || !el.dataset || el.dataset.rowKey === undefined) return null;
            return { rowKey: el.dataset.rowKey, columnKey: el.dataset.columnKey };
        })()",
    )
    .await
}

// ── the matrix a screen reader hears ────────────────────────────────────────

/// The heatmap's non-visual truth is a MATRIX, not a flat list: both axes are
/// header cells with the right scope, so a value is located by its row and
/// column headers rather than by counting.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn the_data_table_states_a_row_by_column_matrix() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    let table = table(&harness, GRID).await;
    let headers = table["columnHeaders"]
        .as_array()
        .expect("the header row exists");
    assert_eq!(
        headers.len(),
        // 7 = the corner cell naming the row axis plus one header per KPI.
        // Six KPIs since the fixture was de-crowded to stop slanted headers
        // overlapping (OVERLAP is a hard failure, never ratcheted).
        7,
        "the corner cell names the row axis, then one header per KPI"
    );
    assert_eq!(headers[0]["text"], json!("Office"));
    assert_eq!(headers[0]["columnKey"], json!(null));
    for header in headers {
        assert_eq!(header["scope"], json!("col"));
    }
    assert_eq!(headers[1]["columnKey"], json!("closed"));

    let rows = table["rows"].as_array().expect("the body rows exist");
    assert_eq!(rows.len(), 3);
    for row in rows {
        assert_eq!(
            row["header"]["scope"],
            json!("row"),
            "a row label must be a header a reader can be told, not a plain cell"
        );
        assert_eq!(
            row["cells"].as_array().expect("cells").len(),
            6,
            "every coordinate has a cell, so a gap is heard at its own position"
        );
    }
    assert_eq!(rows[0]["rowKey"], json!("north"));

    assert_no_browser_errors(&harness, "heatmap data table").await;
}

/// A gap reads as the localized missing copy at its own coordinate; it never
/// arrives as a zero and never silently disappears from the matrix.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_missing_cell_is_stated_as_missing_and_draws_no_tile() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    let cells = cells(&harness, GRID).await;
    let cells = cells.as_array().expect("cells are an array");
    let gap = cells
        .iter()
        .find(|cell| cell["rowKey"] == json!("south") && cell["columnKey"] == json!("handle"))
        .expect("the unreported coordinate is still in the grid");
    assert_eq!(gap["missing"], json!(true));
    assert_eq!(gap["tile"], json!(false), "a gap paints no tile");
    assert_eq!(gap["sense"], json!("neutral"));

    let table = table(&harness, GRID).await;
    let south = table["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|row| row["rowKey"] == json!("south"))
        .expect("the south row");
    let cell = south["cells"]
        .as_array()
        .expect("cells")
        .iter()
        .find(|cell| cell["columnKey"] == json!("handle"))
        .expect("the gap's cell");
    assert_eq!(cell["missing"], json!(true));
    assert_eq!(cell["text"], json!("Not reported"));

    assert_no_browser_errors(&harness, "heatmap missing cell").await;
}

/// The judgement is never carried by hue alone: a judged cell draws a solid or
/// dashed sense rule AND states its verdict in words.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_judgement_is_drawn_and_spoken_rather_than_only_tinted() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    let cells = cells(&harness, GRID).await;
    let cells = cells.as_array().expect("cells are an array");
    let favorable = cells
        .iter()
        .find(|cell| cell["sense"] == json!("favorable"))
        .expect("the fixture carries a favorable cell");
    let unfavorable = cells
        .iter()
        .find(|cell| cell["sense"] == json!("unfavorable"))
        .expect("the fixture carries an unfavorable cell");
    assert_eq!(favorable["rule"], json!("none"));
    assert_eq!(unfavorable["rule"], json!("3 2"));
    assert_ne!(
        favorable["rule"], unfavorable["rule"],
        "the two verdicts must differ in pattern, not only in hue"
    );
    for cell in [favorable, unfavorable] {
        let fill = cell["fill"].as_str().expect("a judged cell is painted");
        assert!(
            fill.starts_with("fill: color-mix("),
            "a theme token must ride on style, never on the fill attribute: {fill}"
        );
    }

    let table = table(&harness, GRID).await;
    let spoken: Vec<String> = table["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(|row| row["cells"].as_array().expect("cells").iter())
        .filter_map(|cell| cell["text"].as_str().map(str::to_owned))
        .collect();
    assert!(
        spoken.iter().any(|text| text.ends_with("Favorable")),
        "no cell stated a favorable verdict in words: {spoken:?}"
    );
    assert!(
        spoken.iter().any(|text| text.ends_with("Unfavorable")),
        "no cell stated an unfavorable verdict in words: {spoken:?}"
    );

    assert_no_browser_errors(&harness, "heatmap judgement").await;
}

// ── keyboard, in two axes ───────────────────────────────────────────────────

/// The ARIA grid model: Left/Right walk the row, Up/Down walk the column, and
/// both clamp at the edge instead of wrapping.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn arrow_keys_move_in_both_axes_and_clamp_at_the_edges() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    press_on(&harness, "south", "sla", "ArrowRight", false).await;
    assert_eq!(
        focused_cell(&harness).await,
        json!({ "rowKey": "south", "columnKey": "handle" })
    );

    press_on(&harness, "south", "sla", "ArrowDown", false).await;
    assert_eq!(
        focused_cell(&harness).await,
        json!({ "rowKey": "east", "columnKey": "sla" })
    );

    press_on(&harness, "north", "closed", "ArrowLeft", false).await;
    assert_eq!(
        focused_cell(&harness).await,
        json!({ "rowKey": "north", "columnKey": "closed" }),
        "the first column clamps rather than wrapping to the last"
    );

    press_on(&harness, "north", "closed", "ArrowUp", false).await;
    assert_eq!(
        focused_cell(&harness).await,
        json!({ "rowKey": "north", "columnKey": "closed" }),
        "the first row clamps rather than wrapping to the last"
    );

    assert_no_browser_errors(&harness, "heatmap arrows").await;
}

/// Home/End are ROW-wise and Ctrl+Home / Ctrl+End are grid-wise, which is what
/// makes a twelve-column row traversable without losing the office.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn home_and_end_stay_in_the_row_while_control_reaches_the_grid_corners() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    press_on(&harness, "south", "sla", "End", false).await;
    assert_eq!(
        focused_cell(&harness).await,
        json!({ "rowKey": "south", "columnKey": "first-touch" })
    );

    press_on(&harness, "south", "sla", "Home", false).await;
    assert_eq!(
        focused_cell(&harness).await,
        json!({ "rowKey": "south", "columnKey": "closed" }),
        "Home must not change the row"
    );

    press_on(&harness, "south", "sla", "End", true).await;
    assert_eq!(
        focused_cell(&harness).await,
        json!({ "rowKey": "east", "columnKey": "first-touch" })
    );

    press_on(&harness, "south", "sla", "Home", true).await;
    assert_eq!(
        focused_cell(&harness).await,
        json!({ "rowKey": "north", "columnKey": "closed" })
    );

    assert_no_browser_errors(&harness, "heatmap home and end").await;
}

/// One tab stop for the whole grid, and Escape drops the highlight without
/// sending a reader back to the corner.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn the_grid_is_one_tab_stop_and_escape_keeps_it_where_it_was() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    let stops = eval_json(
        &harness,
        &format!(
            r#"(() => Array.from(document.querySelectorAll(
                "{GRID} [data-heatmap-focus][tabindex='0']"
            )).map((el) => ({{ rowKey: el.dataset.rowKey, columnKey: el.dataset.columnKey }})))()"#
        ),
    )
    .await;
    assert_eq!(
        stops.as_array().expect("stops").len(),
        1,
        "a roving grid offers exactly one tab stop"
    );

    press_on(&harness, "east", "handle", "Escape", false).await;
    let stops = eval_json(
        &harness,
        &format!(
            r#"(() => Array.from(document.querySelectorAll(
                "{GRID} [data-heatmap-focus][tabindex='0']"
            )).map((el) => ({{ rowKey: el.dataset.rowKey, columnKey: el.dataset.columnKey }})))()"#
        ),
    )
    .await;
    assert_eq!(
        stops,
        json!([{ "rowKey": "east", "columnKey": "handle" }]),
        "Escape must not move the tab stop back to the corner"
    );

    assert_no_browser_errors(&harness, "heatmap tab stop").await;
}

// ── activation ──────────────────────────────────────────────────────────────

/// Pointer and keyboard both activate, and the payload carries two stable keys
/// and no index.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn pointer_and_keyboard_activate_with_stable_row_and_column_keys() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    click(
        &harness,
        &format!("{GRID} [data-heatmap-focus][data-row-key='south'][data-column-key='closed']"),
    )
    .await;
    let state = oracle(&harness).await;
    let activation = &state["state"]["heatmap.activation"];
    assert_eq!(activation["rowKey"], json!("south"));
    assert_eq!(activation["columnKey"], json!("closed"));
    assert_eq!(activation["source"], json!("pointer"));
    assert_eq!(
        activation.get("rowIndex"),
        None,
        "an activation must expose no array position"
    );
    assert_eq!(activation.get("columnIndex"), None);
    let first = state["state"]["heatmap.activation_count"].clone();

    press_on(&harness, "east", "intake", "Enter", false).await;
    let state = oracle(&harness).await;
    let activation = &state["state"]["heatmap.activation"];
    assert_eq!(activation["rowKey"], json!("east"));
    assert_eq!(activation["columnKey"], json!("intake"));
    assert_eq!(activation["source"], json!("keyboard"));
    assert_ne!(
        state["state"]["heatmap.activation_count"], first,
        "the keyboard activation must fire once more, not overwrite silently"
    );

    assert_no_browser_errors(&harness, "heatmap activation").await;
}

/// A coordinate with no measurement is still a coordinate: it activates, and
/// reports an absent intensity rather than a fabricated zero.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_missing_coordinate_activates_without_inventing_a_value() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    click(
        &harness,
        &format!("{GRID} [data-heatmap-focus][data-row-key='south'][data-column-key='handle']"),
    )
    .await;
    let state = oracle(&harness).await;
    let activation = &state["state"]["heatmap.activation"];
    assert_eq!(activation["rowKey"], json!("south"));
    assert_eq!(activation["columnKey"], json!("handle"));
    assert_eq!(activation["intensity"], json!(null));
    assert_eq!(activation["displayValue"], json!("Not reported"));

    assert_no_browser_errors(&harness, "heatmap missing activation").await;
}

// ── reactive data and reactive copy ─────────────────────────────────────────

/// Focus follows the same cell through a reorder of the row axis, and moves
/// predictably when its column disappears.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn focus_follows_the_same_cell_across_reorder_and_removal() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    press_on(&harness, "south", "sla", "ArrowUp", false).await;
    press_on(&harness, "south", "sla", "ArrowDown", false).await;
    let before = roving_cell(&harness).await;

    click(&harness, "[data-testid='heatmap-sort']").await;
    assert_eq!(
        roving_cell(&harness).await,
        before,
        "sorting the offices must not move a reader off the cell they were on"
    );

    click(&harness, "[data-testid='heatmap-remove-column']").await;
    let after = roving_cell(&harness).await;
    assert_ne!(after, json!(null), "focus must not vanish with the column");
    assert_eq!(
        after["rowKey"], before["rowKey"],
        "removing a column keeps the reader on their office"
    );

    click(&harness, "[data-testid='heatmap-restore']").await;
    assert_no_browser_errors(&harness, "heatmap reconciliation").await;
}

/// EN -> ES -> EN changes every framework-owned word and nothing else.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_locale_change_rewrites_the_copy_without_touching_the_data() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    let english = table(&harness, GRID).await;
    let english_cells = cells(&harness, GRID).await;

    click(&harness, "[data-testid='heatmap-locale']").await;
    let spanish = table(&harness, GRID).await;
    assert_eq!(
        spanish["columnHeaders"][0]["text"],
        json!("Oficina"),
        "the row axis name is framework copy and must change"
    );
    assert_eq!(
        spanish["columnHeaders"][1]["text"],
        json!("Matters closed"),
        "the caller's own labels must NOT be translated by the framework"
    );
    assert!(
        spanish["caption"]
            .as_str()
            .expect("caption")
            .starts_with("Desviacion"),
    );
    assert_eq!(
        cells(&harness, GRID).await,
        english_cells,
        "keys, intensities, tiles and sense rules must survive a locale change"
    );

    click(&harness, "[data-testid='heatmap-locale']").await;
    assert_eq!(
        table(&harness, GRID).await,
        english,
        "EN -> ES -> EN must land exactly where it started"
    );

    assert_no_browser_errors(&harness, "heatmap locale").await;
}

/// An emptied matrix draws the LOCALIZED no-data copy — the defect that
/// hard-coded English `"No data"` — and is a different state from a grid whose
/// cells happen to be missing.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn the_empty_state_is_localized_copy_rather_than_hardcoded_english() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    click(&harness, "[data-testid='heatmap-locale']").await;
    click(&harness, "[data-testid='heatmap-clear']").await;
    let empty = eval_json(
        &harness,
        &format!(
            r#"(() => {{
                const grid = document.querySelector("{GRID}");
                return {{
                    text: grid.querySelector('[data-heatmap-empty] text')?.textContent?.trim()
                        ?? null,
                    cells: grid.querySelectorAll('g[data-heatmap-cell]').length,
                    stops: grid.querySelectorAll('[data-heatmap-focus]').length,
                }};
            }})()"#
        ),
    )
    .await;
    assert_eq!(empty["text"], json!("Sin datos"));
    assert_eq!(empty["cells"], json!(0));
    assert_eq!(
        empty["stops"],
        json!(0),
        "an empty grid offers nothing to tab to"
    );

    click(&harness, "[data-testid='heatmap-restore']").await;
    click(&harness, "[data-testid='heatmap-locale']").await;
    assert_no_browser_errors(&harness, "heatmap empty state").await;
}

// ── the descriptive posture ─────────────────────────────────────────────────

/// The consumer's own shape, with no callback: a full matrix, a named and
/// described image, and NO button roles or tab stops anywhere.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn a_reporting_only_heatmap_gains_no_button_roles_or_tab_stops() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, SINGLE).await;
    begin_browser_error_capture(&harness).await;

    let posture = eval_json(
        &harness,
        &format!(
            r#"(() => {{
                const grid = document.querySelector("{SINGLE}");
                const svg = grid.querySelector('[data-heatmap-plot]');
                return {{
                    svgRole: svg.getAttribute('role'),
                    named: !!svg.getAttribute('aria-labelledby'),
                    titled: svg.querySelector('title')?.textContent?.trim() ?? null,
                    buttons: grid.querySelectorAll("[role='button']").length,
                    stops: grid.querySelectorAll('[tabindex]').length,
                    focusTargets: grid.querySelectorAll('[data-heatmap-focus]').length,
                    rows: grid.querySelectorAll('[data-heatmap-table] tbody tr').length,
                    columns: grid.querySelectorAll(
                        '[data-heatmap-table] tbody tr:first-child td'
                    ).length,
                }};
            }})()"#
        ),
    )
    .await;

    assert_eq!(posture["svgRole"], json!("img"));
    assert_eq!(posture["named"], json!(true));
    assert_eq!(posture["titled"], json!("North office scorecard"));
    assert_eq!(posture["buttons"], json!(0));
    assert_eq!(posture["stops"], json!(0));
    assert_eq!(posture["focusTargets"], json!(0));
    assert_eq!(posture["rows"], json!(1));
    assert_eq!(
        posture["columns"],
        json!(6),
        "one office by six KPIs, stated in full"
    );

    assert_no_browser_errors(&harness, "reporting-only heatmap").await;
}

/// The interactive grid is a named GROUP, never `role="img"` — which would
/// make its focusable cells presentational and blocks axe.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (unregistered lane — see module docs)"]
async fn an_interactive_heatmap_is_a_group_with_named_cells() {
    let harness = harness_at("/components/charts").await;
    wait_for_selector(&harness, GRID).await;
    begin_browser_error_capture(&harness).await;

    let posture = eval_json(
        &harness,
        &format!(
            r#"(() => {{
                const grid = document.querySelector("{GRID}");
                const svg = grid.querySelector('[data-heatmap-plot]');
                const target = grid.querySelector(
                    "[data-heatmap-focus][data-row-key='north'][data-column-key='closed']"
                );
                return {{
                    svgRole: svg.getAttribute('role'),
                    targetRole: target.getAttribute('role'),
                    name: target.getAttribute('aria-label'),
                    unnamed: Array.from(
                        grid.querySelectorAll('[data-heatmap-focus]')
                    ).filter((el) => !el.getAttribute('aria-label')).length,
                }};
            }})()"#
        ),
    )
    .await;

    assert_eq!(posture["svgRole"], json!("group"));
    assert_eq!(posture["targetRole"], json!("button"));
    assert_eq!(posture["unnamed"], json!(0));
    let name = posture["name"].as_str().expect("the cell is named");
    for expected in ["Office", "North", "KPI", "Matters closed", "Deviation"] {
        assert!(
            name.contains(expected),
            "the accessible name must locate the cell on both axes: {name}"
        );
    }

    assert_no_browser_errors(&harness, "interactive heatmap roles").await;
}
