//! Per-column filter row for [`DataTable`](super::DataTable): the pure
//! filtering logic plus the `<tr>` of `<select>` dropdowns that drives it.
//!
//! Opt-in per column via [`Column::filterable`](super::Column::filterable).
//! Callers that set no `filterable` column get no filter row and are entirely
//! unaffected.
//!
//! The pure functions here (`distinct_values`, `row_matches_filters`,
//! `prune_stale_filters`) hold all the behaviour worth testing and are
//! unit-tested without a DOM, matching `resize.rs`/`auto_page.rs`.

use crate::components::data_table::types::{Column, TableRow};
use leptos::prelude::*;
use std::collections::HashMap;
use web_sys::wasm_bindgen::JsCast;

/// The `<select>` value meaning "no filter on this column". The empty string
/// can never collide with a real option because [`distinct_values`] skips
/// empty cells.
pub const FILTER_ALL: &str = "";

/// Active per-column filter selections, keyed by [`Column::id`]. A missing key
/// or a [`FILTER_ALL`] value means the column is not filtering.
pub type ColumnFilters = HashMap<&'static str, String>;

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
/// case-insensitive substring match across all columns.
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

/// Whether any column opts into filtering -- i.e. whether to render the filter
/// row at all.
pub fn has_filterable_columns(columns: &[Column]) -> bool {
    columns.iter().any(|c| c.filterable)
}

/// A `<tr>` of per-column filter dropdowns, rendered inside the table's
/// `<thead>` beneath the sortable header row.
///
/// Every column gets a `<th>` so the cells stay aligned with the header and
/// body; only columns with [`Column::filterable`] get a `<select>` inside it.
#[component]
pub fn DataTableFilterRow(
    /// Column definitions -- the same signal the header and body render from.
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Option lists per filterable column id, from [`distinct_values`].
    #[prop(into)]
    options: Signal<HashMap<&'static str, Vec<String>>>,

    /// Active filter selections. Owned by the parent `DataTable` so the
    /// filtering memo and this row share one source of truth.
    filters: RwSignal<ColumnFilters>,

    /// Label for the "no filter" option in every dropdown.
    #[prop(into)]
    all_label: Signal<String>,
) -> impl IntoView {
    view! {
        <tr class="data-table-filter-row">
            {move || {
                columns.get().iter().map(|col| {
                    let col_id = col.id;
                    let header_label = col.header.clone();
                    let is_filterable = col.filterable;

                    view! {
                        <th class="p-1">
                            {is_filterable.then(|| {
                                let col_options = move || {
                                    options.with(|o| o.get(col_id).cloned().unwrap_or_default())
                                };
                                view! {
                                    <select
                                        class="select select-bordered select-xs w-full font-normal"
                                        aria-label=format!("Filter by {}", header_label)
                                        // Bind the rendered selection to the
                                        // signal, so an external reset (or a
                                        // stale-filter prune) is reflected in
                                        // the DOM rather than leaving the
                                        // `<select>` showing a dead value.
                                        prop:value=move || {
                                            filters.with(|f| {
                                                f.get(col_id).cloned().unwrap_or_else(|| FILTER_ALL.to_string())
                                            })
                                        }
                                        on:change=move |ev: leptos::ev::Event| {
                                            // The target is always an
                                            // `HtmlSelectElement` in practice;
                                            // if a browser ever hands us
                                            // something else, ignore the event
                                            // rather than panicking the app.
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
                                        }
                                    >
                                        <option value=FILTER_ALL>{move || all_label.get()}</option>
                                        <For
                                            each=col_options
                                            key=|opt: &String| opt.clone()
                                            let:opt
                                        >
                                            {
                                                let label = opt.clone();
                                                view! { <option value=opt>{label}</option> }
                                            }
                                        </For>
                                    </select>
                                }
                            })}
                        </th>
                    }
                }).collect_view()
            }}
        </tr>
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
