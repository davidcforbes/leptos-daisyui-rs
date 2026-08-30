use super::*;
use crate::patterns::{ActionFeedbackContent, ActionFeedbackModel, ActionFeedbackState};
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

fn content(detail: &str) -> ActionFeedbackContent {
    ActionFeedbackContent {
        primary: None,
        detail: Some(detail.to_owned()),
    }
}

/// A superseded handle's completion is ignored before its content ever
/// reaches the model, so a stale attempt's text can never overwrite a newer
/// attempt's still-pending content (ldui-baz4).
#[test]
fn stale_completion_content_cannot_replace_a_newer_attempts_message() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));

    let stale = state
        .start_action_with_content("claim", content("First attempt detail."))
        .unwrap();
    let fresh = state
        .start_action_with_content("claim", content("Second attempt in progress."))
        .unwrap();
    assert_eq!(
        state.actions().entry(&"claim").unwrap().content().detail,
        Some("Second attempt in progress.".to_owned()),
        "the newer Pending attempt's own content must be visible"
    );

    assert_eq!(
        state
            .finish_action_with_content(
                stale,
                ActionFeedbackState::Success,
                content("STALE COMPLETION — must never display."),
            )
            .unwrap(),
        SnapshotActionDisposition::IgnoredStale
    );
    assert_eq!(
        state.actions().entry(&"claim").unwrap().state(),
        ActionFeedbackState::Pending,
        "the stale completion must not have touched the still-pending entry"
    );
    assert_eq!(
        state.actions().entry(&"claim").unwrap().content().detail,
        Some("Second attempt in progress.".to_owned()),
        "the stale attempt's content must never have been attached"
    );

    assert_eq!(
        state
            .finish_action_with_content(
                fresh,
                ActionFeedbackState::RetryableFailure,
                content("Timed out contacting the service; retry."),
            )
            .unwrap(),
        SnapshotActionDisposition::Applied
    );
    assert_eq!(
        state.actions().entry(&"claim").unwrap().content().detail,
        Some("Timed out contacting the service; retry.".to_owned())
    );
}

/// A second completion of an already-consumed handle is rejected, so its
/// content cannot overwrite the content the first completion recorded.
#[test]
fn duplicate_completion_content_is_rejected_after_the_handle_is_consumed() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));

    let action = state.start_action("claim").unwrap();
    let duplicate = action.clone();
    assert_eq!(
        state
            .finish_action_with_content(
                action,
                ActionFeedbackState::PartialSuccess,
                content("3 of 5 items updated."),
            )
            .unwrap(),
        SnapshotActionDisposition::Applied
    );
    assert_eq!(
        state
            .finish_action_with_content(
                duplicate,
                ActionFeedbackState::Success,
                content("DUPLICATE — must never display."),
            )
            .unwrap(),
        SnapshotActionDisposition::IgnoredConsumed
    );
    assert_eq!(
        state.actions().entry(&"claim").unwrap().content().detail,
        Some("3 of 5 items updated.".to_owned())
    );
}

/// Atomic dataset replacement invalidates the prior action's handle before
/// content can attach, and the cleared model carries none of it forward.
#[test]
fn dataset_replacement_invalidates_old_actions_content() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let initial = state.start_request("mx").unwrap();
    state.complete(initial, data("mx", "r1", &[1]));
    let old_action = state
        .start_action_with_content("delete-1", content("Deleting row 1…"))
        .unwrap();

    let replacement = state.start_request("in").unwrap();
    state.complete(replacement, data("in", "r2", &[2]));
    assert!(state.actions().is_empty());

    assert_eq!(
        state
            .finish_action_with_content(
                old_action,
                ActionFeedbackState::Success,
                content("STALE — must never display."),
            )
            .unwrap(),
        SnapshotActionDisposition::IgnoredStale
    );
    assert!(
        state.actions().is_empty(),
        "the invalidated action must not resurrect with stale content"
    );
}

/// Returning to Allowed after an access replacement starts from a clean
/// action model, so a generation reset cannot leak an old attempt's content.
#[test]
fn access_replacement_clears_pending_action_content() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));
    state
        .start_action_with_content("claim", content("In-flight detail before reset."))
        .unwrap();
    assert!(!state.actions().is_empty());

    state.replace_access(SnapshotAccess::Forbidden);
    assert!(state.actions().is_empty());

    state.replace_access(SnapshotAccess::Allowed);
    assert!(state.actions().is_empty());
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));
    state.start_action("claim").unwrap();
    assert!(
        state
            .actions()
            .entry(&"claim")
            .unwrap()
            .content()
            .is_empty(),
        "a fresh action after generation reset must not inherit the earlier attempt's content"
    );
}

/// A retry that supplies no content must not inherit the prior attempt's
/// detail — content always replaces, it never merges across attempts.
#[test]
fn retry_does_not_inherit_a_prior_attempts_content() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));

    let action = state
        .start_action_with_content("claim", content("First attempt detail."))
        .unwrap();
    state
        .finish_action_with_content(
            action,
            ActionFeedbackState::RetryableFailure,
            content("Timed out; retry."),
        )
        .unwrap();

    // Consumer-driven retry mints a fresh handle with no content supplied.
    state.start_action("claim").unwrap();
    assert!(
        state
            .actions()
            .entry(&"claim")
            .unwrap()
            .content()
            .is_empty(),
        "retry must not carry the failed attempt's detail forward"
    );
}

/// Distinct concurrent action keys keep independent content, and the single
/// live-region announcement always reflects only the latest transition's
/// content — never a mix of two attempts.
#[test]
fn concurrent_actions_retain_independent_content_and_one_coherent_announcement() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));

    let claim_a = state
        .start_action_with_content("row-a", content("Saving row A…"))
        .unwrap();
    let claim_b = state
        .start_action_with_content("row-b", content("Saving row B…"))
        .unwrap();

    state
        .finish_action_with_content(
            claim_a,
            ActionFeedbackState::RecoverableConflict,
            content("Row A changed by another editor."),
        )
        .unwrap();
    assert_eq!(
        state.actions().entry(&"row-b").unwrap().content().detail,
        Some("Saving row B…".to_owned()),
        "row-b's content must be unaffected by row-a's completion"
    );
    let announcement = state.actions().latest_announcement().unwrap();
    assert_eq!(announcement.key(), &"row-a");
    assert_eq!(
        announcement.content().detail,
        Some("Row A changed by another editor.".to_owned())
    );

    state
        .finish_action_with_content(
            claim_b,
            ActionFeedbackState::PartialSuccess,
            content("2 of 4 items on row B."),
        )
        .unwrap();
    assert_eq!(
        state.actions().entry(&"row-a").unwrap().content().detail,
        Some("Row A changed by another editor.".to_owned()),
        "row-a's content must be unaffected by row-b's later completion"
    );
    let announcement = state.actions().latest_announcement().unwrap();
    assert_eq!(announcement.key(), &"row-b");
    assert_eq!(
        announcement.content().detail,
        Some("2 of 4 items on row B.".to_owned())
    );
}

// ldui-vn81 / ldui-cb29: generation-bound displayed-snapshot deltas.

/// A successful delta (e.g. removing a row the caller just claimed)
/// atomically replaces rows/revision/count/metadata without bumping the
/// dataset/access generation, so `EntityTable`'s `focus_scope`/
/// `dataset_identity` bindings (both driven off `generation()`) stay stable
/// across the mutation.
#[test]
fn own_claim_removal_delta_replaces_rows_without_bumping_generation() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2, 3]));
    let before_generation = state.generation();

    let delta = state.start_delta().expect("displayed snapshot exists");
    assert_eq!(delta.generation(), before_generation);
    assert_eq!(
        state.apply_delta(delta, data("mx", "r1-claimed", &[2, 3])),
        SnapshotDeltaDisposition::Applied
    );

    let view = state.view(None);
    assert_eq!(view.generation(), before_generation);
    assert_eq!(view.phase(), SnapshotTablePhase::Displaying);
    let displayed = view.displayed().unwrap();
    assert_eq!(displayed.dataset(), &"mx");
    assert_eq!(displayed.revision(), "r1-claimed");
    assert_eq!(displayed.rows().as_slice(), &[2, 3]);
    assert_eq!(displayed.authoritative_count(), 2);
}

/// Sequential deltas from independent sources -- the caller's own claim, then
/// a different user's SSE removal -- each mint fresh against the
/// just-updated displayed snapshot and both apply in order.
#[test]
fn sequential_own_claim_and_another_user_sse_removal_deltas_both_apply() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2, 3]));
    let generation = state.generation();

    let own_claim = state.start_delta().unwrap();
    assert_eq!(
        state.apply_delta(own_claim, data("mx", "r2", &[2, 3])),
        SnapshotDeltaDisposition::Applied
    );

    let sse_removal = state.start_delta().unwrap();
    assert_eq!(
        state.apply_delta(sse_removal, data("mx", "r3", &[3])),
        SnapshotDeltaDisposition::Applied
    );

    let view = state.view(None);
    assert_eq!(view.generation(), generation, "no delta bumps generation");
    let displayed = view.displayed().unwrap();
    assert_eq!(displayed.revision(), "r3");
    assert_eq!(displayed.rows().as_slice(), &[3]);
}

/// Re-applying an already-applied handle is rejected as stale rather than
/// silently reverting a later delta's rows.
#[test]
fn duplicate_delta_reapplication_is_rejected_as_stale() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2, 3]));

    let delta = state.start_delta().unwrap();
    let duplicate = delta.clone();
    assert_eq!(
        state.apply_delta(delta, data("mx", "r2", &[2, 3])),
        SnapshotDeltaDisposition::Applied
    );
    assert_eq!(
        state.apply_delta(duplicate, data("mx", "r2-again", &[9])),
        SnapshotDeltaDisposition::IgnoredStale
    );
    let displayed = state.view(None).displayed().unwrap();
    assert_eq!(
        displayed.revision(),
        "r2",
        "the duplicate must not overwrite the applied delta"
    );
    assert_eq!(displayed.rows().as_slice(), &[2, 3]);
}

/// A delta minted before an already-applied later delta cannot apply out of
/// order and silently regress rows a newer delta already removed.
#[test]
fn out_of_order_stale_delta_cannot_regress_already_applied_rows() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2, 3]));

    let older = state.start_delta().unwrap();
    let newer = state.start_delta().unwrap();
    assert!(newer.sequence() > older.sequence());

    assert_eq!(
        state.apply_delta(newer, data("mx", "r2", &[3])),
        SnapshotDeltaDisposition::Applied
    );
    assert_eq!(
        state.apply_delta(older, data("mx", "stale", &[1, 2, 3])),
        SnapshotDeltaDisposition::IgnoredStale
    );
    let displayed = state.view(None).displayed().unwrap();
    assert_eq!(displayed.revision(), "r2");
    assert_eq!(displayed.rows().as_slice(), &[3]);
}

/// A delta may be minted and applied while an unrelated office replacement
/// remains in flight, leaving the pending request untouched; completing that
/// replacement afterward still succeeds and supersedes the delta's rows.
#[test]
fn delta_applies_during_an_unrelated_office_replacement_and_leaves_it_intact() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2, 3]));
    let generation = state.generation();

    let replacement = state.start_request("in").unwrap();
    assert_eq!(state.view(None).phase(), SnapshotTablePhase::Replacing);
    assert_eq!(state.view(None).requested_dataset(), Some(&"in"));

    let delta = state
        .start_delta()
        .expect("delta targets the still-displayed mx snapshot");
    assert_eq!(
        state.apply_delta(delta, data("mx", "r2", &[2, 3])),
        SnapshotDeltaDisposition::Applied
    );

    let view = state.view(None);
    assert_eq!(
        view.generation(),
        generation,
        "delta preserves the generation"
    );
    assert_eq!(
        view.phase(),
        SnapshotTablePhase::Replacing,
        "the delta must not disturb the in-flight replacement's phase"
    );
    assert_eq!(view.requested_dataset(), Some(&"in"));
    let displayed = view.displayed().unwrap();
    assert_eq!(displayed.dataset(), &"mx");
    assert_eq!(displayed.rows().as_slice(), &[2, 3]);

    // The still-pending replacement completes normally afterward.
    assert_eq!(
        state.complete(replacement, data("in", "in-r1", &[10, 11])),
        SnapshotTransitionDisposition::Applied
    );
    let after = state.view(None);
    assert_eq!(after.phase(), SnapshotTablePhase::Displaying);
    assert_ne!(after.generation(), generation);
    let displayed = after.displayed().unwrap();
    assert_eq!(displayed.dataset(), &"in");
    assert_eq!(displayed.rows().as_slice(), &[10, 11]);
}

/// A delta minted before a full dataset replacement completes is invalidated
/// by the resulting generation bump, exactly like a stale action handle.
#[test]
fn delta_minted_before_a_completed_replacement_becomes_stale() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2, 3]));

    let delta = state.start_delta().unwrap();
    let replacement = state.start_request("in").unwrap();
    assert_eq!(
        state.complete(replacement, data("in", "r2", &[9])),
        SnapshotTransitionDisposition::Applied
    );

    assert_eq!(
        state.apply_delta(delta, data("mx", "wrong", &[2, 3])),
        SnapshotDeltaDisposition::IgnoredStale
    );
    let displayed = state.view(None).displayed().unwrap();
    assert_eq!(displayed.dataset(), &"in");
    assert_eq!(displayed.rows().as_slice(), &[9]);
}

/// An access replacement (session expiry) also invalidates any outstanding
/// delta handle through the same generation bump.
#[test]
fn delta_minted_before_access_replacement_becomes_stale() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));

    let delta = state.start_delta().unwrap();
    state.replace_access(SnapshotAccess::Expired);
    assert_eq!(
        state.apply_delta(delta, data("mx", "r2", &[])),
        SnapshotDeltaDisposition::IgnoredStale
    );
    assert!(state.view(None).displayed().is_none());
}

/// A delta whose supplied data names a different dataset than the handle was
/// minted against fails closed without touching any field.
#[test]
fn delta_with_mismatched_dataset_is_rejected_without_mutation() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2]));

    let delta = state.start_delta().unwrap();
    assert_eq!(
        state.apply_delta(delta, data("in", "wrong-dataset", &[9])),
        SnapshotDeltaDisposition::IgnoredDatasetMismatch
    );
    let displayed = state.view(None).displayed().unwrap();
    assert_eq!(displayed.dataset(), &"mx");
    assert_eq!(displayed.revision(), "r1");
    assert_eq!(displayed.rows().as_slice(), &[1, 2]);
}

/// Minting fails closed while access is replaced or before any dataset has
/// ever been displayed.
#[test]
fn delta_start_fails_closed_without_allowed_displayed_membership() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    assert_eq!(
        state.start_delta(),
        Err(SnapshotDeltaStartError::NoDisplayedSnapshot)
    );

    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));
    state.replace_access(SnapshotAccess::Forbidden);
    assert_eq!(
        state.start_delta(),
        Err(SnapshotDeltaStartError::AccessUnavailable(
            SnapshotAccess::Forbidden
        ))
    );
}

/// The delta sequence counter fails closed instead of wrapping, mirroring
/// the request/action sequence guarantee.
#[test]
fn delta_sequence_fails_closed_instead_of_wrapping() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1]));

    state.force_next_delta_sequence_for_test(u64::MAX);
    assert_eq!(
        state.start_delta(),
        Err(SnapshotDeltaStartError::SequenceExhausted)
    );
    assert_eq!(state.view(None).displayed().unwrap().revision(), "r1");
}

/// A delta preserves keyed action feedback in the same generation: an action
/// started before the delta can still be completed successfully afterward,
/// and the delta itself never clears the action model (unlike a full
/// [`SnapshotTableState::complete`] replacement).
#[test]
fn keyed_action_completion_remains_valid_after_a_delta_in_the_same_generation() {
    let mut state = SnapshotTableState::<u32, &'static str, &'static str, (), &'static str>::new();
    let request = state.start_request("mx").unwrap();
    state.complete(request, data("mx", "r1", &[1, 2, 3]));

    let action = state.start_action("claim-row-1").unwrap();
    assert_eq!(action.generation(), state.generation());

    let delta = state.start_delta().unwrap();
    assert_eq!(
        state.apply_delta(delta, data("mx", "r2", &[2, 3])),
        SnapshotDeltaDisposition::Applied
    );

    assert_eq!(
        state
            .actions()
            .entry(&"claim-row-1")
            .expect("the delta must not clear unrelated action feedback")
            .state(),
        ActionFeedbackState::Pending
    );
    assert_eq!(
        state
            .finish_action(action, ActionFeedbackState::Success)
            .unwrap(),
        SnapshotActionDisposition::Applied,
        "the action handle's generation is unaffected by the delta"
    );
    assert_eq!(
        state.actions().entry(&"claim-row-1").unwrap().state(),
        ActionFeedbackState::Success
    );
}
