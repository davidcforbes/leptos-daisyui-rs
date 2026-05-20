use super::*;

// PaginationSize tests
#[test]
fn test_pagination_size_default() {
    let val = PaginationSize::default();
    assert_eq!(val.as_str(), "join-md");
}

#[test]
fn test_pagination_size_xs() {
    let val = PaginationSize::Xs;
    assert_eq!(val.as_str(), "join-xs");
}

#[test]
fn test_pagination_size_sm() {
    let val = PaginationSize::Sm;
    assert_eq!(val.as_str(), "join-sm");
}

#[test]
fn test_pagination_size_md() {
    let val = PaginationSize::Md;
    assert_eq!(val.as_str(), "join-md");
}

#[test]
fn test_pagination_size_lg() {
    let val = PaginationSize::Lg;
    assert_eq!(val.as_str(), "join-lg");
}

#[test]
fn test_pagination_size_xl() {
    let val = PaginationSize::Xl;
    assert_eq!(val.as_str(), "join-xl");
}

#[test]
fn test_pagination_size_clone() {
    let v1 = PaginationSize::Lg;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_pagination_size_debug() {
    let val = PaginationSize::Xl;
    assert!(format!("{:?}", val).contains("Xl"));
}

// Comprehensive coverage test
#[test]
fn test_all_pagination_sizes_return_valid_classes() {
    let variants = vec![
        (PaginationSize::Xs, "join-xs"),
        (PaginationSize::Sm, "join-sm"),
        (PaginationSize::Md, "join-md"),
        (PaginationSize::Lg, "join-lg"),
        (PaginationSize::Xl, "join-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
