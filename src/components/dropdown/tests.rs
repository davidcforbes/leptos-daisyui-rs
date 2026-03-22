use super::*;

// DropdownAlignment tests
#[test]
fn test_dropdown_alignment_default() {
    let val = DropdownAlignment::default();
    assert_eq!(val.as_str(), "dropdown-start");
}

#[test]
fn test_dropdown_alignment_start() {
    let val = DropdownAlignment::Start;
    assert_eq!(val.as_str(), "dropdown-start");
}

#[test]
fn test_dropdown_alignment_center() {
    let val = DropdownAlignment::Center;
    assert_eq!(val.as_str(), "dropdown-center");
}

#[test]
fn test_dropdown_alignment_end() {
    let val = DropdownAlignment::End;
    assert_eq!(val.as_str(), "dropdown-end");
}

#[test]
fn test_dropdown_alignment_clone() {
    let v1 = DropdownAlignment::Center;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_dropdown_alignment_debug() {
    let val = DropdownAlignment::End;
    assert!(format!("{:?}", val).contains("End"));
}

// DropdownPlacement tests
#[test]
fn test_dropdown_placement_default() {
    let val = DropdownPlacement::default();
    assert_eq!(val.as_str(), "dropdown-bottom");
}

#[test]
fn test_dropdown_placement_top() {
    let val = DropdownPlacement::Top;
    assert_eq!(val.as_str(), "dropdown-top");
}

#[test]
fn test_dropdown_placement_bottom() {
    let val = DropdownPlacement::Bottom;
    assert_eq!(val.as_str(), "dropdown-bottom");
}

#[test]
fn test_dropdown_placement_left() {
    let val = DropdownPlacement::Left;
    assert_eq!(val.as_str(), "dropdown-left");
}

#[test]
fn test_dropdown_placement_right() {
    let val = DropdownPlacement::Right;
    assert_eq!(val.as_str(), "dropdown-right");
}

#[test]
fn test_dropdown_placement_clone() {
    let v1 = DropdownPlacement::Top;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_dropdown_placement_debug() {
    let val = DropdownPlacement::Right;
    assert!(format!("{:?}", val).contains("Right"));
}

// Comprehensive coverage tests
#[test]
fn test_all_dropdown_alignments_return_valid_classes() {
    let variants = vec![
        (DropdownAlignment::Start, "dropdown-start"),
        (DropdownAlignment::Center, "dropdown-center"),
        (DropdownAlignment::End, "dropdown-end"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_dropdown_placements_return_valid_classes() {
    let variants = vec![
        (DropdownPlacement::Top, "dropdown-top"),
        (DropdownPlacement::Bottom, "dropdown-bottom"),
        (DropdownPlacement::Left, "dropdown-left"),
        (DropdownPlacement::Right, "dropdown-right"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
