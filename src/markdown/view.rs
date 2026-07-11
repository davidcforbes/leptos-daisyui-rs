//! `<MarkdownView>` — view-only renderer.
//!
//! Pipeline:
//! 1. Parse + lay out the source via `editmark_core::build_layout`.
//! 2. Render to an HTML string via `editmark_core::render_html`.
//! 3. Hand the HTML to a `<div>` via `inner_html`.
//! 4. Post-process the live DOM: replace mermaid placeholders with SVG,
//!    syntax-highlight code blocks, sanitize anchor hrefs, attach the
//!    on_link_click intercept.
//!
//! The pipeline is reactive: any change to `source` re-runs from step 1.

use editmark_core::{FixedTextMeasure, build_layout, render_html};
use leptos::prelude::*;
use leptos::tachys::html::node_ref::NodeRefAttribute;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::HtmlElement;

use super::highlight::highlight_to_html;
use super::sanitize::is_safe_href;
use super::theme::{palette_style, use_theme};

/// Reactive markdown view.
///
/// `source` is read on every change.  `inline=true` collapses block-level
/// wrappers so the output flows inline within its surrounding text — useful
/// for hover-cards, single-line citation chips, table cells, etc.
/// `on_link_click`, when set, intercepts anchor clicks and routes the URL
/// through the callback instead of letting the browser navigate.
#[component]
pub fn MarkdownView(
    /// Reactive markdown source.
    #[prop(into)]
    source: Signal<String>,
    /// When true, render in inline flow (single-line, no block margins).
    #[prop(optional)]
    inline: bool,
    /// Optional link-click intercept — receives the href.
    #[prop(optional, into)]
    on_link_click: Option<Callback<String>>,
) -> impl IntoView {
    let theme = use_theme();
    let host: NodeRef<leptos::html::Div> = NodeRef::new();

    // Re-render whenever the source changes.  We attach to NodeRef rather
    // than using `inner_html=` on the view! macro because the post-processing
    // step (mermaid SVG injection, highlight, link wiring) needs the live
    // element handle.
    Effect::new(move |_| {
        let src = source.get();
        let Some(el) = host.get() else { return };
        let html_el: &HtmlElement = el.as_ref();
        let measure = FixedTextMeasure::default();
        let nodes = build_layout(&src, &measure, 900.0);
        let html = render_html(&nodes);
        html_el.set_inner_html(&html);
        post_process(html_el, on_link_click);
    });

    // Reactive palette → inline style on the root.
    let style = move || {
        let palette = theme.palette();
        palette_style(&palette)
    };
    let class = move || {
        if inline {
            "lds-root lds-inline"
        } else {
            "lds-root"
        }
    };

    view! {
        <div class=class style=style node_ref=host></div>
    }
}

/// Walk the rendered DOM and:
/// - Replace `<div class="mermaid">…</div>` placeholders with rendered SVG.
/// - Syntax-highlight `<code class="language-x">` blocks.
/// - Drop unsafe anchor hrefs (`javascript:`, `data:` for arbitrary MIME, …).
/// - Attach an on-click handler that fires `on_link_click` and prevents
///   default navigation.
fn post_process(host: &HtmlElement, on_link_click: Option<Callback<String>>) {
    process_mermaid(host);
    process_code_blocks(host);
    process_links(host, on_link_click);
    super::math::render_math(host);
}

pub(crate) fn process_mermaid(host: &HtmlElement) {
    let Ok(nodes) = host.query_selector_all(".mermaid") else {
        return;
    };
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else { continue };
        let Ok(el) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };
        let source = el.text_content().unwrap_or_default();
        if source.trim().is_empty() {
            continue;
        }
        let svg = match editmark_mermaid::render_text(&source) {
            Ok(svg) => svg,
            Err(_) => {
                // Render error → leave the source visible inside a <pre>
                // so the author can see what's wrong.
                el.set_inner_html(&format!("<pre>{}</pre>", html_escape(&source)));
                continue;
            }
        };
        el.set_inner_html(&svg);
    }
}

fn process_code_blocks(host: &HtmlElement) {
    let Ok(nodes) = host.query_selector_all("pre > code[class^=\"language-\"]") else {
        return;
    };
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else { continue };
        let Ok(code) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };
        // Skip mermaid (handled separately) — render_html doesn't emit
        // language-mermaid, but be defensive.
        let class = code.class_name();
        let lang = class
            .split_whitespace()
            .find_map(|c| c.strip_prefix("language-"))
            .unwrap_or("");
        if lang == "mermaid" {
            continue;
        }
        let source = code.text_content().unwrap_or_default();
        let highlighted = highlight_to_html(&source, lang);
        code.set_inner_html(&highlighted);
    }
}

fn process_links(host: &HtmlElement, on_link_click: Option<Callback<String>>) {
    let Ok(nodes) = host.query_selector_all("a[href]") else {
        return;
    };
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else { continue };
        let Ok(anchor) = node.dyn_into::<web_sys::HtmlAnchorElement>() else {
            continue;
        };
        let href = anchor.get_attribute("href").unwrap_or_default();
        if !is_safe_href(&href) {
            // Strip the href so the element is not clickable.
            let _ = anchor.remove_attribute("href");
            anchor.set_attribute("data-em-rejected-href", &href).ok();
            continue;
        }
        // If the host wants to intercept link clicks, attach a handler.
        if let Some(cb) = on_link_click {
            let cb_clone = cb;
            let href_clone = href.clone();
            let closure =
                Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
                    ev.prevent_default();
                    cb_clone.run(href_clone.clone());
                });
            let _ =
                anchor.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            // Closure ownership: anchors live with the rendered fragment;
            // when source changes, set_inner_html drops them.  Forgetting
            // matches that lifetime — the closure is GC'd with the element.
            closure.forget();
        }
    }
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}
