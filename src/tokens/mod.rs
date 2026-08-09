//! Design tokens shared with the sibling `d2d-ui` workspace via the
//! `ui-tokens` crate.
//!
//! [`UiTokensPreamble`] emits a `<style>` element that exposes the
//! `ui-tokens` durations, easings, and elevation tiers as `--ld-*` CSS
//! custom properties on `:root`. Mount it once near the root of your app:
//!
//! ```ignore
//! use leptos::prelude::*;
//! use leptos_daisyui_rs::tokens::UiTokensPreamble;
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     view! {
//!         <UiTokensPreamble />
//!         // ...rest of the app
//!     }
//! }
//! ```

mod animations;
mod dark_palette;
mod preamble;

pub use animations::*;
pub use dark_palette::*;
pub use preamble::*;

#[cfg(test)]
mod tests;
