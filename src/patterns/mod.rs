//! Opinionated page patterns and their machine-checkable contracts.

mod contracts;
mod macros;

pub use crate::{filter_schema, page_contract};
pub use contracts::{
    ClientSnapshotContract, ContractError, DatasetBehavior, FilterSchema, PageBreakpoint,
    PageContract, PagePattern, PageState,
};

#[cfg(test)]
mod tests;
