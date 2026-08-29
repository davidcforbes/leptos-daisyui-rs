use super::*;

// TabSize tests
#[test]
fn test_tab_size_default() {
    let size = TabSize::default();
    assert_eq!(size.as_str(), "tabs-md");
}

#[test]
fn test_tab_size_xs() {
    assert_eq!(TabSize::Xs.as_str(), "tabs-xs");
}

#[test]
fn test_tab_size_sm() {
    assert_eq!(TabSize::Sm.as_str(), "tabs-sm");
}

#[test]
fn test_tab_size_md() {
    assert_eq!(TabSize::Md.as_str(), "tabs-md");
}

#[test]
fn test_tab_size_lg() {
    assert_eq!(TabSize::Lg.as_str(), "tabs-lg");
}

#[test]
fn test_tab_size_xl() {
    assert_eq!(TabSize::Xl.as_str(), "tabs-xl");
}

#[test]
fn test_tab_size_clone() {
    let s1 = TabSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_tab_size_debug() {
    let size = TabSize::Xl;
    assert!(format!("{:?}", size).contains("Xl"));
}

// TabVariant tests
#[test]
fn test_tab_variant_default() {
    let variant = TabVariant::default();
    assert_eq!(variant.as_str(), "");
}

#[test]
fn test_tab_variant_boxed() {
    assert_eq!(TabVariant::Boxed.as_str(), "tabs-box");
}

#[test]
fn test_tab_variant_border() {
    assert_eq!(TabVariant::Border.as_str(), "tabs-border");
}

#[test]
fn test_tab_variant_lift() {
    assert_eq!(TabVariant::Lift.as_str(), "tabs-lift");
}

#[test]
fn test_tab_variant_clone() {
    let v1 = TabVariant::Boxed;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_tab_variant_debug() {
    let variant = TabVariant::Lift;
    assert!(format!("{:?}", variant).contains("Lift"));
}

// TabPlacement tests
#[test]
fn test_tab_placement_default() {
    let placement = TabPlacement::default();
    assert_eq!(placement.as_str(), "tabs-top");
}

#[test]
fn test_tab_placement_top() {
    assert_eq!(TabPlacement::Top.as_str(), "tabs-top");
}

#[test]
fn test_tab_placement_bottom() {
    assert_eq!(TabPlacement::Bottom.as_str(), "tabs-bottom");
}

#[test]
fn test_tab_placement_clone() {
    let p1 = TabPlacement::Bottom;
    let p2 = p1.clone();
    assert_eq!(p1.as_str(), p2.as_str());
}

#[test]
fn test_tab_placement_debug() {
    let placement = TabPlacement::Bottom;
    assert!(format!("{:?}", placement).contains("Bottom"));
}

#[test]
fn test_all_tab_sizes_return_valid_classes() {
    let variants = vec![
        (TabSize::Xs, "tabs-xs"),
        (TabSize::Sm, "tabs-sm"),
        (TabSize::Md, "tabs-md"),
        (TabSize::Lg, "tabs-lg"),
        (TabSize::Xl, "tabs-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_tab_variants_return_valid_classes() {
    let variants = vec![
        (TabVariant::Default, ""),
        (TabVariant::Boxed, "tabs-box"),
        (TabVariant::Border, "tabs-border"),
        (TabVariant::Lift, "tabs-lift"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_tab_placements_return_valid_classes() {
    let variants = vec![
        (TabPlacement::Top, "tabs-top"),
        (TabPlacement::Bottom, "tabs-bottom"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn controlled_tab_orientation_exposes_aria_values() {
    assert_eq!(TabOrientation::Horizontal.as_str(), "horizontal");
    assert_eq!(TabOrientation::Vertical.as_str(), "vertical");
    assert_eq!(TabOrientation::default(), TabOrientation::Horizontal);
}

#[test]
fn controlled_tab_dom_ids_are_collision_safe_for_stable_keys() {
    use super::component::{tab_dom_id, tab_panel_dom_id};

    assert_eq!(
        tab_dom_id("account", "overview"),
        "account-tab-6f76657276696577"
    );
    assert_eq!(
        tab_panel_dom_id("account", "overview"),
        "account-panel-6f76657276696577"
    );
    assert_ne!(tab_dom_id("account", "a/b"), tab_dom_id("account", "a-b"));
}

#[test]
fn controlled_tab_navigation_wraps_and_skips_disabled_items() {
    use super::component::{TabMove, next_enabled_tab};

    let disabled = [false, true, false, false];
    assert_eq!(next_enabled_tab(Some(0), &disabled, TabMove::Next), Some(2));
    assert_eq!(next_enabled_tab(Some(3), &disabled, TabMove::Next), Some(0));
    assert_eq!(
        next_enabled_tab(Some(0), &disabled, TabMove::Previous),
        Some(3)
    );
    assert_eq!(next_enabled_tab(None, &disabled, TabMove::Home), Some(0));
    assert_eq!(next_enabled_tab(None, &disabled, TabMove::End), Some(3));
    assert_eq!(next_enabled_tab(None, &[true, true], TabMove::Next), None);
}
