//! Design tokens as CSS text, shared with the Direct2D desktop face.
//!
//! The values come from the `ui-tokens` crate in Rust-DeskApp, but this crate
//! never depends on it: `cargo xtask gen-tokens` copies the values it needs
//! into [`generated`], and `gen-tokens --check` (a `cargo xtask verify` step)
//! fails when that file drifts from `ui-tokens`. The emitters below turn
//! those values into the `--ld-*` custom-property block, the shared
//! animation primitives and the dark form-control palette.
//!
//! `leptos-daisyui-rs` mounts the same strings through its
//! `UiTokensPreamble` / `UiAnimationsPreamble` components and re-exports the
//! emitters at `leptos_daisyui_rs::tokens`.

mod animations;
mod dark_palette;
/// Token values generated from `ui-tokens` by `cargo xtask gen-tokens`.
#[rustfmt::skip]
pub mod generated;
mod preamble;

pub use animations::*;
pub use dark_palette::*;
pub use preamble::*;

/// Format a packed `0x00RRGGBB` colour as a lowercase CSS `#rrggbb` string.
///
/// The same formatting as `ui_tokens::color::to_css_hex`, so the emitted CSS
/// is byte-for-byte what it was when the emitters called `ui-tokens` directly.
pub fn to_css_hex(hex: u32) -> String {
    format!("#{:06x}", hex & 0x00FF_FFFF)
}
