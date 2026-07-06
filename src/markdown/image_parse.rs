//! Detect image syntax under a cursor and serialize back.
//!
//! Two forms are recognized:
//! - Markdown:  `![alt](url)` (with optional `"title"`)
//! - HTML tag:  `<img src="url" alt="alt" width="W" height="H" />`
//!
//! The editor uses this to:
//! 1. Decide whether the Image-toolbar button opens an *edit* or *insert*
//!    dialog (depends on whether an image is under the cursor).
//! 2. Pre-fill the dialog from the existing markdown when editing.
//! 3. Round-trip the dialog's output back to source text.
//!
//! Implementation is a small linear scanner — no regex dep, no markdown
//! parser dep beyond what `editmark_core` already pulls in.

use std::ops::Range;

/// Which on-disk form of image markup an [`ImageRef`] came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageForm {
    /// `![alt](url)` — emitted when no dimensions are set.
    Markdown,
    /// `<img src="url" alt="…" width="…" height="…" />` — emitted when
    /// dimensions are set (markdown image syntax has no native sizing).
    HtmlTag,
}

/// One image found in the source, with the range it occupies and the
/// extracted attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRef {
    /// Byte range the image syntax occupies in the source.
    pub range: Range<usize>,
    /// Alt text.
    pub alt: String,
    /// Image URL / `src`.
    pub url: String,
    /// Optional title (markdown `"…"` / HTML `title`).
    pub title: Option<String>,
    /// Optional explicit width (HTML-tag form only).
    pub width: Option<String>,
    /// Optional explicit height (HTML-tag form only).
    pub height: Option<String>,
    /// Which syntax this image uses.
    pub form: ImageForm,
}

/// Find an image syntax that contains or directly abuts the given cursor.
///
/// "Abuts" means cursor == range.start or cursor == range.end — pick those
/// up too, since a freshly-inserted image leaves the cursor at one end.
pub fn find_at_cursor(source: &str, cursor: usize) -> Option<ImageRef> {
    let all = find_all(source);
    all.into_iter()
        .find(|img| img.range.start <= cursor && cursor <= img.range.end)
}

/// All images in `source`, in document order.
pub fn find_all(source: &str) -> Vec<ImageRef> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'!' && bytes.get(i + 1) == Some(&b'[') {
            if let Some(img) = parse_markdown_image(source, i) {
                let end = img.range.end;
                out.push(img);
                i = end;
                continue;
            }
        } else if bytes[i] == b'<' && starts_with_ci(&bytes[i..], b"<img") {
            // Require word boundary after "img".
            let after = i + 4;
            let next = bytes.get(after).copied().unwrap_or(b' ');
            if (next == b' ' || next == b'\t' || next == b'\n' || next == b'>' || next == b'/')
                && let Some(img) = parse_html_image(source, i)
            {
                let end = img.range.end;
                out.push(img);
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Serialize an image back to source.
///
/// Emits `![alt](url)` when both `width` and `height` are `None`; otherwise
/// emits an HTML `<img>` tag with the dimensions.  Title (if present) is
/// only emitted in the markdown form.
pub fn serialize(
    alt: &str,
    url: &str,
    title: Option<&str>,
    width: Option<&str>,
    height: Option<&str>,
) -> String {
    if width.is_none() && height.is_none() {
        match title {
            Some(t) if !t.is_empty() => {
                format!(
                    "![{}]({} \"{}\")",
                    escape_md_alt(alt),
                    escape_md_url(url),
                    escape_md_title(t)
                )
            }
            _ => format!("![{}]({})", escape_md_alt(alt), escape_md_url(url)),
        }
    } else {
        let mut s = String::with_capacity(64 + alt.len() + url.len());
        s.push_str("<img src=\"");
        s.push_str(&escape_html_attr(url));
        s.push_str("\" alt=\"");
        s.push_str(&escape_html_attr(alt));
        s.push('"');
        if let Some(w) = width
            && !w.is_empty()
        {
            s.push_str(" width=\"");
            s.push_str(&escape_html_attr(w));
            s.push('"');
        }
        if let Some(h) = height
            && !h.is_empty()
        {
            s.push_str(" height=\"");
            s.push_str(&escape_html_attr(h));
            s.push('"');
        }
        s.push_str(" />");
        s
    }
}

// -- internals ------------------------------------------------------------

fn parse_markdown_image(source: &str, start: usize) -> Option<ImageRef> {
    // source[start..] begins with `![`.
    let after_open = start + 2;
    // Find the matching `]`. Allow no nesting (basic markdown).
    let rel_close = source[after_open..].find(']')?;
    let alt_end = after_open + rel_close;
    let after_alt = alt_end + 1;
    if source.as_bytes().get(after_alt) != Some(&b'(') {
        return None;
    }
    let after_paren = after_alt + 1;
    // Find the closing `)`. Title (in quotes) may appear inside parentheses.
    // Strategy: scan forward; track whether we're inside `"…"` or `'…'`.
    let bytes = source.as_bytes();
    let mut i = after_paren;
    let mut in_dq = false;
    let mut in_sq = false;
    let mut paren_close = None;
    while i < bytes.len() {
        let b = bytes[i];
        if !in_sq && b == b'"' {
            in_dq = !in_dq;
        } else if !in_dq && b == b'\'' {
            in_sq = !in_sq;
        } else if !in_dq && !in_sq && b == b')' {
            paren_close = Some(i);
            break;
        } else if !in_dq && !in_sq && b == b'\n' {
            // Disallow newlines outside quoted strings — markdown images
            // are single-line.
            return None;
        }
        i += 1;
    }
    let paren_close = paren_close?;
    let inner = &source[after_paren..paren_close];
    let (url, title) = split_url_and_title(inner);
    let alt = source[after_open..alt_end].to_string();
    Some(ImageRef {
        range: start..paren_close + 1,
        alt,
        url,
        title,
        width: None,
        height: None,
        form: ImageForm::Markdown,
    })
}

fn split_url_and_title(inner: &str) -> (String, Option<String>) {
    let trimmed = inner.trim_start();
    let leading = inner.len() - trimmed.len();
    let _ = leading;
    // URL ends at first whitespace (when title follows) or at end.
    let mut url_end = trimmed.len();
    for (idx, c) in trimmed.char_indices() {
        if c.is_whitespace() {
            url_end = idx;
            break;
        }
    }
    let url = trimmed[..url_end].to_string();
    let rest = trimmed[url_end..].trim();
    if rest.is_empty() {
        return (url, None);
    }
    // Title is "..." or '...' or (...) per CommonMark; allow "..." and '...'.
    let title = if (rest.starts_with('"') && rest.ends_with('"'))
        || (rest.starts_with('\'') && rest.ends_with('\''))
    {
        Some(rest[1..rest.len() - 1].to_string())
    } else {
        Some(rest.to_string())
    };
    (url, title)
}

fn parse_html_image(source: &str, start: usize) -> Option<ImageRef> {
    // source[start..] begins with `<img` (case-insensitive).
    // Find the closing `>` (allowing `/>`).
    let rel_close = source[start..].find('>')?;
    let end = start + rel_close + 1;
    let inner = &source[start + 4..start + rel_close]; // attributes section
    let attrs = parse_attributes(inner);

    let url = attrs.get("src").cloned()?;
    let alt = attrs.get("alt").cloned().unwrap_or_default();
    let width = attrs.get("width").cloned();
    let height = attrs.get("height").cloned();
    let title = attrs.get("title").cloned();
    Some(ImageRef {
        range: start..end,
        alt,
        url,
        title,
        width,
        height,
        form: ImageForm::HtmlTag,
    })
}

fn parse_attributes(input: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Skip whitespace and stray slashes (e.g., trailing `/`).
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Read name.
        let name_start = i;
        while i < bytes.len()
            && bytes[i] != b'='
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let name = input[name_start..i].to_ascii_lowercase();
        if name.is_empty() {
            break;
        }
        // Optional `= value`.
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                let v_start = i;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                value = input[v_start..i].to_string();
                if i < bytes.len() {
                    i += 1;
                } // consume closing quote
            } else {
                let v_start = i;
                while i < bytes.len()
                    && !bytes[i].is_ascii_whitespace()
                    && bytes[i] != b'/'
                    && bytes[i] != b'>'
                {
                    i += 1;
                }
                value = input[v_start..i].to_string();
            }
        }
        out.insert(name, decode_html_entities(&value));
    }
    out
}

fn decode_html_entities(s: &str) -> String {
    // Decode the small set we emit ourselves; leave others alone.
    s.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn starts_with_ci(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    haystack
        .iter()
        .zip(needle.iter())
        .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

fn escape_md_alt(s: &str) -> String {
    s.replace('\\', "\\\\").replace(']', "\\]")
}

fn escape_md_url(s: &str) -> String {
    s.replace(')', "%29").replace(' ', "%20")
}

fn escape_md_title(s: &str) -> String {
    s.replace('"', "\\\"")
}

fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_basic_markdown_image() {
        let src = "Hello ![alt](url) world";
        let img = find_all(src);
        assert_eq!(img.len(), 1);
        assert_eq!(img[0].alt, "alt");
        assert_eq!(img[0].url, "url");
        assert_eq!(img[0].form, ImageForm::Markdown);
        assert_eq!(&src[img[0].range.clone()], "![alt](url)");
    }

    #[test]
    fn finds_markdown_image_with_title() {
        let src = r#"![alt](url "the title")"#;
        let img = find_all(src);
        assert_eq!(img.len(), 1);
        assert_eq!(img[0].url, "url");
        assert_eq!(img[0].title.as_deref(), Some("the title"));
    }

    #[test]
    fn finds_html_img_tag() {
        let src = r#"<img src="/v1/assets/123" alt="hi" width="300" />"#;
        let img = find_all(src);
        assert_eq!(img.len(), 1);
        assert_eq!(img[0].url, "/v1/assets/123");
        assert_eq!(img[0].alt, "hi");
        assert_eq!(img[0].width.as_deref(), Some("300"));
        assert_eq!(img[0].form, ImageForm::HtmlTag);
    }

    #[test]
    fn finds_html_img_with_height() {
        let src = r#"<img src="x.png" alt="y" width="100" height="200">"#;
        let img = find_all(src);
        assert_eq!(img.len(), 1);
        assert_eq!(img[0].height.as_deref(), Some("200"));
    }

    #[test]
    fn find_at_cursor_works_inside_alt() {
        let src = "abc ![alt](url) def";
        let img = find_at_cursor(src, 7).expect("cursor inside alt");
        assert_eq!(img.url, "url");
    }

    #[test]
    fn find_at_cursor_returns_none_when_outside() {
        let src = "no images here";
        assert!(find_at_cursor(src, 5).is_none());
    }

    #[test]
    fn serialize_unsized_is_markdown() {
        let s = serialize("alt", "/v1/assets/abc", None, None, None);
        assert_eq!(s, "![alt](/v1/assets/abc)");
    }

    #[test]
    fn serialize_sized_is_html() {
        let s = serialize("alt", "/v1/assets/abc", None, Some("300"), None);
        assert_eq!(s, r#"<img src="/v1/assets/abc" alt="alt" width="300" />"#);
    }

    #[test]
    fn serialize_escapes_attr_quotes() {
        let s = serialize(r#"quote"in"alt"#, "x", None, Some("100"), None);
        assert!(s.contains("&quot;in&quot;"));
    }

    #[test]
    fn round_trip_html_img() {
        let src = r#"<img src="/v1/assets/a1" alt="caption" width="400" height="300" />"#;
        let img = find_all(src).into_iter().next().expect("parsed");
        let out = serialize(
            &img.alt,
            &img.url,
            img.title.as_deref(),
            img.width.as_deref(),
            img.height.as_deref(),
        );
        // Same alt/src/width/height should round-trip.
        let img2 = find_all(&out).into_iter().next().expect("re-parsed");
        assert_eq!(img.url, img2.url);
        assert_eq!(img.alt, img2.alt);
        assert_eq!(img.width, img2.width);
        assert_eq!(img.height, img2.height);
    }

    #[test]
    fn ignores_text_that_looks_like_image_but_isnt() {
        let src = "not an image: ![oops](\nbrokenurl";
        assert_eq!(find_all(src).len(), 0);
    }
}
