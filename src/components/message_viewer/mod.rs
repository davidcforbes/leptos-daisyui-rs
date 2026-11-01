//! # Message viewer components
//!
//! A read-only, opinionated view of a SENT WhatsApp, SMS or MMS message as
//! the recipient saw it: channel chrome in a phone frame, the picture above
//! the text, language / picture / template switches, delivery status and the
//! recipient's replies (ldui-mhur). Built on the rich-message bubble, never a
//! second renderer. [`MessageViewerDialog`] puts it in the house modal, and
//! [`ViewMessageRowAction`] is the EntityTable eye action that opens it.

mod component;
#[cfg(test)]
mod tests;
mod types;

pub use component::*;
pub use types::*;
