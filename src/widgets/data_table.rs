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

/// One rendered cell position in a header row or body row, in visual order.
///
/// The component keeps two distinct index spaces and this enum is the map
/// between them:
///
/// - the **data** index space, where `columns[i]` is described by `rows[n][i]`.
///   `badge_columns`, `link_columns`, the resolved `badge_column_keys` /
///   `link_column_keys`, and the sort state all address this space;
/// - the **visual** index space, the actual `<th>` / `<td>` sequence, which
///   additionally contains the optional leading bulk-select checkbox and the
///   optional trailing action cell.
///
/// Both optional cells are rendered *outside* the loops that enumerate the
/// columns and the row's cells, so neither ever renumbers a data index. That
/// invariant is what keeps `badge_columns` and friends pointing at the same
/// column regardless of which optional cells are switched on, and it is
/// asserted by the unit tests below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VisualCell {
    /// The leading bulk-select checkbox cell (present when `bulk_select` is set).
    BulkSelect,
    /// A data column, carrying its index in the data index space.
    Data(usize),
    /// The trailing per-row action cell (present when `row_actions` is set).
    Action,
}

/// Builds the visual cell sequence for one header/body row.
///
/// This mirrors the render order exactly and is the single source of truth for
/// the empty-state `colspan`.
fn visual_layout(data_columns: usize, bulk_select: bool, action_column: bool) -> Vec<VisualCell> {
    let mut cells = Vec::with_capacity(data_columns + 2);
    if bulk_select {
        cells.push(VisualCell::BulkSelect);
    }
    cells.extend((0..data_columns).map(VisualCell::Data));
    if action_column {
        cells.push(VisualCell::Action);
    }
    cells
}

/// Sorts `rows` (each paired with its original index in the `rows` prop) by the
/// data column at `col_idx`, numerically when both cells parse as numbers and
/// lexicographically otherwise.
///
/// Extracted from the memo so the "sort the column that was clicked" contract is
/// unit-testable. The sort is stable, so rows comparing equal keep their
/// original relative order.
fn sort_indexed_rows(rows: &mut [(usize, Vec<String>)], col_idx: usize, direction: SortDirection) {
    rows.sort_by(|a, b| {
        let va = a.1.get(col_idx).map(|s| s.as_str()).unwrap_or("");
        let vb = b.1.get(col_idx).map(|s| s.as_str()).unwrap_or("");
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
    /// Optional per-row action renderer. When set, the table grows a trailing
    /// action column: for every rendered row the callback receives
    /// `(row_index, row_cells)` and returns the controls for that row, laid out
    /// right-aligned in a single nowrap line.
    ///
    /// `row_index` is the row's index in the `rows` prop — stable across sorting
    /// and paging, so it still points at the same entry of `rows` after the
    /// operator re-sorts. `row_cells` is that row's cells, aligned to `columns`
    /// by index, so the callback can read any field it needs.
    ///
    /// **For anything that identifies a record — navigating to it, deleting it,
    /// completing it — key off the row id, not the index.** The first cell is
    /// the row id by the same convention `selected_row_key` and `bulk_select`
    /// already use, and it is an identity rather than a position: if the caller
    /// built `rows` by filtering or reordering some backing store, `row_index`
    /// indexes the snapshot handed to this component and `store[row_index]` is
    /// a different record. Reach for `row_index` when the caller's own state is
    /// genuinely parallel to `rows` (a `Vec<bool>` of expanded flags, say).
    ///
    /// The action column never participates in sorting: its header is inert and
    /// it is rendered outside the column/cell loops, so `badge_columns`,
    /// `link_columns` and the sort state keep addressing the same data columns
    /// whether or not it is present. When this prop is unset no extra `<th>` or
    /// `<td>` is emitted at all.
    ///
    /// The renderer may return any number of controls, enabled or disabled, and
    /// they may differ per row — wrap a disabled control in a daisyUI
    /// `tooltip` to explain why it is unavailable.
    ///
    /// # Read signals inside the returned view, not in the callback body
    ///
    /// This callback runs inside the reactive effect that renders the whole
    /// `<tbody>` — it has to, because that is the effect sort and pagination
    /// re-run. So a signal read in the **callback body** is tracked by that
    /// effect, and every change to it destroys and rebuilds every row's
    /// controls. That is not an error, it is a silent degradation: focus jumps
    /// from the clicked button to `<body>` so the next Tab restarts at the top
    /// of the document, an open tooltip vanishes mid-hover, and the horizontal
    /// scroll offset of the table's overflow wrapper resets.
    ///
    /// Read the signal **inside the view you return** instead, where it is
    /// tracked by the individual attribute and updates that one node:
    ///
    /// ```ignore
    /// // WRONG — tracked by the tbody effect; rebuilds every row on change.
    /// row_actions=move |(_idx, row): (usize, Vec<String>)| {
    ///     let id = row[0].clone();
    ///     let busy = pending.with(|p| p.contains(&id));
    ///     view! { <button class="btn btn-xs" disabled=busy>"Complete"</button> }.into_any()
    /// }
    ///
    /// // RIGHT — tracked by the `disabled` attribute; updates one button.
    /// row_actions=move |(_idx, row): (usize, Vec<String>)| {
    ///     let id = row[0].clone();
    ///     view! {
    ///         <button
    ///             class="btn btn-xs"
    ///             disabled=move || pending.with(|p| p.contains(&id))
    ///         >"Complete"</button>
    ///     }.into_any()
    /// }
    /// ```
    #[prop(optional, into)]
    row_actions: Option<Callback<(usize, Vec<String>), AnyView>>,
    /// Visible header label for the action column. Ignored when `row_actions`
    /// is unset.
    ///
    /// Defaults to a visually-hidden "Actions", so the column always carries an
    /// accessible name — without one a screen reader announces the row's cells
    /// and then a blank header followed by unexplained buttons. Pass a label to
    /// show it, which is worth doing when the column is wide enough to carry it.
    #[prop(optional, into)]
    action_header: String,
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
    let action_header = StoredValue::new(action_header);

    // Sort state: (column_index, direction)
    let sort_state = RwSignal::new(Option::<(usize, SortDirection)>::None);
    // Current page (0-indexed)
    let current_page = RwSignal::new(0_usize);

    // Derived: sorted rows, each paired with its original index in the `rows`
    // prop so `row_actions` can identify a row after sorting and paging.
    let sorted_rows = Memo::new(move |_| {
        let mut data: Vec<(usize, Vec<String>)> =
            rows.get_value().into_iter().enumerate().collect();
        if let Some((col_idx, direction)) = sort_state.get() {
            sort_indexed_rows(&mut data, col_idx, direction);
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
                                                            .filter_map(|r| r.1.first().cloned())
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
                            // Trailing action header. Emitted outside the column
                            // loop above, so it takes no data index and carries
                            // no sort handler. An unset label still names the
                            // column for assistive tech rather than announcing
                            // a blank header before a run of buttons.
                            {row_actions.map(|_| {
                                let label = action_header.get_value();
                                view! {
                                    <th class="text-right">
                                        {if label.is_empty() {
                                            Either::Left(view! {
                                                <span class="sr-only">"Actions"</span>
                                            })
                                        } else {
                                            Either::Right(label)
                                        }}
                                    </th>
                                }
                            })}
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = visible_rows.get();
                            if rows.is_empty() {
                                let col_count = visual_layout(
                                    columns.get_value().len(),
                                    bulk_select.is_some(),
                                    row_actions.is_some(),
                                ).len();
                                Either::Left(view! {
                                    <tr>
                                        <td colspan=col_count.to_string() class="text-center text-base-content/40 py-8">
                                            "No data available."
                                        </td>
                                    </tr>
                                })
                            } else {
                                Either::Right(rows.into_iter().map(|(row_index, row)| {
                                    let bcols = badge_cols.get_value();
                                    let lcols = link_cols.get_value();
                                    let cols = columns.get_value();
                                    // Trailing action cell. Built here (before the
                                    // cell loop consumes `row`) but rendered after
                                    // it, so it takes no data index — `badge_cols`,
                                    // `link_cols` and the sort state are untouched.
                                    // Only cloned when the prop is actually set.
                                    let action_cell = row_actions.map(|render| {
                                        let cells = row.clone();
                                        view! {
                                            <td class="text-sm text-right whitespace-nowrap">
                                                <div class="flex items-center justify-end gap-1">
                                                    {render.run((row_index, cells))}
                                                </div>
                                            </td>
                                        }
                                    });
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
                                                    if is_link
                                                        && let Some(cb) = on_link_click
                                                    {
                                                        cb.run((col_key.clone(), cell_value.clone()));
                                                    }
                                                };
                                                view! {
                                                    <td class="text-sm">
                                                        <span class=cls on:click=on_click>{cell}</span>
                                                    </td>
                                                }
                                            }).collect::<Vec<_>>()}
                                            {action_cell}
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The visual positions the data columns occupy, in data-index order.
    fn data_positions(data_columns: usize, bulk_select: bool, action_column: bool) -> Vec<usize> {
        let layout = visual_layout(data_columns, bulk_select, action_column);
        let mut positions = vec![usize::MAX; data_columns];
        for (visual, cell) in layout.iter().enumerate() {
            if let VisualCell::Data(data_idx) = cell {
                positions[*data_idx] = visual;
            }
        }
        positions
    }

    // ── visual_layout: the optional cells ──

    #[test]
    fn plain_table_renders_only_its_data_columns() {
        assert_eq!(
            visual_layout(3, false, false),
            vec![
                VisualCell::Data(0),
                VisualCell::Data(1),
                VisualCell::Data(2)
            ],
        );
    }

    #[test]
    fn the_action_column_is_absent_when_the_prop_is_unset() {
        // Existing callers must render byte-identically: no extra cell at all.
        let layout = visual_layout(4, false, false);
        assert_eq!(layout.len(), 4);
        assert!(!layout.contains(&VisualCell::Action));
    }

    #[test]
    fn the_action_column_is_trailing() {
        let layout = visual_layout(4, false, true);
        assert_eq!(layout.len(), 5);
        assert_eq!(layout.last(), Some(&VisualCell::Action));
    }

    #[test]
    fn bulk_select_is_leading() {
        let layout = visual_layout(4, true, false);
        assert_eq!(layout.len(), 5);
        assert_eq!(layout.first(), Some(&VisualCell::BulkSelect));
    }

    #[test]
    fn bulk_select_and_the_action_column_bracket_the_data_columns() {
        assert_eq!(
            visual_layout(2, true, true),
            vec![
                VisualCell::BulkSelect,
                VisualCell::Data(0),
                VisualCell::Data(1),
                VisualCell::Action,
            ],
        );
    }

    // ── the four-combination index matrix ──

    #[test]
    fn the_action_column_never_shifts_a_data_column() {
        // The whole correctness risk of the action column: `badge_columns`,
        // `link_columns` and the sort state address data indices, so appending a
        // column must not renumber them. Only `bulk_select` shifts the *visual*
        // position, and it shifts every data column by exactly one.
        for bulk_select in [false, true] {
            let offset = usize::from(bulk_select);
            for action_column in [false, true] {
                assert_eq!(
                    data_positions(5, bulk_select, action_column),
                    (0..5).map(|i| i + offset).collect::<Vec<_>>(),
                    "bulk_select={bulk_select} action_column={action_column}",
                );
            }
        }
    }

    #[test]
    fn badge_and_link_indices_resolve_to_the_same_visual_column_in_all_four_combinations() {
        // Columns: 0=id (link), 1=name, 2=status (badge), 3=owner.
        const LINK_COL: usize = 0;
        const BADGE_COL: usize = 2;
        for bulk_select in [false, true] {
            let offset = usize::from(bulk_select);
            for action_column in [false, true] {
                let positions = data_positions(4, bulk_select, action_column);
                assert_eq!(
                    positions[LINK_COL],
                    LINK_COL + offset,
                    "link column moved: bulk_select={bulk_select} action_column={action_column}",
                );
                assert_eq!(
                    positions[BADGE_COL],
                    BADGE_COL + offset,
                    "badge column moved: bulk_select={bulk_select} action_column={action_column}",
                );
            }
        }
        // And toggling only the action column leaves every position identical.
        for bulk_select in [false, true] {
            assert_eq!(
                data_positions(4, bulk_select, false),
                data_positions(4, bulk_select, true),
            );
        }
    }

    #[test]
    fn empty_state_colspan_matches_the_rendered_cell_count() {
        assert_eq!(visual_layout(4, false, false).len(), 4);
        assert_eq!(visual_layout(4, true, false).len(), 5);
        assert_eq!(visual_layout(4, false, true).len(), 5);
        assert_eq!(visual_layout(4, true, true).len(), 6);
    }

    #[test]
    fn a_zero_column_table_still_renders_its_optional_cells() {
        assert_eq!(visual_layout(0, false, false).len(), 0);
        assert_eq!(visual_layout(0, true, true).len(), 2);
    }

    // ── sort_indexed_rows ──

    fn fixture() -> Vec<(usize, Vec<String>)> {
        [
            ["c", "Low", "30"],
            ["a", "High", "200"],
            ["b", "Medium", "100"],
        ]
        .into_iter()
        .map(|r| r.map(String::from).to_vec())
        .enumerate()
        .collect()
    }

    #[test]
    fn sorting_uses_the_data_column_that_was_clicked() {
        let mut rows = fixture();
        sort_indexed_rows(&mut rows, 0, SortDirection::Ascending);
        assert_eq!(
            rows.iter().map(|(_, r)| r[0].as_str()).collect::<Vec<_>>(),
            vec!["a", "b", "c"],
        );

        let mut rows = fixture();
        sort_indexed_rows(&mut rows, 1, SortDirection::Ascending);
        assert_eq!(
            rows.iter().map(|(_, r)| r[1].as_str()).collect::<Vec<_>>(),
            vec!["High", "Low", "Medium"],
        );
    }

    #[test]
    fn numeric_columns_compare_numerically_not_lexicographically() {
        let mut rows = fixture();
        sort_indexed_rows(&mut rows, 2, SortDirection::Ascending);
        assert_eq!(
            rows.iter().map(|(_, r)| r[2].as_str()).collect::<Vec<_>>(),
            vec!["30", "100", "200"],
        );
    }

    #[test]
    fn descending_reverses_the_order() {
        let mut rows = fixture();
        sort_indexed_rows(&mut rows, 2, SortDirection::Descending);
        assert_eq!(
            rows.iter().map(|(_, r)| r[2].as_str()).collect::<Vec<_>>(),
            vec!["200", "100", "30"],
        );
    }

    #[test]
    fn sorting_carries_the_original_row_index_so_row_actions_stay_addressable() {
        // `row_actions` receives this index; it must keep pointing at the row's
        // position in the `rows` prop, not at its position on the page.
        let mut rows = fixture();
        sort_indexed_rows(&mut rows, 0, SortDirection::Ascending);
        assert_eq!(
            rows.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            vec![1, 2, 0],
        );
    }

    #[test]
    fn an_out_of_range_sort_column_leaves_every_row_comparing_equal() {
        // Stable sort => original order preserved rather than an arbitrary shuffle.
        let mut rows = fixture();
        sort_indexed_rows(&mut rows, 99, SortDirection::Ascending);
        assert_eq!(
            rows.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            vec![0, 1, 2],
        );
    }
}
