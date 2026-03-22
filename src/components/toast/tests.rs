use super::*;

// ToastPosition tests
#[test]
fn test_toast_position_default() {
    let pos = ToastPosition::default();
    assert_eq!(pos.as_str(), "toast-bottom toast-end");
}

#[test]
fn test_toast_position_top_end() {
    assert_eq!(ToastPosition::TopEnd.as_str(), "toast-top toast-end");
}

#[test]
fn test_toast_position_top_start() {
    assert_eq!(ToastPosition::TopStart.as_str(), "toast-top toast-start");
}

#[test]
fn test_toast_position_top_center() {
    assert_eq!(ToastPosition::TopCenter.as_str(), "toast-top toast-center");
}

#[test]
fn test_toast_position_middle_start() {
    assert_eq!(ToastPosition::MiddleStart.as_str(), "toast-middle toast-start");
}

#[test]
fn test_toast_position_middle_center() {
    assert_eq!(ToastPosition::MiddleCenter.as_str(), "toast-middle toast-center");
}

#[test]
fn test_toast_position_middle_end() {
    assert_eq!(ToastPosition::MiddleEnd.as_str(), "toast-middle toast-end");
}

#[test]
fn test_toast_position_bottom_start() {
    assert_eq!(ToastPosition::BottomStart.as_str(), "toast-bottom toast-start");
}

#[test]
fn test_toast_position_bottom_center() {
    assert_eq!(ToastPosition::BottomCenter.as_str(), "toast-bottom toast-center");
}

#[test]
fn test_toast_position_bottom_end() {
    assert_eq!(ToastPosition::BottomEnd.as_str(), "toast-bottom toast-end");
}

#[test]
fn test_toast_position_clone() {
    let p1 = ToastPosition::TopCenter;
    let p2 = p1.clone();
    assert_eq!(p1.as_str(), p2.as_str());
}

#[test]
fn test_toast_position_debug() {
    let pos = ToastPosition::MiddleCenter;
    assert!(format!("{:?}", pos).contains("MiddleCenter"));
}

#[test]
fn test_all_toast_positions_return_valid_classes() {
    let variants = vec![
        (ToastPosition::TopEnd, "toast-top toast-end"),
        (ToastPosition::TopStart, "toast-top toast-start"),
        (ToastPosition::TopCenter, "toast-top toast-center"),
        (ToastPosition::MiddleStart, "toast-middle toast-start"),
        (ToastPosition::MiddleCenter, "toast-middle toast-center"),
        (ToastPosition::MiddleEnd, "toast-middle toast-end"),
        (ToastPosition::BottomStart, "toast-bottom toast-start"),
        (ToastPosition::BottomCenter, "toast-bottom toast-center"),
        (ToastPosition::BottomEnd, "toast-bottom toast-end"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
