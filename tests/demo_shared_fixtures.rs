//! ldui-rz43: the demo's badge, button and alert variant sections render the
//! shared fixtures in `ldui_design::fixtures`, which 4iiz-kit's gallery
//! renders too, so pixelproof-parity compares identical inputs rendered by
//! two component sets.
//!
//! Source-level, like `src/patterns/ld_pattern_tests.rs`, and not by
//! choice: an example hard-coded back into the demo renders the same markup
//! as the fixture entry it replaced, so no DOM check can see the drift until
//! the fixture changes and only one renderer follows. What this reads:
//!
//! - each page invokes its fixture-section component once per section of the
//!   fixture module's `SECTIONS`, in that order, naming the fixture constant;
//! - no hand-written section on the page reuses a fixture section's title;
//! - the fixture-section component maps every example (`.examples.iter().map(`),
//!   renders exactly one component element per example, and names no variant
//!   and no string literal, so it cannot carry a hard-coded example.
//!
//! Registered as the `test-demo-fixtures` gate step: `test-lib` runs the
//! library's unit tests only, so an integration test not named in the gate
//! never runs.

use ldui_design::fixtures::{FixtureSection, alert, badge, button};
use std::path::Path;

/// One demo page whose variant sections come from a fixture module.
struct Page {
    /// Repo-relative path of the demo page.
    file: &'static str,
    /// The page's fixture-section component.
    renderer: &'static str,
    /// The library component the renderer emits once per example.
    element: &'static str,
    /// Variant-enum paths an example could hard-code.
    enums: &'static [&'static str],
    /// `(fixture constant, section title)` for every section of the fixture
    /// module's `SECTIONS`, in order.
    sections: Vec<(&'static str, &'static str)>,
}

fn titled<T>(
    names: &[&'static str],
    sections: &[FixtureSection<T>],
) -> Vec<(&'static str, &'static str)> {
    assert_eq!(
        names.len(),
        sections.len(),
        "the table below must name every constant in the fixture module's SECTIONS"
    );
    names
        .iter()
        .copied()
        .zip(sections.iter().map(|s| s.title))
        .collect()
}

fn pages() -> [Page; 3] {
    [
        Page {
            file: "demo/src/demos/badge.rs",
            renderer: "BadgeFixtureSection",
            element: "Badge",
            enums: &["BadgeColor::", "BadgeStyle::", "BadgeSize::"],
            sections: titled(&["COLORS", "SIZES", "STYLES"], &badge::SECTIONS),
        },
        Page {
            file: "demo/src/demos/button.rs",
            renderer: "ButtonFixtureSection",
            element: "Button",
            enums: &[
                "ButtonColor::",
                "ButtonStyle::",
                "ButtonSize::",
                "ButtonShape::",
                "ButtonExampleState::",
            ],
            sections: titled(
                &["COLORS", "SIZES", "STYLES", "STATES", "SHAPES"],
                &button::SECTIONS,
            ),
        },
        Page {
            file: "demo/src/demos/alert.rs",
            renderer: "AlertFixtureSection",
            element: "Alert",
            enums: &["AlertColor::", "AlertStyle::"],
            sections: titled(&["COLORS", "STYLES"], &alert::SECTIONS),
        },
    ]
}

/// The fixture constants a page's `<{renderer} section=fixtures::NAME />`
/// invocations name, in source order.
fn invoked_sections<'a>(source: &'a str, renderer: &str) -> Vec<&'a str> {
    let open = format!("<{renderer}");
    let mut names = Vec::new();
    let mut rest = source;
    while let Some(at) = rest.find(&open) {
        rest = &rest[at + open.len()..];
        let tag = &rest[..rest.find('>').unwrap_or(rest.len())];
        let name = tag
            .split_once("section=fixtures::")
            .map(|(_, after)| {
                let end = after
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(after.len());
                &after[..end]
            })
            .unwrap_or("<not a fixture constant>");
        names.push(name);
    }
    names
}

/// The renderer's body: from its `fn` line to the closing brace in column 0,
/// minus whole-line comments.
fn renderer_body(source: &str, renderer: &str) -> Option<String> {
    let start = format!("fn {renderer}(");
    let mut lines = source
        .lines()
        .skip_while(|line| !line.trim_start().starts_with(&start));
    let mut body = vec![lines.next()?];
    for line in lines {
        if line.trim_end() == "}" {
            break;
        }
        if !line.trim_start().starts_with("//") {
            body.push(line);
        }
    }
    Some(body.join("\n"))
}

/// Opening tags of `element` in `text` (`<Badge` followed by whitespace, `>`
/// or `/`, so `<BadgeFixtureSection` does not count).
fn element_count(text: &str, element: &str) -> usize {
    let open = format!("<{element}");
    text.match_indices(&open)
        .filter(|(at, _)| {
            text[at + open.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_whitespace() || c == '>' || c == '/')
        })
        .count()
}

/// Every way `source` breaks the shared-fixture contract for `page`.
fn violations(page: &Page, source: &str) -> Vec<String> {
    let mut found = Vec::new();

    let expected: Vec<&str> = page.sections.iter().map(|(name, _)| *name).collect();
    let invoked = invoked_sections(source, page.renderer);
    if invoked != expected {
        found.push(format!(
            "the page must render `<{} section=fixtures::NAME />` once per fixture section, \
             in order: expected {expected:?}, found {invoked:?}",
            page.renderer
        ));
    }

    for (_, title) in &page.sections {
        let literal = format!("title=\"{title}\"");
        if source.contains(&literal) {
            found.push(format!(
                "a hand-written section reuses the fixture title {title:?} ({literal})"
            ));
        }
    }

    match renderer_body(source, page.renderer) {
        None => found.push(format!("no `fn {}(` on the page", page.renderer)),
        Some(body) => {
            let squashed: String = body.split_whitespace().collect();
            if !squashed.contains(".examples.iter().map(") {
                found.push(format!(
                    "`{}` must map every example (`.examples.iter().map(`), not a filtered or \
                     truncated list",
                    page.renderer
                ));
            }
            let elements = element_count(&body, page.element);
            if elements != 1 {
                found.push(format!(
                    "`{}` renders {elements} `<{}>` elements; exactly one per example, from the \
                     fixture, is allowed",
                    page.renderer, page.element
                ));
            }
            if body.contains('"') {
                found.push(format!(
                    "`{}` contains a string literal: labels, titles and classes come from the \
                     fixture",
                    page.renderer
                ));
            }
            for path in page.enums {
                if body.contains(path) {
                    found.push(format!(
                        "`{}` names `{path}`: variants come from the fixture",
                        page.renderer
                    ));
                }
            }
        }
    }
    found
}

fn read(file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn demo_sections_use_shared_fixtures() {
    let mut failures = Vec::new();
    for page in pages() {
        for violation in violations(&page, &read(page.file)) {
            failures.push(format!("{}: {violation}", page.file));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// The checks have to be able to fail. Each negative control below breaks a
// real page's source in one way a hard-coded example would, and requires a
// violation -- the acceptance BREAK kept as a permanent test.

fn badge_page() -> (Page, String) {
    let page = pages().into_iter().next().expect("badge page");
    let source = read(page.file);
    (page, source)
}

#[test]
fn a_hard_coded_badge_in_the_fixture_renderer_is_caught() {
    let (page, source) = badge_page();
    let broken = source.replacen(
        "                .collect_view()}",
        "                .collect_view()}\n            <Badge color=BadgeColor::Primary>\"Primary\"</Badge>",
        1,
    );
    assert_ne!(broken, source, "the break must land");
    let found = violations(&page, &broken);
    assert!(
        found.iter().any(|v| v.contains("BadgeColor::"))
            && found.iter().any(|v| v.contains("2 `<Badge>`")),
        "{found:?}"
    );
}

#[test]
fn a_fixture_section_written_back_by_hand_is_caught() {
    let (page, source) = badge_page();
    let broken = source.replacen(
        "<BadgeFixtureSection section=fixtures::COLORS />",
        "<Section row=true title=\"Colors\">\n                <Badge>\"Default\"</Badge>\n            </Section>",
        1,
    );
    assert_ne!(broken, source, "the break must land");
    let found = violations(&page, &broken);
    assert!(
        found
            .iter()
            .any(|v| v.contains("expected [\"COLORS\", \"SIZES\", \"STYLES\"]"))
            && found
                .iter()
                .any(|v| v.contains("reuses the fixture title \"Colors\"")),
        "{found:?}"
    );
}

#[test]
fn a_truncated_example_list_is_caught() {
    let (page, source) = badge_page();
    let broken = source.replacen(".iter()", ".iter()\n                .skip(1)", 1);
    assert_ne!(broken, source, "the break must land");
    let found = violations(&page, &broken);
    assert!(
        found
            .iter()
            .any(|v| v.contains("not a filtered or truncated list")),
        "{found:?}"
    );
}

#[test]
fn a_reordered_section_is_caught() {
    let (page, source) = badge_page();
    let broken = source
        .replacen("fixtures::COLORS />", "fixtures::TMP />", 1)
        .replacen("fixtures::SIZES />", "fixtures::COLORS />", 1)
        .replacen("fixtures::TMP />", "fixtures::SIZES />", 1);
    assert_ne!(broken, source, "the break must land");
    assert!(!violations(&page, &broken).is_empty());
}
