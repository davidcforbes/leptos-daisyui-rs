//! One definition of "an image in a DataTransfer", shared by the Markdown
//! editor's paste/drop and `ImageAttachmentField`, plus magic-byte sniffing
//! so a declared MIME type is never trusted.

/// Returns the accepted content type by magic bytes: PNG, JPEG or WebP.
/// Anything else (including GIF, SVG, BMP) is `None`.
pub fn sniff_content_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        Some("image/png")
    } else if bytes.len() >= 3 && bytes[..3] == [0xFF, 0xD8, 0xFF] {
        Some("image/jpeg")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Every `File` in the transfer whose declared type starts with `image/`.
/// The declared type is a pre-filter only; callers sniff the bytes.
pub fn image_files_in(data: &web_sys::DataTransfer) -> Vec<web_sys::File> {
    let mut out = Vec::new();
    let Some(files) = data.files() else {
        return out;
    };
    for i in 0..files.length() {
        if let Some(file) = files.get(i)
            && file.type_().starts_with("image/")
        {
            out.push(file);
        }
    }
    out
}

/// The first image file in the transfer, if any.
pub fn first_image_file(data: &web_sys::DataTransfer) -> Option<web_sys::File> {
    image_files_in(data).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::sniff_content_type;

    #[test]
    fn sniffs_png_jpeg_webp_and_rejects_others() {
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut webp = *b"RIFF\0\0\0\0WEBPVP8 ";
        webp[4] = 1;
        let gif = *b"GIF89a\0\0\0\0\0\0";
        assert_eq!(sniff_content_type(&png), Some("image/png"));
        assert_eq!(sniff_content_type(&jpeg), Some("image/jpeg"));
        assert_eq!(sniff_content_type(&webp), Some("image/webp"));
        assert_eq!(
            sniff_content_type(&gif),
            None,
            "GIF is not an accepted type"
        );
        assert_eq!(sniff_content_type(b"hello"), None);
        assert_eq!(sniff_content_type(&[]), None);
    }
}
