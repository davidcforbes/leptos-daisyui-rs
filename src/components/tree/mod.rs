//! # Tree Component
//!
//! A lazy, expandable, keyboard-navigable tree view (file-explorer style),
//! ported from d2d-ui's `controls::tree::Tree`. See [`Tree`] for usage, the
//! keyboard model, and CSS notes; see [`types`] for the data model and the pure
//! flatten/navigation logic.
//!
//! Intentionally has no `style.rs`: daisyUI has no tree element, so there are
//! no daisyUI variant enums (color/size/style) to wrap — this is fully custom
//! Tailwind styling instead.

mod component;
mod types;

pub use component::*;
pub use types::*;

#[cfg(test)]
mod tests;
