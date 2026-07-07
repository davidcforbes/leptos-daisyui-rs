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

// app_shell_root_class tests

#[test]
fn test_app_shell_root_class_no_status_bar_is_unchanged() {
    assert_eq!(app_shell_root_class(false), "flex h-full w-full");
}

#[test]
fn test_app_shell_root_class_with_status_bar_switches_to_column() {
    let class = app_shell_root_class(true);
    assert_eq!(class, "flex flex-col h-full w-full");
    assert!(class.contains("flex-col"));
    assert_ne!(class, app_shell_root_class(false));
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
