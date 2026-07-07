//! # IconTile Component
//!
//! A tinted rounded-square tile framing a centered icon glyph, with
//! independent background/foreground colors. Ported from d2d-ui's
//! owner-drawn `IconTile` control -- daisyUI has no dedicated equivalent, so
//! this is a plain `<div>` styled with Tailwind utility classes.

mod component;
mod style;

pub use component::*;
pub use style::*;

#[cfg(test)]
mod tests;
