//! # Rich message (SMS, MMS and WhatsApp) components
//!
//! Opinionated message bubbles that render what the recipient sees: media
//! headers with a fallback, extended characters, WhatsApp inline formatting,
//! highlighted placeholders, inert preview buttons and the SMS segment
//! indicator (ldui-g8hh). Built on daisyUI's chat bubble:
//! <https://daisyui.com/components/chat/>

mod component;
pub mod inline;
mod sms;
#[cfg(test)]
mod tests;
mod types;

pub use component::*;
pub use sms::*;
pub use types::*;
