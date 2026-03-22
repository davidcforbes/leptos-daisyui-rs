use super::*;

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
