use crate::components::{Button, ButtonSize};
use leptos::either::Either;
use leptos::prelude::*;

/// Column definition for the data table.
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    /// Unique key identifying this column (used internally for sort state).
    pub key: String,
    /// Display label shown in the header.
    pub label: String,
    /// Whether this column supports sorting by click.
    pub sortable: bool,
    /// Optional CSS width (e.g., "200px", "20%").
    pub width: Option<String>,
}

/// Sort direction indicator.
#[derive(Clone, Debug, PartialEq, Copy)]
enum SortDirection {
    Ascending,
    Descending,
}

/// Maps cell text values to DaisyUI badge class strings.
fn badge_class(value: &str) -> String {
    match value {
        // Risk tiers
        "Critical" => "badge badge-error badge-sm".into(),
        "High" => "badge badge-warning badge-sm".into(),
        "Medium" => "badge badge-info badge-sm".into(),
        "Low" => "badge badge-success badge-sm".into(),
        // Lifecycle statuses
        "Active" => "badge badge-success badge-outline badge-sm".into(),
        "Under Review" | "Review" => "badge badge-info badge-outline badge-sm".into(),
        "Deprecated" => "badge badge-warning badge-outline badge-sm".into(),
        "Retired" => "badge badge-ghost badge-sm".into(),
        // Proxy rule types
        "Block" => "badge badge-error badge-sm".into(),
        "Allow" => "badge badge-success badge-sm".into(),
        "Rate Limit" => "badge badge-warning badge-sm".into(),
        "Monitor" => "badge badge-info badge-sm".into(),
        "Custom" => "badge badge-ghost badge-sm".into(),
        // Network log actions (text-colored variants)
        "Blocked" => "badge badge-error badge-sm".into(),
        "Allowed" => "badge badge-success badge-sm".into(),
        "Monitored" => "badge badge-info badge-sm".into(),
        "Rate Ltd" => "badge badge-warning badge-sm".into(),
        // Network log statuses
        "Flagged" => "badge badge-error badge-sm".into(),
        "Normal" => "badge badge-success badge-outline badge-sm".into(),
        "Alert" => "badge badge-warning badge-sm".into(),
        // Prompt history flags
        "PII" => "badge badge-error badge-sm".into(),
        "Clean" => "badge badge-success badge-outline badge-sm".into(),
        "Cost" => "badge badge-warning badge-sm".into(),
        "Sensitive" => "badge badge-warning badge-outline badge-sm".into(),
        // Expired status
        "Expired" => "badge badge-error badge-outline badge-sm".into(),
        // Incident statuses
        "In Progress" => "badge badge-info badge-sm".into(),
        "Triaged" => "badge badge-secondary badge-sm".into(),
        "New" => "badge badge-ghost badge-sm".into(),
        "Resolved" => "badge badge-success badge-sm".into(),
        "Closed" => "badge badge-ghost badge-sm".into(),
        // Alert-specific statuses
        "Investigating" => "badge badge-info badge-outline badge-sm".into(),
        "Acknowledged" => "badge badge-secondary badge-outline badge-sm".into(),
        // Recertification statuses
        "In Review" => "badge badge-info badge-sm".into(),
        "SoD Conflict" => "badge badge-error badge-sm".into(),
        "Completed" => "badge badge-success badge-outline badge-sm".into(),
        // Request lifecycle statuses
        "Approved" => "badge badge-success badge-sm".into(),
        "Denied" => "badge badge-error badge-sm".into(),
        "Provisioned" => "badge badge-primary badge-sm".into(),
        "Conditional" => "badge badge-warning badge-sm".into(),
        "Pending Review" => "badge badge-info badge-outline badge-sm".into(),
        "Queued" => "badge badge-ghost badge-sm".into(),
        "Provisioning" => "badge badge-warning badge-outline badge-sm".into(),
        // Access level badges
        "Editor" => "badge badge-info badge-outline badge-sm".into(),
        "Admin" => "badge badge-error badge-outline badge-sm".into(),
        "Auditor" => "badge badge-secondary badge-outline badge-sm".into(),
        "Approver" => "badge badge-success badge-outline badge-sm".into(),
        "Reader" => "badge badge-ghost badge-sm".into(),
        // Enrollment statuses
        "Pending" => "badge badge-warning badge-outline badge-sm".into(),
        "Suspended" => "badge badge-error badge-outline badge-sm".into(),
        "Draft" => "badge badge-ghost badge-sm".into(),
        // Report distribution + scheduling statuses (10 Management Reporting):
        // Scheduled=blue, Active=green, Draft=gray, Generated=blue,
        // Failed=red, Distributed=green
        "Scheduled" => "badge badge-info badge-sm".into(),
        "Generated" => "badge badge-info badge-sm".into(),
        "Distributed" => "badge badge-success badge-sm".into(),
        "Failed" => "badge badge-error badge-sm".into(),
        // Training categories
        "Compliance" => "badge badge-success badge-sm".into(),
        "AI Governance" => "badge badge-info badge-sm".into(),
        "Security" => "badge badge-error badge-sm".into(),
        "Elective" => "badge badge-ghost badge-sm".into(),
        // Attestation types
        "Policy" => "badge badge-info badge-sm".into(),
        "Regulatory" => "badge badge-warning badge-sm".into(),
        // Policy categories
        "Usage" => "badge badge-primary badge-outline badge-sm".into(),
        "Data Governance" => "badge badge-warning badge-outline badge-sm".into(),
        _ => String::new(),
    }
}

/// Reusable sortable, paginated data table component.
///
/// Renders tabular data with column-header sort toggles, pagination controls,
/// and a row count display. Uses DaisyUI table classes with responsive overflow.
#[component]
pub fn DataTable(
    /// Column definitions.
    columns: Vec<TableColumn>,
    /// Row data. Each row is a `Vec<String>` aligned to the columns by index.
    rows: Vec<Vec<String>>,
    /// Whether column sorting is enabled.
    #[prop(default = true)]
    sortable: bool,
    /// Whether to show pagination controls.
    #[prop(default = true)]
    paginated: bool,
    /// Number of rows per page.
    #[prop(default = 25)]
    page_size: usize,
    /// Column indices whose cell values should render as color-coded badges.
    /// Brittle to column reordering — prefer `badge_column_keys` for new code.
    #[prop(default = vec![])]
    badge_columns: Vec<usize>,
    /// Column keys (matching `TableColumn.key`) whose cell values should render as
    /// color-coded badges. Resolved to indices at construction time. If a key is
    /// not found, the component panics in debug builds and silently skips in
    /// release builds. Preferred over `badge_columns` for refactor-safety.
    #[prop(default = vec![])]
    badge_column_keys: Vec<&'static str>,
    /// Column indices whose cell values should render as blue links (e.g. ID columns).
    /// Brittle to column reordering — prefer `link_column_keys` for new code.
    #[prop(default = vec![])]
    link_columns: Vec<usize>,
    /// Column keys (matching `TableColumn.key`) whose cell values should render as
    /// blue links. Resolved to indices at construction time, with the same
    /// debug-time validation as `badge_column_keys`.
    #[prop(default = vec![])]
    link_column_keys: Vec<&'static str>,
    /// Optional callback fired when a link cell is clicked. Receives
    /// `(column_key, cell_value)`. If unset, link cells remain visually
    /// styled but inert.
    #[prop(optional)]
    on_link_click: Option<Callback<(String, String)>>,
    /// Optional reactive key (matched against the row's first column value)
    /// of the currently selected row. When set, that row receives a visible
    /// selection highlight (`bg-primary text-primary-content`).
    #[prop(optional, into)]
    selected_row_key: Option<Signal<Option<String>>>,
    /// Optional bulk-select state. When Some, the table renders a leading
    /// checkbox column + a header "select all on this page" checkbox; row
    /// IDs (the first cell value) are toggled into/out of the set as
    /// operators tick checkboxes. The page that owns the signal can read
    /// it to drive the §3.2 bulk-action toolbar's per-verb actions.
    #[prop(optional)]
    bulk_select: Option<RwSignal<std::collections::HashSet<String>>>,
) -> impl IntoView {
    // Resolve key-based column refs to indices, with debug-time validation that
    // the keys actually exist in the column list. Catches drift when columns get
    // reordered or renamed (EUC-3z4v).
    let resolve_keys = |keys: &[&'static str], kind: &str| -> Vec<usize> {
        keys.iter()
            .filter_map(|k| {
                let idx = columns.iter().position(|c| c.key == *k);
                debug_assert!(
                    idx.is_some(),
                    "DataTable: {} column key {:?} not found in columns; valid keys: {:?}",
                    kind,
                    k,
                    columns.iter().map(|c| c.key.as_str()).collect::<Vec<_>>(),
                );
                idx
            })
            .collect()
    };
    // Validate index-based refs are in-bounds — catches the bug class even when
    // callers haven't migrated to keys yet.
    let validate_indices = |indices: &[usize], kind: &str| {
        for &idx in indices {
            debug_assert!(
                idx < columns.len(),
                "DataTable: {} column index {} out of bounds (only {} columns)",
                kind,
                idx,
                columns.len(),
            );
        }
    };
    validate_indices(&badge_columns, "badge_columns");
    validate_indices(&link_columns, "link_columns");
    let mut all_badge = badge_columns;
    all_badge.extend(resolve_keys(&badge_column_keys, "badge_column_keys"));
    let mut all_link = link_columns;
    all_link.extend(resolve_keys(&link_column_keys, "link_column_keys"));

    let columns = StoredValue::new(columns);
    let rows = StoredValue::new(rows);
    let badge_cols = StoredValue::new(all_badge);
    let link_cols = StoredValue::new(all_link);

    // Sort state: (column_index, direction)
    let sort_state = RwSignal::new(Option::<(usize, SortDirection)>::None);
    // Current page (0-indexed)
    let current_page = RwSignal::new(0_usize);

    // Derived: sorted rows
    let sorted_rows = Memo::new(move |_| {
        let mut data = rows.get_value();
        if let Some((col_idx, direction)) = sort_state.get() {
            data.sort_by(|a, b| {
                let va = a.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                let vb = b.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                // Try numeric comparison first, fall back to string
                let cmp = match (va.parse::<f64>(), vb.parse::<f64>()) {
                    (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
                    _ => va.cmp(vb),
                };
                match direction {
                    SortDirection::Ascending => cmp,
                    SortDirection::Descending => cmp.reverse(),
                }
            });
        }
        data
    });

    // Derived: total pages
    let total_pages = Memo::new(move |_| {
        let total = sorted_rows.get().len();
        if !paginated || page_size == 0 {
            return 1;
        }
        total.div_ceil(page_size)
    });

    // Derived: visible (paginated) rows
    let visible_rows = Memo::new(move |_| {
        let all = sorted_rows.get();
        if !paginated {
            return all;
        }
        let page = current_page.get();
        let start = page * page_size;
        all.into_iter()
            .skip(start)
            .take(page_size)
            .collect::<Vec<_>>()
    });

    // Total row count
    let total_rows = Memo::new(move |_| rows.get_value().len());

    view! {
        <div class="flex flex-col gap-2">
            // Table with responsive overflow
            <div class="overflow-x-auto rounded-lg border border-base-300">
                <table class="table table-zebra table-pin-rows table-sm w-full">
                    <thead>
                        <tr class="bg-base-200">
                            {move || bulk_select.map(|sel| {
                                view! {
                                    <th style:width="32px" class="px-2">
                                        <input
                                            type="checkbox"
                                            class="checkbox checkbox-xs"
                                            prop:checked=move || {
                                                let n = sel.with(|s| s.len());
                                                let total = visible_rows.with(|rs| rs.len());
                                                n > 0 && n == total
                                            }
                                            on:change=move |ev| {
                                                let checked = leptos::prelude::event_target_checked(&ev);
                                                if checked {
                                                    let all: std::collections::HashSet<String> =
                                                        visible_rows.with(|rs| rs.iter()
                                                            .filter_map(|r| r.first().cloned())
                                                            .collect());
                                                    sel.set(all);
                                                } else {
                                                    sel.set(std::collections::HashSet::new());
                                                }
                                            }
                                        />
                                    </th>
                                }
                            })}
                            {move || columns.get_value().into_iter().enumerate().map(|(idx, col)| {
                                let is_sortable = sortable && col.sortable;
                                let width_style = col.width.clone().unwrap_or_default();
                                let label = col.label.clone();
                                let sort = sort_state;
                                let page = current_page;
                                view! {
                                    <th
                                        style:width=width_style
                                        class=move || if is_sortable {
                                            "cursor-pointer select-none hover:bg-base-300 transition-colors"
                                        } else {
                                            ""
                                        }
                                        on:click=move |_| {
                                            if is_sortable {
                                                sort.update(|s| {
                                                    *s = Some(match *s {
                                                        Some((col, SortDirection::Ascending)) if col == idx => {
                                                            (idx, SortDirection::Descending)
                                                        }
                                                        _ => (idx, SortDirection::Ascending),
                                                    });
                                                });
                                                page.set(0);
                                            }
                                        }
                                    >
                                        <div class="flex items-center gap-1">
                                            <span>{label}</span>
                                            {move || {
                                                if !is_sortable {
                                                    return None;
                                                }
                                                let s = sort.get();
                                                match s {
                                                    Some((col, SortDirection::Ascending)) if col == idx => {
                                                        Some(view! { <span class="text-xs">{"\u{25B2}"}</span> })
                                                    }
                                                    Some((col, SortDirection::Descending)) if col == idx => {
                                                        Some(view! { <span class="text-xs">{"\u{25BC}"}</span> })
                                                    }
                                                    _ => Some(view! { <span class="text-xs text-base-content/30">{"\u{25B4}\u{25BE}"}</span> })
                                                }
                                            }}
                                        </div>
                                    </th>
                                }
                            }).collect::<Vec<_>>()}
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = visible_rows.get();
                            if rows.is_empty() {
                                let col_count = columns.get_value().len()
                                    + bulk_select.map(|_| 1).unwrap_or(0);
                                Either::Left(view! {
                                    <tr>
                                        <td colspan=col_count.to_string() class="text-center text-base-content/40 py-8">
                                            "No data available."
                                        </td>
                                    </tr>
                                })
                            } else {
                                Either::Right(rows.into_iter().map(|row| {
                                    let bcols = badge_cols.get_value();
                                    let lcols = link_cols.get_value();
                                    let cols = columns.get_value();
                                    let row_key = row.first().cloned().unwrap_or_default();
                                    let row_class = move || {
                                        let is_selected = selected_row_key
                                            .map(|s| s.get().map(|k| k == row_key).unwrap_or(false))
                                            .unwrap_or(false);
                                        if is_selected {
                                            "hover bg-primary text-primary-content"
                                        } else {
                                            "hover"
                                        }
                                    };
                                    let row_id_for_check = row.first().cloned().unwrap_or_default();
                                    view! {
                                        <tr class=row_class>
                                            {bulk_select.map(|sel| {
                                                let row_id = row_id_for_check.clone();
                                                let row_id_change = row_id.clone();
                                                view! {
                                                    <td class="px-2">
                                                        <input
                                                            type="checkbox"
                                                            class="checkbox checkbox-xs"
                                                            prop:checked=move || sel.with(|s| s.contains(&row_id))
                                                            on:change=move |ev| {
                                                                let checked = leptos::prelude::event_target_checked(&ev);
                                                                let id = row_id_change.clone();
                                                                sel.update(|s| {
                                                                    if checked { s.insert(id); } else { s.remove(&id); }
                                                                });
                                                            }
                                                        />
                                                    </td>
                                                }
                                            })}
                                            {row.into_iter().enumerate().map(|(idx, cell)| {
                                                let is_link = lcols.contains(&idx);
                                                let cls = if bcols.contains(&idx) {
                                                    badge_class(&cell)
                                                } else if is_link {
                                                    "text-[#3B82F5] font-medium cursor-pointer hover:underline".into()
                                                } else {
                                                    String::new()
                                                };
                                                let col_key = cols.get(idx).map(|c| c.key.clone()).unwrap_or_default();
                                                let cell_value = cell.clone();
                                                let on_click = move |_| {
                                                    if is_link {
                                                        if let Some(cb) = on_link_click {
                                                            cb.run((col_key.clone(), cell_value.clone()));
                                                        }
                                                    }
                                                };
                                                view! {
                                                    <td class="text-sm">
                                                        <span class=cls on:click=on_click>{cell}</span>
                                                    </td>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tr>
                                    }
                                }).collect::<Vec<_>>())
                            }
                        }}
                    </tbody>
                </table>
            </div>

            // Footer: row count + pagination
            <div class="flex items-center justify-between text-sm text-base-content/60 px-1">
                <span>
                    {move || {
                        let total = total_rows.get();
                        let page = current_page.get();
                        if paginated && total > 0 {
                            let start = page * page_size + 1;
                            let end = ((page + 1) * page_size).min(total);
                            format!("Showing {start}-{end} of {total} rows")
                        } else {
                            format!("{total} rows")
                        }
                    }}
                </span>

                <Show when=move || paginated && (total_pages.get() > 1)>
                    <div class="join">
                        <Button
                            size=ButtonSize::Xs
                            class="join-item"
                            disabled=Signal::derive(move || current_page.get() == 0)
                            on:click=move |_| {
                                current_page.update(|p| {
                                    if *p > 0 { *p -= 1; }
                                });
                            }
                        >
                            "\u{00AB} Prev"
                        </Button>
                        <Button size=ButtonSize::Xs disabled=Signal::derive(|| true) class="join-item">
                            {move || format!("Page {} of {}", current_page.get() + 1, total_pages.get())}
                        </Button>
                        <Button
                            size=ButtonSize::Xs
                            class="join-item"
                            disabled=Signal::derive(move || current_page.get() + 1 >= total_pages.get())
                            on:click=move |_| {
                                current_page.update(|p| *p += 1);
                            }
                        >
                            "Next \u{00BB}"
                        </Button>
                    </div>
                </Show>
            </div>
        </div>
    }
}
