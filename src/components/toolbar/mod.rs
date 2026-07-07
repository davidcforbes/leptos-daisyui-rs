//! # Toolbar Component
//!
//! Horizontal command strip of icon/label buttons — ported from d2d-ui's
//! `controls::toolbar::Toolbar` (a self-painting Direct2D control) to a Leptos
//! + daisyUI composition of `join` + `btn` + `tooltip` + `dropdown`.

mod component;
mod style;
mod types;

pub use component::*;
pub use style::*;
pub use types::*;

#[cfg(test)]
mod tests;
