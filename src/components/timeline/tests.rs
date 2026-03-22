use super::*;

// TimelineDirection tests
#[test]
fn test_timeline_direction_default() {
    let dir = TimelineDirection::default();
    assert_eq!(dir.as_str(), "timeline-vertical");
}

#[test]
fn test_timeline_direction_vertical() {
    assert_eq!(TimelineDirection::Vertical.as_str(), "timeline-vertical");
}

#[test]
fn test_timeline_direction_horizontal() {
    assert_eq!(TimelineDirection::Horizontal.as_str(), "timeline-horizontal");
}

#[test]
fn test_timeline_direction_clone() {
    let d1 = TimelineDirection::Horizontal;
    let d2 = d1.clone();
    assert_eq!(d1.as_str(), d2.as_str());
}

#[test]
fn test_timeline_direction_debug() {
    let dir = TimelineDirection::Horizontal;
    assert!(format!("{:?}", dir).contains("Horizontal"));
}

// TimelineItemPosition tests
#[test]
fn test_timeline_item_position_default() {
    let pos = TimelineItemPosition::default();
    assert!(pos.is_start());
}

#[test]
fn test_timeline_item_position_start() {
    let pos = TimelineItemPosition::Start;
    assert!(pos.is_start());
    assert!(!pos.is_end());
    assert!(!pos.is_between());
}

#[test]
fn test_timeline_item_position_end() {
    let pos = TimelineItemPosition::End;
    assert!(!pos.is_start());
    assert!(pos.is_end());
    assert!(!pos.is_between());
}

#[test]
fn test_timeline_item_position_between() {
    let pos = TimelineItemPosition::Between;
    assert!(!pos.is_start());
    assert!(!pos.is_end());
    assert!(pos.is_between());
}

#[test]
fn test_timeline_item_position_clone() {
    let p1 = TimelineItemPosition::Between;
    let p2 = p1.clone();
    assert!(p2.is_between());
}

#[test]
fn test_timeline_item_position_debug() {
    let pos = TimelineItemPosition::End;
    assert!(format!("{:?}", pos).contains("End"));
}

#[test]
fn test_all_timeline_directions_return_valid_classes() {
    let variants = vec![
        (TimelineDirection::Vertical, "timeline-vertical"),
        (TimelineDirection::Horizontal, "timeline-horizontal"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
