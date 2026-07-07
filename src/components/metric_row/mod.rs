//! # Metric Row Component
//!
//! A compact `label ... value` key/value row for facts grids and detail
//! panels, with an optional stacked layout and hairline bottom divider.
//! Ported from d2d-ui's owner-drawn `MetricRow` control -- daisyUI has no
//! dedicated key/value row component, so this is a plain flexbox layout
//! composed from Tailwind utility classes.

mod component;
mod style;

pub use component::*;
pub use style::*;

#[cfg(test)]
mod tests;
