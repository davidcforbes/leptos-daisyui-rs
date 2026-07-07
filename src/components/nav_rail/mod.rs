//! # NavRail Component
//!
//! A vertical icon navigation rail with a selected-item pill, a left-edge
//! accent indicator, hover highlighting, and a bottom-pinned group support.
//! Ported from d2d-ui's owner-drawn `NavRail` control; daisyUI has no
//! dedicated rail component, so this composes `menu-vertical`-like structure
//! from plain Tailwind utility classes plus context-based active tracking.

mod component;
mod style;

pub use component::*;
pub use style::*;

#[cfg(test)]
mod tests;
