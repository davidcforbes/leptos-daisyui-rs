//! Parse math-core's MathML string into a tree and cut it down to an
//! allow-list before anything reaches the DOM (ldui-r0co).
//!
//! The tree is then built with `createElementNS` and the CSSOM
//! (`mod.rs`), never re-serialised into `innerHTML`, so there is no second
//! HTML parse for a namespace-confusion (mutation XSS) payload to exploit.
//! The parser only has to read math-core's own output: double-quoted
//! attributes, the five XML entities, `&ApplyFunction;` and
//! `&InvisibleTimes;`, numeric references, `<x/>` self-closing tags. Anything else is an error, and the formula falls back to its
//! source.
//!
//! The allow-list is written against math-core 0.9.0's measured output
//! vocabulary (`Cargo.toml` pins it exactly), intersected with MathML Core.
//! Dropped always: `href` and the HTML `<a>` it rides on (the element is
//! unwrapped, its text kept), `id`, `mathcolor`/`mathbackground`, `color` in
//! styles, event handlers, and any element not listed.

/// Elements in the MathML namespace that survive.
pub const MATH_ELEMENTS: &[&str] = &[
    "math",
    "mrow",
    "mi",
    "mn",
    "mo",
    "mtext",
    "ms",
    "mspace",
    "mfrac",
    "msqrt",
    "mroot",
    "msub",
    "msup",
    "msubsup",
    "munder",
    "mover",
    "munderover",
    "mmultiscripts",
    "mprescripts",
    "mtable",
    "mtr",
    "mtd",
    "mpadded",
    "mphantom",
    "mstyle",
    "merror",
    "menclose",
];

/// HTML elements math-core puts inside `<mtext>` for `\textbf`, `\texttt`
/// and friends. Allowed only below an `mtext`.
pub const TEXT_HTML_ELEMENTS: &[&str] = &["b", "i", "em", "code", "span", "s", "u"];

/// Classes math-core emits that `mathmlfixes.css` styles.
const ALLOWED_CLASSES: &[&str] = &[
    "menclose-phasorangle",
    "menclose-updiagonalstrike",
    "menclose-downdiagonalstrike",
    "math-core-sans-serif-font",
    "math-core-serif-font",
];

/// `menclose` notations (MathML 3 vocabulary; Chromium is polyfilled by
/// `mathmlfixes.css`).
const NOTATIONS: &[&str] = &[
    "box",
    "roundedbox",
    "circle",
    "left",
    "right",
    "top",
    "bottom",
    "actuarial",
    "phasorangle",
    "updiagonalstrike",
    "downdiagonalstrike",
    "verticalstrike",
    "horizontalstrike",
    "madruwb",
    "longdiv",
    "radical",
    "updiagonalarrow",
];

const MATHVARIANTS: &[&str] = &[
    "normal",
    "bold",
    "italic",
    "bold-italic",
    "double-struck",
    "bold-fraktur",
    "script",
    "bold-script",
    "fraktur",
    "sans-serif",
    "bold-sans-serif",
    "sans-serif-italic",
    "sans-serif-bold-italic",
    "monospace",
    "initial",
    "tailed",
    "looped",
    "stretched",
];

/// One node of a parsed formula.
#[derive(Clone, Debug, PartialEq)]
pub enum MathNode {
    /// An element; `html` is true for the HTML-namespace text styling
    /// elements inside `mtext`.
    Element {
        /// Lower-case tag name.
        name: String,
        /// True for an HTML-namespace element (inside `mtext`).
        html: bool,
        /// Attributes in source order.
        attrs: Vec<(String, String)>,
        /// Child nodes.
        children: Vec<MathNode>,
    },
    /// Text, already entity-decoded.
    Text(String),
}

impl MathNode {
    /// Every element name in the tree, depth first (tests, diagnostics).
    pub fn element_names(&self) -> Vec<&str> {
        let mut out = Vec::new();
        fn walk<'a>(n: &'a MathNode, out: &mut Vec<&'a str>) {
            if let MathNode::Element { name, children, .. } = n {
                out.push(name);
                for c in children {
                    walk(c, out);
                }
            }
        }
        walk(self, &mut out);
        out
    }

    /// Concatenated text content.
    pub fn text(&self) -> String {
        match self {
            MathNode::Text(t) => t.clone(),
            MathNode::Element { children, .. } => children.iter().map(MathNode::text).collect(),
        }
    }
}

/// Deepest element nesting the parser accepts. The guard caps the source
/// at 64 levels; math-core wraps each level in at most a few elements.
const MAX_TREE_DEPTH: usize = 512;

/// Parse math-core output (one `<math>` element) into a tree.
pub fn parse_mathml(s: &str) -> Result<MathNode, String> {
    let mut p = Parser { s, i: 0 };
    p.skip_ws();
    let root = p.element(0, false)?;
    p.skip_ws();
    if p.i != s.len() {
        return Err(format!("trailing content at byte {}", p.i));
    }
    match &root {
        MathNode::Element { name, .. } if name == "math" => Ok(root),
        _ => Err("root element is not <math>".into()),
    }
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &'a str {
        &self.s[self.i..]
    }

    fn skip_ws(&mut self) {
        let trimmed = self.rest().trim_start_matches([' ', '\n', '\t', '\r']);
        self.i = self.s.len() - trimmed.len();
    }

    fn name(&mut self) -> Result<&'a str, String> {
        let rest = self.rest();
        let len = rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'-' || *b == b':')
            .count();
        if len == 0 {
            return Err(format!("expected a name at byte {}", self.i));
        }
        self.i += len;
        Ok(&rest[..len])
    }

    fn expect(&mut self, lit: &str) -> Result<(), String> {
        if self.rest().starts_with(lit) {
            self.i += lit.len();
            Ok(())
        } else {
            Err(format!("expected {lit:?} at byte {}", self.i))
        }
    }

    fn element(&mut self, depth: usize, in_mtext: bool) -> Result<MathNode, String> {
        if depth > MAX_TREE_DEPTH {
            return Err("output nested too deeply".into());
        }
        self.expect("<")?;
        let name = self.name()?.to_ascii_lowercase();
        let html = in_mtext && name != "mtext";
        let mut attrs = Vec::new();
        loop {
            self.skip_ws();
            if self.rest().starts_with("/>") {
                self.i += 2;
                return Ok(MathNode::Element {
                    name,
                    html,
                    attrs,
                    children: Vec::new(),
                });
            }
            if self.rest().starts_with('>') {
                self.i += 1;
                break;
            }
            let attr = self.name()?.to_ascii_lowercase();
            self.expect("=\"")?;
            let end = self
                .rest()
                .find('"')
                .ok_or_else(|| format!("unterminated attribute at byte {}", self.i))?;
            let raw = &self.rest()[..end];
            self.i += end + 1;
            attrs.push((attr, decode_entities(raw)?));
        }
        let child_in_mtext = in_mtext || name == "mtext";
        let mut children = Vec::new();
        loop {
            if self.rest().starts_with("</") {
                self.i += 2;
                let close = self.name()?.to_ascii_lowercase();
                if close != name {
                    return Err(format!("</{close}> closes <{name}>"));
                }
                self.skip_ws();
                self.expect(">")?;
                return Ok(MathNode::Element {
                    name,
                    html,
                    attrs,
                    children,
                });
            }
            if self.rest().starts_with('<') {
                children.push(self.element(depth + 1, child_in_mtext)?);
                continue;
            }
            if self.rest().is_empty() {
                return Err(format!("<{name}> is never closed"));
            }
            let end = self.rest().find('<').unwrap_or(self.rest().len());
            let raw = &self.rest()[..end];
            self.i += end;
            // Converted with `PrettyPrint::Never`, so every text node is
            // content (`\text{ }` is a lone space) and is kept as is.
            children.push(MathNode::Text(decode_entities(raw)?));
        }
    }
}

fn decode_entities(raw: &str) -> Result<String, String> {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        let semi = after
            .find(';')
            .ok_or_else(|| "unterminated character reference".to_string())?;
        let entity = &after[..semi];
        let ch = match entity {
            "lt" => '<',
            "gt" => '>',
            "amp" => '&',
            "quot" => '"',
            "apos" => '\'',
            // The two named MathML entities math-core 0.9.0 writes (`<mo>`
            // after a function name and between implicit products).
            "ApplyFunction" => '\u{2061}',
            "InvisibleTimes" => '\u{2062}',
            _ => {
                let code =
                    if let Some(hex) = entity.strip_prefix("#x").or(entity.strip_prefix("#X")) {
                        u32::from_str_radix(hex, 16).ok()
                    } else if let Some(dec) = entity.strip_prefix('#') {
                        dec.parse::<u32>().ok()
                    } else {
                        None
                    };
                code.and_then(char::from_u32)
                    .ok_or_else(|| format!("unknown character reference &{entity};"))?
            }
        };
        out.push(ch);
        rest = &after[semi + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Cut `root` down to the allow-list. Returns the cleaned tree and how
/// many elements, attributes and style declarations were removed.
pub fn sanitize_math_tree(root: &MathNode) -> (MathNode, usize) {
    let mut stripped = 0;
    let mut out = clean(root, false, &mut stripped);
    let root = match out.pop() {
        Some(n @ MathNode::Element { .. }) if out.is_empty() => n,
        _ => {
            stripped += 1;
            MathNode::Element {
                name: "math".into(),
                html: false,
                attrs: Vec::new(),
                children: Vec::new(),
            }
        }
    };
    (root, stripped)
}

/// Clean one node; returns the nodes that replace it (none when dropped,
/// the children when an `<a>` is unwrapped).
fn clean(node: &MathNode, in_mtext: bool, stripped: &mut usize) -> Vec<MathNode> {
    let MathNode::Element {
        name,
        html,
        attrs,
        children,
    } = node
    else {
        return vec![node.clone()];
    };
    let child_in_mtext = in_mtext || (!*html && name == "mtext");
    let clean_children = |stripped: &mut usize| -> Vec<MathNode> {
        children
            .iter()
            .flat_map(|c| clean(c, child_in_mtext, stripped))
            .collect()
    };
    if *html {
        if !in_mtext {
            *stripped += 1;
            return Vec::new();
        }
        if name == "a" {
            // `\eqref` links: keep the label text, drop the link.
            *stripped += 1;
            return clean_children(stripped);
        }
        if !TEXT_HTML_ELEMENTS.contains(&name.as_str()) {
            *stripped += 1;
            return Vec::new();
        }
    } else if !MATH_ELEMENTS.contains(&name.as_str()) {
        *stripped += 1;
        return Vec::new();
    }
    let mut kept = Vec::new();
    for (attr, value) in attrs {
        match clean_attribute(name, *html, attr, value) {
            Some(v) => kept.push((attr.clone(), v)),
            None => *stripped += 1,
        }
    }
    vec![MathNode::Element {
        name: name.clone(),
        html: *html,
        attrs: kept,
        children: clean_children(stripped),
    }]
}

/// The value to keep for `attr` on `element`, or `None` to drop it.
pub fn clean_attribute(element: &str, html: bool, attr: &str, value: &str) -> Option<String> {
    let keep = |ok: bool| ok.then(|| value.to_string());
    match attr {
        "style" => clean_style(value),
        "class" => {
            let classes: Vec<&str> = value
                .split_ascii_whitespace()
                .filter(|c| ALLOWED_CLASSES.contains(c))
                .collect();
            (!classes.is_empty()).then(|| classes.join(" "))
        }
        _ if html => None,
        "display" => keep(element == "math" && matches!(value, "block" | "inline")),
        "displaystyle" | "stretchy" | "symmetric" | "largeop" | "movablelimits" | "accent"
        | "accentunder" | "fence" | "separator" => keep(matches!(value, "true" | "false")),
        "scriptlevel" => keep(is_integer(value)),
        "mathvariant" => keep(MATHVARIANTS.contains(&value)),
        "dir" => keep(matches!(value, "ltr" | "rtl")),
        "form" => keep(matches!(value, "prefix" | "infix" | "postfix")),
        "lspace" | "rspace" | "minsize" | "maxsize" | "linethickness" | "width" | "height"
        | "depth" | "voffset" => keep(is_length(value)),
        "columnspan" | "rowspan" => {
            keep(element == "mtd" && value.parse::<u16>().is_ok_and(|n| n > 0))
        }
        "notation" => keep(
            element == "menclose"
                && !value.trim().is_empty()
                && value
                    .split_ascii_whitespace()
                    .all(|t| NOTATIONS.contains(&t)),
        ),
        _ => None,
    }
}

fn is_integer(v: &str) -> bool {
    let digits = v.strip_prefix(['+', '-']).unwrap_or(v);
    !digits.is_empty() && digits.len() <= 3 && digits.bytes().all(|b| b.is_ascii_digit())
}

/// A CSS length (`0`, `1.5em`, `-0.2ex`, `50%`, `3px`), as math-core and
/// MathML Core write them. No functions, no keywords beyond `0`.
pub fn is_length(v: &str) -> bool {
    let v = v.trim();
    if v == "0" {
        return true;
    }
    let num_end = v
        .bytes()
        .position(|b| !(b.is_ascii_digit() || b == b'.' || b == b'-' || b == b'+'))
        .unwrap_or(v.len());
    let (num, unit) = v.split_at(num_end);
    let num = num.strip_prefix(['+', '-']).unwrap_or(num);
    let is_number = !num.is_empty()
        && num.len() <= 12
        && num.bytes().filter(|b| *b == b'.').count() <= 1
        && num.bytes().any(|b| b.is_ascii_digit())
        && num.bytes().all(|b| b.is_ascii_digit() || b == b'.');
    is_number
        && matches!(unit, "em" | "ex" | "px" | "pt" | "%" | "rem" | "mu" | "")
        && !(unit.is_empty() && num != "0")
}

/// Keep only the declarations math-core uses for layout, each value from
/// a closed grammar. Returns the re-joined declarations, or `None` when
/// nothing survives.
pub fn clean_style(style: &str) -> Option<String> {
    let kept: Vec<String> = style_declarations(style)
        .into_iter()
        .filter(|(prop, value)| is_allowed_declaration(prop, value))
        .map(|(prop, value)| format!("{prop}: {value}"))
        .collect();
    (!kept.is_empty()).then(|| kept.join("; "))
}

/// Split a `style` attribute into lower-cased `(property, value)` pairs.
pub fn style_declarations(style: &str) -> Vec<(String, String)> {
    style
        .split(';')
        .filter_map(|decl| {
            let (prop, value) = decl.split_once(':')?;
            let prop = prop.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            (!prop.is_empty() && !value.is_empty()).then_some((prop, value))
        })
        .collect()
}

fn is_allowed_declaration(prop: &str, value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    // Closed grammars below never need these; refuse them up front.
    if lower.contains(['(', '\\', '/', '"', '\'', '<', '>', '@', '!']) {
        return false;
    }
    let tokens: Vec<&str> = lower.split_ascii_whitespace().collect();
    match prop {
        "text-align" => matches!(lower.as_str(), "left" | "right" | "center"),
        "justify-items" => matches!(lower.as_str(), "start" | "end" | "center"),
        "font-variant-caps" => lower == "small-caps",
        "width" | "font-size" | "margin-left" | "margin-right" | "padding-top"
        | "padding-bottom" | "padding-left" | "padding-right" => {
            tokens.len() == 1 && is_length(tokens[0])
        }
        "padding" => (1..=4).contains(&tokens.len()) && tokens.iter().all(|t| is_length(t)),
        "border-top" | "border-bottom" | "border-left" | "border-right" => {
            tokens.len() == 3
                && is_length(tokens[0])
                && matches!(tokens[1], "solid" | "dashed")
                && tokens[2] == "currentcolor"
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el(name: &str, attrs: &[(&str, &str)], children: Vec<MathNode>) -> MathNode {
        MathNode::Element {
            name: name.into(),
            html: false,
            attrs: attrs
                .iter()
                .map(|(a, v)| (a.to_string(), v.to_string()))
                .collect(),
            children,
        }
    }

    #[test]
    fn parses_math_core_shapes() {
        let tree = parse_mathml(
            "<math display=\"block\"><mrow><mi>x</mi><mo>&lt;</mo><mspace width=\"0.1667em\"/><mtext>a &amp; b&#39;</mtext></mrow></math>",
        )
        .unwrap();
        assert_eq!(
            tree.element_names(),
            ["math", "mrow", "mi", "mo", "mspace", "mtext"]
        );
        assert_eq!(tree.text(), "x<a & b'");
    }

    #[test]
    fn rejects_malformed_output() {
        for bad in [
            "<math><mi>x</mo></math>",
            "<math><mi>x</mi>",
            "<math><mi a=x>y</mi></math>",
            "<mrow></mrow>",
            "<math></math><math></math>",
            "<math>&bogus;</math>",
        ] {
            assert!(parse_mathml(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn html_inside_mtext_is_marked_and_kept() {
        let tree = parse_mathml(
            "<math><mtext><b>bold</b><span class=\"math-core-serif-font\">s</span></mtext></math>",
        )
        .unwrap();
        let (clean, stripped) = sanitize_math_tree(&tree);
        assert_eq!(stripped, 0);
        assert_eq!(clean, tree);
        let MathNode::Element { children, .. } = &clean else {
            unreachable!()
        };
        let MathNode::Element { children, .. } = &children[0] else {
            unreachable!()
        };
        assert!(matches!(&children[0], MathNode::Element { html: true, name, .. } if name == "b"));
    }

    #[test]
    fn eqref_links_are_unwrapped_to_their_text() {
        let tree =
            parse_mathml("<math><mtext><a href=\"javascript:alert(1)\">(1)</a></mtext></math>")
                .unwrap();
        let (clean, stripped) = sanitize_math_tree(&tree);
        assert_eq!(stripped, 1);
        assert_eq!(clean.element_names(), ["math", "mtext"]);
        assert_eq!(clean.text(), "(1)");
    }

    #[test]
    fn unknown_and_dangerous_elements_are_dropped_with_their_subtree() {
        let tree = el(
            "math",
            &[],
            vec![
                el(
                    "maction",
                    &[("actiontype", "toggle")],
                    vec![MathNode::Text("x".into())],
                ),
                el("annotation-xml", &[("encoding", "text/html")], vec![]),
                el("semantics", &[], vec![]),
                el("mglyph", &[], vec![]),
                el("mi", &[], vec![MathNode::Text("y".into())]),
            ],
        );
        let (clean, stripped) = sanitize_math_tree(&tree);
        assert_eq!(stripped, 4);
        assert_eq!(clean.element_names(), ["math", "mi"]);
        assert_eq!(clean.text(), "y");
    }

    #[test]
    fn html_outside_mtext_is_dropped() {
        let tree = MathNode::Element {
            name: "math".into(),
            html: false,
            attrs: vec![],
            children: vec![MathNode::Element {
                name: "b".into(),
                html: true,
                attrs: vec![],
                children: vec![MathNode::Text("x".into())],
            }],
        };
        let (clean, stripped) = sanitize_math_tree(&tree);
        assert_eq!(stripped, 1);
        assert_eq!(clean.element_names(), ["math"]);
    }

    #[test]
    fn dangerous_attributes_are_dropped() {
        for (element, attr, value) in [
            ("mi", "href", "javascript:alert(1)"),
            ("mi", "xlink:href", "javascript:alert(1)"),
            ("mi", "onclick", "alert(1)"),
            ("mi", "id", "x"),
            ("mi", "mathcolor", "red"),
            ("mi", "mathbackground", "red"),
            ("mi", "data-x", "1"),
            ("mi", "mathvariant", "javascript"),
            ("mi", "display", "block"),
            ("math", "display", "contents"),
            ("mspace", "width", "calc(100vw)"),
            ("mspace", "width", "1e9em"),
            ("menclose", "notation", "box url"),
            ("mtd", "columnspan", "0"),
            ("mi", "class", "btn btn-primary"),
        ] {
            assert_eq!(
                clean_attribute(element, false, attr, value),
                None,
                "{element} {attr}={value}"
            );
        }
        assert_eq!(clean_attribute("b", true, "mathvariant", "bold"), None);
    }

    #[test]
    fn math_core_attributes_survive() {
        for (element, attr, value) in [
            ("math", "display", "block"),
            ("mo", "stretchy", "false"),
            ("mo", "lspace", "0.2778em"),
            ("mo", "minsize", "1.2em"),
            ("mfrac", "linethickness", "0"),
            ("mtable", "scriptlevel", "0"),
            ("mtable", "displaystyle", "true"),
            ("mi", "mathvariant", "normal"),
            ("mspace", "width", "-0.1667em"),
            ("mpadded", "voffset", "0.5ex"),
            (
                "menclose",
                "notation",
                "updiagonalstrike downdiagonalstrike",
            ),
            ("mrow", "class", "menclose-phasorangle"),
        ] {
            assert_eq!(
                clean_attribute(element, false, attr, value).as_deref(),
                Some(value),
                "{element} {attr}={value}"
            );
        }
    }

    #[test]
    fn styles_keep_only_layout_declarations() {
        assert_eq!(
            clean_style("width: 50%;text-align: right;justify-items: end;").as_deref(),
            Some("width: 50%; text-align: right; justify-items: end")
        );
        assert_eq!(
            clean_style("border-top: 0.05em dashed currentcolor;").as_deref(),
            Some("border-top: 0.05em dashed currentcolor")
        );
        assert_eq!(
            clean_style("font-variant-caps: small-caps").as_deref(),
            Some("font-variant-caps: small-caps")
        );
        assert_eq!(
            clean_style("padding-left: 0.1em;padding-right: 0.1em;").as_deref(),
            Some("padding-left: 0.1em; padding-right: 0.1em")
        );
        for hostile in [
            "color:#000000;",
            "background: url(javascript:alert(1))",
            "width: expression(alert(1))",
            "width: var(--x)",
            "position: fixed; inset: 0",
            "border-top: 1px solid red",
            "width: 1\\30 px",
            "width: 1px /* x */",
            "behavior: url(x.htc)",
        ] {
            assert_eq!(clean_style(hostile), None, "{hostile}");
        }
    }
}
