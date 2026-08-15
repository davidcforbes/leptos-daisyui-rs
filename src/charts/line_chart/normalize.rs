use super::types::{LineCategory, LinePoint, LineSeries};

/// The finite y-range used by categorical line-chart geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Domain {
    pub min: f64,
    pub max: f64,
}

/// One category prepared for rendering without changing its callback identity.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedCategory {
    pub key: String,
    pub label: String,
    pub dom_id: String,
}

/// One finite-or-missing value aligned to a normalized category.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedPoint {
    pub value: Option<f64>,
    pub display_value: Option<String>,
    pub data_label: Option<String>,
    pub marker_color: Option<String>,
}

/// One categorical line with point indexing made safe for renderers.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedSeries {
    pub id: String,
    pub name: String,
    pub points: Vec<NormalizedPoint>,
    pub color: String,
    pub pattern: super::types::LinePattern,
    pub marker: super::types::MarkerStyle,
    pub show_data_labels: bool,
    pub dom_id: String,
}

/// Fully aligned categorical data and its usable y-domain.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedChart {
    pub categories: Vec<NormalizedCategory>,
    pub series: Vec<NormalizedSeries>,
    pub domain: Option<Domain>,
}

/// Converts public categorical values into render-safe, category-aligned data.
pub(super) fn normalize_categorical(
    categories: &[LineCategory],
    series: &[LineSeries],
) -> NormalizedChart {
    warn_duplicate_identifiers(
        "category key",
        categories.iter().map(|category| category.key.as_str()),
    );
    warn_duplicate_identifiers("series id", series.iter().map(|series| series.id.as_str()));

    let categories = categories
        .iter()
        .enumerate()
        .map(|(index, category)| NormalizedCategory {
            key: category.key.clone(),
            label: category.label.clone(),
            dom_id: format!("{}-{index}", category.key),
        })
        .collect::<Vec<_>>();
    let category_count = categories.len();
    let series = series
        .iter()
        .enumerate()
        .map(|(series_index, series)| NormalizedSeries {
            id: series.id.clone(),
            name: series.name.clone(),
            points: (0..category_count)
                .map(|index| normalize_point(series.points.get(index)))
                .collect(),
            color: series.color.clone(),
            pattern: series.pattern.clone(),
            marker: series.marker.clone(),
            show_data_labels: series.show_data_labels,
            dom_id: format!("{}-{series_index}", series.id),
        })
        .collect::<Vec<_>>();

    let domain = domain_for(&series);
    NormalizedChart {
        categories,
        series,
        domain,
    }
}

fn normalize_point(point: Option<&LinePoint>) -> NormalizedPoint {
    let Some(point) = point else {
        return NormalizedPoint {
            value: None,
            display_value: None,
            data_label: None,
            marker_color: None,
        };
    };

    NormalizedPoint {
        value: point.value.filter(|value| value.is_finite()),
        display_value: point.display_value.clone(),
        data_label: point.data_label.clone(),
        marker_color: point.marker_color.clone(),
    }
}

fn domain_for(series: &[NormalizedSeries]) -> Option<Domain> {
    let mut values = series
        .iter()
        .flat_map(|series| series.points.iter().filter_map(|point| point.value));
    let first = values.next()?;
    let (min, max) = values.fold((first, first), |(min, max), value| {
        (min.min(value), max.max(value))
    });

    if min == max {
        let expansion = (min.abs() * 0.05).max(1.0);
        Some(Domain {
            min: min - expansion,
            max: max + expansion,
        })
    } else {
        Some(Domain { min, max })
    }
}

#[cfg(all(debug_assertions, target_arch = "wasm32"))]
fn warn_duplicate_identifiers<'a>(kind: &str, values: impl Iterator<Item = &'a str>) {
    use std::collections::BTreeSet;

    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            duplicates.insert(value);
        }
    }
    for value in duplicates {
        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!(
            "duplicate line chart {kind}: {value}"
        )));
    }
}

#[cfg(not(all(debug_assertions, target_arch = "wasm32")))]
fn warn_duplicate_identifiers<'a>(_kind: &str, _values: impl Iterator<Item = &'a str>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::{LineCategory, LinePoint, LineSeries};

    fn categories() -> Vec<LineCategory> {
        vec![
            LineCategory {
                key: "jan".into(),
                label: "January".into(),
            },
            LineCategory {
                key: "feb".into(),
                label: "February".into(),
            },
            LineCategory {
                key: "mar".into(),
                label: "March".into(),
            },
        ]
    }

    #[test]
    fn normalizes_short_series_by_padding_missing_points() {
        let chart = normalize_categorical(
            &categories(),
            &[LineSeries::new(
                "actual",
                "Actual",
                "blue",
                vec![LinePoint::new(4.0)],
            )],
        );

        assert_eq!(chart.series[0].points.len(), 3);
        assert_eq!(chart.series[0].points[0].value, Some(4.0));
        assert_eq!(chart.series[0].points[1].value, None);
        assert_eq!(chart.series[0].points[2].value, None);
    }

    #[test]
    fn normalizes_extra_points_by_truncating_them_to_category_count() {
        let chart = normalize_categorical(
            &categories()[..2],
            &[LineSeries::new(
                "actual",
                "Actual",
                "blue",
                vec![
                    LinePoint::new(4.0),
                    LinePoint::new(8.0),
                    LinePoint::new(99.0),
                ],
            )],
        );

        assert_eq!(chart.series[0].points.len(), 2);
        assert_eq!(chart.series[0].points[1].value, Some(8.0));
        assert_eq!(chart.domain, Some(Domain { min: 4.0, max: 8.0 }));
    }

    #[test]
    fn converts_none_nan_and_infinite_values_to_gaps_without_losing_metadata() {
        let chart = normalize_categorical(
            &categories(),
            &[LineSeries::new(
                "actual",
                "Actual",
                "blue",
                vec![
                    LinePoint::missing().with_display_value("not measured"),
                    LinePoint::new(f64::NAN).with_data_label("bad"),
                    LinePoint::new(f64::INFINITY),
                ],
            )],
        );

        assert_eq!(
            chart.series[0]
                .points
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
            vec![None, None, None]
        );
        assert_eq!(
            chart.series[0].points[0].display_value.as_deref(),
            Some("not measured")
        );
        assert_eq!(chart.series[0].points[1].data_label.as_deref(), Some("bad"));
        assert_eq!(chart.domain, None);
    }

    #[test]
    fn duplicate_identifiers_preserve_callback_identity_but_have_unique_dom_identity() {
        let duplicate_categories = vec![
            LineCategory {
                key: "week".into(),
                label: "Week 1".into(),
            },
            LineCategory {
                key: "week".into(),
                label: "Week 2".into(),
            },
        ];
        let chart = normalize_categorical(
            &duplicate_categories,
            &[
                LineSeries::new("actual", "Actual", "blue", vec![LinePoint::new(1.0)]),
                LineSeries::new("actual", "Forecast", "red", vec![LinePoint::new(2.0)]),
            ],
        );

        assert_eq!(chart.categories[0].key, "week");
        assert_eq!(chart.categories[1].key, "week");
        assert_ne!(chart.categories[0].dom_id, chart.categories[1].dom_id);
        assert_eq!(chart.series[0].id, "actual");
        assert_eq!(chart.series[1].id, "actual");
        assert_ne!(chart.series[0].dom_id, chart.series[1].dom_id);
    }

    #[test]
    fn all_missing_data_has_no_domain() {
        let chart = normalize_categorical(
            &categories(),
            &[LineSeries::new(
                "actual",
                "Actual",
                "blue",
                vec![LinePoint::missing(), LinePoint::missing()],
            )],
        );

        assert_eq!(chart.domain, None);
    }

    #[test]
    fn singleton_domain_expands_symmetrically_by_the_documented_amount() {
        let chart = normalize_categorical(
            &categories()[..1],
            &[LineSeries::new(
                "actual",
                "Actual",
                "blue",
                vec![LinePoint::new(20.0)],
            )],
        );

        assert_eq!(
            chart.domain,
            Some(Domain {
                min: 19.0,
                max: 21.0
            })
        );
    }
}
