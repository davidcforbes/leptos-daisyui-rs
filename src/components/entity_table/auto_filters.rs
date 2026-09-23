//! Framework-owned "the filter row on every column" mechanics for
//! [`EntityTable`](super::EntityTable) (Office op-ulgfu, owner rule
//! 2026-09-14: *the standard record-list page is a Page Title plus one
//! opinionated EntityTable with `+` New, the filter row, pagination and an
//! Actions column*).
//!
//! `EntityTable`'s filter row is CONTROLLED: every [`EntityColumnFilter`]
//! carries a caller-owned value signal and only proposes changes. That is
//! the right contract for a page that binds its filters to a server
//! vocabulary, and the wrong amount of ceremony for the thirty record-list
//! subpages that just want a text box over every column. This module owns
//! the ceremony once:
//!
//! * one value signal per filterable column, built from the column's own
//!   [`EntityColumnFilterMode`] declaration;
//! * the filter controls (`EntityColumnFilter::text` / `::select`), with the
//!   select's option list derived from the column's distinct cell texts;
//! * the row predicate -- case-insensitive substring for a text filter,
//!   exact match for an option filter -- applied to the SOURCE rows so the
//!   table's own empty-state classification (`no_rows` vs
//!   `no_matching_rows`) keeps working through `source_data`.
//!
//! What it deliberately does not own: which columns filter (the column
//! declares that), the copy (the consumer localizes
//! [`EntityAutoFilterTexts`]), and any server round trip (this is a local
//! snapshot filter; a server-vocabulary page keeps its controlled filters).
//!
//! ```rust,ignore
//! let auto = EntityAutoFilters::new(&columns, data, "notes", texts);
//! view! {
//!     <EntityTable
//!         data=auto.rows()
//!         source_data=data
//!         column_filters=auto.filters()
//!         columns=columns
//!         // ...
//!     />
//! }
//! ```
//!
//! The record-list composite in the consuming application turns the filter
//! row ON by default: every column of a `RecordListPage` table is
//! `.filterable()` unless the page opts one out with `.not_filterable()`.
//!
//! # The header is read live (ldui-xgj8)
//!
//! Every generated label, placeholder and "All" option substitutes the
//! column HEADER into the consumer's [`EntityAutoFilterTexts`] template. The
//! texts are a signal, and consumers swap them on a language change -- but a
//! consumer that rebuilds its columns reactively (Spanish headers) needs the
//! `{column}` half to move too, or the cells read "Filtrar Work type". Build
//! with [`EntityAutoFilters::from_columns`] when your columns are a signal:
//! the filter SET (which columns, their ids, kinds, accessors and value
//! signals) is fixed from the columns' value at build time, while each
//! filter's header is looked up in the live signal by column id on every
//! read. [`EntityAutoFilters::new`] takes a plain slice and keeps a static
//! header; both go through the same internals.
//!
//! ```rust,ignore
//! let columns = Signal::derive_local(move || columns_for(language.get()));
//! let auto = EntityAutoFilters::from_columns(columns, data, "notes", texts);
//! view! { <EntityTable columns=columns column_filters=auto.filters() /* … */ /> }
//! ```
//!
//! A column whose filter is a server round trip declares
//! `.filterable_text_debounced(ms)` (ldui-ga96) and gets an
//! [`EntityColumnFilter::text_debounced`] cell: it applies once typing goes
//! quiet or on Enter, never per keystroke and never through an Apply button.
//! The local predicate still runs on the committed value.

use super::types::{
    EntityColumn, EntityColumnFilter, EntityColumnFilterOption, EntityColumnFilters,
    EntityResolvedFilterKind,
};
use leptos::prelude::*;
use std::collections::BTreeSet;
use std::rc::Rc;

/// Consumer-localized copy for framework-built filters. Every string is
/// caller-owned, like every other `*Texts` struct in this crate; the
/// defaults are English placeholders, not the product's copy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityAutoFilterTexts {
    /// Accessible name of one filter control; `{column}` is replaced by the
    /// column header. Default `"Filter {column}"`.
    pub label: String,
    /// Placeholder inside a text filter; `{column}` is replaced by the
    /// column header, same as [`Self::label`]. Default `"Filter…"`.
    pub placeholder: String,
    /// The reset option of an option-list filter; `{column}` is replaced by
    /// the column header, same as [`Self::label`]. Default `"All"`.
    pub all: String,
}

impl Default for EntityAutoFilterTexts {
    fn default() -> Self {
        Self {
            label: "Filter {column}".to_owned(),
            placeholder: "Filter…".to_owned(),
            all: "All".to_owned(),
        }
    }
}

/// Substitutes `{column}` in an [`EntityAutoFilterTexts`] template with the
/// given column header. Applied uniformly to `label`, `placeholder`, and
/// `all` so a page whose vocabulary is "phases" can read "Filter phases" /
/// "Filter phases…" / "All phases" instead of a bare "All" on the reset
/// option (4iiz-etl bd_4iiz-etl-fbhe, via ldui-v6ca).
pub fn entity_auto_filter_text(template: &str, column_header: &str) -> String {
    template.replace("{column}", column_header)
}

/// Whether one cell passes one framework-built filter value.
///
/// An empty (or whitespace-only) `value` is "no filter" and admits every
/// cell. A text filter matches a case-insensitive substring; an option
/// filter matches the cell text exactly, because its value came from the
/// column's own distinct cell texts.
pub fn entity_auto_filter_matches(kind: EntityResolvedFilterKind, cell: &str, value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return true;
    }
    match kind {
        EntityResolvedFilterKind::Text | EntityResolvedFilterKind::TextDebounced { .. } => {
            cell.to_lowercase().contains(&value.to_lowercase())
        }
        EntityResolvedFilterKind::Options => cell == value,
    }
}

/// The option list of an option-kind filter: the DISTINCT, non-empty cell
/// texts in sorted order, each its own value and label. Sorted so the list
/// is stable across refreshes that reorder rows; empty texts are dropped
/// because the empty string is the select's reserved "All" value.
pub fn entity_auto_filter_options(
    values: impl IntoIterator<Item = String>,
) -> Vec<EntityColumnFilterOption> {
    values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|value| EntityColumnFilterOption::new(value.clone(), value))
        .collect()
}

struct AutoFilterPredicate<T> {
    kind: EntityResolvedFilterKind,
    text: Rc<dyn Fn(&T) -> String>,
    value: RwSignal<String>,
}

/// The live copy of one framework-built filter: the column header it
/// substitutes and the three texts derived from it. Every field is a
/// signal, so a consumer can render the same accessible name the filter
/// row uses (a chip, a tooltip) without re-deriving the template.
#[derive(Clone, Copy, Debug)]
pub struct EntityAutoFilterCopy {
    /// The column this copy belongs to.
    pub column_id: &'static str,
    /// The header substituted into every template: live from the columns
    /// signal under [`EntityAutoFilters::from_columns`], captured once under
    /// [`EntityAutoFilters::new`].
    pub header: Signal<String>,
    /// [`EntityAutoFilterTexts::label`] with `{column}` substituted.
    pub label: Signal<String>,
    /// [`EntityAutoFilterTexts::placeholder`] with `{column}` substituted.
    pub placeholder: Signal<String>,
    /// [`EntityAutoFilterTexts::all`] with `{column}` substituted.
    pub all: Signal<String>,
}

/// The framework-built filter set for one table: its controls, its value
/// signals and the filtered rows. Build once per column set with
/// [`EntityAutoFilters::new`] (a plain slice) or
/// [`EntityAutoFilters::from_columns`] (a columns signal whose headers are
/// read live); hand [`Self::filters`] to `EntityTable::column_filters` and
/// [`Self::rows`] to `EntityTable::data`.
pub struct EntityAutoFilters<T: 'static> {
    filters: Vec<EntityColumnFilter>,
    values: Rc<Vec<(&'static str, RwSignal<String>)>>,
    copy: Rc<Vec<EntityAutoFilterCopy>>,
    rows: Signal<Rc<Vec<T>>, LocalStorage>,
}

impl<T: 'static> Clone for EntityAutoFilters<T> {
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone(),
            values: Rc::clone(&self.values),
            copy: Rc::clone(&self.copy),
            rows: self.rows,
        }
    }
}

impl<T: Clone + 'static> EntityAutoFilters<T> {
    /// Builds the filter set for every column whose
    /// [`EntityColumn::resolved_filter_kind`] is `Some`.
    ///
    /// `control_id_prefix` scopes the generated control ids
    /// (`"{prefix}-{column id}-filter"`) so two tables on one page cannot
    /// collide; it must not be empty. `data` is the table's SOURCE snapshot;
    /// pass the same signal to `EntityTable::source_data` so an over-narrow
    /// filter reads as "no matching rows" rather than "no rows".
    ///
    /// The column header each label, placeholder and "All" option
    /// substitutes is captured from this slice. When your columns are a
    /// signal that is rebuilt on a language change, build with
    /// [`Self::from_columns`] instead so the header follows it (ldui-xgj8).
    ///
    /// # Panics
    ///
    /// When `control_id_prefix` is empty, for the same reason
    /// [`EntityColumnFilter::text`] refuses an empty control id: the id is
    /// the `<label for>` target and an empty one labels nothing.
    pub fn new(
        columns: &[EntityColumn<T>],
        data: Signal<Rc<Vec<T>>, LocalStorage>,
        control_id_prefix: impl Into<String>,
        texts: impl Into<Signal<EntityAutoFilterTexts>>,
    ) -> Self {
        Self::build(columns, None, data, control_id_prefix, texts)
    }

    /// Builds the filter set from a columns SIGNAL, reading each column's
    /// header live (ldui-xgj8).
    ///
    /// The filter set itself -- which columns filter, their control ids
    /// (`"{prefix}-{column id}-filter"`), kinds, accessors and value signals
    /// -- is fixed from the signal's value at build time, exactly as
    /// [`Self::new`] fixes it from a slice, so filter identity survives a
    /// column rebuild: a typed value keeps filtering and a saved filter
    /// keeps applying. What follows the signal is the copy: every label,
    /// placeholder and "All" option looks its column up in the live signal
    /// by id on each read, so a consumer that swaps to Spanish headers and
    /// Spanish [`EntityAutoFilterTexts`] gets "Filtrar Estado", not
    /// "Filtrar Status". A column whose id has vanished from the signal
    /// falls back to the header captured at build time.
    ///
    /// Pass the same signal to `EntityTable::columns`. Adding or removing a
    /// FILTERABLE column later needs a rebuild -- the set does not grow.
    ///
    /// # Panics
    ///
    /// When `control_id_prefix` is empty; see [`Self::new`].
    pub fn from_columns(
        columns: Signal<Vec<EntityColumn<T>>, LocalStorage>,
        data: Signal<Rc<Vec<T>>, LocalStorage>,
        control_id_prefix: impl Into<String>,
        texts: impl Into<Signal<EntityAutoFilterTexts>>,
    ) -> Self {
        columns.with_untracked(|snapshot| {
            Self::build(snapshot, Some(columns), data, control_id_prefix, texts)
        })
    }

    fn build(
        columns: &[EntityColumn<T>],
        live_columns: Option<Signal<Vec<EntityColumn<T>>, LocalStorage>>,
        data: Signal<Rc<Vec<T>>, LocalStorage>,
        control_id_prefix: impl Into<String>,
        texts: impl Into<Signal<EntityAutoFilterTexts>>,
    ) -> Self {
        let prefix = control_id_prefix.into();
        assert!(
            !prefix.trim().is_empty(),
            "EntityAutoFilters control_id_prefix must not be empty"
        );
        let texts = texts.into();
        // `Signal::derive` wants a `Send + Sync` closure and the columns
        // signal is thread-local; park it in a `StoredValue::new_local` so
        // the closures below capture only a `Copy` handle (the same pattern
        // the option list uses for `data`).
        let live_columns = StoredValue::new_local(live_columns);
        let mut filters = Vec::new();
        let mut values = Vec::new();
        let mut copy = Vec::new();
        let mut predicates: Vec<AutoFilterPredicate<T>> = Vec::new();
        for column in columns {
            let Some(kind) = column.resolved_filter_kind() else {
                continue;
            };
            let value = RwSignal::new(String::new());
            let control_id = format!("{prefix}-{}-filter", column.id);
            let column_id = column.id;
            let captured_header = column.header.clone();
            let header = Signal::derive(move || {
                live_columns
                    .with_value(|live| {
                        live.as_ref().and_then(|columns| {
                            columns.with(|columns| {
                                columns
                                    .iter()
                                    .find(|column| column.id == column_id)
                                    .map(|column| column.header.clone())
                            })
                        })
                    })
                    .unwrap_or_else(|| captured_header.clone())
            });
            let label = Signal::derive(move || {
                texts.with(|texts| {
                    header.with(|header| entity_auto_filter_text(&texts.label, header))
                })
            });
            let placeholder = Signal::derive(move || {
                texts.with(|texts| {
                    header.with(|header| entity_auto_filter_text(&texts.placeholder, header))
                })
            });
            let all = Signal::derive(move || {
                texts
                    .with(|texts| header.with(|header| entity_auto_filter_text(&texts.all, header)))
            });
            copy.push(EntityAutoFilterCopy {
                column_id,
                header,
                label,
                placeholder,
                all,
            });
            let on_change = Callback::new(move |next: String| value.set(next));
            let filter = match kind {
                EntityResolvedFilterKind::Text => EntityColumnFilter::text(
                    column.id,
                    control_id,
                    label,
                    value,
                    placeholder,
                    on_change,
                ),
                EntityResolvedFilterKind::TextDebounced { debounce_ms } => {
                    EntityColumnFilter::text_debounced(
                        column.id,
                        control_id,
                        label,
                        value,
                        placeholder,
                        on_change,
                        debounce_ms,
                    )
                }
                EntityResolvedFilterKind::Options => {
                    // The option list is derived from the live source rows.
                    // `Signal::derive` wants a `Send + Sync` closure, and the
                    // column's text accessor is an `Rc`; parking both the
                    // accessor and the (thread-local-storage) data signal in
                    // `StoredValue::new_local` makes the closure capture only
                    // `Copy` handles, which is the established pattern here.
                    let text = StoredValue::new_local(column.filter_accessor());
                    let source = StoredValue::new_local(data);
                    let options = Signal::derive(move || {
                        source.with_value(|source| {
                            source.with(|rows| {
                                text.with_value(|text| {
                                    entity_auto_filter_options(rows.iter().map(|row| text(row)))
                                })
                            })
                        })
                    });
                    EntityColumnFilter::select(
                        column.id, control_id, label, value, all, options, on_change,
                    )
                }
            };
            filters.push(filter);
            values.push((column.id, value));
            predicates.push(AutoFilterPredicate {
                kind,
                text: column.filter_accessor(),
                value,
            });
        }

        let rows = Signal::derive_local(move || {
            let source = data.get();
            let active = predicates
                .iter()
                .filter_map(|predicate| {
                    let value = predicate.value.get();
                    (!value.trim().is_empty())
                        .then(|| (predicate.kind, Rc::clone(&predicate.text), value))
                })
                .collect::<Vec<_>>();
            if active.is_empty() {
                return source;
            }
            Rc::new(
                source
                    .iter()
                    .filter(|row| {
                        active.iter().all(|(kind, text, value)| {
                            entity_auto_filter_matches(*kind, &text(row), value)
                        })
                    })
                    .cloned()
                    .collect::<Vec<T>>(),
            )
        });

        Self {
            filters,
            values: Rc::new(values),
            copy: Rc::new(copy),
            rows,
        }
    }

    /// The controls, for `EntityTable::column_filters`. Static: the set is
    /// fixed by the column declarations this was built from.
    pub fn filters(&self) -> EntityColumnFilters {
        EntityColumnFilters::Static(self.filters.clone())
    }

    /// The live copy (header, label, placeholder, "All") of one column's
    /// filter, or `None` when the column has no framework-built filter.
    /// Under [`Self::from_columns`] every signal follows the columns signal;
    /// under [`Self::new`] the header is the one captured at build time.
    pub fn copy(&self, column_id: &str) -> Option<EntityAutoFilterCopy> {
        self.copy
            .iter()
            .find(|copy| copy.column_id == column_id)
            .copied()
    }

    /// The accessible name of one column's filter control, live; see
    /// [`Self::copy`].
    pub fn label(&self, column_id: &str) -> Option<Signal<String>> {
        self.copy(column_id).map(|copy| copy.label)
    }

    /// The placeholder of one column's text filter, live; see
    /// [`Self::copy`]. Present for an option filter too (unused there).
    pub fn placeholder(&self, column_id: &str) -> Option<Signal<String>> {
        self.copy(column_id).map(|copy| copy.placeholder)
    }

    /// The reset-option label of one column's option filter, live; see
    /// [`Self::copy`]. Present for a text filter too (unused there).
    pub fn all_label(&self, column_id: &str) -> Option<Signal<String>> {
        self.copy(column_id).map(|copy| copy.all)
    }

    /// The filtered rows, for `EntityTable::data`.
    pub fn rows(&self) -> Signal<Rc<Vec<T>>, LocalStorage> {
        self.rows
    }

    /// How many columns received a framework-built filter.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Whether no column asked for a framework-built filter.
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// The value signal behind one column's filter, for a page that wants to
    /// seed or read it (a deep link, a saved view).
    pub fn value(&self, column_id: &str) -> Option<RwSignal<String>> {
        self.values
            .iter()
            .find(|(id, _)| *id == column_id)
            .map(|(_, value)| *value)
    }

    /// Reactive count of filters with a non-empty value.
    pub fn active_count(&self) -> usize {
        self.values
            .iter()
            .filter(|(_, value)| value.with(|value| !value.trim().is_empty()))
            .count()
    }

    /// Clears every filter value.
    pub fn clear_all(&self) {
        for (_, value) in self.values.iter() {
            value.set(String::new());
        }
    }

    /// The current captured values as `(column_id, value)` pairs, one per
    /// framework-built filter -- the shape
    /// [`EntitySavedFilter::values`](super::EntitySavedFilter::values)
    /// stores and applies. Empty values are included so a save reflects the
    /// full filter state; consumers comparing to a saved set use the same
    /// ordering this returns (declaration order).
    pub fn current_values(&self) -> Vec<(String, String)> {
        self.values
            .iter()
            .map(|(column_id, value)| ((*column_id).to_owned(), value.get()))
            .collect()
    }
}

impl<T: 'static> std::fmt::Debug for EntityAutoFilters<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EntityAutoFilters")
            .field("filters", &self.filters)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::badge::BadgeColor;
    use crate::components::entity_table::{EntityBadgePresentation, EntityColumnFilterMode};
    use leptos::reactive::owner::Owner;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Survey {
        client: &'static str,
        status: &'static str,
        score: u32,
    }

    fn surveys() -> Vec<Survey> {
        vec![
            Survey {
                client: "Acme Holdings",
                status: "Open",
                score: 9,
            },
            Survey {
                client: "Beta Legal",
                status: "Closed",
                score: 4,
            },
            Survey {
                client: "Acme Partners",
                status: "Open",
                score: 7,
            },
        ]
    }

    fn columns() -> Vec<EntityColumn<Survey>> {
        vec![
            EntityColumn::text("client", "Client", |row: &Survey| row.client.to_owned())
                .filterable(),
            EntityColumn::text("status", "Status", |row: &Survey| row.status.to_owned())
                .badge_with(|row: &Survey| {
                    Some(EntityBadgePresentation::new(if row.status == "Open" {
                        BadgeColor::Success
                    } else {
                        BadgeColor::Neutral
                    }))
                })
                .filterable(),
            EntityColumn::text("score", "Score", |row: &Survey| row.score.to_string())
                .numeric()
                .filterable(),
            EntityColumn::text("note", "Note", |_: &Survey| String::new()),
            EntityColumn::action("actions", "Actions", |_: &Survey| String::new()).filterable(),
        ]
    }

    #[test]
    fn auto_resolves_options_for_a_badge_column_and_text_for_the_rest() {
        let columns = columns();
        assert_eq!(
            columns[0].resolved_filter_kind(),
            Some(EntityResolvedFilterKind::Text)
        );
        assert_eq!(
            columns[1].resolved_filter_kind(),
            Some(EntityResolvedFilterKind::Options),
            "a badge-presented column is categorical by declaration"
        );
        assert_eq!(
            columns[2].resolved_filter_kind(),
            Some(EntityResolvedFilterKind::Text)
        );
        assert_eq!(columns[3].resolved_filter_kind(), None, "not declared");
        assert_eq!(
            columns[4].resolved_filter_kind(),
            None,
            "an action column never filters"
        );
        assert_eq!(columns[4].filter_mode, EntityColumnFilterMode::Auto);
    }

    #[test]
    fn explicit_modes_override_the_presentation_rule() {
        let text = EntityColumn::text("status", "Status", |row: &Survey| row.status.to_owned())
            .filterable_options();
        assert_eq!(
            text.resolved_filter_kind(),
            Some(EntityResolvedFilterKind::Options)
        );
        let badge = columns().remove(1).filterable_text();
        assert_eq!(
            badge.resolved_filter_kind(),
            Some(EntityResolvedFilterKind::Text)
        );
        let off = columns().remove(0).not_filterable();
        assert_eq!(off.resolved_filter_kind(), None);
        assert_eq!(
            EntityColumnFilterMode::default(),
            EntityColumnFilterMode::None
        );
    }

    #[test]
    fn column_text_substitutes_column_into_every_template_including_placeholder_and_all() {
        assert_eq!(
            entity_auto_filter_text("Filter {column}", "Phase"),
            "Filter Phase"
        );
        assert_eq!(
            entity_auto_filter_text("Filter {column}…", "Phase"),
            "Filter Phase…"
        );
        assert_eq!(
            entity_auto_filter_text("All {column}", "phases"),
            "All phases"
        );
        assert_eq!(
            entity_auto_filter_text("All", "phases"),
            "All",
            "a template with no placeholder is returned unchanged"
        );
    }

    #[test]
    fn the_predicate_is_substring_for_text_and_exact_for_options() {
        use EntityResolvedFilterKind::{Options, Text};
        assert!(entity_auto_filter_matches(Text, "Acme Holdings", "hold"));
        assert!(entity_auto_filter_matches(Text, "Acme Holdings", "  ACME "));
        assert!(!entity_auto_filter_matches(Text, "Acme Holdings", "beta"));
        assert!(
            entity_auto_filter_matches(Text, "anything", ""),
            "empty = no filter"
        );
        assert!(entity_auto_filter_matches(Options, "Open", "Open"));
        assert!(
            !entity_auto_filter_matches(Options, "Open", "open"),
            "options are exact"
        );
        assert!(entity_auto_filter_matches(Options, "Open", "   "));
    }

    #[test]
    fn options_are_distinct_sorted_and_never_include_the_reserved_empty_value() {
        let options = entity_auto_filter_options(
            ["Open", "Closed", "Open", "", "  "]
                .into_iter()
                .map(str::to_owned),
        );
        assert_eq!(
            options,
            vec![
                EntityColumnFilterOption::new("Closed", "Closed"),
                EntityColumnFilterOption::new("Open", "Open"),
            ]
        );
    }

    #[test]
    fn a_filter_set_builds_one_control_per_declared_column_and_filters_the_rows() {
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let columns = columns();
            let auto = EntityAutoFilters::new(
                &columns,
                data.into(),
                "nps",
                EntityAutoFilterTexts::default(),
            );

            assert_eq!(
                auto.len(),
                3,
                "client, status and score; note undeclared, actions refused"
            );
            let EntityColumnFilters::Static(filters) = auto.filters() else {
                panic!("auto filters are a static declaration set");
            };
            let ids = filters
                .iter()
                .map(|filter| filter.column_id)
                .collect::<Vec<_>>();
            assert_eq!(ids, vec!["client", "status", "score"]);
            assert_eq!(filters[0].control_id(), Some("nps-client-filter"));
            assert!(auto.value("note").is_none());

            assert_eq!(auto.rows().get().len(), 3, "no value set: every source row");
            assert_eq!(auto.active_count(), 0);

            auto.value("client")
                .expect("client filter")
                .set("acme".to_owned());
            assert_eq!(auto.active_count(), 1);
            let rows = auto.rows().get();
            assert_eq!(rows.len(), 2);
            assert!(rows.iter().all(|row| row.client.starts_with("Acme")));

            auto.value("status")
                .expect("status filter")
                .set("Closed".to_owned());
            assert_eq!(auto.rows().get().len(), 0, "filters AND together");

            auto.clear_all();
            assert_eq!(auto.active_count(), 0);
            assert_eq!(auto.rows().get().len(), 3);

            // A source refresh flows through with the current filter applied.
            auto.value("score")
                .expect("score filter")
                .set("9".to_owned());
            data.set(Rc::new(vec![
                Survey {
                    client: "Gamma",
                    status: "Open",
                    score: 9,
                },
                Survey {
                    client: "Delta",
                    status: "Open",
                    score: 19,
                },
            ]));
            assert_eq!(
                auto.rows().get().len(),
                2,
                "substring: 9 matches 19 as well"
            );
        });
    }

    #[test]
    fn current_values_captures_every_column_in_declaration_order() {
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let columns = columns();
            let auto = EntityAutoFilters::new(
                &columns,
                data.into(),
                "nps",
                EntityAutoFilterTexts::default(),
            );

            // Initially every value is empty, in declaration order.
            assert_eq!(
                auto.current_values(),
                vec![
                    ("client".to_owned(), String::new()),
                    ("status".to_owned(), String::new()),
                    ("score".to_owned(), String::new()),
                ]
            );

            auto.value("status")
                .expect("status filter")
                .set("Open".to_owned());
            auto.value("client")
                .expect("client filter")
                .set("acme".to_owned());
            assert_eq!(
                auto.current_values(),
                vec![
                    ("client".to_owned(), "acme".to_owned()),
                    ("status".to_owned(), "Open".to_owned()),
                    ("score".to_owned(), String::new()),
                ],
                "the shape saved filters store and badges compare,"
            );

            auto.clear_all();
            assert!(auto.current_values().iter().all(|(_, v)| v.is_empty()));
        });
    }

    #[test]
    fn rows_react_to_an_external_set_through_a_stored_value_handle() {
        // The controlled filter signal is settable from OUTSIDE the component
        // (a saved-filters apply, a deep link) -- this is the shape the saved-
        // filters fixture uses, with the `EntityAutoFilters` held in a
        // `StoredValue` and its rows signal handed off once at build time.
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let auto = StoredValue::new_local(EntityAutoFilters::new(
                &columns(),
                data.into(),
                "nps",
                EntityAutoFilterTexts::default(),
            ));
            // Handed off once, like `data=auto.with_value(|a| a.rows())`.
            let rows_signal = auto.with_value(|a| a.rows());
            assert_eq!(rows_signal.get().len(), 3);

            // The exact apply path: clear, then set just the named columns.
            auto.with_value(|a| {
                a.value("status")
                    .expect("status filter")
                    .set("Closed".to_owned())
            });
            assert_eq!(
                rows_signal.get().len(),
                1,
                "a set from outside must narrow the derived rows"
            );
        });
    }

    /// A column that exports a wire code and renders a localized label must
    /// MATCH the label. Matching the exported value instead is the
    /// silent-wrong-match class this accessor exists to prevent: typing what is
    /// on screen matches nothing, and reports itself as an ordinary empty
    /// result rather than an error (Office op-e6dsi).
    #[test]
    fn a_filter_column_matches_its_filter_text_not_its_exported_text() {
        let owner = Owner::new();
        owner.with(|| {
            let columns = vec![
                EntityColumn::text("status", "Status", |row: &Survey| match row.status {
                    // Exported/stable: the wire code a downstream system needs.
                    "Open" => "ST_OPEN".to_owned(),
                    _ => "ST_CLOSED".to_owned(),
                })
                .with_filter_text(|row: &Survey| row.status.to_owned())
                .filterable_options(),
            ];
            let data = RwSignal::new_local(Rc::new(surveys()));
            let filters = EntityAutoFilters::new(
                &columns,
                data.into(),
                "survey",
                EntityAutoFilterTexts::default(),
            );
            let value = filters
                .value("status")
                .expect("the declared column gets a filter value signal");

            value.set("Open".to_owned());
            assert_eq!(
                filters.rows().get().len(),
                2,
                "matching must run against the filter text the user can read"
            );

            value.set("ST_OPEN".to_owned());
            assert_eq!(
                filters.rows().get().len(),
                0,
                "the exported code is no longer what the filter matches"
            );
        });
    }

    /// The fallback is the common case and must stay silent: a column that
    /// never calls `with_filter_text` filters exactly as it did before the
    /// accessor existed.
    #[test]
    fn a_column_without_filter_text_still_filters_by_its_exported_text() {
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let filters = EntityAutoFilters::new(
                &columns(),
                data.into(),
                "survey",
                EntityAutoFilterTexts::default(),
            );
            filters
                .value("status")
                .expect("status declares a filter")
                .set("Open".to_owned());
            assert_eq!(filters.rows().get().len(), 2);
        });
    }

    #[test]
    #[should_panic(expected = "EntityAutoFilters control_id_prefix must not be empty")]
    fn an_empty_control_id_prefix_is_refused() {
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let _ = EntityAutoFilters::new(
                &columns(),
                data.into(),
                "  ",
                EntityAutoFilterTexts::default(),
            );
        });
    }

    /// `columns()` with the given headers, same ids: what a consumer's
    /// `Signal::derive_local(move || columns_for(language.get()))` yields
    /// on either side of a language change.
    fn columns_with_headers(client: &str, status: &str) -> Vec<EntityColumn<Survey>> {
        vec![
            EntityColumn::text("client", client, |row: &Survey| row.client.to_owned()).filterable(),
            EntityColumn::text("status", status, |row: &Survey| row.status.to_owned())
                .filterable_options(),
        ]
    }

    fn spanish_texts() -> EntityAutoFilterTexts {
        EntityAutoFilterTexts {
            label: "Filtrar {column}".to_owned(),
            placeholder: "Filtrar {column}…".to_owned(),
            all: "Todos ({column})".to_owned(),
        }
    }

    /// ldui-xgj8: the header a filter substitutes must follow the columns
    /// signal, while the filter's IDENTITY (control id, value signal,
    /// predicate) stays what it was built as.
    #[test]
    fn from_columns_reads_the_header_live_and_keeps_filter_identity() {
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let columns = RwSignal::new_local(columns_with_headers("Client", "Status"));
            let texts = RwSignal::new(EntityAutoFilterTexts {
                placeholder: "Filter {column}…".to_owned(),
                all: "All ({column})".to_owned(),
                ..EntityAutoFilterTexts::default()
            });
            let auto = EntityAutoFilters::from_columns(
                columns.into(),
                data.into(),
                "nps",
                Signal::from(texts),
            );
            assert_eq!(auto.len(), 2);

            let status = auto.copy("status").expect("status filter copy");
            assert_eq!(status.header.get(), "Status");
            assert_eq!(status.label.get(), "Filter Status");
            assert_eq!(status.all.get(), "All (Status)");
            let client_label = auto.label("client").expect("client label");
            let client_placeholder = auto.placeholder("client").expect("client placeholder");
            assert_eq!(client_label.get(), "Filter Client");
            assert_eq!(client_placeholder.get(), "Filter Client…");
            assert!(auto.copy("note").is_none(), "no filter, no copy");

            // A value typed BEFORE the language change keeps filtering.
            auto.value("status")
                .expect("status filter")
                .set("Closed".to_owned());
            assert_eq!(auto.rows().get().len(), 1);

            // The consumer swaps its columns and its texts, as Office does.
            columns.set(columns_with_headers("Cliente", "Estado"));
            texts.set(spanish_texts());

            assert_eq!(
                status.label.get(),
                "Filtrar Estado",
                "the {{column}} half must move with the columns signal"
            );
            assert_eq!(status.all.get(), "Todos (Estado)");
            assert_eq!(client_label.get(), "Filtrar Cliente");
            assert_eq!(client_placeholder.get(), "Filtrar Cliente…");

            // Identity is untouched: same control ids, same value signal,
            // same rows.
            let EntityColumnFilters::Static(filters) = auto.filters() else {
                panic!("auto filters are a static declaration set");
            };
            assert_eq!(filters[0].control_id(), Some("nps-client-filter"));
            assert_eq!(filters[1].control_id(), Some("nps-status-filter"));
            assert_eq!(
                auto.value("status").expect("status filter").get(),
                "Closed",
                "the value signal is the one built, not a fresh one"
            );
            assert_eq!(auto.rows().get().len(), 1, "the typed value still applies");

            // Only the texts changing back moves the template, not the header.
            texts.set(EntityAutoFilterTexts::default());
            assert_eq!(status.label.get(), "Filter Estado");
        });
    }

    /// A column whose id has left the live signal keeps the header it was
    /// built with rather than reading as "Filter " -- the set is fixed at
    /// build time, so the control still exists and still needs a name.
    #[test]
    fn from_columns_falls_back_to_the_captured_header_when_the_id_vanishes() {
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let columns = RwSignal::new_local(columns_with_headers("Client", "Status"));
            let auto = EntityAutoFilters::from_columns(
                columns.into(),
                data.into(),
                "nps",
                EntityAutoFilterTexts::default(),
            );
            let label = auto.label("status").expect("status label");
            assert_eq!(label.get(), "Filter Status");

            columns.set(vec![
                EntityColumn::text("client", "Cliente", |row: &Survey| row.client.to_owned())
                    .filterable(),
            ]);
            assert_eq!(
                auto.label("client").expect("client label").get(),
                "Filter Cliente"
            );
            assert_eq!(
                label.get(),
                "Filter Status",
                "captured header is the fallback"
            );
            assert_eq!(auto.len(), 2, "the set never shrinks after build");
        });
    }

    /// The slice constructor is unchanged: its header is the one captured.
    #[test]
    fn new_with_a_slice_keeps_a_static_header_and_still_follows_the_texts() {
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let texts = RwSignal::new(EntityAutoFilterTexts::default());
            let auto = EntityAutoFilters::new(
                &columns_with_headers("Client", "Status"),
                data.into(),
                "nps",
                Signal::from(texts),
            );
            let label = auto.label("status").expect("status label");
            assert_eq!(label.get(), "Filter Status");
            texts.set(spanish_texts());
            assert_eq!(label.get(), "Filtrar Status");
            assert_eq!(auto.copy("status").expect("copy").header.get(), "Status");
        });
    }

    /// ldui-ga96: the debounced mode resolves to its own kind (so the auto
    /// path builds the debounced cell), overrides a badge presentation like
    /// the other explicit modes, and matches exactly as a text filter does.
    #[test]
    fn filterable_text_debounced_resolves_to_the_debounced_kind_with_the_text_predicate() {
        let column = EntityColumn::text("client", "Client", |row: &Survey| row.client.to_owned())
            .filterable_text_debounced(300);
        assert_eq!(
            column.filter_mode,
            EntityColumnFilterMode::TextDebounced { debounce_ms: 300 }
        );
        assert_eq!(column.filter_mode.as_str(), "text-debounced");
        let kind = column.resolved_filter_kind();
        assert_eq!(
            kind,
            Some(EntityResolvedFilterKind::TextDebounced { debounce_ms: 300 })
        );
        assert_eq!(
            kind.expect("declared").as_str(),
            "text",
            "the rendered data-entity-filter-kind stays text; apply says debounced"
        );

        let badge = columns().remove(1).filterable_text_debounced(250);
        assert_eq!(
            badge.resolved_filter_kind(),
            Some(EntityResolvedFilterKind::TextDebounced { debounce_ms: 250 }),
            "an explicit mode overrides the badge presentation rule"
        );
        assert_eq!(
            columns()
                .remove(4)
                .filterable_text_debounced(250)
                .resolved_filter_kind(),
            None,
            "an action column never filters"
        );

        let kind = EntityResolvedFilterKind::TextDebounced { debounce_ms: 300 };
        assert!(entity_auto_filter_matches(kind, "Acme Holdings", "hold"));
        assert!(entity_auto_filter_matches(kind, "Acme Holdings", "  ACME "));
        assert!(!entity_auto_filter_matches(kind, "Acme Holdings", "beta"));
        assert!(entity_auto_filter_matches(kind, "anything", ""));
    }

    /// The auto path builds the debounced cell under the same identity rules
    /// and filters locally on the COMMITTED value signal.
    #[test]
    fn a_debounced_column_gets_a_filter_with_the_standard_identity_and_local_predicate() {
        let owner = Owner::new();
        owner.with(|| {
            let data = RwSignal::new_local(Rc::new(surveys()));
            let columns = vec![
                EntityColumn::text("client", "Client", |row: &Survey| row.client.to_owned())
                    .filterable_text_debounced(300),
                EntityColumn::text("status", "Status", |row: &Survey| row.status.to_owned())
                    .filterable(),
            ];
            let auto = EntityAutoFilters::new(
                &columns,
                data.into(),
                "nps",
                EntityAutoFilterTexts::default(),
            );
            assert_eq!(auto.len(), 2);
            let EntityColumnFilters::Static(filters) = auto.filters() else {
                panic!("auto filters are a static declaration set");
            };
            assert_eq!(filters[0].column_id, "client");
            assert_eq!(filters[0].control_id(), Some("nps-client-filter"));
            assert_eq!(auto.label("client").expect("label").get(), "Filter Client");

            // The committed value is what the rows follow; the cell's own
            // typing buffer never reaches the predicate.
            auto.value("client")
                .expect("client filter")
                .set("acme".to_owned());
            assert_eq!(auto.active_count(), 1);
            assert_eq!(auto.rows().get().len(), 2);
            auto.clear_all();
            assert_eq!(auto.rows().get().len(), 3);
        });
    }
}
