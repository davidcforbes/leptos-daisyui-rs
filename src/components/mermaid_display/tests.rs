use super::component::{decode_entities, outcome_html, pending_html};
use super::*;
use crate::pele::{PeleRenderError, PeleSvg};

// MermaidTheme tests (the prop is kept for API compatibility and reflected
// as `data-mermaid-theme`; it no longer recolours).

#[test]
fn test_mermaid_theme_default() {
    let theme = MermaidTheme::default();
    assert_eq!(theme.as_str(), "light");
}

#[test]
fn test_mermaid_theme_all_variants() {
    let themes = vec![
        (MermaidTheme::Default, "light"),
        (MermaidTheme::Dark, "dark"),
        (MermaidTheme::Auto, "auto"),
    ];

    for (theme, expected) in themes {
        assert_eq!(theme.as_str(), expected);
    }
}

#[test]
fn test_mermaid_theme_clone() {
    let theme1 = MermaidTheme::Dark;
    let theme2 = theme1.clone();
    assert_eq!(theme1.as_str(), theme2.as_str());
}

// Render-state and markup tests. Rendering itself calls the browser global
// `pele` and is covered by `cargo xtask test-mermaid-pele`, never natively.

#[test]
fn render_state_attribute_values() {
    assert_eq!(MermaidRenderState::default(), MermaidRenderState::Empty);
    let states = [
        (MermaidRenderState::Empty, "empty"),
        (MermaidRenderState::Pending, "pending"),
        (MermaidRenderState::Rendered, "rendered"),
        (MermaidRenderState::Error, "error"),
    ];
    for (state, expected) in states {
        assert_eq!(state.as_str(), expected);
    }
}

#[test]
fn entities_decode_in_the_historical_order() {
    assert_eq!(
        decode_entities("A --&gt; B[&quot;x &amp; y&quot;] &lt;br&gt;"),
        "A --> B[\"x & y\"] <br>"
    );
    // `&amp;` is decoded after `&lt;`, so a double-escaped entity stays one
    // level escaped rather than turning into markup.
    assert_eq!(decode_entities("&amp;lt;"), "&lt;");
}

#[test]
fn pending_state_shows_the_source_escaped_and_muted() {
    let html = pending_html("flowchart LR\n  A[<b>x</b>] --> B");
    assert!(html.starts_with("<pre class=\"mermaid-pending"), "{html}");
    assert!(html.contains("text-base-content/75"), "{html}");
    assert!(html.contains("aria-busy=\"true\""), "{html}");
    assert!(!html.contains("<b>"), "source must be escaped: {html}");
    assert!(html.contains("A[&lt;b&gt;x&lt;/b&gt;] --&gt; B"), "{html}");
}

#[test]
fn outcome_maps_svg_to_rendered_and_errors_to_the_error_box() {
    let (html, state) = outcome_html(Ok(PeleSvg {
        svg: "<svg class=\"pele pele-flowchart\"></svg>".to_string(),
        width: 10.0,
        height: 20.0,
    }));
    assert_eq!(state, MermaidRenderState::Rendered);
    assert_eq!(html, "<svg class=\"pele pele-flowchart\"></svg>");

    let (html, state) = outcome_html(Err(PeleRenderError::new(
        "syntax",
        "Expected '-->' but found \"<script>\"",
    )));
    assert_eq!(state, MermaidRenderState::Error);
    assert!(html.contains("class=\"mermaid-error"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
}

#[test]
fn test_empty_source_produces_empty_string() {
    // Component logic: blank source renders nothing and never loads pele.
    assert!(decode_entities("  \n ").trim().is_empty());
}

// Theme mapping: the shipped stylesheet maps every colour pele reads onto
// the daisyUI 5 theme variables, so diagrams follow `data-theme`.

/// The `.pele { ... }` rule body in `styles/tokens.css`.
fn pele_rule() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/styles/tokens.css");
    let css = std::fs::read_to_string(path).expect("styles/tokens.css must exist");
    let start = css.find(".pele {").unwrap_or_else(|| {
        panic!("styles/tokens.css has no `.pele` rule; run `cargo xtask gen-tokens` (ldui-qe72)")
    });
    let body = &css[start..];
    body[..body.find('}').expect(".pele rule must close")].to_string()
}

#[test]
fn stylesheet_maps_every_pele_colour_to_a_daisyui_variable() {
    let rule = pele_rule();
    let colours = [
        "--pele-bg",
        "--pele-fg",
        "--pele-muted",
        "--pele-line",
        "--pele-surface",
        "--pele-surface-alt",
        "--pele-border",
        "--pele-accent",
        "--pele-critical",
        "--pele-error",
        "--pele-series-1",
        "--pele-series-2",
        "--pele-series-3",
        "--pele-series-4",
        "--pele-series-5",
        "--pele-series-6",
        "--pele-series-7",
        "--pele-series-8",
    ];
    for name in colours {
        let decl = rule
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{name}:")))
            .unwrap_or_else(|| panic!("`.pele` does not set {name}:\n{rule}"));
        assert!(
            decl.contains("var(--color-"),
            "{name} must derive from a daisyUI theme variable: {decl}"
        );
    }
    for name in ["--pele-radius", "--pele-title-weight"] {
        assert!(
            rule.contains(&format!("{name}:")),
            "`.pele` must set {name}"
        );
    }
}

#[test]
fn stylesheet_leaves_pele_fonts_to_the_measured_container_font() {
    // pele measures labels with the font render() is given; a CSS font that
    // differs from it would paint labels wider than their boxes.
    let rule = pele_rule();
    assert!(!rule.contains("--pele-font"), "{rule}");
}
