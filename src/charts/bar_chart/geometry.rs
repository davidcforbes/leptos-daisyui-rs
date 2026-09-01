//! Pure bar geometry: where the zero line sits, and how long each bar is.
//!
//! Everything here is a function of numbers only, so the properties the
//! acceptance criteria name — equal magnitudes get equal geometry on opposite
//! sides of zero, and no width or height is ever negative — are testable
//! natively without a browser.

use super::normalize::Domain;
use super::types::BarChartLayout;

/// The plot rectangle in view-box units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Bounds {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl Bounds {
    pub(super) fn width(&self) -> f64 {
        (self.right - self.left).max(0.0)
    }

    pub(super) fn height(&self) -> f64 {
        (self.bottom - self.top).max(0.0)
    }
}

/// The gutters reserved around the plot.
///
/// A signed chart needs more room than an all-positive one, because a negative
/// bar's value label sits at the bar's *outward* end — below the plot floor in
/// a vertical chart, and to the left of the plot in a horizontal one, which is
/// where the category-label gutter already is. Reserving that room only when
/// the data actually contains a negative value is what keeps every existing
/// all-positive caller's plot rectangle — and therefore its bars, labels and
/// baseline — exactly where it has always been.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Insets {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    /// Distance below the plot floor at which a vertical chart's category
    /// labels sit. `15.0` is the original value.
    pub category_label_offset: f64,
    /// Distance left of the plot at which a horizontal chart's category labels
    /// end. `5.0` is the original value; a signed chart widens it so a negative
    /// bar's value label has somewhere to go.
    pub category_gutter_offset: f64,
}

impl Insets {
    /// The gutters for `layout`, given whether the domain reaches below zero.
    pub(super) fn new(layout: BarChartLayout, has_negative: bool) -> Self {
        if layout.is_horizontal() {
            // Legacy horizontal padding, preserved exactly for all-positive
            // data. `DivergingHorizontal` always takes the roomier variant
            // because a caller reaches for it precisely when a filtering of the
            // data may turn negative.
            if has_negative || matches!(layout, BarChartLayout::DivergingHorizontal) {
                Self {
                    left: 120.0,
                    right: 48.0,
                    top: 10.0,
                    bottom: 10.0,
                    category_label_offset: 15.0,
                    category_gutter_offset: 48.0,
                }
            } else {
                Self {
                    left: 80.0,
                    right: 30.0,
                    top: 10.0,
                    bottom: 10.0,
                    category_label_offset: 15.0,
                    category_gutter_offset: 5.0,
                }
            }
        } else if has_negative {
            Self {
                left: 40.0,
                right: 10.0,
                top: 10.0,
                bottom: 54.0,
                category_label_offset: 30.0,
                category_gutter_offset: 5.0,
            }
        } else {
            Self {
                left: 40.0,
                right: 10.0,
                top: 10.0,
                bottom: 40.0,
                category_label_offset: 15.0,
                category_gutter_offset: 5.0,
            }
        }
    }
}

/// The plot rectangle inside `width` x `height` for `insets`.
pub(super) fn plot_bounds(width: f64, height: f64, insets: Insets) -> Bounds {
    Bounds {
        left: insets.left,
        right: (width - insets.right).max(insets.left),
        top: insets.top,
        bottom: (height - insets.bottom).max(insets.top),
    }
}

/// One category's slot along the category axis: `(offset, thickness)`.
///
/// The 70/30 split between bar and gap is the chart's original proportion, and
/// the arithmetic is written the way the original wrote it, so an existing
/// chart's bars land on the same coordinates to the last decimal.
pub(super) fn band(index: usize, count: usize, start: f64, extent: f64) -> (f64, f64) {
    if count == 0 {
        return (start, 0.0);
    }
    let slot = extent / count as f64;
    let thickness = slot * 0.7;
    let gap = slot * 0.3;
    (start + index as f64 * slot + gap / 2.0, thickness)
}

/// The ordered `(start, length)` of a bar running between the zero line and its
/// value.
///
/// This is the whole fix for negative geometry: the bar is the *interval*
/// between two positions, so its start is whichever of them comes first and its
/// length is their distance. A negative value therefore produces a bar on the
/// other side of zero rather than a negative `width` or `height`.
pub(super) fn span(zero: f64, value: f64) -> (f64, f64) {
    if !zero.is_finite() || !value.is_finite() {
        return (zero, 0.0);
    }
    (zero.min(value), (value - zero).abs())
}

/// Where a value sits along the value axis.
///
/// A vertical chart's value axis runs *up* the screen, so a larger value gets a
/// smaller y. A horizontal chart's runs left to right.
pub(super) fn value_position(
    domain: Domain,
    bounds: Bounds,
    layout: BarChartLayout,
    value: f64,
) -> f64 {
    let fraction = domain.fraction(value);
    if layout.is_horizontal() {
        bounds.left + fraction * bounds.width()
    } else {
        bounds.bottom - fraction * bounds.height()
    }
}

/// Where the zero line sits along the value axis.
///
/// Read from [`Domain::zero_fraction`] rather than by projecting the value
/// `0.0`, so the axis' own statement of where zero is and the line the chart
/// draws there are the same number by construction.
pub(super) fn zero_position(domain: Domain, bounds: Bounds, layout: BarChartLayout) -> f64 {
    let fraction = domain.zero_fraction();
    if layout.is_horizontal() {
        bounds.left + fraction * bounds.width()
    } else {
        bounds.bottom - fraction * bounds.height()
    }
}

/// One category's whole slot along the category axis, bar plus gap:
/// `(offset, extent)`.
///
/// This — not [`band`] — is the hit and focus target, because a zero value
/// draws a bar of no thickness at all and must still be reachable by pointer
/// and keyboard.
pub(super) fn slot(index: usize, count: usize, start: f64, extent: f64) -> (f64, f64) {
    if count == 0 {
        return (start, 0.0);
    }
    let slot = extent / count as f64;
    (start + index as f64 * slot, slot)
}

/// One bar's rectangle in view-box units. `width` and `height` are never
/// negative, by construction of [`span`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BarRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The rectangle for bar `index` of `count` carrying `value`.
pub(super) fn bar_rect(
    layout: BarChartLayout,
    bounds: Bounds,
    domain: Domain,
    index: usize,
    count: usize,
    value: f64,
) -> BarRect {
    let zero = zero_position(domain, bounds, layout);
    let position = value_position(domain, bounds, layout, value);
    let (start, length) = span(zero, position);
    if layout.is_horizontal() {
        let (offset, thickness) = band(index, count, bounds.top, bounds.height());
        BarRect {
            x: start,
            y: offset,
            width: length,
            height: thickness,
        }
    } else {
        let (offset, thickness) = band(index, count, bounds.left, bounds.width());
        BarRect {
            x: offset,
            y: start,
            width: thickness,
            height: length,
        }
    }
}

/// The rule drawn across a bar's *outward* end — the end away from zero — as
/// `(x1, y1, x2, y2)`.
///
/// This is how a caller-owned status reaches a reader who cannot use colour.
/// A favorable bar's cap is solid and an unfavorable one's is dashed, and a
/// dash pattern survives forced-colors mode, where every fill in the chart
/// collapses to the same system colour and hue alone says nothing.
pub(super) fn cap_line(
    layout: BarChartLayout,
    rect: BarRect,
    negative: bool,
) -> (f64, f64, f64, f64) {
    if layout.is_horizontal() {
        let x = if negative {
            rect.x
        } else {
            rect.x + rect.width
        };
        (x, rect.y, x, rect.y + rect.height)
    } else {
        let y = if negative {
            rect.y + rect.height
        } else {
            rect.y
        };
        (rect.x, y, rect.x + rect.width, y)
    }
}

/// A view-box coordinate as the chart has always written them.
pub(super) fn n(value: f64) -> String {
    format!("{value:.2}")
}

#[cfg(test)]
mod tests {
    use super::super::normalize::signed_domain;
    use super::*;

    fn domain(values: &[f64]) -> Domain {
        signed_domain(values.iter().copied()).expect("finite values have a domain")
    }

    // ── the interval, which is where negative geometry was produced ─────────

    #[test]
    fn a_span_is_never_negative_whichever_side_of_zero_the_value_is_on() {
        for (zero, value) in [
            (100.0, 40.0),
            (100.0, 160.0),
            (100.0, 100.0),
            (0.0, -0.0),
            (-50.0, 50.0),
        ] {
            let (start, length) = span(zero, value);
            assert!(length >= 0.0, "({zero}, {value}) -> {length}");
            assert!(start.is_finite());
            assert!((start + length - zero.max(value)).abs() < 1e-12);
        }
    }

    #[test]
    fn a_span_from_a_non_finite_position_collapses_rather_than_leaking_nan() {
        for (zero, value) in [(f64::NAN, 1.0), (1.0, f64::NAN), (1.0, f64::INFINITY)] {
            let (_, length) = span(zero, value);
            assert_eq!(length, 0.0, "({zero}, {value})");
        }
    }

    // ── the four value shapes the acceptance criteria name ──────────────────

    #[test]
    fn all_positive_vertical_geometry_matches_the_original_arithmetic() {
        // The compatibility proof at the geometry level. The original chart
        // computed `bh = (value / v_max) * chart_h` and `by = pad_top + chart_h
        // - bh`, with the baseline at `pad_top + chart_h`. The signed path must
        // reproduce both to the bit.
        let values = [18.0, 24.0, 21.0, 31.0];
        let d = domain(&values);
        let insets = Insets::new(BarChartLayout::Vertical, false);
        let bounds = plot_bounds(400.0, 200.0, insets);
        let chart_h = 200.0 - 10.0 - 40.0;

        assert_eq!(insets.bottom, 40.0, "legacy vertical padding is preserved");
        assert_eq!(insets.category_label_offset, 15.0);
        assert_eq!(bounds.height(), chart_h);
        assert_eq!(
            zero_position(d, bounds, BarChartLayout::Vertical),
            10.0 + chart_h,
            "the zero line is the baseline the chart already drew"
        );

        for value in values {
            let zero = zero_position(d, bounds, BarChartLayout::Vertical);
            let y = value_position(d, bounds, BarChartLayout::Vertical, value);
            let (start, length) = span(zero, y);
            let legacy_height = (value / 31.0) * chart_h;
            assert!((length - legacy_height).abs() < 1e-9, "{value}");
            assert!(
                (start - (10.0 + chart_h - legacy_height)).abs() < 1e-9,
                "{value}"
            );
        }
    }

    #[test]
    fn all_positive_horizontal_geometry_matches_the_original_arithmetic() {
        let values = [12.0, 7.0, 19.0];
        let d = domain(&values);
        let insets = Insets::new(BarChartLayout::Horizontal, false);
        let bounds = plot_bounds(400.0, 200.0, insets);
        let chart_w = 400.0 - 80.0 - 30.0;

        assert_eq!(insets.left, 80.0, "legacy horizontal padding is preserved");
        assert_eq!(insets.right, 30.0);
        assert_eq!(insets.category_gutter_offset, 5.0);
        assert_eq!(bounds.width(), chart_w);
        assert_eq!(zero_position(d, bounds, BarChartLayout::Horizontal), 80.0);

        for value in values {
            let zero = zero_position(d, bounds, BarChartLayout::Horizontal);
            let x = value_position(d, bounds, BarChartLayout::Horizontal, value);
            let (start, length) = span(zero, x);
            assert_eq!(
                start, 80.0,
                "a positive bar still starts at the plot's left"
            );
            assert!((length - (value / 19.0) * chart_w).abs() < 1e-9, "{value}");
        }
    }

    #[test]
    fn all_negative_bars_hang_from_a_zero_line_and_keep_positive_lengths() {
        let values = [-5.0, -1.0, -3.0];
        let d = domain(&values);
        let bounds = plot_bounds(400.0, 200.0, Insets::new(BarChartLayout::Vertical, true));
        let zero = zero_position(d, bounds, BarChartLayout::Vertical);

        assert_eq!(
            zero, bounds.top,
            "an all-negative axis puts zero at the top"
        );
        for value in values {
            let y = value_position(d, bounds, BarChartLayout::Vertical, value);
            let (start, length) = span(zero, y);
            assert!(length > 0.0, "{value} must draw a bar");
            assert_eq!(start, zero, "a negative bar hangs downward from zero");
            assert!(start + length <= bounds.bottom + 1e-9, "{value}");
        }
    }

    #[test]
    fn mixed_signs_put_equal_magnitudes_at_equal_lengths_on_opposite_sides() {
        // The symmetry criterion, at the geometry layer rather than the domain
        // layer, so the projection is included in the claim.
        let d = domain(&[-12.0, 12.0, 40.0]);
        let bounds = plot_bounds(
            480.0,
            200.0,
            Insets::new(BarChartLayout::DivergingHorizontal, true),
        );
        let zero = zero_position(d, bounds, BarChartLayout::DivergingHorizontal);

        let (down_start, down_len) = span(
            zero,
            value_position(d, bounds, BarChartLayout::DivergingHorizontal, -12.0),
        );
        let (up_start, up_len) = span(
            zero,
            value_position(d, bounds, BarChartLayout::DivergingHorizontal, 12.0),
        );

        assert!((down_len - up_len).abs() < 1e-9, "{down_len} vs {up_len}");
        assert!(down_len > 0.0);
        assert!(
            (down_start + down_len - zero).abs() < 1e-9,
            "negative ends at zero"
        );
        assert!((up_start - zero).abs() < 1e-9, "positive starts at zero");
    }

    #[test]
    fn a_single_zero_value_draws_a_zero_length_bar_on_the_baseline() {
        let d = domain(&[0.0]);
        let bounds = plot_bounds(400.0, 200.0, Insets::new(BarChartLayout::Vertical, false));
        let zero = zero_position(d, bounds, BarChartLayout::Vertical);
        let y = value_position(d, bounds, BarChartLayout::Vertical, 0.0);
        let (start, length) = span(zero, y);

        assert_eq!(length, 0.0);
        assert_eq!(start, bounds.bottom);
        assert!(length.is_finite());
    }

    #[test]
    fn a_degenerate_all_equal_set_fills_every_bar_exactly_as_before() {
        let d = domain(&[7.0, 7.0, 7.0]);
        let bounds = plot_bounds(400.0, 200.0, Insets::new(BarChartLayout::Vertical, false));
        let zero = zero_position(d, bounds, BarChartLayout::Vertical);

        for _ in 0..3 {
            let y = value_position(d, bounds, BarChartLayout::Vertical, 7.0);
            let (start, length) = span(zero, y);
            assert!((length - bounds.height()).abs() < 1e-9);
            assert!((start - bounds.top).abs() < 1e-9);
        }
    }

    #[test]
    fn no_bar_in_any_shape_of_data_ever_gets_a_negative_dimension() {
        // The hard acceptance criterion, swept rather than spot-checked.
        let sets: Vec<Vec<f64>> = vec![
            vec![1.0, 2.0, 3.0],
            vec![-1.0, -2.0, -3.0],
            vec![-4.0, 0.0, 9.0],
            vec![0.0, 0.0],
            vec![-1e9, 1e9],
            vec![5.0, 5.0, 5.0],
        ];
        for layout in [
            BarChartLayout::Vertical,
            BarChartLayout::Horizontal,
            BarChartLayout::DivergingHorizontal,
        ] {
            for values in &sets {
                let d = domain(values);
                let insets = Insets::new(layout, d.min < 0.0);
                let bounds = plot_bounds(400.0, 200.0, insets);
                let zero = zero_position(d, bounds, layout);
                for value in values {
                    let pos = value_position(d, bounds, layout, *value);
                    let (start, length) = span(zero, pos);
                    assert!(length >= 0.0, "{layout:?} {values:?} {value}: {length}");
                    assert!(start.is_finite() && length.is_finite());
                }
            }
        }
    }

    // ── the category axis ───────────────────────────────────────────────────

    #[test]
    fn a_band_reproduces_the_original_seventy_thirty_split() {
        let (offset, thickness) = band(0, 4, 40.0, 350.0);
        let slot = 350.0 / 4.0;

        assert!((thickness - slot * 0.7).abs() < 1e-12);
        assert!((offset - (40.0 + slot * 0.3 / 2.0)).abs() < 1e-12);

        let (offset, _) = band(3, 4, 40.0, 350.0);
        assert!((offset - (40.0 + 3.0 * slot + slot * 0.3 / 2.0)).abs() < 1e-12);
    }

    #[test]
    fn an_empty_band_does_not_divide_by_zero() {
        assert_eq!(band(0, 0, 40.0, 350.0), (40.0, 0.0));
        assert_eq!(slot(0, 0, 40.0, 350.0), (40.0, 0.0));
    }

    #[test]
    fn a_slot_encloses_its_band_so_a_zero_length_bar_is_still_reachable() {
        for index in 0..4 {
            let (band_offset, thickness) = band(index, 4, 40.0, 350.0);
            let (slot_offset, extent) = slot(index, 4, 40.0, 350.0);
            assert!(slot_offset <= band_offset, "{index}");
            assert!(slot_offset + extent >= band_offset + thickness, "{index}");
            assert!((extent - 350.0 / 4.0).abs() < 1e-12);
        }
        // The slots tile the axis exactly, so there is no dead gap between two
        // adjacent hit targets.
        let (last_offset, last_extent) = slot(3, 4, 40.0, 350.0);
        assert!((last_offset + last_extent - (40.0 + 350.0)).abs() < 1e-12);
    }

    #[test]
    fn a_signed_chart_reserves_room_a_positive_one_does_not_need() {
        // The rule that keeps every existing caller's plot rectangle where it
        // was: the roomier gutters appear only when a negative value or the
        // diverging layout asks for them.
        assert_eq!(
            Insets::new(BarChartLayout::Vertical, false),
            Insets::new(BarChartLayout::Vertical, false)
        );
        assert_eq!(Insets::new(BarChartLayout::Vertical, false).bottom, 40.0);
        assert_eq!(Insets::new(BarChartLayout::Vertical, true).bottom, 54.0);
        assert_eq!(Insets::new(BarChartLayout::Horizontal, false).left, 80.0);
        assert_eq!(Insets::new(BarChartLayout::Horizontal, true).left, 120.0);
        assert_eq!(
            Insets::new(BarChartLayout::DivergingHorizontal, false).left,
            120.0,
            "the diverging layout always reserves the negative-label gutter"
        );
    }

    #[test]
    fn plot_bounds_never_invert_when_the_viewbox_is_smaller_than_its_gutters() {
        let bounds = plot_bounds(40.0, 20.0, Insets::new(BarChartLayout::Horizontal, true));

        assert!(bounds.right >= bounds.left);
        assert!(bounds.bottom >= bounds.top);
        assert_eq!(bounds.width(), 0.0);
        assert_eq!(bounds.height(), 0.0);
    }

    // ── whole rectangles ────────────────────────────────────────────────────

    #[test]
    fn a_rectangle_never_has_a_negative_dimension_and_meets_the_zero_line() {
        let values = [-8.0, 0.0, 3.0, 12.0];
        let d = domain(&values);
        for layout in [
            BarChartLayout::Vertical,
            BarChartLayout::DivergingHorizontal,
        ] {
            let bounds = plot_bounds(480.0, 220.0, Insets::new(layout, true));
            let zero = zero_position(d, bounds, layout);
            for (index, value) in values.iter().enumerate() {
                let rect = bar_rect(layout, bounds, d, index, values.len(), *value);
                assert!(
                    rect.width >= 0.0 && rect.height >= 0.0,
                    "{layout:?} {value}"
                );
                let (near, far) = if layout.is_horizontal() {
                    (rect.x, rect.x + rect.width)
                } else {
                    (rect.y, rect.y + rect.height)
                };
                assert!(
                    (near - zero).abs() < 1e-9 || (far - zero).abs() < 1e-9,
                    "{layout:?} {value}: one end of the bar must sit on the zero line"
                );
            }
        }
    }

    #[test]
    fn equal_magnitudes_produce_congruent_rectangles() {
        let values = [-12.0, 12.0];
        let d = domain(&values);
        let layout = BarChartLayout::DivergingHorizontal;
        let bounds = plot_bounds(480.0, 200.0, Insets::new(layout, true));

        let negative = bar_rect(layout, bounds, d, 0, 2, -12.0);
        let positive = bar_rect(layout, bounds, d, 1, 2, 12.0);

        assert!((negative.width - positive.width).abs() < 1e-9);
        assert!((negative.height - positive.height).abs() < 1e-9);
        assert!(
            negative.x < positive.x,
            "they sit on opposite sides of zero"
        );
    }

    #[test]
    fn a_status_cap_sits_on_the_end_of_the_bar_away_from_zero() {
        let values = [-6.0, 6.0];
        let d = domain(&values);

        let layout = BarChartLayout::DivergingHorizontal;
        let bounds = plot_bounds(480.0, 200.0, Insets::new(layout, true));
        let positive = bar_rect(layout, bounds, d, 1, 2, 6.0);
        let (x1, y1, x2, y2) = cap_line(layout, positive, false);
        assert!((x1 - (positive.x + positive.width)).abs() < 1e-9);
        assert_eq!(x1, x2, "a horizontal bar's cap is vertical");
        assert!((y2 - y1 - positive.height).abs() < 1e-9);

        let negative = bar_rect(layout, bounds, d, 0, 2, -6.0);
        let (x1, _, x2, _) = cap_line(layout, negative, true);
        assert!(
            (x1 - negative.x).abs() < 1e-9,
            "the far end of a negative bar"
        );
        assert_eq!(x1, x2);

        let layout = BarChartLayout::Vertical;
        let bounds = plot_bounds(480.0, 200.0, Insets::new(layout, true));
        let up = bar_rect(layout, bounds, d, 1, 2, 6.0);
        let (x1, y1, x2, y2) = cap_line(layout, up, false);
        assert!(
            (y1 - up.y).abs() < 1e-9,
            "a positive column's cap is its top"
        );
        assert_eq!(y1, y2, "a vertical bar's cap is horizontal");
        assert!((x2 - x1 - up.width).abs() < 1e-9);

        let down = bar_rect(layout, bounds, d, 0, 2, -6.0);
        let (_, y1, _, y2) = cap_line(layout, down, true);
        assert!((y1 - (down.y + down.height)).abs() < 1e-9);
        assert_eq!(y1, y2);
    }

    #[test]
    fn coordinates_are_written_with_the_two_decimals_the_chart_always_wrote() {
        assert_eq!(n(40.0), "40.00");
        assert_eq!(n(123.456), "123.46");
    }
}
