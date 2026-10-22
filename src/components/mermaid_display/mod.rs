//! # Mermaid Diagram Display Component
//!
//! Renders Mermaid diagram source as inline SVG with pele, lazy-loaded in the
//! browser (see [`crate::pele`]). Colours follow the daisyUI theme through
//! the `--pele-*` CSS variables mapped in `styles/tokens.css`.

mod component;
mod style;
#[cfg(test)]
mod tests;

pub use component::*;
pub use style::*;
