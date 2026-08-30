//! Typed Layer 3 composition root for client-snapshot table pages.

use super::{
    ActionFeedback, ActionFeedbackModel, ActionFeedbackTexts, DatasetOption, DatasetSelector,
    DatasetSelectorTexts, LocalResultSummary, PageStatePanel, PageStatePanelTexts,
    SnapshotLocalRowProjection, SnapshotTablePhase, SnapshotTableState,
};
use crate::components::{
    EntityColumnChooserTrigger, EntityColumnFilters, EntityColumns, EntityCompactRow, EntityRowKey,
    EntityTable, EntityTableActionColumnPolicy, EntityTableDisplayProjection,
    EntityTablePreferenceOwnership, EntityTableTexts, EntityTableViewportFit,
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
///
/// Everything else the internally owned `EntityTable` can express as pure
/// behavior -- local-filter page reset, viewport-fit paging, a caller
/// toolbar, the display-projection callback/action-column policy, and
/// chooser presentation -- is a typed passthrough here (`ldui-myhh`,
/// `ldui-5ano`). None of these can carry rows, dataset identity, revision,
/// count, or generation: their types simply have no such field, so a caller
/// cannot smuggle identity through them even by accident.
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
    page_reset_key: Option<Signal<String>>,
    viewport_fit: Option<EntityTableViewportFit>,
    toolbar_actions: Option<ChildrenFn>,
    on_display_projection: Option<Callback<EntityTableDisplayProjection>>,
    projection_action_columns: EntityTableActionColumnPolicy,
    column_chooser_trigger: Signal<EntityColumnChooserTrigger>,
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
            page_reset_key: None,
            viewport_fit: None,
            toolbar_actions: None,
            on_display_projection: None,
            projection_action_columns: EntityTableActionColumnPolicy::default(),
            column_chooser_trigger: Signal::stored(EntityColumnChooserTrigger::default()),
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

    /// Supplies a caller-owned view-state identity distinct from dataset
    /// identity and generation, which the page continues to inject itself.
    /// Changing it resets pagination while preserving dataset-independent
    /// filters, sort, page size, and columns -- for example hashing local
    /// filter values so a filter change made from a later page returns the
    /// table to page one without disturbing the dataset/access generation
    /// this page already binds to `dataset_identity`/`focus_scope`.
    pub fn with_page_reset_key(mut self, key: impl Into<Signal<String>>) -> Self {
        self.page_reset_key = Some(key.into());
        self
    }

    /// Opts the internally owned `EntityTable` into framework-measured
    /// viewport-fit paging. Presentation-only: the measured row capacity
    /// never changes persisted table preferences, rows, or dataset identity.
    pub fn with_viewport_fit(mut self, viewport_fit: EntityTableViewportFit) -> Self {
        self.viewport_fit = Some(viewport_fit);
        self
    }

    /// Supplies caller-rendered table toolbar content, such as Export or
    /// Refresh. The table owns placement (after page size, before the
    /// framework-owned column chooser) and wrapping; the caller owns all
    /// behavior. Never a route to identity: the renderer receives no rows,
    /// dataset, revision, or generation from this config.
    pub fn with_toolbar_actions(
        mut self,
        render: impl Fn() -> AnyView + Send + Sync + 'static,
    ) -> Self {
        self.toolbar_actions = Some(Arc::new(render));
        self
    }

    /// Supplies the atomic read-only display-projection callback. It fires
    /// whenever displayed rows, columns, ordering, paging, or canonical text
    /// change; the caller owns storage, export encoding, authorization, and
    /// download behavior. The projection is a read-only copy of what is
    /// already rendered -- it carries no dataset identity, revision, or
    /// generation of its own.
    pub fn on_display_projection(
        mut self,
        callback: Callback<EntityTableDisplayProjection>,
    ) -> Self {
        self.on_display_projection = Some(callback);
        self
    }

    /// Sets the action-column policy applied to `on_display_projection`
    /// snapshots. Defaults to excluding action columns, since their canonical
    /// copy normally describes UI rather than exported domain data.
    pub const fn with_projection_action_columns(
        mut self,
        policy: EntityTableActionColumnPolicy,
    ) -> Self {
        self.projection_action_columns = policy;
        self
    }

    /// Sets the framework-owned column-chooser trigger presentation (the
    /// localized text label by default, or a compact icon glyph). Both
    /// presentations keep the same accessible semantics; this never changes
    /// what the chooser controls.
    pub fn with_column_chooser_trigger(
        mut self,
        trigger: impl Into<Signal<EntityColumnChooserTrigger>>,
    ) -> Self {
        self.column_chooser_trigger = trigger.into();
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
                    {entity_table.with_value(|config| {
                        // `ChildrenFn` is reusable (`Arc<dyn Fn() -> AnyView>`)
                        // because this whole block can re-run whenever `Show`
                        // remounts the table, but `EntityTable::toolbar_actions`
                        // wants one single-shot `Children` per instance -- so a
                        // fresh box is built from the stored renderer every time.
                        let toolbar_actions = config
                            .toolbar_actions
                            .clone()
                            .map(|render| Box::new(move || render()) as Children);
                        view! {
                            <EntityTable
                                data=table_rows
                                source_data=authoritative_rows
                                columns=config.columns.clone()
                                row_key=Rc::clone(&config.row_key)
                                dataset_identity=generation_marker
                                nostrip:page_reset_key=config.page_reset_key
                                nostrip:viewport_fit=config.viewport_fit.clone()
                                focus_scope=generation_marker
                                compact_row=config.compact_row.clone()
                                column_filters=config.column_filters.clone()
                                nostrip:on_row_activate=config.on_row_activate
                                preference_ownership=config.preference_ownership.clone()
                                preference_version=config.preference_version
                                texts=config.texts
                                page_size_control_id=format!("{contract_id}-rows-per-page")
                                show_reset_actions=config.show_reset_actions
                                nostrip:toolbar_actions=toolbar_actions
                                nostrip:on_display_projection=config.on_display_projection
                                projection_action_columns=config.projection_action_columns
                                column_chooser_trigger=config.column_chooser_trigger
                                zebra=config.zebra
                                class=config.class
                            />
                        }
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
        assert!(table.page_reset_key.is_none());
        assert!(table.viewport_fit.is_none());
        assert!(table.toolbar_actions.is_none());
        assert!(table.on_display_projection.is_none());
        assert_eq!(
            table.projection_action_columns,
            EntityTableActionColumnPolicy::Exclude
        );
        assert_eq!(
            table.column_chooser_trigger.get_untracked(),
            EntityColumnChooserTrigger::Text
        );
    }

    fn base_table_config() -> SnapshotEntityTableConfig<Row> {
        let preferences = RwSignal::new(EntityTablePreferences::new(1));
        SnapshotEntityTableConfig::<Row>::new(
            Vec::new(),
            Rc::new(|_: &Row| "row".to_owned()),
            EntityTablePreferenceOwnership::controlled(
                preferences.into(),
                Callback::new(move |next| preferences.set(next)),
            ),
        )
    }

    /// `ldui-myhh` / `ldui-5ano`: every typed behavior-only builder actually
    /// lands on the config's private field, and each field's type structurally
    /// forbids carrying rows, dataset identity, revision, count, or
    /// generation -- there is no setter that could smuggle one through.
    #[test]
    fn behavior_only_builders_forward_to_typed_fields() {
        let key = RwSignal::new("filters:urgent".to_owned());
        let trigger = RwSignal::new(EntityColumnChooserTrigger::Icon);
        let projection_calls = RwSignal::new(0_u32);
        let table = base_table_config()
            .with_page_reset_key(key)
            .with_viewport_fit(EntityTableViewportFit::fill_parent().with_min_rows(4))
            .with_toolbar_actions(|| view! { <button>"Export"</button> }.into_any())
            .on_display_projection(Callback::new(move |_: EntityTableDisplayProjection| {
                projection_calls.update(|count| *count += 1);
            }))
            .with_projection_action_columns(EntityTableActionColumnPolicy::Include)
            .with_column_chooser_trigger(trigger);

        assert_eq!(
            table
                .page_reset_key
                .expect("page_reset_key was set")
                .get_untracked(),
            "filters:urgent"
        );
        let viewport_fit = table.viewport_fit.expect("viewport_fit was set");
        assert_eq!(viewport_fit.min_rows(), 4);
        assert!(table.toolbar_actions.is_some());
        let on_display_projection = table
            .on_display_projection
            .expect("on_display_projection was set");
        on_display_projection.run(EntityTableDisplayProjection::default());
        assert_eq!(projection_calls.get_untracked(), 1);
        assert_eq!(
            table.projection_action_columns,
            EntityTableActionColumnPolicy::Include
        );
        assert_eq!(
            table.column_chooser_trigger.get_untracked(),
            EntityColumnChooserTrigger::Icon
        );
    }
}
