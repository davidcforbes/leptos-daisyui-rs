/// Every string `ImageAttachmentField` renders. Built by struct literal;
/// `{filename}`, `{max_mb}` and `{max}` are substituted at render time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageAttachmentTexts {
    /// Accessible name of the drop zone.
    pub label: String,
    /// Hint text shown inside the drop zone.
    pub drop_hint: String,
    /// The file-picker button's label.
    pub browse: String,
    /// Accessible name of a preview's remove button; `{filename}` substituted.
    pub remove: String,
    /// Announced when a file's type is not accepted; `{filename}` substituted.
    pub rejected_type: String,
    /// Announced when a file is too large; `{filename}`/`{max_mb}` substituted.
    pub rejected_size: String,
    /// Announced when the count cap is hit; `{max}` substituted.
    pub rejected_count: String,
    /// Announced when reading a file failed; `{filename}` substituted.
    pub read_failed: String,
}

impl Default for ImageAttachmentTexts {
    fn default() -> Self {
        Self {
            label: "Screenshots".into(),
            drop_hint: "Paste, drop or choose PNG, JPEG or WebP images".into(),
            browse: "Choose files".into(),
            remove: "Remove {filename}".into(),
            rejected_type: "{filename} is not a PNG, JPEG or WebP image".into(),
            rejected_size: "{filename} is larger than {max_mb} MB".into(),
            rejected_count: "At most {max} images".into(),
            read_failed: "Could not read {filename}".into(),
        }
    }
}
