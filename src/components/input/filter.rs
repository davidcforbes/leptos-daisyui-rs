//! Character-class input filters for the [`Input`](super::Input) component.
//!
//! Ported from d2d-ui's `TextField::InputFilter` (d2d-ui/src/controls/text_field.rs),
//! with the character classes narrowed to match this crate's acceptance criteria
//! (`Numeric` is digits-only here; d2d's also allows `- . ,`).

/// Restricts which characters an [`Input`](super::Input) accepts as the user
/// types. Enforced in the `on:input` handler, which strips any character the
/// active filter doesn't permit before the value is echoed back via
/// `on_input`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputFilter {
    /// No filtering -- every character is accepted (default).
    #[default]
    None,
    /// Digits only (`0`-`9`).
    Numeric,
    /// Digits plus phone punctuation: `(`, `)`, `+`, `-`, and space.
    Phone,
}

impl InputFilter {
    /// Whether `c` is allowed by this filter.
    pub fn permits(self, c: char) -> bool {
        match self {
            InputFilter::None => true,
            InputFilter::Numeric => c.is_ascii_digit(),
            InputFilter::Phone => c.is_ascii_digit() || matches!(c, '(' | ')' | '+' | '-' | ' '),
        }
    }

    /// Strip every character not permitted by this filter from `s`, preserving
    /// the order and byte content of the characters that remain.
    pub fn apply(self, s: &str) -> String {
        if self == InputFilter::None {
            return s.to_string();
        }
        s.chars().filter(|&c| self.permits(c)).collect()
    }
}
