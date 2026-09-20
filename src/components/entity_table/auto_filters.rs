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
    /// Placeholder inside a text filter. Default `"Filter…"`.
    pub placeholder: String,
    /// The reset option of an option-list filter. Default `"All"`.
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
        EntityResolvedFilterKind::Text => cell.to_lowercase().contains(&value.to_lowercase()),
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

/// The framework-built filter set for one table: its controls, its value
/// signals and the filtered rows. Build once per column set with
/// [`EntityAutoFilters::new`]; hand [`Self::filters`] to
/// `EntityTable::column_filters` and [`Self::rows`] to `EntityTable::data`.
pub struct EntityAutoFilters<T: 'static> {
    filters: Vec<EntityColumnFilter>,
    values: Rc<Vec<(&'static str, RwSignal<String>)>>,
    rows: Signal<Rc<Vec<T>>, LocalStorage>,
}

impl<T: 'static> Clone for EntityAutoFilters<T> {
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone(),
            values: Rc::clone(&self.values),
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
        let prefix = control_id_prefix.into();
        assert!(
            !prefix.trim().is_empty(),
            "EntityAutoFilters control_id_prefix must not be empty"
        );
        let texts = texts.into();
        let mut filters = Vec::new();
        let mut values = Vec::new();
        let mut predicates: Vec<AutoFilterPredicate<T>> = Vec::new();
        for column in columns {
            let Some(kind) = column.resolved_filter_kind() else {
                continue;
            };
            let value = RwSignal::new(String::new());
            let control_id = format!("{prefix}-{}-filter", column.id);
            let header = column.header.clone();
            let label = Signal::derive(move || {
                texts.with(|texts| texts.label.replace("{column}", &header))
            });
            let on_change = Callback::new(move |next: String| value.set(next));
            let filter = match kind {
                EntityResolvedFilterKind::Text => {
                    let placeholder =
                        Signal::derive(move || texts.with(|texts| texts.placeholder.clone()));
                    EntityColumnFilter::text(
                        column.id,
                        control_id,
                        label,
                        value,
                        placeholder,
                        on_change,
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
                    let all = Signal::derive(move || texts.with(|texts| texts.all.clone()));
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
            rows,
        }
    }

    /// The controls, for `EntityTable::column_filters`. Static: the set is
    /// fixed by the column declarations this was built from.
    pub fn filters(&self) -> EntityColumnFilters {
        EntityColumnFilters::Static(self.filters.clone())
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
}
