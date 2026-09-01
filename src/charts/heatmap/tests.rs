//! Component-level decisions that are pure functions of the data, and so are
//! provable natively rather than only in a browser: which roles are claimed,
//! what a cell is named, and what an activation may carry.

use super::*;

fn grid() -> NormalizedHeatmap {
    normalize(&HeatmapMatrix::new(
        vec![
            HeatmapCategory::new("north", "North"),
            HeatmapCategory::new("south", "South"),
        ],
        vec![
            HeatmapCategory::new("closed", "Matters closed"),
            HeatmapCategory::new("handle", "Handle time"),
        ],
        vec![
            HeatmapValue::new("north", "closed", 0.6)
                .with_display_value("+12%")
                .with_accessible_value("12 percent above the 12-week baseline"),
            HeatmapValue::new("south", "closed", -0.45).with_display_value("-9%"),
            HeatmapValue::new("north", "handle", 0.0).with_display_value("0%"),
        ],
    ))
}

fn cell(row: usize, column: usize) -> NormalizedCell {
    grid().cell(row, column).cloned().expect("cell")
}

fn spanish() -> HeatmapTexts {
    HeatmapTexts {
        no_data: "Sin datos".to_string(),
        data_table_caption: "Datos del mapa".to_string(),
        row_header: "Oficina".to_string(),
        column_header: "Indicador".to_string(),
        value_header: "Valor".to_string(),
        missing_value: "Sin valor".to_string(),
        sense_favorable: "Favorable".to_string(),
        sense_unfavorable: "Desfavorable".to_string(),
        sense_neutral: "Neutral".to_string(),
    }
}

// ── roles ───────────────────────────────────────────────────────────────────

#[test]
fn an_interactive_heatmap_is_a_group_and_never_an_image() {
    // role="img" makes every descendant presentational, which contradicts the
    // focusable cells inside it — an axe blocker, and the reactivity lane
    // carries a zero-blocking axe gate.
    assert_eq!(svg_role(true), "group");
    assert_eq!(svg_role(false), "img");
}

#[test]
fn only_a_wired_callback_earns_button_semantics() {
    assert_eq!(target_role(true), "button");
    assert_eq!(
        target_role(false),
        "group",
        "a grid a reader may explore but not act on must not announce buttons"
    );
}

// ── what a cell is named ────────────────────────────────────────────────────

#[test]
fn a_cell_states_both_axis_names_because_an_svg_target_has_no_headers() {
    // The defect: a bare "+12%" painted as SVG text carries no row, no column
    // and no measure, so a screen-reader user hears a percentage with nothing
    // to attach it to.
    let texts = HeatmapTexts {
        row_header: "Office".to_string(),
        column_header: "KPI".to_string(),
        ..HeatmapTexts::default()
    };

    assert_eq!(
        accessible_name(&cell(0, 0), HeatScale::Magnitude, &texts),
        "Office: North, KPI: Matters closed, Value: 12 percent above the 12-week baseline"
    );
}

#[test]
fn a_judged_cell_names_its_verdict_so_the_hue_is_not_the_only_carrier() {
    let texts = HeatmapTexts::default();

    assert_eq!(
        accessible_name(&cell(1, 0), HeatScale::Judgement, &texts),
        "Row: South, Column: Matters closed, Value: -9%, Unfavorable"
    );
    assert_eq!(
        accessible_name(&cell(0, 0), HeatScale::Judgement, &texts),
        "Row: North, Column: Matters closed, \
         Value: 12 percent above the 12-week baseline, Favorable"
    );
}

#[test]
fn an_exactly_zero_deviation_is_named_without_a_verdict() {
    // Zero is a real measurement and paints fully transparent under either
    // hue, so claiming a verdict for it would be inventing one.
    assert_eq!(
        accessible_name(&cell(0, 1), HeatScale::Judgement, &HeatmapTexts::default()),
        "Row: North, Column: Handle time, Value: 0%"
    );
}

#[test]
fn a_missing_cell_is_named_missing_rather_than_zero() {
    assert_eq!(
        accessible_name(&cell(1, 1), HeatScale::Judgement, &HeatmapTexts::default()),
        "Row: South, Column: Handle time, Value: No value"
    );
}

#[test]
fn every_word_in_an_accessible_name_comes_from_the_supplied_copy() {
    // The i18n criterion in one assertion: EN -> ES changes every
    // framework-owned word and leaves the caller's own data untouched.
    let english = accessible_name(&cell(1, 0), HeatScale::Judgement, &HeatmapTexts::default());
    let spanish_name = accessible_name(&cell(1, 0), HeatScale::Judgement, &spanish());

    assert_eq!(
        spanish_name,
        "Oficina: South, Indicador: Matters closed, Valor: -9%, Desfavorable"
    );
    assert_ne!(english, spanish_name);
    for framework_word in ["Row", "Column", "Value", "Unfavorable"] {
        assert!(
            !spanish_name.contains(framework_word),
            "{framework_word} survived the locale change in {spanish_name}"
        );
    }
    assert!(
        spanish_name.contains("South") && spanish_name.contains("-9%"),
        "the caller's own labels and values must not be translated"
    );
}

#[test]
fn a_missing_cell_states_the_supplied_missing_copy() {
    assert!(
        accessible_name(&cell(1, 1), HeatScale::Judgement, &spanish()).ends_with("Sin valor"),
        "the missing-cell copy is supplied, not hardcoded"
    );
}

// ── what an activation carries ──────────────────────────────────────────────

#[test]
fn an_activation_carries_both_stable_keys_and_never_an_index() {
    // The defect this replaces: `(row_index, col_index)`. Sorting the offices
    // worst-first, or hiding a KPI column, re-points both numbers at a
    // different cell with no error anywhere.
    let intent = activation_for(
        &cell(1, 0),
        HeatScale::Judgement,
        &HeatmapTexts::default(),
        HeatmapActivationSource::Keyboard,
        modifiers_of(true, false, false, false),
    );

    assert_eq!(intent.row_key, "south");
    assert_eq!(intent.row_label, "South");
    assert_eq!(intent.column_key, "closed");
    assert_eq!(intent.column_label, "Matters closed");
    assert_eq!(intent.intensity, Some(-0.45));
    assert_eq!(intent.display_value, "-9%");
    assert_eq!(intent.sense, HeatmapSense::Unfavorable);
    assert_eq!(intent.source, HeatmapActivationSource::Keyboard);
    assert!(intent.modifiers.shift);
    assert!(!intent.modifiers.ctrl);

    // The type itself cannot carry a position: this is the whole guarantee,
    // and it is checked by the debug rendering rather than by a comment.
    let rendered = format!("{intent:?}");
    assert!(
        !rendered.contains("index"),
        "an activation must expose no index field: {rendered}"
    );
}

#[test]
fn a_missing_cell_activates_as_a_coordinate_rather_than_as_a_zero() {
    // A heatmap cell is an Office-by-KPI coordinate; a reader drilling into
    // one with no reported number is asking about the coordinate. The
    // intensity is therefore absent, never a fabricated zero.
    let intent = activation_for(
        &cell(1, 1),
        HeatScale::Judgement,
        &HeatmapTexts::default(),
        HeatmapActivationSource::Pointer,
        HeatmapModifiers::default(),
    );

    assert_eq!(intent.row_key, "south");
    assert_eq!(intent.column_key, "handle");
    assert_eq!(intent.intensity, None);
    assert_eq!(intent.display_value, "No value");
    assert_eq!(intent.sense, HeatmapSense::Neutral);
}

#[test]
fn an_activation_states_the_value_exactly_as_the_table_states_it() {
    // One resolver feeds the drawn text, the accessible name, the table cell
    // and the payload, so a caller's complete localized text cannot reach one
    // and not the others.
    let texts = HeatmapTexts::default();
    let cell = cell(0, 0);
    let intent = activation_for(
        &cell,
        HeatScale::Judgement,
        &texts,
        HeatmapActivationSource::Pointer,
        HeatmapModifiers::default(),
    );

    assert_eq!(intent.display_value, cell.value_text(&texts));
    assert!(
        cell.stated_text(HeatScale::Judgement, &texts)
            .starts_with(&intent.display_value)
    );
    assert_eq!(cell.visible_text().as_deref(), Some("+12%"));
}

#[test]
fn an_activation_reports_the_verdict_the_hue_showed() {
    let texts = HeatmapTexts::default();
    let magnitude = activation_for(
        &cell(1, 0),
        HeatScale::Magnitude,
        &texts,
        HeatmapActivationSource::Pointer,
        HeatmapModifiers::default(),
    );

    assert_eq!(
        magnitude.sense,
        HeatmapSense::Neutral,
        "the magnitude scale expresses no verdict, so neither does its payload"
    );
}

// ── ids ─────────────────────────────────────────────────────────────────────

#[test]
fn target_ids_are_unique_per_heatmap_instance_and_cell() {
    // Two heatmaps on one page must not share an id, or focusing a cell in the
    // first would move focus into the second.
    assert_eq!(target_id(3, 1, 2), "heatmap-3-r1-c2");
    assert_ne!(target_id(3, 1, 2), target_id(4, 1, 2));
    assert_ne!(target_id(3, 1, 2), target_id(3, 2, 1));
}
