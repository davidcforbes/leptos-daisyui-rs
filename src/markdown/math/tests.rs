use super::sanitize::MathNode;
use super::*;
use sha2::{Digest, Sha256};

#[test]
fn strips_inline_dollars() {
    assert_eq!(strip_delimiters("$E=mc^2$", false), "E=mc^2");
}

#[test]
fn strips_display_dollars() {
    assert_eq!(
        strip_delimiters("$$\\int_0^1 x dx$$", true),
        "\\int_0^1 x dx"
    );
}

#[test]
fn tolerates_missing_delimiters() {
    assert_eq!(strip_delimiters("E=mc^2", false), "E=mc^2");
}

#[test]
fn vendored_mathml_fixes_match_the_pinned_sha256() {
    let hex: String = Sha256::digest(MATHML_FIXES_CSS.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        hex, MATHML_FIXES_SHA256,
        "assets/math-core/mathmlfixes.css is not math-core v{MATH_CORE_VERSION}'s copy; \
         re-vendor it from the matching tag (assets/math-core/README.md)"
    );
}

/// Everyday formulas a chat or note contains. Every one must convert AND
/// pass the allow-list with nothing removed: a strip here means math-core
/// emits something the allow-list does not know (vocabulary drift after an
/// upgrade), which would silently damage rendering.
const CORPUS: &[(&str, bool)] = &[
    ("E=mc^2", false),
    ("a^2 + b^2 = c^2", false),
    ("\\frac{a}{b} + \\dfrac{1}{2} - \\tfrac{3}{4}", false),
    ("\\sqrt{2} \\sqrt[3]{x+1}", false),
    ("\\int_0^1 x^2\\,dx = \\frac{1}{3}", true),
    ("\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}", true),
    (
        "\\prod_{k=1}^{\\infty} \\left(1 - \\frac{1}{p_k^s}\\right)^{-1}",
        true,
    ),
    ("\\lim_{x \\to 0} \\frac{\\sin x}{x} = 1", true),
    ("\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}", true),
    ("\\begin{bmatrix} 1 & 0 \\\\ 0 & 1 \\end{bmatrix}", true),
    ("\\begin{vmatrix} a & b \\\\ c & d \\end{vmatrix}", true),
    (
        "f(x) = \\begin{cases} 1 & x > 0 \\\\ 0 & \\text{otherwise} \\end{cases}",
        true,
    ),
    (
        "\\begin{aligned} a &= b + c \\\\ d &= e \\end{aligned}",
        true,
    ),
    ("\\begin{align} x &= 1 \\\\ y &= 2 \\end{align}", true),
    (
        "\\begin{array}{c|c} a & b \\\\ \\hline c & d \\end{array}",
        true,
    ),
    (
        "\\hat{x} \\bar{y} \\vec{v} \\tilde{n} \\dot{a} \\ddot{b}",
        false,
    ),
    ("\\overline{AB} \\underline{x}", false),
    (
        "\\mathbb{R} \\mathcal{L} \\mathfrak{g} \\mathscr{F} \\mathbf{v} \\mathrm{d}",
        false,
    ),
    ("\\operatorname{argmax}_x f(x)", false),
    ("\\text{if } x \\in A \\text{, then } x \\notin B", false),
    (
        "\\textbf{bold} \\textit{it} \\texttt{mono} \\textsf{sans}",
        false,
    ),
    ("\\alpha \\beta \\gamma \\Gamma \\pi \\Omega", false),
    ("x \\le y \\ge z \\ne w \\approx v \\equiv u", false),
    ("a < b > c \\& d", false),
    ("\\binom{n}{k} = \\frac{n!}{k!(n-k)!}", true),
    ("\\underbrace{a + b}_{\\text{sum}} \\overbrace{c}^{d}", true),
    ("\\boxed{x = 1}", false),
    ("\\cancel{x} \\bcancel{y} \\xcancel{z}", false),
    ("\\left\\{ x \\mid x > 0 \\right\\}", false),
    ("\\langle u, v \\rangle \\lVert x \\rVert", false),
    ("x' + f''(x) + y_1^2", false),
    (
        "\\nabla \\cdot \\mathbf{E} = \\frac{\\rho}{\\varepsilon_0}",
        true,
    ),
    ("\\quad \\qquad a \\, b \\; c \\! d", false),
    ("\\ldots \\cdots \\vdots \\ddots", false),
    ("{}_{n}C_{k}", false),
    ("\\xrightarrow{f} \\Rightarrow \\iff \\mapsto", false),
    // math-core accepts size commands in text mode only.
    ("\\text{\\Large big \\small small}", false),
    ("\\text{\\textsc{Small Caps}}", false),
];

#[test]
fn corpus_converts_with_nothing_stripped() {
    for (latex, display) in CORPUS {
        let math = convert_formula(latex, *display).unwrap_or_else(|e| panic!("{latex}: {e:?}"));
        assert_eq!(
            math.stripped, 0,
            "{latex}: the allow-list removed {} item(s) from math-core output",
            math.stripped
        );
        let MathNode::Element { name, attrs, .. } = &math.root else {
            panic!("{latex}: root is text");
        };
        assert_eq!(name, "math");
        let block = attrs.iter().any(|(a, v)| a == "display" && v == "block");
        assert_eq!(block, *display, "{latex}: display attribute");
    }
}

/// The corpus must reach the allow-list's riskier paths, or "nothing
/// stripped" proves little: styles, classes, `menclose`, HTML in `mtext`.
#[test]
fn corpus_exercises_styles_classes_and_text_html() {
    fn walk(n: &MathNode, names: &mut Vec<String>, attrs: &mut Vec<String>) {
        if let MathNode::Element {
            name,
            html,
            attrs: a,
            children,
        } = n
        {
            names.push(if *html {
                format!("html:{name}")
            } else {
                name.clone()
            });
            for (k, v) in a {
                attrs.push(format!("{k}={v}"));
            }
            children.iter().for_each(|c| walk(c, names, attrs));
        }
    }
    let (mut names, mut attrs) = (Vec::new(), Vec::new());
    for (latex, display) in CORPUS {
        walk(
            &convert_formula(latex, *display).unwrap().root,
            &mut names,
            &mut attrs,
        );
    }
    for want in ["menclose", "mtable", "html:b", "html:code", "html:span"] {
        assert!(
            names.iter().any(|n| n == want),
            "no <{want}> in corpus output"
        );
    }
    for want in [
        "style=text-align",
        "style=border",
        "style=font-size",
        "style=font-variant-caps",
        "class=menclose-",
        "notation=",
        "mathvariant=",
        "stretchy=",
    ] {
        let (k, v) = want.split_once('=').unwrap();
        assert!(
            attrs
                .iter()
                .any(|a| a.starts_with(&format!("{k}=")) && a.contains(v)),
            "no {want} in corpus output"
        );
    }
}

fn contains_attr(node: &MathNode, attr: &str) -> bool {
    match node {
        MathNode::Text(_) => false,
        MathNode::Element {
            attrs, children, ..
        } => {
            attrs.iter().any(|(a, _)| a == attr) || children.iter().any(|c| contains_attr(c, attr))
        }
    }
}

#[test]
fn links_and_raw_html_never_come_through() {
    for latex in [
        "\\href{javascript:alert(1)}{x}",
        "\\url{javascript:alert(1)}",
        "\\htmlId{x}{y}",
        "\\includegraphics{x.png}",
    ] {
        assert!(
            matches!(convert_formula(latex, false), Err(MathError::Latex(_))),
            "{latex}"
        );
    }
    let math = convert_formula("\\text{<script>alert(1)</script>}", false).unwrap();
    assert_eq!(math.root.element_names(), ["math", "mtext"]);
    assert_eq!(math.root.text(), "<script>alert(1)</script>");
}

#[test]
fn eqref_keeps_its_label_but_loses_the_link_and_ids() {
    let math = convert_formula(
        "\\begin{equation} x = 1 \\label{eq:a} \\end{equation} \\eqref{eq:a}",
        true,
    );
    // Either math-core refuses this combination or the link is unwrapped;
    // never an href or an id in the result.
    if let Ok(math) = math {
        assert!(!contains_attr(&math.root, "href"));
        assert!(!contains_attr(&math.root, "id"));
        assert!(!math.root.element_names().contains(&"a"));
    }
}

#[test]
fn colours_are_dropped_so_dark_mode_keeps_contrast() {
    let math = convert_formula("\\color{red}{x} + \\textcolor{blue}{y}", false).unwrap();
    assert!(math.stripped > 0);
    let mut styles = Vec::new();
    fn collect(n: &MathNode, out: &mut Vec<String>) {
        if let MathNode::Element {
            attrs, children, ..
        } = n
        {
            out.extend(
                attrs
                    .iter()
                    .filter(|(a, _)| a == "style")
                    .map(|(_, v)| v.clone()),
            );
            children.iter().for_each(|c| collect(c, out));
        }
    }
    collect(&math.root, &mut styles);
    assert!(styles.iter().all(|s| !s.contains("color")), "{styles:?}");
    assert_eq!(math.root.text(), "x+y");
}

#[test]
fn errors_say_which_stage_refused() {
    let deep = format!("{}x{}", "{".repeat(200), "}".repeat(200));
    let err = convert_formula(&deep, false).unwrap_err();
    assert_eq!(err.kind(), "guard");
    let err = convert_formula("\\notacommand", false).unwrap_err();
    assert_eq!(err.kind(), "latex");
    assert!(err.message().contains("notacommand"), "{}", err.message());
}

#[test]
fn macro_bombs_and_definitions_are_refused_before_conversion() {
    let err = convert_formula("\\def\\a{\\a\\a}\\a", false).unwrap_err();
    assert_eq!(err.kind(), "guard");
    let err = convert_formula("\\newcommand{\\R}{\\mathbb{R}} \\R", false).unwrap_err();
    assert_eq!(err.kind(), "guard");
}

#[test]
fn nesting_just_under_the_guard_converts() {
    // A braced \frac level is two guard entries (the command, its brace).
    let n = guard::MAX_MATH_DEPTH / 2;
    let latex = format!("{}x{}", "\\frac{1}{".repeat(n), "}".repeat(n));
    let math = convert_formula(&latex, true).unwrap();
    assert_eq!(math.stripped, 0);
}
