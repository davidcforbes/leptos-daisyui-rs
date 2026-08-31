use crate::components::badge::Badge;
use crate::components::data_table::selection::{
    click_swallowed_by_inspect, key_inspects, row_aria_selected,
};
use crate::components::data_table::types::{
    CellRenderer, Column, DataTableTexts, RowDetailRenderer, TableRow, TypedCell, TypedCellFn,
};
use crate::components::icon::Icon;
use crate::merge_classes;
use leptos::prelude::*;
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use web_sys::wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq, Eq)]
enum StableRowKeyError {
    Empty {
        index: usize,
    },
    Duplicate {
        key: String,
        first_index: usize,
        duplicate_index: usize,
    },
}

impl fmt::Display for StableRowKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty { index } => write!(
                formatter,
                "DataTable row_key returned an empty key for page row {index}"
            ),
            Self::Duplicate {
                key,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "DataTable row_key returned duplicate key {key:?} for page rows {first_index} and {duplicate_index}"
            ),
        }
    }
}

fn validate_stable_row_keys(
    rows: &[(usize, TableRow)],
    key_of: impl Fn(&TableRow) -> String,
) -> Result<Vec<String>, StableRowKeyError> {
    let mut seen = HashMap::<String, usize>::with_capacity(rows.len());
    let mut keys = Vec::with_capacity(rows.len());
    for (index, row) in rows {
        let key = key_of(row);
        if key.trim().is_empty() {
            return Err(StableRowKeyError::Empty { index: *index });
        }
        if let Some(first_index) = seen.insert(key.clone(), *index) {
            return Err(StableRowKeyError::Duplicate {
                key,
                first_index,
                duplicate_index: *index,
            });
        }
        keys.push(key);
    }
    Ok(keys)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedBodyRow {
    render_key: String,
    stable_key: Option<String>,
    index: usize,
    row: TableRow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolvedBodyRows {
    Valid(Vec<ResolvedBodyRow>),
    Invalid(StableRowKeyError),
}

fn resolve_body_rows(
    rows: Vec<(usize, TableRow)>,
    row_key: Option<Callback<TableRow, String>>,
) -> ResolvedBodyRows {
    let stable_keys = match row_key {
        Some(key_of) => match validate_stable_row_keys(&rows, |row| key_of.run(row.clone())) {
            Ok(keys) => Some(keys),
            Err(error) => return ResolvedBodyRows::Invalid(error),
        },
        None => None,
    };

    ResolvedBodyRows::Valid(
        rows.into_iter()
            .enumerate()
            .map(|(position, (index, row))| {
                let stable_key = stable_keys.as_ref().map(|keys| keys[position].clone());
                let render_key = stable_key.as_ref().map_or_else(
                    || format!("position:{index}"),
                    |key| format!("stable:{key}"),
                );
                ResolvedBodyRow {
                    render_key,
                    stable_key,
                    index,
                    row,
                }
            })
            .collect(),
    )
}

/// Re-focuses the `<tbody>` row whose `data-row-key` equals `key`, but only
/// when focus is currently STRANDED on `<body>` -- the symptom left behind
/// by a native `insertBefore`-based keyed-row move (see the call site's own
/// doc comment for the full story). Never steals focus the user
/// legitimately moved elsewhere: if `document.activeElement` is anything
/// other than `<body>`, this is a deliberate no-op.
fn restore_stranded_focus(tbody_ref: NodeRef<leptos::html::Tbody>, key: &str) {
    // Late-firing guard (ldui-d54): a router navigation between scheduling
    // this and it firing may have disposed this table's reactive owner;
    // try-forms degrade to a no-op rather than panicking the whole wasm app.
    let Some(tbody) = tbody_ref.try_get_untracked().flatten() else {
        return;
    };
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let stranded = document
        .active_element()
        .is_none_or(|active| active.tag_name().eq_ignore_ascii_case("body"));
    if !stranded {
        return;
    }
    let Ok(rows) = tbody.query_selector_all("tr[data-row-key]") else {
        return;
    };
    for i in 0..rows.length() {
        let Some(node) = rows.item(i) else { continue };
        let Some(element) = node.dyn_ref::<web_sys::Element>() else {
            continue;
        };
        if element.get_attribute("data-row-key").as_deref() == Some(key)
            && let Some(html_element) = node.dyn_ref::<web_sys::HtmlElement>()
        {
            let _ = html_element.focus();
            return;
        }
    }
}

#[derive(Clone)]
pub struct DataTableBodyRow {
    pub index: usize,
    pub stable_key: Option<String>,
    pub row: TableRow,
}

#[derive(Clone)]
pub struct DataTableBodyClick {
    pub row: DataTableBodyRow,
    pub ctrl: bool,
    pub shift: bool,
}

/// DataTable body component with loading and empty states
#[component]
pub fn DataTableBody(
    /// Column definitions
    #[prop(into)]
    columns: Signal<Vec<Column>>,

    /// Current page rows paired with their absolute index into the underlying data
    #[prop(into)]
    rows: Signal<Vec<(usize, TableRow)>>,

    /// Loading state
    #[prop(into)]
    loading: Signal<bool>,

    /// Custom text strings
    #[prop(into)]
    texts: Signal<DataTableTexts>,

    /// Custom body cell class
    #[prop(optional, into)]
    body_cell_class: &'static str,

    /// Custom row class
    #[prop(optional, into)]
    row_class: &'static str,

    /// Class applied when a row's absolute index is in `selected_rows`
    #[prop(optional, into)]
    selected_row_class: &'static str,

    /// Selected-row state, keyed by absolute index into the underlying data
    #[prop(optional, into)]
    selected_rows: Signal<BTreeSet<usize>>,

    /// Loading row class
    #[prop(optional, into)]
    loading_row_class: &'static str,

    /// Empty row class
    #[prop(optional, into)]
    empty_row_class: &'static str,

    /// Row-interaction callback, invoked with `(absolute_index, ctrl_or_meta,
    /// shift)`. Modifiers are passed as plain bools rather than an event so the
    /// same path serves both a mouse click and a keyboard Enter/Space.
    #[prop(optional_no_strip)]
    on_row_click: Option<Callback<DataTableBodyClick>>,

    /// Secondary row activation: fired with the row's absolute index on a
    /// double-click, or Shift+Enter from the keyboard (`ldui-tmr`). When set,
    /// the repeat click of a double-click (`detail > 1`) is swallowed so
    /// `on_row_click` fires exactly once per double-click.
    ///
    /// `optional_no_strip` on purpose: the parent forwards its own
    /// `Option<Callback<usize>>` prop verbatim. Plain `optional` strips the
    /// `Option` (the setter wants a bare `Callback`, E0308), and `into` has
    /// no `IntoReactiveValue` impl for `Option<Callback<_>>` in this leptos
    /// version (E0277) — both found by this workspace's CI, 2026-08-24.
    #[prop(optional_no_strip)]
    on_row_inspect: Option<Callback<DataTableBodyRow>>,

    /// Optional stable business key. When supplied, the body reconciles row
    /// DOM by this key rather than by page position. Empty and duplicate keys
    /// suppress every data row and render a visible configuration error.
    #[prop(optional_no_strip)]
    row_key: Option<Callback<TableRow, String>>,

    /// Whether rows are keyboard-operable: focusable (`tabindex=0`) with
    /// Enter/Space activating the same behaviour as a click. Set by
    /// `DataTable` only when the consumer opted into interaction
    /// (`selected_rows` or `on_row_activate`), so plain display tables gain
    /// no tab stops.
    #[prop(optional, into)]
    interactive: bool,

    /// Whether a selection binding is actually configured (a `selected_rows`
    /// signal supplied by the caller, or -- on `ServerDataTable` -- a
    /// `selection` prop). Gates the `aria-selected` attribute independently
    /// of `interactive`: an `on_row_activate`-only table is interactive
    /// (focusable, Enter/Space activates) but has no selection concept, so it
    /// must not claim any row is selectable to assistive tech (ldui-cyhz).
    #[prop(optional, into)]
    has_selection: bool,

    /// Per-cell renderers. A column with `renderer_index = Some(i)` invokes
    /// `cell_renderers[i]` with `(abs_idx, row)`; otherwise the cell renders
    /// `row[col.id]` as plain text. Out-of-bounds indices fall back to text.
    #[prop(optional)]
    cell_renderers: Vec<CellRenderer>,

    /// Column-width overrides (set by dragging a header divider in
    /// `DataTableHeader`), keyed by column id. Applied to each `<td>` so
    /// cell widths stay in sync with the header regardless of the table's
    /// layout algorithm.
    #[prop(optional, into)]
    column_widths: Signal<HashMap<&'static str, f64>>,

    /// Per-column typed-cell resolvers indexed by `Column::typed_cell_index`,
    /// for lightweight `Badge`/`Icon` rendering without a full custom
    /// `CellRenderer`. Checked only when `renderer_index` is `None` or out
    /// of bounds -- `cell_renderers`/`renderer_index` always takes
    /// precedence when both are set on a column.
    #[prop(optional)]
    typed_cells: Vec<TypedCellFn>,

    /// Optional per-row content in a full-width sibling detail row.
    #[prop(optional_no_strip)]
    detail_renderer: Option<RowDetailRenderer>,

    /// Optional per-row extra CSS classes computed from the row's absolute
    /// index and data (e.g. a background tint). Merged after `row_class` /
    /// `selected_row_class`. `optional_no_strip` (rather than plain
    /// `optional`) because the caller (`DataTable`/`ServerDataTable`)
    /// forwards its own already-`Option`-wrapped prop straight through.
    #[prop(optional_no_strip)]
    row_class_fn: Option<Callback<(usize, TableRow), String>>,
) -> impl IntoView {
    let resolved_rows = Memo::new(move |_| resolve_body_rows(rows.get(), row_key));
    let cell_renderers = StoredValue::new(cell_renderers);
    let typed_cells = StoredValue::new(typed_cells);

    // ── Focus-preserving reorder ──
    //
    // `<For>`'s keyed reconciliation moves an existing row's DOM node via a
    // native `insertBefore` when its position changes -- the whole point of
    // keying by `row_key` is that it is the SAME node afterward (proven: its
    // `isConnected` never flips false). But Chromium blurs a currently
    // focused element to `<body>` during that move regardless, even though
    // the node stays connected throughout -- reusing the node is not enough
    // to keep it focused. `focused_stable_key` remembers which row was under
    // keyboard focus (a delegated `focusin`/`focusout` pair on the `<tbody>`,
    // since plain `focus`/`blur` do not bubble) and the effect below
    // re-focuses that key's row -- via a FRESH query, never a stale
    // reference -- whenever a rows change left focus stranded on `<body>`.
    let focused_stable_key: StoredValue<Option<String>> = StoredValue::new(None);
    let tbody_ref = NodeRef::<leptos::html::Tbody>::new();
    let restore_focus_handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    on_cleanup(move || {
        if let Some(handle) = restore_focus_handle.try_get_value().flatten() {
            handle.clear();
        }
    });
    Effect::new(move |ran_before: Option<()>| {
        let _ = resolved_rows.get();
        // Skip the initial mount pass: there is nothing to restore yet, and
        // `document.activeElement` may legitimately be `<body>` before the
        // user has interacted with the table at all.
        if ran_before.is_none() {
            return;
        }
        let Some(key) = focused_stable_key.get_value() else {
            return;
        };
        if let Some(handle) = restore_focus_handle.try_get_value().flatten() {
            handle.clear();
        }
        // Deferred to a fresh macrotask (same rationale as `DataTable`'s own
        // `schedule_measure`/`overflow_handle`, ldui-89rp/ldui-d54): this
        // effect and `<For>`'s own reconciliation both react to the same
        // `resolved_rows` signal with no ordering guarantee between them, so
        // checking `document.activeElement` synchronously here could run
        // before the DOM move has actually happened. A macrotask runs after
        // every synchronous DOM mutation from this reactive flush settles.
        //
        // Two independently-owned closures (rather than one reused across
        // the `Ok`/`Err` arms below, as `DataTable`'s own `check_overflow`
        // does): that pattern relies on the closure's whole capture set
        // being `Copy` so calling it does not consume it, which holds for
        // `check_overflow`'s all-`Copy` captures but not here -- `key` is an
        // owned `String`. Cloning it into each arm keeps both closures
        // independently callable without fighting a move.
        let scheduled_key = key.clone();
        match set_timeout_with_handle(
            move || restore_stranded_focus(tbody_ref, &scheduled_key),
            std::time::Duration::ZERO,
        ) {
            Ok(handle) => {
                restore_focus_handle.try_update_value(|slot| *slot = Some(handle));
            }
            // No `window` to schedule against: checking now is better than
            // not at all.
            Err(_) => restore_stranded_focus(tbody_ref, &key),
        }
    });

    view! {
        <tbody
            node_ref=tbody_ref
            on:focusin=move |event: web_sys::FocusEvent| {
                if let Some(target) = event.target()
                    && let Ok(element) = target.dyn_into::<web_sys::Element>()
                    && let Ok(Some(row)) = element.closest("tr[data-row-key]")
                    && let Some(key) = row.get_attribute("data-row-key")
                {
                    focused_stable_key.set_value(Some(key));
                }
            }
            on:focusout=move |event: web_sys::FocusEvent| {
                // A `None` `related_target` is the ambiguous case a
                // reconciliation-induced blur-to-body produces (focus does
                // not move to any genuinely focused element) -- keep the
                // remembered key so the effect above can correct it. Clear
                // it only when focus demonstrably moved to something outside
                // any row (a deliberate user action), so a later unrelated
                // reorder never steals focus back from where the user
                // actually put it.
                let moved_to_non_row = match event.related_target() {
                    Some(related) => related
                        .dyn_into::<web_sys::Element>()
                        .ok()
                        .and_then(|element| element.closest("tr[data-row-key]").ok().flatten())
                        .is_none(),
                    None => false,
                };
                if moved_to_non_row {
                    focused_stable_key.set_value(None);
                }
            }
        >
            <Show when=move || loading.get()>
                <tr class=loading_row_class>
                    <td
                        colspan=move || columns.with(|columns| columns.len().max(1))
                        class="border border-table-grid py-8 text-center forced-colors:border-[CanvasText]"
                    >
                        {move || texts.with(|texts| texts.loading.clone())}
                    </td>
                </tr>
            </Show>
            <Show when=move || {
                !loading.get() && matches!(&*resolved_rows.read(), ResolvedBodyRows::Valid(rows) if rows.is_empty())
            }>
                <tr class=empty_row_class>
                    <td
                        colspan=move || columns.with(|columns| columns.len().max(1))
                        class="border border-table-grid py-8 text-center forced-colors:border-[CanvasText]"
                    >
                        {move || texts.with(|texts| texts.empty.clone())}
                    </td>
                </tr>
            </Show>
            <Show when=move || {
                !loading.get() && matches!(&*resolved_rows.read(), ResolvedBodyRows::Invalid(_))
            }>
                <tr data-table-row-key-error="true">
                    <td
                        colspan=move || columns.with(|columns| columns.len().max(1))
                        role="alert"
                        class="border border-error bg-error/10 px-3 py-4 text-error forced-colors:border-[CanvasText] forced-colors:text-[CanvasText]"
                    >
                        {move || resolved_rows.with(|rows| match rows {
                            ResolvedBodyRows::Invalid(error) => error.to_string(),
                            ResolvedBodyRows::Valid(_) => String::new(),
                        })}
                    </td>
                </tr>
            </Show>
            <Show when=move || {
                !loading.get() && matches!(&*resolved_rows.read(), ResolvedBodyRows::Valid(rows) if !rows.is_empty())
            }>
                <For
                    each=move || resolved_rows.with(|rows| match rows {
                        ResolvedBodyRows::Valid(rows) => rows
                            .iter()
                            .map(|row| (row.render_key.clone(), row.stable_key.clone()))
                            .collect::<Vec<_>>(),
                        ResolvedBodyRows::Invalid(_) => Vec::new(),
                    })
                    key=|(render_key, _)| render_key.clone()
                    children=move |(render_key, stable_key)| {
                        let current_row = Memo::new(move |_| {
                            resolved_rows.with(|rows| match rows {
                                ResolvedBodyRows::Valid(rows) => rows
                                    .iter()
                                    .find(|row| row.render_key == render_key)
                                    .cloned(),
                                ResolvedBodyRows::Invalid(_) => None,
                            })
                        });
                        let renderers = cell_renderers.get_value();
                        let typed_cell_fns = typed_cells.get_value();

                        view! {
                            <>
                                <tr
                                    data-row-key=stable_key
                                    data-row-index=move || current_row.get().map(|row| row.index)
                                    class=move || current_row.with(|current| {
                                        let Some(current) = current else {
                                            return String::new();
                                        };
                                        let extra = row_class_fn
                                            .map(|callback| callback.run((current.index, current.row.clone())))
                                            .unwrap_or_default();
                                        if selected_rows.with(|selected| selected.contains(&current.index)) {
                                            merge_classes!(row_class, selected_row_class, extra).to_class()
                                        } else {
                                            merge_classes!(row_class, extra).to_class()
                                        }
                                    })
                                    tabindex=interactive.then_some(0)
                                    aria-selected=move || {
                                        let is_selected = current_row.with(|current| {
                                            current
                                                .as_ref()
                                                .is_some_and(|row| selected_rows.with(|selected| selected.contains(&row.index)))
                                        });
                                        row_aria_selected(has_selection, is_selected)
                                    }
                                    on:click=move |event: web_sys::MouseEvent| {
                                        if click_swallowed_by_inspect(event.detail(), on_row_inspect.is_some()) {
                                            return;
                                        }
                                        if let (Some(callback), Some(row)) = (on_row_click, current_row.get_untracked()) {
                                            callback.run(DataTableBodyClick {
                                                row: DataTableBodyRow {
                                                    index: row.index,
                                                    stable_key: row.stable_key,
                                                    row: row.row,
                                                },
                                                ctrl: event.ctrl_key() || event.meta_key(),
                                                shift: event.shift_key(),
                                            });
                                        }
                                    }
                                    on:dblclick=move |event: web_sys::MouseEvent| {
                                        if let (Some(callback), Some(row)) = (on_row_inspect, current_row.get_untracked()) {
                                            event.prevent_default();
                                            callback.run(DataTableBodyRow {
                                                index: row.index,
                                                stable_key: row.stable_key,
                                                row: row.row,
                                            });
                                        }
                                    }
                                    on:keydown=move |event: web_sys::KeyboardEvent| {
                                        if !interactive {
                                            return;
                                        }
                                        let key = event.key();
                                        let ctrl = event.ctrl_key() || event.meta_key();
                                        if key_inspects(&key, ctrl, event.shift_key(), on_row_inspect.is_some()) {
                                            event.prevent_default();
                                            if let (Some(callback), Some(row)) = (on_row_inspect, current_row.get_untracked()) {
                                                callback.run(DataTableBodyRow {
                                                    index: row.index,
                                                    stable_key: row.stable_key,
                                                    row: row.row,
                                                });
                                            }
                                            return;
                                        }
                                        if key == "Enter" || key == " " {
                                            event.prevent_default();
                                            if let (Some(callback), Some(row)) = (on_row_click, current_row.get_untracked()) {
                                                callback.run(DataTableBodyClick {
                                                    row: DataTableBodyRow {
                                                        index: row.index,
                                                        stable_key: row.stable_key,
                                                        row: row.row,
                                                    },
                                                    ctrl,
                                                    shift: event.shift_key(),
                                                });
                                            }
                                        }
                                    }
                                >
                                    {move || current_row.get().map(|current| {
                                        columns.get().iter().map(|column| {
                                            let cell_value = current.row.get(column.id).cloned().unwrap_or_default();
                                            let cell_class = merge_classes!(
                                                "border border-table-grid forced-colors:border-[CanvasText]",
                                                body_cell_class,
                                                column.effective_class()
                                            );
                                            let column_id = column.id;
                                            let is_action = column.is_action;
                                            let truncate_style = if column.truncate {
                                                let max_width = column
                                                    .max_width
                                                    .map(|width| format!("max-width: {width}px; "))
                                                    .unwrap_or_default();
                                                Some(format!(
                                                    "{max_width}overflow: hidden; text-overflow: ellipsis; white-space: nowrap;"
                                                ))
                                            } else {
                                                None
                                            };
                                            let style_attr = move || {
                                                let width_style = column_widths
                                                    .with(|widths| widths.get(column_id).copied())
                                                    .map(|width| format!("width: {}px; ", width.round()));
                                                match (width_style, truncate_style.clone()) {
                                                    (Some(width), Some(truncate)) => Some(format!("{width}{truncate}")),
                                                    (Some(width), None) => Some(width),
                                                    (None, Some(truncate)) => Some(truncate),
                                                    (None, None) => None,
                                                }
                                            };
                                            let title = column.truncate.then_some(cell_value.clone());
                                            let content = match column
                                                .renderer_index
                                                .and_then(|index| renderers.get(index))
                                            {
                                                Some(renderer) => renderer.run((current.index, current.row.clone())),
                                                None => match column
                                                    .typed_cell_index
                                                    .and_then(|index| typed_cell_fns.get(index))
                                                {
                                                    Some(typed_cell) => match typed_cell.run((current.index, current.row.clone())) {
                                                        TypedCell::Text(text) => view! { {text} }.into_any(),
                                                        TypedCell::Badge { text, color } => view! {
                                                            <Badge color=color>{text}</Badge>
                                                        }.into_any(),
                                                        TypedCell::Icon { name, color } => view! {
                                                            <Icon name=name color=color />
                                                        }.into_any(),
                                                    },
                                                    None => view! { {cell_value} }.into_any(),
                                                },
                                            };

                                            view! {
                                                <td
                                                    class=cell_class
                                                    style=style_attr
                                                    title=title
                                                    on:click=move |event: web_sys::MouseEvent| {
                                                        if is_action {
                                                            event.stop_propagation();
                                                        }
                                                    }
                                                    on:keydown=move |event: web_sys::KeyboardEvent| {
                                                        if is_action {
                                                            event.stop_propagation();
                                                        }
                                                    }
                                                >
                                                    {content}
                                                </td>
                                            }
                                        }).collect_view()
                                    })}
                                </tr>
                                {move || current_row.get().and_then(|current| {
                                    detail_renderer
                                        .and_then(|renderer| renderer.run((current.index, current.row)))
                                        .map(|detail| view! {
                                            <tr
                                                class="data-table-detail-row bg-base-100"
                                                data-table-detail-row="true"
                                                data-table-detail-for=current.index
                                                data-row-key=current.stable_key
                                                on:click=move |event| event.stop_propagation()
                                                on:dblclick=move |event| event.stop_propagation()
                                                on:keydown=move |event| event.stop_propagation()
                                            >
                                                <td
                                                    colspan=move || columns.with(|columns| columns.len().max(1))
                                                    class="border border-table-grid px-3 py-2 text-sm text-base-content/80 forced-colors:border-[CanvasText]"
                                                >
                                                    {detail}
                                                </td>
                                            </tr>
                                        })
                                })}
                            </>
                        }
                    }
                />
            </Show>
        </tbody>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, name: &str) -> TableRow {
        HashMap::from([("id", id.to_owned()), ("name", name.to_owned())])
    }

    #[test]
    fn stable_row_keys_preserve_business_identity_across_reorder_and_updates() {
        let first = vec![
            (0, row("matter-1", "First")),
            (1, row("matter-2", "Second")),
        ];
        let reordered = vec![
            (0, row("matter-2", "Updated")),
            (1, row("matter-1", "First")),
        ];

        let first_keys =
            validate_stable_row_keys(&first, |row| row["id"].clone()).expect("unique stable keys");
        let reordered_keys = validate_stable_row_keys(&reordered, |row| row["id"].clone())
            .expect("same stable keys after replacement");

        assert_eq!(first_keys, vec!["matter-1", "matter-2"]);
        assert_eq!(reordered_keys, vec!["matter-2", "matter-1"]);
    }

    #[test]
    fn empty_and_duplicate_stable_row_keys_fail_closed() {
        let empty = vec![(0, row("", "Missing"))];
        assert_eq!(
            validate_stable_row_keys(&empty, |row| row["id"].clone()),
            Err(StableRowKeyError::Empty { index: 0 })
        );

        let duplicate = vec![
            (0, row("matter-1", "First")),
            (1, row("matter-1", "Duplicate")),
        ];
        assert_eq!(
            validate_stable_row_keys(&duplicate, |row| row["id"].clone()),
            Err(StableRowKeyError::Duplicate {
                key: "matter-1".to_owned(),
                first_index: 0,
                duplicate_index: 1,
            })
        );
    }
}
