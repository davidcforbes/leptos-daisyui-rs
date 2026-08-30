/// # Button Color Variants
///
/// Style enum for daisyUI button color classes that control the semantic color scheme
/// of button components. Colors follow daisyUI's semantic color system.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ButtonColor {
    /// Default button color (no color class applied)
    #[default]
    Default,

    /// Neutral dark color for non-saturated UI elements
    Neutral,

    /// Primary brand color for main actions
    Primary,

    /// Secondary brand color for secondary actions
    Secondary,

    /// Accent brand color for highlighted actions
    Accent,

    /// Info color for informational actions
    Info,

    /// Success color for positive/safe actions
    Success,

    /// Warning color for caution actions
    Warning,

    /// Error color for destructive/dangerous actions
    Error,
}

impl ButtonColor {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            ButtonColor::Default => "",
            ButtonColor::Neutral => "btn-neutral",
            ButtonColor::Primary => "btn-primary",
            ButtonColor::Secondary => "btn-secondary",
            ButtonColor::Accent => "btn-accent",
            ButtonColor::Info => "btn-info",
            ButtonColor::Success => "btn-success",
            ButtonColor::Warning => "btn-warning",
            ButtonColor::Error => "btn-error",
        }
    }
}

/// # Button Style Variants
///
/// Style enum for daisyUI button style classes that control the visual appearance
/// and treatment of button components.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ButtonStyle {
    /// Default filled button style

    #[default]
    Default,

    /// Transparent background with colored border
    Outline,

    /// Dashed border style
    Dash,

    /// Subtle background with soft appearance
    Soft,

    /// Transparent background, shows color on hover
    Ghost,

    /// Styled like a text link
    Link,
}

impl ButtonStyle {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            ButtonStyle::Default => "",
            ButtonStyle::Outline => "btn-outline",
            ButtonStyle::Dash => "btn-dash",
            ButtonStyle::Soft => "btn-soft",
            ButtonStyle::Ghost => "btn-ghost",
            ButtonStyle::Link => "btn-link",
        }
    }
}

/// # Button Size Variants
///
/// Style enum for daisyUI button size classes that control the dimensions
/// and typography scale of button components.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ButtonSize {
    /// Extra small button size
    Xs,

    /// Small button size
    Sm,

    /// Medium button size (default)

    #[default]
    Md,

    /// Large button size
    Lg,

    /// Extra large button size
    Xl,
}

impl ButtonSize {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            ButtonSize::Xs => "btn-xs",
            ButtonSize::Sm => "btn-sm",
            ButtonSize::Md => "btn-md",
            ButtonSize::Lg => "btn-lg",
            ButtonSize::Xl => "btn-xl",
        }
    }
}

/// # Button Type Variants
///
/// The native HTML `type` attribute of the `<button>` element, controlling
/// how the button participates in its containing `<form>`. This is separate
/// from [`ButtonColor`]/[`ButtonStyle`]/[`ButtonSize`]/[`ButtonShape`], which
/// are purely presentational — `ButtonType` changes browser *behavior*.
///
/// See [`Button`](super::Button)'s doc comment for the full precedence and
/// disabled/loading-interaction contract (ldui-9vs).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ButtonType {
    /// No default form action. Use for actions handled entirely via
    /// `on_click` (the pre-existing, and still default, behavior).
    #[default]
    Button,

    /// Submits the containing `<form>` using that form's own `action`/
    /// `method`/`target` — a real native browser submission, not a JS shim.
    /// `Button` does not own or infer form action/method/target; the
    /// consumer's `<form>` element does.
    Submit,

    /// Resets the containing `<form>`'s fields to their initial
    /// (`defaultValue`/`defaultChecked`) values, per native `<form>` reset
    /// behavior. `Button` adds no JS for this — the browser does it.
    Reset,
}

impl ButtonType {
    /// The literal `type` attribute value emitted on the `<button>` element.
    pub fn as_str(&self) -> &'static str {
        match self {
            ButtonType::Button => "button",
            ButtonType::Submit => "submit",
            ButtonType::Reset => "reset",
        }
    }
}

/// # Button Shape Variants
///
/// Style enum for daisyUI button shape/layout classes that control the geometry
/// and layout behavior of button components.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ButtonShape {
    /// Default button shape

    #[default]
    Default,

    /// Wider than normal button
    Wide,

    /// Full width button (block-level)
    Block,

    /// Square aspect ratio button
    Square,

    /// Circular button shape
    Circle,
}

impl ButtonShape {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            ButtonShape::Default => "",
            ButtonShape::Wide => "btn-wide",
            ButtonShape::Block => "btn-block",
            ButtonShape::Square => "btn-square",
            ButtonShape::Circle => "btn-circle",
        }
    }
}
