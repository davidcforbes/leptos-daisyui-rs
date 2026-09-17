//! Where a hovering tooltip bubble should hang off its trigger, decided from
//! real geometry -- promoted from `RecordHeader` (ldui-q73d) so `KpiStrip`
//! can use the same answer (ldui-rzvv). Never from an element's position in
//! a list: whichever trigger ends up nearest an edge is the one that flips,
//! whatever its index, which is the only rule that survives wrapping.

use super::TooltipPosition;

/// Conservative estimate of half the width (CSS px) a hovering daisyUI
/// tooltip bubble will render at, from the length of the text alone.
///
/// DaisyUI's tooltip has no DOM node to measure ahead of time -- its
/// visible content is a `::before` pseudo-element whose text comes from a
/// `data-tip` attribute -- so there is nothing a real
/// `getBoundingClientRect` call can read before the control is hovered or
/// focused. This stands in: average glyph width plus the bubble's own
/// inline padding (`padding-inline: .5rem` each side in daisyUI's tooltip
/// CSS), capped at daisyUI's own `max-width: 20rem` for the bubble. It
/// leans wide on purpose -- overestimating only flips a control that would
/// in fact have fit without flipping, while underestimating lets the exact
/// spill this exists to prevent back in.
pub fn estimated_tooltip_half_width(label_len: usize) -> f64 {
    const AVG_GLYPH_PX: f64 = 7.0;
    const BUBBLE_INLINE_PADDING_PX: f64 = 16.0;
    const MAX_BUBBLE_WIDTH_PX: f64 = 320.0;
    let natural = (label_len as f64) * AVG_GLYPH_PX + BUBBLE_INLINE_PADDING_PX;
    natural.min(MAX_BUBBLE_WIDTH_PX) / 2.0
}

/// Chooses which edge of the trigger a tooltip bubble should hang off of,
/// from real geometry. A trigger with less than `half_width` of room to the
/// row's right edge flips to `Left`; one with less room to the left (an
/// unusual layout, but not an impossible one) flips to `Right`; a trigger
/// with room on both sides keeps daisyUI's own default `Top`, centred on
/// the trigger.
pub fn resolved_tooltip_position(
    row_left: f64,
    row_right: f64,
    trigger_left: f64,
    trigger_width: f64,
    half_width: f64,
) -> TooltipPosition {
    let trigger_center = trigger_left + trigger_width / 2.0;
    let right_room = row_right - trigger_center;
    let left_room = trigger_center - row_left;
    if right_room < half_width && left_room >= half_width {
        TooltipPosition::Left
    } else if left_room < half_width && right_room >= half_width {
        TooltipPosition::Right
    } else {
        TooltipPosition::Top
    }
}

/// Measures `bounds` (the element the bubble must stay inside) and
/// `trigger` (the tooltip's wrapping element) and returns the placement.
/// A hovering-but-invisible bubble still occupies layout space, so a
/// caller measures on mount, on every reflow of `bounds`, and again on
/// hover/focus as a belt-and-braces pass.
pub fn measure_tooltip_position(
    bounds: &web_sys::Element,
    trigger: &web_sys::Element,
    label_len: usize,
) -> TooltipPosition {
    let bounds_rect = bounds.get_bounding_client_rect();
    let trigger_rect = trigger.get_bounding_client_rect();
    resolved_tooltip_position(
        bounds_rect.left(),
        bounds_rect.right(),
        trigger_rect.left(),
        trigger_rect.width(),
        estimated_tooltip_half_width(label_len),
    )
}
