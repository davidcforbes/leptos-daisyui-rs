//! Render-safe matrix: a dense `rows x columns` grid of resolved cells.
//!
//! Two things happen here and nowhere else. A non-finite intensity is folded to
//! "no measurement" before any renderer can format it into an attribute, and a
//! value naming a row or column key the matrix does not have is dropped rather
//! than being placed at a guessed position — the two ways a positional grid
//! silently paints the wrong tile.
//!
//! The result is DENSE: every `(row, column)` combination exists as a cell,
//! populated or missing. That is what lets the data table state a complete
//! matrix (so a reader can locate any value by its two headers) and what lets
//! the keyboard grid move to any coordinate, rather than skipping over gaps and
//! leaving a reader unable to tell an empty cell from an absent one.

use super::scale::{HeatScale, HeatmapSense};
use super::types::{HeatmapCategory, HeatmapMatrix, HeatmapTexts, HeatmapValue};

/// One grid position, fully resolved: its two identities, its two labels, and
/// whatever measurement was reported for it.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedCell {
    pub row_key: String,
    pub column_key: String,
    pub row_label: String,
    pub column_label: String,
    /// Finite or absent. Normalization has already rejected NaN and infinity,
    /// so no renderer downstream can leak one into an attribute.
    pub intensity: Option<f64>,
    pub display_value: Option<String>,
    pub accessible_value: Option<String>,
}

impl NormalizedCell {
    /// A grid position no value was reported for.
    fn missing(row: &HeatmapCategory, column: &HeatmapCategory) -> Self {
        Self {
            row_key: row.key.clone(),
            column_key: column.key.clone(),
            row_label: row.label.clone(),
            column_label: column.label.clone(),
            intensity: None,
            display_value: None,
            accessible_value: None,
        }
    }

    /// Whether a measurement was reported for this position.
    pub(super) fn is_measured(&self) -> bool {
        self.intensity.is_some()
    }

    /// The judgement this cell's colour carries under `scale`.
    pub(super) fn sense(&self, scale: HeatScale) -> HeatmapSense {
        HeatmapSense::of(self.intensity, scale)
    }

    /// The short text DRAWN inside the cell, or `None` for a cell that carries
    /// colour only. Never fabricated: a missing cell draws nothing rather than
    /// a placeholder a sighted reader would mistake for a measurement.
    pub(super) fn visible_text(&self) -> Option<String> {
        self.display_value.clone()
    }

    /// The value text this cell is STATED as — in the data table, in its
    /// accessible name, and in the activation payload.
    ///
    /// One resolution order, used by all three, so the table cannot say
    /// something the accessible name does not: the caller's own complete
    /// localized text wins, then the short drawn text, then the localized
    /// missing-value copy.
    pub(super) fn value_text(&self, texts: &HeatmapTexts) -> String {
        self.accessible_value
            .clone()
            .or_else(|| self.display_value.clone())
            .unwrap_or_else(|| texts.missing_value.clone())
    }

    /// What a reader hears for this cell: the value, plus the judgement in
    /// WORDS whenever the caller's sign convention expressed one.
    ///
    /// The second half is why a judged heatmap is not colour-only. Under the
    /// magnitude scale, and for an exactly-zero deviation, there is no verdict
    /// to state and this is the value alone.
    pub(super) fn stated_text(&self, scale: HeatScale, texts: &HeatmapTexts) -> String {
        let value = self.value_text(texts);
        match self.sense(scale) {
            HeatmapSense::Neutral => value,
            sense => format!("{value}, {}", texts.sense_text(sense)),
        }
    }
}

/// A dense, render-safe grid plus the two axes it was built from.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedHeatmap {
    pub rows: Vec<HeatmapCategory>,
    pub columns: Vec<HeatmapCategory>,
    /// `cells[row][column]`, always exactly `rows.len()` by `columns.len()`.
    cells: Vec<Vec<NormalizedCell>>,
}

impl NormalizedHeatmap {
    /// Whether there is no grid to draw at all — which is a different state
    /// from a grid whose cells are all missing, and reads as the localized
    /// no-data copy rather than as a matrix of gaps.
    pub(super) fn is_empty(&self) -> bool {
        self.rows.is_empty() || self.columns.is_empty()
    }

    /// The cell at `(row, column)`, or `None` when either index is out of the
    /// grid.
    pub(super) fn cell(&self, row: usize, column: usize) -> Option<&NormalizedCell> {
        self.cells.get(row)?.get(column)
    }

    /// Every cell in reading order, with its grid coordinates.
    pub(super) fn iter(&self) -> impl Iterator<Item = (usize, usize, &NormalizedCell)> {
        self.cells.iter().enumerate().flat_map(|(row, cells)| {
            cells
                .iter()
                .enumerate()
                .map(move |(column, cell)| (row, column, cell))
        })
    }

    /// The row axis' keys, top to bottom.
    pub(super) fn row_keys(&self) -> Vec<String> {
        self.rows.iter().map(|row| row.key.clone()).collect()
    }

    /// The column axis' keys, left to right.
    pub(super) fn column_keys(&self) -> Vec<String> {
        self.columns
            .iter()
            .map(|column| column.key.clone())
            .collect()
    }
}

/// Index of `key` within `categories`, or `None` when the axis does not have
/// it.
fn position(categories: &[HeatmapCategory], key: &str) -> Option<usize> {
    categories.iter().position(|category| category.key == key)
}

/// Builds the dense grid.
///
/// A value whose `row_key` or `column_key` is not on the corresponding axis is
/// DROPPED — the position it would occupy is unknowable, and placing it at a
/// guessed index is exactly the silent mis-paint stable keys exist to prevent.
/// Two values for the same position resolve last-wins, so a caller appending a
/// correction does not have to remove the original first.
pub(super) fn normalize(matrix: &HeatmapMatrix) -> NormalizedHeatmap {
    let cells: Vec<Vec<NormalizedCell>> = matrix
        .rows
        .iter()
        .map(|row| {
            matrix
                .columns
                .iter()
                .map(|column| NormalizedCell::missing(row, column))
                .collect()
        })
        .collect();
    let mut normalized = NormalizedHeatmap {
        rows: matrix.rows.clone(),
        columns: matrix.columns.clone(),
        cells,
    };

    for value in &matrix.values {
        let Some(row) = position(&matrix.rows, &value.row_key) else {
            continue;
        };
        let Some(column) = position(&matrix.columns, &value.column_key) else {
            continue;
        };
        let Some(cell) = normalized
            .cells
            .get_mut(row)
            .and_then(|cells| cells.get_mut(column))
        else {
            continue;
        };
        apply(cell, value);
    }

    normalized
}

/// Copies a caller's value onto its grid position, folding a non-finite
/// intensity to "no measurement" on the way through.
fn apply(cell: &mut NormalizedCell, value: &HeatmapValue) {
    cell.intensity = value.intensity.filter(|intensity| intensity.is_finite());
    cell.display_value = value.display_value.clone();
    cell.accessible_value = value.accessible_value.clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis(keys: &[(&str, &str)]) -> Vec<HeatmapCategory> {
        keys.iter()
            .map(|(key, label)| HeatmapCategory::new(*key, *label))
            .collect()
    }

    fn office_kpi(values: Vec<HeatmapValue>) -> NormalizedHeatmap {
        normalize(&HeatmapMatrix::new(
            axis(&[("north", "North"), ("south", "South")]),
            axis(&[("closed", "Closed"), ("sla", "SLA met")]),
            values,
        ))
    }

    #[test]
    fn the_grid_is_dense_so_every_coordinate_exists() {
        // A reader must be able to reach, and hear, a position nobody reported
        // a number for — otherwise a gap is indistinguishable from the end of
        // the row.
        let grid = office_kpi(vec![HeatmapValue::new("north", "closed", 0.5)]);

        assert_eq!(grid.iter().count(), 4);
        assert!(grid.cell(0, 0).expect("populated").is_measured());
        assert!(!grid.cell(0, 1).expect("gap").is_measured());
        assert!(!grid.cell(1, 0).expect("gap").is_measured());
        assert_eq!(grid.cell(2, 0), None);
        assert_eq!(grid.cell(0, 2), None);
    }

    #[test]
    fn a_cell_carries_both_of_its_identities_and_both_of_its_labels() {
        let grid = office_kpi(vec![]);
        let cell = grid.cell(1, 1).expect("cell");

        assert_eq!(cell.row_key, "south");
        assert_eq!(cell.column_key, "sla");
        assert_eq!(cell.row_label, "South");
        assert_eq!(cell.column_label, "SLA met");
    }

    #[test]
    fn a_value_naming_an_unknown_key_is_dropped_rather_than_guessed_at() {
        // Placing it at index 0 is how a typo silently repaints the wrong
        // office; refusing it leaves an honest gap.
        let grid = office_kpi(vec![
            HeatmapValue::new("nrth", "closed", 1.0),
            HeatmapValue::new("north", "clsed", 1.0),
        ]);

        assert!(grid.iter().all(|(_, _, cell)| !cell.is_measured()));
    }

    #[test]
    fn a_non_finite_intensity_folds_to_no_measurement() {
        for intensity in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let grid = office_kpi(vec![HeatmapValue::new("north", "closed", intensity)]);
            assert_eq!(
                grid.cell(0, 0).expect("cell").intensity,
                None,
                "{intensity}"
            );
        }
    }

    #[test]
    fn a_later_value_for_the_same_position_wins() {
        let grid = office_kpi(vec![
            HeatmapValue::new("north", "closed", 0.1).with_display_value("stale"),
            HeatmapValue::new("north", "closed", 0.9).with_display_value("fresh"),
        ]);

        let cell = grid.cell(0, 0).expect("cell");
        assert_eq!(cell.intensity, Some(0.9));
        assert_eq!(cell.visible_text().as_deref(), Some("fresh"));
    }

    #[test]
    fn an_axis_with_nothing_on_it_is_empty_rather_than_a_grid_of_gaps() {
        let no_rows = normalize(&HeatmapMatrix::new(
            vec![],
            axis(&[("closed", "Closed")]),
            vec![],
        ));
        let no_columns = normalize(&HeatmapMatrix::new(
            axis(&[("north", "North")]),
            vec![],
            vec![],
        ));

        assert!(no_rows.is_empty());
        assert!(no_columns.is_empty());
        assert!(
            !office_kpi(vec![]).is_empty(),
            "a grid of gaps is not empty"
        );
    }

    #[test]
    fn the_axes_keys_come_back_in_visual_order() {
        let grid = office_kpi(vec![]);

        assert_eq!(grid.row_keys(), vec!["north", "south"]);
        assert_eq!(grid.column_keys(), vec!["closed", "sla"]);
    }

    // --- what a cell is stated as ---------------------------------------

    #[test]
    fn the_callers_complete_text_wins_over_the_drawn_abbreviation() {
        let texts = HeatmapTexts::default();
        let grid = office_kpi(vec![
            HeatmapValue::new("north", "closed", 0.5)
                .with_display_value("+12%")
                .with_accessible_value("12 percent above the 12-week baseline"),
        ]);
        let cell = grid.cell(0, 0).expect("cell");

        assert_eq!(cell.visible_text().as_deref(), Some("+12%"));
        assert_eq!(
            cell.value_text(&texts),
            "12 percent above the 12-week baseline"
        );
    }

    #[test]
    fn without_a_complete_text_the_drawn_abbreviation_is_what_is_stated() {
        let texts = HeatmapTexts::default();
        let grid = office_kpi(vec![
            HeatmapValue::new("north", "closed", 0.5).with_display_value("+12%"),
        ]);

        assert_eq!(grid.cell(0, 0).expect("cell").value_text(&texts), "+12%");
    }

    #[test]
    fn a_gap_states_the_supplied_missing_copy_and_draws_nothing() {
        let texts = HeatmapTexts {
            missing_value: "Sin datos".to_string(),
            ..HeatmapTexts::default()
        };
        let grid = office_kpi(vec![]);
        let cell = grid.cell(0, 0).expect("cell");

        assert_eq!(cell.visible_text(), None);
        assert_eq!(cell.value_text(&texts), "Sin datos");
    }

    #[test]
    fn a_judged_cell_states_its_verdict_in_words_not_only_in_hue() {
        // The colour-only defect, closed: the same two cells differ by hue on
        // screen AND by a word for anyone who cannot use the hue.
        let texts = HeatmapTexts::default();
        let grid = office_kpi(vec![
            HeatmapValue::new("north", "closed", 0.6).with_display_value("+12%"),
            HeatmapValue::new("south", "closed", -0.6).with_display_value("-9%"),
        ]);

        assert_eq!(
            grid.cell(0, 0)
                .expect("cell")
                .stated_text(HeatScale::Judgement, &texts),
            "+12%, Favorable"
        );
        assert_eq!(
            grid.cell(1, 0)
                .expect("cell")
                .stated_text(HeatScale::Judgement, &texts),
            "-9%, Unfavorable"
        );
    }

    #[test]
    fn a_magnitude_cell_states_no_verdict_because_it_expresses_none() {
        let texts = HeatmapTexts::default();
        let grid = office_kpi(vec![
            HeatmapValue::new("north", "closed", 0.6).with_display_value("12%"),
        ]);

        assert_eq!(
            grid.cell(0, 0)
                .expect("cell")
                .stated_text(HeatScale::Magnitude, &texts),
            "12%"
        );
    }

    #[test]
    fn the_verdict_words_come_from_the_supplied_copy() {
        let texts = HeatmapTexts {
            sense_unfavorable: "Desfavorable".to_string(),
            ..HeatmapTexts::default()
        };
        let grid = office_kpi(vec![
            HeatmapValue::new("north", "closed", -0.6).with_display_value("-9%"),
        ]);

        assert_eq!(
            grid.cell(0, 0)
                .expect("cell")
                .stated_text(HeatScale::Judgement, &texts),
            "-9%, Desfavorable"
        );
    }
}
