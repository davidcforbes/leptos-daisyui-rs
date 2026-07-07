//! # VerticalSteps Component
//!
//! A rich, status-driven vertical connection-path control, ported from
//! d2d-ui's owner-drawn `controls::vertical_steps::VerticalSteps`. See
//! [`VerticalSteps`] for usage, CSS notes, and guidance on when to reach for
//! it instead of the plain daisyUI [`Steps`](crate::components::Steps).

mod component;
mod style;
mod types;

pub use component::*;
pub use style::*;
pub use types::*;

#[cfg(test)]
mod tests;
