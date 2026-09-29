//! The alert demo's variant sections (`demo/src/demos/alert.rs`).

use super::FixtureSection;
use crate::components::alert::{AlertColor, AlertStyle};

/// One alert the demo renders: its message, its variant props and whether it
/// leads with a status icon.
#[derive(Clone, Debug, PartialEq)]
pub struct AlertExample {
    /// The alert's message.
    pub message: &'static str,
    /// The `color` prop.
    pub color: AlertColor,
    /// The `style` prop.
    pub style: AlertStyle,
    /// Whether the alert leads with its colour's status icon. Which glyph
    /// that is belongs to the renderer's icon set.
    pub icon: bool,
}

const fn example(
    message: &'static str,
    color: AlertColor,
    style: AlertStyle,
    icon: bool,
) -> AlertExample {
    AlertExample {
        message,
        color,
        style,
        icon,
    }
}

/// Every status colour, in the default style, each with its status icon.
pub const COLORS: FixtureSection<AlertExample> = FixtureSection {
    title: "Colors",
    examples: &[
        example(
            "Info alert - Something noteworthy",
            AlertColor::Info,
            AlertStyle::Default,
            true,
        ),
        example(
            "Success! Task completed",
            AlertColor::Success,
            AlertStyle::Default,
            true,
        ),
        example(
            "Warning - Please review",
            AlertColor::Warning,
            AlertStyle::Default,
            true,
        ),
        example(
            "Error - Something went wrong",
            AlertColor::Error,
            AlertStyle::Default,
            true,
        ),
    ],
};

/// Every style, each on a different status colour, without icons.
pub const STYLES: FixtureSection<AlertExample> = FixtureSection {
    title: "Style Variants",
    examples: &[
        example(
            "Default filled style",
            AlertColor::Info,
            AlertStyle::Default,
            false,
        ),
        example(
            "Outline style with border",
            AlertColor::Success,
            AlertStyle::Outline,
            false,
        ),
        example(
            "Dashed border style",
            AlertColor::Warning,
            AlertStyle::Dash,
            false,
        ),
        example(
            "Soft subtle background",
            AlertColor::Error,
            AlertStyle::Soft,
            false,
        ),
    ],
};

/// The alert demo's fixture sections, in page order.
pub const SECTIONS: [FixtureSection<AlertExample>; 2] = [COLORS, STYLES];
