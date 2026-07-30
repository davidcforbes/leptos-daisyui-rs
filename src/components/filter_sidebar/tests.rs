//! Guards for `FilterSidebar`'s MEASURED geometry and its one non-negotiable
//! behaviour.
//!
//! The numbers in this component are not design choices, they are measurements
//! taken from a running reference. A tidy-up that rounds 220px to `w-56` or 44px
//! to `w-11` would produce something that looks nearly right and no longer
//! matches the product it exists to match — and nothing else in the build would
//! object.

const SRC: &str = include_str!("component.rs");

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
