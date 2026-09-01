//! The one place a categorical chart value or an axis name becomes text.
//!
//! With two value axes a chart states numbers in four places at once — the
//! tick scale, the hover card, the accessible hidden table, and the typed
//! activation payload — and each of them must state a series' value against
//! *its own* axis. Formatting them separately is how a unit ends up on the
//! ticks but not in the table, or a duration gets read as a count. Every one
//! of those surfaces therefore calls [`value_text`] or [`tick_text`] with the
//! axis options the series was normalized against, so a unit or precision is
//! written once and reaches all of them.
//!
//! The fallbacks are chosen to reproduce the pre-existing output exactly when
//! a caller configures nothing: a value renders as `f64::to_string` did, and a
//! tick renders with one decimal as its `format!` did.

use super::types::{LineAxes, LineAxisOptions, LineChartTexts, LineValueAxis};

/// Rust's formatting precision is bounded well below this; the clamp only
/// keeps an absurd caller value from allocating a gigantic string.
const MAX_DECIMALS: usize = 17;

fn format_value(value: f64, options: &LineAxisOptions, fallback_decimals: Option<usize>) -> String {
    let mut text = match options.decimals.or(fallback_decimals) {
        Some(decimals) => format!(
            "{value:.precision$}",
            precision = decimals.min(MAX_DECIMALS)
        ),
        None => value.to_string(),
    };
    if let Some(unit) = options.unit.as_deref().filter(|unit| !unit.is_empty()) {
        text.push_str(unit);
    }
    text
}

/// Formats a data value for a reader: the hover card, the hidden table, the
/// focus target's accessible name, and the activation payload.
pub(super) fn value_text(value: f64, options: &LineAxisOptions) -> String {
    format_value(value, options, None)
}

/// Formats an axis scale value for a tick label.
pub(super) fn tick_text(value: f64, options: &LineAxisOptions) -> String {
    format_value(value, options, Some(1))
}

/// The reader-facing name of `axis`: its own label when the caller set one,
/// else the localized fallback from [`LineChartTexts`].
pub(super) fn axis_name(axis: LineValueAxis, axes: &LineAxes, texts: &LineChartTexts) -> String {
    axes.options(axis)
        .label
        .clone()
        .unwrap_or_else(|| match axis {
            LineValueAxis::Primary => texts.primary_axis.clone(),
            LineValueAxis::Secondary => texts.secondary_axis.clone(),
        })
}

/// A series caption that names the axis its numbers belong to.
///
/// Used by the legend and the hidden table so a reader meets one attribution,
/// not two. Only reached when the chart actually renders two axes — a
/// single-axis chart keeps the bare series name it always had.
pub(super) fn series_caption(series_name: &str, axis_name: &str) -> String {
    format!("{series_name} ({axis_name})")
}

/// The machine-readable axis token written to `data-axis` on the elements that
/// carry a series' value, so a browser test locates an axis by identity rather
/// than by position.
pub(super) fn axis_token(axis: LineValueAxis) -> &'static str {
    match axis {
        LineValueAxis::Primary => "primary",
        LineValueAxis::Secondary => "secondary",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unconfigured_axis_formats_values_exactly_as_the_chart_always_did() {
        // The regression this guards: routing every value through one
        // formatter must not restate a single existing caller's numbers.
        let options = LineAxisOptions::default();
        for value in [0.0, 42.0, -3.5, 1234.5678, 0.125] {
            assert_eq!(value_text(value, &options), value.to_string(), "{value}");
        }
    }

    #[test]
    fn an_unconfigured_axis_formats_ticks_with_the_one_decimal_it_always_did() {
        let options = LineAxisOptions::default();
        for value in [0.0, 42.0, -3.5, 17.25] {
            assert_eq!(tick_text(value, &options), format!("{value:.1}"), "{value}");
        }
    }

    #[test]
    fn a_unit_reaches_ticks_and_values_from_one_place() {
        let options = LineAxisOptions::default().with_unit(" s").with_decimals(2);

        assert_eq!(value_text(1.5, &options), "1.50 s");
        assert_eq!(tick_text(1.5, &options), "1.50 s");
    }

    #[test]
    fn a_unit_is_appended_verbatim_so_the_caller_owns_the_separator() {
        let percent = LineAxisOptions::default().with_unit("%").with_decimals(0);
        assert_eq!(value_text(37.4, &percent), "37%");

        let spaced = LineAxisOptions::default().with_unit(" cases");
        assert_eq!(value_text(12.0, &spaced), "12 cases");

        let empty = LineAxisOptions::default().with_unit("");
        assert_eq!(value_text(12.0, &empty), "12", "an empty unit adds nothing");
    }

    #[test]
    fn an_absurd_precision_is_clamped_rather_than_allocating_without_bound() {
        let options = LineAxisOptions::default().with_decimals(usize::MAX);

        assert_eq!(value_text(1.0, &options).len(), 1 + 1 + MAX_DECIMALS);
    }

    #[test]
    fn an_axis_name_prefers_its_label_and_falls_back_to_localized_text() {
        let texts = LineChartTexts {
            primary_axis: "Ejes primario".to_string(),
            secondary_axis: "Eje secundario".to_string(),
            ..LineChartTexts::default()
        };
        let axes = LineAxes {
            primary: LineAxisOptions::default().with_label("Conversations"),
            secondary: LineAxisOptions::default(),
        };

        assert_eq!(
            axis_name(LineValueAxis::Primary, &axes, &texts),
            "Conversations"
        );
        assert_eq!(
            axis_name(LineValueAxis::Secondary, &axes, &texts),
            "Eje secundario",
            "no English is hardcoded at the fallback"
        );
    }

    #[test]
    fn a_series_caption_names_its_axis_once() {
        assert_eq!(
            series_caption("Average first response", "Duration"),
            "Average first response (Duration)"
        );
    }

    #[test]
    fn axis_tokens_are_stable_selectors() {
        assert_eq!(axis_token(LineValueAxis::Primary), "primary");
        assert_eq!(axis_token(LineValueAxis::Secondary), "secondary");
    }
}
