//! Generic image intake: a keyboard-operable drop zone that opens a hidden
//! file picker, accepts drag-drop and clipboard paste, sniffs bytes, and
//! renders removable previews. Emits `ImageAttachment`s into `images`.
//!
//! # CSS
//!
//! ```css
//! @source inline("btn btn-ghost btn-sm btn-xs btn-outline rounded-box border-dashed");
//! ```

use super::admission::{ImageAttachmentCaps, ImageRejection, admit};
use super::texts::ImageAttachmentTexts;
use crate::markdown::file_io::read_file_bytes;
use crate::patterns::ImageAttachment;
use crate::utils::image_files_in;
use leptos::ev;
use leptos::html::Input;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// One preview row: the admitted filename and its object URL.
type Preview = (String, String);

#[component]
pub fn ImageAttachmentField(
    /// The admitted images, owned by the caller.
    #[prop(into)]
    images: RwSignal<Vec<ImageAttachment>>,
    /// All rendered copy.
    #[prop(optional, into, default = Signal::stored(ImageAttachmentTexts::default()))]
    texts: Signal<ImageAttachmentTexts>,
    /// Count and per-file byte caps.
    #[prop(optional)]
    caps: ImageAttachmentCaps,
    /// Also accept Ctrl+V anywhere in the document while mounted (dialog use).
    #[prop(optional)]
    capture_document_paste: bool,
    /// Disables the drop zone and the file picker.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called for every rejected file, after the live region announces it.
    #[prop(optional)]
    on_reject: Option<Callback<ImageRejection>>,
    /// Extra classes merged onto the root.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    let input_ref = NodeRef::<Input>::new();
    let dragging = RwSignal::new(0i32);
    let is_dragging = move || dragging.get() > 0;
    let dragging_label = move || (dragging.get() > 0).to_string();
    let announcement = RwSignal::new(String::new());
    let previews: RwSignal<Vec<Preview>> = RwSignal::new(Vec::new());
    let is_disabled = move || disabled.get();

    let announce_rejection = move |r: ImageRejection| {
        let t = texts.get_untracked();
        let msg = match &r {
            ImageRejection::UnsupportedType { filename } => {
                t.rejected_type.replace("{filename}", filename)
            }
            ImageRejection::TooLarge {
                filename,
                max_bytes,
            } => t
                .rejected_size
                .replace("{filename}", filename)
                .replace("{max_mb}", &(max_bytes / (1024 * 1024)).to_string()),
            ImageRejection::TooMany { max_count } => {
                t.rejected_count.replace("{max}", &max_count.to_string())
            }
        };
        announcement.set(msg);
        if let Some(cb) = on_reject {
            cb.run(r);
        }
    };

    // Reads each file, admits it, pushes it. Sequential so `existing` is
    // right for every file in one drop. Try-forms throughout: the closure
    // may outlive the component (ldui-d54).
    let ingest = move |files: Vec<web_sys::File>| {
        if disabled.get_untracked() || files.is_empty() {
            return;
        }
        spawn_local(async move {
            for file in files {
                let name = file.name();
                let bytes = match read_file_bytes(&file).await {
                    Ok(b) => b,
                    Err(_) => {
                        announcement.try_set(
                            texts
                                .get_untracked()
                                .read_failed
                                .replace("{filename}", &name),
                        );
                        continue;
                    }
                };
                let existing = images.try_get_untracked().map(|v| v.len()).unwrap_or(0);
                match admit(existing, &name, bytes, &caps) {
                    Ok(att) => {
                        if let Some(url) = object_url(&att) {
                            previews.try_update(|p| p.push((att.filename.clone(), url)));
                        }
                        images.try_update(|v| v.push(att));
                        announcement.try_set(String::new());
                    }
                    Err(r) => announce_rejection(r),
                }
            }
        });
    };

    let on_paste = move |ev: ev::ClipboardEvent| {
        let Some(data) = ev.clipboard_data() else {
            return;
        };
        let files = image_files_in(&data);
        if !files.is_empty() {
            ev.prevent_default();
            ingest(files);
        }
    };

    if capture_document_paste {
        let handle = window_event_listener(ev::paste, move |ev| {
            let Some(data) = ev.clipboard_data() else {
                return;
            };
            let files = image_files_in(&data);
            if !files.is_empty() {
                ev.prevent_default();
                ingest(files);
            }
        });
        on_cleanup(move || handle.remove());
    }

    on_cleanup(move || {
        for (_, url) in previews.get_untracked() {
            let _ = web_sys::Url::revoke_object_url(&url);
        }
    });

    let remove = move |idx: usize| {
        previews.update(|p| {
            if idx < p.len() {
                let (_, url) = p.remove(idx);
                let _ = web_sys::Url::revoke_object_url(&url);
            }
        });
        images.update(|v| {
            if idx < v.len() {
                v.remove(idx);
            }
        });
    };

    let on_dragover = move |ev: ev::DragEvent| ev.prevent_default();
    let on_dragenter = move |ev: ev::DragEvent| {
        ev.prevent_default();
        dragging.update(|n| *n += 1);
    };
    let on_dragleave = move |_ev: ev::DragEvent| dragging.update(|n| *n = (*n - 1).max(0));
    let on_drop = move |ev: ev::DragEvent| {
        ev.prevent_default();
        dragging.set(0);
        if let Some(data) = ev.data_transfer() {
            ingest(image_files_in(&data));
        }
    };

    let on_input_change = move |ev: ev::Event| {
        let input = event_target::<web_sys::HtmlInputElement>(&ev);
        let mut files = Vec::new();
        if let Some(list) = input.files() {
            for i in 0..list.length() {
                if let Some(f) = list.get(i) {
                    files.push(f);
                }
            }
        }
        input.set_value("");
        ingest(files);
    };

    view! {
        <div class=format!("flex flex-col gap-2 {class}") data-image-attachment-field="">
            <button
                type="button"
                class="btn btn-ghost h-auto min-h-24 w-full flex-col gap-2 rounded-box border border-dashed border-base-content/30 bg-base-200 p-4 font-normal"
                class:border-primary=is_dragging
                data-image-attachment-dropzone=""
                data-dragging=dragging_label
                aria-label=move || texts.get().label
                disabled=is_disabled
                on:click=move |_| {
                    if let Some(i) = input_ref.get() {
                        i.click();
                    }
                }
                on:paste=on_paste
                on:dragover=on_dragover
                on:dragenter=on_dragenter
                on:dragleave=on_dragleave
                on:drop=on_drop
            >
                <span class="text-sm text-base-content/75">{move || texts.get().drop_hint}</span>
                <span class="btn btn-sm btn-outline" aria-hidden="true">
                    {move || texts.get().browse}
                </span>
            </button>
            <input
                type="file"
                class="hidden"
                accept="image/png,image/jpeg,image/webp"
                multiple=true
                tabindex="-1"
                aria-hidden="true"
                node_ref=input_ref
                data-image-attachment-input=""
                on:change=on_input_change
            />
            <ul class="flex flex-wrap gap-2" data-image-attachment-list="">
                <For
                    each=move || previews.get().into_iter().enumerate()
                    key=|(i, (_, url))| format!("{i}-{url}")
                    let:entry
                >
                    {
                        let (idx, (name, url)) = entry;
                        let label_name = name.clone();
                        let remove_label =
                            move || texts.get().remove.replace("{filename}", &label_name);
                        view! {
                            <li
                                class="flex items-center gap-2 rounded-box border border-base-300 bg-base-100 p-2"
                                data-image-attachment-item=""
                            >
                                <img src=url alt="" class="h-12 w-12 rounded object-cover" />
                                <span class="max-w-32 truncate text-sm">{name}</span>
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs"
                                    aria-label=remove_label
                                    data-image-attachment-remove=""
                                    on:click=move |_| remove(idx)
                                >
                                    "×"
                                </button>
                            </li>
                        }
                    }
                </For>
            </ul>
            <p
                class="text-sm text-error"
                role="status"
                aria-live="polite"
                data-image-attachment-status=""
            >
                {move || announcement.get()}
            </p>
        </div>
    }
}

fn object_url(att: &ImageAttachment) -> Option<String> {
    let array = js_sys::Uint8Array::from(att.bytes.as_slice());
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(&att.content_type);
    let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(&parts, &opts).ok()?;
    web_sys::Url::create_object_url_with_blob(&blob).ok()
}
