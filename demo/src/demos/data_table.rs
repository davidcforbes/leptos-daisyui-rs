use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use std::collections::{BTreeSet, HashMap};

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

    // Filterable columns: only the low-cardinality ones (role, department,
    // status) opt in -- a dropdown of 60 distinct names or emails would not be
    // a usable filter, so `name`/`email` stay plain.
    let filterable_columns = RwSignal::new(vec![
        Column::new("name", "Name"),
        Column::new("email", "Email"),
        Column::new("role", "Role").filterable(),
        Column::new("department", "Department").filterable(),
        Column::new("status", "Status").filterable(),
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

    // Runtime localization: columns and texts derived from a locale signal,
    // the pattern a `t()`-based app uses. Toggling the locale must re-render
    // the table chrome (headers, empty state) — asserted by the reactivity
    // suite's `data_table_headers_relocalize_via_dom`.
    let locale_es = RwSignal::new(false);
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
                empty: "No hay datos disponibles".to_string(),
                ..Default::default()
            }
        } else {
            DataTableTexts::default()
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
                    <label class="label">
                        <span class="text-sm">"Rows per page:"</span>
                    </label>
                    <select
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
                />
            </Section>

            // Responsive / auto-growing page size
            <Section title="Responsive Page Size (auto_page_size)">
                <p class="text-sm opacity-70 mb-4">
                    "With " <code>"auto_page_size=true"</code> " the row count is derived from the "
                    "table's rendered height instead of a fixed " <code>"page_size"</code> ", so a "
                    "taller window shows more rows. Drag the resizer below (or resize the window) and "
                    "watch the row count and \"Showing X\u{2013}Y of Z\" caption follow."
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
                        row_range: "Mostrando {start}\u{2013}{end} de {total}".to_string(),
                        filter_all: "Todos".to_string(),
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
                    <kbd class="kbd kbd-xs">"Shift"</kbd> " + Enter selects."
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
                </div>
                <DataTable
                    data=selection_data
                    columns=selection_columns
                    page_size=8
                    selected_rows=activate_selected
                    on_row_activate=Callback::new(move |idx: usize| {
                        activated_row.set(Some(idx));
                        activate_count.update(|n| *n += 1);
                    })
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
        </ContentLayout>
    }
}
