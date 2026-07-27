use super::*;

// rail_class tests

#[test]
fn test_rail_class_is_stable() {
    assert_eq!(
        rail_class(),
        "flex h-full w-16 flex-col items-center gap-1 bg-base-300 py-2"
    );
}

// group_class tests

#[test]
fn test_group_class_unpinned() {
    assert_eq!(group_class(false), "flex flex-col items-center gap-1");
}

#[test]
fn test_group_class_pinned_appends_mt_auto() {
    let class = group_class(true);
    assert_eq!(class, "flex flex-col items-center gap-1 mt-auto");
    assert!(class.contains("mt-auto"));
}

// item_class tests

#[test]
fn test_item_class_resting() {
    let class = item_class(false);
    assert!(class.contains("text-base-content/60"));
    assert!(class.contains("hover:bg-base-200"));
    assert!(!class.contains("bg-base-200 text-primary"));
}

#[test]
fn test_item_class_active() {
    let class = item_class(true);
    assert!(class.contains("bg-base-200"));
    assert!(class.contains("text-primary"));
}

#[test]
fn test_item_class_active_and_resting_differ() {
    assert_ne!(item_class(true), item_class(false));
}

// indicator_class tests

#[test]
fn test_indicator_class_resting_is_transparent() {
    assert!(indicator_class(false).contains("bg-transparent"));
}

#[test]
fn test_indicator_class_active_is_accented() {
    let class = indicator_class(true);
    assert!(class.contains("bg-primary"));
    assert!(!class.contains("bg-transparent"));
}

#[test]
fn test_indicator_class_shares_positioning_shell() {
    // Both states keep the same absolute-positioning shell so only the
    // fill color changes -- the DOM shape stays stable across toggles.
    let resting = indicator_class(false);
    let active = indicator_class(true);
    for shared in [
        "absolute",
        "left-0",
        "top-1/2",
        "h-6",
        // The bar's width comes from the shared stroke family, not a literal:
        // `--border-width-accent` is generated from `ui_tokens::stroke::ACCENT`
        // (3px), which is what the Direct2D face draws for the same bar. It was
        // `w-1` (4px) and silently disagreed with the desktop -- see ldui-mai.1.
        "w-(--border-width-accent)",
    ] {
        assert!(resting.contains(shared), "resting missing {shared}");
        assert!(active.contains(shared), "active missing {shared}");
    }
}

#[test]
fn test_indicator_width_is_the_shared_accent_stroke() {
    // Guards the convergence: if anyone reverts to a hardcoded width, the two
    // faces drift apart again by a pixel and nothing else would catch it.
    assert_eq!(ui_tokens::stroke::ACCENT, 3.0);
    assert_eq!(
        ui_tokens::spacing::NAV_ACCENT_WIDTH,
        ui_tokens::stroke::ACCENT
    );
    for state in [true, false] {
        let class = indicator_class(state);
        assert!(
            !class.contains("w-1 ") && !class.ends_with("w-1"),
            "indicator width regressed to a hardcoded value: {class}"
        );
    }
}

// Comprehensive coverage table, mirroring the pattern used by other
// component style modules (see metric_row::tests).
#[test]
fn test_all_item_states_return_valid_classes() {
    let variants = vec![(false, "text-base-content/60"), (true, "text-primary")];
    for (active, expected_fragment) in variants {
        assert!(item_class(active).contains(expected_fragment));
    }
}
