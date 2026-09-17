use super::*;

// TooltipPosition tests
#[test]
fn test_tooltip_position_default() {
    let pos = TooltipPosition::default();
    assert_eq!(pos.as_str(), "tooltip-top");
}

#[test]
fn test_tooltip_position_top() {
    assert_eq!(TooltipPosition::Top.as_str(), "tooltip-top");
}

#[test]
fn test_tooltip_position_bottom() {
    assert_eq!(TooltipPosition::Bottom.as_str(), "tooltip-bottom");
}

#[test]
fn test_tooltip_position_left() {
    assert_eq!(TooltipPosition::Left.as_str(), "tooltip-left");
}

#[test]
fn test_tooltip_position_right() {
    assert_eq!(TooltipPosition::Right.as_str(), "tooltip-right");
}

#[test]
fn test_tooltip_position_clone() {
    let p1 = TooltipPosition::Left;
    let p2 = p1.clone();
    assert_eq!(p1.as_str(), p2.as_str());
}

#[test]
fn test_tooltip_position_debug() {
    let pos = TooltipPosition::Right;
    assert!(format!("{:?}", pos).contains("Right"));
}

// TooltipColor tests
#[test]
fn test_tooltip_color_default() {
    let color = TooltipColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_tooltip_color_neutral() {
    assert_eq!(TooltipColor::Neutral.as_str(), "tooltip-neutral");
}

#[test]
fn test_tooltip_color_primary() {
    assert_eq!(TooltipColor::Primary.as_str(), "tooltip-primary");
}

#[test]
fn test_tooltip_color_secondary() {
    assert_eq!(TooltipColor::Secondary.as_str(), "tooltip-secondary");
}

#[test]
fn test_tooltip_color_accent() {
    assert_eq!(TooltipColor::Accent.as_str(), "tooltip-accent");
}

#[test]
fn test_tooltip_color_info() {
    assert_eq!(TooltipColor::Info.as_str(), "tooltip-info");
}

#[test]
fn test_tooltip_color_success() {
    assert_eq!(TooltipColor::Success.as_str(), "tooltip-success");
}

#[test]
fn test_tooltip_color_warning() {
    assert_eq!(TooltipColor::Warning.as_str(), "tooltip-warning");
}

#[test]
fn test_tooltip_color_error() {
    assert_eq!(TooltipColor::Error.as_str(), "tooltip-error");
}

#[test]
fn test_tooltip_color_clone() {
    let c1 = TooltipColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_tooltip_color_debug() {
    let color = TooltipColor::Warning;
    assert!(format!("{:?}", color).contains("Warning"));
}

#[test]
fn test_all_tooltip_positions_return_valid_classes() {
    let variants = vec![
        (TooltipPosition::Top, "tooltip-top"),
        (TooltipPosition::Bottom, "tooltip-bottom"),
        (TooltipPosition::Left, "tooltip-left"),
        (TooltipPosition::Right, "tooltip-right"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_tooltip_colors_return_valid_classes() {
    let variants = vec![
        (TooltipColor::Default, ""),
        (TooltipColor::Neutral, "tooltip-neutral"),
        (TooltipColor::Primary, "tooltip-primary"),
        (TooltipColor::Secondary, "tooltip-secondary"),
        (TooltipColor::Accent, "tooltip-accent"),
        (TooltipColor::Info, "tooltip-info"),
        (TooltipColor::Success, "tooltip-success"),
        (TooltipColor::Warning, "tooltip-warning"),
        (TooltipColor::Error, "tooltip-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

// TooltipEdge / edge-aware placement tests (Office op-du33s)

/// The defect: a `KpiStrip`'s trailing help trigger opened a horizontally
/// centered `tooltip-top` bubble off the strip's right edge. A trigger
/// against the right edge must grow LEFT, and one against the left edge must
/// grow RIGHT, whatever placement was preferred.
///
/// BREAK: return `preferred` for `TooltipEdge::Right` in
/// `tooltip_position_for_edge`; the first assertion fails with
/// `tooltip-top`.
#[test]
fn a_centered_placement_flips_inward_at_a_container_edge() {
    assert_eq!(
        tooltip_position_for_edge(&TooltipPosition::Top, TooltipEdge::Right).as_str(),
        "tooltip-left"
    );
    assert_eq!(
        tooltip_position_for_edge(&TooltipPosition::Bottom, TooltipEdge::Right).as_str(),
        "tooltip-left"
    );
    assert_eq!(
        tooltip_position_for_edge(&TooltipPosition::Top, TooltipEdge::Left).as_str(),
        "tooltip-right"
    );
    assert_eq!(
        tooltip_position_for_edge(&TooltipPosition::Bottom, TooltipEdge::Left).as_str(),
        "tooltip-right"
    );
}

/// A horizontal placement that already grows toward the edge is the whole
/// defect one step further on, so it flips too -- and one that already grows
/// inward is left alone, which makes the function idempotent.
///
/// BREAK: make the `TooltipEdge::Left` arm return `TooltipPosition::Left`;
/// the idempotence assertion fails.
#[test]
fn a_horizontal_placement_flips_away_from_its_edge_and_is_idempotent() {
    assert_eq!(
        tooltip_position_for_edge(&TooltipPosition::Left, TooltipEdge::Left).as_str(),
        "tooltip-right"
    );
    assert_eq!(
        tooltip_position_for_edge(&TooltipPosition::Right, TooltipEdge::Right).as_str(),
        "tooltip-left"
    );

    for edge in [
        TooltipEdge::Interior,
        TooltipEdge::Left,
        TooltipEdge::Right,
        TooltipEdge::Both,
    ] {
        for preferred in [
            TooltipPosition::Top,
            TooltipPosition::Bottom,
            TooltipPosition::Left,
            TooltipPosition::Right,
        ] {
            let once = tooltip_position_for_edge(&preferred, edge);
            let twice = tooltip_position_for_edge(&once, edge);
            assert_eq!(once, twice, "not idempotent for {preferred:?} at {edge:?}");
        }
    }
}

/// `Interior` is the default and must be a pure pass-through, or adding the
/// prop would silently move every existing tooltip in the portfolio.
/// `Both` keeps the preferred side deliberately: in a container no wider
/// than the trigger, a flip only moves the overflow to the other edge.
///
/// BREAK: give `TooltipEdge::Both` its own flip arm; the `Both` assertion
/// fails.
#[test]
fn interior_and_both_leave_the_preferred_placement_untouched() {
    assert_eq!(TooltipEdge::default(), TooltipEdge::Interior);
    for preferred in [
        TooltipPosition::Top,
        TooltipPosition::Bottom,
        TooltipPosition::Left,
        TooltipPosition::Right,
    ] {
        assert_eq!(
            tooltip_position_for_edge(&preferred, TooltipEdge::Interior),
            preferred
        );
        assert_eq!(
            tooltip_position_for_edge(&preferred, TooltipEdge::Both),
            preferred
        );
    }
}

/// The resolved side is published as a bare word so a DOM oracle reads
/// `data-tooltip-placement` instead of parsing merged utility classes, and
/// the two spellings never drift apart.
///
/// BREAK: return `"top"` from `TooltipPosition::Left::as_placement`; the
/// suffix assertion fails.
#[test]
fn the_placement_word_matches_its_class() {
    for position in [
        TooltipPosition::Top,
        TooltipPosition::Bottom,
        TooltipPosition::Left,
        TooltipPosition::Right,
    ] {
        assert_eq!(
            position.as_str(),
            format!("tooltip-{}", position.as_placement())
        );
    }
    for edge in [
        TooltipEdge::Interior,
        TooltipEdge::Left,
        TooltipEdge::Right,
        TooltipEdge::Both,
    ] {
        assert!(!edge.as_str().is_empty());
    }
}

/// The component must emit exactly ONE `tooltip-*` position class: two
/// equal-specificity position classes on one element would be resolved by
/// stylesheet order, which is not something this crate controls.
///
/// BREAK: add a second literal `tooltip-top` to the component's
/// `merge_classes!` call; the count assertion fails.
#[test]
fn the_component_emits_exactly_one_position_class() {
    let source = include_str!("component.rs");
    let markup = source
        .split_once("-> impl IntoView {")
        .expect("component body")
        .1;
    let literals = markup.matches("\"tooltip-").count();
    assert_eq!(
        literals, 0,
        "the position class comes from the resolved placement, never a literal: {markup}"
    );
    assert!(
        markup.contains("tooltip_position_for_edge(&position.get(), edge.get())"),
        "the placement must be resolved through the edge-aware helper: {markup}"
    );
}
