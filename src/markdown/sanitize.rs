//! URL sanitization for rendered markdown links.
//!
//! As of em-vk40, `editmark_core::render_html` is `set_inner_html`-safe
//! by contract — raw HTML fragments are scrubbed through
//! `editmark_core::sanitize_html` and every anchor / image URL is
//! routed through the same scheme allow-list this module exposes.
//!
//! `is_safe_href` lives here for backwards compatibility with existing
//! DOM-post-processing code (`view.rs::process_links` and any external
//! consumer); it forwards to the core implementation so there's a
//! single source of truth.

pub use editmark_core::is_safe_href;

#[cfg(test)]
mod tests {
    use super::is_safe_href;

    #[test]
    fn allows_http() {
        assert!(is_safe_href("http://example.com"));
        assert!(is_safe_href("https://example.com/x?y=1"));
    }

    #[test]
    fn allows_relative_and_fragment() {
        assert!(is_safe_href("/path/to/page"));
        assert!(is_safe_href("./relative"));
        assert!(is_safe_href("#anchor"));
        assert!(is_safe_href(""));
    }

    #[test]
    fn allows_wiki_schemes() {
        assert!(is_safe_href("sec_abc123"));
        assert!(is_safe_href("doc_42"));
    }

    #[test]
    fn rejects_javascript() {
        assert!(!is_safe_href("javascript:alert(1)"));
        assert!(!is_safe_href("JavaScript:alert(1)"));
        assert!(!is_safe_href("  javascript:alert(1)"));
    }

    #[test]
    fn rejects_other_code_schemes() {
        assert!(!is_safe_href("vbscript:msgbox"));
        assert!(!is_safe_href("data:text/html,<script>x</script>"));
        assert!(!is_safe_href("file:///etc/passwd"));
    }
}
