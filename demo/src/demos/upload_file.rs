use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// `UploadFile` with its default English and with Spanish strings, and a
/// Spanish `FileDropZone` for PDFs and PNGs whose callbacks write what they
/// received below it. `tests/upload_file_smoke.rs` reads this page
/// (`cargo xtask test-upload-file`, ldui-1ff4).
#[component]
pub fn UploadFileDemo() -> impl IntoView {
    let (english_picked, set_english_picked) = signal(String::new());
    let (received, set_received) = signal(Vec::<String>::new());
    let (rejected, set_rejected) = signal(0usize);

    let spanish = UploadFileText::new("Haga clic para subir", "Seleccionado: ");
    let drop_text = FileDropZoneText::new(
        "Suelte el documento aquí o pulse Intro para elegirlo",
        "Seleccionado: ",
    );
    // Named here: the `view!` parser reads the `<` of a `Vec<File>` type
    // annotation as a tag.
    let on_files = move |files: Vec<web_sys::File>| {
        set_received.update(|r| r.extend(files.iter().map(|f| f.name())));
    };
    let on_rejected = move |files: Vec<web_sys::File>| {
        set_rejected.update(|n| *n += files.len());
    };

    view! {
        <div class="space-y-6">
            <h1 class="text-3xl font-bold">"Upload File"</h1>
            <p class="text-base-content/70">
                "A styled file input and a drag-and-drop zone. Every visible string comes from the caller."
            </p>

            <div id="upload-file-fixture" class="space-y-6">
                <section data-testid="upload-file-english" class="space-y-2">
                    <h2 class="text-xl font-semibold">"Default (English)"</h2>
                    <UploadFile on_upload=move |name: String| set_english_picked.set(name) />
                    <p class="text-sm" data-testid="upload-file-english-picked">
                        {english_picked}
                    </p>
                </section>

                <section data-testid="upload-file-spanish" class="space-y-2">
                    <h2 class="text-xl font-semibold">"Caller-supplied strings (Spanish)"</h2>
                    <UploadFile text=spanish accept="application/pdf".to_string() />
                </section>

                <section data-testid="upload-file-drop" class="space-y-2">
                    <h2 class="text-xl font-semibold">"Drop zone (Spanish, PDF and PNG only)"</h2>
                    <FileDropZone
                        accept="application/pdf,.png".to_string()
                        multiple=true
                        text=drop_text
                        on_files=on_files
                        on_rejected=on_rejected
                    />
                    <ul class="list-disc pl-6 text-sm" data-testid="upload-file-drop-received">
                        {move || {
                            received
                                .get()
                                .into_iter()
                                .map(|name| view! { <li>{name}</li> })
                                .collect_view()
                        }}
                    </ul>
                    <p class="text-sm">
                        "Left out by the accept list: "
                        <span data-testid="upload-file-drop-rejected">{rejected}</span>
                    </p>
                </section>
            </div>
        </div>
    }
}
