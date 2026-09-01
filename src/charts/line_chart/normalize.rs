use super::types::{LineAxes, LineAxisOptions, LineCategory, LinePoint, LineSeries, LineValueAxis};

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
    pub axis: LineValueAxis,
    /// The axis' formatting options, copied here so every surface that states
    /// this series' numbers formats them the same way without re-resolving.
    pub format: LineAxisOptions,
}

/// Fully aligned categorical data and its usable per-axis y-domains.
///
/// `domain` is the primary (left) axis and keeps its original name and
/// meaning. `secondary_domain` is `None` unless at least one series opted onto
/// the secondary axis *and* has a finite value there, which is what keeps a
/// single-axis chart from growing a phantom right-hand scale.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedChart {
    pub categories: Vec<NormalizedCategory>,
    pub series: Vec<NormalizedSeries>,
    pub domain: Option<Domain>,
    pub secondary_domain: Option<Domain>,
    pub axes: LineAxes,
}

impl NormalizedChart {
    /// Attaches the caller's axis options, giving every series the formatting
    /// its own axis defines.
    pub(super) fn with_axes(mut self, axes: LineAxes) -> Self {
        for series in &mut self.series {
            series.format = axes.options(series.axis).clone();
        }
        self.axes = axes;
        self
    }

    /// Whether a right-hand axis should be drawn at all.
    pub(super) fn has_secondary_axis(&self) -> bool {
        self.secondary_domain.is_some()
    }
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
            axis: series.axis,
            format: LineAxisOptions::default(),
        })
        .collect::<Vec<_>>();

    // Each axis sees only its own series, so a duration series can never widen
    // a count scale and a count series can never flatten a duration one.
    let domain = domain_for(&series, LineValueAxis::Primary);
    let secondary_domain = domain_for(&series, LineValueAxis::Secondary);
    NormalizedChart {
        categories,
        series,
        domain,
        secondary_domain,
        axes: LineAxes::default(),
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

/// The finite range of the series assigned to `axis`, or `None` when no series
/// is on that axis or none of them has a finite value there.
///
/// A single-value or all-equal axis (`min == max`) would otherwise give the
/// projection a zero-height domain and a division by zero, so it is expanded
/// symmetrically here — explicitly, rather than being absorbed downstream by
/// the projection's `unwrap_or(0.5)` fallback, which would silently flatten
/// every point of that axis onto the plot's mid-line.
fn domain_for(series: &[NormalizedSeries], axis: LineValueAxis) -> Option<Domain> {
    let mut values = series
        .iter()
        .filter(|series| series.axis == axis)
        .flat_map(|series| series.points.iter().filter_map(|point| point.value));
    let first = values.next()?;
    let (min, max) = values.fold((first, first), |(min, max), value| {
        (min.min(value), max.max(value))
    });

    if min == max {
        let expansion = (min.abs() * 0.05).max(1.0);
        let expanded_min = min - expansion;
        let expanded_max = max + expansion;
        Some(Domain {
            min: if expanded_min.is_finite() {
                expanded_min
            } else {
                min
            },
            max: if expanded_max.is_finite() {
                expanded_max
            } else {
                max
            },
        })
    } else {
        Some(Domain { min, max })
    }
}

// Called from the wasm32 debug warning path and unit tests only, so the
// native non-test lint pass sees it as dead.
#[cfg_attr(not(all(debug_assertions, target_arch = "wasm32")), allow(dead_code))]
fn duplicate_identifier_set<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    use std::collections::BTreeSet;

    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            duplicates.insert(value);
        }
    }
    duplicates.into_iter().map(str::to_owned).collect()
}

#[cfg(all(debug_assertions, target_arch = "wasm32"))]
fn warn_duplicate_identifiers<'a>(kind: &str, values: impl Iterator<Item = &'a str>) {
    let duplicates = duplicate_identifier_set(values);
    if !duplicates.is_empty() {
        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!(
            "duplicate line chart {kind} set: {}",
            duplicates.join(", ")
        )));
    }
}

#[cfg(not(all(debug_assertions, target_arch = "wasm32")))]
fn warn_duplicate_identifiers<'a>(_kind: &str, _values: impl Iterator<Item = &'a str>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::{LineAxisOptions, LineCategory, LinePoint, LineSeries, LineValueAxis};

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

    #[test]
    fn singleton_extreme_values_keep_a_finite_nonzero_domain() {
        for value in [f64::MAX, -f64::MAX] {
            let chart = normalize_categorical(
                &categories()[..1],
                &[LineSeries::new(
                    "actual",
                    "Actual",
                    "blue",
                    vec![LinePoint::new(value)],
                )],
            );
            let domain = chart.domain.expect("a finite point has a domain");

            assert!(domain.min.is_finite(), "{value:?}: {domain:?}");
            assert!(domain.max.is_finite(), "{value:?}: {domain:?}");
            assert!(domain.min < domain.max, "{value:?}: {domain:?}");
        }
    }

    /// The single-axis guarantee, proven rather than asserted: a chart whose
    /// series never mention an axis produces the same primary domain it always
    /// did AND no secondary domain at all, so no right-hand scale can render.
    #[test]
    fn a_chart_of_default_series_has_no_secondary_axis() {
        let chart = normalize_categorical(
            &categories(),
            &[
                LineSeries::new(
                    "actual",
                    "Actual",
                    "blue",
                    vec![LinePoint::new(4.0), LinePoint::new(9.0)],
                ),
                LineSeries::new(
                    "target",
                    "Target",
                    "red",
                    vec![LinePoint::new(6.0), LinePoint::new(6.0)],
                ),
            ],
        );

        assert!(
            chart
                .series
                .iter()
                .all(|series| series.axis == LineValueAxis::Primary)
        );
        assert_eq!(chart.domain, Some(Domain { min: 4.0, max: 9.0 }));
        assert_eq!(chart.secondary_domain, None);
        assert!(!chart.has_secondary_axis());
    }

    /// The bead's motivating case: three count series and one duration series
    /// that is three orders of magnitude smaller. Each axis' domain must see
    /// only its own series, or the duration line flatlines against the counts.
    #[test]
    fn each_axis_computes_its_domain_from_its_own_series_only() {
        let chart = normalize_categorical(
            &categories(),
            &[
                LineSeries::new(
                    "opened",
                    "Opened",
                    "blue",
                    vec![
                        LinePoint::new(120.0),
                        LinePoint::new(150.0),
                        LinePoint::new(90.0),
                    ],
                ),
                LineSeries::new(
                    "first-response",
                    "Average first response",
                    "orange",
                    vec![
                        LinePoint::new(0.4),
                        LinePoint::new(1.2),
                        LinePoint::new(0.9),
                    ],
                )
                .on_secondary_axis(),
            ],
        );

        assert_eq!(
            chart.domain,
            Some(Domain {
                min: 90.0,
                max: 150.0
            }),
            "the count axis must not be widened down to the duration values"
        );
        assert_eq!(
            chart.secondary_domain,
            Some(Domain { min: 0.4, max: 1.2 }),
            "the duration axis must not be widened up to the count values"
        );
        assert!(chart.has_secondary_axis());
    }

    /// A degenerate axis must not reach the projection with `min == max`: that
    /// is the division by zero, and the projection's fallback would put every
    /// point of that axis on the mid-line without saying so.
    #[test]
    fn an_all_equal_secondary_axis_expands_instead_of_dividing_by_zero() {
        let chart = normalize_categorical(
            &categories(),
            &[
                LineSeries::new(
                    "opened",
                    "Opened",
                    "blue",
                    vec![LinePoint::new(1.0), LinePoint::new(5.0)],
                ),
                LineSeries::new(
                    "sla",
                    "SLA",
                    "orange",
                    vec![
                        LinePoint::new(12.0),
                        LinePoint::new(12.0),
                        LinePoint::new(12.0),
                    ],
                )
                .on_secondary_axis(),
            ],
        );
        let secondary = chart.secondary_domain.expect("finite values have a domain");

        assert!(secondary.min < secondary.max, "{secondary:?}");
        assert!(secondary.min.is_finite() && secondary.max.is_finite());
        assert_eq!(
            secondary,
            Domain {
                min: 11.0,
                max: 13.0
            },
            "the documented symmetric expansion, at least one unit wide"
        );
        assert_eq!(
            chart.domain,
            Some(Domain { min: 1.0, max: 5.0 }),
            "expanding one axis must not disturb the other"
        );
    }

    #[test]
    fn a_secondary_series_with_no_finite_values_renders_no_secondary_axis() {
        let chart = normalize_categorical(
            &categories(),
            &[
                LineSeries::new("opened", "Opened", "blue", vec![LinePoint::new(3.0)]),
                LineSeries::new(
                    "sla",
                    "SLA",
                    "orange",
                    vec![LinePoint::missing(), LinePoint::new(f64::NAN)],
                )
                .on_secondary_axis(),
            ],
        );

        assert!(!chart.has_secondary_axis());
        assert_eq!(chart.secondary_domain, None);
        assert!(chart.domain.is_some());
    }

    #[test]
    fn a_chart_of_only_secondary_series_still_has_a_secondary_domain() {
        let chart = normalize_categorical(
            &categories(),
            &[LineSeries::new(
                "sla",
                "SLA",
                "orange",
                vec![LinePoint::new(2.0), LinePoint::new(8.0)],
            )
            .on_secondary_axis()],
        );

        assert_eq!(chart.domain, None, "no series is on the primary axis");
        assert_eq!(chart.secondary_domain, Some(Domain { min: 2.0, max: 8.0 }));
    }

    #[test]
    fn axis_options_reach_every_series_through_its_own_axis() {
        let chart = normalize_categorical(
            &categories(),
            &[
                LineSeries::new("opened", "Opened", "blue", vec![LinePoint::new(3.0)]),
                LineSeries::new("sla", "SLA", "orange", vec![LinePoint::new(2.0)])
                    .on_secondary_axis(),
            ],
        )
        .with_axes(LineAxes {
            primary: LineAxisOptions::default().with_unit(" cases"),
            secondary: LineAxisOptions::default().with_unit(" s").with_decimals(1),
        });

        assert_eq!(chart.series[0].format.unit.as_deref(), Some(" cases"));
        assert_eq!(chart.series[0].format.decimals, None);
        assert_eq!(chart.series[1].format.unit.as_deref(), Some(" s"));
        assert_eq!(chart.series[1].format.decimals, Some(1));
    }

    #[test]
    fn duplicate_identifier_set_aggregates_all_values_into_one_diagnostic_payload() {
        assert_eq!(
            duplicate_identifier_set(["actual", "forecast", "actual", "forecast"].into_iter()),
            vec!["actual", "forecast"]
        );
        assert!(duplicate_identifier_set(["actual", "forecast"].into_iter()).is_empty());
    }
}
