use super::*;

// ── next_enabled_index (ldui-jcs.21) ────────────────────────────────────────

#[test]
fn next_enabled_index_on_empty_list_is_none() {
    assert_eq!(next_enabled_index(None, &[], true), None);
    assert_eq!(next_enabled_index(Some(0), &[], false), None);
}

#[test]
fn next_enabled_index_all_disabled_is_none() {
    let disabled = [true, true, true];
    assert_eq!(next_enabled_index(None, &disabled, true), None);
    assert_eq!(next_enabled_index(Some(1), &disabled, false), None);
}

#[test]
fn next_enabled_index_from_none_forward_lands_on_first_enabled() {
    let disabled = [false, false, false];
    assert_eq!(next_enabled_index(None, &disabled, true), Some(0));
}

#[test]
fn next_enabled_index_from_none_backward_lands_on_last_enabled() {
    let disabled = [false, false, false];
    assert_eq!(next_enabled_index(None, &disabled, false), Some(2));
}

#[test]
fn next_enabled_index_steps_forward() {
    let disabled = [false, false, false];
    assert_eq!(next_enabled_index(Some(0), &disabled, true), Some(1));
    assert_eq!(next_enabled_index(Some(1), &disabled, true), Some(2));
}

#[test]
fn next_enabled_index_steps_backward() {
    let disabled = [false, false, false];
    assert_eq!(next_enabled_index(Some(2), &disabled, false), Some(1));
    assert_eq!(next_enabled_index(Some(1), &disabled, false), Some(0));
}

#[test]
fn next_enabled_index_wraps_forward() {
    let disabled = [false, false, false];
    assert_eq!(next_enabled_index(Some(2), &disabled, true), Some(0));
}

#[test]
fn next_enabled_index_wraps_backward() {
    let disabled = [false, false, false];
    assert_eq!(next_enabled_index(Some(0), &disabled, false), Some(2));
}

#[test]
fn next_enabled_index_skips_disabled_items() {
    // index 1 is disabled — stepping forward from 0 should land on 2.
    let disabled = [false, true, false, false];
    assert_eq!(next_enabled_index(Some(0), &disabled, true), Some(2));
}

#[test]
fn next_enabled_index_skips_disabled_when_wrapping() {
    // Only index 0 is disabled; stepping backward from 1 wraps past 0 to 3.
    let disabled = [true, false, false, false];
    assert_eq!(next_enabled_index(Some(1), &disabled, false), Some(3));
}

#[test]
fn next_enabled_index_single_enabled_item_stays_put() {
    let disabled = [true, false, true];
    assert_eq!(next_enabled_index(Some(1), &disabled, true), Some(1));
    assert_eq!(next_enabled_index(None, &disabled, true), Some(1));
}

// ── first_enabled_index / last_enabled_index (ldui-jcs.21) ─────────────────

#[test]
fn first_enabled_index_empty_is_none() {
    assert_eq!(first_enabled_index(&[]), None);
}

#[test]
fn first_enabled_index_all_disabled_is_none() {
    assert_eq!(first_enabled_index(&[true, true]), None);
}

#[test]
fn first_enabled_index_skips_leading_disabled() {
    assert_eq!(first_enabled_index(&[true, true, false, false]), Some(2));
}

#[test]
fn last_enabled_index_empty_is_none() {
    assert_eq!(last_enabled_index(&[]), None);
}

#[test]
fn last_enabled_index_all_disabled_is_none() {
    assert_eq!(last_enabled_index(&[true, true]), None);
}

#[test]
fn last_enabled_index_skips_trailing_disabled() {
    assert_eq!(last_enabled_index(&[false, false, true, true]), Some(1));
}

// MenuDirection tests
#[test]
fn test_menu_direction_default() {
    let val = MenuDirection::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_menu_direction_vertical() {
    let val = MenuDirection::Vertical;
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_menu_direction_horizontal() {
    let val = MenuDirection::Horizontal;
    assert_eq!(val.as_str(), "menu-horizontal");
}

#[test]
fn test_menu_direction_clone() {
    let v1 = MenuDirection::Horizontal;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_menu_direction_debug() {
    let val = MenuDirection::Horizontal;
    assert!(format!("{:?}", val).contains("Horizontal"));
}

// MenuSize tests
#[test]
fn test_menu_size_default() {
    let val = MenuSize::default();
    assert_eq!(val.as_str(), "menu-md");
}

#[test]
fn test_menu_size_xs() {
    let val = MenuSize::Xs;
    assert_eq!(val.as_str(), "menu-xs");
}

#[test]
fn test_menu_size_sm() {
    let val = MenuSize::Sm;
    assert_eq!(val.as_str(), "menu-sm");
}

#[test]
fn test_menu_size_md() {
    let val = MenuSize::Md;
    assert_eq!(val.as_str(), "menu-md");
}

#[test]
fn test_menu_size_lg() {
    let val = MenuSize::Lg;
    assert_eq!(val.as_str(), "menu-lg");
}

#[test]
fn test_menu_size_xl() {
    let val = MenuSize::Xl;
    assert_eq!(val.as_str(), "menu-xl");
}

#[test]
fn test_menu_size_clone() {
    let v1 = MenuSize::Lg;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_menu_size_debug() {
    let val = MenuSize::Xl;
    assert!(format!("{:?}", val).contains("Xl"));
}

// Comprehensive coverage tests
#[test]
fn test_all_menu_directions_return_valid_classes() {
    let variants = vec![
        (MenuDirection::Vertical, ""),
        (MenuDirection::Horizontal, "menu-horizontal"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_menu_sizes_return_valid_classes() {
    let variants = vec![
        (MenuSize::Xs, "menu-xs"),
        (MenuSize::Sm, "menu-sm"),
        (MenuSize::Md, "menu-md"),
        (MenuSize::Lg, "menu-lg"),
        (MenuSize::Xl, "menu-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
