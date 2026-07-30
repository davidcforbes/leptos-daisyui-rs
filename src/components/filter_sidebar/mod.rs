//! # Filter Sidebar Component
//!
//! A left-hand collapsible filter panel: header + toggle, filter-search field,
//! stacked labelled fields, and a collapsed rail carrying the ACTIVE FILTER COUNT
//! and a vertical label.
//!
//! Every dimension is measured from a running reference (4Ease's
//! `pilot-filter-sidebar`) rather than chosen. daisyUI has no equivalent: its
//! `Drawer` is an off-canvas overlay driven by a hidden checkbox and its `Filter`
//! is a radio-button group, so neither is an inline panel that participates in
//! page layout and animates its own width.
//!
//! The collapsed rail's filter count is the single most important detail — a
//! collapsed filter panel with no such indication is how a filtered list gets
//! read as the whole list.

mod component;

pub use component::*;

#[cfg(test)]
mod tests;
