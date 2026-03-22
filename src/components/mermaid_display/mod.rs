//! # Mermaid Diagram Display Component
//!
//! Renders mermaid diagram syntax as inline SVG using native Rust rendering.

mod component;
mod style;
#[cfg(test)]
mod tests;

pub use component::*;
pub use style::*;
