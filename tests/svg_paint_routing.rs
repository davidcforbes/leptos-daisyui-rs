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
//! **A scanner's blind spots are defects of the same kind.** Review of the
//! widening found three shapes that shipped a paint defect past this file with
//! the gate green, and each is now a named negative control: a `#[cfg(test)]`
//! mention inside a comment silently truncating a whole file out of the scan;
//! `attr:`-prefixed attributes never being looked at (which is how a fourth
//! dead token survived in `dependency_link.rs`); and a `let` that merely
//! *mentions* a router laundering its name into the accepted set. Coverage is
//! now also compared per file against the unscanned source, so it cannot be
//! zeroed without saying so.
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
///
/// `stop-color`, `flood-color` and `lighting-color` have no callers today. They
/// are listed anyway because this array is a *registry*, and the file-level
/// registry (`every_chart_module_is_covered_by_a_routing_test`) that used to
/// force a periodic "is the coverage list complete?" review was deleted when
/// the directory walk replaced it. The walk replaced the file registry; nothing
/// replaced the attribute-name registry, so the names are enumerated up front
/// rather than the first time a gradient ships.
const COLOR_ATTRS: [&str; 5] = [
    "fill",
    "stroke",
    "stop-color",
    "flood-color",
    "lighting-color",
];

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
///
/// The anchor must be a **line that starts with** the attribute, not a byte
/// search over raw source. `src.find("#[cfg(test)]")` matches inside a `//!`,
/// `///` or `//` comment and inside string literals, and this repo writes that
/// token in prose constantly — including in this file's own module doc. One
/// such line near the top of a component silently deletes the rest of the file
/// from the scan, which would reproduce the exact ldui-xxc failure (a guard
/// reading green over a live defect) one abstraction up.
fn render_code(src: &str) -> &str {
    let mut offset = 0usize;
    for line in src.split_inclusive('\n') {
        if !is_comment(line) && line.trim_start().starts_with("#[cfg(test)]") {
            return &src[..offset];
        }
        offset += line.len();
    }
    src
}

fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with('*')
}

/// True when `expr`'s **head** — after an optional `move ||` — is a router
/// call, rather than a router merely appearing somewhere inside it.
///
/// "Mentions a router" is not good enough: `let stroke = if themed {
/// raw.get() } else { paint_attrs(c).0.unwrap_or_default() };` mentions one,
/// and would launder `stroke` into the accepted set while its themed branch
/// puts a raw caller colour straight into the attribute.
fn router_heads_expression(expr: &str) -> bool {
    let mut e = expr.trim_start();
    if let Some(r) = e.strip_prefix("move ") {
        e = r.trim_start();
    }
    if let Some(r) = e.strip_prefix("||") {
        e = r.trim_start();
    }
    let Some(open) = e.find('(') else {
        return false;
    };
    // Tolerate a path qualifier: `paint::stroke_attrs(…)`, `super::paint::…`.
    let head = e[..open].rsplit("::").next().unwrap_or("").trim();
    ROUTERS.iter().any(|r| r.trim_end_matches('(') == head)
}

/// Joins a `let` that rustfmt wrapped after the `=` with its continuation, so
/// the initialiser's head sits on the same logical line as the binding.
///
/// Without this, a long binding that formatting later wraps would be read as
/// having an empty initialiser, fail the router-head check, and poison a name
/// that is in fact correctly routed — turning a formatting change into a gate
/// failure. Every such binding in `src/` is one line today; this keeps that
/// from being load-bearing.
fn logical_lines(code: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut pending: Option<String> = None;
    for line in code.lines() {
        if is_comment(line) {
            continue;
        }
        let joined = match pending.take() {
            Some(prev) => format!("{prev} {}", line.trim_start()),
            None => line.to_string(),
        };
        if joined.trim_end().ends_with('=') {
            pending = Some(joined.trim_end().to_string());
            continue;
        }
        out.push(joined);
    }
    out.extend(pending);
    out
}

/// Splits a `let` line into `(bound identifiers, initialiser expression)`.
fn parse_let(line: &str) -> Option<(Vec<String>, &str)> {
    let t = line.trim_start();
    let rest = t.strip_prefix("let ")?;
    let (names, expr) = match rest.strip_prefix('(') {
        // `let (attr, style) = router(…)` destructures the pair.
        Some(tuple) => {
            let close = tuple.find(')')?;
            let after = tuple[close + 1..].trim_start();
            (&tuple[..close], after.strip_prefix('=').unwrap_or(after))
        }
        // `let name = …` binds one value (usually a closure).
        None => {
            let eq = rest.find('=')?;
            (&rest[..eq], &rest[eq + 1..])
        }
    };
    let idents = names
        .split(',')
        .filter_map(|ident| {
            // Tolerate `mut x` and `x: Ty`.
            let ident = ident.trim().trim_start_matches("mut ").trim();
            let ident = ident.split(':').next().unwrap_or(ident).trim();
            (!ident.is_empty()).then(|| ident.to_string())
        })
        .collect();
    Some((idents, expr))
}

/// The identifiers a colour attribute may carry: those bound by a `let` whose
/// initialiser is *headed* by a `charts::paint` router — `a`/`b` in
/// `let (a, b) = paint_attrs(…);`, and `f` in
/// `let f = move || stroke_attrs_with(BASE, c.get()).0;`.
///
/// An identifier bound anywhere in the file by a `let` that is **not**
/// router-headed is subtracted again. A line scanner cannot track scopes, so a
/// later `let stroke = raw.get();` shadowing an earlier routed `stroke` would
/// otherwise inherit its trust. Poisoning the name file-wide errs toward
/// reporting, which is the safe direction for a guard.
fn paint_bound_idents(code: &str) -> HashSet<String> {
    let mut routed: HashSet<String> = HashSet::new();
    let mut poisoned: HashSet<String> = HashSet::new();
    for line in logical_lines(code) {
        let Some((idents, expr)) = parse_let(&line) else {
            continue;
        };
        let target = if router_heads_expression(expr) {
            &mut routed
        } else {
            &mut poisoned
        };
        for ident in idents {
            target.insert(ident);
        }
    }
    routed.retain(|i| !poisoned.contains(i));
    routed
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
        let before = &line[..start];
        let preceded_ok = start == 0
            || before.ends_with(|c: char| c.is_whitespace())
            || before.ends_with('<')
            || before.ends_with(':');
        if !preceded_ok {
            continue;
        }
        // Leptos prefixes a colour attribute two ways, and only one is a
        // presentation attribute. `style:stroke=` writes a *declaration*, where
        // `var()` substitution is specified to work — that is the safe channel
        // and is deliberately allow-listed. Everything else, notably the
        // `attr:` spread that CLAUDE.md names as the extension path
        // (`<Sparkline attr:stroke="var(--color-primary)" />`), writes the
        // presentation attribute and carries the full hazard. Skipping every
        // prefixed form is how `dependency_link.rs:192` survived the first
        // sweep of this bead.
        if let Some(head) = before.strip_suffix(':') {
            let prefix = head
                .rsplit(|c: char| c.is_whitespace() || c == '<')
                .next()
                .unwrap_or("");
            if prefix == "style" {
                continue;
            }
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

/// How many colour attributes `code` contains, ignoring routing entirely. Used
/// to compare the scanned region against the whole file.
fn colour_attr_count(code: &str) -> usize {
    code.lines()
        .filter(|l| !is_comment(l))
        .map(|line| {
            COLOR_ATTRS
                .iter()
                .map(|attr| attr_values(line, attr).len())
                .sum::<usize>()
        })
        .sum()
}

/// Checks one file's rendering code, returning `(attributes checked, offenders)`.
fn audit(path: &str, src: &str) -> (usize, Vec<String>) {
    let code = render_code(src);
    let routed = paint_bound_idents(code);

    let mut offenders = Vec::new();
    let mut checked = 0usize;

    // Coverage cannot be silently zeroed. If the file draws colour but the
    // region actually scanned draws none, the `#[cfg(test)]` anchor truncated
    // everything real away — which is a scanner failure that must be loud,
    // because the global floor and the per-tree reach assertions would both
    // still hold from the other ~113 attributes elsewhere.
    if colour_attr_count(code) == 0 && colour_attr_count(src) > 0 {
        offenders.push(format!(
            "{path} — the file contains colour attributes but the scanned region contains \
             none: a `#[cfg(test)]` occurrence truncated the file away before the render \
             code was read"
        ));
    }

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

// --- the three bypasses found in review of ldui-xxc -------------------------

#[test]
fn a_cfg_test_mention_in_a_comment_does_not_zero_the_scan() {
    // Bypass (a): `src.find("#[cfg(test)]")` is a byte search over raw source,
    // so a doc comment naming the attribute — an idiom this very file uses —
    // deleted the rest of the file from the scan. The defect below must still
    // be seen.
    let src = "//! Covered by the crate's #[cfg(test)] paint-routing scan.\n\
               view! { <path stroke=\"var(--fallback-su,oklch(var(--su)/1))\" /> }\n";
    assert!(
        render_code(src).contains("<path"),
        "a comment must not truncate the file"
    );
    let (checked, offenders) = audit("fixture.rs", src);
    assert_eq!(checked, 1);
    assert_eq!(offenders.len(), 1, "{offenders:?}");
}

#[test]
fn a_real_cfg_test_module_still_truncates() {
    // …but the genuine anchor must keep working, or every test fixture in the
    // crate starts being reported.
    let src = "fn render() {}\n#[cfg(test)]\nmod tests { fn f() { let a = 1; } }";
    assert_eq!(render_code(src), "fn render() {}\n");
    // Indented (a nested module) counts too.
    let nested = "fn render() {}\n    #[cfg(test)]\n    mod tests {}";
    assert_eq!(render_code(nested), "fn render() {}\n");
}

#[test]
fn truncating_a_files_only_colour_away_is_itself_reported() {
    // The backstop behind the anchor fix: even if some future truncation bug
    // zeroed a file, the mismatch between file and scanned region is loud.
    let src = "const S: &str = \"#[cfg(test)]\";\nview! { <path fill=\"red\" /> }\n";
    let (_, offenders) = audit("fixture.rs", src);
    assert!(
        offenders.is_empty(),
        "a string literal must not truncate either: {offenders:?}"
    );
}

#[test]
fn an_attr_prefixed_colour_is_checked_and_a_style_prefixed_one_is_not() {
    // Bypass (b): `preceded_ok` accepted only whitespace/`<`/start, so every
    // prefixed form was invisible. `attr:` IS the presentation attribute and
    // is the documented spread-attribute extension path.
    let spread = r#"view! { <Sparkline attr:stroke="var(--color-primary)" /> }"#;
    let (checked, offenders) = audit("fixture.rs", spread);
    assert_eq!(checked, 1, "attr:stroke= must be scanned");
    assert_eq!(offenders.len(), 1, "{offenders:?}");

    // `style:` writes a declaration, where `var()` is specified to work.
    let decl = r#"style:stroke=move || if sel.get() { "var(--color-primary)" } else { "none" }"#;
    let (checked, offenders) = audit("fixture.rs", decl);
    assert_eq!(checked, 0, "style:stroke= is the safe channel");
    assert!(offenders.is_empty(), "{offenders:?}");
}

#[test]
fn a_router_buried_inside_an_expression_does_not_launder_the_binding() {
    // Bypass (c): the old check was "this line mentions a router", so the
    // themed branch could put a raw caller colour in the attribute.
    let code = "let stroke = if themed { raw.get() } else { paint_attrs(c).0.unwrap_or_default() };\n\
                view! { <path stroke=stroke /> }";
    assert!(
        !paint_bound_idents(code).contains("stroke"),
        "a router must head the bound expression, not merely appear in it"
    );
    let (_, offenders) = audit("fixture.rs", code);
    assert_eq!(offenders.len(), 1, "{offenders:?}");
}

#[test]
fn shadowing_a_routed_binding_poisons_it() {
    // The same laundering by a second `let`, which no line scanner can scope.
    let code = "let (stroke, st) = stroke_attrs(c);\n\
                let stroke = raw_colour.get();\n\
                view! { <path stroke=stroke /> }";
    assert!(
        !paint_bound_idents(code).contains("stroke"),
        "a name rebound from a non-router expression must lose its trust"
    );
    let (_, offenders) = audit("fixture.rs", code);
    assert_eq!(offenders.len(), 1, "{offenders:?}");
}

#[test]
fn a_binding_wrapped_after_the_equals_is_still_recognised() {
    let code = "let (today_line_stroke, today_line_style) =\n\
                    stroke_attrs_with(TODAY_LINE_STYLE, today_line_color());\n";
    let idents = paint_bound_idents(code);
    for want in ["today_line_stroke", "today_line_style"] {
        assert!(idents.contains(want), "missing {want} in {idents:?}");
    }
}

#[test]
fn the_gradient_colour_attributes_are_registered() {
    // No callers today; the registry is what keeps that from being permanent.
    for attr in ["stop-color", "flood-color", "lighting-color"] {
        assert!(
            COLOR_ATTRS.contains(&attr),
            "{attr} missing from COLOR_ATTRS"
        );
    }
    let grad = r#"view! { <stop offset="0" stop-color="var(--color-primary)" /> }"#;
    let (checked, offenders) = audit("fixture.rs", grad);
    assert_eq!(checked, 1);
    assert_eq!(offenders.len(), 1, "{offenders:?}");
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
