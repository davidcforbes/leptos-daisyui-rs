//! # ResultList Component
//!
//! A flat, ranked, keyboard-navigable results list (search picker), ported
//! from d2d-ui's `controls::result_list::ResultList`. See [`ResultList`] for
//! usage and CSS notes.

mod component;
mod core;
mod selection;
mod types;

pub use component::*;
pub use selection::{
    KeyedResultListSelection, KeyedResultListSelectionCause, KeyedResultListSelectionProposal,
};
pub use types::*;

#[cfg(test)]
mod tests;
