//! Controlled, accessible row grouping for [`EntityTable`](super::EntityTable)
//! (`ldui-iyfa`).
//!
//! # The identity is the key, never the label
//!
//! A group is identified by [`EntityRowGroup::key`] and nothing else. The
//! label is display copy: it is localizable, it may repeat across two
//! genuinely different groups, and changing it can never repartition the
//! table, reorder a section, or move a collapse flag onto another group.
//! Every mechanism in this module -- partitioning, rank, collapse state, the
//! exported group identity -- reads the key; only the rendered heading and the
//! exported group *column* read the label.
//!
//! # Grouping never invents an order of its own
//!
//! Rows arrive here already sorted by the table's own
//! [`EntitySort`](super::EntitySort). Grouping applies a **stable** partition
//! by group rank on top of that permutation, so row sorting happens *within*
//! groups and the caller's declared group order is what separates the
//! sections. Selecting an explicit [`EntityGroupOrder`] other than
//! [`Declared`](EntityGroupOrder::Declared) replaces only the rank, never the
//! within-group row order.
//!
//! Groups the caller never declared are not dropped: they rank after every
//! declared group, in first-appearance order, so a dataset that grows a new
//! group key still shows every record.
//!
//! # Filtering, empty groups, and the orphan heading
//!
//! Filters apply to child rows, exactly as they always did -- nothing in this
//! module filters. A group whose rows are all filtered away simply has no rows
//! left, and [`entity_grouped_order`] emits no run for it, so the heading
//! disappears with its children.
//!
//! A heading is never inserted independently of the rows it heads: every
//! expanded section's heading is derived from a row that is actually on the
//! page. An orphan heading -- an expanded group's heading stranded as the last
//! visible row with its children on the next page -- is therefore
//! *unrepresentable* rather than merely avoided. A collapsed group's heading
//! is its complete rendering, not an orphan.
//!
//! # Collapse is a filter, not a visibility toggle
//!
//! Collapsing removes a group's rows from the displayed model outright: they
//! leave paging, the row-range summary, the displayed-page selection
//! population, and the display projection together. That is what keeps every
//! count truthful, and it is why collapsed children leave the accessibility
//! tree instead of being painted and hidden.
//!
//! Collapse is optional and controlled. With no collapse binding, every
//! filtered row is exposed.

use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::Range;
use std::rc::Rc;

/// A callback returning the stable group key of a borrowed row.
pub type EntityGroupKey<T> = Rc<dyn Fn(&T) -> String>;

/// A callback rendering one group heading's optional compact actions.
pub type EntityGroupActions = Rc<dyn Fn(&EntityRowGroup) -> AnyView>;

/// Fixed `<col>`-free synthetic column id carrying group identity in an
/// [`EntityTableDisplayProjection`](super::EntityTableDisplayProjection).
///
/// Namespaced out of caller reach so it can never collide with an
/// [`EntityColumn`](super::EntityColumn) id, appear in the column chooser, or
/// take part in sorting, filtering, or resizing.
pub const ENTITY_GROUP_COLUMN_ID: &str = "__ldui-entity-group";

/// One caller-declared row group.
///
/// The key is the identity; the label is display copy. Two groups may carry
/// the same label and remain entirely distinct, and relabelling a group
/// changes nothing but the rendered heading and the exported group column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityRowGroup {
    key: String,
    label: String,
    meta: Option<String>,
}

impl EntityRowGroup {
    /// Declares one group by stable key and display label.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            meta: None,
        }
    }

    /// Adds compact metadata rendered beside the heading.
    ///
    /// When omitted, the table renders its own localized row count from
    /// [`EntityGroupTexts::row_count`].
    pub fn with_meta(mut self, meta: impl Into<String>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// The stable identity. Never the label.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The display label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Caller-supplied compact metadata, if any.
    pub fn meta(&self) -> Option<&str> {
        self.meta.as_deref()
    }
}

/// How the group sections are ordered relative to one another.
///
/// Whichever variant is selected, rows keep the table's own sort order
/// *within* each group -- the rank only decides where the sections sit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityGroupOrder {
    /// The caller's declared order. Undeclared keys follow, in
    /// first-appearance order.
    #[default]
    Declared,
    /// An explicit group sort by localized label, ascending. Ties fall back to
    /// declared order, so the result is total.
    LabelAscending,
    /// An explicit group sort by localized label, descending.
    LabelDescending,
}

/// Why a collapse proposal was emitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityGroupCollapseCause {
    /// One group heading's disclosure control was activated.
    Group {
        /// Stable group identity.
        key: String,
        /// Whether the gesture proposes collapsing rather than expanding.
        collapsed: bool,
    },
}

/// One atomic collapse proposal carrying the COMPLETE resulting key set.
///
/// Mirrors
/// [`EntityTableSelectionProposal`](super::EntityTableSelectionProposal): it
/// is never a per-group delta the caller has to reassemble, and it is stamped
/// with the scope it was computed against so a caller that swaps datasets can
/// refuse a stale proposal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityGroupCollapseProposal {
    /// The complete set of collapsed group keys after the gesture.
    pub keys: BTreeSet<String>,
    /// What produced the proposal.
    pub cause: EntityGroupCollapseCause,
    /// Dataset/selection scope the proposal was computed against.
    pub scope: String,
}

/// Returns the complete collapsed-key set after toggling one group.
pub fn propose_entity_group_collapse(
    current: &BTreeSet<String>,
    key: &str,
    collapsed: bool,
) -> BTreeSet<String> {
    let mut next = current.clone();
    if collapsed {
        next.insert(key.to_owned());
    } else {
        next.remove(key);
    }
    next
}

/// Controlled row grouping supplied to [`EntityTable`](super::EntityTable).
///
/// Everything here is caller-owned. The component never invents a group, never
/// stores collapse state, and never derives a group from a column's rendered
/// text -- the group key comes from `group_of`, the declared order and labels
/// come from `groups`, and collapse (when bound at all) is proposed and
/// accepted exactly like multi-selection.
pub struct EntityRowGrouping<T: 'static> {
    pub(crate) group_of: EntityGroupKey<T>,
    pub(crate) groups: Signal<Vec<EntityRowGroup>, LocalStorage>,
    pub(crate) order: Signal<EntityGroupOrder>,
    pub(crate) collapsed: Option<Signal<BTreeSet<String>>>,
    pub(crate) on_collapse_change: Option<Callback<EntityGroupCollapseProposal>>,
    pub(crate) actions: Option<EntityGroupActions>,
    pub(crate) texts: Signal<EntityGroupTexts>,
}

impl<T: 'static> Clone for EntityRowGrouping<T> {
    fn clone(&self) -> Self {
        Self {
            group_of: Rc::clone(&self.group_of),
            groups: self.groups,
            order: self.order,
            collapsed: self.collapsed,
            on_collapse_change: self.on_collapse_change,
            actions: self.actions.clone(),
            texts: self.texts,
        }
    }
}

impl<T: 'static> fmt::Debug for EntityRowGrouping<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EntityRowGrouping")
            .field("collapsible", &self.collapsed.is_some())
            .field("actions", &self.actions.is_some())
            .finish()
    }
}

impl<T: 'static> EntityRowGrouping<T> {
    /// Groups rows by a stable key, rendered in the caller's declared order.
    ///
    /// Group keys the declarations never mention are still rendered, after
    /// every declared group, in first-appearance order.
    pub fn controlled(
        group_of: EntityGroupKey<T>,
        groups: impl Into<Signal<Vec<EntityRowGroup>, LocalStorage>>,
    ) -> Self {
        Self {
            group_of,
            groups: groups.into(),
            order: Signal::stored(EntityGroupOrder::Declared),
            collapsed: None,
            on_collapse_change: None,
            actions: None,
            texts: Signal::stored(EntityGroupTexts::default()),
        }
    }

    /// Selects an explicit group sort in place of the declared order.
    pub fn with_order(mut self, order: impl Into<Signal<EntityGroupOrder>>) -> Self {
        self.order = order.into();
        self
    }

    /// Opts into controlled collapse.
    ///
    /// Collapsing removes a group's rows from the displayed model -- from
    /// paging, from the row-range summary, from the displayed-page selection
    /// population, and from the display projection -- so every count stays
    /// truthful and collapsed children leave the accessibility tree entirely.
    /// Without this binding the table exposes every filtered row.
    pub fn collapsible(
        mut self,
        collapsed: impl Into<Signal<BTreeSet<String>>>,
        on_collapse_change: Callback<EntityGroupCollapseProposal>,
    ) -> Self {
        self.collapsed = Some(collapsed.into());
        self.on_collapse_change = Some(on_collapse_change);
        self
    }

    /// Renders caller-owned compact actions inside each group heading.
    pub fn with_actions(mut self, actions: EntityGroupActions) -> Self {
        self.actions = Some(actions);
        self
    }

    /// Overrides the localizable heading copy.
    pub fn with_texts(mut self, texts: impl Into<Signal<EntityGroupTexts>>) -> Self {
        self.texts = texts.into();
        self
    }
}

/// Localizable copy for the group headings a grouped `EntityTable` renders.
///
/// A separate struct rather than five more fields on
/// [`EntityTableTexts`](super::EntityTableTexts), for the same reason
/// [`EntityTableSelectionTexts`](super::EntityTableSelectionTexts) is
/// separate: this copy exists only when the feature is configured, and
/// widening the always-required texts struct would break every consumer that
/// builds one as a literal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityGroupTexts {
    /// Header of the synthetic group column added to a grouped table's display
    /// projection, so an export carries the group identity the visual table
    /// deliberately stops repeating in every row.
    pub column_header: String,
    /// Default compact metadata for a heading, with a `{count}` placeholder.
    /// Used only when the declaration supplies no metadata of its own.
    pub row_count: String,
    /// Continuation heading template with a `{group}` placeholder, used when a
    /// group's rows resume on a later page.
    pub continued: String,
    /// Accessible name of a heading's disclosure control while the group is
    /// expanded, with a `{group}` placeholder. It contains the visible group
    /// label, so it satisfies label-in-name rather than replacing it.
    pub collapse: String,
    /// Accessible name of a heading's disclosure control while the group is
    /// collapsed, with a `{group}` placeholder.
    pub expand: String,
}

impl Default for EntityGroupTexts {
    fn default() -> Self {
        Self {
            column_header: "Group".to_owned(),
            row_count: "{count} rows".to_owned(),
            continued: "{group} (continued)".to_owned(),
            collapse: "Collapse {group}".to_owned(),
            expand: "Expand {group}".to_owned(),
        }
    }
}

impl EntityGroupTexts {
    /// Heading text for a group, marked as a continuation when its earlier
    /// rows are on a previous page.
    pub fn heading(&self, label: &str, continued: bool) -> String {
        if continued {
            return self.continued.replace("{group}", label);
        }
        label.to_owned()
    }

    /// Default compact metadata naming how many records the group holds.
    pub fn row_count_label(&self, count: usize) -> String {
        self.row_count.replace("{count}", &count.to_string())
    }

    /// Accessible name for a heading's disclosure control.
    pub fn toggle_label(&self, label: &str, collapsed: bool) -> String {
        if collapsed {
            return self.expand.replace("{group}", label);
        }
        self.collapse.replace("{group}", label)
    }
}

/// One non-empty group run over the filtered dataset, in rank order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityGroupRun {
    /// Stable group identity.
    pub key: String,
    /// How many filtered rows the group holds, ignoring collapse.
    pub row_count: usize,
    /// Whether the caller currently collapses this group.
    pub collapsed: bool,
}

/// The table's displayed row order after grouping and collapse.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityGroupedOrder {
    /// Displayed source-row indices, grouped and collapse-filtered.
    pub indices: Vec<usize>,
    /// Group key of each displayed index, parallel to `indices`.
    pub group_keys: Vec<String>,
    /// Every non-empty group, in rank order, collapsed or not.
    pub runs: Vec<EntityGroupRun>,
}

/// Ranks the declared groups, then any group key the data introduced.
///
/// Returns a key-to-rank map that is total over both sources: a declared group
/// keeps its declared position (or its label position under an explicit group
/// sort), and an undeclared key ranks after every declared one in
/// first-appearance order. Nothing is ever dropped for being undeclared.
pub(crate) fn entity_group_ranks(
    groups: &[EntityRowGroup],
    order: EntityGroupOrder,
    encountered: &[String],
) -> BTreeMap<String, usize> {
    let mut declared: Vec<(usize, &EntityRowGroup)> = groups.iter().enumerate().collect();
    match order {
        EntityGroupOrder::Declared => {}
        EntityGroupOrder::LabelAscending => {
            declared.sort_by(|left, right| {
                left.1
                    .label
                    .cmp(&right.1.label)
                    .then_with(|| left.0.cmp(&right.0))
            });
        }
        EntityGroupOrder::LabelDescending => {
            declared.sort_by(|left, right| {
                right
                    .1
                    .label
                    .cmp(&left.1.label)
                    .then_with(|| left.0.cmp(&right.0))
            });
        }
    }

    let mut ranks = BTreeMap::new();
    for (rank, (_, group)) in declared.into_iter().enumerate() {
        ranks.entry(group.key.clone()).or_insert(rank);
    }
    let mut next_rank = groups.len();
    for key in encountered {
        if !ranks.contains_key(key) {
            ranks.insert(key.clone(), next_rank);
            next_rank += 1;
        }
    }
    ranks
}

/// Applies grouping and collapse to an already-sorted index permutation.
///
/// The partition is a **stable** sort by group rank, so the incoming row order
/// survives untouched inside every group -- that is the whole of "row sorting
/// occurs within groups". Collapsed groups keep their run (and therefore their
/// heading and their honest row count) but contribute no displayed rows.
pub(crate) fn entity_grouped_order(
    sorted_indices: &[usize],
    group_key_of: &dyn Fn(usize) -> String,
    groups: &[EntityRowGroup],
    order: EntityGroupOrder,
    collapsed: &BTreeSet<String>,
) -> EntityGroupedOrder {
    let keys: Vec<String> = sorted_indices
        .iter()
        .map(|index| group_key_of(*index))
        .collect();
    let ranks = entity_group_ranks(groups, order, &keys);

    let mut ordered: Vec<(usize, String)> = sorted_indices.iter().copied().zip(keys).collect();
    ordered.sort_by_key(|(_, key)| ranks.get(key).copied().unwrap_or(usize::MAX));

    let mut runs: Vec<EntityGroupRun> = Vec::new();
    for (_, key) in &ordered {
        match runs.last_mut() {
            Some(run) if run.key == *key => run.row_count += 1,
            _ => runs.push(EntityGroupRun {
                key: key.clone(),
                row_count: 1,
                collapsed: collapsed.contains(key),
            }),
        }
    }

    let mut indices = Vec::with_capacity(ordered.len());
    let mut group_keys = Vec::with_capacity(ordered.len());
    for (index, key) in ordered {
        if collapsed.contains(&key) {
            continue;
        }
        indices.push(index);
        group_keys.push(key);
    }

    EntityGroupedOrder {
        indices,
        group_keys,
        runs,
    }
}

/// One rendered group section on the current page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityGroupedSection {
    /// Stable group identity.
    pub group_key: String,
    /// Whether this section resumes a group whose earlier rows are on a
    /// previous page. Continuation headings are announced as such.
    pub continued: bool,
    /// Whether the group is collapsed, in which case it renders as a heading
    /// with no child rows at all.
    pub collapsed: bool,
    /// Filtered rows the group holds in total, ignoring collapse and paging.
    pub group_row_count: usize,
    /// Stable row keys this section paints, in row order.
    pub row_keys: Vec<String>,
    /// Page-local ordinal of this section's first row among the page's data
    /// rows. Headings are presentation and never take an ordinal.
    pub first_row_position: usize,
}

/// Builds the rendered sections for one page of a grouped table.
///
/// `page_group_keys` and `page_row_keys` are parallel and describe exactly the
/// data rows the body paints, so a section can never claim a row the table is
/// not showing. `previous_group_key` is the group of the row immediately
/// before the page window, which is the only thing that distinguishes a fresh
/// heading from a continuation.
///
/// Collapsed groups hold no rows, so they cannot be placed by a row. Each is
/// anchored to the next *fresh* section in rank order and rendered
/// immediately before it; collapsed groups with no later fresh section on any
/// page render at the end of the last page. That placement is deterministic
/// and total: every non-empty collapsed group renders on exactly one page.
pub(crate) fn entity_grouped_page_sections(
    runs: &[EntityGroupRun],
    page_group_keys: &[String],
    page_row_keys: &[String],
    previous_group_key: Option<&str>,
    is_last_page: bool,
) -> Vec<EntityGroupedSection> {
    let rank_of = |key: &str| runs.iter().position(|run| run.key == key);
    let count_of = |key: &str| {
        runs.iter()
            .find(|run| run.key == key)
            .map_or(0, |run| run.row_count)
    };

    let mut sections: Vec<EntityGroupedSection> = Vec::new();
    for (position, (group_key, row_key)) in page_group_keys.iter().zip(page_row_keys).enumerate() {
        match sections.last_mut() {
            Some(section) if section.group_key == *group_key => {
                section.row_keys.push(row_key.clone());
            }
            _ => {
                let continued = position == 0 && previous_group_key == Some(group_key.as_str());
                sections.push(EntityGroupedSection {
                    group_key: group_key.clone(),
                    continued,
                    collapsed: false,
                    group_row_count: count_of(group_key),
                    row_keys: vec![row_key.clone()],
                    first_row_position: position,
                });
            }
        }
    }

    let mut placed: Vec<EntityGroupedSection> = Vec::new();
    let mut pending: Vec<&EntityGroupRun> = runs
        .iter()
        .filter(|run| run.collapsed && run.row_count > 0)
        .collect();

    for section in sections {
        if !section.continued {
            let anchor_rank = rank_of(&section.group_key).unwrap_or(usize::MAX);
            let (before, rest): (Vec<_>, Vec<_>) = pending
                .into_iter()
                .partition(|run| rank_of(&run.key).unwrap_or(usize::MAX) < anchor_rank);
            for run in before {
                placed.push(collapsed_section(run));
            }
            pending = rest;
        }
        placed.push(section);
    }

    if is_last_page {
        for run in pending {
            placed.push(collapsed_section(run));
        }
    }

    placed
}

fn collapsed_section(run: &EntityGroupRun) -> EntityGroupedSection {
    EntityGroupedSection {
        group_key: run.key.clone(),
        continued: false,
        collapsed: true,
        group_row_count: run.row_count,
        row_keys: Vec::new(),
        first_row_position: 0,
    }
}

/// The group key of the displayed row immediately before a page window.
pub(crate) fn entity_previous_group_key(
    group_keys: &[String],
    bounds: &Range<usize>,
) -> Option<String> {
    bounds
        .start
        .checked_sub(1)
        .and_then(|index| group_keys.get(index))
        .cloned()
}

/// Columns a full-width group heading must span.
///
/// Deliberately the same arithmetic as
/// [`entity_empty_state_colspan`](super::component::entity_empty_state_colspan):
/// the heading spans the CURRENT visible column count plus the leading
/// selection control cell when one is rendered. The selection cell is not a
/// column but it is a cell, and a heading short by one leaves a ragged grid
/// line under the checkbox track (`ldui-ibjk`).
pub(crate) const fn entity_group_header_colspan(
    visible_columns: usize,
    has_selection_column: bool,
) -> usize {
    super::component::entity_empty_state_colspan(visible_columns, has_selection_column)
}

/// The display label for a group key, falling back to the key itself.
///
/// A key with no declaration is still rendered rather than hidden, because the
/// rows exist and must remain reachable.
pub(crate) fn entity_group_label(groups: &[EntityRowGroup], key: &str) -> String {
    groups
        .iter()
        .find(|group| group.key == key)
        .map_or_else(|| key.to_owned(), |group| group.label.clone())
}

/// Caller-supplied compact metadata for a group key, if declared.
pub(crate) fn entity_group_meta(groups: &[EntityRowGroup], key: &str) -> Option<String> {
    groups
        .iter()
        .find(|group| group.key == key)
        .and_then(|group| group.meta.clone())
}
