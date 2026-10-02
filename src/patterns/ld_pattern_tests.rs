//! ldui-wra5: every pixelproof-parity catalogue pattern declares itself.
//!
//! pixelproof-parity's recogniser reads `data-ld-pattern` before any
//! heuristic (PixelProof `crates/pixelproof-parity/src/recognize.rs`), and a
//! declared catalogue name wins outright, so an ldui page is classified
//! exactly instead of by a class-and-tag guess. 4iiz-kit emits the same
//! values on its counterparts, so the kit's parity lane compares like with
//! like.
//!
//! This crate has no native HTML renderer: `AnyView::to_html` panics unless
//! tachys' `ssr` feature is on, which the library never enables, and every
//! component with `children` renders through `AnyView`. Its native markup
//! checks are therefore source-level (`badge/tests.rs`, the `include_str!`
//! guards in `page_header.rs` and `kpi_strip.rs`), and so is this one: it
//! reads each catalogued component's render code and requires the
//! declaration on the ROOT element — the outermost element of a `view!` —
//! of every branch the component can render, and nowhere else in it.

use std::collections::BTreeSet;

/// The pattern names pixelproof-parity's catalogue carries
/// (PixelProof `crates/pixelproof-parity/src/catalog.rs`). A declared name
/// outside this list does not short-circuit recognition.
const CATALOGUE: [&str; 14] = [
    "PageHeader",
    "KpiStrip",
    "SnapshotTable",
    "FilterBar",
    "ListPage",
    "AsyncDataSection",
    "SectionHeading",
    "PageStatePanel",
    "Card",
    "DataTable",
    "Stats",
    "Tabs",
    "Dialog",
    "Badge",
];

/// One catalogue pattern and the component whose root declares it.
struct Declaration {
    /// The catalogue name the root declares.
    pattern: &'static str,
    /// The `#[component]` function rendering it.
    component: &'static str,
    /// Repo-relative path, for failure messages.
    file: &'static str,
    /// The file's source.
    source: &'static str,
    /// Root elements the component can render: one per `view!` branch it
    /// returns (`PageHeader` has three navigation layouts, `PageStatePanel` an
    /// error and a status branch).
    roots: usize,
}

fn declarations() -> [Declaration; 14] {
    [
        Declaration {
            pattern: "PageHeader",
            component: "PageHeader",
            file: "src/patterns/page_header.rs",
            source: include_str!("page_header.rs"),
            roots: 3,
        },
        Declaration {
            pattern: "KpiStrip",
            component: "KpiStrip",
            file: "src/patterns/kpi_strip.rs",
            source: include_str!("kpi_strip.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "SnapshotTable",
            component: "SnapshotTablePage",
            file: "src/patterns/snapshot_table_page.rs",
            source: include_str!("snapshot_table_page.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "FilterBar",
            component: "FilterBar",
            file: "src/patterns/filter_bar.rs",
            source: include_str!("filter_bar.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "ListPage",
            component: "ListPage",
            file: "src/patterns/list_page.rs",
            source: include_str!("list_page.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "AsyncDataSection",
            component: "AsyncDataSection",
            file: "src/patterns/async_data_section.rs",
            source: include_str!("async_data_section.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "SectionHeading",
            component: "SectionHeading",
            file: "src/patterns/section_heading.rs",
            source: include_str!("section_heading.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "PageStatePanel",
            component: "PageStatePanel",
            file: "src/patterns/page_state_panel.rs",
            source: include_str!("page_state_panel.rs"),
            roots: 2,
        },
        Declaration {
            pattern: "Card",
            component: "Card",
            file: "src/components/card/component.rs",
            source: include_str!("../components/card/component.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "DataTable",
            component: "DataTable",
            file: "src/components/data_table/component.rs",
            source: include_str!("../components/data_table/component.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "Stats",
            component: "Stats",
            file: "src/components/stats/component.rs",
            source: include_str!("../components/stats/component.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "Tabs",
            component: "Tabs",
            file: "src/components/tab/component.rs",
            source: include_str!("../components/tab/component.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "Dialog",
            component: "Modal",
            file: "src/components/modal/component.rs",
            source: include_str!("../components/modal/component.rs"),
            roots: 1,
        },
        Declaration {
            pattern: "Badge",
            component: "Badge",
            file: "src/components/badge/component.rs",
            source: include_str!("../components/badge/component.rs"),
            roots: 1,
        },
    ]
}

/// The component function's body, from its `pub fn` line to the closing
/// brace in column 0, minus whole-line comments (which mention markup in
/// prose, e.g. the KpiStrip container's note about a bare `<div>`).
fn component_body(source: &str, component: &str) -> Option<String> {
    let by_paren = format!("pub fn {component}(");
    let by_generic = format!("pub fn {component}<");
    let mut lines = source.lines().skip_while(|line| {
        let line = line.trim_start();
        !(line.starts_with(&by_paren) || line.starts_with(&by_generic))
    });
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

/// The opening tag of every `view!` root in `body`: the first element after
/// each `view! {`, through the `>` that ends its attribute list.
fn view_roots(body: &str) -> Vec<&str> {
    const OPEN: &str = "view! {";
    let mut roots = Vec::new();
    let mut rest = body;
    while let Some(at) = rest.find(OPEN) {
        rest = &rest[at + OPEN.len()..];
        let element = rest.trim_start();
        if element.starts_with('<') {
            roots.push(opening_tag(element));
        }
    }
    roots
}

/// `tag` starts at `<`. Returns it through the `>` that closes the attribute
/// list, skipping string literals, anything nested in brackets, `=>` and `->`
/// — attribute values are Rust expressions (`move || match x { A => … }`).
fn opening_tag(tag: &str) -> &str {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    let mut previous = ' ';
    for (i, c) in tag.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else {
            match c {
                '"' => in_string = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                '>' if depth == 0 && previous != '=' && previous != '-' => return &tag[..=i],
                _ => {}
            }
        }
        previous = c;
    }
    tag
}

#[test]
fn pattern_roots_carry_data_ld_pattern() {
    let declarations = declarations();
    let covered: BTreeSet<&str> = declarations.iter().map(|d| d.pattern).collect();
    let catalogue: BTreeSet<&str> = CATALOGUE.into_iter().collect();
    assert_eq!(
        covered, catalogue,
        "every catalogue pattern needs exactly one declaring component"
    );

    for d in &declarations {
        let body = component_body(d.source, d.component)
            .unwrap_or_else(|| panic!("{}: no `pub fn {}`", d.file, d.component));
        let declared = format!("data-ld-pattern=\"{}\"", d.pattern);
        let roots = view_roots(&body);
        let declaring = roots.iter().filter(|tag| tag.contains(&declared)).count();
        assert_eq!(
            declaring,
            d.roots,
            "{}: `{}` must put {declared} on the root element of each of its {} rendered \
             branch(es); found it on {declaring} of its {} `view!` roots",
            d.file,
            d.component,
            d.roots,
            roots.len()
        );
        assert_eq!(
            body.matches("data-ld-pattern=").count(),
            d.roots,
            "{}: `{}` declares a pattern somewhere other than its root element(s)",
            d.file,
            d.component
        );
    }
}

/// The scanner has to be able to fail: a declaration on a child element is
/// not a root declaration.
#[test]
fn a_declaration_on_a_child_element_is_not_a_root_declaration() {
    let body = "view! {\n    <div class=\"x\">\n        <span data-ld-pattern=\"Badge\"></span>\n    </div>\n}";
    assert_eq!(view_roots(body), vec!["<div class=\"x\">"]);
}

/// A multi-line opening tag whose attribute values hold closures, `match`
/// arms and a string containing `>` is read to its real end.
#[test]
fn the_root_scanner_reads_a_multi_line_opening_tag() {
    let body = "view! {\n        <section\n            aria-busy=move || matches!(state, A | B).then_some(\"true\")\n            class=move || match size { Size::Sm => \"a > b\", _ => \"c\" }\n            data-ld-pattern=\"AsyncDataSection\"\n        >\n            <p>\"x\"</p>\n        </section>\n    }";
    let roots = view_roots(body);
    assert_eq!(roots.len(), 1, "{roots:?}");
    assert!(
        roots[0].ends_with("data-ld-pattern=\"AsyncDataSection\"\n        >"),
        "{:?}",
        roots[0]
    );
}

/// The body ends at the function's closing brace, and comment lines — where
/// prose names elements — are not markup.
#[test]
fn component_body_stops_at_the_function_end_and_drops_comment_lines() {
    let source = "/// view! { <b> }\npub fn Demo() -> impl IntoView {\n    // a <div> in prose\n    view! { <i class=\"a\"></i> }\n}\n\npub fn Other() {\n    view! { <u></u> }\n}\n";
    let body = component_body(source, "Demo").expect("Demo is defined");
    assert_eq!(view_roots(&body), vec!["<i class=\"a\">"]);
    assert!(component_body(source, "Missing").is_none());
}
