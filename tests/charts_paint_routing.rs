//! Every chart's colour must reach the DOM through `charts::paint` (ldui-1g5).
//!
//! SVG2 parses a presentation attribute with the property's value grammar in
//! isolation, so `var()` substitution — a mechanism defined for *declarations*
//! — is not specified to run inside `fill=` or `stroke=`. Chromium already
//! declines it for length presentation attributes and the SVG WG has an open
//! request (Nov 2025) to specify or forbid it. On the "forbid" outcome a
//! token-bearing value stops parsing and the shape falls back to the initial
//! `fill: black` (a solid black bar) or `stroke: none` (a line that simply
//! isn't there) — silently, with no console error.
//!
//! `charts::paint::{paint_attrs, stroke_attrs}` split a colour so a
//! `var()`-bearing value rides on `style` (specified to parse as a declaration
//! block) while a literal keeps the presentation attribute, leaving the DOM
//! byte-identical for literal-colour callers.
//!
//! A source scan is the right shape of test here for the same reason the
//! daisyUI-4 class guard is one: the property is "no `fill=`/`stroke=` in this
//! module can *ever* carry a token", which is a statement about every possible
//! caller's colour, not about one rendered tree. A per-chart render test would
//! only prove the colours that test happened to pass. This instead pins the
//! structural invariant — every colour-bearing attribute in `src/charts` is
//! either a quoted literal with no `var(` in it, or a binding produced by
//! `paint_attrs`/`stroke_attrs` — which no caller can defeat.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The colour-carrying SVG presentation attributes. `stroke-width` and friends
/// are hyphenated, so an exact `"<name>="` match never picks them up.
const COLOR_ATTRS: [&str; 2] = ["fill", "stroke"];

fn chart_source(name: &str) -> String {
    let p: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/charts")
        .join(format!("{name}.rs"));
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("reading {}: {e}", p.display()))
}

/// The part of a source file that actually renders — everything before the
/// `#[cfg(test)]` module. Scanning the test module too would both flag its
/// fixture strings and widen the accepted-binding set with names that never
/// reach a view.
fn render_code(src: &str) -> &str {
    match src.find("#[cfg(test)]") {
        Some(i) => &src[..i],
        None => src,
    }
}

fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with('*')
}

/// The identifiers bound by destructuring a `paint_attrs`/`stroke_attrs` call,
/// i.e. `a` and `b` in `let (a, b) = paint_attrs(…);`. These are the only
/// non-literal values a colour attribute may carry.
fn paint_bound_idents(code: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in code.lines().filter(|l| !is_comment(l)) {
        let t = line.trim_start();
        let routed = t.contains("paint_attrs(") || t.contains("stroke_attrs(");
        if !routed || !t.starts_with("let (") {
            continue;
        }
        let Some(close) = t.find(')') else { continue };
        for ident in t["let (".len()..close].split(',') {
            let ident = ident.trim();
            if !ident.is_empty() {
                out.insert(ident.to_string());
            }
        }
    }
    out
}

/// Every value assigned to `attr=` on `line`, as written in the source: a
/// quoted literal keeps its quotes, a binding comes back bare.
fn attr_values(line: &str, attr: &str) -> Vec<String> {
    let pat = format!("{attr}=");
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(pos) = line[cursor..].find(&pat) {
        let start = cursor + pos;
        let value_start = start + pat.len();
        cursor = value_start;
        // Reject `xfill=` / `my_stroke=`: the attribute name must start a token.
        let preceded_ok = start == 0
            || line[..start].ends_with(|c: char| c.is_whitespace())
            || line[..start].ends_with('<');
        if !preceded_ok {
            continue;
        }
        let rest = &line[value_start..];
        let value = match rest.strip_prefix('"') {
            Some(inner) => {
                let end = inner.find('"').unwrap_or(inner.len());
                format!("\"{}\"", &inner[..end])
            }
            None => rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '>' && *c != '/')
                .collect(),
        };
        out.push(value);
    }
    out
}

/// Assert every `fill=`/`stroke=` in one chart's rendering code is either a
/// `var()`-free literal or a `paint_attrs`/`stroke_attrs` binding.
fn assert_colors_are_routed(chart: &str) {
    let src = chart_source(chart);
    let code = render_code(&src);
    let routed = paint_bound_idents(code);

    let mut offenders = Vec::new();
    let mut checked = 0usize;
    for (i, line) in code.lines().enumerate() {
        if is_comment(line) {
            continue;
        }
        for attr in COLOR_ATTRS {
            for value in attr_values(line, attr) {
                checked += 1;
                if let Some(literal) = value.strip_prefix('"') {
                    let literal = literal.trim_end_matches('"');
                    if literal.contains("var(") {
                        offenders.push(format!(
                            "src/charts/{chart}.rs:{} — {attr}=\"{literal}\" puts a custom \
                             property in a presentation attribute",
                            i + 1
                        ));
                    }
                } else if !routed.contains(&value) {
                    offenders.push(format!(
                        "src/charts/{chart}.rs:{} — {attr}={value} is not a paint_attrs/\
                         stroke_attrs binding, so a caller passing var(--color-primary) would \
                         land it in the presentation attribute",
                        i + 1
                    ));
                }
            }
        }
    }

    assert!(
        checked > 0,
        "found no fill=/stroke= attributes in src/charts/{chart}.rs — the scanner is \
         not looking at what it thinks it is"
    );
    assert!(offenders.is_empty(), "{}", offenders.join("\n  "));
}

macro_rules! routing_test {
    ($name:ident, $chart:literal) => {
        #[test]
        fn $name() {
            assert_colors_are_routed($chart);
        }
    };
}

routing_test!(area_chart_colors_are_routed_through_paint, "area_chart");
routing_test!(bar_chart_colors_are_routed_through_paint, "bar_chart");
routing_test!(heatmap_colors_are_routed_through_paint, "heatmap");
routing_test!(line_chart_colors_are_routed_through_paint, "line_chart");
routing_test!(pie_chart_colors_are_routed_through_paint, "pie_chart");
routing_test!(sparkline_colors_are_routed_through_paint, "sparkline");
routing_test!(
    stacked_area_chart_colors_are_routed_through_paint,
    "stacked_area_chart"
);
routing_test!(
    stacked_bar_chart_colors_are_routed_through_paint,
    "stacked_bar_chart"
);

/// Every chart file must actually be listed above. A chart added later that
/// nobody remembers to register here would be silently unguarded, which is the
/// failure mode this whole file exists to prevent.
#[test]
fn every_chart_module_is_covered_by_a_routing_test() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/charts");
    let this_file = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/charts_paint_routing.rs"),
    )
    .expect("reading this test's own source");
    let mut missing = Vec::new();
    for entry in fs::read_dir(&dir).expect("reading src/charts").flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        // `mod.rs` renders nothing; `paint.rs` is the routing itself.
        if stem == "mod" || stem == "paint" {
            continue;
        }
        if !this_file.contains(&format!("\"{stem}\"")) {
            missing.push(stem.to_string());
        }
    }
    assert!(
        missing.is_empty(),
        "chart modules with no paint-routing test: {missing:?}"
    );
}

// --- negative controls: the scanner has to be able to fail ------------------

#[test]
fn the_scanner_reads_literal_and_bound_attribute_values() {
    let line = r#"<rect x=bx fill=c style=st stroke="white" stroke-width="1" />"#;
    assert_eq!(attr_values(line, "fill"), vec!["c".to_string()]);
    assert_eq!(attr_values(line, "stroke"), vec!["\"white\"".to_string()]);
}

#[test]
fn the_scanner_does_not_mistake_hyphenated_stroke_properties_for_a_colour() {
    let line = r#"<line stroke-width="1" stroke-opacity="0.3" stroke-linecap="round" />"#;
    assert!(attr_values(line, "stroke").is_empty(), "{line}");
}

#[test]
fn the_scanner_ignores_an_attribute_name_that_is_only_a_suffix() {
    // `opacity=fill_opacity_str` contains "fill" but is not a `fill=`.
    let line = r#"<polygon fill=area_fill opacity=fill_opacity_str />"#;
    assert_eq!(attr_values(line, "fill"), vec!["area_fill".to_string()]);
}

#[test]
fn the_scanner_collects_paint_and_stroke_bindings() {
    let code = "let (c, st) = paint_attrs(color.clone());\n\
                let (line_stroke, line_style) = stroke_attrs(color);\n";
    let idents = paint_bound_idents(code);
    for want in ["c", "st", "line_stroke", "line_style"] {
        assert!(idents.contains(want), "missing {want} in {idents:?}");
    }
}

#[test]
fn the_scanner_does_not_accept_an_unrouted_binding() {
    // The pre-ldui-1g5 shape: the raw prop straight into the attribute.
    let code = "view! { <polyline stroke=color /> }";
    let routed = paint_bound_idents(code);
    assert!(
        !routed.contains("color"),
        "an unrouted `color` must not count as paint-routed"
    );
    assert_eq!(attr_values(code, "stroke"), vec!["color".to_string()]);
}

#[test]
fn the_scanner_ignores_comments_explaining_the_hazard() {
    assert!(is_comment("    // a token must not ride on fill=\"...\""));
    assert!(is_comment("/// `stroke=` carries the same hazard"));
}

#[test]
fn render_code_stops_at_the_test_module() {
    let src = "fn render() {}\n#[cfg(test)]\nmod tests { fn x() { let a = 1; } }";
    assert_eq!(render_code(src), "fn render() {}\n");
}
