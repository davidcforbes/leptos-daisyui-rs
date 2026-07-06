//! Small reactive doc-stats bar.
//!
//! `<DocStatsBar source=Signal<String> />` renders a single-line summary
//! of word / character / heading / reading-time counts that updates as
//! the source signal changes.  Counts come from
//! `editmark_core::document_stats` so the desktop and browser surfaces
//! agree on what counts (code-block text excluded from `words`, etc.).
//!
//! The consumer chooses where to mount it — directly under the editor,
//! in a side rail, in a status bar at the bottom of the page.

use editmark_core::{DocumentStats, document_stats};
use leptos::prelude::*;

/// Reactive document-stats line.
///
/// Renders something like:
/// `342 words · 1,820 chars · 5 headings · 2 link(s) · 2 min read`.
#[component]
pub fn DocStatsBar(
    /// The markdown source to summarize.
    #[prop(into)]
    source: Signal<String>,
    /// Optional override for which fields to show.  `None` shows the
    /// default field set.
    #[prop(optional)]
    fields: Option<DocStatsFields>,
) -> impl IntoView {
    let fields = fields.unwrap_or_default();
    let stats = Signal::derive(move || document_stats(&source.get()));

    let label = move || format_stats_line(&stats.get(), fields);

    view! {
        <div class="lds-doc-stats">{label}</div>
    }
}

/// Which fields the `<DocStatsBar>` should display.  Each `true` field
/// contributes a segment to the rendered line (separated by `·`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocStatsFields {
    /// Show the word count.
    pub words: bool,
    /// Show the character count.
    pub chars: bool,
    /// Show the heading count.
    pub headings: bool,
    /// Show the link count.
    pub links: bool,
    /// Show the image count.
    pub images: bool,
    /// Show the count of code lines.
    pub code_lines: bool,
    /// Show the estimated reading time in minutes.
    pub reading_minutes: bool,
}

impl Default for DocStatsFields {
    /// Words + chars + headings + links + reading time.  Images and
    /// code-lines are off by default — they're useful but cluttered
    /// for the common "how long is this doc" question.
    fn default() -> Self {
        Self {
            words: true,
            chars: true,
            headings: true,
            links: true,
            images: false,
            code_lines: false,
            reading_minutes: true,
        }
    }
}

impl DocStatsFields {
    /// Compact preset: just words and reading time.
    pub fn compact() -> Self {
        Self {
            words: true,
            chars: false,
            headings: false,
            links: false,
            images: false,
            code_lines: false,
            reading_minutes: true,
        }
    }

    /// All fields enabled.
    pub fn full() -> Self {
        Self {
            words: true,
            chars: true,
            headings: true,
            links: true,
            images: true,
            code_lines: true,
            reading_minutes: true,
        }
    }
}

fn format_stats_line(stats: &DocumentStats, fields: DocStatsFields) -> String {
    let mut parts: Vec<String> = Vec::new();
    if fields.words {
        parts.push(format!("{} words", group_thousands(stats.words)));
    }
    if fields.chars {
        parts.push(format!("{} chars", group_thousands(stats.chars)));
    }
    if fields.headings {
        parts.push(plural_label(stats.headings, "heading", "headings"));
    }
    if fields.links {
        parts.push(plural_label(stats.links, "link", "links"));
    }
    if fields.images {
        parts.push(plural_label(stats.images, "image", "images"));
    }
    if fields.code_lines {
        parts.push(format!("{} code lines", group_thousands(stats.code_lines)));
    }
    if fields.reading_minutes {
        parts.push(format!("{} min read", stats.reading_minutes()));
    }
    parts.join(" · ")
}

fn plural_label(n: usize, singular: &str, plural: &str) -> String {
    if n == 1 {
        format!("1 {singular}")
    } else {
        format!("{} {}", group_thousands(n), plural)
    }
}

/// Insert thousands separators (US-style commas) into `n`.
#[allow(clippy::manual_is_multiple_of)]
fn group_thousands(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        let rem = bytes.len() - i;
        if i > 0 && rem % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_thousands_basic() {
        assert_eq!(group_thousands(0), "0");
        assert_eq!(group_thousands(42), "42");
        assert_eq!(group_thousands(999), "999");
        assert_eq!(group_thousands(1_000), "1,000");
        assert_eq!(group_thousands(12_345), "12,345");
        assert_eq!(group_thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn plural_label_singular_vs_plural() {
        assert_eq!(plural_label(0, "link", "links"), "0 links");
        assert_eq!(plural_label(1, "link", "links"), "1 link");
        assert_eq!(plural_label(2, "link", "links"), "2 links");
        assert_eq!(plural_label(1_500, "link", "links"), "1,500 links");
    }

    #[test]
    fn format_stats_default_fields() {
        let stats = DocumentStats {
            words: 342,
            chars: 1820,
            headings: 5,
            links: 2,
            images: 0,
            code_lines: 0,
        };
        let line = format_stats_line(&stats, DocStatsFields::default());
        assert!(line.contains("342 words"));
        assert!(line.contains("1,820 chars"));
        assert!(line.contains("5 headings"));
        assert!(line.contains("2 links"));
        assert!(line.contains("2 min read"));
        // Images / code lines off by default.
        assert!(!line.contains("images"));
        assert!(!line.contains("code lines"));
    }

    #[test]
    fn format_stats_compact() {
        let stats = DocumentStats {
            words: 100,
            chars: 500,
            headings: 1,
            links: 0,
            images: 0,
            code_lines: 0,
        };
        let line = format_stats_line(&stats, DocStatsFields::compact());
        assert_eq!(line, "100 words · 1 min read");
    }

    #[test]
    fn format_stats_full_includes_everything() {
        let stats = DocumentStats {
            words: 100,
            chars: 500,
            headings: 3,
            links: 2,
            images: 1,
            code_lines: 12,
        };
        let line = format_stats_line(&stats, DocStatsFields::full());
        assert!(line.contains("100 words"));
        assert!(line.contains("500 chars"));
        assert!(line.contains("3 headings"));
        assert!(line.contains("2 links"));
        assert!(line.contains("1 image"));
        assert!(line.contains("12 code lines"));
        assert!(line.contains("min read"));
    }
}
