/// Sum of every segment's value, each floored at `0.0` so a stray negative
/// input can't shrink the denominator (and therefore inflate every other
/// segment's share). Mirrors `capacity_bar`'s non-negative-clamp discipline.
pub fn segmented_bar_total(segments: &[(f64, &str)]) -> f64 {
    segments.iter().map(|(v, _)| v.max(0.0)).sum()
}

/// One segment's width as a percentage of the track -- `value`'s share of
/// `total`, clamped to `[0, 100]`. Returns `0.0` (never divides by zero) when
/// `total` is non-positive or `value` itself is non-positive, so an all-zero
/// or empty `segments` list renders an empty track rather than panicking or
/// drawing a misleadingly full bar.
pub fn segmented_bar_percent(value: f64, total: f64) -> f64 {
    if total > 0.0 && value > 0.0 {
        (value / total * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    }
}
