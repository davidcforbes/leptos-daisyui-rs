use super::*;

// MaskType tests
#[test]
fn test_mask_type_default() {
    let val = MaskType::default();
    assert_eq!(val.as_str(), "mask-squircle");
}

#[test]
fn test_mask_type_squircle() {
    let val = MaskType::Squircle;
    assert_eq!(val.as_str(), "mask-squircle");
}

#[test]
fn test_mask_type_heart() {
    let val = MaskType::Heart;
    assert_eq!(val.as_str(), "mask-heart");
}

#[test]
fn test_mask_type_hexagon() {
    let val = MaskType::Hexagon;
    assert_eq!(val.as_str(), "mask-hexagon");
}

#[test]
fn test_mask_type_hexagon_two() {
    let val = MaskType::HexagonTwo;
    assert_eq!(val.as_str(), "mask-hexagon-2");
}

#[test]
fn test_mask_type_decagon() {
    let val = MaskType::Decagon;
    assert_eq!(val.as_str(), "mask-decagon");
}

#[test]
fn test_mask_type_pentagon() {
    let val = MaskType::Pentagon;
    assert_eq!(val.as_str(), "mask-pentagon");
}

#[test]
fn test_mask_type_diamond() {
    let val = MaskType::Diamond;
    assert_eq!(val.as_str(), "mask-diamond");
}

#[test]
fn test_mask_type_square() {
    let val = MaskType::Square;
    assert_eq!(val.as_str(), "mask-square");
}

#[test]
fn test_mask_type_circle() {
    let val = MaskType::Circle;
    assert_eq!(val.as_str(), "mask-circle");
}

#[test]
fn test_mask_type_parallelogram() {
    let val = MaskType::Parallelogram;
    assert_eq!(val.as_str(), "mask-parallelogram");
}

#[test]
fn test_mask_type_parallelogram_two() {
    let val = MaskType::ParallelogramTwo;
    assert_eq!(val.as_str(), "mask-parallelogram-2");
}

#[test]
fn test_mask_type_parallelogram_three() {
    let val = MaskType::ParallelogramThree;
    assert_eq!(val.as_str(), "mask-parallelogram-3");
}

#[test]
fn test_mask_type_parallelogram_four() {
    let val = MaskType::ParallelogramFour;
    assert_eq!(val.as_str(), "mask-parallelogram-4");
}

#[test]
fn test_mask_type_star() {
    let val = MaskType::Star;
    assert_eq!(val.as_str(), "mask-star");
}

#[test]
fn test_mask_type_star_two() {
    let val = MaskType::StarTwo;
    assert_eq!(val.as_str(), "mask-star-2");
}

#[test]
fn test_mask_type_triangle() {
    let val = MaskType::Triangle;
    assert_eq!(val.as_str(), "mask-triangle");
}

#[test]
fn test_mask_type_triangle_two() {
    let val = MaskType::TriangleTwo;
    assert_eq!(val.as_str(), "mask-triangle-2");
}

#[test]
fn test_mask_type_triangle_three() {
    let val = MaskType::TriangleThree;
    assert_eq!(val.as_str(), "mask-triangle-3");
}

#[test]
fn test_mask_type_triangle_four() {
    let val = MaskType::TriangleFour;
    assert_eq!(val.as_str(), "mask-triangle-4");
}

#[test]
fn test_mask_type_clone() {
    let v1 = MaskType::Heart;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_mask_type_debug() {
    let val = MaskType::Diamond;
    assert!(format!("{:?}", val).contains("Diamond"));
}

// Comprehensive coverage test
#[test]
fn test_all_mask_types_return_valid_classes() {
    let variants = vec![
        (MaskType::Squircle, "mask-squircle"),
        (MaskType::Heart, "mask-heart"),
        (MaskType::Hexagon, "mask-hexagon"),
        (MaskType::HexagonTwo, "mask-hexagon-2"),
        (MaskType::Decagon, "mask-decagon"),
        (MaskType::Pentagon, "mask-pentagon"),
        (MaskType::Diamond, "mask-diamond"),
        (MaskType::Square, "mask-square"),
        (MaskType::Circle, "mask-circle"),
        (MaskType::Parallelogram, "mask-parallelogram"),
        (MaskType::ParallelogramTwo, "mask-parallelogram-2"),
        (MaskType::ParallelogramThree, "mask-parallelogram-3"),
        (MaskType::ParallelogramFour, "mask-parallelogram-4"),
        (MaskType::Star, "mask-star"),
        (MaskType::StarTwo, "mask-star-2"),
        (MaskType::Triangle, "mask-triangle"),
        (MaskType::TriangleTwo, "mask-triangle-2"),
        (MaskType::TriangleThree, "mask-triangle-3"),
        (MaskType::TriangleFour, "mask-triangle-4"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
