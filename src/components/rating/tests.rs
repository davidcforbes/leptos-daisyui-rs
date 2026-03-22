use super::*;

// RatingSize tests
#[test]
fn test_rating_size_default() {
    let size = RatingSize::default();
    assert_eq!(size.as_str(), "");
}

#[test]
fn test_rating_size_xs() {
    assert_eq!(RatingSize::Xs.as_str(), "rating-xs");
}

#[test]
fn test_rating_size_sm() {
    assert_eq!(RatingSize::Sm.as_str(), "rating-sm");
}

#[test]
fn test_rating_size_md() {
    assert_eq!(RatingSize::Md.as_str(), "rating-md");
}

#[test]
fn test_rating_size_lg() {
    assert_eq!(RatingSize::Lg.as_str(), "rating-lg");
}

#[test]
fn test_rating_size_clone() {
    let s1 = RatingSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_rating_size_debug() {
    let size = RatingSize::Sm;
    assert!(format!("{:?}", size).contains("Sm"));
}

#[test]
fn test_all_rating_sizes_return_valid_classes() {
    let variants = vec![
        (RatingSize::Default, ""),
        (RatingSize::Xs, "rating-xs"),
        (RatingSize::Sm, "rating-sm"),
        (RatingSize::Md, "rating-md"),
        (RatingSize::Lg, "rating-lg"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
