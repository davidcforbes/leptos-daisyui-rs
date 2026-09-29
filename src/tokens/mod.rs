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
//!
//! The emitters themselves (`ui_tokens_css`, `ui_animations_css`,
//! `dark_form_css_vars` and the step tables) live in the Leptos-free
//! `ldui_design::tokens` (ldui-3u3p) and are re-exported here unchanged;
//! only the two Leptos mount points are defined in this crate.

mod animations;
mod dark_palette;
mod preamble;

pub use animations::*;
pub use dark_palette::*;
pub use preamble::*;

#[cfg(test)]
mod tests;
