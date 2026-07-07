/// Root `<nav>` container classes for the rail.
///
/// A fixed-width vertical flex column with a themed background; width can be
/// overridden by the caller's `class` prop (e.g. `class="w-20"`) since
/// `merge_classes!` appends user classes after these.
pub fn rail_class() -> &'static str {
    "flex h-full w-16 flex-col items-center gap-1 bg-base-300 py-2"
}

/// Group wrapper classes. `pinned` appends `mt-auto`, pushing the group (and
/// everything after it) to the bottom of the rail's flex column -- this is
/// how the "Settings pinned to the bottom" layout from d2d-ui's `NavRail`
/// (`bottom_items` stacking up from the rail's bottom edge) is expressed in
/// CSS instead of manual rect math.
pub fn group_class(pinned: bool) -> &'static str {
    if pinned {
        "flex flex-col items-center gap-1 mt-auto"
    } else {
        "flex flex-col items-center gap-1"
    }
}

/// Item button classes for the resting/hover vs. active (selected) states.
///
/// The active state renders a filled "pill" (`bg-base-200`) behind the icon
/// and tints the icon with the primary color, mirroring d2d-ui's
/// `theme::CONTROL_PRESSED` selection fill; the resting state is a muted
/// icon that gains a lighter hover pill and full-opacity text on `:hover`
/// (replacing the renderer's manual `update_hover` hit-testing with CSS).
pub fn item_class(active: bool) -> &'static str {
    if active {
        "relative flex h-12 w-12 cursor-pointer items-center justify-center rounded-box bg-base-200 text-primary transition-colors hover:bg-base-300"
    } else {
        "relative flex h-12 w-12 cursor-pointer items-center justify-center rounded-box text-base-content/60 transition-colors hover:bg-base-200 hover:text-base-content"
    }
}

/// Left-edge accent indicator bar classes. Always rendered (so the DOM shape
/// is stable across active/inactive toggles); `bg-transparent` hides it when
/// the item isn't active. Mirrors d2d-ui's `theme::ACCENT` bar drawn at the
/// rail's left edge, inset from the item's top/bottom (`INDICATOR_INSET`).
pub fn indicator_class(active: bool) -> &'static str {
    if active {
        "absolute left-0 top-1/2 h-6 w-1 -translate-y-1/2 rounded-r-full bg-primary"
    } else {
        "absolute left-0 top-1/2 h-6 w-1 -translate-y-1/2 rounded-r-full bg-transparent"
    }
}
