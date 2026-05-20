use super::style::MermaidTheme;
use crate::components::SvgDisplay;
use editmark_mermaid::{RenderConfig, Theme, detect_init, remove_directives};
use leptos::{html::Figure, prelude::*};

/// # Mermaid Diagram Display Component
///
/// A reactive Leptos component that renders mermaid diagram source text as inline SVG
/// using the native `editmark-mermaid` renderer. Composes `SvgDisplay` internally.
///
/// Supports `%%{init}%%` directives for theme and spacing overrides, and
/// automatically decodes HTML entities (`&lt;`, `&gt;`, `&amp;`, `&quot;`)
/// before parsing.
///
/// ## Node References
/// - `node_ref` - References the `<figure>` element ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement))
#[component]
pub fn MermaidDisplay(
    /// Mermaid diagram source text
    #[prop(into)]
    source: Signal<String>,

    /// Color theme for the diagram
    #[prop(optional, into)]
    theme: Signal<MermaidTheme>,

    /// Maximum width in pixels (CSS max-width)
    #[prop(optional, into)]
    max_width: Signal<Option<f32>>,

    /// Maximum height in pixels (CSS max-height)
    #[prop(optional, into)]
    max_height: Signal<Option<f32>>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the figure element
    #[prop(optional)]
    node_ref: NodeRef<Figure>,
) -> impl IntoView {
    // Reactively render mermaid source to SVG
    let svg_content = Memo::new(move |_| {
        let src = source.get();
        if src.trim().is_empty() {
            return String::new();
        }

        // Decode HTML entities before processing
        let src = src
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"");

        // Process init directives before parsing
        let directive_config = detect_init(&src);
        let clean_source = remove_directives(&src);

        match editmark_mermaid::parse(&clean_source) {
            Ok(diagram) => {
                let is_dark = matches!(theme.get(), MermaidTheme::Dark);

                // Build theme: directive overrides take precedence over prop
                let render_theme = if let Some(ref dc) = directive_config {
                    if dc.theme.is_some() || !dc.theme_variables.is_empty() {
                        Theme::from_directive(dc)
                    } else if is_dark {
                        Theme::dark()
                    } else {
                        Theme::default()
                    }
                } else if is_dark {
                    Theme::dark()
                } else {
                    Theme::default()
                };

                let mut config = RenderConfig {
                    theme: render_theme,
                    theme_css: directive_config
                        .as_ref()
                        .and_then(|dc| dc.theme_css.clone()),
                    ..Default::default()
                };

                // Extract spacing overrides from directive config
                if let Some(ref dc) = directive_config {
                    for key in &["flowchart", "sequence", "class", "state", "er"] {
                        if let Some(serde_json::Value::Object(obj)) = dc.extra.get(*key) {
                            if let Some(serde_json::Value::Number(n)) = obj.get("nodeSpacing") {
                                config.node_spacing = n.as_f64();
                            }
                            if let Some(serde_json::Value::Number(n)) = obj.get("rankSpacing") {
                                config.rank_spacing = n.as_f64();
                            }
                        }
                    }
                }

                editmark_mermaid::render_with_config(&diagram, &config).unwrap_or_else(|e| {
                    format!("<pre class=\"mermaid-error\">Render error: {}</pre>", e)
                })
            }
            Err(e) => {
                format!("<pre class=\"mermaid-error\">Parse error: {}</pre>", e)
            }
        }
    });

    view! {
        <SvgDisplay
            content=Signal::derive(move || svg_content.get())
            alt="Mermaid diagram"
            max_width=max_width
            max_height=max_height
            class=class
            node_ref=node_ref
        />
    }
}
