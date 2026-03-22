/// # SVG Fit Variants
///
/// Style enum for CSS object-fit classes that control how SVG content
/// is sized within its container.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SvgFit {
    /// Default fit (no class applied)
    #[default]
    Default,

    /// Scale to fit within container while maintaining aspect ratio
    Contain,

    /// Scale to cover container while maintaining aspect ratio
    Cover,

    /// Stretch to fill container exactly
    Fill,

    /// Scale down only if larger than container
    ScaleDown,

    /// No scaling, display at natural size
    None,
}

impl SvgFit {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            SvgFit::Default => "",
            SvgFit::Contain => "object-contain",
            SvgFit::Cover => "object-cover",
            SvgFit::Fill => "object-fill",
            SvgFit::ScaleDown => "object-scale-down",
            SvgFit::None => "object-none",
        }
    }
}
