use super::style::ToolbarSize;
use super::types::{ToolbarItem, visible_count_for_width};
use crate::components::button::ButtonSize;

// ─── ToolbarItem builder ─────────────────────────────────────────────────────

#[test]
fn new_item_is_enabled_unchecked_no_tooltip() {
    let item = ToolbarItem::new("save", "Save");
    assert_eq!(item.id, "save");
    assert_eq!(item.label, "Save");
    assert_eq!(item.tooltip, None);
    assert_eq!(item.checked, None);
    assert!(item.enabled);
}

#[test]
fn toolbar_item_builder_chaining() {
    let item = ToolbarItem::new("bold", "B")
        .tooltip("Bold")
        .toggle(true)
        .disabled();
    assert_eq!(item.id, "bold");
    assert_eq!(item.tooltip.as_deref(), Some("Bold"));
    assert_eq!(item.checked, Some(true));
    assert!(!item.enabled);
}

#[test]
fn toggle_sets_checked_state() {
    let item = ToolbarItem::new("italic", "I").toggle(false);
    assert_eq!(item.checked, Some(false));
}

// ─── ToolbarSize ─────────────────────────────────────────────────────────────

#[test]
fn toolbar_size_defaults_md() {
    assert_eq!(ToolbarSize::default(), ToolbarSize::Md);
}

#[test]
fn toolbar_size_maps_to_button_size() {
    assert_eq!(ToolbarSize::Xs.button_size(), ButtonSize::Xs);
    assert_eq!(ToolbarSize::Sm.button_size(), ButtonSize::Sm);
    assert_eq!(ToolbarSize::Md.button_size(), ButtonSize::Md);
    assert_eq!(ToolbarSize::Lg.button_size(), ButtonSize::Lg);
}

// ─── visible_count_for_width (pure overflow-fit logic) ──────────────────────

#[test]
fn zero_items_returns_zero() {
    assert_eq!(visible_count_for_width(500.0, &[], 4.0, 32.0), 0);
}

#[test]
fn zero_container_width_returns_zero() {
    assert_eq!(visible_count_for_width(0.0, &[20.0, 20.0], 4.0, 32.0), 0);
}

#[test]
fn all_items_fit_when_total_width_within_container() {
    // 3 items x 40 + 2 gaps x 4 = 128, well within 500.
    let widths = [40.0, 40.0, 40.0];
    assert_eq!(visible_count_for_width(500.0, &widths, 4.0, 32.0), 3);
}

#[test]
fn exact_fit_boundary_is_not_overflow() {
    // 2 items x 40 + 1 gap x 4 = 84, container exactly 84 -> both fit, no
    // overflow button needed at all.
    let widths = [40.0, 40.0];
    assert_eq!(visible_count_for_width(84.0, &widths, 4.0, 32.0), 2);
}

#[test]
fn overflow_triggers_when_total_exceeds_container() {
    // 5 items x 40 + 4 gaps x 4 = 216 > 100 available.
    let widths = [40.0, 40.0, 40.0, 40.0, 40.0];
    let count = visible_count_for_width(100.0, &widths, 4.0, 32.0);
    assert!(
        count < widths.len(),
        "expected overflow, got {count} visible"
    );
    assert!(
        count >= 1,
        "expected at least one visible item, got {count}"
    );
}

#[test]
fn overflow_reserves_space_for_overflow_button() {
    // Budget = 100 - 32 (overflow) = 68. Item 0 (40) fits (used=40). Item 1
    // (40) would need 40+4+40=84 > 68, so only 1 item visible.
    let widths = [40.0, 40.0, 40.0];
    assert_eq!(visible_count_for_width(100.0, &widths, 4.0, 32.0), 1);
}

#[test]
fn overflow_never_shows_all_items_even_if_math_would_allow() {
    // Degenerate case: overflow_width is 0, and the budget after reserving 0
    // technically fits every item — but since the *unreserved* total already
    // exceeded the container, we must still hold back at least one item so
    // there is something for the overflow button to represent.
    let widths = [40.0, 40.0];
    // full_width = 40+40+4 = 84 > 80 available -> overflow branch taken.
    assert_eq!(visible_count_for_width(80.0, &widths, 4.0, 0.0), 1);
}

#[test]
fn single_item_wider_than_container_returns_zero_visible() {
    let widths = [200.0];
    assert_eq!(visible_count_for_width(50.0, &widths, 4.0, 32.0), 0);
}

#[test]
fn negative_or_zero_overflow_width_does_not_panic() {
    let widths = [40.0, 40.0, 40.0];
    // overflow_width larger than container: budget clamps to 0.
    let count = visible_count_for_width(50.0, &widths, 4.0, 1000.0);
    assert_eq!(count, 0);
}

#[test]
fn narrowing_container_reduces_visible_count_monotonically() {
    let widths = [30.0, 30.0, 30.0, 30.0];
    let wide = visible_count_for_width(400.0, &widths, 2.0, 32.0);
    let narrow = visible_count_for_width(90.0, &widths, 2.0, 32.0);
    assert!(narrow <= wide);
    assert_eq!(wide, 4);
}
