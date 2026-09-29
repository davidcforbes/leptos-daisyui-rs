#![warn(missing_docs)]
#![forbid(unsafe_code)]
//! The Leptos-free design layer of `leptos-daisyui-rs` (ldui-3u3p).
//!
//! Four things a non-Leptos consumer (4iiz-kit's askama + htmx components)
//! needs to render the same design system, with no dependency on Leptos,
//! `web-sys` or the out-of-repo `ui-tokens` crate:
//!
//! - [`tokens`]: the `--ld-*` custom-property block, the shared animation
//!   primitives and the dark form-control palette as CSS strings, plus the
//!   generated token values they are built from.
//! - [`components`]: every component's variant enums and class helpers
//!   (`ButtonColor::Primary.as_str() == "btn-primary"`), one module per
//!   component and also flattened into the crate root, so both
//!   `ldui_design::button::ButtonColor` and `ldui_design::ButtonColor` work.
//! - [`contracts`]: the page-contract vocabulary (`PageState`,
//!   `PageContractV2`, …).
//! - [`fixtures`]: the examples the demo's badge, button and alert variant
//!   sections render, as plain data a second renderer can render too
//!   (ldui-rz43).
//!
//! `leptos-daisyui-rs` depends on this crate and re-exports every item at
//! its historical path, so its own consumers see no change.
//!
//! Class literals live in this crate's sources, so a Tailwind build must
//! scan them: add `@source "<path-to>/crates/ldui-design/src/**/*.rs";` next
//! to the existing `@source` for `leptos-daisyui-rs/src`.

pub mod components;
pub mod contracts;
pub mod fixtures;
pub mod tokens;

pub use components::*;
