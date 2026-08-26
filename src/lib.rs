#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod auth;
pub mod charts;
pub mod components;
pub mod markdown;
pub mod motion;
pub mod patterns;
#[cfg(feature = "test-mode")]
pub mod test_mode;
pub mod theme;
pub mod tokens;
pub mod utils;
// Migrated composite widgets carry many self-descriptive `#[component]` prop
// and state fields; doc-enforcement is relaxed for this module rather than
// annotating every one. The hand-authored modules keep the crate's policy.
#[allow(missing_docs)]
pub mod widgets;
