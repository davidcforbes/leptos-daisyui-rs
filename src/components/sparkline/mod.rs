//! # Sparkline Component
//!
//! Small time-series line chart -- an inline SVG polyline with an optional
//! framed card and current/peak readout. Ported from d2d-ui's owner-drawn
//! `Sparkline` control. daisyUI has no dedicated sparkline component.

mod component;
mod style;

pub use component::*;
pub use style::*;

#[cfg(test)]
mod tests;
