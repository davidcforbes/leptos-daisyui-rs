/// Root classes for `AppShell`. When a status bar slot is present, the root
/// switches from a single-row layout to a column layout so the main
/// 3-panel row and the status bar can stack vertically; the row itself then
/// gets `flex-1 min-h-0` (applied separately in the component) so it fills
/// the remaining height above the status bar. When no status bar is given,
/// this is unchanged from the shell's original single-row root class.
pub fn app_shell_root_class(has_status_bar: bool) -> &'static str {
    if has_status_bar {
        "flex flex-col h-full w-full"
    } else {
        "flex h-full w-full"
    }
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
