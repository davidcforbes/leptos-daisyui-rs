//! `<MarkdownEditor>` — textarea source + optional live preview.
//!
//! The editor is intentionally source-visible (the user types markdown
//! syntax, sees the rendered result alongside or on-toggle).
//!
//! Image upload integrates via the optional `on_asset_upload` prop:
//! - Toolbar 🖼 button opens an Insert dialog (or Edit dialog when the
//!   cursor sits on an existing image), with file picker + alt/width/height.
//! - Paste of an image (Cmd-V / Ctrl-V from clipboard) uploads and inserts
//!   silently at the cursor.
//! - Drag-drop of an image file uploads and inserts at the drop point
//!   (currently approximated as the current cursor position; precise
//!   character-under-pointer mapping is a v2 concern).
//!
//! All three paths route through the consumer-supplied [`AssetUploader`].

use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;

use super::asset_upload::{AssetUploadRequest, AssetUploader};
use super::file_io::read_file_bytes;
use super::find_overlay::{FindMode, FindOverlay, FindState};
use super::help_overlay::HelpOverlay;
use super::image_dialog::{DialogMode, ImageDialog, ImageDialogResult, ImageDialogState};
use super::image_parse;
use super::theme::{palette_style, use_theme};
// `MarkdownView` was used by the legacy Source-mode side-by-side
// preview; that path is superseded by the 3-way Mode toggle's `Split`
// mode now, which renders a real WYSIWYG editor on the right.

/// CSS `height` for the editor body.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Length {
    /// `height: auto`
    #[default]
    Auto,
    /// `height: <n>px`
    Px(u32),
}

impl Length {
    fn to_css(self) -> String {
        match self {
            Length::Auto => "auto".to_string(),
            Length::Px(n) => format!("{n}px"),
        }
    }
}

/// Toolbar density.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolbarPreset {
    /// No toolbar.
    None,
    /// Bold, italic, link, code, list, quote, image.
    Minimal,
    /// Minimal + headings, table, codeblock, hr.
    #[default]
    Full,
}

/// Where the preview pane lives — or whether it lives at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewMode {
    /// Source only.
    Off,
    /// Source + preview side-by-side.
    #[default]
    Split,
    /// One-or-the-other, toggleable.
    TogglePreview,
}

/// Editor surface — source-text editing, WYSIWYG, or both side-by-side.
/// Phase 5 (em-berj.5) flipped the default to `Mode::Graphic` now that
/// tables and atomic widgets are also editable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Textarea source editor + optional live preview pane.
    Source,
    /// WYSIWYG editing on the rendered HTML via `<MarkdownGraphicEditor>`.
    #[default]
    Graphic,
    /// Graphic editor + source textarea side-by-side, both bound to
    /// the same source signal so edits in either pane flow through to
    /// the other immediately.
    Split,
}

impl Mode {
    fn as_persist_str(self) -> &'static str {
        match self {
            Mode::Source => "source",
            Mode::Graphic => "graphic",
            Mode::Split => "split",
        }
    }
    fn from_persist_str(s: &str) -> Option<Self> {
        match s {
            "source" => Some(Mode::Source),
            "graphic" => Some(Mode::Graphic),
            "split" => Some(Mode::Split),
            _ => None,
        }
    }
}

/// localStorage key for the editor mode, scoped by document id when
/// the caller provided one (LLM-Wiki passes the page slug); otherwise
/// the per-app default key.
fn mode_storage_key(doc_id: Option<&str>) -> String {
    match doc_id {
        Some(id) if !id.is_empty() => format!("editmark-mode:{id}"),
        _ => "editmark-mode:default".to_string(),
    }
}

fn load_persisted_mode(doc_id: Option<&str>) -> Option<Mode> {
    let storage = web_sys::window()?.local_storage().ok().flatten()?;
    let key = mode_storage_key(doc_id);
    let raw = storage.get_item(&key).ok().flatten()?;
    Mode::from_persist_str(&raw)
}

fn persist_mode(doc_id: Option<&str>, mode: Mode) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let key = mode_storage_key(doc_id);
    let _ = storage.set_item(&key, mode.as_persist_str());
}

/// Bound, two-way markdown editor with a configurable toolbar and live preview.
#[component]
pub fn MarkdownEditor(
    /// Two-way source binding.
    source: RwSignal<String>,
    /// Placeholder shown when the textarea is empty.
    #[prop(optional)]
    placeholder: &'static str,
    /// Optional change notification, fired ~250ms after the user stops typing.
    #[prop(optional, into)]
    on_change: Option<Callback<String>>,
    /// CSS height for the editor body.
    #[prop(optional)]
    height: Length,
    /// Toolbar density preset.
    #[prop(optional)]
    toolbar: ToolbarPreset,
    /// Whether/where the preview lives.
    #[prop(optional)]
    preview_mode: PreviewMode,
    /// Bridge to your asset-storage backend.  When set, the editor enables
    /// image insertion via toolbar button, paste, and drag-drop; each path
    /// uploads via this callback and inserts the returned URL as
    /// `![filename](url)` (or `<img …>` when sized).
    #[prop(optional)]
    on_asset_upload: Option<AssetUploader>,
    /// Optional save callback fired by Ctrl+S (Cmd+S on macOS).  Receives
    /// the current source.  The editor does NOT mutate state on save —
    /// persistence is the consumer's responsibility.
    #[prop(optional, into)]
    on_save: Option<Callback<String>>,
    /// Initial editor surface — defaults to [`Mode::Graphic`].  The
    /// in-toolbar segmented control lets the user switch to Source or
    /// Split at runtime; the selection persists per `doc_id` (or
    /// per-app when `doc_id` is `None`) so reloads remember the
    /// choice.
    #[prop(optional)]
    mode: Mode,
    /// Optional document identifier used to namespace the persisted
    /// mode choice.  LLM-Wiki passes the page slug here so each page
    /// keeps its own preferred mode; callers without a per-document
    /// notion can leave this unset and get the app-wide default.
    #[prop(optional, into)]
    doc_id: Option<String>,
) -> impl IntoView {
    // Resolve the starting mode: persisted choice (if any) overrides
    // the `mode` prop so reloads land in the user's last surface.
    let doc_id_for_load = doc_id.clone();
    let initial_mode = load_persisted_mode(doc_id_for_load.as_deref()).unwrap_or(mode);
    let mode_signal: RwSignal<Mode> = RwSignal::new(initial_mode);
    // Persist any change to the mode.  Capture the doc_id by clone so
    // the Effect doesn't borrow the outer prop.
    let doc_id_for_persist = doc_id.clone();
    Effect::new(move |_| {
        let m = mode_signal.get();
        persist_mode(doc_id_for_persist.as_deref(), m);
    });

    let theme = use_theme();
    let textarea: NodeRef<leptos::html::Textarea> = NodeRef::new();
    let dialog_state = RwSignal::new(ImageDialogState::default());
    let find_state: RwSignal<FindState> = RwSignal::new(FindState::default());
    let help_open: RwSignal<bool> = RwSignal::new(false);
    // Transient "Lint: N fixes applied" notice; cleared by a setTimeout
    // a few seconds after each lint run.
    let lint_status: RwSignal<String> = RwSignal::new(String::new());
    // Transient error toast for upload failures.
    let error_toast: RwSignal<String> = RwSignal::new(String::new());
    // Drag-drop visual cue.  Counter-based to survive child-element
    // dragenter/leave flicker (each child element fires its own
    // enter/leave as the cursor crosses it; only when our counter
    // hits zero is the drag truly out of the textarea).
    let drag_depth: RwSignal<i32> = RwSignal::new(0);
    let is_dropping = Signal::derive(move || drag_depth.get() > 0);

    // Hoist the (non-Copy) uploader into a Copy handle so closures inside
    // the reactive view can capture it freely without going FnOnce.
    let uploader_handle: Option<StoredValue<AssetUploader>> =
        on_asset_upload.clone().map(StoredValue::new);
    let uploader_present = uploader_handle.is_some();

    // `preview_visible` was the toggle backing the legacy Source-mode
    // preview pane; that pane is gone now (use `Mode::Split` for
    // side-by-side editing).  The `preview_mode` prop is kept for
    // backwards compatibility but currently has no effect.
    let _ = preview_mode;

    // Debounced change notification.
    let on_change_outer = on_change;
    let debounce_handle: RwSignal<Option<i32>> = RwSignal::new(None);

    let schedule_change = move |value: String| {
        let Some(cb) = on_change_outer else { return };
        if let Some(win) = web_sys::window() {
            if let Some(prev) = debounce_handle.get_untracked() {
                win.clear_timeout_with_handle(prev);
            }
            let cb_clone = cb;
            let val_clone = value.clone();
            let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                cb_clone.run(val_clone.clone());
            });
            if let Ok(handle) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                250,
            ) {
                debounce_handle.set(Some(handle));
            }
            closure.forget();
        }
    };

    let oninput = move |ev: ev::Event| {
        let Some(target) = ev.target() else { return };
        let Ok(ta) = target.dyn_into::<HtmlTextAreaElement>() else {
            return;
        };
        let value = ta.value();
        source.set(value.clone());
        schedule_change(value);
    };

    let onkeydown = move |ev: ev::KeyboardEvent| {
        // Find/Replace + Save shortcuts take priority over formatting.
        let ctrl = ev.ctrl_key() || ev.meta_key();
        let key = ev.key();
        if ctrl && (key == "f" || key == "F") {
            ev.prevent_default();
            find_state.update(|s| {
                s.open = true;
                s.mode = FindMode::FindOnly;
            });
            return;
        }
        if ctrl && (key == "h" || key == "H") {
            ev.prevent_default();
            find_state.update(|s| {
                s.open = true;
                s.mode = FindMode::FindReplace;
            });
            return;
        }
        if ctrl && (key == "s" || key == "S") {
            ev.prevent_default();
            if let Some(cb) = on_save {
                cb.run(source.get_untracked());
            }
            return;
        }
        if ctrl && key == "/" {
            ev.prevent_default();
            help_open.update(|v| *v = !*v);
            return;
        }
        if key == "Escape" {
            if help_open.get_untracked() {
                ev.prevent_default();
                help_open.set(false);
                return;
            }
            if find_state.with_untracked(|s| s.open) {
                ev.prevent_default();
                find_state.update(|s| {
                    s.open = false;
                    s.current_index = None;
                });
                return;
            }
        }
        handle_key(ev, textarea, source);
    };

    // Paste handler — looks for image files in the clipboard.
    let onpaste = move |ev: ev::ClipboardEvent| {
        let Some(handle) = uploader_handle else {
            return;
        };
        let Some(data) = ev.clipboard_data() else {
            return;
        };
        let Some(files) = data.files() else { return };
        for i in 0..files.length() {
            let Some(file) = files.get(i) else { continue };
            if file.type_().starts_with("image/") {
                ev.prevent_default();
                let uploader = handle.get_value();
                upload_and_insert_at_cursor(file, uploader, source, textarea, Some(error_toast));
                return;
            }
        }
    };

    // Drag-drop handlers.  dragover must preventDefault to enable drop;
    // drop preventDefault always (to suppress the browser's default
    // "navigate to file" behavior).  We also track dragenter/dragleave
    // for the visual drop-target cue (counter-based to absorb child
    // element flicker).
    let ondragover = move |ev: ev::DragEvent| {
        ev.prevent_default();
    };
    let ondragenter = move |ev: ev::DragEvent| {
        ev.prevent_default();
        drag_depth.update(|n| *n += 1);
    };
    let ondragleave = move |_ev: ev::DragEvent| {
        drag_depth.update(|n| *n = (*n - 1).max(0));
    };
    let ondrop = move |ev: ev::DragEvent| {
        ev.prevent_default();
        drag_depth.set(0);
        let Some(handle) = uploader_handle else {
            show_error_toast(error_toast, "Drop ignored: no uploader configured.");
            return;
        };
        let Some(data) = ev.data_transfer() else {
            return;
        };
        let files = data.files();
        let Some(files) = files else { return };
        for i in 0..files.length() {
            let Some(file) = files.get(i) else { continue };
            if file.type_().starts_with("image/") {
                let uploader = handle.get_value();
                upload_and_insert_at_cursor(file, uploader, source, textarea, Some(error_toast));
                return;
            }
        }
        show_error_toast(error_toast, "Drop ignored: no image found.");
    };

    let body_style = move || format!("height: {};", height.to_css());
    let root_style = move || palette_style(&theme.palette());

    // Lint button click — runs the shared editmark-core linter, applies
    // the fixed text, and surfaces a one-line summary that auto-clears.
    let on_lint_click = move || {
        let current = source.get_untracked();
        let report = editmark_core::lint::lint_and_fix(&current);
        if report.changed() {
            source.set(report.fixed_text.clone());
            if let Some(el) = textarea.get_untracked() {
                let ta: &HtmlTextAreaElement = el.as_ref();
                ta.set_value(&report.fixed_text);
            }
        }
        lint_status.set(report.summary());
        if let Some(win) = web_sys::window() {
            let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                lint_status.set(String::new());
            });
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                3000,
            );
            closure.forget();
        }
    };

    // Image button click — context-aware: edit existing or insert new.
    let on_image_click = move || {
        let Some(el) = textarea.get_untracked() else {
            return;
        };
        let ta: &HtmlTextAreaElement = el.as_ref();
        let value = ta.value();
        let cursor = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
        let new_state = match image_parse::find_at_cursor(&value, cursor) {
            Some(img) => ImageDialogState {
                open: true,
                mode: DialogMode::Edit,
                alt: img.alt,
                url: img.url,
                width: img.width.unwrap_or_default(),
                height: img.height.unwrap_or_default(),
                edit_range: Some(img.range),
                uploading: false,
                error: None,
            },
            None => ImageDialogState {
                open: true,
                mode: DialogMode::Insert,
                alt: String::new(),
                url: String::new(),
                width: String::new(),
                height: String::new(),
                edit_range: Some(cursor..cursor),
                uploading: false,
                error: None,
            },
        };
        dialog_state.set(new_state);
    };

    // Dialog save handler — splice serialized markup into the source.
    let on_dialog_save = Callback::new(move |result: ImageDialogResult| {
        let markup = image_parse::serialize(
            &result.alt,
            &result.url,
            None,
            result.width.as_deref(),
            result.height.as_deref(),
        );
        let value = source.get_untracked();
        let range = result
            .edit_range
            .clone()
            .unwrap_or(value.len()..value.len());
        let start = range.start.min(value.len());
        let end = range.end.min(value.len()).max(start);
        let new_value = format!("{}{}{}", &value[..start], markup, &value[end..]);
        source.set(new_value.clone());
        if let Some(el) = textarea.get_untracked() {
            let ta: &HtmlTextAreaElement = el.as_ref();
            ta.set_value(&new_value);
            let cursor = (start + markup.len()) as u32;
            let _ = ta.set_selection_range(cursor, cursor);
            let _ = ta.focus();
        }
    });

    // Mode toggle row click handlers — each sets the mode signal,
    // which triggers persistence + the reactive body re-render.
    let on_set_graphic = move |_ev: ev::MouseEvent| mode_signal.set(Mode::Graphic);
    let on_set_source = move |_ev: ev::MouseEvent| mode_signal.set(Mode::Source);
    let on_set_split = move |_ev: ev::MouseEvent| mode_signal.set(Mode::Split);
    // Reactive style functions for the toggle buttons — highlight the
    // currently-selected mode with a darker background.
    let style_for = move |target: Mode| -> String {
        let selected = mode_signal.get() == target;
        format!(
            "padding:4px 12px;border:1px solid #c0c0c0;background:{};color:{};\
             cursor:pointer;font:12px/1.4 'Segoe UI',sans-serif;",
            if selected { "#2563eb" } else { "transparent" },
            if selected { "white" } else { "inherit" },
        )
    };
    let graphic_btn_style = move || style_for(Mode::Graphic);
    let source_btn_style = move || style_for(Mode::Source);
    let split_btn_style = move || style_for(Mode::Split);
    let toggle_row_style = "display:flex;gap:0;justify-content:flex-end;align-items:center;padding:4px;\
         border-bottom:1px solid #eee;";

    view! {
        <div class="lds-root lds-editor" style=root_style>
            <div class="lds-mode-toggle" style=toggle_row_style>
                <button
                    type="button"
                    style=graphic_btn_style
                    title="Graphic — WYSIWYG editing"
                    on:click=on_set_graphic
                >
                    "Graphic"
                </button>
                <button
                    type="button"
                    style=source_btn_style
                    title="Source — textarea + live preview"
                    on:click=on_set_source
                >
                    "Source"
                </button>
                <button
                    type="button"
                    style=split_btn_style
                    title="Split — graphic + source side-by-side"
                    on:click=on_set_split
                >
                    "Split"
                </button>
            </div>
            {move || match mode_signal.get() {
                Mode::Graphic => view! {
                    <super::graphic_editor::MarkdownGraphicEditor
                        source=source
                        on_asset_upload=on_asset_upload.clone()
                    />
                }.into_any(),
                Mode::Source => view! {
                    <Toolbar
                        preset=toolbar
                        textarea=textarea
                        source=source
                        show_image_button=uploader_present
                        on_image=on_image_click
                        on_lint=on_lint_click
                        lint_status=lint_status
                        on_find=move || find_state.update(|s| { s.open = true; s.mode = FindMode::FindOnly; })
                        on_help=move || help_open.set(true)
                    />
                    <FindOverlay state=find_state source=source textarea=textarea />
                    <HelpOverlay open=help_open />
                    // Source mode is the textarea-only surface — the
                    // legacy `PreviewMode::Split` side-by-side preview
                    // is superseded by the 3-way Mode toggle's `Split`
                    // mode (which renders a real WYSIWYG editor on the
                    // right, not a read-only MarkdownView).  Force the
                    // stacked layout and skip the preview pane so
                    // Source = just the textarea + toolbar.
                    <div
                        class="lds-editor-body lds-stacked"
                        style=body_style
                    >
                        <textarea
                            class="lds-editor-textarea"
                            class:em-dropping=is_dropping
                            placeholder=placeholder
                            prop:value=move || source.get()
                            on:input=oninput
                            on:keydown=onkeydown
                            on:paste=onpaste
                            on:dragover=ondragover
                            on:dragenter=ondragenter
                            on:dragleave=ondragleave
                            on:drop=ondrop
                            node_ref=textarea
                        ></textarea>
                        <Show when=move || !error_toast.get().is_empty()>
                            <div class="lds-error-toast">{move || error_toast.get()}</div>
                        </Show>
                    </div>
                    <ImageDialog
                        state=dialog_state
                        uploader=on_asset_upload.clone()
                        on_save=on_dialog_save
                    />
                }.into_any(),
                Mode::Split => view! {
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;\
                                height:100%;min-height:300px;">
                        <super::graphic_editor::MarkdownGraphicEditor
                            source=source
                            on_asset_upload=on_asset_upload.clone()
                        />
                        <textarea
                            class="lds-editor-textarea"
                            placeholder=placeholder
                            prop:value=move || source.get()
                            on:input=move |ev: ev::Event| {
                                let Some(target) = ev.target() else { return };
                                let Ok(ta) = target.dyn_into::<HtmlTextAreaElement>() else {
                                    return;
                                };
                                source.set(ta.value());
                            }
                            style="height:100%;width:100%;box-sizing:border-box;\
                                   font:13px/1.4 'Consolas',monospace;padding:8px;\
                                   border:1px solid #ddd;border-radius:3px;resize:none;"
                        ></textarea>
                    </div>
                }.into_any(),
            }}
        </div>
    }
    .into_any()
}

/// Async upload + insertion of `![filename](url)` at the textarea's current
/// cursor.  Used by paste and drop.  When `error_toast` is supplied,
/// upload failures surface a transient toast; otherwise they fall silently.
fn upload_and_insert_at_cursor(
    file: web_sys::File,
    uploader: AssetUploader,
    source: RwSignal<String>,
    textarea: NodeRef<leptos::html::Textarea>,
    error_toast: Option<RwSignal<String>>,
) {
    let filename = file.name();
    let content_type = file.type_();
    spawn_local(async move {
        let bytes = match read_file_bytes(&file).await {
            Ok(b) => b,
            Err(e) => {
                if let Some(t) = error_toast {
                    show_error_toast(t, &format!("Read failed: {e}"));
                }
                return;
            }
        };
        let req = AssetUploadRequest {
            bytes,
            filename: filename.clone(),
            content_type,
        };
        let url = match uploader.upload(req).await {
            Ok(u) => u,
            Err(e) => {
                if let Some(t) = error_toast {
                    show_error_toast(t, &format!("Upload failed: {e}"));
                }
                return;
            }
        };
        let alt = strip_extension(&filename);
        let markup = image_parse::serialize(&alt, &url, None, None, None);
        let Some(el) = textarea.get_untracked() else {
            return;
        };
        let ta: &HtmlTextAreaElement = el.as_ref();
        let value = ta.value();
        let cursor = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
        let cursor = cursor.min(value.len());
        let new_value = format!("{}{}{}", &value[..cursor], markup, &value[cursor..]);
        source.set(new_value.clone());
        ta.set_value(&new_value);
        let new_cursor = (cursor + markup.len()) as u32;
        let _ = ta.set_selection_range(new_cursor, new_cursor);
    });
}

/// Set a transient error message that auto-clears after 4 seconds.
fn show_error_toast(toast: RwSignal<String>, message: &str) {
    toast.set(message.to_string());
    if let Some(win) = web_sys::window() {
        let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            toast.set(String::new());
        });
        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            4000,
        );
        closure.forget();
    }
}

fn strip_extension(name: &str) -> String {
    match name.rfind('.') {
        Some(i) => name[..i].to_string(),
        None => name.to_string(),
    }
}

// -- toolbar --------------------------------------------------------------

#[component]
fn Toolbar(
    preset: ToolbarPreset,
    textarea: NodeRef<leptos::html::Textarea>,
    source: RwSignal<String>,
    show_image_button: bool,
    /// Click handler for the Image button.
    on_image: impl Fn() + Copy + Send + Sync + 'static,
    /// Click handler for the Lint button.
    on_lint: impl Fn() + Copy + Send + Sync + 'static,
    /// Click handler for the Find button.
    on_find: impl Fn() + Copy + Send + Sync + 'static,
    /// Click handler for the Help button.
    on_help: impl Fn() + Copy + Send + Sync + 'static,
    /// Transient status message shown to the right of the toolbar buttons.
    lint_status: RwSignal<String>,
) -> impl IntoView {
    if matches!(preset, ToolbarPreset::None) {
        return ().into_any();
    }
    let full = matches!(preset, ToolbarPreset::Full);
    view! {
        <div class="lds-editor-toolbar">
            <ToolbarButton label="B" title="Bold (Ctrl+B)" action=EditorAction::Bold textarea=textarea source=source />
            <ToolbarButton label="I" title="Italic (Ctrl+I)" action=EditorAction::Italic textarea=textarea source=source />
            <ToolbarButton label="</>" title="Inline code" action=EditorAction::InlineCode textarea=textarea source=source />
            <ToolbarButton label="🔗" title="Link (Ctrl+K)" action=EditorAction::Link textarea=textarea source=source />
            <ToolbarButton label="•" title="Bullet list" action=EditorAction::BulletList textarea=textarea source=source />
            <ToolbarButton label="\"" title="Quote" action=EditorAction::Blockquote textarea=textarea source=source />
            <Show when=move || show_image_button>
                <button
                    class="btn btn-sm btn-ghost"
                    title="Image (file picker, paste, or drop)"
                    on:click=move |_| on_image()
                >
                    "🖼"
                </button>
            </Show>
            <Show when=move || full>
                <ToolbarButton label="H1" title="Heading 1" action=EditorAction::Heading(1) textarea=textarea source=source />
                <ToolbarButton label="H2" title="Heading 2" action=EditorAction::Heading(2) textarea=textarea source=source />
                <ToolbarButton label="H3" title="Heading 3" action=EditorAction::Heading(3) textarea=textarea source=source />
                <ToolbarButton label="```" title="Code block" action=EditorAction::CodeBlock textarea=textarea source=source />
                <ToolbarButton label="—" title="Horizontal rule" action=EditorAction::HorizontalRule textarea=textarea source=source />
                <ToolbarButton label="≡" title="Table" action=EditorAction::Table textarea=textarea source=source />
            </Show>
            <button
                class="btn btn-sm btn-ghost"
                title="Find (Ctrl+F) / Replace (Ctrl+H)"
                on:click=move |_| on_find()
            >
                "🔍"
            </button>
            <button
                class="btn btn-sm btn-ghost"
                title="Lint and fix markdown whitespace / line-break hygiene"
                on:click=move |_| on_lint()
            >
                "🪄 Lint"
            </button>
            <button
                class="btn btn-sm btn-ghost"
                title="Keyboard shortcuts (Ctrl+/)"
                on:click=move |_| on_help()
            >
                "?"
            </button>
            <span class="lds-lint-status">
                {move || lint_status.get()}
            </span>
        </div>
    }.into_any()
}

#[component]
fn ToolbarButton(
    label: &'static str,
    title: &'static str,
    action: EditorAction,
    textarea: NodeRef<leptos::html::Textarea>,
    source: RwSignal<String>,
) -> impl IntoView {
    view! {
        <button
            class="btn btn-sm btn-ghost"
            title=title
            on:click=move |_| apply_action(action, textarea, source)
        >
            {label}
        </button>
    }
}

#[derive(Debug, Clone, Copy)]
enum EditorAction {
    Bold,
    Italic,
    InlineCode,
    Link,
    BulletList,
    Blockquote,
    Heading(u8),
    CodeBlock,
    HorizontalRule,
    Table,
}

fn apply_action(
    action: EditorAction,
    textarea: NodeRef<leptos::html::Textarea>,
    source: RwSignal<String>,
) {
    let Some(el) = textarea.get_untracked() else {
        return;
    };
    let ta: &HtmlTextAreaElement = el.as_ref();
    let value = ta.value();
    let start = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
    let end = ta.selection_end().ok().flatten().unwrap_or(0) as usize;
    let (start, end) = clamp_selection(&value, start, end);
    let selected = &value[start..end];

    let (new_text, new_cursor) = match action {
        EditorAction::Bold => wrap(&value, start, end, "**", "**", selected, "bold text"),
        EditorAction::Italic => wrap(&value, start, end, "*", "*", selected, "italic text"),
        EditorAction::InlineCode => wrap(&value, start, end, "`", "`", selected, "code"),
        EditorAction::Link => {
            let text = if selected.is_empty() {
                "link text"
            } else {
                selected
            };
            let inserted = format!("[{text}](url)");
            let new_value = splice(&value, start, end, &inserted);
            let cursor = start + text.len() + 3;
            (new_value, cursor)
        }
        EditorAction::BulletList => line_prefix(&value, start, end, "- "),
        EditorAction::Blockquote => line_prefix(&value, start, end, "> "),
        EditorAction::Heading(level) => {
            let hashes = "#".repeat(level as usize);
            line_prefix(&value, start, end, &format!("{hashes} "))
        }
        EditorAction::CodeBlock => {
            let body = if selected.is_empty() {
                "code"
            } else {
                selected
            };
            let inserted = format!("```\n{body}\n```");
            let new_value = splice(&value, start, end, &inserted);
            (new_value, start + 4)
        }
        EditorAction::HorizontalRule => {
            let inserted = "\n\n---\n\n";
            let new_value = splice(&value, start, end, inserted);
            (new_value, start + inserted.len())
        }
        EditorAction::Table => {
            let inserted = "\n| Col 1 | Col 2 |\n| --- | --- |\n| a | b |\n";
            let new_value = splice(&value, start, end, inserted);
            (new_value, start + inserted.len())
        }
    };

    source.set(new_text.clone());
    ta.set_value(&new_text);
    let n = new_cursor as u32;
    let _ = ta.set_selection_range(n, n);
    let _ = ta.focus();
}

fn handle_key(
    ev: ev::KeyboardEvent,
    textarea: NodeRef<leptos::html::Textarea>,
    source: RwSignal<String>,
) {
    let key = ev.key();
    let ctrl = ev.ctrl_key() || ev.meta_key();
    if ctrl {
        match key.as_str() {
            "b" | "B" => {
                ev.prevent_default();
                apply_action(EditorAction::Bold, textarea, source);
            }
            "i" | "I" => {
                ev.prevent_default();
                apply_action(EditorAction::Italic, textarea, source);
            }
            "k" | "K" => {
                ev.prevent_default();
                apply_action(EditorAction::Link, textarea, source);
            }
            _ => {}
        }
    } else if key == "Tab" {
        ev.prevent_default();
        let Some(el) = textarea.get_untracked() else {
            return;
        };
        let ta: &HtmlTextAreaElement = el.as_ref();
        let value = ta.value();
        let start = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
        let end = ta.selection_end().ok().flatten().unwrap_or(0) as usize;
        let (start, end) = clamp_selection(&value, start, end);
        let multi_line_selection = start < end && value[start..end].contains('\n');
        let (new_value, new_cursor) = if ev.shift_key() {
            dedent_lines(&value, start, end)
        } else if multi_line_selection {
            // Indent every line in the selection by two spaces.
            line_prefix(&value, start, end, "  ")
        } else {
            let inserted = "  ";
            (splice(&value, start, end, inserted), start + inserted.len())
        };
        source.set(new_value.clone());
        ta.set_value(&new_value);
        let n = new_cursor as u32;
        let _ = ta.set_selection_range(n, n);
    } else if key == "Enter" && !ev.shift_key() && !ctrl && !ev.alt_key() {
        // Smart list continuation: Enter at the end of a list-item line
        // continues the list; Enter on an empty list-item line exits it.
        // Anything else falls through to the browser's default Enter.
        let Some(el) = textarea.get_untracked() else {
            return;
        };
        let ta: &HtmlTextAreaElement = el.as_ref();
        let value = ta.value();
        let start = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
        let end = ta.selection_end().ok().flatten().unwrap_or(0) as usize;
        if start != end {
            return;
        }
        if let Some((new_value, new_cursor)) = continue_list(&value, start) {
            ev.prevent_default();
            source.set(new_value.clone());
            ta.set_value(&new_value);
            let n = new_cursor as u32;
            let _ = ta.set_selection_range(n, n);
        }
    }
}

/// One list-marker variant recognized by [`continue_list`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListMarker {
    /// `-`, `*`, or `+` followed by a single space.
    Bullet(char),
    /// `N.` followed by a space.  Numbers are incremented on continuation.
    Ordered(u32),
    /// `- [ ]` — task item.  Continuation defaults back to unchecked.
    TaskUnchecked,
    /// `- [x]` or `- [X]` — checked task; continuation resets to unchecked.
    TaskChecked,
}

impl ListMarker {
    fn next(self) -> String {
        match self {
            ListMarker::Bullet(c) => format!("{c} "),
            ListMarker::Ordered(n) => format!("{}. ", n + 1),
            ListMarker::TaskUnchecked | ListMarker::TaskChecked => "- [ ] ".to_string(),
        }
    }
}

/// If `cursor` sits at the end of a list-item line in `source`, return a
/// `(new_source, new_cursor)` pair that either continues the list (when the
/// item has content) or exits the list (when the item is empty).  Returns
/// `None` if the cursor isn't at end-of-line or the line isn't a list item.
fn continue_list(source: &str, cursor: usize) -> Option<(String, usize)> {
    let line_start = source[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end_rel = source[cursor..].find('\n').unwrap_or(source.len() - cursor);
    let line_end = cursor + line_end_rel;
    if cursor != line_end {
        return None;
    }
    let line = &source[line_start..line_end];
    let (indent, marker, content) = parse_list_prefix(line)?;

    if content.trim().is_empty() {
        // Empty list item — strip the prefix to exit the list cleanly.
        let new_value = format!("{}{}", &source[..line_start], &source[cursor..]);
        return Some((new_value, line_start));
    }

    let next_marker = marker.next();
    let inserted = format!("\n{indent}{next_marker}");
    let new_value = format!("{}{}{}", &source[..cursor], inserted, &source[cursor..]);
    let new_cursor = cursor + inserted.len();
    Some((new_value, new_cursor))
}

/// Recognize a list prefix at the start of `line`.  Returns
/// `(indent, marker, content_after_marker)` on match.
fn parse_list_prefix(line: &str) -> Option<(&str, ListMarker, &str)> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    // Bullet markers — must be followed by exactly one space.  Task
    // checkbox markers nest inside the bullet form.
    for ch in ['-', '*', '+'] {
        let prefix = format!("{ch} ");
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            if ch == '-' {
                if let Some(t) = rest.strip_prefix("[ ] ") {
                    return Some((indent, ListMarker::TaskUnchecked, t));
                }
                if let Some(t) = rest
                    .strip_prefix("[x] ")
                    .or_else(|| rest.strip_prefix("[X] "))
                {
                    return Some((indent, ListMarker::TaskChecked, t));
                }
            }
            return Some((indent, ListMarker::Bullet(ch), rest));
        }
    }

    // Ordered: digits + "." + " ".
    let digits_end = trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .map(|c| c.len_utf8())
        .sum::<usize>();
    if digits_end == 0 {
        return None;
    }
    let bytes = trimmed.as_bytes();
    if bytes.get(digits_end) == Some(&b'.') && bytes.get(digits_end + 1) == Some(&b' ') {
        let n: u32 = trimmed[..digits_end].parse().ok()?;
        let rest = &trimmed[digits_end + 2..];
        return Some((indent, ListMarker::Ordered(n), rest));
    }
    None
}

// -- text manipulation helpers --------------------------------------------

fn clamp_selection(value: &str, start: usize, end: usize) -> (usize, usize) {
    let len = value.len();
    let start = start.min(len);
    let end = end.min(len).max(start);
    let start = snap_to_char_boundary(value, start);
    let end = snap_to_char_boundary(value, end);
    (start, end)
}

fn snap_to_char_boundary(value: &str, mut i: usize) -> usize {
    while i > 0 && !value.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn splice(value: &str, start: usize, end: usize, replacement: &str) -> String {
    let mut out = String::with_capacity(value.len() + replacement.len());
    out.push_str(&value[..start]);
    out.push_str(replacement);
    out.push_str(&value[end..]);
    out
}

fn wrap(
    value: &str,
    start: usize,
    end: usize,
    open: &str,
    close: &str,
    selected: &str,
    placeholder: &str,
) -> (String, usize) {
    let body = if selected.is_empty() {
        placeholder
    } else {
        selected
    };
    let inserted = format!("{open}{body}{close}");
    let new_value = splice(value, start, end, &inserted);
    let cursor_end = start + inserted.len() - close.len();
    (new_value, cursor_end)
}

fn line_prefix(value: &str, start: usize, end: usize, prefix: &str) -> (String, usize) {
    let line_start = value[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let mut out = String::with_capacity(value.len() + prefix.len() * 4);
    out.push_str(&value[..line_start]);
    let region = &value[line_start..end];
    let last_idx = region.len();
    let mut emitted = 0usize;
    let mut new_end = line_start;
    for (i, line) in region.split_inclusive('\n').enumerate() {
        out.push_str(prefix);
        out.push_str(line);
        emitted += line.len();
        new_end = line_start + emitted + prefix.len() * (i + 1);
        if emitted >= last_idx {
            break;
        }
    }
    out.push_str(&value[end..]);
    (out, new_end)
}

fn dedent_lines(value: &str, start: usize, end: usize) -> (String, usize) {
    let line_start = value[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let mut out = String::with_capacity(value.len());
    out.push_str(&value[..line_start]);
    let region = &value[line_start..end];
    let mut removed = 0usize;
    for line in region.split_inclusive('\n') {
        let trimmed = if let Some(rest) = line.strip_prefix("  ") {
            removed += 2;
            rest
        } else if let Some(rest) = line.strip_prefix('\t') {
            removed += 1;
            rest
        } else {
            line
        };
        out.push_str(trimmed);
    }
    out.push_str(&value[end..]);
    let new_cursor = end.saturating_sub(removed);
    (out, new_cursor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splice_basic() {
        assert_eq!(splice("hello world", 6, 11, "Rust"), "hello Rust");
    }

    #[test]
    fn wrap_with_selection() {
        let (out, cursor) = wrap("abc", 0, 3, "**", "**", "abc", "x");
        assert_eq!(out, "**abc**");
        assert_eq!(cursor, 5);
    }

    #[test]
    fn wrap_empty_selection_uses_placeholder() {
        let (out, _) = wrap("", 0, 0, "*", "*", "", "italic text");
        assert_eq!(out, "*italic text*");
    }

    #[test]
    fn line_prefix_single_line() {
        let (out, _) = line_prefix("hello", 0, 5, "- ");
        assert_eq!(out, "- hello");
    }

    #[test]
    fn dedent_removes_two_spaces() {
        let (out, _) = dedent_lines("  hello", 0, 7);
        assert_eq!(out, "hello");
    }

    #[test]
    fn strip_extension_removes_last_dot_segment() {
        assert_eq!(strip_extension("hello.png"), "hello");
        assert_eq!(strip_extension("archive.tar.gz"), "archive.tar");
        assert_eq!(strip_extension("no-extension"), "no-extension");
    }

    #[test]
    fn parse_list_prefix_bullet() {
        let (indent, m, rest) = parse_list_prefix("- item").unwrap();
        assert_eq!(indent, "");
        assert_eq!(m, ListMarker::Bullet('-'));
        assert_eq!(rest, "item");
    }

    #[test]
    fn parse_list_prefix_ordered() {
        let (_, m, rest) = parse_list_prefix("1. first").unwrap();
        assert_eq!(m, ListMarker::Ordered(1));
        assert_eq!(rest, "first");
        let (_, m, _) = parse_list_prefix("42. forty-two").unwrap();
        assert_eq!(m, ListMarker::Ordered(42));
    }

    #[test]
    fn parse_list_prefix_task_items() {
        let (_, m, rest) = parse_list_prefix("- [ ] todo").unwrap();
        assert_eq!(m, ListMarker::TaskUnchecked);
        assert_eq!(rest, "todo");
        let (_, m, _) = parse_list_prefix("- [x] done").unwrap();
        assert_eq!(m, ListMarker::TaskChecked);
        let (_, m, _) = parse_list_prefix("- [X] done").unwrap();
        assert_eq!(m, ListMarker::TaskChecked);
    }

    #[test]
    fn parse_list_prefix_with_indent() {
        let (indent, m, _) = parse_list_prefix("  - nested").unwrap();
        assert_eq!(indent, "  ");
        assert_eq!(m, ListMarker::Bullet('-'));
    }

    #[test]
    fn parse_list_prefix_non_list_returns_none() {
        assert!(parse_list_prefix("plain text").is_none());
        assert!(parse_list_prefix("").is_none());
        assert!(parse_list_prefix("-no space").is_none());
        assert!(parse_list_prefix("1.no space").is_none());
    }

    #[test]
    fn next_marker_bullet_preserves_char() {
        assert_eq!(ListMarker::Bullet('-').next(), "- ");
        assert_eq!(ListMarker::Bullet('*').next(), "* ");
        assert_eq!(ListMarker::Bullet('+').next(), "+ ");
    }

    #[test]
    fn next_marker_ordered_increments() {
        assert_eq!(ListMarker::Ordered(1).next(), "2. ");
        assert_eq!(ListMarker::Ordered(99).next(), "100. ");
    }

    #[test]
    fn next_marker_tasks_reset_to_unchecked() {
        assert_eq!(ListMarker::TaskUnchecked.next(), "- [ ] ");
        assert_eq!(ListMarker::TaskChecked.next(), "- [ ] ");
    }

    #[test]
    fn continue_list_continues_bullet() {
        let src = "- one";
        let (out, cursor) = continue_list(src, src.len()).unwrap();
        assert_eq!(out, "- one\n- ");
        assert_eq!(cursor, out.len());
    }

    #[test]
    fn continue_list_continues_ordered() {
        let src = "1. one";
        let (out, _) = continue_list(src, src.len()).unwrap();
        assert_eq!(out, "1. one\n2. ");
    }

    #[test]
    fn continue_list_exits_on_empty_item() {
        let src = "- one\n- ";
        let (out, cursor) = continue_list(src, src.len()).unwrap();
        assert_eq!(out, "- one\n");
        assert_eq!(cursor, "- one\n".len());
    }

    #[test]
    fn continue_list_none_in_middle_of_line() {
        let src = "- item content";
        // cursor in middle (between "item" and " content")
        assert!(continue_list(src, 6).is_none());
    }

    #[test]
    fn continue_list_none_on_non_list() {
        let src = "plain text";
        assert!(continue_list(src, src.len()).is_none());
    }

    #[test]
    fn continue_list_preserves_indent() {
        let src = "  - nested";
        let (out, _) = continue_list(src, src.len()).unwrap();
        assert_eq!(out, "  - nested\n  - ");
    }
}
