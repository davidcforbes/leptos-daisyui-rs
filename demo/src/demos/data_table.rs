use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::widgets::{DataTable as WidgetDataTable, TableColumn as WidgetTableColumn};
use std::collections::{BTreeSet, HashMap, HashSet};

#[component]
pub fn DataTableDemo() -> impl IntoView {
    // Sample user data
    let generate_users = |count: usize| -> Vec<HashMap<&'static str, String>> {
        (0..count)
            .map(|i| {
                HashMap::from([
                    ("id", format!("{:03}", i + 1)),
                    ("name", format!("User {}", i + 1)),
                    ("email", format!("user{}@example.com", i + 1)),
                    ("role", {
                        let roles = ["Admin", "Developer", "Designer", "Manager", "Analyst"];
                        roles[i % 5].to_string()
                    }),
                    ("department", {
                        let depts = ["Engineering", "Sales", "Marketing", "HR", "Finance"];
                        depts[i % 5].to_string()
                    }),
                    (
                        "status",
                        if i % 3 == 0 { "Active" } else { "Inactive" }.to_string(),
                    ),
                    (
                        "joined",
                        format!("2024-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1),
                    ),
                ])
            })
            .collect()
    };

    // Standard columns (store in RwSignal for multiple use)
    let standard_columns = RwSignal::new(vec![
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new("role", "Role"),
        Column::new("status", "Status"),
        Column::new("joined", "Joined Date"),
    ]);

    // Columns with non-sortable
    let mixed_columns = RwSignal::new(vec![
        Column::new("id", "ID"),
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new_non_sortable("status", "Status"),
        Column::new("joined", "Joined Date"),
    ]);

    // Typed sorting: money / duration / date columns whose display strings do
    // not sort correctly as text ("$1,000" < "$900" because '1' < '9'). The em
    // dash means "not measured" and must not sort as 0.
    let typed_sort_data = RwSignal::new(vec![
        HashMap::from([
            ("account", "Northwind".to_string()),
            ("balance", "$900".to_string()),
            ("days", "9".to_string()),
            ("opened", "2026-03-04".to_string()),
        ]),
        HashMap::from([
            ("account", "Contoso".to_string()),
            ("balance", "$1,000".to_string()),
            ("days", "525".to_string()),
            ("opened", "2025-11-30".to_string()),
        ]),
        HashMap::from([
            ("account", "Fabrikam".to_string()),
            ("balance", "$85".to_string()),
            ("days", "10".to_string()),
            ("opened", "2026-07-01".to_string()),
        ]),
        HashMap::from([
            ("account", "Tailspin".to_string()),
            ("balance", "($1,250.50)".to_string()),
            ("days", "\u{2014}".to_string()),
            ("opened", "2024-01-15".to_string()),
        ]),
    ]);
    let typed_sort_columns = RwSignal::new(vec![
        Column::new("account", "Account"),
        Column::new("balance", "Balance").with_sort_as(SortAs::Number),
        Column::new("days", "Days in Stage").with_sort_as(SortAs::Number),
        Column::new("opened", "Opened").with_sort_as(SortAs::Date),
    ]);

    // Data sets (store in RwSignal for multiple use)
    let small_data = RwSignal::new(generate_users(5));
    let medium_data = RwSignal::new(generate_users(25));
    let large_data = RwSignal::new(generate_users(10000));

    // States
    let (loading, set_loading) = signal(false);
    let (page_size, set_page_size) = signal(10_usize);
    let detail_row_activation = RwSignal::new(0_usize);
    let detail_renderer: RowDetailRenderer = Callback::new(
        move |(absolute_index, row): (usize, TableRow)| {
            (absolute_index % 2 == 0).then(|| {
                let name = row.get("name").cloned().unwrap_or_default();
                let role = row.get("role").cloned().unwrap_or_default();
                view! {
                    <p data-testid="data-table-row-detail">
                        <strong>{name}</strong>
                        {format!(" has a row-specific {role} review note ({absolute_index}).")}
                        <Button class="btn-ghost btn-xs" attr:data-testid="data-table-detail-action">
                            "Review note"
                        </Button>
                    </p>
                }
                .into_any()
            })
        },
    );

    // Runtime localization: columns and texts derived from a locale signal,
    // the pattern a `t()`-based app uses. Toggling the locale must re-render
    // the table chrome (headers, empty state) — asserted by the reactivity
    // suite's `data_table_headers_relocalize_via_dom`.
    let locale_es = RwSignal::new(false);
    // Only low-cardinality columns opt in to filtering. This existing
    // searchable/filterable fixture also consumes the locale signal so the
    // browser suite can prove stateful control localization without adding a
    // duplicate set of audited controls to the showcase page.
    let filterable_columns = Signal::derive(move || {
        if locale_es.get() {
            vec![
                Column::new("name", "Nombre"),
                Column::new("email", "Correo"),
                Column::new("role", "Rol").filterable(),
                Column::new("department", "Departamento").filterable(),
                Column::new("status", "Estado").filterable(),
            ]
        } else {
            vec![
                Column::new("name", "Name"),
                Column::new("email", "Email"),
                Column::new("role", "Role").filterable(),
                Column::new("department", "Department").filterable(),
                Column::new("status", "Status").filterable(),
            ]
        }
    });
    let localized_columns = Signal::derive(move || {
        if locale_es.get() {
            vec![
                Column::new("name", "Nombre"),
                Column::new("email", "Correo"),
            ]
        } else {
            vec![Column::new("name", "Name"), Column::new("email", "Email")]
        }
    });
    let localized_texts = Signal::derive(move || {
        if locale_es.get() {
            DataTableTexts {
                loading: "Cargando datos...".to_string(),
                empty: "No hay datos disponibles".to_string(),
                page_indicator: "Página {current} de {total}".to_string(),
                previous: "Anterior".to_string(),
                next: "Siguiente".to_string(),
                search_placeholder: "Buscar...".to_string(),
                search_label: "Buscar en la tabla".to_string(),
                row_range: "Mostrando {start}\u{2013}{end} de {total}".to_string(),
                filter_all: "Todos".to_string(),
                filter_label: "Filtrar por {column}".to_string(),
            }
        } else {
            DataTableTexts::default()
        }
    });
    let localized_sort_texts = Signal::derive(move || {
        if locale_es.get() {
            DataTableSortTexts {
                unsorted: "{column}, sin ordenar. Activar para ordenar ascendente.".to_string(),
                ascending: "{column}, orden ascendente. Activar para ordenar descendente."
                    .to_string(),
                descending: "{column}, orden descendente. Activar para ordenar ascendente."
                    .to_string(),
            }
        } else {
            DataTableSortTexts::default()
        }
    });

    // Multi-select state for the selection demo
    let selected_rows = RwSignal::new(BTreeSet::<usize>::new());
    let selection_anchor = RwSignal::new(Option::<usize>::None);
    let selection_data = RwSignal::new(generate_users(20));
    let selection_columns = RwSignal::new(vec![
        Column::new("id", "ID"),
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new("role", "Role"),
    ]);

    // Row-activation state. Its own `selected_rows` so this demo's selection is
    // independent of the multi-select demo above -- the point here is that a
    // plain click does NOT land in this set.
    let activated_row = RwSignal::new(Option::<usize>::None);
    let activate_count = RwSignal::new(0_usize);
    let activate_selected = RwSignal::new(BTreeSet::<usize>::new());
    // Secondary activation (`on_row_inspect`, ldui-tmr): double-click or
    // Shift+Enter. The count exists so a double-click firing activate twice
    // (the defect the detail>1 swallow prevents) cannot pass unnoticed.
    let inspected_row = RwSignal::new(Option::<usize>::None);
    let inspect_count = RwSignal::new(0_usize);

    // Action column for the activation demo: clicking "Open" must NOT also
    // activate the row — `Column::action()` scopes the cell's events away
    // from row interaction (asserted by the reactivity suite's
    // `action_cell_click_does_not_activate_row`).
    let open_count = RwSignal::new(0_usize);
    let activation_columns = RwSignal::new(vec![
        Column::new("id", "ID"),
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new_non_sortable("actions", "Actions")
            .action()
            .with_renderer(0),
    ]);
    let open_renderer: CellRenderer = Callback::new(
        move |(_idx, _row): (usize, HashMap<&'static str, String>)| {
            view! {
                <Button
                    size=ButtonSize::Xs
                    color=ButtonColor::Primary
                    on:click=move |_| open_count.update(|n| *n += 1)
                >
                    "Open"
                </Button>
            }
            .into_any()
        },
    );

    // Keyed row identity (`row_key`): selection keys off a stable row id, so
    // replacing the data vec (server page swap, live pool removals) keeps the
    // same *rows* selected instead of clearing or drifting by position.
    let keyed_data = RwSignal::new(generate_users(6));
    let keyed_selected = RwSignal::new(BTreeSet::<usize>::new());
    let keyed_columns = RwSignal::new(vec![
        Column::new("id", "ID"),
        Column::new("name", "Name"),
        Column::new("email", "Email"),
    ]);

    // Controlled custom filter (`extra_filter` + `toolbar`): a derived domain
    // filter (here "Admins only") the distinct-value dropdowns can't express,
    // composing with the built-in toolbar instead of replacing it.
    let admins_only = RwSignal::new(false);
    let custom_filter_data = RwSignal::new(generate_users(25));
    let custom_filter_columns = RwSignal::new(vec![
        Column::new("name", "Name"),
        Column::new("role", "Role"),
        Column::new("status", "Status").filterable(),
    ]);

    // ServerDataTable + on_query_change: a simulated backend applies the
    // emitted TableQuery (search/sort/filters/page) to a 57-row fixture and
    // returns one 10-row page — the round-trip a real server table performs.
    let server_fixture = StoredValue::new(generate_users(57));
    let server_rows = RwSignal::new(Vec::<HashMap<&'static str, String>>::new());
    let server_page = RwSignal::new(1_i64);
    let server_total = RwSignal::new(0_i64);
    let last_query = RwSignal::new(String::new());
    // Server-variant activation forwarding (ldui-1gp): the same
    // click/dblclick contract as the client table, with page-local indices.
    let server_activated = RwSignal::new(Option::<usize>::None);
    let server_inspected = RwSignal::new(Option::<usize>::None);
    let server_columns = RwSignal::new(vec![
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new("role", "Role").filterable(),
        Column::new("status", "Status").filterable(),
    ]);

    let run_server_query = move |q: TableQuery| {
        let query_debug = serde_json::json!({
            "page": q.page,
            "page_size": q.page_size,
            "search": q.search.clone(),
            "sort": q.sort.map(|(column, order)| serde_json::json!({
                "column": column,
                "order": order.as_aria_str(),
            })),
            "filters": q.filters.clone(),
        });
        let mut items = server_fixture.get_value();
        items.retain(|row| {
            q.filters
                .iter()
                .all(|(col, v)| v.is_empty() || row.get(col).is_some_and(|c| c == v))
        });
        if !q.search.is_empty() {
            let s = q.search.to_lowercase();
            items.retain(|row| row.values().any(|v| v.to_lowercase().contains(&s)));
        }
        if let Some((col, order)) = q.sort {
            items.sort_by(|a, b| {
                let av = a.get(col).cloned().unwrap_or_default();
                let bv = b.get(col).cloned().unwrap_or_default();
                let c = av.cmp(&bv);
                match order {
                    SortOrder::Asc => c,
                    SortOrder::Desc => c.reverse(),
                }
            });
        }
        server_total.set(items.len() as i64);
        let start = ((q.page - 1) * q.page_size).max(0) as usize;
        server_rows.set(
            items
                .into_iter()
                .skip(start)
                .take(q.page_size.max(0) as usize)
                .collect(),
        );
        server_page.set(q.page);
        last_query.set(format!(
            "page={} size={} search={:?} sort={:?} filters={:?}",
            q.page, q.page_size, q.search, q.sort, q.filters
        ));
        crate::debug_state::set("server_datatable.query", query_debug);
    };
    // Initial fetch: page 1, no query shape.
    run_server_query(TableQuery {
        page: 1,
        page_size: 10,
        search: String::new(),
        sort: None,
        filters: ColumnFilters::new(),
    });

    // Column resize + typed cells (Badge/Icon) + row background + clipboard export
    let feature_data = RwSignal::new(generate_users(12));
    let feature_columns = RwSignal::new(vec![
        Column::new("id", "ID").with_min_width(60),
        Column::new("name", "Name").with_min_width(120),
        Column::new("email", "Email").with_min_width(180),
        Column::new("role", "Role")
            .with_min_width(110)
            .with_typed_cell(0),
        Column::new("status", "Status")
            .with_min_width(100)
            .with_typed_cell(1),
        Column::new_non_sortable("actions", "Actions")
            .with_min_width(90)
            .non_resizable()
            .with_renderer(0),
    ]);

    // Typed cell: render "Role" as a Lucide icon instead of plain text.
    let role_typed_cell: TypedCellFn =
        Callback::new(|(_idx, row): (usize, HashMap<&'static str, String>)| {
            let role = row.get("role").cloned().unwrap_or_default();
            let icon_name = match role.as_str() {
                "Admin" => "shield",
                "Developer" => "code",
                "Designer" => "palette",
                "Manager" => "briefcase",
                _ => "user",
            };
            TypedCell::Icon {
                name: icon_name.to_string(),
                color: "text-primary".to_string(),
            }
        });

    // Typed cell: render "Status" as a colored Badge instead of plain text.
    let status_typed_cell: TypedCellFn =
        Callback::new(|(_idx, row): (usize, HashMap<&'static str, String>)| {
            let status = row.get("status").cloned().unwrap_or_default();
            let color = if status == "Active" {
                BadgeColor::Success
            } else {
                BadgeColor::Neutral
            };
            TypedCell::Badge {
                text: status,
                color,
            }
        });

    // Row background hook: tint inactive rows.
    let feature_row_class_fn =
        Callback::new(|(_idx, row): (usize, HashMap<&'static str, String>)| {
            if row.get("status").map(String::as_str) == Some("Inactive") {
                "bg-base-200/60".to_string()
            } else {
                String::new()
            }
        });

    // Clipboard export: "Copy" button per row, using the crate's pure
    // `row_with_headers_text` helper and the same wasm-gated
    // `navigator().clipboard().write_text(...)` pattern as `AiChat`'s
    // message-copy button.
    let copy_row_renderer: CellRenderer = Callback::new(
        move |(_abs_idx, row): (usize, HashMap<&'static str, String>)| {
            let columns_for_copy = feature_columns.get_untracked();
            let row_for_copy = row.clone();
            view! {
                <button
                    type="button"
                    class="btn btn-ghost btn-xs"
                    title="Copy row (tab-separated, with headers)"
                    on:click=move |_| {
                        let _text = row_with_headers_text(&row_for_copy, &columns_for_copy);
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.navigator().clipboard().write_text(&_text);
                            }
                        }
                    }
                >
                    "Copy"
                </button>
            }
            .into_any()
        },
    );

    // ── widgets::DataTable — per-row action column (ldui-3qc) ──
    //
    // The consumer shape this was built for: an "Open Work" queue where every
    // row carries several controls, a few of them disabled with a tooltip that
    // explains why. The action column is a trailing cell that takes no data
    // index, so `badge_column_keys` and the sort state keep addressing the same
    // columns whether or not it (or the leading `bulk_select` checkbox) is on.
    let widget_columns = vec![
        WidgetTableColumn {
            key: "ref".into(),
            label: "Ref".into(),
            sortable: true,
            width: Some("110px".into()),
        },
        WidgetTableColumn {
            key: "work_type".into(),
            label: "Work Type".into(),
            sortable: true,
            width: None,
        },
        WidgetTableColumn {
            key: "status".into(),
            label: "Status".into(),
            sortable: true,
            width: Some("150px".into()),
        },
        WidgetTableColumn {
            key: "owner".into(),
            label: "Owner".into(),
            sortable: true,
            width: Some("160px".into()),
        },
    ];
    // Twelve rows against page_size=5 so the table actually pages (three pages)
    // — the readout below it then demonstrates that the index the callback
    // receives survives paging, not just sorting.
    let widget_rows: Vec<Vec<String>> = [
        ["WK-1041", "Intake Call", "Active", "M. Gonzalez"],
        ["WK-1042", "Court Filing", "Blocked", "J. Smith"],
        ["WK-1043", "Records Request", "Active", "A. Tanaka"],
        ["WK-1044", "Court Filing", "Under Review", "O. Haddad"],
        ["WK-1045", "Intake Call", "Active", "P. Patel"],
        ["WK-1046", "Records Request", "Blocked", "L. Nielsen"],
        ["WK-1047", "Intake Call", "Under Review", "C. Wei"],
        ["WK-1048", "Court Filing", "Active", "F. Zahra"],
        ["WK-1049", "Records Request", "Active", "D. Rivera"],
        ["WK-1050", "Court Filing", "Blocked", "A. Kowalska"],
        ["WK-1051", "Intake Call", "Active", "M. Gonzalez"],
        ["WK-1052", "Records Request", "Under Review", "J. Smith"],
    ]
    .into_iter()
    .map(|r| r.into_iter().map(String::from).collect())
    .collect();

    let widget_last_action = RwSignal::new(String::from("\u{2014}"));
    let widget_bulk = RwSignal::new(HashSet::<String>::new());

    // A bare closure: `row_actions` takes `impl Into<Callback<_, _>>`, so no
    // `Callback::new(...)` wrapper is needed.
    let widget_row_actions = move |(row_index, row): (usize, Vec<String>)| {
        // Identity comes from the row id (cell 0), not from `row_index` —
        // `row_index` is a position into the `rows` snapshot this component was
        // handed, which is not necessarily a position into the caller's store.
        let work_ref = row.first().cloned().unwrap_or_default();
        let work_type = row.get(1).cloned().unwrap_or_default();
        let status = row.get(2).cloned().unwrap_or_default();

        // Records the click so the demo shows that both the id and the index
        // stay bound to the right row after sorting and after paging.
        let record = move |verb: &'static str, subject: String| {
            move |_| {
                widget_last_action.set(format!("{verb} {subject} (row_index {row_index})"));
            }
        };

        // One control: a live button, or an unavailable one wrapped in a
        // tooltip that says why.
        let control = move |label: &'static str,
                            extra: &'static str,
                            verb: &'static str,
                            blocked: Option<&'static str>,
                            subject: String|
              -> AnyView {
            match blocked {
                None => view! {
                    <Button size=ButtonSize::Xs class=extra on:click=record(verb, subject)>
                        {label}
                    </Button>
                }
                .into_any(),
                // `aria-disabled` on a still-focusable button rather than the
                // native `disabled` attribute: a natively-disabled button is
                // removed from the tab order, so a keyboard user could never
                // reach the tooltip explaining why the control is unavailable.
                // daisyUI shows the tooltip on `:has(:focus-visible)`, so this
                // shape surfaces it by keyboard as well as by hover. The click
                // handler short-circuits, since aria-disabled is advisory only.
                Some(why) => view! {
                    <Tooltip
                        tip=Signal::derive(move || why.to_string())
                        position=TooltipPosition::Left
                    >
                        <button
                            type="button"
                            class=format!("btn btn-xs btn-disabled {extra}")
                            aria-disabled="true"
                            on:click=move |ev: leptos::ev::MouseEvent| ev.prevent_default()
                        >
                            {label}
                        </button>
                    </Tooltip>
                }
                .into_any(),
            }
        };

        // Court filings have no telephony route, and blocked work cannot be
        // completed — so the control set differs row by row.
        let no_route = (work_type == "Court Filing")
            .then_some("No call/SMS route resolves for every work type this row can carry.");
        let blocked = (status == "Blocked")
            .then_some("Blocked by an unresolved dependency; clear it before completing.");

        view! {
            {control("Open", "btn-ghost", "Opened", None, work_ref.clone())}
            {control("Complete", "btn-ghost", "Completed", blocked, work_ref.clone())}
            {control("Assign", "btn-ghost", "Assigned", None, work_ref.clone())}
            {control("Note", "btn-ghost", "Noted", None, work_ref.clone())}
            {control("Call", "btn-ghost", "Called", no_route, work_ref.clone())}
            {control("SMS", "btn-ghost", "Texted", no_route, work_ref.clone())}
            {control("Delete", "btn-ghost text-error", "Deleted", None, work_ref.clone())}
        }
        .into_any()
    };

    view! {
        <ContentLayout
            title="DataTable"
            description="Production-ready data table with sorting, pagination, loading states, and efficient handling of large datasets"
        >
            // Basic Usage
            <Section title="Basic DataTable">
                <p class="text-sm opacity-70 mb-4">
                    "Click column headers to sort. Navigate pages using the controls below."
                </p>
                <DataTable
                    data=small_data
                    columns=standard_columns
                    page_size=3
                    // PixelProof oracle (ldui-49w.1): surface the internal sort
                    // state at window.__APP_DEBUG__.state().state["datatable.sort"].
                    on_sort_change=Callback::new(|(col, order): (&'static str, SortOrder)| {
                        crate::debug_state::set(
                            "datatable.sort",
                            serde_json::json!({ "column": col, "order": order.as_aria_str() }),
                        );
                    })
                    attr:id="keyboard-sort-table"
                />
            </Section>

            // Sortable vs Non-Sortable Columns
            <Section title="Sortable and Non-Sortable Columns">
                <p class="text-sm opacity-70 mb-4">
                    "The 'Status' column is marked as non-sortable and won't respond to clicks."
                </p>
                <DataTable
                    data=small_data
                    columns=mixed_columns
                    page_size=5
                    attr:id="mixed-sort-table"
                />
            </Section>

            // Typed sorting (SortAs)
            <Section title="Typed Sorting (Number and Date columns)">
                <p class="text-sm opacity-70 mb-4">
                    "Columns sort as text by default, which is wrong for formatted numbers: "
                    <code>"\"$1,000\""</code>" would sort before "<code>"\"$900\""</code>
                    " on its first digit. "<code>"Column::with_sort_as(SortAs::Number)"</code>
                    " compares the parsed value instead — currency symbols, thousands separators, "
                    "percent signs and accounting parentheses are all understood. "
                    <code>"SortAs::Date"</code>" does the same for dates. A cell that holds no value "
                    "(the em dash below) is not zero, so it sorts last in both directions."
                </p>
                <DataTable
                    data=typed_sort_data
                    columns=typed_sort_columns
                    paginate=false
                />
            </Section>

            // Table Sizes
            <Section title="Table Size Variants">
                <div class="space-y-6">
                    <div>
                        <h4 class="font-semibold mb-2">"Extra Small (XS)"</h4>
                        <DataTable
                            data=small_data
                            columns=standard_columns
                            page_size=3
                            table_size=TableSize::Xs
                        />
                    </div>

                    <div>
                        <h4 class="font-semibold mb-2">"Small (SM)"</h4>
                        <DataTable
                            data=small_data
                            columns=standard_columns
                            page_size=3
                            table_size=TableSize::Sm
                        />
                    </div>

                    <div>
                        <h4 class="font-semibold mb-2">"Medium (MD - Default)"</h4>
                        <DataTable
                            data=small_data
                            columns=standard_columns
                            page_size=3
                            table_size=TableSize::Md
                        />
                    </div>

                    <div>
                        <h4 class="font-semibold mb-2">"Large (LG)"</h4>
                        <DataTable
                            data=small_data
                            columns=standard_columns
                            page_size=3
                            table_size=TableSize::Lg
                        />
                    </div>
                </div>
            </Section>

            // Styling Options
            <Section title="Styling Options">
                <div class="space-y-6">
                    <div>
                        <h4 class="font-semibold mb-2">"Zebra Striping"</h4>
                        <DataTable
                            data=small_data
                            columns=standard_columns
                            page_size=5
                            zebra=true
                        />
                    </div>

                    <div>
                        <h4 class="font-semibold mb-2">"Pinned Rows (Header stays visible on scroll)"</h4>
                        <div class="max-h-64 overflow-auto">
                            <DataTable
                                data=medium_data
                                columns=standard_columns
                                page_size=50
                                pin_rows=true
                                paginate=false
                            />
                        </div>
                    </div>

                    <div>
                        <h4 class="font-semibold mb-2">"Pinned Columns (First column stays on scroll)"</h4>
                        <div class="overflow-x-auto max-w-md">
                            <DataTable
                                data=small_data
                                columns=mixed_columns
                                page_size=5
                                pin_cols=true
                            />
                        </div>
                    </div>

                    <div>
                        <h4 class="font-semibold mb-2">"Combined: Zebra + Pinned Rows + Small Size"</h4>
                        <DataTable
                            data=small_data
                            columns=standard_columns
                            page_size=5
                            zebra=true
                            pin_rows=true
                            table_size=TableSize::Sm
                        />
                    </div>
                </div>
            </Section>

            // Loading State
            <Section title="Loading State">
                <button
                    class="btn btn-primary btn-sm mb-4"
                    on:click=move |_| set_loading.update(|l| *l = !*l)
                >
                    {move || if loading.get() { "Hide Loading" } else { "Show Loading" }}
                </button>
                <DataTable
                    data=medium_data
                    columns=standard_columns
                    page_size=10
                    loading=loading
                />
            </Section>

            // Empty State
            <Section title="Empty State">
                <p class="text-sm opacity-70 mb-4">
                    "Displays a message when no data is available."
                </p>
                <DataTable
                    data=Signal::derive(Vec::<HashMap<&'static str, String>>::new)
                    columns=standard_columns
                    page_size=10
                />
            </Section>

            // Dynamic Page Size
            <Section title="Dynamic Page Size">
                <div class="mb-4 flex items-center gap-4">
                    <label class="label" r#for="dynamic-page-size">
                        <span class="text-sm">"Rows per page:"</span>
                    </label>
                    <select
                        id="dynamic-page-size"
                        class="select select-bordered select-sm"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            if let Ok(size) = value.parse::<usize>() {
                                set_page_size.set(size);
                            }
                        }
                    >
                        <option value="5">"5"</option>
                        <option value="10" selected>"10"</option>
                        <option value="25">"25"</option>
                        <option value="50">"50"</option>
                    </select>
                </div>
                <DataTable
                    data=medium_data
                    columns=standard_columns
                    page_size=page_size
                />
            </Section>

            // Per-column filter row
            <Section title="Per-Column Filter Row">
                <p class="text-sm opacity-70 mb-4">
                    "Columns opt in with " <code>"Column::filterable()"</code> ", which gives them a "
                    "dropdown of their distinct values beneath the header. Filters combine with each "
                    "other and with the search box (all must match). Columns that don't opt in get no "
                    "dropdown, and a table with no filterable column renders no filter row at all."
                </p>
                <DataTable
                    data=Signal::derive(move || generate_users(60))
                    columns=filterable_columns
                    page_size=8
                    searchable=true
                    texts=localized_texts
                    attr:id="filter-row-table"
                />
            </Section>

            <Section title="Per-Row Full-Width Detail">
                <p class="text-sm opacity-70 mb-4">
                    "Only rows with genuinely row-specific explanatory text receive a full-width detail row. The detail stays paired through sorting and paging."
                </p>
                <p class="text-xs opacity-60 mb-2">
                    "Row activations: "
                    <strong data-testid="detail-row-activation-count">{move || detail_row_activation.get()}</strong>
                </p>
                <DataTable
                    data=small_data
                    columns=standard_columns
                    page_size=3
                    detail_renderer=detail_renderer
                    on_row_activate=Callback::new(move |_| {
                        detail_row_activation.update(|count| *count += 1);
                    })
                    attr:id="detail-row-table"
                />
            </Section>

            // Responsive / auto-growing page size
            <Section title="Responsive Page Size (auto_page_size)">
                <p class="text-sm opacity-70 mb-4">
                    "With " <code>"auto_page_size=true"</code> " the row count is derived from the "
                    "table's rendered height instead of a fixed " <code>"page_size"</code> ", so a "
                    "taller window shows more rows. Drag the resizer below (or resize the window) and "
                    "watch the row count and \"Showing X\u{2013}Y of Z\" caption follow. Below the "
                    "five-row usability floor it keeps the configured page size and scrolls instead."
                </p>
                <div class="alert alert-info mb-4">
                    <span>
                        "Needs a definite height: pass " <code>"max_height"</code>
                        " (used here, and promoted to a real " <code>"height"</code> ") or give the "
                        "table a parent that fixes its height. Sized from its own rows instead, the "
                        "table's height would be a function of the row count derived from it."
                    </span>
                </div>
                // `resize-y` + `overflow-auto` makes this box user-resizable, so the
                // ResizeObserver can be exercised without resizing the browser.
                <div class="resize-y overflow-auto border border-base-300 rounded-lg p-3 h-96 min-h-32">
                    <DataTable
                        data=Signal::derive(move || generate_users(200))
                        columns=standard_columns
                        auto_page_size=true
                        max_height="100%"
                        attr:id="auto-page-table"
                    />
                </div>
            </Section>

            // No Pagination
            <Section title="Without Pagination">
                <p class="text-sm opacity-70 mb-4">
                    "Disable pagination to show all rows at once (useful for small datasets)."
                </p>
                <DataTable
                    data=small_data
                    columns=standard_columns
                    paginate=false
                />
            </Section>

            // Performance Test
            <Section title="Performance Test: 10,000 Rows">
                <p class="text-sm opacity-70 mb-4">
                    "Efficient index-based operations handle large datasets smoothly. Try sorting by different columns."
                    " With 10,000 rows at 50 per page (200 pages), the pagination bar below shows the numbered "
                    "page-button windowing with ellipsis, plus the \"Showing X\u{2013}Y of Z\" row-range caption."
                </p>
                <div class="alert alert-info mb-4">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        class="stroke-current shrink-0 w-6 h-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        ></path>
                    </svg>
                    <div>
                        <p class="font-semibold">"Showing 10,000 rows"</p>
                        <p class="text-sm">
                            "Only the current page's rows are rendered for optimal performance."
                        </p>
                    </div>
                </div>
                <DataTable
                    data=large_data
                    columns=Signal::derive(move || {
                        vec![
                            Column::new("id", "ID"),
                            Column::new("name", "Name"),
                            Column::new("email", "Email"),
                            Column::new("department", "Department"),
                            Column::new("role", "Role"),
                            Column::new_non_sortable("status", "Status"),
                        ]
                    })
                    page_size=50
                    zebra=true
                    table_size=TableSize::Sm
                    attr:id="geometry-sort-table"
                />
            </Section>

            // Custom Text Strings
            <Section title="Custom Text Strings">
                <p class="text-sm opacity-70 mb-4">
                    "Customize loading, empty, and pagination text for internationalization."
                </p>
                <DataTable
                    data=Signal::derive(Vec::<HashMap<&'static str, String>>::new)
                    columns=standard_columns
                    page_size=10
                    texts=DataTableTexts {
                        loading: "Cargando datos...".to_string(),
                        empty: "No hay datos disponibles".to_string(),
                        page_indicator: "Página {current} de {total}".to_string(),
                        previous: "Anterior".to_string(),
                        next: "Siguiente".to_string(),
                        search_placeholder: "Buscar...".to_string(),
                        search_label: "Buscar en la tabla".to_string(),
                        row_range: "Mostrando {start}\u{2013}{end} de {total}".to_string(),
                        filter_all: "Todos".to_string(),
                        filter_label: "Filtrar por {column}".to_string(),
                    }
                />
            </Section>

            // Runtime localization: language switch re-renders table chrome
            <Section title="Runtime Localization">
                <p class="text-sm opacity-70 mb-4">
                    "Headers and texts derived from a locale signal re-render on a language switch — no remount needed."
                </p>
                <Button
                    color=ButtonColor::Secondary
                    on:click=move |_| locale_es.update(|es| *es = !*es)
                    attr:id="locale-toggle"
                >
                    {move || if locale_es.get() { "Switch to English" } else { "Cambiar a español" }}
                </Button>
                <DataTable
                    data=Signal::derive(Vec::<HashMap<&'static str, String>>::new)
                    columns=localized_columns
                    texts=localized_texts
                    sort_texts=localized_sort_texts
                    paginate=false
                    attr:id="localized-table"
                />
            </Section>

            // Code Example
            <Section title="Usage Example">
                <div class="mockup-code text-sm">
                    <pre data-prefix="1">
                        <code>"let columns = vec!["</code>
                    </pre>
                    <pre data-prefix="2">
                        <code>"    Column::new(\"name\", \"Name\"),"</code>
                    </pre>
                    <pre data-prefix="3">
                        <code>"    Column::new(\"email\", \"Email\"),"</code>
                    </pre>
                    <pre data-prefix="4">
                        <code>"    Column::new_non_sortable(\"status\", \"Status\"),"</code>
                    </pre>
                    <pre data-prefix="5">
                        <code>"];"</code>
                    </pre>
                    <pre data-prefix="6">
                        <code>""</code>
                    </pre>
                    <pre data-prefix="7">
                        <code>"let data = vec!["</code>
                    </pre>
                    <pre data-prefix="8">
                        <code>"    HashMap::from(["</code>
                    </pre>
                    <pre data-prefix="9">
                        <code>"        (\"name\", \"Alice\".to_string()),"</code>
                    </pre>
                    <pre data-prefix="10">
                        <code>"        (\"email\", \"alice@example.com\".to_string()),"</code>
                    </pre>
                    <pre data-prefix="11">
                        <code>"        (\"status\", \"Active\".to_string()),"</code>
                    </pre>
                    <pre data-prefix="12">
                        <code>"    ]),"</code>
                    </pre>
                    <pre data-prefix="13">
                        <code>"];"</code>
                    </pre>
                    <pre data-prefix="14">
                        <code>""</code>
                    </pre>
                    <pre data-prefix="15">
                        <code>"view! {"</code>
                    </pre>
                    <pre data-prefix="16">
                        <code>"    <DataTable"</code>
                    </pre>
                    <pre data-prefix="17">
                        <code>"        data=Signal::derive(move || data.clone())"</code>
                    </pre>
                    <pre data-prefix="18">
                        <code>"        columns=Signal::derive(move || columns.clone())"</code>
                    </pre>
                    <pre data-prefix="19">
                        <code>"        page_size=10"</code>
                    </pre>
                    <pre data-prefix="20">
                        <code>"        zebra=true"</code>
                    </pre>
                    <pre data-prefix="21">
                        <code>"    />"</code>
                    </pre>
                    <pre data-prefix="22">
                        <code>"}"</code>
                    </pre>
                </div>
            </Section>

            // Multi-select rows (Ctrl / Shift)
            <Section title="Multi-select rows">
                <p class="text-sm opacity-70 mb-2">
                    "Click a row to select it. " <kbd class="kbd kbd-xs">"Ctrl"</kbd>
                    "+click toggles, " <kbd class="kbd kbd-xs">"Shift"</kbd>
                    "+click extends the range from the anchor. Rows are keyboard-operable too: "
                    <kbd class="kbd kbd-xs">"Tab"</kbd> " to a row, then "
                    <kbd class="kbd kbd-xs">"Enter"</kbd> " / " <kbd class="kbd kbd-xs">"Space"</kbd>
                    " (with " <kbd class="kbd kbd-xs">"Ctrl"</kbd> " / " <kbd class="kbd kbd-xs">"Shift"</kbd>
                    ") does the same as a click."
                </p>
                <p class="text-sm opacity-70 mb-4">
                    "Selected absolute indices: "
                    <code>
                        {move || {
                            let s = selected_rows.get();
                            if s.is_empty() {
                                "(none)".to_string()
                            } else {
                                s.iter()
                                    .map(|i| i.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            }
                        }}
                    </code>
                    " — anchor: "
                    <code>
                        {move || {
                            selection_anchor
                                .get()
                                .map(|i| i.to_string())
                                .unwrap_or_else(|| "(none)".to_string())
                        }}
                    </code>
                </p>
                <DataTable
                    data=selection_data
                    columns=selection_columns
                    page_size=8
                    selected_rows=selected_rows
                    selection_anchor=selection_anchor
                />
            </Section>

            // Row activation (opt-in `on_row_activate`)
            <Section title="Row Activation (on_row_activate)">
                <p class="text-sm opacity-70 mb-2">
                    "Pass " <code>"on_row_activate"</code>
                    " and a plain click stops selecting and instead calls the callback with the "
                    "row's absolute index \u{2014} the same index space as "
                    <code>"selected_rows"</code> ", so it survives pagination and sorting. Use it "
                    "to open a detail page from a drilldown grid."
                </p>
                <p class="text-sm opacity-70 mb-4">
                    "Modified clicks still select, so both interactions coexist on one table: "
                    <kbd class="kbd kbd-xs">"Ctrl"</kbd> "+click toggles and "
                    <kbd class="kbd kbd-xs">"Shift"</kbd>
                    "+click extends, neither activating. Without the callback, every click selects "
                    "exactly as before it existed. Keyboard works the same: "
                    <kbd class="kbd kbd-xs">"Enter"</kbd> " / " <kbd class="kbd kbd-xs">"Space"</kbd>
                    " activates, " <kbd class="kbd kbd-xs">"Ctrl"</kbd> " / "
                    <kbd class="kbd kbd-xs">"Shift"</kbd> " + Enter selects. With "
                    <code>"on_row_inspect"</code>
                    " also set, a double-click (or " <kbd class="kbd kbd-xs">"Shift"</kbd> "+"
                    <kbd class="kbd kbd-xs">"Enter"</kbd>
                    ") fires the inspector instead \u{2014} the first click still activates once, "
                    "and the repeat click is swallowed so activation never fires twice."
                </p>
                <div class="flex flex-wrap gap-4 mb-4 text-sm">
                    <span>
                        "Last activated: "
                        <code data-testid="activated-row">
                            {move || {
                                activated_row
                                    .get()
                                    .map(|i| i.to_string())
                                    .unwrap_or_else(|| "(none)".to_string())
                            }}
                        </code>
                    </span>
                    <span>
                        "Activations: "
                        <code data-testid="activate-count">{move || activate_count.get()}</code>
                    </span>
                    <span>
                        "Selected (Ctrl/Shift only): "
                        <code data-testid="activate-selected">
                            {move || {
                                let s = activate_selected.get();
                                if s.is_empty() {
                                    "(none)".to_string()
                                } else {
                                    s.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")
                                }
                            }}
                        </code>
                    </span>
                    <span>
                        "Opens (action cell, never activates): "
                        <code data-testid="open-count">{move || open_count.get()}</code>
                    </span>
                    <span>
                        "Last inspected (dblclick / Shift+Enter): "
                        <code data-testid="inspected-row">
                            {move || {
                                inspected_row
                                    .get()
                                    .map(|i| i.to_string())
                                    .unwrap_or_else(|| "(none)".to_string())
                            }}
                        </code>
                    </span>
                    <span>
                        "Inspects: "
                        <code data-testid="inspect-count">{move || inspect_count.get()}</code>
                    </span>
                </div>
                <DataTable
                    data=selection_data
                    columns=activation_columns
                    page_size=8
                    selected_rows=activate_selected
                    cell_renderers=vec![open_renderer]
                    on_row_activate=Callback::new(move |idx: usize| {
                        activated_row.set(Some(idx));
                        activate_count.update(|n| *n += 1);
                    })
                    on_row_inspect=Callback::new(move |idx: usize| {
                        inspected_row.set(Some(idx));
                        inspect_count.update(|n| *n += 1);
                    })
                    attr:id="activation-table"
                />
            </Section>

            // Keyed row identity (opt-in `row_key`)
            <Section title="Keyed Row Identity (row_key)">
                <p class="text-sm opacity-70 mb-2">
                    "Pass " <code>"row_key"</code>
                    " and selection keys off each row's stable identity instead of its position. "
                    "Select a row, then reverse the data \u{2014} the same row stays selected even "
                    "though every index changed. Without a key, replacing the data clears the "
                    "selection."
                </p>
                <div class="mb-4 flex items-center gap-4 text-sm">
                    <Button
                        size=ButtonSize::Sm
                        color=ButtonColor::Secondary
                        on:click=move |_| keyed_data.update(|d| d.reverse())
                        attr:id="keyed-reverse"
                    >
                        "Reverse rows"
                    </Button>
                    <span>
                        "Selected ids: "
                        <code data-testid="keyed-selected-ids">
                            {move || {
                                let d = keyed_data.get();
                                let s = keyed_selected.get();
                                if s.is_empty() {
                                    "(none)".to_string()
                                } else {
                                    s.iter()
                                        .filter_map(|&i| d.get(i).and_then(|r| r.get("id")).cloned())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                }
                            }}
                        </code>
                    </span>
                </div>
                <DataTable
                    data=keyed_data
                    columns=keyed_columns
                    paginate=false
                    selected_rows=keyed_selected
                    row_key=Callback::new(|row: HashMap<&'static str, String>| {
                        row.get("id").cloned().unwrap_or_default()
                    })
                    attr:id="keyed-table"
                />
            </Section>

            // Controlled custom filter (opt-in `extra_filter` + `toolbar`)
            <Section title="Custom Filters (extra_filter + toolbar)">
                <p class="text-sm opacity-70 mb-4">
                    "A derived domain filter the per-column dropdowns can't express: the "
                    "\"Admins only\" toggle lives in the " <code>"toolbar"</code>
                    " slot beside the built-in search box, and its "
                    <code>"extra_filter"</code>
                    " predicate ANDs with the Status dropdown and the search — the built-in "
                    "toolbar stays, nothing is rebuilt."
                </p>
                <DataTable
                    data=custom_filter_data
                    columns=custom_filter_columns
                    paginate=false
                    searchable=true
                    extra_filter=Callback::new(
                        move |(_idx, row): (usize, HashMap<&'static str, String>)| {
                            !admins_only.get() || row.get("role").is_some_and(|r| r == "Admin")
                        },
                    )
                    toolbar=ViewFn::from(move || {
                        view! {
                            <Button
                                size=ButtonSize::Sm
                                style=ButtonStyle::Outline
                                color=Signal::derive(move || {
                                    if admins_only.get() {
                                        ButtonColor::Primary
                                    } else {
                                        ButtonColor::Neutral
                                    }
                                })
                                on:click=move |_| admins_only.update(|b| *b = !*b)
                                attr:id="admins-only-toggle"
                            >
                                {move || {
                                    if admins_only.get() {
                                        "Admins only: on"
                                    } else {
                                        "Admins only: off"
                                    }
                                }}
                            </Button>
                        }
                    })
                    attr:id="custom-filter-table"
                />
            </Section>

            // Server-owned table: the typed query round-trip
            <Section title="Server-Owned Table (ServerDataTable + on_query_change)">
                <p class="text-sm opacity-70 mb-2">
                    "The table renders whatever the server returned and never sorts or filters "
                    "client-side. Every user change \u{2014} page, debounced search, header sort, "
                    "filter dropdown \u{2014} emits one typed " <code>"TableQuery"</code>
                    " for the caller to re-fetch with. Here a simulated backend serves a 57-row "
                    "fixture; " <code>"filter_options"</code>
                    " supplies population-wide dropdown options so a page window can't lie about "
                    "the population."
                </p>
                <p class="text-xs opacity-60 mb-4">
                    "Last query: "
                    <code data-testid="server-last-query">{move || last_query.get()}</code>
                    " \u{b7} Activated (page-local): "
                    <code data-testid="server-activated-row">
                        {move || {
                            server_activated
                                .get()
                                .map(|i| i.to_string())
                                .unwrap_or_else(|| "(none)".to_string())
                        }}
                    </code>
                    " \u{b7} Inspected: "
                    <code data-testid="server-inspected-row">
                        {move || {
                            server_inspected
                                .get()
                                .map(|i| i.to_string())
                                .unwrap_or_else(|| "(none)".to_string())
                        }}
                    </code>
                </p>
                <ServerDataTable
                    rows=server_rows
                    columns=server_columns
                    current_page=Signal::derive(move || server_page.get())
                    total_count=Signal::derive(move || server_total.get())
                    page_size=10_i64
                    on_page_change=Callback::new(move |_page: i64| {
                        // The typed query fires too; the work happens there.
                    })
                    on_query_change=Callback::new(run_server_query)
                    filter_options=Signal::derive(move || {
                        let all = server_fixture.get_value();
                        HashMap::from([
                            ("role", distinct_values(&all, "role")),
                            ("status", distinct_values(&all, "status")),
                        ])
                    })
                    on_row_activate=Callback::new(move |idx: usize| {
                        server_activated.set(Some(idx));
                    })
                    on_row_inspect=Callback::new(move |idx: usize| {
                        server_inspected.set(Some(idx));
                    })
                    attr:id="server-table"
                />
            </Section>

            // Column resize, typed cells (Badge/Icon), row background, clipboard export
            <Section title="Resizable Columns, Typed Cells, Row Styling, Clipboard Export">
                <p class="text-sm opacity-70 mb-2">
                    "Drag a header's right edge to resize its column (all columns "
                    "are resizable by default). \"Role\" renders an "
                    <code>"Icon"</code>
                    " typed cell, \"Status\" renders a "
                    <code>"Badge"</code>
                    " typed cell, inactive rows get a subtle background via "
                    <code>"row_class_fn"</code>
                    ", and \"Actions\" copies the row (tab-separated, with headers) "
                    "to the clipboard."
                </p>
                <DataTable
                    data=feature_data
                    columns=feature_columns
                    page_size=8
                    typed_cells=vec![role_typed_cell, status_typed_cell]
                    row_class_fn=feature_row_class_fn
                    cell_renderers=vec![copy_row_renderer]
                />
            </Section>

            // widgets::DataTable per-row action column
            <Section title="Widget DataTable: Per-Row Action Column (row_actions)">
                <p class="text-sm opacity-70 mb-2">
                    <code>"widgets::DataTable"</code>
                    " is the simpler row-vector table. "
                    <code>"row_actions"</code>
                    " gives it a trailing action column: the callback receives "
                    <code>"(row_index, row_cells)"</code>
                    " and returns that row's controls, right-aligned on one line. "
                    "Court filings have no telephony route and blocked work "
                    "cannot be completed, so those controls render unavailable "
                    "behind a tooltip \u{2014} focusable rather than natively "
                    <code>"disabled"</code>
                    ", so the explanation is reachable by keyboard too. "
                    "The action column takes no data index: sorting, "
                    <code>"badge_column_keys"</code>
                    " and the leading "
                    <code>"bulk_select"</code>
                    " checkbox are all unaffected \u{2014} click the headers to check."
                </p>
                <p class="text-sm opacity-70 mb-2">
                    "Twelve rows over three pages: sort by Owner descending, page "
                    "to the last page and act on a row \u{2014} the readout shows "
                    "the id and the "
                    <code>"row_index"</code>
                    " the callback received, which is the row's position in the "
                    <code>"rows"</code>
                    " prop rather than its position on the page. Prefer the id for "
                    "anything that identifies a record; the index only tracks the "
                    "snapshot handed to the component."
                </p>
                <WidgetDataTable
                    columns=widget_columns
                    rows=widget_rows
                    page_size=5
                    badge_column_keys=vec!["status"]
                    bulk_select=widget_bulk
                    action_header="Actions"
                    row_actions=widget_row_actions
                />
                <p class="text-sm opacity-70 mt-2">
                    "Last action: "
                    <span class="font-mono">{move || widget_last_action.get()}</span>
                    " \u{2022} selected: "
                    <span class="font-mono">
                        {move || widget_bulk.with(|s| s.len().to_string())}
                    </span>
                </p>
            </Section>
        </ContentLayout>
    }
}
