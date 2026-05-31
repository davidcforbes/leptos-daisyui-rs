//! Syntax highlighting for code blocks.
//!
//! Wraps `editmark_core::markdown_highlight` for code-block content and emits
//! HTML with span classes that match the CSS in `theme::BASE_STYLES`.
//!
//! This is intentionally a thin layer.  The single source of truth for what
//! a token *is* lives in editmark-core; we just convert its `SyntaxKind`
//! values into CSS class names.

use editmark_core::{markdown_highlight, SyntaxKind, SyntaxSpan};

/// HTML-escape `s` and append it to `out`.
fn push_escaped(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
}

/// Return the CSS class for a token kind, or `None` if the kind shouldn't be
/// painted differently from the surrounding text.
fn class_for_kind(kind: SyntaxKind) -> Option<&'static str> {
    match kind {
        SyntaxKind::Code | SyntaxKind::CodeBlock => None,
        // editmark-core's `markdown_highlight` was written to highlight
        // markdown source, not arbitrary language code.  For language-tagged
        // code blocks we re-purpose it: most of its token kinds map onto
        // sensible code-coloring buckets, the rest fall through as plain.
        SyntaxKind::Heading { .. } => Some("lds-syn-kw"),
        SyntaxKind::Strong => Some("lds-syn-kw"),
        SyntaxKind::Emphasis => Some("lds-syn-ty"),
        SyntaxKind::Strikethrough => Some("lds-syn-com"),
        SyntaxKind::Link => Some("lds-syn-fn"),
        SyntaxKind::LinkUrl => Some("lds-syn-str"),
        SyntaxKind::Image => Some("lds-syn-fn"),
        SyntaxKind::Blockquote => Some("lds-syn-com"),
        SyntaxKind::ListMarker => Some("lds-syn-op"),
        // ListItem spans are paragraph-level (indentation only); the
        // HTML highlighter doesn't render indentation, so emit no class.
        SyntaxKind::ListItem { .. } => None,
        SyntaxKind::Html => Some("lds-syn-pp"),
        _ => None,
    }
}

/// Render `source` as a syntax-highlighted HTML span sequence.
///
/// The implementation is two-pass for clarity: collect spans, then walk the
/// source emitting characters inside or outside spans.  Overlapping spans
/// (markdown can nest emphasis inside strong) are flattened — innermost wins.
pub fn highlight_to_html(source: &str, _lang: &str) -> String {
    let spans = markdown_highlight(source);
    render_spans(source, &spans)
}

fn render_spans(source: &str, spans: &[SyntaxSpan]) -> String {
    // Build a per-byte "class active here" map. innermost-wins: the span
    // with the smallest length covering byte i decides its class.
    let mut active: Vec<Option<&'static str>> = vec![None; source.len()];
    let mut sorted = spans.to_vec();
    // Longest first so shorter inner spans overwrite.
    sorted.sort_by_key(|s| std::cmp::Reverse(s.end - s.start));
    for span in &sorted {
        let Some(class) = class_for_kind(span.kind) else {
            continue;
        };
        let start = span.start.min(source.len());
        let end = span.end.min(source.len());
        for slot in &mut active[start..end] {
            *slot = Some(class);
        }
    }

    let mut out = String::with_capacity(source.len() + 32);
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut current: Option<&'static str> = None;
    while i < source.len() {
        let want = active[i];
        if want != current {
            if current.is_some() {
                out.push_str("</span>");
            }
            if let Some(cls) = want {
                out.push_str("<span class=\"");
                out.push_str(cls);
                out.push_str("\">");
            }
            current = want;
        }
        // Advance by one UTF-8 char.
        let mut end = i + 1;
        while end < bytes.len() && (bytes[end] & 0xC0) == 0x80 {
            end += 1;
        }
        push_escaped(&mut out, &source[i..end]);
        i = end;
    }
    if current.is_some() {
        out.push_str("</span>");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_some_tokens() {
        let html = highlight_to_html("# Heading\n**bold**", "");
        // Heading + Strong both map to lds-syn-kw — at minimum some span
        // emission happens.
        assert!(html.contains("lds-syn-"), "expected a span class: {html}");
    }

    #[test]
    fn plain_text_unstyled() {
        let html = highlight_to_html("just words no markup", "");
        assert!(!html.contains("<span"), "expected no spans: {html}");
        assert_eq!(html, "just words no markup");
    }

    #[test]
    fn escapes_html() {
        let html = highlight_to_html("<script>", "");
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }
}
