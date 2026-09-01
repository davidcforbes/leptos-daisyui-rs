use super::*;

const EPS: f64 = 0.001;

fn approx(a: f64, b: f64) {
    assert!((a - b).abs() < EPS, "expected {a} ~= {b}");
}

// ---------------------------------------------------------------------
// HourFormat
// ---------------------------------------------------------------------

#[test]
fn hour_format_default_is_twenty_four() {
    assert_eq!(HourFormat::default(), HourFormat::TwentyFour);
}

#[test]
fn hour_format_twenty_four_labels() {
    assert_eq!(HourFormat::TwentyFour.label(0), "00:00");
    assert_eq!(HourFormat::TwentyFour.label(9), "09:00");
    assert_eq!(HourFormat::TwentyFour.label(13), "13:00");
    assert_eq!(HourFormat::TwentyFour.label(23), "23:00");
}

#[test]
fn hour_format_twelve_labels() {
    assert_eq!(HourFormat::Twelve.label(0), "12 AM");
    assert_eq!(HourFormat::Twelve.label(9), "9 AM");
    assert_eq!(HourFormat::Twelve.label(12), "12 PM");
    assert_eq!(HourFormat::Twelve.label(13), "1 PM");
    assert_eq!(HourFormat::Twelve.label(23), "11 PM");
    assert_eq!(HourFormat::Twelve.label(24), "12 AM");
}

// ---------------------------------------------------------------------
// SchedulerEvent
// ---------------------------------------------------------------------

#[test]
fn scheduler_event_new_keeps_valid_range() {
    let ev = SchedulerEvent::new("Standup", 540, 555, SchedulerEventColor::Primary);
    assert_eq!(ev.title, "Standup");
    assert_eq!(ev.start_min, 540);
    assert_eq!(ev.end_min, 555);
}

#[test]
fn scheduler_event_new_clamps_zero_or_negative_duration() {
    // end == start
    let ev = SchedulerEvent::new("Instant", 600, 600, SchedulerEventColor::Error);
    assert_eq!(ev.end_min, 601);

    // end < start
    let ev2 = SchedulerEvent::new("Backwards", 600, 500, SchedulerEventColor::Error);
    assert_eq!(ev2.end_min, 601);
}

// ---------------------------------------------------------------------
// SchedulerEventColor
// ---------------------------------------------------------------------

#[test]
fn scheduler_event_color_default_is_neutral() {
    assert_eq!(SchedulerEventColor::default(), SchedulerEventColor::Neutral);
}

#[test]
fn scheduler_event_color_bg_classes() {
    assert_eq!(SchedulerEventColor::Neutral.bg_class(), "bg-neutral/15");
    assert_eq!(SchedulerEventColor::Primary.bg_class(), "bg-primary/15");
    assert_eq!(SchedulerEventColor::Secondary.bg_class(), "bg-secondary/15");
    assert_eq!(SchedulerEventColor::Accent.bg_class(), "bg-accent/15");
    assert_eq!(SchedulerEventColor::Info.bg_class(), "bg-info/15");
    assert_eq!(SchedulerEventColor::Success.bg_class(), "bg-success/15");
    assert_eq!(SchedulerEventColor::Warning.bg_class(), "bg-warning/15");
    assert_eq!(SchedulerEventColor::Error.bg_class(), "bg-error/15");
}

#[test]
fn scheduler_event_color_border_classes() {
    assert_eq!(
        SchedulerEventColor::Neutral.border_class(),
        "border-neutral"
    );
    assert_eq!(
        SchedulerEventColor::Primary.border_class(),
        "border-primary"
    );
    assert_eq!(
        SchedulerEventColor::Secondary.border_class(),
        "border-secondary"
    );
    assert_eq!(SchedulerEventColor::Accent.border_class(), "border-accent");
    assert_eq!(SchedulerEventColor::Info.border_class(), "border-info");
    assert_eq!(
        SchedulerEventColor::Success.border_class(),
        "border-success"
    );
    assert_eq!(
        SchedulerEventColor::Warning.border_class(),
        "border-warning"
    );
    assert_eq!(SchedulerEventColor::Error.border_class(), "border-error");
}

#[test]
fn all_scheduler_event_colors_map_to_valid_classes() {
    let variants = [
        SchedulerEventColor::Neutral,
        SchedulerEventColor::Primary,
        SchedulerEventColor::Secondary,
        SchedulerEventColor::Accent,
        SchedulerEventColor::Info,
        SchedulerEventColor::Success,
        SchedulerEventColor::Warning,
        SchedulerEventColor::Error,
    ];
    for v in variants {
        assert!(v.bg_class().starts_with("bg-"));
        assert!(v.bg_class().ends_with("/15"));
        assert!(v.border_class().starts_with("border-"));
    }
}

// ---------------------------------------------------------------------
// minute_to_percent
// ---------------------------------------------------------------------

#[test]
fn minute_to_percent_maps_range_endpoints() {
    approx(minute_to_percent(8.0 * 60.0, 8, 18), 0.0);
    approx(minute_to_percent(18.0 * 60.0, 8, 18), 100.0);
    approx(minute_to_percent(13.0 * 60.0, 8, 18), 50.0);
}

#[test]
fn minute_to_percent_clamps_outside_range() {
    approx(minute_to_percent(0.0, 8, 18), 0.0);
    approx(minute_to_percent(24.0 * 60.0, 8, 18), 100.0);
}

#[test]
fn minute_to_percent_coerces_degenerate_end_hour() {
    // end_hour <= start_hour is coerced to start_hour + 1.
    let a = minute_to_percent(8.0 * 60.0, 8, 8);
    let b = minute_to_percent(9.0 * 60.0, 8, 8);
    approx(a, 0.0);
    approx(b, 100.0);
}

// ---------------------------------------------------------------------
// effective_height_px
// ---------------------------------------------------------------------

#[test]
fn effective_height_px_uses_explicit_override() {
    approx(effective_height_px(480.0, 8, 18), 480.0);
}

#[test]
fn effective_height_px_auto_computes_sixty_per_hour() {
    approx(effective_height_px(0.0, 8, 18), 600.0);
    approx(effective_height_px(0.0, 0, 24), 1440.0);
}

#[test]
fn effective_height_px_coerces_degenerate_range() {
    approx(effective_height_px(0.0, 10, 10), 60.0);
    approx(effective_height_px(0.0, 10, 5), 60.0);
}

// ---------------------------------------------------------------------
// compute_event_layout -- the core overlap/lane algorithm
// ---------------------------------------------------------------------

fn ev(start: u32, end: u32) -> SchedulerEvent {
    SchedulerEvent::new("E", start, end, SchedulerEventColor::Primary)
}

#[test]
fn compute_event_layout_empty_events_is_empty() {
    let layouts = compute_event_layout(&[], 8, 18);
    assert!(layouts.is_empty());
}

#[test]
fn compute_event_layout_single_event_spans_full_width() {
    let events = vec![ev(11 * 60, 12 * 60)];
    let layouts = compute_event_layout(&events, 8, 18);
    approx(layouts[0].left_pct, 0.0);
    approx(layouts[0].width_pct, 100.0);
}

#[test]
fn compute_event_layout_no_overlap_each_spans_full_width() {
    // Sequential, touching but not overlapping (strict '<' in the cluster
    // growth condition treats touching boundaries as non-overlap).
    let events = vec![ev(500, 600), ev(600, 700)];
    let layouts = compute_event_layout(&events, 8, 20);
    for l in &layouts {
        approx(l.left_pct, 0.0);
        approx(l.width_pct, 100.0);
    }
}

#[test]
fn compute_event_layout_no_overlap_with_gap_each_spans_full_width() {
    let events = vec![ev(500, 550), ev(600, 650)];
    let layouts = compute_event_layout(&events, 8, 20);
    for l in &layouts {
        approx(l.left_pct, 0.0);
        approx(l.width_pct, 100.0);
    }
}

#[test]
fn compute_event_layout_two_overlapping_events_split_into_two_lanes() {
    let events = vec![ev(9 * 60, 10 * 60), ev(9 * 60 + 30, 10 * 60 + 30)];
    let layouts = compute_event_layout(&events, 8, 18);
    approx(layouts[0].width_pct, 50.0);
    approx(layouts[1].width_pct, 50.0);
    // First (earlier-starting) event gets the left lane.
    approx(layouts[0].left_pct, 0.0);
    approx(layouts[1].left_pct, 50.0);
}

#[test]
fn compute_event_layout_chain_overlap_not_transitive() {
    // A: 500-600, B: 550-660 (overlaps A), C: 650-700 (overlaps B, NOT A).
    // d2d-ui's greedy algorithm groups all three into one cluster (since
    // cluster growth only checks against the running cluster end) and A/C
    // end up sharing a lane even though they don't actually overlap --
    // this quirky-but-faithful behavior is preserved from the source.
    let events = vec![ev(500, 600), ev(550, 660), ev(650, 700)];
    let layouts = compute_event_layout(&events, 0, 24);

    approx(layouts[0].width_pct, 50.0);
    approx(layouts[1].width_pct, 50.0);
    approx(layouts[2].width_pct, 50.0);

    approx(layouts[0].left_pct, 0.0); // A -> lane 0
    approx(layouts[1].left_pct, 50.0); // B -> lane 1 (A's lane busy)
    approx(layouts[2].left_pct, 0.0); // C -> lane 0 (A's lane freed by then)
}

#[test]
fn compute_event_layout_full_nesting_gets_two_lanes() {
    // B is fully nested inside A's time range.
    let events = vec![ev(500, 700), ev(550, 600)];
    let layouts = compute_event_layout(&events, 0, 24);

    approx(layouts[0].width_pct, 50.0);
    approx(layouts[1].width_pct, 50.0);
    approx(layouts[0].left_pct, 0.0);
    approx(layouts[1].left_pct, 50.0);

    // B's vertical span is nested within A's.
    assert!(layouts[1].top_pct >= layouts[0].top_pct);
    assert!(
        layouts[1].top_pct + layouts[1].height_pct
            <= layouts[0].top_pct + layouts[0].height_pct + EPS
    );
}

#[test]
fn compute_event_layout_identical_times_split_into_two_lanes() {
    let events = vec![ev(500, 600), ev(500, 600)];
    let layouts = compute_event_layout(&events, 0, 24);

    approx(layouts[0].width_pct, 50.0);
    approx(layouts[1].width_pct, 50.0);
    approx(layouts[0].left_pct, 0.0);
    approx(layouts[1].left_pct, 50.0);
    // Same time range -> identical top/height.
    approx(layouts[0].top_pct, layouts[1].top_pct);
    approx(layouts[0].height_pct, layouts[1].height_pct);
}

#[test]
fn compute_event_layout_top_and_height_track_minute_to_percent() {
    let events = vec![ev(11 * 60, 12 * 60)];
    let layouts = compute_event_layout(&events, 8, 18);
    let expected_top = minute_to_percent(11.0 * 60.0, 8, 18);
    let expected_bottom = minute_to_percent(12.0 * 60.0, 8, 18);
    approx(layouts[0].top_pct, expected_top);
    approx(layouts[0].height_pct, expected_bottom - expected_top);
}

#[test]
fn compute_event_layout_three_way_mutual_overlap_needs_three_lanes() {
    // All three mutually overlap at minute 560.
    let events = vec![ev(500, 600), ev(510, 610), ev(520, 620)];
    let layouts = compute_event_layout(&events, 0, 24);
    for l in &layouts {
        approx(l.width_pct, 100.0 / 3.0);
    }
    approx(layouts[0].left_pct, 0.0);
    approx(layouts[1].left_pct, 100.0 / 3.0);
    approx(layouts[2].left_pct, 200.0 / 3.0);
}

// ---------------------------------------------------------------------
// Keyboard contract (event_key_intent) + accessible names
// ---------------------------------------------------------------------

#[test]
fn minute_label_formats_hh_mm() {
    assert_eq!(minute_label(0), "00:00");
    assert_eq!(minute_label(9 * 60 + 5), "09:05");
    assert_eq!(minute_label(23 * 60 + 59), "23:59");
    // 24:00 wraps to a clock hour rather than printing "24".
    assert_eq!(minute_label(24 * 60), "00:00");
}

#[test]
fn event_aria_label_is_title_plus_time_range() {
    let e = SchedulerEvent::new("Standup", 9 * 60, 9 * 60 + 15, SchedulerEventColor::Primary);
    assert_eq!(event_aria_label(&e), "Standup, 09:00 to 09:15");
}

// ---------------------------------------------------------------------
// resolve_event_accessible_label (ldui-kx7y): the localization seam for
// interactive event accessible names.
// ---------------------------------------------------------------------

#[test]
fn resolve_accessible_label_defaults_to_english_when_no_formatter_supplied() {
    // No `event_accessible_label` callback at all (`formatted: None`) --
    // existing call sites must see the exact current English default.
    let e = SchedulerEvent::new("Standup", 9 * 60, 9 * 60 + 15, SchedulerEventColor::Primary);
    assert_eq!(
        resolve_event_accessible_label(&e, None),
        "Standup, 09:00 to 09:15"
    );
}

#[test]
fn resolve_accessible_label_uses_the_localized_override_verbatim() {
    // A caller's formatter can reorder and inflect -- Spanish puts the
    // interval after "de ... a ...", not English's "start to end".
    let e = SchedulerEvent::new(
        "Intake review",
        9 * 60,
        10 * 60,
        SchedulerEventColor::Primary,
    );
    let spanish = "Intake review, de 09:00 a 10:00".to_string();
    assert_eq!(
        resolve_event_accessible_label(&e, Some(spanish.clone())),
        spanish
    );
    assert!(
        !spanish.contains(" to "),
        "the localized proof string must contain no framework-generated English 'to' token"
    );
}

#[test]
fn resolve_accessible_label_falls_back_on_empty_formatter_output() {
    // A formatter that ran but returned "" (e.g. a translation-catalogue
    // gap) must not leave the block unnamed -- fall back to English rather
    // than emitting an empty aria-label.
    let e = SchedulerEvent::new("Standup", 9 * 60, 9 * 60 + 15, SchedulerEventColor::Primary);
    assert_eq!(
        resolve_event_accessible_label(&e, Some(String::new())),
        "Standup, 09:00 to 09:15"
    );
}

#[test]
fn resolve_accessible_label_falls_back_on_whitespace_only_formatter_output() {
    // Whitespace-only is treated the same as empty -- " \t\n" is not a name.
    let e = SchedulerEvent::new("Standup", 9 * 60, 9 * 60 + 15, SchedulerEventColor::Primary);
    assert_eq!(
        resolve_event_accessible_label(&e, Some("   \t\n  ".to_string())),
        "Standup, 09:00 to 09:15"
    );
}

#[test]
fn enter_and_space_activate() {
    assert_eq!(
        event_key_intent("Enter", false, 15),
        Some(EventKeyIntent::Activate)
    );
    assert_eq!(
        event_key_intent(" ", false, 15),
        Some(EventKeyIntent::Activate)
    );
}

#[test]
fn arrows_move_by_the_step() {
    assert_eq!(
        event_key_intent("ArrowUp", false, 15),
        Some(EventKeyIntent::Move(-15))
    );
    assert_eq!(
        event_key_intent("ArrowDown", false, 30),
        Some(EventKeyIntent::Move(30))
    );
}

#[test]
fn shift_arrows_resize_instead_of_moving() {
    assert_eq!(
        event_key_intent("ArrowUp", true, 15),
        Some(EventKeyIntent::Resize(-15))
    );
    assert_eq!(
        event_key_intent("ArrowDown", true, 15),
        Some(EventKeyIntent::Resize(15))
    );
}

#[test]
fn tab_and_other_keys_are_left_for_focus_navigation() {
    assert_eq!(event_key_intent("Tab", false, 15), None);
    assert_eq!(event_key_intent("Escape", false, 15), None);
    assert_eq!(event_key_intent("a", false, 15), None);
}

#[test]
fn zero_step_still_produces_a_nonzero_move() {
    // A misconfigured step must not turn arrows into no-ops.
    assert_eq!(
        event_key_intent("ArrowDown", false, 0),
        Some(EventKeyIntent::Move(1))
    );
}
