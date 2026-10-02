//! Typed Layer 3 composition root for client-snapshot table pages.

use super::{
    ActionFeedback, ActionFeedbackModel, ActionFeedbackTexts, ActiveFilterChip, DatasetOption,
    DatasetSelector, DatasetSelectorTexts, FilterBar, FilterBarTexts, FilterResultSummary,
    LocalResultSummary, PageStatePanel, PageStatePanelTexts, SnapshotDefaultSave,
    SnapshotLocalRowProjection, SnapshotRenderDecision, SnapshotTablePhase, SnapshotTableState,
};
use crate::components::{
    EntityColumnChooserTrigger, EntityColumnFilters, EntityColumns, EntityCompactRow, EntityRowKey,
    EntityTable, EntityTableActionColumnPolicy, EntityTableDisplayProjection,
    EntityTablePreferenceOwnership, EntityTableTexts, EntityTableViewportFit,
};
use leptos::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

/// ldui-75xy: a consumer-filled slot that renders no element costs nothing --
/// no box and, because it is `display: none`, no flex gap either. Without it
/// an empty `filters` slot still spent the column's 16px gap (4iiz-Office
/// No-Hires measured ~90px of blank between the dataset select and the table).
const EMPTY_SLOT_HIDDEN: &str = "[&:not(:has(*))]:hidden";

/// ldui-xhrw: a slot whose element descendants are ALL empty, UNSTYLED
/// `div`/`span` wrappers (a consumer's bare `<div>`) renders nothing either,
/// but [`EMPTY_SLOT_HIDDEN`] cannot see that, so the slot stayed a 0px flex
/// item spending a gap. It goes `sr-only` -- out of flow -- rather than
/// `hidden`, because such a wrapper may hold a live region that must already
/// be rendered when its first message arrives. "Unstyled" (no `class`, no
/// `style`) is load-bearing: an empty element can still paint -- a daisyUI
/// `loading` spinner or a `skeleton` block has no children -- so any styled
/// element, any other element (an input, a paragraph) or any text keeps the
/// slot in flow. The rule errs toward showing.
const EMPTY_WRAPPERS_OUT_OF_FLOW: &str =
    "[&:has(*):not(:has(:not(:is(div,span):empty:not([class]):not([style]))))]:sr-only";

/// ldui-8ia5: a `filters` slot whose only child is a [`FilterBar`] that has
/// collapsed itself (`data-filter-bar-empty`) is still an element, so
/// [`EMPTY_SLOT_HIDDEN`] keeps it as a 0px flex item that spends a gap. It
/// goes `sr-only` -- out of flow, like the idle feedback slot -- and never
/// `display: none`: the FilterBar decides "empty" by observing its own
/// rendered geometry, and a live region inside it must stay rendered to be
/// announced. When the bar shows content it re-expands, the attribute goes,
/// and the slot rejoins the flow.
const COLLAPSED_FILTER_BAR_OUT_OF_FLOW: &str =
    "[&:has(>[data-filter-bar-empty=true]:only-child)]:sr-only";

fn snapshot_page_layout(
    fit: Option<&EntityTableViewportFit>,
) -> (&'static str, Option<&'static str>) {
    if fit.is_some_and(|fit| fit.height().is_none()) {
        (
            "flex w-full min-w-0 flex-col gap-4 min-h-0 flex-1",
            Some("flex min-h-0 flex-1 flex-col"),
        )
    } else {
        ("flex w-full min-w-0 flex-col gap-4", None)
    }
}

/// The table slot's class once an optional side panel is known.
///
/// Without a panel the slot keeps EXACTLY the class `snapshot_page_layout`
/// chose, so the fill_parent height chain (op-at7nn) is untouched for every
/// page that does not opt in. With one it becomes a row -- and in fill mode the
/// row must still carry `min-h-0 flex-1`, or the table inside loses its height
/// budget and collapses with no error, the failure op-at7nn exists to prevent.
fn snapshot_table_slot_class(
    fill_slot_class: Option<&'static str>,
    has_side_panel: bool,
) -> Option<&'static str> {
    match (fill_slot_class, has_side_panel) {
        (class, false) => class,
        (Some(_), true) => Some("flex min-h-0 flex-1 flex-col gap-4 lg:flex-row"),
        (None, true) => Some("flex flex-col gap-4 lg:flex-row"),
    }
}

/// What the table slot holds in front of an optional side panel
/// (ldui-pt0x, Office op-zwrly).
///
/// With a side panel the slot is a row and the `<aside>` comes AFTER the
/// table. Until the table mounts, `Show` renders the replacement
/// [`PageStatePanel`] there instead, and that panel is a content-sized flex
/// item -- a daisyUI alert sets no width, so it is as wide as its one
/// sentence -- so the aside painted just right of the sentence and jumped to
/// the right edge when the table mounted. 4iiz-Office No-Hire measured the
/// Client Coordinator panel at x=243, then 215, then 1041 once the table
/// arrived: one 0.2016 layout shift. Every replacement state (never loaded,
/// initial loading, initial error, empty dataset, no local results, expired,
/// forbidden) therefore renders its panel inside a placeholder sized like
/// the table, so the aside is in its final place from the first frame.
/// Without a side panel nothing follows the table, and the panel stays bare
/// exactly as before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapshotTableSlotLead {
    /// The table is mounted; a retained notice, if any, overlays it.
    Table,
    /// No side panel: the replacement panel renders bare, as it always has.
    Panel,
    /// A side panel follows: the replacement panel renders inside the
    /// [`SNAPSHOT_TABLE_PLACEHOLDER_CLASS`] placeholder.
    ReservedPanel,
}

/// The placeholder's whole class: `EntityTable`'s root sizing, which is
/// `w-full min-w-0` in both its natural and its viewport-fit class. In the
/// `lg:flex-row` slot it therefore takes the table's flex basis and shrinks
/// the same way beside the `shrink-0` aside, so the aside lands exactly where
/// the table will leave it. Nothing else -- no padding, border, background or
/// overflow -- so it paints nothing and adds no scrollbar. It is a role-less
/// `div` with no text of its own, and it is NOT `aria-hidden`: the panel it
/// holds is a live status that must still be announced.
const SNAPSHOT_TABLE_PLACEHOLDER_CLASS: &str = "w-full min-w-0";

/// The one decision both halves of the table slot's `Show` read: `when`
/// mounts the table only for [`SnapshotTableSlotLead::Table`], and the
/// fallback wraps the panel only for [`SnapshotTableSlotLead::ReservedPanel`].
fn snapshot_table_slot_lead(
    decision: SnapshotRenderDecision,
    has_side_panel: bool,
) -> SnapshotTableSlotLead {
    match (decision.table_mounted(), has_side_panel) {
        (true, _) => SnapshotTableSlotLead::Table,
        (false, false) => SnapshotTableSlotLead::Panel,
        (false, true) => SnapshotTableSlotLead::ReservedPanel,
    }
}

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
    compact: bool,
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
            compact: false,
        }
    }

    /// Adds an explicit consumer-owned availability gate.
    pub fn with_disabled(mut self, disabled: impl Into<Signal<bool>>) -> Self {
        self.disabled = disabled.into();
        self
    }

    /// Renders the bare select -- no card, eyebrow, spinner or "Showing"
    /// caption -- the portfolio rule for an office selector centred in the
    /// page header (Office op-v7c5g). See `DatasetSelector`'s `compact`.
    #[must_use]
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Whether the selector renders compact.
    pub const fn is_compact(&self) -> bool {
        self.compact
    }
}

/// EntityTable mechanics that deliberately omit rows, dataset identity,
/// revision, authoritative count, and generation. The page injects all of
/// those identity-critical bindings from the same state view as the selector.
///
/// Everything else the internally owned `EntityTable` can express as pure
/// behavior -- local-filter page reset, viewport-fit paging, a caller
/// toolbar, the display-projection callback/action-column policy, and
/// chooser and empty-range presentation -- is a typed passthrough here
/// (`ldui-myhh`, `ldui-5ano`, `ldui-r50n`). None of these can carry rows,
/// dataset identity, revision, count, or generation: their types simply have
/// no such field, so a caller cannot smuggle identity through them even by
/// accident.
pub struct SnapshotEntityTableConfig<R: 'static> {
    columns: EntityColumns<R>,
    row_key: EntityRowKey<R>,
    preference_ownership: EntityTablePreferenceOwnership,
    preference_version: u16,
    compact_row: EntityCompactRow<R>,
    column_filters: EntityColumnFilters,
    on_row_activate: Option<Callback<String>>,
    texts: Signal<EntityTableTexts>,
    empty_row_range: MaybeProp<String>,
    show_reset_actions: bool,
    page_reset_key: Option<Signal<String>>,
    viewport_fit: Option<EntityTableViewportFit>,
    toolbar_actions: Option<ChildrenFn>,
    toolbar_leading: Option<ChildrenFn>,
    toolbar_trailing: Option<ChildrenFn>,
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
            empty_row_range: MaybeProp::default(),
            show_reset_actions: false,
            page_reset_key: None,
            viewport_fit: None,
            toolbar_actions: None,
            toolbar_leading: None,
            toolbar_trailing: None,
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

    /// Supplies the reactive localized template used when the internally
    /// owned `EntityTable` mounts with no displayed rows. The table still owns
    /// the empty-range decision and placeholder values.
    pub fn with_empty_row_range(mut self, template: impl Into<MaybeProp<String>>) -> Self {
        self.empty_row_range = template.into();
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
    /// With `fill_parent`, mount the page in a definite-height flex column.
    /// The page root and table slot then carry the remaining height budget;
    /// natural and explicit `max_height` modes do not stretch the page.
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

    /// Toolbar content rendered at the LEADING edge of the table toolbar, for
    /// an action on the current SELECTION (Office op-ps299).
    ///
    /// [`Self::with_toolbar_actions`] renders in the trailing cluster, which
    /// is right for a utility acting on the VIEW (Export, Refresh) and wrong
    /// for one acting on the ticked rows: that is the most consequential
    /// control in the row and reads first. Mirrors
    /// `EntityTable::toolbar_leading`, which is where this ends up. Like
    /// every passthrough here, the renderer receives no rows, dataset,
    /// revision, or generation from this config.
    pub fn with_toolbar_leading(
        mut self,
        render: impl Fn() -> AnyView + Send + Sync + 'static,
    ) -> Self {
        self.toolbar_leading = Some(Arc::new(render));
        self
    }

    /// Toolbar content rendered LAST in the table toolbar's trailing cluster,
    /// after the caller's actions, the framework column chooser and any reset
    /// actions -- the action row's far-right corner (Office op-fxfxo).
    /// Mirrors `EntityTable::toolbar_trailing`, which is where this ends up.
    pub fn with_toolbar_trailing(
        mut self,
        render: impl Fn() -> AnyView + Send + Sync + 'static,
    ) -> Self {
        self.toolbar_trailing = Some(Arc::new(render));
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

/// Framework-owned utility-row furniture for a snapshot table page
/// (`ldui-nj3q`): the localized visible/total result count, one Reset, and
/// one explicit Save as Default.
///
/// Supplying it opts [`SnapshotTablePage`]'s `filters` slot into the
/// framework [`FilterBar`], which then owns the row's layout, the result
/// string, and both actions. Omitting it leaves the `filters` slot exactly
/// as it renders today -- the consumer's content, unwrapped.
///
/// Deliberately carries no counts. Visible and total are the values the page
/// already owns -- the identity-bound [`LocalResultSummary`] minted by
/// `state`, and the authoritative displayed snapshot -- so a consumer cannot
/// pair a count with the wrong generation by supplying one here. Like the
/// selector and table configs, no field's type can carry rows, dataset
/// identity, revision, count, or generation.
pub struct SnapshotFilterActionsConfig {
    texts: Signal<FilterBarTexts>,
    on_reset: Option<Callback<()>>,
    default_save: Option<SnapshotDefaultSave>,
    active_filters: Option<Signal<Vec<ActiveFilterChip>>>,
    on_remove: Option<Callback<String>>,
    show_result_count: bool,
    class: &'static str,
}

impl Default for SnapshotFilterActionsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotFilterActionsConfig {
    /// Creates a utility row that renders only the framework result count.
    ///
    /// Reset and Save as Default are added explicitly, so a page that offers
    /// neither still gets a single framework-owned place for the count.
    pub fn new() -> Self {
        Self {
            texts: Signal::stored(FilterBarTexts::default()),
            on_reset: None,
            default_save: None,
            active_filters: None,
            on_remove: None,
            show_result_count: true,
            class: "",
        }
    }

    /// Supplies reactive framework-owned copy, including the `{visible}` /
    /// `{total}` result template and both action labels.
    pub fn with_texts(mut self, texts: impl Into<Signal<FilterBarTexts>>) -> Self {
        self.texts = texts.into();
        self
    }

    /// Adds the one canonical Reset action. The consumer owns what resetting
    /// means; the framework owns the label, placement, and enablement.
    pub const fn on_reset(mut self, callback: Callback<()>) -> Self {
        self.on_reset = Some(callback);
        self
    }

    /// Adds the explicit, persistence-neutral Save as Default action. Its
    /// pending / saved / conflict / failure copy renders in the same row's
    /// live region.
    pub fn with_default_save(mut self, binding: SnapshotDefaultSave) -> Self {
        self.default_save = Some(binding);
        self
    }

    /// Adds controlled active-filter chips. Their presence is also what lets
    /// Reset report "nothing to reset" by disabling itself; without them
    /// Reset stays enabled.
    pub fn with_active_filters(
        mut self,
        chips: impl Into<Signal<Vec<ActiveFilterChip>>>,
        on_remove: Callback<String>,
    ) -> Self {
        self.active_filters = Some(chips.into());
        self.on_remove = Some(on_remove);
        self
    }

    /// Hides the framework result count while keeping the actions. Rarely
    /// right: the count is the reason the row exists.
    pub const fn show_result_count(mut self, show: bool) -> Self {
        self.show_result_count = show;
        self
    }

    /// Adds outer filter-row classes.
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
    /// Optional column rendered beside the table, INSIDE this page's
    /// section (Office op-trula).
    ///
    /// Exists because a page whose section heading lives inside
    /// `SnapshotTablePage` cannot compose a table-plus-panel row itself:
    /// the panel could only sit outside the component, below the whole
    /// section. `None` -- the default -- renders exactly as before.
    ///
    /// Side by side from `lg` up; stacked below it, where a fixed column
    /// would leave the table no room. From `lg` the column takes its
    /// CONTENT's width -- a slot must not impose one, or a fixed-width panel
    /// such as `PersonPicker` (360px) overflows it, and a panel collapsed to
    /// its rail leaves a dead column (4iiz-Office measured 40px and 276px).
    /// It scrolls internally rather than stretching the row, so a tall panel
    /// cannot push a `fill_parent` table out of its height budget.
    ///
    /// Until the table mounts (loading, a load error, an empty or filtered-out
    /// dataset, an access panel) a placeholder sized like the table holds its
    /// place, so the column is where it will stay from the first frame
    /// instead of painting at the left edge and jumping right (ldui-pt0x).
    #[prop(optional)]
    side_panel: Option<Children>,
    /// Controlled local-filter utility content.
    filters: Children,
    /// Optional framework-owned result count, Reset, and Save as Default
    /// (`ldui-nj3q`). When supplied, `filters` is composed inside the
    /// framework [`FilterBar`]; when absent, the `filters` slot renders
    /// exactly as before.
    #[prop(optional)]
    filter_actions: Option<SnapshotFilterActionsConfig>,
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
        compact,
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

    // `ldui-nj3q`: the opt-in utility row. The counts are read here, from
    // the same identity-bound proof the render decision uses, rather than
    // accepted from the caller -- `SnapshotFilterActionsConfig` has no field
    // that could carry one. `FilterResultSummary` is `Copy + Send + Sync`,
    // so the two `LocalStorage` sources are bridged through a plain signal
    // exactly as `generation_marker`/`loading`/`load_error` already are.
    let filters_slot = filter_actions.map(|config| {
        let SnapshotFilterActionsConfig {
            texts,
            on_reset,
            default_save,
            active_filters,
            on_remove,
            show_result_count,
            class,
        } = config;
        let result_summary = RwSignal::new(FilterResultSummary::new(0, 0));
        Effect::new(move |_| {
            let total = authoritative_rows.with(|rows| rows.len());
            let visible = effective_local_result
                .get()
                .map_or(total, |summary| summary.filtered_count());
            result_summary.set(FilterResultSummary::new(visible, total));
        });
        (
            texts,
            on_reset,
            default_save,
            active_filters,
            on_remove,
            show_result_count.then_some(Signal::from(result_summary)),
            class,
        )
    });

    let dataset_id = format!("{contract_id}-dataset");
    let kpis_id = format!("{contract_id}-kpis");
    let filters_id = format!("{contract_id}-filters");
    let feedback_id = format!("{contract_id}-feedback");
    let table_id = format!("{contract_id}-table");
    // The header, KPI and filter slots cost nothing when they render nothing:
    // no element at all (ldui-75xy) or only empty wrappers (ldui-xhrw).
    let empty_slot_class = format!("{EMPTY_SLOT_HIDDEN} {EMPTY_WRAPPERS_OUT_OF_FLOW}");
    let (root_class, table_slot_class) =
        entity_table.with_value(|config| snapshot_page_layout(config.viewport_fit.as_ref()));
    // A panel turns the table slot into a row; without one the slot keeps
    // its exact class, so the fill_parent height chain (op-at7nn) is
    // untouched for every page that does not opt in. With one, the row
    // still carries `min-h-0 flex-1` in fill mode, and EntityTable's own
    // root is `w-full min-w-0`, so it shrinks beside the panel instead of
    // overflowing it.
    let has_side_panel = side_panel.is_some();
    let table_slot_class = snapshot_table_slot_class(table_slot_class, has_side_panel);
    // ldui-pt0x: both halves of the table slot's `Show` read
    // `snapshot_table_slot_lead`, so "mount the table" and "hold its place in
    // front of a side panel" come from one decision.
    let table_mounted = move || {
        state.with(|state| {
            let summary = effective_local_result.get();
            let decision = state.view(summary.as_ref()).render_decision();
            snapshot_table_slot_lead(decision, has_side_panel) == SnapshotTableSlotLead::Table
        })
    };
    let table_fallback = move || {
        state.with(|state| {
            let summary = effective_local_result.get();
            let view = state.view(summary.as_ref());
            let decision = view.render_decision();
            let panel = decision.panel().map(|kind| {
                view! {
                    <PageStatePanel
                        kind=kind
                        texts=panel_texts
                        nostrip:on_retry=on_retry
                        detail=Signal::stored(view.load_error().map(ToString::to_string))
                    />
                }
            });
            match snapshot_table_slot_lead(decision, has_side_panel) {
                SnapshotTableSlotLead::ReservedPanel => view! {
                    <div
                        class=SNAPSHOT_TABLE_PLACEHOLDER_CLASS
                        data-snapshot-page-table-placeholder="true"
                    >
                        {panel}
                    </div>
                }
                .into_any(),
                SnapshotTableSlotLead::Panel | SnapshotTableSlotLead::Table => panel.into_any(),
            }
        })
    };

    view! {
        <section
            id=contract_id
            class=format!("{root_class} {class}")
            data-snapshot-table-page="true"
            data-ld-pattern="SnapshotTable"
            data-snapshot-generation=move || generation_marker.get()
            data-snapshot-phase=move || state.with(|state| format!("{:?}", state.view(None).phase()))
        >
            <div class=empty_slot_class.clone() data-snapshot-page-slot="header">{header()}</div>
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
                    compact=compact
                    nostrip:on_retry=on_retry
                />
            </div>
            {kpis.map(|kpis| view! {
                <div id=kpis_id class=empty_slot_class.clone() data-snapshot-page-slot="kpis">{kpis()}</div>
            })}
            <div
                id=filters_id
                class=format!("{empty_slot_class} {COLLAPSED_FILTER_BAR_OUT_OF_FLOW}")
                data-snapshot-page-slot="filters"
            >
                {match filters_slot {
                    None => filters().into_any(),
                    Some((
                        texts,
                        on_reset,
                        default_save,
                        active_filters,
                        on_remove,
                        result,
                        class,
                    )) => view! {
                        <FilterBar
                            texts=texts
                            nostrip:on_reset=on_reset
                            nostrip:default_save=default_save
                            nostrip:active_filters=active_filters
                            nostrip:on_remove=on_remove
                            nostrip:result=result
                            class=class
                        >
                            {filters()}
                        </FilterBar>
                    }
                    .into_any(),
                }}
            </div>
            // ldui-75xy: the feedback slot shows only keyed action outcomes.
            // With none it collapses to `sr-only` -- absolutely positioned, so
            // it is not a flex item and costs no gap -- and NOT `hidden`:
            // ActionFeedback's `aria-live` announcement region lives in here,
            // and a live region that is display:none until the moment its
            // first message arrives is routinely not announced. The
            // retained "Replacing"/"retained error" notice no longer lives
            // here: in the flow above a viewport-fitted table it took a row
            // from the fit every time a dataset switched, so consumers
            // reserved its height permanently. It is an overlay on the table
            // slot now (see below).
            <div
                id=feedback_id
                class="space-y-2"
                class:sr-only=move || action_model.with(|model| model.is_empty())
                data-snapshot-feedback-idle=move || action_model.with(|model| model.is_empty()).then_some("true")
                data-snapshot-page-slot="feedback"
            >
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
                class=format!("relative {}", table_slot_class.unwrap_or_default())
                data-snapshot-page-slot="table"
                data-snapshot-side-panel=has_side_panel.then_some("true")
                data-snapshot-generation=move || generation_marker.get()
            >
                // ldui-75xy: the retained notice as an OVERLAY pinned to the
                // top of the table slot -- out of flow, so it never moves the
                // table or changes how many rows a viewport-fitted table fits.
                // The wrapper passes pointer events through; the panel itself
                // takes them, so a retained error's Retry still works.
                {move || state.with(|state| {
                    let summary = effective_local_result.get();
                    let decision = state.view(summary.as_ref()).render_decision();
                    decision.retained_notice().then(|| {
                        let kind = decision.panel().expect("retained notice has panel");
                        view! {
                            <div
                                class="pointer-events-none absolute inset-x-0 top-0 z-20 flex justify-center p-2"
                                data-snapshot-retained-notice="true"
                            >
                                <div class="pointer-events-auto w-full max-w-xl rounded-box bg-base-100 shadow-lg">
                                    <PageStatePanel
                                        kind=kind
                                        texts=panel_texts
                                        nostrip:on_retry=on_retry
                                        detail=load_error
                                    />
                                </div>
                            </div>
                        }
                    })
                })}
                // ldui-pt0x: the fallback holds the table's place in front of
                // a side panel (see `snapshot_table_slot_lead`).
                <Show when=table_mounted fallback=table_fallback>
                    {entity_table.with_value(|config| {
                        // `ChildrenFn` is reusable (`Arc<dyn Fn() -> AnyView>`)
                        // because this whole block can re-run whenever `Show`
                        // remounts the table, but each `EntityTable` toolbar slot
                        // wants one single-shot `Children` per instance -- so a
                        // fresh box is built from each stored renderer every time.
                        let toolbar_leading = config
                            .toolbar_leading
                            .clone()
                            .map(|render| Box::new(move || render()) as Children);
                        let toolbar_actions = config
                            .toolbar_actions
                            .clone()
                            .map(|render| Box::new(move || render()) as Children);
                        let toolbar_trailing = config
                            .toolbar_trailing
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
                                empty_row_range=config.empty_row_range
                                page_size_control_id=format!("{contract_id}-rows-per-page")
                                show_reset_actions=config.show_reset_actions
                                nostrip:toolbar_leading=toolbar_leading
                                nostrip:toolbar_actions=toolbar_actions
                                nostrip:toolbar_trailing=toolbar_trailing
                                nostrip:on_display_projection=config.on_display_projection
                                projection_action_columns=config.projection_action_columns
                                column_chooser_trigger=config.column_chooser_trigger
                                zebra=config.zebra
                                class=config.class
                            />
                        }
                    })}
                </Show>
                {side_panel.map(|panel| view! {
                    <aside
                        class="w-full min-w-0 shrink-0 lg:w-auto lg:min-h-0 lg:overflow-auto"
                        data-snapshot-page-slot="side-panel"
                    >
                        {panel()}
                    </aside>
                })}
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{EntityTablePreferenceOwnership, EntityTablePreferences};
    use crate::patterns::{
        FilterSchema, PageStatePanelKind, SnapshotAccess, SnapshotData, SnapshotDefaultSaveState,
        SnapshotTransitionDisposition, SnapshotViewDefaults, filter_result_summary,
    };

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
        assert!(table.empty_row_range.get_untracked().is_none());
        assert!(table.page_reset_key.is_none());
        assert!(table.viewport_fit.is_none());
        assert!(table.toolbar_actions.is_none());
        assert!(table.toolbar_leading.is_none());
        assert!(table.toolbar_trailing.is_none());
        assert!(table.on_display_projection.is_none());
        assert_eq!(
            table.projection_action_columns,
            EntityTableActionColumnPolicy::Exclude
        );
        assert_eq!(
            table.column_chooser_trigger.get_untracked(),
            EntityColumnChooserTrigger::Icon,
            "ldui-q85o: the gear is the default chooser trigger"
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

    #[test]
    fn empty_row_range_builder_keeps_the_callers_reactive_localized_copy() {
        let template = RwSignal::new("Activity rows 0 of {total}".to_owned());
        let table = base_table_config().with_empty_row_range(template);

        assert_eq!(
            table.empty_row_range.get_untracked().as_deref(),
            Some("Activity rows 0 of {total}")
        );

        template.set("Filas de actividad: 0 de {total}".to_owned());
        assert_eq!(
            table.empty_row_range.get_untracked().as_deref(),
            Some("Filas de actividad: 0 de {total}")
        );
    }

    /// `ldui-nj3q`: the utility-row config defaults to the count alone, and
    /// its type has no field that could carry a count, rows, dataset
    /// identity, revision, or generation -- the page reads all of those from
    /// `state`.
    #[test]
    fn filter_actions_config_defaults_to_the_count_alone() {
        let config = SnapshotFilterActionsConfig::new();
        assert!(config.show_result_count);
        assert!(config.on_reset.is_none());
        assert!(config.default_save.is_none());
        assert!(config.active_filters.is_none());
        assert!(config.on_remove.is_none());
        assert_eq!(config.class, "");
        assert_eq!(
            config.texts.get_untracked().result_count,
            "{visible} of {total} results"
        );
        assert_eq!(config.texts.get_untracked().reset, "Reset");
        assert_eq!(config.texts.get_untracked().save_default, "Save as Default");
    }

    /// `ldui-nj3q`: every builder lands on its private field, including the
    /// localized copy a Spanish consumer supplies.
    #[test]
    fn filter_actions_builders_forward_to_typed_fields() {
        let resets = RwSignal::new(0_u32);
        let removed = RwSignal::new(String::new());
        let saves = RwSignal::new(0_u32);
        let preferences = EntityTablePreferences::new(1);
        let schema = FilterSchema::<()>::new("office", &["status"]);
        let defaults = schema
            .project_defaults([("status", serde_json::json!("urgent"))], preferences)
            .expect("schema projects the fixture defaults");
        let spanish = FilterBarTexts {
            result_count: "{visible} de {total} resultados".to_owned(),
            reset: "Restablecer".to_owned(),
            save_default: "Guardar como predeterminado".to_owned(),
            clean_reason: "Los valores predeterminados ya están guardados".to_owned(),
            ..FilterBarTexts::default()
        };

        let config = SnapshotFilterActionsConfig::new()
            .with_texts(Signal::stored(spanish))
            .on_reset(Callback::new(move |()| resets.update(|count| *count += 1)))
            .with_default_save(SnapshotDefaultSave::new(
                Signal::stored(defaults),
                Signal::stored(SnapshotDefaultSaveState::Dirty),
                Callback::new(move |_: SnapshotViewDefaults| saves.update(|count| *count += 1)),
            ))
            .with_active_filters(
                Signal::stored(vec![ActiveFilterChip::new("status", "Status", "Urgent")]),
                Callback::new(move |key: String| removed.set(key)),
            )
            .show_result_count(false)
            .with_class("mt-2");

        assert!(!config.show_result_count);
        assert_eq!(config.class, "mt-2");
        assert_eq!(
            config.texts.get_untracked().result_count,
            "{visible} de {total} resultados"
        );
        assert_eq!(config.texts.get_untracked().reset, "Restablecer");
        assert_eq!(
            config.texts.get_untracked().save_default,
            "Guardar como predeterminado"
        );

        config.on_reset.expect("on_reset was set").run(());
        assert_eq!(resets.get_untracked(), 1);

        config
            .on_remove
            .expect("on_remove was set")
            .run("status".to_owned());
        assert_eq!(removed.get_untracked(), "status");
        assert_eq!(
            config
                .active_filters
                .expect("active_filters was set")
                .get_untracked()
                .len(),
            1
        );

        let save = config.default_save.expect("default_save was set");
        assert_eq!(save.state(), SnapshotDefaultSaveState::Dirty);
        // Reading the projected payload must never perform persistence.
        let payload = save.defaults();
        assert_eq!(saves.get_untracked(), 0);
        assert_eq!(
            payload.filters().get("status"),
            Some(&serde_json::json!("urgent"))
        );
    }

    /// `ldui-nj3q` negative control: the localized result string comes from
    /// `FilterBar`'s own template, so one page's copy cannot drift from
    /// another's.
    #[test]
    fn filter_actions_result_string_is_owned_by_filter_bar() {
        let english = FilterBarTexts::default();
        assert_eq!(
            filter_result_summary(FilterResultSummary::new(3, 3), &english),
            "3 of 3 results"
        );
        let spanish = FilterBarTexts {
            result_count: "{visible} de {total} resultados".to_owned(),
            ..FilterBarTexts::default()
        };
        assert_eq!(
            filter_result_summary(FilterResultSummary::new(1, 3), &spanish),
            "1 de 3 resultados"
        );
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
            // Placeholder content only: the test checks the forwarding, and a
            // <span> keeps no_bare_library_buttons' scan clean.
            .with_toolbar_leading(|| view! { <span>"Assign"</span> }.into_any())
            .with_toolbar_trailing(|| view! { <span>"Assign Selected"</span> }.into_any())
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
        // Office op-ps299 / op-fxfxo: the leading and trailing passthroughs
        // land on their own fields and reach the table's own slots.
        // Deliberate break: drop either `nostrip:` forward and the source
        // assertion fails.
        assert!(table.toolbar_leading.is_some());
        assert!(table.toolbar_trailing.is_some());
        let production = include_str!("snapshot_table_page.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the production half");
        for forward in [
            concat!("nostrip:toolbar_", "leading=toolbar_leading"),
            concat!("nostrip:toolbar_", "trailing=toolbar_trailing"),
        ] {
            assert!(
                production.contains(forward),
                "the toolbar slot is forwarded to the EntityTable: {forward}"
            );
        }
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

    #[test]
    fn fill_parent_carries_the_budget_through_both_root_and_slot() {
        let (root, slot) = snapshot_page_layout(Some(&EntityTableViewportFit::fill_parent()));
        for classes in [root, slot.expect("filling table slot")] {
            for required in ["flex", "flex-col", "min-h-0", "flex-1"] {
                assert!(
                    classes.split_whitespace().any(|class| class == required),
                    "missing {required} in {classes}"
                );
            }
        }
    }

    #[test]
    fn a_side_panel_leaves_an_unset_slot_unchanged_and_keeps_the_fill_chain() {
        // Unset: byte-identical to what snapshot_page_layout chose, in both modes.
        for fill in [None, Some("flex min-h-0 flex-1 flex-col")] {
            assert_eq!(snapshot_table_slot_class(fill, false), fill);
        }
        // Set, fill mode: the row must keep the height-chain classes, or the
        // table inside collapses silently.
        let row = snapshot_table_slot_class(Some("flex min-h-0 flex-1 flex-col"), true)
            .expect("a panel always yields a class");
        for required in ["flex", "min-h-0", "flex-1", "lg:flex-row"] {
            assert!(
                row.split_ascii_whitespace().any(|class| class == required),
                "fill-mode side-panel row lost `{required}`: {row}"
            );
        }
        // Set, natural mode: a row, but it must NOT stretch the page.
        let natural = snapshot_table_slot_class(None, true).expect("a class");
        assert!(natural.contains("lg:flex-row"));
        assert!(
            !natural
                .split_ascii_whitespace()
                .any(|class| class == "flex-1"),
            "natural mode must not stretch: {natural}"
        );
    }

    #[test]
    fn natural_and_explicit_height_do_not_stretch_the_page_or_slot() {
        for fit in [None, Some(EntityTableViewportFit::max_height("24rem"))] {
            let (root, slot) = snapshot_page_layout(fit.as_ref());
            assert_eq!(root, "flex w-full min-w-0 flex-col gap-4");
            assert!(slot.is_none());
        }
    }

    /// ldui-pt0x (Office op-zwrly): EVERY state in which the table is not
    /// mounted holds the table's place in front of a side panel, so the
    /// aside never paints beside a content-sized status sentence and then
    /// jumps right; a mounted table needs no placeholder; and without a side
    /// panel the replacement panel stays bare, exactly as before. The
    /// decisions come from real controller states, not hand-built ones, and
    /// the render path is checked to read this same function.
    #[test]
    fn a_side_panel_row_holds_the_tables_place_in_every_unmounted_state() {
        type State = SnapshotTableState<Row, &'static str, &'static str, (), &'static str>;

        fn displaying(rows: Vec<Row>) -> State {
            let mut state = State::new();
            let request = state.start_request("mx").expect("first request");
            let count = rows.len();
            let data = SnapshotData::new("mx", Rc::new(rows), "r1", count, None)
                .expect("complete snapshot");
            assert_eq!(
                state.complete(request, data),
                SnapshotTransitionDisposition::Applied
            );
            state
        }

        fn decide(state: &State, filtered: Option<usize>) -> SnapshotRenderDecision {
            let summary = filtered.and_then(|count| state.local_result_summary(count));
            state.view(summary.as_ref()).render_decision()
        }

        let never_loaded = State::new();
        let mut initial_loading = State::new();
        let _pending = initial_loading.start_request("mx").expect("first request");
        let mut initial_error = State::new();
        let failing = initial_error.start_request("mx").expect("first request");
        assert_eq!(
            initial_error.fail(failing, "down"),
            SnapshotTransitionDisposition::Applied
        );
        let loaded = displaying(vec![Row, Row]);
        let empty = displaying(Vec::new());
        let mut expired = displaying(vec![Row]);
        expired.replace_access(SnapshotAccess::Expired);
        let mut forbidden = displaying(vec![Row]);
        forbidden.replace_access(SnapshotAccess::Forbidden);
        let mut replacing = displaying(vec![Row]);
        let _replacement = replacing.start_request("us").expect("replacement request");
        let mut retained_error = displaying(vec![Row]);
        let replacement = retained_error
            .start_request("us")
            .expect("replacement request");
        assert_eq!(
            retained_error.fail(replacement, "down"),
            SnapshotTransitionDisposition::Applied
        );

        let unmounted = [
            (decide(&never_loaded, None), PageStatePanelKind::NeverLoaded),
            (
                decide(&initial_loading, None),
                PageStatePanelKind::InitialLoading,
            ),
            (
                decide(&initial_error, None),
                PageStatePanelKind::InitialError,
            ),
            (decide(&empty, Some(0)), PageStatePanelKind::EmptyDataset),
            (decide(&loaded, Some(0)), PageStatePanelKind::NoLocalResults),
            (decide(&expired, None), PageStatePanelKind::Expired),
            (decide(&forbidden, None), PageStatePanelKind::Forbidden),
        ];
        for (decision, kind) in unmounted {
            assert_eq!(decision.panel(), Some(kind), "the fixture reaches {kind:?}");
            assert!(!decision.table_mounted(), "{kind:?} replaces the table");
            assert_eq!(
                snapshot_table_slot_lead(decision, true),
                SnapshotTableSlotLead::ReservedPanel,
                "{kind:?} with a side panel must hold the table's place, or the aside \
                 paints beside the status sentence and jumps right when the table mounts"
            );
            assert_eq!(
                snapshot_table_slot_lead(decision, false),
                SnapshotTableSlotLead::Panel,
                "{kind:?} without a side panel renders the bare panel, exactly as before"
            );
        }

        // A mounted table -- whole, locally filtered, authoritatively empty
        // with no local proof, or under a retained notice -- needs no
        // placeholder, with or without a side panel.
        for decision in [
            decide(&loaded, None),
            decide(&loaded, Some(1)),
            decide(&empty, None),
            decide(&replacing, None),
            decide(&retained_error, None),
        ] {
            assert!(decision.table_mounted(), "{decision:?}");
            for has_side_panel in [true, false] {
                assert_eq!(
                    snapshot_table_slot_lead(decision, has_side_panel),
                    SnapshotTableSlotLead::Table,
                    "{decision:?}, side panel {has_side_panel}"
                );
            }
        }

        // Both halves of the `Show` read this decision. Deliberate break:
        // restore the old inline `when`/`fallback` and this fails.
        let production = include_str!("snapshot_table_page.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the production half");
        assert_eq!(
            production
                .matches(concat!(
                    "snapshot_table_slot_lead(",
                    "decision, has_side_panel)"
                ))
                .count(),
            2,
            "the table slot's `when` and `fallback` must both read snapshot_table_slot_lead"
        );
        assert!(production.contains(concat!(
            "<Show when=table_mounted ",
            "fallback=table_fallback>"
        )));
    }

    /// ldui-pt0x: the placeholder takes the table's flex basis -- exactly
    /// the sizing of `EntityTable`'s root -- and nothing that paints or
    /// scrolls, so reserving the place cannot move the final geometry. If
    /// either EntityTable root class stops being `w-full min-w-0`, the
    /// placeholder no longer matches the table it stands in for.
    #[test]
    fn the_table_placeholder_is_sized_like_the_entity_table_root_and_paints_nothing() {
        let placeholder: Vec<&str> = SNAPSHOT_TABLE_PLACEHOLDER_CLASS
            .split_ascii_whitespace()
            .collect();
        assert_eq!(placeholder, ["w-full", "min-w-0"]);

        let source = include_str!("../components/entity_table/component.rs");
        let start = source
            .find("let root_class = if viewport_fit_enabled")
            .expect("EntityTable chooses its root class by viewport fit");
        let choice = &source[start..];
        let choice = &choice[..choice.find("};").expect("the end of the root-class choice")];
        let roots: Vec<&str> = choice
            .split("merge_classes!(\"")
            .skip(1)
            .map(|rest| &rest[..rest.find('"').expect("a closed class literal")])
            .collect();
        assert_eq!(roots.len(), 2, "natural and viewport-fit roots: {roots:?}");
        for root in roots {
            for required in &placeholder {
                assert!(
                    root.split_ascii_whitespace()
                        .any(|class| class == *required),
                    "EntityTable root `{root}` lost `{required}`; the side-panel placeholder \
                     must keep the table's sizing or the aside moves when the table mounts"
                );
            }
        }
    }
}
