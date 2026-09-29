//! The button demo's variant sections (`demo/src/demos/button.rs`).

use super::FixtureSection;
use crate::components::button::{ButtonColor, ButtonShape, ButtonSize, ButtonStyle};

/// The interaction state a demo button is shown in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonExampleState {
    /// Enabled, at rest.
    #[default]
    Normal,
    /// The pressed look: ldui's `active` prop (`btn-active`).
    Active,
    /// Disabled, with the reason every disabled action must give
    /// (ldui-p82h): ldui's `disabled_reason` prop.
    Disabled {
        /// The reason: the button's description and its `title`.
        reason: &'static str,
    },
    /// daisyUI's `loading` class on an otherwise ordinary button.
    Loading,
}

/// One button the demo renders: its text, every variant prop and its state.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonExample {
    /// The button's text.
    pub label: &'static str,
    /// The `color` prop.
    pub color: ButtonColor,
    /// The `style` prop.
    pub style: ButtonStyle,
    /// The `size` prop.
    pub size: ButtonSize,
    /// The `shape` prop.
    pub shape: ButtonShape,
    /// The interaction state.
    pub state: ButtonExampleState,
}

const fn example(
    label: &'static str,
    color: ButtonColor,
    style: ButtonStyle,
    size: ButtonSize,
    shape: ButtonShape,
    state: ButtonExampleState,
) -> ButtonExample {
    ButtonExample {
        label,
        color,
        style,
        size,
        shape,
        state,
    }
}

/// A button in `color` at the default style, size and shape, at rest.
const fn colored(label: &'static str, color: ButtonColor) -> ButtonExample {
    example(
        label,
        color,
        ButtonStyle::Default,
        ButtonSize::Md,
        ButtonShape::Default,
        ButtonExampleState::Normal,
    )
}

/// A default-colour button in `size`, at rest.
const fn sized(label: &'static str, size: ButtonSize) -> ButtonExample {
    example(
        label,
        ButtonColor::Default,
        ButtonStyle::Default,
        size,
        ButtonShape::Default,
        ButtonExampleState::Normal,
    )
}

/// A default-colour button in `style`, at rest.
const fn styled(label: &'static str, style: ButtonStyle) -> ButtonExample {
    example(
        label,
        ButtonColor::Default,
        style,
        ButtonSize::Md,
        ButtonShape::Default,
        ButtonExampleState::Normal,
    )
}

/// A primary button in `state`.
const fn in_state(label: &'static str, state: ButtonExampleState) -> ButtonExample {
    example(
        label,
        ButtonColor::Primary,
        ButtonStyle::Default,
        ButtonSize::Md,
        ButtonShape::Default,
        state,
    )
}

/// A primary button in `shape`, at rest.
const fn shaped(label: &'static str, shape: ButtonShape) -> ButtonExample {
    example(
        label,
        ButtonColor::Primary,
        ButtonStyle::Default,
        ButtonSize::Md,
        shape,
        ButtonExampleState::Normal,
    )
}

/// Every colour, at the default style, size and shape.
pub const COLORS: FixtureSection<ButtonExample> = FixtureSection {
    title: "Colors",
    examples: &[
        colored("Default", ButtonColor::Default),
        colored("Neutral", ButtonColor::Neutral),
        colored("Primary", ButtonColor::Primary),
        colored("Secondary", ButtonColor::Secondary),
        colored("Accent", ButtonColor::Accent),
        colored("Info", ButtonColor::Info),
        colored("Success", ButtonColor::Success),
        colored("Warning", ButtonColor::Warning),
        colored("Error", ButtonColor::Error),
    ],
};

/// Every size, in the default colour.
pub const SIZES: FixtureSection<ButtonExample> = FixtureSection {
    title: "Sizes",
    examples: &[
        sized("XS", ButtonSize::Xs),
        sized("SM", ButtonSize::Sm),
        sized("MD", ButtonSize::Md),
        sized("LG", ButtonSize::Lg),
        sized("XL", ButtonSize::Xl),
    ],
};

/// Every style, in the default colour.
pub const STYLES: FixtureSection<ButtonExample> = FixtureSection {
    title: "Styles",
    examples: &[
        styled("Default", ButtonStyle::Default),
        styled("Outline", ButtonStyle::Outline),
        styled("Ghost", ButtonStyle::Ghost),
        styled("Link", ButtonStyle::Link),
        styled("Soft", ButtonStyle::Soft),
        styled("Dash", ButtonStyle::Dash),
    ],
};

/// Every interaction state, in primary.
pub const STATES: FixtureSection<ButtonExample> = FixtureSection {
    title: "States",
    examples: &[
        in_state("Normal", ButtonExampleState::Normal),
        in_state("Active", ButtonExampleState::Active),
        in_state(
            "Disabled",
            ButtonExampleState::Disabled {
                reason: "Shown disabled for the state showcase",
            },
        ),
        in_state("Loading", ButtonExampleState::Loading),
    ],
};

/// Every non-default shape, in primary. The square and circle buttons carry
/// a single glyph, as the demo shows them.
pub const SHAPES: FixtureSection<ButtonExample> = FixtureSection {
    title: "Shapes",
    examples: &[
        shaped("Wide", ButtonShape::Wide),
        shaped("\u{25a1}", ButtonShape::Square),
        shaped("\u{25cb}", ButtonShape::Circle),
        shaped("Block", ButtonShape::Block),
    ],
};

/// The button demo's fixture sections, in page order.
pub const SECTIONS: [FixtureSection<ButtonExample>; 5] = [COLORS, SIZES, STYLES, STATES, SHAPES];

#[cfg(test)]
mod tests {
    use super::{ButtonExampleState, SECTIONS};

    /// A disabled example is what the demo, and every renderer of this
    /// list, shows as the pattern to copy, so it must carry a reason
    /// (ldui-p82h's `disabled-without-reason` audit rule).
    #[test]
    fn every_disabled_example_says_why() {
        for section in &SECTIONS {
            for example in section.examples {
                if let ButtonExampleState::Disabled { reason } = example.state {
                    assert!(
                        !reason.trim().is_empty(),
                        "{} / {}: a disabled example needs a reason",
                        section.title,
                        example.label
                    );
                }
            }
        }
    }
}
