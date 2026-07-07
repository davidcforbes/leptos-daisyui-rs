//! Pure keyboard-navigation logic for [`Menu`](super::Menu)'s highlight
//! state.
//!
//! Ported from d2d-ui's `controls::menu::Menu::step_highlight` /
//! `highlight_next` / `highlight_prev`: arrow-key highlight movement that
//! wraps at the ends and skips non-selectable items. d2d skips items that
//! fail `MenuItem::is_selectable()` (dividers, mainly); the Leptos port only
//! ever registers *interactive* items with the highlight tracker (dividers
//! and titles never register — see `MenuNav::register` in `component.rs`),
//! so the equivalent "skip" here is skipping *disabled* registered items.
//!
//! These functions are pure index math over a `disabled: &[bool]` slice (one
//! entry per registered item, in document order) so they're unit-testable
//! without a DOM.

/// Move the highlight to the next (`forward = true`) or previous
/// (`forward = false`) enabled item, wrapping around the ends. Direct port of
/// d2d-ui's `Menu::step_highlight`.
///
/// A `None` current highlight starts the scan just before the first item
/// (`forward`) or just after the last item (`!forward`), so the first
/// `ArrowDown` from no highlight lands on item `0` (if enabled) and the first
/// `ArrowUp` lands on the last item (if enabled) — matching d2d.
///
/// Returns `None` if `disabled` is empty, or every item is disabled.
pub fn next_enabled_index(
    current: Option<usize>,
    disabled: &[bool],
    forward: bool,
) -> Option<usize> {
    let n = disabled.len();
    if n == 0 {
        return None;
    }

    let mut idx: isize = match current {
        Some(i) => i as isize,
        None => {
            if forward {
                -1
            } else {
                n as isize
            }
        }
    };

    for _ in 0..n {
        idx += if forward { 1 } else { -1 };
        if idx < 0 {
            idx = n as isize - 1;
        } else if idx >= n as isize {
            idx = 0;
        }
        if !disabled[idx as usize] {
            return Some(idx as usize);
        }
    }

    // Every item is disabled.
    None
}

/// The first enabled item (`Home` key), or `None` if there are no items, or
/// every item is disabled.
pub fn first_enabled_index(disabled: &[bool]) -> Option<usize> {
    disabled.iter().position(|&d| !d)
}

/// The last enabled item (`End` key), or `None` if there are no items, or
/// every item is disabled.
pub fn last_enabled_index(disabled: &[bool]) -> Option<usize> {
    disabled.iter().rposition(|&d| !d)
}

/// Whether an `is_submenu` container [`MenuItem`](super::MenuItem) has its
/// own click semantics — a non-empty `value` (participates in selection
/// tracking) or an `on_click` callback — as opposed to being a purely
/// structural wrapper (e.g. a `MenuTitle` + nested `SubMenu`, the demo
/// sidebar's category-item pattern in `demo/src/core/layout.rs`). Pure so
/// it's unit-testable without a DOM; see `MenuItem`'s `is_submenu` branch in
/// `component.rs` for how it gates the click handler and `cursor-pointer`
/// styling (ldui-jcs.21 review: unconditionally attaching those to every
/// `is_submenu` container let clicks on nested, individually-clickable
/// `SubMenu` items bubble up and spuriously re-activate the container).
pub fn submenu_container_is_interactive(has_value: bool, has_on_click: bool) -> bool {
    has_value || has_on_click
}
