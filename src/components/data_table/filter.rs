//! Per-column filter row for [`DataTable`](super::DataTable): the pure
//! filtering logic plus the aligned dropdown/text controls that drive it.
//!
//! Opt-in per column via [`Column::filterable`](super::Column::filterable).
//! Callers that set no `filterable` column get no filter row and are entirely
//! unaffected.
//!
//! The pure functions here (`distinct_values`, `row_matches_filters`,
//! `prune_stale_filters`) hold all the behaviour worth testing and are
//! unit-tested without a DOM, matching `resize.rs`/`auto_page.rs`.

use crate::components::data_table::types::{Column, ColumnFilterKind, TableRow};
use leptos::html::Input;
use leptos::prelude::*;
use std::collections::HashMap;
use std::fmt;
use web_sys::wasm_bindgen::JsCast;

/// The `<select>` value meaning "no filter on this column". The empty string
/// can never collide with a real option because [`distinct_values`] skips
/// empty cells.
pub const FILTER_ALL: &str = "";

/// Delay before a column substring input replaces the active query value.
pub const COLUMN_TEXT_FILTER_DEBOUNCE_MS: u64 = 150;

/// Active per-column filter selections, keyed by [`Column::id`]. A missing key
/// or a [`FILTER_ALL`] value means the column is not filtering.
pub type ColumnFilters = HashMap<&'static str, String>;

/// One exact-value filter choice with separate transport identity and display
/// copy. `value` is stored in [`ColumnFilters`] and sent to the server; `label`
/// is reactive presentation supplied through the surrounding options signal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataTableFilterOption {
    /// Stable query/transport value. Empty is reserved for [`FILTER_ALL`].
    pub value: String,
    /// User-facing, localizable label.
    pub label: String,
}

impl DataTableFilterOption {
    /// Creates an option whose stable value and display label may differ.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }

    /// Creates the source-compatible shorthand where value equals label.
    pub fn same(value: impl Into<String>) -> Self {
        let value = value.into();
        Self::new(value.clone(), value)
    }
}

/// Exact filter choices keyed by [`Column::id`].
pub type DataTableFilterOptions = HashMap<&'static str, Vec<DataTableFilterOption>>;

/// Converts the historical string-only option map into typed value/label
/// pairs without changing ordering.
pub fn filter_options_from_strings(
    options: HashMap<&'static str, Vec<String>>,
) -> DataTableFilterOptions {
    options
        .into_iter()
        .map(|(column, options)| {
            (
                column,
                options
                    .into_iter()
                    .map(DataTableFilterOption::same)
                    .collect(),
            )
        })
        .collect()
}

/// Invalid typed vocabulary that would alias a real option with the empty
/// sentinel or map two labels to the same submitted value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataTableFilterOptionError {
    /// A real option used the reserved empty value.
    EmptyValue {
        /// Column containing the invalid option.
        column: &'static str,
        /// Zero-based option position.
        index: usize,
    },
    /// Two options in one column used the same stable value.
    DuplicateValue {
        /// Column containing the duplicate.
        column: &'static str,
        /// Duplicated stable value.
        value: String,
        /// Position of the first option.
        first_index: usize,
        /// Position of the duplicate option.
        duplicate_index: usize,
    },
}

impl fmt::Display for DataTableFilterOptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyValue { column, index } => write!(
                formatter,
                "DataTable filter option {index} for column {column:?} has an empty reserved value"
            ),
            Self::DuplicateValue {
                column,
                value,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "DataTable filter options {first_index} and {duplicate_index} for column {column:?} share value {value:?}"
            ),
        }
    }
}

/// Validates that every real option has one unique, non-empty submitted value
/// within its column.
pub fn validate_filter_options(
    options: &DataTableFilterOptions,
) -> Result<(), DataTableFilterOptionError> {
    for (column, options) in options {
        let mut seen = HashMap::<&str, usize>::with_capacity(options.len());
        for (index, option) in options.iter().enumerate() {
            if option.value.trim().is_empty() {
                return Err(DataTableFilterOptionError::EmptyValue {
                    column: *column,
                    index,
                });
            }
            if let Some(first_index) = seen.insert(option.value.as_str(), index) {
                return Err(DataTableFilterOptionError::DuplicateValue {
                    column: *column,
                    value: option.value.clone(),
                    first_index,
                    duplicate_index: index,
                });
            }
        }
    }
    Ok(())
}

/// The distinct, non-empty values of `col_id` across `data`, sorted and
/// deduplicated -- the option list for that column's filter dropdown.
///
/// Empty cells are skipped rather than offered as a blank option: they would be
/// indistinguishable from [`FILTER_ALL`] in the `<select>`.
///
/// ```
/// use std::collections::HashMap;
/// use leptos_daisyui_rs::components::distinct_values;
///
/// let data = vec![
///     HashMap::from([("status", "Open".to_string())]),
///     HashMap::from([("status", "Closed".to_string())]),
///     HashMap::from([("status", "Open".to_string())]),
/// ];
/// assert_eq!(distinct_values(&data, "status"), vec!["Closed", "Open"]);
/// ```
pub fn distinct_values(data: &[TableRow], col_id: &str) -> Vec<String> {
    let mut values: Vec<String> = data
        .iter()
        .filter_map(|row| row.get(col_id))
        .filter(|v| !v.is_empty())
        .cloned()
        .collect();
    values.sort();
    values.dedup();
    values
}

/// Whether `row` satisfies every active filter in `filters` (an AND across
/// columns). Filters set to [`FILTER_ALL`] are inactive and ignored.
///
/// Matching is exact and case-sensitive: the options come from the data itself,
/// so a selected option always corresponds to a real cell value. This is
/// deliberately unlike the free-text `searchable` box, which is a
/// case-insensitive substring match across the declared searchable columns
/// ([`row_matches_search`]).
///
/// A filter on a column the row has no value for excludes the row -- the row
/// cannot equal the selected value.
pub fn row_matches_filters(row: &TableRow, filters: &ColumnFilters) -> bool {
    filters.iter().all(|(col_id, wanted)| {
        if wanted == FILTER_ALL {
            return true;
        }
        row.get(*col_id).map(|v| v == wanted).unwrap_or(false)
    })
}

/// Whether `row` satisfies every active filter using the matching behavior
/// declared by `columns`.
///
/// Exact columns preserve [`row_matches_filters`] behavior. Text columns use
/// a Unicode-aware lowercase substring comparison. An unknown/stale column id
/// falls back to exact matching, preserving the historical map contract.
pub fn row_matches_column_filters(
    row: &TableRow,
    columns: &[Column],
    filters: &ColumnFilters,
) -> bool {
    filters.iter().all(|(col_id, wanted)| {
        if wanted == FILTER_ALL {
            return true;
        }
        let kind = columns
            .iter()
            .find(|column| column.id == *col_id)
            .and_then(Column::filter_kind)
            .unwrap_or(ColumnFilterKind::Exact);
        row.get(*col_id).is_some_and(|value| match kind {
            ColumnFilterKind::Exact => value == wanted,
            ColumnFilterKind::Contains => value.to_lowercase().contains(&wanted.to_lowercase()),
        })
    })
}

/// Drop any filter selection that is no longer one of its column's available
/// options, returning `true` if anything was removed.
///
/// Called when `data` changes: a filter pinned to a value the new data no
/// longer contains would silently match zero rows, which reads as "the table is
/// broken" rather than "your filter is stale".
pub fn prune_stale_filters(
    filters: &mut ColumnFilters,
    options: &HashMap<&'static str, Vec<String>>,
) -> bool {
    let before = filters.len();
    filters.retain(|col_id, wanted| {
        wanted == FILTER_ALL
            || options
                .get(col_id)
                .map(|opts| opts.iter().any(|o| o == wanted))
                .unwrap_or(false)
    });
    filters.len() != before
}

/// Prunes stale exact selections while retaining free-form substring input.
///
/// A contains value is not drawn from a finite vocabulary, so a replacement
/// data slice cannot make it structurally stale even when it currently
/// matches zero rows.
pub fn prune_stale_column_filters(
    filters: &mut ColumnFilters,
    options: &HashMap<&'static str, Vec<String>>,
    columns: &[Column],
) -> bool {
    let before = filters.len();
    filters.retain(|col_id, wanted| {
        if wanted == FILTER_ALL {
            return true;
        }
        if columns
            .iter()
            .find(|column| column.id == *col_id)
            .and_then(Column::filter_kind)
            == Some(ColumnFilterKind::Contains)
        {
            return true;
        }
        options
            .get(col_id)
            .is_some_and(|available| available.iter().any(|option| option == wanted))
    });
    filters.len() != before
}

/// Whether any column opts into filtering -- i.e. whether to render the filter
/// row at all.
pub fn has_filterable_columns(columns: &[Column]) -> bool {
    columns.iter().any(|c| c.filterable)
}

/// Whether any column needs a finite exact-value vocabulary.
pub fn has_exact_filterable_columns(columns: &[Column]) -> bool {
    columns
        .iter()
        .any(|column| column.filter_kind() == Some(ColumnFilterKind::Exact))
}

/// Whether `row` matches the free-text search `query_lower` (already
/// lowercased; `""` matches everything).
///
/// The search contract is **column-scoped**: only the values of declared
/// columns with [`Column::searched`] left at its `true` default participate.
/// A `TableRow` entry with no declared column -- renderer-only metadata such
/// as stable state codes, route ids or raw epoch instants -- never matches,
/// so a user typing (in any language) can't hit hidden English codes or
/// digits they cannot see.
pub fn row_matches_search(row: &TableRow, columns: &[Column], query_lower: &str) -> bool {
    if query_lower.is_empty() {
        return true;
    }
    columns.iter().filter(|c| c.searched).any(|c| {
        row.get(c.id)
            .is_some_and(|v| v.to_lowercase().contains(query_lower))
    })
}

#[component]
fn DataTableTextFilter(
    col_id: &'static str,
    filters: RwSignal<ColumnFilters>,
    on_filters_change: Option<Callback<ColumnFilters>>,
    #[prop(into)] accessible_label: Signal<String>,
) -> impl IntoView {
    let accepted_value =
        move || filters.with(|active| active.get(col_id).cloned().unwrap_or_default());
    let draft = RwSignal::new(accepted_value());
    let input_ref = NodeRef::<Input>::new();
    let (debounce_handle, set_debounce_handle) = signal(Option::<TimeoutHandle>::None);

    // Controlled server ownership can reject a proposal. Re-project accepted
    // truth into both the reactive draft and the live DOM control so a
    // browser-managed input value never gets stranded ahead of TableQuery.
    Effect::new(move |_| {
        let accepted = accepted_value();
        if draft.get_untracked() != accepted {
            draft.set(accepted.clone());
            if let Some(input) = input_ref.get() {
                input.set_value(&accepted);
            }
        }
    });

    let apply_filter = Callback::new(move |value: String| {
        if filters.try_get_untracked().is_none() {
            return;
        }
        filters.update(|active| {
            if value.is_empty() {
                active.remove(col_id);
            } else {
                active.insert(col_id, value);
            }
        });
        if let Some(callback) = on_filters_change {
            callback.run(filters.get_untracked());
        }

        // The callback may synchronously reassert controlled truth.
        let accepted = filters
            .try_get_untracked()
            .and_then(|active| active.get(col_id).cloned())
            .unwrap_or_default();
        let _ = draft.try_set(accepted.clone());
        if let Some(input) = input_ref.get() {
            input.set_value(&accepted);
        }
    });

    let on_input = move |event: leptos::ev::Event| {
        let Some(target) = event.target() else {
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            return;
        };
        let value = input.value();
        draft.set(value.clone());
        if let Some(handle) = debounce_handle.get_untracked() {
            handle.clear();
        }
        let delayed = value.clone();
        match set_timeout_with_handle(
            move || {
                if draft.try_get_untracked().is_none() {
                    return;
                }
                apply_filter.run(delayed);
                let _ = set_debounce_handle.try_set(None);
            },
            std::time::Duration::from_millis(COLUMN_TEXT_FILTER_DEBOUNCE_MS),
        ) {
            Ok(handle) => set_debounce_handle.set(Some(handle)),
            Err(_) => {
                apply_filter.run(value);
                set_debounce_handle.set(None);
            }
        }
    };

    on_cleanup(move || {
        if let Some(handle) = debounce_handle.try_get_untracked().flatten() {
            handle.clear();
        }
    });

    view! {
        <label class="block w-full">
            <span class="sr-only">{move || accessible_label.get()}</span>
            <input
                node_ref=input_ref
                type="text"
                class="input input-bordered input-xs w-full bg-table-filter font-normal text-table-filter-content forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                aria-label=move || accessible_label.get()
                autocomplete="off"
                data-table-filter-kind="contains"
                prop:value=move || draft.get()
                on:input=on_input
            />
        </label>
    }
}

/// A `<tr>` of per-column exact dropdowns or substring text boxes inside the
/// `<thead>` beneath the sortable header row.
///
/// Every column gets a `<th>` so the cells stay aligned with the header and
/// body; exact columns get a `<select>` and text-filter columns get an input.
#[component]
pub fn DataTableFilterRow(
    /// Column definitions -- the same signal the header and body render from.
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Typed option lists per filterable column id. Local string values are
    /// adapted with [`filter_options_from_strings`].
    #[prop(into)]
    options: Signal<DataTableFilterOptions>,

    /// Active filter selections. Owned by the parent `DataTable` so the
    /// filtering memo and this row share one source of truth.
    filters: RwSignal<ColumnFilters>,

    /// Optional controlled change observer. Receives the complete replacement
    /// after one select transition; the parent may immediately reassert a
    /// declined supplied value through `filters`.
    #[prop(optional)]
    on_filters_change: Option<Callback<ColumnFilters>>,

    /// Label for the "no filter" option in every dropdown.
    #[prop(into)]
    all_label: Signal<String>,

    /// Associated-label template; `{column}` is replaced with the live header.
    #[prop(into, default = Signal::stored("Filter by {column}".to_owned()))]
    filter_label: Signal<String>,

    /// Associated-label template for substring inputs; `{column}` is replaced
    /// with the live header.
    #[prop(into, default = Signal::stored("Filter {column} by text".to_owned()))]
    text_filter_label: Signal<String>,
) -> impl IntoView {
    let options_error = Memo::new(move |_| validate_filter_options(&options.get()).err());

    view! {
        <>
        <Show when=move || options_error.get().is_some()>
            <tr data-table-filter-options-error="true">
                <th
                    colspan=move || columns.with(|columns| columns.len().max(1))
                    role="alert"
                    class="border border-error bg-error/10 px-3 py-2 text-sm text-error forced-colors:border-[CanvasText] forced-colors:text-[CanvasText]"
                >
                    {move || options_error
                        .get()
                        .map(|error| error.to_string())
                        .unwrap_or_default()}
                </th>
            </tr>
        </Show>
        <Show when=move || options_error.get().is_none()>
        <tr
            class="data-table-filter-row bg-table-filter text-table-filter-content forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
            data-table-filter-row="true"
        >
            <For
                each=move || columns.get()
                key=|col| (col.id, col.header.clone(), col.filterable, col.filter_kind)
                children=move |col| {
                    let col_id = col.id;
                    let header_label = col.header.clone();
                    let text_header_label = col.header.clone();
                    let filter_kind = col.filter_kind();
                    let exact_accessible_label = Signal::derive(move || {
                        filter_label.get().replace("{column}", &header_label)
                    });
                    let text_accessible_label = Signal::derive(move || {
                        text_filter_label.get().replace("{column}", &text_header_label)
                    });

                    view! {
                        <th
                            class="border border-table-grid bg-table-filter p-1 text-table-filter-content forced-colors:border-[CanvasText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                            scope="col"
                            data-table-filter-column=col_id
                            on:pointerdown=move |event| event.stop_propagation()
                            on:click=move |event| event.stop_propagation()
                            on:keydown=move |event| event.stop_propagation()
                        >
                            {match filter_kind {
                                Some(ColumnFilterKind::Exact) => {
                                    let col_options = move || {
                                        options.with(|o| o.get(col_id).cloned().unwrap_or_default())
                                    };
                                    Some(view! {
                                        <label class="block w-full">
                                        <span class="sr-only">{move || exact_accessible_label.get()}</span>
                                        <select
                                            class="select select-bordered select-xs w-full bg-table-filter font-normal text-table-filter-content forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
                                            aria-label=move || exact_accessible_label.get()
                                            data-table-filter-kind="exact"
                                            prop:value=move || {
                                                filters.with(|f| {
                                                    f.get(col_id).cloned().unwrap_or_else(|| FILTER_ALL.to_string())
                                                })
                                            }
                                            on:change=move |ev: leptos::ev::Event| {
                                                let Some(target) = ev.target() else { return };
                                                let Ok(select) = target.dyn_into::<web_sys::HtmlSelectElement>() else {
                                                    return;
                                                };
                                                let value = select.value();
                                                filters.update(|f| {
                                                    if value == FILTER_ALL {
                                                        f.remove(col_id);
                                                    } else {
                                                        f.insert(col_id, value);
                                                    }
                                                });
                                                if let Some(callback) = on_filters_change {
                                                    callback.run(filters.get_untracked());
                                                }
                                            }
                                        >
                                            <option value=FILTER_ALL>{move || all_label.get()}</option>
                                            <For
                                                each=col_options
                                                key=|option: &DataTableFilterOption| {
                                                    (option.value.clone(), option.label.clone())
                                                }
                                                let:opt
                                            >
                                                {
                                                    let value = opt.value;
                                                    let label = opt.label;
                                                    view! { <option value=value>{label}</option> }
                                                }
                                            </For>
                                        </select>
                                        </label>
                                    }.into_any())
                                }
                                Some(ColumnFilterKind::Contains) => Some(view! {
                                    <DataTableTextFilter
                                        col_id=col_id
                                        filters=filters
                                        on_filters_change=on_filters_change
                                        accessible_label=text_accessible_label
                                    />
                                }.into_any()),
                                None => None,
                            }}
                        </th>
                    }
                }
            />
        </tr>
        </Show>
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(pairs: &[(&'static str, &str)]) -> TableRow {
        pairs
            .iter()
            .map(|(k, v)| (*k, v.to_string()))
            .collect::<HashMap<_, _>>()
    }

    // ── distinct_values ──

    #[test]
    fn distinct_values_are_sorted_and_deduped() {
        let data = vec![
            row(&[("status", "Open")]),
            row(&[("status", "Closed")]),
            row(&[("status", "Open")]),
            row(&[("status", "Blocked")]),
        ];
        assert_eq!(
            distinct_values(&data, "status"),
            vec!["Blocked", "Closed", "Open"]
        );
    }

    #[test]
    fn distinct_values_skips_empty_cells() {
        // An empty option would be indistinguishable from FILTER_ALL.
        let data = vec![row(&[("status", "Open")]), row(&[("status", "")])];
        assert_eq!(distinct_values(&data, "status"), vec!["Open"]);
    }

    #[test]
    fn distinct_values_skips_rows_missing_the_column() {
        let data = vec![row(&[("status", "Open")]), row(&[("other", "x")])];
        assert_eq!(distinct_values(&data, "status"), vec!["Open"]);
    }

    #[test]
    fn distinct_values_of_empty_data_is_empty() {
        assert!(distinct_values(&[], "status").is_empty());
    }

    #[test]
    fn distinct_values_of_unknown_column_is_empty() {
        let data = vec![row(&[("status", "Open")])];
        assert!(distinct_values(&data, "nope").is_empty());
    }

    #[test]
    fn typed_filter_options_separate_stable_values_from_display_labels() {
        let option = DataTableFilterOption::new("desk_provider", "Desk provider");
        assert_eq!(option.value, "desk_provider");
        assert_eq!(option.label, "Desk provider");

        let converted = filter_options_from_strings(HashMap::from([(
            "role",
            vec!["Admin".to_owned(), "Analyst".to_owned()],
        )]));
        assert_eq!(
            converted["role"],
            vec![
                DataTableFilterOption::same("Admin"),
                DataTableFilterOption::same("Analyst"),
            ]
        );
    }

    #[test]
    fn invalid_typed_filter_values_are_rejected_without_aliasing_options() {
        let empty = DataTableFilterOptions::from([(
            "role",
            vec![DataTableFilterOption::new("", "Everything")],
        )]);
        assert_eq!(
            validate_filter_options(&empty),
            Err(DataTableFilterOptionError::EmptyValue {
                column: "role",
                index: 0,
            })
        );

        let duplicate = DataTableFilterOptions::from([(
            "role",
            vec![
                DataTableFilterOption::new("admin", "Administrator"),
                DataTableFilterOption::new("admin", "Admin"),
            ],
        )]);
        assert_eq!(
            validate_filter_options(&duplicate),
            Err(DataTableFilterOptionError::DuplicateValue {
                column: "role",
                value: "admin".to_owned(),
                first_index: 0,
                duplicate_index: 1,
            })
        );
    }

    // ── row_matches_filters ──

    #[test]
    fn no_filters_matches_everything() {
        let r = row(&[("status", "Open")]);
        assert!(row_matches_filters(&r, &ColumnFilters::new()));
    }

    #[test]
    fn all_sentinel_is_not_a_filter() {
        let r = row(&[("status", "Open")]);
        let filters = ColumnFilters::from([("status", FILTER_ALL.to_string())]);
        assert!(row_matches_filters(&r, &filters));
    }

    #[test]
    fn matching_filter_keeps_the_row() {
        let r = row(&[("status", "Open")]);
        let filters = ColumnFilters::from([("status", "Open".to_string())]);
        assert!(row_matches_filters(&r, &filters));
    }

    #[test]
    fn non_matching_filter_excludes_the_row() {
        let r = row(&[("status", "Open")]);
        let filters = ColumnFilters::from([("status", "Closed".to_string())]);
        assert!(!row_matches_filters(&r, &filters));
    }

    #[test]
    fn multiple_filters_combine_with_and() {
        let r = row(&[("status", "Open"), ("owner", "alice")]);
        let both_match = ColumnFilters::from([
            ("status", "Open".to_string()),
            ("owner", "alice".to_string()),
        ]);
        assert!(row_matches_filters(&r, &both_match));

        let one_fails =
            ColumnFilters::from([("status", "Open".to_string()), ("owner", "bob".to_string())]);
        assert!(
            !row_matches_filters(&r, &one_fails),
            "AND semantics: one failing filter must exclude the row"
        );
    }

    #[test]
    fn filter_on_absent_column_excludes_the_row() {
        let r = row(&[("status", "Open")]);
        let filters = ColumnFilters::from([("owner", "alice".to_string())]);
        assert!(!row_matches_filters(&r, &filters));
    }

    #[test]
    fn matching_is_case_sensitive_and_exact() {
        let r = row(&[("status", "Open")]);
        assert!(!row_matches_filters(
            &r,
            &ColumnFilters::from([("status", "open".to_string())])
        ));
        assert!(!row_matches_filters(
            &r,
            &ColumnFilters::from([("status", "Ope".to_string())])
        ));
    }

    #[test]
    fn column_filter_kind_distinguishes_exact_and_contains_without_changing_the_value_map() {
        let exact = Column::new("status", "Status").filterable();
        let contains = Column::new("job", "Job").filterable_text();

        assert_eq!(exact.filter_kind(), Some(ColumnFilterKind::Exact));
        assert_eq!(contains.filter_kind(), Some(ColumnFilterKind::Contains));
        assert_eq!(Column::new("name", "Name").filter_kind(), None);
    }

    #[test]
    fn contains_filters_match_case_insensitive_substrings_and_combine_with_exact_filters() {
        let columns = vec![
            Column::new("job", "Job").filterable_text(),
            Column::new("status", "Status").filterable(),
        ];
        let filters =
            ColumnFilters::from([("job", "MAT".to_owned()), ("status", "Ready".to_owned())]);

        assert!(row_matches_column_filters(
            &row(&[("job", "zoho-matters"), ("status", "Ready")]),
            &columns,
            &filters,
        ));
        assert!(row_matches_column_filters(
            &row(&[("job", "Matter_Timeline"), ("status", "Ready")]),
            &columns,
            &filters,
        ));
        assert!(!row_matches_column_filters(
            &row(&[("job", "contacts"), ("status", "Ready")]),
            &columns,
            &filters,
        ));
        assert!(!row_matches_column_filters(
            &row(&[("job", "zoho-matters"), ("status", "ready")]),
            &columns,
            &filters,
        ));
    }

    #[test]
    fn stale_pruning_preserves_contains_text_but_drops_missing_exact_options() {
        let columns = vec![
            Column::new("job", "Job").filterable_text(),
            Column::new("status", "Status").filterable(),
        ];
        let mut filters =
            ColumnFilters::from([("job", "mat".to_owned()), ("status", "Closed".to_owned())]);
        let options = HashMap::from([("status", vec!["Ready".to_owned()])]);

        assert!(prune_stale_column_filters(&mut filters, &options, &columns,));
        assert_eq!(filters, ColumnFilters::from([("job", "mat".to_owned())]));
    }

    // ── prune_stale_filters ──

    #[test]
    fn prune_keeps_a_still_valid_filter() {
        let mut filters = ColumnFilters::from([("status", "Open".to_string())]);
        let options = HashMap::from([("status", vec!["Open".to_string(), "Closed".to_string()])]);
        assert!(!prune_stale_filters(&mut filters, &options));
        assert_eq!(filters.get("status").map(String::as_str), Some("Open"));
    }

    #[test]
    fn prune_drops_a_filter_whose_value_vanished() {
        let mut filters = ColumnFilters::from([("status", "Open".to_string())]);
        let options = HashMap::from([("status", vec!["Closed".to_string()])]);
        assert!(prune_stale_filters(&mut filters, &options));
        assert!(
            filters.is_empty(),
            "a filter matching zero rows must be dropped, not silently kept"
        );
    }

    #[test]
    fn prune_drops_a_filter_whose_column_vanished() {
        let mut filters = ColumnFilters::from([("status", "Open".to_string())]);
        assert!(prune_stale_filters(&mut filters, &HashMap::new()));
        assert!(filters.is_empty());
    }

    #[test]
    fn prune_keeps_the_all_sentinel() {
        let mut filters = ColumnFilters::from([("status", FILTER_ALL.to_string())]);
        assert!(!prune_stale_filters(&mut filters, &HashMap::new()));
        assert_eq!(filters.len(), 1);
    }

    #[test]
    fn prune_of_no_filters_reports_no_change() {
        let mut filters = ColumnFilters::new();
        let options = HashMap::from([("status", vec!["Open".to_string()])]);
        assert!(!prune_stale_filters(&mut filters, &options));
    }

    // ── row_matches_search ──

    #[test]
    fn search_matches_a_declared_column_value() {
        let r = row(&[("name", "Maria Gonzalez")]);
        let cols = vec![Column::new("name", "Name")];
        assert!(row_matches_search(&r, &cols, "gonza"));
    }

    #[test]
    fn search_never_matches_undeclared_metadata() {
        // The Office queue's rows carry renderer-only metadata: an English
        // state code and a raw epoch. Neither has a declared column, so a
        // user's search (in any language) must not hit them.
        let r = row(&[
            ("estado", "Abierto"),
            ("state_code", "OPEN_UNASSIGNED"),
            ("deadline_epoch", "1791283600"),
        ]);
        let cols = vec![Column::new("estado", "Estado")];
        assert!(
            !row_matches_search(&r, &cols, "open"),
            "hidden English state codes must not match"
        );
        assert!(
            !row_matches_search(&r, &cols, "1791"),
            "raw epoch digits must not match"
        );
    }

    #[test]
    fn search_matches_the_localized_visible_value() {
        let r = row(&[("estado", "Abierto"), ("state_code", "OPEN_UNASSIGNED")]);
        let cols = vec![Column::new("estado", "Estado")];
        assert!(
            row_matches_search(&r, &cols, "abierto"),
            "the visible localized value is what a user searches for"
        );
    }

    #[test]
    fn search_skips_a_column_that_opted_out() {
        // Visible but renderer-formatted: the raw digits should not match.
        let r = row(&[("deadline_epoch", "1791283600"), ("name", "Maria")]);
        let cols = vec![
            Column::new("deadline_epoch", "Deadline").searched(false),
            Column::new("name", "Name"),
        ];
        assert!(!row_matches_search(&r, &cols, "1791"));
        assert!(row_matches_search(&r, &cols, "maria"));
    }

    #[test]
    fn search_empty_query_matches_everything() {
        let r = row(&[("name", "Maria")]);
        assert!(row_matches_search(&r, &[], ""));
        assert!(row_matches_search(&r, &[Column::new("name", "Name")], ""));
    }

    #[test]
    fn search_is_case_insensitive_on_the_cell_side() {
        // Callers pass the query pre-lowercased; cells are lowercased here.
        let r = row(&[("name", "MARIA")]);
        let cols = vec![Column::new("name", "Name")];
        assert!(row_matches_search(&r, &cols, "maria"));
    }

    // ── has_filterable_columns ──

    #[test]
    fn has_filterable_is_false_by_default() {
        let columns = vec![Column::new("a", "A"), Column::new("b", "B")];
        assert!(
            !has_filterable_columns(&columns),
            "filtering must stay opt-in so existing callers are unaffected"
        );
    }

    #[test]
    fn has_filterable_is_true_when_any_column_opts_in() {
        let columns = vec![Column::new("a", "A"), Column::new("b", "B").filterable()];
        assert!(has_filterable_columns(&columns));
    }

    #[test]
    fn has_filterable_of_no_columns_is_false() {
        assert!(!has_filterable_columns(&[]));
    }
}
