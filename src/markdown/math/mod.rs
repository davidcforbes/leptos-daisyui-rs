//! Math rendering for `<MarkdownView>` and `<MarkdownGraphicEditor>`
//! (ldui-r0co). Compiled only with the `math` cargo feature (ldui-dtm3):
//! math-core 0.9.0 needs Rust 1.96, and the consumer 4iiz-Office builds the
//! library on 1.95, so the default build has no math-core dependency and
//! `render_math` is a no-op there (the `$…$` source stays visible). Which
//! `$…$` pairs count as math at all is decided before this module runs, in
//! every build, by [`super::currency`].
//!
//! `editmark_core::render_html` emits math as:
//! - inline:  `<span class="math-inline">$…$</span>`
//! - display: `<div class="math-display">$$…$$</div>`
//!
//! Each formula is converted to native MathML Core by the Rust crate
//! [math-core](https://github.com/tmke8/math-core), compiled into this
//! library. There is no JavaScript, no CDN and no worker, so math renders
//! under a `script-src 'self'` CSP with nothing for the host to serve. The
//! browser lays the MathML out (Chrome/Edge 109+, Firefox, Safari).
//!
//! The pipeline, per formula:
//! 1. [`guard::check_formula`] bounds size and nesting. math-core parses
//!    recursively and a deep formula overflows wasm's stack, which traps
//!    and takes the whole app down, so nothing unbounded reaches it.
//! 2. math-core converts; its error message is kept.
//! 3. [`sanitize::parse_mathml`] reads the output into a tree and
//!    [`sanitize::sanitize_math_tree`] cuts it to an allow-list.
//! 4. The tree is built with `createElementNS`, styles through the CSSOM
//!    (`style.setProperty`, which a strict `style-src` allows), and wrapped
//!    as `<math><semantics><mrow>…</mrow><annotation
//!    encoding="application/x-tex">` with the source. No `aria-label`:
//!    screen readers (NVDA's MathCAT) read the MathML structure itself.
//!
//! The host element records the outcome: `data-math-state="rendered"` with
//! `data-math-stripped` (how much the allow-list removed), or
//! `data-math-state="error"` with `data-math-error` (`guard`, `latex` or
//! `output`) and the message in `title`. On an error the `$…$` source stays
//! visible, unchanged.
//!
//! math-core's browser fixes (`assets/math-core/mathmlfixes.css`, pinned by
//! [`MATHML_FIXES_SHA256`]) are compiled in and installed once as
//! `<style id="ldui-mathmlfixes">` when the first formula renders. The font
//! and dark-mode rules live in the markdown base stylesheet
//! (`theme.rs`). A page that still loads KaTeX is unaffected:
//! `window.katex` is no longer consulted.

mod arity;
pub mod guard;
pub mod sanitize;

use math_core::{LatexToMathML, MathCoreConfig, MathDisplay};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element, HtmlElement, Node};

use sanitize::MathNode;

/// The math-core version this module and its allow-list are written for.
pub const MATH_CORE_VERSION: &str = "0.9.0";

/// SHA-256 of the vendored `assets/math-core/mathmlfixes.css` (math-core tag
/// `v0.9.0`), checked by a unit test. See `assets/math-core/README.md`.
pub const MATHML_FIXES_SHA256: &str =
    "15128a36f105da7de7b382e5d7fd26dc1f9bc814a29d13f639d4681ff8eda679";

/// math-core's browser fixes for its MathML output.
pub const MATHML_FIXES_CSS: &str = include_str!("../../../assets/math-core/mathmlfixes.css");

const MATHML_NS: &str = "http://www.w3.org/1998/Math/MathML";
const HTML_NS: &str = "http://www.w3.org/1999/xhtml";
const FIXES_STYLE_ID: &str = "ldui-mathmlfixes";

/// Why a formula was left as source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MathError {
    /// Refused before conversion (too long, too deep, macro definition).
    Guard(String),
    /// math-core could not convert it (unknown command, bad syntax).
    Latex(String),
    /// math-core's output could not be read (never expected).
    Output(String),
}

impl MathError {
    /// The `data-math-error` value.
    pub fn kind(&self) -> &'static str {
        match self {
            MathError::Guard(_) => "guard",
            MathError::Latex(_) => "latex",
            MathError::Output(_) => "output",
        }
    }

    /// A human-readable message.
    pub fn message(&self) -> &str {
        match self {
            MathError::Guard(m) | MathError::Latex(m) | MathError::Output(m) => m,
        }
    }
}

/// A converted, sanitised formula.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderedMath {
    /// The `<math>` root after the allow-list.
    pub root: MathNode,
    /// Elements, attributes and style declarations the allow-list removed.
    pub stripped: usize,
}

thread_local! {
    static CONVERTER: Option<LatexToMathML> =
        LatexToMathML::new(MathCoreConfig::default(), Vec::new()).ok();
}

/// Convert one formula (delimiters already stripped) to a sanitised tree.
/// Pure: no DOM, so it is unit-tested natively.
pub fn convert_formula(latex: &str, display: bool) -> Result<RenderedMath, MathError> {
    guard::check_formula(latex).map_err(|e| MathError::Guard(e.message()))?;
    let mode = if display {
        MathDisplay::Block
    } else {
        MathDisplay::Inline
    };
    let mathml = CONVERTER.with(|c| match c {
        Some(c) => c
            .convert_with_local_state(latex, mode)
            .map(|r| r.mathml)
            .map_err(|e| MathError::Latex(e.error_message())),
        None => Err(MathError::Output("math converter unavailable".into())),
    })?;
    let tree = sanitize::parse_mathml(&mathml).map_err(MathError::Output)?;
    let (root, stripped) = sanitize::sanitize_math_tree(&tree);
    Ok(RenderedMath { root, stripped })
}

/// Render every `.math-inline` and `.math-display` inside `host`. Use this
/// in read-only render surfaces (MarkdownView).
pub fn render_math(host: &HtmlElement) {
    render_math_impl(host, true);
}

/// Render only `.math-display` (block `$$…$$`), leaving inline `$…$` as
/// source. For `MarkdownGraphicEditor`: display math is an atomic,
/// non-editable widget whose markdown comes from `data-em-src`, but inline
/// math sits in editable text, and replacing `$E=mc^2$` with MathML would
/// corrupt the DOM→markdown round-trip on the next edit of that paragraph.
pub fn render_math_display_only(host: &HtmlElement) {
    render_math_impl(host, false);
}

fn render_math_impl(host: &HtmlElement, include_inline: bool) {
    let Some(doc) = host.owner_document() else {
        return;
    };
    let selector = if include_inline {
        ".math-inline, .math-display"
    } else {
        ".math-display"
    };
    let Ok(nodes) = host.query_selector_all(selector) else {
        return;
    };
    let mut rendered_any = false;
    for i in 0..nodes.length() {
        let Some(el) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) else {
            continue;
        };
        let display = el.class_list().contains("math-display");
        rendered_any |= render_one(&doc, &el, display);
    }
    if rendered_any {
        install_fixes_once(&doc);
    }
}

/// Render `el` in place; true when MathML was inserted.
fn render_one(doc: &Document, el: &Element, display: bool) -> bool {
    let source = el.text_content().unwrap_or_default();
    let formula = strip_delimiters(&source, display);
    if formula.is_empty() {
        return false;
    }
    let outcome = convert_formula(&formula, display)
        .and_then(|math| build_math(doc, &math.root, &formula).map(|node| (node, math.stripped)));
    match outcome {
        Ok((node, stripped)) => {
            el.set_text_content(None);
            let _ = el.append_child(&node);
            let _ = el.set_attribute("data-math-state", "rendered");
            let _ = el.set_attribute("data-math-stripped", &stripped.to_string());
            let _ = el.remove_attribute("data-math-error");
            let _ = el.remove_attribute("title");
            true
        }
        Err(err) => {
            let _ = el.set_attribute("data-math-state", "error");
            let _ = el.set_attribute("data-math-error", err.kind());
            let _ = el.set_attribute("title", err.message());
            let _ = el.remove_attribute("data-math-stripped");
            false
        }
    }
}

/// `<math …><semantics><mrow>children</mrow><annotation
/// encoding="application/x-tex">source</annotation></semantics></math>`.
fn build_math(doc: &Document, root: &MathNode, source: &str) -> Result<Node, MathError> {
    let MathNode::Element {
        attrs, children, ..
    } = root
    else {
        return Err(MathError::Output("root is not an element".into()));
    };
    let dom = |e: JsValue| MathError::Output(format!("{e:?}"));
    let math = doc
        .create_element_ns(Some(MATHML_NS), "math")
        .map_err(dom)?;
    apply_attributes(&math, attrs);
    let semantics = doc
        .create_element_ns(Some(MATHML_NS), "semantics")
        .map_err(dom)?;
    let mrow = doc
        .create_element_ns(Some(MATHML_NS), "mrow")
        .map_err(dom)?;
    for child in children {
        mrow.append_child(&build_node(doc, child)?).map_err(dom)?;
    }
    let annotation = doc
        .create_element_ns(Some(MATHML_NS), "annotation")
        .map_err(dom)?;
    let _ = annotation.set_attribute("encoding", "application/x-tex");
    annotation.set_text_content(Some(source));
    semantics.append_child(&mrow).map_err(dom)?;
    semantics.append_child(&annotation).map_err(dom)?;
    math.append_child(&semantics).map_err(dom)?;
    Ok(math.into())
}

fn build_node(doc: &Document, node: &MathNode) -> Result<Node, MathError> {
    let dom = |e: JsValue| MathError::Output(format!("{e:?}"));
    match node {
        MathNode::Text(t) => Ok(doc.create_text_node(t).into()),
        MathNode::Element {
            name,
            html,
            attrs,
            children,
        } => {
            let ns = if *html { HTML_NS } else { MATHML_NS };
            let el = doc.create_element_ns(Some(ns), name).map_err(dom)?;
            apply_attributes(&el, attrs);
            for child in children {
                el.append_child(&build_node(doc, child)?).map_err(dom)?;
            }
            Ok(el.into())
        }
    }
}

/// Set sanitised attributes; `style` goes through the CSSOM so a strict
/// `style-src` cannot drop it.
fn apply_attributes(el: &Element, attrs: &[(String, String)]) {
    for (name, value) in attrs {
        if name == "style" {
            let Some(style) = js_sys::Reflect::get(el, &"style".into())
                .ok()
                .and_then(|s| s.dyn_into::<web_sys::CssStyleDeclaration>().ok())
            else {
                continue;
            };
            for (prop, val) in sanitize::style_declarations(value) {
                let _ = style.set_property(&prop, &val);
            }
        } else {
            let _ = el.set_attribute(name, value);
        }
    }
}

fn install_fixes_once(doc: &Document) {
    if doc.get_element_by_id(FIXES_STYLE_ID).is_some() {
        return;
    }
    let Ok(style) = doc.create_element("style") else {
        return;
    };
    style.set_id(FIXES_STYLE_ID);
    style.set_text_content(Some(MATHML_FIXES_CSS));
    if let Some(head) = doc.head() {
        let _ = head.append_child(&style);
    }
}

fn strip_delimiters(s: &str, display: bool) -> String {
    let trimmed = s.trim();
    if display {
        let inner = trimmed.strip_prefix("$$").unwrap_or(trimmed);
        let inner = inner.strip_suffix("$$").unwrap_or(inner);
        inner.trim().to_string()
    } else {
        let inner = trimmed.strip_prefix('$').unwrap_or(trimmed);
        let inner = inner.strip_suffix('$').unwrap_or(inner);
        inner.trim().to_string()
    }
}

#[cfg(test)]
mod tests;
