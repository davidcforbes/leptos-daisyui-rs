use super::*;

// KbdSize tests
#[test]
fn test_kbd_size_default() {
    let val = KbdSize::default();
    assert_eq!(val.as_str(), "kbd-md");
}

#[test]
fn test_kbd_size_xs() {
    let val = KbdSize::Xs;
    assert_eq!(val.as_str(), "kbd-xs");
}

#[test]
fn test_kbd_size_sm() {
    let val = KbdSize::Sm;
    assert_eq!(val.as_str(), "kbd-sm");
}

#[test]
fn test_kbd_size_md() {
    let val = KbdSize::Md;
    assert_eq!(val.as_str(), "kbd-md");
}

#[test]
fn test_kbd_size_lg() {
    let val = KbdSize::Lg;
    assert_eq!(val.as_str(), "kbd-lg");
}

#[test]
fn test_kbd_size_xl() {
    let val = KbdSize::Xl;
    assert_eq!(val.as_str(), "kbd-xl");
}

#[test]
fn test_kbd_size_clone() {
    let v1 = KbdSize::Lg;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_kbd_size_debug() {
    let val = KbdSize::Xl;
    assert!(format!("{:?}", val).contains("Xl"));
}

// Comprehensive coverage test
#[test]
fn test_all_kbd_sizes_return_valid_classes() {
    let variants = vec![
        (KbdSize::Xs, "kbd-xs"),
        (KbdSize::Sm, "kbd-sm"),
        (KbdSize::Md, "kbd-md"),
        (KbdSize::Lg, "kbd-lg"),
        (KbdSize::Xl, "kbd-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
