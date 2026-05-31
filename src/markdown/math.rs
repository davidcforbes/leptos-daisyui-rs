//! KaTeX auto-render hook for `<MarkdownView>`.
//!
//! `editmark_core::render_html` emits math as:
//! - inline:  `<span class="math-inline">$…$</span>`
//! - display: `<div class="math-display">$$…$$</div>`
//!
//! When the host page has KaTeX loaded (a `window.katex` global), we walk
//! those elements and call `katex.render(formula, element, { displayMode })`
//! to replace the `$…$` source with rendered MathML.  When KaTeX is absent,
//! we leave the source visible — no error, no fallback fetching.
//!
//! editmark-leptos does NOT ship KaTeX itself.  llm-wiki (or any consumer)
//! opts into math rendering by including the KaTeX library and stylesheet
//! in its page chrome:
//!
//! ```html
//! <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16/dist/katex.min.css">
//! <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16/dist/katex.min.js"></script>
//! ```
//!
//! This keeps the editmark-leptos bundle slim and lets the consumer choose
//! the KaTeX version / hosting strategy.

use js_sys::{Function, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement};

/// Render every `.math-inline` and `.math-display` inside `host` via KaTeX
/// if it's available.  No-op when `window.katex` is undefined.
pub fn render_math(host: &HtmlElement) {
    let Some(katex) = katex_global() else {
        return;
    };
    let Some(render_fn) = Reflect::get(&katex, &"render".into())
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
    else {
        return;
    };

    if let Ok(nodes) = host.query_selector_all(".math-inline") {
        for i in 0..nodes.length() {
            let Some(n) = nodes.item(i) else { continue };
            let Ok(el) = n.dyn_into::<Element>() else {
                continue;
            };
            render_one(&render_fn, &katex, &el, false);
        }
    }
    if let Ok(nodes) = host.query_selector_all(".math-display") {
        for i in 0..nodes.length() {
            let Some(n) = nodes.item(i) else { continue };
            let Ok(el) = n.dyn_into::<Element>() else {
                continue;
            };
            render_one(&render_fn, &katex, &el, true);
        }
    }
}

fn katex_global() -> Option<JsValue> {
    let win = web_sys::window()?;
    let v = Reflect::get(&win, &"katex".into()).ok()?;
    if v.is_undefined() || v.is_null() {
        None
    } else {
        Some(v)
    }
}

fn render_one(render_fn: &Function, katex: &JsValue, el: &Element, display_mode: bool) {
    let source = el.text_content().unwrap_or_default();
    let formula = strip_delimiters(&source, display_mode);
    if formula.is_empty() {
        return;
    }
    let options = Object::new();
    let _ = Reflect::set(
        &options,
        &"displayMode".into(),
        &JsValue::from_bool(display_mode),
    );
    // KaTeX defaults to throwing on error — for a robust renderer we'd
    // rather see the raw LaTeX than a stack trace inside the document.
    let _ = Reflect::set(&options, &"throwOnError".into(), &JsValue::from_bool(false));
    let _ = render_fn.call3(
        katex,
        &JsValue::from_str(&formula),
        el.as_ref(),
        options.as_ref(),
    );
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
mod tests {
    use super::strip_delimiters;

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
}
