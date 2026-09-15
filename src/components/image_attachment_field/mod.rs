//! `ImageAttachmentField`: paste, drop or pick PNG/JPEG/WebP images, sniffed
//! by magic bytes, with removable previews. Shares its "image in a
//! DataTransfer" definition with the Markdown editor via `utils::image_files`.

mod admission;
mod component;
mod texts;

pub use admission::{ImageAttachmentCaps, ImageRejection, admit};
pub use component::*;
pub use texts::ImageAttachmentTexts;
