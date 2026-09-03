use leptos::prelude::*;

/// The data model accepted by [`super::LineChart`].
#[derive(Clone, Debug, PartialEq)]
pub enum LineChartData {
    /// Numeric x/y points retained for compatibility with the original chart.
    XY(Vec<(f64, f64)>),
    /// Named categories and one or more series aligned to those categories.
    Categorical {
        /// Stable category identities and display labels.
        categories: Vec<LineCategory>,
        /// Series rendered across the categories.
        series: Vec<LineSeries>,
    },
}

/// A categorical x-axis item with a stable key and user-facing label.
#[derive(Clone, Debug, PartialEq)]
pub struct LineCategory {
    /// Stable identity for consumers and activation callbacks.
    pub key: String,
    /// Text presented on the x-axis and in chart affordances.
    pub label: String,
}

/// A named line and its display configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct LineSeries {
    /// Stable series identity for consumers and activation callbacks.
    pub id: String,
    /// User-facing series name.
    pub name: String,
    /// Values aligned with the input category order.
    pub points: Vec<LinePoint>,
    /// CSS color used for the series stroke and default markers.
    pub color: String,
    /// Dash pattern used for the series stroke.
    pub pattern: LinePattern,
    /// Marker configuration used for finite points.
    pub marker: MarkerStyle,
    /// Whether individual point labels are rendered.
    pub show_data_labels: bool,
    /// Where those labels sit relative to their markers (`ldui-raa7`).
    /// Defaults to [`LineLabelPlacement::Above`], so a chart that already
    /// draws labels is unchanged.
    pub label_placement: LineLabelPlacement,
    /// Which value axis this series is measured against; primary by default.
    pub axis: LineValueAxis,
}

/// Where a series' point labels sit relative to their markers (`ldui-raa7`).
///
/// Exists because label collision between two close series is real, not
/// hypothetical: the production chart this replaced put the value series'
/// labels ABOVE their nodes and the 12-week baseline's BELOW, with a source
/// comment saying exactly why. One series drawing both is unreadable the
/// moment the lines converge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineLabelPlacement {
    /// Above the marker. The default, and what every existing labelled chart
    /// already draws.
    #[default]
    Above,
    /// Below the marker, so a second series can be labelled without colliding
    /// with the first.
    Below,
}

/// Which value axis a categorical series is measured against.
///
/// [`LineValueAxis::Primary`] is the single left-hand axis every categorical
/// chart has always had, and is the default, so a series that never names an
/// axis keeps exactly the geometry, ticks, legend text, tooltip text and table
/// columns it had before a second axis existed. The right-hand axis is drawn
/// only when at least one series opts into it, which is why a single-axis
/// chart cannot grow a phantom second scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineValueAxis {
    /// The left-hand value axis. Every series' default.
    #[default]
    Primary,
    /// The right-hand value axis, with its own independent domain.
    Secondary,
}

/// Localized naming and value formatting for one value axis.
///
/// This is the single source for a unit: ticks, the hover card, the accessible
/// hidden table, the focus target's accessible name and the typed activation
/// payload all format an axis' values through it, so none of them can drift
/// from the others. A point's own `display_value` still wins where it is set —
/// that contract is unchanged.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LineAxisOptions {
    /// Axis title drawn beside its ticks and used to attribute a series in the
    /// legend and hidden table. Falls back to the matching [`LineChartTexts`]
    /// name when unset.
    pub label: Option<String>,
    /// Unit appended verbatim to every value this axis formats, so a caller
    /// controls whether a space precedes it. Examples: `"%"`, `" s"`.
    pub unit: Option<String>,
    /// Decimal places used for values this axis formats. Unset keeps the
    /// pre-existing rendering: shortest round-trip text for a value, one
    /// decimal for a tick.
    pub decimals: Option<usize>,
}

impl LineAxisOptions {
    /// Sets the axis title used by ticks, the legend and the hidden table.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the unit appended to every value this axis formats.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Sets the decimal places used for values this axis formats.
    pub fn with_decimals(mut self, decimals: usize) -> Self {
        self.decimals = Some(decimals);
        self
    }
}

/// Both value axes' options, resolved once per render.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct LineAxes {
    pub primary: LineAxisOptions,
    pub secondary: LineAxisOptions,
}

impl LineAxes {
    /// Returns the options belonging to `axis`.
    pub(super) fn options(&self, axis: LineValueAxis) -> &LineAxisOptions {
        match axis {
            LineValueAxis::Primary => &self.primary,
            LineValueAxis::Secondary => &self.secondary,
        }
    }
}

/// User-visible chart copy that is not supplied per series or per point.
///
/// The defaults reproduce the strings the chart emitted before this struct
/// existed, so adopting it changes nothing until a field is overridden.
#[derive(Clone, Debug, PartialEq)]
pub struct LineChartTexts {
    /// Header of the hidden table's category column.
    pub category_header: String,
    /// Hidden-table cell text for a category a series has no value at.
    pub no_value: String,
    /// Name used for the primary axis when it carries no label of its own.
    pub primary_axis: String,
    /// Name used for the secondary axis when it carries no label of its own.
    pub secondary_axis: String,
}

impl Default for LineChartTexts {
    fn default() -> Self {
        Self {
            category_header: "Category".to_string(),
            no_value: "No value".to_string(),
            primary_axis: "Primary axis".to_string(),
            secondary_axis: "Secondary axis".to_string(),
        }
    }
}

/// One optional series value and its display overrides.
#[derive(Clone, Debug, PartialEq)]
pub struct LinePoint {
    /// Numeric value, or `None` for a missing point.
    pub value: Option<f64>,
    /// Consumer-supplied value text for tooltips and accessible descriptions.
    pub display_value: Option<String>,
    /// Consumer-supplied label text drawn beside a marker.
    pub data_label: Option<String>,
    /// Optional CSS color that overrides this point's marker color.
    pub marker_color: Option<String>,
}

/// Stroke pattern for a categorical line.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum LinePattern {
    /// A continuous stroke.
    #[default]
    Solid,
    /// A dashed stroke.
    Dashed,
    /// A dotted stroke.
    Dotted,
    /// SVG dash lengths, normalized to a solid stroke when invalid.
    Custom(Vec<f64>),
}

/// Shape drawn for a categorical point marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MarkerShape {
    /// Do not draw a marker.
    None,
    /// A circular marker.
    #[default]
    Circle,
    /// A square marker.
    Square,
    /// A diamond marker.
    Diamond,
}

/// Appearance of a categorical point marker.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerStyle {
    /// Marker geometry.
    pub shape: MarkerShape,
    /// SVG view-box radius units; defaults to the finite value `4.0`.
    pub size: f64,
    /// Optional CSS fill that overrides the series color.
    pub fill: Option<String>,
    /// SVG stroke width in view-box units.
    pub stroke_width: f64,
}

impl Default for MarkerStyle {
    fn default() -> Self {
        Self {
            shape: MarkerShape::Circle,
            size: 4.0,
            fill: None,
            stroke_width: 1.0,
        }
    }
}

/// Determines when a categorical chart legend is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineLegendMode {
    /// Show a legend when categorical data has two or more series.
    #[default]
    Auto,
    /// Always show a legend.
    Always,
    /// Never show a legend.
    Never,
}

/// Determines whether categorical point interaction is enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineInteractionMode {
    /// Enable categorical interaction and preserve legacy XY behavior.
    #[default]
    Auto,
    /// Always enable categorical interaction.
    Enabled,
    /// Disable categorical interaction.
    Disabled,
}

/// Owned activation intent emitted by a categorical chart.
#[derive(Clone, Debug, PartialEq)]
pub struct LineChartActivation {
    /// Zero-based index of the activated category.
    pub category_index: usize,
    /// Stable key of the activated category.
    pub category_key: String,
    /// Display label of the activated category.
    pub category_label: String,
    /// Preferred series for the activation, when a finite point was selected.
    pub preferred_series_id: Option<String>,
    /// All finite values in the category, in series order.
    pub values: Vec<LineChartActivationValue>,
    /// Input method that triggered the activation.
    pub source: LineChartActivationSource,
    /// Modifier state captured with the activation.
    pub modifiers: LineChartModifiers,
}

/// One finite series value included in an activation intent.
#[derive(Clone, Debug, PartialEq)]
pub struct LineChartActivationValue {
    /// Stable series identity.
    pub series_id: String,
    /// User-facing series name.
    pub series_name: String,
    /// Finite numeric point value.
    pub value: f64,
    /// Consumer-supplied or fallback display text.
    pub display_value: String,
}

/// The input device that activated a chart category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineChartActivationSource {
    /// A pointer activated the category.
    #[default]
    Pointer,
    /// Keyboard input activated the category.
    Keyboard,
}

/// Keyboard modifier state accompanying an activation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LineChartModifiers {
    /// Whether Shift was held.
    pub shift: bool,
    /// Whether Control was held.
    pub ctrl: bool,
    /// Whether Alt was held.
    pub alt: bool,
    /// Whether Meta was held.
    pub meta: bool,
}

/// Static or reactive transport for [`LineChartData`].
#[derive(Clone, Debug, PartialEq)]
pub struct LineChartDataSource(LineChartDataSourceInner);

#[derive(Clone, Debug, PartialEq)]
enum LineChartDataSourceInner {
    Static(LineChartData),
    Reactive(Signal<LineChartData>),
}

impl LineChartDataSource {
    /// Returns the current data while tracking reactive sources.
    pub(crate) fn get(&self) -> LineChartData {
        match &self.0 {
            LineChartDataSourceInner::Static(data) => data.clone(),
            LineChartDataSourceInner::Reactive(data) => data.get(),
        }
    }
}

impl From<Vec<(f64, f64)>> for LineChartDataSource {
    fn from(data: Vec<(f64, f64)>) -> Self {
        Self(LineChartDataSourceInner::Static(LineChartData::XY(data)))
    }
}

impl From<LineChartData> for LineChartDataSource {
    fn from(data: LineChartData) -> Self {
        Self(LineChartDataSourceInner::Static(data))
    }
}

impl From<Signal<LineChartData>> for LineChartDataSource {
    fn from(data: Signal<LineChartData>) -> Self {
        Self(LineChartDataSourceInner::Reactive(data))
    }
}

impl From<RwSignal<LineChartData>> for LineChartDataSource {
    fn from(data: RwSignal<LineChartData>) -> Self {
        Self(LineChartDataSourceInner::Reactive(data.into()))
    }
}

impl LinePoint {
    /// Creates a finite point with no display overrides.
    pub fn new(value: f64) -> Self {
        Self {
            value: Some(value),
            display_value: None,
            data_label: None,
            marker_color: None,
        }
    }

    /// Creates a missing point that breaks a categorical line segment.
    pub fn missing() -> Self {
        Self {
            value: None,
            display_value: None,
            data_label: None,
            marker_color: None,
        }
    }

    /// Sets the tooltip and accessible display value.
    pub fn with_display_value(mut self, value: impl Into<String>) -> Self {
        self.display_value = Some(value.into());
        self
    }

    /// Sets the optional marker-adjacent label.
    pub fn with_data_label(mut self, label: impl Into<String>) -> Self {
        self.data_label = Some(label.into());
        self
    }
}

impl LineSeries {
    /// Creates a series with solid circular markers, no data labels, and the
    /// primary value axis.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        color: impl Into<String>,
        points: Vec<LinePoint>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            points,
            color: color.into(),
            pattern: LinePattern::Solid,
            marker: MarkerStyle::default(),
            show_data_labels: false,
            label_placement: LineLabelPlacement::default(),
            axis: LineValueAxis::Primary,
        }
    }

    /// Measures this series against `axis` instead of the primary one.
    pub fn with_axis(mut self, axis: LineValueAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Measures this series against the right-hand secondary value axis.
    pub fn on_secondary_axis(self) -> Self {
        self.with_axis(LineValueAxis::Secondary)
    }
}

impl LineChartData {
    /// Creates categorical chart data from aligned categories and series.
    pub fn categorical(categories: Vec<LineCategory>, series: Vec<LineSeries>) -> Self {
        Self::Categorical { categories, series }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_vec_becomes_static_xy_data() {
        let source = LineChartDataSource::from(vec![(0.0, 2.0), (1.0, 4.0)]);

        assert_eq!(
            source.get(),
            LineChartData::XY(vec![(0.0, 2.0), (1.0, 4.0)])
        );
    }

    #[test]
    fn static_and_reactive_data_sources_return_current_data() {
        let static_data = LineChartData::XY(vec![(2.0, 3.0)]);
        assert_eq!(
            LineChartDataSource::from(static_data.clone()).get(),
            static_data
        );

        let owner = Owner::new();
        owner.set();
        let reactive_data = RwSignal::new(LineChartData::XY(vec![(4.0, 5.0)]));
        let source = LineChartDataSource::from(reactive_data);
        assert_eq!(source.get(), LineChartData::XY(vec![(4.0, 5.0)]));

        reactive_data.set(LineChartData::XY(vec![(6.0, 7.0)]));
        assert_eq!(source.get(), LineChartData::XY(vec![(6.0, 7.0)]));
    }

    #[test]
    fn point_and_series_builders_apply_documented_defaults() {
        let point = LinePoint::new(12.5)
            .with_display_value("12.5%")
            .with_data_label("target");
        assert_eq!(point.value, Some(12.5));
        assert_eq!(point.display_value.as_deref(), Some("12.5%"));
        assert_eq!(point.data_label.as_deref(), Some("target"));
        assert_eq!(LinePoint::missing().value, None);

        let series = LineSeries::new("closed", "Closed", "var(--color-primary)", vec![point]);
        assert_eq!(series.pattern, LinePattern::Solid);
        assert_eq!(series.marker, MarkerStyle::default());
        assert!(!series.show_data_labels);
        assert_eq!(series.marker.size, 4.0);
        assert_eq!(series.marker.shape, MarkerShape::Circle);
    }

    #[test]
    fn a_series_is_measured_against_the_primary_axis_unless_it_opts_out() {
        let series = LineSeries::new("closed", "Closed", "blue", vec![LinePoint::new(1.0)]);

        assert_eq!(
            series.axis,
            LineValueAxis::Primary,
            "the constructor must not move an existing caller onto a new axis"
        );
        assert_eq!(LineValueAxis::default(), LineValueAxis::Primary);
        assert_eq!(
            series.clone().on_secondary_axis().axis,
            LineValueAxis::Secondary
        );
        assert_eq!(
            series.with_axis(LineValueAxis::Secondary).axis,
            LineValueAxis::Secondary
        );
    }

    #[test]
    fn axis_options_default_to_naming_and_formatting_nothing() {
        let options = LineAxisOptions::default();

        assert_eq!(options.label, None);
        assert_eq!(options.unit, None);
        assert_eq!(options.decimals, None);

        let configured = LineAxisOptions::default()
            .with_label("Average first response")
            .with_unit(" s")
            .with_decimals(1);
        assert_eq!(configured.label.as_deref(), Some("Average first response"));
        assert_eq!(configured.unit.as_deref(), Some(" s"));
        assert_eq!(configured.decimals, Some(1));
    }

    #[test]
    fn axes_resolve_options_per_axis() {
        let axes = LineAxes {
            primary: LineAxisOptions::default().with_unit(" cases"),
            secondary: LineAxisOptions::default().with_unit(" s"),
        };

        assert_eq!(
            axes.options(LineValueAxis::Primary).unit.as_deref(),
            Some(" cases")
        );
        assert_eq!(
            axes.options(LineValueAxis::Secondary).unit.as_deref(),
            Some(" s")
        );
    }

    #[test]
    fn chart_texts_default_to_the_strings_the_chart_already_emitted() {
        let texts = LineChartTexts::default();

        assert_eq!(texts.category_header, "Category");
        assert_eq!(texts.no_value, "No value");
        assert_eq!(texts.primary_axis, "Primary axis");
        assert_eq!(texts.secondary_axis, "Secondary axis");
    }

    #[test]
    fn categorical_builder_keeps_categories_and_series() {
        let categories = vec![LineCategory {
            key: "2026-w01".to_string(),
            label: "Week 1".to_string(),
        }];
        let series = vec![LineSeries::new(
            "closed",
            "Closed",
            "blue",
            vec![LinePoint::new(4.0)],
        )];

        assert_eq!(
            LineChartData::categorical(categories.clone(), series.clone()),
            LineChartData::Categorical { categories, series }
        );
    }

    #[test]
    fn fieldless_enums_have_their_documented_defaults() {
        assert_eq!(LinePattern::default(), LinePattern::Solid);
        assert_eq!(MarkerShape::default(), MarkerShape::Circle);
        assert_eq!(LineLegendMode::default(), LineLegendMode::Auto);
        assert_eq!(LineInteractionMode::default(), LineInteractionMode::Auto);
        assert_eq!(
            LineChartActivationSource::default(),
            LineChartActivationSource::Pointer
        );
    }
}

#[cfg(test)]
mod label_placement_tests {
    use super::{LineLabelPlacement, LinePoint, LineSeries};

    /// `ldui-raa7`: a labelled chart that never names a placement must keep
    /// drawing exactly where it always did.
    #[test]
    fn placement_defaults_to_above() {
        assert_eq!(LineLabelPlacement::default(), LineLabelPlacement::Above);
        let series = LineSeries::new("s", "S", "var(--color-primary)", Vec::new());
        assert_eq!(series.label_placement, LineLabelPlacement::Above);
        assert!(
            !series.show_data_labels,
            "labels stay opt-in; this bead adds placement, not labels-by-default"
        );
    }

    /// The two placements must be distinguishable, since the whole reason the
    /// enum exists is that one series draws above and another below so their
    /// labels cannot collide.
    #[test]
    fn above_and_below_are_distinct() {
        assert_ne!(LineLabelPlacement::Above, LineLabelPlacement::Below);
    }

    /// The no-math contract: the renderer falls back to `display_value` when
    /// no explicit `data_label` was given, but it can never invent one.
    /// A point with neither stays unlabelled rather than showing a number the
    /// server did not send.
    #[test]
    fn a_point_without_either_string_has_nothing_to_draw() {
        let bare = LinePoint::new(42.0);
        assert!(bare.data_label.is_none());
        assert!(bare.display_value.is_none());

        // display_value alone is enough -- that is the discoverability fix.
        let from_display = LinePoint::new(42.0).with_display_value("42 resolved");
        assert!(from_display.data_label.is_none());
        assert_eq!(from_display.display_value.as_deref(), Some("42 resolved"));

        // An explicit data_label still wins over display_value.
        let explicit = LinePoint::new(42.0)
            .with_display_value("42 resolved")
            .with_data_label("42");
        assert_eq!(explicit.data_label.as_deref(), Some("42"));
        assert_eq!(explicit.display_value.as_deref(), Some("42 resolved"));
    }
}
