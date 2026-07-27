//! Unit tests for Icon component

use super::style::IconSize;

#[test]
fn test_icon_size_count() {
    let sizes = [
        IconSize::XSmall,
        IconSize::Small,
        IconSize::Medium,
        IconSize::Large,
        IconSize::XLarge,
    ];

    assert_eq!(sizes.len(), 5, "Should have 5 icon sizes");
}

#[test]
fn test_icon_size_px_values() {
    assert_eq!(IconSize::XSmall.as_px(), 16, "XSmall should be 16px");
    assert_eq!(IconSize::Small.as_px(), 20, "Small should be 20px");
    assert_eq!(IconSize::Medium.as_px(), 24, "Medium should be 24px");
    assert_eq!(IconSize::Large.as_px(), 32, "Large should be 32px");
    assert_eq!(IconSize::XLarge.as_px(), 48, "XLarge should be 48px");
}

#[test]
fn test_icon_size_css_classes() {
    assert_eq!(
        IconSize::XSmall.as_str(),
        "w-4 h-4",
        "XSmall should use w-4 h-4"
    );
    assert_eq!(
        IconSize::Small.as_str(),
        "w-5 h-5",
        "Small should use w-5 h-5"
    );
    assert_eq!(
        IconSize::Medium.as_str(),
        "w-6 h-6",
        "Medium should use w-6 h-6"
    );
    assert_eq!(
        IconSize::Large.as_str(),
        "w-8 h-8",
        "Large should use w-8 h-8"
    );
    assert_eq!(
        IconSize::XLarge.as_str(),
        "w-12 h-12",
        "XLarge should use w-12 h-12"
    );
}

#[test]
fn test_icon_size_default() {
    assert_eq!(
        IconSize::default(),
        IconSize::Medium,
        "Default size should be Medium"
    );
}

#[test]
fn test_icon_size_clone() {
    let size = IconSize::Large;
    let cloned = size.clone();
    assert_eq!(size, cloned, "Cloned size should equal original");
}

#[test]
fn test_icon_size_debug() {
    let size = IconSize::Small;
    let debug_str = format!("{:?}", size);
    assert!(
        debug_str.contains("Small"),
        "Debug output should contain size name"
    );
}

#[test]
fn test_icon_sizes_are_ascending() {
    assert!(IconSize::XSmall.as_px() < IconSize::Small.as_px());
    assert!(IconSize::Small.as_px() < IconSize::Medium.as_px());
    assert!(IconSize::Medium.as_px() < IconSize::Large.as_px());
    assert!(IconSize::Large.as_px() < IconSize::XLarge.as_px());
}

#[test]
fn test_icon_size_css_classes_match_px() {
    // Tailwind's w-4 = 1rem = 16px (at default 16px base)
    assert_eq!(IconSize::XSmall.as_px(), 16);
    assert!(IconSize::XSmall.as_str().contains("w-4"));

    // w-12 = 3rem = 48px
    assert_eq!(IconSize::XLarge.as_px(), 48);
    assert!(IconSize::XLarge.as_str().contains("w-12"));
}

// ---------------------------------------------------------------------------
// The size ramp is a family of its own (ldui-mai.4)
// ---------------------------------------------------------------------------

/// Icon sizes are **sizes**, not spacing, and are deliberately NOT required to
/// sit on `ui_tokens::spacing::SCALE`.
///
/// Spacing answers "how far apart?" and must land on the canonical scale.
/// A size ramp answers "how big?" and follows its own roughly-geometric
/// progression — 16, 20, 24, 32, 48. Two of those (20 and 40, the latter in
/// [`crate::components::icon_tile::IconTileSize`]) are on the 4px grid but not
/// on the 9-step scale, and snapping them would collide with their
/// neighbours and collapse a 5-step ramp to 4. The shared token crate takes
/// the same position: `TABLE_ROW_HEIGHT` is 40.
#[test]
fn size_ramp_is_on_the_4px_grid_but_not_bound_to_the_spacing_scale() {
    for size in [
        IconSize::XSmall,
        IconSize::Small,
        IconSize::Medium,
        IconSize::Large,
        IconSize::XLarge,
    ] {
        assert_eq!(size.as_px() % 4, 0, "{size:?} is off the 4px grid");
    }
    // Explicitly: the ramp is allowed off the spacing scale, and does use it.
    assert!(!ui_tokens::spacing::is_on_scale(
        IconSize::Small.as_px() as f32
    ));
}

#[test]
fn size_ramp_is_strictly_ascending() {
    let px: Vec<u32> = [
        IconSize::XSmall,
        IconSize::Small,
        IconSize::Medium,
        IconSize::Large,
        IconSize::XLarge,
    ]
    .iter()
    .map(|s| s.as_px())
    .collect();
    for w in px.windows(2) {
        assert!(w[0] < w[1], "ramp not ascending: {px:?}");
    }
}

#[test]
fn medium_matches_the_shared_nav_icon_token() {
    // Where the desktop face names an icon dimension, the web ramp must
    // agree — this is the step a nav rail draws.
    assert_eq!(
        IconSize::Medium.as_px() as f32,
        ui_tokens::spacing::NAV_ICON_SIZE
    );
}

#[test]
fn as_str_and_as_px_never_disagree() {
    // The class string and the pixel value are two encodings of one decision;
    // a Tailwind step is 4px, so `w-N` must be `as_px() / 4`.
    for size in [
        IconSize::XSmall,
        IconSize::Small,
        IconSize::Medium,
        IconSize::Large,
        IconSize::XLarge,
    ] {
        let step = size.as_px() / 4;
        let expected = format!("w-{step} h-{step}");
        assert_eq!(size.as_str(), expected, "{size:?} class/px mismatch");
    }
}
