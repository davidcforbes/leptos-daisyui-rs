use super::*;

// phase_fill_percent

#[test]
fn completed_phases_are_solid() {
    assert_eq!(phase_fill_percent(0, 1, 40), 100.0);
    assert_eq!(phase_fill_percent(1, 2, 0), 100.0);
}

#[test]
fn active_phase_fills_to_pct() {
    assert_eq!(phase_fill_percent(1, 1, 40), 40.0);
    assert_eq!(phase_fill_percent(0, 0, 0), 0.0);
    assert_eq!(phase_fill_percent(2, 2, 100), 100.0);
}

#[test]
fn active_phase_pct_clamps_to_100() {
    assert_eq!(phase_fill_percent(1, 1, 250), 100.0);
}

#[test]
fn future_phases_are_empty() {
    assert_eq!(phase_fill_percent(2, 1, 40), 0.0);
    assert_eq!(phase_fill_percent(1, 0, 99), 0.0);
}

#[test]
fn current_past_the_end_means_every_phase_is_solid() {
    // A finished run (current == phase count) must not leave the last
    // segment partial.
    assert_eq!(phase_fill_percent(0, 3, 0), 100.0);
    assert_eq!(phase_fill_percent(2, 3, 0), 100.0);
}

// phase_overall_percent

#[test]
fn overall_percent_weights_phases_equally() {
    // capture -> reconcile -> apply, reconcile at 50%: 1.5/3 = 50%.
    assert!((phase_overall_percent(3, 1, 50) - 50.0).abs() < 0.0001);
}

#[test]
fn overall_percent_is_zero_at_the_start() {
    assert_eq!(phase_overall_percent(3, 0, 0), 0.0);
}

#[test]
fn overall_percent_is_100_when_finished() {
    assert_eq!(phase_overall_percent(3, 3, 0), 100.0);
    assert_eq!(phase_overall_percent(3, 99, 0), 100.0);
}

#[test]
fn overall_percent_never_divides_by_zero() {
    assert_eq!(phase_overall_percent(0, 0, 50), 0.0);
}

#[test]
fn overall_percent_clamps_a_wild_pct() {
    assert!((phase_overall_percent(2, 1, 250) - 100.0).abs() < 0.0001);
}

// phase_progress_value_text

fn phases() -> Vec<String> {
    vec![
        "capture".to_string(),
        "reconcile".to_string(),
        "apply".to_string(),
    ]
}

#[test]
fn value_text_names_the_active_phase_pct_and_ordinal() {
    assert_eq!(
        phase_progress_value_text(&phases(), 1, 40, false),
        "reconcile 40% (phase 2 of 3)"
    );
}

#[test]
fn value_text_reports_failure_of_the_active_phase() {
    assert_eq!(
        phase_progress_value_text(&phases(), 2, 15, true),
        "apply failed at 15% (phase 3 of 3)"
    );
}

#[test]
fn value_text_handles_empty_and_finished_runs() {
    assert_eq!(phase_progress_value_text(&[], 0, 0, false), "no phases");
    assert_eq!(
        phase_progress_value_text(&phases(), 3, 0, false),
        "all 3 phases complete"
    );
}

#[test]
fn value_text_clamps_a_wild_pct() {
    assert_eq!(
        phase_progress_value_text(&phases(), 0, 250, false),
        "capture 100% (phase 1 of 3)"
    );
}

// phase_state

#[test]
fn phase_state_covers_all_four_states() {
    assert_eq!(phase_state(0, 1, false), "complete");
    assert_eq!(phase_state(1, 1, false), "active");
    assert_eq!(phase_state(1, 1, true), "failed");
    assert_eq!(phase_state(2, 1, true), "pending");
}

#[test]
fn failed_only_marks_the_active_phase() {
    // Completed and pending segments keep their factual state; "failed"
    // describes where the run stopped, not the whole history.
    assert_eq!(phase_state(0, 1, true), "complete");
    assert_eq!(phase_state(2, 1, true), "pending");
}
