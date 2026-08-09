use super::*;

fn cells(labels: &[&str]) -> Vec<RosterCell> {
    labels
        .iter()
        .map(|l| RosterCell::new(*l, ShiftState::Full))
        .collect()
}

// ---------------------------------------------------------------------
// normalize_cells -- the ragged-input rule (requirement 3)
// ---------------------------------------------------------------------

#[test]
fn normalize_cells_exact_length_is_unchanged() {
    let input = cells(&["a", "b", "c"]);
    assert_eq!(normalize_cells(&input, 3), input);
}

#[test]
fn normalize_cells_pads_a_short_row_with_off() {
    let out = normalize_cells(&cells(&["a"]), 5);
    assert_eq!(out.len(), 5);
    assert_eq!(out[0].label, "a");
    for pad in &out[1..] {
        assert_eq!(pad, &RosterCell::off());
        assert_eq!(pad.state, ShiftState::Off);
        assert!(pad.label.is_empty());
    }
}

#[test]
fn normalize_cells_truncates_a_long_row() {
    let out = normalize_cells(&cells(&["a", "b", "c", "d", "e"]), 2);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].label, "a");
    assert_eq!(out[1].label, "b");
}

#[test]
fn normalize_cells_keeps_position_so_columns_never_slip() {
    // The whole point of padding rather than dropping: cell `i` must still be
    // under column `i` after normalisation, in both directions.
    let out = normalize_cells(&cells(&["mon", "tue"]), 4);
    assert_eq!(out[0].label, "mon");
    assert_eq!(out[1].label, "tue");
    let out = normalize_cells(&cells(&["mon", "tue", "wed", "thu", "fri"]), 3);
    assert_eq!(out[2].label, "wed");
}

#[test]
fn normalize_cells_empty_input_pads_to_full_width() {
    let out = normalize_cells(&[], 5);
    assert_eq!(out.len(), 5);
    assert!(out.iter().all(|c| c.state == ShiftState::Off));
}

#[test]
fn normalize_cells_zero_columns_yields_no_cells() {
    // Empty `columns` is an empty state in the component, but the pure rule
    // must still be total rather than panicking on a zero-width grid.
    assert!(normalize_cells(&cells(&["a", "b"]), 0).is_empty());
    assert!(normalize_cells(&[], 0).is_empty());
}

#[test]
fn normalize_cells_is_idempotent() {
    let once = normalize_cells(&cells(&["a"]), 4);
    let twice = normalize_cells(&once, 4);
    assert_eq!(once, twice);
}

// ---------------------------------------------------------------------
// Columns -- caller-supplied, with a Mon-Fri default
// ---------------------------------------------------------------------

#[test]
fn default_columns_are_monday_to_friday() {
    assert_eq!(DEFAULT_ROSTER_COLUMNS, ["Mon", "Tue", "Wed", "Thu", "Fri"]);
    assert_eq!(
        default_roster_columns(),
        vec!["Mon", "Tue", "Wed", "Thu", "Fri"]
    );
}

#[test]
fn default_empty_title_is_non_empty() {
    // The empty state must actually say something; an empty title would render
    // as a blank box, which reads as a broken table rather than "no data".
    assert!(!default_empty_title().is_empty());
}

// ---------------------------------------------------------------------
// ShiftState -- every variant maps to classes and a name
// ---------------------------------------------------------------------

#[test]
fn shift_state_all_lists_every_variant_exactly_once() {
    // Guards the demo/legend and the exhaustive tests below: adding a variant
    // without adding it here would silently skip it everywhere.
    let mut seen = ShiftState::ALL.to_vec();
    seen.sort_by_key(|s| format!("{s:?}"));
    seen.dedup();
    assert_eq!(seen.len(), ShiftState::ALL.len());
    assert_eq!(ShiftState::ALL.len(), 5);
}

#[test]
fn shift_state_class_mappings_are_distinct_and_non_empty() {
    let mut bgs: Vec<&str> = ShiftState::ALL.iter().map(|s| s.as_class()).collect();
    assert!(bgs.iter().all(|c| !c.is_empty()));
    bgs.sort_unstable();
    bgs.dedup();
    assert_eq!(
        bgs.len(),
        5,
        "two states share a tint -- they'd be unreadable"
    );

    let mut borders: Vec<&str> = ShiftState::ALL
        .iter()
        .map(|s| s.as_border_class())
        .collect();
    borders.sort_unstable();
    borders.dedup();
    assert_eq!(borders.len(), 5);
}

#[test]
fn shift_state_exact_class_mapping() {
    assert_eq!(ShiftState::Full.as_class(), "bg-success/15");
    assert_eq!(ShiftState::Half.as_class(), "bg-info/15");
    assert_eq!(ShiftState::Off.as_class(), "bg-base-200/60");
    assert_eq!(ShiftState::Holiday.as_class(), "bg-accent/15");
    assert_eq!(ShiftState::Leave.as_class(), "bg-warning/15");
}

#[test]
fn shift_state_labels_are_distinct_and_non_empty() {
    let mut labels: Vec<&str> = ShiftState::ALL.iter().map(|s| s.as_label()).collect();
    assert!(labels.iter().all(|l| !l.is_empty()));
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), 5);
}

/// Colour must not be the only channel: the working/not-working split is also
/// encoded in the border *style*, which survives greyscale and colour
/// blindness. This is the test that fails if someone "tidies" the border
/// classes into one shared value.
#[test]
fn border_style_encodes_working_state_without_colour() {
    for &state in ShiftState::ALL {
        let b = state.as_border_class();
        if state.is_working() {
            assert!(b.contains("border-solid"), "{state:?} -> {b}");
        } else {
            assert!(b.contains("border-dashed"), "{state:?} -> {b}");
        }
    }
}

#[test]
fn is_working_is_true_only_for_the_two_shift_variants() {
    assert!(ShiftState::Full.is_working());
    assert!(ShiftState::Half.is_working());
    assert!(!ShiftState::Off.is_working());
    assert!(!ShiftState::Holiday.is_working());
    assert!(!ShiftState::Leave.is_working());
}

#[test]
fn shift_state_default_is_off() {
    // A cell nobody filled in is not rostered; anything else would invent work.
    assert_eq!(ShiftState::default(), ShiftState::Off);
    assert_eq!(RosterCell::default().state, ShiftState::Off);
    assert_eq!(RosterCell::off(), RosterCell::default());
}

// ---------------------------------------------------------------------
// RosterDensity -- a size ramp, not spacing
// ---------------------------------------------------------------------

#[test]
fn both_densities_map_to_distinct_classes() {
    for d in [RosterDensity::Compact, RosterDensity::Comfortable] {
        assert!(!d.as_table_class().is_empty());
        assert!(!d.as_cell_class().is_empty());
    }
    assert_ne!(
        RosterDensity::Compact.as_table_class(),
        RosterDensity::Comfortable.as_table_class()
    );
    assert_ne!(
        RosterDensity::Compact.as_cell_class(),
        RosterDensity::Comfortable.as_cell_class()
    );
}

#[test]
fn density_default_is_comfortable() {
    assert_eq!(RosterDensity::default(), RosterDensity::Comfortable);
}

#[test]
fn row_heights_are_a_size_ramp_on_the_4px_grid() {
    // Sizes answer "how big?" and follow their own ramp; they are NOT required
    // to sit on the nine-step spacing scale (cf. `IconSize`). They must still
    // land on the 4px grid, and must ascend.
    let compact = RosterDensity::Compact.row_height_px();
    let comfortable = RosterDensity::Comfortable.row_height_px();
    assert_eq!(compact % 4, 0);
    assert_eq!(comfortable % 4, 0);
    assert!(compact < comfortable);
}

#[test]
fn comfortable_row_height_matches_the_shared_table_row_token() {
    // Where the desktop face names a row height, the web face must agree --
    // 40 is deliberately off the spacing scale for exactly this reason.
    assert_eq!(
        RosterDensity::Comfortable.row_height_px() as f32,
        ui_tokens::spacing::TABLE_ROW_HEIGHT
    );
    assert!(!ui_tokens::spacing::is_on_scale(
        RosterDensity::Comfortable.row_height_px() as f32
    ));
}

#[test]
fn cell_class_height_step_matches_row_height_px() {
    // The class string and the pixel value are two encodings of one decision;
    // a Tailwind height step is 4px, so `h-N` must be `row_height_px() / 4`.
    for d in [RosterDensity::Compact, RosterDensity::Comfortable] {
        let expected = format!("h-{}", d.row_height_px() / 4);
        assert!(
            d.as_cell_class().split_whitespace().any(|c| c == expected),
            "{d:?} class {:?} lacks {expected}",
            d.as_cell_class()
        );
    }
}

/// Internal padding must not exceed the gap between neighbours: the tile pads
/// `px-2` (8px) at BOTH densities while the cell pads `p-1` (4px) on each side,
/// making the inter-tile gap 8px. Growing tile padding with the density would
/// break that at the comfortable step, so the density ramp deliberately carries
/// only height and text size.
#[test]
fn density_ramp_carries_no_horizontal_padding() {
    for d in [RosterDensity::Compact, RosterDensity::Comfortable] {
        for class in d.as_cell_class().split_whitespace() {
            assert!(
                !class.starts_with("px-") && !class.starts_with("p-") && !class.starts_with("py-"),
                "{d:?} smuggled padding into the size ramp: {class}"
            );
        }
    }
}

// ---------------------------------------------------------------------
// Accessible naming
// ---------------------------------------------------------------------

#[test]
fn cell_aria_label_includes_worker_column_label_and_state() {
    let cell = RosterCell::new("09:00-17:00", ShiftState::Full);
    assert_eq!(
        cell_aria_label("Ada Lovelace", "Mon", &cell, ShiftState::Full.as_label()),
        "Ada Lovelace, Mon, 09:00-17:00, Full shift"
    );
}

#[test]
fn cell_aria_label_omits_an_empty_label_without_leaving_a_gap() {
    // A blank tile must not announce "Ada, Mon, , Off" -- the doubled comma
    // is read aloud as a pause with nothing in it.
    let cell = RosterCell::off();
    assert_eq!(
        cell_aria_label("Ada", "Sat", &cell, ShiftState::Off.as_label()),
        "Ada, Sat, Off"
    );
    assert!(!cell_aria_label("Ada", "Sat", &cell, "Off").contains(", ,"));
}

#[test]
fn cell_aria_label_uses_the_supplied_state_name_not_the_english_default() {
    // The localisation hook: `state_label`'s output flows straight through.
    let cell = RosterCell::new("08-16", ShiftState::Full);
    let out = cell_aria_label("Ada", "Lun", &cell, "Jornada completa");
    assert!(out.contains("Jornada completa"));
    assert!(!out.contains("Full shift"));
}

#[test]
fn cell_aria_label_carries_every_state_name() {
    for &state in ShiftState::ALL {
        let cell = RosterCell::new("x", state);
        assert!(cell_aria_label("W", "C", &cell, state.as_label()).ends_with(state.as_label()));
    }
}

// ---------------------------------------------------------------------
// Naming the table -- Modal's label / labelled_by rule
// ---------------------------------------------------------------------

#[test]
fn labelled_by_suppresses_aria_label() {
    // An `aria-label` would override the visible heading `labelled_by` points
    // at, so a screen reader would hear something different from what sighted
    // users read. Same rule as `modal_aria_label`.
    assert_eq!(
        roster_table_aria_label(Some("Ward B".to_string()), true),
        None
    );
    assert_eq!(roster_table_aria_label(None, true), None);
}

#[test]
fn a_label_with_no_labelled_by_becomes_the_aria_label() {
    assert_eq!(
        roster_table_aria_label(Some("Ward B, week of 12 May".to_string()), false),
        Some("Ward B, week of 12 May".to_string())
    );
}

/// Unlike `Modal`, there is NO generic fallback. An unnamed `<dialog>` is an
/// axe violation; an unnamed `<table>` is ordinary, and inventing "Roster"
/// would announce English into a localised page.
#[test]
fn an_unnamed_table_gets_no_invented_name() {
    assert_eq!(roster_table_aria_label(None, false), None);
}

// ---------------------------------------------------------------------
// Tooltips -- the tile's own value, not its accessible name
// ---------------------------------------------------------------------

#[test]
fn cell_title_is_the_shift_value_only() {
    let cell = RosterCell::new("09:00-17:00", ShiftState::Full);
    let title = cell_title(&cell).expect("a labelled tile has a tooltip");
    assert_eq!(title, "09:00-17:00");
    // NOT the accessible name: duplicating it tooltips visible content on the
    // display-only path and doubles `aria-label` on the interactive one.
    let aria = cell_aria_label("Ada", "Mon", &cell, ShiftState::Full.as_label());
    assert_ne!(title, aria);
    assert!(!title.contains("Ada"));
    assert!(!title.contains("Full shift"));
}

#[test]
fn an_empty_tile_has_no_tooltip_at_all() {
    // A tooltip exists to reveal a value `truncate` ellipsised; there is no
    // value here, so an empty `title` attribute would be noise.
    assert_eq!(cell_title(&RosterCell::off()), None);
    assert_eq!(cell_title(&RosterCell::new("", ShiftState::Holiday)), None);
}

// ---------------------------------------------------------------------
// Shared horizontal-overflow contract
// ---------------------------------------------------------------------

/// The scroll wrapper is DataTable's `TABLE_SCROLL_WRAPPER_CLASS`, not a
/// second spelling of `overflow-x-auto`. `pub(super)` on that const means
/// "visible throughout `crate::components`", and `roster_grid` is inside it —
/// so reusing it needed no visibility widening.
#[test]
fn the_scroll_wrapper_is_the_shared_constant_not_a_copy() {
    use crate::components::data_table::TABLE_SCROLL_WRAPPER_CLASS;

    let source = include_str!("component.rs");
    assert!(
        source.contains("TABLE_SCROLL_WRAPPER_CLASS"),
        "RosterGrid stopped sharing the horizontal-scroll contract"
    );
    // Doc lines are excluded: the `@source inline(...)` block legitimately
    // lists the emitted classes for Tailwind, which is not a second definition.
    let code_only = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("///"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code_only.contains("w-full overflow-x-auto"),
        "the literal came back -- the contract now lives in two places"
    );
    assert!(
        TABLE_SCROLL_WRAPPER_CLASS
            .split_ascii_whitespace()
            .any(|class| class == "overflow-x-auto")
    );
}

// ---------------------------------------------------------------------
// Keyboard + selection
// ---------------------------------------------------------------------

#[test]
fn only_enter_and_space_activate_a_cell() {
    assert!(cell_key_activates("Enter"));
    assert!(cell_key_activates(" "));
    for key in ["Tab", "Escape", "ArrowRight", "a", "Spacebar", ""] {
        assert!(!cell_key_activates(key), "{key} must not activate");
    }
}

// ---------------------------------------------------------------------
// Roving focus -- key mapping
// ---------------------------------------------------------------------

#[test]
fn arrows_and_home_end_map_to_movements() {
    use RosterFocusMove::*;
    assert_eq!(roster_focus_move("ArrowLeft", false), Some(Left));
    assert_eq!(roster_focus_move("ArrowRight", false), Some(Right));
    assert_eq!(roster_focus_move("ArrowUp", false), Some(Up));
    assert_eq!(roster_focus_move("ArrowDown", false), Some(Down));
    assert_eq!(roster_focus_move("Home", false), Some(RowStart));
    assert_eq!(roster_focus_move("End", false), Some(RowEnd));
    assert_eq!(roster_focus_move("Home", true), Some(GridStart));
    assert_eq!(roster_focus_move("End", true), Some(GridEnd));
}

/// Tab must reach the browser untouched or the grid becomes a keyboard trap,
/// and the activation keys must NOT also move focus -- the two vocabularies are
/// disjoint by construction, which is what makes `cell_key_activates` free to
/// stay Enter/Space-only.
#[test]
fn movement_and_activation_keys_are_disjoint_and_tab_is_untouched() {
    for key in ["Tab", "Escape", "a", "PageDown", "Spacebar", ""] {
        assert!(roster_focus_move(key, false).is_none(), "{key} moved focus");
        assert!(roster_focus_move(key, true).is_none(), "ctrl+{key} moved");
    }
    for key in ["Enter", " "] {
        assert!(cell_key_activates(key));
        assert!(roster_focus_move(key, false).is_none());
    }
    for key in [
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
        "Home",
        "End",
    ] {
        assert!(roster_focus_move(key, false).is_some());
        assert!(!cell_key_activates(key), "{key} must not activate");
    }
}

// ---------------------------------------------------------------------
// clamp_focus_cell -- the data-shrinks-under-the-focus rule
//
// The roving-focus counterpart of `normalize_cells`, and the most likely
// defect in the feature: `rows`/`columns` are Signals, so a filter or a fetch
// can shrink the grid under a remembered coordinate. Out of range must clamp,
// never panic and never index.
// ---------------------------------------------------------------------

#[test]
fn clamp_focus_cell_leaves_an_in_range_coordinate_alone() {
    assert_eq!(clamp_focus_cell((0, 0), 3, 3), Some((0, 0)));
    assert_eq!(clamp_focus_cell((1, 2), 3, 3), Some((1, 2)));
    assert_eq!(clamp_focus_cell((2, 2), 3, 3), Some((2, 2)));
}

#[test]
fn clamp_focus_cell_when_rows_shrink() {
    // 20x7 roster filtered down to 3 workers: the column is still valid.
    assert_eq!(clamp_focus_cell((12, 5), 3, 7), Some((2, 5)));
}

#[test]
fn clamp_focus_cell_when_columns_shrink() {
    // A full week narrowed to a working week.
    assert_eq!(clamp_focus_cell((12, 6), 20, 5), Some((12, 4)));
}

#[test]
fn clamp_focus_cell_when_both_shrink() {
    // The case in the bead: (12, 5) against a roster that just became 3x3.
    assert_eq!(clamp_focus_cell((12, 5), 3, 3), Some((2, 2)));
}

#[test]
fn clamp_focus_cell_of_an_empty_grid_is_none_not_a_coordinate() {
    // Zero rows or zero columns is the EmptyState path: there is no cell to
    // focus, so inventing (0, 0) would put `tabindex=0` on nothing and
    // `n - 1` would underflow. `None` is the only honest answer.
    assert_eq!(clamp_focus_cell((0, 0), 0, 5), None);
    assert_eq!(clamp_focus_cell((0, 0), 5, 0), None);
    assert_eq!(clamp_focus_cell((0, 0), 0, 0), None);
    assert_eq!(clamp_focus_cell((12, 5), 0, 0), None);
}

#[test]
fn clamp_is_non_destructive_so_a_transient_shrink_restores_the_users_place() {
    // The component stores the RAW coordinate and clamps on read, so a filter
    // typed and then cleared puts focus back where the user left it rather
    // than stranding them at the grid's edge.
    let remembered = (12, 5);
    assert_eq!(clamp_focus_cell(remembered, 3, 3), Some((2, 2)));
    assert_eq!(clamp_focus_cell(remembered, 0, 0), None);
    assert_eq!(clamp_focus_cell(remembered, 20, 7), Some((12, 5)));
}

#[test]
fn clamp_focus_cell_never_returns_an_out_of_bounds_index() {
    for n_rows in 0..6usize {
        for n_cols in 0..6usize {
            for focus in [(0, 0), (3, 3), (99, 0), (0, 99), (99, 99)] {
                match clamp_focus_cell(focus, n_rows, n_cols) {
                    Some((r, c)) => {
                        assert!(r < n_rows && c < n_cols, "{focus:?} -> ({r}, {c})");
                    }
                    None => assert!(n_rows == 0 || n_cols == 0),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------
// next_focus_cell -- movement stops at the edges, never wraps
// ---------------------------------------------------------------------

#[test]
fn next_focus_cell_moves_one_cell_per_arrow() {
    use RosterFocusMove::*;
    assert_eq!(next_focus_cell((1, 1), 3, 3, Left), Some((1, 0)));
    assert_eq!(next_focus_cell((1, 1), 3, 3, Right), Some((1, 2)));
    assert_eq!(next_focus_cell((1, 1), 3, 3, Up), Some((0, 1)));
    assert_eq!(next_focus_cell((1, 1), 3, 3, Down), Some((2, 1)));
}

/// Wrapping is optional in the ARIA Data Grid pattern and wrong here: a wrap
/// from Friday would land the user on Monday of a *different worker*, which is
/// a plausible-looking answer to a question they did not ask.
#[test]
fn next_focus_cell_stops_at_the_edges_instead_of_wrapping() {
    use RosterFocusMove::*;
    assert_eq!(next_focus_cell((0, 0), 3, 3, Left), Some((0, 0)));
    assert_eq!(next_focus_cell((0, 0), 3, 3, Up), Some((0, 0)));
    assert_eq!(next_focus_cell((2, 2), 3, 3, Right), Some((2, 2)));
    assert_eq!(next_focus_cell((2, 2), 3, 3, Down), Some((2, 2)));
}

#[test]
fn home_and_end_are_the_rows_extremes_and_ctrl_is_the_grids() {
    use RosterFocusMove::*;
    assert_eq!(next_focus_cell((1, 3), 4, 7, RowStart), Some((1, 0)));
    assert_eq!(next_focus_cell((1, 3), 4, 7, RowEnd), Some((1, 6)));
    assert_eq!(next_focus_cell((1, 3), 4, 7, GridStart), Some((0, 0)));
    assert_eq!(next_focus_cell((1, 3), 4, 7, GridEnd), Some((3, 6)));
}

#[test]
fn next_focus_cell_clamps_a_stale_start_before_moving() {
    use RosterFocusMove::*;
    // Focus remembered at (12, 5); the roster shrank to 3x3 before the key
    // press. The move starts from the clamped (2, 2), not from nowhere.
    assert_eq!(next_focus_cell((12, 5), 3, 3, Left), Some((2, 1)));
    assert_eq!(next_focus_cell((12, 5), 3, 3, Up), Some((1, 2)));
    assert_eq!(next_focus_cell((12, 5), 3, 3, RowStart), Some((2, 0)));
}

#[test]
fn next_focus_cell_of_an_empty_grid_is_none() {
    for movement in [
        RosterFocusMove::Left,
        RosterFocusMove::Right,
        RosterFocusMove::Up,
        RosterFocusMove::Down,
        RosterFocusMove::RowStart,
        RosterFocusMove::RowEnd,
        RosterFocusMove::GridStart,
        RosterFocusMove::GridEnd,
    ] {
        assert_eq!(next_focus_cell((0, 0), 0, 0, movement), None);
        assert_eq!(next_focus_cell((3, 3), 0, 5, movement), None);
        assert_eq!(next_focus_cell((3, 3), 5, 0, movement), None);
    }
}

#[test]
fn a_one_by_one_grid_never_moves_off_its_only_cell() {
    for movement in [
        RosterFocusMove::Left,
        RosterFocusMove::Right,
        RosterFocusMove::Up,
        RosterFocusMove::Down,
        RosterFocusMove::RowStart,
        RosterFocusMove::RowEnd,
        RosterFocusMove::GridStart,
        RosterFocusMove::GridEnd,
    ] {
        assert_eq!(next_focus_cell((0, 0), 1, 1, movement), Some((0, 0)));
    }
}

#[test]
fn every_reachable_movement_result_is_in_bounds() {
    let movements = [
        RosterFocusMove::Left,
        RosterFocusMove::Right,
        RosterFocusMove::Up,
        RosterFocusMove::Down,
        RosterFocusMove::RowStart,
        RosterFocusMove::RowEnd,
        RosterFocusMove::GridStart,
        RosterFocusMove::GridEnd,
    ];
    for n_rows in 1..5usize {
        for n_cols in 1..5usize {
            for r in 0..n_rows {
                for c in 0..n_cols {
                    for m in movements {
                        let (nr, nc) = next_focus_cell((r, c), n_rows, n_cols, m).unwrap();
                        assert!(nr < n_rows && nc < n_cols, "{m:?} from ({r}, {c})");
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------
// Cell DOM ids -- how focus actually moves
// ---------------------------------------------------------------------

#[test]
fn cell_dom_ids_are_unique_per_instance_and_coordinate() {
    assert_ne!(
        roster_cell_dom_id(0, 1, 2),
        roster_cell_dom_id(1, 1, 2),
        "two rosters on one page would fight over the same ids"
    );
    assert_ne!(roster_cell_dom_id(0, 1, 2), roster_cell_dom_id(0, 2, 1));

    let mut ids: Vec<String> = (0..3)
        .flat_map(|r| (0..3).map(move |c| roster_cell_dom_id(7, r, c)))
        .collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), total);
}

#[test]
fn cell_dom_ids_are_valid_html_ids() {
    // Must start with a letter and contain nothing that needs escaping in a
    // `getElementById` lookup.
    let id = roster_cell_dom_id(3, 12, 5);
    assert!(id.starts_with("ld-roster-"));
    assert!(id.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-'));
}

// ---------------------------------------------------------------------
// grid_is_interactive -- only a callback earns focus semantics
// ---------------------------------------------------------------------

#[test]
fn an_activation_callback_makes_the_grid_interactive() {
    assert!(grid_is_interactive(true, false));
    assert!(grid_is_interactive(true, true));
}

/// The fix for review round 1. `selected_cell` is a READ-ONLY `Signal`, so with
/// no `on_cell_activate` a click or Enter press does nothing. Treating it as
/// opting in (which DataTable and DayScheduler correctly do, because THEIR
/// selection props are writable `RwSignal`s) would stamp `role="button"`,
/// `tabindex=0` and `aria-pressed` onto every tile of a display-only roster --
/// 140 unresponsive "buttons" on a 20x7 grid, i.e. WCAG 4.1.2. Flipping this
/// back to `has_activate || has_selection` fails here.
#[test]
fn selection_alone_does_not_make_the_grid_interactive() {
    assert!(!grid_is_interactive(false, true));
}

#[test]
fn a_display_only_grid_is_not_interactive() {
    assert!(!grid_is_interactive(false, false));
}

#[test]
fn cell_is_selected_matches_only_the_exact_coordinate() {
    assert!(cell_is_selected(Some((1, 2)), 1, 2));
    assert!(!cell_is_selected(Some((1, 2)), 2, 1));
    assert!(!cell_is_selected(None, 1, 2));
    assert!(!cell_is_selected(Some((0, 0)), 1, 2));
}

/// A selection coordinate outside the grid must be inert, not fatal. Selection
/// is COMPARED, never used as an index, so a stale `(99, 99)` left behind by a
/// larger roster simply matches nothing.
#[test]
fn out_of_range_selection_matches_nothing_and_does_not_panic() {
    let n_rows = 3;
    let n_cols = 5;
    for selected in [Some((99, 99)), Some((0, 99)), Some((99, 0)), Some((3, 5))] {
        let hits = (0..n_rows)
            .flat_map(|r| (0..n_cols).map(move |c| (r, c)))
            .filter(|(r, c)| cell_is_selected(selected, *r, *c))
            .count();
        assert_eq!(hits, 0, "{selected:?} selected a cell it should not have");
    }
}

// ---------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------

#[test]
fn roster_row_new_keeps_worker_and_cells() {
    let row = RosterRow::new("Grace Hopper", cells(&["a", "b"]));
    assert_eq!(row.worker, "Grace Hopper");
    assert_eq!(row.cells.len(), 2);
}

#[test]
fn roster_cell_new_keeps_label_and_state() {
    let cell = RosterCell::new("Nights", ShiftState::Half);
    assert_eq!(cell.label, "Nights");
    assert_eq!(cell.state, ShiftState::Half);
}
