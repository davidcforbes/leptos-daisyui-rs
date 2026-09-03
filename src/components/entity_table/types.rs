//! Public types used to configure a typed entity table.

use super::date_filter::{EntityDateBound, EntityDateFilterCause, EntityDateFilterProposal};
use super::draft_edit::{EntityCellEditor, EntityEditOutcome, EntityEditTarget};
use crate::components::badge::{BadgeColor, BadgeStyle};
use crate::components::input::{Input, InputSize, InputType};
use crate::components::select::{Select, SelectSize};
use leptos::html::{Input as HtmlInput, Select as HtmlSelect};
use leptos::prelude::{
    AddAnyAttr, AnyView, Callable, Callback, ClassAttribute, CollectView, ElementChild, Get,
    GetUntracked, GlobalAttributes, IntoAny, LocalStorage, NodeRef, Signal, view,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::num::NonZeroU8;
use std::rc::Rc;

/// A callback that renders one typed cell from a borrowed row.
pub type EntityCellRenderer<T> = Rc<dyn Fn(&T) -> AnyView>;

/// A callback that maps a typed row to an ordinary semantic badge treatment.
pub type EntityBadgeCell<T> = Rc<dyn Fn(&T) -> Option<EntityBadgePresentation>>;

/// A callback that maps a typed row to an ordinary semantic icon treatment.
pub type EntityIconCell<T> = Rc<dyn Fn(&T) -> Option<EntityIconPresentation>>;

/// A callback that renders the primary (first) line of a two-line canonical
/// text presentation from a borrowed row.
pub type EntityPrimaryTextCell<T> = Rc<dyn Fn(&T) -> String>;

/// A callback that renders the optional secondary (second) line of a
/// two-line canonical text presentation from a borrowed row. `None` (or an
/// empty/whitespace-only value) omits the secondary line entirely.
pub type EntitySecondaryTextCell<T> = Rc<dyn Fn(&T) -> Option<String>>;

/// Framework-owned visual treatment for a canonical text value.
pub enum EntityCellPresentation<T> {
    /// Render canonical text inside an LDUI badge when the mapper returns `Some`.
    Badge(EntityBadgeCell<T>),
    /// Render a decorative LDUI icon plus canonical screen-reader text.
    Icon(EntityIconCell<T>),
    /// Render an opinionated primary line plus an optional muted secondary
    /// line beneath it, in place of one plain canonical-text line.
    PrimarySecondary {
        /// Renders the primary line for a borrowed row.
        primary: EntityPrimaryTextCell<T>,
        /// Renders the optional secondary line for a borrowed row.
        secondary: EntitySecondaryTextCell<T>,
    },
}

impl<T> Clone for EntityCellPresentation<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Badge(mapper) => Self::Badge(Rc::clone(mapper)),
            Self::Icon(mapper) => Self::Icon(Rc::clone(mapper)),
            Self::PrimarySecondary { primary, secondary } => Self::PrimarySecondary {
                primary: Rc::clone(primary),
                secondary: Rc::clone(secondary),
            },
        }
    }
}

impl<T> fmt::Debug for EntityCellPresentation<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Badge(_) => "EntityCellPresentation::Badge(..)",
            Self::Icon(_) => "EntityCellPresentation::Icon(..)",
            Self::PrimarySecondary { .. } => "EntityCellPresentation::PrimarySecondary(..)",
        })
    }
}

/// Normalizes a caller-supplied optional secondary line, folding an empty or
/// whitespace-only value to `None`.
///
/// [`EntityColumn::primary_secondary`] renders no secondary line -- and
/// therefore no extra spacing or punctuation -- when this returns `None`.
pub(crate) fn normalize_entity_secondary_text(secondary: Option<String>) -> Option<String> {
    secondary.filter(|value| !value.trim().is_empty())
}

/// Framework-owned badge appearance. Visible text always comes from the
/// column's canonical text callback.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityBadgePresentation {
    /// daisyUI semantic badge color.
    pub color: BadgeColor,
    /// daisyUI badge surface treatment.
    pub style: BadgeStyle,
}

impl EntityBadgePresentation {
    /// Creates an opinionated soft badge in one semantic color.
    pub fn new(color: BadgeColor) -> Self {
        Self {
            color,
            style: BadgeStyle::Soft,
        }
    }

    /// Replaces the opinionated soft treatment.
    pub fn with_style(mut self, style: BadgeStyle) -> Self {
        self.style = style;
        self
    }
}

/// Semantic text color for an icon-only EntityTable cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityIconColor {
    /// Inherit the table's normal content color.
    #[default]
    Default,
    /// Neutral semantic foreground.
    Neutral,
    /// Primary brand foreground.
    Primary,
    /// Informational foreground.
    Info,
    /// Positive foreground.
    Success,
    /// Caution foreground.
    Warning,
    /// Error foreground.
    Error,
}

impl EntityIconColor {
    /// Static semantic utility used by the framework-owned icon.
    pub const fn as_class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Neutral => "text-neutral",
            Self::Primary => "text-primary",
            Self::Info => "text-info",
            Self::Success => "text-success",
            Self::Warning => "text-warning",
            Self::Error => "text-error",
        }
    }
}

/// Framework-owned icon treatment; canonical text remains its accessible label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityIconPresentation {
    /// Lucide-compatible LDUI icon name.
    pub name: String,
    /// Framework semantic foreground.
    pub color: EntityIconColor,
}

impl EntityIconPresentation {
    /// Creates one icon-only presentation.
    pub fn new(name: impl Into<String>, color: EntityIconColor) -> Self {
        Self {
            name: name.into(),
            color,
        }
    }
}

/// A callback that renders the compact representation of a borrowed row.
pub type EntityRowRenderer<T> = Rc<dyn Fn(&T) -> AnyView>;

/// A callback that renders one controlled filter beneath its stable column.
pub type EntityColumnFilterRenderer = Rc<dyn Fn() -> AnyView>;

type EntityControlledColumnFilterRenderer = Rc<dyn Fn(EntityColumnFilterPlacement) -> AnyView>;

#[derive(Clone, Copy)]
pub(crate) enum EntityColumnFilterPlacement {
    Header,
    Responsive,
}

/// Row scope selected from an atomic [`EntityTableDisplayProjection`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityTableProjectionScope {
    /// Rows on the table's current effective page.
    #[default]
    CurrentPage,
    /// Every locally filtered row in the table's current sort order.
    AllFiltered,
}

/// Whether action columns participate in a display projection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityTableActionColumnPolicy {
    /// Omit action columns because their canonical copy normally describes UI,
    /// not exported domain data.
    #[default]
    Exclude,
    /// Include action columns explicitly.
    Include,
}

/// One ordered visible column in an EntityTable display projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTableDisplayColumn {
    /// Stable column identity.
    pub id: String,
    /// Current reactive/localized column label.
    pub label: String,
    /// Whether the column is an action column included by explicit policy.
    pub is_action: bool,
}

impl EntityTableDisplayColumn {
    /// Creates one owned projected column descriptor.
    pub fn new(id: impl Into<String>, label: impl Into<String>, is_action: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            is_action,
        }
    }
}

/// One stable row and its canonical cell text in projected column order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTableDisplayRow {
    /// Stable row key supplied to the table.
    pub key: String,
    /// Canonical full text aligned with the projection's ordered columns.
    pub cells: Vec<String>,
    /// Stable group identity when the table is grouped (`ldui-iyfa`).
    ///
    /// This is the group KEY, never the display label -- the label already
    /// travels as the leading synthetic group cell in `cells`, so an export
    /// carries both the human column and the identity a re-import can join
    /// on. Absent on an ungrouped table, which keeps every pre-grouping
    /// serialized projection byte-identical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_key: Option<String>,
}

impl EntityTableDisplayRow {
    /// Creates one owned projected row.
    pub fn new<I, S>(key: impl Into<String>, cells: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            key: key.into(),
            cells: cells.into_iter().map(Into::into).collect(),
            group_key: None,
        }
    }

    /// Stamps the row's stable group identity.
    pub fn with_group_key(mut self, group_key: impl Into<String>) -> Self {
        self.group_key = Some(group_key.into());
        self
    }
}

/// Atomic, read-only projection of the table's displayed data model.
///
/// `rows(AllFiltered)` and `rows(CurrentPage)` share the same ordered row
/// storage, so callers cannot accidentally combine columns from one table
/// state with rows from another. The projection deliberately carries no
/// dataset identity or download policy.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTableDisplayProjection {
    /// Ordered visible columns used by every projected row.
    pub columns: Vec<EntityTableDisplayColumn>,
    all_filtered_rows: Vec<EntityTableDisplayRow>,
    current_page_start: usize,
    current_page_end: usize,
}

impl EntityTableDisplayProjection {
    pub(crate) fn from_parts(
        columns: Vec<EntityTableDisplayColumn>,
        all_filtered_rows: Vec<EntityTableDisplayRow>,
        current_page_start: usize,
        current_page_end: usize,
    ) -> Self {
        let end = current_page_end.min(all_filtered_rows.len());
        let start = current_page_start.min(end);
        Self {
            columns,
            all_filtered_rows,
            current_page_start: start,
            current_page_end: end,
        }
    }

    /// Returns rows for the explicitly selected export/display scope.
    pub fn rows(&self, scope: EntityTableProjectionScope) -> &[EntityTableDisplayRow] {
        match scope {
            EntityTableProjectionScope::CurrentPage => {
                &self.all_filtered_rows[self.current_page_start..self.current_page_end]
            }
            EntityTableProjectionScope::AllFiltered => &self.all_filtered_rows,
        }
    }

    /// Half-open bounds of the current page inside `AllFiltered` rows.
    pub fn current_page_bounds(&self) -> std::ops::Range<usize> {
        self.current_page_start..self.current_page_end
    }
}

/// A callback that returns the stable identity of a borrowed row.
pub type EntityRowKey<T> = Rc<dyn Fn(&T) -> String>;

/// A callback that compares two borrowed rows for one column.
pub type EntityComparator<T> = Rc<dyn Fn(&T, &T) -> Ordering>;

/// Prepared index comparator produced after typed keys have been extracted.
pub type EntityPreparedSortComparator = Rc<dyn Fn(usize, usize) -> Ordering>;

/// Type-erased factory for typed local sort keys.
///
/// Implementations extract one key per row and return an index comparator, so
/// the `O(n log n)` sort loop never reruns a consumer extractor. Most callers
/// use [`EntityColumn::sortable_by_key`] or
/// [`EntityColumn::sortable_by_optional_key`] rather than implementing this
/// trait directly.
pub trait EntitySortKeyFactory<T> {
    /// Extracts keys for `rows` and prepares comparison for one direction.
    fn prepare(&self, rows: &[T], direction: EntitySortDirection) -> EntityPreparedSortComparator;
}

/// A clonable, type-erased typed-key factory stored by [`EntityColumn`].
pub type EntitySortKey<T> = Rc<dyn EntitySortKeyFactory<T>>;

struct TypedEntitySortKey<T, K> {
    extract: Rc<dyn Fn(&T) -> K>,
}

impl<T: 'static, K: Ord + 'static> EntitySortKeyFactory<T> for TypedEntitySortKey<T, K> {
    fn prepare(&self, rows: &[T], direction: EntitySortDirection) -> EntityPreparedSortComparator {
        let keys = rows
            .iter()
            .map(|row| (self.extract)(row))
            .collect::<Vec<_>>();
        Rc::new(move |left, right| {
            let ordering = keys[left].cmp(&keys[right]);
            match direction {
                EntitySortDirection::Ascending => ordering,
                EntitySortDirection::Descending => ordering.reverse(),
            }
        })
    }
}

type OptionalEntitySortExtractor<T, K> = dyn Fn(&T) -> Option<K>;

struct OptionalEntitySortKey<T, K> {
    null_order: EntityNullOrder,
    extract: Rc<OptionalEntitySortExtractor<T, K>>,
}

impl<T: 'static, K: Ord + 'static> EntitySortKeyFactory<T> for OptionalEntitySortKey<T, K> {
    fn prepare(&self, rows: &[T], direction: EntitySortDirection) -> EntityPreparedSortComparator {
        let null_order = self.null_order;
        let keys = rows
            .iter()
            .map(|row| (self.extract)(row))
            .collect::<Vec<_>>();
        Rc::new(move |left, right| match (&keys[left], &keys[right]) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => match null_order {
                EntityNullOrder::First => Ordering::Less,
                EntityNullOrder::Last => Ordering::Greater,
            },
            (Some(_), None) => match null_order {
                EntityNullOrder::First => Ordering::Greater,
                EntityNullOrder::Last => Ordering::Less,
            },
            (Some(left), Some(right)) => {
                let ordering = left.cmp(right);
                match direction {
                    EntitySortDirection::Ascending => ordering,
                    EntitySortDirection::Descending => ordering.reverse(),
                }
            }
        })
    }
}

fn typed_entity_sort_key<T: 'static, K: Ord + 'static>(
    extract: impl Fn(&T) -> K + 'static,
) -> EntitySortKey<T> {
    Rc::new(TypedEntitySortKey {
        extract: Rc::new(extract),
    })
}

/// Absolute placement of absent optional sort keys.
///
/// The policy is preserved for both ascending and descending value order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityNullOrder {
    /// Missing values precede every present value.
    First,
    /// Missing values follow every present value.
    #[default]
    Last,
}

/// Static or reactive typed column declarations for [`EntityTable`](super::EntityTable).
///
/// `From<Vec<_>>` preserves historical call sites. The local reactive variant
/// supports `Rc` render/comparison callbacks that intentionally are not
/// `Send` while still updating mounted headers and compact copy.
pub enum EntityColumns<T: 'static> {
    /// Column declarations fixed for this component instance.
    Static(Vec<EntityColumn<T>>),
    /// Column declarations replaced reactively, typically on locale changes.
    Reactive(Signal<Vec<EntityColumn<T>>, LocalStorage>),
}

impl<T> Clone for EntityColumns<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(columns) => Self::Static(columns.clone()),
            Self::Reactive(columns) => Self::Reactive(*columns),
        }
    }
}

impl<T: 'static> From<Vec<EntityColumn<T>>> for EntityColumns<T> {
    fn from(columns: Vec<EntityColumn<T>>) -> Self {
        Self::Static(columns)
    }
}

impl<T: 'static> From<Signal<Vec<EntityColumn<T>>, LocalStorage>> for EntityColumns<T> {
    fn from(columns: Signal<Vec<EntityColumn<T>>, LocalStorage>) -> Self {
        Self::Reactive(columns)
    }
}

/// Static, reactive, or default compact-row rendering.
#[derive(Default)]
pub enum EntityCompactRow<T: 'static> {
    /// Use the framework's current-column compact renderer.
    #[default]
    Default,
    /// Use one renderer fixed for this component instance.
    Static(EntityRowRenderer<T>),
    /// Replace the renderer reactively, typically on locale changes.
    Reactive(Signal<EntityRowRenderer<T>, LocalStorage>),
}

impl<T: 'static> Clone for EntityCompactRow<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Default => Self::Default,
            Self::Static(renderer) => Self::Static(Rc::clone(renderer)),
            Self::Reactive(renderer) => Self::Reactive(*renderer),
        }
    }
}

impl<T: 'static> From<EntityRowRenderer<T>> for EntityCompactRow<T> {
    fn from(renderer: EntityRowRenderer<T>) -> Self {
        Self::Static(renderer)
    }
}

impl<T: 'static> From<Signal<EntityRowRenderer<T>, LocalStorage>> for EntityCompactRow<T> {
    fn from(renderer: Signal<EntityRowRenderer<T>, LocalStorage>) -> Self {
        Self::Reactive(renderer)
    }
}

/// One controlled filter rendered in the second header row.
#[derive(Clone)]
pub struct EntityColumnFilter {
    /// Stable target column identifier.
    pub column_id: &'static str,
    renderer: EntityColumnFilterRender,
    responsive: Option<EntityColumnFilterResponsive>,
    control_id: Option<Rc<str>>,
}

#[derive(Clone)]
enum EntityColumnFilterRender {
    Custom(EntityColumnFilterRenderer),
    Controlled(EntityControlledColumnFilterRenderer),
}

/// One stable submitted value and its current localized display label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityColumnFilterOption {
    /// Stable value proposed to the controlled filter owner.
    pub value: String,
    /// Current user-facing label for the stable value.
    pub label: String,
}

impl EntityColumnFilterOption {
    /// Creates a stable value/display-label pair.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone)]
struct EntityColumnFilterResponsive {
    label: Signal<String>,
    active: Signal<bool>,
    on_clear: Callback<()>,
}

impl EntityColumnFilter {
    /// Creates a filter renderer for one stable column.
    pub fn new(column_id: &'static str, render: impl Fn() -> AnyView + 'static) -> Self {
        Self {
            column_id,
            renderer: EntityColumnFilterRender::Custom(Rc::new(render)),
            responsive: None,
            control_id: None,
        }
    }

    /// Creates an opinionated controlled text filter.
    ///
    /// `control_id` must be unique within the document. The header control
    /// uses it verbatim and the responsive copy uses a deterministic suffix,
    /// so both presentations can coexist without duplicate DOM IDs. The
    /// callback only proposes replacements; the supplied `value` remains the
    /// sole source of truth for the control and active-filter state.
    pub fn text(
        column_id: &'static str,
        control_id: impl Into<String>,
        label: impl Into<Signal<String>>,
        value: impl Into<Signal<String>>,
        placeholder: impl Into<Signal<String>>,
        on_change: Callback<String>,
    ) -> Self {
        let control_id = Rc::<str>::from(control_id.into());
        assert_valid_entity_filter_control_id(&control_id);
        let label = label.into();
        let value = value.into();
        let placeholder = placeholder.into();
        let renderer_control_id = Rc::clone(&control_id);
        let renderer = Rc::new(move |placement| {
            let id = placed_entity_filter_control_id(&renderer_control_id, placement);
            let label_for = id.clone();
            let node_ref = NodeRef::<HtmlInput>::new();
            let restore_ref = node_ref;
            let controlled_change = Callback::new(move |next| {
                on_change.run(next);
                if let Some(input) = restore_ref.get() {
                    input.set_value(&value.get_untracked());
                }
            });
            view! {
                <label class="block w-full" for=label_for>
                    <span class="sr-only">{move || label.get()}</span>
                    <Input
                        size=InputSize::Xs
                        class="input-bordered w-full bg-table-filter text-table-filter-content"
                        value=value
                        placeholder=placeholder
                        on_input=controlled_change
                        node_ref=node_ref
                        attr:id=id
                        attr:data-entity-filter-control=column_id
                        attr:data-entity-filter-kind="text"
                        attr:data-entity-filter-placement=entity_filter_placement_name(placement)
                    />
                </label>
            }
            .into_any()
        });
        Self::controlled(
            column_id,
            control_id,
            label,
            Signal::derive(move || !value.get().is_empty()),
            Callback::new(move |()| on_change.run(String::new())),
            renderer,
        )
    }

    /// Creates an opinionated controlled single-select filter.
    ///
    /// The empty string is reserved for the localized `all_label` option.
    /// Option labels may be replaced reactively without changing the stable
    /// controlled value. As with [`Self::text`], proposals never mutate the
    /// supplied value inside the component.
    pub fn select(
        column_id: &'static str,
        control_id: impl Into<String>,
        label: impl Into<Signal<String>>,
        value: impl Into<Signal<String>>,
        all_label: impl Into<Signal<String>>,
        options: impl Into<Signal<Vec<EntityColumnFilterOption>>>,
        on_change: Callback<String>,
    ) -> Self {
        let control_id = Rc::<str>::from(control_id.into());
        assert_valid_entity_filter_control_id(&control_id);
        let label = label.into();
        let value = value.into();
        let all_label = all_label.into();
        let options = options.into();
        let renderer_control_id = Rc::clone(&control_id);
        let renderer = Rc::new(move |placement| {
            let id = placed_entity_filter_control_id(&renderer_control_id, placement);
            let label_for = id.clone();
            let node_ref = NodeRef::<HtmlSelect>::new();
            let restore_ref = node_ref;
            let controlled_change = Callback::new(move |next| {
                on_change.run(next);
                if let Some(select) = restore_ref.get() {
                    select.set_value(&value.get_untracked());
                }
            });
            view! {
                <label class="block w-full" for=label_for>
                    <span class="sr-only">{move || label.get()}</span>
                    <Select
                        size=SelectSize::Xs
                        class="select-bordered w-full bg-table-filter text-table-filter-content"
                        id=id
                        label=Signal::derive(move || Some(label.get()))
                        value=value
                        on_change=controlled_change
                        node_ref=node_ref
                        attr:data-entity-filter-control=column_id
                        attr:data-entity-filter-kind="select"
                        attr:data-entity-filter-placement=entity_filter_placement_name(placement)
                    >
                        <option value="">{move || all_label.get()}</option>
                        {move || {
                            options
                                .get()
                                .into_iter()
                                .filter(|option| !option.value.is_empty())
                                .map(|option| {
                                    view! { <option value=option.value>{option.label}</option> }
                                })
                                .collect_view()
                        }}
                    </Select>
                </label>
            }
            .into_any()
        });
        Self::controlled(
            column_id,
            control_id,
            label,
            Signal::derive(move || !value.get().is_empty()),
            Callback::new(move |()| on_change.run(String::new())),
            renderer,
        )
    }

    /// Creates an opinionated controlled date filter (`ldui-lx5t`).
    ///
    /// The control is a native `date` input in the shared filter styling, so
    /// it inherits the platform picker, keyboard operation and locale-aware
    /// *presentation* for free while its value stays the machine `YYYY-MM-DD`
    /// text every other layer already speaks. It carries the same contract as
    /// [`Self::text`] and [`Self::select`]: `control_id` is used verbatim in
    /// the header and with a deterministic suffix in the responsive copy,
    /// `value` is the sole source of truth, and `on_change` only proposes.
    ///
    /// Unlike the other two, the proposal is typed:
    /// [`EntityDateFilterProposal`] carries the complete resulting text, the
    /// already-interpreted [`EntityDateBound`], a typed
    /// [`EntityDateFilterCause`], and the column/control scope stamp -- so a
    /// caller wiring several date filters through one callback routes on
    /// identity rather than on call order, exactly as
    /// [`EntityTableSelectionProposal`](super::EntityTableSelectionProposal)
    /// does for selection.
    ///
    /// # What it compares
    ///
    /// Nothing, by itself. The filter row is a control surface; the caller
    /// applies the constraint to its own rows, and
    /// [`EntityDateFilter`] is the framework-owned predicate for doing that
    /// against an [`EntityDate`] accessor rather than against rendered cell
    /// text. Both range ends are inclusive; see that type for what an empty,
    /// half-open, impossible or unreadable filter does.
    ///
    /// # Unreadable values are visible, not silent
    ///
    /// A native picker cannot produce a bad value, but a value restored from
    /// a URL, a saved view or a migrated preference can be one. When `value`
    /// is neither empty nor a real `YYYY-MM-DD` day the control adds
    /// `aria-invalid`, the daisyUI `input-error` treatment, a
    /// `data-entity-filter-invalid` hook, and `invalid_hint` as its
    /// accessible description. This matters because the browser refuses to
    /// display an unparseable value in a `date` input at all -- without the
    /// error state an unreadable constraint would look exactly like no
    /// constraint while still hiding every row.
    ///
    /// The filter reads as active whenever `value` is non-empty, including
    /// while it is unreadable, so the responsive panel always offers the
    /// clear action that recovers from it.
    ///
    /// ### Add to `input.css`
    /// ```css
    /// @source inline("input input-xs input-error");
    /// ```
    pub fn date(
        column_id: &'static str,
        control_id: impl Into<String>,
        label: impl Into<Signal<String>>,
        value: impl Into<Signal<String>>,
        invalid_hint: impl Into<Signal<String>>,
        on_change: Callback<EntityDateFilterProposal>,
    ) -> Self {
        let control_id = Rc::<str>::from(control_id.into());
        assert_valid_entity_filter_control_id(&control_id);
        let label = label.into();
        let value = value.into();
        let invalid_hint = invalid_hint.into();
        let invalid = Signal::derive(move || EntityDateBound::parse(&value.get()).is_invalid());
        let renderer_control_id = Rc::clone(&control_id);
        let renderer = Rc::new(move |placement| {
            let id = placed_entity_filter_control_id(&renderer_control_id, placement);
            let label_for = id.clone();
            let hint_id = format!("{id}-invalid");
            let described_by = hint_id.clone();
            let proposal_control_id = renderer_control_id.to_string();
            let node_ref = NodeRef::<HtmlInput>::new();
            let restore_ref = node_ref;
            let controlled_change = Callback::new(move |next: String| {
                on_change.run(EntityDateFilterProposal::new(
                    next,
                    EntityDateFilterCause::Edited,
                    column_id,
                    proposal_control_id.clone(),
                ));
                if let Some(input) = restore_ref.get() {
                    input.set_value(&value.get_untracked());
                }
            });
            view! {
                <label class="block w-full" for=label_for>
                    <span class="sr-only">{move || label.get()}</span>
                    <Input
                        input_type=InputType::Date
                        size=InputSize::Xs
                        class="input-bordered w-full bg-table-filter text-table-filter-content"
                        value=value
                        on_input=controlled_change
                        node_ref=node_ref
                        attr:id=id
                        attr:data-entity-filter-control=column_id
                        attr:data-entity-filter-kind="date"
                        attr:data-entity-filter-placement=entity_filter_placement_name(placement)
                        attr:data-entity-filter-invalid=move || invalid.get().then_some("true")
                        attr:aria-invalid=move || invalid.get().then_some("true")
                        attr:aria-describedby=move || invalid.get().then(|| described_by.clone())
                        class:input-error=move || invalid.get()
                    />
                    <span id=hint_id class="sr-only">
                        {move || if invalid.get() { invalid_hint.get() } else { String::new() }}
                    </span>
                </label>
            }
            .into_any()
        });
        let clear_control_id = control_id.to_string();
        Self::controlled(
            column_id,
            control_id,
            label,
            Signal::derive(move || !value.get().trim().is_empty()),
            Callback::new(move |()| {
                on_change.run(EntityDateFilterProposal::new(
                    String::new(),
                    EntityDateFilterCause::Cleared,
                    column_id,
                    clear_control_id.clone(),
                ));
            }),
            renderer,
        )
    }

    fn controlled(
        column_id: &'static str,
        control_id: Rc<str>,
        label: Signal<String>,
        active: Signal<bool>,
        on_clear: Callback<()>,
        renderer: EntityControlledColumnFilterRenderer,
    ) -> Self {
        Self {
            column_id,
            renderer: EntityColumnFilterRender::Controlled(renderer),
            responsive: Some(EntityColumnFilterResponsive {
                label,
                active,
                on_clear,
            }),
            control_id: Some(control_id),
        }
    }

    /// Caller-owned base DOM ID for a typed controlled filter.
    ///
    /// Custom renderers return `None` because their markup remains entirely
    /// caller-owned.
    pub fn control_id(&self) -> Option<&str> {
        self.control_id.as_deref()
    }

    /// Adds localized compact/hidden-column presentation to a controlled filter.
    ///
    /// The caller continues to own the filter value. `active` discloses whether
    /// hiding this column would conceal an effective constraint, and `on_clear`
    /// supplies the matching consumer-owned reset intent.
    pub fn with_responsive(
        mut self,
        label: impl Into<Signal<String>>,
        active: impl Into<Signal<bool>>,
        on_clear: Callback<()>,
    ) -> Self {
        self.responsive = Some(EntityColumnFilterResponsive {
            label: label.into(),
            active: active.into(),
            on_clear,
        });
        self
    }

    /// Localized responsive label, falling back to the current column header.
    pub fn label(&self, fallback: &str) -> String {
        self.responsive
            .as_ref()
            .map_or_else(|| fallback.to_owned(), |state| state.label.get())
    }

    /// Whether this caller-owned filter currently constrains the result set.
    pub fn is_active(&self) -> bool {
        self.responsive
            .as_ref()
            .is_some_and(|state| state.active.get())
    }

    /// Emits the caller-owned clear intent when responsive metadata was supplied.
    pub fn clear(&self) {
        if let Some(state) = &self.responsive {
            state.on_clear.run(());
        }
    }

    pub(crate) fn clear_callback(&self) -> Option<Callback<()>> {
        self.responsive.as_ref().map(|state| state.on_clear)
    }

    pub(crate) fn render(&self, placement: EntityColumnFilterPlacement) -> AnyView {
        match &self.renderer {
            EntityColumnFilterRender::Custom(renderer) => renderer(),
            EntityColumnFilterRender::Controlled(renderer) => renderer(placement),
        }
    }
}

fn assert_valid_entity_filter_control_id(control_id: &str) {
    assert!(
        !control_id.trim().is_empty(),
        "EntityColumnFilter control_id must not be empty"
    );
}

fn placed_entity_filter_control_id(
    control_id: &str,
    placement: EntityColumnFilterPlacement,
) -> String {
    match placement {
        EntityColumnFilterPlacement::Header => control_id.to_owned(),
        EntityColumnFilterPlacement::Responsive => format!("{control_id}-responsive"),
    }
}

fn entity_filter_placement_name(placement: EntityColumnFilterPlacement) -> &'static str {
    match placement {
        EntityColumnFilterPlacement::Header => "header",
        EntityColumnFilterPlacement::Responsive => "responsive",
    }
}

impl fmt::Debug for EntityColumnFilter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EntityColumnFilter")
            .field("column_id", &self.column_id)
            .finish_non_exhaustive()
    }
}

/// Static, reactive, or absent aligned column filters.
#[derive(Clone, Default)]
pub enum EntityColumnFilters {
    /// No second header row.
    #[default]
    None,
    /// Filter declarations fixed for this component instance.
    Static(Vec<EntityColumnFilter>),
    /// Filter declarations replaced reactively without owning their values.
    Reactive(Signal<Vec<EntityColumnFilter>, LocalStorage>),
}

impl From<Vec<EntityColumnFilter>> for EntityColumnFilters {
    fn from(filters: Vec<EntityColumnFilter>) -> Self {
        Self::Static(filters)
    }
}

impl From<Signal<Vec<EntityColumnFilter>, LocalStorage>> for EntityColumnFilters {
    fn from(filters: Signal<Vec<EntityColumnFilter>, LocalStorage>) -> Self {
        Self::Reactive(filters)
    }
}

/// Direction of one clause in an [`EntitySort`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitySortDirection {
    /// Sort values from low to high.
    Ascending,
    /// Sort values from high to low.
    Descending,
}

impl EntitySortDirection {
    pub(crate) const fn aria_value(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }
}

/// One ordered column-and-direction clause in an [`EntitySort`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySortColumn {
    /// Stable column identifier.
    pub column: String,
    /// Direction applied after all preceding clauses compare equal.
    pub direction: EntitySortDirection,
}

impl EntitySortColumn {
    /// Creates an ascending clause for a column.
    pub fn ascending(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            direction: EntitySortDirection::Ascending,
        }
    }

    /// Creates a descending clause for a column.
    pub fn descending(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            direction: EntitySortDirection::Descending,
        }
    }
}

/// The table's ordered client-side sort clauses.
///
/// An empty clause list preserves the dataset's server-supplied system order.
/// Serialization always uses the canonical clause array. Deserialization also
/// accepts the historical `System`/`Ascending`/`Descending` enum payload so
/// legacy local-storage values migrate without a separate browser pass.
/// Historical single-column source patterns also remain available:
///
/// ```
/// use leptos_daisyui_rs::components::EntitySort;
///
/// let sort = EntitySort::ascending("status");
/// assert!(matches!(
///     sort,
///     EntitySort::Ascending { ref column } if column == "status"
/// ));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum EntitySort {
    /// Preserve the dataset's server-supplied order.
    #[default]
    System,
    /// Sort one column from low to high.
    Ascending {
        /// Stable column identifier.
        column: String,
    },
    /// Sort one column from high to low.
    Descending {
        /// Stable column identifier.
        column: String,
    },
    /// Apply two or more ordered sort clauses.
    Multiple {
        /// Clauses in primary-to-last priority order.
        clauses: Vec<EntitySortColumn>,
    },
}

impl Serialize for EntitySort {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.clauses().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EntitySort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WireSort {
            Canonical(Vec<EntitySortColumn>),
            Legacy(LegacySort),
        }

        #[derive(Deserialize)]
        enum LegacySort {
            System,
            Ascending { column: String },
            Descending { column: String },
        }

        Ok(match WireSort::deserialize(deserializer)? {
            WireSort::Canonical(clauses) => Self::multiple(clauses),
            WireSort::Legacy(LegacySort::System) => Self::System,
            WireSort::Legacy(LegacySort::Ascending { column }) => Self::ascending(column),
            WireSort::Legacy(LegacySort::Descending { column }) => Self::descending(column),
        })
    }
}

impl EntitySort {
    /// Creates an ascending sort for a column.
    pub fn ascending(column: impl Into<String>) -> Self {
        Self::Ascending {
            column: column.into(),
        }
    }

    /// Creates a descending sort for a column.
    pub fn descending(column: impl Into<String>) -> Self {
        Self::Descending {
            column: column.into(),
        }
    }

    /// Creates an ordered multi-column sort.
    pub fn multiple(clauses: impl IntoIterator<Item = EntitySortColumn>) -> Self {
        let mut clauses = clauses.into_iter();
        let Some(first) = clauses.next() else {
            return Self::System;
        };
        let Some(second) = clauses.next() else {
            return match first.direction {
                EntitySortDirection::Ascending => Self::ascending(first.column),
                EntitySortDirection::Descending => Self::descending(first.column),
            };
        };
        let mut multiple = vec![first, second];
        multiple.extend(clauses);
        Self::Multiple { clauses: multiple }
    }

    /// Returns the ordered clauses. An empty value means system order.
    pub fn clauses(&self) -> Vec<EntitySortColumn> {
        match self {
            Self::System => Vec::new(),
            Self::Ascending { column } => vec![EntitySortColumn::ascending(column.clone())],
            Self::Descending { column } => vec![EntitySortColumn::descending(column.clone())],
            Self::Multiple { clauses } => clauses.clone(),
        }
    }

    /// Returns whether the dataset remains in system order.
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System)
            || matches!(self, Self::Multiple { clauses } if clauses.is_empty())
    }

    /// Returns the primary clause, if the table is sorted.
    pub fn primary(&self) -> Option<EntitySortColumn> {
        match self {
            Self::System => None,
            Self::Ascending { column } => Some(EntitySortColumn::ascending(column.clone())),
            Self::Descending { column } => Some(EntitySortColumn::descending(column.clone())),
            Self::Multiple { clauses } => clauses.first().cloned(),
        }
    }

    /// Returns the clause for a column, if active.
    pub fn clause_for(&self, column_id: &str) -> Option<EntitySortColumn> {
        match self {
            Self::System => None,
            Self::Ascending { column } if column == column_id => {
                Some(EntitySortColumn::ascending(column.clone()))
            }
            Self::Descending { column } if column == column_id => {
                Some(EntitySortColumn::descending(column.clone()))
            }
            Self::Multiple { clauses } => clauses
                .iter()
                .find(|clause| clause.column == column_id)
                .cloned(),
            Self::Ascending { .. } | Self::Descending { .. } => None,
        }
    }

    /// Returns a column's one-based sort priority, if active.
    pub fn priority_for(&self, column_id: &str) -> Option<usize> {
        match self {
            Self::System => None,
            Self::Ascending { column } | Self::Descending { column } if column == column_id => {
                Some(1)
            }
            Self::Multiple { clauses } => clauses
                .iter()
                .position(|clause| clause.column == column_id)
                .map(|index| index + 1),
            Self::Ascending { .. } | Self::Descending { .. } => None,
        }
    }

    /// Returns a column's active direction, if any.
    pub fn direction_for(&self, column_id: &str) -> Option<EntitySortDirection> {
        self.clause_for(column_id).map(|clause| clause.direction)
    }

    /// Returns the primary column, or `None` for system order.
    pub fn column(&self) -> Option<&str> {
        match self {
            Self::System => None,
            Self::Ascending { column } | Self::Descending { column } => Some(column),
            Self::Multiple { clauses } => clauses.first().map(|clause| clause.column.as_str()),
        }
    }

    /// Returns the WAI-ARIA sort value for the actively sorted header cell.
    ///
    /// Inactive and non-sortable headers omit `aria-sort`; assistive
    /// technology only needs the state on the one active sort column.
    pub fn aria_value_for(&self, column_id: &str) -> Option<&'static str> {
        match self {
            Self::Ascending { column } if column == column_id => Some("ascending"),
            Self::Descending { column } if column == column_id => Some("descending"),
            Self::Multiple { clauses } => clauses
                .first()
                .filter(|clause| clause.column == column_id)
                .map(|clause| clause.direction.aria_value()),
            Self::System | Self::Ascending { .. } | Self::Descending { .. } => None,
        }
    }

    /// Returns an accessible label describing the next sort state.
    pub fn next_label(&self, column_id: &str) -> &'static str {
        match self.direction_for(column_id) {
            Some(EntitySortDirection::Ascending) => "Sort descending",
            Some(EntitySortDirection::Descending) => "Restore system order",
            None => "Sort ascending",
        }
    }

    /// Describes the column's current direction and multi-sort priority.
    pub fn current_label(&self, column_id: &str) -> String {
        match (self.direction_for(column_id), self.priority_for(column_id)) {
            (Some(direction), Some(priority)) => format!(
                "Currently sorted {} at priority {priority} of {}",
                direction.aria_value(),
                self.clauses().len()
            ),
            _ => "Not currently sorted".to_owned(),
        }
    }

    /// Describes the result of activating a sort button without Shift.
    pub fn plain_action_label(&self, column_id: &str) -> &'static str {
        match self.direction_for(column_id) {
            Some(EntitySortDirection::Ascending) => "Activate to sort descending as the only sort",
            Some(EntitySortDirection::Descending) => "Activate to restore system order",
            None => "Activate to sort ascending as the only sort",
        }
    }

    /// Describes the result of Shift-activating a sort button.
    pub fn additive_action_label(&self, column_id: &str) -> String {
        match (self.direction_for(column_id), self.priority_for(column_id)) {
            (Some(EntitySortDirection::Ascending), Some(priority)) => {
                format!("Shift+activate to change priority {priority} to descending")
            }
            (Some(EntitySortDirection::Descending), Some(priority)) => {
                format!("Shift+activate to remove priority {priority}")
            }
            _ => format!(
                "Shift+activate to add ascending at priority {}",
                self.clauses().len() + 1
            ),
        }
    }
}

/// Framework-owned overflow policy for a plain [`EntityColumn`] text value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityTextOverflow {
    /// Wrap naturally, including at otherwise-unbreakable content.
    #[default]
    Wrap,
    /// Keep one line and show an ellipsis when the declared column is narrower.
    Ellipsis,
    /// Clip to a positive number of visual lines.
    LineClamp(NonZeroU8),
}

impl EntityTextOverflow {
    /// Stable runtime marker emitted by the default cell renderer.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wrap => "wrap",
            Self::Ellipsis => "ellipsis",
            Self::LineClamp(_) => "line-clamp",
        }
    }

    /// Positive line count for a clamp policy.
    pub const fn lines(self) -> Option<NonZeroU8> {
        match self {
            Self::LineClamp(lines) => Some(lines),
            Self::Wrap | Self::Ellipsis => None,
        }
    }
}

pub(crate) fn entity_text_overflow_style(overflow: EntityTextOverflow) -> String {
    match overflow {
        EntityTextOverflow::Wrap => {
            "min-width:0;max-width:100%;overflow-wrap:anywhere;white-space:normal;".to_owned()
        }
        EntityTextOverflow::Ellipsis => "display:block;min-width:0;max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;".to_owned(),
        EntityTextOverflow::LineClamp(lines) => format!(
            "display:-webkit-box;min-width:0;max-width:100%;overflow:hidden;overflow-wrap:anywhere;-webkit-box-orient:vertical;-webkit-line-clamp:{};",
            lines.get()
        ),
    }
}

/// Horizontal presentation of an EntityTable column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EntityColumnAlignment {
    /// Compatibility default: start-aligned wide content and end-aligned
    /// values in the framework's compact label/value rows.
    #[default]
    Auto,
    /// Align headers and values to the inline start edge.
    Start,
    /// Center headers and values.
    Center,
    /// Align headers and values to the inline end edge.
    End,
}

impl EntityColumnAlignment {
    /// Stable marker emitted for browser audits.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

pub(crate) const fn entity_alignment_class(alignment: EntityColumnAlignment) -> &'static str {
    match alignment {
        EntityColumnAlignment::Auto | EntityColumnAlignment::Start => "text-left",
        EntityColumnAlignment::Center => "text-center",
        EntityColumnAlignment::End => "text-right",
    }
}

pub(crate) const fn entity_compact_alignment_class(
    alignment: EntityColumnAlignment,
) -> &'static str {
    match alignment {
        EntityColumnAlignment::Auto | EntityColumnAlignment::End => "text-right",
        EntityColumnAlignment::Start => "text-left",
        EntityColumnAlignment::Center => "text-center",
    }
}

pub(crate) const fn entity_header_justify_class(alignment: EntityColumnAlignment) -> &'static str {
    match alignment {
        EntityColumnAlignment::Auto | EntityColumnAlignment::Start => "justify-start",
        EntityColumnAlignment::Center => "justify-center",
        EntityColumnAlignment::End => "justify-end",
    }
}

/// Semantic presentation kind for an [`EntityColumn`]'s cells.
///
/// [`EntityColumn::new`]/[`EntityColumn::text`] default every column to
/// [`EntityColumnKind::Text`], the plain rendering the component has always
/// had. [`EntityColumn::numeric`] and [`EntityColumn::identifier`] opt a
/// column into a presentation the component owns, instead of hand-writing
/// the same Tailwind utilities at every call site -- `DataTable`'s sibling
/// `ColumnKind` (`ldui-lrig`) measured one consumer at 43 `with_class` calls
/// carrying `tabular-nums` and 20 carrying `font-mono`; `EntityTable` had
/// the same fact traveling as an ad-hoc `tabular_numbers: bool` (`ldui-no94`).
///
/// Unlike `DataTable::Column`, `EntityColumn` never exposes a raw CSS class
/// escape hatch -- every visual decision (alignment, emphasis, this kind) is
/// a narrow framework-owned enum, by design (see `emphasis.rs`). So there is
/// no `with_class`/`effective_class` override pair to mirror here: a kind's
/// contributed token composes with the already-independent
/// [`EntityColumn::alignment`] field exactly as it always has, and a caller
/// overrides a kind's effect on alignment the same way any builder call
/// overrides an earlier one -- by calling `.align_start()`/`.align_center()`/
/// `.align_end()` *after* `.numeric()`. See [`EntityColumn::numeric`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EntityColumnKind {
    /// Plain text rendering; contributes no default class. The default.
    #[default]
    Text,
    /// A number: tabular (monospaced) figures so digits line up
    /// column-to-column -- money, counts, percentages, durations. Set via
    /// [`EntityColumn::numeric`] (which also right-aligns) or the lower-level
    /// [`EntityColumn::tabular_numbers`] (figures only, alignment untouched).
    Numeric,
    /// An identifier: the theme's declared monospace face, so every
    /// character occupies equal width and visually similar glyphs (`0`/`O`,
    /// `1`/`l`) stay distinguishable -- ids, codes, hashes, SKUs. Set via
    /// [`EntityColumn::identifier`].
    Identifier,
}

impl EntityColumnKind {
    /// Stable marker emitted as `data-entity-column-kind` for browser audits.
    pub const fn as_str(self) -> &'static str {
        match self {
            EntityColumnKind::Text => "text",
            EntityColumnKind::Numeric => "numeric",
            EntityColumnKind::Identifier => "identifier",
        }
    }

    /// The Tailwind utility class this kind contributes by default. `None`
    /// for [`EntityColumnKind::Text`] -- it changes nothing. This is an
    /// additive token composed with [`entity_alignment_class`] at render
    /// time, not a whole-class override -- see the type's doc comment for
    /// why `EntityColumn` has no override slot to mirror `DataTable::Column`'s
    /// `effective_class`.
    pub(crate) const fn default_class(self) -> Option<&'static str> {
        match self {
            EntityColumnKind::Text => None,
            EntityColumnKind::Numeric => Some("tabular-nums"),
            // Resolves against the theme's `--font-mono` variable
            // (`ThemeConfig::font_family.monospace` in `src/theme`), so this
            // tracks a themed consumer's declared mono face rather than
            // hardcoding one.
            EntityColumnKind::Identifier => Some("font-mono"),
        }
    }
}

/// Column behavior and borrowed-row callbacks for [`EntityTable`](super::EntityTable).
pub struct EntityColumn<T> {
    /// Stable identifier used by sort and persisted preferences.
    pub id: &'static str,
    /// Visible column heading.
    pub header: String,
    /// Whether the header cycles table ordering.
    pub sortable: bool,
    /// Whether users are forbidden from hiding this column.
    pub required: bool,
    /// Whether this cell contains actions and therefore suppresses row activation.
    pub is_action: bool,
    /// Whether users may resize this column.
    pub resizable: bool,
    /// Optional column-specific minimum width in pixels.
    pub min_width: Option<u32>,
    /// Optional initial width in pixels.
    pub initial_width: Option<u32>,
    /// Overflow policy used by the framework's plain-text renderer.
    pub text_overflow: EntityTextOverflow,
    /// Framework-owned horizontal alignment.
    pub alignment: EntityColumnAlignment,
    /// Semantic presentation kind (default [`EntityColumnKind::Text`]). Set
    /// via [`EntityColumn::numeric`], [`EntityColumn::identifier`], or the
    /// lower-level [`EntityColumn::tabular_numbers`]; contributes an
    /// additive class resolved by [`EntityColumnKind::default_class`].
    pub kind: EntityColumnKind,
    /// Plain text used for default rendering and accessible/exported content.
    pub text: Rc<dyn Fn(&T) -> String>,
    /// Optional rich renderer invoked with a borrowed typed row.
    pub renderer: Option<EntityCellRenderer<T>>,
    /// Optional framework-owned semantic badge or icon presentation.
    pub presentation: Option<EntityCellPresentation<T>>,
    /// Typed comparator invoked with borrowed rows.
    pub comparator: Option<EntityComparator<T>>,
    /// Normalized text key extracted once per row by the default sorter.
    pub sort_key: Option<EntitySortKey<T>>,
    /// Optional inline editor (`ldui-ff2f`). `None` — the default — means the
    /// column stays read-only even in a live row, which is the right answer
    /// for a derived or action column.
    pub editor: Option<EntityCellEditor<T>>,
}

/// What the table hands the consumer when Save is pressed (`ldui-ff2f`).
///
/// The table never writes. It surrenders the edited row and a `resolve`
/// handle, then waits: the session stays in flight until the consumer answers,
/// which is what keeps a failed write from silently discarding the user's
/// typing.
pub struct EntityDraftCommit<T> {
    /// The edited row. The consumer validates and persists this.
    pub row: T,
    /// Which row was submitted — a new draft, or an existing row by key.
    pub target: EntityEditTarget,
    /// Answers the commit. Until this runs the table stays in flight, so Save
    /// cannot fire twice and the row cannot change underneath the write.
    pub resolve: Callback<EntityEditOutcome>,
}

/// Copy for the inline-edit controls (`ldui-ff2f`).
///
/// Deliberately its OWN type rather than three more fields on
/// [`EntityTableTexts`]. That struct has public fields and consumers build it
/// with struct literals — 21 4iiz-Office surfaces do, none of them with
/// `..Default::default()` — so adding a field to it is a breaking change for
/// every one. Copy for an opt-in feature belongs with the opt-in config,
/// where only a consumer who asked for the feature ever has to supply it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityDraftTexts {
    /// Label and accessible name for the `+` action.
    pub add_row: String,
    /// Label the row action shows while that row is live.
    pub save_row: String,
    /// Label the row action shows when the row can be edited.
    pub edit_row: String,
    /// Accessible name for the cancel affordance (Escape also cancels).
    pub cancel_edit: String,
}

impl Default for EntityDraftTexts {
    fn default() -> Self {
        Self {
            add_row: "Add row".to_owned(),
            save_row: "Save".to_owned(),
            edit_row: "Edit".to_owned(),
            cancel_edit: "Cancel".to_owned(),
        }
    }
}

/// Opt-in inline draft-row and per-row editing (`ldui-ff2f`).
///
/// Absent — the default — the table has no `+`, no edit mode and no extra
/// DOM: byte-identical to a table that never heard of this feature.
pub struct EntityDraftRow<T: 'static> {
    /// Builds the blank row `+` inserts. The consumer owns the type, so the
    /// framework never has to invent a `T`.
    pub new_row: Rc<dyn Fn() -> T>,
    /// Fired on Save.
    pub on_commit: Callback<EntityDraftCommit<T>, ()>,
    /// Reactive copy for the edit controls.
    pub texts: Signal<EntityDraftTexts>,
}

impl<T: 'static> EntityDraftRow<T> {
    /// Enables inline editing for this table.
    pub fn new(
        new_row: impl Fn() -> T + 'static,
        on_commit: Callback<EntityDraftCommit<T>, ()>,
    ) -> Self {
        Self {
            new_row: Rc::new(new_row),
            on_commit,
            texts: Signal::stored(EntityDraftTexts::default()),
        }
    }

    /// Supplies localized copy for the edit controls.
    #[must_use]
    pub fn with_texts(mut self, texts: impl Into<Signal<EntityDraftTexts>>) -> Self {
        self.texts = texts.into();
        self
    }
}

impl<T: 'static> Clone for EntityDraftRow<T> {
    fn clone(&self) -> Self {
        Self {
            new_row: Rc::clone(&self.new_row),
            on_commit: self.on_commit,
            texts: self.texts,
        }
    }
}

impl<T> Clone for EntityColumn<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            header: self.header.clone(),
            sortable: self.sortable,
            required: self.required,
            is_action: self.is_action,
            resizable: self.resizable,
            min_width: self.min_width,
            initial_width: self.initial_width,
            text_overflow: self.text_overflow,
            alignment: self.alignment,
            kind: self.kind,
            text: Rc::clone(&self.text),
            renderer: self.renderer.as_ref().map(Rc::clone),
            presentation: self.presentation.clone(),
            comparator: self.comparator.as_ref().map(Rc::clone),
            sort_key: self.sort_key.as_ref().map(Rc::clone),
            editor: self.editor.clone(),
        }
    }
}

impl<T> fmt::Debug for EntityColumn<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EntityColumn")
            .field("id", &self.id)
            .field("header", &self.header)
            .field("sortable", &self.sortable)
            .field("required", &self.required)
            .field("is_action", &self.is_action)
            .field("resizable", &self.resizable)
            .field("min_width", &self.min_width)
            .field("initial_width", &self.initial_width)
            .field("text_overflow", &self.text_overflow)
            .field("alignment", &self.alignment)
            .field("kind", &self.kind)
            .field("presentation", &self.presentation)
            .finish_non_exhaustive()
    }
}

impl<T: 'static> EntityColumn<T> {
    /// Creates a sortable text column.
    pub fn new(
        id: &'static str,
        header: impl Into<String>,
        text: impl Fn(&T) -> String + 'static,
    ) -> Self {
        let text: Rc<dyn Fn(&T) -> String> = Rc::new(text);
        let comparator_text = Rc::clone(&text);
        Self {
            id,
            header: header.into(),
            sortable: true,
            required: false,
            is_action: false,
            resizable: true,
            min_width: None,
            initial_width: None,
            text_overflow: EntityTextOverflow::Wrap,
            alignment: EntityColumnAlignment::Auto,
            kind: EntityColumnKind::Text,
            text,
            renderer: None,
            presentation: None,
            comparator: None,
            sort_key: Some(typed_entity_sort_key(move |row| {
                comparator_text(row).to_lowercase()
            })),
            editor: None,
        }
    }

    /// Makes this column editable while its row is live (`ldui-ff2f`).
    ///
    /// Opt-in per column, because most columns are derived and must stay
    /// read-only. A column that never calls this renders its normal
    /// read-only cell even inside the row being edited.
    ///
    /// ```rust,ignore
    /// EntityColumn::text("name", "Name", |r: &WorkType| r.name.clone())
    ///     .editable(EntityCellEditor::text(
    ///         |r: &WorkType| r.name.clone(),
    ///         |r: &mut WorkType, v| r.name = v,
    ///     ))
    /// ```
    #[must_use]
    pub fn editable(mut self, editor: EntityCellEditor<T>) -> Self {
        self.editor = Some(editor);
        self
    }

    /// Whether this column accepts input while its row is live.
    pub const fn is_editable(&self) -> bool {
        self.editor.is_some()
    }

    /// Creates a sortable text column; an explicit alias for [`Self::new`].
    pub fn text(
        id: &'static str,
        header: impl Into<String>,
        text: impl Fn(&T) -> String + 'static,
    ) -> Self {
        Self::new(id, header, text)
    }

    /// Creates a non-sortable action column.
    pub fn action(
        id: &'static str,
        header: impl Into<String>,
        text: impl Fn(&T) -> String + 'static,
    ) -> Self {
        let mut column = Self::new(id, header, text);
        column.sortable = false;
        column.is_action = true;
        column.comparator = None;
        column.sort_key = None;
        column
    }

    /// Makes this column mandatory in the visible-column set.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Uses a typed comparator for this column.
    pub fn sortable_by(mut self, compare: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        self.sortable = true;
        self.comparator = Some(Rc::new(compare));
        self.sort_key = None;
        self
    }

    /// Uses a rich cell renderer while retaining the text callback for accessibility.
    pub fn render_with(mut self, render: impl Fn(&T) -> AnyView + 'static) -> Self {
        self.renderer = Some(Rc::new(render));
        self
    }

    /// Maps rows to a framework-owned badge; `None` falls back to plain text.
    /// A rich [`Self::render_with`] renderer always takes visual precedence.
    pub fn badge_with(
        mut self,
        presentation: impl Fn(&T) -> Option<EntityBadgePresentation> + 'static,
    ) -> Self {
        self.presentation = Some(EntityCellPresentation::Badge(Rc::new(presentation)));
        self
    }

    /// Maps rows to a framework-owned icon; `None` falls back to plain text.
    /// A rich [`Self::render_with`] renderer always takes visual precedence.
    pub fn icon_with(
        mut self,
        presentation: impl Fn(&T) -> Option<EntityIconPresentation> + 'static,
    ) -> Self {
        self.presentation = Some(EntityCellPresentation::Icon(Rc::new(presentation)));
        self
    }

    /// Renders canonical text as an opinionated primary line plus an
    /// optional muted secondary line beneath it, instead of one plain line.
    ///
    /// `primary` and `secondary` control only the visual split. The
    /// column's canonical `text` callback (see [`Self::new`]) remains the
    /// sole accessible name and exported value -- it must stay complete on
    /// its own, because the framework announces it once (never the visible
    /// primary/secondary text a second time). `secondary` returning `None`,
    /// or an empty or whitespace-only string, renders no secondary line and
    /// therefore no extra spacing or punctuation. Sorting is untouched by
    /// this call: the column keeps whatever `sort_key`/comparator it already
    /// had, defaulting to the canonical text unless overridden with
    /// [`Self::sortable_by_key`] or [`Self::sortable_by`]. A rich
    /// [`Self::render_with`] renderer always takes visual precedence.
    pub fn primary_secondary(
        mut self,
        primary: impl Fn(&T) -> String + 'static,
        secondary: impl Fn(&T) -> Option<String> + 'static,
    ) -> Self {
        self.presentation = Some(EntityCellPresentation::PrimarySecondary {
            primary: Rc::new(primary),
            secondary: Rc::new(secondary),
        });
        self
    }

    /// Uses an ordered typed key extracted once per row.
    ///
    /// Integers, signed values, strings, date/time types, tuples, and domain
    /// newtypes implementing [`Ord`] all use this path. Equal keys retain
    /// stable source order. Use [`Self::sortable_by_optional_key`] rather than
    /// relying on `Option`'s implicit ordering when absence is meaningful.
    pub fn sortable_by_key<K: Ord + 'static>(
        mut self,
        extract: impl Fn(&T) -> K + 'static,
    ) -> Self {
        self.sortable = true;
        self.comparator = None;
        self.sort_key = Some(typed_entity_sort_key(extract));
        self
    }

    /// Uses an optional ordered key with explicit, direction-independent null placement.
    pub fn sortable_by_optional_key<K: Ord + 'static>(
        mut self,
        null_order: EntityNullOrder,
        extract: impl Fn(&T) -> Option<K> + 'static,
    ) -> Self {
        self.sortable = true;
        self.comparator = None;
        self.sort_key = Some(Rc::new(OptionalEntitySortKey {
            null_order,
            extract: Rc::new(extract),
        }));
        self
    }

    /// Clips the framework-rendered canonical text to one line with ellipsis.
    /// A rich [`Self::render_with`] renderer takes visual precedence.
    pub fn ellipsis(mut self) -> Self {
        self.text_overflow = EntityTextOverflow::Ellipsis;
        self
    }

    /// Clips the framework-rendered canonical text to `lines` visual lines.
    ///
    /// # Panics
    /// Panics when `lines` is zero; a zero-line cell has no useful or
    /// accessible visual presentation.
    pub fn line_clamp(mut self, lines: u8) -> Self {
        self.text_overflow = EntityTextOverflow::LineClamp(
            NonZeroU8::new(lines).expect("EntityColumn line clamp must be positive"),
        );
        self
    }

    /// Aligns the wide header/value and compact value to the inline start.
    pub fn align_start(mut self) -> Self {
        self.alignment = EntityColumnAlignment::Start;
        self
    }

    /// Centers the wide header/value and compact value.
    pub fn align_center(mut self) -> Self {
        self.alignment = EntityColumnAlignment::Center;
        self
    }

    /// Aligns the wide header/value and compact value to the inline end.
    pub fn align_end(mut self) -> Self {
        self.alignment = EntityColumnAlignment::End;
        self
    }

    /// Uses tabular-width numeral glyphs without formatting the canonical
    /// text, and without changing [`alignment`](EntityColumn::alignment).
    ///
    /// This is the lower-level primitive [`EntityColumn::numeric`] is built
    /// from (`kind` only); it remains a distinct method because `EntityColumn`
    /// has no raw class escape hatch, so a column that wants tabular figures
    /// under a non-right alignment (e.g. a centered date column) has no other
    /// way to say so. Prefer [`EntityColumn::numeric`] for the common
    /// right-aligned numeric case.
    pub fn tabular_numbers(mut self) -> Self {
        self.kind = EntityColumnKind::Numeric;
        self
    }

    /// Marks this column as numeric: tabular (monospaced) figures plus
    /// right alignment, so digits line up column-to-column -- money, counts,
    /// percentages, durations. Equivalent to
    /// `.tabular_numbers().align_end()`.
    ///
    /// Unlike `DataTable::Column::numeric`, this does **not** imply a
    /// numeric sort key. `DataTable`'s rows are untyped
    /// (`HashMap<String, String>`), so its `SortAs::Number` re-parses the
    /// displayed text at sort time -- the only numeric comparison it has
    /// available. `EntityColumn` is typed over `T` and already has an exact,
    /// zero-parsing way to say "sort this numerically":
    /// [`EntityColumn::sortable_by_key`]. Deriving a sort key from this
    /// method by re-parsing the same formatted display text `.numeric()`
    /// styles would be strictly less correct than a caller's own typed
    /// extractor, and doing so only when no other sort key had been set yet
    /// would make the result depend on builder call order -- exactly the
    /// kind of silent disagreement `ldui-lrig` measured and this bead exists
    /// to remove. Presentation and sorting stay two independent, explicit
    /// choices: reach for [`EntityColumn::sortable_by_key`] alongside this.
    ///
    /// Calling `.align_start()`/`.align_center()`/`.align_end()` *after*
    /// `.numeric()` overrides the alignment it sets, the same as any other
    /// builder call order.
    ///
    /// ```
    /// use leptos_daisyui_rs::components::{EntityColumn, EntityColumnAlignment, EntityColumnKind};
    ///
    /// struct Row { balance: i64 }
    ///
    /// let balance = EntityColumn::new("balance", "Balance", |row: &Row| row.balance.to_string())
    ///     .sortable_by_key(|row: &Row| row.balance)
    ///     .numeric();
    /// assert_eq!(balance.kind, EntityColumnKind::Numeric);
    /// assert_eq!(balance.alignment, EntityColumnAlignment::End);
    /// ```
    pub fn numeric(mut self) -> Self {
        self.kind = EntityColumnKind::Numeric;
        self.alignment = EntityColumnAlignment::End;
        self
    }

    /// Marks this column as an identifier (id, code, hash, SKU): the
    /// theme's declared monospace face (`font-mono`, themed via the
    /// `--font-mono` CSS variable) so characters line up and visually
    /// similar glyphs (`0`/`O`, `1`/`l`) stay distinguishable.
    ///
    /// Does not change [`alignment`](EntityColumn::alignment) or sorting --
    /// identifiers still align and compare correctly as plain text (the
    /// defaults).
    ///
    /// Note: `font-mono` currently trips the style audit's typography-family
    /// check regardless of whether the component or the caller applied it --
    /// `StyleProfile` records one dominant family per page and flags every
    /// deviation, with no way yet to mark a mono face as intentional (tracked
    /// separately as `ldui-kq9w`, the same gap `DataTable::Column::identifier`
    /// documents). That is a known gap in the audit, not a reason to avoid
    /// `.identifier()`.
    ///
    /// ```
    /// use leptos_daisyui_rs::components::{EntityColumn, EntityColumnKind};
    ///
    /// struct Row { job_id: String }
    ///
    /// let job = EntityColumn::new("job", "Job", |row: &Row| row.job_id.clone()).identifier();
    /// assert_eq!(job.kind, EntityColumnKind::Identifier);
    /// ```
    pub fn identifier(mut self) -> Self {
        self.kind = EntityColumnKind::Identifier;
        self
    }

    /// Sets this column's minimum width in pixels.
    pub fn with_min_width(mut self, width: u32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Sets this column's initial width in pixels.
    pub fn with_width(mut self, width: u32) -> Self {
        self.initial_width = Some(width);
        self
    }

    /// Prevents interactive width changes.
    pub fn non_resizable(mut self) -> Self {
        self.resizable = false;
        self
    }
}

/// Framework-owned presentation for the column-chooser trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityColumnChooserTrigger {
    /// Show the localized `EntityTableTexts::choose_columns` label.
    #[default]
    Text,
    /// Show a compact gear glyph while retaining the localized accessible name.
    Icon,
}

/// What the user asked rows-per-page to be, before any layout is measured.
///
/// This is the persistable half of the pagination decision: it is a stable
/// user preference, never a measured row count. The measured count is
/// transient presentation state and is combined with this intent exactly once
/// per render by
/// [`resolve_entity_page_size`](super::resolve_entity_page_size), which
/// produces the single [`EntityPageSize`] the body, summary, control, and
/// pager all read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityPageSizeIntent {
    /// Fit the page to the measured viewport. Only offered by a table that
    /// opted into [`EntityTableViewportFit`]; anywhere else it resolves to the
    /// explicit [`EntityTablePreferences::page_size`].
    #[default]
    Auto,
    /// Always page at [`EntityTablePreferences::page_size`], scrolling the
    /// table region when the viewport cannot show that many rows.
    Fixed,
}

/// The one resolved rows-per-page decision for a single render.
///
/// `EntityTable` derives exactly one of these per render and every consumer of
/// a page size reads it: the rendered body, the `Showing x-y of z` summary,
/// the rows-per-page control's selected value and label, and the pager's page
/// count. Because the mode and the row count are one indivisible value, the
/// control cannot advertise a size the body is not rendering (ldui-5p06).
///
/// The fields are private and the only constructors clamp the row count, so
/// three wrong states are unrepresentable rather than merely avoided:
///
/// - a row count with no mode (an "effective size" nobody can label, which is
///   how a control showing `25` came to sit over a five-row page);
/// - a mode with no row count (an `Auto` the summary and pager cannot use);
/// - a zero row count (an empty page, and a division by zero in page counting).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityPageSize {
    intent: EntityPageSizeIntent,
    rows: usize,
}

impl EntityPageSize {
    /// A viewport-fitted decision currently rendering `rows` rows.
    #[must_use]
    pub const fn auto(rows: usize) -> Self {
        Self {
            intent: EntityPageSizeIntent::Auto,
            rows: if rows == 0 { 1 } else { rows },
        }
    }

    /// An explicit decision rendering up to `rows` rows.
    #[must_use]
    pub const fn fixed(rows: usize) -> Self {
        Self {
            intent: EntityPageSizeIntent::Fixed,
            rows: if rows == 0 { 1 } else { rows },
        }
    }

    /// Rows this page actually renders. Always at least one.
    #[must_use]
    pub const fn rows(self) -> usize {
        self.rows
    }

    /// The intent this decision resolved from.
    #[must_use]
    pub const fn intent(self) -> EntityPageSizeIntent {
        self.intent
    }

    /// Whether the row count came from a viewport measurement.
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self.intent, EntityPageSizeIntent::Auto)
    }

    /// The rows-per-page `<option>` value that represents this decision.
    #[must_use]
    pub fn control_value(self) -> String {
        if self.is_auto() {
            ENTITY_PAGE_SIZE_AUTO_VALUE.to_owned()
        } else {
            self.rows.to_string()
        }
    }

    /// The localized rows-per-page `<option>` label for this decision.
    ///
    /// Auto substitutes the measured row count into
    /// [`EntityTableTexts::rows_per_page_auto`], so the control reads
    /// `Auto (5)` rather than a number the body never renders.
    #[must_use]
    pub fn control_label(self, texts: &EntityTableTexts) -> String {
        if self.is_auto() {
            texts
                .rows_per_page_auto
                .replace("{rows}", &self.rows.to_string())
        } else {
            self.rows.to_string()
        }
    }
}

/// The rows-per-page `<option>` value that selects viewport-fitted paging.
pub const ENTITY_PAGE_SIZE_AUTO_VALUE: &str = "auto";

/// Versioned user preferences persisted independently of a dataset snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityTablePreferences {
    /// Consumer-defined schema version used to invalidate incompatible payloads.
    pub schema_version: u16,
    /// Number of rows rendered per page in [`EntityPageSizeIntent::Fixed`],
    /// and the fallback before a viewport-fit measurement exists.
    pub page_size: usize,
    /// Whether rows-per-page follows the viewport or the explicit `page_size`.
    ///
    /// This is the only persisted half of the pagination decision: a measured
    /// row count is transient presentation state and is never stored here
    /// (ldui-5p06).
    #[serde(default)]
    pub page_size_mode: EntityPageSizeIntent,
    /// Current local ordering.
    pub sort: EntitySort,
    /// Explicit display order of stable column identifiers.
    #[serde(default)]
    pub column_order: Vec<String>,
    /// Optional columns hidden by the user.
    pub hidden_columns: BTreeSet<String>,
    /// User-adjusted widths keyed by stable column identifier.
    pub column_widths: BTreeMap<String, u32>,
}

impl EntityTablePreferences {
    /// Creates the opinionated defaults for a preference schema.
    pub fn new(schema_version: u16) -> Self {
        Self {
            schema_version,
            page_size: 25,
            page_size_mode: EntityPageSizeIntent::Auto,
            sort: EntitySort::System,
            column_order: Vec::new(),
            hidden_columns: BTreeSet::new(),
            column_widths: BTreeMap::new(),
        }
    }
}

/// Opt-in viewport-fit paging policy for [`EntityTable`](super::EntityTable).
///
/// The derived row capacity is presentation state. It never replaces or
/// persists [`EntityTablePreferences::page_size`], which remains the fixed-mode
/// value and the safe fallback for very short viewports.
///
/// Supplying this policy adds an explicit `Auto` choice to the rows-per-page
/// control and makes it the default. It no longer overrides an explicit
/// numeric choice: selecting `25` records
/// [`EntityPageSizeIntent::Fixed`] and renders up to 25 rows, scrolling the
/// table region (ldui-5p06).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityTableViewportFit {
    height: Option<String>,
    min_rows: usize,
}

impl EntityTableViewportFit {
    /// Fills a parent that already provides a definite height.
    pub const fn fill_parent() -> Self {
        Self {
            height: None,
            min_rows: 5,
        }
    }

    /// Uses a caller-supplied CSS height/maximum-height expression as the
    /// table's definite layout budget.
    pub fn max_height(height: impl Into<String>) -> Self {
        Self {
            height: Some(height.into()),
            min_rows: 5,
        }
    }

    /// Sets the minimum usable row count before fixed-size fallback scrolling.
    pub const fn with_min_rows(mut self, min_rows: usize) -> Self {
        self.min_rows = if min_rows == 0 { 1 } else { min_rows };
        self
    }

    /// Explicit CSS height budget, or `None` when filling a definite parent.
    pub fn height(&self) -> Option<&str> {
        self.height.as_deref()
    }

    /// Minimum usable responsive row count.
    pub const fn min_rows(&self) -> usize {
        self.min_rows
    }
}

/// Component-owned persistence used only by uncontrolled tables.
///
/// Controlled tables never carry this policy: their consumer owns both the
/// current value and any persistence performed after a change callback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityTablePreferencePersistence {
    /// Keep preferences in memory for this component instance only.
    #[default]
    Disabled,
    /// Preserve the historical automatic `localStorage` behavior.
    LegacyLocalStorage {
        /// Namespace appended to the framework's EntityTable storage prefix.
        storage_key: &'static str,
    },
}

/// Declares who owns an [`EntityTablePreferences`] value.
///
/// Controlled ownership is persistence-neutral: each UI operation emits one
/// normalized full replacement and the consumer decides whether or where to
/// store it. Uncontrolled ownership retains the component's compatibility
/// behavior and may opt into the legacy browser-storage mechanism.
#[derive(Clone)]
pub enum EntityTablePreferenceOwnership {
    /// The consumer supplies the current value and receives replacements.
    Controlled {
        /// Reactive current preferences supplied by the consumer.
        current: Signal<EntityTablePreferences>,
        /// Receives one normalized full replacement per UI preference action.
        on_change: Callback<EntityTablePreferences>,
    },
    /// The component owns its in-memory preference signal.
    Uncontrolled {
        /// Optional component-managed persistence.
        persistence: EntityTablePreferencePersistence,
    },
}

impl EntityTablePreferenceOwnership {
    /// Creates consumer-controlled, persistence-neutral ownership.
    pub fn controlled(
        current: Signal<EntityTablePreferences>,
        on_change: Callback<EntityTablePreferences>,
    ) -> Self {
        Self::Controlled { current, on_change }
    }

    /// Creates component-owned preferences with the selected persistence.
    pub fn uncontrolled(persistence: EntityTablePreferencePersistence) -> Self {
        Self::Uncontrolled { persistence }
    }
}

impl fmt::Debug for EntityTablePreferenceOwnership {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Controlled { .. } => formatter.debug_struct("Controlled").finish_non_exhaustive(),
            Self::Uncontrolled { persistence } => formatter
                .debug_struct("Uncontrolled")
                .field("persistence", persistence)
                .finish(),
        }
    }
}

/// Localizable labels used by [`EntityTable`](super::EntityTable).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityTableTexts {
    /// Accessible name for the programmatically focusable table region.
    pub region_label: String,
    /// Label for the page-size control.
    pub rows_per_page: String,
    /// Rows-per-page option label for viewport-fitted paging, with a `{rows}`
    /// placeholder carrying the row count currently rendered.
    pub rows_per_page_auto: String,
    /// Accessible label for the column chooser.
    pub choose_columns: String,
    /// Label for the responsive controlled-filter panel.
    pub filters: String,
    /// Active-filter status shown beside a chooser item that cannot be hidden.
    pub filter_active: String,
    /// Clear-filter template with a `{column}` placeholder.
    pub clear_filter: String,
    /// Visible and accessible label for the column-order list.
    pub column_order: String,
    /// Move-earlier template with `{column}`, `{position}`, and `{total}`.
    pub move_earlier: String,
    /// Move-later template with `{column}`, `{position}`, and `{total}`.
    pub move_later: String,
    /// Resize-handle name template with `{column}`.
    pub resize_column: String,
    /// Resize value text template with `{pixels}`.
    pub pixel_value: String,
    /// Current-sort copy for an inactive column.
    pub sort_not_sorted: String,
    /// Current-sort template with `{direction}`, `{priority}`, and `{total}`.
    pub sort_current: String,
    /// Plain activation for an inactive column.
    pub sort_plain_ascending: String,
    /// Plain activation for an ascending column.
    pub sort_plain_descending: String,
    /// Plain activation for a descending column.
    pub sort_plain_system: String,
    /// Additive activation template for a new clause with `{priority}`.
    pub sort_add: String,
    /// Additive direction-change template with `{priority}` and `{direction}`.
    pub sort_change: String,
    /// Additive removal template with `{priority}`.
    pub sort_remove: String,
    /// Localized ascending direction word.
    pub ascending: String,
    /// Localized descending direction word.
    pub descending: String,
    /// Live-region text for server/system order.
    pub system_order: String,
    /// Live sort-summary template with `{clauses}`.
    pub sort_summary: String,
    /// One summary clause with `{priority}`, `{column}`, and `{direction}`.
    pub sort_clause: String,
    /// Action label that restores server-supplied ordering.
    pub reset_sort: String,
    /// Action label that restores default column visibility, widths, and order.
    pub reset_columns: String,
    /// Previous-page action label.
    pub previous: String,
    /// Next-page action label.
    pub next: String,
    /// Row-range template with `{start}`, `{end}`, and `{total}` placeholders.
    pub row_range: String,
    /// Message shown when the **authoritative source dataset** holds no rows
    /// at all — the provider genuinely has nothing to show (`ldui-g4nw`).
    ///
    /// This is where a domain sentence belongs ("No contribution credits are
    /// present in this snapshot."), because it is the only case in which such
    /// a sentence is true.
    pub no_rows: String,
    /// Message shown when source rows exist but the current projection is
    /// empty — every row was filtered, searched, date-bounded or collapsed
    /// away (`ldui-g4nw`).
    ///
    /// Kept separate from [`Self::no_rows`] because reusing one string makes
    /// the table assert the provider is empty when it is not, which reads as
    /// missing data rather than an over-narrow filter. A caller that overrides
    /// only `no_rows` keeps that copy for the provider-empty case and inherits
    /// this default for the filtered case, so the distinction costs an
    /// existing consumer nothing.
    pub no_matching_rows: String,
}

/// Which empty state a table is in (`ldui-g4nw`).
///
/// The table already knows both counts, so the choice is a total function of
/// the authoritative source row count — never a guess, and never the same
/// sentence for both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityEmptyState {
    /// The authoritative source dataset itself is empty.
    #[default]
    Provider,
    /// Source rows exist; the current filtered/searched/collapsed projection
    /// selected none of them.
    Filtered,
}

impl EntityEmptyState {
    /// Classifies from the authoritative source row count.
    ///
    /// `source_row_count` is the count of the **source** snapshot — the
    /// `source_data` prop when supplied, otherwise the rendered `data`
    /// snapshot, which is the same fallback focus recovery already uses.
    #[must_use]
    pub const fn from_source_rows(source_row_count: usize) -> Self {
        if source_row_count == 0 {
            Self::Provider
        } else {
            Self::Filtered
        }
    }

    /// Stable `data-entity-empty-state` value.
    ///
    /// A stable attribute rather than a copy comparison: the copy is
    /// localizable, so asserting on it would make a browser proof fail the
    /// moment a consumer translates the table.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Provider => "provider",
            Self::Filtered => "filtered",
        }
    }
}

impl Default for EntityTableTexts {
    fn default() -> Self {
        Self {
            region_label: "Data table".to_owned(),
            rows_per_page: "Rows per page".to_owned(),
            rows_per_page_auto: "Auto ({rows})".to_owned(),
            choose_columns: "Choose columns".to_owned(),
            filters: "Filters".to_owned(),
            filter_active: "Filter active".to_owned(),
            clear_filter: "Clear {column} filter".to_owned(),
            column_order: "Column order".to_owned(),
            move_earlier: "Move {column} earlier from position {position} of {total}".to_owned(),
            move_later: "Move {column} later from position {position} of {total}".to_owned(),
            resize_column: "Resize {column} column".to_owned(),
            pixel_value: "{pixels} pixels".to_owned(),
            sort_not_sorted: "Not currently sorted".to_owned(),
            sort_current: "Currently sorted {direction} at priority {priority} of {total}"
                .to_owned(),
            sort_plain_ascending: "Activate to sort ascending as the only sort".to_owned(),
            sort_plain_descending: "Activate to sort descending as the only sort".to_owned(),
            sort_plain_system: "Activate to restore system order".to_owned(),
            sort_add: "Shift+activate to add ascending at priority {priority}".to_owned(),
            sort_change: "Shift+activate to change priority {priority} to {direction}".to_owned(),
            sort_remove: "Shift+activate to remove priority {priority}".to_owned(),
            ascending: "ascending".to_owned(),
            descending: "descending".to_owned(),
            system_order: "System order".to_owned(),
            sort_summary: "Sorted by {clauses}".to_owned(),
            sort_clause: "priority {priority}: {column} {direction}".to_owned(),
            reset_sort: "Reset sort".to_owned(),
            reset_columns: "Reset columns".to_owned(),
            previous: "Previous".to_owned(),
            next: "Next".to_owned(),
            row_range: "Showing {start}-{end} of {total}".to_owned(),
            no_rows: "No rows".to_owned(),
            no_matching_rows: "No rows match the current filters".to_owned(),
        }
    }
}

impl EntityTableTexts {
    /// The one empty-state sentence for the state the table is actually in.
    #[must_use]
    pub fn empty_state_message(&self, state: EntityEmptyState) -> &str {
        match state {
            EntityEmptyState::Provider => &self.no_rows,
            EntityEmptyState::Filtered => &self.no_matching_rows,
        }
    }
}
