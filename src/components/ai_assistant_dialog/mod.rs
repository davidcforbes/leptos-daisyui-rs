//! # AI Assistant Dialog
//!
//! The standard floating frame for an AI assistant: a right-docked modal with
//! a title row, host controls, a `<<` / `>>` toggle that widens it to 90% of
//! the window and back, and a close button (ldui-kq3w). Promoted from
//! 4iiz-Office's `AssistantDialog` (op-zm1ow.1).

mod component;

pub use component::*;

#[cfg(test)]
mod tests;
