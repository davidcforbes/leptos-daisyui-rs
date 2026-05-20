use super::*;

// JoinDirection tests
#[test]
fn test_join_direction_default() {
    let val = JoinDirection::default();
    assert_eq!(val.as_str(), "join-horizontal");
}

#[test]
fn test_join_direction_horizontal() {
    let val = JoinDirection::Horizontal;
    assert_eq!(val.as_str(), "join-horizontal");
}

#[test]
fn test_join_direction_vertical() {
    let val = JoinDirection::Vertical;
    assert_eq!(val.as_str(), "join-vertical");
}

#[test]
fn test_join_direction_clone() {
    let v1 = JoinDirection::Vertical;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_join_direction_debug() {
    let val = JoinDirection::Vertical;
    assert!(format!("{:?}", val).contains("Vertical"));
}

// Comprehensive coverage test
#[test]
fn test_all_join_directions_return_valid_classes() {
    let variants = vec![
        (JoinDirection::Horizontal, "join-horizontal"),
        (JoinDirection::Vertical, "join-vertical"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
