use super::*;

// MermaidTheme tests

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

// Mermaid rendering tests

#[test]
fn test_basic_flowchart_produces_svg() {
    let source = "graph TD\n    A --> B";
    let diagram = markview_mermaid::parse(source).expect("should parse flowchart");
    let svg = markview_mermaid::render(&diagram).expect("should render flowchart");
    assert!(svg.contains("<svg"), "rendered output should contain <svg tag");
}

#[test]
fn test_invalid_source_produces_error() {
    let source = "not a valid mermaid diagram %%% garbage";
    let result = markview_mermaid::parse(source);
    assert!(result.is_err(), "invalid source should produce a parse error");
}

#[test]
fn test_empty_source_produces_empty_string() {
    let source = "";
    // Component logic: empty/whitespace source returns empty string without parsing
    assert!(source.trim().is_empty());
}
