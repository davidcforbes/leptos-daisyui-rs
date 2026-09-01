use leptos::prelude::*;

/// The data model accepted by [`super::BarChart`].
#[derive(Clone, Debug, PartialEq)]
pub enum BarChartData {
    /// Positional `(label, value)` pairs, retained exactly as the chart's
    /// original surface. Colour for these bars still comes from the positional
    /// `bar_colors` prop, whose mismatch-safe behaviour is unchanged.
    Simple(Vec<(String, f64)>),
    /// Typed items, each carrying its own key, label, value, formatted display
    /// text, semantic status and optional colour in ONE value.
    Categorical(Vec<BarChartItem>),
}

impl BarChartData {
    /// Creates typed categorical data.
    pub fn categorical(items: Vec<BarChartItem>) -> Self {
        Self::Categorical(items)
    }
}

/// One category of a typed bar chart.
///
/// Everything about a bar lives here: its stable identity, its localized
/// label, its signed value, the caller's formatted display text, its
/// caller-owned semantic status and — when the caller wants one — its explicit
/// colour. That is the point of the type. The chart's original per-bar colour
/// surface is a *second positional vector* (`bar_colors`), so sorting the data
/// without sorting the colours repaints every bar with a neighbour's
/// judgement, silently and with no error. A colour that travels inside the item
/// cannot be separated from the value it describes by any reordering,
/// filtering or truncation the caller does.
#[derive(Clone, Debug, PartialEq)]
pub struct BarChartItem {
    /// Stable identity emitted by activations and used to reconcile focus
    /// across reorders and removals. Never an array index.
    pub key: String,
    /// Localized text drawn beside the bar and read in the accessible table.
    pub label: String,
    /// Signed value, or `None` for a missing measurement. A non-finite value
    /// supplied here is normalized to `None` rather than becoming a fabricated
    /// zero.
    pub value: Option<f64>,
    /// Caller-formatted value text. Wins over the chart's own formatter
    /// wherever a value is stated.
    pub display_value: Option<String>,
    /// Caller-owned judgement. The framework never infers one: for a limit
    /// measure "up" is bad and for a throughput measure "up" is good, and only
    /// the caller knows which this is.
    pub status: BarStatus,
    /// Explicit CSS colour for this bar, overriding the status colour and the
    /// chart-wide colour.
    pub color: Option<String>,
}

impl BarChartItem {
    /// Creates a finite, neutral item with no display overrides.
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: f64) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: Some(value),
            display_value: None,
            status: BarStatus::Neutral,
            color: None,
        }
    }

    /// Creates an item with no measurement. It is drawn as no bar at all and
    /// is stated as missing in the accessible table — never as zero.
    pub fn missing(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: None,
            display_value: None,
            status: BarStatus::Neutral,
            color: None,
        }
    }

    /// Sets the caller-formatted value text.
    pub fn with_display_value(mut self, display_value: impl Into<String>) -> Self {
        self.display_value = Some(display_value.into());
        self
    }

    /// Sets the caller-owned semantic status.
    pub fn with_status(mut self, status: BarStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets an explicit colour, overriding the status and chart-wide colours.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// A caller-owned judgement about one bar.
///
/// [`BarStatus::Neutral`] is the default and paints with the chart-wide colour,
/// so an activity measure — a count of things that happened, where neither
/// direction is good or bad — needs no status at all and looks exactly as it
/// always did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarStatus {
    /// No judgement. The chart-wide colour applies.
    #[default]
    Neutral,
    /// The caller considers this value good.
    Favorable,
    /// The caller considers this value bad.
    Unfavorable,
}

impl BarStatus {
    /// The machine-readable token written to `data-status`, so a browser test
    /// locates a bar's judgement by identity rather than by its colour.
    pub(super) fn token(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Favorable => "favorable",
            Self::Unfavorable => "unfavorable",
        }
    }
}

/// How the bars are laid out against the value axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarChartLayout {
    /// Follows the legacy `horizontal` prop: vertical bars unless
    /// `horizontal=true`. This is the default, so no existing caller's layout
    /// moves.
    #[default]
    Auto,
    /// Vertical bars growing up (or down) from the zero line.
    Vertical,
    /// Horizontal bars growing right (or left) from the zero line, with the
    /// category labels in a left gutter.
    Horizontal,
    /// Horizontal bars with the zero rule always drawn, even when every value
    /// shares a sign. This is the mode for a signed decomposition — a
    /// current-minus-baseline delta per office, sorted most-dragging-first —
    /// where the reader is comparing direction as much as magnitude and the
    /// baseline must stay visible in every filtering of the data.
    DivergingHorizontal,
}

impl BarChartLayout {
    /// Resolves [`BarChartLayout::Auto`] against the legacy `horizontal` prop.
    pub(super) fn resolve(self, horizontal: bool) -> Self {
        match self {
            Self::Auto if horizontal => Self::Horizontal,
            Self::Auto => Self::Vertical,
            other => other,
        }
    }

    /// Whether bars run along the x axis.
    pub(super) fn is_horizontal(self) -> bool {
        matches!(self, Self::Horizontal | Self::DivergingHorizontal)
    }

    /// The machine-readable token written to `data-bar-chart-layout`.
    pub(super) fn token(self) -> &'static str {
        match self {
            Self::Auto | Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
            Self::DivergingHorizontal => "diverging-horizontal",
        }
    }
}

/// Determines whether typed bars are interactive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarInteractionMode {
    /// Interactive for typed categorical data; never for the legacy surface.
    #[default]
    Auto,
    /// Always interactive for typed categorical data.
    Enabled,
    /// Never interactive. A purely descriptive chart gains no tab stops.
    Disabled,
}

/// Numeric formatting for every value this chart states.
///
/// The ticks, the drawn value labels, the accessible names, the hidden table
/// and the typed activation payload all format through this, so a unit written
/// once cannot end up on the ticks and missing from the table. An item's own
/// [`BarChartItem::display_value`] still wins wherever it is set.
///
/// Unset `decimals` keeps the one-decimal rendering the chart has always
/// emitted, so adopting the struct changes no existing caller's numbers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BarValueFormat {
    /// Appended verbatim to every formatted value, so the caller owns the
    /// separator: `"%"` and `" cases"` are both correct for their locale.
    pub unit: Option<String>,
    /// Decimal places. Unset means one decimal, the chart's original output.
    pub decimals: Option<usize>,
}

impl BarValueFormat {
    /// Sets the unit appended to every formatted value.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Sets the decimal places used for every formatted value.
    pub fn with_decimals(mut self, decimals: usize) -> Self {
        self.decimals = Some(decimals);
        self
    }
}

/// Every user-visible string the chart produces itself.
///
/// The defaults reproduce what the chart emitted before this struct existed —
/// including the previously hard-coded English `"No data"` — so adopting it
/// changes nothing until a field is overridden. Supplied as a `Signal`, so a
/// locale change re-renders the copy without touching keys, values, order,
/// focus or selection.
#[derive(Clone, Debug, PartialEq)]
pub struct BarChartTexts {
    /// Drawn in place of the chart when there is nothing to plot.
    pub empty: String,
    /// Header of the accessible table's category column.
    pub category_header: String,
    /// Header of the accessible table's value column.
    pub value_header: String,
    /// Header of the accessible table's status column.
    pub status_header: String,
    /// Cell text and accessible-name text for an item with no measurement.
    pub no_value: String,
    /// Reader-facing name of [`BarStatus::Neutral`].
    pub status_neutral: String,
    /// Reader-facing name of [`BarStatus::Favorable`].
    pub status_favorable: String,
    /// Reader-facing name of [`BarStatus::Unfavorable`].
    pub status_unfavorable: String,
}

impl Default for BarChartTexts {
    fn default() -> Self {
        Self {
            empty: "No data".to_string(),
            category_header: "Category".to_string(),
            value_header: "Value".to_string(),
            status_header: "Status".to_string(),
            no_value: "No value".to_string(),
            status_neutral: "Neutral".to_string(),
            status_favorable: "Favorable".to_string(),
            status_unfavorable: "Unfavorable".to_string(),
        }
    }
}

impl BarChartTexts {
    /// The reader-facing name of `status`.
    pub(super) fn status_text(&self, status: BarStatus) -> &str {
        match status {
            BarStatus::Neutral => &self.status_neutral,
            BarStatus::Favorable => &self.status_favorable,
            BarStatus::Unfavorable => &self.status_unfavorable,
        }
    }
}

/// Owned activation intent emitted by a typed bar chart.
///
/// There is deliberately **no index field**. An index re-points at a different
/// category the moment the data is sorted, filtered or replaced, so a host that
/// stored one and acted on it later would act on the wrong office. The key is
/// the identity, exactly as `ldui-nz6d`/`ldui-px06` established for tables.
#[derive(Clone, Debug, PartialEq)]
pub struct BarChartActivation {
    /// Stable key of the activated item.
    pub category_key: String,
    /// Localized label of the activated item.
    pub category_label: String,
    /// The item's finite signed value. A missing or non-finite item is never
    /// activatable, so this can never be NaN, infinite or a fabricated zero.
    pub value: f64,
    /// The item's display text — its own `display_value` when it has one, else
    /// the chart's formatting of `value`.
    pub display_value: String,
    /// The caller-owned status the item carried.
    pub status: BarStatus,
    /// Input method that triggered the activation.
    pub source: BarChartActivationSource,
    /// Modifier state captured with the activation.
    pub modifiers: BarChartModifiers,
}

/// The input device that activated a bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarChartActivationSource {
    /// A pointer activated the bar.
    #[default]
    Pointer,
    /// Keyboard input activated the bar.
    Keyboard,
}

/// Keyboard modifier state accompanying an activation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BarChartModifiers {
    /// Whether Shift was held.
    pub shift: bool,
    /// Whether Control was held.
    pub ctrl: bool,
    /// Whether Alt was held.
    pub alt: bool,
    /// Whether Meta was held.
    pub meta: bool,
}

/// Static or reactive transport for [`BarChartData`].
///
/// The `From` impls are what keep every existing caller source-compatible: a
/// bare `Vec<(String, f64)>` still converts, so `data=closed_by_week()`
/// compiles and renders unchanged.
#[derive(Clone, Debug, PartialEq)]
pub struct BarChartDataSource(BarChartDataSourceInner);

#[derive(Clone, Debug, PartialEq)]
enum BarChartDataSourceInner {
    Static(BarChartData),
    Reactive(Signal<BarChartData>),
}

impl BarChartDataSource {
    /// Returns the current data while tracking reactive sources.
    pub(crate) fn get(&self) -> BarChartData {
        match &self.0 {
            BarChartDataSourceInner::Static(data) => data.clone(),
            BarChartDataSourceInner::Reactive(data) => data.get(),
        }
    }
}

impl From<Vec<(String, f64)>> for BarChartDataSource {
    fn from(data: Vec<(String, f64)>) -> Self {
        Self(BarChartDataSourceInner::Static(BarChartData::Simple(data)))
    }
}

impl From<Vec<BarChartItem>> for BarChartDataSource {
    fn from(items: Vec<BarChartItem>) -> Self {
        Self(BarChartDataSourceInner::Static(BarChartData::Categorical(
            items,
        )))
    }
}

impl From<BarChartData> for BarChartDataSource {
    fn from(data: BarChartData) -> Self {
        Self(BarChartDataSourceInner::Static(data))
    }
}

impl From<Signal<BarChartData>> for BarChartDataSource {
    fn from(data: Signal<BarChartData>) -> Self {
        Self(BarChartDataSourceInner::Reactive(data))
    }
}

impl From<RwSignal<BarChartData>> for BarChartDataSource {
    fn from(data: RwSignal<BarChartData>) -> Self {
        Self(BarChartDataSourceInner::Reactive(data.into()))
    }
}

impl From<Memo<BarChartData>> for BarChartDataSource {
    fn from(data: Memo<BarChartData>) -> Self {
        Self(BarChartDataSourceInner::Reactive(data.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_legacy_label_value_vector_still_converts_into_the_data_source() {
        // The compatibility contract in one assertion: the exact type every
        // existing caller passes is still accepted, and still means the
        // original positional surface rather than the typed one.
        let source =
            BarChartDataSource::from(vec![("Mon".to_string(), 4.0), ("Tue".to_string(), 7.0)]);

        assert_eq!(
            source.get(),
            BarChartData::Simple(vec![("Mon".to_string(), 4.0), ("Tue".to_string(), 7.0)])
        );
    }

    #[test]
    fn a_typed_item_vector_converts_straight_into_categorical_data() {
        let items = vec![BarChartItem::new("north", "North", -4.0)];

        assert_eq!(
            BarChartDataSource::from(items.clone()).get(),
            BarChartData::Categorical(items)
        );
    }

    #[test]
    fn static_and_reactive_data_sources_return_current_data() {
        let static_data = BarChartData::Simple(vec![("a".to_string(), 1.0)]);
        assert_eq!(
            BarChartDataSource::from(static_data.clone()).get(),
            static_data
        );

        let owner = Owner::new();
        owner.set();
        let reactive = RwSignal::new(BarChartData::Categorical(vec![BarChartItem::new(
            "north", "North", 1.0,
        )]));
        let source = BarChartDataSource::from(reactive);
        assert_eq!(
            source.get(),
            BarChartData::Categorical(vec![BarChartItem::new("north", "North", 1.0)])
        );

        reactive.set(BarChartData::Categorical(vec![BarChartItem::new(
            "south", "South", -2.0,
        )]));
        assert_eq!(
            source.get(),
            BarChartData::Categorical(vec![BarChartItem::new("south", "South", -2.0)])
        );
    }

    #[test]
    fn an_item_carries_its_own_colour_and_status_so_neither_can_misalign() {
        // The defect this type replaces: colour arrived as a second vector
        // positionally parallel to the data. Here the two are one value, so no
        // reordering can pair a value with another item's judgement.
        let item = BarChartItem::new("north", "North", -12.5)
            .with_display_value("-12.5 pts")
            .with_status(BarStatus::Unfavorable)
            .with_color("var(--color-warning)");

        assert_eq!(item.key, "north");
        assert_eq!(item.label, "North");
        assert_eq!(item.value, Some(-12.5));
        assert_eq!(item.display_value.as_deref(), Some("-12.5 pts"));
        assert_eq!(item.status, BarStatus::Unfavorable);
        assert_eq!(item.color.as_deref(), Some("var(--color-warning)"));
    }

    #[test]
    fn a_missing_item_is_missing_rather_than_zero() {
        let item = BarChartItem::missing("north", "North");

        assert_eq!(item.value, None);
        assert_eq!(item.status, BarStatus::Neutral);
        assert_eq!(item.color, None);
    }

    #[test]
    fn layout_auto_follows_the_legacy_horizontal_prop() {
        // The whole backward-compatibility story for layout: a caller that
        // never mentions `layout` keeps the orientation `horizontal` chose.
        assert_eq!(
            BarChartLayout::default().resolve(false),
            BarChartLayout::Vertical
        );
        assert_eq!(
            BarChartLayout::default().resolve(true),
            BarChartLayout::Horizontal
        );
        // An explicit layout ignores the legacy prop entirely.
        assert_eq!(
            BarChartLayout::DivergingHorizontal.resolve(false),
            BarChartLayout::DivergingHorizontal
        );
        assert_eq!(
            BarChartLayout::Vertical.resolve(true),
            BarChartLayout::Vertical
        );
    }

    #[test]
    fn layout_orientation_and_tokens_are_stable_selectors() {
        assert!(!BarChartLayout::Vertical.is_horizontal());
        assert!(BarChartLayout::Horizontal.is_horizontal());
        assert!(BarChartLayout::DivergingHorizontal.is_horizontal());
        assert_eq!(BarChartLayout::Vertical.token(), "vertical");
        assert_eq!(BarChartLayout::Horizontal.token(), "horizontal");
        assert_eq!(
            BarChartLayout::DivergingHorizontal.token(),
            "diverging-horizontal"
        );
        assert_eq!(BarStatus::Neutral.token(), "neutral");
        assert_eq!(BarStatus::Favorable.token(), "favorable");
        assert_eq!(BarStatus::Unfavorable.token(), "unfavorable");
    }

    #[test]
    fn chart_texts_default_to_the_strings_the_chart_already_emitted() {
        let texts = BarChartTexts::default();

        assert_eq!(
            texts.empty, "No data",
            "the previously hard-coded empty string must stay the default"
        );
        assert_eq!(texts.category_header, "Category");
        assert_eq!(texts.value_header, "Value");
        assert_eq!(texts.status_header, "Status");
        assert_eq!(texts.no_value, "No value");
    }

    #[test]
    fn status_text_reads_from_the_supplied_copy_rather_than_a_hardcoded_name() {
        let texts = BarChartTexts {
            status_favorable: "Favorable".to_string(),
            status_unfavorable: "Desfavorable".to_string(),
            status_neutral: "Neutral".to_string(),
            ..BarChartTexts::default()
        };

        assert_eq!(texts.status_text(BarStatus::Unfavorable), "Desfavorable");
        assert_eq!(texts.status_text(BarStatus::Favorable), "Favorable");
        assert_eq!(texts.status_text(BarStatus::Neutral), "Neutral");
    }

    #[test]
    fn fieldless_enums_have_their_documented_defaults() {
        assert_eq!(BarStatus::default(), BarStatus::Neutral);
        assert_eq!(BarChartLayout::default(), BarChartLayout::Auto);
        assert_eq!(BarInteractionMode::default(), BarInteractionMode::Auto);
        assert_eq!(
            BarChartActivationSource::default(),
            BarChartActivationSource::Pointer
        );
        assert_eq!(BarValueFormat::default().unit, None);
        assert_eq!(BarValueFormat::default().decimals, None);
    }

    #[test]
    fn value_format_builders_apply_what_they_are_given() {
        let format = BarValueFormat::default().with_unit(" pts").with_decimals(2);

        assert_eq!(format.unit.as_deref(), Some(" pts"));
        assert_eq!(format.decimals, Some(2));
    }
}
