//! Horizontal, wrapping local-filter composition and explicit default saving.

use super::{ActiveFilterChip, ActiveFilterChips, ActiveFilterTexts, SnapshotViewDefaults};
use crate::components::Button;
use leptos::prelude::*;

/// Canonical horizontal-filter classes.
pub const FILTER_BAR_BASE_CLASS: &str = "flex w-full min-w-0 flex-wrap items-end gap-3 rounded-box border border-base-300 bg-base-100 p-3";

/// Merges caller classes with the canonical filter-bar contract.
pub fn filter_bar_class(class: &str) -> String {
    [FILTER_BAR_BASE_CLASS, class]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Local result counts rendered by the controlled utility row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterResultSummary {
    /// Rows remaining after local filtering.
    pub visible: usize,
    /// Rows in the complete displayed snapshot.
    pub total: usize,
}

impl FilterResultSummary {
    /// Creates a local result-count summary.
    pub const fn new(visible: usize, total: usize) -> Self {
        Self { visible, total }
    }
}

/// Consumer-owned state of the explicit default-view save operation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SnapshotDefaultSaveState {
    /// Current controls already match the accepted saved default.
    #[default]
    Clean,
    /// Current controls differ and may be saved.
    Dirty,
    /// A save request is in flight.
    Pending,
    /// The latest save was accepted.
    Saved,
    /// The server rejected a stale preference revision; the user may retry.
    Conflict(String),
    /// The latest save failed; the user may retry.
    Failure(String),
}

/// Persistence-neutral binding for one explicit Save as Default action.
#[derive(Clone)]
pub struct SnapshotDefaultSave {
    defaults: Signal<SnapshotViewDefaults>,
    state: Signal<SnapshotDefaultSaveState>,
    on_save: Callback<SnapshotViewDefaults>,
}

impl SnapshotDefaultSave {
    /// Creates an explicit save binding. Rendering or changing either signal
    /// never invokes `on_save`; only an enabled user activation does.
    pub fn new(
        defaults: impl Into<Signal<SnapshotViewDefaults>>,
        state: impl Into<Signal<SnapshotDefaultSaveState>>,
        on_save: Callback<SnapshotViewDefaults>,
    ) -> Self {
        Self {
            defaults: defaults.into(),
            state: state.into(),
            on_save,
        }
    }

    /// Returns the current projected payload without invoking persistence.
    pub fn defaults(&self) -> SnapshotViewDefaults {
        self.defaults.get_untracked()
    }

    /// Returns the consumer-owned request state.
    pub fn state(&self) -> SnapshotDefaultSaveState {
        self.state.get_untracked()
    }
}

/// Reactive framework-owned copy for the complete utility filter row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterBarTexts {
    /// Accessible name for the filter region.
    pub region_label: String,
    /// Active-filter summary when no filters are active.
    pub active_none: String,
    /// Active-filter summary when exactly one filter is active.
    pub active_one: String,
    /// Active-filter summary template for multiple filters.
    pub active_many: String,
    /// Accessible remove-chip template; `{label}` is replaced.
    pub remove_filter: String,
    /// Result template with `{visible}` and `{total}` placeholders.
    pub result_count: String,
    /// Reset action label.
    pub reset: String,
    /// Explicit persistence action label.
    pub save_default: String,
    /// Reason Save is disabled for a clean view.
    pub clean_reason: String,
    /// Reason Save is disabled during persistence.
    pub pending_reason: String,
    /// Live pending feedback.
    pub pending_feedback: String,
    /// Live success feedback.
    pub saved_feedback: String,
    /// Conflict template; `{message}` is replaced.
    pub conflict_feedback: String,
    /// Failure template; `{message}` is replaced.
    pub failure_feedback: String,
}

impl Default for FilterBarTexts {
    fn default() -> Self {
        Self {
            region_label: "Filters".to_owned(),
            active_none: "No active filters".to_owned(),
            active_one: "1 active filter".to_owned(),
            active_many: "{count} active filters".to_owned(),
            remove_filter: "Remove {label} filter".to_owned(),
            result_count: "{visible} of {total} results".to_owned(),
            reset: "Reset".to_owned(),
            save_default: "Save as Default".to_owned(),
            clean_reason: "Defaults are already saved".to_owned(),
            pending_reason: "A default view save is in progress".to_owned(),
            pending_feedback: "Saving default view".to_owned(),
            saved_feedback: "Default view saved".to_owned(),
            conflict_feedback: "Default view conflict: {message}".to_owned(),
            failure_feedback: "Could not save default view: {message}".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FilterSavePresentation {
    enabled: bool,
    disabled_reason: Option<String>,
    feedback: Option<String>,
    alert: bool,
}

fn filter_save_presentation(
    state: &SnapshotDefaultSaveState,
    texts: &FilterBarTexts,
) -> FilterSavePresentation {
    match state {
        SnapshotDefaultSaveState::Clean => FilterSavePresentation {
            enabled: false,
            disabled_reason: Some(texts.clean_reason.clone()),
            feedback: None,
            alert: false,
        },
        SnapshotDefaultSaveState::Dirty => FilterSavePresentation {
            enabled: true,
            disabled_reason: None,
            feedback: None,
            alert: false,
        },
        SnapshotDefaultSaveState::Pending => FilterSavePresentation {
            enabled: false,
            disabled_reason: Some(texts.pending_reason.clone()),
            feedback: Some(texts.pending_feedback.clone()),
            alert: false,
        },
        SnapshotDefaultSaveState::Saved => FilterSavePresentation {
            enabled: false,
            disabled_reason: Some(texts.clean_reason.clone()),
            feedback: Some(texts.saved_feedback.clone()),
            alert: false,
        },
        SnapshotDefaultSaveState::Conflict(message) => FilterSavePresentation {
            enabled: true,
            disabled_reason: None,
            feedback: Some(texts.conflict_feedback.replace("{message}", message)),
            alert: true,
        },
        SnapshotDefaultSaveState::Failure(message) => FilterSavePresentation {
            enabled: true,
            disabled_reason: None,
            feedback: Some(texts.failure_feedback.replace("{message}", message)),
            alert: true,
        },
    }
}

/// Produces the localized active-filter count from complete FilterBar copy.
pub fn filter_active_summary(count: usize, texts: &FilterBarTexts) -> String {
    match count {
        0 => texts.active_none.clone(),
        1 => texts.active_one.clone(),
        count => texts.active_many.replace("{count}", &count.to_string()),
    }
}

/// Produces the localized visible/total result summary.
pub fn filter_result_summary(summary: FilterResultSummary, texts: &FilterBarTexts) -> String {
    texts
        .result_count
        .replace("{visible}", &summary.visible.to_string())
        .replace("{total}", &summary.total.to_string())
}

/// Search-first, actions-last row for local filters.
///
/// The historical layout-only `search`/`actions`/`children` shape remains
/// available. Supplying the controlled optional bindings adds the canonical
/// active summary, result count, one Reset, one Save as Default, and save
/// feedback without taking ownership of any filter or persistence state.
#[component]
pub fn FilterBar(
    /// Search control rendered first and allowed to grow.
    search: Children,
    /// Compatibility action content rendered before framework actions.
    #[prop(optional)]
    actions: Option<Children>,
    /// Controlled active-filter chips. Requires `on_remove` when supplied.
    #[prop(optional)]
    active_filters: Option<Signal<Vec<ActiveFilterChip>>>,
    /// Removes one controlled filter by stable key.
    #[prop(optional)]
    on_remove: Option<Callback<String>>,
    /// The one canonical Reset action across utility and aligned filters.
    #[prop(optional)]
    on_reset: Option<Callback<()>>,
    /// Current locally visible and authoritative row counts.
    #[prop(optional)]
    result: Option<Signal<FilterResultSummary>>,
    /// Optional explicit persistence binding.
    #[prop(optional)]
    default_save: Option<SnapshotDefaultSave>,
    /// Reactive framework-owned visible and accessibility copy.
    #[prop(into, default = Signal::stored(FilterBarTexts::default()))]
    texts: Signal<FilterBarTexts>,
    /// Additional classes.
    #[prop(optional, into)]
    class: &'static str,
    /// Selects and other utility-only local filter controls.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let active_binding = match (active_filters, on_remove) {
        (Some(chips), Some(remove)) => Some((chips, remove)),
        (None, None) => None,
        (Some(_), None) => panic!("FilterBar active_filters requires on_remove"),
        (None, Some(_)) => panic!("FilterBar on_remove requires active_filters"),
    };
    let save = StoredValue::new(default_save);
    let has_framework_actions = on_reset.is_some() || save.get_value().is_some();
    let active_texts = Signal::derive(move || {
        texts.with(|texts| ActiveFilterTexts {
            none: texts.active_none.clone(),
            one: texts.active_one.clone(),
            many: texts.active_many.clone(),
            remove: texts.remove_filter.clone(),
            clear: texts.reset.clone(),
        })
    });

    view! {
        <section
            class=filter_bar_class(class)
            data-filter-bar="local"
            aria-label=move || texts.with(|texts| texts.region_label.clone())
        >
            <div class="min-w-56 flex-[2_1_20rem]" data-filter-search="true">
                {search()}
            </div>
            <div class="flex min-w-0 flex-[3_1_28rem] flex-wrap items-end gap-3">
                {children.map(|children| children())}
            </div>
            {active_binding.map(|(chips, on_remove)| view! {
                <div class="basis-full" data-filter-summary="true">
                    <ActiveFilterChips chips=chips on_remove=on_remove texts=active_texts />
                </div>
            })}
            {result.map(|result| view! {
                <p class="text-xs text-base-content/75" data-filter-result-count="true">
                    {move || texts.with(|texts| filter_result_summary(result.get(), texts))}
                </p>
            })}
            {(actions.is_some() || has_framework_actions).then(|| view! {
                <div class="ml-auto flex shrink-0 items-center gap-2" data-filter-actions="true">
                    {actions.map(|actions| actions())}
                    {on_reset.map(|callback| view! {
                        <Button
                            class="btn-ghost btn-sm"
                            attr:data-filter-reset="true"
                            disabled=Signal::derive(move || {
                                active_binding
                                    .as_ref()
                                    .is_some_and(|(chips, _)| chips.with(|chips| chips.is_empty()))
                            })
                            on_click=Callback::new(move |_| callback.run(()))
                        >
                            {move || texts.with(|texts| texts.reset.clone())}
                        </Button>
                    })}
                    {save.get_value().map(|binding| {
                        let label_binding = binding.clone();
                        let disabled_binding = binding.clone();
                        let click_binding = binding.clone();
                        view! {
                            <Button
                                class="btn-primary btn-sm"
                                attr:data-filter-save-default="true"
                                attr:aria-label=move || texts.with(|texts| {
                                    let presentation = filter_save_presentation(
                                        &label_binding.state.get(),
                                        texts,
                                    );
                                    presentation.disabled_reason.map_or_else(
                                        || texts.save_default.clone(),
                                        |reason| format!("{}. {reason}", texts.save_default),
                                    )
                                })
                                disabled=Signal::derive(move || texts.with(|texts| {
                                    !filter_save_presentation(
                                        &disabled_binding.state.get(),
                                        texts,
                                    ).enabled
                                }))
                                on_click=Callback::new(move |_| {
                                    let state = click_binding.state.get_untracked();
                                    let texts = texts.get_untracked();
                                    if filter_save_presentation(&state, &texts).enabled {
                                        click_binding.on_save.run(
                                            click_binding.defaults.get_untracked(),
                                        );
                                    }
                                })
                            >
                                {move || texts.with(|texts| texts.save_default.clone())}
                            </Button>
                        }
                    })}
                </div>
            })}
            {save.get_value().map(|binding| view! {
                {move || texts.with(|texts| {
                    let presentation = filter_save_presentation(&binding.state.get(), texts);
                    presentation.feedback.map(|message| {
                        if presentation.alert {
                            view! {
                                <p
                                    class="basis-full text-xs text-error"
                                    role="alert"
                                    data-filter-save-feedback="alert"
                                >
                                    {message}
                                </p>
                            }.into_any()
                        } else {
                            view! {
                                <p
                                    class="basis-full text-xs text-base-content/75"
                                    role="status"
                                    data-filter-save-feedback="status"
                                >
                                    {message}
                                </p>
                            }.into_any()
                        }
                    })
                })}
            })}
        </section>
    }
}

#[cfg(test)]
#[path = "filter_bar/tests.rs"]
mod tests;
