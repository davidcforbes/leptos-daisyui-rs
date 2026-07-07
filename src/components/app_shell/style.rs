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

/// Classes for a group of `AppShellIconNavItem`s within `AppShellIconNav`.
/// `pinned` appends `mt-auto`, pushing the group (and anything after it) to
/// the bottom of the icon nav strip -- the CSS equivalent of d2d-ui's
/// `AppShell::add_bottom_nav_item`, which stacked a second cluster upward
/// from the rail's foot.
pub fn nav_group_class(pinned: bool) -> &'static str {
    if pinned {
        "flex flex-col items-center gap-1 mt-auto"
    } else {
        "flex flex-col items-center gap-1"
    }
}
