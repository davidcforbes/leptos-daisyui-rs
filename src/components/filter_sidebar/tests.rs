//! Guards for `FilterSidebar`'s MEASURED geometry and its one non-negotiable
//! behaviour.
//!
//! The numbers in this component are not design choices, they are measurements
//! taken from a running reference. A tidy-up that rounds 220px to `w-56` or 44px
//! to `w-11` would produce something that looks nearly right and no longer
//! matches the product it exists to match — and nothing else in the build would
//! object.

use super::component::filter_sidebar_header_actions_wrapper;
use super::style::{SidebarSide, join_side_class};
use leptos::prelude::*;
use leptos::reactive::owner::Owner;

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
    // The title AND the optional `header_actions` slot (ldui-bx6n) both fade
    // on collapse; only the button element itself must stay put. Isolate the
    // button's own markup rather than counting fades across the whole header.
    let button_markup = header
        .split("<button")
        .nth(1)
        .expect("the toggle button must exist");
    assert!(
        !button_markup.contains("hidden_when_collapsed()")
            && !button_markup.contains("opacity-0 pointer-events-none"),
        "the toggle itself must stay visible on collapse, even though the \
         title and any header_actions fade"
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

// ── search accessible name (ldui-g66e) ──────────────────────────────────────
//
// The search box previously carried only a `placeholder` -- placeholder text
// is not an accessible name (it disappears once the field is focused or
// filled, and many screen readers never announce it at all). These guard the
// fix the same way `data_table`'s `search_label`/`search_input_id` pattern
// does: a real `<label>` associated by `id`/`for`, reactive so it can trail a
// locale switch, and never emitted when `search` is omitted.

#[test]
fn search_label_prop_is_reactive_with_a_documented_fallback() {
    assert!(
        VIEW_SRC.contains("search_label: Signal<String>"),
        "the accessible label must be a reactive Signal<String>, matching \
         `title`/`toggle_label`, so it can trail a locale switch"
    );
    assert!(
        VIEW_SRC.contains("Signal::stored(String::from(\"Search filters\"))"),
        "existing callers that do not supply `search_label` must still get a \
         real, documented accessible name rather than none"
    );
}

#[test]
fn search_input_has_an_associated_label_independent_of_placeholder_and_value() {
    let search_block = VIEW_SRC
        .split(".map(|s| {")
        .nth(1)
        .expect("the optional search box must map over `search: Option<RwSignal<String>>`");

    assert!(
        search_block.contains("<label class=\"sr-only\""),
        "the search input needs a visually-hidden, screen-reader-only label"
    );
    assert!(
        search_block.contains("r#for="),
        "the label must be associated to the input via `for`/`id`, not merely \
         placed nearby"
    );
    assert!(
        search_block.contains("aria-label=move || search_label.get()"),
        "the accessible name must come from `search_label`, not `search_placeholder`"
    );
    assert!(
        !search_block.contains("aria-label=move || search_placeholder.get()"),
        "the accessible name must be independent of the placeholder text"
    );
    // The `aria-label` line must not read the search VALUE signal `s` --
    // the name must not change as the user types.
    let aria_label_line = search_block
        .lines()
        .find(|line| line.contains("aria-label="))
        .expect("an aria-label attribute must exist on the search input");
    assert!(
        !aria_label_line.contains("s.get()"),
        "the accessible name must be independent of the typed value: {aria_label_line}"
    );
}

#[test]
fn search_input_id_is_generated_per_instance() {
    assert!(
        VIEW_SRC.contains("next_filter_sidebar_search_id"),
        "each FilterSidebar instance needs its own id so multiple panels on \
         one page stay independently named -- a hardcoded id would collide"
    );
    assert!(
        VIEW_SRC.contains("AtomicU64"),
        "the id generator must be a process-wide counter, mirroring \
         DataTable's `next_data_table_search_id`"
    );
}

#[test]
fn no_hidden_label_is_emitted_when_search_is_omitted() {
    // The label markup must live ONLY inside the `search.map(...)` branch --
    // there must be exactly one occurrence of the sr-only search label in the
    // source, and it must not exist as a sibling emitted unconditionally.
    assert_eq!(
        VIEW_SRC.matches("<label class=\"sr-only\"").count(),
        1,
        "the sr-only search label must be emitted exactly once, inside the \
         `search.map` branch -- never unconditionally"
    );
}

// ── header-actions slot (ldui-bx6n) ─────────────────────────────────────────
//
// A collapsible right-side Assistant panel needs panel-scoped controls (a
// model selector plus a setup action) in the SAME header row as the title and
// toggle. Mirrors `FilterBar`'s own `Option`-gated slot idiom
// (`filter_bar_children_wrapper`): an unconditionally-rendered empty wrapper
// is a phantom flex item, so the wrapper-building logic is a standalone pure
// function that can be asserted directly without a DOM/SSR renderer (this
// crate has none -- see the module doc comment on `filter_bar/tests.rs`).

#[test]
fn header_actions_wrapper_is_absent_when_no_slot_is_supplied() {
    let owner = Owner::new();
    owner.with(|| {
        let collapsed = Signal::stored(false);
        assert!(
            filter_sidebar_header_actions_wrapper(None, collapsed).is_none(),
            "an absent header_actions slot must emit no wrapper node at all, \
             so existing callers stay render-compatible"
        );
    });
}

#[test]
fn header_actions_wrapper_is_present_when_a_slot_is_supplied() {
    let owner = Owner::new();
    owner.with(|| {
        let collapsed = Signal::stored(false);
        assert!(
            filter_sidebar_header_actions_wrapper(
                Some(ToChildren::to_children(|| view! { "model select" })),
                collapsed,
            )
            .is_some(),
            "a supplied header_actions slot must still emit its wrapper"
        );
    });
}

#[test]
fn header_actions_slot_is_typed_and_optional_in_the_props() {
    assert!(
        VIEW_SRC.contains("header_actions: Option<Children>"),
        "header_actions must be an optional typed composition slot, matching \
         the `Option<Children>` shape `FilterBar` uses for its own slots"
    );
}

#[test]
fn header_actions_sits_between_the_title_and_the_toggle() {
    // DOM order, not CSS, is what carries left/right mirroring: the header
    // row's `flex-row-reverse` (ldui-vh6) reverses whatever is written here,
    // so the slot must be written between the title and the toggle button in
    // source order for both sides to come out correct.
    let header = VIEW_SRC
        .split("── expanded content")
        .next()
        .expect("the header section must exist");
    let title_pos = header
        .find("{move || title.get()}")
        .expect("the title must be rendered in the header");
    let actions_pos = header
        .find("filter_sidebar_header_actions_wrapper(header_actions, collapsed)")
        .expect("the header_actions wrapper must be called in the header");
    let button_pos = header
        .find("<button")
        .expect("the toggle button must exist in the header");
    assert!(
        title_pos < actions_pos && actions_pos < button_pos,
        "header_actions must sit between the title and the toggle in DOM order"
    );
}

// ── ldui-8hba: collapsed header actions contribute zero layout width ───────
//
// `ldui-bx6n` hid the collapsed slot with `opacity-0 pointer-events-none`
// alone, which hides paint but not layout: `shrink-0` kept the wrapper's
// full intrinsic width, and on a right-docked panel with a wide slot that
// pushed the toggle button past the panel's own edge, where the panel's own
// `overflow-hidden` clipped it -- `elementFromPoint` at the toggle's center
// resolved whatever sat underneath instead (a P1 consumer blocker: the
// toggle could not be clicked at all). These guard the fix: the collapsed
// wrapper must claim zero width, not merely zero opacity, and must be
// removed from the tab order and accessibility tree rather than merely
// losing pointer events.

#[test]
fn collapsed_header_actions_claim_zero_layout_width() {
    let source = include_str!("component.rs");
    let wrapper_fn = source
        .split("pub(crate) fn filter_sidebar_header_actions_wrapper")
        .nth(1)
        .expect("the header_actions wrapper function must exist");
    assert!(
        wrapper_fn.contains("w-0"),
        "the collapsed wrapper must claim zero width -- opacity alone hides \
         paint, not the layout space that pushed the toggle off the panel \
         in ldui-8hba"
    );
    assert!(
        wrapper_fn.contains("mx-0"),
        "the collapsed wrapper's margin must also collapse to zero, or the \
         margin alone still pushes the toggle"
    );
    assert!(
        wrapper_fn.contains("overflow-hidden"),
        "the collapsed wrapper must clip its own children so nothing inside \
         it can bleed past the zero-width box"
    );
}

#[test]
fn collapsed_header_actions_are_unfocusable_and_unannounced() {
    let source = include_str!("component.rs");
    let wrapper_fn = source
        .split("pub(crate) fn filter_sidebar_header_actions_wrapper")
        .nth(1)
        .expect("the header_actions wrapper function must exist");
    assert!(
        wrapper_fn.contains("inert=move || collapsed.get()"),
        "collapsed header actions must go `inert`, which removes them from \
         both the tab order and the accessibility tree -- \
         `pointer-events-none` alone leaves them keyboard- and \
         screen-reader-reachable"
    );
    assert!(
        wrapper_fn.contains("aria-hidden=move || collapsed.get().to_string()"),
        "belt-and-braces `aria-hidden`, matching the collapsed rail's own \
         treatment, in case a user agent does not yet honour `inert`"
    );
}

#[test]
fn header_actions_fades_with_the_title_on_collapse() {
    // "Shown only in the expanded header" (ldui-bx6n): hidden in place on
    // collapse, matching the title and main content's own treatment, never
    // unmounted -- collapsing must not discard state inside a header control.
    let source = include_str!("component.rs");
    let wrapper_fn = source
        .split("pub(crate) fn filter_sidebar_header_actions_wrapper")
        .nth(1)
        .expect("the header_actions wrapper function must exist");
    assert!(
        wrapper_fn.contains("opacity-0 pointer-events-none"),
        "the header_actions slot must hide in place on collapse, not merely \
         become invisible while still catching clicks"
    );
    assert!(
        wrapper_fn.contains("collapsed.get()"),
        "the fade must be driven by the same `collapsed` signal as the title"
    );
}
