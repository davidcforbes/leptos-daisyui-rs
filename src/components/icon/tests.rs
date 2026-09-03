//! Unit tests for Icon component

use super::component::lucide_to_sprite;
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

// ---------------------------------------------------------------------------
// Coordinator semantic icon mappings (ldui-af4b)
// ---------------------------------------------------------------------------
//
// Each of these names resolves to a real symbol in the shared Office sprite
// (verified against `crates/office-perf-web/assets/brand/icons.svg`) rather
// than the blank fallback, so consumers can request them without shipping
// their own SVG markup.

#[test]
fn dollar_sign_maps_to_us_dollar() {
    assert_eq!(lucide_to_sprite("dollar-sign"), "us-dollar");
}

#[test]
fn thumbs_up_maps_to_thumbs_up() {
    assert_eq!(lucide_to_sprite("thumbs-up"), "thumbs-up");
}

#[test]
fn bar_chart_3_maps_to_performance_stats() {
    assert_eq!(lucide_to_sprite("bar-chart-3"), "performance-stats");
}

#[test]
fn phone_call_maps_to_phone_ring() {
    assert_eq!(lucide_to_sprite("phone-call"), "phone-ring");
}

#[test]
fn whatsapp_maps_to_whatsapp() {
    assert_eq!(lucide_to_sprite("whatsapp"), "whatsapp");
}

#[test]
fn unknown_name_still_falls_back_to_blank() {
    assert_eq!(lucide_to_sprite("not-a-real-icon-name"), "blank");
}

// ---------------------------------------------------------------------------
// Run-control glyphs (ldui-kybz)
// ---------------------------------------------------------------------------
//
// `pause`, `play` and `save` (plus their alternate/consumer-vocabulary
// spellings) unblock `PageQuickActions` for control-plane toolbars like
// 4iiz-etl's operator portal. Each assertion follows the precedent set by
// `record_header.rs`'s `every_tone_glyph_resolves_in_the_sprite`: an alias
// that silently degrades to `blank` is indistinguishable from success at
// runtime (an empty `<svg>`, no error), so every name added to the table
// must be pinned here as resolving to something other than `blank`.

#[test]
fn pause_and_its_alternate_resolve() {
    assert_ne!(lucide_to_sprite("pause"), "blank");
    assert_ne!(lucide_to_sprite("square-pause"), "blank");
}

#[test]
fn play_resume_and_the_alternate_spelling_resolve_to_the_same_symbol() {
    assert_ne!(lucide_to_sprite("play"), "blank");
    assert_eq!(lucide_to_sprite("circle-play"), lucide_to_sprite("play"));
    assert_eq!(
        lucide_to_sprite("resume"),
        lucide_to_sprite("play"),
        "resume is the consumer-vocabulary alias onto play"
    );
}

#[test]
fn save_snapshot_and_the_alternate_spelling_resolve_to_the_same_symbol() {
    assert_ne!(lucide_to_sprite("save"), "blank");
    assert_eq!(lucide_to_sprite("floppy-disk"), lucide_to_sprite("save"));
    assert_eq!(
        lucide_to_sprite("snapshot"),
        lucide_to_sprite("save"),
        "snapshot is the consumer-vocabulary alias onto save"
    );
}

/// The exact six-button row from the bead
/// (Pause all / Resume all / Restart all / Snapshot / Reap orphans /
/// Freshness check) must render six real icons and no `blank`. `refresh`,
/// `trash` and `activity` already resolved before this bead; `pause`,
/// `resume` and `snapshot` are the three this bead adds.
#[test]
fn page_quick_actions_run_control_row_has_no_blank_icon() {
    let row = [
        ("Pause all", "pause"),
        ("Resume all", "resume"),
        ("Restart all", "refresh"),
        ("Snapshot", "snapshot"),
        ("Reap orphans", "trash"),
        ("Freshness check", "activity"),
    ];
    for (label, icon) in row {
        assert_ne!(
            lucide_to_sprite(icon),
            "blank",
            "{label} icon {icon:?} must not fall back to blank"
        );
    }
}

/// `ldui-q8bj`: an unmapped name must be REPORTED, not absorbed.
///
/// The old behaviour returned `blank` and nothing else changed — real box,
/// correct classes, well-formed `<use>` — so a typo was indistinguishable
/// from a working icon. These pin the three signals that now exist.
#[test]
fn an_unmapped_icon_name_is_distinguishable_from_a_mapped_one() {
    use super::component::{UNRESOLVED_SYMBOL, lucide_sprite_lookup, lucide_sprite_names};

    // A mapped name resolves to Some(symbol).
    assert_eq!(lucide_sprite_lookup("clock"), Some("clock"));

    // The two names that actually failed in 4iiz-Office. Both are real Lucide
    // names, which is why they were written; neither is in this crate's map.
    // The point is not that they are absent -- the host document's sprite
    // bounds the vocabulary -- but that their absence is now VISIBLE.
    for typo in ["clipboard-list", "user-check", "definitely-not-an-icon"] {
        assert_eq!(
            lucide_sprite_lookup(typo),
            None,
            "{typo:?} is unmapped, and the lookup must say so rather than \
             returning a plausible symbol"
        );
    }

    // The published vocabulary is non-empty and every entry actually resolves,
    // so a caller checking against it cannot be misled.
    let names = lucide_sprite_names();
    assert!(
        names.len() >= 50,
        "the vocabulary should be published in full"
    );
    for name in &names {
        assert!(
            lucide_sprite_lookup(name).is_some(),
            "published name {name:?} does not resolve -- the list has drifted \
             from the match arms"
        );
    }
    // Sorted, so a diff of the supported set is readable.
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);

    assert_eq!(UNRESOLVED_SYMBOL, "blank");
}

/// Regression guard for the bug my first `ldui-q8bj` fix introduced.
///
/// `"circle" => "blank"` is a legitimate EXPLICIT mapping to the blank glyph.
/// A first attempt inferred "unmapped" by comparing the resolved symbol to
/// `"blank"`, which misreported `circle` as a typo. The distinction is now
/// structural (`Option`), so it cannot be got wrong by inspection.
#[test]
fn an_explicit_mapping_to_blank_is_not_an_unmapped_name() {
    use super::component::lucide_sprite_lookup;

    assert_eq!(
        lucide_sprite_lookup("circle"),
        Some("blank"),
        "circle maps to the blank glyph ON PURPOSE and must resolve"
    );
    assert_eq!(lucide_sprite_lookup("not-a-real-icon-name"), None);
    // Both produce the same SYMBOL, which is exactly why comparing symbols
    // cannot distinguish them.
    assert_eq!(lucide_to_sprite("circle"), "blank");
    assert_eq!(lucide_to_sprite("not-a-real-icon-name"), "blank");
}
