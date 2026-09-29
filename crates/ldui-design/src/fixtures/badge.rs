//! The badge demo's variant sections (`demo/src/demos/badge.rs`).

use super::FixtureSection;
use crate::components::badge::{BadgeColor, BadgeSize, BadgeStyle};

/// One badge the demo renders: its text and every variant prop.
#[derive(Clone, Debug, PartialEq)]
pub struct BadgeExample {
    /// The badge's text.
    pub label: &'static str,
    /// The `color` prop.
    pub color: BadgeColor,
    /// The `style` prop.
    pub style: BadgeStyle,
    /// The `size` prop.
    pub size: BadgeSize,
}

const fn example(
    label: &'static str,
    color: BadgeColor,
    style: BadgeStyle,
    size: BadgeSize,
) -> BadgeExample {
    BadgeExample {
        label,
        color,
        style,
        size,
    }
}

/// A badge in `color` at the default style and size.
const fn colored(label: &'static str, color: BadgeColor) -> BadgeExample {
    example(label, color, BadgeStyle::Default, BadgeSize::Md)
}

/// A default-colour badge in `size`.
const fn sized(label: &'static str, size: BadgeSize) -> BadgeExample {
    example(label, BadgeColor::Default, BadgeStyle::Default, size)
}

/// A primary badge in `style`.
const fn styled(label: &'static str, style: BadgeStyle) -> BadgeExample {
    example(label, BadgeColor::Primary, style, BadgeSize::Md)
}

/// Every colour, at the default style and size.
pub const COLORS: FixtureSection<BadgeExample> = FixtureSection {
    title: "Colors",
    examples: &[
        colored("Default", BadgeColor::Default),
        colored("Neutral", BadgeColor::Neutral),
        colored("Primary", BadgeColor::Primary),
        colored("Secondary", BadgeColor::Secondary),
        colored("Accent", BadgeColor::Accent),
        colored("Info", BadgeColor::Info),
        colored("Success", BadgeColor::Success),
        colored("Warning", BadgeColor::Warning),
        colored("Error", BadgeColor::Error),
    ],
};

/// Every size, at the default colour and style.
pub const SIZES: FixtureSection<BadgeExample> = FixtureSection {
    title: "Sizes",
    examples: &[
        sized("XS", BadgeSize::Xs),
        sized("SM", BadgeSize::Sm),
        sized("MD", BadgeSize::Md),
        sized("LG", BadgeSize::Lg),
        sized("XL", BadgeSize::Xl),
    ],
};

/// Every style, in primary so the style is visible.
pub const STYLES: FixtureSection<BadgeExample> = FixtureSection {
    title: "Styles",
    examples: &[
        styled("Default", BadgeStyle::Default),
        styled("Outline", BadgeStyle::Outline),
        styled("Ghost", BadgeStyle::Ghost),
        styled("Soft", BadgeStyle::Soft),
        styled("Dash", BadgeStyle::Dash),
    ],
};

/// The badge demo's fixture sections, in page order.
pub const SECTIONS: [FixtureSection<BadgeExample>; 3] = [COLORS, SIZES, STYLES];
