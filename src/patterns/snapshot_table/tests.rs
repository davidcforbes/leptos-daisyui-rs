use super::*;
use crate::patterns::{ActionFeedbackModel, ActionFeedbackState};
use std::rc::Rc;

fn data(
    dataset: &'static str,
    revision: &str,
    rows: &[u32],
) -> SnapshotData<u32, &'static str, ()> {
    SnapshotData::new(
        dataset,
        Rc::new(rows.to_vec()),
        revision,
        rows.len(),
        Some(()),
    )
    .expect("valid complete snapshot")
}

#[test]
fn request_handles_are_framework_issued_strictly_increasing_and_consumed() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let first = state.start_request("mx").expect("first request");
    let second = state.start_request("in").expect("superseding request");

    assert!(second.sequence() > first.sequence());
    assert_eq!(
        state.complete(first.clone(), data("mx", "r1", &[1])),
        SnapshotTransitionDisposition::IgnoredStale
    );
    assert_eq!(
        state.complete(second.clone(), data("in", "r2", &[2, 3])),
        SnapshotTransitionDisposition::Applied
    );
    assert_eq!(
        state.complete(second, data("in", "r3", &[4])),
        SnapshotTransitionDisposition::IgnoredConsumed
    );
    assert_eq!(state.view(None).displayed().unwrap().revision(), "r2");
}

#[test]
fn mismatched_dataset_completion_cannot_change_any_displayed_field() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let initial = state.start_request("mx").unwrap();
    assert_eq!(
        state.complete(initial, data("mx", "r1", &[10, 11])),
        SnapshotTransitionDisposition::Applied
    );
    let before_generation = state.generation();
    let replacement = state.start_request("in").unwrap();
    let consumed_probe = replacement.clone();

    assert_eq!(
        state.complete(replacement, data("mx", "wrong", &[99])),
        SnapshotTransitionDisposition::IgnoredDatasetMismatch
    );
    let view = state.view(None);
    let displayed = view.displayed().unwrap();
    assert_eq!(displayed.dataset(), &"mx");
    assert_eq!(displayed.revision(), "r1");
    assert_eq!(displayed.rows().as_slice(), &[10, 11]);
    assert_eq!(displayed.authoritative_count(), 2);
    assert_eq!(view.generation(), before_generation);
    assert_eq!(view.phase(), SnapshotTablePhase::Displaying);
    assert_eq!(
        state.complete(consumed_probe, data("in", "r2", &[12])),
        SnapshotTransitionDisposition::IgnoredConsumed,
        "a dataset mismatch consumes the matching request instead of trapping the page in Replacing"
    );
}

#[test]
fn stale_local_summary_is_ignored_after_atomic_dataset_replacement() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let first = state.start_request("mx").unwrap();
    state.complete(first, data("mx", "r1", &[1, 2]));
    let stale_empty = state.local_result_summary(0).unwrap();

    let second = state.start_request("in").unwrap();
    state.complete(second, data("in", "r2", &[3]));

    let view = state.view(Some(&stale_empty));
    assert_eq!(view.phase(), SnapshotTablePhase::Displaying);
    assert_eq!(view.local_filtered_count(), None);
    assert_eq!(
        view.render_decision(),
        SnapshotRenderDecision::table_without_panel()
    );
}

#[test]
fn local_row_projection_is_minted_for_the_displayed_snapshot_and_rejects_stale_rows() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let first = state.start_request("mx").unwrap();
    state.complete(first, data("mx", "r1", &[1, 2, 3]));

    let projection = state
        .local_row_projection(Rc::new(vec![2]))
        .expect("a subset can bind to the displayed snapshot");
    assert_eq!(projection.rows().as_slice(), &[2]);
    assert_eq!(
        state
            .validated_local_rows(&projection)
            .expect("the current projection is valid")
            .as_slice(),
        &[2]
    );
    assert_eq!(
        state
            .view(Some(projection.summary()))
            .local_filtered_count(),
        Some(1)
    );

    let replacement = state.start_request("in").unwrap();
    assert_eq!(
        state
            .validated_local_rows(&projection)
            .expect("retained rows remain current while replacement is pending")
            .as_slice(),
        &[2]
    );
    state.complete(replacement, data("in", "r1", &[9, 10]));

    assert!(state.validated_local_rows(&projection).is_none());
    assert_eq!(
        state
            .view(Some(projection.summary()))
            .local_filtered_count(),
        None
    );
}

#[test]
fn local_row_projection_fails_closed_without_allowed_displayed_membership() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    assert!(state.local_row_projection(Rc::new(vec![])).is_none());

    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2]));
    let allowed_projection = state
        .local_row_projection(Rc::new(vec![1]))
        .expect("allowed displayed rows can be projected");
    assert!(
        state.local_row_projection(Rc::new(vec![1, 2, 3])).is_none(),
        "a projection cannot claim more rows than the complete snapshot"
    );

    state.replace_access(SnapshotAccess::Forbidden);
    assert!(state.validated_local_rows(&allowed_projection).is_none());
    assert_eq!(
        state
            .view(Some(allowed_projection.summary()))
            .local_filtered_count(),
        None
    );
    assert!(state.local_row_projection(Rc::new(vec![])).is_none());
}

#[test]
fn matching_local_summary_distinguishes_empty_dataset_from_no_results() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2]));
    let no_results = state.local_result_summary(0).unwrap();
    assert_eq!(
        state.view(Some(&no_results)).render_decision(),
        SnapshotRenderDecision::replacement(PageStatePanelKind::NoLocalResults)
    );

    let request = state.start_request("empty").unwrap();
    state.complete(request, data("empty", "r2", &[]));
    let empty = state.local_result_summary(0).unwrap();
    assert_eq!(
        state.view(Some(&empty)).render_decision(),
        SnapshotRenderDecision::replacement(PageStatePanelKind::EmptyDataset)
    );
}

#[test]
fn replacing_and_retained_failure_keep_the_displayed_snapshot_mounted() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let initial = state.start_request("mx").unwrap();
    state.complete(initial, data("mx", "r1", &[1, 2]));

    let replacement = state.start_request("in").unwrap();
    let replacing = state.view(None);
    assert_eq!(replacing.phase(), SnapshotTablePhase::Replacing);
    assert_eq!(replacing.requested_dataset(), Some(&"in"));
    assert_eq!(
        replacing.render_decision(),
        SnapshotRenderDecision::retained(PageStatePanelKind::Replacing)
    );

    assert_eq!(
        state.fail(replacement, "offline"),
        SnapshotTransitionDisposition::Applied
    );
    let retained = state.view(None);
    assert_eq!(retained.phase(), SnapshotTablePhase::RetainedError);
    assert_eq!(retained.requested_dataset(), Some(&"in"));
    assert_eq!(retained.load_error(), Some(&"offline"));
    assert_eq!(
        retained.render_decision(),
        SnapshotRenderDecision::retained(PageStatePanelKind::RetainedError)
    );
}

#[test]
fn expired_and_forbidden_access_consume_requests_clear_actions_and_never_resurrect_rows() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let initial = state.start_request("mx").unwrap();
    state.complete(initial, data("mx", "r1", &[1]));
    state.start_action("claim").unwrap();
    let pending = state.start_request("in").unwrap();

    state.replace_access(SnapshotAccess::Expired);
    assert_eq!(state.view(None).phase(), SnapshotTablePhase::Expired);
    assert!(state.view(None).displayed().is_none());
    assert!(state.actions().is_empty());
    assert_eq!(
        state.complete(pending, data("in", "r2", &[2])),
        SnapshotTransitionDisposition::IgnoredConsumed
    );

    state.replace_access(SnapshotAccess::Allowed);
    assert_eq!(state.view(None).phase(), SnapshotTablePhase::NeverLoaded);
    assert!(state.view(None).displayed().is_none());
    state.replace_access(SnapshotAccess::Forbidden);
    assert_eq!(
        state.view(None).render_decision(),
        SnapshotRenderDecision::replacement(PageStatePanelKind::Forbidden)
    );
}

#[test]
fn distinct_action_keys_remain_concurrent_and_only_latest_transition_is_announced() {
    let mut model = ActionFeedbackModel::new();
    model.set("a", ActionFeedbackState::Pending).unwrap();
    model.set("b", ActionFeedbackState::Pending).unwrap();
    assert_eq!(model.len(), 2);
    assert_eq!(
        model.entry(&"a").unwrap().state(),
        ActionFeedbackState::Pending
    );
    assert_eq!(
        model.entry(&"b").unwrap().state(),
        ActionFeedbackState::Pending
    );

    model
        .set("a", ActionFeedbackState::RecoverableConflict)
        .unwrap();
    let announcement = model.latest_announcement().unwrap();
    assert_eq!(announcement.key(), &"a");
    assert_eq!(
        announcement.state(),
        ActionFeedbackState::RecoverableConflict
    );
    assert_eq!(
        model.entry(&"b").unwrap().state(),
        ActionFeedbackState::Pending
    );

    assert!(
        !model.dismiss(&"b"),
        "pending work cannot be hidden as idle"
    );
    assert!(model.dismiss(&"a"));
    assert!(model.entry(&"a").is_none());
    assert!(model.entry(&"b").is_some());
    assert!(
        model.latest_announcement().is_none(),
        "dismissing the announced key must not leave a live-region message for an absent entry"
    );
}

#[test]
fn action_handles_are_generation_bound_consumed_and_cleared_by_access_replacement() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));

    let action = state.start_action("claim").expect("displayed row action");
    let duplicate = action.clone();
    assert_eq!(
        state.actions().entry(&"claim").unwrap().state(),
        ActionFeedbackState::Pending
    );
    assert_eq!(
        state
            .finish_action(action, ActionFeedbackState::Success)
            .unwrap(),
        SnapshotActionDisposition::Applied
    );
    assert_eq!(
        state
            .finish_action(duplicate, ActionFeedbackState::Success)
            .unwrap(),
        SnapshotActionDisposition::IgnoredConsumed
    );

    let stale = state.start_action("claim").unwrap();
    state.replace_access(SnapshotAccess::Expired);
    assert!(state.actions().is_empty());
    assert_eq!(
        state
            .finish_action(stale, ActionFeedbackState::Success)
            .unwrap(),
        SnapshotActionDisposition::IgnoredStale
    );
    assert_eq!(
        state.start_action("claim"),
        Err(SnapshotActionStartError::AccessUnavailable(
            SnapshotAccess::Expired
        ))
    );
    state.replace_access(SnapshotAccess::Allowed);
    assert_eq!(
        state.start_action("claim"),
        Err(SnapshotActionStartError::NoDisplayedSnapshot)
    );
}

#[test]
fn atomic_dataset_replacement_invalidates_old_dataset_actions() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let initial = state.start_request("mx").unwrap();
    state.complete(initial, data("mx", "r1", &[1]));
    let old_action = state.start_action("delete-1").unwrap();

    let replacement = state.start_request("in").unwrap();
    state.complete(replacement, data("in", "r2", &[2]));

    assert!(state.actions().is_empty());
    assert_eq!(
        state
            .finish_action(old_action, ActionFeedbackState::Success)
            .unwrap(),
        SnapshotActionDisposition::IgnoredStale
    );
}

#[test]
fn pending_action_updates_require_a_new_framework_handle() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));
    let action = state.start_action("claim").unwrap();
    let valid_completion = action.clone();

    assert_eq!(
        state.finish_action(action, ActionFeedbackState::Pending),
        Err(ActionTransitionError::PendingRequiresStart)
    );
    assert_eq!(
        state
            .finish_action(valid_completion, ActionFeedbackState::Success)
            .unwrap(),
        SnapshotActionDisposition::Applied,
        "rejecting an invalid Pending completion must not consume the valid active handle"
    );
}

#[test]
fn initial_phase_precedence_is_derived_not_caller_constructed() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    assert_eq!(
        state.view(None).render_decision(),
        SnapshotRenderDecision::replacement(PageStatePanelKind::NeverLoaded)
    );
    let request = state.start_request("mx").unwrap();
    assert_eq!(
        state.view(None).render_decision(),
        SnapshotRenderDecision::replacement(PageStatePanelKind::InitialLoading)
    );
    state.fail(request, "network");
    assert_eq!(state.view(None).phase(), SnapshotTablePhase::InitialError);
    assert_eq!(
        state.view(None).render_decision(),
        SnapshotRenderDecision::replacement(PageStatePanelKind::InitialError)
    );
}

#[test]
fn request_and_action_sequences_fail_closed_instead_of_wrapping() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    state.force_next_request_sequence_for_test(u64::MAX);
    assert_eq!(
        state.start_request("mx"),
        Err(SnapshotRequestError::SequenceExhausted)
    );
    assert_eq!(state.view(None).phase(), SnapshotTablePhase::NeverLoaded);

    let mut actions = ActionFeedbackModel::new();
    actions.force_next_sequence_for_test(u64::MAX);
    assert_eq!(
        actions.set("claim", ActionFeedbackState::Pending),
        Err(crate::patterns::ActionTransitionError::SequenceExhausted)
    );
    assert!(actions.is_empty());
    assert!(actions.latest_announcement().is_none());
}
