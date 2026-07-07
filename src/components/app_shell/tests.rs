use super::*;

// badge_text tests

#[test]
fn test_badge_text_small_counts_pass_through() {
    assert_eq!(badge_text(0), "0");
    assert_eq!(badge_text(1), "1");
    assert_eq!(badge_text(42), "42");
}

#[test]
fn test_badge_text_clamps_at_99() {
    assert_eq!(badge_text(99), "99");
    assert_eq!(badge_text(100), "99+");
    assert_eq!(badge_text(1000), "99+");
}

// badge_visible tests

#[test]
fn test_badge_visible_none_is_hidden() {
    assert!(!badge_visible(None));
}

#[test]
fn test_badge_visible_zero_is_hidden() {
    assert!(!badge_visible(Some(0)));
}

#[test]
fn test_badge_visible_positive_counts_show() {
    assert!(badge_visible(Some(1)));
    assert!(badge_visible(Some(500)));
}

// nav_group_class tests

#[test]
fn test_nav_group_class_unpinned() {
    assert_eq!(nav_group_class(false), "flex flex-col items-center gap-1");
}

#[test]
fn test_nav_group_class_pinned_appends_mt_auto() {
    let class = nav_group_class(true);
    assert!(class.contains("mt-auto"));
    assert_ne!(class, nav_group_class(false));
}
