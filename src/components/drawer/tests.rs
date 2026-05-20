use super::*;

// DrawerPlacement tests
#[test]
fn test_drawer_placement_default() {
    let val = DrawerPlacement::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_drawer_placement_start() {
    let val = DrawerPlacement::Start;
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_drawer_placement_end() {
    let val = DrawerPlacement::End;
    assert_eq!(val.as_str(), "drawer-end");
}

#[test]
fn test_drawer_placement_clone() {
    let v1 = DrawerPlacement::End;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_drawer_placement_debug() {
    let val = DrawerPlacement::End;
    assert!(format!("{:?}", val).contains("End"));
}

// Comprehensive coverage test
#[test]
fn test_all_drawer_placements_return_valid_classes() {
    let variants = vec![
        (DrawerPlacement::Start, ""),
        (DrawerPlacement::End, "drawer-end"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
