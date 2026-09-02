//! Reactive renderer for the typed client-side table model.

use super::emphasis::{
    EntityRowEmphasis, EntityRowEmphasisClassifier, entity_row_emphasis_cell_class,
    entity_row_emphasis_for, entity_row_emphasis_row_class,
};
use super::focus_request::{
    EntityFocusRequest, EntityFocusRequestOutcome, EntityFocusRequestResolution,
    entity_focus_request_outcome,
};
use super::grouping::{
    EntityGroupActions, EntityGroupCollapseCause, EntityGroupCollapseProposal, EntityGroupKey,
    EntityGroupOrder, EntityGroupRun, EntityGroupTexts, EntityGroupedSection, EntityRowGroup,
    EntityRowGrouping, entity_group_header_colspan, entity_group_label, entity_group_meta,
    entity_grouped_order, entity_grouped_page_sections, entity_previous_group_key,
    propose_entity_group_collapse,
};
use super::identity::{
    entity_page_size_control_id, entity_selection_header_control_id,
    entity_selection_row_control_id, next_entity_control_id, normalize_entity_control_id,
    resolve_entity_control_id,
};
use super::model::{
    ENTITY_PAGE_SIZE_CHOICES, EntityColumnMove, EntityFocusRecord, EntityFocusTarget,
    EntityProjectionGrouping, SortedIndexCache, emit_normalized_preference_change,
    entity_table_display_projection_from_indices, focus_target, move_column, next_sort,
    next_sort_additive, normalize_preferences, ordered_columns, page_after_dataset_change,
    reset_columns, reset_sort, resolve_entity_page_size, set_preferred_width, toggle_hidden_column,
};
use super::multi_selection::{
    EntityTableMultiSelection, EntityTableSelectionCause, EntityTableSelectionProposal,
    SELECTION_COLUMN_TRACK_ID, SELECTION_COLUMN_TRACK_WIDTH, displayed_page_selection_state,
    displayed_row_label, off_page_selected_count, propose_entity_displayed_page_toggle,
    propose_entity_row_toggle, resolve_entity_selection_mode,
};
use super::paging::{EntityPagePlan, entity_displayed_run_lengths};
use super::selection::{
    EntityTableSelection, entity_row_aria_selected, entity_row_hover_class, entity_row_is_selected,
    entity_selection_proposal,
};
use super::storage::{load_preferences, save_preferences};
use super::types::{
    ENTITY_PAGE_SIZE_AUTO_VALUE, EntityCellPresentation, EntityColumn, EntityColumnAlignment,
    EntityColumnChooserTrigger, EntityColumnFilter, EntityColumnFilterPlacement,
    EntityColumnFilters, EntityColumnKind, EntityColumns, EntityCompactRow, EntityEmptyState,
    EntityPageSize, EntityPageSizeIntent, EntityRowKey, EntityRowRenderer, EntitySort,
    EntitySortDirection, EntityTableActionColumnPolicy, EntityTableDisplayProjection,
    EntityTablePreferenceOwnership, EntityTablePreferencePersistence, EntityTablePreferences,
    EntityTableTexts, EntityTableViewportFit, EntityTextOverflow, entity_alignment_class,
    entity_compact_alignment_class, entity_header_justify_class, entity_text_overflow_style,
    normalize_entity_secondary_text,
};
use crate::components::badge::{Badge, BadgeSize};
use crate::components::button::Button;
use crate::components::checkbox::{Checkbox, CheckboxSize};
use crate::components::data_table::{
    FALLBACK_HEADER_HEIGHT, FALLBACK_ROW_HEIGHT, MAX_COLUMN_WIDTH, PageSlot, StableColumnTrack,
    StableTableColGroup, auto_page_size_for_height, clamp_page, effective_min_width,
    keyboard_resized_width, page_window, stable_table_content_style,
};
use crate::components::icon::{Icon, IconSize};
use crate::components::menu::{Menu, MenuCheckItem};
use crate::components::pagination::Pagination;
use crate::components::select::Select;
use crate::merge_classes;
use leptos::prelude::*;
use leptos::tachys::reactive_graph::OwnedView;
use std::collections::{BTreeMap, BTreeSet};
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

/// Combines this table's stored intent with its transient measurement into the
/// one [`EntityPageSize`] every part of the render reads (ldui-5p06).
///
/// A thin adapter over
/// [`resolve_entity_page_size`](super::model::resolve_entity_page_size) that
/// reads the intent off the supplied preferences, so no call site has to pair
/// `page_size` with `page_size_mode` by hand.
pub(super) fn resolved_page_size(
    preferences: &EntityTablePreferences,
    auto_available: bool,
    measured_rows: Option<usize>,
) -> EntityPageSize {
    resolve_entity_page_size(
        preferences.page_size_mode,
        auto_available,
        preferences.page_size.max(1),
        measured_rows,
    )
}

/// Prepends the leading selection control track to the data-column tracks.
///
/// The control track is declared here rather than derived from `columns`
/// because it is not a column: it carries a fixed width, never resizes, and
/// can never become the flexible sink (that stays whichever data column
/// [`entity_flexible_column_id`] picked). It is likewise invisible to the
/// column chooser, the sort model, the filter vocabulary and the display
/// projection, because none of those ever see a track.
pub(super) fn entity_stable_tracks(
    has_selection_column: bool,
    data_tracks: Vec<StableColumnTrack>,
) -> Vec<StableColumnTrack> {
    let leading = has_selection_column
        .then(|| StableColumnTrack::new(SELECTION_COLUMN_TRACK_ID, SELECTION_COLUMN_TRACK_WIDTH));
    leading.into_iter().chain(data_tracks).collect()
}

/// Columns the empty-state message must span.
///
/// The leading selection control cell is not a column, but it IS a cell: the
/// message row is short by one without it, which leaves a ragged grid line
/// under the checkbox column.
pub(super) const fn entity_empty_state_colspan(
    visible_columns: usize,
    has_selection_column: bool,
) -> usize {
    let columns = if visible_columns == 0 {
        1
    } else {
        visible_columns
    };
    columns + if has_selection_column { 1 } else { 0 }
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

/// The rows one render displays, in the order it displays them.
///
/// Built once per dependency change and shared by `Rc`. On an ungrouped table
/// `indices` is the sort cache's own permutation (no copy) and both grouping
/// vectors are empty, so nothing about grouping costs an ungrouped table
/// anything.
struct EntityDisplayedOrder {
    /// Displayed source-row indices, sorted, then grouped, then
    /// collapse-filtered.
    indices: Rc<Vec<usize>>,
    /// Group key of each displayed index, parallel to `indices`. Empty when
    /// the table is not grouped.
    group_keys: Vec<String>,
    /// Every non-empty group in rank order, collapsed or not. Empty when the
    /// table is not grouped.
    runs: Vec<EntityGroupRun>,
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
    kind: EntityColumnKind,
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

/// Applies one rows-per-page selection, then reasserts the control's live DOM
/// value from the supplied preferences.
///
/// Both halves of the decision move in one preference update, so a controlled
/// consumer never observes an `Auto` intent paired with the previous numeric
/// size (or the reverse). `auto_available` mirrors whether this table opted
/// into viewport-fit paging: without it an `auto` request is ignored, exactly
/// as an unknown numeric request is.
pub(super) fn apply_page_size_change<T: 'static>(
    preferences: PreferenceState<T>,
    current_page: RwSignal<usize>,
    auto_available: bool,
    requested_value: &str,
    reassert_live_value: impl FnOnce(String),
) {
    if requested_value == ENTITY_PAGE_SIZE_AUTO_VALUE {
        if auto_available {
            preferences
                .update(|preferences| preferences.page_size_mode = EntityPageSizeIntent::Auto);
            current_page.set(0);
        }
    } else if let Ok(page_size) = requested_value.parse::<usize>()
        && ENTITY_PAGE_SIZE_CHOICES.contains(&page_size)
    {
        preferences.update(|preferences| {
            preferences.page_size = page_size;
            preferences.page_size_mode = EntityPageSizeIntent::Fixed;
        });
        current_page.set(0);
    }

    // Reread rather than echo the request: a controlled consumer may decline
    // or delay it. The measurement is deliberately omitted -- a control value
    // names the mode, never the measured row count.
    let supplied_value = preferences.with_untracked(|preferences| {
        resolved_page_size(preferences, auto_available, None).control_value()
    });
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
    /// Optional typed focus request for mutations this table never sees
    /// (`ldui-o0iw`).
    ///
    /// An editor beside the table that deletes the selected row destroys the
    /// control that had focus, so focus falls to `<body>` and the table's own
    /// `focusin`-seeded recovery has nothing to recover from. Supplying the
    /// stable successor here is how a page says "focus this row" without
    /// querying and focusing DOM this crate owns.
    ///
    /// Each [`EntityFocusRequest::id`] is applied at most once, so a signal
    /// that keeps reporting the same request cannot take focus back from the
    /// user later. The request is resolved against the rows this table is
    /// actually painting — after filtering, sorting, paging, grouping and
    /// collapse — and never against source order; a row that is not on screen
    /// takes the documented table-region fallback instead of a positional
    /// guess. A stale `scope` is rejected outright.
    ///
    /// Issue one only for a mutation that was **accepted**: it is an
    /// instruction to move focus, so a failed or declined mutation should
    /// issue nothing and leave the editor's own focus alone.
    #[prop(optional, into)]
    focus_request: Option<Signal<Option<EntityFocusRequest>>>,
    /// Reports what the table actually did with each `focus_request`, so a
    /// consumer can announce or log the real outcome rather than assume the
    /// request succeeded.
    #[prop(optional)]
    on_focus_request_resolved: Option<Callback<EntityFocusRequestResolution>>,
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
    /// Optional controlled checkbox multi-selection for bulk page actions
    /// (`ldui-nz6d`), keyed by the table's mandatory `row_key`.
    ///
    /// Supplying it renders a leading checkbox column plus a header checkbox.
    /// Every gesture emits exactly ONE
    /// [`EntityTableSelectionProposal`](super::EntityTableSelectionProposal)
    /// carrying the complete resulting key set -- never a stream of per-row
    /// deltas -- and nothing is applied until the caller's own signal changes.
    ///
    /// **The header checkbox governs the rows currently displayed, and only
    /// those.** Its state is computed over the keys this table is rendering
    /// right now, after filtering, sorting and paging: checked when every
    /// displayed row is selected, `indeterminate` when some but not all are,
    /// unchecked otherwise. Selected keys on other pages never tint it, and
    /// are carried through every proposal untouched; their count is announced
    /// in a live region instead.
    ///
    /// Mutually exclusive with `selection`. Supplying both is refused at
    /// construction rather than resolved by precedence, because silently
    /// honouring one would make a bulk-assignment workflow act on a single
    /// row, or a single-row workflow act on a set.
    #[prop(optional)]
    multi_selection: Option<EntityTableMultiSelection>,
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
    /// Optional controlled row grouping by stable group key (`ldui-iyfa`).
    ///
    /// Supplying it partitions the rendered rows into accessible sections, one
    /// `<tbody>` per group, each opened by a full-width
    /// `<th scope="colgroup">` heading carrying the caller's label plus
    /// optional compact metadata and actions. There is still exactly ONE
    /// global column header and one filter row: grouping never splits the
    /// table into one instance per group.
    ///
    /// Sorting and filtering are unchanged and explicit. Filters apply to
    /// child rows; a group whose rows are all filtered away has no heading
    /// left to render. Row sorting happens *within* groups -- grouping applies
    /// a stable partition by group rank over the table's own sort permutation
    /// -- and the section order is the caller's declared order unless an
    /// explicit [`EntityGroupOrder`] is selected.
    ///
    /// Pagination never lies about counts. Group headings are presentation
    /// rows: they are not records, they never enter the row-range summary, and
    /// they are not part of the displayed-page population the
    /// `multi_selection` header checkbox governs. A heading is only ever
    /// derived from a row that is on the page, so an expanded group's heading
    /// can never strand itself as the last visible row.
    #[prop(optional)]
    row_grouping: Option<EntityRowGrouping<T>>,
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
    /// Stable prefix every other framework-owned control derives its `id` and
    /// `name` from (`ldui-izkq`) — today the select-all checkbox and each
    /// row's selection checkbox.
    ///
    /// Row identity is the prefix plus the escape-encoded **stable row key**,
    /// never the row's position: an index-derived id re-points at a different
    /// row the moment the table sorts, filters, pages, groups or collapses,
    /// and an id that silently aliases to another row is worse than no id.
    ///
    /// When omitted, a process-unique prefix is minted per mounted instance,
    /// so two tables on one page still never share a control id. Supply your
    /// own when you want an id that is stable across builds — a mount-order
    /// counter is not. A supplied value is trimmed and escaped into
    /// `[A-Za-z0-9_-]`, because an `id` may not contain whitespace and a `.`
    /// or `#` breaks every selector built from it.
    ///
    /// `page_size_control_id` predates this prop and still wins outright for
    /// the rows-per-page select; supplying only `control_id` names that select
    /// too, and supplying neither leaves its own minted id untouched.
    #[prop(optional, into)]
    control_id: MaybeProp<String>,
    /// Emits the resolved rows-per-page decision whenever it changes.
    ///
    /// This is how a consumer learns the effective page size without measuring
    /// the DOM or keeping duplicate pagination state (ldui-5p06). Under
    /// `viewport_fit` it fires again after a resize changes the fitted row
    /// count. Persist [`EntityTablePreferences::page_size_mode`] and
    /// [`EntityTablePreferences::page_size`] from the preference-change
    /// callback instead — those are the explicit user choice; the row count
    /// reported here is transient presentation state.
    #[prop(optional)]
    on_page_size_resolved: Option<Callback<EntityPageSize>>,
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
    // Refused at construction, before a single node is built, with the
    // conflicting prop names in the message -- never resolved to one model by
    // an invisible precedence rule. This mirrors `resolve_preference_ownership`
    // below, which is how `EntityTable` already refuses its other incompatible
    // configuration.
    if let Err(message) =
        resolve_entity_selection_mode(selection.is_some(), multi_selection.is_some())
    {
        panic!("{message}");
    }
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
    // ── Controlled row grouping (ldui-iyfa) ──
    //
    // Split into individually `Copy` pieces so every closure below captures
    // exactly the part it reads. `has_grouping` is the single gate: false
    // renders the ungrouped body byte-for-byte as it did before this prop
    // existed -- one `<tbody>`, no headings, no `data-entity-group-*`
    // attributes anywhere.
    let has_grouping = row_grouping.is_some();
    let group_of: StoredValue<Option<EntityGroupKey<T>>, LocalStorage> = StoredValue::new_local(
        row_grouping
            .as_ref()
            .map(|model| Rc::clone(&model.group_of)),
    );
    let group_actions: StoredValue<Option<EntityGroupActions>, LocalStorage> =
        StoredValue::new_local(
            row_grouping
                .as_ref()
                .and_then(|model| model.actions.clone()),
        );
    let group_declarations: Signal<Vec<EntityRowGroup>, LocalStorage> = row_grouping
        .as_ref()
        .map_or_else(|| Signal::stored_local(Vec::new()), |model| model.groups);
    let group_order: Signal<EntityGroupOrder> = row_grouping.as_ref().map_or_else(
        || Signal::stored(EntityGroupOrder::Declared),
        |model| model.order,
    );
    let collapsible_groups = row_grouping
        .as_ref()
        .is_some_and(|model| model.collapsed.is_some());
    let collapsed_groups: Signal<BTreeSet<String>> = row_grouping
        .as_ref()
        .and_then(|model| model.collapsed)
        .unwrap_or_else(|| Signal::stored(BTreeSet::new()));
    let on_collapse_change = row_grouping
        .as_ref()
        .and_then(|model| model.on_collapse_change);
    let group_texts: Signal<EntityGroupTexts> = row_grouping.as_ref().map_or_else(
        || Signal::stored(EntityGroupTexts::default()),
        |model| model.texts,
    );
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
    // ── The one table control prefix (ldui-izkq) ──
    //
    // Minted ONCE per mounted instance, outside every reactive closure: a
    // prefix re-minted on each render would hand the same checkbox a new
    // `id`/`name` on every keystroke, which is worse than having none.
    let minted_control_id = next_entity_control_id();
    let table_control_id: Signal<String> =
        Signal::derive(move || resolve_entity_control_id(control_id.get(), &minted_control_id));
    let page_size_select_id: Signal<Option<String>> = Signal::derive(move || {
        Some(page_size_control_id.get().unwrap_or_else(|| {
            // A table that supplies neither prop keeps the exact id it has
            // always minted (ldui-kl55); only a usable `control_id` renames
            // this control, and it renames it into the same one scheme.
            if control_id
                .get()
                .as_deref()
                .and_then(normalize_entity_control_id)
                .is_some()
            {
                entity_page_size_control_id(&table_control_id.get())
            } else {
                default_page_size_control_id.clone()
            }
        }))
    });
    let configured_page_size =
        Signal::derive(move || preferences.with(|preferences| preferences.page_size.max(1)));
    // ── The one resolved page size (ldui-5p06) ──
    // Everything downstream -- the rendered body, the `Showing x-y of z`
    // summary, the rows-per-page control, the pager, and the exported display
    // projection -- reads this and nothing else. There is deliberately no
    // second "how many rows" signal for them to disagree over. A `Memo` rather
    // than a derived signal so the resolved value settles before any observer
    // (including `on_page_size_resolved`) sees it.
    let page_size: Memo<EntityPageSize> = Memo::new(move |_| {
        let measured = viewport_fit_enabled
            .then(|| measured_page_size.get())
            .flatten();
        preferences
            .with(|preferences| resolved_page_size(preferences, viewport_fit_enabled, measured))
    });
    // What choosing `Auto` would render right now, used for the control's
    // `Auto (n)` option label. Identical to `page_size` whenever auto is the
    // active intent, because it is the same resolution.
    let auto_page_size: Memo<EntityPageSize> = Memo::new(move |_| {
        let measured = viewport_fit_enabled
            .then(|| measured_page_size.get())
            .flatten();
        resolve_entity_page_size(
            EntityPageSizeIntent::Auto,
            viewport_fit_enabled,
            configured_page_size.get(),
            measured,
        )
    });

    if let Some(on_page_size_resolved) = on_page_size_resolved {
        Effect::new(move |_| on_page_size_resolved.run(page_size.get()));
    }

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
    let has_selection_column = multi_selection.is_some();
    // ── The one full-width span (ldui-ibjk, ldui-iyfa) ──
    //
    // The empty-state row and every group heading span the SAME derived
    // count -- the columns visible right now plus the leading selection cell
    // when one is rendered. Two independent colspan computations is how a
    // full-width row comes to be short by one and desync the declared
    // `<colgroup>` tracks, so there is deliberately only one.
    let visible_column_count = Signal::derive_local(move || {
        let preferences_value = preferences.get();
        column_store.with_value(|columns| {
            ordered_columns(&preferences_value, columns)
                .into_iter()
                .filter(|column| !preferences_value.hidden_columns.contains(column.id))
                .count()
        })
    });
    let body_colspan = Signal::derive_local(move || {
        entity_group_header_colspan(visible_column_count.get(), has_selection_column)
    });
    let stable_tracks = Signal::derive(move || {
        let widths = column_widths.get();
        let data_tracks = header_descriptors
            .get()
            .into_iter()
            .map(|column| {
                let track = StableColumnTrack::resolve(
                    column.id,
                    widths.get(column.id).copied().map(f64::from),
                    column.initial_width.or(column.min_width),
                );
                if flexible_column_id.get() == Some(column.id) {
                    track.flexible()
                } else {
                    track
                }
            })
            .collect::<Vec<_>>();
        entity_stable_tracks(has_selection_column, data_tracks)
    });

    // ── The one displayed row order (ldui-iyfa, extending ldui-5p06) ──
    //
    // Sorting, grouping and collapse resolve here exactly once. The rendered
    // body, the row-range summary, the pager, the displayed-page selection
    // population, the focus-recovery window and the display projection all
    // read this and nothing else. Recomputing "which rows are displayed" a
    // second time is precisely the bug class 5p06 fixed, and grouping (which
    // reorders rows AND can remove them) would have multiplied it.
    let displayed_order: Signal<Rc<EntityDisplayedOrder>, LocalStorage> =
        Signal::derive_local(move || {
            let rows = data.get();
            let columns = column_store.get_value();
            let preferences_value = preferences.get();
            let sorted = sorted_index_cache
                .try_update_value(|cache| {
                    cache.indices(
                        Rc::clone(&rows),
                        &columns,
                        &preferences_value.sort,
                        semantic_generation.get(),
                    )
                })
                .expect("entity-table sort cache is still mounted");
            let Some(group_of) = group_of.get_value() else {
                // Ungrouped: the cached permutation IS the displayed order,
                // shared by `Rc` rather than copied.
                return Rc::new(EntityDisplayedOrder {
                    indices: sorted,
                    group_keys: Vec::new(),
                    runs: Vec::new(),
                });
            };
            let group_key_of = |index: usize| group_of(&rows[index]);
            // Stable partition by group rank over the sort permutation, so row
            // sorting stays *within* groups.
            let grouped = entity_grouped_order(
                sorted.as_slice(),
                &group_key_of,
                &group_declarations.get(),
                group_order.get(),
                &collapsed_groups.get(),
            );
            Rc::new(EntityDisplayedOrder {
                indices: Rc::new(grouped.indices),
                group_keys: grouped.group_keys,
                runs: grouped.runs,
            })
        });
    // Data rows only. A group heading is a presentation row and never a
    // record, so it can never inflate this count, the row-range summary, or
    // the page count.
    let total_rows =
        Signal::derive_local(move || displayed_order.with(|order| order.indices.len()));
    // ── Provider-empty vs filtered-empty (ldui-g4nw) ──
    //
    // The table already knows both counts, so it never has to spend one
    // sentence on two different facts. `source_data` falls back to the
    // rendered snapshot, which keeps an ungoverned table correct: with no
    // separate source, a zero-row render genuinely IS provider-empty.
    let empty_state = Signal::derive_local(move || {
        EntityEmptyState::from_source_rows(source_data.with(|rows| rows.len()))
    });
    let empty_state_message = Signal::derive_local(move || {
        let state = empty_state.get();
        texts.with(|texts| texts.empty_state_message(state).to_owned())
    });
    // ── The one page plan (ldui-5in5, extending ldui-5p06) ──
    //
    // Page BOUNDARIES, like the page size before them, resolve exactly once.
    // A grouped table keeps a fitting group whole, so some pages deliberately
    // stop short of capacity and `page * capacity` stops naming the rows the
    // body paints; the pager, the footer range, the selection population, the
    // focus window and the export all read this plan instead of recomputing
    // that arithmetic. An ungrouped table gets the uniform plan, which is the
    // arithmetic it always had.
    let page_plan: Signal<Rc<EntityPagePlan>, LocalStorage> = Signal::derive_local(move || {
        let capacity = page_size.get().rows();
        displayed_order.with(|order| {
            if !has_grouping {
                return Rc::new(EntityPagePlan::uniform(order.indices.len(), capacity));
            }
            // Run lengths come from the DISPLAYED keys, so collapse and
            // filtering have already been applied -- group boundaries are
            // recomputed before paging, never after it.
            Rc::new(EntityPagePlan::grouped(
                &entity_displayed_run_lengths(&order.group_keys),
                capacity,
            ))
        })
    });
    let total_pages = Signal::derive_local(move || page_plan.with(|plan| plan.page_count()));
    let page_bounds_signal =
        Signal::derive_local(move || page_plan.with(|plan| plan.bounds(current_page.get())));
    let page_row_keys = Signal::derive_local(move || {
        let rows = data.get();
        let bounds = page_bounds_signal.get();
        let row_key = row_key.get_value();
        displayed_order.with(|order| {
            order.indices[bounds]
                .iter()
                .map(|index| row_key(&rows[*index]))
                .collect::<Vec<_>>()
        })
    });
    // Parallel to `page_row_keys`; empty on an ungrouped table.
    let page_group_keys = Signal::derive_local(move || {
        let bounds = page_bounds_signal.get();
        displayed_order.with(|order| {
            if order.group_keys.is_empty() {
                return Vec::new();
            }
            order.group_keys[bounds].to_vec()
        })
    });
    // Sections are derived FROM the page's rows, which is why an expanded
    // group's heading can never be stranded as the last visible row with its
    // children on the next page: the heading has no independent existence.
    let page_sections: Signal<Vec<EntityGroupedSection>, LocalStorage> =
        Signal::derive_local(move || {
            if !has_grouping {
                return Vec::new();
            }
            let bounds = page_bounds_signal.get();
            let previous =
                displayed_order.with(|order| entity_previous_group_key(&order.group_keys, &bounds));
            let is_last_page = current_page.get() + 1 >= total_pages.get().max(1);
            displayed_order.with(|order| {
                page_group_keys.with(|group_keys| {
                    page_row_keys.with(|row_keys| {
                        entity_grouped_page_sections(
                            &order.runs,
                            group_keys,
                            row_keys,
                            previous.as_deref(),
                            is_last_page,
                        )
                    })
                })
            })
        });

    // Clamps the page against the DISPLAYED row count, so collapsing the
    // groups that held the current page's rows lands on a real page instead of
    // an empty one past the end.
    Effect::new(move |_| {
        // Clamped against the PLAN, so a page that only existed under the old
        // group shape (or before a collapse removed its rows) lands on a real
        // page instead of an empty one past the end.
        let next_page = page_plan.with(|plan| plan.clamp(current_page.get_untracked()));
        if next_page != current_page.get_untracked() {
            current_page.set(next_page);
        }
    });

    // ── Controlled checkbox multi-selection (ldui-nz6d) ──
    //
    // "The rows currently displayed" is `page_row_keys` and nothing else --
    // the same signal the `<tbody>` iterates to paint rows, which is itself
    // derived from the one resolved `page_size` memo (ldui-5p06). Deriving
    // the header checkbox's population from a second, independently
    // recomputed page window is exactly the class of bug 5p06 fixed, so the
    // header and the body physically cannot disagree about which rows they
    // mean.
    let accepted_keys: Signal<BTreeSet<String>> = multi_selection.map_or_else(
        || Signal::stored(BTreeSet::new()),
        EntityTableMultiSelection::selected_keys,
    );
    let displayed_page_state = Signal::derive_local(move || {
        page_row_keys.with(|keys| {
            accepted_keys.with(|accepted| displayed_page_selection_state(keys, accepted))
        })
    });
    let off_page_selected = Signal::derive_local(move || {
        page_row_keys
            .with(|keys| accepted_keys.with(|accepted| off_page_selected_count(accepted, keys)))
    });
    // Defaults to the table's own dataset identity, so a dataset swap already
    // stamps a distinguishable scope without the caller wiring anything.
    let selection_scope: Signal<String> = multi_selection
        .and_then(|model| model.scope)
        .unwrap_or(dataset_identity);
    // `None` unless multi-selection was configured, so an unconfigured table
    // renders exactly the markup it always did -- no leading track, no
    // leading cells, no live region.
    let selection_header_ref = NodeRef::<leptos::html::Input>::new();
    let selection_header_id: Signal<String> =
        Signal::derive(move || entity_selection_header_control_id(&table_control_id.get()));
    let selection_header = multi_selection.map(|model| {
        let selection_texts = model.texts;
        view! {
            <th
                class="w-12 border border-table-grid bg-table-header p-2 text-center text-table-header-content forced-colors:border-[CanvasText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                scope="col"
                data-entity-selection-header="true"
                data-entity-selection-page-state=move || displayed_page_state.get().as_str()
            >
                <span class="sr-only">
                    {move || selection_texts.with(|texts| texts.column_header.clone())}
                </span>
                <Checkbox
                    size=CheckboxSize::Sm
                    node_ref=selection_header_ref
                    class="align-middle"
                    // `aria-disabled`, not the native `disabled`: an empty
                    // table's header checkbox stays in the tab order so a
                    // keyboard user still reaches it and hears WHY it is
                    // inert ("No rows are displayed"). The `on:change`
                    // handler below is the enforcement -- it returns without
                    // proposing when no rows are displayed.
                    attr:aria-disabled=move || {
                        displayed_page_state.get().is_disabled().then_some("true")
                    }
                    attr:data-entity-selection-toggle="page"
                    // An accessible NAME is not a DOM IDENTITY: without these
                    // the control is a form field with neither id nor name,
                    // unreferenceable from a `label[for]`, an `aria-controls`,
                    // or a form submission (ldui-izkq).
                    attr:id=move || selection_header_id.get()
                    attr:name=move || selection_header_id.get()
                    attr:aria-label=move || {
                        let state = displayed_page_state.get();
                        let count = page_row_keys.with(Vec::len);
                        selection_texts.with(|texts| texts.page_label(state, count))
                    }
                    prop:checked=move || displayed_page_state.get().is_checked()
                    // `indeterminate` has NO HTML attribute -- it exists only
                    // as a DOM property, so writing `indeterminate="true"` in
                    // markup would do nothing at all. `prop:` writes the
                    // property, and the `on:change` handler re-writes it via
                    // `set_indeterminate` because the browser clears it as
                    // soon as the user clicks.
                    prop:indeterminate=move || displayed_page_state.get().is_indeterminate()
                    on:change=move |_| {
                        let state = displayed_page_state.get_untracked();
                        // Controlled: re-assert accepted truth onto the
                        // element the browser just toggled, BEFORE proposing,
                        // so a declined or delayed proposal leaves no
                        // optimistic divergence to reconcile.
                        if let Some(input) = selection_header_ref.get_untracked() {
                            input.set_checked(state.is_checked());
                            input.set_indeterminate(state.is_indeterminate());
                        }
                        let keys = page_row_keys.get_untracked();
                        if keys.is_empty() {
                            return;
                        }
                        let selected = state.toggles_to_selected();
                        let accepted = accepted_keys.get_untracked();
                        model.on_change.run(EntityTableSelectionProposal {
                            keys: propose_entity_displayed_page_toggle(&accepted, &keys, selected),
                            cause: EntityTableSelectionCause::DisplayedPage { selected, keys },
                            scope: selection_scope.get_untracked(),
                        });
                    }
                />
            </th>
        }
    });
    let selection_status = multi_selection.map(|model| {
        let selection_texts = model.texts;
        view! {
            <p
                class="sr-only"
                role="status"
                aria-live="polite"
                data-entity-selection-summary="true"
            >
                {move || {
                    let total = accepted_keys.with(BTreeSet::len);
                    let off_page = off_page_selected.get();
                    selection_texts.with(|texts| texts.summary_label(total, off_page))
                }}
            </p>
        }
    });

    if let Some(on_display_projection) = on_display_projection {
        Effect::new(move |_| {
            let rows = data.get();
            let columns = column_store.get_value();
            let preferences_value = preferences.get();
            // The SAME displayed order the body paints, so an export can never
            // describe a different row set or a different order than the
            // screen -- including under grouping and collapse.
            let order = displayed_order.get();
            let declarations = has_grouping.then(|| group_declarations.get());
            let column_header = group_texts.with(|texts| texts.column_header.clone());
            let label_of = |key: &str| {
                declarations
                    .as_ref()
                    .map_or_else(|| key.to_owned(), |groups| entity_group_label(groups, key))
            };
            // Group identity travels with the export even though the visual
            // table has stopped repeating it in every row -- that suppression
            // is the whole point of grouping, and losing the fact on the way
            // out would trade one defect for another.
            let grouping = has_grouping.then(|| EntityProjectionGrouping {
                group_keys: order.group_keys.as_slice(),
                label_of: &label_of,
                column_header: column_header.as_str(),
            });
            on_display_projection.run(entity_table_display_projection_from_indices(
                rows.as_slice(),
                &columns,
                &preferences_value,
                order.indices.as_slice(),
                page_bounds_signal.get(),
                row_key.get_value().as_ref(),
                projection_action_columns,
                grouping,
            ));
        });
    }

    Effect::new(move |_| {
        let Some(record) = focus_record.get() else {
            return;
        };
        let current_scope = focus_scope.get();
        let source_rows = source_data.get();
        let row_key = row_key.get_value();
        let source_keys = source_rows
            .iter()
            .map(|row| row_key(row))
            .collect::<Vec<_>>();
        // The keys the body is actually painting, not a second independently
        // recomputed page window. Grouping reorders rows and collapse removes
        // them, so a recomputed window would recover focus onto a row that is
        // not on screen (ldui-iyfa); reading the painted set is also what
        // makes focus survive a collapse, an expansion, and a filtered row's
        // removal by exactly the same code path.
        let visible_keys = page_row_keys.get();
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

    // ── Caller-issued focus requests (ldui-o0iw) ──
    //
    // Deliberately a SECOND, independent path from the `focus_record` recovery
    // above: that one answers a mutation the table observed, this one answers a
    // mutation it never saw. Neither reads the other's state, so an external
    // request cannot disturb internal row-action recovery, and a request that
    // is refused leaves no residue behind.
    if let Some(focus_request) = focus_request {
        let applied_request = StoredValue::new(Option::<u64>::None);
        Effect::new(move |_| {
            let Some(request) = focus_request.get() else {
                return;
            };
            // One id, one application. A controlled signal that keeps reporting
            // the honored request must not take focus back from the user.
            if applied_request.get_value() == Some(request.id) {
                return;
            }
            applied_request.set_value(Some(request.id));
            let request_id = request.id;
            let report = move |outcome: EntityFocusRequestOutcome| {
                if let Some(on_focus_request_resolved) = on_focus_request_resolved {
                    on_focus_request_resolved.run(EntityFocusRequestResolution {
                        request_id,
                        outcome,
                    });
                }
            };
            // The keys the body is painting right now -- the same signal the
            // `<tbody>` iterates -- so the request is answered against the
            // filtered, sorted, paged, grouped presentation and never against
            // source order.
            let intent = entity_focus_request_outcome(
                &request,
                &focus_scope.get_untracked(),
                &page_row_keys.get_untracked(),
            );
            if matches!(intent, EntityFocusRequestOutcome::StaleScope) {
                report(intent);
                return;
            }
            // Focus as it stood when the request was observed, compared again
            // after the replacement paints. The element it names may well be
            // destroyed by then, which is exactly the case this serves.
            let focused_at_request = entity_active_element();
            request_animation_frame(move || {
                apply_entity_focus_request(table_region, intent, focused_at_request, 1, report);
            });
        });
    }

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
            let _ = page_size.get();
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
    // Keyboard reachability of the scroll container (ldui-0bwg). Interactive
    // rows carry the table's single roving tab stop, so the region itself
    // stays out of the tab order (`-1`: focusable by script for focus
    // recovery, never a second stop). Non-interactive rows have no stop at
    // all, and a region that scrolls (`viewport_fit`, or the compact layout
    // on a narrow viewport) would then be unreachable from the keyboard --
    // axe `scrollable-region-focusable` -- so the region is the stop instead.
    let region_tabindex = entity_region_tabindex(on_row_activate.is_some() || selection.is_some());

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
            data-entity-effective-page-size=move || page_size.get().rows().to_string()
            data-entity-configured-page-size=move || configured_page_size.get().to_string()
            data-entity-page-size-mode=move || {
                if page_size.get().is_auto() { "auto" } else { "fixed" }
            }
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
            <div
                class="flex shrink-0 flex-wrap items-center justify-end gap-2"
                data-entity-table-toolbar="true"
            >
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
                            <p class="px-2 pb-1 text-xs font-semibold text-base-content/75">
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
            {selection_status}

            <div
                node_ref=table_region
                class=region_class
                role="region"
                tabindex=region_tabindex
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
                <div style=move || {
                    // Compact mode ignores the desktop colgroup's forced
                    // min-width entirely -- it must fit its containing
                    // block, not the sum of desktop column tracks
                    // (ldui-ibjk). Desktop keeps the exact prior style.
                    (!compact_filter_layout.get())
                        .then(|| stable_table_content_style(&stable_tracks.get()))
                }>
                <table
                    class="table table-sm w-full border-collapse border border-table-grid"
                    class:table-fixed=move || !compact_filter_layout.get()
                    class:table-zebra=move || zebra.get()
                    data-entity-table-grid="true"
                    data-table-layout="stable"
                    data-entity-table-compact=move || compact_filter_layout.get().then_some("true")
                >
                    // The stable colgroup pins one `<col>` per desktop
                    // column. Hiding a compact `<td>` with `lg:hidden` does
                    // not stop its `<col>` track from claiming width, so
                    // compact mode must not emit the colgroup at all -- it
                    // is desktop-only geometry (ldui-ibjk).
                    {move || {
                        (!compact_filter_layout.get())
                            .then(|| view! { <StableTableColGroup tracks=stable_tracks /> })
                    }}
                    <thead class="hidden lg:table-header-group">
                        <tr>
                            {selection_header}
                            <For
                                each=move || header_descriptors.get()
                                key=|column| (
                                    column.id,
                                    column.sortable,
                                    column.resizable,
                                    column.min_width,
                                    column.initial_width,
                                    column.alignment,
                                    column.kind,
                                )
                                children=move |column| {
                                let column_id = column.id;
                                let sortable = column.sortable;
                                let resizable = column.resizable;
                                let minimum_width = column.min_width;
                                let alignment = column.alignment;
                                let kind = column.kind;
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
                                            kind.default_class().unwrap_or("")
                                        )
                                        scope="col"
                                        data-entity-column=column_id
                                        data-entity-alignment=alignment.as_str()
                                        data-entity-column-kind=kind.as_str()
                                        data-entity-tabular-numbers=(kind == EntityColumnKind::Numeric).then_some("true")
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
                                    {has_selection_column.then(|| view! {
                                        // Structural spacer: the selection
                                        // control column has no filter
                                        // vocabulary, and must not shift the
                                        // data columns' filters out from
                                        // under their own headers.
                                        <th
                                            class="border border-table-grid bg-table-filter p-1 forced-colors:border-[CanvasText] forced-colors:bg-[Canvas]"
                                            data-entity-selection-filter-cell="true"
                                        ></th>
                                    })}
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
                    // Ungrouped: exactly the single `<tbody>` this table has
                    // always rendered. Grouping costs an ungrouped table no
                    // markup at all.
                    {(!has_grouping).then(|| view! {
                        <tbody>
                            {move || page_row_keys.with(|keys| keys.is_empty()).then(|| view! {
                                <tr>
                                    <td
                                        colspan=move || body_colspan.get()
                                        data-entity-empty-state=move || empty_state.get().as_str()
                                        class="border border-table-grid py-10 text-center text-base-content/75 forced-colors:border-[CanvasText]"
                                    >
                                        {move || empty_state_message.get()}
                                    </td>
                                </tr>
                            })}
                            {local_for_enumerate(
                                move || page_row_keys.get(),
                                |key| key.clone(),
                                move |visible_position, key| render_keyed_row(
                                    key,
                                    visible_position.into(),
                                    KeyedRowContext {
                                        data,
                                        column_store,
                                        preferences,
                                        row_key,
                                        compact_row,
                                        on_row_activate,
                                        selection,
                                        multi_selection,
                                        accepted_keys,
                                        selection_scope,
                                        row_emphasis,
                                        table_control_id,
                                    },
                                ),
                            )}
                        </tbody>
                    })}
                    // Grouped: one `<tbody>` per rendered section (ldui-iyfa).
                    // `<tbody>` is already `role="rowgroup"`, so the section
                    // boundary IS the structural grouping; the heading names
                    // it, and `scope="colgroup"` is what attributes every
                    // child cell below to that heading without the label being
                    // repeated in a single data cell.
                    {has_grouping.then(|| {
                        let section_context = GroupSectionContext {
                            page_sections,
                            group_declarations,
                            group_actions,
                            collapsed_groups,
                            on_collapse_change,
                            collapsible_groups,
                            selection_scope,
                            body_colspan,
                            group_texts,
                            row: KeyedRowContext {
                                data,
                                column_store,
                                preferences,
                                row_key,
                                compact_row,
                                on_row_activate,
                                selection,
                                multi_selection,
                                accepted_keys,
                                selection_scope,
                                row_emphasis,
                                table_control_id,
                            },
                        };
                        view! {
                            {move || page_sections.with(Vec::is_empty).then(|| view! {
                                <tbody>
                                    <tr>
                                        <td
                                            colspan=move || body_colspan.get()
                                            data-entity-empty-state=move || empty_state.get().as_str()
                                            class="border border-table-grid py-10 text-center text-base-content/75 forced-colors:border-[CanvasText]"
                                        >
                                            {move || empty_state_message.get()}
                                        </td>
                                    </tr>
                                </tbody>
                            })}
                            // Keyed by section IDENTITY, not by its rows: a data
                            // change that leaves the sections themselves alone
                            // must not remount every `<tbody>` and tear the
                            // keyed rows inside it out of the DOM with it.
                            {local_for_enumerate(
                                move || page_sections.get(),
                                |section| (
                                    section.group_key.clone(),
                                    section.continued,
                                    section.collapsed,
                                ),
                                move |_position, section| render_group_section(
                                    section,
                                    section_context,
                                ),
                            )}
                        }
                    })}
                </table>
                </div>
            </div>

            <div
                class="flex shrink-0 flex-wrap items-center justify-between gap-3"
                data-entity-table-footer="true"
            >
                <div class="flex min-w-0 flex-wrap items-center gap-3">
                    // Stable hook for tests and consumers. Positional queries such as
                    // `[data-entity-table] label select` used to find this control
                    // because it lived in the toolbar; ldui-z0n1 moved it to the
                    // footer, so the first label-select in the table is now the
                    // status filter and those queries silently read the wrong
                    // element. Identity should not depend on document order.
                    <label data-entity-page-size-control="true" class="flex min-w-0 max-w-full flex-wrap items-center gap-2 text-sm text-base-content/75">
                        <span class="min-w-0 break-words">{move || texts.with(|texts| texts.rows_per_page.clone())}</span>
                        <Select
                            class="select-sm w-20 shrink-0"
                            id=page_size_select_id
                            name=page_size_select_id
                            label=Signal::derive(move || {
                                Some(texts.with(|texts| texts.rows_per_page.clone()))
                            })
                            value=Signal::derive(move || page_size.get().control_value())
                            node_ref=page_size_select
                            on_change=Callback::new(move |value: String| {
                                apply_page_size_change(
                                    preferences,
                                    current_page,
                                    viewport_fit_enabled,
                                    &value,
                                    move |supplied_value| {
                                        if let Some(select) = page_size_select.get() {
                                            select.set_value(&supplied_value);
                                        }
                                    },
                                );
                            })
                        >
                            // Auto is a rows-per-page CHOICE, not a silent
                            // override of one: its option carries the row
                            // count it currently resolves to, so the control
                            // can never read `25` over a five-row page.
                            {viewport_fit_enabled.then(|| view! {
                                <option value=ENTITY_PAGE_SIZE_AUTO_VALUE>
                                    {move || {
                                        texts.with(|texts| auto_page_size.get().control_label(texts))
                                    }}
                                </option>
                            })}
                            {ENTITY_PAGE_SIZE_CHOICES.into_iter().map(|choice| view! {
                                <option value=choice.to_string()>
                                    {EntityPageSize::fixed(choice).control_value()}
                                </option>
                            }).collect_view()}
                        </Select>
                    </label>
                    // Stable hook, for the same reason as the page-size control above:
                    // the row-range used to be reachable as the footer's
                    // last-child span, and ldui-z0n1 regrouped the footer so
                    // that query now returns null. Identity should not depend
                    // on document position.
                    <span data-entity-row-range="true" class="text-sm text-base-content/75">
                        {move || {
                            let total = total_rows.get();
                            if total == 0 {
                                return String::new();
                            }
                            // Read off the plan, never multiplied out of the
                            // page index: a grouped page can hold fewer rows
                            // than the capacity, and reciting `page * capacity`
                            // there is how a truthful count becomes a lie.
                            let (start, end) = page_plan.with(|plan| {
                                plan.row_range(current_page.get())
                            });
                            if start == 0 {
                                return String::new();
                            }
                            texts
                                .with(|texts| texts.row_range.clone())
                                .replace("{start}", &start.to_string())
                                .replace("{end}", &end.to_string())
                                .replace("{total}", &total.to_string())
                        }}
                    </span>
                </div>
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
                                    page_size.get_untracked().rows(),
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

static ENTITY_GROUP_ID: AtomicU64 = AtomicU64::new(0);

fn next_entity_group_id() -> String {
    format!(
        "ldui-entity-group-{}",
        ENTITY_GROUP_ID.fetch_add(1, Ordering::Relaxed)
    )
}

struct GroupSectionContext<T: 'static> {
    page_sections: Signal<Vec<EntityGroupedSection>, LocalStorage>,
    group_declarations: Signal<Vec<EntityRowGroup>, LocalStorage>,
    group_actions: StoredValue<Option<EntityGroupActions>, LocalStorage>,
    collapsed_groups: Signal<BTreeSet<String>>,
    on_collapse_change: Option<Callback<EntityGroupCollapseProposal>>,
    collapsible_groups: bool,
    selection_scope: Signal<String>,
    body_colspan: Signal<usize, LocalStorage>,
    group_texts: Signal<EntityGroupTexts>,
    row: KeyedRowContext<T>,
}

impl<T: 'static> Clone for GroupSectionContext<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for GroupSectionContext<T> {}

/// Renders one accessible group section: a `<tbody>` opened by a full-width
/// heading, followed by that group's rows.
///
/// # Why `<tbody>` plus `<th scope="colgroup">`
///
/// Two independent mechanisms, neither of which repeats the group label in a
/// data cell -- which is the defect being fixed:
///
/// - A `<tbody>` is already `role="rowgroup"`, so the section boundary is the
///   structural fact that these rows belong together. It carries
///   `aria-labelledby` pointing at the heading, so assistive technology that
///   names row groups reads the group's name on entry.
/// - The heading is a `<th scope="colgroup">` spanning every column. HTML's
///   own header-association algorithm applies a `colgroup`-scoped header to
///   the remaining cells in those columns, so a screen reader in table
///   navigation attributes each child cell to the heading automatically. No
///   per-row attribute, no repeated cell.
///
/// The alternative -- one table per group -- was refused outright: it
/// duplicates the column header and the filter row per group, which is exactly
/// what `ldui-iyfa` exists to stop consumers doing.
///
/// The heading row itself is never focusable and never `aria-selected`: it is
/// presentation, not a record. When the group is collapsible the heading holds
/// one ordinary `<button aria-expanded aria-controls>`, which is a single
/// normal tab stop inside the table region -- no trap, no roving state of its
/// own. Collapsed children are not rendered at all, so they leave the
/// accessibility tree rather than being painted and hidden.
fn render_group_section<T: Clone + 'static>(
    section: EntityGroupedSection,
    context: GroupSectionContext<T>,
) -> impl IntoView {
    let GroupSectionContext {
        page_sections,
        group_declarations,
        group_actions,
        collapsed_groups,
        on_collapse_change,
        collapsible_groups,
        selection_scope,
        body_colspan,
        group_texts,
        row,
    } = context;
    let group_key = section.group_key.clone();
    let continued = section.continued;
    let collapsed = section.collapsed;
    let group_row_count = section.group_row_count;
    let body_id = next_entity_group_id();
    let heading_id = format!("{body_id}-heading");
    let labelled_by = heading_id.clone();

    let label_key = group_key.clone();
    let label = move || {
        let declared = group_declarations.with(|groups| entity_group_label(groups, &label_key));
        group_texts.with(|texts| texts.heading(&declared, continued))
    };
    let meta_key = group_key.clone();
    let meta = move || {
        group_declarations
            .with(|groups| entity_group_meta(groups, &meta_key))
            .unwrap_or_else(|| group_texts.with(|texts| texts.row_count_label(group_row_count)))
    };
    let actions_key = group_key.clone();
    let actions = group_actions.with_value(|render| {
        let render = render.clone()?;
        group_declarations.with_untracked(|groups| {
            groups
                .iter()
                .find(|group| group.key() == actions_key)
                .map(|group| render(group))
        })
    });

    let toggle_label_key = group_key.clone();
    let toggle_label = move || {
        let declared =
            group_declarations.with(|groups| entity_group_label(groups, &toggle_label_key));
        group_texts.with(|texts| texts.toggle_label(&declared, collapsed))
    };
    let toggle_key = group_key.clone();
    let toggle_controls = body_id.clone();
    let heading_content = if collapsible_groups {
        view! {
            <button
                type="button"
                class="ld-focus-ring flex min-w-0 items-center gap-2 rounded-field px-1 text-left"
                data-entity-group-toggle=group_key.clone()
                aria-expanded=(!collapsed).to_string()
                aria-controls=toggle_controls
                aria-label=toggle_label
                on:click=move |event| {
                    let Some(on_collapse_change) = on_collapse_change else {
                        return;
                    };
                    restore_group_toggle_focus(event, toggle_key.clone());
                    let current = collapsed_groups.get_untracked();
                    on_collapse_change.run(EntityGroupCollapseProposal {
                        keys: propose_entity_group_collapse(&current, &toggle_key, !collapsed),
                        cause: EntityGroupCollapseCause::Group {
                            key: toggle_key.clone(),
                            collapsed: !collapsed,
                        },
                        scope: selection_scope.get_untracked(),
                    });
                }
            >
                <span
                    class="inline-flex shrink-0 transition-transform"
                    class:rotate-90=!collapsed
                    aria-hidden="true"
                >
                    <Icon name="chevron-right" size=IconSize::XSmall />
                </span>
                <span class="ld-text-body min-w-0 font-semibold">{label}</span>
            </button>
        }
        .into_any()
    } else {
        view! { <span class="ld-text-body min-w-0 font-semibold">{label}</span> }.into_any()
    };

    let rows_key = group_key.clone();
    let rows_continued = continued;
    let section_row_keys = move || {
        page_sections.with(|sections| {
            sections
                .iter()
                .find(|candidate| {
                    candidate.group_key == rows_key && candidate.continued == rows_continued
                })
                .map_or_else(Vec::new, |candidate| candidate.row_keys.clone())
        })
    };
    let base_key = group_key.clone();
    let base_position = Signal::derive_local(move || {
        page_sections.with(|sections| {
            sections
                .iter()
                .find(|candidate| {
                    candidate.group_key == base_key && candidate.continued == rows_continued
                })
                .map_or(0, |candidate| candidate.first_row_position)
        })
    });

    view! {
        <tbody
            id=body_id
            aria-labelledby=labelled_by
            data-entity-group=group_key.clone()
            data-entity-group-collapsed=collapsed.then_some("true")
        >
            <tr
                class="bg-table-filter text-table-filter-content forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                data-entity-group-header=group_key.clone()
                data-entity-group-continued=continued.then_some("true")
            >
                <th
                    id=heading_id
                    scope="colgroup"
                    colspan=move || body_colspan.get()
                    class="border border-table-grid px-3 py-2 text-left forced-colors:border-[CanvasText]"
                >
                    <div class="flex min-w-0 flex-wrap items-center gap-2">
                        {heading_content}
                        <span
                            class="ld-text-caption min-w-0 font-normal text-table-filter-content/75"
                            data-entity-group-meta="true"
                        >
                            {meta}
                        </span>
                        {actions.map(|actions| view! {
                            <span class="ml-auto flex shrink-0 items-center gap-2" data-entity-group-actions="true">
                                {actions}
                            </span>
                        })}
                    </div>
                </th>
            </tr>
            {(!collapsed).then(move || local_for_enumerate(
                section_row_keys,
                |key| key.clone(),
                move |position, key| render_keyed_row(
                    key,
                    Signal::derive_local(move || base_position.get() + position.get()),
                    row,
                ),
            ))}
        </tbody>
    }
}

struct KeyedRowContext<T: 'static> {
    data: Signal<Rc<Vec<T>>, LocalStorage>,
    column_store: ColumnStore<T>,
    preferences: PreferenceState<T>,
    row_key: StoredValue<EntityRowKey<T>, LocalStorage>,
    compact_row: CompactRowStore<T>,
    on_row_activate: Option<Callback<String>>,
    selection: Option<EntityTableSelection>,
    multi_selection: Option<EntityTableMultiSelection>,
    accepted_keys: Signal<BTreeSet<String>>,
    selection_scope: Signal<String>,
    row_emphasis: StoredValue<Option<EntityRowEmphasisClassifier<T>>, LocalStorage>,
    table_control_id: Signal<String>,
}

impl<T: 'static> Clone for KeyedRowContext<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for KeyedRowContext<T> {}

fn render_keyed_row<T: Clone + 'static>(
    key: String,
    visible_position: Signal<usize, LocalStorage>,
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
        multi_selection,
        accepted_keys,
        selection_scope,
        row_emphasis,
        table_control_id,
    } = context;
    // A table with only `selection` (no `on_row_activate`) is still
    // keyboard-operable, mirroring `data_table::row_is_interactive`.
    //
    // `multi_selection` deliberately does NOT make the whole row interactive:
    // its gesture lives entirely in the leading checkbox, which is already a
    // native, keyboard-operable control. Making the row a click target too
    // would mean a plain click both activated the row and toggled its
    // checkbox.
    let interactive = on_row_activate.is_some() || selection.is_some();
    // `aria-selected` and selected styling are gated on `selection` alone,
    // not `interactive`: an activate-only table (no `selection` supplied)
    // has no selection concept at all, so it must emit no `aria-selected`
    // attribute -- painting `aria-selected="false"` there would tell
    // assistive tech the row is selectable when it never was. This keeps
    // every existing `on_row_activate`-only caller's DOM byte-for-byte
    // unchanged.
    let has_selection = selection.is_some() || multi_selection.is_some();
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
    // Both models resolve to the same pure per-row key comparison, so neither
    // can alias a rendered position onto another entity's selection.
    let is_row_selected = move |current_key: &str| {
        if multi_selection.is_some() {
            return accepted_keys.with(|accepted| accepted.contains(current_key));
        }
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

    // ── The leading selection control cell (ldui-nz6d) ──
    //
    // Rendered in BOTH presentations (no `lg:` visibility switch): the wide
    // and compact layouts share this one `<tr>`, so a mobile user gets the
    // same checkbox rather than losing the affordance below the `lg`
    // breakpoint.
    let selection_cell = multi_selection.map(|model| {
        let selection_texts = model.texts;
        let checkbox_ref = NodeRef::<leptos::html::Input>::new();
        let dom_key = key.clone();
        let cell_key = key.clone();
        // Resolved on every read rather than captured once, so a reactive row
        // update (a renamed client) reaches the accessible name without
        // remounting the checkbox and stealing focus from under the user.
        let row_name = move || {
            if let Some(row_label) = model.row_label {
                return row_label.run(cell_key.clone());
            }
            // Default: the row's leading visible, non-action cell text, so a
            // screen reader hears the row's own name rather than "checkbox"
            // or a raw id.
            let primary = cached_row.with(|row| {
                let row = row.as_ref()?;
                let preferences_value = preferences.get();
                column_store.with_value(|columns| {
                    ordered_columns(&preferences_value, columns)
                        .into_iter()
                        .find(|column| {
                            !column.is_action
                                && !preferences_value.hidden_columns.contains(column.id)
                        })
                        .map(|column| (column.text)(row))
                })
            });
            displayed_row_label(&cell_key, primary.as_deref())
        };
        let accepted_key = key.clone();
        let is_accepted = move || accepted_keys.with(|accepted| accepted.contains(&accepted_key));
        // Identity from the STABLE ROW KEY, never the rendered position: the
        // leading cell is built per key, so there is no page index in scope to
        // reach for even by accident (ldui-izkq).
        let identity_key = key.clone();
        let row_control_id: Signal<String> = Signal::derive(move || {
            entity_selection_row_control_id(&table_control_id.get(), &identity_key)
        });

        let label_name = row_name;
        let label_accepted = is_accepted.clone();
        let checked_accepted = is_accepted.clone();
        let change_accepted = is_accepted;
        let change_key = key.clone();
        view! {
            <td
                class="w-12 border border-table-grid p-2 text-center align-middle forced-colors:border-[CanvasText]"
                data-entity-selection-cell="true"
                // The checkbox owns its gesture outright: without this the
                // same click would also reach the row's activation handler.
                on:click=move |event: web_sys::MouseEvent| event.stop_propagation()
                on:keydown=move |event: web_sys::KeyboardEvent| event.stop_propagation()
            >
                <Checkbox
                    size=CheckboxSize::Sm
                    node_ref=checkbox_ref
                    class="align-middle"
                    attr:data-entity-selection-row=dom_key
                    attr:id=move || row_control_id.get()
                    attr:name=move || row_control_id.get()
                    attr:aria-label=move || {
                        let name = label_name();
                        selection_texts.with(|texts| texts.row_label(&name, label_accepted()))
                    }
                    prop:checked=checked_accepted
                    on:change=move |_| {
                        let accepted_now = change_accepted();
                        // Same controlled re-assertion as the header: the
                        // browser already flipped the box, so put accepted
                        // truth back before proposing anything.
                        if let Some(input) = checkbox_ref.get_untracked() {
                            input.set_checked(accepted_now);
                        }
                        let selected = !accepted_now;
                        let accepted = accepted_keys.get_untracked();
                        model.on_change.run(EntityTableSelectionProposal {
                            keys: propose_entity_row_toggle(&accepted, &change_key, selected),
                            cause: EntityTableSelectionCause::Row {
                                key: change_key.clone(),
                                selected,
                            },
                            scope: selection_scope.get_untracked(),
                        });
                    }
                />
            </td>
        }
    });

    view! {
        <tr
            data-row-key=key.clone()
            data-entity-row-key=key
            data-entity-visible-position=move || visible_position.get()
            tabindex=interactive.then_some(0)
            class=move || {
                let selected = has_selection && is_row_selected(&selected_class_key);
                merge_classes!(
                    if interactive { "cursor-pointer ld-focus-ring" } else { "" },
                    if selected { "bg-base-200" } else { "" },
                    // See `entity_row_hover_class`'s doc comment for the
                    // hover-vs-selected precedence this encodes (ldui-jdzr).
                    entity_row_hover_class(interactive, selected),
                    entity_row_emphasis_row_class(cached_emphasis.get())
                )
            }
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
            {selection_cell}
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
            let kind = column.kind;
            view! {
                <td
                    class=move || merge_classes!(
                        "hidden border border-table-grid forced-colors:border-[CanvasText] lg:table-cell",
                        entity_alignment_class(alignment),
                        kind.default_class().unwrap_or(""),
                        emphasis_cell_class
                    )
                    data-entity-column=column.id
                    data-entity-action=column.is_action.then_some("true")
                    data-entity-alignment=alignment.as_str()
                    data-entity-column-kind=kind.as_str()
                    data-entity-tabular-numbers=(kind == EntityColumnKind::Numeric).then_some("true")
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

/// `tabindex` of the table's scroll region (ldui-0bwg). With interactive rows
/// the rows are the roving tab stop and the region must not add a second one;
/// without them the region is the only thing a keyboard user can land on to
/// scroll the table, so it joins the tab order.
pub(crate) fn entity_region_tabindex(rows_interactive: bool) -> &'static str {
    if rows_interactive { "-1" } else { "0" }
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

/// Focuses one row by stable key, if the table made that row focusable.
///
/// A display-only table has no focusable rows, so this returns `false` rather
/// than minting a tab stop the keyboard model never had; the caller then takes
/// the documented table-region fallback.
fn focus_row(region: NodeRef<leptos::html::Div>, row_key: &str) -> bool {
    let Some(region) = region.get_untracked() else {
        return false;
    };
    // Iterated and compared by attribute rather than interpolated into a
    // selector: a row key is an arbitrary consumer string, and building a
    // selector from one is a quoting bug waiting for the first key with a
    // quote or a bracket in it.
    let Ok(rows) = region.query_selector_all("[data-entity-row-key]") else {
        return false;
    };
    for index in 0..rows.length() {
        let Some(row) = rows
            .item(index)
            .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
        else {
            continue;
        };
        if row.get_attribute("data-entity-row-key").as_deref() != Some(row_key) {
            continue;
        }
        if row.get_attribute("tabindex").is_none() {
            return false;
        }
        if let Ok(row) = row.dyn_into::<web_sys::HtmlElement>() {
            return row.focus().is_ok();
        }
    }
    false
}

fn entity_active_element() -> Option<web_sys::Element> {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element())
}

/// Whether focus has moved to another meaningful target since a request was
/// observed.
///
/// `<body>` (and a detached element) is not a meaningful target: it is where
/// focus lands when the element that had it was destroyed, which is precisely
/// the situation a focus request exists to repair.
fn entity_focus_moved_since(at_request: Option<&web_sys::Element>) -> bool {
    let Some(active) = entity_active_element() else {
        return false;
    };
    if !active.is_connected() || active.tag_name().eq_ignore_ascii_case("body") {
        return false;
    }
    at_request != Some(&active)
}

/// Applies one resolved focus request after the accepted projection paints.
///
/// Re-queries by stable key rather than holding an element reference across the
/// replacement, and tries again on the following frame before falling back —
/// the requested row has just been destroyed and recreated, so the first frame
/// can legitimately arrive before its new element exists.
fn apply_entity_focus_request<F>(
    region: NodeRef<leptos::html::Div>,
    intent: EntityFocusRequestOutcome,
    focused_at_request: Option<web_sys::Element>,
    retries: usize,
    report: F,
) where
    F: Fn(EntityFocusRequestOutcome) + Clone + 'static,
{
    if entity_focus_moved_since(focused_at_request.as_ref()) {
        report(EntityFocusRequestOutcome::Declined);
        return;
    }
    let honored = match &intent {
        EntityFocusRequestOutcome::Row { row_key } => focus_row(region, row_key),
        EntityFocusRequestOutcome::RowAction { row_key, action_id } => {
            focus_row_action(region, row_key, action_id)
        }
        // The row is already known to be off the page: retrying cannot make it
        // appear, so take the fallback now.
        EntityFocusRequestOutcome::TableRegion
        | EntityFocusRequestOutcome::StaleScope
        | EntityFocusRequestOutcome::Declined => {
            focus_table_region(region);
            report(EntityFocusRequestOutcome::TableRegion);
            return;
        }
    };
    if honored {
        report(intent);
        return;
    }
    if retries > 0 {
        let next = intent;
        let next_focused = focused_at_request;
        let next_report = report.clone();
        request_animation_frame(move || {
            apply_entity_focus_request(region, next, next_focused, retries - 1, next_report);
        });
        return;
    }
    focus_table_region(region);
    report(EntityFocusRequestOutcome::TableRegion);
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
            kind: column.kind,
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
            let kind = column.kind;
            view! {
                <div
                    class="flex items-start justify-between gap-3 py-1"
                    data-entity-column=column.id
                    data-entity-action=column.is_action.then_some("true")
                    data-entity-alignment=alignment.as_str()
                    data-entity-column-kind=kind.as_str()
                    data-entity-tabular-numbers=(kind == EntityColumnKind::Numeric).then_some("true")
                >
                    <span class="text-xs font-medium uppercase tracking-wide text-base-content/75">
                        {column.header}
                    </span>
                    <span class=move || merge_classes!(
                        "min-w-0",
                        entity_compact_alignment_class(alignment),
                        kind.default_class().unwrap_or("")
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

/// Return focus to a group's collapse toggle after the section re-renders.
///
/// The heading takes `collapsed` by value, so toggling REPLACES the button
/// rather than mutating it; the browser then drops focus to `<body>` and a
/// keyboard user is thrown to the top of the document mid-task. Same shape as
/// the column-move case above (`ldui-9j16`), and the same remedy: re-query by
/// the stable `data-entity-group-toggle` key on the next frame, once the new
/// button exists. Keyed by group, so focus lands on the SAME group's toggle
/// rather than merely somewhere plausible.
fn restore_group_toggle_focus(event: web_sys::MouseEvent, group_key: String) {
    let Some(root) = event
        .target()
        .or_else(|| event.current_target())
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| element.closest("[data-entity-table]").ok().flatten())
    else {
        return;
    };
    request_animation_frame(move || {
        let selector = format!("[data-entity-group-toggle=\"{group_key}\"]");
        if let Ok(Some(button)) = root.query_selector(&selector)
            && let Ok(button) = button.dyn_into::<web_sys::HtmlElement>()
        {
            let _ = button.focus();
        }
    });
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
