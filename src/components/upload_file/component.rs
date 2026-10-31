use std::sync::atomic::{AtomicU64, Ordering};

use crate::merge_classes;
use leptos::{html::Div, prelude::*};

static NEXT_UPLOAD_FILE_ID: AtomicU64 = AtomicU64::new(0);

/// A process-unique id for one upload's prompt (`ld-upload-file-0`, ...),
/// which names the file input through `aria-labelledby`. A monotonic
/// counter, like `Collapse`'s.
fn next_upload_file_id() -> String {
    format!(
        "ld-upload-file-{}",
        NEXT_UPLOAD_FILE_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// The visible strings of [`UploadFile`], supplied by the caller so a page
/// renders them in its own language (ldui-1ff4). [`Default`] is the English
/// the component showed while the strings were hard-coded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadFileText {
    /// The prompt above the input ("Click to upload").
    pub prompt: String,
    /// The prefix before the chosen file's name ("Selected: ").
    pub selected: String,
}

impl UploadFileText {
    /// Both strings at once.
    pub fn new(prompt: impl Into<String>, selected: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            selected: selected.into(),
        }
    }
}

impl Default for UploadFileText {
    fn default() -> Self {
        Self::new("Click to upload", "Selected: ")
    }
}

/// # UploadFile Component
///
/// A file upload component with styled file input.
/// Built on daisyUI's file-input styling.
///
/// Every visible string comes from the `text` prop ([`UploadFileText`]), so a
/// caller rendering in another language passes its own words; the default is
/// the English shown before. The prompt is also the input's accessible name
/// (`aria-labelledby`). For a drag-and-drop target see
/// [`FileDropZone`](super::FileDropZone).
///
/// ### Add to `input.css`
/// ```css
/// @source inline("file-input file-input-bordered card bg-base-200");
/// ```
///
/// ## Node References
/// - `node_ref` - References the container div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn UploadFile(
    /// Callback when files are selected
    #[prop(optional, into)]
    on_upload: Option<Callback<String>>,

    /// Whether multiple files can be uploaded
    #[prop(optional, into)]
    multiple: Signal<bool>,

    /// Accepted file types (e.g., "image/*", ".pdf")
    #[prop(optional, into)]
    accept: Signal<String>,

    /// Whether the upload is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// The visible strings; English when omitted (ldui-1ff4)
    #[prop(optional, into)]
    text: Signal<UploadFileText>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the container div
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let (selected_file, set_selected_file) = signal(String::new());
    let prompt_id = next_upload_file_id();
    let labelled_by = prompt_id.clone();

    view! {
        <div node_ref=node_ref class=move || merge_classes!("flex flex-col gap-2", class)>
            <div class="card bg-base-200 p-6">
                <div class="flex flex-col items-center gap-4">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-12 w-12 text-base-content/75"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        aria-hidden="true"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                        ></path>
                    </svg>

                    <div class="text-center">
                        <p id=prompt_id class="text-lg font-semibold">{move || text.get().prompt}</p>
                        {move || {
                            let file = selected_file.get();
                            if !file.is_empty() {
                                let selected = text.get().selected;
                                view! {
                                    <div class="mt-2">
                                        <p class="text-sm text-base-content/75">
                                            {selected} {file}
                                        </p>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                ().into_any()
                            }
                        }}

                    </div>

                    <input
                        type="file"
                        class="file-input file-input-bordered w-full max-w-xs"
                        aria-labelledby=labelled_by
                        multiple=move || multiple.get()
                        accept=move || {
                            let a = accept.get();
                            if a.is_empty() { None } else { Some(a) }
                        }

                        disabled=disabled
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            let filename = val
                                .split('\\')
                                .next_back()
                                .unwrap_or(&val)
                                .to_string();
                            set_selected_file.set(filename.clone());
                            if let Some(ref callback) = on_upload {
                                callback.run(filename);
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
