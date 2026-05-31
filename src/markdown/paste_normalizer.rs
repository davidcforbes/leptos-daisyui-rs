//! Paste normalizer for the graphic-mode editor (em-berj.2).
//!
//! The browser's default contenteditable paste pulls whatever the
//! clipboard hands over straight into the DOM — including Word /
//! Google Docs styling soup and, more dangerously, raw `<script>` /
//! `onerror=` payloads from untrusted sources.  This module owns the
//! normalization pipeline that runs *before* the source signal sees a
//! paste:
//!
//! 1. Prefer `text/html` from the clipboard; fall back to `text/plain`.
//! 2. Scrub the HTML through [`editmark_core::sanitize_html_fragment`]
//!    so attacker-authored markup can't smuggle scripts past the
//!    contenteditable.
//! 3. Walk the sanitized HTML through
//!    [`editmark_core::dom_to_markdown`] to produce a markdown
//!    fragment.  Lossy on exotic constructs (tables, math) — that's
//!    acceptable per the em-berj.2 contract; XSS is not.
//! 4. Return the resulting markdown chunk plus a hint about whether
//!    the source was HTML or plain text, so the caller can decide on
//!    whitespace handling.
//!
//! The actual DOM event wiring lives in `graphic_editor.rs` — this
//! module is pure logic so it stays straightforward to unit-test.

use web_sys::ClipboardEvent;

/// The HTML mime type a clipboard payload may carry.
const MIME_HTML: &str = "text/html";
const MIME_PLAIN: &str = "text/plain";

/// Outcome of normalizing a clipboard payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasteResult {
    /// The markdown chunk to splice into the source.  Empty if the
    /// clipboard had nothing usable.
    pub markdown: String,
    /// `true` if the chunk was derived from `text/html` (and therefore
    /// passed through the HTML sanitizer); `false` for the plain-text
    /// fallback or when the clipboard was empty.
    pub from_html: bool,
}

impl PasteResult {
    pub fn is_empty(&self) -> bool {
        self.markdown.is_empty()
    }
}

/// Pull the best representation from the clipboard event and produce a
/// markdown chunk ready to splice through the edit funnel.
///
/// Returns an empty [`PasteResult`] when the event has no usable
/// payload (e.g. the user pasted from a source the browser gated
/// access to).  Callers should treat that as "nothing to do" — the
/// `ev.prevent_default()` they already issued is enough.
pub fn normalize_clipboard_event(ev: &ClipboardEvent) -> PasteResult {
    let Some(clipboard) = ev.clipboard_data() else {
        return PasteResult {
            markdown: String::new(),
            from_html: false,
        };
    };
    let html = clipboard.get_data(MIME_HTML).unwrap_or_default();
    if !html.trim().is_empty() {
        return PasteResult {
            markdown: normalize_html(&html),
            from_html: true,
        };
    }
    let plain = clipboard.get_data(MIME_PLAIN).unwrap_or_default();
    PasteResult {
        markdown: plain,
        from_html: false,
    }
}

/// Sanitize a raw HTML fragment and convert it to markdown.  Public
/// so the wiring layer can test the round-trip without a real DOM
/// `ClipboardEvent`.
pub fn normalize_html(html: &str) -> String {
    // Strip Office / Google Docs MS-prefixed cruft — the meta /
    // namespace tags that prefix most copy-from-Word payloads aren't
    // hostile but they aren't useful either, and our sanitizer turns
    // them into HTML-escaped noise that dom_to_markdown then drags
    // into the resulting markdown as literal text.  Strip them up
    // front so the markdown stays clean.
    let trimmed = strip_office_preamble(html);
    let safe = editmark_core::sanitize_html_fragment(&trimmed);
    editmark_core::dom_to_markdown(&safe)
}

/// Drop the `<html>`, `<head>`, `<meta>`, `<style>`, and
/// `<!--StartFragment-->` cruft Word / Outlook / Google Docs prefix
/// to clipboard HTML.  We're not trying to be exhaustive — the
/// sanitizer downstream already handles unknown tags safely — we
/// just want the resulting markdown to start with the user's
/// content, not a wall of escaped meta tags.
fn strip_office_preamble(html: &str) -> String {
    // Find `<!--StartFragment-->`; if it exists, slice everything
    // after it (up to `<!--EndFragment-->` if present).
    if let Some(start) = html.find("<!--StartFragment-->") {
        let after = &html[start + "<!--StartFragment-->".len()..];
        if let Some(end) = after.find("<!--EndFragment-->") {
            return after[..end].to_string();
        }
        return after.to_string();
    }
    // No fragment markers — drop a leading `<html>...<body>` wrapper
    // if present so we start at the body content.
    if let Some(idx) = html.find("<body") {
        if let Some(close) = html[idx..].find('>') {
            let after_body = &html[idx + close + 1..];
            // Trim a matching `</body>...</html>` tail.
            if let Some(end) = after_body.find("</body>") {
                return after_body[..end].to_string();
            }
            return after_body.to_string();
        }
    }
    html.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_office_start_end_fragment_markers() {
        let html = "<html><head><meta></head><body><!--StartFragment--><p>Hi</p><!--EndFragment--></body></html>";
        let out = strip_office_preamble(html);
        assert_eq!(out, "<p>Hi</p>");
    }

    #[test]
    fn strips_html_body_wrapper_when_no_fragment_markers() {
        let html = "<html><head></head><body><p>Hi</p></body></html>";
        let out = strip_office_preamble(html);
        assert_eq!(out, "<p>Hi</p>");
    }

    #[test]
    fn passes_through_when_no_wrapper() {
        let html = "<p>Hi</p>";
        let out = strip_office_preamble(html);
        assert_eq!(out, "<p>Hi</p>");
    }

    #[test]
    fn html_with_script_yields_safe_markdown() {
        // The `<script>` survives only as HTML-escaped text in the
        // sanitized output, and dom_to_markdown emits it as a literal
        // markdown text — never as live HTML.
        let md = normalize_html("<p>hello <script>alert(1)</script> world</p>");
        assert!(!md.to_lowercase().contains("<script"));
    }

    #[test]
    fn html_with_bold_and_link_round_trips_as_markdown() {
        let md = normalize_html(
            "<p>Hello <strong>bold</strong> and <a href=\"https://example.com\">link</a>.</p>",
        );
        assert!(md.contains("**bold**"), "expected bold markdown in {md:?}");
        assert!(
            md.contains("[link](https://example.com)"),
            "expected link markdown in {md:?}"
        );
    }

    #[test]
    fn javascript_uri_in_paste_link_becomes_hash() {
        let md = normalize_html("<a href=\"javascript:alert(1)\">x</a>");
        assert!(!md.to_lowercase().contains("javascript:"));
    }
}
