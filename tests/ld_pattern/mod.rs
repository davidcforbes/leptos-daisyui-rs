//! Shared probe for the ldui-wra5 `data-ld-pattern` browser proof
//! (`ld_pattern_smoke.rs` on the demo, `ld_pattern_snapshot_smoke.rs` on the
//! client-snapshot test host; one xtask lane, `cargo xtask test-ld-pattern`,
//! runs both).
//!
//! pixelproof-parity's recogniser trusts `data-ld-pattern` over every
//! heuristic, so the attribute has to sit on the pattern's ROOT element in
//! the real DOM: on a wrapper or a child, the recogniser classifies the
//! wrong box. `src/patterns/ld_pattern_tests.rs` checks the component
//! source; this checks what Leptos actually rendered.

use super::common;
use pixelproof_web::Harness;
use serde_json::{Value, json};

/// Every pixelproof-parity catalogue pattern, the selector for the root
/// element of the component that declares it, and whether that selector is
/// a hook ONLY that component emits (so every match must declare the
/// pattern). `div.card`-style class selectors are not exclusive: raw daisyUI
/// markup carries the same classes.
pub const ROOTS: [(&str, &str, bool); 14] = [
    ("PageHeader", r#"header[data-page-header="true"]"#, true),
    ("KpiStrip", r#"div[data-kpi-strip-container="true"]"#, true),
    (
        "SnapshotTable",
        r#"section[data-snapshot-table-page="true"]"#,
        true,
    ),
    ("FilterBar", "section[data-filter-bar]", true),
    ("ListPage", r#"div[data-list-page="true"]"#, true),
    (
        "AsyncDataSection",
        r#"section[data-async-data-section="true"]"#,
        true,
    ),
    (
        "SectionHeading",
        r#"div[data-section-heading="true"]"#,
        true,
    ),
    ("PageStatePanel", "[data-page-state-panel]", true),
    ("Card", "div.card", false),
    (
        "DataTable",
        r#"div[data-table-data-mode="compatibility-client"]"#,
        true,
    ),
    ("Stats", "div.stats", false),
    ("Tabs", "div.tabs[data-tab-mode]", true),
    ("Dialog", "dialog.modal[data-modal-close-mode]", true),
    ("Badge", "div.badge", false),
];

/// Which trunk entry point serves a page.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Host {
    /// The general demo app (`index.html`).
    Demo,
    /// `client-snapshot-test-host.html`, where `SnapshotTablePage` lives.
    ClientSnapshot,
}

/// A page and the patterns it must render, declared, at rest.
pub struct PatternPage {
    pub host: Host,
    pub route: &'static str,
    pub patterns: &'static [&'static str],
    /// The `data-page-header-navigation-layout` a declared `PageHeader` on
    /// this page must carry: `PageHeader` has one root per layout, and each
    /// must be proven.
    pub page_header_layout: Option<&'static str>,
}

pub const PAGES: [PatternPage; 11] = [
    PatternPage {
        host: Host::Demo,
        route: "/components/badge",
        patterns: &["Badge"],
        page_header_layout: None,
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/card",
        patterns: &["Card"],
        page_header_layout: None,
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/data-table",
        patterns: &["DataTable"],
        page_header_layout: None,
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/stats",
        patterns: &["Stats"],
        page_header_layout: None,
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/tab",
        patterns: &["Tabs"],
        page_header_layout: None,
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/modal",
        patterns: &["Dialog"],
        page_header_layout: None,
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/kpi_strip",
        patterns: &["KpiStrip"],
        page_header_layout: None,
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/section_heading",
        patterns: &["SectionHeading"],
        page_header_layout: None,
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/client-snapshot-list",
        patterns: &["ListPage", "PageHeader", "FilterBar", "AsyncDataSection"],
        page_header_layout: Some("inline-responsive"),
    },
    PatternPage {
        host: Host::ClientSnapshot,
        route: "/components/snapshot-table-page",
        patterns: &["SnapshotTable", "PageHeader"],
        page_header_layout: Some("dedicated-row"),
    },
    PatternPage {
        host: Host::Demo,
        route: "/components/page_quick_actions",
        patterns: &["PageHeader"],
        page_header_layout: Some("on-divider"),
    },
];

/// Patterns only an interaction renders: `PageStatePanel` appears in the
/// search-picker dialog once a search comes back empty (its status root)
/// or fails (its error root). `ld_pattern_smoke.rs` drives that journey.
pub const JOURNEY_PATTERNS: [&str; 1] = ["PageStatePanel"];

/// Every catalogue pattern is proven somewhere: at rest on a page, or by a
/// journey. A pattern added to `ROOTS` with no page proves nothing.
pub fn assert_catalogue_covered() {
    let mut covered: Vec<&str> = PAGES
        .iter()
        .flat_map(|page| page.patterns.iter().copied())
        .chain(JOURNEY_PATTERNS)
        .collect();
    covered.sort_unstable();
    covered.dedup();
    let mut catalogue: Vec<&str> = ROOTS.iter().map(|(name, _, _)| *name).collect();
    catalogue.sort_unstable();
    assert_eq!(
        covered, catalogue,
        "every catalogue pattern needs a page (or journey) that proves it"
    );
}

fn root_selector(pattern: &str) -> &'static str {
    ROOTS
        .iter()
        .find(|(name, _, _)| *name == pattern)
        .map(|(_, selector, _)| *selector)
        .unwrap_or_else(|| panic!("{pattern} is not a catalogue pattern"))
}

/// Everything the page says about `data-ld-pattern`:
///
/// - `counts`: declarations per pattern name;
/// - `misplaced`: declarations on an element that is not that pattern's
///   root (or of a name outside the catalogue);
/// - `undeclared`: elements an exclusive root selector matches that do not
///   declare the pattern;
/// - `layouts`: each declared `PageHeader`'s navigation layout;
/// - `panels`: each declared `PageStatePanel`'s tag and state slug.
pub async fn probe(h: &Harness) -> Value {
    let roots = json!(ROOTS);
    let script = format!(
        r#"(() => {{
            const roots = {roots};
            const selectorOf = Object.fromEntries(roots.map(([name, selector]) => [name, selector]));
            const describe = el => {{
                const classes = (el.getAttribute('class') || '').trim().split(/\s+/).filter(Boolean).slice(0, 4);
                return [el.tagName.toLowerCase(), ...classes].join('.');
            }};
            const counts = {{}};
            const misplaced = [];
            for (const el of document.querySelectorAll('[data-ld-pattern]')) {{
                const name = el.getAttribute('data-ld-pattern');
                counts[name] = (counts[name] || 0) + 1;
                const selector = selectorOf[name];
                if (!selector || !el.matches(selector)) misplaced.push({{ name, element: describe(el) }});
            }}
            const undeclared = [];
            for (const [name, selector, exclusive] of roots) {{
                if (!exclusive) continue;
                for (const el of document.querySelectorAll(selector)) {{
                    if (el.getAttribute('data-ld-pattern') !== name) {{
                        undeclared.push({{
                            name,
                            element: describe(el),
                            declared: el.getAttribute('data-ld-pattern'),
                        }});
                    }}
                }}
            }}
            const layouts = [...document.querySelectorAll('[data-ld-pattern="PageHeader"]')]
                .map(el => el.getAttribute('data-page-header-navigation-layout'));
            const panels = [...document.querySelectorAll('[data-ld-pattern="PageStatePanel"]')]
                .map(el => ({{ tag: el.tagName.toLowerCase(), slug: el.getAttribute('data-page-state-panel') }}));
            return {{ counts, misplaced, undeclared, layouts, panels }};
        }})()"#
    );
    h.page()
        .evaluate(script.as_str())
        .await
        .expect("evaluate data-ld-pattern probe")
        .into_value()
        .expect("data-ld-pattern probe returns JSON")
}

/// No declaration off its root and no exclusive root left undeclared.
pub fn assert_roots_consistent(report: &Value, context: &str) {
    assert_eq!(
        report["misplaced"],
        json!([]),
        "{context}: data-ld-pattern on an element that is not the pattern's root: {report}"
    );
    assert_eq!(
        report["undeclared"],
        json!([]),
        "{context}: a pattern root rendered without its data-ld-pattern: {report}"
    );
}

/// Load `page` on the current host, wait for each expected root, and prove
/// every expected pattern is declared on its root and nothing is declared
/// anywhere else.
pub async fn check_page(page: &PatternPage) {
    let h = common::harness_at(page.route).await;
    common::begin_browser_error_capture(&h).await;
    for pattern in page.patterns {
        common::wait_for_selector(&h, root_selector(pattern)).await;
    }
    let report = probe(&h).await;
    assert_roots_consistent(&report, page.route);
    for pattern in page.patterns {
        let count = report["counts"][*pattern].as_u64().unwrap_or(0);
        assert!(
            count >= 1,
            "{}: no element declares data-ld-pattern=\"{pattern}\": {report}",
            page.route
        );
    }
    if let Some(layout) = page.page_header_layout {
        let layouts = report["layouts"].as_array().cloned().unwrap_or_default();
        assert!(
            layouts.contains(&json!(layout)),
            "{}: expected a declared PageHeader in the {layout} layout: {report}",
            page.route
        );
    }
    common::assert_no_browser_errors(&h, page.route).await;
}

/// Run every page of `host`, after proving the page table covers the whole
/// catalogue.
pub async fn check_host(host: Host) {
    assert_catalogue_covered();
    for page in PAGES.iter().filter(|page| page.host == host) {
        check_page(page).await;
    }
}
