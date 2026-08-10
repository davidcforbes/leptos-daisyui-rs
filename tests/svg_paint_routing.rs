//! Every SVG colour in the library must reach the DOM through `charts::paint`
//! (ldui-1g5, widened in ldui-xxc).
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
//! byte-identical for literal-colour callers. `paint_attrs_with` /
//! `stroke_attrs_with` do the same for an element that already carries static
//! `style` declarations.
//!
//! **The scan covers all of `src/`, not just `src/charts/`.** The first version
//! of this guard scanned the chart module alone and reported 13/13 green while
//! `gantt/timeline/dependency_preview.rs` was shipping a dead daisyUI-4 token
//! in a `stroke=` attribute — an invisible preview line (ldui-xxc). The hazard
//! belongs to the *attribute*, so a guard scoped to one module is scoped to
//! less than the defect class it exists to catch. Widening it in place, rather
//! than adding a second guard beside it, keeps one statement of the invariant:
//! a new SVG-drawing component is covered the moment it is written, with
//! nothing to remember to register.
//!
//! A source scan is the right shape of test here for the same reason the
//! daisyUI-4 class guard is one: the property is "no `fill=`/`stroke=` in this
//! crate can *ever* carry a token", which is a statement about every possible
//! caller's colour, not about one rendered tree. A per-component render test
//! would only prove the colours that test happened to pass. This instead pins
//! the structural invariant — every colour-bearing attribute is either a quoted
//! literal with no `var(` in it, or a binding produced by a `charts::paint`
//! router — which no caller can defeat.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The colour-carrying SVG presentation attributes. `stroke-width` and friends
/// are hyphenated, so an exact `"<name>="` match never picks them up.
const COLOR_ATTRS: [&str; 2] = ["fill", "stroke"];

/// The `charts::paint` routers. `paint_attrs(` and `paint_attrs_with(` are both
/// listed because the trailing paren is what stops `stroke_attrs` matching
/// `stroke_attrs_with`'s prefix and vice versa.
const ROUTERS: [&str; 4] = [
    "paint_attrs(",
    "paint_attrs_with(",
    "stroke_attrs(",
    "stroke_attrs_with(",
];

fn src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every `.rs` file under `src/`, as `(path relative to the crate root, source)`.
fn library_sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    let mut stack = vec![src_dir()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let src = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
            out.push((rel, src));
        }
    }
    out.sort();
    out
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

fn calls_a_router(line: &str) -> bool {
    ROUTERS.iter().any(|r| line.contains(r))
}

/// The identifiers bound by a `let` that calls a `charts::paint` router —
/// `a`/`b` in `let (a, b) = paint_attrs(…);`, and `f` in
/// `let f = move || stroke_attrs_with(BASE, c.get()).0;`. These are the only
/// non-literal values a colour attribute may carry.
fn paint_bound_idents(code: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in code.lines().filter(|l| !is_comment(l)) {
        let t = line.trim_start();
        if !calls_a_router(t) {
            continue;
        }
        let Some(rest) = t.strip_prefix("let ") else {
            continue;
        };
        let names = match rest.strip_prefix('(') {
            // `let (attr, style) = router(…)` destructures the pair.
            Some(tuple) => &tuple[..tuple.find(')').unwrap_or(tuple.len())],
            // `let name = …router(…)` binds one value (usually a closure).
            None => &rest[..rest.find('=').unwrap_or(rest.len())],
        };
        for ident in names.split(',') {
            // Tolerate `mut x` and `x: Ty`.
            let ident = ident.trim().trim_start_matches("mut ").trim();
            let ident = ident.split(':').next().unwrap_or(ident).trim();
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

/// Checks one file's rendering code, returning `(attributes checked, offenders)`.
fn audit(path: &str, src: &str) -> (usize, Vec<String>) {
    let code = render_code(src);
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
                            "{path}:{} — {attr}=\"{literal}\" puts a custom property in a \
                             presentation attribute",
                            i + 1
                        ));
                    }
                } else if !routed.contains(&value) {
                    // A closure written inline (`stroke=move || …`) parses as
                    // the bare value `move`, which is never a routed binding,
                    // so it lands here too. That is deliberate: routing has to
                    // happen at a named binding for this scan to see it.
                    offenders.push(format!(
                        "{path}:{} — {attr}={value} is not a charts::paint binding, so a \
                         var(--color-…) value would land in the presentation attribute. Bind \
                         it first: `let f = move || stroke_attrs_with(BASE, c.get()).0;`",
                        i + 1
                    ));
                }
            }
        }
    }
    (checked, offenders)
}

/// The whole guard: no `fill=`/`stroke=` anywhere in the library may carry a
/// custom property directly, nor an unrouted binding that could.
#[test]
fn every_svg_paint_attribute_in_the_library_is_routed() {
    let mut offenders = Vec::new();
    let mut checked = 0usize;
    for (path, src) in library_sources() {
        let (n, mut found) = audit(&path, &src);
        checked += n;
        offenders.append(&mut found);
    }
    assert!(
        checked > 50,
        "only {checked} fill=/stroke= attributes found across src/ — the scanner is not \
         looking at what it thinks it is"
    );
    assert!(offenders.is_empty(), "\n  {}", offenders.join("\n  "));
}

/// The scan has to actually reach the SVG-heavy modules. `src/charts` is the
/// original scope; `src/components/gantt` is the one the narrower guard missed,
/// and a walk that silently stopped short of it would pass vacuously.
#[test]
fn the_scan_reaches_both_svg_heavy_module_trees() {
    let sources = library_sources();
    assert!(
        sources.len() > 100,
        "walked only {} files under src/",
        sources.len()
    );
    for tree in ["src/charts/", "src/components/gantt/"] {
        let checked: usize = sources
            .iter()
            .filter(|(p, _)| p.starts_with(tree))
            .map(|(p, s)| audit(p, s).0)
            .sum();
        assert!(
            checked > 0,
            "no fill=/stroke= attributes checked under {tree} — the walk does not reach it"
        );
    }
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
fn the_scanner_collects_tuple_and_closure_paint_bindings() {
    let code = "let (c, st) = paint_attrs(color.clone());\n\
                let (line_stroke, line_style) = stroke_attrs(color);\n\
                let preview = move || stroke_attrs_with(BASE, c.get()).0;\n\
                let mut label_fill = move || paint_attrs_with(BASE, c.get()).0;\n";
    let idents = paint_bound_idents(code);
    for want in [
        "c",
        "st",
        "line_stroke",
        "line_style",
        "preview",
        "label_fill",
    ] {
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
    let (checked, offenders) = audit("fixture.rs", code);
    assert_eq!(checked, 1);
    assert_eq!(offenders.len(), 1, "{offenders:?}");
}

#[test]
fn the_scanner_flags_the_ldui_xxc_shape() {
    // Both defects the widened scan exists to catch: a dead daisyUI-4 token
    // written straight into `stroke=`, and an inline closure that hides the
    // colour from any named-binding check.
    let dead_token = r#"view! { <path stroke="var(--fallback-su,oklch(var(--su)/1))" /> }"#;
    let (_, offenders) = audit("fixture.rs", dead_token);
    assert_eq!(offenders.len(), 1, "{offenders:?}");
    assert!(offenders[0].contains("custom property"), "{offenders:?}");

    let inline_closure = "view! { <path stroke=move || stroke_color.get() /> }";
    let (_, offenders) = audit("fixture.rs", inline_closure);
    assert_eq!(offenders.len(), 1, "{offenders:?}");
}

#[test]
fn the_scanner_accepts_a_routed_closure_binding() {
    let code = "let preview_stroke = move || stroke_attrs_with(BASE, c.get()).0;\n\
                view! { <path stroke=preview_stroke /> }";
    let (checked, offenders) = audit("fixture.rs", code);
    assert_eq!(checked, 1);
    assert!(offenders.is_empty(), "{offenders:?}");
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
