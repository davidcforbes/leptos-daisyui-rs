//! `<MarkdownView>` — view-only renderer.
//!
//! Pipeline:
//! 1. Parse + lay out the source via `editmark_core::build_layout`.
//! 2. Render to an HTML string via `editmark_core::render_html`.
//! 3. Hand the HTML to a `<div>` via `inner_html`.
//! 4. Post-process the live DOM: replace mermaid placeholders with SVG
//!    (via pele, lazy-loaded, so diagrams may land a moment later),
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
/// - Replace `<div class="mermaid">…</div>` placeholders with pele's SVG.
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

/// Render every `.mermaid` placeholder under `host` with pele.
///
/// Synchronous when pele is already loaded. Otherwise each placeholder shows
/// its source in a `pre` (`data-pele-state="pending"` on that `pre`) while
/// the bundle loads once per page ([`crate::pele::load_pele`]), and then
/// draws. A placeholder that was detached in the meantime (the source
/// changed and the host was re-rendered) is skipped, so the last render
/// wins. On a pele error the source stays visible in a `pre` and pele's
/// message, escaped, follows it in the `mermaid-error` box; a failed load
/// is reported the same way and the next render retries it.
///
/// Also called by `MarkdownGraphicEditor`: there the placeholder is an
/// atomic `contenteditable=false` widget whose markdown comes from
/// `data-em-src`, so replacing its children late is display-only.
pub(crate) fn process_mermaid(host: &HtmlElement) {
    let Ok(nodes) = host.query_selector_all(".mermaid") else {
        return;
    };
    // Read every source before touching any placeholder's children.
    let mut diagrams: Vec<(web_sys::Element, String)> = Vec::new();
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else { continue };
        let Ok(el) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };
        let source = el.text_content().unwrap_or_default();
        if source.trim().is_empty() {
            continue;
        }
        diagrams.push((el, source));
    }
    if diagrams.is_empty() {
        return;
    }

    if crate::pele::pele_ready() {
        for (el, source) in &diagrams {
            draw_mermaid(el, source);
        }
        return;
    }

    for (el, source) in &diagrams {
        el.set_inner_html(&format!(
            "<pre data-pele-state=\"pending\" aria-busy=\"true\">{}</pre>",
            html_escape(source)
        ));
    }
    wasm_bindgen_futures::spawn_local(async move {
        let loaded = crate::pele::load_pele().await;
        for (el, source) in &diagrams {
            if !el.is_connected() {
                continue;
            }
            if loaded {
                draw_mermaid(el, source);
            } else {
                let err =
                    crate::pele::PeleRenderError::load_failed(&crate::pele::pele_script_url());
                el.set_inner_html(&mermaid_fallback_html(source, &err));
            }
        }
    });
}

/// Draw one placeholder with the loaded pele, measuring labels in the
/// placeholder's own font.
fn draw_mermaid(el: &web_sys::Element, source: &str) {
    let options = crate::pele::pele_options_for(Some(el));
    match crate::pele::render(source, &options) {
        Ok(drawn) => el.set_inner_html(&drawn.svg),
        Err(err) => el.set_inner_html(&mermaid_fallback_html(source, &err)),
    }
}

/// The source, left visible so the author can see what is wrong, followed
/// by pele's message. Both escaped.
fn mermaid_fallback_html(source: &str, err: &crate::pele::PeleRenderError) -> String {
    format!(
        "<div class=\"mermaid-fallback\"><pre>{}</pre>{}</div>",
        html_escape(source),
        err.to_html()
    )
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
    crate::pele::escape_html(s)
}

#[cfg(test)]
mod tests {
    use super::mermaid_fallback_html;
    use crate::pele::PeleRenderError;

    #[test]
    fn fallback_keeps_the_source_visible_and_shows_pele_message_escaped() {
        let err = PeleRenderError::new("syntax", "Unexpected '<b>'");
        let html = mermaid_fallback_html("graph TD\n  A --> <b>", &err);
        assert!(
            html.starts_with(
                "<div class=\"mermaid-fallback\"><pre>graph TD\n  A --&gt; &lt;b&gt;</pre>"
            ),
            "{html}"
        );
        assert!(html.contains("class=\"mermaid-error"), "{html}");
        assert!(
            html.contains("Diagram error: Unexpected &#39;&lt;b&gt;&#39;"),
            "{html}"
        );
        assert!(!html.contains("<b>"), "{html}");
    }
}
