use crate::components::badge::Badge;
use crate::components::data_table::types::{
    CellRenderer, Column, DataTableTexts, TableRow, TypedCell, TypedCellFn,
};
use crate::components::icon::Icon;
use crate::merge_classes;
use leptos::prelude::*;
use std::collections::{BTreeSet, HashMap};

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
    texts: DataTableTexts,

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
    #[prop(optional, into)]
    on_row_click: Option<Callback<(usize, bool, bool)>>,

    /// Whether rows are keyboard-operable: focusable (`tabindex=0`) with
    /// Enter/Space activating the same behaviour as a click, and carrying
    /// `aria-selected`. Set by `DataTable` only when the consumer opted into
    /// interaction (`selected_rows` or `on_row_activate`), so plain display
    /// tables gain no tab stops.
    #[prop(optional, into)]
    interactive: bool,

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

    /// Optional per-row extra CSS classes computed from the row's absolute
    /// index and data (e.g. a background tint). Merged after `row_class` /
    /// `selected_row_class`. `optional_no_strip` (rather than plain
    /// `optional`) because the caller (`DataTable`/`ServerDataTable`)
    /// forwards its own already-`Option`-wrapped prop straight through.
    #[prop(optional_no_strip)]
    row_class_fn: Option<Callback<(usize, TableRow), String>>,
) -> impl IntoView {
    view! {
        <tbody>
            {move || {
                if loading.get() {
                    // Loading state
                    let col_count = columns.get().len();
                    view! {
                        <tr class=loading_row_class>
                            <td colspan=col_count class="text-center py-8">
                                {texts.loading}
                            </td>
                        </tr>
                    }.into_any()
                } else if rows.get().is_empty() {
                    // Empty state
                    let col_count = columns.get().len();
                    view! {
                        <tr class=empty_row_class>
                            <td colspan=col_count class="text-center py-8">
                                {texts.empty}
                            </td>
                        </tr>
                    }.into_any()
                } else {
                    // Data rows
                    let rows_vec = rows.get();
                    let cols = columns.get();
                    let renderers = cell_renderers.clone();
                    let typed_cell_fns = typed_cells.clone();

                    rows_vec.iter().map(|(abs_idx, row)| {
                        let abs_idx = *abs_idx;
                        let extra_row_class = row_class_fn
                            .map(|f| f.run((abs_idx, row.clone())))
                            .unwrap_or_default();
                        let row_class_dyn = Signal::derive(move || {
                            let extra = extra_row_class.clone();
                            if selected_rows.with(|s| s.contains(&abs_idx)) {
                                merge_classes!(row_class, selected_row_class, extra).to_class()
                            } else {
                                merge_classes!(row_class, extra).to_class()
                            }
                        });

                        // `tabindex`/`aria-selected` only on interactive tables,
                        // so a plain display table adds no tab stops. A `<tr>`
                        // carries the implicit ARIA role `row`, on which
                        // `aria-selected` is valid.
                        let tabindex = interactive.then_some(0);
                        let aria_selected = move || {
                            interactive.then(|| {
                                if selected_rows.with(|s| s.contains(&abs_idx)) {
                                    "true"
                                } else {
                                    "false"
                                }
                            })
                        };

                        view! {
                            <tr
                                class=move || row_class_dyn.get()
                                tabindex=tabindex
                                aria-selected=aria_selected
                                on:click=move |ev: web_sys::MouseEvent| {
                                    if let Some(cb) = on_row_click {
                                        cb.run((abs_idx, ev.ctrl_key() || ev.meta_key(), ev.shift_key()));
                                    }
                                }
                                on:keydown=move |ev: web_sys::KeyboardEvent| {
                                    if !interactive {
                                        return;
                                    }
                                    // Enter and Space activate/select, matching a
                                    // click; Space additionally would scroll the
                                    // page, so suppress its default.
                                    let key = ev.key();
                                    if key == "Enter" || key == " " {
                                        ev.prevent_default();
                                        if let Some(cb) = on_row_click {
                                            cb.run((abs_idx, ev.ctrl_key() || ev.meta_key(), ev.shift_key()));
                                        }
                                    }
                                }
                            >
                                {cols.iter().map(|col| {
                                    let cell_value = row.get(col.id).cloned().unwrap_or_default();
                                    let cell_class = merge_classes!(body_cell_class, col.class.unwrap_or(""));
                                    let col_id = col.id;

                                    // Build truncation style if enabled. Static per column (doesn't
                                    // depend on `column_widths`), computed once here and cloned into
                                    // the reactive width closure below.
                                    let truncate_style = if col.truncate {
                                        let max_w = col.max_width.map(|w| format!("max-width: {}px; ", w)).unwrap_or_default();
                                        Some(format!("{}overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", max_w))
                                    } else {
                                        None
                                    };

                                    // Column-resize width override, kept in sync with the header.
                                    // Scoped to its own reactive closure bound to just this cell's
                                    // `style` attribute (mirrors `row_class_dyn` above) instead of
                                    // being read in this outer per-row/per-column map -- so a
                                    // column-width drag only re-renders each cell's style, not the
                                    // whole body.
                                    let style_attr = move || {
                                        let width_style = column_widths
                                            .with(|m| m.get(col_id).copied())
                                            .map(|w| format!("width: {}px; ", w.round()));
                                        match (width_style, truncate_style.clone()) {
                                            (Some(w), Some(t)) => Some(format!("{w}{t}")),
                                            (Some(w), None) => Some(w),
                                            (None, Some(t)) => Some(t),
                                            (None, None) => None,
                                        }
                                    };

                                    // Title attribute for native tooltip when truncated
                                    let title_attr = if col.truncate {
                                        Some(cell_value.clone())
                                    } else {
                                        None
                                    };

                                    // Precedence: full custom renderer, then typed cell
                                    // (Badge/Icon), then plain text. `renderer_index`
                                    // always wins when both are set on a column.
                                    let content = match col.renderer_index.and_then(|i| renderers.get(i)) {
                                        Some(renderer) => renderer.run((abs_idx, row.clone())),
                                        None => match col.typed_cell_index.and_then(|i| typed_cell_fns.get(i)) {
                                            Some(typed_fn) => match typed_fn.run((abs_idx, row.clone())) {
                                                TypedCell::Text(s) => view! { {s} }.into_any(),
                                                TypedCell::Badge { text, color } => view! {
                                                    <Badge color=color>{text}</Badge>
                                                }.into_any(),
                                                TypedCell::Icon { name, color } => view! {
                                                    <Icon name=name color=color />
                                                }.into_any(),
                                            },
                                            None => view! { {cell_value.clone()} }.into_any(),
                                        },
                                    };

                                    view! {
                                        <td class=cell_class style=style_attr title=title_attr>
                                            {content}
                                        </td>
                                    }
                                }).collect_view()}
                            </tr>
                        }
                    }).collect_view().into_any()
                }
            }}
        </tbody>
    }
}
