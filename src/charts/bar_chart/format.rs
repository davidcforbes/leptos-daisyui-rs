//! The one place a bar chart value becomes text.
//!
//! A bar chart states a number in four places at once: the value label drawn
//! beside the bar, the focus target's accessible name, the hidden data table,
//! and the typed activation payload. Formatting them separately is how a unit
//! reaches the drawn label but not the table, or how a percentage is read out
//! as a bare count. All four call [`value_text`] with the same
//! [`BarValueFormat`], so a unit or a precision is declared once and reaches
//! every one of them.
//!
//! The fallback is chosen to reproduce the chart's existing output exactly when
//! a caller configures nothing: the original renderer wrote its value labels
//! with `format!("{value:.1}")`, so unset `decimals` means one decimal.

use super::types::BarValueFormat;

/// Rust's formatting precision is bounded well below this; the clamp only keeps
/// an absurd caller value from allocating a gigantic string.
const MAX_DECIMALS: usize = 17;

/// The chart's original value-label precision, kept as the unset default.
const DEFAULT_DECIMALS: usize = 1;

/// Formats `value` for a reader: the drawn label, the accessible name, the
/// hidden table cell and the activation payload.
pub(super) fn value_text(value: f64, format: &BarValueFormat) -> String {
    let decimals = format.decimals.unwrap_or(DEFAULT_DECIMALS);
    let mut text = format!(
        "{value:.precision$}",
        precision = decimals.min(MAX_DECIMALS)
    );
    if let Some(unit) = format.unit.as_deref().filter(|unit| !unit.is_empty()) {
        text.push_str(unit);
    }
    text
}

/// The text a bar states: the caller's own `display_value` when it set one,
/// else this chart's formatting of the value, else the localized missing-value
/// copy. Every surface resolves the same way through this function, so the
/// drawn label and the table cell cannot disagree.
pub(super) fn displayed_value(
    value: Option<f64>,
    display_value: Option<&str>,
    format: &BarValueFormat,
    no_value: &str,
) -> String {
    match value {
        Some(value) => display_value
            .map(str::to_owned)
            .unwrap_or_else(|| value_text(value, format)),
        None => display_value
            .map(str::to_owned)
            .unwrap_or_else(|| no_value.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unconfigured_format_writes_the_one_decimal_the_chart_always_wrote() {
        // The regression this guards: routing every value through one formatter
        // must not restate a single existing caller's numbers. The original
        // renderer wrote `format!("{value:.1}")`.
        let format = BarValueFormat::default();
        for value in [0.0, 4.0, 7.0, 5.0, 9.0, -12.5, 1234.5678] {
            assert_eq!(value_text(value, &format), format!("{value:.1}"), "{value}");
        }
    }

    #[test]
    fn a_negative_value_keeps_its_sign_in_text() {
        // Direction is the whole point of a diverging chart; an absolute value
        // in the table would erase it for exactly the readers who cannot see
        // which side of zero the bar is on.
        let format = BarValueFormat::default();
        assert_eq!(value_text(-12.5, &format), "-12.5");
        assert_eq!(value_text(-0.04, &format), "-0.0");
    }

    #[test]
    fn a_unit_and_a_precision_are_declared_once() {
        let format = BarValueFormat::default().with_unit(" pts").with_decimals(2);

        assert_eq!(value_text(-1.5, &format), "-1.50 pts");
        assert_eq!(
            displayed_value(Some(-1.5), None, &format, "No value"),
            "-1.50 pts",
            "every surface resolves through the same formatter"
        );
    }

    #[test]
    fn a_unit_is_appended_verbatim_so_the_caller_owns_the_separator() {
        let percent = BarValueFormat::default().with_unit("%").with_decimals(0);
        assert_eq!(value_text(37.4, &percent), "37%");

        let spaced = BarValueFormat::default()
            .with_unit(" cases")
            .with_decimals(0);
        assert_eq!(value_text(12.0, &spaced), "12 cases");

        let empty = BarValueFormat::default().with_unit("").with_decimals(0);
        assert_eq!(value_text(12.0, &empty), "12", "an empty unit adds nothing");
    }

    #[test]
    fn an_absurd_precision_is_clamped_rather_than_allocating_without_bound() {
        let format = BarValueFormat::default().with_decimals(usize::MAX);

        assert_eq!(value_text(1.0, &format).len(), 1 + 1 + MAX_DECIMALS);
    }

    #[test]
    fn a_callers_display_value_wins_over_the_chart_formatter() {
        let format = BarValueFormat::default().with_unit(" pts");

        assert_eq!(
            displayed_value(Some(-12.5), Some("12.5 behind"), &format, "No value"),
            "12.5 behind"
        );
        assert_eq!(
            displayed_value(Some(-12.5), None, &format, "No value"),
            "-12.5 pts"
        );
    }

    #[test]
    fn a_missing_value_reads_as_missing_and_never_as_zero() {
        // The honest presentation the acceptance criteria require: a gap in the
        // data must not arrive at a reader as a measured zero.
        let format = BarValueFormat::default();

        assert_eq!(displayed_value(None, None, &format, "No value"), "No value");
        assert_eq!(
            displayed_value(None, None, &format, "Sin datos"),
            "Sin datos",
            "the missing-value copy is supplied, not hardcoded"
        );
        assert_eq!(
            displayed_value(None, Some("not reported"), &format, "No value"),
            "not reported",
            "a caller may still name its own gap"
        );
    }
}
