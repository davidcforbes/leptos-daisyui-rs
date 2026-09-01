//! Render-safe bar data and the signed value domain.
//!
//! The defect this module exists to fix: the original chart computed its
//! domain as `max(values).max(0.0)` and divided every value by it. For an
//! all-positive series that is a `0..max` domain and is correct. For a series
//! containing a negative value it is not a domain at all — the negative value
//! divides to a negative fraction and becomes a negative `width`/`height` on a
//! `<rect>`, which is invalid SVG geometry. An all-negative series is worse:
//! `max(...).max(0.0)` collapses to `0.0`, the guard substitutes a range of
//! `1.0`, and every bar is drawn at minus its raw value in view-box units.
//!
//! [`signed_domain`] instead spans the finite values *and always includes
//! zero*, so the zero line is a real datum on the axis rather than an assumed
//! edge, and a bar's length is always measured from it.

use super::format;
use super::types::{BarChartData, BarChartItem, BarChartTexts, BarStatus, BarValueFormat};

/// The finite signed value range a bar chart is drawn against. `min <= 0 <= max`
/// always holds, and `max > min` always holds, so no consumer of a domain has
/// to guard against a zero-width span or a zero line outside the axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Domain {
    /// Lowest value on the axis; never above zero.
    pub min: f64,
    /// Highest value on the axis; never below zero.
    pub max: f64,
}

impl Domain {
    /// The `0.0..=1.0` position of `value` along the axis, measured from
    /// [`Domain::min`]. Clamped, so a value from outside the domain — which
    /// normalization makes impossible, but which a future caller could
    /// introduce — cannot place a mark outside the plot.
    pub(super) fn fraction(&self, value: f64) -> f64 {
        let span = self.max - self.min;
        if span <= 0.0 || !value.is_finite() {
            return 0.0;
        }
        ((value - self.min) / span).clamp(0.0, 1.0)
    }

    /// The `0.0..=1.0` position of the zero line.
    ///
    /// It is `0.0` for an all-positive chart — the bottom of a vertical plot
    /// and the left edge of a horizontal one, which is exactly where the
    /// original chart drew its baseline — and `1.0` for an all-negative one.
    pub(super) fn zero_fraction(&self) -> f64 {
        self.fraction(0.0)
    }

    /// Whether the data reaches below zero, which is when a bar must be able to
    /// extend the other way from the line and the chart must reserve room for
    /// its label there.
    pub(super) fn has_negative(&self) -> bool {
        self.min < 0.0
    }
}

/// One bar prepared for rendering, with its identity, colour and judgement in
/// one value rather than spread across parallel vectors.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedBar {
    pub key: String,
    pub label: String,
    /// Finite or absent. Normalization has already rejected NaN and infinity,
    /// so no renderer downstream can leak one into an attribute.
    pub value: Option<f64>,
    pub display_value: Option<String>,
    pub status: BarStatus,
    pub color: Option<String>,
    /// Unique even when two items share a key, so duplicate keys cannot
    /// collide in the DOM while still reporting the caller's own identity.
    pub dom_id: String,
}

impl NormalizedBar {
    /// Whether this bar can be focused and activated: only a finite value can,
    /// which is what keeps a missing measurement from ever reaching a callback
    /// as a fabricated zero.
    pub(super) fn is_activatable(&self) -> bool {
        self.value.is_some()
    }

    /// The text this bar states, resolved identically for the drawn label, the
    /// accessible name, the table cell and the activation payload.
    pub(super) fn value_text(&self, format: &BarValueFormat, texts: &BarChartTexts) -> String {
        format::displayed_value(
            self.value,
            self.display_value.as_deref(),
            format,
            &texts.no_value,
        )
    }
}

/// Fully normalized bars plus the signed domain they are drawn against.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct NormalizedBarChart {
    pub bars: Vec<NormalizedBar>,
    /// `None` when no bar has a finite value, which is the case a renderer
    /// must treat as empty rather than dividing by.
    pub domain: Option<Domain>,
}

impl NormalizedBarChart {
    /// The keys of the bars a reader can focus, in visual order.
    pub(super) fn navigable_keys(&self) -> Vec<String> {
        self.bars
            .iter()
            .filter(|bar| bar.is_activatable())
            .map(|bar| bar.key.clone())
            .collect()
    }

    /// Whether there is anything to plot.
    pub(super) fn is_empty(&self) -> bool {
        self.bars.is_empty() || self.domain.is_none()
    }
}

/// Converts either public data shape into render-safe bars and a signed domain.
pub(super) fn normalize(data: &BarChartData) -> NormalizedBarChart {
    let bars: Vec<NormalizedBar> = match data {
        // The legacy surface has no identities. Synthesized keys never leave
        // this crate: the legacy render path is non-interactive, so no
        // activation payload and no DOM id is built from them.
        BarChartData::Simple(pairs) => pairs
            .iter()
            .enumerate()
            .map(|(index, (label, value))| NormalizedBar {
                key: format!("bar-{index}"),
                label: label.clone(),
                value: Some(*value).filter(|value| value.is_finite()),
                display_value: None,
                status: BarStatus::Neutral,
                color: None,
                dom_id: format!("bar-{index}"),
            })
            .collect(),
        BarChartData::Categorical(items) => items
            .iter()
            .enumerate()
            .map(|(index, item)| normalize_item(index, item))
            .collect(),
    };
    let domain = signed_domain(bars.iter().filter_map(|bar| bar.value));
    NormalizedBarChart { bars, domain }
}

fn normalize_item(index: usize, item: &BarChartItem) -> NormalizedBar {
    NormalizedBar {
        key: item.key.clone(),
        label: item.label.clone(),
        // A NaN or infinite value becomes a gap, not a zero: it is a broken
        // measurement, and drawing it at the baseline would assert the office
        // was exactly on target.
        value: item.value.filter(|value| value.is_finite()),
        display_value: item.display_value.clone(),
        status: item.status,
        color: item.color.clone(),
        dom_id: format!("{}-{index}", item.key),
    }
}

/// The signed axis range for `values`, or `None` when none of them is finite.
///
/// Three properties, each of which the previous `max(...).max(0.0)` broke:
///
/// - **Zero is always on the axis.** `min` is clamped at or below zero and
///   `max` at or above it, so a bar's length is always measurable from the zero
///   line and never from a plot edge that happens to be somewhere else.
/// - **All-positive data is unchanged.** Every value being above zero makes the
///   clamp produce `0..max` — byte for byte the domain the chart always used,
///   with the zero line at the bottom/left edge where the baseline was drawn.
/// - **A degenerate span never divides by zero.** `min == max` is only reachable
///   when every finite value is exactly zero (any other value pushes one of the
///   two off it), and that case takes `0..1` — the same `1.0` fallback range the
///   original code used, so a chart of zeroes still draws zero-length bars
///   rather than dividing by nothing.
pub(super) fn signed_domain(values: impl IntoIterator<Item = f64>) -> Option<Domain> {
    let mut finite = values.into_iter().filter(|value| value.is_finite());
    let first = finite.next()?;
    let (min, max) = finite.fold((first, first), |(min, max), value| {
        (min.min(value), max.max(value))
    });

    let min = min.min(0.0);
    let max = max.max(0.0);
    if max - min <= 0.0 {
        return Some(Domain { min: 0.0, max: 1.0 });
    }
    Some(Domain { min, max })
}

/// Resolves the fill for legacy bar `index`.
///
/// `overrides` is the optional per-bar color list, which is deliberately
/// **not** required to match `data` in length: an index past its end, or an
/// empty string at that index, falls back to `fallback` (the chart-wide
/// `color`). Bars are always driven by `data`, never by this list, so a
/// mismatched list can neither drop a bar nor panic — a chart that panicked on
/// a length mismatch would take the consumer's whole page down (ldui-jm6).
///
/// The empty-string escape hatch lets a caller override only some bars, e.g.
/// `vec![String::new(), "red".into()]` colors the second bar only.
///
/// This positional behaviour is preserved exactly, and is why the typed model
/// exists beside it rather than replacing it: a positional colour list is
/// mismatch-*safe* but not misalignment-safe, and only a colour that lives
/// inside its item is the latter.
pub(super) fn bar_fill<'a>(index: usize, overrides: &'a [String], fallback: &'a str) -> &'a str {
    match overrides.get(index) {
        Some(c) if !c.is_empty() => c,
        _ => fallback,
    }
}

/// The colour a normalized bar paints with, in precedence order: its own
/// explicit colour, then its status colour, then the positional legacy list,
/// then the chart-wide colour.
///
/// Status colours are daisyUI theme tokens, so they reach the DOM through
/// `charts::paint` like every other colour in this crate.
pub(super) fn resolve_color(
    bar: &NormalizedBar,
    index: usize,
    legacy_overrides: &[String],
    fallback: &str,
) -> String {
    if let Some(color) = bar.color.as_deref() {
        return color.to_string();
    }
    match bar.status {
        BarStatus::Favorable => "var(--color-success)".to_string(),
        BarStatus::Unfavorable => "var(--color-error)".to_string(),
        BarStatus::Neutral => bar_fill(index, legacy_overrides, fallback).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cols(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn domain_of(values: &[f64]) -> Domain {
        signed_domain(values.iter().copied()).expect("finite values have a domain")
    }

    // ── the signed domain ───────────────────────────────────────────────────

    #[test]
    fn all_positive_data_keeps_the_zero_based_domain_the_chart_always_used() {
        // The backward-compatibility case. Before this bead the domain was
        // `max(values).max(0.0)` with the axis implicitly starting at zero;
        // the signed domain must produce exactly that, with the zero line at
        // the bottom/left edge where the baseline was already drawn.
        let domain = domain_of(&[18.0, 24.0, 21.0, 31.0]);

        assert_eq!(
            domain,
            Domain {
                min: 0.0,
                max: 31.0
            }
        );
        assert_eq!(domain.zero_fraction(), 0.0);
        assert!(!domain.has_negative());
        assert_eq!(domain.fraction(31.0), 1.0);
        // The exact projection the old code computed: value / max.
        assert!((domain.fraction(24.0) - 24.0 / 31.0).abs() < 1e-12);
    }

    #[test]
    fn all_negative_data_hangs_from_a_zero_line_at_the_top_of_the_axis() {
        // The broken case: the old domain collapsed to 0.0, fell back to a
        // range of 1.0, and drew every bar at minus its raw value.
        let domain = domain_of(&[-5.0, -1.0, -3.0]);

        assert_eq!(
            domain,
            Domain {
                min: -5.0,
                max: 0.0
            }
        );
        assert_eq!(domain.zero_fraction(), 1.0);
        assert!(domain.has_negative());
        assert_eq!(domain.fraction(-5.0), 0.0);
        assert!((domain.fraction(-1.0) - 0.8).abs() < 1e-12);
    }

    #[test]
    fn mixed_signs_span_the_whole_signed_range_with_zero_inside_it() {
        let domain = domain_of(&[-4.0, 2.0, 6.0, -1.0]);

        assert_eq!(
            domain,
            Domain {
                min: -4.0,
                max: 6.0
            }
        );
        assert!(domain.has_negative());
        assert!(domain.max > 0.0, "zero sits strictly inside the axis");
        assert!((domain.zero_fraction() - 0.4).abs() < 1e-12);
    }

    #[test]
    fn equal_magnitudes_of_opposite_sign_have_equal_geometry() {
        // The acceptance criterion stated as a property: the bar length is
        // |value| / span whatever the sign, so a -12 and a +12 are the same
        // length on opposite sides of zero — in ANY domain, not just a
        // symmetric one.
        for values in [
            vec![-12.0, 12.0],
            vec![-12.0, 12.0, 40.0],
            vec![-30.0, -12.0, 12.0],
        ] {
            let domain = domain_of(&values);
            let zero = domain.zero_fraction();
            let below = zero - domain.fraction(-12.0);
            let above = domain.fraction(12.0) - zero;
            assert!(
                (below - above).abs() < 1e-12,
                "{values:?}: -12 spans {below}, +12 spans {above}"
            );
            assert!(below > 0.0, "{values:?}: a nonzero value must have length");
        }
    }

    #[test]
    fn a_single_zero_value_produces_a_usable_domain_and_a_zero_length_bar() {
        // Degenerate span: every finite value is exactly zero, so min == max.
        // The original code's `v_range` fallback of 1.0 is preserved, which
        // keeps the appearance identical (a flat row of zero-length bars)
        // instead of dividing by zero and producing NaN geometry.
        let domain = domain_of(&[0.0]);

        assert_eq!(domain, Domain { min: 0.0, max: 1.0 });
        assert_eq!(domain.zero_fraction(), 0.0);
        assert_eq!(domain.fraction(0.0), 0.0);
        assert!(!domain.has_negative());
    }

    #[test]
    fn an_all_zero_set_is_the_same_degenerate_case() {
        assert_eq!(domain_of(&[0.0, 0.0, 0.0]), Domain { min: 0.0, max: 1.0 });
        assert_eq!(domain_of(&[0.0, -0.0]), Domain { min: 0.0, max: 1.0 });
    }

    #[test]
    fn a_degenerate_all_equal_set_still_spans_from_zero() {
        // All-equal but nonzero is NOT degenerate here, because zero is always
        // on the axis: the span is 0..7, so the bars are full length. That is
        // what the chart already did for all-equal positive data, and its
        // mirror image for all-equal negative data.
        let positive = domain_of(&[7.0, 7.0, 7.0]);
        assert_eq!(positive, Domain { min: 0.0, max: 7.0 });
        assert_eq!(positive.fraction(7.0), 1.0);

        let negative = domain_of(&[-7.0, -7.0]);
        assert_eq!(
            negative,
            Domain {
                min: -7.0,
                max: 0.0
            }
        );
        assert_eq!(negative.fraction(-7.0), 0.0);
        assert_eq!(negative.zero_fraction(), 1.0);
    }

    #[test]
    fn a_domain_always_contains_zero_and_never_has_a_zero_span() {
        // The two invariants every renderer relies on, swept over a range of
        // shapes rather than asserted on one fixture.
        let sets: Vec<Vec<f64>> = vec![
            vec![1.0],
            vec![-1.0],
            vec![0.0],
            vec![-3.0, -2.0],
            vec![3.0, 2.0],
            vec![-3.0, 4.0],
            vec![f64::MIN_POSITIVE],
            vec![-f64::MIN_POSITIVE],
            vec![1e300, -1e300],
        ];
        for values in sets {
            let domain = domain_of(&values);
            assert!(domain.min <= 0.0, "{values:?}: {domain:?}");
            assert!(domain.max >= 0.0, "{values:?}: {domain:?}");
            assert!(domain.max > domain.min, "{values:?}: {domain:?}");
            let zero = domain.zero_fraction();
            assert!((0.0..=1.0).contains(&zero), "{values:?}: {zero}");
            for value in &values {
                let f = domain.fraction(*value);
                assert!(f.is_finite() && (0.0..=1.0).contains(&f), "{values:?}: {f}");
            }
        }
    }

    #[test]
    fn non_finite_values_neither_form_a_domain_nor_leak_into_one() {
        assert_eq!(signed_domain([f64::NAN, f64::INFINITY]), None);
        assert_eq!(signed_domain(std::iter::empty()), None);

        // A NaN alongside real data is skipped rather than poisoning min/max.
        let domain = domain_of(&[f64::NAN, -2.0, 5.0, f64::NEG_INFINITY]);
        assert_eq!(
            domain,
            Domain {
                min: -2.0,
                max: 5.0
            }
        );
        assert!(domain.fraction(f64::NAN).is_finite());
    }

    // ── normalization ───────────────────────────────────────────────────────

    #[test]
    fn legacy_pairs_normalize_into_neutral_uncoloured_bars() {
        let chart = normalize(&BarChartData::Simple(vec![
            ("Mon".to_string(), 4.0),
            ("Tue".to_string(), 7.0),
        ]));

        assert_eq!(chart.bars.len(), 2);
        assert_eq!(chart.bars[0].label, "Mon");
        assert_eq!(chart.bars[0].value, Some(4.0));
        assert_eq!(chart.bars[0].status, BarStatus::Neutral);
        assert_eq!(chart.bars[0].color, None);
        assert_eq!(chart.domain, Some(Domain { min: 0.0, max: 7.0 }));
    }

    #[test]
    fn a_non_finite_item_becomes_a_gap_rather_than_a_zero() {
        let chart = normalize(&BarChartData::Categorical(vec![
            BarChartItem::new("a", "A", f64::NAN),
            BarChartItem::new("b", "B", f64::INFINITY),
            BarChartItem::missing("c", "C"),
            BarChartItem::new("d", "D", -4.0),
        ]));

        assert_eq!(chart.bars[0].value, None);
        assert_eq!(chart.bars[1].value, None);
        assert_eq!(chart.bars[2].value, None);
        assert_eq!(chart.bars[3].value, Some(-4.0));
        assert!(!chart.bars[0].is_activatable());
        assert!(chart.bars[3].is_activatable());
        assert_eq!(
            chart.domain,
            Some(Domain {
                min: -4.0,
                max: 0.0
            })
        );
        assert_eq!(chart.navigable_keys(), vec!["d".to_string()]);
    }

    #[test]
    fn an_all_missing_chart_has_no_domain_and_reads_as_empty() {
        let chart = normalize(&BarChartData::Categorical(vec![
            BarChartItem::missing("a", "A"),
            BarChartItem::missing("b", "B"),
        ]));

        assert_eq!(chart.domain, None);
        assert!(chart.is_empty());
        assert!(chart.navigable_keys().is_empty());
        assert_eq!(
            chart.bars.len(),
            2,
            "the bars still exist, they just cannot be reached"
        );
    }

    #[test]
    fn duplicate_keys_keep_caller_identity_but_get_unique_dom_identity() {
        let chart = normalize(&BarChartData::Categorical(vec![
            BarChartItem::new("north", "North", 1.0),
            BarChartItem::new("north", "North (2)", 2.0),
        ]));

        assert_eq!(chart.bars[0].key, "north");
        assert_eq!(chart.bars[1].key, "north");
        assert_ne!(chart.bars[0].dom_id, chart.bars[1].dom_id);
    }

    #[test]
    fn a_bars_text_resolves_the_same_way_everywhere() {
        let texts = BarChartTexts::default();
        let format = BarValueFormat::default().with_unit(" pts");
        let chart = normalize(&BarChartData::Categorical(vec![
            BarChartItem::new("a", "A", -12.5),
            BarChartItem::new("b", "B", 3.0).with_display_value("3 ahead"),
            BarChartItem::missing("c", "C"),
        ]));

        assert_eq!(chart.bars[0].value_text(&format, &texts), "-12.5 pts");
        assert_eq!(chart.bars[1].value_text(&format, &texts), "3 ahead");
        assert_eq!(chart.bars[2].value_text(&format, &texts), "No value");
    }

    // ── colour resolution ───────────────────────────────────────────────────

    #[test]
    fn colour_precedence_is_explicit_then_status_then_legacy_then_chart() {
        let chart = normalize(&BarChartData::Categorical(vec![
            BarChartItem::new("a", "A", 1.0).with_color("var(--color-warning)"),
            BarChartItem::new("b", "B", 1.0).with_status(BarStatus::Favorable),
            BarChartItem::new("c", "C", 1.0).with_status(BarStatus::Unfavorable),
            BarChartItem::new("d", "D", 1.0),
        ]));
        let overrides = cols(&["", "", "", "var(--color-info)"]);

        assert_eq!(
            resolve_color(&chart.bars[0], 0, &overrides, "base"),
            "var(--color-warning)",
            "an explicit colour outranks the status colour"
        );
        assert_eq!(
            resolve_color(&chart.bars[1], 1, &overrides, "base"),
            "var(--color-success)"
        );
        assert_eq!(
            resolve_color(&chart.bars[2], 2, &overrides, "base"),
            "var(--color-error)"
        );
        assert_eq!(
            resolve_color(&chart.bars[3], 3, &overrides, "base"),
            "var(--color-info)",
            "a neutral bar still honours the legacy positional list"
        );
    }

    #[test]
    fn a_neutral_bar_with_no_overrides_paints_the_chart_colour() {
        let chart = normalize(&BarChartData::Simple(vec![("Mon".to_string(), 1.0)]));

        assert_eq!(resolve_color(&chart.bars[0], 0, &[], "base"), "base");
    }

    // ── the preserved legacy positional colour behaviour ────────────────────

    #[test]
    fn bar_fill_no_overrides_uses_chart_color() {
        assert_eq!(bar_fill(0, &[], "base"), "base");
        assert_eq!(bar_fill(7, &[], "base"), "base");
    }

    #[test]
    fn bar_fill_uses_override_at_index() {
        let o = cols(&["red", "green", "blue"]);
        assert_eq!(bar_fill(0, &o, "base"), "red");
        assert_eq!(bar_fill(1, &o, "base"), "green");
        assert_eq!(bar_fill(2, &o, "base"), "blue");
    }

    #[test]
    fn bar_fill_shorter_list_falls_back_for_out_of_range() {
        // 2 overrides against a 4-bar series: bars 2 and 3 keep the chart color
        // rather than panicking or dropping.
        let o = cols(&["red", "green"]);
        assert_eq!(bar_fill(2, &o, "base"), "base");
        assert_eq!(bar_fill(3, &o, "base"), "base");
    }

    #[test]
    fn bar_fill_longer_list_ignores_surplus() {
        // 5 overrides against a 2-bar series: only the first two are consulted,
        // and asking for them never reads past `data`.
        let o = cols(&["a", "b", "c", "d", "e"]);
        assert_eq!(bar_fill(0, &o, "base"), "a");
        assert_eq!(bar_fill(1, &o, "base"), "b");
    }

    #[test]
    fn bar_fill_empty_entry_falls_back() {
        // Escape hatch: override only some bars by leaving the others blank.
        let o = cols(&["", "green", ""]);
        assert_eq!(bar_fill(0, &o, "base"), "base");
        assert_eq!(bar_fill(1, &o, "base"), "green");
        assert_eq!(bar_fill(2, &o, "base"), "base");
    }

    #[test]
    fn bar_fill_composed_with_paint_attrs_routes_tokens_to_style() {
        // The demo colours bars with daisyUI tokens, so the composition of
        // `bar_fill` and `paint_attrs` is the path that actually reaches the
        // DOM. A literal keeps the legacy `fill` attribute; a token does not.
        use crate::charts::paint::paint_attrs;

        let o = cols(&["var(--color-success)", "oklch(0.65 0.2 250)"]);
        let (fill, style) = paint_attrs(bar_fill(0, &o, "base").to_string());
        assert_eq!(fill, None);
        assert_eq!(style.as_deref(), Some("fill: var(--color-success)"));

        let (fill, style) = paint_attrs(bar_fill(1, &o, "base").to_string());
        assert_eq!(fill.as_deref(), Some("oklch(0.65 0.2 250)"));
        assert_eq!(style, None);
    }

    #[test]
    fn bar_fill_default_chart_color_keeps_the_fill_attribute() {
        // The default `color` is a literal, so an unstyled chart's DOM is
        // unchanged from before the judgement work.
        use crate::charts::paint::paint_attrs;

        let (fill, style) = paint_attrs(bar_fill(0, &[], "oklch(0.65 0.2 250)").to_string());
        assert_eq!(fill.as_deref(), Some("oklch(0.65 0.2 250)"));
        assert_eq!(style, None);
    }

    #[test]
    fn bar_fill_never_panics_across_a_full_series_sweep() {
        // The property that matters: for every bar index a chart would draw,
        // `bar_fill` returns something, whatever the override length.
        let fallback = "base";
        for overrides_len in 0..8usize {
            let o: Vec<String> = (0..overrides_len).map(|i| format!("c{i}")).collect();
            for i in 0..5usize {
                let got = bar_fill(i, &o, fallback);
                assert!(!got.is_empty());
            }
        }
    }
}
