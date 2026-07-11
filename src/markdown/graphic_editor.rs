//! `<MarkdownGraphicEditor>` — WYSIWYG ("graphic mode") editor.
//!
//! This is the Phase 1 minimum-viable graphic-mode editor: a single root
//! `contenteditable` div rendered from the markdown source by
//! `editmark_core::render_html`.  User edits are intercepted on the `input`
//! event, the dirty content is serialized back to markdown via
//! `editmark_core::dom_to_markdown`, and the result is funneled through
//! `editmark_core::apply_edit` to enforce the invariants the desktop
//! WYSIWYG editor depends on (block-boundary clamp, UTF-8 boundary snap).
//!
//! # What v1 handles
//!
//! * Typing in paragraphs / headings / lists / blockquotes — works via
//!   the browser's native contenteditable behaviour, with the funnel
//!   catching the resulting DOM mutations on `input`.
//! * Bold / italic via Ctrl+B / Ctrl+I — `keydown` handler wraps the
//!   current selection in `<strong>` / `<em>` before letting `input` fire.
//! * Backspace across block boundaries — natural contenteditable
//!   behaviour; the serializer re-derives the resulting markdown.
//!
//! # What v1 explicitly defers
//!
//! * ~~IME composition state machine~~ — wired in em-berj.2 via
//!   [`super::ime_state::ImeState`].
//! * ~~Paste normalization~~ — wired in em-berj.2 via
//!   [`super::paste_normalizer::normalize_clipboard_event`].
//! * ~~Rust-side undo stack~~ — wired in em-berj.2 via
//!   [`editmark_core::UndoStack`].  Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z.
//! * Table cell editing (Phase 3)
//! * Atomic widgets — code, math, mermaid (Phase 4)
//! * Caret restoration after full re-render — for v1 the browser keeps
//!   the caret naturally when the DOM isn't replaced; if `apply_edit`
//!   clamps the edit (rare), the caret falls back to end-of-document.
//!
//! # Design choices
//!
//! * **Single root contenteditable**, mirroring both Muya and ProseMirror.
//!   Per-block contenteditable has well-documented browser quirks.
//! * **Render via `render_html` + DOM walk** to stamp `data-em-src` on
//!   each top-level block from `block_source_spans`.  The bridge can use
//!   those annotations to resolve clicks back to source ranges later.
//! * **Self-mutation guard** — when the editor writes the signal in
//!   response to its own input event, the next `Effect` tick skips the
//!   full DOM rewrite (the browser already produced the correct DOM).
//!   External signal changes (e.g. toggling from source mode and back)
//!   still trigger a full re-render.

use editmark_core::{
    Alignment, EditRequest, FixedTextMeasure, NodeKind, Snapshot, UndoStack, apply_edit,
    block_source_spans, build_layout, dom_to_markdown_with_source,
    table_edit::{self, Reason, build_table_context_from_nodes},
};
use leptos::prelude::*;
use leptos::tachys::html::node_ref::NodeRefAttribute;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{ClipboardEvent, CompositionEvent, Element, HtmlElement, MouseEvent};

use super::asset_upload::{AssetUploadRequest, AssetUploader};
use super::file_io::read_file_bytes;
use super::find;
use super::ime_state::ImeState;
use super::paste_normalizer::normalize_clipboard_event;
use super::table_ui::{CellCoord, resolve_cell_in_dom};
use super::theme::{palette_style, use_theme};

/// One action the table context-menu can dispatch (em-berj.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableAction {
    InsertRowAbove,
    InsertRowBelow,
    InsertColLeft,
    InsertColRight,
    DeleteRow,
    DeleteCol,
}

/// State for the right-click context menu that appears over a table
/// cell.  `None` when the menu is hidden.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TableMenuState {
    /// Viewport coordinates the menu should anchor to.
    x: i32,
    y: i32,
    /// Identifies which cell the user right-clicked.
    coord: CellCoord,
}

/// State for the atomic-widget overlay (em-berj.4) — opened when the
/// user double-clicks a fenced code block, display-math, or mermaid
/// widget.  `None` when no overlay is open.
#[derive(Debug, Clone, PartialEq)]
struct AtomicEditState {
    /// Byte range in the source the widget covers — also where the
    /// commit replaces.
    source_start: usize,
    source_end: usize,
    /// Viewport rect the overlay should anchor to (left, top, width,
    /// height in CSS pixels).
    rect_left: f64,
    rect_top: f64,
    rect_width: f64,
    rect_height: f64,
    /// `"code"`, `"math-display"`, or `"mermaid"`.
    kind: String,
    /// Initial source slice — populates the textarea.  We snapshot
    /// here so a re-render of the document doesn't lose the user's
    /// in-progress edit (the source range itself can shift).
    initial: String,
}

/// Inline CSS for each menu button.  Hoisted to module scope so the
/// view! macro inside the menu component can reference it as a
/// `&'static str` literal in every menu item.
const ITEM_STYLE: &str = "display:block;width:100%;padding:6px 14px;text-align:left;\
    border:none;background:transparent;font:inherit;cursor:pointer;color:inherit;";

/// Read `file` asynchronously, upload via `uploader`, and splice
/// `![filename](url)` at the end of `source` (em-berj.6).  Mirrors
/// the source-mode helper but uses `apply_edit` so the block-clamp +
/// UTF-8 invariants are enforced.  Silent on error in v1 — a
/// user-visible toast can land alongside em-berj.6's "transient
/// error toast" polish if a consumer needs it.
fn upload_and_splice_at_end(
    file: web_sys::File,
    uploader: AssetUploader,
    source: RwSignal<String>,
    undo: Rc<RefCell<UndoStack>>,
) {
    let filename = file.name();
    let content_type = file.type_();
    spawn_local(async move {
        let Ok(bytes) = read_file_bytes(&file).await else {
            return;
        };
        let req = AssetUploadRequest {
            bytes,
            filename: filename.clone(),
            content_type,
        };
        let Ok(url) = uploader.upload(req).await else {
            return;
        };
        let markup = format!("![{filename}]({url})");
        let current = source.get_untracked();
        undo.borrow_mut()
            .push(Snapshot::new(&current, current.len()));
        // Glue with a blank line when there's prior content so the
        // inserted image lands as its own block.
        let separator = if current.is_empty() || current.ends_with("\n\n") {
            ""
        } else if current.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        };
        let replacement = format!("{separator}{markup}\n");
        let insert_at = current.len();
        let request = EditRequest {
            source_range: insert_at..insert_at,
            replacement,
            caret_after_byte: usize::MAX,
        };
        let response = apply_edit(&current, request);
        source.set(response.new_source);
    });
}

/// Walk up from `node` to the nearest ancestor carrying
/// `data-em-atomic` — i.e. one of the atomic widgets stamped by
/// `stamp_block_sources`.  Returns the widget element when found, or
/// `None` when the double-click happened outside any atomic widget.
fn atomic_widget_ancestor(node: &web_sys::Node) -> Option<Element> {
    let mut current: Option<web_sys::Node> = Some(node.clone());
    while let Some(n) = current {
        if let Some(el) = n.dyn_ref::<Element>()
            && el.has_attribute("data-em-atomic")
        {
            return Some(el.clone());
        }
        current = n.parent_node();
    }
    None
}

/// Commit the atomic-widget overlay's textarea value back to the
/// source via the edit funnel (em-berj.4).  Snapshots the pre-edit
/// state to the undo stack, runs `apply_edit` over the widget's
/// `[source_start, source_end)` range, and clears the overlay state.
/// Re-render happens automatically via the source signal Effect.
fn commit_atomic_widget(
    state: &AtomicEditState,
    new_value: &str,
    source: RwSignal<String>,
    undo: &Rc<RefCell<UndoStack>>,
    atomic_state: RwSignal<Option<AtomicEditState>>,
) {
    let current = source.get_untracked();
    if new_value == state.initial {
        // No-op edit — close without touching undo or source.
        atomic_state.set(None);
        return;
    }
    undo.borrow_mut()
        .push(Snapshot::new(&current, current.len()));
    let request = EditRequest {
        source_range: state.source_start..state.source_end,
        replacement: new_value.to_string(),
        caret_after_byte: state.source_start + new_value.len(),
    };
    let response = apply_edit(&current, request);
    source.set(response.new_source);
    atomic_state.set(None);
}

/// Resolve a table context-menu action against the live source and
/// broadcast the result.  Reads the current source from `source`,
/// re-runs `build_layout` to locate the targeted `TableRow` nodes,
/// builds a `TableContext`, calls the matching `table_edit` splice
/// helper, pushes a pre-mutation snapshot to the undo stack, and
/// updates the source signal.  Closes the menu when done.
fn dispatch_table_action(
    action: TableAction,
    coord: &CellCoord,
    source: RwSignal<String>,
    undo: &Rc<RefCell<UndoStack>>,
    menu_state: RwSignal<Option<TableMenuState>>,
) {
    let current = source.get_untracked();
    let measure = FixedTextMeasure::default();
    let nodes = build_layout(&current, &measure, 900.0);
    let row_indices: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| match &n.kind {
            NodeKind::TableRow {
                row_source_range, ..
            } if row_source_range.start >= coord.table_source_start
                && row_source_range.end <= coord.table_source_end =>
            {
                Some(i)
            }
            _ => None,
        })
        .collect();
    let Some(caret_node_idx) = row_indices.get(coord.dom_row_idx).copied() else {
        menu_state.set(None);
        return;
    };
    let Some(ctx) =
        build_table_context_from_nodes(&nodes, &current, caret_node_idx, Some(coord.col_idx))
    else {
        menu_state.set(None);
        return;
    };
    let result: Result<(String, usize), Reason> = match action {
        TableAction::InsertRowAbove => {
            // "Above" the caret row.  Header row falls back to inserting
            // just after the alignment line so we don't accidentally
            // relabel the new row as the header.
            let boundary = if ctx.caret_row_idx == 0 {
                ctx.alignment_span.end
            } else {
                ctx.body_rows[ctx.caret_row_idx - 1].row_range.start
            };
            table_edit::insert_row_at(&current, boundary, ctx.col_count)
        }
        TableAction::InsertRowBelow => {
            let boundary = if ctx.caret_row_idx == 0 {
                ctx.alignment_span.end
            } else {
                ctx.body_rows[ctx.caret_row_idx - 1].row_range.end
            };
            table_edit::insert_row_at(&current, boundary, ctx.col_count)
        }
        TableAction::InsertColLeft => table_edit::insert_column(
            &current,
            &ctx.header,
            ctx.alignment_span.clone(),
            &ctx.body_rows,
            ctx.caret_cell_idx,
            ctx.col_count,
            Alignment::None,
            ctx.caret_row_idx,
        ),
        TableAction::InsertColRight => table_edit::insert_column(
            &current,
            &ctx.header,
            ctx.alignment_span.clone(),
            &ctx.body_rows,
            ctx.caret_cell_idx + 1,
            ctx.col_count,
            Alignment::None,
            ctx.caret_row_idx,
        ),
        TableAction::DeleteRow => {
            if ctx.caret_row_idx == 0 {
                Err(Reason::HeaderRow)
            } else {
                let body = &ctx.body_rows[ctx.caret_row_idx - 1];
                table_edit::delete_row(&current, body.row_range.clone(), false)
            }
        }
        TableAction::DeleteCol => table_edit::delete_column(
            &current,
            &ctx.header,
            ctx.alignment_span.clone(),
            &ctx.body_rows,
            ctx.caret_cell_idx,
            ctx.col_count,
            ctx.caret_row_idx,
        ),
    };
    if let Ok((new_source, _caret_after)) = result {
        undo.borrow_mut()
            .push(Snapshot::new(&current, current.len()));
        // No self-set marker — let the Effect re-render the DOM
        // from the canonical source so the new row / column shows up.
        source.set(new_source);
    }
    // Silent refusal on Err(_) in v1 — em-berj.6 (polish) can route
    // the Reason into a status-bar or toast hint.
    menu_state.set(None);
}

/// Phase 1 graphic-mode editor.
///
/// `source` is bound two-way: typing updates the signal, and external
/// changes to the signal re-render the DOM.  When the signal change came
/// from this component's own input handler, the re-render is suppressed
/// (the DOM is already correct).
#[component]
pub fn MarkdownGraphicEditor(
    /// Two-way source binding.
    source: RwSignal<String>,
    /// Optional asset-upload bridge — when present, image files from
    /// paste / drop are uploaded via the callback and the returned
    /// URL is inserted as `![filename](url)` at the end of the source
    /// (em-berj.6).  Mirrors the contract used by `<MarkdownEditor>`.
    #[prop(default = None)]
    on_asset_upload: Option<AssetUploader>,
) -> impl IntoView {
    let theme = use_theme();
    let host: NodeRef<leptos::html::Div> = NodeRef::new();

    // Tracks the source string this component last *wrote to itself*.  When
    // the next reactive read of `source` returns that same string, the
    // Effect knows the DOM is already up-to-date and skips the re-render.
    // Wrapped in Rc<RefCell<_>> so input handlers and the Effect can share
    // mutable access from inside their respective Copy closures.
    let last_self_set: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // IME composition flag — `true` between `compositionstart` and
    // `compositionend`.  The `on_input` handler bails when set so the
    // funnel sees one committed edit per composition, not one per
    // preedit keystroke.
    let ime = ImeState::new();

    // Bounded undo / redo stack.  Snapshot-before-edit is the contract:
    // every dispatch path (input, format keys, paste) pushes the
    // *pre-edit* state before calling `apply_edit`.
    let undo_stack: Rc<RefCell<UndoStack>> = Rc::new(RefCell::new(UndoStack::new()));

    // Right-click table context menu — `None` when hidden.
    let menu_state: RwSignal<Option<TableMenuState>> = RwSignal::new(None);

    // Asset uploader hoisted into a Copy handle so event closures can
    // capture it freely.  None when the caller didn't provide one —
    // in that case paste / drop of image files falls through to the
    // text/html path.
    let uploader_handle: Option<StoredValue<AssetUploader>> =
        on_asset_upload.clone().map(StoredValue::new);

    // Atomic-widget overlay editor — `None` when no widget is open.
    let atomic_state: RwSignal<Option<AtomicEditState>> = RwSignal::new(None);
    // The current textarea value for the open widget (drives the
    // commit path).  Decoupled from `atomic_state` so typing into the
    // textarea doesn't re-render the whole overlay.
    let atomic_draft: RwSignal<String> = RwSignal::new(String::new());

    // Initial render + reactive re-render on external source changes.
    {
        let last_self_set = last_self_set.clone();
        Effect::new(move |_| {
            let src = source.get();
            let Some(el) = host.get() else { return };
            let html_el: &HtmlElement = el.as_ref();
            let is_self_set = last_self_set
                .borrow()
                .as_deref()
                .map(|s| s == src.as_str())
                .unwrap_or(false);
            if is_self_set {
                // Our own input handler already left the DOM in the
                // correct shape; skip the rewrite to preserve caret,
                // composition state, and selection.
                return;
            }
            render_into(html_el, &src);
        });
    }

    // input handler: browser already mutated the DOM, we serialize and
    // funnel through apply_edit to enforce invariants, then update the
    // source signal.
    let last_self_set_for_input = last_self_set.clone();
    let ime_for_input = ime.clone();
    let undo_for_input = undo_stack.clone();
    let on_input = move |_ev: leptos::ev::Event| {
        // While an IME composition is active, the browser is
        // mutating the DOM with preedit text we don't yet own —
        // dispatching now would corrupt the source and force a
        // re-render that ends the composition.  See ime_state.rs.
        if ime_for_input.is_composing() {
            return;
        }
        let Some(el) = host.get() else { return };
        let html_el: &HtmlElement = el.as_ref();
        let dirty_html = html_el.inner_html();
        let current = source.get_untracked();
        // Pass the canonical source so dom_to_markdown can substitute
        // atomic-widget regions (mermaid SVG, KaTeX HTML, highlighted
        // code) with their `data-em-src` slice instead of re-serializing
        // the rendered DOM (em-berj.4).
        let new_markdown = dom_to_markdown_with_source(&dirty_html, Some(&current));
        // This is a full-document round-trip — the whole new markdown
        // is the new source.  We deliberately bypass `apply_edit` here:
        // apply_edit's invariant #2 (block-clamp) would shrink the
        // `0..current.len()` range to the source span of the top-level
        // block containing byte 0 (the first block), then splice
        // `new_markdown` in place of those bytes and APPEND the rest
        // of the old source — producing `new_full_doc + old_tail` and
        // doubling everything after the first block (em-ziqq).  The
        // block-clamp is correct for scoped edits with caret-based
        // ranges (the desktop's em-01l9.1 invariant); it's just wrong
        // for full-doc round-trips, where we already have the entire
        // new source in hand.
        if new_markdown == current {
            return;
        }
        undo_for_input
            .borrow_mut()
            .push(Snapshot::new(&current, current.len()));
        *last_self_set_for_input.borrow_mut() = Some(new_markdown.clone());
        source.set(new_markdown);
    };

    // Composition start: suspend input dispatch until end.
    let ime_for_start = ime.clone();
    let on_composition_start = move |_ev: CompositionEvent| {
        ime_for_start.begin();
    };

    // Composition end: clear the flag and synthesize a single edit
    // by re-reading the DOM (which now contains the committed text).
    let ime_for_end = ime.clone();
    let last_self_set_for_end = last_self_set.clone();
    let undo_for_end = undo_stack.clone();
    let on_composition_end = move |_ev: CompositionEvent| {
        if !ime_for_end.end() {
            return;
        }
        let Some(el) = host.get() else { return };
        let html_el: &HtmlElement = el.as_ref();
        let dirty_html = html_el.inner_html();
        let current = source.get_untracked();
        let new_markdown = dom_to_markdown_with_source(&dirty_html, Some(&current));
        if new_markdown == current {
            return;
        }
        undo_for_end
            .borrow_mut()
            .push(Snapshot::new(&current, current.len()));
        // Bypass apply_edit for the same reason as on_input — see
        // em-ziqq.  IME commit is a full-doc round-trip; apply_edit's
        // block-clamp would corrupt the source.
        *last_self_set_for_end.borrow_mut() = Some(new_markdown.clone());
        source.set(new_markdown);
    };

    // Paste handler: try image-file upload first (em-berj.6), then
    // fall through to the text/html normalizer.  Both paths splice
    // into the source via apply_edit and let the Effect re-render so
    // the inserted markdown shows up formatted.
    let undo_for_paste = undo_stack.clone();
    let on_paste = move |ev: ClipboardEvent| {
        ev.prevent_default();
        // Image-file path — only fires when the caller wired an
        // uploader and the clipboard carries at least one image/*
        // file.  Returns true when the paste was consumed.
        if let Some(handle) = uploader_handle
            && let Some(data) = ev.clipboard_data()
            && let Some(files) = data.files()
        {
            for i in 0..files.length() {
                let Some(file) = files.get(i) else { continue };
                if file.type_().starts_with("image/") {
                    let uploader = handle.get_value();
                    upload_and_splice_at_end(file, uploader, source, undo_for_paste.clone());
                    return;
                }
            }
        }
        let result = normalize_clipboard_event(&ev);
        if result.is_empty() {
            return;
        }
        let current = source.get_untracked();
        undo_for_paste
            .borrow_mut()
            .push(Snapshot::new(&current, current.len()));
        let insert_at = current.len();
        // Glue paragraphs together with a blank line when the source
        // is non-empty and doesn't already end in one — otherwise the
        // pasted block runs into the previous block's last paragraph
        // when re-rendered.
        let separator = if current.is_empty()
            || current.ends_with("\n\n")
            || (!result.from_html && !result.markdown.starts_with('\n'))
        {
            ""
        } else if current.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        };
        let mut replacement = String::with_capacity(separator.len() + result.markdown.len());
        replacement.push_str(separator);
        replacement.push_str(&result.markdown);
        let request = EditRequest {
            source_range: insert_at..insert_at,
            replacement,
            caret_after_byte: usize::MAX,
        };
        let response = apply_edit(&current, request);
        source.set(response.new_source);
    };

    // Double-click on an atomic widget → open the overlay editor.
    let on_dblclick = move |ev: MouseEvent| {
        let Some(target) = ev.target() else { return };
        let Ok(node) = target.dyn_into::<web_sys::Node>() else {
            return;
        };
        let Some(widget) = atomic_widget_ancestor(&node) else {
            return;
        };
        let Some(kind) = widget.get_attribute("data-em-atomic") else {
            return;
        };
        let Some(src_attr) = widget.get_attribute("data-em-src") else {
            return;
        };
        let Some((start_s, end_s)) = src_attr.split_once('-') else {
            return;
        };
        let Ok(start) = start_s.parse::<usize>() else {
            return;
        };
        let Ok(end) = end_s.parse::<usize>() else {
            return;
        };
        let current = source.get_untracked();
        if end > current.len() || start > end {
            return;
        }
        let initial = current[start..end].to_string();
        let rect = widget.get_bounding_client_rect();
        ev.prevent_default();
        atomic_draft.set(initial.clone());
        atomic_state.set(Some(AtomicEditState {
            source_start: start,
            source_end: end,
            rect_left: rect.left(),
            rect_top: rect.top(),
            rect_width: rect.width(),
            rect_height: rect.height(),
            kind,
            initial,
        }));
    };

    // Drag-drop handlers (em-berj.6) — mirror the source-mode paths
    // from <MarkdownEditor>.  dragover must preventDefault to enable
    // the drop event to fire; drop preventDefault always to suppress
    // the browser's "navigate to file" default.  Image files upload
    // via the AssetUploader and splice at end-of-source.
    let on_dragover = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
    };
    let undo_for_drop = undo_stack.clone();
    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        let Some(handle) = uploader_handle else {
            return;
        };
        let Some(data) = ev.data_transfer() else {
            return;
        };
        let Some(files) = data.files() else { return };
        for i in 0..files.length() {
            let Some(file) = files.get(i) else { continue };
            if file.type_().starts_with("image/") {
                let uploader = handle.get_value();
                upload_and_splice_at_end(file, uploader, source, undo_for_drop.clone());
                return;
            }
        }
    };

    // Right-click on a table cell → open the table context menu.
    let on_contextmenu = move |ev: MouseEvent| {
        let Some(target) = ev.target() else { return };
        let Ok(node) = target.dyn_into::<web_sys::Node>() else {
            return;
        };
        let Some(cell) = super::table_ui::cell_ancestor(&node) else {
            return;
        };
        let Some(coord) = resolve_cell_in_dom(&cell) else {
            return;
        };
        ev.prevent_default();
        menu_state.set(Some(TableMenuState {
            x: ev.client_x(),
            y: ev.client_y(),
            coord,
        }));
    };

    // -- Graphic-mode Find (em-i8j9.3) -----------------------------------
    //
    // Only 'static/Copy scalars live in signals; the `Vec<web_sys::Range>`
    // is recomputed from the live DOM inside each handler (Range is !Send
    // and awkward to store).  `find_current` is a 0-based index into the
    // recomputed matches; `find_count` mirrors its length for the "n/m"
    // readout.
    let find_open: RwSignal<bool> = RwSignal::new(false);
    let find_query: RwSignal<String> = RwSignal::new(String::new());
    let find_current: RwSignal<usize> = RwSignal::new(0);
    let find_count: RwSignal<usize> = RwSignal::new(0);
    let find_case: RwSignal<bool> = RwSignal::new(false);
    let find_input: NodeRef<leptos::html::Input> = NodeRef::new();

    // Autofocus the query input whenever the bar opens.  The Show renders
    // the input asynchronously, so `find_input.get()` starts `None`; the
    // NodeRef read is tracked, so the Effect re-runs and focuses once it
    // mounts.
    Effect::new(move |_| {
        if find_open.get()
            && let Some(input) = find_input.get()
        {
            let el: &web_sys::HtmlInputElement = input.as_ref();
            let _ = el.focus();
        }
    });

    // Recompute matches from the live DOM for `desired`-th current match,
    // repaint highlights, sync the count/current signals, and scroll the
    // current match into view.  Shared by the query-input, next, prev, and
    // case-toggle handlers.  `desired` is clamped/wrapped against the live
    // match count.
    let refresh_find = move |desired: usize| {
        let Some(el) = host.get_untracked() else {
            return;
        };
        let html_el: &HtmlElement = el.as_ref();
        let query = find_query.get_untracked();
        let ranges = compute_match_ranges(html_el, &query, find_case.get_untracked());
        find_count.set(ranges.len());
        if ranges.is_empty() {
            find_current.set(0);
            clear_find_highlights();
            return;
        }
        let current = desired % ranges.len();
        find_current.set(current);
        set_find_highlights(&ranges, current);
        scroll_range_into_view(&ranges[current]);
    };

    // Typing in the query box: re-search from match 0.
    let on_find_input = move |ev: leptos::ev::Event| {
        find_query.set(event_target_value(&ev));
        refresh_find(0);
    };
    let on_find_next = move || {
        let total = find_count.get_untracked();
        let next = if total == 0 {
            0
        } else {
            (find_current.get_untracked() + 1) % total
        };
        refresh_find(next);
    };
    let on_find_prev = move || {
        let total = find_count.get_untracked();
        let prev = if total == 0 {
            0
        } else {
            (find_current.get_untracked() + total - 1) % total
        };
        refresh_find(prev);
    };
    let on_find_toggle_case = move || {
        find_case.update(|c| *c = !*c);
        refresh_find(find_current.get_untracked());
    };
    // Close + clear: drop highlights, reset state, and return focus to the
    // editable surface so the caret is live again.
    let close_find = move || {
        clear_find_highlights();
        find_open.set(false);
        find_query.set(String::new());
        find_current.set(0);
        find_count.set(0);
        if let Some(el) = host.get_untracked() {
            let html_el: &HtmlElement = el.as_ref();
            let _ = html_el.focus();
        }
    };
    // Open (or re-focus) the bar; re-run any existing query so highlights
    // and the count come back.
    let open_find = move || {
        find_open.set(true);
        if !find_query.get_untracked().is_empty() {
            refresh_find(find_current.get_untracked());
        }
    };

    // Enter = next, Shift+Enter = prev, Esc = close, all within the input.
    let on_find_keydown = move |ev: leptos::ev::KeyboardEvent| match ev.key().as_str() {
        "Enter" => {
            ev.prevent_default();
            if ev.shift_key() {
                on_find_prev();
            } else {
                on_find_next();
            }
        }
        "Escape" => {
            ev.prevent_default();
            ev.stop_propagation();
            close_find();
        }
        _ => {}
    };

    let find_count_label = move || {
        let total = find_count.get();
        if total == 0 {
            "0/0".to_string()
        } else {
            format!("{}/{}", find_current.get() + 1, total)
        }
    };
    let find_case_on = move || find_case.get();

    // keydown handler: format shortcuts (Ctrl+B / Ctrl+I) and
    // undo / redo (Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z).  We bypass the
    // browser's native execCommand path (deprecated, inconsistent
    // across browsers) and own all three actions explicitly so the
    // Rust-side undo stack is the single source of truth.
    let undo_for_keys = undo_stack.clone();
    let last_self_set_for_keys = last_self_set.clone();
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let ctrl = ev.ctrl_key() || ev.meta_key();
        let shift = ev.shift_key();
        let key_raw = ev.key();
        let key = key_raw.to_ascii_lowercase();

        // Graphic-mode Find (em-i8j9.3).  Ctrl/Cmd+F opens the find bar and
        // preempts the browser's native find; Esc closes it when open.
        // stop_propagation keeps any source-mode Ctrl+F handler (editor.rs)
        // from also firing when both surfaces are mounted.
        if ctrl && key == "f" {
            ev.prevent_default();
            ev.stop_propagation();
            open_find();
            return;
        }
        if key == "escape" && find_open.get_untracked() {
            ev.prevent_default();
            ev.stop_propagation();
            close_find();
            return;
        }

        // Table-cell key handling (em-berj.3): when the caret sits
        // inside a `<td>` / `<th>`, Enter is silently rejected (a
        // `\n` would break the row), Tab / Shift+Tab navigates the
        // next / previous cell, and ArrowUp / ArrowDown move to the
        // cell directly above / below at the same column index.
        // The cell-ancestor lookup happens lazily — only when one of
        // these keys actually fires — so non-table typing pays
        // nothing for the check.
        let table_relevant =
            !ctrl && (key == "enter" || key == "tab" || key == "arrowup" || key == "arrowdown");
        if table_relevant
            && let Some(window) = web_sys::window()
            && let Some(cell) = super::table_ui::caret_cell(&window)
        {
            if key == "enter" {
                ev.prevent_default();
                return;
            }
            let target_cell = if key == "tab" {
                if shift {
                    super::table_ui::prev_cell(&cell)
                } else {
                    super::table_ui::next_cell(&cell)
                }
            } else if key == "arrowup" {
                super::table_ui::cell_above(&cell)
            } else {
                super::table_ui::cell_below(&cell)
            };
            // Arrow keys fall through to native handling when
            // there's no target cell, so the caret can exit
            // the table.  Tab always preventDefault to avoid
            // browser focus drift to the next focusable
            // element on the page.
            if let Some(target) = target_cell {
                ev.prevent_default();
                if let Some(doc) = window.document() {
                    super::table_ui::focus_cell(&window, &doc, &target);
                }
                return;
            } else if key == "tab" {
                ev.prevent_default();
                return;
            }
        }

        if !ctrl {
            return;
        }

        // Undo / redo
        if key == "z" && !shift {
            ev.prevent_default();
            let current = source.get_untracked();
            let now = Snapshot::new(&current, current.len());
            if let Some(prev) = undo_for_keys.borrow_mut().undo(now) {
                // External-looking set so the Effect re-renders.
                source.set(prev.source);
            }
            return;
        }
        if (key == "z" && shift) || key == "y" {
            ev.prevent_default();
            let current = source.get_untracked();
            let now = Snapshot::new(&current, current.len());
            if let Some(next) = undo_for_keys.borrow_mut().redo(now) {
                source.set(next.source);
            }
            return;
        }

        let tag = match key.as_str() {
            "b" => "strong",
            "i" => "em",
            _ => return,
        };
        ev.prevent_default();
        wrap_selection_in_tag(tag);
        // Manually fire an input-like update so the source signal catches
        // up — we don't get a beforeinput/input event for our own DOM edit.
        if let Some(el) = host.get() {
            let html_el: &HtmlElement = el.as_ref();
            let dirty_html = html_el.inner_html();
            let current = source.get_untracked();
            let new_markdown = dom_to_markdown_with_source(&dirty_html, Some(&current));
            if new_markdown != current {
                undo_for_keys
                    .borrow_mut()
                    .push(Snapshot::new(&current, current.len()));
                *last_self_set_for_keys.borrow_mut() = Some(new_markdown.clone());
                source.set(new_markdown);
            }
        }
    };

    // Per-button mousedown handler factory — captures `source`,
    // `undo_for_menu` (cloned per call), and `menu_state` per button.
    // Kept as a regular closure (not boxed) so each invocation moves a
    // fresh clone of the Rc, avoiding the `Send` bound issues that
    // would arise from sharing one `Rc<dyn Fn>` across the view tree.
    let undo_for_menu = undo_stack.clone();

    let style = move || {
        let palette = theme.palette();
        palette_style(&palette)
    };

    // Reactive style functions — capture only `menu_state` (a Copy
    // RwSignal), so they're Send-compliant by construction.  The menu
    // DOM is always present; visibility is toggled via `display:none`
    // / `display:block` on the wrapper.  The per-button mousedown
    // closures (below) read `menu_state.get_untracked()` at click time
    // to pick up the current coord — that way they never need to
    // capture any non-Send context themselves.
    let backdrop_style = move || {
        if menu_state.with(|s| s.is_some()) {
            "position:fixed;inset:0;z-index:9998;background:transparent;display:block;".to_string()
        } else {
            "display:none;".to_string()
        }
    };
    let menu_style = move || match menu_state.get() {
        Some(s) => format!(
            "position:fixed;left:{}px;top:{}px;z-index:9999;background:white;\
             border:1px solid #c0c0c0;border-radius:4px;padding:4px 0;\
             box-shadow:0 2px 8px rgba(0,0,0,0.15);min-width:170px;\
             font:13px/1.4 'Segoe UI',sans-serif;color:#222;display:block;",
            s.x, s.y
        ),
        None => "display:none;".to_string(),
    };

    // One mousedown closure per menu action.  Each captures its own
    // clone of the undo Rc — event-listener closures don't need to be
    // `Send` (they fire on the main thread), so the Rc is fine here
    // even though the parent component is otherwise Send-bounded.
    let mk_handler = |action: TableAction, undo: Rc<RefCell<UndoStack>>| {
        move |ev: MouseEvent| {
            ev.prevent_default();
            ev.stop_propagation();
            if let Some(state) = menu_state.get_untracked() {
                dispatch_table_action(action, &state.coord, source, &undo, menu_state);
            }
        }
    };
    let on_row_above = mk_handler(TableAction::InsertRowAbove, undo_for_menu.clone());
    let on_row_below = mk_handler(TableAction::InsertRowBelow, undo_for_menu.clone());
    let on_col_left = mk_handler(TableAction::InsertColLeft, undo_for_menu.clone());
    let on_col_right = mk_handler(TableAction::InsertColRight, undo_for_menu.clone());
    let on_row_delete = mk_handler(TableAction::DeleteRow, undo_for_menu.clone());
    let on_col_delete = mk_handler(TableAction::DeleteCol, undo_for_menu.clone());

    // Atomic-widget overlay style + event handlers (em-berj.4).
    let overlay_backdrop_style = move || {
        if atomic_state.with(|s| s.is_some()) {
            "position:fixed;inset:0;z-index:9990;background:rgba(0,0,0,0.05);display:block;"
                .to_string()
        } else {
            "display:none;".to_string()
        }
    };
    let overlay_style = move || match atomic_state.get() {
        Some(s) => format!(
            "position:fixed;left:{}px;top:{}px;width:{}px;min-height:{}px;\
             z-index:9991;background:white;border:1px solid #888;border-radius:4px;\
             box-shadow:0 4px 12px rgba(0,0,0,0.18);padding:6px;\
             font:13px/1.4 'Consolas',monospace;color:#222;display:block;",
            s.rect_left,
            s.rect_top,
            s.rect_width.max(320.0),
            s.rect_height.max(120.0),
        ),
        None => "display:none;".to_string(),
    };
    let overlay_label = move || {
        atomic_state
            .get()
            .map(|s| match s.kind.as_str() {
                "code" => "Edit code block — Ctrl+Enter to commit, Esc to cancel",
                "math-display" => "Edit math — Ctrl+Enter to commit, Esc to cancel",
                "mermaid" => "Edit mermaid — Ctrl+Enter to commit, Esc to cancel",
                _ => "Edit — Ctrl+Enter to commit, Esc to cancel",
            })
            .unwrap_or("")
            .to_string()
    };
    let textarea_value = move || atomic_draft.get();
    let on_overlay_input = move |ev: leptos::ev::Event| {
        if let Some(target) = ev.target()
            && let Ok(ta) = target.dyn_into::<web_sys::HtmlTextAreaElement>()
        {
            atomic_draft.set(ta.value());
        }
    };
    let undo_for_overlay = undo_stack.clone();
    let undo_for_overlay_kd = undo_stack.clone();
    let undo_for_backdrop = undo_stack.clone();
    let on_overlay_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let key = ev.key();
        if key == "Escape" {
            ev.prevent_default();
            ev.stop_propagation();
            atomic_state.set(None);
            return;
        }
        if (ev.ctrl_key() || ev.meta_key()) && key == "Enter" {
            ev.prevent_default();
            ev.stop_propagation();
            if let Some(state) = atomic_state.get_untracked() {
                let value = atomic_draft.get_untracked();
                commit_atomic_widget(&state, &value, source, &undo_for_overlay_kd, atomic_state);
            }
        }
    };
    let on_commit_button = move |ev: MouseEvent| {
        ev.prevent_default();
        if let Some(state) = atomic_state.get_untracked() {
            let value = atomic_draft.get_untracked();
            commit_atomic_widget(&state, &value, source, &undo_for_overlay, atomic_state);
        }
    };
    let on_cancel_button = move |ev: MouseEvent| {
        ev.prevent_default();
        atomic_state.set(None);
    };
    // Backdrop click commits (matches the bead's spec — click outside
    // applies, the same as Ctrl+Enter).  Esc cancels via the keydown
    // handler above.
    let on_overlay_backdrop = move |ev: MouseEvent| {
        ev.prevent_default();
        if let Some(state) = atomic_state.get_untracked() {
            let value = atomic_draft.get_untracked();
            commit_atomic_widget(&state, &value, source, &undo_for_backdrop, atomic_state);
        }
    };

    view! {
        <>
            // em-berj.6 polish: scoped CSS for selection highlight,
            // atomic-widget hover affordance, and the doubleclick
            // hint.  Inline so we don't ship a separate stylesheet.
            <style>
                {include_str!("graphic_editor.css")}
            </style>
            // Relative wrapper so the find bar can anchor top-right of the
            // editable surface via position:absolute.  The find bar is a
            // SIBLING of the contenteditable (never a child) so it can't
            // corrupt the editable content or the dom→markdown round-trip.
            <div style="position:relative;">
                <div
                    class="lds-graphic lds-root"
                    contenteditable="true"
                    spellcheck="true"
                    style=style
                    node_ref=host
                    on:input=on_input
                    on:keydown=on_keydown
                    on:compositionstart=on_composition_start
                    on:compositionend=on_composition_end
                    on:paste=on_paste
                    on:contextmenu=on_contextmenu
                    on:dblclick=on_dblclick
                    on:dragover=on_dragover
                    on:drop=on_drop
                ></div>
                <Show when=move || find_open.get()>
                    <div class="lds-graphic-find">
                        <input
                            type="text"
                            class="lds-find-input"
                            placeholder="Find"
                            prop:value=move || find_query.get()
                            on:input=on_find_input
                            on:keydown=on_find_keydown
                            node_ref=find_input
                        />
                        <span class="lds-find-count">{find_count_label}</span>
                        <button
                            class="btn btn-xs btn-ghost"
                            title="Previous match (Shift+Enter)"
                            on:click=move |_| on_find_prev()
                        >
                            "◀"
                        </button>
                        <button
                            class="btn btn-xs btn-ghost"
                            title="Next match (Enter)"
                            on:click=move |_| on_find_next()
                        >
                            "▶"
                        </button>
                        <button
                            class="btn btn-xs btn-ghost"
                            title="Case sensitive"
                            class:lds-find-toggle-on=find_case_on
                            on:click=move |_| on_find_toggle_case()
                        >
                            "Aa"
                        </button>
                        <button
                            class="btn btn-xs btn-ghost"
                            title="Close (Esc)"
                            on:click=move |_| close_find()
                        >
                            "✕"
                        </button>
                    </div>
                </Show>
            </div>
            <div
                style=backdrop_style
                on:mousedown=move |_| menu_state.set(None)
            ></div>
            <div
                style=menu_style
                on:contextmenu=move |ev: MouseEvent| ev.prevent_default()
            >
                <button type="button" style=ITEM_STYLE on:mousedown=on_row_above>
                    "Insert row above"
                </button>
                <button type="button" style=ITEM_STYLE on:mousedown=on_row_below>
                    "Insert row below"
                </button>
                <button type="button" style=ITEM_STYLE on:mousedown=on_col_left>
                    "Insert column left"
                </button>
                <button type="button" style=ITEM_STYLE on:mousedown=on_col_right>
                    "Insert column right"
                </button>
                <button type="button" style=ITEM_STYLE on:mousedown=on_row_delete>
                    "Delete row"
                </button>
                <button type="button" style=ITEM_STYLE on:mousedown=on_col_delete>
                    "Delete column"
                </button>
            </div>
            <div
                style=overlay_backdrop_style
                on:mousedown=on_overlay_backdrop
            ></div>
            <div style=overlay_style>
                <div style="font:11px/1.4 'Segoe UI',sans-serif;color:#666;margin-bottom:4px;">
                    {overlay_label}
                </div>
                <textarea
                    style="display:block;width:100%;min-height:120px;\
                           font:inherit;border:1px solid #ddd;border-radius:3px;\
                           padding:6px;box-sizing:border-box;resize:vertical;"
                    prop:value=textarea_value
                    on:input=on_overlay_input
                    on:keydown=on_overlay_keydown
                ></textarea>
                <div style="display:flex;gap:6px;justify-content:flex-end;margin-top:6px;">
                    <button type="button"
                        style="padding:4px 12px;border:1px solid #ccc;background:#f5f5f5;\
                               border-radius:3px;cursor:pointer;font:inherit;"
                        on:mousedown=on_cancel_button
                    >
                        "Cancel"
                    </button>
                    <button type="button"
                        style="padding:4px 12px;border:1px solid #2563eb;background:#2563eb;\
                               color:white;border-radius:3px;cursor:pointer;font:inherit;"
                        on:mousedown=on_commit_button
                    >
                        "Commit"
                    </button>
                </div>
            </div>
        </>
    }
}

/// Render `source` into the contenteditable host, then walk the resulting
/// DOM and stamp `data-em-src="START-END"` on each top-level block from
/// `block_source_spans`.
fn render_into(host: &HtmlElement, source: &str) {
    use editmark_core::{FixedTextMeasure, build_layout, render_html};
    let measure = FixedTextMeasure::default();
    let nodes = build_layout(source, &measure, 900.0);
    let html = render_html(&nodes);
    host.set_inner_html(&html);
    stamp_block_sources(host, source);
    // Render mermaid placeholders to SVG, same as read-only MarkdownView.
    // The atomic-widget attributes (data-em-atomic / data-em-src /
    // contenteditable=false) live on the <div class="mermaid"> and survive the
    // inner-HTML swap; the double-click editor reads the source from the
    // data-em-src markdown range, not the div's DOM, so the injected SVG is
    // display-only and editing still works. (Runs only on external source
    // changes — see the is_self_set guard at the render_into call site — so it
    // is not a per-keystroke cost.)
    super::view::process_mermaid(host);
}

/// Walk the top-level children of `host` and tag each with the source
/// byte range from `block_source_spans`.  Atomic widgets (fenced code,
/// display math, mermaid) additionally get `data-em-atomic="kind"` and
/// `contenteditable="false"` (em-berj.4) so the browser treats them
/// as opaque islands and the double-click handler can open the
/// overlay editor for them.
///
/// The browser may add or merge children during editing, so these
/// annotations are authoritative only for the *just-rendered* state.
fn stamp_block_sources(host: &HtmlElement, source: &str) {
    let spans = block_source_spans(source);
    let top_level: Vec<_> = spans.into_iter().filter(|s| s.depth == 0).collect();
    let children = host.children();
    let mut span_idx = 0usize;
    for i in 0..children.length() {
        let Some(node) = children.item(i) else {
            continue;
        };
        if !is_block_element(&node) {
            continue;
        }
        if span_idx >= top_level.len() {
            break;
        }
        let span = &top_level[span_idx];
        let _ = node.set_attribute(
            "data-em-src",
            &format!("{}-{}", span.source_range.start, span.source_range.end),
        );
        if let Some(kind) = atomic_kind(&node) {
            let _ = node.set_attribute("data-em-atomic", kind);
            let _ = node.set_attribute("contenteditable", "false");
        }
        span_idx += 1;
    }
}

/// Classify a top-level rendered element as one of the atomic widget
/// kinds (em-berj.4) — `"code"`, `"math-display"`, or `"mermaid"` —
/// or `None` when the element should remain editable inline.
fn atomic_kind(el: &Element) -> Option<&'static str> {
    let tag = el.tag_name().to_ascii_lowercase();
    if tag == "pre" {
        // `<pre><code class="language-…">` — fenced code block.
        return Some("code");
    }
    if tag == "div" {
        let class = el.class_name();
        if class.split_ascii_whitespace().any(|c| c == "math-display") {
            return Some("math-display");
        }
        if class.split_ascii_whitespace().any(|c| c == "mermaid") {
            return Some("mermaid");
        }
    }
    None
}

fn is_block_element(el: &Element) -> bool {
    let tag = el.tag_name().to_ascii_lowercase();
    matches!(
        tag.as_str(),
        "p" | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "blockquote"
            | "pre"
            | "table"
            | "hr"
            | "div"
            | "dl"
    )
}

/// Wrap the current browser selection in a `<tag>...</tag>` element.
/// Mirrors what `document.execCommand('bold')` used to do, without
/// relying on the deprecated API.  No-op when the selection is empty.
fn wrap_selection_in_tag(tag: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Ok(Some(selection)) = window.get_selection() else {
        return;
    };
    if selection.is_collapsed() {
        return;
    }
    let Ok(range) = selection.get_range_at(0) else {
        return;
    };
    let extracted = match range.extract_contents() {
        Ok(frag) => frag,
        Err(_) => return,
    };
    let Ok(wrapper) = document.create_element(tag) else {
        return;
    };
    if wrapper.append_child(&extracted).is_err() {
        return;
    }
    if range.insert_node(&wrapper).is_err() {
        return;
    }
    // Place the caret just after the wrapped content so subsequent typing
    // exits the new mark — matches the desktop's "wrap then exit" UX.
    let _ = range.set_start_after(&wrapper);
    let _ = range.set_end_after(&wrapper);
    let _ = selection.remove_all_ranges();
    let _ = selection.add_range(&range);
}

// -- Graphic-mode Find (em-i8j9.3) ----------------------------------------
//
// Find-only search over the RENDERED text of the contenteditable, painted
// via the CSS Custom Highlight API so matches are highlighted WITHOUT
// mutating the editable DOM (which would corrupt both the editable content
// and the dom→markdown round-trip).  State lives in plain signals; the
// `web_sys::Range` vec is recomputed from the live DOM inside each handler
// rather than stored (Range is `!Send` and awkward to hold in a signal).

/// Recursively collect the `web_sys::Text` descendants of `node` in
/// document order.
fn collect_text_nodes(node: &web_sys::Node, out: &mut Vec<web_sys::Text>) {
    let children = node.child_nodes();
    for i in 0..children.length() {
        let Some(child) = children.item(i) else {
            continue;
        };
        if child.node_type() == web_sys::Node::TEXT_NODE {
            if let Some(text) = child.dyn_ref::<web_sys::Text>() {
                out.push(text.clone());
            }
        } else {
            collect_text_nodes(&child, out);
        }
    }
}

/// Recompute the per-text-node match ranges for `query` over the rendered
/// text under `host`, in document order.
///
/// LIMITATION: matches are found per text node only.  A query that
/// straddles an inline-element boundary (e.g. a match spanning the end of a
/// `<strong>` and the following plain text — two separate text nodes) is
/// acceptably skipped in this v1; single-node ranges keep the DOM Range
/// construction simple and robust.
fn compute_match_ranges(
    host: &HtmlElement,
    query: &str,
    case_sensitive: bool,
) -> Vec<web_sys::Range> {
    let mut ranges = Vec::new();
    if query.is_empty() {
        return ranges;
    }
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return ranges;
    };
    let mut texts: Vec<web_sys::Text> = Vec::new();
    collect_text_nodes(host.unchecked_ref::<web_sys::Node>(), &mut texts);
    for text in &texts {
        // `data()` lives on CharacterData; Text derefs to it.
        let data = text.data();
        for br in find::find_all_matches(&data, query, case_sensitive) {
            // DOM Range offsets are UTF-16 code units, but `find` returns
            // UTF-8 byte ranges — convert by counting UTF-16 code units in
            // the leading slices.
            let start_u16 = data[..br.start].encode_utf16().count() as u32;
            let end_u16 = data[..br.end].encode_utf16().count() as u32;
            let Ok(range) = document.create_range() else {
                continue;
            };
            let node = text.unchecked_ref::<web_sys::Node>();
            if range.set_start(node, start_u16).is_err() {
                continue;
            }
            if range.set_end(node, end_u16).is_err() {
                continue;
            }
            ranges.push(range);
        }
    }
    ranges
}

/// Highlight the current match by selecting its `Range` via the stable
/// Selection API — the browser paints the selection. Consistent with
/// Source-mode find (which uses the textarea selection). Only the current match
/// is shown (a Selection holds one range); next/prev move it.
///
/// This deliberately avoids the CSS Custom Highlight API: its web-sys bindings
/// (`css::highlights` / `Highlight` / `HighlightRegistry`) are gated behind
/// `--cfg web_sys_unstable_apis`, a GLOBAL flag that flips other web-sys
/// signatures (`client_x`, `scroll_top`, …) from `i32` to `f64` and breaks the
/// rest of this crate's components. Selecting the range needs no such flag.
fn set_find_highlights(ranges: &[web_sys::Range], current: usize) {
    let Some(win) = web_sys::window() else {
        return;
    };
    let Ok(Some(sel)) = win.get_selection() else {
        return;
    };
    let _ = sel.remove_all_ranges();
    if let Some(cur) = ranges.get(current) {
        let _ = sel.add_range(cur);
    }
}

/// Drop the find selection.
fn clear_find_highlights() {
    if let Some(win) = web_sys::window()
        && let Ok(Some(sel)) = win.get_selection()
    {
        let _ = sel.remove_all_ranges();
    }
}

/// Scroll the current match into view.  `web_sys::Range` has no scroll
/// method, so we scroll the nearest ancestor `Element` of the match's start
/// container (the text node's parent element).
fn scroll_range_into_view(range: &web_sys::Range) {
    let Ok(container) = range.start_container() else {
        return;
    };
    let element = container
        .dyn_ref::<Element>()
        .cloned()
        .or_else(|| container.parent_element());
    if let Some(el) = element {
        let opts = web_sys::ScrollIntoViewOptions::new();
        opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
        el.scroll_into_view_with_scroll_into_view_options(&opts);
    }
}
