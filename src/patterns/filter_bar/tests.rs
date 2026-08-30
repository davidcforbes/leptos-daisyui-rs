use super::*;
use crate::components::EntityTablePreferences;
use crate::patterns::FilterSchema;
use leptos::prelude::{Callback, RwSignal, ToChildren};
use leptos::reactive::owner::Owner;
use serde_json::json;

/// Builds `FilterBar` directly (bypassing the `view!` call-site macro, same
/// as the existing modal/entity_table smoke tests) so a missing/duplicate
/// prop is a compile error rather than a silent regression, inside an
/// `Owner` scope so the reactive props (`Signal`/`Callback`) are safe to
/// construct. These are construction-level smoke tests: they prove the
/// optional `search` slot type-checks in every combination the acceptance
/// criteria names and that removing it does not disturb any other prop's
/// shape. The structural claim itself — that an absent `search` renders no
/// `[data-filter-search]` wrapper while chips/count/actions/feedback claim
/// the full row width — is asserted by the three browser fixtures this bead
/// adds to `tests/reactivity_smoke.rs` (`actions-only`, `columns-only`,
/// `search`), since this crate has no native DOM/SSR renderer.
fn stored_texts() -> Signal<FilterBarTexts> {
    Signal::stored(FilterBarTexts::default())
}

#[test]
fn filter_bar_builds_with_the_historical_search_first_shape() {
    let owner = Owner::new();
    owner.with(|| {
        let _ = FilterBar(FilterBarProps {
            search: Some(ToChildren::to_children(|| view! { "search control" })),
            actions: None,
            active_filters: None,
            on_remove: None,
            on_reset: None,
            result: None,
            default_save: None,
            texts: stored_texts(),
            class: "",
            children: None,
        });
    });
}

#[test]
fn filter_bar_builds_actions_only_without_search_or_column_filters() {
    let owner = Owner::new();
    owner.with(|| {
        let _ = FilterBar(FilterBarProps {
            search: None,
            actions: Some(ToChildren::to_children(|| view! { "extra action" })),
            active_filters: None,
            on_remove: None,
            on_reset: Some(Callback::new(|()| {})),
            result: None,
            default_save: None,
            texts: stored_texts(),
            class: "",
            children: None,
        });
    });
}

#[test]
fn filter_bar_builds_column_filters_only_summary_without_search() {
    let owner = Owner::new();
    owner.with(|| {
        let chips = RwSignal::new(vec![ActiveFilterChip::new("status", "Status", "Urgent")]);
        let result = RwSignal::new(FilterResultSummary::new(7, 40));
        let _ = FilterBar(FilterBarProps {
            search: None,
            actions: None,
            active_filters: Some(chips.into()),
            on_remove: Some(Callback::new(|_: String| {})),
            on_reset: None,
            result: Some(result.into()),
            default_save: None,
            texts: stored_texts(),
            class: "",
            children: Some(ToChildren::to_children(|| view! { "status select" })),
        });
    });
}

#[test]
fn filter_bar_builds_the_ordinary_search_configuration_with_every_framework_action() {
    let owner = Owner::new();
    owner.with(|| {
        let chips = RwSignal::new(Vec::<ActiveFilterChip>::new());
        let result = RwSignal::new(FilterResultSummary::new(72, 72));
        let schema = FilterSchema::<()>::new("office", &["status"]);
        let defaults = Signal::stored(
            schema
                .project_defaults([("status", json!("ready"))], EntityTablePreferences::new(1))
                .expect("schema-declared filter projects"),
        );
        let save_state = RwSignal::new(SnapshotDefaultSaveState::Dirty);
        let default_save = SnapshotDefaultSave::new(
            defaults,
            save_state,
            Callback::new(|_: SnapshotViewDefaults| {}),
        );
        let _ = FilterBar(FilterBarProps {
            search: Some(ToChildren::to_children(|| view! { "search control" })),
            actions: None,
            active_filters: Some(chips.into()),
            on_remove: Some(Callback::new(|_: String| {})),
            on_reset: Some(Callback::new(|()| {})),
            result: Some(result.into()),
            default_save: Some(default_save),
            texts: stored_texts(),
            class: "",
            children: None,
        });
    });
}

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
    let texts = FilterBarTexts {
        active_many: "{count} filtros activos".to_owned(),
        result_count: "{visible} de {total} resultados".to_owned(),
        ..FilterBarTexts::default()
    };

    assert_eq!(filter_active_summary(3, &texts), "3 filtros activos");
    assert_eq!(
        filter_result_summary(FilterResultSummary::new(7, 40), &texts),
        "7 de 40 resultados"
    );
}
