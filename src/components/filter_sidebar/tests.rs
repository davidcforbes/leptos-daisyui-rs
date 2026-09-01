//! Guards for `FilterSidebar`'s MEASURED geometry and its one non-negotiable
//! behaviour.
//!
//! The numbers in this component are not design choices, they are measurements
//! taken from a running reference. A tidy-up that rounds 220px to `w-56` or 44px
//! to `w-11` would produce something that looks nearly right and no longer
//! matches the product it exists to match — and nothing else in the build would
//! object.

use super::component::filter_sidebar_header_actions_wrapper;
use super::style::{FilterSidebarTogglePlacement, SidebarSide, join_side_class};
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

// ── main content is `inert` and `aria-hidden` while collapsed (ldui-gae5) ──
//
// `opacity-0 pointer-events-none` hides PAINT, not the accessibility tree or
// the tab order: a collapsed panel's search input and every control inside
// `children` (the bug report's example was a textarea and an action button)
// stayed keyboard-focusable and screen-reader-announced. These guard the
// fix the same way `ldui-8hba` guards `header_actions`'s identical
// treatment: `inert` (which removes a subtree from both focus and the
// accessibility tree) plus a belt-and-braces `aria-hidden`, scoped to the
// main content region specifically so a regression there cannot hide behind
// an unrelated `inert` elsewhere in the file.

/// Isolates the "expanded content" `<div>` -- the main content region
/// (search box + `children`) -- by its unique `data-filter-sidebar-content`
/// marker, bounded at the next `<div` so an assertion about what this
/// region does NOT contain cannot be satisfied by later markup (the
/// collapsed rail, for instance, has its own unrelated `aria-hidden`).
fn main_content_div_markup() -> &'static str {
    let after_marker = VIEW_SRC
        .split("── expanded content")
        .nth(1)
        .expect("the expanded content region must exist");
    after_marker
        .split("{search")
        .next()
        .expect("the content div's opening tag must precede the search slot")
}

#[test]
fn collapsed_main_content_is_inert_and_aria_hidden() {
    let content = main_content_div_markup();
    assert!(
        content.contains("inert=move || collapsed.get()"),
        "the main content region must go `inert` while collapsed, removing \
         its search box and every child control (a textarea, an action \
         button, anything) from both the tab order and the accessibility \
         tree -- this is the ldui-gae5 fix"
    );
    assert!(
        content.contains("aria-hidden=move || collapsed.get().to_string()"),
        "belt-and-braces `aria-hidden`, matching `header_actions`'s own \
         treatment (ldui-8hba), in case a user agent does not yet honour \
         `inert`"
    );
    assert!(
        content.contains("data-filter-sidebar-content=\"true\""),
        "a stable data attribute is required to select this region without \
         relying on document position"
    );
}

#[test]
fn collapse_transition_still_clips_via_the_asides_own_overflow_hidden() {
    // `inert` changes focusability and announcement, nothing about layout
    // or paint -- the `<aside>`'s own `overflow-hidden` must still be the
    // thing that CLIPS the 220px content during the width transition, or
    // the transition visibly reflows text on every frame (see the `<aside>`
    // class comment). This must not have moved onto the content div itself,
    // which would double-clip and is not what was asked for.
    //
    // Matched via the aside's class format string directly rather than by
    // locating its opening tag: the repo checks out with CRLF line endings,
    // which would break a newline-anchored split, and the type docs also
    // mention `` `<aside>` `` (closed immediately, no attributes) elsewhere
    // in the file, which a bare `"<aside"` search would match first instead
    // of the real multi-line tag. `"flex-col overflow-hidden"` is the literal
    // head of the aside's own `format!` class string and appears nowhere else.
    assert!(
        VIEW_SRC.contains("flex-col overflow-hidden"),
        "the aside must still own the clip that makes the width transition \
         non-reflowing"
    );
}

/// Isolates `filter_sidebar_toggle_button`'s own function body -- the ONE
/// place the toggle's markup is written (`ldui-vshu`), shared by
/// `FilterSidebar`'s internal header button and the external
/// `FilterSidebarToggle`. Bounded at the next doc comment so assertions
/// about what the button does NOT contain (e.g. a fade class) cannot be
/// satisfied by unrelated markup later in the file.
fn toggle_button_fn_body() -> &'static str {
    VIEW_SRC
        .split("fn filter_sidebar_toggle_button(")
        .nth(1)
        .expect("the shared toggle button function must exist")
        .split("\n/// # FilterSidebarToggle")
        .next()
        .expect("the function must be followed by FilterSidebarToggle's doc comment")
}

#[test]
fn the_toggle_survives_collapsing_and_is_labelled() {
    // A control that disappears when you use it is a trap: the toggle is the only
    // way back from the collapsed state. The markup lives in ONE shared
    // function now (`ldui-vshu`), so this asserts against that function
    // directly rather than the header region.
    let toggle_fn = toggle_button_fn_body();
    assert!(
        toggle_fn.contains("aria-label=move || toggle_label.get()"),
        "the toggle is icon-only, so it needs an accessible label"
    );
    assert!(
        toggle_fn.contains("aria-expanded=move || (!collapsed.get()).to_string()"),
        "the toggle must report its state to assistive technology"
    );
    assert!(
        !toggle_fn.contains("hidden_when_collapsed()")
            && !toggle_fn.contains("opacity-0 pointer-events-none"),
        "the toggle itself must stay visible on collapse, even though the \
         title and any header_actions fade"
    );
}

// ── externally placed toggle (ldui-vshu) ────────────────────────────────────
//
// A consumer with its own page-level Hide/Show action must be able to place
// the panel's toggle elsewhere without duplicating it or hand-rolling ARIA
// that could drift from the built-in control. `filter_sidebar_toggle_button`
// is the single source of truth both paths call through; these guard that
// (a) it is genuinely the same function on both paths, (b) the internal
// button is omitted entirely -- not hidden -- for `External` placement, and
// (c) `FilterSidebarToggle` is a real typed component, not a documented
// pattern the consumer has to hand-assemble.

#[test]
fn toggle_placement_defaults_to_internal_so_every_existing_caller_is_unchanged() {
    assert_eq!(
        FilterSidebarTogglePlacement::default(),
        FilterSidebarTogglePlacement::Internal,
        "omitting `toggle_placement` must reproduce the pre-ldui-vshu \
         behaviour exactly"
    );
}

#[test]
fn filter_sidebar_toggle_placement_prop_is_typed_and_reactive() {
    assert!(
        VIEW_SRC.contains("toggle_placement: Signal<FilterSidebarTogglePlacement>"),
        "toggle_placement must be a reactive, typed knob -- matching the \
         `side` prop's own shape -- not a stringly-typed or untyped slot"
    );
}

#[test]
fn internal_toggle_is_gated_on_the_placement_and_calls_the_shared_function() {
    let filter_sidebar_body = VIEW_SRC
        .split("pub fn FilterSidebar(")
        .nth(1)
        .expect("FilterSidebar must exist");
    assert!(
        filter_sidebar_body
            .contains("matches!(toggle_placement.get(), FilterSidebarTogglePlacement::Internal)"),
        "the internal toggle must be conditional on the placement, not \
         unconditionally rendered"
    );
    assert!(
        filter_sidebar_body.contains(".then(|| {")
            && filter_sidebar_body.contains("filter_sidebar_toggle_button("),
        "the internal toggle must render through the SAME shared function \
         `FilterSidebarToggle` uses, so the two paths cannot drift apart"
    );
    // `None` for `controls`: the internal toggle's markup must stay
    // byte-for-byte what it was before ldui-vshu -- no new `aria-controls`
    // sneaking onto a mode the bead requires stay unchanged.
    let toggle_call = filter_sidebar_body
        .split("filter_sidebar_toggle_button(")
        .nth(1)
        .expect("the internal call site must exist");
    // Bounded to the call's own argument list (up to its closing paren) so
    // this cannot be satisfied by `Some(controls)` further down in
    // `FilterSidebarToggle`'s own call site.
    let call_args = toggle_call
        .split(')')
        .next()
        .expect("the call must have a closing paren");
    assert!(
        call_args.contains("None"),
        "the internal toggle must pass `controls: None`, preserving its \
         pre-ldui-vshu markup exactly"
    );
    assert!(
        !call_args.contains("Some("),
        "the internal toggle must NOT pass `aria-controls` -- that would be \
         a behaviour change for the mode the bead requires stay unchanged"
    );
}

#[test]
fn external_placement_leaves_no_toggle_node_for_the_panel_to_render() {
    // Not merely hidden -- the acceptance criteria require NO
    // phantom/hidden internal toggle at all when placement is External.
    // The gate above proves the render is `Option`-conditional
    // (`bool::then`), which means `None` emits no node whatsoever, matching
    // how `header_actions` omits its own wrapper when the slot is absent.
    let filter_sidebar_body = VIEW_SRC
        .split("pub fn FilterSidebar(")
        .nth(1)
        .expect("FilterSidebar must exist");
    assert!(
        filter_sidebar_body.contains(".then(|| {"),
        "the internal toggle must be `Option`-conditional so `External` \
         placement renders nothing, not something hidden"
    );
}

#[test]
fn filter_sidebar_toggle_is_a_typed_component_sharing_the_button_function() {
    assert!(
        VIEW_SRC.contains("pub fn FilterSidebarToggle("),
        "the external toggle must be a real typed component, not a \
         documented pattern the consumer hand-assembles"
    );
    let component_body = VIEW_SRC
        .split("pub fn FilterSidebarToggle(")
        .nth(1)
        .expect("FilterSidebarToggle must exist");
    assert!(
        component_body.contains("controls: Signal<String>"),
        "FilterSidebarToggle must accept the paired panel's id so its \
         aria-controls is truthful"
    );
    assert!(
        component_body.contains("filter_sidebar_toggle_button(")
            && component_body.contains("Some(controls)"),
        "FilterSidebarToggle must render through the shared function with \
         `aria-controls` set, matching the built-in toggle in every other \
         respect"
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
    //
    // Scoped to `FilterSidebar`'s OWN body (`ldui-vshu`): the literal
    // "<button" text now lives earlier in the file, inside the shared
    // `filter_sidebar_toggle_button` function both this panel's internal
    // toggle and `FilterSidebarToggle` call through -- searching the whole
    // preamble for it would find that definition instead of the call site.
    let header = VIEW_SRC
        .split("pub fn FilterSidebar(")
        .nth(1)
        .expect("FilterSidebar must exist")
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
        .find("filter_sidebar_toggle_button(")
        .expect("the internal toggle's call site must exist in the header");
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
