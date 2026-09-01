use super::{
    normalize::{Domain, NormalizedChart, NormalizedSeries},
    types::{LinePattern, LineValueAxis, MarkerStyle},
};

const DEFAULT_MARKER_SIZE: f64 = 4.0;
const DEFAULT_MARKER_STROKE_WIDTH: f64 = 1.0;
/// Largest rendered marker radius or stroke width, in SVG view-box units.
const MAX_MARKER_SIZE: f64 = 100.0;
/// Largest visual marker radius, including half of the maximum stroke width.
const MAX_MARKER_VISUAL_RADIUS: f64 = MAX_MARKER_SIZE + MAX_MARKER_SIZE / 2.0;
const MAX_SVG_COORDINATE: f64 = 1_000_000.0;

/// The interior rectangle available for categorical line-chart drawing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PlotBounds {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// A data-only categorical/value mapping into the SVG plot rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Projection {
    pub bounds: PlotBounds,
    pub category_count: usize,
    pub domain: Domain,
}

impl Projection {
    /// Returns the evenly spaced x-coordinate for a category.
    pub(super) fn category_x(&self, index: usize) -> f64 {
        if self.category_count <= 1 {
            return (self.bounds.left + self.bounds.right) / 2.0;
        }
        let index = index.min(self.category_count - 1);
        self.bounds.left
            + (self.bounds.right - self.bounds.left) * index as f64
                / (self.category_count - 1) as f64
    }

    /// Returns the SVG y-coordinate for a finite data value.
    pub(super) fn value_y(&self, value: f64) -> f64 {
        let ratio = self.domain_ratio(value).unwrap_or(0.5);
        self.bounds.bottom - ratio * (self.bounds.bottom - self.bounds.top)
    }

    fn domain_ratio(&self, value: f64) -> Option<f64> {
        if !value.is_finite()
            || !self.domain.min.is_finite()
            || !self.domain.max.is_finite()
            || self.domain.min >= self.domain.max
        {
            return None;
        }
        let scale = self.domain.min.abs().max(self.domain.max.abs());
        if !scale.is_finite() || scale == 0.0 {
            return None;
        }
        let min = self.domain.min / scale;
        let max = self.domain.max / scale;
        let span = max - min;
        let ratio = (value / scale - min) / span;
        ratio.is_finite().then(|| ratio.clamp(0.0, 1.0))
    }

    /// Returns the nearest category index for an SVG x-coordinate.
    pub(super) fn category_at_x(&self, x: f64) -> Option<usize> {
        if self.category_count == 0 || !x.is_finite() {
            return None;
        }
        if self.category_count == 1 {
            return Some(0);
        }
        let width = self.bounds.right - self.bounds.left;
        if !width.is_finite() || width <= 0.0 {
            return Some(0);
        }
        let index = ((x - self.bounds.left) / width * (self.category_count - 1) as f64)
            .round()
            .clamp(0.0, (self.category_count - 1) as f64);
        Some(index as usize)
    }
}

/// One projection per value axis, over one shared plot rectangle.
///
/// Category spacing is axis-independent, so the x geometry is answered once by
/// the primary projection; only the value-to-y mapping differs, which is the
/// entire point of a second scale. When no series is on the secondary axis
/// there is no secondary projection at all, and a lookup for it falls back to
/// the primary — a series can therefore never be projected against a domain
/// that does not exist.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AxisProjections {
    primary: Projection,
    secondary: Option<Projection>,
}

impl AxisProjections {
    /// Builds the projections for a plot, or `None` when neither axis has a
    /// finite domain and there is nothing to draw.
    pub(super) fn new(
        bounds: PlotBounds,
        category_count: usize,
        primary: Option<Domain>,
        secondary: Option<Domain>,
    ) -> Option<Self> {
        // A chart may legitimately have every series on the secondary axis, in
        // which case the primary projection borrows that domain purely so the
        // shared category geometry stays well defined. Nothing is drawn
        // against it: a primary series with a finite value would have given
        // the primary axis a domain of its own.
        let fallback = primary.or(secondary)?;
        Some(Self {
            primary: Projection {
                bounds,
                category_count,
                domain: primary.unwrap_or(fallback),
            },
            secondary: secondary.map(|domain| Projection {
                bounds,
                category_count,
                domain,
            }),
        })
    }

    /// The projection a series measured against `axis` is drawn with.
    pub(super) fn for_axis(&self, axis: LineValueAxis) -> Projection {
        match axis {
            LineValueAxis::Primary => self.primary,
            LineValueAxis::Secondary => self.secondary.unwrap_or(self.primary),
        }
    }

    /// The evenly spaced x-coordinate for a category, shared by both axes.
    pub(super) fn category_x(&self, index: usize) -> f64 {
        self.primary.category_x(index)
    }

    /// The nearest category index for an SVG x-coordinate, shared by both axes.
    pub(super) fn category_at_x(&self, x: f64) -> Option<usize> {
        self.primary.category_at_x(x)
    }

    /// The SVG y-coordinate a value takes on `axis`.
    pub(super) fn value_y(&self, axis: LineValueAxis, value: f64) -> f64 {
        self.for_axis(axis).value_y(value)
    }

    /// Wraps one projection as a primary-only pair.
    #[cfg(test)]
    pub(super) fn single(projection: Projection) -> Self {
        Self {
            primary: projection,
            secondary: None,
        }
    }
}

/// A two-dimensional SVG or CSS coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Point {
    pub x: f64,
    pub y: f64,
}

/// A two-dimensional extent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Size {
    pub width: f64,
    pub height: f64,
}

/// A tooltip corner relative to its active-point anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TooltipCorner {
    UpperRight,
    UpperLeft,
    LowerRight,
    LowerLeft,
}

/// The resolved, wrapper-relative tooltip position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TooltipPlacement {
    pub origin: Point,
    pub corner: TooltipCorner,
}

/// Creates a point without involving browser types.
pub(super) fn point(x: f64, y: f64) -> Point {
    Point { x, y }
}

/// Creates a size without involving browser types.
pub(super) fn size(width: f64, height: f64) -> Size {
    Size { width, height }
}

/// Gutter space the plot must leave for annotations drawn outside it.
///
/// Every field defaults to "nothing extra", so a chart that renders no data
/// labels and no second axis gets exactly the bounds it always did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PlotInsets {
    /// Largest visual marker radius across the chart's series.
    pub max_marker_radius: f64,
    /// Point labels are drawn beside markers.
    pub has_data_labels: bool,
    /// A right-hand tick scale is drawn.
    pub secondary_ticks: bool,
    /// A rotated title is drawn outside the left axis.
    pub primary_label: bool,
    /// A rotated title is drawn outside the right axis.
    pub secondary_label: bool,
}

/// Approximate width of a rotated axis title, in view-box units.
const AXIS_TITLE_GUTTER: f64 = 16.0;
/// Approximate width of a right-hand tick label column, in view-box units.
const SECONDARY_TICK_GUTTER: f64 = 34.0;

/// Calculates plot bounds while keeping a minimum one-unit drawable region.
pub(super) fn plot_bounds(width: f64, height: f64, insets: PlotInsets) -> PlotBounds {
    let width = dimension(width);
    let height = dimension(height);
    let marker = marker_radius(insets.max_marker_radius);
    let left = inset(
        40.0 + marker + gutter(insets.primary_label, AXIS_TITLE_GUTTER),
        width,
    );
    let right_padding = 12.0
        + marker
        + if insets.has_data_labels { 40.0 } else { 0.0 }
        + gutter(insets.secondary_ticks, SECONDARY_TICK_GUTTER)
        + gutter(insets.secondary_label, AXIS_TITLE_GUTTER);
    // Data-label baselines sit five units above the marker. Reserve their
    // approximate 10px ascent as well, so a top-domain value remains inside
    // the SVG viewBox rather than being clipped at its first glyph.
    let top = inset(
        12.0 + marker
            + if insets.has_data_labels {
                5.0 + 10.0
            } else {
                0.0
            },
        height,
    );
    let bottom_padding = 32.0 + marker;

    PlotBounds {
        left,
        top,
        right: (width - inset(right_padding, width)).max(left + 1.0),
        bottom: (height - inset(bottom_padding, height)).max(top + 1.0),
    }
}

/// Converts finite points into independent SVG path segments, projected
/// against the series' own value axis.
pub(super) fn path_segments(
    series: &NormalizedSeries,
    projections: &AxisProjections,
) -> Vec<String> {
    let projection = projections.for_axis(series.axis);
    let mut segments = Vec::new();
    let mut commands = Vec::new();
    for (index, point) in series.points.iter().enumerate() {
        let Some(value) = point.value.filter(|value| value.is_finite()) else {
            if !commands.is_empty() {
                segments.push(commands.join(" "));
                commands.clear();
            }
            continue;
        };
        let command = if commands.is_empty() { "M" } else { "L" };
        commands.push(format!(
            "{command} {} {}",
            svg_coordinate(projection.category_x(index)),
            svg_coordinate(projection.value_y(value))
        ));
    }
    if !commands.is_empty() {
        segments.push(commands.join(" "));
    }
    segments
}

/// Returns tick indices that fit the available CSS width and retain both edges.
pub(super) fn visible_tick_indices(
    category_count: usize,
    css_width: f64,
    minimum_gap_px: f64,
) -> Vec<usize> {
    match category_count {
        0 => return Vec::new(),
        1 => return vec![0],
        _ => {}
    }
    if !css_width.is_finite() || !minimum_gap_px.is_finite() || minimum_gap_px <= 0.0 {
        return (0..category_count).collect();
    }
    let available = (css_width.max(0.0) / minimum_gap_px)
        .floor()
        .min((category_count - 1) as f64) as usize
        + 1;
    let tick_count = available.clamp(2, category_count);
    if tick_count == category_count {
        return (0..category_count).collect();
    }
    // Even integer stride, never rounded interpolation: rounding a
    // near-full tick count left ADJACENT categories selected at the ends,
    // where the start/end-anchored edge labels extend toward their
    // neighbours and collided (style-audit hard OVERLAP, ldui-9tr.7). The
    // final category is always labelled; a stepped pick closer than one
    // stride to it is dropped instead of letting the two labels touch.
    let stride = (((category_count - 1) as f64 / (tick_count - 1) as f64).ceil() as usize).max(1);
    let mut indices: Vec<usize> = (0..category_count - 1).step_by(stride).collect();
    while indices
        .last()
        .is_some_and(|&last| category_count - 1 - last < stride)
    {
        indices.pop();
    }
    indices.push(category_count - 1);
    indices
}

/// Selects the finite series point closest to an SVG y-coordinate.
///
/// Each series is compared where it is actually drawn — on its own axis — so
/// the hover card picks the mark under the pointer rather than the one that
/// would be there if every series shared one scale.
pub(super) fn nearest_series_at(
    chart: &NormalizedChart,
    projections: &AxisProjections,
    category_index: usize,
    svg_y: f64,
) -> Option<usize> {
    if category_index >= chart.categories.len() || !svg_y.is_finite() {
        return None;
    }
    chart
        .series
        .iter()
        .enumerate()
        .filter_map(|(index, series)| {
            series
                .points
                .get(category_index)
                .and_then(|point| point.value)
                .map(|value| {
                    (
                        index,
                        (projections.value_y(series.axis, value) - svg_y).abs(),
                    )
                })
        })
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
}

/// Places a tooltip in the first fitting preferred corner, then clamps it.
pub(super) fn place_tooltip(
    anchor: Point,
    tooltip: Size,
    container: Size,
    gap: f64,
) -> TooltipPlacement {
    let anchor = Point {
        x: finite_nonnegative(anchor.x),
        y: finite_nonnegative(anchor.y),
    };
    let tooltip = Size {
        width: finite_nonnegative(tooltip.width),
        height: finite_nonnegative(tooltip.height),
    };
    let container = Size {
        width: finite_nonnegative(container.width),
        height: finite_nonnegative(container.height),
    };
    let gap = finite_nonnegative(gap);
    let candidates = [
        TooltipPlacement {
            origin: point(anchor.x + gap, anchor.y - gap - tooltip.height),
            corner: TooltipCorner::UpperRight,
        },
        TooltipPlacement {
            origin: point(
                anchor.x - gap - tooltip.width,
                anchor.y - gap - tooltip.height,
            ),
            corner: TooltipCorner::UpperLeft,
        },
        TooltipPlacement {
            origin: point(anchor.x + gap, anchor.y + gap),
            corner: TooltipCorner::LowerRight,
        },
        TooltipPlacement {
            origin: point(anchor.x - gap - tooltip.width, anchor.y + gap),
            corner: TooltipCorner::LowerLeft,
        },
    ];
    let placement = candidates
        .iter()
        .copied()
        .find(|candidate| fits(*candidate, tooltip, container))
        .unwrap_or_else(|| {
            candidates
                .into_iter()
                .min_by(|left, right| {
                    overflow(*left, tooltip, container)
                        .total_cmp(&overflow(*right, tooltip, container))
                })
                .expect("tooltip candidates are non-empty")
        });
    TooltipPlacement {
        origin: point(
            placement
                .origin
                .x
                .clamp(0.0, (container.width - tooltip.width).max(0.0)),
            placement
                .origin
                .y
                .clamp(0.0, (container.height - tooltip.height).max(0.0)),
        ),
        corner: placement.corner,
    }
}

/// Serializes a valid pattern for SVG, with solid as the invalid fallback.
pub(super) fn dasharray(pattern: &LinePattern) -> Option<String> {
    let values: &[f64] = match pattern {
        LinePattern::Solid => return None,
        LinePattern::Dashed => &[6.0, 4.0],
        LinePattern::Dotted => &[2.0, 3.0],
        LinePattern::Custom(values) => values,
    };
    if values.is_empty()
        || values
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return None;
    }
    Some(
        values
            .iter()
            .map(|value| svg_number(*value))
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// Resolves invalid marker sizes to the documented safe default.
pub(super) fn marker_size(marker: &MarkerStyle) -> f64 {
    capped_positive_or(marker.size, DEFAULT_MARKER_SIZE, MAX_MARKER_SIZE)
}

/// Resolves invalid marker stroke widths to the documented safe default.
pub(super) fn marker_stroke_width(marker: &MarkerStyle) -> f64 {
    capped_positive_or(
        marker.stroke_width,
        DEFAULT_MARKER_STROKE_WIDTH,
        MAX_MARKER_SIZE,
    )
}

/// Formats a finite, bounded SVG numeric attribute.
pub(super) fn svg_number(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }
    value
        .clamp(-MAX_SVG_COORDINATE, MAX_SVG_COORDINATE)
        .to_string()
}

fn svg_coordinate(value: f64) -> String {
    let value = if value.is_finite() {
        value.clamp(-MAX_SVG_COORDINATE, MAX_SVG_COORDINATE)
    } else {
        0.0
    };
    format!("{value:.2}")
}

fn dimension(value: f64) -> f64 {
    positive_or(value, 1.0)
}

fn marker_radius(value: f64) -> f64 {
    capped_positive_or(value, DEFAULT_MARKER_SIZE, MAX_MARKER_VISUAL_RADIUS)
}

fn gutter(present: bool, width: f64) -> f64 {
    if present { width } else { 0.0 }
}

fn inset(value: f64, dimension: f64) -> f64 {
    value.min((dimension - 1.0).max(0.0) / 2.0)
}

fn positive_or(value: f64, default: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value.min(MAX_SVG_COORDINATE)
    } else {
        default
    }
}

fn capped_positive_or(value: f64, default: f64, maximum: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value.min(maximum)
    } else {
        default
    }
}

fn finite_nonnegative(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, MAX_SVG_COORDINATE)
    } else {
        0.0
    }
}

fn fits(placement: TooltipPlacement, tooltip: Size, container: Size) -> bool {
    placement.origin.x >= 0.0
        && placement.origin.y >= 0.0
        && placement.origin.x + tooltip.width <= container.width
        && placement.origin.y + tooltip.height <= container.height
}

fn overflow(placement: TooltipPlacement, tooltip: Size, container: Size) -> f64 {
    (-placement.origin.x).max(0.0)
        + (-placement.origin.y).max(0.0)
        + (placement.origin.x + tooltip.width - container.width).max(0.0)
        + (placement.origin.y + tooltip.height - container.height).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::line_chart::normalize::normalize_categorical;
    use crate::charts::{LineCategory, LinePattern, LinePoint, LineSeries, MarkerStyle};

    fn projection() -> Projection {
        Projection {
            bounds: PlotBounds {
                left: 10.0,
                top: 20.0,
                right: 110.0,
                bottom: 120.0,
            },
            category_count: 4,
            domain: Domain {
                min: 0.0,
                max: 30.0,
            },
        }
    }

    fn insets(max_marker_radius: f64, has_data_labels: bool) -> PlotInsets {
        PlotInsets {
            max_marker_radius,
            has_data_labels,
            ..PlotInsets::default()
        }
    }

    #[test]
    fn plot_bounds_clamp_tiny_dimensions_and_reserve_marker_and_label_clearance() {
        let tiny = plot_bounds(0.0, -5.0, insets(4.0, false));
        let labelled = plot_bounds(400.0, 200.0, insets(8.0, true));
        let unlabelled = plot_bounds(400.0, 200.0, insets(8.0, false));

        assert!(tiny.right - tiny.left >= 1.0);
        assert!(tiny.bottom - tiny.top >= 1.0);
        assert!(labelled.right < unlabelled.right);
        assert!(
            labelled.top >= 35.0,
            "a 12px point label needs its 10px ascent plus the 5px marker gap above the top marker"
        );
    }

    /// The single-axis guarantee at the geometry layer: the plot rectangle a
    /// chart with no second axis and no axis titles gets is bit-identical to
    /// the one the pre-secondary-axis code computed.
    #[test]
    fn a_chart_without_a_second_axis_keeps_its_original_plot_rectangle() {
        let base = plot_bounds(400.0, 200.0, insets(4.0, false));

        assert_eq!(
            base,
            PlotBounds {
                left: 44.0,
                top: 16.0,
                right: 384.0,
                bottom: 164.0
            }
        );
        assert_eq!(
            plot_bounds(
                400.0,
                200.0,
                PlotInsets {
                    max_marker_radius: 4.0,
                    ..PlotInsets::default()
                }
            ),
            base
        );
    }

    #[test]
    fn a_second_axis_reserves_right_gutters_without_moving_the_left_one() {
        let base = plot_bounds(400.0, 200.0, insets(4.0, false));
        let ticks = plot_bounds(
            400.0,
            200.0,
            PlotInsets {
                max_marker_radius: 4.0,
                secondary_ticks: true,
                ..PlotInsets::default()
            },
        );
        let both = plot_bounds(
            400.0,
            200.0,
            PlotInsets {
                max_marker_radius: 4.0,
                secondary_ticks: true,
                secondary_label: true,
                primary_label: true,
                ..PlotInsets::default()
            },
        );

        assert_eq!(ticks.left, base.left, "the left gutter is untouched");
        assert_eq!(ticks.right, base.right - 34.0);
        assert_eq!(both.left, base.left + 16.0);
        assert_eq!(both.right, base.right - 34.0 - 16.0);
        assert_eq!((both.top, both.bottom), (base.top, base.bottom));
    }

    #[test]
    fn each_series_is_projected_against_its_own_axis() {
        let projections = AxisProjections::new(
            PlotBounds {
                left: 0.0,
                top: 0.0,
                right: 100.0,
                bottom: 100.0,
            },
            2,
            Some(Domain {
                min: 0.0,
                max: 100.0,
            }),
            Some(Domain { min: 0.0, max: 1.0 }),
        )
        .expect("both axes have domains");

        // The same number is near the floor of a count axis and at the ceiling
        // of a duration axis: reading one against the other is the defect.
        assert_eq!(projections.value_y(LineValueAxis::Primary, 1.0), 99.0);
        assert_eq!(projections.value_y(LineValueAxis::Secondary, 1.0), 0.0);
        assert_eq!(projections.category_x(0), 0.0);
        assert_eq!(projections.category_x(1), 100.0);
    }

    #[test]
    fn a_missing_secondary_axis_falls_back_to_the_primary_projection() {
        let bounds = PlotBounds {
            left: 0.0,
            top: 0.0,
            right: 100.0,
            bottom: 100.0,
        };
        let domain = Domain {
            min: 0.0,
            max: 10.0,
        };
        let projections =
            AxisProjections::new(bounds, 2, Some(domain), None).expect("a primary domain");

        assert_eq!(
            projections.for_axis(LineValueAxis::Secondary),
            projections.for_axis(LineValueAxis::Primary)
        );
        assert!(
            AxisProjections::new(bounds, 2, None, None).is_none(),
            "no finite value anywhere means nothing to project"
        );
        // Every series on the secondary axis: the shared category geometry
        // still has to work, so the primary borrows that domain.
        let secondary_only = AxisProjections::new(bounds, 2, None, Some(domain))
            .expect("a secondary domain is enough");
        assert_eq!(
            secondary_only.for_axis(LineValueAxis::Primary),
            secondary_only.for_axis(LineValueAxis::Secondary)
        );
    }

    #[test]
    fn the_nearest_series_is_the_one_nearest_where_it_is_actually_drawn() {
        let chart = normalize_categorical(
            &categories(1),
            &[
                LineSeries::new("count", "Count", "blue", vec![LinePoint::new(10.0)]),
                LineSeries::new("duration", "Duration", "red", vec![LinePoint::new(0.9)])
                    .on_secondary_axis(),
            ],
        );
        let projections = AxisProjections::new(
            PlotBounds {
                left: 0.0,
                top: 0.0,
                right: 100.0,
                bottom: 100.0,
            },
            1,
            Some(Domain {
                min: 0.0,
                max: 100.0,
            }),
            Some(Domain { min: 0.0, max: 1.0 }),
        )
        .expect("both axes have domains");

        // Count 10 draws near the floor at y=90; duration 0.9 draws near the
        // ceiling at y=10 on its own scale. Read against the count scale it
        // would sit at y=99.1 and the pointer below would pick the count.
        assert_eq!(projections.value_y(LineValueAxis::Primary, 10.0), 90.0);
        assert_eq!(projections.value_y(LineValueAxis::Secondary, 0.9), 10.0);
        assert_eq!(
            nearest_series_at(&chart, &projections, 0, 15.0),
            Some(1),
            "the pointer is next to the duration mark, not the count mark"
        );
    }

    #[test]
    fn category_projection_and_tick_thinning_keep_both_edges() {
        let projection = projection();

        assert_eq!(projection.category_x(0), 10.0);
        assert_eq!(projection.category_x(3), 110.0);
        assert_eq!(projection.category_at_x(-1.0), Some(0));
        assert_eq!(projection.category_at_x(111.0), Some(3));
        assert_eq!(visible_tick_indices(10, 100.0, 30.0), vec![0, 3, 6, 9]);
        assert_eq!(visible_tick_indices(1, 100.0, 30.0), vec![0]);
        assert_eq!(visible_tick_indices(0, 100.0, 30.0), Vec::<usize>::new());
    }

    #[test]
    fn tick_thinning_never_selects_adjacent_categories_when_thinning() {
        // The regression: 14 categories at 672px/56px min gap used to pick a
        // near-full 13 ticks whose rounded positions left W01/W02 (and
        // W13/W14) adjacent — exactly where the edge labels anchor toward
        // each other and collide.
        let picked = visible_tick_indices(14, 672.0, 56.0);
        assert_eq!(*picked.last().expect("non-empty"), 13, "last category kept");
        assert_eq!(picked[0], 0, "first category kept");
        for pair in picked.windows(2) {
            assert!(
                pair[1] - pair[0] >= 2,
                "thinned ticks must never be adjacent: {picked:?}"
            );
        }
    }

    #[test]
    fn tick_thinning_saturates_huge_finite_width_before_usize_arithmetic() {
        assert_eq!(visible_tick_indices(2, f64::MAX, 1.0), vec![0, 1]);
    }

    #[test]
    fn paths_are_segmented_at_missing_points() {
        let chart = normalize_categorical(
            &categories(4),
            &[LineSeries::new(
                "actual",
                "Actual",
                "blue",
                vec![
                    LinePoint::new(0.0),
                    LinePoint::missing(),
                    LinePoint::new(20.0),
                    LinePoint::new(30.0),
                ],
            )],
        );

        assert_eq!(
            path_segments(&chart.series[0], &AxisProjections::single(projection())),
            vec![
                "M 10.00 120.00".to_string(),
                "M 76.67 53.33 L 110.00 20.00".to_string()
            ]
        );
    }

    #[test]
    fn invalid_dash_and_marker_values_resolve_to_safe_svg_values() {
        assert_eq!(dasharray(&LinePattern::Solid), None);
        assert_eq!(dasharray(&LinePattern::Custom(vec![])), None);
        assert_eq!(dasharray(&LinePattern::Custom(vec![3.0, 0.0])), None);
        assert_eq!(dasharray(&LinePattern::Custom(vec![f64::INFINITY])), None);
        assert_eq!(
            dasharray(&LinePattern::Custom(vec![3.0, 2.5])),
            Some("3 2.5".into())
        );

        let marker = MarkerStyle {
            size: f64::NAN,
            stroke_width: -2.0,
            ..MarkerStyle::default()
        };
        assert_eq!(marker_size(&marker), 4.0);
        assert_eq!(marker_stroke_width(&marker), 1.0);
        assert_eq!(svg_number(f64::NAN), "0");
    }

    #[test]
    fn capped_marker_visual_radius_is_fully_reserved_by_plot_bounds() {
        let marker = MarkerStyle {
            size: f64::MAX,
            stroke_width: f64::MAX,
            ..MarkerStyle::default()
        };

        assert_eq!(marker_size(&marker), 100.0);
        assert_eq!(marker_stroke_width(&marker), 100.0);
        let bounds = plot_bounds(
            1_000.0,
            500.0,
            insets(
                marker_size(&marker) + marker_stroke_width(&marker) / 2.0,
                false,
            ),
        );
        assert_eq!(bounds.left, 190.0);
        assert_eq!(bounds.top, 162.0);
    }

    #[test]
    fn opposite_finite_extrema_project_to_distinct_plot_edges() {
        let projection = Projection {
            bounds: PlotBounds {
                left: 0.0,
                top: 0.0,
                right: 100.0,
                bottom: 100.0,
            },
            category_count: 2,
            domain: Domain {
                min: -f64::MAX,
                max: f64::MAX,
            },
        };

        assert_eq!(projection.value_y(-f64::MAX), 100.0);
        assert_eq!(projection.value_y(0.0), 50.0);
        assert_eq!(projection.value_y(f64::MAX), 0.0);
    }

    #[test]
    fn nearest_series_distinguishes_opposite_finite_extrema() {
        let chart = normalize_categorical(
            &categories(1),
            &[
                LineSeries::new("low", "Low", "blue", vec![LinePoint::new(-f64::MAX)]),
                LineSeries::new("high", "High", "red", vec![LinePoint::new(f64::MAX)]),
            ],
        );
        let projection = Projection {
            bounds: PlotBounds {
                left: 0.0,
                top: 0.0,
                right: 100.0,
                bottom: 100.0,
            },
            category_count: 1,
            domain: Domain {
                min: -f64::MAX,
                max: f64::MAX,
            },
        };

        assert_eq!(
            nearest_series_at(&chart, &AxisProjections::single(projection), 0, 5.0),
            Some(1)
        );
    }

    #[test]
    fn paths_bound_non_finite_projection_coordinates_before_svg_serialization() {
        let chart = normalize_categorical(
            &categories(1),
            &[LineSeries::new(
                "actual",
                "Actual",
                "blue",
                vec![LinePoint::new(10.0)],
            )],
        );
        let projection = Projection {
            bounds: PlotBounds {
                left: 0.0,
                top: 0.0,
                right: f64::INFINITY,
                bottom: f64::INFINITY,
            },
            category_count: 1,
            domain: Domain {
                min: 0.0,
                max: 10.0,
            },
        };

        assert_eq!(
            path_segments(&chart.series[0], &AxisProjections::single(projection)),
            vec!["M 0.00 0.00".to_string()]
        );
    }

    #[test]
    fn nearest_category_and_series_choose_geometrically_closest_finite_point() {
        let chart = normalize_categorical(
            &categories(3),
            &[
                LineSeries::new(
                    "low",
                    "Low",
                    "blue",
                    vec![
                        LinePoint::new(0.0),
                        LinePoint::missing(),
                        LinePoint::new(0.0),
                    ],
                ),
                LineSeries::new(
                    "high",
                    "High",
                    "red",
                    vec![
                        LinePoint::new(30.0),
                        LinePoint::new(20.0),
                        LinePoint::new(30.0),
                    ],
                ),
            ],
        );
        let projection = Projection {
            bounds: PlotBounds {
                left: 0.0,
                top: 0.0,
                right: 100.0,
                bottom: 100.0,
            },
            category_count: 3,
            domain: Domain {
                min: 0.0,
                max: 30.0,
            },
        };

        let projections = AxisProjections::single(projection);

        assert_eq!(projection.category_at_x(56.0), Some(1));
        assert_eq!(nearest_series_at(&chart, &projections, 0, 5.0), Some(1));
        assert_eq!(nearest_series_at(&chart, &projections, 1, 80.0), Some(1));
        assert_eq!(nearest_series_at(&chart, &projections, 9, 50.0), None);
    }

    #[test]
    fn tooltip_uses_upper_right_when_it_fits() {
        assert_eq!(
            place_tooltip(point(40.0, 40.0), size(20.0, 10.0), size(100.0, 100.0), 5.0),
            TooltipPlacement {
                origin: point(45.0, 25.0),
                corner: TooltipCorner::UpperRight
            }
        );
    }

    #[test]
    fn tooltip_flips_to_upper_left_when_upper_right_overflows() {
        assert_eq!(
            place_tooltip(point(90.0, 40.0), size(20.0, 10.0), size(100.0, 100.0), 5.0),
            TooltipPlacement {
                origin: point(65.0, 25.0),
                corner: TooltipCorner::UpperLeft
            }
        );
    }

    #[test]
    fn tooltip_flips_to_lower_right_when_upper_positions_overflow() {
        assert_eq!(
            place_tooltip(point(40.0, 5.0), size(20.0, 10.0), size(100.0, 100.0), 5.0),
            TooltipPlacement {
                origin: point(45.0, 10.0),
                corner: TooltipCorner::LowerRight
            }
        );
    }

    #[test]
    fn tooltip_flips_to_lower_left_when_right_positions_overflow() {
        assert_eq!(
            place_tooltip(point(90.0, 5.0), size(20.0, 10.0), size(100.0, 100.0), 5.0),
            TooltipPlacement {
                origin: point(65.0, 10.0),
                corner: TooltipCorner::LowerLeft
            }
        );
    }

    #[test]
    fn tooltip_clamps_both_axes_when_no_preferred_corner_fits() {
        assert_eq!(
            place_tooltip(point(2.0, 2.0), size(150.0, 120.0), size(100.0, 100.0), 5.0),
            TooltipPlacement {
                origin: point(0.0, 0.0),
                corner: TooltipCorner::LowerRight
            }
        );
    }

    fn categories(count: usize) -> Vec<LineCategory> {
        (0..count)
            .map(|index| LineCategory {
                key: index.to_string(),
                label: index.to_string(),
            })
            .collect()
    }
}
