use super::*;

// ── StepStatus::dot_class ──

#[test]
fn dot_class_ready_is_success() {
    assert_eq!(
        StepStatus::Ready.dot_class(),
        "bg-success border-2 border-success"
    );
}

#[test]
fn dot_class_checking_is_accent_and_pulses() {
    let class = StepStatus::Checking.dot_class();
    assert!(class.contains("bg-accent"));
    assert!(class.contains("animate-pulse"));
}

#[test]
fn dot_class_pending_is_hollow() {
    let class = StepStatus::Pending.dot_class();
    assert!(class.contains("bg-base-100"));
    assert!(class.contains("border-base-300"));
}

#[test]
fn dot_class_needs_you_is_warning() {
    assert_eq!(
        StepStatus::NeedsYou.dot_class(),
        "bg-warning border-2 border-warning"
    );
}

#[test]
fn dot_class_down_is_error() {
    assert_eq!(
        StepStatus::Down.dot_class(),
        "bg-error border-2 border-error"
    );
}

#[test]
fn dot_classes_are_pairwise_distinct() {
    let all = [
        StepStatus::Ready,
        StepStatus::Checking,
        StepStatus::Pending,
        StepStatus::NeedsYou,
        StepStatus::Down,
    ];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i].dot_class(), all[j].dot_class());
        }
    }
}

// ── StepStatus::label ──

#[test]
fn label_matches_each_status() {
    assert_eq!(StepStatus::Ready.label(), "Ready");
    assert_eq!(StepStatus::Checking.label(), "Checking");
    assert_eq!(StepStatus::Pending.label(), "Pending");
    assert_eq!(StepStatus::NeedsYou.label(), "Needs you");
    assert_eq!(StepStatus::Down.label(), "Down");
}

// ── StepStatus::default ──

#[test]
fn step_status_default_is_pending() {
    assert_eq!(StepStatus::default(), StepStatus::Pending);
}

// ── segment_lit ──

#[test]
fn segment_lit_true_only_for_ready() {
    assert!(segment_lit(StepStatus::Ready));
    assert!(!segment_lit(StepStatus::Checking));
    assert!(!segment_lit(StepStatus::Pending));
    assert!(!segment_lit(StepStatus::NeedsYou));
    assert!(!segment_lit(StepStatus::Down));
}

// ── vstep_rail_class ──

#[test]
fn vstep_rail_class_lit_is_accent() {
    assert_eq!(vstep_rail_class(true), "bg-accent");
}

#[test]
fn vstep_rail_class_unlit_is_base_300() {
    assert_eq!(vstep_rail_class(false), "bg-base-300");
}

// ── has_rail_segment ──

#[test]
fn has_rail_segment_true_for_all_but_last() {
    assert!(has_rail_segment(0, 3));
    assert!(has_rail_segment(1, 3));
    assert!(!has_rail_segment(2, 3));
}

#[test]
fn has_rail_segment_false_for_single_step() {
    assert!(!has_rail_segment(0, 1));
}

#[test]
fn has_rail_segment_false_for_empty() {
    // index would never be reached for len == 0, but the function should
    // still resolve to false rather than panic/underflow.
    assert!(!has_rail_segment(0, 0));
}

// ── content_class ──

#[test]
fn content_class_with_segment_has_bottom_padding() {
    assert_eq!(content_class(true), "flex-1 pb-6");
}

#[test]
fn content_class_last_step_drops_bottom_padding() {
    assert_eq!(content_class(false), "flex-1");
}

// ── step_key ──

#[test]
fn step_key_equal_steps_have_equal_keys() {
    let a = VerticalStep::new(StepStatus::Ready, "PC", "Online").with_tech("10.0.0.1");
    let b = VerticalStep::new(StepStatus::Ready, "PC", "Online").with_tech("10.0.0.1");
    assert_eq!(step_key(&a), step_key(&b));
}

#[test]
fn step_key_changes_when_status_changes() {
    let a = VerticalStep::new(StepStatus::Checking, "Gateway", "Signing in...");
    let b = VerticalStep::new(StepStatus::Ready, "Gateway", "Signing in...");
    assert_ne!(step_key(&a), step_key(&b));
}

#[test]
fn step_key_changes_when_body_changes_without_status_change() {
    let a = VerticalStep::new(StepStatus::Checking, "Gateway", "Checking...");
    let b = VerticalStep::new(StepStatus::Checking, "Gateway", "Checked 3s ago");
    assert_ne!(step_key(&a), step_key(&b));
}

#[test]
fn step_key_changes_when_action_appears() {
    let a = VerticalStep::new(StepStatus::Down, "Database", "Unreachable");
    let b = a.clone().with_action("Retry");
    assert_ne!(step_key(&a), step_key(&b));
}

// ── VerticalStep::new / builders ──

#[test]
fn new_step_has_no_tech_or_action() {
    let step = VerticalStep::new(StepStatus::Ready, "PC", "Your computer is online");
    assert_eq!(step.status, StepStatus::Ready);
    assert_eq!(step.title, "PC");
    assert_eq!(step.body, "Your computer is online");
    assert_eq!(step.tech, None);
    assert_eq!(step.action_label, None);
}

#[test]
fn with_tech_sets_tech() {
    let step = VerticalStep::new(StepStatus::Down, "Database", "Unreachable")
        .with_tech("db-01.internal:5432");
    assert_eq!(step.tech.as_deref(), Some("db-01.internal:5432"));
    assert_eq!(step.action_label, None);
}

#[test]
fn with_action_sets_action_label() {
    let step =
        VerticalStep::new(StepStatus::NeedsYou, "Gateway", "Needs your sign-in").with_action("Fix");
    assert_eq!(step.action_label.as_deref(), Some("Fix"));
    assert_eq!(step.tech, None);
}

#[test]
fn builders_chain() {
    let step = VerticalStep::new(StepStatus::NeedsYou, "Gateway", "Needs your sign-in")
        .with_tech("vpn-gw-03.internal:443")
        .with_action("Fix");
    assert_eq!(step.tech.as_deref(), Some("vpn-gw-03.internal:443"));
    assert_eq!(step.action_label.as_deref(), Some("Fix"));
}

#[test]
fn vertical_step_clone_equals_original() {
    let step = VerticalStep::new(StepStatus::Ready, "PC", "Online").with_tech("10.0.0.1");
    let cloned = step.clone();
    assert_eq!(step, cloned);
}
