//! The image insert/edit dialog mounted by `<MarkdownEditor>`.
//!
//! Surfaces a small floating panel over the editor with:
//! - File picker (always for Insert; "Replace file" for Edit)
//! - Alt text input
//! - Width / Height inputs (px values; empty == omit)
//! - Upload progress / error state
//!
//! On save, the dialog uploads any picked file via the consumer's
//! `AssetUploader`, then fires `on_save` with the assembled URL +
//! attributes; the parent editor splices the result into the source.

use std::ops::Range;

use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

use super::asset_upload::{AssetUploadRequest, AssetUploader};
use super::file_io::read_file_bytes;

/// Whether the dialog is opening over a new insertion point or over an
/// existing image we're editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogMode {
    Insert,
    Edit,
}

/// Mutable form state shared between the editor and the dialog.
#[derive(Debug, Clone)]
pub struct ImageDialogState {
    pub open: bool,
    pub mode: DialogMode,
    pub alt: String,
    pub url: String,
    pub width: String,
    pub height: String,
    pub edit_range: Option<Range<usize>>,
    pub uploading: bool,
    pub error: Option<String>,
}

impl Default for ImageDialogState {
    fn default() -> Self {
        Self {
            open: false,
            mode: DialogMode::Insert,
            alt: String::new(),
            url: String::new(),
            width: String::new(),
            height: String::new(),
            edit_range: None,
            uploading: false,
            error: None,
        }
    }
}

/// Result handed to the editor when Save is clicked.
#[derive(Debug, Clone)]
pub struct ImageDialogResult {
    pub alt: String,
    pub url: String,
    pub width: Option<String>,
    pub height: Option<String>,
    pub edit_range: Option<Range<usize>>,
}

#[component]
pub fn ImageDialog(
    /// Shared dialog state — bound directly to form inputs.
    state: RwSignal<ImageDialogState>,
    /// Asset uploader (for picked / replaced files).  When `None`, the
    /// file picker is hidden; Edit mode can still proceed (alt/width/height
    /// only) but Insert is blocked unless `url` is non-empty.
    uploader: Option<AssetUploader>,
    /// Called with the resolved image attributes when the user clicks Save.
    on_save: Callback<ImageDialogResult>,
) -> impl IntoView {
    // Hoist the (non-Copy) uploader into a Copy handle so closures inside
    // the reactive view! can capture by move without going FnOnce.
    let uploader_handle: Option<StoredValue<AssetUploader>> = uploader.map(StoredValue::new);

    let show = move || state.get().open;
    let mode_is_edit = move || state.with(|s| matches!(s.mode, DialogMode::Edit));
    let save_disabled = move || state.with(|s| s.uploading);
    let uploader_present = uploader_handle.is_some();

    view! {
        <Show when=show>
            <div class="lds-image-dialog-backdrop">
                <div class="lds-image-dialog" role="dialog">
                    <div class="lds-image-dialog-title">
                        {move || if mode_is_edit() { "Edit image" } else { "Insert image" }}
                    </div>

                    <label class="lds-image-dialog-field">
                        <span>"Alt text"</span>
                        <input
                            type="text"
                            prop:value=move || state.with(|s| s.alt.clone())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                state.update(|s| s.alt = v);
                            }
                        />
                    </label>

                    <div class="lds-image-dialog-row">
                        <label class="lds-image-dialog-field lds-half">
                            <span>"Width (px)"</span>
                            <input
                                type="text"
                                placeholder="auto"
                                prop:value=move || state.with(|s| s.width.clone())
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    state.update(|s| s.width = v);
                                }
                            />
                        </label>
                        <label class="lds-image-dialog-field lds-half">
                            <span>"Height (px)"</span>
                            <input
                                type="text"
                                placeholder="auto"
                                prop:value=move || state.with(|s| s.height.clone())
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    state.update(|s| s.height = v);
                                }
                            />
                        </label>
                    </div>

                    <Show when=move || uploader_present>
                        <label class="lds-image-dialog-field">
                            <span>
                                {move || if mode_is_edit() { "Replace file" } else { "Choose file" }}
                            </span>
                            <input
                                type="file"
                                accept="image/*"
                                on:change=move |ev| {
                                    if let Some(handle) = uploader_handle {
                                        handle_file_pick(ev, state, handle.get_value());
                                    }
                                }
                            />
                        </label>
                    </Show>

                    <div class="lds-image-dialog-status">
                        <Show when=move || state.with(|s| s.uploading)>
                            <span class="lds-image-dialog-spinner">"Uploading…"</span>
                        </Show>
                        <Show when=move || state.with(|s| s.error.is_some())>
                            <span class="lds-image-dialog-error">
                                {move || state.with(|s| s.error.clone().unwrap_or_default())}
                            </span>
                        </Show>
                        <Show when=move || state.with(|s| !s.url.is_empty() && !s.uploading)>
                            <span class="lds-image-dialog-ok">
                                {"Ready: "}{move || state.with(|s| s.url.clone())}
                            </span>
                        </Show>
                    </div>

                    <div class="lds-image-dialog-actions">
                        <button
                            class="btn btn-sm btn-ghost"
                            on:click=move |_| {
                                state.update(|s| {
                                    s.open = false;
                                    s.error = None;
                                });
                            }
                        >
                            "Cancel"
                        </button>
                        <button
                            class="btn btn-sm btn-primary"
                            prop:disabled=save_disabled
                            on:click=move |_| {
                                let snapshot = state.get_untracked();
                                if snapshot.url.is_empty() {
                                    state.update(|s| s.error = Some("Pick a file before saving.".into()));
                                    return;
                                }
                                let result = ImageDialogResult {
                                    alt: snapshot.alt.clone(),
                                    url: snapshot.url.clone(),
                                    width: trim_to_option(&snapshot.width),
                                    height: trim_to_option(&snapshot.height),
                                    edit_range: snapshot.edit_range.clone(),
                                };
                                state.update(|s| {
                                    s.open = false;
                                    s.error = None;
                                });
                                on_save.run(result);
                            }
                        >
                            "Save"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// File-picker change handler.  Reads bytes, uploads, writes the returned
/// URL into the dialog state's `url` field.
fn handle_file_pick(ev: ev::Event, state: RwSignal<ImageDialogState>, uploader: AssetUploader) {
    let Some(target) = ev.target() else { return };
    let Ok(input) = target.dyn_into::<HtmlInputElement>() else {
        return;
    };
    let Some(files) = input.files() else { return };
    let Some(file) = files.get(0) else { return };
    let filename = file.name();
    let content_type = file.type_();
    let alt_was_empty = state.with_untracked(|s| s.alt.is_empty());
    state.update(|s| {
        s.uploading = true;
        s.error = None;
        if alt_was_empty {
            s.alt = strip_extension(&filename);
        }
    });
    spawn_local(async move {
        let bytes = match read_file_bytes(&file).await {
            Ok(b) => b,
            Err(e) => {
                state.update(|s| {
                    s.uploading = false;
                    s.error = Some(format!("File read failed: {e}"));
                });
                return;
            }
        };
        let req = AssetUploadRequest {
            bytes,
            filename,
            content_type,
        };
        match uploader.upload(req).await {
            Ok(url) => state.update(|s| {
                s.uploading = false;
                s.url = url;
            }),
            Err(e) => state.update(|s| {
                s.uploading = false;
                s.error = Some(e);
            }),
        }
    });
}

fn trim_to_option(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn strip_extension(name: &str) -> String {
    match name.rfind('.') {
        Some(i) => name[..i].to_string(),
        None => name.to_string(),
    }
}
