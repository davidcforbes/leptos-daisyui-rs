//! Document outline / table of contents.
//!
//! `<DocOutline source=Signal<String> />` renders a clickable list of all
//! headings in the markdown source.  Each entry's href points at the
//! `#slug` anchor that `editmark_core::render_html` puts on rendered
//! headings, so a default click navigates the same-page rendered preview
//! to that section.
//!
//! Consumers can intercept clicks via `on_heading_click` if they want to
//! do custom navigation (animated scroll, in-app routing, etc.).

use editmark_core::{build_layout, runs_plain_text, FixedTextMeasure, NodeKind};
use leptos::ev;
use leptos::prelude::*;

/// One heading extracted from the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    /// Heading level (1–6).
    pub level: u8,
    /// Plain text of the heading (markup stripped).
    pub text: String,
    /// URL-safe slug that matches the `id=` attribute
    /// `editmark_core::render_html` emits on the heading element.
    pub slug: String,
}

/// Extract all headings from a markdown source string.
pub fn extract_headings(source: &str) -> Vec<OutlineEntry> {
    let measure = FixedTextMeasure::default();
    let nodes = build_layout(source, &measure, 900.0);
    let mut out = Vec::new();
    for node in nodes {
        if let NodeKind::Heading { level, text } = &node.kind {
            let plain = runs_plain_text(text);
            let slug = heading_slug(&plain);
            out.push(OutlineEntry {
                level: *level,
                text: plain,
                slug,
            });
        }
    }
    out
}

/// URL-safe slug — matches `editmark_core::html_render::heading_slug` so
/// the outline's `href="#slug"` lands on the rendered heading's `id`.
fn heading_slug(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-', "")
        .replace(' ', "-")
}

#[component]
pub fn DocOutline(
    /// Reactive markdown source.
    #[prop(into)]
    source: Signal<String>,
    /// Optional click-intercept.  When set, fires with the clicked entry
    /// and suppresses the browser's default anchor navigation.
    #[prop(optional, into)]
    on_heading_click: Option<Callback<OutlineEntry>>,
    /// Optional title to render above the list (e.g. `"Contents"`).
    /// Omit (empty) to show no title.
    #[prop(optional)]
    title: &'static str,
) -> impl IntoView {
    let entries = Signal::derive(move || extract_headings(&source.get()));
    let title_str = title;
    let show_title = !title.is_empty();

    let click_cb = on_heading_click;

    view! {
        <nav class="lds-outline">
            <Show when=move || show_title>
                <div class="lds-outline-title">{title_str}</div>
            </Show>
            <ul class="lds-outline-list">
                <For
                    each=move || entries.get()
                    key=|e| format!("{}-{}", e.level, e.slug)
                    children=move |e: OutlineEntry| {
                        let entry_for_click = e.clone();
                        let level_class = format!("lds-outline-h{}", e.level);
                        let href = format!("#{}", e.slug);
                        let text = e.text.clone();
                        view! {
                            <li class=level_class>
                                <a
                                    href=href
                                    on:click=move |ev: ev::MouseEvent| {
                                        if let Some(cb) = click_cb {
                                            ev.prevent_default();
                                            cb.run(entry_for_click.clone());
                                        }
                                    }
                                >
                                    {text}
                                </a>
                            </li>
                        }
                    }
                />
            </ul>
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_headings_basic() {
        let src = "# Intro\n\nbody\n\n## Details\n\n### Example\n";
        let h = extract_headings(src);
        assert_eq!(h.len(), 3);
        assert_eq!(h[0].level, 1);
        assert_eq!(h[0].text, "Intro");
        assert_eq!(h[0].slug, "intro");
        assert_eq!(h[1].level, 2);
        assert_eq!(h[1].slug, "details");
        assert_eq!(h[2].level, 3);
    }

    #[test]
    fn extract_headings_with_markup() {
        let src = "# Hello **bold** *italic* `code`\n";
        let h = extract_headings(src);
        assert_eq!(h.len(), 1);
        // Plain-text extraction strips the inline formatting markers.
        assert_eq!(h[0].text, "Hello bold italic code");
        assert_eq!(h[0].slug, "hello-bold-italic-code");
    }

    #[test]
    fn extract_headings_special_chars() {
        let src = "# Section 1: Data Management (2024)\n";
        let h = extract_headings(src);
        assert_eq!(h[0].slug, "section-1-data-management-2024");
    }

    #[test]
    fn extract_headings_empty_source() {
        assert_eq!(extract_headings("").len(), 0);
        assert_eq!(extract_headings("no headings here\n").len(), 0);
    }

    #[test]
    fn slug_matches_render_html() {
        // The desktop / browser renderer's heading_slug must match the
        // outline's slug for in-page navigation to work.  Sample case
        // that exercises lowercase, punctuation strip, space-to-hyphen.
        assert_eq!(heading_slug("Hello, World! (2024)"), "hello-world-2024");
        assert_eq!(heading_slug("API Reference"), "api-reference");
        assert_eq!(heading_slug("Already-Hyphenated"), "already-hyphenated");
    }
}
