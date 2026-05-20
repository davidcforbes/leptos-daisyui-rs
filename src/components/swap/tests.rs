use super::*;

// SwapRotate tests
#[test]
fn test_swap_rotate_default() {
    let rotate = SwapRotate::default();
    assert_eq!(rotate.as_str(), "");
}

#[test]
fn test_swap_rotate_none() {
    assert_eq!(SwapRotate::None.as_str(), "");
}

#[test]
fn test_swap_rotate_rotate() {
    assert_eq!(SwapRotate::Rotate.as_str(), "swap-rotate");
}

#[test]
fn test_swap_rotate_flip() {
    assert_eq!(SwapRotate::Flip.as_str(), "swap-flip");
}

#[test]
fn test_swap_rotate_clone() {
    let r1 = SwapRotate::Rotate;
    let r2 = r1.clone();
    assert_eq!(r1.as_str(), r2.as_str());
}

#[test]
fn test_swap_rotate_debug() {
    let rotate = SwapRotate::Flip;
    assert!(format!("{:?}", rotate).contains("Flip"));
}

#[test]
fn test_all_swap_rotates_return_valid_classes() {
    let variants = vec![
        (SwapRotate::None, ""),
        (SwapRotate::Rotate, "swap-rotate"),
        (SwapRotate::Flip, "swap-flip"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
