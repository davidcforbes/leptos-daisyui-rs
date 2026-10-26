use super::*;

// The strings a caller does not supply are the English the components
// showed before they were props (ldui-1ff4); the browser lane
// (`cargo xtask test-upload-file`) proves Spanish ones render in their place.
#[test]
fn default_text_is_the_english_the_components_showed_before() {
    let upload = UploadFileText::default();
    assert_eq!(upload.prompt, "Click to upload");
    assert_eq!(upload.selected, "Selected: ");
    let drop = FileDropZoneText::default();
    assert_eq!(drop.prompt, "Drop files here, or press Enter to choose");
    assert_eq!(drop.selected, "Selected: ");
}

#[test]
fn caller_text_is_kept_verbatim() {
    let upload = UploadFileText::new("Haga clic para subir", "Seleccionado: ");
    assert_eq!(upload.prompt, "Haga clic para subir");
    assert_eq!(upload.selected, "Seleccionado: ");
    let drop = FileDropZoneText::new("Suelte el documento aquí", "Seleccionado: ");
    assert_eq!(drop.prompt, "Suelte el documento aquí");
}

#[test]
fn accept_list_grammar_extension_family_and_exact_type() {
    // Exact MIME type, any case.
    assert!(accept_matches(
        "application/pdf",
        "scan.pdf",
        "application/pdf"
    ));
    assert!(accept_matches(
        "application/pdf",
        "scan.PDF",
        "Application/PDF"
    ));
    assert!(!accept_matches(
        "application/pdf",
        "notes.txt",
        "text/plain"
    ));
    // Extension, any case, by the last dot only.
    assert!(accept_matches(".pdf", "scan.PDF", ""));
    assert!(accept_matches(".png", "photo.final.png", "image/png"));
    assert!(!accept_matches(
        ".png",
        "photo.png.exe",
        "application/octet-stream"
    ));
    assert!(!accept_matches(".pdf", "pdf", ""));
    // Type family.
    assert!(accept_matches("image/*", "photo.heic", "image/heic"));
    assert!(!accept_matches("image/*", "clip.mp4", "video/mp4"));
    // A mixed list, with spaces, admits any one match.
    let list = "application/pdf, .png ,image/*";
    assert!(accept_matches(list, "scan.pdf", "application/pdf"));
    assert!(accept_matches(list, "shot.png", ""));
    assert!(accept_matches(list, "pic.webp", "image/webp"));
    assert!(!accept_matches(
        list,
        "sheet.xlsx",
        "application/vnd.ms-excel"
    ));
    // An empty list admits everything.
    assert!(accept_matches(
        "",
        "anything.bin",
        "application/octet-stream"
    ));
    assert!(accept_matches(" , ", "anything.bin", ""));
}

#[test]
fn a_dropped_pdf_is_accepted_and_the_rest_turned_away_in_order() {
    let dropped = vec![
        ("scan.pdf", "application/pdf"),
        ("notes.txt", "text/plain"),
        ("form.PDF", "application/pdf"),
        ("clip.mp4", "video/mp4"),
    ];
    let (accepted, rejected) = partition_by_accept(
        "application/pdf",
        dropped,
        |f| f.0.to_string(),
        |f| f.1.to_string(),
    );
    assert_eq!(
        accepted,
        [
            ("scan.pdf", "application/pdf"),
            ("form.PDF", "application/pdf")
        ]
    );
    assert_eq!(
        rejected,
        [("notes.txt", "text/plain"), ("clip.mp4", "video/mp4")]
    );
}
