use super::style::MermaidTheme;
use crate::components::SvgDisplay;
use leptos::{html::Figure, prelude::*};

/// # Mermaid Diagram Display Component
///
/// A reactive Leptos component that renders mermaid diagram source text as inline SVG
/// using the native `markview-mermaid` renderer. Composes `SvgDisplay` internally.
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

        let is_dark = matches!(theme.get(), MermaidTheme::Dark);

        match markview_mermaid::parse(&src) {
            Ok(diagram) => {
                let render_theme = if is_dark {
                    markview_mermaid::Theme::dark()
                } else {
                    markview_mermaid::Theme::default()
                };
                let config = markview_mermaid::RenderConfig {
                    theme: render_theme,
                    ..Default::default()
                };
                markview_mermaid::render_with_config(&diagram, &config).unwrap_or_else(|e| {
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
