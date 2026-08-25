//! # Unstyled Pressable Primitive
//!
//! A real `<button>` with the library's behavioral contract (type,
//! disabled, focus ring, press affordance, auditable marker) and **no**
//! daisyUI `.btn` geometry — for actions that are semantically buttons but
//! visually menu items, icons, cells, cards, or links.

mod component;

#[cfg(test)]
mod tests;

pub use component::*;
