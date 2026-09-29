/// # Mermaid Theme Variants
///
/// Style enum for controlling the color theme of rendered mermaid diagrams.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MermaidTheme {
    /// Light theme (default)
    #[default]
    Default,

    /// Dark theme for dark backgrounds
    Dark,

    /// Automatically follow system preference
    Auto,
}

impl MermaidTheme {
    /// Theme identifier string
    pub fn as_str(&self) -> &'static str {
        match self {
            MermaidTheme::Default => "light",
            MermaidTheme::Dark => "dark",
            MermaidTheme::Auto => "auto",
        }
    }
}
