//! The hover/focus card's view model, built independently of its DOM so the
//! row order, preferred-series highlight, and anchor arithmetic are all
//! testable without a browser (ldui-9tr.5). The component renders this model
//! into an absolutely positioned HTML card and only then measures/places it
//! via [`super::geometry::place_tooltip`].

use super::geometry::{Point, Projection, point};
use super::interaction::ActivePoint;
use super::normalize::NormalizedChart;
use super::types::{LinePattern, MarkerShape};

/// One series row of the card: the series identity, its host display string,
/// and enough of its visual identity (color, pattern, marker) to draw the
/// same swatch the legend draws.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TooltipRow {
    pub series_id: String,
    pub series_name: String,
    pub display_value: String,
    pub color: String,
    pub pattern: LinePattern,
    pub marker_shape: MarkerShape,
}

/// The card itself: category header, finite series rows in input order, the
/// preferred series to highlight, and the SVG-space anchor the placement
/// math starts from.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TooltipModel {
    pub id: String,
    pub category_label: String,
    pub rows: Vec<TooltipRow>,
    pub preferred_series_id: Option<String>,
    pub anchor: Point,
}

/// Builds the card for `active`, or `None` when the category has no finite
/// value (nothing to show). Rows keep series input order and skip gaps; the
/// anchor sits on the preferred series' point (falling back to the first
/// finite series) so the card opens next to the mark the user is on.
pub(super) fn tooltip_model(
    chart: &NormalizedChart,
    projection: Projection,
    active: &ActivePoint,
    tooltip_id: &str,
) -> Option<TooltipModel> {
    let category = chart.categories.get(active.category_index)?;
    let rows: Vec<TooltipRow> = chart
        .series
        .iter()
        .filter_map(|series| {
            let point = series.points.get(active.category_index)?;
            let value = point.value.filter(|value| value.is_finite())?;
            Some(TooltipRow {
                series_id: series.id.clone(),
                series_name: series.name.clone(),
                display_value: point
                    .display_value
                    .clone()
                    .unwrap_or_else(|| value.to_string()),
                color: series.color.clone(),
                pattern: series.pattern.clone(),
                marker_shape: series.marker.shape,
            })
        })
        .collect();
    if rows.is_empty() {
        return None;
    }

    let finite_index = |index: usize| {
        chart.series.get(index).and_then(|series| {
            series
                .points
                .get(active.category_index)
                .and_then(|point| point.value)
                .filter(|value| value.is_finite())
                .map(|value| (index, value))
        })
    };
    let (preferred_index, preferred_value) =
        active
            .preferred_series_index
            .and_then(finite_index)
            .or_else(|| (0..chart.series.len()).find_map(finite_index))?;

    Some(TooltipModel {
        id: tooltip_id.to_string(),
        category_label: category.label.clone(),
        rows,
        preferred_series_id: chart
            .series
            .get(preferred_index)
            .map(|series| series.id.clone()),
        anchor: point(
            projection.category_x(active.category_index),
            projection.value_y(preferred_value),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::super::normalize::normalize_categorical;
    use super::super::types::{LineCategory, LinePoint, LineSeries};
    use super::*;
    use crate::charts::line_chart::geometry::{PlotBounds, Projection};

    fn fixture() -> NormalizedChart {
        let categories = vec![
            LineCategory {
                key: "week-01".to_string(),
                label: "W01".to_string(),
            },
            LineCategory {
                key: "week-02".to_string(),
                label: "W02".to_string(),
            },
        ];
        let series = vec![
            LineSeries::new(
                "actual",
                "Actual",
                "var(--color-primary)",
                vec![
                    LinePoint::new(42.0).with_display_value("42 resolved"),
                    LinePoint::missing(),
                ],
            ),
            LineSeries::new(
                "target",
                "Target",
                "var(--color-accent)",
                vec![LinePoint::new(48.0), LinePoint::new(50.0)],
            ),
        ];
        normalize_categorical(&categories, &series)
    }

    fn projection(chart: &NormalizedChart) -> Projection {
        Projection {
            bounds: PlotBounds {
                left: 10.0,
                top: 10.0,
                right: 110.0,
                bottom: 110.0,
            },
            category_count: chart.categories.len(),
            domain: chart.domain.expect("fixture has a domain"),
        }
    }

    #[test]
    fn rows_keep_series_input_order_and_host_display_strings() {
        let chart = fixture();
        let model = tooltip_model(
            &chart,
            projection(&chart),
            &ActivePoint {
                category_index: 0,
                preferred_series_index: None,
            },
            "tip-0",
        )
        .expect("category 0 has finite values");

        assert_eq!(model.category_label, "W01");
        assert_eq!(
            model
                .rows
                .iter()
                .map(|row| row.series_id.as_str())
                .collect::<Vec<_>>(),
            vec!["actual", "target"]
        );
        assert_eq!(model.rows[0].display_value, "42 resolved");
        assert_eq!(model.rows[1].display_value, "48");
        assert_eq!(model.preferred_series_id.as_deref(), Some("actual"));
    }

    #[test]
    fn a_gap_series_is_skipped_and_preference_falls_forward() {
        let chart = fixture();
        let model = tooltip_model(
            &chart,
            projection(&chart),
            &ActivePoint {
                category_index: 1,
                // Preferred points at the gapped series; the model must fall
                // back to the first finite one instead of showing nothing.
                preferred_series_index: Some(0),
            },
            "tip-0",
        )
        .expect("category 1 still has the target value");

        assert_eq!(model.rows.len(), 1);
        assert_eq!(model.rows[0].series_id, "target");
        assert_eq!(model.preferred_series_id.as_deref(), Some("target"));
    }

    #[test]
    fn anchor_sits_on_the_preferred_point_and_is_finite() {
        let chart = fixture();
        let projection = projection(&chart);
        let model = tooltip_model(
            &chart,
            projection,
            &ActivePoint {
                category_index: 0,
                preferred_series_index: Some(1),
            },
            "tip-0",
        )
        .expect("model");

        assert_eq!(model.anchor.x, projection.category_x(0));
        assert_eq!(model.anchor.y, projection.value_y(48.0));
        assert!(model.anchor.x.is_finite() && model.anchor.y.is_finite());
    }

    #[test]
    fn a_category_with_no_finite_value_yields_no_model() {
        let categories = vec![LineCategory {
            key: "week-01".to_string(),
            label: "W01".to_string(),
        }];
        let series = vec![LineSeries::new(
            "actual",
            "Actual",
            "var(--color-primary)",
            vec![LinePoint::missing()],
        )];
        let chart = normalize_categorical(&categories, &series);
        let projection = Projection {
            bounds: PlotBounds {
                left: 0.0,
                top: 0.0,
                right: 100.0,
                bottom: 100.0,
            },
            category_count: 1,
            domain: super::super::normalize::Domain { min: 0.0, max: 1.0 },
        };

        assert!(
            tooltip_model(
                &chart,
                projection,
                &ActivePoint {
                    category_index: 0,
                    preferred_series_index: None,
                },
                "tip-0",
            )
            .is_none()
        );
    }
}
