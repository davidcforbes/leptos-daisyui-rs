use super::*;

// FabDirection tests
#[test]
fn test_fab_direction_default() {
    let val = FabDirection::default();
    assert_eq!(val.as_str(), "fab-flower-bottom-right");
}

#[test]
fn test_fab_direction_bottom_right() {
    let val = FabDirection::BottomRight;
    assert_eq!(val.as_str(), "fab-flower-bottom-right");
}

#[test]
fn test_fab_direction_bottom_left() {
    let val = FabDirection::BottomLeft;
    assert_eq!(val.as_str(), "fab-flower-bottom-left");
}

#[test]
fn test_fab_direction_top_right() {
    let val = FabDirection::TopRight;
    assert_eq!(val.as_str(), "fab-flower-top-right");
}

#[test]
fn test_fab_direction_top_left() {
    let val = FabDirection::TopLeft;
    assert_eq!(val.as_str(), "fab-flower-top-left");
}

#[test]
fn test_fab_direction_clone() {
    let v1 = FabDirection::TopLeft;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_fab_direction_debug() {
    let val = FabDirection::TopRight;
    assert!(format!("{:?}", val).contains("TopRight"));
}

// Comprehensive coverage test
#[test]
fn test_all_fab_directions_return_valid_classes() {
    let variants = vec![
        (FabDirection::BottomRight, "fab-flower-bottom-right"),
        (FabDirection::BottomLeft, "fab-flower-bottom-left"),
        (FabDirection::TopRight, "fab-flower-top-right"),
        (FabDirection::TopLeft, "fab-flower-top-left"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
