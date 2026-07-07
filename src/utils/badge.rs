//! Shared nav-item badge-count helpers, used by [`AppShellIconNavItem`](crate::components::AppShellIconNavItem)
//! and [`NavRailItem`](crate::components::NavRailItem) -- both render a small
//! count badge pinned to the item's top-right corner and share the same
//! clamping/visibility rules, mirroring d2d-ui's `NavItem::with_badge`.

/// Formats a nav-item badge count for display, clamping large counts to
/// `"99+"` -- mirrors d2d-ui's `NavItem` badge rendering
/// (`if count > 99 { "99+" } else { count.to_string() }`).
pub fn badge_text(count: u32) -> String {
    if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    }
}

/// Whether a badge should render at all. `None` (no badge) and `Some(0)`
/// (an explicit zero count) are both hidden, matching d2d-ui's
/// `NavItem::with_badge`, which only sets the badge when `count > 0`.
pub fn badge_visible(count: Option<u32>) -> bool {
    count.is_some_and(|n| n > 0)
}

#[cfg(test)]
mod tests {
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
}
