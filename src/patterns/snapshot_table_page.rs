//! Typed Layer 3 composition root for client-snapshot table pages.

use super::{
    ActionFeedback, ActionFeedbackModel, ActionFeedbackTexts, DatasetOption, DatasetSelector,
    DatasetSelectorTexts, LocalResultSummary, PageStatePanel, PageStatePanelTexts,
    SnapshotLocalRowProjection, SnapshotTablePhase, SnapshotTableState,
};
use crate::components::{
    EntityColumnFilters, EntityColumns, EntityCompactRow, EntityRowKey, EntityTable,
    EntityTablePreferenceOwnership, EntityTableTexts,
};
use leptos::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

/// One typed dataset option used by the canonical selector config.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotDatasetOption<V> {
    value: V,
    label: String,
    disabled: bool,
}

impl<V> SnapshotDatasetOption<V> {
    /// Creates an enabled typed dataset option.
    pub fn new(value: V, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }

    /// Marks the option unavailable without changing its identity.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Typed value sent to the consumer request callback.
    pub const fn value(&self) -> &V {
        &self.value
    }

    /// Localized option label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Whether the option is currently unavailable.
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }
}

/// Dataset selector mechanics that deliberately omit selected, requested, or
/// displayed identity. [`SnapshotTablePage`] injects those values from its
/// private-field state view.
pub struct SnapshotDatasetSelectorConfig<V: Send + Sync + 'static> {
    label: Signal<String>,
    options: Signal<Vec<SnapshotDatasetOption<V>>>,
    value_key: Arc<dyn Fn(&V) -> String + Send + Sync>,
    on_request: Callback<V>,
    disabled: Signal<bool>,
}

impl<V: Send + Sync + 'static> SnapshotDatasetSelectorConfig<V> {
    /// Creates a selector config with framework-injected selection state.
    pub fn new(
        label: impl Into<Signal<String>>,
        options: impl Into<Signal<Vec<SnapshotDatasetOption<V>>>>,
        value_key: Arc<dyn Fn(&V) -> String + Send + Sync>,
        on_request: Callback<V>,
    ) -> Self {
        Self {
            label: label.into(),
            options: options.into(),
            value_key,
            on_request,
            disabled: Signal::stored(false),
        }
    }

    /// Adds an explicit consumer-owned availability gate.
    pub fn with_disabled(mut self, disabled: impl Into<Signal<bool>>) -> Self {
        self.disabled = disabled.into();
        self
    }
}

/// EntityTable mechanics that deliberately omit rows, dataset identity,
/// revision, authoritative count, and generation. The page injects all of
/// those identity-critical bindings from the same state view as the selector.
pub struct SnapshotEntityTableConfig<R: 'static> {
    columns: EntityColumns<R>,
    row_key: EntityRowKey<R>,
    preference_ownership: EntityTablePreferenceOwnership,
    preference_version: u16,
    compact_row: EntityCompactRow<R>,
    column_filters: EntityColumnFilters,
    on_row_activate: Option<Callback<String>>,
    texts: Signal<EntityTableTexts>,
    show_reset_actions: bool,
    zebra: Signal<bool>,
    class: &'static str,
}

impl<R: 'static> SnapshotEntityTableConfig<R> {
    /// Creates a persistence-neutral canonical table config.
    pub fn new(
        columns: impl Into<EntityColumns<R>>,
        row_key: EntityRowKey<R>,
        preference_ownership: EntityTablePreferenceOwnership,
    ) -> Self {
        Self {
            columns: columns.into(),
            row_key,
            preference_ownership,
            preference_version: 1,
            compact_row: EntityCompactRow::Default,
            column_filters: EntityColumnFilters::None,
            on_row_activate: None,
            texts: Signal::stored(EntityTableTexts::default()),
            show_reset_actions: false,
            zebra: Signal::stored(false),
            class: "",
        }
    }

    /// Sets the consumer preference schema version.
    pub const fn with_preference_version(mut self, version: u16) -> Self {
        self.preference_version = version;
        self
    }

    /// Supplies a compact-row renderer without changing row identity.
    pub fn with_compact_row(mut self, renderer: impl Into<EntityCompactRow<R>>) -> Self {
        self.compact_row = renderer.into();
        self
    }

    /// Supplies controlled filters aligned beneath stable table columns.
    pub fn with_column_filters(mut self, filters: impl Into<EntityColumnFilters>) -> Self {
        self.column_filters = filters.into();
        self
    }

    /// Supplies the typed row-activation intent.
    pub fn on_row_activate(mut self, callback: Callback<String>) -> Self {
        self.on_row_activate = Some(callback);
        self
    }

    /// Supplies reactive EntityTable-owned copy.
    pub fn with_texts(mut self, texts: impl Into<Signal<EntityTableTexts>>) -> Self {
        self.texts = texts.into();
        self
    }

    /// Shows explicit reset-sort and reset-columns actions.
    pub const fn show_reset_actions(mut self, show: bool) -> Self {
        self.show_reset_actions = show;
        self
    }

    /// Opts into alternating rows.
    pub fn with_zebra(mut self, zebra: impl Into<Signal<bool>>) -> Self {
        self.zebra = zebra.into();
        self
    }

    /// Adds outer EntityTable classes.
    pub const fn with_class(mut self, class: &'static str) -> Self {
        self.class = class;
        self
    }
}

/// Canonical, explicit composition root for one complete client-snapshot page.
///
/// The component owns slot order and identity wiring only. Consumers still own
/// request transport, domain rows/columns, filtering, permissions, mutation
/// callbacks, persistence callbacks, and routes.
#[component]
pub fn SnapshotTablePage<R, V, E, M, K>(
    /// Stable page-contract ID; also prefixes observable region IDs.
    contract_id: &'static str,
    /// The single private-field runtime controller.
    state: Signal<SnapshotTableState<R, V, E, M, K>, LocalStorage>,
    /// Optional local filtered-count proof minted by the same state.
    #[prop(optional)]
    local_result: Option<Signal<Option<LocalResultSummary>, LocalStorage>>,
    /// Optional controlled rows minted by `state`. When present, their
    /// identity-bound summary supersedes the legacy count-only proof.
    #[prop(optional)]
    local_rows: Option<Signal<Option<SnapshotLocalRowProjection<R>>, LocalStorage>>,
    /// PageHeader composition. The canonical path leaves its dataset slot empty.
    header: Children,
    /// Typed selector mechanics with no selected/displayed identity field.
    dataset_selector: SnapshotDatasetSelectorConfig<V>,
    /// Optional full-width KPI content.
    #[prop(optional)]
    kpis: Option<Children>,
    /// Controlled local-filter utility content.
    filters: Children,
    /// Reactive dataset loading/display/error/retry copy.
    #[prop(into, default = Signal::stored(DatasetSelectorTexts::default()))]
    dataset_texts: Signal<DatasetSelectorTexts>,
    /// Typed table mechanics with no rows/dataset/revision/generation field.
    entity_table: SnapshotEntityTableConfig<R>,
    /// Reactive page-state copy.
    #[prop(into, default = Signal::stored(PageStatePanelTexts::default()))]
    panel_texts: Signal<PageStatePanelTexts>,
    /// Optional retry intent for initial or retained load failure.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
    /// Reactive keyed-action copy.
    #[prop(into, default = Signal::stored(ActionFeedbackTexts::default()))]
    action_texts: Signal<ActionFeedbackTexts>,
    /// Stable human-readable label for one action key.
    action_key_label: Rc<dyn Fn(&K) -> String>,
    /// Optional keyed retry intent.
    #[prop(optional)]
    on_action_retry: Option<Callback<K>>,
    /// Optional keyed dismissal intent.
    #[prop(optional)]
    on_action_dismiss: Option<Callback<K>>,
    /// Additional page-root classes.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView
where
    R: Clone + 'static,
    V: Clone + PartialEq + Send + Sync + 'static,
    E: ToString + 'static,
    M: 'static,
    K: Clone + Eq + Send + Sync + 'static,
{
    let SnapshotDatasetSelectorConfig {
        label,
        options,
        value_key,
        on_request,
        disabled,
    } = dataset_selector;
    let selected = RwSignal::new(String::new());
    let selector_options = RwSignal::new(Vec::<DatasetOption>::new());
    let loading = RwSignal::new(false);
    let load_error = RwSignal::new(Option::<String>::None);
    let generation_marker = RwSignal::new("0".to_owned());
    let local_result = StoredValue::new_local(local_result);
    let local_rows = StoredValue::new_local(local_rows);
    let entity_table = StoredValue::new_local(entity_table);
    let effective_local_result = Signal::derive_local(move || {
        local_rows
            .with_value(|projection| {
                projection.as_ref().and_then(|projection| {
                    projection.with(|projection| {
                        projection
                            .as_ref()
                            .map(|projection| projection.summary().clone())
                    })
                })
            })
            .or_else(|| {
                local_result
                    .with_value(|summary| summary.as_ref().and_then(|summary| summary.get()))
            })
    });

    let option_value_key = Arc::clone(&value_key);
    Effect::new(move |_| {
        let projected = options.with(|options| {
            options
                .iter()
                .map(|option| DatasetOption {
                    value: option_value_key(&option.value),
                    label: option.label.clone(),
                    disabled: option.disabled,
                })
                .collect::<Vec<_>>()
        });
        selector_options.set(projected);
    });

    let selected_value_key = Arc::clone(&value_key);
    Effect::new(move |_| {
        state.with(|state| {
            let summary = effective_local_result.get();
            let view = state.view(summary.as_ref());
            generation_marker.set(view.generation().marker());
            loading.set(matches!(
                view.phase(),
                SnapshotTablePhase::InitialLoading | SnapshotTablePhase::Replacing
            ));
            load_error.set(view.load_error().map(ToString::to_string));
            let selected_dataset = view
                .requested_dataset()
                .or_else(|| view.displayed().map(|snapshot| snapshot.dataset()));
            selected.set(
                selected_dataset
                    .map(|value| selected_value_key(value))
                    .unwrap_or_default(),
            );
        });
    });

    let request_options = options;
    let request_value_key = Arc::clone(&value_key);
    let on_selector_change = Callback::new(move |next: String| {
        request_options.with(|options| {
            if let Some(option) = options
                .iter()
                .find(|option| request_value_key(&option.value) == next && !option.disabled)
            {
                on_request.run(option.value.clone());
            }
        });
    });

    let authoritative_rows = Signal::derive_local(move || {
        state.with(|state| {
            state
                .view(None)
                .displayed()
                .map(|snapshot| Rc::clone(snapshot.rows()))
                .unwrap_or_else(|| Rc::new(Vec::new()))
        })
    });
    let table_rows = Signal::derive_local(move || {
        state.with(|state| {
            local_rows
                .with_value(|projection| {
                    projection.as_ref().and_then(|projection| {
                        projection.with(|projection| {
                            projection.as_ref().and_then(|projection| {
                                state.validated_local_rows(projection).map(Rc::clone)
                            })
                        })
                    })
                })
                .or_else(|| {
                    state
                        .view(None)
                        .displayed()
                        .map(|snapshot| Rc::clone(snapshot.rows()))
                })
                .unwrap_or_else(|| Rc::new(Vec::new()))
        })
    });
    let action_model = Signal::derive_local(move || {
        state.with(|state| {
            let model: &ActionFeedbackModel<K> = state.actions();
            model.clone()
        })
    });

    let dataset_id = format!("{contract_id}-dataset");
    let kpis_id = format!("{contract_id}-kpis");
    let filters_id = format!("{contract_id}-filters");
    let feedback_id = format!("{contract_id}-feedback");
    let table_id = format!("{contract_id}-table");

    view! {
        <section
            id=contract_id
            class=format!("flex w-full min-w-0 flex-col gap-4 {class}")
            data-snapshot-table-page="true"
            data-snapshot-generation=move || generation_marker.get()
            data-snapshot-phase=move || state.with(|state| format!("{:?}", state.view(None).phase()))
        >
            <div data-snapshot-page-slot="header">{header()}</div>
            <div
                id=dataset_id
                data-snapshot-page-slot="dataset"
                data-snapshot-generation=move || generation_marker.get()
            >
                <DatasetSelector
                    control_id=format!("{contract_id}-dataset-select")
                    label=label
                    selected=selected
                    options=selector_options
                    on_change=on_selector_change
                    loading=loading
                    disabled=disabled
                    error=load_error
                    texts=dataset_texts
                    nostrip:on_retry=on_retry
                />
            </div>
            {kpis.map(|kpis| view! {
                <div id=kpis_id data-snapshot-page-slot="kpis">{kpis()}</div>
            })}
            <div id=filters_id data-snapshot-page-slot="filters">{filters()}</div>
            <div id=feedback_id class="space-y-2" data-snapshot-page-slot="feedback">
                {move || state.with(|state| {
                    let summary = effective_local_result.get();
                    let decision = state.view(summary.as_ref()).render_decision();
                    decision.retained_notice().then(|| {
                        let kind = decision.panel().expect("retained notice has panel");
                        view! {
                            <PageStatePanel
                                kind=kind
                                texts=panel_texts
                                nostrip:on_retry=on_retry
                                detail=load_error
                            />
                        }
                    })
                })}
                <ActionFeedback
                    model=action_model
                    texts=action_texts
                    key_label=action_key_label
                    nostrip:on_retry=on_action_retry
                    nostrip:on_dismiss=on_action_dismiss
                />
            </div>
            <div
                id=table_id
                data-snapshot-page-slot="table"
                data-snapshot-generation=move || generation_marker.get()
            >
                <Show
                    when=move || state.with(|state| {
                        let summary = effective_local_result.get();
                        state.view(summary.as_ref()).render_decision().table_mounted()
                    })
                    fallback=move || state.with(|state| {
                        let summary = effective_local_result.get();
                        let view = state.view(summary.as_ref());
                        let decision = view.render_decision();
                        decision.panel().map(|kind| view! {
                            <PageStatePanel
                                kind=kind
                                texts=panel_texts
                                nostrip:on_retry=on_retry
                                detail=Signal::stored(view.load_error().map(ToString::to_string))
                            />
                        })
                    })
                >
                    {entity_table.with_value(|config| view! {
                        <EntityTable
                            data=table_rows
                            source_data=authoritative_rows
                            columns=config.columns.clone()
                            row_key=Rc::clone(&config.row_key)
                            dataset_identity=generation_marker
                            focus_scope=generation_marker
                            compact_row=config.compact_row.clone()
                            column_filters=config.column_filters.clone()
                            nostrip:on_row_activate=config.on_row_activate
                            preference_ownership=config.preference_ownership.clone()
                            preference_version=config.preference_version
                            texts=config.texts
                            page_size_control_id=format!("{contract_id}-rows-per-page")
                            show_reset_actions=config.show_reset_actions
                            zebra=config.zebra
                            class=config.class
                        />
                    })}
                </Show>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{EntityTablePreferenceOwnership, EntityTablePreferences};

    #[derive(Clone)]
    struct Row;

    #[test]
    fn canonical_configs_are_constructed_without_identity_critical_values() {
        let selector = SnapshotDatasetSelectorConfig::new(
            Signal::stored("Office".to_owned()),
            Signal::stored(vec![SnapshotDatasetOption::new("mx", "Mexico")]),
            Arc::new(|value: &&str| (*value).to_owned()),
            Callback::new(|_: &str| {}),
        );
        assert!(!selector.disabled.get_untracked());

        let preferences = RwSignal::new(EntityTablePreferences::new(1));
        let table = SnapshotEntityTableConfig::<Row>::new(
            Vec::new(),
            Rc::new(|_: &Row| "row".to_owned()),
            EntityTablePreferenceOwnership::controlled(
                preferences.into(),
                Callback::new(move |next| preferences.set(next)),
            ),
        );
        assert_eq!(table.preference_version, 1);
        assert!(!table.show_reset_actions);
    }
}
