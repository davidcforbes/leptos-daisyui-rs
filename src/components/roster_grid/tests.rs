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
    for state in ShiftState::ALL {
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
    for state in ShiftState::ALL {
        let cell = RosterCell::new("x", state);
        assert!(cell_aria_label("W", "C", &cell, state.as_label()).ends_with(state.as_label()));
    }
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
