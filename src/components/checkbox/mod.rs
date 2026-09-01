//! # Checkbox Component
//!
//! For more information, see: <https://daisyui.com/components/checkbox/>

mod component;
mod state;
mod style;
#[cfg(test)]
mod tests;

pub use component::*;
pub use state::{CheckboxBinding, CheckboxChangeProposal, CheckboxState};
pub use style::*;
