/// # Tooltip Position Variants
///
/// Position variants for daisyUI tooltip component that control where the tooltip appears
/// relative to its trigger element.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum TooltipPosition {
    /// Tooltip appears above the element (default)
    #[default]
    Top,

    /// Tooltip appears below the element
    Bottom,

    /// Tooltip appears to the left of the element
    Left,

    /// Tooltip appears to the right of the element
    Right,
}

impl TooltipPosition {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            TooltipPosition::Top => "tooltip-top",
            TooltipPosition::Bottom => "tooltip-bottom",
            TooltipPosition::Left => "tooltip-left",
            TooltipPosition::Right => "tooltip-right",
        }
    }

    /// The placement as a bare word, emitted as `data-tooltip-placement` so
    /// a test or a consumer can read the RESOLVED side without parsing a
    /// merged utility-class string (the `data-section-grid-columns`
    /// precedent).
    pub const fn as_placement(&self) -> &'static str {
        match self {
            TooltipPosition::Top => "top",
            TooltipPosition::Bottom => "bottom",
            TooltipPosition::Left => "left",
            TooltipPosition::Right => "right",
        }
    }
}

/// # Tooltip Container Edge
///
/// Which edge of its CONTAINER a tooltip trigger is pressed against, so the
/// placement can be flipped inward before the bubble opens off the edge
/// (Office op-du33s: the trailing card of a `KpiStrip` opened its help
/// tooltip past the strip's right edge, where the page clipped it).
///
/// daisyUI's tooltip is a `:before`/`:after` pseudo-element positioned by a
/// static class; there is no measurement and no collision detection in the
/// component. The trigger's owner is the only thing that knows whether it
/// sits against an edge, so it declares it here and the component picks the
/// side -- no JS measurement, no ResizeObserver, one class either way.
///
/// The edges are PHYSICAL (left/right), matching daisyUI's physical
/// `tooltip-left`/`tooltip-right` classes, not writing-direction logical
/// sides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TooltipEdge {
    /// Not against either edge: the preferred placement is used unchanged.
    #[default]
    Interior,

    /// The trigger hugs the container's LEFT edge, so a horizontally
    /// centered bubble (`tooltip-top`/`tooltip-bottom`) would spill off the
    /// left. The bubble opens to the right instead.
    Left,

    /// The trigger hugs the container's RIGHT edge. The bubble opens to the
    /// left instead.
    Right,

    /// The trigger hugs BOTH edges -- a container no wider than the trigger
    /// itself, where no flip buys any room. The preferred placement is kept,
    /// deliberately: an arbitrary flip would only move the overflow.
    Both,
}

impl TooltipEdge {
    /// The edge as a bare word, for `data-tooltip-edge`.
    pub const fn as_str(&self) -> &'static str {
        match self {
            TooltipEdge::Interior => "interior",
            TooltipEdge::Left => "left",
            TooltipEdge::Right => "right",
            TooltipEdge::Both => "both",
        }
    }
}

/// Flips a preferred placement whose natural side would overflow the named
/// container edge, and returns the placement that actually ships.
///
/// The rule is one sentence: **a bubble never grows toward an edge the
/// trigger is already touching.**
///
/// * `Top`/`Bottom` are horizontally CENTERED on the trigger, so half the
///   bubble (up to 10rem of daisyUI's `max-width`) crosses whichever edge is
///   near -- both flip to the horizontal placement that grows inward.
/// * `Left` against the left edge flips to `Right`, and `Right` against the
///   right edge flips to `Left`.
/// * A placement already growing inward is returned unchanged, so the
///   function is idempotent: applying it twice with the same edge changes
///   nothing.
/// * [`TooltipEdge::Interior`] and [`TooltipEdge::Both`] return the
///   preferred placement untouched.
pub const fn tooltip_position_for_edge(
    preferred: &TooltipPosition,
    edge: TooltipEdge,
) -> TooltipPosition {
    match edge {
        TooltipEdge::Interior | TooltipEdge::Both => match preferred {
            TooltipPosition::Top => TooltipPosition::Top,
            TooltipPosition::Bottom => TooltipPosition::Bottom,
            TooltipPosition::Left => TooltipPosition::Left,
            TooltipPosition::Right => TooltipPosition::Right,
        },
        // Against the left edge: nothing may grow leftward.
        TooltipEdge::Left => TooltipPosition::Right,
        // Against the right edge: nothing may grow rightward.
        TooltipEdge::Right => TooltipPosition::Left,
    }
}

/// # Tooltip Color Variants
///
/// Color variants for daisyUI tooltip component that control the background color
/// of the tooltip bubble.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum TooltipColor {
    /// Default tooltip color (theme-dependent)
    #[default]
    Default,

    /// Neutral color
    Neutral,

    /// Primary brand color
    Primary,

    /// Secondary brand color
    Secondary,

    /// Accent brand color
    Accent,

    /// Info color for informational tooltips
    Info,

    /// Success color for positive tooltips
    Success,

    /// Warning color for caution tooltips
    Warning,

    /// Error color for error tooltips
    Error,
}

impl TooltipColor {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            TooltipColor::Default => "",
            TooltipColor::Neutral => "tooltip-neutral",
            TooltipColor::Primary => "tooltip-primary",
            TooltipColor::Secondary => "tooltip-secondary",
            TooltipColor::Accent => "tooltip-accent",
            TooltipColor::Info => "tooltip-info",
            TooltipColor::Success => "tooltip-success",
            TooltipColor::Warning => "tooltip-warning",
            TooltipColor::Error => "tooltip-error",
        }
    }
}
