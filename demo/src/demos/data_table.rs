use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::widgets::{DataTable as WidgetDataTable, TableColumn as WidgetTableColumn};
use std::collections::{BTreeSet, HashMap, HashSet};

/// Ten narrow-content columns for the column-track fit fixture (ldui-qsqz).
/// With `declared = None` every column is undeclared (auto track); with
/// `Some(px)` the first two declare that minimum width so the declared
/// tracks cannot fit a 1280px container.
fn fit_columns(declared: Option<u32>) -> Vec<Column> {
    let ids = [
        ("sym", "Sym"),
        ("qty", "Qty"),
        ("cost", "Cost"),
        ("mkt", "Mkt"),
        ("pnl", "P&L"),
        ("delta", "Delta"),
        ("gamma", "Gamma"),
        ("vega", "Vega"),
        ("var", "VaR"),
        ("overdue", "Overdue"),
    ];
    ids.into_iter()
        .enumerate()
        .map(|(index, (id, header))| {
            let column = Column::new(id, header);
            match declared {
                Some(width) if index < 2 => column.with_min_width(width),
                _ => column,
            }
        })
        .collect()
}

fn fit_rows(count: usize) -> Vec<HashMap<&'static str, String>> {
    (0..count)
        .map(|i| {
            let mut row = HashMap::new();
            row.insert("sym", format!("SYM{i:02}"));
            row.insert("qty", format!("{}", 10 + i));
            row.insert("cost", format!("{:.2}", 100.0 + i as f64));
            row.insert("mkt", format!("{:.2}", 101.5 + i as f64));
            row.insert("pnl", format!("{:.2}", 1.5 * i as f64));
            row.insert("delta", format!("{:.2}", 0.5 + 0.01 * i as f64));
            row.insert("gamma", format!("{:.3}", 0.01 * i as f64));
            row.insert("vega", format!("{:.2}", 0.2 * i as f64));
            row.insert("var", format!("{:.2}", 12.0 + i as f64));
            row.insert(
                "overdue",
                if i % 2 == 0 {
                    "no".into()
                } else {
                    "yes".into()
                },
            );
            row
        })
        .collect()
}

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

    // Columns with non-sortable. `id` demonstrates `.identifier()` -- the
    // theme's mono face, applied by the component instead of hand-written
    // `font-mono` (ldui-lrig).
    let mixed_columns = RwSignal::new(vec![
        Column::new("id", "ID").identifier(),
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new_non_sortable("status", "Status"),
        Column::new("joined", "Joined Date"),
    ]);

    // Typed sorting: money / duration / date columns whose display strings do
    // not sort correctly as text ("$1,000" < "$900" because '1' < '9'). The em
    // dash means "not measured" and must not sort as 0. `.numeric()` on
    // "balance"/"days" supplies right-align + tabular-nums presentation AND
    // implies SortAs::Number in one call (ldui-lrig); "opened" is a date, not
    // a number, so it stays on plain `with_sort_as`.
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
        Column::new("balance", "Balance").numeric(),
        Column::new("days", "Days in Stage").numeric(),
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
    // A high-cardinality name uses a substring text box; low-cardinality
    // columns use exact dropdowns. This existing fixture also consumes the
    // locale signal so the browser suite can prove stateful control
    // localization without adding duplicate audited controls.
    let filterable_columns = Signal::derive(move || {
        if locale_es.get() {
            vec![
                Column::new("name", "Nombre").filterable_text(),
                Column::new("email", "Correo"),
                Column::new("role", "Rol").filterable(),
                Column::new("department", "Departamento").filterable(),
                Column::new("status", "Estado").filterable(),
            ]
        } else {
            vec![
                Column::new("name", "Name").filterable_text(),
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
                page_size_label: "Filas por página".to_string(),
                row_range: "Mostrando {start}\u{2013}{end} de {total}".to_string(),
                filter_all: "Todos".to_string(),
                filter_label: "Filtrar por {column}".to_string(),
                // ldui-vooa: the resize separator's accessible name follows
                // this same reactive signal, so toggling the locale updates
                // it without remounting either table.
                resize_column: "Cambiar el ancho de la columna {column}".to_string(),
            }
        } else {
            DataTableTexts::default()
        }
    });
    let localized_text_filter_label = Signal::derive(move || {
        if locale_es.get() {
            "Filtrar {column} por texto".to_owned()
        } else {
            "Filter {column} by text".to_owned()
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

    // Variable-height auto_page_size regression guard (ldui-89rp): 20 rows,
    // every one short except the row at index 12 (id "013") -- deliberately
    // PAST the default `page_size` of 10, so it is absent from the very
    // first render and only shows up once a measurement pass derives a
    // larger page. A tall row inside every plausible page (e.g. index 2)
    // cannot exercise the multi-pass oscillation the CRITICAL fix damps
    // (component.rs's `RowHeightEra` high-water mark): the bug is in what
    // happens when the tall row is revealed *and then excluded again* by
    // successive derived page sizes, not in a single measurement missing it.
    let variable_height_data = RwSignal::new(generate_users(20));
    let variable_height_columns = RwSignal::new(vec![
        Column::new("id", "ID"),
        Column::new("name", "Name"),
        Column::new_non_sortable("notes", "Notes").with_renderer(0),
    ]);
    let tall_row_renderer: CellRenderer = Callback::new(
        move |(_idx, row): (usize, HashMap<&'static str, String>)| {
            if row.get("id").map(String::as_str) == Some("013") {
                view! {
                    <div style="min-height: 108px; display: flex; align-items: center;">
                        "Deliberately tall wrapped content (ldui-89rp): past the default page size, so auto_page_size must both catch it once revealed and never forget it once caught."
                    </div>
                }
                .into_any()
            } else {
                view! { <span>"short"</span> }.into_any()
            }
        },
    );

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
    let server_query = RwSignal::new(TableQuery::first_page(10));
    let server_accept_changes = RwSignal::new(true);
    let server_query_scope = RwSignal::new("dataset-a".to_owned());
    let server_proposal_count = RwSignal::new(0_u32);
    let last_query = RwSignal::new(String::new());
    // Server-variant activation forwarding (ldui-1gp): the same
    // click/dblclick contract as the client table, with page-local indices.
    let server_activated = RwSignal::new(Option::<usize>::None);
    let server_inspected = RwSignal::new(Option::<usize>::None);
    let server_columns = RwSignal::new(vec![
        Column::new("name", "Name").required(),
        Column::new("email", "Email"),
        Column::new("role", "Role").filterable(),
        Column::new("status", "Status").filterable(),
    ]);
    // Opt-in presentation tools (ldui-9j16): the compact gear chooser, an
    // Export toolbar action beside it, and the atomic displayed-slice
    // projection -- the current server page only, never "all rows".
    let server_column_preferences = RwSignal::new(EntityTablePreferences::new(1));
    let server_displayed_slice = RwSignal::new(ServerTableDisplayedSlice::default());
    let server_export_count = RwSignal::new(0_u32);

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
    let propose_server_query = move |mut query: TableQuery| {
        server_proposal_count.update(|count| *count += 1);
        if !server_accept_changes.get_untracked() {
            last_query.set(format!("declined proposal: {query:?}"));
            return;
        }
        // Simulated server normalization: route/query persistence commonly
        // trims and case-folds free text before accepting displayed truth.
        query.search = query.search.trim().to_lowercase();
        server_query.set(query.clone());
        run_server_query(query);
    };
    // Initial fetch: page 1, no query shape.
    run_server_query(server_query.get_untracked());

    // Cursor-owned server table: cursors are opaque to the component. This
    // fixture interprets its own `offset:*` tokens only to simulate a backend.
    let cursor_fixture = StoredValue::new(generate_users(13));
    let cursor_rows = RwSignal::new(Vec::<HashMap<&'static str, String>>::new());
    let cursor_query = RwSignal::new(ServerCursorQuery::first_slice(4));
    let cursor_page = RwSignal::new(ServerCursorPage::default());
    let cursor_loading = RwSignal::new(false);
    let cursor_proposal_count = RwSignal::new(0_u32);
    let cursor_last_request = RwSignal::new(String::new());
    let cursor_keyed_activation = RwSignal::new(Option::<ServerTableRowAction>::None);
    let cursor_keyed_inspection = RwSignal::new(Option::<ServerTableRowAction>::None);
    let cursor_selected_key = RwSignal::new(Option::<String>::None);
    let cursor_accept_selection = RwSignal::new(true);
    let cursor_selection_proposals = RwSignal::new(0_u32);
    let cursor_last_selection_proposal = RwSignal::new(Option::<String>::None);
    let cursor_accept_changes = RwSignal::new(true);
    let cursor_include_analyst_option = RwSignal::new(true);
    let cursor_columns = RwSignal::new(vec![
        Column::new("name", "Name").filterable_text(),
        Column::new("email", "Email"),
        Column::new("role", "Role").filterable(),
    ]);
    let cursor_filter_option_entries = Signal::derive(move || {
        let spanish = locale_es.get();
        let mut roles = vec![
            DataTableFilterOption::new(
                "role.admin",
                if spanish {
                    "Administrador"
                } else {
                    "Administrator"
                },
            ),
            DataTableFilterOption::new(
                "role.developer",
                if spanish {
                    "Desarrollador"
                } else {
                    "Developer"
                },
            ),
            DataTableFilterOption::new(
                "role.designer",
                if spanish { "Diseñador" } else { "Designer" },
            ),
            DataTableFilterOption::new("role.manager", if spanish { "Gerente" } else { "Manager" }),
        ];
        if cursor_include_analyst_option.get() {
            roles.push(DataTableFilterOption::new(
                "role.analyst",
                if spanish { "Analista" } else { "Analyst" },
            ));
        }
        HashMap::from([("role", roles)])
    });
    let run_cursor_query = move |query: ServerCursorQuery| {
        let request_label = match &query.request {
            ServerCursorRequest::First => "First".to_owned(),
            ServerCursorRequest::Previous(cursor) => {
                format!("Previous({})", cursor.as_str())
            }
            ServerCursorRequest::Next(cursor) => format!("Next({})", cursor.as_str()),
        };
        let requested_offset = match &query.request {
            ServerCursorRequest::First => 0,
            ServerCursorRequest::Previous(cursor) | ServerCursorRequest::Next(cursor) => cursor
                .as_str()
                .strip_prefix("offset:")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0),
        };
        let mut items = cursor_fixture.get_value();
        items.retain(|row| {
            query.filters.iter().all(|(column, value)| {
                if *column == "name" {
                    return row
                        .get(column)
                        .is_some_and(|cell| cell.to_lowercase().contains(&value.to_lowercase()));
                }
                let canonical_value = match (*column, value.as_str()) {
                    ("role", "role.admin") => "Admin",
                    ("role", "role.developer") => "Developer",
                    ("role", "role.designer") => "Designer",
                    ("role", "role.manager") => "Manager",
                    ("role", "role.analyst") => "Analyst",
                    _ => value,
                };
                row.get(column).is_some_and(|cell| cell == canonical_value)
            })
        });
        if !query.search.is_empty() {
            let search = query.search.to_lowercase();
            items.retain(|row| {
                row.values()
                    .any(|value| value.to_lowercase().contains(&search))
            });
        }
        if let Some((column, order)) = query.sort {
            items.sort_by(|left, right| {
                let ordering = left.get(column).cmp(&right.get(column));
                match order {
                    SortOrder::Asc => ordering,
                    SortOrder::Desc => ordering.reverse(),
                }
            });
        }
        let size = query.page_size.max(1) as usize;
        let start = requested_offset.min(items.len());
        let previous = (start > 0)
            .then(|| ServerCursorToken::new(format!("offset:{}", start.saturating_sub(size))));
        let next = (start + size < items.len())
            .then(|| ServerCursorToken::new(format!("offset:{}", start + size)));
        cursor_rows.set(items.into_iter().skip(start).take(size).collect());
        cursor_page.set(ServerCursorPage::new(previous, next));
        cursor_loading.set(false);
        cursor_last_request.set(format!(
            "request={request_label} size={} search={:?} sort={:?} filters={:?}",
            query.page_size, query.search, query.sort, query.filters
        ));
    };
    let propose_cursor_query = move |query: ServerCursorQuery| {
        cursor_proposal_count.update(|count| *count += 1);
        if !cursor_accept_changes.get_untracked() {
            cursor_last_request.set(format!("declined filters={:?}", query.filters));
            return;
        }
        cursor_query.set(query.clone());
        run_cursor_query(query);
    };
    run_cursor_query(cursor_query.get_untracked());
    let cursor_pagination = ServerTablePagination::cursor(ServerCursorPagination::controlled(
        cursor_query.into(),
        cursor_page.into(),
        Callback::new(propose_cursor_query),
    ));
    let navigation_only_query = RwSignal::new(ServerCursorQuery::first_slice(4));
    let navigation_only_page = RwSignal::new(ServerCursorPage::new(
        None,
        Some(ServerCursorToken::new("navigation-only-next")),
    ));
    let navigation_only_proposals = RwSignal::new(0_u32);
    let navigation_only_pagination =
        ServerTablePagination::cursor(ServerCursorPagination::controlled(
            navigation_only_query.into(),
            navigation_only_page.into(),
            Callback::new(move |query| {
                navigation_only_proposals.update(|count| *count += 1);
                navigation_only_query.set(query);
            }),
        ));
    let mixed_capability_query = RwSignal::new(ServerCursorQuery::first_slice(4));
    let mixed_capability_page = RwSignal::new(ServerCursorPage::new(
        None,
        Some(ServerCursorToken::new("mixed-next")),
    ));
    let mixed_capability_proposals = RwSignal::new(0_u32);
    let mixed_capability_pagination =
        ServerTablePagination::cursor(ServerCursorPagination::controlled(
            mixed_capability_query.into(),
            mixed_capability_page.into(),
            Callback::new(move |query| {
                mixed_capability_proposals.update(|count| *count += 1);
                mixed_capability_query.set(query);
            }),
        ));
    let current_slice_query = RwSignal::new(ServerCursorQuery::first_slice(4));
    let current_slice_page = RwSignal::new(ServerCursorPage::new(
        None,
        Some(ServerCursorToken::new("current-slice-next")),
    ));
    let current_slice_pagination =
        ServerTablePagination::cursor(ServerCursorPagination::controlled(
            current_slice_query.into(),
            current_slice_page.into(),
            Callback::new(move |query| current_slice_query.set(query)),
        ));
    let conflicting_capability_query =
        RwSignal::new(ServerCursorQuery::first_slice(4).with_search("unsupported supplied search"));
    let conflicting_capability_page = RwSignal::new(ServerCursorPage::default());
    let conflicting_capability_pagination =
        ServerTablePagination::cursor(ServerCursorPagination::controlled(
            conflicting_capability_query.into(),
            conflicting_capability_page.into(),
            Callback::new(|_| {}),
        ));

    // ── Viewport-fit query sizing (ldui-2bt3) ──
    //
    // `viewport_fit=true` measures the rendered height exactly like
    // `DataTable`'s `auto_page_size`, but PROPOSES a page-size query change
    // instead of slicing rows locally: offset resets to page one, cursor
    // resets to `First` (an existing previous/next token was minted for the
    // old size and must never be replayed).
    let viewport_fit_fixture = StoredValue::new(generate_users(48));
    let viewport_fit_rows = RwSignal::new(Vec::<HashMap<&'static str, String>>::new());
    let viewport_fit_page = RwSignal::new(1_i64);
    let viewport_fit_total = RwSignal::new(0_i64);
    let viewport_fit_query = RwSignal::new(TableQuery::first_page(5));
    let viewport_fit_accept = RwSignal::new(true);
    let viewport_fit_proposals = RwSignal::new(0_u32);
    let viewport_fit_last_query = RwSignal::new(String::new());
    let viewport_fit_columns = RwSignal::new(vec![
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new("role", "Role"),
    ]);
    // Own-induced refetch across DIFFERING row heights (ldui-2bt3 CRITICAL
    // fix): row "009" (0-based index 8) renders as a deliberately tall
    // wrapped cell, same pattern as the client `auto_page_size` regression
    // guard above. Growing the offset table's container is guaranteed to
    // eventually accept a page size that includes it, proposing a shrink;
    // the shrunk (short-only) page must not propose growing again forever.
    let viewport_fit_offset_columns = RwSignal::new(vec![
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new_non_sortable("role", "Role").with_renderer(0),
    ]);
    let viewport_fit_tall_row_renderer: CellRenderer =
        Callback::new(move |(_idx, row): (usize, HashMap<&'static str, String>)| {
            if row.get("id").map(String::as_str) == Some("009") {
                view! {
                    <div style="min-height: 108px; display: flex; align-items: center;">
                        "Deliberately tall wrapped content (ldui-2bt3): a viewport-fit \
                         proposal's own-induced refetch must carry its row-height memory \
                         forward instead of forgetting this row once a shrunk page \
                         excludes it again."
                    </div>
                }
                .into_any()
            } else {
                view! { <span>{move || row.get("role").cloned().unwrap_or_default()}</span> }
                    .into_any()
            }
        });
    let run_viewport_fit_query = move |q: TableQuery| {
        let items = viewport_fit_fixture.get_value();
        viewport_fit_total.set(items.len() as i64);
        let start = ((q.page - 1) * q.page_size).max(0) as usize;
        viewport_fit_rows.set(
            items
                .into_iter()
                .skip(start)
                .take(q.page_size.max(0) as usize)
                .collect(),
        );
        viewport_fit_page.set(q.page);
        viewport_fit_last_query.set(format!("page={} size={}", q.page, q.page_size));
    };
    let propose_viewport_fit_query = move |query: TableQuery| {
        viewport_fit_proposals.update(|count| *count += 1);
        if !viewport_fit_accept.get_untracked() {
            viewport_fit_last_query.set(format!(
                "declined: page={} size={}",
                query.page, query.page_size
            ));
            return;
        }
        viewport_fit_query.set(query.clone());
        run_viewport_fit_query(query);
    };
    run_viewport_fit_query(viewport_fit_query.get_untracked());

    // Cursor variant: same measurement, but the query owns opaque tokens.
    let viewport_fit_cursor_fixture = StoredValue::new(generate_users(30));
    let viewport_fit_cursor_rows = RwSignal::new(Vec::<HashMap<&'static str, String>>::new());
    let viewport_fit_cursor_query = RwSignal::new(ServerCursorQuery::first_slice(5));
    let viewport_fit_cursor_page = RwSignal::new(ServerCursorPage::default());
    let viewport_fit_cursor_proposals = RwSignal::new(0_u32);
    let viewport_fit_cursor_last_request = RwSignal::new(String::new());
    let run_viewport_fit_cursor_query = move |query: ServerCursorQuery| {
        let requested_offset = match &query.request {
            ServerCursorRequest::First => 0,
            ServerCursorRequest::Previous(cursor) | ServerCursorRequest::Next(cursor) => cursor
                .as_str()
                .strip_prefix("offset:")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0),
        };
        let items = viewport_fit_cursor_fixture.get_value();
        let size = query.page_size.max(1) as usize;
        let start = requested_offset.min(items.len());
        let previous = (start > 0)
            .then(|| ServerCursorToken::new(format!("offset:{}", start.saturating_sub(size))));
        let next = (start + size < items.len())
            .then(|| ServerCursorToken::new(format!("offset:{}", start + size)));
        viewport_fit_cursor_rows.set(items.into_iter().skip(start).take(size).collect());
        viewport_fit_cursor_page.set(ServerCursorPage::new(previous, next));
        let request_label = match &query.request {
            ServerCursorRequest::First => "First".to_owned(),
            ServerCursorRequest::Previous(cursor) => format!("Previous({})", cursor.as_str()),
            ServerCursorRequest::Next(cursor) => format!("Next({})", cursor.as_str()),
        };
        viewport_fit_cursor_last_request
            .set(format!("request={request_label} size={}", query.page_size));
    };
    let propose_viewport_fit_cursor_query = move |query: ServerCursorQuery| {
        viewport_fit_cursor_proposals.update(|count| *count += 1);
        viewport_fit_cursor_query.set(query.clone());
        run_viewport_fit_cursor_query(query);
    };
    run_viewport_fit_cursor_query(viewport_fit_cursor_query.get_untracked());
    let viewport_fit_cursor_pagination =
        ServerTablePagination::cursor(ServerCursorPagination::controlled(
            viewport_fit_cursor_query.into(),
            viewport_fit_cursor_page.into(),
            Callback::new(propose_viewport_fit_cursor_query),
        ));

    // Fail-closed fixture: viewport_fit requested against a fixed-slice
    // (navigation-only) endpoint, which cannot accept page-size changes.
    let viewport_fit_rejected_query = RwSignal::new(ServerCursorQuery::first_slice(4));
    let viewport_fit_rejected_page = RwSignal::new(ServerCursorPage::new(
        None,
        Some(ServerCursorToken::new("rejected-next")),
    ));
    let viewport_fit_rejected_pagination =
        ServerTablePagination::cursor(ServerCursorPagination::controlled(
            viewport_fit_rejected_query.into(),
            viewport_fit_rejected_page.into(),
            Callback::new(move |query| viewport_fit_rejected_query.set(query)),
        ));

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

    // ---- ldui-px06: controlled checkbox multi-selection over a server slice ----
    //
    // A cursor-backed dataset paged three rows at a time. Everything the
    // component shows is derived from `multi_accepted`, which this fixture
    // owns: the table proposes, the fixture decides. "Reject proposals" makes
    // that separation visible -- every gesture still emits, and nothing moves.
    let multi_population = RwSignal::new(vec![
        ("conv-1", "Ticket 1 \u{2014} billing dispute", "Open"),
        ("conv-2", "Ticket 2 \u{2014} refund request", "Open"),
        ("conv-3", "Ticket 3 \u{2014} address change", "Archived"),
        ("conv-4", "Ticket 4 \u{2014} contract review", "Open"),
        ("conv-5", "Ticket 5 \u{2014} escalation", "Open"),
        ("conv-6", "Ticket 6 \u{2014} intake", "Archived"),
    ]);
    let multi_page_index = RwSignal::new(0_usize);
    let multi_open_only = RwSignal::new(false);
    let multi_accepted = RwSignal::new(BTreeSet::<String>::new());
    let multi_accept_proposals = RwSignal::new(true);
    let multi_proposal_count = RwSignal::new(0_u32);
    let multi_last_cause = RwSignal::new("(none)".to_owned());
    let multi_last_scope = RwSignal::new("(none)".to_owned());
    let multi_scope = RwSignal::new("conversations/v1".to_owned());

    const MULTI_PAGE_SIZE: usize = 3;
    let multi_matching = Memo::new(move |_| {
        let open_only = multi_open_only.get();
        multi_population.with(|population| {
            population
                .iter()
                .filter(|(_, _, status)| !open_only || *status == "Open")
                .copied()
                .collect::<Vec<_>>()
        })
    });
    let multi_rows = Memo::new(move |_| {
        let page = multi_page_index.get();
        multi_matching.with(|matching| {
            matching
                .iter()
                .skip(page * MULTI_PAGE_SIZE)
                .take(MULTI_PAGE_SIZE)
                .map(|(id, subject, status)| {
                    HashMap::from([
                        ("id", (*id).to_owned()),
                        ("subject", (*subject).to_owned()),
                        ("status", (*status).to_owned()),
                    ])
                })
                .collect::<Vec<TableRow>>()
        })
    });
    let multi_columns = RwSignal::new(vec![
        Column::new_non_sortable("subject", "Subject"),
        Column::new_non_sortable("status", "Status").non_resizable(),
    ]);
    let multi_cursor_query = RwSignal::new(ServerCursorQuery::first_slice(MULTI_PAGE_SIZE as i64));
    let multi_pagination = ServerTablePagination::cursor(ServerCursorPagination::controlled(
        multi_cursor_query.into(),
        Signal::derive(move || {
            let page = multi_page_index.get();
            let total = multi_matching.with(Vec::len);
            ServerCursorPage::new(
                (page > 0).then(|| ServerCursorToken::from(format!("page-{}", page - 1))),
                ((page + 1) * MULTI_PAGE_SIZE < total)
                    .then(|| ServerCursorToken::from(format!("page-{}", page + 1))),
            )
        }),
        Callback::new(move |query: ServerCursorQuery| {
            match &query.request {
                ServerCursorRequest::First => multi_page_index.set(0),
                ServerCursorRequest::Previous(_) => {
                    multi_page_index.update(|page| *page = page.saturating_sub(1));
                }
                ServerCursorRequest::Next(_) => multi_page_index.update(|page| *page += 1),
            }
            multi_cursor_query.set(query);
        }),
    ));

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

            // ldui-qsqz: ten columns that declare no width must fit a 1280px
            // container (they used to get 160px tracks each: 1600px, wider
            // than a 1440px viewport). Browser proof: tests/data_table_fit_smoke.rs.
            <Section title="Column-track fit -- ten undeclared columns fit a 1280px container (ldui-qsqz)">
                <p class="text-sm opacity-70 mb-4">
                    "No column declares a width, so every track is auto and the table fits w-full."
                </p>
                <div class="w-full max-w-7xl" data-testid="data-table-fit-undeclared">
                    <DataTable
                        data=RwSignal::new(fit_rows(6))
                        columns=RwSignal::new(fit_columns(None))
                        column_chooser=true
                        page_size=10
                    />
                </div>
                <p class="text-sm opacity-70 mb-4 mt-6">
                    "Two columns declare 400px minimums, so the declared tracks cannot fit and the wrapper scrolls instead of spilling."
                </p>
                <div class="w-full max-w-7xl" data-testid="data-table-fit-declared">
                    <DataTable
                        data=RwSignal::new(fit_rows(6))
                        columns=RwSignal::new(fit_columns(Some(400)))
                        column_chooser=true
                        page_size=10
                    />
                </div>
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
                    <code class="font-sans">"Column::filterable()"</code>
                    " gives low-cardinality columns an exact "
                    "dropdown; " <code class="font-sans">"Column::filterable_text()"</code>
                    " gives high-cardinality "
                    "columns a debounced substring box in the same aligned row. Filters combine with "
                    "each other and the table search (all must match)."
                </p>
                <DataTable
                    data=Signal::derive(move || generate_users(60))
                    columns=filterable_columns
                    page_size=8
                    searchable=true
                    texts=localized_texts
                    text_filter_label=localized_text_filter_label
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

            // Regression guard for ldui-89rp
            <Section title="Responsive Page Size — Variable-Height Rows">
                <p class="text-sm opacity-70 mb-4">
                    "A regression guard for " <code class="font-sans">"ldui-89rp"</code>
                    ": row 13 of 20 wraps to roughly three lines while every other row stays "
                    "short, and it sits past the default page size so it only shows up once a "
                    "measurement pass grows the page. "
                    <code class="font-sans">"auto_page_size"</code>
                    " must measure the tallest currently rendered row rather than just the "
                    "first, never forget it once seen, and settle instead of oscillating -- "
                    "so the derived page never overflows its own scroll wrapper."
                </p>
                <div class="resize-y overflow-auto border border-base-300 rounded-lg p-3 h-96 min-h-32">
                    <DataTable
                        data=variable_height_data
                        columns=variable_height_columns
                        cell_renderers=vec![tall_row_renderer]
                        auto_page_size=true
                        max_height="100%"
                        attr:id="auto-page-variable-height-table"
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
                        page_size_label: "Filas por página".to_string(),
                        row_range: "Mostrando {start}\u{2013}{end} de {total}".to_string(),
                        filter_all: "Todos".to_string(),
                        filter_label: "Filtrar por {column}".to_string(),
                        resize_column: "Cambiar el ancho de la columna {column}".to_string(),
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
                    " \u{b7} Displayed slice (this page only): "
                    <code class="font-sans" data-testid="server-displayed-slice-rows">
                        {move || server_displayed_slice.get().rows.len().to_string()}
                    </code>
                    " rows \u{b7} Exports clicked: "
                    <code class="font-sans" data-testid="server-export-count">
                        {move || server_export_count.get().to_string()}
                    </code>
                </p>
                <div class="mb-3 flex flex-wrap items-center gap-2" data-testid="server-query-controls">
                    <Button
                        on:click=move |_| {
                            let reset = TableQuery::first_page(
                                server_query.get_untracked().page_size,
                            );
                            server_query.set(reset.clone());
                            run_server_query(reset);
                        }
                        attr:data-testid="server-query-reset"
                    >
                        "Reset query"
                    </Button>
                    <Button
                        on:click=move |_| {
                            server_accept_changes.update(|accept| *accept = !*accept)
                        }
                        attr:data-testid="server-query-accept"
                    >
                        {move || if server_accept_changes.get() {
                            "Reject proposals"
                        } else {
                            "Accept proposals"
                        }}
                    </Button>
                    <Button
                        on:click=move |_| {
                            server_query_scope.update(|scope| {
                                *scope = if scope == "dataset-a" {
                                    "dataset-b".to_owned()
                                } else {
                                    "dataset-a".to_owned()
                                }
                            })
                        }
                        attr:data-testid="server-query-scope"
                    >
                        "Change dataset/access scope"
                    </Button>
                    <span class="text-xs">
                        "Proposals: "
                        <code class="font-sans" data-testid="server-query-proposals">
                            {move || server_proposal_count.get().to_string()}
                        </code>
                    </span>
                </div>
                <ServerDataTable
                    rows=server_rows
                    columns=server_columns
                    current_page=Signal::derive(move || server_page.get())
                    total_count=Signal::derive(move || server_total.get())
                    page_size=Signal::derive(move || server_query.get().page_size)
                    on_page_change=Callback::new(move |_page: i64| {
                        // Controlled query ownership carries the full proposal.
                    })
                    query_ownership=ServerTableQueryOwnership::controlled(
                        server_query.into(),
                        Callback::new(propose_server_query),
                    )
                    query_reset_key=server_query_scope
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
                    texts=localized_texts
                    sort_texts=localized_sort_texts
                    column_tools=ServerTableColumnTools::new(
                        EntityTablePreferenceOwnership::controlled(
                            server_column_preferences.into(),
                            Callback::new(move |next| server_column_preferences.set(next)),
                        ),
                        1,
                    )
                        .with_chooser_trigger(EntityColumnChooserTrigger::Icon)
                        .with_toolbar_actions(move || {
                            view! {
                                <Button
                                    class="btn-ghost btn-xs"
                                    attr:data-testid="server-export-slice"
                                    on_click=Callback::new(move |_| {
                                        server_export_count.update(|count| *count += 1);
                                    })
                                >
                                    "Export this page"
                                </Button>
                            }
                                .into_any()
                        })
                    on_displayed_slice=Callback::new(move |slice: ServerTableDisplayedSlice| {
                        server_displayed_slice.set(slice);
                    })
                    attr:id="server-table"
                />
            </Section>

            <Section title="Cursor-Owned Server Table">
                <p class="text-sm opacity-70 mb-2">
                    "This table has opaque Previous/Next cursors and no population total or "
                    "fabricated page number. Query-shape changes restart from the first cursor."
                </p>
                <p class="text-xs opacity-60 mb-3">
                    "Last query: "
                    <code class="font-sans" data-testid="cursor-last-query">
                        {move || cursor_last_request.get()}
                    </code>
                    " · Proposals: "
                    <code class="font-sans" data-testid="cursor-query-proposals">
                        {move || cursor_proposal_count.get().to_string()}
                    </code>
                    " · Keyed activation: "
                    <code class="font-sans" data-testid="cursor-keyed-activation">
                        {move || cursor_keyed_activation.get().map_or_else(
                            || "(none)".to_owned(),
                            |action| format!(
                                "{}|{}|{}",
                                action.key,
                                action.page_index,
                                action.row.get("name").cloned().unwrap_or_default(),
                            ),
                        )}
                    </code>
                    " · Keyed inspection: "
                    <code class="font-sans" data-testid="cursor-keyed-inspection">
                        {move || cursor_keyed_inspection.get().map_or_else(
                            || "(none)".to_owned(),
                            |action| format!(
                                "{}|{}|{}",
                                action.key,
                                action.page_index,
                                action.row.get("name").cloned().unwrap_or_default(),
                            ),
                        )}
                    </code>
                    " · Selected: "
                    <code class="font-sans" data-testid="cursor-selected-key">
                        {move || cursor_selected_key.get().unwrap_or_else(|| "(none)".to_owned())}
                    </code>
                    " · Selection proposals: "
                    <code class="font-sans" data-testid="cursor-selection-proposals">
                        {move || cursor_selection_proposals.get().to_string()}
                    </code>
                    " · Last selection proposal: "
                    <code class="font-sans" data-testid="cursor-last-selection-proposal">
                        {move || cursor_last_selection_proposal.get().unwrap_or_else(|| "(none)".to_owned())}
                    </code>
                </p>
                <div class="mb-3 flex flex-wrap gap-2">
                    <Button
                        on:click=move |_| cursor_accept_changes.update(|accept| *accept = !*accept)
                        attr:data-testid="cursor-query-accept"
                    >
                        {move || if cursor_accept_changes.get() {
                            "Reject cursor proposals"
                        } else {
                            "Accept cursor proposals"
                        }}
                    </Button>
                    <Button
                        on:click=move |_| locale_es.update(|spanish| *spanish = !*spanish)
                        attr:data-testid="cursor-filter-locale"
                    >
                        "Toggle filter labels"
                    </Button>
                    <Button
                        on:click=move |_| {
                            cursor_include_analyst_option.update(|include| *include = !*include)
                        }
                        attr:data-testid="cursor-filter-active-option"
                    >
                        "Toggle active option metadata"
                    </Button>
                    <Button
                        on:click=move |_| cursor_accept_selection.update(|accept| *accept = !*accept)
                        attr:data-testid="cursor-selection-accept"
                    >
                        {move || if cursor_accept_selection.get() {
                            "Reject selection proposals"
                        } else {
                            "Accept selection proposals"
                        }}
                    </Button>
                    <Button
                        on:click=move |_| cursor_rows.update(|rows| rows.reverse())
                        attr:data-testid="cursor-reverse-rows"
                    >
                        "Reverse displayed rows"
                    </Button>
                    <Button
                        on:click=move |_| {
                            cursor_rows.update(|rows| {
                                if rows.iter().all(|row| row.get("id").is_none_or(|id| id != "inserted")) {
                                    rows.insert(0, HashMap::from([
                                        ("id", "inserted".to_owned()),
                                        ("name", "Inserted row".to_owned()),
                                        ("email", "inserted@example.com".to_owned()),
                                        ("role", "Analyst".to_owned()),
                                        ("department", "Finance".to_owned()),
                                        ("status", "Active".to_owned()),
                                        ("joined", "2026-08-29".to_owned()),
                                    ]));
                                }
                            })
                        }
                        attr:data-testid="cursor-insert-row"
                    >
                        "Insert displayed row"
                    </Button>
                    <Button
                        on:click=move |_| {
                            cursor_rows.update(|rows| {
                                rows.retain(|row| row.get("id").is_none_or(|id| id != "inserted"));
                            })
                        }
                        attr:data-testid="cursor-remove-inserted-row"
                    >
                        "Remove inserted row"
                    </Button>
                    <Button
                        on:click=move |_| {
                            cursor_rows.update(|rows| {
                                if let Some(first) = rows.first().cloned() {
                                    rows.push(first);
                                }
                            })
                        }
                        attr:data-testid="cursor-duplicate-row-key"
                    >
                        "Duplicate displayed key"
                    </Button>
                    <Button
                        on:click=move |_| run_cursor_query(cursor_query.get_untracked())
                        attr:data-testid="cursor-restore-rows"
                    >
                        "Restore server slice"
                    </Button>
                    <Button
                        on:click=move |_| {
                            cursor_page.update(|page| {
                                page.state = ServerCursorSliceState::RetainedWhileLoading;
                            });
                            cursor_loading.set(true);
                        }
                        attr:data-testid="cursor-retain-loading"
                    >
                        "Retain while loading"
                    </Button>
                    <Button
                        on:click=move |_| {
                            cursor_page.update(|page| {
                                page.state = ServerCursorSliceState::RetainedAfterFailure;
                            });
                            cursor_loading.set(false);
                        }
                        attr:data-testid="cursor-retain-failure"
                    >
                        "Retain after failure"
                    </Button>
                    <Button
                        on:click=move |_| {
                            cursor_page.update(|page| {
                                page.state = ServerCursorSliceState::Current;
                            });
                            cursor_loading.set(false);
                        }
                        attr:data-testid="cursor-current"
                    >
                        "Mark current"
                    </Button>
                </div>
                <ServerDataTable
                    rows=cursor_rows
                    columns=cursor_columns
                    pagination=cursor_pagination
                    loading=cursor_loading
                    page_size_options=Signal::stored(vec![4_i64, 8_i64, 13_i64])
                    filter_option_entries=cursor_filter_option_entries
                    row_key=Callback::new(|row: TableRow| {
                        row.get("id").cloned().unwrap_or_default()
                    })
                    selection=ServerTableSelection::controlled(
                        cursor_selected_key.into(),
                        Callback::new(move |proposed: Option<String>| {
                            cursor_selection_proposals.update(|count| *count += 1);
                            cursor_last_selection_proposal.set(proposed.clone());
                            if cursor_accept_selection.get_untracked() {
                                cursor_selected_key.set(proposed);
                            }
                        }),
                    )
                    on_row_activate_keyed=Callback::new(move |action| {
                        cursor_keyed_activation.set(Some(action));
                    })
                    on_row_inspect_keyed=Callback::new(move |action| {
                        cursor_keyed_inspection.set(Some(action));
                    })
                    attr:id="cursor-server-table"
                />
                <div class="mt-6 grid gap-4 xl:grid-cols-2">
                    <div>
                        <h3 class="font-semibold">"Fixed-slice navigation only"</h3>
                        <p class="text-xs opacity-60">
                            "Proposals: "
                            <code class="font-sans" data-testid="cursor-navigation-only-proposals">
                                {move || navigation_only_proposals.get().to_string()}
                            </code>
                        </p>
                        <ServerDataTable
                            rows=cursor_rows
                            columns=cursor_columns
                            pagination=navigation_only_pagination
                            query_capabilities=ServerQueryCapabilities::navigation_only()
                            attr:id="cursor-navigation-only-table"
                        />
                    </div>
                    <div>
                        <h3 class="font-semibold">"Mixed: search and sort"</h3>
                        <p class="text-xs opacity-60">
                            "Proposals: "
                            <code class="font-sans" data-testid="cursor-mixed-capability-proposals">
                                {move || mixed_capability_proposals.get().to_string()}
                            </code>
                        </p>
                        <ServerDataTable
                            rows=cursor_rows
                            columns=cursor_columns
                            pagination=mixed_capability_pagination
                            query_capabilities=ServerQueryCapabilities::navigation_only()
                                .with_search(true)
                                .with_sorting(true)
                            attr:id="cursor-mixed-capability-table"
                        />
                    </div>
                </div>
                <div class="mt-6 grid gap-4 xl:grid-cols-2">
                    <div id="cursor-current-slice-vocabulary">
                        <h3 class="font-semibold">"Explicit current-slice vocabulary"</h3>
                        <ServerDataTable
                            rows=cursor_rows
                            columns=cursor_columns
                            pagination=current_slice_pagination
                            query_capabilities=ServerQueryCapabilities::navigation_only()
                                .with_filtering(true)
                            filter_vocabulary=ServerFilterVocabulary::current_slice(
                                Signal::stored(ServerCurrentSliceFilterTexts::default()),
                            )
                        />
                    </div>
                    <div id="cursor-missing-vocabulary">
                        <h3 class="font-semibold">"Rejected ambiguous vocabulary"</h3>
                        <ServerDataTable
                            rows=cursor_rows
                            columns=cursor_columns
                            pagination=current_slice_pagination
                            query_capabilities=ServerQueryCapabilities::navigation_only()
                                .with_filtering(true)
                        />
                    </div>
                </div>
                <div class="mt-3" id="cursor-capability-conflict">
                    <ServerDataTable
                        rows=cursor_rows
                        columns=cursor_columns
                        pagination=conflicting_capability_pagination
                        query_capabilities=ServerQueryCapabilities::navigation_only()
                    />
                </div>
                <div class="mt-3" id="cursor-mixed-config">
                    <ServerDataTable
                        rows=cursor_rows
                        columns=cursor_columns
                        pagination=cursor_pagination
                        current_page=Signal::stored(1_i64)
                        total_count=Signal::stored(13_i64)
                        page_size=Signal::stored(4_i64)
                        on_page_change=Callback::new(|_| {})
                    />
                </div>
            </Section>

            <Section title="Viewport-Fit Query Sizing (Server)">
                <p class="text-sm opacity-70 mb-4">
                    "With " <code class="font-sans">"viewport_fit=true"</code> " a "
                    <code class="font-sans">"ServerDataTable"</code>
                    " measures its rendered height exactly like "
                    <code class="font-sans">"DataTable"</code>"'s "
                    <code class="font-sans">"auto_page_size"</code>
                    ", but PROPOSES a page-size query change instead of slicing rows "
                    "locally: offset queries reset to page one, cursor queries reset "
                    "to " <code class="font-sans">"First"</code>
                    " (an existing previous/next token was minted for the old size "
                    "and is never replayed against a new one)."
                </p>
                <div class="alert alert-info mb-4">
                    <span>
                        "Needs an endpoint that accepts page-size changes -- a "
                        "fixed-slice cursor endpoint or a disabled page-size "
                        "capability rejects the policy visibly instead of silently "
                        "ignoring it (see the rejected fixture below)."
                    </span>
                </div>
                <p class="text-xs opacity-60 mb-3">
                    "Offset -- last query: "
                    <code class="font-sans" data-testid="viewport-fit-last-query">
                        {move || viewport_fit_last_query.get()}
                    </code>
                    " · Proposals: "
                    <code class="font-sans" data-testid="viewport-fit-proposals">
                        {move || viewport_fit_proposals.get().to_string()}
                    </code>
                </p>
                <div class="mb-3 flex flex-wrap gap-2">
                    <Button
                        on:click=move |_| viewport_fit_accept.update(|accept| *accept = !*accept)
                        attr:data-testid="viewport-fit-accept"
                    >
                        {move || if viewport_fit_accept.get() {
                            "Reject proposals"
                        } else {
                            "Accept proposals"
                        }}
                    </Button>
                </div>
                // `resize-y` + `overflow-auto`, same as the DataTable auto_page_size
                // demo above: user-resizable so the ResizeObserver can be exercised
                // without resizing the browser window itself.
                <div class="resize-y overflow-auto border border-base-300 rounded-lg p-3 h-96 min-h-32">
                    <ServerDataTable
                        rows=viewport_fit_rows
                        columns=viewport_fit_offset_columns
                        cell_renderers=vec![viewport_fit_tall_row_renderer]
                        current_page=Signal::derive(move || viewport_fit_page.get())
                        total_count=Signal::derive(move || viewport_fit_total.get())
                        page_size=Signal::derive(move || viewport_fit_query.get().page_size)
                        on_page_change=Callback::new(move |_page: i64| {})
                        query_ownership=ServerTableQueryOwnership::controlled(
                            viewport_fit_query.into(),
                            Callback::new(propose_viewport_fit_query),
                        )
                        viewport_fit=true
                        viewport_fit_min_rows=3_usize
                        max_height="100%"
                        attr:id="viewport-fit-offset-server-table"
                    />
                </div>

                <p class="text-xs opacity-60 mb-3 mt-6">
                    "Cursor -- last query: "
                    <code class="font-sans" data-testid="viewport-fit-cursor-last-query">
                        {move || viewport_fit_cursor_last_request.get()}
                    </code>
                    " · Proposals: "
                    <code class="font-sans" data-testid="viewport-fit-cursor-proposals">
                        {move || viewport_fit_cursor_proposals.get().to_string()}
                    </code>
                </p>
                <div class="resize-y overflow-auto border border-base-300 rounded-lg p-3 h-96 min-h-32">
                    <ServerDataTable
                        rows=viewport_fit_cursor_rows
                        columns=viewport_fit_columns
                        pagination=viewport_fit_cursor_pagination
                        viewport_fit=true
                        viewport_fit_min_rows=3_usize
                        max_height="100%"
                        attr:id="viewport-fit-cursor-server-table"
                    />
                </div>

                <div class="mt-6" id="viewport-fit-rejected-table">
                    <h3 class="font-semibold">"Rejected: fixed-slice endpoint"</h3>
                    <ServerDataTable
                        rows=viewport_fit_cursor_rows
                        columns=viewport_fit_columns
                        pagination=viewport_fit_rejected_pagination
                        query_capabilities=ServerQueryCapabilities::navigation_only()
                        viewport_fit=true
                    />
                </div>
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

            // ldui-px06
            <Section title="Server Slice Multi-Selection (controlled checkboxes)">
                <p class="text-sm text-base-content/75 mb-2">
                    "A cursor-backed dataset paged three rows at a time. The header "
                    "checkbox means "
                    <strong>"the rows on this page"</strong>
                    " and nothing else: it is checked only when every selectable "
                    "displayed row is accepted, and indeterminate only when some of "
                    "them are. Keys accepted on other pages never tint it \u{2014} they "
                    "are reported separately, below the toolbar, and they survive "
                    "paging because every proposal is a complete key set that carries "
                    "them through untouched."
                </p>
                <p class="text-sm text-base-content/75 mb-3">
                    "Archived conversations are not selectable and say why; their "
                    "checkboxes stay focusable so the reason is reachable by keyboard. "
                    "Accepted truth is this page's own signal \u{2014} switch to "
                    "\"Reject proposals\" and every gesture still emits while nothing "
                    "moves."
                </p>
                <p class="text-sm text-base-content/75 mb-3">
                    "Accepted: "
                    <code class="font-sans" data-testid="multi-accepted-keys">
                        {move || multi_accepted.with(|keys| {
                            if keys.is_empty() {
                                "(none)".to_owned()
                            } else {
                                keys.iter().cloned().collect::<Vec<_>>().join(",")
                            }
                        })}
                    </code>
                    " \u{00b7} Proposals: "
                    <code class="font-sans" data-testid="multi-proposal-count">
                        {move || multi_proposal_count.get().to_string()}
                    </code>
                    " \u{00b7} Last cause: "
                    <code class="font-sans" data-testid="multi-last-cause">
                        {move || multi_last_cause.get()}
                    </code>
                    " \u{00b7} Proposal scope: "
                    <code class="font-sans" data-testid="multi-last-scope">
                        {move || multi_last_scope.get()}
                    </code>
                </p>
                <div class="mb-3 flex flex-wrap gap-2">
                    <Button
                        on:click=move |_| {
                            multi_accept_proposals.update(|accept| *accept = !*accept)
                        }
                        attr:data-testid="multi-accept-toggle"
                    >
                        {move || if multi_accept_proposals.get() {
                            "Reject proposals"
                        } else {
                            "Accept proposals"
                        }}
                    </Button>
                    <Button
                        on:click=move |_| {
                            multi_open_only.update(|open_only| *open_only = !*open_only);
                            multi_page_index.set(0);
                        }
                        attr:data-testid="multi-filter-toggle"
                    >
                        {move || if multi_open_only.get() {
                            "Show all conversations"
                        } else {
                            "Filter to open only"
                        }}
                    </Button>
                    <Button
                        on:click=move |_| {
                            multi_population.update(|population| {
                                population.retain(|(id, _, _)| *id != "conv-1");
                            });
                        }
                        attr:data-testid="multi-remove-row"
                    >
                        "Remove conv-1 server-side"
                    </Button>
                    <Button
                        on:click=move |_| {
                            // A dataset change is an ATOMIC caller action:
                            // move the scope and clear the accepted set in the
                            // same update, so no key can be relabelled.
                            multi_scope.set("conversations/v2".to_owned());
                            multi_accepted.set(BTreeSet::new());
                            multi_page_index.set(0);
                        }
                        attr:data-testid="multi-change-scope"
                    >
                        "Switch dataset scope"
                    </Button>
                </div>
                <ServerDataTable
                    rows=multi_rows
                    columns=multi_columns
                    pagination=multi_pagination
                    query_capabilities=ServerQueryCapabilities::navigation_only()
                    row_key=Callback::new(|row: TableRow| {
                        row.get("id").cloned().unwrap_or_default()
                    })
                    multi_selection=ServerTableMultiSelection::controlled(
                        multi_accepted.into(),
                        Callback::new(move |proposal: ServerTableSelectionProposal| {
                            multi_proposal_count.update(|count| *count += 1);
                            multi_last_scope.set(proposal.scope.clone());
                            multi_last_cause.set(match &proposal.cause {
                                ServerTableSelectionCause::Row { key, selected } => {
                                    format!("row:{key}:{selected}")
                                }
                                ServerTableSelectionCause::CurrentSlice { selected, keys } => {
                                    format!("slice:{}:{selected}", keys.len())
                                }
                            });
                            // Stale-scope proposals are refused outright, which
                            // is the whole point of stamping them.
                            if proposal.scope != multi_scope.get_untracked() {
                                return;
                            }
                            if multi_accept_proposals.get_untracked() {
                                multi_accepted.set(proposal.keys);
                            }
                        }),
                    )
                        .with_scope(multi_scope.into())
                        .with_row_label(Callback::new(|row: TableRow| {
                            row.get("subject").cloned().unwrap_or_default()
                        }))
                        .with_row_selectable(Callback::new(|row: TableRow| {
                            if row.get("status").is_some_and(|status| status == "Archived") {
                                ServerTableRowSelectability::blocked(
                                    "Archived conversations cannot be reassigned",
                                )
                            } else {
                                ServerTableRowSelectability::Selectable
                            }
                        }))
                    // A caller-supplied stable identity prefix (ldui-j6sh).
                    // Every framework-owned control below derives its
                    // `id`/`name` from it, so the row checkboxes are
                    // addressable as `conversations-select-row-conv_2d1`
                    // rather than by slice position -- and stay so across
                    // paging, filtering and the dataset-scope switch.
                    control_id="conversations"
                    attr:id="server-multi-select-table"
                />
            </Section>
        </ContentLayout>
    }
}
