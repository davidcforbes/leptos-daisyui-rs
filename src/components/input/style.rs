/// # Input Style Variants
///
/// Style enum for daisyUI input style classes that control the visual appearance
/// of input components.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum InputStyle {
    /// Default input style (no style class applied)
    #[default]
    Default,

    /// Ghost style with transparent background
    Ghost,
}

impl InputStyle {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            InputStyle::Default => "",
            InputStyle::Ghost => "input-ghost",
        }
    }
}

/// # Input Color Variants
///
/// Style enum for daisyUI input color classes that control the semantic color scheme
/// of input components. Colors follow daisyUI's semantic system for context and meaning.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum InputColor {
    /// Default input color (no color class applied)
    #[default]
    Default,

    /// Neutral color for subdued inputs
    Neutral,

    /// Primary brand color for main action inputs
    Primary,

    /// Secondary brand color for secondary inputs
    Secondary,

    /// Accent brand color for highlighted inputs
    Accent,

    /// Info color for informational inputs
    Info,

    /// Success color for positive action inputs
    Success,

    /// Warning color for cautionary inputs
    Warning,

    /// Error color for error state inputs
    Error,
}

impl InputColor {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            InputColor::Default => "",
            InputColor::Neutral => "input-neutral",
            InputColor::Primary => "input-primary",
            InputColor::Secondary => "input-secondary",
            InputColor::Accent => "input-accent",
            InputColor::Info => "input-info",
            InputColor::Success => "input-success",
            InputColor::Warning => "input-warning",
            InputColor::Error => "input-error",
        }
    }
}

/// # Input Size Variants
///
/// Style enum for daisyUI input size classes that control the physical dimensions
/// of input components. Sizes scale proportionally for various contexts.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum InputSize {
    /// Extra small size for compact layouts
    Xs,

    /// Small size for minimal space usage
    Sm,

    /// Medium size for standard usage
    #[default]
    Md,

    /// Large size for emphasis and visibility
    Lg,

    /// Extra large size for prominent display
    Xl,
}

impl InputSize {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            InputSize::Xs => "input-xs",
            InputSize::Sm => "input-sm",
            InputSize::Md => "input-md",
            InputSize::Lg => "input-lg",
            InputSize::Xl => "input-xl",
        }
    }
}

/// # Input Type Variants
///
/// Maps to the HTML `type` attribute of the underlying `<input>` element,
/// controlling browser-level input semantics (keyboard, validation, password
/// masking, etc.). Ported from d2d-ui's password/leading-icon `TextField`
/// (d2d-ui/src/controls/text_field.rs).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum InputType {
    /// Plain single-line text (default)
    #[default]
    Text,

    /// Password field; browsers mask the value. Combine with `revealable` on
    /// [`Input`](super::Input) to add a show/hide toggle.
    Password,

    /// Email address; browsers may offer format validation/autocomplete
    Email,

    /// Numeric entry; browsers may show a numeric keypad on mobile
    Number,

    /// Telephone number; browsers may show a phone keypad on mobile
    Tel,

    /// Search field; browsers may add a native clear affordance
    Search,

    /// URL; browsers may offer format validation/autocomplete
    Url,

    /// Calendar date (`YYYY-MM-DD`); browsers typically render a native date
    /// picker. Value parsing/validation, timezone policy, and `min`/`max`/
    /// `step` are caller-owned — pass them as spread attrs (`attr:min`,
    /// `attr:max`, `attr:step`). Ported from d2d-ui's desktop date field
    /// (see the `Precedence vs a spread` note on
    /// [`Input`](super::component::Input) for `attr:type`'s own caveat).
    Date,

    /// Time of day (`HH:MM` or `HH:MM:SS`); browsers typically render a
    /// native time picker. Same caller-owned parsing/`min`/`max`/`step`
    /// contract as [`InputType::Date`].
    Time,

    /// Year-and-month (`YYYY-MM`); browsers typically render a native
    /// month picker. Same caller-owned contract as [`InputType::Date`].
    Month,

    /// ISO week (`YYYY-Www`); browsers typically render a native week
    /// picker. Same caller-owned contract as [`InputType::Date`].
    Week,

    /// Local date and time with no timezone offset
    /// (`YYYY-MM-DDTHH:MM[:SS]`); browsers typically render a combined
    /// date/time picker. Same caller-owned contract as [`InputType::Date`]
    /// — in particular, LDUI applies no timezone conversion; the string is
    /// passed through verbatim.
    DateTimeLocal,
}

impl InputType {
    /// HTML `type` attribute value
    pub fn as_str(&self) -> &'static str {
        match self {
            InputType::Text => "text",
            InputType::Password => "password",
            InputType::Email => "email",
            InputType::Number => "number",
            InputType::Tel => "tel",
            InputType::Search => "search",
            InputType::Url => "url",
            InputType::Date => "date",
            InputType::Time => "time",
            InputType::Month => "month",
            InputType::Week => "week",
            InputType::DateTimeLocal => "datetime-local",
        }
    }
}
