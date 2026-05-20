use super::*;

// StackPlacement tests
#[test]
fn test_stack_placement_default() {
    let placement = StackPlacement::default();
    assert_eq!(placement.as_str(), "stack-bottom");
}

#[test]
fn test_stack_placement_top() {
    assert_eq!(StackPlacement::Top.as_str(), "stack-top");
}

#[test]
fn test_stack_placement_bottom() {
    assert_eq!(StackPlacement::Bottom.as_str(), "stack-bottom");
}

#[test]
fn test_stack_placement_start() {
    assert_eq!(StackPlacement::Start.as_str(), "stack-start");
}

#[test]
fn test_stack_placement_end() {
    assert_eq!(StackPlacement::End.as_str(), "stack-end");
}

#[test]
fn test_stack_placement_clone() {
    let p1 = StackPlacement::Top;
    let p2 = p1.clone();
    assert_eq!(p1.as_str(), p2.as_str());
}

#[test]
fn test_stack_placement_debug() {
    let placement = StackPlacement::Start;
    assert!(format!("{:?}", placement).contains("Start"));
}

#[test]
fn test_stack_placement_partial_eq() {
    assert_eq!(StackPlacement::Top, StackPlacement::Top);
    assert_ne!(StackPlacement::Top, StackPlacement::Bottom);
}

#[test]
fn test_all_stack_placements_return_valid_classes() {
    let variants = vec![
        (StackPlacement::Top, "stack-top"),
        (StackPlacement::Bottom, "stack-bottom"),
        (StackPlacement::Start, "stack-start"),
        (StackPlacement::End, "stack-end"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
