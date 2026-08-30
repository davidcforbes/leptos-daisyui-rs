//! Reactive renderer for the typed client-side table model.

use super::emphasis::{
    EntityRowEmphasis, EntityRowEmphasisClassifier, entity_row_emphasis_cell_class,
    entity_row_emphasis_for, entity_row_emphasis_row_class,
};
use super::model::{
    ENTITY_PAGE_SIZE_CHOICES, EntityColumnMove, EntityFocusRecord, EntityFocusTarget,
    SortedIndexCache, emit_normalized_preference_change,
    entity_table_display_projection_from_indices, focus_target, move_column, next_sort,
    next_sort_additive, normalize_preferences, ordered_columns, page_after_dataset_change,
    page_after_row_delta, reset_columns, reset_sort, set_preferred_width, sorted_indices,
    toggle_hidden_column,
};
use super::selection::{
    EntityTableSelection, entity_row_aria_selected, entity_row_is_selected,
    entity_selection_proposal,
};
use super::storage::{load_preferences, save_preferences};
use super::types::{
    EntityCellPresentation, EntityColumn, EntityColumnAlignment, EntityColumnChooserTrigger,
    EntityColumnFilter, EntityColumnFilterPlacement, EntityColumnFilters, EntityColumns,
    EntityCompactRow, EntityRowKey, EntityRowRenderer, EntitySort, EntitySortDirection,
    EntityTableActionColumnPolicy, EntityTableDisplayProjection, EntityTablePreferenceOwnership,
    EntityTablePreferencePersistence, EntityTablePreferences, EntityTableTexts,
    EntityTableViewportFit, EntityTextOverflow, entity_alignment_class,
    entity_compact_alignment_class, entity_header_justify_class, entity_text_overflow_style,
    normalize_entity_secondary_text,
};
use crate::components::badge::{Badge, BadgeSize};
use crate::components::button::Button;
use crate::components::data_table::{
    FALLBACK_HEADER_HEIGHT, FALLBACK_ROW_HEIGHT, MAX_COLUMN_WIDTH, PageSlot, StableColumnTrack,
    StableTableColGroup, auto_page_size_for_height, clamp_page, effective_min_width,
    keyboard_resized_width, page_bounds, page_count, page_window, row_range, stable_column_width,
    stable_table_content_style,
};
use crate::components::icon::{Icon, IconSize};
use crate::components::menu::{Menu, MenuCheckItem};
use crate::components::pagination::Pagination;
use crate::components::select::Select;
use crate::merge_classes;
use leptos::prelude::*;
use leptos::tachys::reactive_graph::OwnedView;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use web_sys::wasm_bindgen::JsCast;

const MAX_VISIBLE_PAGES: usize = 7;
static ENTITY_CHOOSER_ID: AtomicU64 = AtomicU64::new(0);
static ENTITY_PAGE_SIZE_ID: AtomicU64 = AtomicU64::new(0);

fn next_entity_chooser_id() -> String {
    format!(
        "ldui-entity-column-chooser-{}",
        ENTITY_CHOOSER_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// A process-unique id base for one `EntityTable`'s framework-owned
/// rows-per-page `<select>`, minted only when the caller does not supply
/// `page_size_control_id`. Monotonic counter, not randomness — stable within
/// a page's lifetime and unique across every mounted `EntityTable` instance,
/// which is all `id`/`name` association needs (ldui-kl55: Office satellites
/// mount three `EntityTable`s on one Setup page).
pub(crate) fn next_entity_page_size_id() -> String {
    format!(
        "ldui-entity-page-size-{}",
        ENTITY_PAGE_SIZE_ID.fetch_add(1, Ordering::Relaxed)
    )
}

pub(super) fn effective_page_size(
    measured_rows: Option<usize>,
    configured_page_size: usize,
) -> usize {
    match measured_rows {
        Some(rows) => rows.max(1),
        None => configured_page_size.max(1),
    }
}

/// Marks one stable, repeatable row action for framework-owned focus recovery.
///
/// Wrap the actual button or link. The table will only recover focus to a
/// rendered, enabled, visible, focusable descendant carrying the same action
/// ID on a neighboring row after the focused source row is removed.
#[component]
pub fn EntityRowAction(
    /// Stable identity of this action within one row.
    #[prop(into)]
    action_id: String,
    /// The consumer-owned action control.
    children: Children,
) -> impl IntoView {
    view! {
        <span class="contents" data-entity-row-action=action_id>
            {children()}
        </span>
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResizeDrag {
    column_id: String,
    start_x: f64,
    start_width: f64,
    minimum_width: Option<u32>,
}

/// Send-safe header presentation split from an `EntityColumn<T>`'s local
/// `Rc` render/sort callbacks. Leptos's keyed `For` requires `Send` items;
/// keeping only header mechanics here lets the behavioral columns remain
/// deliberately local while sort-only preference changes preserve DOM nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct EntityHeaderDescriptor {
    id: &'static str,
    header: String,
    sortable: bool,
    resizable: bool,
    min_width: Option<u32>,
    initial_width: Option<u32>,
    alignment: EntityColumnAlignment,
    tabular_numbers: bool,
}

#[derive(Clone, Copy)]
enum PreferenceSource {
    Controlled {
        current: Signal<EntityTablePreferences>,
        on_change: Callback<EntityTablePreferences>,
    },
    Uncontrolled {
        current: RwSignal<EntityTablePreferences>,
        persistence: EntityTablePreferencePersistence,
    },
}

pub(super) enum ColumnStore<T: 'static> {
    Static(StoredValue<Vec<EntityColumn<T>>, LocalStorage>),
    Reactive(Signal<Vec<EntityColumn<T>>, LocalStorage>),
}

impl<T: 'static> Clone for ColumnStore<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for ColumnStore<T> {}

impl<T: 'static> From<StoredValue<Vec<EntityColumn<T>>, LocalStorage>> for ColumnStore<T> {
    fn from(columns: StoredValue<Vec<EntityColumn<T>>, LocalStorage>) -> Self {
        Self::Static(columns)
    }
}

impl<T: 'static> From<Signal<Vec<EntityColumn<T>>, LocalStorage>> for ColumnStore<T> {
    fn from(columns: Signal<Vec<EntityColumn<T>>, LocalStorage>) -> Self {
        Self::Reactive(columns)
    }
}

impl<T: 'static> ColumnStore<T> {
    fn with_value<R>(self, read: impl FnOnce(&Vec<EntityColumn<T>>) -> R) -> R {
        match self {
            Self::Static(columns) => columns.with_value(read),
            Self::Reactive(columns) => columns.with(read),
        }
    }

    fn get_value(self) -> Vec<EntityColumn<T>> {
        self.with_value(Clone::clone)
    }
}

enum CompactRowStore<T: 'static> {
    Default,
    Static(StoredValue<EntityRowRenderer<T>, LocalStorage>),
    Reactive(Signal<EntityRowRenderer<T>, LocalStorage>),
}

impl<T: 'static> Clone for CompactRowStore<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for CompactRowStore<T> {}

impl<T: 'static> CompactRowStore<T> {
    fn new(renderer: EntityCompactRow<T>) -> Self {
        match renderer {
            EntityCompactRow::Default => Self::Default,
            EntityCompactRow::Static(renderer) => Self::Static(StoredValue::new_local(renderer)),
            EntityCompactRow::Reactive(renderer) => Self::Reactive(renderer),
        }
    }

    fn get_value(self) -> Option<EntityRowRenderer<T>> {
        match self {
            Self::Default => None,
            Self::Static(renderer) => Some(renderer.get_value()),
            Self::Reactive(renderer) => Some(renderer.get()),
        }
    }
}

fn column_filter_signal(
    filters: EntityColumnFilters,
) -> Signal<Vec<EntityColumnFilter>, LocalStorage> {
    match filters {
        EntityColumnFilters::None => RwSignal::new_local(Vec::new()).into(),
        EntityColumnFilters::Static(filters) => RwSignal::new_local(filters).into(),
        EntityColumnFilters::Reactive(filters) => filters,
    }
}

fn compact_filter_layout_signal() -> RwSignal<bool> {
    const QUERY: &str = "(max-width: 1023.98px)";

    let media = web_sys::window()
        .and_then(|window| window.match_media(QUERY).ok())
        .flatten();
    let compact = RwSignal::new(media.as_ref().is_some_and(web_sys::MediaQueryList::matches));

    if let Some(media) = media {
        let observed_media = media.clone();
        let listener =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
                compact.set(observed_media.matches())
            }) as Box<dyn FnMut(web_sys::Event)>);
        let _ = media.add_event_listener_with_callback("change", listener.as_ref().unchecked_ref());
        let guard = send_wrapper::SendWrapper::new((media, listener));
        on_cleanup(move || {
            let (media, listener) = guard.take();
            let _ = media
                .remove_event_listener_with_callback("change", listener.as_ref().unchecked_ref());
            drop(listener);
        });
    }

    compact
}

pub(super) struct PreferenceState<T: 'static> {
    source: PreferenceSource,
    columns: ColumnStore<T>,
    schema_version: u16,
}

impl<T: 'static> Clone for PreferenceState<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for PreferenceState<T> {}

impl<T: 'static> PreferenceState<T> {
    pub(super) fn new(
        ownership: EntityTablePreferenceOwnership,
        columns: impl Into<ColumnStore<T>>,
        schema_version: u16,
    ) -> Self {
        let columns = columns.into();
        let source = match ownership {
            EntityTablePreferenceOwnership::Controlled { current, on_change } => {
                PreferenceSource::Controlled { current, on_change }
            }
            EntityTablePreferenceOwnership::Uncontrolled { persistence } => {
                let initial = columns
                    .with_value(|columns| load_preferences(persistence, schema_version, columns));
                PreferenceSource::Uncontrolled {
                    current: RwSignal::new(initial),
                    persistence,
                }
            }
        };
        Self {
            source,
            columns,
            schema_version,
        }
    }

    fn normalize_after_columns_change(self) {
        let current = match self.source {
            PreferenceSource::Controlled { current, .. } => current.get_untracked(),
            PreferenceSource::Uncontrolled { current, .. } => current.get_untracked(),
        };
        let normalized = self
            .columns
            .with_value(|columns| normalize_preferences(&current, self.schema_version, columns));
        if normalized == current {
            return;
        }
        match self.source {
            PreferenceSource::Controlled { on_change, .. } => on_change.run(normalized),
            PreferenceSource::Uncontrolled { current, .. } => current.set(normalized),
        }
    }

    pub(super) fn get(self) -> EntityTablePreferences {
        let current = match self.source {
            PreferenceSource::Controlled { current, .. } => current.get(),
            PreferenceSource::Uncontrolled { current, .. } => current.get(),
        };
        self.columns
            .with_value(|columns| normalize_preferences(&current, self.schema_version, columns))
    }

    fn get_untracked(self) -> EntityTablePreferences {
        let current = match self.source {
            PreferenceSource::Controlled { current, .. } => current.get_untracked(),
            PreferenceSource::Uncontrolled { current, .. } => current.get_untracked(),
        };
        self.columns
            .with_value(|columns| normalize_preferences(&current, self.schema_version, columns))
    }

    fn with_untracked<R>(self, read: impl FnOnce(&EntityTablePreferences) -> R) -> R {
        read(&self.get_untracked())
    }

    fn with<R>(self, read: impl FnOnce(&EntityTablePreferences) -> R) -> R {
        read(&self.get())
    }

    fn rendered_widths(self) -> BTreeMap<String, u32> {
        let current = self.get_untracked();
        self.columns
            .with_value(|columns| rendered_column_widths(&current, columns))
    }

    pub(super) fn update_and_rendered_widths(
        self,
        update: impl FnOnce(&mut EntityTablePreferences),
    ) -> BTreeMap<String, u32> {
        self.update(update);
        self.rendered_widths()
    }

    pub(super) fn update(
        self,
        update: impl FnOnce(&mut EntityTablePreferences),
    ) -> EntityTablePreferences {
        let current = self.get_untracked();
        self.columns.with_value(|columns| {
            emit_normalized_preference_change(
                &current,
                self.schema_version,
                columns,
                update,
                |replacement| match self.source {
                    PreferenceSource::Controlled { on_change, .. } => on_change.run(replacement),
                    PreferenceSource::Uncontrolled { current, .. } => current.set(replacement),
                },
            )
        })
    }
}

pub(super) struct DatasetTransitionController<T: 'static> {
    current_page: RwSignal<usize>,
    preferences: PreferenceState<T>,
}

impl<T: 'static> Clone for DatasetTransitionController<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for DatasetTransitionController<T> {}

impl<T: 'static> DatasetTransitionController<T> {
    pub(super) const fn new(
        current_page: RwSignal<usize>,
        preferences: PreferenceState<T>,
    ) -> Self {
        Self {
            current_page,
            preferences,
        }
    }

    pub(super) fn apply(self, previous_dataset: String, next_dataset: String) {
        let supplied_preferences = self.preferences.get_untracked();
        let next_page = page_after_dataset_change(
            self.current_page.get_untracked(),
            previous_dataset,
            next_dataset,
        );
        self.current_page.set(next_page);
        debug_assert_eq!(
            self.preferences.get_untracked(),
            supplied_preferences,
            "dataset changes must preserve supplied EntityTable preferences"
        );
    }
}

pub(super) fn apply_page_size_change<T: 'static>(
    preferences: PreferenceState<T>,
    current_page: RwSignal<usize>,
    requested_value: &str,
    reassert_live_value: impl FnOnce(String),
) {
    if let Ok(page_size) = requested_value.parse::<usize>()
        && ENTITY_PAGE_SIZE_CHOICES.contains(&page_size)
    {
        preferences.update(|preferences| preferences.page_size = page_size);
        current_page.set(0);
    }

    let supplied_value =
        preferences.with_untracked(|preferences| preferences.page_size.to_string());
    reassert_live_value(supplied_value);
}

pub(super) fn resolve_preference_ownership(
    explicit: Option<EntityTablePreferenceOwnership>,
    legacy_storage_key: Option<&'static str>,
) -> EntityTablePreferenceOwnership {
    match (explicit, legacy_storage_key) {
        (Some(_), Some(_)) => {
            panic!("EntityTable configuration cannot combine preference_ownership with storage_key")
        }
        (Some(ownership), None) => ownership,
        (None, Some(storage_key)) => EntityTablePreferenceOwnership::Uncontrolled {
            persistence: EntityTablePreferencePersistence::LegacyLocalStorage { storage_key },
        },
        (None, None) => EntityTablePreferenceOwnership::Uncontrolled {
            persistence: EntityTablePreferencePersistence::Disabled,
        },
    }
}

/// A typed, client-side table for complete dataset snapshots.
///
/// Ordering is represented as an index permutation, so source data and row
/// identity are never mutated. Only rows on the current page are cloned for
/// rendering. Wide and compact layouts share the same `<tr>` nodes, preventing
/// hidden duplicate pages in the DOM. Wide tables use stable declared tracks,
/// a semantic dark-blue header, and a faint full-cell grid. Sorting updates the
/// body order and sort metadata without replacing header nodes or moving the
/// table shell; a non-resizable utility column absorbs spare full-width space.
#[component]
pub fn EntityTable<T>(
    /// Complete, locally filterable dataset. Use a local signal when `T` is not `Send`.
    #[prop(into)]
    data: Signal<Rc<Vec<T>>, LocalStorage>,
    /// Typed column definitions in system order.
    #[prop(into)]
    columns: EntityColumns<T>,
    /// Stable key callback used for DOM identity and row activation.
    row_key: EntityRowKey<T>,
    /// Identity of the selected dataset. Changing it resets pagination only.
    #[prop(into)]
    dataset_identity: Signal<String>,
    /// Optional view-state identity. Changing it resets pagination while
    /// preserving dataset-independent filters, sort, page size, and columns.
    /// Use this for immediate local-filter changes.
    #[prop(optional, into)]
    page_reset_key: Option<Signal<String>>,
    /// Optional framework-owned viewport-fit paging. The measured capacity is
    /// presentation state and never changes persisted table preferences.
    #[prop(optional)]
    viewport_fit: Option<EntityTableViewportFit>,
    /// Optional renderer for the single-cell compact row layout.
    #[prop(optional, into)]
    compact_row: EntityCompactRow<T>,
    /// Controlled filters aligned beneath their stable desktop columns.
    #[prop(optional, into)]
    column_filters: EntityColumnFilters,
    /// Complete authoritative source membership used only for focus recovery.
    /// When omitted, the rendered `data` snapshot is also the source snapshot.
    #[prop(optional)]
    source_data: Option<Signal<Rc<Vec<T>>, LocalStorage>>,
    /// Opaque dataset/access generation. Focus recovery never crosses a change.
    /// When omitted, the dataset identity is used as the focus scope.
    #[prop(optional, into)]
    focus_scope: Option<Signal<String>>,
    /// Optional callback that makes rows mouse- and keyboard-operable.
    #[prop(optional)]
    on_row_activate: Option<Callback<String>>,
    /// Optional controlled single-row selection, keyed by the table's
    /// mandatory `row_key`. The accepted signal drives selected styling and
    /// `aria-selected` on both the wide and compact presentations of a row
    /// (they share one `<tr>`); a plain click or keyboard Enter/Space emits
    /// one replacement proposal. Ctrl/Meta/Shift gestures neither select nor
    /// activate. Separate from `on_row_activate` -- both can be supplied
    /// together, and a plain click/Enter/Space fires both.
    ///
    /// Selection is a pure per-row key comparison, so it fails safe under
    /// every displayed-data change without special-cased handling: sorting,
    /// filtering, paging, a dataset swap, or the selected row's removal all
    /// simply stop matching any rendered row (no row paints selected, and no
    /// positional fallback can alias a different entity) until the caller
    /// supplies a key that is visible again. `EntityTable` has no per-row
    /// disabled concept, so there is no separate disabled-row fail-safe case
    /// to represent here.
    ///
    /// `aria-selected` and selected styling are gated on this prop being
    /// supplied at all, not on general row interactivity: an
    /// `on_row_activate`-only table (no `selection`) renders no
    /// `aria-selected` attribute and no selected class on any row, exactly
    /// as before this prop existed -- it has no selection concept to report,
    /// and `aria-selected="false"` on every row would wrongly claim it does.
    #[prop(optional)]
    selection: Option<EntityTableSelection>,
    /// Optional per-row semantic classification into a narrow,
    /// framework-owned [`EntityRowEmphasis`] -- `Standard`, `Summary`,
    /// `Muted`, or `Attention` -- never an unrestricted class-string hook.
    /// `EntityTable` owns every token, stroke width, and forced-colors rule
    /// a variant applies, identically in the wide and compact presentations
    /// (they share one `<tr>`); the caller owns only the classification
    /// predicate, so no per-column renderer needs to change.
    ///
    /// Presentation-only: classification never changes row keys, ordering,
    /// accessible names, action eligibility, sort values, selection, or
    /// source data. It is a pure function of the row's own content, so it
    /// automatically follows a row across sorting, filtering, and paging
    /// rather than pinning a look to a rendered position. Every variant's
    /// tokens are text/border only -- never `background-color` -- so
    /// emphasis composes with, rather than fights, the selected-row
    /// background painted independently when `selection` is also supplied,
    /// and with `zebra` striping. Omitting this prop renders identically to
    /// a table that predates it: no extra class, no
    /// `data-entity-row-emphasis` attribute on any row.
    #[prop(optional)]
    row_emphasis: Option<EntityRowEmphasisClassifier<T>>,
    /// Preference namespace appended to the framework storage prefix.
    ///
    /// This compatibility prop selects `LegacyLocalStorage` when
    /// `preference_ownership` is omitted. Supplying both is a configuration
    /// error so controlled ownership can never silently perform browser I/O.
    #[prop(optional)]
    storage_key: Option<&'static str>,
    /// Typed preference ownership. Controlled mode performs no component I/O.
    #[prop(optional)]
    preference_ownership: Option<EntityTablePreferenceOwnership>,
    /// Consumer-controlled preference schema version.
    #[prop(default = 1)]
    preference_version: u16,
    /// Localizable labels for table controls.
    #[prop(into, default = Signal::stored(EntityTableTexts::default()))]
    texts: Signal<EntityTableTexts>,
    /// Stable DOM identity for the rows-per-page select's `id` and `name`
    /// attributes. When omitted, a process-unique default is generated so
    /// every mounted `EntityTable` still gets a non-empty, mutually unique
    /// `id`/`name` (ldui-kl55) — a caller-supplied value always wins and
    /// stays stable across re-renders.
    #[prop(optional, into)]
    page_size_control_id: MaybeProp<String>,
    /// Shows separate reset-sort and reset-columns actions.
    #[prop(optional, default = false)]
    show_reset_actions: bool,
    /// Optional caller-rendered table utilities such as Export or Refresh.
    /// The table owns placement and wrapping; the caller owns all behavior.
    #[prop(optional)]
    toolbar_actions: Option<Children>,
    /// Emits an atomic read-only projection whenever displayed rows, columns,
    /// ordering, paging, or canonical text change. Callers own storage, export
    /// encoding, authorization, and download behavior.
    #[prop(optional)]
    on_display_projection: Option<Callback<EntityTableDisplayProjection>>,
    /// Explicit action-column policy for `on_display_projection` snapshots.
    #[prop(optional)]
    projection_action_columns: EntityTableActionColumnPolicy,
    /// Visible presentation of the framework-owned column-chooser trigger.
    #[prop(into, default = Signal::stored(EntityColumnChooserTrigger::Text))]
    column_chooser_trigger: Signal<EntityColumnChooserTrigger>,
    /// Enable alternating body-row striping. The opinionated default is a
    /// clean faint grid without zebra banding.
    #[prop(optional, into)]
    zebra: Signal<bool>,
    /// Additional outer-container classes.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView
where
    T: Clone + 'static,
{
    let (column_store, reactive_columns) = match columns {
        EntityColumns::Static(columns) => {
            (ColumnStore::Static(StoredValue::new_local(columns)), None)
        }
        EntityColumns::Reactive(columns) => (ColumnStore::Reactive(columns), Some(columns)),
    };
    let preference_ownership = resolve_preference_ownership(preference_ownership, storage_key);
    let preferences = PreferenceState::new(preference_ownership, column_store, preference_version);
    let initial_widths = column_store
        .with_value(|columns| rendered_column_widths(&preferences.get_untracked(), columns));
    let row_key = StoredValue::new_local(row_key);
    let row_emphasis = StoredValue::new_local(row_emphasis);
    let compact_row = CompactRowStore::new(compact_row);
    let column_filters = column_filter_signal(column_filters);
    let compact_filter_layout = compact_filter_layout_signal();
    let source_data = source_data.unwrap_or(data);
    let focus_scope = focus_scope.unwrap_or(dataset_identity);
    let viewport_fit_enabled = viewport_fit.is_some();
    let viewport_fit_height = viewport_fit
        .as_ref()
        .and_then(EntityTableViewportFit::height)
        .map(str::to_owned);
    let viewport_fit_min_rows = viewport_fit
        .as_ref()
        .map_or(5, EntityTableViewportFit::min_rows);
    let measured_page_size = RwSignal::new(Option::<usize>::None);
    let sorted_index_cache = StoredValue::new_local(SortedIndexCache::new());
    let semantic_generation = RwSignal::new(0_u64);
    let column_widths = RwSignal::new(initial_widths);
    let header_descriptors =
        RwSignal::new(column_store.with_value(|columns| {
            entity_header_descriptors(&preferences.get_untracked(), columns)
        }));
    let current_page = RwSignal::new(0_usize);
    let previous_dataset = StoredValue::new(dataset_identity.get_untracked());
    let resize_drag = RwSignal::new(Option::<ResizeDrag>::None);
    let focus_record = RwSignal::new_local(Option::<EntityFocusRecord>::None);
    let table_region = NodeRef::<leptos::html::Div>::new();
    let dataset_transition = DatasetTransitionController::new(current_page, preferences);
    let page_size_select = NodeRef::<leptos::html::Select>::new();
    let column_chooser_open = RwSignal::new(false);
    let column_chooser_trigger_ref = NodeRef::<leptos::html::Button>::new();
    let column_chooser_menu_id = next_entity_chooser_id();
    let column_chooser_controls_id = column_chooser_menu_id.clone();
    // Framework-owned default for the rows-per-page select's id/name, used
    // only when the caller omits `page_size_control_id`. Minted once per
    // mounted instance, so two or more `EntityTable`s on one page never
    // collide even without a caller-supplied override (ldui-kl55).
    let default_page_size_control_id = next_entity_page_size_id();
    let page_size_select_id: Signal<Option<String>> = Signal::derive(move || {
        Some(
            page_size_control_id
                .get()
                .unwrap_or_else(|| default_page_size_control_id.clone()),
        )
    });
    let configured_page_size =
        Signal::derive(move || preferences.with(|preferences| preferences.page_size.max(1)));
    let page_capacity = Signal::derive(move || {
        effective_page_size(
            viewport_fit_enabled
                .then(|| measured_page_size.get())
                .flatten(),
            configured_page_size.get(),
        )
    });

    if let Some(reactive_columns) = reactive_columns {
        let initial_run = StoredValue::new(true);
        Effect::new(move |_| {
            let _ = reactive_columns.get();
            if initial_run.get_value() {
                initial_run.set_value(false);
                return;
            }
            semantic_generation.update(|generation| *generation = generation.wrapping_add(1));
            preferences.normalize_after_columns_change();
        });
    }

    Effect::new(move |_| {
        let next_dataset = dataset_identity.get();
        let previous = previous_dataset.get_value();
        dataset_transition.apply(previous, next_dataset.clone());
        previous_dataset.set_value(next_dataset);
    });

    if let Some(page_reset_key) = page_reset_key {
        let previous_page_reset = StoredValue::new(page_reset_key.get_untracked());
        Effect::new(move |_| {
            let next_key = page_reset_key.get();
            let previous = previous_page_reset.get_value();
            let next_page =
                page_after_dataset_change(current_page.get_untracked(), previous, next_key.clone());
            current_page.set(next_page);
            previous_page_reset.set_value(next_key);
        });
    }

    Effect::new(move |_| {
        let total_rows = data.get().len();
        let page_size = page_capacity.get();
        let next_page = page_after_row_delta(current_page.get_untracked(), page_size, total_rows);
        if next_page != current_page.get_untracked() {
            current_page.set(next_page);
        }
    });

    if let PreferenceSource::Uncontrolled {
        current,
        persistence,
    } = preferences.source
    {
        Effect::new(move |_| {
            current.with(|preferences| save_preferences(persistence, preferences));
        });
    }

    Effect::new(move |_| {
        let next_widths = column_store.with_value(|columns| {
            preferences.with(|preferences| rendered_column_widths(preferences, columns))
        });
        if next_widths != column_widths.get_untracked() {
            column_widths.set(next_widths);
        }
    });

    Effect::new(move |_| {
        let next = column_store.with_value(|columns| {
            preferences.with(|preferences| entity_header_descriptors(preferences, columns))
        });
        if next != header_descriptors.get_untracked() {
            header_descriptors.set(next);
        }
    });

    let flexible_column_id = Signal::derive(move || {
        header_descriptors.with(|columns| entity_flexible_column_id(columns))
    });
    let stable_tracks = Signal::derive(move || {
        let widths = column_widths.get();
        header_descriptors
            .get()
            .into_iter()
            .map(|column| {
                let track = StableColumnTrack::new(
                    column.id,
                    widths.get(column.id).copied().unwrap_or_else(|| {
                        stable_column_width(None, column.initial_width.or(column.min_width))
                    }),
                );
                if flexible_column_id.get() == Some(column.id) {
                    track.flexible()
                } else {
                    track
                }
            })
            .collect::<Vec<_>>()
    });

    let total_rows = Signal::derive_local(move || data.get().len());
    let total_pages = Signal::derive(move || page_count(total_rows.get(), page_capacity.get()));
    let page_row_keys = Signal::derive_local(move || {
        let rows = data.get();
        let columns = column_store.get_value();
        let preferences_value = preferences.get();
        let indices = sorted_index_cache
            .try_update_value(|cache| {
                cache.indices(
                    Rc::clone(&rows),
                    &columns,
                    &preferences_value.sort,
                    semantic_generation.get(),
                )
            })
            .expect("entity-table sort cache is still mounted");
        let bounds = page_bounds(current_page.get(), page_capacity.get(), indices.len());
        let row_key = row_key.get_value();
        indices[bounds]
            .iter()
            .map(|index| row_key(&rows[*index]))
            .collect::<Vec<_>>()
    });

    if let Some(on_display_projection) = on_display_projection {
        Effect::new(move |_| {
            let rows = data.get();
            let columns = column_store.get_value();
            let preferences_value = preferences.get();
            let indices = sorted_index_cache
                .try_update_value(|cache| {
                    cache.indices(
                        Rc::clone(&rows),
                        &columns,
                        &preferences_value.sort,
                        semantic_generation.get(),
                    )
                })
                .expect("entity-table sort cache is still mounted");
            on_display_projection.run(entity_table_display_projection_from_indices(
                rows.as_slice(),
                &columns,
                &preferences_value,
                indices.as_slice(),
                current_page.get(),
                page_capacity.get(),
                row_key.get_value().as_ref(),
                projection_action_columns,
            ));
        });
    }

    Effect::new(move |_| {
        let Some(record) = focus_record.get() else {
            return;
        };
        let current_scope = focus_scope.get();
        let source_rows = source_data.get();
        let rendered_rows = data.get();
        let mut preferences_value = preferences.get();
        preferences_value.page_size = page_capacity.get();
        let columns = column_store.get_value();
        let row_key = row_key.get_value();
        let source_keys = source_rows
            .iter()
            .map(|row| row_key(row))
            .collect::<Vec<_>>();
        let visible_keys = visible_row_keys(
            rendered_rows.as_slice(),
            &columns,
            &preferences_value,
            current_page.get(),
            row_key.as_ref(),
        );
        let target = focus_target(
            &record,
            &source_keys,
            &visible_keys,
            &current_scope,
            false,
            true,
        );

        match target {
            EntityFocusTarget::NoChange => {}
            EntityFocusTarget::Clear => focus_record.set(None),
            EntityFocusTarget::TableRegion => {
                request_animation_frame(move || {
                    if focus_record.get_untracked().as_ref() != Some(&record)
                        || focus_moved_from_record(table_region, &record)
                    {
                        focus_record.set(None);
                        return;
                    }
                    focus_table_region(table_region);
                });
            }
            EntityFocusTarget::RowAction { row_key, action_id } => {
                request_animation_frame(move || {
                    if focus_record.get_untracked().as_ref() != Some(&record)
                        || focus_moved_from_record(table_region, &record)
                    {
                        focus_record.set(None);
                        return;
                    }
                    if !focus_row_action(table_region, &row_key, &action_id) {
                        focus_table_region(table_region);
                    }
                });
            }
        }
    });

    let measure_rows = move || {
        if !viewport_fit_enabled {
            return;
        }
        let Some(region) = table_region.get_untracked() else {
            return;
        };
        let viewport_height = region.client_height() as f64;
        let header_height = region
            .query_selector("thead")
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
            .map_or(FALLBACK_HEADER_HEIGHT, |element| {
                element.get_bounding_client_rect().height()
            });
        let row_height = region
            .query_selector("tbody tr")
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
            .map(|element| element.get_bounding_client_rect().height())
            .filter(|height| *height > 0.0)
            .unwrap_or(FALLBACK_ROW_HEIGHT);
        let Some(configured) = configured_page_size.try_get_untracked() else {
            return;
        };
        let rows = auto_page_size_for_height(
            viewport_height,
            header_height,
            row_height,
            configured,
            viewport_fit_min_rows,
        );
        if measured_page_size.try_get_untracked() != Some(Some(rows)) {
            let _ = measured_page_size.try_set(Some(rows));
        }
    };
    let measure_handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let schedule_measure = move || {
        if !viewport_fit_enabled {
            return;
        }
        if let Some(handle) = measure_handle.try_get_value().flatten() {
            handle.clear();
        }
        match set_timeout_with_handle(measure_rows, std::time::Duration::ZERO) {
            Ok(handle) => {
                measure_handle.try_update_value(|slot| *slot = Some(handle));
            }
            Err(_) => measure_rows(),
        }
    };
    on_cleanup(move || {
        if let Some(handle) = measure_handle.try_get_value().flatten() {
            handle.clear();
        }
    });

    if viewport_fit_enabled {
        Effect::new(move |_| {
            let _ = data.get();
            let _ = column_filters.get();
            let _ = header_descriptors.get();
            let _ = texts.get();
            let _ = preferences.get();
            let _ = page_capacity.get();
            schedule_measure();
        });

        Effect::new(move |_| {
            let Some(region) = table_region.get() else {
                return;
            };
            schedule_measure();
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
                move |_entries: js_sys::Array, _observer: web_sys::ResizeObserver| {
                    schedule_measure();
                },
            )
                as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);
            match web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) {
                Ok(observer) => {
                    observer.observe(region.unchecked_ref::<web_sys::Element>());
                    if let Ok(Some(table)) = region.query_selector("table") {
                        observer.observe(&table);
                    }
                    let guard = send_wrapper::SendWrapper::new((closure, observer));
                    on_cleanup(move || {
                        let (closure, observer) = guard.take();
                        observer.disconnect();
                        drop(closure);
                    });
                }
                Err(_) => drop(closure),
            }
        });
    }

    let root_class = if viewport_fit_enabled {
        merge_classes!("flex h-full w-full min-h-0 min-w-0 flex-col gap-3", class)
    } else {
        merge_classes!("w-full min-w-0 space-y-3", class)
    };
    let root_style = viewport_fit_enabled.then(|| match viewport_fit_height.as_deref() {
        Some(height) => format!("height: {height}; max-height: {height}"),
        None => "height: 100%".to_owned(),
    });
    let region_class = if viewport_fit_enabled {
        "min-h-0 w-full flex-1 overflow-auto rounded-box border border-table-grid bg-base-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-primary"
    } else {
        "w-full overflow-x-auto rounded-box border border-table-grid bg-base-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-primary"
    };

    view! {
        <section
            class=root_class
            style=root_style
            data-entity-table="true"
            data-table-data-mode="client-snapshot"
            data-entity-viewport-fit=viewport_fit_enabled.then_some("true")
            data-entity-effective-page-size=move || page_capacity.get().to_string()
            data-entity-configured-page-size=move || configured_page_size.get().to_string()
        >
            {move || {
                let filters = column_filters.get();
                let compact = compact_filter_layout.get();
                let preferences_value = preferences.get();
                let fallback_filters = filters
                    .into_iter()
                    .filter(|filter| {
                        compact
                            || (filter.is_active()
                                && preferences_value
                                    .hidden_columns
                                    .contains(filter.column_id))
                    })
                    .collect::<Vec<_>>();
                if fallback_filters.is_empty() {
                    return None;
                }

                Some(view! {
                    <section
                        class="shrink-0 rounded-box border border-table-grid bg-table-filter p-3 text-table-filter-content forced-colors:border-[CanvasText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                        data-entity-responsive-filter-panel="true"
                        aria-label=move || texts.with(|texts| texts.filters.clone())
                        aria-live="polite"
                    >
                        <p class="mb-2 text-sm font-semibold">
                            {move || texts.with(|texts| texts.filters.clone())}
                        </p>
                        <div class="grid min-w-0 gap-3 sm:grid-cols-2">
                            {fallback_filters
                                .into_iter()
                                .map(|filter| {
                                    let column_label = current_column_header(
                                        column_store,
                                        filter.column_id,
                                    );
                                    let label = filter.label(&column_label);
                                    let active = filter.is_active();
                                    let hidden = preferences_value
                                        .hidden_columns
                                        .contains(filter.column_id);
                                    let on_clear = filter.clear_callback();
                                    let clear_label = texts.with(|texts| {
                                        texts.clear_filter.replace("{column}", &label)
                                    });
                                    view! {
                                        <div
                                            class="min-w-0 rounded-field border border-table-grid/70 bg-base-100 p-2 text-base-content forced-colors:border-[CanvasText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                                            role="group"
                                            aria-label=label.clone()
                                            data-entity-responsive-filter=filter.column_id
                                            data-entity-active-hidden-filter=(active && hidden).then_some("true")
                                        >
                                            <div class="mb-1 flex min-w-0 items-center justify-between gap-2">
                                                <span class="min-w-0 text-xs font-semibold">{label.clone()}</span>
                                                {active.then(|| {
                                                    let on_clear = on_clear
                                                        .expect("active responsive filters carry clear intent");
                                                    view! {
                                                    <Button
                                                        class="btn-ghost btn-xs shrink-0"
                                                        attr:data-entity-clear-filter=filter.column_id
                                                        attr:aria-label=clear_label
                                                        on_click=Callback::new(move |_| on_clear.run(()))
                                                    >
                                                        {move || texts.with(|texts| texts.filter_active.clone())}
                                                    </Button>
                                                }})}
                                            </div>
                                            {filter.render(EntityColumnFilterPlacement::Responsive)}
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </section>
                })
            }}
            <div class="flex shrink-0 flex-wrap items-center justify-end gap-2">
                <label class="flex min-w-0 max-w-full flex-wrap items-center justify-end gap-2 text-sm text-base-content/75">
                    <span class="min-w-0 break-words">{move || texts.with(|texts| texts.rows_per_page.clone())}</span>
                    <Select
                        class="select-sm w-20 shrink-0"
                        id=page_size_select_id
                        name=page_size_select_id
                        label=Signal::derive(move || {
                            Some(texts.with(|texts| texts.rows_per_page.clone()))
                        })
                        value=Signal::derive(move || {
                            preferences.with(|preferences| preferences.page_size.to_string())
                        })
                        node_ref=page_size_select
                        on_change=Callback::new(move |value: String| {
                            apply_page_size_change(
                                preferences,
                                current_page,
                                &value,
                                move |supplied_value| {
                                    if let Some(select) = page_size_select.get() {
                                        select.set_value(&supplied_value);
                                    }
                                },
                            );
                        })
                    >
                        {ENTITY_PAGE_SIZE_CHOICES.into_iter().map(|page_size| view! {
                            <option value=page_size.to_string()>{page_size}</option>
                        }).collect_view()}
                    </Select>
                </label>

                {toolbar_actions.map(|render_actions| view! {
                    <div class="contents" data-entity-toolbar-actions="true">
                        {render_actions()}
                    </div>
                })}

                <div
                    class="dropdown dropdown-end dropdown-bottom"
                    class:dropdown-open=move || column_chooser_open.get()
                    // daisyUI also shows `.dropdown-content` on `:focus-within`,
                    // and Escape deliberately returns focus to the trigger
                    // button (still inside this container) so keyboard users
                    // don't lose their place. Without `.dropdown-close` that
                    // `:focus-within` match alone would keep the menu visible
                    // even after `column_chooser_open` goes false (ldui-vn81
                    // follow-up); `.dropdown-close` unconditionally overrides
                    // `:focus-within` in daisyUI's own selector.
                    class:dropdown-close=move || !column_chooser_open.get()
                    data-entity-column-chooser-open=move || column_chooser_open.get().then_some("true")
                    on:focusout=move |event: web_sys::FocusEvent| {
                        let remains_inside = event
                            .current_target()
                            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                            .zip(
                                event
                                    .related_target()
                                    .and_then(|target| target.dyn_into::<web_sys::Node>().ok()),
                            )
                            .is_some_and(|(root, next)| root.contains(Some(&next)));
                        if !remains_inside {
                            column_chooser_open.set(false);
                        }
                    }
                    on:keydown=move |event: web_sys::KeyboardEvent| {
                        if event.key() == "Escape" && column_chooser_open.get_untracked() {
                            event.prevent_default();
                            event.stop_propagation();
                            column_chooser_open.set(false);
                            if let Some(trigger) = column_chooser_trigger_ref.get() {
                                let _ = trigger.focus();
                            }
                        }
                    }
                >
                    <button
                        node_ref=column_chooser_trigger_ref
                        type="button"
                        data-entity-column-chooser="true"
                        data-entity-column-chooser-presentation=move || match column_chooser_trigger.get() {
                            EntityColumnChooserTrigger::Text => "text",
                            EntityColumnChooserTrigger::Icon => "icon",
                        }
                        aria-label=move || texts.with(|texts| texts.choose_columns.clone())
                        aria-haspopup="menu"
                        aria-expanded=move || column_chooser_open.get().to_string()
                        aria-controls=column_chooser_controls_id
                        // `gap-0`: a flex/grid item's computed `display` is
                        // "blockified" per spec, so the single `<span>` child
                        // below counts as a layout child even though there is
                        // only ever one of them -- leaving daisyUI's stock
                        // `.btn` `gap: .375rem` (6px, off the canonical
                        // spacing scale) "declared" with no visual effect.
                        // Pin it to the canonical 0 explicitly rather than
                        // leave dead, off-scale CSS for the audit to flag.
                        class=move || match column_chooser_trigger.get() {
                            EntityColumnChooserTrigger::Text => "btn btn-ghost btn-sm gap-0",
                            EntityColumnChooserTrigger::Icon => "btn btn-ghost btn-sm btn-square gap-0 forced-colors:border forced-colors:border-[ButtonText] forced-colors:text-[ButtonText]",
                        }
                        on:click=move |_| column_chooser_open.update(|open| *open = !*open)
                    >
                        {move || match column_chooser_trigger.get() {
                            EntityColumnChooserTrigger::Text => view! {
                                <span>{texts.with(|texts| texts.choose_columns.clone())}</span>
                            }.into_any(),
                            EntityColumnChooserTrigger::Icon => view! {
                                // `text-base` (16px) is on the type ramp;
                                // `text-lg` (18px) is not.
                                <span aria-hidden="true" class="text-base leading-none">"⚙"</span>
                            }.into_any(),
                        }}
                    </button>
                    <div class="dropdown-content bg-base-100 rounded-box z-[2] w-72 p-0 shadow-lg border border-base-300">
                        <Menu class="w-full" attr:id=column_chooser_menu_id>
                            {move || column_store.with_value(|columns| {
                                columns
                                    .iter()
                                    .filter(|column| !column.required)
                                    .cloned()
                                    .map(|column| {
                                        let column_id = column.id;
                                        let checked = Signal::derive(move || {
                                            !preferences.with(|preferences| {
                                                preferences.hidden_columns.contains(column_id)
                                            })
                                        });
                                        let active_filter = Signal::derive(move || {
                                            column_filters.with(|filters| {
                                                filters
                                                    .iter()
                                                    .find(|filter| filter.column_id == column_id)
                                                    .is_some_and(EntityColumnFilter::is_active)
                                            })
                                        });
                                        let on_toggle = Callback::new(move |_| {
                                            column_store.with_value(|columns| {
                                                preferences.update(|preferences| {
                                                    toggle_hidden_column(
                                                        preferences,
                                                        columns,
                                                        column_id,
                                                    );
                                                });
                                            });
                                        });
                                        // `EntityTableTexts::filter_active`'s own doc says this
                                        // item "cannot be hidden" -- a one-directional guard.
                                        // Gating on `active_filter` alone also disabled SHOWING
                                        // an already-hidden column once its filter went active
                                        // (e.g. set from the responsive fallback panel while
                                        // hidden), permanently trapping it there with no way to
                                        // restore it short of first clearing the filter. Disable
                                        // only the hide direction: a visible, actively-filtered
                                        // column can't be unchecked; a hidden one can always be
                                        // rechecked.
                                        let disabled = Signal::derive(move || {
                                            checked.get() && active_filter.get()
                                        });
                                        view! {
                                            <MenuCheckItem
                                                checked=checked
                                                disabled=disabled
                                                on_toggle=on_toggle
                                                attr:data-entity-column=column_id
                                                attr:data-entity-active-filter=move || active_filter.get().then_some("true")
                                            >
                                                <span class="flex min-w-0 items-center justify-between gap-2">
                                                    <span class="min-w-0 truncate">{column.header}</span>
                                                    <Show when=move || active_filter.get()>
                                                        <span class="badge badge-sm shrink-0">
                                                            {move || texts.with(|texts| texts.filter_active.clone())}
                                                        </span>
                                                    </Show>
                                                </span>
                                            </MenuCheckItem>
                                        }
                                    })
                                .collect_view()
                            })}
                        </Menu>
                        <div class="border-t border-base-300 p-2">
                            <p class="px-2 pb-1 text-xs font-semibold text-base-content/65">
                                {move || texts.with(|texts| texts.column_order.clone())}
                            </p>
                            <ol
                                class="space-y-1"
                                aria-label=move || texts.with(|texts| texts.column_order.clone())
                            >
                                <For
                                    each=move || column_store.with_value(|columns| {
                                        ordered_columns(&preferences.get(), columns)
                                            .into_iter()
                                            .map(|column| column.id)
                                            .collect::<Vec<_>>()
                                    })
                                    key=|column_id| *column_id
                                    children=move |column_id| {
                                        view! {
                                            <li
                                                class="flex items-center gap-1 rounded-field px-2 py-1"
                                                data-entity-column-order=column_id
                                            >
                                                <span class="min-w-0 flex-1 truncate text-sm">
                                                    {move || current_column_header(column_store, column_id)}
                                                </span>
                                                <Button
                                                    class="btn-ghost btn-xs btn-square"
                                                    attr:data-entity-column-order=column_id
                                                    attr:data-entity-column-move="earlier"
                                                    attr:aria-label=move || {
                                                        let (position, total) = preferences.with(|preferences| {
                                                            (
                                                                preferences
                                                                    .column_order
                                                                    .iter()
                                                                    .position(|id| id == column_id)
                                                                    .map(|index| index + 1)
                                                                    .unwrap_or(1),
                                                                preferences.column_order.len(),
                                                            )
                                                        });
                                                        texts.with(|texts| {
                                                            format_move_label(
                                                                &texts.move_earlier,
                                                                &current_column_header(column_store, column_id),
                                                                position,
                                                                total,
                                                            )
                                                        })
                                                    }
                                                    disabled=Signal::derive(move || {
                                                        preferences.with(|preferences| {
                                                            preferences.column_order.first().is_some_and(|id| id == column_id)
                                                        })
                                                    })
                                                    on_click=Callback::new(move |event: web_sys::MouseEvent| {
                                                        restore_column_move_focus(
                                                            event,
                                                            column_id,
                                                            EntityColumnMove::Earlier,
                                                        );
                                                        column_store.with_value(|columns| {
                                                            preferences.update(|preferences| {
                                                                move_column(
                                                                    preferences,
                                                                    columns,
                                                                    column_id,
                                                                    EntityColumnMove::Earlier,
                                                                );
                                                            });
                                                        });
                                                    })
                                                >
                                                    <span aria-hidden="true">"↑"</span>
                                                </Button>
                                                <Button
                                                    class="btn-ghost btn-xs btn-square"
                                                    attr:data-entity-column-order=column_id
                                                    attr:data-entity-column-move="later"
                                                    attr:aria-label=move || {
                                                        let (position, total) = preferences.with(|preferences| {
                                                            (
                                                                preferences
                                                                    .column_order
                                                                    .iter()
                                                                    .position(|id| id == column_id)
                                                                    .map(|index| index + 1)
                                                                    .unwrap_or(1),
                                                                preferences.column_order.len(),
                                                            )
                                                        });
                                                        texts.with(|texts| {
                                                            format_move_label(
                                                                &texts.move_later,
                                                                &current_column_header(column_store, column_id),
                                                                position,
                                                                total,
                                                            )
                                                        })
                                                    }
                                                    disabled=Signal::derive(move || {
                                                        preferences.with(|preferences| {
                                                            preferences.column_order.last().is_some_and(|id| id == column_id)
                                                        })
                                                    })
                                                    on_click=Callback::new(move |event: web_sys::MouseEvent| {
                                                        restore_column_move_focus(
                                                            event,
                                                            column_id,
                                                            EntityColumnMove::Later,
                                                        );
                                                        column_store.with_value(|columns| {
                                                            preferences.update(|preferences| {
                                                                move_column(
                                                                    preferences,
                                                                    columns,
                                                                    column_id,
                                                                    EntityColumnMove::Later,
                                                                );
                                                            });
                                                        });
                                                    })
                                                >
                                                    <span aria-hidden="true">"↓"</span>
                                                </Button>
                                            </li>
                                        }
                                    }
                                />
                            </ol>
                        </div>
                    </div>
                </div>

                {show_reset_actions.then(|| view! {
                    <Button
                        class="btn-ghost btn-sm"
                        attr:data-entity-reset-sort="true"
                        disabled=Signal::derive(move || {
                            preferences.with(|preferences| preferences.sort.is_system())
                        })
                        on_click=Callback::new(move |_| {
                            preferences.update(|preferences| {
                                reset_sort(preferences);
                            });
                            current_page.set(0);
                        })
                    >
                        {move || texts.with(|texts| texts.reset_sort.clone())}
                    </Button>
                    <Button
                        class="btn-ghost btn-sm"
                        attr:data-entity-reset-columns="true"
                        disabled=Signal::derive(move || preferences.with(|preferences| {
                            preferences.hidden_columns.is_empty()
                                && preferences.column_widths.is_empty()
                                && column_store.with_value(|columns| {
                                    preferences
                                        .column_order
                                        .iter()
                                        .map(String::as_str)
                                        .eq(columns.iter().map(|column| column.id))
                                })
                        }))
                        on_click=Callback::new(move |_| {
                            column_widths.set(
                                preferences.update_and_rendered_widths(|preferences| {
                                    reset_columns(preferences);
                                }),
                            );
                        })
                    >
                        {move || texts.with(|texts| texts.reset_columns.clone())}
                    </Button>
                })}
            </div>

            <p class="sr-only" aria-live="polite" data-entity-sort-summary="true">
                {move || column_store.with_value(|columns| {
                    preferences.with(|preferences| {
                        texts.with(|texts| sort_summary(&preferences.sort, columns, texts))
                    })
                })}
            </p>

            <div
                node_ref=table_region
                class=region_class
                role="region"
                tabindex="-1"
                aria-label=move || texts.with(|texts| texts.region_label.clone())
                data-entity-focus-region="true"
                data-entity-column-generation=move || semantic_generation.get().to_string()
                on:focusin=move |event: web_sys::FocusEvent| {
                    focus_record.set(focus_record_from_event(
                        &event,
                        &focus_scope.get_untracked(),
                    ));
                }
            >
                <div style=move || stable_table_content_style(&stable_tracks.get())>
                <table
                    class="table table-sm table-fixed w-full border-collapse border border-table-grid"
                    class:table-zebra=move || zebra.get()
                    data-entity-table-grid="true"
                    data-table-layout="stable"
                >
                    <StableTableColGroup tracks=stable_tracks />
                    <thead class="hidden lg:table-header-group">
                        <tr>
                            <For
                                each=move || header_descriptors.get()
                                key=|column| (
                                    column.id,
                                    column.sortable,
                                    column.resizable,
                                    column.min_width,
                                    column.initial_width,
                                    column.alignment,
                                    column.tabular_numbers,
                                )
                                children=move |column| {
                                let column_id = column.id;
                                let sortable = column.sortable;
                                let resizable = column.resizable;
                                let minimum_width = column.min_width;
                                let alignment = column.alignment;
                                let tabular_numbers = column.tabular_numbers;
                                let minimum_value = effective_min_width(minimum_width);
                                let width_style = move || {
                                    if flexible_column_id.get() == Some(column_id) {
                                        return minimum_width.map(|_| {
                                            format!("min-width: {}px", minimum_value.round())
                                        });
                                    }
                                    column_widths
                                        .with(|widths| widths.get(column_id).copied())
                                        .map(|width| format!(
                                            "width: {width}px; min-width: {width}px; max-width: {width}px"
                                        ))
                                        .or_else(|| {
                                            minimum_width.map(|_| {
                                                format!("min-width: {}px", minimum_value.round())
                                            })
                                        })
                                };
                                let sort_label = move || preferences.with(|preferences| {
                                    texts.with(|texts| sort_accessible_label(
                                        &preferences.sort,
                                        column_id,
                                        &current_header(&header_descriptors, column_id),
                                        texts,
                                    ))
                                });
                                view! {
                                    <th
                                        class=move || merge_classes!(
                                            "relative border border-table-grid bg-table-header text-table-header-content forced-colors:border-[CanvasText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]",
                                            entity_alignment_class(alignment),
                                            if tabular_numbers { "tabular-nums" } else { "" }
                                        )
                                        scope="col"
                                        data-entity-column=column_id
                                        data-entity-alignment=alignment.as_str()
                                        data-entity-tabular-numbers=tabular_numbers.then_some("true")
                                        aria-sort=move || preferences.with(|preferences| {
                                            preferences.sort.aria_value_for(column_id)
                                        })
                                        data-entity-sort-priority=move || preferences.with(|preferences| {
                                            preferences.sort.priority_for(column_id).map(|priority| priority.to_string())
                                        })
                                        data-entity-sort-direction=move || preferences.with(|preferences| {
                                            preferences.sort.direction_for(column_id).map(|direction| {
                                                direction.aria_value()
                                            })
                                        })
                                        style=width_style
                                    >
                                        {if sortable {
                                            Some(view! {
                                                <Button
                                                    class="btn-ghost btn-xs h-auto !min-h-0 w-full justify-start gap-1 rounded-sm px-0 py-1 text-left font-semibold text-table-header-content !shadow-none hover:bg-white/15 focus-visible:outline-white forced-colors:text-[CanvasText]"
                                                    attr:data-entity-sort-column=column_id
                                                    attr:aria-label=sort_label
                                                    on:keydown=move |event: web_sys::KeyboardEvent| {
                                                        if !event.shift_key()
                                                            || !matches!(event.key().as_str(), "Enter" | " " | "Spacebar")
                                                        {
                                                            return;
                                                        }
                                                        event.prevent_default();
                                                        event.stop_propagation();
                                                        preferences.update(|preferences| {
                                                            preferences.sort = next_sort_additive(
                                                                &preferences.sort,
                                                                column_id,
                                                                true,
                                                            );
                                                        });
                                                        current_page.set(0);
                                                    }
                                                    on_click=Callback::new(move |event: web_sys::MouseEvent| {
                                                        preferences.update(|preferences| {
                                                            preferences.sort = if event.shift_key() {
                                                                next_sort_additive(
                                                                    &preferences.sort,
                                                                    column_id,
                                                                    true,
                                                                )
                                                            } else {
                                                                next_sort(
                                                                    &preferences.sort,
                                                                    column_id,
                                                                    true,
                                                                )
                                                            };
                                                        });
                                                        current_page.set(0);
                                                    })
                                                >
                                                    <span
                                                        class=move || merge_classes!(
                                                            "flex w-full items-center gap-1",
                                                            entity_header_justify_class(alignment),
                                                            entity_alignment_class(alignment)
                                                        )
                                                    >
                                                        <span>{move || current_header(&header_descriptors, column_id)}</span>
                                                        <span
                                                            aria-hidden="true"
                                                            data-entity-sort-indicator="true"
                                                            class="inline-flex w-6 shrink-0 justify-center text-xs"
                                                        >
                                                            {move || preferences.with(|preferences| {
                                                                let Some(direction) = preferences.sort.direction_for(column_id) else {
                                                                    return "↕".to_owned();
                                                                };
                                                                let marker = match direction {
                                                                    EntitySortDirection::Ascending => "▲",
                                                                    EntitySortDirection::Descending => "▼",
                                                                };
                                                                format!(
                                                                    "{marker}{}",
                                                                    preferences.sort.priority_for(column_id).unwrap_or(1)
                                                                )
                                                            })}
                                                        </span>
                                                    </span>
                                                </Button>
                                            })
                                        } else {
                                            None
                                        }}
                                        {(!sortable).then(|| view! {
                                            <span class=entity_alignment_class(alignment)>
                                                {move || current_header(&header_descriptors, column_id)}
                                            </span>
                                        })}
                                        {resizable.then(|| view! {
                                            <span
                                                class="absolute top-0 right-0 z-10 h-full w-2 cursor-col-resize select-none opacity-0 hover:opacity-100 hover:bg-primary/50 focus:opacity-100 focus:bg-primary/50 focus:outline focus:outline-2 focus:outline-primary active:opacity-100 active:bg-primary/70"
                                                role="separator"
                                                tabindex="0"
                                                aria-orientation="vertical"
                                                aria-label=move || texts.with(|texts| {
                                                    texts.resize_column.replace(
                                                        "{column}",
                                                        &current_header(&header_descriptors, column_id),
                                                    )
                                                })
                                                aria-valuemin=minimum_value.round() as u32
                                                aria-valuemax=MAX_COLUMN_WIDTH.round() as u32
                                                aria-valuenow=move || column_widths.with(|widths| {
                                                    widths
                                                        .get(column_id)
                                                        .copied()
                                                        .unwrap_or_else(|| {
                                                            minimum_value.round() as u32
                                                        })
                                                })
                                                aria-valuetext=move || column_widths.with(|widths| {
                                                    let pixels = widths
                                                        .get(column_id)
                                                        .copied()
                                                        .unwrap_or_else(|| minimum_value.round() as u32);
                                                    texts.with(|texts| {
                                                        texts.pixel_value.replace(
                                                            "{pixels}",
                                                            &pixels.to_string(),
                                                        )
                                                    })
                                                })
                                                on:click=move |event: web_sys::MouseEvent| event.stop_propagation()
                                                on:focus=move |event: web_sys::FocusEvent| {
                                                    if let Some(rendered_width) = separator_parent_width(event.target()) {
                                                        let width = rendered_width
                                                            .clamp(minimum_value, MAX_COLUMN_WIDTH)
                                                            .round() as u32;
                                                        column_widths.update(|widths| {
                                                            widths.insert(column_id.to_owned(), width);
                                                        });
                                                    }
                                                }
                                                on:keydown=move |event: web_sys::KeyboardEvent| {
                                                    let current_width = separator_parent_width(
                                                        event.current_target().or_else(|| event.target()),
                                                    )
                                                    .or_else(|| column_widths.with_untracked(|widths| {
                                                        widths.get(column_id).copied().map(f64::from)
                                                    }))
                                                    .unwrap_or(minimum_value);
                                                    let Some(requested_width) = keyboard_resized_width(
                                                        current_width,
                                                        &event.key(),
                                                        minimum_value,
                                                    ) else {
                                                        return;
                                                    };
                                                    event.prevent_default();
                                                    event.stop_propagation();
                                                    column_widths.set(
                                                        preferences.update_and_rendered_widths(|preferences| {
                                                            set_preferred_width(
                                                                preferences,
                                                                column_id,
                                                                requested_width,
                                                                minimum_width,
                                                            );
                                                        }),
                                                    );
                                                }
                                                on:pointerdown=move |event: web_sys::PointerEvent| {
                                                    event.stop_propagation();
                                                    let rendered_width = event
                                                        .target()
                                                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                                                        .and_then(|element| element.parent_element())
                                                        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
                                                        .map(|element| f64::from(element.offset_width()));
                                                    let start_width = rendered_width
                                                        .or_else(|| column_widths.with_untracked(|widths| {
                                                            widths.get(column_id).copied().map(f64::from)
                                                        }))
                                                        .unwrap_or_else(|| f64::from(minimum_width.unwrap_or(48)));
                                                    resize_drag.set(Some(ResizeDrag {
                                                        column_id: column_id.to_owned(),
                                                        start_x: f64::from(event.client_x()),
                                                        start_width,
                                                        minimum_width,
                                                    }));
                                                    if let Some(target) = event.target()
                                                        && let Ok(element) = target.dyn_into::<web_sys::Element>()
                                                    {
                                                        let _ = element.set_pointer_capture(event.pointer_id());
                                                    }
                                                }
                                                on:pointermove=move |event: web_sys::PointerEvent| {
                                                    if let Some(drag) = resize_drag.get_untracked() {
                                                        let requested = drag.start_width
                                                            + (f64::from(event.client_x()) - drag.start_x);
                                                        let mut scratch = EntityTablePreferences::new(
                                                            preference_version,
                                                        );
                                                        set_preferred_width(
                                                            &mut scratch,
                                                            drag.column_id.clone(),
                                                            requested,
                                                            drag.minimum_width,
                                                        );
                                                        if let Some(width) = scratch.column_widths.get(&drag.column_id) {
                                                            column_widths.update(|widths| {
                                                                widths.insert(drag.column_id.clone(), *width);
                                                            });
                                                        }
                                                    }
                                                }
                                                on:pointerup=move |event: web_sys::PointerEvent| {
                                                    finish_resize(
                                                        event.target(),
                                                        event.pointer_id(),
                                                        resize_drag,
                                                        column_widths,
                                                        preferences,
                                                    );
                                                }
                                                on:pointercancel=move |event: web_sys::PointerEvent| {
                                                    finish_resize(
                                                        event.target(),
                                                        event.pointer_id(),
                                                        resize_drag,
                                                        column_widths,
                                                        preferences,
                                                    );
                                                }
                                            ></span>
                                        })}
                                    </th>
                                }
                                }
                            />
                        </tr>
                        {move || {
                            let filters = column_filters.get();
                            if filters.is_empty() || compact_filter_layout.get() {
                                return None;
                            }
                            Some(view! {
                                <tr
                                    class="data-table-filter-row bg-table-filter text-table-filter-content forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                                    data-entity-column-filter-row="true"
                                >
                                    {header_descriptors
                                        .get()
                                        .into_iter()
                                        .map(|column| {
                                            let filter = filters
                                                .iter()
                                                .find(|filter| filter.column_id == column.id)
                                                .cloned();
                                            view! {
                                                <th
                                                    class="border border-table-grid bg-table-filter p-1 text-table-filter-content forced-colors:border-[CanvasText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                                                    data-entity-column=column.id
                                                    data-entity-column-filter-cell="true"
                                                    on:click=move |event| event.stop_propagation()
                                                    on:keydown=move |event| event.stop_propagation()
                                                    on:pointerdown=move |event| event.stop_propagation()
                                                >
                                                    {filter.map(|filter| {
                                                        filter.render(EntityColumnFilterPlacement::Header)
                                                    })}
                                                </th>
                                            }
                                        })
                                        .collect_view()}
                                </tr>
                            })
                        }}
                    </thead>
                    <tbody>
                        {move || page_row_keys.with(|keys| keys.is_empty()).then(|| {
                            let colspan = column_store.with_value(|columns| {
                                let preferences_value = preferences.get();
                                ordered_columns(&preferences_value, columns)
                                    .into_iter()
                                    .filter(|column| {
                                        !preferences_value.hidden_columns.contains(column.id)
                                    })
                                    .count()
                                    .max(1)
                            });
                            view! {
                                    <tr>
                                        <td
                                            colspan=colspan
                                            class="border border-table-grid py-10 text-center text-base-content/65 forced-colors:border-[CanvasText]"
                                        >
                                            {texts.with(|texts| texts.no_rows.clone())}
                                        </td>
                                    </tr>
                            }
                        })}
                        {local_for_enumerate(
                            move || page_row_keys.get(),
                            |key| key.clone(),
                            move |visible_position, key| render_keyed_row(
                                key,
                                visible_position,
                                KeyedRowContext {
                                    data,
                                    column_store,
                                    preferences,
                                    row_key,
                                    compact_row,
                                    on_row_activate,
                                    selection,
                                    row_emphasis,
                                },
                            ),
                        )}
                    </tbody>
                </table>
                </div>
            </div>

            <div class="flex shrink-0 flex-wrap items-center justify-between gap-3">
                <span class="text-sm text-base-content/75">
                    {move || {
                        let total = total_rows.get();
                        if total == 0 {
                            return String::new();
                        }
                        let page_size = page_capacity.get();
                        let (start, end) = row_range(current_page.get(), page_size, total);
                        texts
                            .with(|texts| texts.row_range.clone())
                            .replace("{start}", &start.to_string())
                            .replace("{end}", &end.to_string())
                            .replace("{total}", &total.to_string())
                    }}
                </span>
                <Pagination class="max-w-full flex flex-wrap items-center justify-end gap-1">
                    <Button
                        class="join-item btn-sm"
                        attr:data-entity-page="previous"
                        disabled=Signal::derive(move || current_page.get() == 0)
                        on_click=Callback::new(move |_| {
                            current_page.update(|page| *page = page.saturating_sub(1));
                        })
                    >
                        {move || texts.with(|texts| texts.previous.clone())}
                    </Button>
                    {move || page_window(current_page.get(), total_pages.get(), MAX_VISIBLE_PAGES)
                        .into_iter()
                        .map(|slot| match slot {
                            PageSlot::Page(page) => view! {
                                <Button
                                    class="join-item btn-sm"
                                    attr:data-entity-page=(page + 1).to_string()
                                    active=page == current_page.get()
                                    disabled=page == current_page.get()
                                    on_click=Callback::new(move |_| current_page.set(page))
                                >
                                    {(page + 1).to_string()}
                                </Button>
                            }.into_any(),
                            PageSlot::Ellipsis => view! {
                                <span class="join-item btn btn-sm btn-disabled" aria-hidden="true">"…"</span>
                            }.into_any(),
                        })
                        .collect_view()}
                    <Button
                        class="join-item btn-sm"
                        attr:data-entity-page="next"
                        disabled=Signal::derive(move || {
                            current_page.get() + 1 >= total_pages.get()
                        })
                        on_click=Callback::new(move |_| {
                            current_page.update(|page| {
                                *page = clamp_page(
                                    page.saturating_add(1),
                                    page_capacity.get_untracked(),
                                    total_rows.get_untracked(),
                                );
                            });
                        })
                    >
                        {move || texts.with(|texts| texts.next.clone())}
                    </Button>
                </Pagination>
            </div>
        </section>
    }
}

fn local_for_enumerate<IF, I, T, EF, N, KF, K>(each: IF, key: KF, children: EF) -> impl IntoView
where
    IF: Fn() -> I + 'static,
    I: IntoIterator<Item = T> + Send + 'static,
    EF: Fn(ReadSignal<usize, LocalStorage>, T) -> N + Clone + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + Send + Clone + 'static,
    K: Eq + std::hash::Hash + leptos::tachys::view::keyed::SerializableKey + 'static,
    T: 'static,
{
    // Tachys requires the keyed view factory to be `Send` even for a CSR-only
    // local view. The factory is still created, called, and dropped on the one
    // browser thread; these wrappers encode that invariant while allowing the
    // row renderer itself to retain `Rc` callbacks and `LocalStorage` signals.
    let parent = send_wrapper::SendWrapper::new(
        Owner::current().expect("entity-table keyed rows require a reactive owner"),
    );
    let each = send_wrapper::SendWrapper::new(each);
    let child_renderer = send_wrapper::SendWrapper::new(children);
    let children = move |index, child| {
        let owner = parent.with(Owner::new);
        let ((_, set_index), view) = owner.with(|| {
            let index = RwSignal::new_local(index).split();
            let view = child_renderer(index.0, child);
            (index, view)
        });
        (
            move |next_index| set_index.set(next_index),
            OwnedView::new_with_owner(view, owner),
        )
    };
    move || leptos::tachys::view::keyed::keyed(each(), key.clone(), children.clone())
}

struct KeyedRowContext<T: 'static> {
    data: Signal<Rc<Vec<T>>, LocalStorage>,
    column_store: ColumnStore<T>,
    preferences: PreferenceState<T>,
    row_key: StoredValue<EntityRowKey<T>, LocalStorage>,
    compact_row: CompactRowStore<T>,
    on_row_activate: Option<Callback<String>>,
    selection: Option<EntityTableSelection>,
    row_emphasis: StoredValue<Option<EntityRowEmphasisClassifier<T>>, LocalStorage>,
}

fn render_keyed_row<T: Clone + 'static>(
    key: String,
    visible_position: ReadSignal<usize, LocalStorage>,
    context: KeyedRowContext<T>,
) -> impl IntoView {
    let KeyedRowContext {
        data,
        column_store,
        preferences,
        row_key,
        compact_row,
        on_row_activate,
        selection,
        row_emphasis,
    } = context;
    // A table with only `selection` (no `on_row_activate`) is still
    // keyboard-operable, mirroring `data_table::row_is_interactive`.
    let interactive = on_row_activate.is_some() || selection.is_some();
    // `aria-selected` and selected styling are gated on `selection` alone,
    // not `interactive`: an activate-only table (no `selection` supplied)
    // has no selection concept at all, so it must emit no `aria-selected`
    // attribute -- painting `aria-selected="false"` there would tell
    // assistive tech the row is selectable when it never was. This keeps
    // every existing `on_row_activate`-only caller's DOM byte-for-byte
    // unchanged.
    let has_selection = selection.is_some();
    // Mirrors `has_selection`'s gating: with no `row_emphasis` classifier at
    // all, no row emits a `data-entity-row-emphasis` attribute or any
    // emphasis class, restoring the exact DOM of a table that predates this
    // prop.
    let has_row_emphasis = row_emphasis.with_value(Option::is_some);
    let click_key = key.clone();
    let keydown_key = key.clone();
    let selected_class_key = key.clone();
    let selected_aria_key = key.clone();

    // Focus and selection are deliberately distinct: Tab/roving focus never
    // proposes or paints selection by itself -- only a click or Enter/Space
    // does, via the handlers below.
    let is_row_selected = move |current_key: &str| {
        selection.is_some_and(|selection| {
            entity_row_is_selected(current_key, selection.selected_key().get().as_deref())
        })
    };

    // The row's own content is looked up by key once here and cached, then
    // shared by the `<tr>` class, the `data-entity-row-emphasis` attribute,
    // and cell rendering below -- resolving classification (a pure function
    // of the row's own content, so it follows a row across sorting,
    // filtering, and paging rather than pinning a look to a rendered
    // position) exactly once per row per `data` change, instead of an
    // independent `O(total_rows)` dataset scan at each of those three sites.
    // The lookup effect depends only on `data`, so it never reruns for
    // unrelated reactivity -- a selection click, in particular, previously
    // forced a full-dataset rescan here purely because it shares the same
    // `class` closure.
    let lookup_key = key.clone();
    let resolve_current_row = move |rows: &Rc<Vec<T>>| -> Option<T> {
        let row_key_fn = row_key.get_value();
        rows.iter()
            .find(|row| row_key_fn(row) == lookup_key)
            .cloned()
    };
    let initial_row = resolve_current_row(&data.get_untracked());
    let initial_emphasis = row_emphasis.with_value(|classifier| {
        entity_row_emphasis_for(classifier.as_ref(), initial_row.as_ref())
    });
    let cached_row: RwSignal<Option<T>, LocalStorage> = RwSignal::new_local(initial_row);
    let cached_emphasis = RwSignal::new(initial_emphasis);
    Effect::new(move |_| {
        let row = resolve_current_row(&data.get());
        let emphasis = row_emphasis
            .with_value(|classifier| entity_row_emphasis_for(classifier.as_ref(), row.as_ref()));
        cached_row.set(row);
        if cached_emphasis.get_untracked() != emphasis {
            cached_emphasis.set(emphasis);
        }
    });

    view! {
        <tr
            data-row-key=key.clone()
            data-entity-row-key=key
            data-entity-visible-position=move || visible_position.get()
            tabindex=interactive.then_some(0)
            class=move || merge_classes!(
                if interactive { "cursor-pointer ld-focus-ring" } else { "" },
                if has_selection && is_row_selected(&selected_class_key) { "bg-base-200" } else { "" },
                entity_row_emphasis_row_class(cached_emphasis.get())
            )
            aria-selected=move || entity_row_aria_selected(has_selection, is_row_selected(&selected_aria_key))
            data-entity-row-emphasis=move || {
                has_row_emphasis.then(|| cached_emphasis.get().as_str())
            }
            on:click=move |event: web_sys::MouseEvent| {
                if event_origin_is_action(event.target()) {
                    return;
                }
                let ctrl = event.ctrl_key() || event.meta_key();
                let shift = event.shift_key();
                if let Some(selection) = selection {
                    let Some(proposed) = entity_selection_proposal(&click_key, ctrl, shift) else {
                        // Modified click with selection enabled: consumed,
                        // neither selects nor activates.
                        return;
                    };
                    selection.propose(proposed);
                    if let Some(callback) = on_row_activate {
                        callback.run(click_key.clone());
                    }
                    return;
                }
                if let Some(callback) = on_row_activate {
                    callback.run(click_key.clone());
                }
            }
            on:keydown=move |event: web_sys::KeyboardEvent| {
                if !(event.key() == "Enter" || event.key() == " ")
                    || event_origin_is_action(event.target())
                {
                    return;
                }
                let ctrl = event.ctrl_key() || event.meta_key();
                let shift = event.shift_key();
                if let Some(selection) = selection {
                    let Some(proposed) = entity_selection_proposal(&keydown_key, ctrl, shift) else {
                        return;
                    };
                    event.prevent_default();
                    selection.propose(proposed);
                    if let Some(callback) = on_row_activate {
                        callback.run(keydown_key.clone());
                    }
                    return;
                }
                if let Some(callback) = on_row_activate {
                    event.prevent_default();
                    callback.run(keydown_key.clone());
                }
            }
        >
            {move || {
                // Reads the same cached lookup the `<tr>` class and
                // `data-entity-row-emphasis` attribute above already read --
                // no second dataset scan here.
                let Some(row) = cached_row.get() else {
                    return ().into_any();
                };
                let emphasis = cached_emphasis.get();
                let preferences_value = preferences.get();
                let columns = column_store.with_value(|columns| {
                    ordered_columns(&preferences_value, columns)
                        .into_iter()
                        .filter(|column| {
                            !preferences_value.hidden_columns.contains(column.id)
                        })
                        .collect::<Vec<_>>()
                });
                render_row_cells(row, columns, compact_row.get_value(), emphasis)
            }}
        </tr>
    }
}

fn render_row_cells<T: Clone + 'static>(
    row: T,
    columns: Vec<EntityColumn<T>>,
    compact_row: Option<EntityRowRenderer<T>>,
    emphasis: EntityRowEmphasis,
) -> AnyView {
    let compact_view = compact_row
        .map(|renderer| renderer(&row))
        .unwrap_or_else(|| render_default_compact_row(&row, &columns));
    // Applied identically to every wide-layout cell and to the compact
    // single-cell wrapper below, so a totals rule reads the same in both
    // presentations -- they share one `<tr>`; only the cells differ.
    let emphasis_cell_class = entity_row_emphasis_cell_class(emphasis);
    let wide_cells = columns
        .iter()
        .cloned()
        .map(|column| {
            let cell = render_cell(&row, &column);
            let alignment = column.alignment;
            let tabular_numbers = column.tabular_numbers;
            view! {
                <td
                    class=move || merge_classes!(
                        "hidden border border-table-grid forced-colors:border-[CanvasText] lg:table-cell",
                        entity_alignment_class(alignment),
                        if tabular_numbers { "tabular-nums" } else { "" },
                        emphasis_cell_class
                    )
                    data-entity-column=column.id
                    data-entity-action=column.is_action.then_some("true")
                    data-entity-alignment=alignment.as_str()
                    data-entity-tabular-numbers=tabular_numbers.then_some("true")
                    on:click=move |event| {
                        if column.is_action {
                            event.stop_propagation();
                        }
                    }
                    on:keydown=move |event| {
                        if column.is_action {
                            event.stop_propagation();
                        }
                    }
                >
                    {cell}
                </td>
            }
        })
        .collect_view();

    view! {
        <td
            colspan=columns.len().max(1)
            class=merge_classes!(
                "border border-table-grid p-0 forced-colors:border-[CanvasText] lg:hidden",
                emphasis_cell_class
            )
        >
            <div class="p-3">{compact_view}</div>
        </td>
        {wide_cells}
    }
    .into_any()
}

fn visible_row_keys<T>(
    rows: &[T],
    columns: &[EntityColumn<T>],
    preferences: &EntityTablePreferences,
    current_page: usize,
    row_key: &dyn Fn(&T) -> String,
) -> Vec<String> {
    let indices = sorted_indices(rows, columns, &preferences.sort);
    let bounds = page_bounds(current_page, preferences.page_size, indices.len());
    indices[bounds]
        .iter()
        .map(|index| row_key(&rows[*index]))
        .collect()
}

fn focus_record_from_event(event: &web_sys::FocusEvent, scope: &str) -> Option<EntityFocusRecord> {
    let target = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())?;
    let action = target.closest("[data-entity-row-action]").ok().flatten()?;
    let row = target.closest("[data-entity-row-key]").ok().flatten()?;
    Some(EntityFocusRecord {
        scope: scope.to_owned(),
        row_key: row.get_attribute("data-entity-row-key")?,
        action_id: action.get_attribute("data-entity-row-action")?,
        visible_position: row
            .get_attribute("data-entity-visible-position")?
            .parse()
            .ok()?,
    })
}

fn focus_moved_from_record(region: NodeRef<leptos::html::Div>, record: &EntityFocusRecord) -> bool {
    let Some(region) = region.get_untracked() else {
        return false;
    };
    let Some(active) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element())
    else {
        return false;
    };
    if !active.is_connected() || active.tag_name().eq_ignore_ascii_case("body") {
        return false;
    }
    let Ok(Some(action)) = active.closest("[data-entity-row-action]") else {
        return true;
    };
    let Ok(Some(row)) = active.closest("[data-entity-row-key]") else {
        return true;
    };
    if !region.contains(Some(&active)) {
        return true;
    }
    if action.get_attribute("data-entity-row-action").as_deref() != Some(&record.action_id) {
        return true;
    }
    let same_key = row.get_attribute("data-entity-row-key").as_deref() == Some(&record.row_key);
    let same_rendered_position = row
        .get_attribute("data-entity-visible-position")
        .and_then(|position| position.parse::<usize>().ok())
        == Some(record.visible_position);
    !same_key && !same_rendered_position
}

fn focus_row_action(region: NodeRef<leptos::html::Div>, row_key: &str, action_id: &str) -> bool {
    let Some(region) = region.get_untracked() else {
        return false;
    };
    let Ok(actions) = region.query_selector_all("[data-entity-row-action]") else {
        return false;
    };
    for index in 0..actions.length() {
        let Some(action) = actions
            .item(index)
            .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
        else {
            continue;
        };
        if action.get_attribute("data-entity-row-action").as_deref() != Some(action_id) {
            continue;
        }
        let Ok(Some(row)) = action.closest("[data-entity-row-key]") else {
            continue;
        };
        if row.get_attribute("data-entity-row-key").as_deref() != Some(row_key) {
            continue;
        }
        let Ok(Some(candidate)) = action.query_selector(
            "button:not([disabled]):not([aria-disabled='true']), a[href]:not([aria-disabled='true']), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1']):not([aria-disabled='true'])",
        ) else {
            continue;
        };
        let rect = candidate.get_bounding_client_rect();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            continue;
        }
        if let Ok(candidate) = candidate.dyn_into::<web_sys::HtmlElement>() {
            return candidate.focus().is_ok();
        }
    }
    false
}

fn focus_table_region(region: NodeRef<leptos::html::Div>) {
    if let Some(region) = region.get_untracked() {
        let _ = region.focus();
    }
}

fn current_header(descriptors: &RwSignal<Vec<EntityHeaderDescriptor>>, column_id: &str) -> String {
    descriptors.with(|descriptors| {
        descriptors
            .iter()
            .find(|column| column.id == column_id)
            .map(|column| column.header.clone())
            .unwrap_or_default()
    })
}

fn current_column_header<T: 'static>(columns: ColumnStore<T>, column_id: &str) -> String {
    columns.with_value(|columns| {
        columns
            .iter()
            .find(|column| column.id == column_id)
            .map(|column| column.header.clone())
            .unwrap_or_default()
    })
}

fn format_move_label(template: &str, column: &str, position: usize, total: usize) -> String {
    template
        .replace("{column}", column)
        .replace("{position}", &position.to_string())
        .replace("{total}", &total.to_string())
}

fn sort_direction_text(direction: EntitySortDirection, texts: &EntityTableTexts) -> &str {
    match direction {
        EntitySortDirection::Ascending => &texts.ascending,
        EntitySortDirection::Descending => &texts.descending,
    }
}

fn sort_accessible_label(
    sort: &EntitySort,
    column_id: &str,
    header: &str,
    texts: &EntityTableTexts,
) -> String {
    let current = match (sort.direction_for(column_id), sort.priority_for(column_id)) {
        (Some(direction), Some(priority)) => texts
            .sort_current
            .replace("{direction}", sort_direction_text(direction, texts))
            .replace("{priority}", &priority.to_string())
            .replace("{total}", &sort.clauses().len().to_string()),
        _ => texts.sort_not_sorted.clone(),
    };
    let plain = match sort.direction_for(column_id) {
        Some(EntitySortDirection::Ascending) => texts.sort_plain_descending.clone(),
        Some(EntitySortDirection::Descending) => texts.sort_plain_system.clone(),
        None => texts.sort_plain_ascending.clone(),
    };
    let additive = match (sort.direction_for(column_id), sort.priority_for(column_id)) {
        (Some(EntitySortDirection::Ascending), Some(priority)) => texts
            .sort_change
            .replace("{priority}", &priority.to_string())
            .replace("{direction}", &texts.descending),
        (Some(EntitySortDirection::Descending), Some(priority)) => texts
            .sort_remove
            .replace("{priority}", &priority.to_string()),
        _ => texts
            .sort_add
            .replace("{priority}", &(sort.clauses().len() + 1).to_string()),
    };
    format!("{header}: {current}. {plain}. {additive}.")
}

fn sort_summary<T>(
    sort: &EntitySort,
    columns: &[EntityColumn<T>],
    texts: &EntityTableTexts,
) -> String {
    if sort.is_system() {
        return texts.system_order.clone();
    }
    let clauses = sort
        .clauses()
        .iter()
        .enumerate()
        .filter_map(|(index, clause)| {
            let column = columns.iter().find(|column| column.id == clause.column)?;
            Some(
                texts
                    .sort_clause
                    .replace("{priority}", &(index + 1).to_string())
                    .replace("{column}", &column.header)
                    .replace("{direction}", sort_direction_text(clause.direction, texts)),
            )
        })
        .collect::<Vec<_>>();
    texts
        .sort_summary
        .replace("{clauses}", &clauses.join(", then "))
}

fn entity_header_descriptors<T>(
    preferences: &EntityTablePreferences,
    columns: &[EntityColumn<T>],
) -> Vec<EntityHeaderDescriptor> {
    ordered_columns(preferences, columns)
        .into_iter()
        .filter(|column| !preferences.hidden_columns.contains(column.id))
        .map(|column| EntityHeaderDescriptor {
            id: column.id,
            header: column.header,
            sortable: column.sortable,
            resizable: column.resizable,
            min_width: column.min_width,
            initial_width: column.initial_width,
            alignment: column.alignment,
            tabular_numbers: column.tabular_numbers,
        })
        .collect()
}

fn entity_flexible_column_id(columns: &[EntityHeaderDescriptor]) -> Option<&'static str> {
    columns
        .iter()
        .rev()
        .find(|column| !column.resizable)
        .map(|column| column.id)
}

fn rendered_column_widths<T>(
    preferences: &EntityTablePreferences,
    columns: &[EntityColumn<T>],
) -> BTreeMap<String, u32> {
    let mut widths = preferences.column_widths.clone();
    for column in columns {
        if let Some(width) = column.initial_width {
            widths.entry(column.id.to_owned()).or_insert(width);
        }
    }
    widths
}

fn render_default_compact_row<T: 'static>(row: &T, columns: &[EntityColumn<T>]) -> AnyView {
    columns
        .iter()
        .cloned()
        .map(|column| {
            let cell = render_cell(row, &column);
            let alignment = column.alignment;
            let tabular_numbers = column.tabular_numbers;
            view! {
                <div
                    class="flex items-start justify-between gap-3 py-1"
                    data-entity-column=column.id
                    data-entity-action=column.is_action.then_some("true")
                    data-entity-alignment=alignment.as_str()
                    data-entity-tabular-numbers=tabular_numbers.then_some("true")
                >
                    <span class="text-xs font-medium uppercase tracking-wide text-base-content/60">
                        {column.header}
                    </span>
                    <span class=move || merge_classes!(
                        "min-w-0",
                        entity_compact_alignment_class(alignment),
                        if tabular_numbers { "tabular-nums" } else { "" }
                    )>{cell}</span>
                </div>
            }
        })
        .collect_view()
        .into_any()
}

fn render_cell<T: 'static>(row: &T, column: &EntityColumn<T>) -> AnyView {
    if let Some(renderer) = column.renderer.as_ref() {
        return renderer(row);
    }

    let text = (column.text)(row);
    let Some(presentation) = column.presentation.as_ref() else {
        return render_plain_cell(text, column.text_overflow, None);
    };
    let presentation_kind = match presentation {
        EntityCellPresentation::Badge(_) => "badge",
        EntityCellPresentation::Icon(_) => "icon",
        EntityCellPresentation::PrimarySecondary { .. } => "primary-secondary",
    };
    if text.is_empty() {
        return view! {
            <span
                data-entity-semantic-cell=presentation_kind
                data-entity-semantic-fallback="empty"
            ></span>
        }
        .into_any();
    }

    match presentation {
        EntityCellPresentation::Badge(mapper) => {
            let Some(badge) = mapper(row) else {
                return render_plain_cell(text, column.text_overflow, Some("plain"));
            };
            view! {
                <span
                    class="inline-flex min-w-0 max-w-full items-center forced-colors:text-[CanvasText]"
                    data-entity-semantic-cell="badge"
                >
                    <Badge
                        color=badge.color
                        style=badge.style
                        size=BadgeSize::Sm
                        class="max-w-full forced-colors:border-[CanvasText] forced-colors:text-[CanvasText]"
                    >
                        {text}
                    </Badge>
                </span>
            }
            .into_any()
        }
        EntityCellPresentation::Icon(mapper) => {
            let Some(icon) = mapper(row) else {
                return render_plain_cell(text, column.text_overflow, Some("plain"));
            };
            let icon_name = icon.name;
            let icon_name_marker = icon_name.clone();
            view! {
                <span
                    class="inline-flex min-w-0 max-w-full items-center justify-center forced-colors:text-[CanvasText]"
                    data-entity-semantic-cell="icon"
                    data-entity-icon-name=icon_name_marker
                >
                    <Icon
                        name=icon_name
                        color=icon.color.as_class().to_owned()
                        size=IconSize::Small
                        class="shrink-0 forced-colors:text-[CanvasText]"
                    />
                    <span class="sr-only">{text}</span>
                </span>
            }
            .into_any()
        }
        EntityCellPresentation::PrimarySecondary { primary, secondary } => {
            let primary_text = primary(row);
            let secondary_text = normalize_entity_secondary_text(secondary(row));
            render_primary_secondary_cell(text, primary_text, secondary_text, column.text_overflow)
        }
    }
}

fn render_primary_secondary_cell(
    accessible_text: String,
    primary: String,
    secondary: Option<String>,
    overflow: EntityTextOverflow,
) -> AnyView {
    let title = (!matches!(overflow, EntityTextOverflow::Wrap)).then(|| accessible_text.clone());
    view! {
        <span
            class="inline-flex min-w-0 max-w-full flex-col"
            title=title
            data-entity-semantic-cell="primary-secondary"
        >
            <span aria-hidden="true" class="flex min-w-0 max-w-full flex-col">
                <span
                    class="ld-text-body block min-w-0 max-w-full forced-colors:text-[CanvasText]"
                    style=entity_text_overflow_style(overflow)
                    data-entity-text-overflow=overflow.as_str()
                    data-entity-line-clamp=overflow.lines().map(|lines| lines.get())
                    data-entity-primary-secondary-line="primary"
                >
                    {primary}
                </span>
                {secondary.map(|secondary_text| {
                    view! {
                        <span
                            class="ld-text-caption block min-w-0 max-w-full text-base-content/75 forced-colors:text-[CanvasText]"
                            style=entity_text_overflow_style(overflow)
                            data-entity-text-overflow=overflow.as_str()
                            data-entity-line-clamp=overflow.lines().map(|lines| lines.get())
                            data-entity-primary-secondary-line="secondary"
                        >
                            {secondary_text}
                        </span>
                    }
                })}
            </span>
            <span class="sr-only">{accessible_text}</span>
        </span>
    }
    .into_any()
}

fn render_plain_cell(
    text: String,
    overflow: EntityTextOverflow,
    semantic_fallback: Option<&'static str>,
) -> AnyView {
    let title = (!matches!(overflow, EntityTextOverflow::Wrap)).then(|| text.clone());
    view! {
        <span
            class="block min-w-0 max-w-full"
            style=entity_text_overflow_style(overflow)
            title=title
            data-entity-text-overflow=overflow.as_str()
            data-entity-line-clamp=overflow.lines().map(|lines| lines.get())
            data-entity-semantic-fallback=semantic_fallback
        >
            {text}
        </span>
    }
    .into_any()
}

fn event_origin_is_action(target: Option<web_sys::EventTarget>) -> bool {
    target
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| {
            element
                .closest(
                    "button, a, input, select, textarea, [role='button'], [data-entity-action='true']",
                )
                .ok()
                .flatten()
        })
        .is_some()
}

fn separator_parent_width(target: Option<web_sys::EventTarget>) -> Option<f64> {
    target
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| element.parent_element())
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        .map(|element| f64::from(element.offset_width()))
}

fn restore_column_move_focus(
    event: web_sys::MouseEvent,
    column_id: &'static str,
    direction: EntityColumnMove,
) {
    let Some(root) = event
        .target()
        .or_else(|| event.current_target())
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| element.closest("[data-entity-table]").ok().flatten())
    else {
        return;
    };
    if let Ok(Some(anchor)) = root.query_selector("[data-entity-column-chooser]")
        && let Ok(anchor) = anchor.dyn_into::<web_sys::HtmlElement>()
    {
        let _ = anchor.focus();
    }
    let preferred_direction = match direction {
        EntityColumnMove::Earlier => "earlier",
        EntityColumnMove::Later => "later",
    };
    let fallback_direction = match direction {
        EntityColumnMove::Earlier => "later",
        EntityColumnMove::Later => "earlier",
    };
    request_animation_frame(move || {
        let Ok(nodes) = root.query_selector_all("[data-entity-column-move]") else {
            return;
        };
        for direction in [preferred_direction, fallback_direction] {
            for index in 0..nodes.length() {
                let Some(node) = nodes.item(index) else {
                    continue;
                };
                let Ok(element) = node.dyn_into::<web_sys::Element>() else {
                    continue;
                };
                if element.get_attribute("data-entity-column-order").as_deref() == Some(column_id)
                    && element.get_attribute("data-entity-column-move").as_deref()
                        == Some(direction)
                    && !element.has_attribute("disabled")
                    && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
                {
                    let _ = element.focus();
                    return;
                }
            }
        }
    });
}

fn finish_resize<T: 'static>(
    target: Option<web_sys::EventTarget>,
    pointer_id: i32,
    resize_drag: RwSignal<Option<ResizeDrag>>,
    column_widths: RwSignal<BTreeMap<String, u32>>,
    preferences: PreferenceState<T>,
) {
    if let Some(target) = target
        && let Ok(element) = target.dyn_into::<web_sys::Element>()
    {
        let _ = element.release_pointer_capture(pointer_id);
    }
    if let Some(drag) = resize_drag.get_untracked()
        && let Some(width) =
            column_widths.with_untracked(|widths| widths.get(&drag.column_id).copied())
    {
        column_widths.set(preferences.update_and_rendered_widths(|preferences| {
            preferences.column_widths.insert(drag.column_id, width);
        }));
    }
    resize_drag.set(None);
}
