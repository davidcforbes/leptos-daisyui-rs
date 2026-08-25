/// Angular sweep of the gauge arc, in degrees. A ~240-degree open dial with
/// the gap at the bottom, matching the classic instrument-cluster form the
/// 4iiz-etl portal's server gauges use (`ldui-nx5`).
pub const GAUGE_SWEEP_DEG: f64 = 240.0;

/// Angle of the arc's start (fraction 0), in SVG screen coordinates: degrees
/// from the positive x-axis, increasing clockwise because SVG's y-axis points
/// down. 150 degrees is the bottom-left shoulder; sweeping 240 degrees
/// clockwise passes the top and ends at the bottom-right shoulder.
pub const GAUGE_START_DEG: f64 = 150.0;

/// Fraction of the dial a value covers: `value / max` clamped to `[0, 1]`.
/// Returns `0.0` for a non-positive `max` (never a divide-by-zero) and for a
/// `NaN` value, mirroring `capacity_bar_percent`'s defensive posture.
pub fn gauge_fraction(value: f64, max: f64) -> f64 {
    if max > 0.0 && value.is_finite() {
        (value / max).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Point on the dial circle at `frac` of the sweep, in viewBox units.
pub fn gauge_point(cx: f64, cy: f64, r: f64, frac: f64) -> (f64, f64) {
    let angle =
        (GAUGE_START_DEG + GAUGE_SWEEP_DEG * frac.clamp(0.0, 1.0)).to_radians();
    (cx + r * angle.cos(), cy + r * angle.sin())
}

/// SVG path for the arc from `from` to `to` (fractions of the sweep, both
/// clamped to `[0, 1]`). Returns an empty string when the span is empty or
/// inverted, so a caller can bind it straight to `d=` and get no mark rather
/// than a degenerate arc.
pub fn gauge_arc_path(cx: f64, cy: f64, r: f64, from: f64, to: f64) -> String {
    let from = from.clamp(0.0, 1.0);
    let to = to.clamp(0.0, 1.0);
    if to <= from {
        return String::new();
    }
    let (x0, y0) = gauge_point(cx, cy, r, from);
    let (x1, y1) = gauge_point(cx, cy, r, to);
    let large_arc = if (to - from) * GAUGE_SWEEP_DEG > 180.0 { 1 } else { 0 };
    format!("M {x0:.2} {y0:.2} A {r:.2} {r:.2} 0 {large_arc} 1 {x1:.2} {y1:.2}")
}

/// `(warn_band, error_band)` as `(from, to)` sweep fractions, from the
/// threshold fractions. The warn band runs from `warn_from` to the error
/// threshold (or the end of the dial); the error band runs from `error_from`
/// to the end. Thresholds are clamped to `[0, 1]`; an empty or inverted band
/// (e.g. `warn_from` at or past `error_from`) is dropped rather than drawn
/// backwards.
pub fn gauge_bands(
    warn_from: Option<f64>,
    error_from: Option<f64>,
) -> (Option<(f64, f64)>, Option<(f64, f64)>) {
    let error = error_from.map(|f| f.clamp(0.0, 1.0));
    let warn = warn_from.map(|f| f.clamp(0.0, 1.0));
    let warn_end = error.unwrap_or(1.0);
    let warn_band = warn.filter(|from| *from < warn_end).map(|from| (from, warn_end));
    let error_band = error.filter(|from| *from < 1.0).map(|from| (from, 1.0));
    (warn_band, error_band)
}

/// Theme-token paint for the value arc: primary while under budget, the
/// warning tone once the value enters the warn band, the error tone once it
/// enters the error band. Factual zone coloring, same spirit as
/// `CapacityBarColor::for_direction` -- the dial should not stay serenely
/// primary while the needle sits in a red zone.
pub fn gauge_value_paint(
    frac: f64,
    warn_from: Option<f64>,
    error_from: Option<f64>,
) -> &'static str {
    if error_from.is_some_and(|from| frac >= from.clamp(0.0, 1.0)) {
        "var(--color-error)"
    } else if warn_from.is_some_and(|from| frac >= from.clamp(0.0, 1.0)) {
        "var(--color-warning)"
    } else {
        "var(--color-primary)"
    }
}

/// Default readout formatting for the center number: whole numbers at ten
/// and above (a `87.3%` CPU reads `87`), one decimal below ten with a
/// trailing `.0` trimmed, and an en-dash placeholder for a non-finite value.
/// Hosts with their own display strings pass `display` instead.
pub fn gauge_readout(value: f64) -> String {
    if !value.is_finite() {
        return "\u{2013}".to_string();
    }
    if value.abs() >= 10.0 {
        format!("{value:.0}")
    } else {
        let text = format!("{value:.1}");
        text.strip_suffix(".0")
            .map(|s| s.to_string())
            .unwrap_or(text)
    }
}
