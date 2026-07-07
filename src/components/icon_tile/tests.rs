use super::*;

// IconTileColor tests

#[test]
fn test_icon_tile_color_default() {
    let color = IconTileColor::default();
    assert_eq!(color, IconTileColor::Primary);
    assert_eq!(color.as_bg_class(), "bg-primary/10");
    assert_eq!(color.as_fg_class(), "text-primary");
}

#[test]
fn test_icon_tile_color_bg_classes() {
    assert_eq!(IconTileColor::Neutral.as_bg_class(), "bg-neutral/10");
    assert_eq!(IconTileColor::Primary.as_bg_class(), "bg-primary/10");
    assert_eq!(IconTileColor::Secondary.as_bg_class(), "bg-secondary/10");
    assert_eq!(IconTileColor::Accent.as_bg_class(), "bg-accent/10");
    assert_eq!(IconTileColor::Info.as_bg_class(), "bg-info/10");
    assert_eq!(IconTileColor::Success.as_bg_class(), "bg-success/10");
    assert_eq!(IconTileColor::Warning.as_bg_class(), "bg-warning/10");
    assert_eq!(IconTileColor::Error.as_bg_class(), "bg-error/10");
}

#[test]
fn test_icon_tile_color_fg_classes() {
    assert_eq!(IconTileColor::Neutral.as_fg_class(), "text-neutral");
    assert_eq!(IconTileColor::Primary.as_fg_class(), "text-primary");
    assert_eq!(IconTileColor::Secondary.as_fg_class(), "text-secondary");
    assert_eq!(IconTileColor::Accent.as_fg_class(), "text-accent");
    assert_eq!(IconTileColor::Info.as_fg_class(), "text-info");
    assert_eq!(IconTileColor::Success.as_fg_class(), "text-success");
    assert_eq!(IconTileColor::Warning.as_fg_class(), "text-warning");
    assert_eq!(IconTileColor::Error.as_fg_class(), "text-error");
}

#[test]
fn test_icon_tile_color_bg_and_fg_are_independent() {
    // bg and fg can be set to different variants -- e.g. a red-tinted tile
    // with a neutral glyph -- so the two methods must not collapse to the
    // same class for differing variants.
    let bg = IconTileColor::Error;
    let fg = IconTileColor::Neutral;
    assert_eq!(bg.as_bg_class(), "bg-error/10");
    assert_eq!(fg.as_fg_class(), "text-neutral");
    assert_ne!(bg.as_bg_class(), fg.as_bg_class());
}

#[test]
fn test_icon_tile_color_clone_and_debug() {
    let c1 = IconTileColor::Accent;
    let c2 = c1.clone();
    assert_eq!(c1, c2);
    assert!(format!("{:?}", c1).contains("Accent"));
}

#[test]
fn test_all_icon_tile_colors_return_valid_classes() {
    let variants = vec![
        (IconTileColor::Neutral, "bg-neutral/10", "text-neutral"),
        (IconTileColor::Primary, "bg-primary/10", "text-primary"),
        (IconTileColor::Secondary, "bg-secondary/10", "text-secondary"),
        (IconTileColor::Accent, "bg-accent/10", "text-accent"),
        (IconTileColor::Info, "bg-info/10", "text-info"),
        (IconTileColor::Success, "bg-success/10", "text-success"),
        (IconTileColor::Warning, "bg-warning/10", "text-warning"),
        (IconTileColor::Error, "bg-error/10", "text-error"),
    ];
    for (variant, expected_bg, expected_fg) in variants {
        assert_eq!(variant.as_bg_class(), expected_bg);
        assert_eq!(variant.as_fg_class(), expected_fg);
    }
}

// IconTileSize tests

#[test]
fn test_icon_tile_size_default() {
    let size = IconTileSize::default();
    assert_eq!(size, IconTileSize::Md);
    assert_eq!(size.as_str(), "w-10 h-10 text-base");
}

#[test]
fn test_icon_tile_size_xs() {
    assert_eq!(IconTileSize::Xs.as_str(), "w-6 h-6 text-xs");
}

#[test]
fn test_icon_tile_size_sm() {
    assert_eq!(IconTileSize::Sm.as_str(), "w-8 h-8 text-sm");
}

#[test]
fn test_icon_tile_size_md() {
    assert_eq!(IconTileSize::Md.as_str(), "w-10 h-10 text-base");
}

#[test]
fn test_icon_tile_size_lg() {
    assert_eq!(IconTileSize::Lg.as_str(), "w-12 h-12 text-lg");
}

#[test]
fn test_icon_tile_size_xl() {
    assert_eq!(IconTileSize::Xl.as_str(), "w-16 h-16 text-2xl");
}

#[test]
fn test_icon_tile_size_clone_and_debug() {
    let s1 = IconTileSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1, s2);
    assert!(format!("{:?}", s1).contains("Lg"));
}

#[test]
fn test_all_icon_tile_sizes_return_valid_classes() {
    let variants = vec![
        (IconTileSize::Xs, "w-6 h-6 text-xs"),
        (IconTileSize::Sm, "w-8 h-8 text-sm"),
        (IconTileSize::Md, "w-10 h-10 text-base"),
        (IconTileSize::Lg, "w-12 h-12 text-lg"),
        (IconTileSize::Xl, "w-16 h-16 text-2xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

// Circle / corner-radius selection logic (mirrors the component's inline
// `radius_class` closure -- ported from d2d-ui's
// `with_corner_radius(size / 2.0)` circle override).

fn radius_class(circle: bool) -> &'static str {
    if circle { "rounded-full" } else { "rounded-lg" }
}

#[test]
fn test_radius_class_default_is_rounded_square() {
    assert_eq!(radius_class(false), "rounded-lg");
}

#[test]
fn test_radius_class_circle_is_rounded_full() {
    assert_eq!(radius_class(true), "rounded-full");
}
