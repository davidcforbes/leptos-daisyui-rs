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
