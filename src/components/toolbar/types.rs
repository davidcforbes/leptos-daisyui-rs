//! Data model and pure layout logic for the [`Toolbar`](super::component::Toolbar) component.
//!
//! Ported from d2d-ui's `controls::toolbar::ToolbarItem` (beads-hhg8's fixed-width,
//! DIP-based fit math). The Leptos port measures real (variable-width) DOM
//! buttons instead of a fixed button size, so [`visible_count_for_width`]
//! generalizes d2d-ui's `Toolbar::layout` overflow calculation to a slice of
//! per-item widths rather than `n * fixed_width`.

/// A single command or toggle item in a [`Toolbar`](super::component::Toolbar).
///
/// Mirrors d2d-ui's `ToolbarItem`: an identifier dispatched on click, a label
/// rendered inside the button, an optional tooltip, an optional checked/toggle
/// state (rendered as an accent underline), and an enabled flag.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarItem {
    /// Command identifier passed to `Toolbar`'s `on_item_click` callback.
    pub id: String,
    /// Label rendered inside the button (text or icon glyph/emoji).
    pub label: String,
    /// Hover tooltip text, rendered via daisyUI's `tooltip` component.
    pub tooltip: Option<String>,
    /// `Some(true)` renders the button as an active/checked toggle with an
    /// accent underline bar; `Some(false)` or `None` renders a plain command
    /// button.
    pub checked: Option<bool>,
    /// Disabled items are dimmed, excluded from click dispatch, and marked
    /// `disabled` on the underlying `<button>`.
    pub enabled: bool,
}

impl ToolbarItem {
    /// Create an enabled, unchecked, tooltip-less item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tooltip: None,
            checked: None,
            enabled: true,
        }
    }

    /// Attach a hover tooltip string.
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Make this a toggle button with the given initial checked state.
    pub fn toggle(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Disable the item (dimmed, excluded from click dispatch).
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Given the toolbar's available `container_width` and the natural (measured)
/// width of each item in `item_widths`, return how many *leading* items fit
/// left-to-right before an overflow ("...") button is needed.
///
/// - If every item plus `gap`-separated spacing fits within `container_width`,
///   all items are visible (no overflow button reserved).
/// - Otherwise, `overflow_width` (plus one `gap`) is reserved at the right
///   end, and items are greedily accepted left-to-right while they still fit
///   the remaining budget. The result never exceeds `item_widths.len() - 1`
///   when it's non-empty and overflow is unavoidable — mirrors d2d-ui's
///   `Toolbar::layout`, which always leaves at least one item in overflow
///   once the strip doesn't fully fit.
///
/// Pure function — no DOM access. `container_width`, `item_widths`, `gap`, and
/// `overflow_width` are supplied by the component after a `ResizeObserver` /
/// hidden-measurement-row pass.
pub fn visible_count_for_width(
    container_width: f64,
    item_widths: &[f64],
    gap: f64,
    overflow_width: f64,
) -> usize {
    let n = item_widths.len();
    if n == 0 || container_width <= 0.0 {
        return 0;
    }

    let full_width: f64 = item_widths.iter().sum::<f64>() + gap * (n as f64 - 1.0).max(0.0);
    if full_width <= container_width {
        return n;
    }

    // Reserve space for the overflow button plus the gap before it.
    let budget = (container_width - overflow_width).max(0.0);
    let mut used = 0.0;
    let mut count = 0;
    for (i, width) in item_widths.iter().enumerate() {
        let next = used + width + if i > 0 { gap } else { 0.0 };
        if next <= budget {
            used = next;
            count += 1;
        } else {
            break;
        }
    }

    // Always leave at least one item in overflow once we know not everything
    // fits (matches d2d-ui's `.min(n.saturating_sub(1))` guard).
    count.min(n.saturating_sub(1))
}
