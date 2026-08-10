//! Guards for `FilterSidebar`'s MEASURED geometry and its one non-negotiable
//! behaviour.
//!
//! The numbers in this component are not design choices, they are measurements
//! taken from a running reference. A tidy-up that rounds 220px to `w-56` or 44px
//! to `w-11` would produce something that looks nearly right and no longer
//! matches the product it exists to match — and nothing else in the build would
//! object.

use super::style::{SidebarSide, join_side_class};

/// Both files, because `ldui-vh6` moved the orientation-dependent class
/// fragments out of the view and into `style.rs`. Scanning only `component.rs`
/// would quietly stop guarding them — a source-scanning test that reads the
/// wrong file passes forever.
const SRC: &str = concat!(include_str!("component.rs"), include_str!("style.rs"));

/// The view alone, for the assertions that must not be satisfied by a mention
/// in `style.rs`'s own doc comments.
const VIEW_SRC: &str = include_str!("component.rs");

#[test]
fn the_two_widths_are_the_measured_ones() {
    // Both `width` AND `min-width`: a flex parent will otherwise shrink the panel
    // below its own width and the collapse transition jumps.
    for cls in ["w-[220px] min-w-[220px]", "w-[44px] min-w-[44px]"] {
        assert!(
            SRC.contains(cls),
            "measured geometry `{cls}` is gone - rounding it to Tailwind's scale \
             produces a panel that looks nearly right and matches nothing"
        );
    }
}

#[test]
fn the_header_height_and_transition_are_the_measured_ones() {
    assert!(SRC.contains("h-[52px]"), "the header is 52px");
    assert!(
        SRC.contains("transition-[width,min-width] duration-[250ms]"),
        "both width and min-width must transition, over the measured 250ms - \
         animating only one leaves the other snapping"
    );
}

#[test]
fn the_collapsed_rail_shows_the_active_filter_count() {
    // THE MOST IMPORTANT LINE IN THIS FILE. A collapsed filter panel with no
    // indication of active filters is how a filtered list gets read as the whole
    // list - and on a work queue, that means believing there is less work than
    // there is.
    assert!(
        SRC.contains("active_count.get() != 0"),
        "the collapsed badge must be driven by the active filter count"
    );
    assert!(
        SRC.contains("bg-primary") && SRC.contains("size-[22px]"),
        "the badge is a 22px solid-primary circle"
    );
    assert!(
        SRC.contains("[writing-mode:vertical-rl]"),
        "the collapsed rail carries a vertical title, so a collapsed panel still \
         says what it is"
    );
}

#[test]
fn collapsing_hides_content_without_unmounting_it() {
    // Unmounting would be less code and worse behaviour: it loses scroll
    // position, breaks the width transition, and discards a half-typed value.
    assert!(
        SRC.contains("opacity-0 pointer-events-none"),
        "collapsed content must be hidden in place, not removed"
    );
    assert!(
        !SRC.contains("<Show when=move || !collapsed"),
        "the content must NOT be conditionally mounted"
    );
}

#[test]
fn the_toggle_survives_collapsing_and_is_labelled() {
    // A control that disappears when you use it is a trap: the toggle is the only
    // way back from the collapsed state.
    let header = SRC
        .split("── expanded content")
        .next()
        .expect("the header section must exist");
    assert!(
        header.contains("aria-label=move || toggle_label.get()"),
        "the toggle is icon-only, so it needs an accessible label"
    );
    assert!(
        header.contains("aria-expanded"),
        "the toggle must report its state to assistive technology"
    );
    assert!(
        !header.contains("hidden_when_collapsed()")
            || header.matches("hidden_when_collapsed()").count() == 1,
        "only the TITLE fades on collapse - the button must stay visible"
    );
}

// ── orientation (ldui-vh6) ──────────────────────────────────────────────────
//
// The class mapping is the part of this feature that rots silently: a wrong
// border side or a missing rotation still renders a panel, just a slightly
// wrong-looking one that nobody files. Each mapping is a pure function so it
// can be asserted here rather than eyeballed in a browser once.

#[test]
fn left_is_the_default_so_no_existing_caller_changes() {
    assert_eq!(SidebarSide::default(), SidebarSide::Left);
}

#[test]
fn left_emits_exactly_what_it_emitted_before_the_side_prop() {
    // THE COMPATIBILITY GUARD. This component is a path dependency in sibling
    // repos; `Left` must be byte-identical, not merely equivalent. Each of
    // these strings is copied from the pre-`ldui-vh6` markup.
    let left = SidebarSide::Left;
    assert_eq!(left.as_border_class(), "border-r");
    assert_eq!(left.chevron_name(true), "chevron-right");
    assert_eq!(left.chevron_name(false), "chevron-left");
    assert_eq!(
        left.as_rail_title_class(),
        "[writing-mode:vertical-rl] [text-orientation:mixed] rotate-180"
    );
    // Empty, NOT the equivalent `flex-row` - appending a visual no-op would
    // still change the emitted attribute.
    assert_eq!(left.as_header_class(), "");

    let header_base = "flex h-[52px] shrink-0 items-center justify-between px-3 pb-2.5 pt-3.5";
    assert_eq!(
        join_side_class(header_base, left.as_header_class()),
        header_base,
        "the default header class must come out unchanged, with no trailing space"
    );
}

#[test]
fn right_mirrors_every_one_of_the_four_sites() {
    let (l, r) = (SidebarSide::Left, SidebarSide::Right);

    // 1. The hairline moves to the other inner edge.
    assert_eq!(r.as_border_class(), "border-l");
    assert_ne!(l.as_border_class(), r.as_border_class());

    // 2. The chevron points the way the panel would move, so BOTH collapsed
    //    states invert - not just one of them, which is the easy half-fix.
    assert_eq!(r.chevron_name(true), "chevron-left");
    assert_eq!(r.chevron_name(false), "chevron-right");
    for collapsed in [true, false] {
        assert_ne!(l.chevron_name(collapsed), r.chevron_name(collapsed));
    }

    // 3. The toggle moves to the inner edge, i.e. the header row reverses.
    assert_eq!(r.as_header_class(), "flex-row-reverse");

    // 4. The rail title reads top-to-bottom, i.e. the half turn comes off.
    assert_eq!(
        r.as_rail_title_class(),
        "[writing-mode:vertical-rl] [text-orientation:mixed]"
    );
}

#[test]
fn the_rail_title_keeps_its_writing_mode_on_both_sides() {
    // Only the ROTATION flips. Dropping the writing-mode with it would leave a
    // horizontal title in a 44px rail, which truncates to nothing.
    for side in [SidebarSide::Left, SidebarSide::Right] {
        assert!(
            side.as_rail_title_class()
                .contains("[writing-mode:vertical-rl]"),
            "{side:?} lost its vertical writing mode"
        );
        assert!(
            side.as_rail_title_class()
                .contains("[text-orientation:mixed]"),
            "{side:?} lost its text orientation"
        );
    }
    assert!(
        SidebarSide::Left
            .as_rail_title_class()
            .contains("rotate-180")
    );
    assert!(
        !SidebarSide::Right
            .as_rail_title_class()
            .contains("rotate-180"),
        "a right-edge label reads top-to-bottom; the half turn is what makes a \
         left-edge one read bottom-to-top"
    );
    // No `origin-*` on either: the span turns about its own centre, and an
    // origin correction would push the label off the 44px rail.
    for side in [SidebarSide::Left, SidebarSide::Right] {
        assert!(!side.as_rail_title_class().contains("origin-"));
    }
}

#[test]
fn join_side_class_never_emits_a_stray_separator() {
    assert_eq!(join_side_class("a b", ""), "a b");
    assert_eq!(join_side_class("a b", "c"), "a b c");
}

#[test]
fn the_view_reads_every_orientation_mapping() {
    // A mapping that exists and is never called is worse than none: the unit
    // tests above stay green while the panel renders left-oriented on both
    // sides. Assert the view actually consults each one.
    for call in [
        "as_border_class()",
        "chevron_name(collapsed.get())",
        "as_header_class()",
        "as_rail_title_class()",
    ] {
        assert!(
            VIEW_SRC.contains(call),
            "`{call}` is mapped but never read by the view"
        );
    }
    assert!(
        !VIEW_SRC.contains("overflow-hidden border-r"),
        "the border side must come from `SidebarSide`, not be hardcoded on the panel"
    );
}

#[test]
fn the_search_affordance_is_not_mirrored() {
    // Reading direction, not panel orientation. If someone "finishes the
    // mirroring" by flipping these, the icon lands where the caret goes.
    assert!(
        VIEW_SRC.contains("absolute left-2.5") && VIEW_SRC.contains("pl-[30px] pr-2.5"),
        "the filter-search magnifier stays on the left on BOTH sides"
    );
}
