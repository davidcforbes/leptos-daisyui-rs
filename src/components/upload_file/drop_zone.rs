use std::sync::atomic::{AtomicU64, Ordering};

use crate::merge_classes;
use leptos::{
    html::{Div, Input},
    prelude::*,
};
use web_sys::{DragEvent, File, FileList, HtmlInputElement, KeyboardEvent};

static NEXT_DROP_ZONE_ID: AtomicU64 = AtomicU64::new(0);

/// A process-unique id for one zone's prompt (`ld-file-drop-0`, ...), the
/// target of the zone's `aria-labelledby`. A monotonic counter, like
/// `Collapse`'s: stable within a page's lifetime, which is all a label
/// reference needs.
fn next_drop_zone_id() -> String {
    format!(
        "ld-file-drop-{}",
        NEXT_DROP_ZONE_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// The visible strings of [`FileDropZone`], supplied by the caller so a page
/// renders them in its own language (ldui-1ff4). [`Default`] is English.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDropZoneText {
    /// The prompt inside the zone, which is also its accessible name
    /// ("Drop files here, or press Enter to choose").
    pub prompt: String,
    /// The prefix before the names of the files taken ("Selected: ").
    pub selected: String,
}

impl FileDropZoneText {
    /// Both strings at once.
    pub fn new(prompt: impl Into<String>, selected: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            selected: selected.into(),
        }
    }
}

impl Default for FileDropZoneText {
    fn default() -> Self {
        Self::new("Drop files here, or press Enter to choose", "Selected: ")
    }
}

/// Whether one file passes an `accept` list in the `<input type="file">`
/// grammar: a comma-separated mix of extensions (`.pdf`), type families
/// (`image/*`) and exact MIME types (`application/pdf`), matched without
/// regard to case. An empty list admits every file.
pub fn accept_matches(accept: &str, name: &str, mime: &str) -> bool {
    let mut listed = false;
    for token in accept.split(',').map(str::trim).filter(|t| !t.is_empty()) {
        listed = true;
        let hit = if let Some(ext) = token.strip_prefix('.') {
            name.rsplit_once('.')
                .is_some_and(|(_, file_ext)| file_ext.eq_ignore_ascii_case(ext))
        } else if let Some(family) = token.strip_suffix("/*") {
            mime.split_once('/')
                .is_some_and(|(mime_family, _)| mime_family.eq_ignore_ascii_case(family))
        } else {
            mime.eq_ignore_ascii_case(token)
        };
        if hit {
            return true;
        }
    }
    !listed
}

/// Split `files` into the ones `accept` admits and the ones it turns away,
/// each in its original order. `name` and `mime` read a file's two facts so
/// the rule is testable without a browser `File`.
pub fn partition_by_accept<T>(
    accept: &str,
    files: Vec<T>,
    name: impl Fn(&T) -> String,
    mime: impl Fn(&T) -> String,
) -> (Vec<T>, Vec<T>) {
    files
        .into_iter()
        .partition(|file| accept_matches(accept, &name(file), &mime(file)))
}

fn files_of(list: Option<FileList>) -> Vec<File> {
    let Some(list) = list else {
        return Vec::new();
    };
    (0..list.length()).filter_map(|i| list.get(i)).collect()
}

/// # FileDropZone Component
///
/// A drag-and-drop target for files (ldui-1ff4), the counterpart of
/// [`UploadFile`](super::UploadFile) for a desk that drops a scanner's PDF
/// instead of browsing for it. Dropped files, and files chosen through the
/// visually hidden `<input type="file">` the zone opens, are filtered by the
/// same `accept` list and handed to `on_files` as `Vec<web_sys::File>`;
/// files the list turns away go to `on_rejected`. With `multiple` off only
/// the first dropped file is considered.
///
/// The zone is one keyboard tab stop (`role="button"`, `tabindex="0"`,
/// named by its own prompt through `aria-labelledby`) that opens the picker
/// on Enter or Space, draws the shared `ld-focus-ring` when focused, and
/// highlights (`data-dragging="true"`, `border-primary`) while a drag is
/// over it. The input is a sibling of the zone, not a child, so no
/// interactive control nests inside another. Every visible string comes
/// from `text` ([`FileDropZoneText`]).
///
/// ### Add to `input.css`
/// ```css
/// @source inline("card bg-base-200 bg-base-300 border-2 border-dashed border-base-300 border-primary cursor-pointer cursor-not-allowed opacity-60 sr-only ld-focus-ring");
/// ```
///
/// ## Node References
/// - `node_ref` - References the container div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn FileDropZone(
    /// Called with the files the `accept` list admits, dropped or chosen
    #[prop(optional, into)]
    on_files: Option<Callback<Vec<File>>>,

    /// Called with the files the `accept` list turned away
    #[prop(optional, into)]
    on_rejected: Option<Callback<Vec<File>>>,

    /// Accepted file types (e.g., "image/*", ".pdf", "application/pdf")
    #[prop(optional, into)]
    accept: Signal<String>,

    /// Whether several files may be dropped or chosen at once
    #[prop(optional, into)]
    multiple: Signal<bool>,

    /// Whether the zone is disabled: no drop, no picker, not focusable
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// The visible strings; English when omitted
    #[prop(optional, into)]
    text: Signal<FileDropZoneText>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the container div
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let input_ref: NodeRef<Input> = NodeRef::new();
    let (dragging, set_dragging) = signal(false);
    let (selected, set_selected) = signal(Vec::<String>::new());
    let prompt_id = next_drop_zone_id();
    let labelled_by = prompt_id.clone();

    let take = move |files: Vec<File>| {
        let files = if multiple.get_untracked() {
            files
        } else {
            files.into_iter().take(1).collect()
        };
        let (accepted, rejected) =
            partition_by_accept(&accept.get_untracked(), files, |f| f.name(), |f| f.type_());
        if !rejected.is_empty()
            && let Some(cb) = on_rejected
        {
            cb.run(rejected);
        }
        if !accepted.is_empty() {
            set_selected.set(accepted.iter().map(File::name).collect());
            if let Some(cb) = on_files {
                cb.run(accepted);
            }
        }
    };

    let open_picker = move || {
        if disabled.get_untracked() {
            return;
        }
        if let Some(input) = input_ref.get_untracked() {
            input.click();
        }
    };

    view! {
        <div node_ref=node_ref class=move || merge_classes!("flex flex-col gap-2", class)>
            <div
                role="button"
                tabindex=move || if disabled.get() { "-1" } else { "0" }
                aria-disabled=move || disabled.get().then_some("true")
                aria-labelledby=labelled_by
                data-dragging=move || dragging.get().then_some("true")
                class=move || {
                    merge_classes!(
                        "card bg-base-200 p-6 border-2 border-dashed ld-focus-ring",
                        if dragging.get() { "border-primary bg-base-300" } else { "border-base-300" },
                        if disabled.get() { "opacity-60 cursor-not-allowed" } else { "cursor-pointer" }
                    )
                }
                on:click=move |_| open_picker()
                on:keydown=move |ev: KeyboardEvent| {
                    if matches!(ev.key().as_str(), "Enter" | " " | "Spacebar") {
                        ev.prevent_default();
                        open_picker();
                    }
                }
                on:dragenter=move |ev: DragEvent| {
                    ev.prevent_default();
                }
                on:dragover=move |ev: DragEvent| {
                    // Without this the browser refuses the drop.
                    ev.prevent_default();
                    if !disabled.get_untracked() {
                        if let Some(dt) = ev.data_transfer() {
                            dt.set_drop_effect("copy");
                        }
                        set_dragging.set(true);
                    }
                }
                on:dragleave=move |_ev: DragEvent| {
                    set_dragging.set(false);
                }
                on:drop=move |ev: DragEvent| {
                    ev.prevent_default();
                    set_dragging.set(false);
                    if disabled.get_untracked() {
                        return;
                    }
                    take(files_of(ev.data_transfer().and_then(|dt| dt.files())));
                }
            >
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
                            let names = selected.get();
                            if names.is_empty() {
                                ().into_any()
                            } else {
                                let prefix = text.get().selected;
                                view! {
                                    <p class="mt-2 text-sm text-base-content/75">
                                        {prefix} {names.join(", ")}
                                    </p>
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </div>
            </div>

            // The picker the zone opens: visually hidden, out of the tab order
            // and the accessibility tree (the zone is the control), and a
            // sibling of the zone so no interactive control nests in another.
            <input
                node_ref=input_ref
                type="file"
                class="sr-only"
                tabindex="-1"
                aria-hidden="true"
                multiple=move || multiple.get()
                accept=move || {
                    let a = accept.get();
                    if a.is_empty() { None } else { Some(a) }
                }

                disabled=disabled
                on:change=move |ev| {
                    let input = event_target::<HtmlInputElement>(&ev);
                    take(files_of(input.files()));
                    // So choosing the same file again fires `change` again.
                    input.set_value("");
                }
            />
        </div>
    }
}
