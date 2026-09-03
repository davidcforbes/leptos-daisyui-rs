/// # Alert Style Variants
///
/// Style enum for daisyUI alert style classes that control the visual appearance
/// and treatment of alert components.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum AlertStyle {
    /// Default filled alert style
    #[default]
    Default,

    /// Transparent background with colored border
    Outline,

    /// Dashed border style
    Dash,

    /// Subtle background with soft appearance
    Soft,
}

impl AlertStyle {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertStyle::Default => "",
            AlertStyle::Outline => "alert-outline",
            AlertStyle::Dash => "alert-dash",
            AlertStyle::Soft => "alert-soft",
        }
    }
}

/// # Alert Color Variants
///
/// Style enum for daisyUI alert color classes that control the semantic color scheme
/// of alert components. Colors convey the meaning and urgency of the alert message.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum AlertColor {
    /// Default alert color (no color class applied)
    #[default]
    Default,

    /// Info color for informational messages
    Info,

    /// Success color for positive feedback
    Success,

    /// Warning color for caution messages
    Warning,

    /// Error color for critical/error messages
    Error,
}

impl AlertColor {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertColor::Default => "",
            AlertColor::Info => "alert-info",
            AlertColor::Success => "alert-success",
            AlertColor::Warning => "alert-warning",
            AlertColor::Error => "alert-error",
        }
    }
}

/// # Alert Direction Variants
///
/// Style enum for daisyUI alert direction classes that control the layout orientation
/// of alert components and their content arrangement.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum AlertDirection {
    /// Default layout direction
    #[default]
    Default,

    /// Vertical layout with stacked content
    Vertical,

    /// Horizontal layout with inline content
    Horizontal,
}

impl AlertDirection {
    /// CSS class string
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertDirection::Default => "",
            AlertDirection::Vertical => "alert-vertical",
            AlertDirection::Horizontal => "alert-horizontal",
        }
    }
}

/// How assistive technology should treat an [`Alert`](super::Alert)'s content
/// (`ldui-fmiu`).
///
/// `role="alert"` carries an implicit `aria-live="assertive"`, which tells a
/// screen reader to **interrupt the user** and announce the content at once.
/// That is right for a transient message and wrong for a permanent panel — and
/// consumers reach for `Alert` for both, because it is the only component with
/// that visual treatment.
///
/// The live case that prompted this: a static "Why they did not hire" panel,
/// present on every page load and never updated, interrupting a screen-reader
/// user every single time. Nothing about it was an alert; it just needed a
/// soft-warning-coloured box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertLiveness {
    /// `role="alert"` — interrupts the user. The default, so every existing
    /// call site keeps today's behaviour exactly.
    ///
    /// Correct for a message that appears in response to something the user
    /// did and that they must hear about now: "Send failed", "Session expired".
    #[default]
    Assertive,
    /// `role="status"` — announced at the next natural pause instead of
    /// interrupting.
    ///
    /// Correct for a message that updates in place and matters, but not
    /// urgently: "Saved", "3 of 5 uploaded".
    Polite,
    /// No ARIA role at all — a plain styled container.
    ///
    /// Correct for permanent page furniture that happens to want the alert
    /// visual: a standing informational panel that is part of the record, not
    /// an event. A live region that never changes has nothing to announce, and
    /// announcing it on every load is pure noise.
    Static,
}

impl AlertLiveness {
    /// The ARIA role this liveness emits, or `None` for [`Self::Static`].
    pub const fn role(self) -> Option<&'static str> {
        match self {
            Self::Assertive => Some("alert"),
            Self::Polite => Some("status"),
            Self::Static => None,
        }
    }
}
