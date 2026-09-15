//! Pure admission rules: what may enter the attachment list.

use crate::patterns::ImageAttachment;
use crate::utils::sniff_content_type;

/// Limits on how many images, and how large, `ImageAttachmentField` admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageAttachmentCaps {
    /// Maximum number of images.
    pub max_count: usize,
    /// Maximum size of one image, in bytes.
    pub max_bytes: usize,
}

impl Default for ImageAttachmentCaps {
    fn default() -> Self {
        Self {
            max_count: 5,
            max_bytes: 5 * 1024 * 1024,
        }
    }
}

/// Why a file was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageRejection {
    /// The bytes did not sniff as PNG, JPEG or WebP.
    UnsupportedType {
        /// The file's name.
        filename: String,
    },
    /// The file exceeded [`ImageAttachmentCaps::max_bytes`].
    TooLarge {
        /// The file's name.
        filename: String,
        /// The cap that was exceeded.
        max_bytes: usize,
    },
    /// The list already holds [`ImageAttachmentCaps::max_count`] images.
    TooMany {
        /// The cap that was exceeded.
        max_count: usize,
    },
}

/// Admit one file into a list that already holds `existing` images. The
/// content type is sniffed from the bytes; the declared name is kept, or a
/// default is minted for clipboard pastes that carry none.
pub fn admit(
    existing: usize,
    filename: &str,
    bytes: Vec<u8>,
    caps: &ImageAttachmentCaps,
) -> Result<ImageAttachment, ImageRejection> {
    if existing >= caps.max_count {
        return Err(ImageRejection::TooMany {
            max_count: caps.max_count,
        });
    }
    let Some(content_type) = sniff_content_type(&bytes) else {
        return Err(ImageRejection::UnsupportedType {
            filename: filename.to_owned(),
        });
    };
    if bytes.len() > caps.max_bytes {
        return Err(ImageRejection::TooLarge {
            filename: filename.to_owned(),
            max_bytes: caps.max_bytes,
        });
    }
    let filename = if filename.trim().is_empty() {
        format!("pasted-image.{}", &content_type["image/".len()..])
    } else {
        filename.to_owned()
    };
    Ok(ImageAttachment {
        filename,
        content_type: content_type.to_owned(),
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: [u8; 12] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];

    #[test]
    fn admits_a_png_and_uses_the_sniffed_type() {
        let caps = ImageAttachmentCaps::default();
        let a = admit(0, "shot.jpg", PNG.to_vec(), &caps).unwrap();
        assert_eq!(a.content_type, "image/png");
        assert_eq!(a.filename, "shot.jpg");
    }

    #[test]
    fn rejects_type_size_and_count() {
        let caps = ImageAttachmentCaps {
            max_count: 1,
            max_bytes: 16,
        };
        assert_eq!(
            admit(0, "x.txt", b"hello".to_vec(), &caps),
            Err(ImageRejection::UnsupportedType {
                filename: "x.txt".into()
            })
        );
        let big = [PNG.to_vec(), vec![0; 16]].concat();
        assert_eq!(
            admit(0, "big.png", big, &caps),
            Err(ImageRejection::TooLarge {
                filename: "big.png".into(),
                max_bytes: 16
            })
        );
        assert_eq!(
            admit(1, "y.png", PNG.to_vec(), &caps),
            Err(ImageRejection::TooMany { max_count: 1 })
        );
    }

    #[test]
    fn blank_filenames_get_a_default() {
        let a = admit(0, "", PNG.to_vec(), &ImageAttachmentCaps::default()).unwrap();
        assert_eq!(a.filename, "pasted-image.png");
    }
}
