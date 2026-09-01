use leptos::prelude::*;

use super::scale::HeatmapSense;

/// A single populated cell within a positional heatmap grid.
///
/// This is the chart's **original** surface, kept exactly as it was: a cell is
/// addressed by its array position in `row_labels` / `col_labels`. It is
/// preserved so every existing caller keeps compiling and rendering
/// identically, but new code should reach for [`HeatmapValue`] inside a
/// [`HeatmapMatrix`], where a cell is addressed by stable row and column keys
/// that survive a sort, a filter or a wholesale data replacement.
#[derive(Clone, Debug, PartialEq)]
pub struct HeatmapCell {
    /// Row index (0-based), matched against `row_labels` by position.
    pub row: usize,
    /// Column index (0-based), matched against `col_labels` by position.
    pub col: usize,
    /// Text rendered centered inside the cell.
    pub label: String,
    /// Cell intensity. Its meaning depends on the heatmap's `HeatScale`:
    ///
    /// - `HeatScale::Magnitude` (default): `0.0..=1.0`, mapped to fill alpha
    ///   (capped at 0.55) over the single `rgb` hue. Negative values clamp to 0.
    /// - `HeatScale::Judgement`: `-1.0..=1.0`. The SIGN picks the hue
    ///   (positive = favorable, negative = unfavorable) and the MAGNITUDE picks
    ///   the alpha over the same ramp.
    ///
    /// Callers normalize before passing cells in — this component only applies
    /// the linear alpha mapping and the hue choice.
    pub intensity: f64,
}

/// One axis entry of a typed heatmap: a row or a column.
///
/// The key is the identity the chart reports and reconciles focus by; the label
/// is the localized text a reader sees. Keeping them in one value is the point:
/// the alternative — a `Vec<String>` of labels beside a parallel `Vec<String>`
/// of ids — lets a caller sort one and not the other, which silently re-points
/// every cell at a neighbouring office with no error anywhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeatmapCategory {
    /// Stable identity. Emitted by activations, written to `data-row-key` /
    /// `data-column-key`, and used to follow focus across a reorder. Never an
    /// array index.
    pub key: String,
    /// Localized display text, drawn on the axis and read in the data table.
    pub label: String,
}

impl HeatmapCategory {
    /// Creates a category from its stable key and its localized label.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// One populated cell of a typed heatmap, addressed by stable keys.
///
/// Everything about a cell travels in one value: which row and column it
/// belongs to, its signed intensity, the short text drawn inside it, and the
/// complete localized text a screen reader should hear instead of that
/// abbreviation. A `(row, col)` pair carrying no [`HeatmapValue`] is a
/// **missing** cell, stated as missing rather than drawn as a zero.
#[derive(Clone, Debug, PartialEq)]
pub struct HeatmapValue {
    /// Key of the row this cell belongs to. A key not present in the matrix's
    /// rows is dropped rather than silently landing in the wrong row.
    pub row_key: String,
    /// Key of the column this cell belongs to.
    pub column_key: String,
    /// Signed intensity, or `None` for a cell with no measurement. A
    /// non-finite intensity supplied here normalizes to `None` rather than
    /// becoming a fabricated zero or an unparseable fill.
    pub intensity: Option<f64>,
    /// Short text drawn centered in the cell — the abbreviation a sighted
    /// reader sees, e.g. `"+12%"`. Optional: a cell may be colour only.
    pub display_value: Option<String>,
    /// The complete localized value text a screen reader hears, e.g.
    /// `"12 percent above the 12-week baseline"`. It replaces
    /// `display_value` in the data table, in the cell's accessible name and
    /// in the activation payload — everywhere the value is *stated* rather
    /// than *drawn* — so an abbreviation that only makes sense next to a
    /// column header never has to serve as the whole reading.
    pub accessible_value: Option<String>,
}

impl HeatmapValue {
    /// Creates a measured cell with the given signed intensity.
    pub fn new(row_key: impl Into<String>, column_key: impl Into<String>, intensity: f64) -> Self {
        Self {
            row_key: row_key.into(),
            column_key: column_key.into(),
            intensity: Some(intensity),
            display_value: None,
            accessible_value: None,
        }
    }

    /// Creates a cell with no measurement. It draws no tile and is stated as
    /// missing wherever it is read — never as zero.
    pub fn missing(row_key: impl Into<String>, column_key: impl Into<String>) -> Self {
        Self {
            row_key: row_key.into(),
            column_key: column_key.into(),
            intensity: None,
            display_value: None,
            accessible_value: None,
        }
    }

    /// Sets the short text drawn inside the cell.
    pub fn with_display_value(mut self, display_value: impl Into<String>) -> Self {
        self.display_value = Some(display_value.into());
        self
    }

    /// Sets the complete localized text a screen reader hears for this cell.
    pub fn with_accessible_value(mut self, accessible_value: impl Into<String>) -> Self {
        self.accessible_value = Some(accessible_value.into());
        self
    }
}

/// A typed heatmap: its row axis, its column axis, and the cells that are
/// populated.
///
/// The two axes are ordered — top-to-bottom and left-to-right — and the values
/// are unordered, because a value names the row and column it belongs to. That
/// is what lets a caller re-sort either axis without touching the values, and
/// what makes a missing `(row, column)` combination simply an absent value
/// rather than a hole a parallel array has to encode.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeatmapMatrix {
    /// Row axis, top to bottom.
    pub rows: Vec<HeatmapCategory>,
    /// Column axis, left to right.
    pub columns: Vec<HeatmapCategory>,
    /// The populated cells, in any order.
    pub values: Vec<HeatmapValue>,
}

impl HeatmapMatrix {
    /// Creates a matrix from its two axes and its populated cells.
    pub fn new(
        rows: Vec<HeatmapCategory>,
        columns: Vec<HeatmapCategory>,
        values: Vec<HeatmapValue>,
    ) -> Self {
        Self {
            rows,
            columns,
            values,
        }
    }
}

/// Static or reactive transport for a [`HeatmapMatrix`], plus the "no typed
/// data at all" state that selects the legacy positional render.
///
/// [`HeatmapDataSource::default`] is that absent state, which is what makes the
/// component's `data` prop optional without an `Option` in the signature: a
/// caller that never mentions `data` gets exactly the chart it always had, and
/// an *empty* typed matrix is a different thing entirely — it renders the
/// localized no-data copy.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeatmapDataSource(HeatmapDataSourceInner);

#[derive(Clone, Debug, Default, PartialEq)]
enum HeatmapDataSourceInner {
    /// No typed data was supplied; the positional props render instead.
    #[default]
    Absent,
    Static(HeatmapMatrix),
    Reactive(Signal<HeatmapMatrix>),
}

impl HeatmapDataSource {
    /// Whether no typed data was supplied, so the legacy positional props are
    /// what should render.
    pub(super) fn is_absent(&self) -> bool {
        matches!(self.0, HeatmapDataSourceInner::Absent)
    }

    /// Returns the current matrix while tracking reactive sources.
    pub(super) fn get(&self) -> HeatmapMatrix {
        match &self.0 {
            HeatmapDataSourceInner::Absent => HeatmapMatrix::default(),
            HeatmapDataSourceInner::Static(matrix) => matrix.clone(),
            HeatmapDataSourceInner::Reactive(matrix) => matrix.get(),
        }
    }
}

impl From<HeatmapMatrix> for HeatmapDataSource {
    fn from(matrix: HeatmapMatrix) -> Self {
        Self(HeatmapDataSourceInner::Static(matrix))
    }
}

impl From<Signal<HeatmapMatrix>> for HeatmapDataSource {
    fn from(matrix: Signal<HeatmapMatrix>) -> Self {
        Self(HeatmapDataSourceInner::Reactive(matrix))
    }
}

impl From<RwSignal<HeatmapMatrix>> for HeatmapDataSource {
    fn from(matrix: RwSignal<HeatmapMatrix>) -> Self {
        Self(HeatmapDataSourceInner::Reactive(matrix.into()))
    }
}

impl From<Memo<HeatmapMatrix>> for HeatmapDataSource {
    fn from(matrix: Memo<HeatmapMatrix>) -> Self {
        Self(HeatmapDataSourceInner::Reactive(matrix.into()))
    }
}

/// Determines whether a typed heatmap's cells are navigable and activatable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeatmapInteractionMode {
    /// Interactive exactly when an activation callback is wired. A heatmap
    /// that only describes itself gains no tab stops at all.
    ///
    /// This is deliberately stricter than `BarChart`'s equivalent default,
    /// which makes typed bars navigable whether or not anything listens. A
    /// matrix has rows times columns cells rather than a handful, and its
    /// complete non-visual truth is already in the data table, so putting a
    /// tab stop into a purely descriptive grid buys a keyboard reader nothing
    /// and costs them a stop to escape from.
    #[default]
    Auto,
    /// Always navigable, even with no callback: arrow-key exploration of the
    /// grid with no button semantics and nothing to press.
    Enabled,
    /// Never navigable. No roles, no tab stops, no listeners.
    Disabled,
}

/// Every user-visible string a heatmap produces itself.
///
/// The defaults reproduce what the chart emitted before this struct existed —
/// including the previously hard-coded English `"No data"` — so adopting it
/// changes nothing until a field is overridden. Supplied as a `Signal`, so a
/// locale change re-renders the copy without touching keys, intensities, order,
/// focus or the identity an activation reports.
#[derive(Clone, Debug, PartialEq)]
pub struct HeatmapTexts {
    /// Drawn in place of the grid when there is nothing to plot.
    pub no_data: String,
    /// The accessible data table's caption.
    pub data_table_caption: String,
    /// Name of the ROW AXIS — `"Office"`, not a row's own label. Heads the
    /// column of row labels in the data table, and prefixes the row in a
    /// cell's accessible name.
    pub row_header: String,
    /// Name of the COLUMN AXIS — `"KPI"`, not a column's own label. Prefixes
    /// the column in a cell's accessible name.
    pub column_header: String,
    /// Name of the measure the cells carry — `"Value"`. Prefixes the value in
    /// a cell's accessible name.
    pub value_header: String,
    /// Stated wherever a `(row, column)` combination carries no measurement,
    /// so a gap reads as a gap rather than as a zero.
    pub missing_value: String,
    /// Reader-facing name of `HeatmapSense::Favorable`.
    pub sense_favorable: String,
    /// Reader-facing name of `HeatmapSense::Unfavorable`.
    pub sense_unfavorable: String,
    /// Reader-facing name of `HeatmapSense::Neutral`. Never appended to a
    /// cell's reading — a neutral cell simply carries no verdict — but it is
    /// here so a caller localizing the set does not have to guess which of the
    /// three the framework will use.
    pub sense_neutral: String,
}

impl Default for HeatmapTexts {
    fn default() -> Self {
        Self {
            no_data: "No data".to_string(),
            data_table_caption: "Heatmap data".to_string(),
            row_header: "Row".to_string(),
            column_header: "Column".to_string(),
            value_header: "Value".to_string(),
            missing_value: "No value".to_string(),
            sense_favorable: "Favorable".to_string(),
            sense_unfavorable: "Unfavorable".to_string(),
            sense_neutral: "Neutral".to_string(),
        }
    }
}

impl HeatmapTexts {
    /// The reader-facing name of `sense`.
    pub(super) fn sense_text(&self, sense: HeatmapSense) -> &str {
        match sense {
            HeatmapSense::Neutral => &self.sense_neutral,
            HeatmapSense::Favorable => &self.sense_favorable,
            HeatmapSense::Unfavorable => &self.sense_unfavorable,
        }
    }
}

/// Owned activation intent emitted by a typed heatmap cell.
///
/// There is deliberately **no row index and no column index**. An index
/// re-points at a different office or a different KPI the moment either axis is
/// sorted, filtered or replaced, so a host that stored one and acted on it
/// later would drill into the wrong thing. A cell's identity is the PAIR of
/// stable keys — `row_key` and `column_key` — exactly as `ldui-nz6d` /
/// `ldui-px06` established for tables and `ldui-y2ed` for the bar chart.
#[derive(Clone, Debug, PartialEq)]
pub struct HeatmapActivation {
    /// Stable key of the activated cell's row.
    pub row_key: String,
    /// Localized label of the activated cell's row.
    pub row_label: String,
    /// Stable key of the activated cell's column.
    pub column_key: String,
    /// Localized label of the activated cell's column.
    pub column_label: String,
    /// The cell's finite signed intensity, or `None` when the cell carries no
    /// measurement. Every grid position is activatable — a reader drills into
    /// an Office by KPI coordinate whether or not a number was reported for it
    /// — so this is an `Option` rather than a fabricated zero.
    pub intensity: Option<f64>,
    /// The value exactly as the chart states it: the cell's
    /// `accessible_value` when it has one, else its `display_value`, else the
    /// localized missing-value copy.
    pub display_value: String,
    /// The judgement the cell's colour carries, in the same terms the data
    /// table states it.
    pub sense: HeatmapSense,
    /// Input method that triggered the activation.
    pub source: HeatmapActivationSource,
    /// Modifier state captured with the activation.
    pub modifiers: HeatmapModifiers,
}

/// The input device that activated a heatmap cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeatmapActivationSource {
    /// A pointer activated the cell.
    #[default]
    Pointer,
    /// Keyboard input activated the cell.
    Keyboard,
}

/// Keyboard modifier state accompanying an activation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeatmapModifiers {
    /// Whether Shift was held.
    pub shift: bool,
    /// Whether Control was held.
    pub ctrl: bool,
    /// Whether Alt was held.
    pub alt: bool,
    /// Whether Meta was held.
    pub meta: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_category_bundles_the_identity_with_the_text_so_neither_can_misalign() {
        // The defect this type replaces: labels arrived as one Vec<String> and
        // ids (when a consumer kept any) as a second, parallel one. Sorting
        // the first without the second re-points every cell, silently.
        let category = HeatmapCategory::new("office-north", "North");

        assert_eq!(category.key, "office-north");
        assert_eq!(category.label, "North");
    }

    #[test]
    fn a_value_is_addressed_by_keys_rather_than_by_position() {
        let value = HeatmapValue::new("north", "closed", 0.6)
            .with_display_value("+12%")
            .with_accessible_value("12 percent above baseline");

        assert_eq!(value.row_key, "north");
        assert_eq!(value.column_key, "closed");
        assert_eq!(value.intensity, Some(0.6));
        assert_eq!(value.display_value.as_deref(), Some("+12%"));
        assert_eq!(
            value.accessible_value.as_deref(),
            Some("12 percent above baseline")
        );
    }

    #[test]
    fn a_missing_value_is_missing_rather_than_zero() {
        let value = HeatmapValue::missing("north", "closed");

        assert_eq!(value.intensity, None);
        assert_eq!(value.display_value, None);
        assert_eq!(value.accessible_value, None);
    }

    #[test]
    fn an_absent_data_source_is_distinguishable_from_an_empty_matrix() {
        // The whole backward-compatibility hinge: "the caller passed no typed
        // data, render the positional props" and "the caller passed a matrix
        // with nothing in it, render the localized no-data copy" are different
        // states and must not collapse into one.
        assert!(HeatmapDataSource::default().is_absent());

        let empty = HeatmapDataSource::from(HeatmapMatrix::default());
        assert!(!empty.is_absent());
        assert_eq!(empty.get(), HeatmapMatrix::default());
    }

    #[test]
    fn static_and_reactive_data_sources_return_the_current_matrix() {
        let matrix = HeatmapMatrix::new(
            vec![HeatmapCategory::new("north", "North")],
            vec![HeatmapCategory::new("closed", "Closed")],
            vec![HeatmapValue::new("north", "closed", 0.5)],
        );
        assert_eq!(HeatmapDataSource::from(matrix.clone()).get(), matrix);

        let owner = Owner::new();
        owner.set();
        let reactive = RwSignal::new(matrix.clone());
        let source = HeatmapDataSource::from(reactive);
        assert_eq!(source.get(), matrix);

        let replaced = HeatmapMatrix::new(
            vec![HeatmapCategory::new("south", "South")],
            vec![HeatmapCategory::new("sla", "SLA met")],
            vec![],
        );
        reactive.set(replaced.clone());
        assert_eq!(source.get(), replaced);
    }

    #[test]
    fn chart_texts_default_to_the_strings_the_chart_already_emitted() {
        let texts = HeatmapTexts::default();

        assert_eq!(
            texts.no_data, "No data",
            "the previously hard-coded empty string must stay the default"
        );
        assert_eq!(texts.data_table_caption, "Heatmap data");
        assert_eq!(texts.row_header, "Row");
        assert_eq!(texts.column_header, "Column");
        assert_eq!(texts.value_header, "Value");
        assert_eq!(texts.missing_value, "No value");
    }

    #[test]
    fn sense_text_reads_from_the_supplied_copy_rather_than_a_hardcoded_name() {
        let texts = HeatmapTexts {
            sense_favorable: "Favorable".to_string(),
            sense_unfavorable: "Desfavorable".to_string(),
            sense_neutral: "Neutral".to_string(),
            ..HeatmapTexts::default()
        };

        assert_eq!(texts.sense_text(HeatmapSense::Unfavorable), "Desfavorable");
        assert_eq!(texts.sense_text(HeatmapSense::Favorable), "Favorable");
        assert_eq!(texts.sense_text(HeatmapSense::Neutral), "Neutral");
    }

    #[test]
    fn fieldless_enums_have_their_documented_defaults() {
        assert_eq!(
            HeatmapInteractionMode::default(),
            HeatmapInteractionMode::Auto
        );
        assert_eq!(
            HeatmapActivationSource::default(),
            HeatmapActivationSource::Pointer
        );
        assert_eq!(HeatmapModifiers::default(), HeatmapModifiers::default());
    }
}
