//! # Empty State Component
//!
//! Centered icon + title + muted subtitle for empty regions, with an
//! optional action slot. Ported from d2d-ui's owner-drawn `EmptyState`
//! control -- daisyUI has no dedicated empty-state component, so this is a
//! plain flexbox layout composed from Tailwind utility classes.

mod component;
mod style;

pub use component::*;
pub use style::*;

#[cfg(test)]
mod tests;
