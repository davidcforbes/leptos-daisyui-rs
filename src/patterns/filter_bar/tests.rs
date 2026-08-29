use super::*;

#[test]
fn save_presentation_enables_only_explicit_dirty_or_retryable_states() {
    let texts = FilterBarTexts::default();

    let clean = filter_save_presentation(&SnapshotDefaultSaveState::Clean, &texts);
    assert!(!clean.enabled);
    assert_eq!(
        clean.disabled_reason.as_deref(),
        Some("Defaults are already saved")
    );
    assert!(clean.feedback.is_none());

    let dirty = filter_save_presentation(&SnapshotDefaultSaveState::Dirty, &texts);
    assert!(dirty.enabled);
    assert!(dirty.disabled_reason.is_none());

    let pending = filter_save_presentation(&SnapshotDefaultSaveState::Pending, &texts);
    assert!(!pending.enabled);
    assert_eq!(pending.feedback.as_deref(), Some("Saving default view"));

    let saved = filter_save_presentation(&SnapshotDefaultSaveState::Saved, &texts);
    assert!(!saved.enabled);
    assert_eq!(saved.feedback.as_deref(), Some("Default view saved"));

    let conflict = filter_save_presentation(
        &SnapshotDefaultSaveState::Conflict("revision changed".to_owned()),
        &texts,
    );
    assert!(conflict.enabled, "a conflict may be retried explicitly");
    assert_eq!(
        conflict.feedback.as_deref(),
        Some("Default view conflict: revision changed")
    );

    let failed = filter_save_presentation(
        &SnapshotDefaultSaveState::Failure("network unavailable".to_owned()),
        &texts,
    );
    assert!(failed.enabled, "a failure may be retried explicitly");
    assert_eq!(
        failed.feedback.as_deref(),
        Some("Could not save default view: network unavailable")
    );
}

#[test]
fn localized_templates_cover_filter_and_result_counts() {
    let mut texts = FilterBarTexts::default();
    texts.active_many = "{count} filtros activos".to_owned();
    texts.result_count = "{visible} de {total} resultados".to_owned();

    assert_eq!(filter_active_summary(3, &texts), "3 filtros activos");
    assert_eq!(
        filter_result_summary(FilterResultSummary::new(7, 40), &texts),
        "7 de 40 resultados"
    );
}
